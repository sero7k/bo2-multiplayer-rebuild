//! bo2mp: what the Black Ops III lighting path bakes at load, timed:
//! `t7_bake <zone.ff> <out.png> [cx cy span]` gathers the light blockers,
//! renders the sun map over the square (default: round the spawn points),
//! writes it as a picture (sun-lit ground bright), and times the per-vertex
//! occlusion over the world's vertices.
use fastfile_t7::light::{Bake, Blocks, SunMap, casters, sun_toward};
use fastfile_t7::map::{MapSources, T7Map};

fn main() {
    let a: Vec<String> = std::env::args().collect();
    let t0 = std::time::Instant::now();
    let map = T7Map::load(&MapSources::beside(std::path::Path::new(&a[1]))).expect("map");
    println!("map read in {:.1}s", t0.elapsed().as_secs_f32());
    let bake = Bake::read(&map).expect("light");
    for l in &bake.report {
        println!("{l}");
    }
    println!("units: {:.1}, sky gain {:.5}", bake.units(), bake.sky_gain());
    let blocks = |m: usize| {
        let mat = &map.materials[m];
        let w: Vec<&str> = mat.name.split(['_', '/', '#']).chain(mat.technique_set.split(['_', '/', '#'])).collect();
        let has = |x: &str| w.iter().any(|v| v.starts_with(x));
        if has("sky") || has("tool") || has("caulk") || has("clip") || has("trigger") || has("decal") || has("water") || has("glass") || has("transparent") {
            Blocks::No
        } else if has("vegetation") || has("foliage") || has("leaf") || has("leaves") || has("grass") || has("alphatest") || has("chainlink") {
            Blocks::Partly
        } else {
            Blocks::Fully
        }
    };
    // The spawn points' square.
    let mut lo = [f32::MAX; 2];
    let mut hi = [f32::MIN; 2];
    for line in map.entities.split('}') {
        if !(line.contains("\"classname\" \"mp_") && line.contains("spawn")) {
            continue;
        }
        if let Some(o) = line.split("\"origin\" \"").nth(1).and_then(|s| s.split('"').next()) {
            let v: Vec<f32> = o.split_whitespace().filter_map(|x| x.parse().ok()).collect();
            if v.len() == 3 {
                lo = [lo[0].min(v[0]), lo[1].min(v[1])];
                hi = [hi[0].max(v[0]), hi[1].max(v[1])];
            }
        }
    }
    let f = |i: usize, d: f32| a.get(i).and_then(|v| v.parse().ok()).unwrap_or(d);
    let (cx, cy) = (f(3, (lo[0] + hi[0]) * 0.5), f(4, (lo[1] + hi[1]) * 0.5));
    let span = f(5, ((hi[0] - lo[0]).max(hi[1] - lo[1]) + 3000.0).clamp(2048.0, 12000.0));
    println!("spawns over {lo:.0?}..{hi:.0?}: square at {cx:.0} {cy:.0}, {span:.0} units");
    let t = std::time::Instant::now();
    let margin = 3000.0;
    let area = [[cx - span * 0.5 - margin, cy - span * 0.5 - margin], [cx + span * 0.5 + margin, cy + span * 0.5 + margin]];
    let c = casters(&map, &blocks, 8.0, area);
    println!("casters: {} triangles ({} partial) in {:.1}s", c.len(), c.iter().filter(|c| c.partial).count(), t.elapsed().as_secs_f32());
    let sun = map.sun.as_ref().map_or([0.3, 0.2, 0.9], sun_toward);
    println!("sun toward {sun:.3?}");
    let t = std::time::Instant::now();
    let z = (map.mins[2] + map.maxs[2]) * 0.5;
    let sm = SunMap::render(&c, sun, [cx, cy, z.clamp(-2000.0, 2000.0)], span * 1.15, 4096);
    println!("sun map {}px, {:.2} units a texel, heights {:.0}+{:.0}, in {:.1}s", sm.depth.size, sm.depth.texel, sm.base, sm.range, t.elapsed().as_secs_f32());
    let t = std::time::Instant::now();
    let corners: Vec<[[f32; 3]; 3]> = c.iter().filter(|c| !c.partial).map(|c| c.p).collect();
    let tracer = fastfile_t7::bake::Tracer::new(&corners);
    println!("tracer over {} triangles in {:.1}s", tracer.len(), t.elapsed().as_secs_f32());
    let t = std::time::Instant::now();
    let n = map.vertices.len();
    let open = fastfile_t7::bake::parallel_map(n, |i| {
        let v = &map.vertices[i];
        tracer.openness(v.xyz, v.normal, 160.0, 8, i as u32, 1.0)
    });
    let lit = fastfile_t7::bake::parallel_map(n, |i| sm.depth.visibility(map.vertices[i].xyz, 4.0));
    println!(
        "{n} vertices: openness mean {:.2}, sun-lit mean {:.2}, in {:.1}s",
        open.iter().sum::<f32>() / n.max(1) as f32,
        lit.iter().sum::<f32>() / n.max(1) as f32,
        t.elapsed().as_secs_f32()
    );
    // The picture: per texel, whether the top surface there sees the sun
    // (always) shaded by its height; nothing: black.
    let s = sm.depth.size as usize;
    let step = s.div_ceil(1024);
    let w = s / step;
    let (hlo, hhi) = sm.depth.height_range();
    let mut img = vec![0u8; w * w];
    for y in 0..w {
        for x in 0..w {
            let h = sm.depth.height[(y * step) * s + x * step];
            if h != f32::MIN {
                img[y * w + x] = (32.0 + 223.0 * (h - hlo) / (hhi - hlo).max(1.0)).round() as u8;
            }
        }
    }
    let file = std::fs::File::create(&a[2]).expect("png");
    let mut e = png::Encoder::new(std::io::BufWriter::new(file), w as u32, w as u32);
    e.set_color(png::ColorType::Grayscale);
    e.set_depth(png::BitDepth::Eight);
    e.write_header().expect("header").write_image_data(&img).expect("png data");
    println!("total {:.1}s", t0.elapsed().as_secs_f32());
}
