//! bo2mp: what a Black Ops III map zone holds for its lighting:
//! `t7_light <zone.ff>...` prints the GfxWorld's sun volumes (reflection
//! probe arrays, probe volumes, sky boxes, exposure grids), its fog, LUT,
//! exposure and lit fog volumes, the header materials, the images they name
//! (size, format, streamed parts) and the materials whose technique sets
//! read as sky. `T7_LIGHT_HEX=<what>` (sun, fog, lut, exposure, litfog,
//! probes, header) dumps those records as floats.
use fastfile_t7::walk::{Options, walk_with};

fn f32_at(b: &[u8], o: usize) -> f32 {
    f32::from_le_bytes(b[o..o + 4].try_into().unwrap())
}
fn u32_at(b: &[u8], o: usize) -> u32 {
    u32::from_le_bytes(b[o..o + 4].try_into().unwrap())
}

/// A record as rows of four words, each as a float when it reads as a
/// plausible one, else as hex.
fn dump(b: &[u8], at: usize, len: usize) {
    for row in (0..len).step_by(16) {
        let mut line = format!("    +{row:04x}:");
        for k in (row..(row + 16).min(len)).step_by(4) {
            let u = u32_at(b, at + k);
            let f = f32_at(b, at + k);
            if u != 0 && f.is_finite() && f.abs() > 1e-6 && f.abs() < 1e7 {
                line.push_str(&format!(" {f:>12.5}"));
            } else {
                line.push_str(&format!("   {u:#010x}"));
            }
        }
        println!("{line}");
    }
}

fn main() {
    let hex = std::env::var("T7_LIGHT_HEX").unwrap_or_default();
    for path in std::env::args().skip(1) {
        let path = std::path::PathBuf::from(path);
        let z = match fastfile_t7::Zone::open(&path) {
            Ok(z) => z,
            Err(e) => {
                println!("{}: {e}", path.display());
                continue;
            }
        };
        let w = walk_with(&z, Options::default());
        let cap = &w.capture;
        println!("{}: {:?}", path.display(), w.stop);
        let image = |name: &Option<String>| -> String {
            let Some(n) = name else { return "-".into() };
            match cap.image(n.trim_start_matches(',')).or_else(|| cap.image(n)) {
                Some(i) => format!(
                    "{n} {}x{}x{} levels {} fmt {} {}",
                    i.width,
                    i.height,
                    i.depth,
                    i.levels,
                    i.format,
                    if i.xpak.is_empty() {
                        format!("zone {:?}", i.pixels.map(|p| p.1))
                    } else {
                        format!("xpak {:?}", i.xpak.iter().map(|p| (p.width, p.height, p.levels, p.size)).collect::<Vec<_>>())
                    }
                ),
                None => format!("{n} (no record)"),
            }
        };
        for g in &cap.world.gfx_worlds {
            let h = g.header_at;
            println!("  gfxworld {} header {h:#x}", g.name);
            println!("  header materials: {:?}", g.header_materials);
            println!("  sky models: {:?}", g.sky_models);
            for (i, (v, l)) in g.sun_volumes.iter().zip(&g.sun_lighting).enumerate() {
                println!("  sun volume {i} at {v:#x}");
                for (k, (im, c, b)) in l.reflection_probes.iter().enumerate() {
                    println!("    reflection probes {k}: {} configs {c:x?} blends {b:x?}", image(im));
                }
                for (k, names) in l.probe_volumes.iter().enumerate() {
                    println!("    probe volume {k}: {} | {} | {}", image(&names[0]), image(&names[1]), image(&names[2]));
                }
                for (k, im) in l.sky_images.iter().enumerate() {
                    println!("    sky {k}: {}", image(im));
                }
                println!("    exposure grids {:x?}", l.exposure_grids);
                for (what, list) in [("probe", &l.probe_buffers), ("volume", &l.volume_buffers), ("sky", &l.sky_buffers)] {
                    for (k, b) in list.iter().enumerate() {
                        if let Some(b) = b {
                            let raw = &z.image[*b..*b + 64];
                            println!("    {what} buffer {k} at {b:#x}: {:02x?}", raw);
                        }
                    }
                }
                if hex.contains("sun") {
                    dump(&z.image, *v, 1056);
                    println!("    ... 2096..");
                    dump(&z.image, v + 2160, 4536 - 2160);
                }
                if hex.contains("probes") {
                    for (k, (_, c, b)) in l.reflection_probes.iter().enumerate() {
                        if let Some((at, n)) = c {
                            for j in 0..(*n).min(4) {
                                println!("    probes {k} config {j}:");
                                dump(&z.image, at + 328 * j, 328);
                            }
                        }
                        if let Some((at, n)) = b
                            && *n > 0
                        {
                            println!("    probes {k} blend 0:");
                            dump(&z.image, *at, 604);
                        }
                    }
                }
            }
            for (what, sp, size) in [
                ("fog", g.fog_volumes, 624),
                ("lut", g.lut_volumes, 60),
                ("exposure", g.exposure_volumes, 108),
                ("litfog", g.lit_fog_volumes, 360),
            ] {
                println!("  {what} volumes: {sp:x?}");
                if hex.contains(what)
                    && let Some((at, n)) = sp
                {
                    for j in 0..n.min(4) {
                        println!("   {what} {j}:");
                        dump(&z.image, at + size * j, size);
                    }
                }
            }
            if hex.contains("header") {
                println!("  header 0x600..0x1610:");
                dump(&z.image, h + 0x600, 0x1010);
                println!("  header 0x1640..0x1b40:");
                dump(&z.image, h + 0x1640, 0x500);
                println!("  header 0x1bb0..0x1ff0:");
                dump(&z.image, h + 0x1bb0, 0x440);
            }
        }
        for m in &cap.materials {
            let ts = m.techset.as_deref().unwrap_or_default();
            if ts.contains("sky") || m.name.contains("skybox") || m.name.contains("lut") {
                println!("  material {} techset {ts}", m.name);
                for (hsh, i) in &m.textures {
                    println!("    {hsh:#010x} {}", image(&Some(i.clone())));
                }
            }
        }
        for name in &g_sky_models(cap) {
            if let Some(x) = cap.model(name.trim_start_matches(',')) {
                println!("  sky model {name}: {} lods", x.lods.len());
                for l in &x.lods {
                    println!("    mesh {} materials {:?}", l.mesh, l.materials);
                }
            } else {
                println!("  sky model {name}: not in the zone");
            }
        }
    }
}

fn g_sky_models(cap: &fastfile_t7::walk::capture::Capture) -> Vec<String> {
    cap.world.gfx_worlds.iter().flat_map(|g| g.sky_models.clone()).collect()
}
