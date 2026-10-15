//! bo2mp: Modern Warfare III's sky as the world's sky cube.
//!
//! A map zone keeps its skies as half-float pictures (BC6H, format 0x2d,
//! the sky's use 0x3a), twice as wide as high: two mirrored squares, each
//! the upper hemisphere seen from above, the horizon on the square's edge.
//! A square point `(a, b)` in -1..1 is a direction by the concentric
//! mapping (azimuth from the square's rings, `r = max(|a|, |b|)`) and a
//! height `z = 1 - r^k`; `k` is fitted so the picture's sun stands as high
//! as worldspawn's (Rebirth Island: 37.8 degrees, `k` about 2.6), and the
//! picture is turned about the vertical to put its sun where worldspawn's
//! is. The picture with the brightest sun is the sky; its light is
//! exposed so the sky away from the sun averages a pale daylight, and
//! written as the RGBA8 sRGB cube the builder draws (faces +x -x +y -y +z
//! -z, z up); below the horizon, the horizon's colour darkens to the sea's
//! haze.

use fastfile_t7::texture::ImageLevels;
use fastfile_jup::map::materials::{MapImage, Packages};
use fastfile_jup::oodle::Decompressor;

/// A decoded half of a sky picture: width = height = `n`, linear RGB.
struct Dome {
    n: usize,
    rgb: Vec<[f32; 3]>,
}

fn f16(b: u16) -> f32 {
    let e = i32::from((b >> 10) & 31);
    let m = f32::from(b & 1023);
    let v = match e {
        0 => m / 1024.0 * 2f32.powi(-14),
        31 => 65504.0,
        _ => (1.0 + m / 1024.0) * 2f32.powi(e - 15),
    };
    if b & 0x8000 != 0 { -v } else { v }
}

/// The picture's first (smaller) group decoded, its left square.
fn dome(img: &MapImage, paks: &Packages, oodle: &dyn Decompressor) -> Option<Dome> {
    let group = img.groups.first()?;
    let data = paks.fetch(group.key, oodle)?;
    // BC6H is a byte a pixel; the picture is twice as wide as high.
    let h = ((data.len() / 2) as f64).sqrt() as usize;
    let w = 2 * h;
    if h < 64 || w * h > data.len() || h % 4 != 0 {
        return None;
    }
    let bw = w / 4;
    let mut rgb = vec![[0.0f32; 3]; h * h];
    let mut tmp = [0u16; 48];
    for by in 0..h / 4 {
        for bx in 0..h / 4 {
            let o = (by * bw + bx) * 16;
            let src = data.get(o..o + 16)?;
            bcdec_rs::bc6h_half(src, &mut tmp, 12, false);
            for y in 0..4 {
                for x in 0..4 {
                    let p = &mut rgb[(by * 4 + y) * h + bx * 4 + x];
                    for c in 0..3 {
                        p[c] = f16(tmp[y * 12 + x * 3 + c]).max(0.0);
                    }
                }
            }
        }
    }
    Some(Dome { n: h, rgb })
}

fn lum(c: [f32; 3]) -> f32 {
    0.2126 * c[0] + 0.7152 * c[1] + 0.0722 * c[2]
}

/// The square point's ring radius and azimuth (concentric mapping).
fn ring(a: f32, b: f32) -> (f32, f32) {
    use std::f32::consts::FRAC_PI_4;
    if a == 0.0 && b == 0.0 {
        return (0.0, 0.0);
    }
    if a.abs() > b.abs() {
        let phi = FRAC_PI_4 * (b / a);
        if a > 0.0 { (a, phi) } else { (-a, phi + std::f32::consts::PI) }
    } else {
        let phi = 2.0 * FRAC_PI_4 - FRAC_PI_4 * (a / b);
        if b > 0.0 { (b, phi) } else { (-b, phi + std::f32::consts::PI) }
    }
}

/// The square point (in -1..1) of a ring radius and azimuth.
fn unring(r: f32, phi: f32) -> (f32, f32) {
    use std::f32::consts::{FRAC_PI_4, PI};
    let mut phi = phi.rem_euclid(2.0 * PI);
    // Back into the concentric mapping's four sectors.
    if phi < FRAC_PI_4 || phi >= 7.0 * FRAC_PI_4 {
        if phi >= 7.0 * FRAC_PI_4 {
            phi -= 2.0 * PI;
        }
        (r, r * phi / FRAC_PI_4)
    } else if phi < 3.0 * FRAC_PI_4 {
        (r * (2.0 * FRAC_PI_4 - phi) / FRAC_PI_4, r)
    } else if phi < 5.0 * FRAC_PI_4 {
        (-r, -r * (phi - PI) / FRAC_PI_4)
    } else {
        (-r * (2.0 * FRAC_PI_4 - (phi - PI)) / FRAC_PI_4, -r)
    }
}

