//! A vertex's packed tangent frame (4 bytes): a unit quaternion with its
//! largest component dropped. Bits 30..31 name the dropped component
//! (0 x, 1 y, 2 z, 3 w); bits 0..9, 10..19, 20..29 are the other three in
//! order, each `v / 1023 * 2 - 1` scaled by 1/sqrt 2 (no kept component
//! of a unit quaternion is larger). The dropped one is
//! `sqrt(1 - a² - b² - c²)`. The normal is the quaternion's rotated z
//! axis, the tangent its x axis.
//!
//! Measured on the Holger 556's receiver (`att_jup_vm_jp01_ar_golf36_rec`,
//! 9 surfaces): of every reading tried (index bits high or low, signed or
//! unsigned components, scales, slot orders, axes) this one's normals
//! agree best with the triangles' (mean cosine 0.78 to 0.93 per surface
//! against the next best's 0.6).
#![forbid(unsafe_code)]

/// The quaternion (x, y, z, w) of a packed frame.
#[must_use]
pub fn quaternion(w: u32) -> [f32; 4] {
    let s = std::f32::consts::FRAC_1_SQRT_2;
    let c = |sh: u32| (((w >> sh) & 0x3ff) as f32 / 1023.0 * 2.0 - 1.0) * s;
    let kept = [c(0), c(10), c(20)];
    let m = (1.0 - kept.iter().map(|v| v * v).sum::<f32>())
        .max(0.0)
        .sqrt();
    let idx = (w >> 30) as usize;
    let mut q = [0.0f32; 4];
    let mut k = 0;
    for (i, v) in q.iter_mut().enumerate() {
        if i == idx {
            *v = m;
        } else {
            *v = kept[k];
            k += 1;
        }
    }
    let l = q.iter().map(|v| v * v).sum::<f32>().sqrt().max(1e-9);
    q.map(|v| v / l)
}

/// The normal and tangent of a packed frame.
#[must_use]
pub fn normal_tangent(w: u32) -> ([f32; 3], [f32; 3]) {
    let [x, y, z, ww] = quaternion(w);
    let n = [
        2.0 * (x * z + ww * y),
        2.0 * (y * z - ww * x),
        1.0 - 2.0 * (x * x + y * y),
    ];
    let t = [
        1.0 - 2.0 * (y * y + z * z),
        2.0 * (x * y + ww * z),
        2.0 * (x * z - ww * y),
    ];
    (n, t)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn pack(dropped: u32, a: f32, b: f32, c: f32) -> u32 {
        let s = std::f32::consts::SQRT_2;
        let e = |v: f32| (((v * s + 1.0) / 2.0 * 1023.0).round() as u32) & 0x3ff;
        dropped << 30 | e(a) | e(b) << 10 | e(c) << 20
    }

    #[test]
    fn identity_frame() {
        // w dropped (= 1), x = y = z = 0: normal +z, tangent +x.
        let (n, t) = normal_tangent(pack(3, 0.0, 0.0, 0.0));
        assert!((n[2] - 1.0).abs() < 1e-2 && n[0].abs() < 1e-2);
        assert!((t[0] - 1.0).abs() < 1e-2);
    }

    #[test]
    fn half_turn_about_x() {
        // x dropped (= 1): a half turn about x puts the normal at -z.
        let (n, _) = normal_tangent(pack(0, 0.0, 0.0, 0.0));
        assert!((n[2] + 1.0).abs() < 1e-2);
    }
}
