//! bo2mp offline probe: a CoD4 (`iw3`) or World at War (`t4`) map zone's
//! lighting inputs: the sun (`sunParse`), the lightmap pages' value ranges,
//! the light grid's colours, the world materials' normal and specular maps
//! (their IWI channel layout) and the lit world pixel shaders (token dump).
//!
//! `cargo run -p asset_world --example legacy_light_probe -- <zone.ff>...`
//! (`PROBE_SHADERS=n` dumps the lit pixel shaders of n technique sets.)

use std::collections::BTreeMap;

fn main() {
    for arg in std::env::args().skip(1) {
        let path = std::path::Path::new(&arg);
        let bytes = std::fs::read(path).unwrap();
        let version = u32::from_le_bytes(bytes[8..12].try_into().unwrap());
        drop(bytes);
        let zone = if version == 0x183 {
            t4::load_map(path).unwrap().zone
        } else {
            let data = iw3::fastfile::load(path).unwrap();
            iw3::zone::Zone::parse(&data, iw3::zone::ParseOptions::default()).unwrap()
        };
        println!("== {} (version {version:#x})", path.display());
        let Some(world) = zone.gfx_world() else {
            println!("no GfxWorld");
            continue;
        };
        let s = &world.sun;
        println!(
            "sun: name={:?} sunLight={} sunColor={:?} ambientScale={} ambientColor={:?} diffuseFraction={} diffuseColor={:?} angles={:?} colorFromBsp={:?}",
            s.name, s.sun_light, s.sun_color, s.ambient_scale, s.ambient_color, s.diffuse_fraction, s.diffuse_color, s.angles, world.sun_color_from_bsp
        );
        lightmaps(&zone, world);
        grid(world);
        let main = asset_transport::game_root_for_zone(path).ok().map(|r| r.join("main"));
        materials(&zone, world, main.as_deref());
        if let Ok(n) = std::env::var("PROBE_SHADERS") {
            shaders(&zone, world, n.parse().unwrap_or(2));
        }
    }
}

fn lightmaps(zone: &iw3::zone::Zone, world: &iw3::zone::GfxWorld) {
    for (i, pair) in world.lightmaps.iter().enumerate() {
        let def = |id: Option<iw3::zone::AssetId>| id.and_then(|id| zone.image(id)).and_then(|i| i.load_def.as_ref().map(|d| (i.name.clone(), d)));
        let (Some((pn, p)), Some((sn, sdef))) = (def(pair.primary), def(pair.secondary)) else {
            println!("lightmap {i}: missing");
            continue;
        };
        println!(
            "lightmap {i}: primary {pn} {:?} fmt {} ({} bytes); secondary {sn} {:?} fmt {} ({} bytes)",
            p.dimensions, p.format, p.data.len(), sdef.dimensions, sdef.format, sdef.data.len()
        );
        // Secondary: BGRA, two stacked pages.
        let (w, h) = (usize::from(sdef.dimensions[0]), usize::from(sdef.dimensions[1]));
        let page = w * (h / 2) * 4;
        for (k, name) in ["A", "B"].iter().enumerate() {
            let Some(px) = sdef.data.get(k * page..(k + 1) * page) else { continue };
            let mut hist = [[0usize; 8]; 4];
            let mut sum = [0f64; 4];
            for p in px.chunks_exact(4) {
                // B G R A -> R G B A
                let rgba = [p[2], p[1], p[0], p[3]];
                for c in 0..4 {
                    hist[c][usize::from(rgba[c] / 32)] += 1;
                    sum[c] += f64::from(rgba[c]);
                }
            }
            let n = (px.len() / 4) as f64;
            println!(
                "  page {name}: mean rgba {:?}; R hist {:?}; A hist {:?}",
                sum.map(|v| (v / n).round()),
                hist[0],
                hist[3]
            );
        }
        let mut hist = [0usize; 8];
        for &v in p.data.iter().take(usize::from(p.dimensions[0]) * usize::from(p.dimensions[1])) {
            hist[usize::from(v / 32)] += 1;
        }
        println!("  primary hist {hist:?}");
    }
}