/// The sky cube of the map's sky pictures, its sun turned toward
/// `toward_sun` (unit, z up). `None` when no picture can be read.
pub(crate) fn sky_cube(
    images: &[MapImage],
    paks: &Packages,
    oodle: &dyn Decompressor,
    toward_sun: [f32; 3],
    size: u32,
    report: &mut Vec<String>,
) -> Option<ImageLevels> {
    let domes: Vec<Dome> = images.iter().filter_map(|i| dome(i, paks, oodle)).collect();
    // The sky with the brightest sun.
    let (d, peak_at) = domes
        .iter()
        .map(|d| {
            let (i, _) = d
                .rgb
                .iter()
                .enumerate()
                .max_by(|a, b| lum(*a.1).total_cmp(&lum(*b.1)))
                .unwrap_or((0, &[0.0; 3]));
            (d, i)
        })
        .max_by(|a, b| lum(a.0.rgb[a.1]).total_cmp(&lum(b.0.rgb[b.1])))?;
    let n = d.n;
    let at = |a: f32, b: f32| -> [f32; 3] {
        // Bilinear, (a, b) in -1..1.
        let fx = ((a + 1.0) * 0.5 * n as f32 - 0.5).clamp(0.0, n as f32 - 1.0);
        let fy = ((b + 1.0) * 0.5 * n as f32 - 0.5).clamp(0.0, n as f32 - 1.0);
        let (x0, y0) = (fx.floor() as usize, fy.floor() as usize);
        let (x1, y1) = ((x0 + 1).min(n - 1), (y0 + 1).min(n - 1));
        let (tx, ty) = (fx - x0 as f32, fy - y0 as f32);
        let p = |x: usize, y: usize| d.rgb[y * n + x];
        std::array::from_fn(|c| {
            let top = p(x0, y0)[c] * (1.0 - tx) + p(x1, y0)[c] * tx;
            let bot = p(x0, y1)[c] * (1.0 - tx) + p(x1, y1)[c] * tx;
            top * (1.0 - ty) + bot * ty
        })
    };
    // The picture's sun: its ring radius and azimuth; the exponent that
    // puts it at worldspawn's height, the turn that puts it at its yaw.
    let (sa, sb) = (
        ((peak_at % n) as f32 + 0.5) / n as f32 * 2.0 - 1.0,
        ((peak_at / n) as f32 + 0.5) / n as f32 * 2.0 - 1.0,
    );
    let (sr, sphi) = ring(sa, sb);
    let want_z = toward_sun[2].clamp(0.05, 0.95);
    let k = if sr > 0.05 && sr < 0.98 { ((1.0 - want_z).ln() / sr.ln()).clamp(1.2, 4.0) } else { 2.0 };
    let turn = toward_sun[1].atan2(toward_sun[0]) - sphi;
    // Exposure: the sky away from the sun (all but the brightest
    // hundredth) averages a pale daylight.
    let mut ls: Vec<f32> = d.rgb.iter().map(|c| lum(*c)).collect();
    ls.sort_by(f32::total_cmp);
    let body = &ls[..ls.len() * 99 / 100];
    let mean = body.iter().sum::<f32>() / body.len().max(1) as f32;
    let expose = if mean > 1e-6 { 0.9 / mean } else { 1.0 };
    let tone = |c: [f32; 3]| -> [u8; 3] {
        c.map(|v| {
            let t = 1.0 - (-v * expose).exp();
            let s = if t <= 0.003_130_8 { t * 12.92 } else { 1.055 * t.powf(1.0 / 2.4) - 0.055 };
            (s.clamp(0.0, 1.0) * 255.0).round() as u8
        })
    };
    let haze = [0.42f32, 0.52, 0.58];
    let size_u = size as usize;
    let mut data = Vec::with_capacity(6 * size_u * size_u * 4);
    for face in 0..6 {
        for y in 0..size_u {
            for x in 0..size_u {
                let s = (x as f32 + 0.5) / size as f32 * 2.0 - 1.0;
                let t = (y as f32 + 0.5) / size as f32 * 2.0 - 1.0;
                let v = match face {
                    0 => [1.0, -t, -s],
                    1 => [-1.0, -t, s],
                    2 => [s, 1.0, t],
                    3 => [s, -1.0, -t],
                    4 => [s, -t, 1.0],
                    _ => [-s, -t, -1.0],
                };
                let l = (v[0] * v[0] + v[1] * v[1] + v[2] * v[2]).sqrt();
                let v = v.map(|c| c / l);
                let phi = v[1].atan2(v[0]) - turn;
                let up = v[2].max(0.0);
                let r = (1.0 - up).max(0.0).powf(1.0 / k).min(0.999);
                let (a, b) = unring(r, phi);
                let c = tone(at(a, b));
                let c = if v[2] >= 0.0 {
                    c
                } else {
                    // Under the horizon: its colour toward the sea's haze.
                    let w = (-v[2]).powf(0.35);
                    let hz = haze.map(|h| h * 255.0);
                    [0, 1, 2].map(|i| (f32::from(c[i]) + (hz[i] - f32::from(c[i])) * w).round() as u8)
                };
                data.extend_from_slice(&c);
                data.push(255);
            }
        }
    }
    report.push(format!(
        "jup sky: {} of {} pictures read; the brightest sun's at ring {sr:.2} (height exponent {k:.2}), turned {:.0} degrees; exposure {expose:.3}",
        domes.len(),
        images.len(),
        turn.to_degrees()
    ));
    Some(ImageLevels {
        format: 29,
        width: size,
        height: size,
        levels: 1,
        data,
        part: None,
    })
}

#[cfg(test)]
mod tests {
    #[test]
    fn rings_round_trip() {
        for &(a, b) in &[(0.5f32, 0.2f32), (-0.3, 0.7), (-0.6, -0.1), (0.2, -0.9), (0.9, 0.9)] {
            let (r, phi) = super::ring(a, b);
            let (a2, b2) = super::unring(r, phi);
            assert!((a - a2).abs() < 1e-4 && (b - b2).abs() < 1e-4, "{a} {b} -> {a2} {b2}");
        }
    }
}