fn grid(world: &iw3::zone::GfxWorld) {
    let g = &world.light_grid;
    let points = g.points();
    println!(
        "light grid: mins {:?} maxs {:?} row_axis {} col_axis {} rows {} raw {} entries {} colours {} points {}",
        g.mins,
        g.maxs,
        g.row_axis,
        g.col_axis,
        g.row_data_start.len(),
        g.raw_row_data.len(),
        g.entries.len(),
        g.colors.len(),
        points.len()
    );
    let mut means = Vec::new();
    let mut primaries: BTreeMap<u8, usize> = BTreeMap::new();
    let mut traces = 0usize;
    for e in &g.entries {
        *primaries.entry(e.primary_light_index).or_default() += 1;
        traces += usize::from(e.needs_trace != 0);
    }
    for c in &g.colors {
        let mut m = 0f32;
        for d in &c.0 {
            m += (f32::from(d[0]) + f32::from(d[1]) + f32::from(d[2])) / 3.0;
        }
        means.push(m / 56.0);
    }
    means.sort_by(f32::total_cmp);
    if !means.is_empty() {
        let q = |f: f32| means[((means.len() - 1) as f32 * f) as usize];
        println!(
            "  colour set mean (0..255): min {:.1} p10 {:.1} p50 {:.1} p90 {:.1} max {:.1}; primaries {primaries:?}; needs trace {traces}",
            q(0.0),
            q(0.1),
            q(0.5),
            q(0.9),
            q(1.0)
        );
        if let Some(c) = g.colors.get(g.colors.len() / 2) {
            println!("  a colour set: {:?}", &c.0[..8]);
        }
    }
    if let Some((p, _)) = points.first() {
        println!("  first point {:?} at {:?}", p, iw3::zone::LightGrid::world_pos(*p));
    }
}

fn materials(zone: &iw3::zone::Zone, world: &iw3::zone::GfxWorld, main: Option<&std::path::Path>) {
    let mut seen = std::collections::HashSet::new();
    let mut techs: BTreeMap<String, usize> = BTreeMap::new();
    let mut sems: BTreeMap<String, usize> = BTreeMap::new();
    let mut samples = Vec::new();
    for s in &world.surfaces {
        let Some(mid) = s.material else { continue };
        if !seen.insert(mid) {
            continue;
        }
        let Some(m) = zone.material(mid) else { continue };
        let tech = m.technique_set.and_then(|t| zone.technique_set(t)).map_or("?".into(), |t| t.name.clone());
        *techs.entry(tech).or_default() += 1;
        for t in &m.textures {
            *sems.entry(format!("{:?}", t.semantic)).or_default() += 1;
            if matches!(t.semantic, iw3::zone::TextureSemantic::Normal | iw3::zone::TextureSemantic::Specular)
                && samples.len() < 10
                && let Some(img) = t.image.and_then(|i| zone.image(i))
            {
                samples.push((format!("{:?}", t.semantic), img.name.clone()));
            }
        }
    }
    println!("world materials {}; technique sets (top): ", seen.len());
    let mut v: Vec<_> = techs.into_iter().collect();
    v.sort_by(|a, b| b.1.cmp(&a.1));
    for (t, n) in v.iter().take(12) {
        println!("  {n:4} {t}");
    }
    println!("  texture semantics: {sems:?}");
    let Some(main) = main else { return };
    let Ok(index) = asset_transport::IwdIndex::open(main) else { return };
    for (sem, name) in samples {
        let bare = name.trim_start_matches(['*', ',', '$']);
        let Some(file) = index.image_candidates(bare).and_then(|c| c.first()) else {
            println!("  {sem} {name}: not in the IWDs");
            continue;
        };
        let Ok(bytes) = file.read() else { continue };
        let format = bytes.get(4).copied().unwrap_or(0);
        match asset_material::decode_iwi_rgba(&bytes) {
            Ok((w, h, px)) => {
                let mut mean = [0f64; 4];
                let mut min = [255u8; 4];
                let mut max = [0u8; 4];
                for p in px.chunks_exact(4) {
                    for c in 0..4 {
                        mean[c] += f64::from(p[c]);
                        min[c] = min[c].min(p[c]);
                        max[c] = max[c].max(p[c]);
                    }
                }
                let n = (px.len() / 4) as f64;
                println!(
                    "  {sem} {name}: IWI v{} format {format} {w}x{h}; mean rgba {:?} min {min:?} max {max:?}",
                    bytes[3],
                    mean.map(|v| (v / n).round())
                );
            }
            Err(e) => println!("  {sem} {name}: IWI format {format}: {e}"),
        }
    }
}

fn reg(r: d3d9_sm3::Sm3Register) -> String {
    use d3d9_sm3::Sm3RegisterFile as F;
    let p = match r.file {
        F::Temporary => "r",
        F::Input => "v",
        F::FloatConstant => "c",
        F::Texture => "t",
        F::ColourOutput => "oC",
        F::Sampler => "s",
        F::Misc => "vMisc",
        F::IntegerConstant => "i",
        F::BooleanConstant => "b",
        _ => "?",
    };
    format!("{p}{}", r.index)
}

fn swz(s: [u8; 4]) -> String {
    if s == [0, 1, 2, 3] {
        return String::new();
    }
    format!(".{}", s.iter().map(|&c| ['x', 'y', 'z', 'w'][usize::from(c)]).collect::<String>())
}

fn shaders(zone: &iw3::zone::Zone, world: &iw3::zone::GfxWorld, n: usize) {
    let mut done = std::collections::HashSet::new();
    for s in &world.surfaces {
        if done.len() >= n {
            break;
        }
        let Some(m) = s.material.and_then(|m| zone.material(m)) else { continue };
        let Some(ts) = m.technique_set.and_then(|t| zone.technique_set(t)) else { continue };
        if !done.insert(ts.name.clone()) {
            continue;
        }
        for t in [7usize, 8, 9] {
            let Some(Some(tech)) = ts.techniques.get(t) else { continue };
            for pass in &tech.passes {
                let Some(ps) = &pass.pixel_shader else { continue };
                println!("-- {} technique {t} ({}) pixel shader {} args {:?}", ts.name, tech.name, ps.name, pass.args);
                let ir = match d3d9_sm3::decode_sm3_program(&ps.program, d3d9_sm3::ShaderStage::Pixel) {
                    Ok(ir) => ir,
                    Err(e) => {
                        println!("   undecodable: {e:?}");
                        continue;
                    }
                };
                for ins in &ir.instructions {
                    match &ins.body {
                        d3d9_sm3::Sm3InstructionBody::Operation { destination, sources } => {
                            let d = destination.map(|d| {
                                let mask: String =
                                    (0..4).filter(|c| d.write_mask & (1 << c) != 0).map(|c| ['x', 'y', 'z', 'w'][c]).collect();
                                format!("{}{}{}", if d.saturate { "_sat " } else { " " }, reg(d.register), if mask == "xyzw" { String::new() } else { format!(".{mask}") })
                            });
                            let srcs: Vec<String> =
                                sources.iter().map(|s| format!("{:?}({}{})", s.modifier, reg(s.register), swz(s.swizzle))).collect();
                            println!("   {:?}{} {}", ins.opcode, d.unwrap_or_default(), srcs.join(", "));
                        }
                        d3d9_sm3::Sm3InstructionBody::FloatDefinition { destination, values } => {
                            println!("   def {} {:?}", reg(destination.register), values.map(f32::from_bits));
                        }
                        other => println!("   {:?} {other:?}", ins.opcode),
                    }
                }
            }
        }
    }
}
