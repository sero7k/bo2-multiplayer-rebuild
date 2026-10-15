//! bo2mp: the words a Black Ops III map keeps for its fog and its post
//! effects, to read their layout: `t7_fog <zone.ff>` prints the GfxWorld
//! header's inline settings (0x5f0..0x2040, words that are not zero, as
//! floats where they look like one), each fog volume (624), lit fog volume
//! (360), LUT volume (60) and exposure volume (108), and how many records
//! each post-effect list holds.
use fastfile_t7::walk::{Options, walk_with};

fn word(b: &[u8], o: usize) -> String {
    let u = u32::from_le_bytes(b[o..o + 4].try_into().unwrap());
    let f = f32::from_bits(u);
    let e = (u >> 23) & 0xff;
    if u != 0 && (100..160).contains(&e) && f.is_finite() { format!("{f:.5}") } else { format!("{u:#x}") }
}

fn dump(b: &[u8], at: usize, len: usize, label: &str) {
    println!("{label} at {at:#x} ({len} bytes)");
    let mut o = 0;
    while o + 4 <= len {
        let row: Vec<String> = (0..4).filter(|k| o + 4 * k + 4 <= len).map(|k| word(b, at + o + 4 * k)).collect();
        if row.iter().any(|w| w != "0x0") {
            println!("  +{o:04x}: {}", row.join("  "));
        }
        o += 16;
    }
}

fn main() {
    let a: Vec<String> = std::env::args().collect();
    let path = std::path::Path::new(&a[1]);
    let z = fastfile_t7::Zone::open(path).expect("zone");
    let w = walk_with(&z, Options::default());
    let g = w.capture.world.gfx_worlds.first().expect("gfxworld");
    let b = &z.image;
    let h = g.header_at;
    println!("{} header {h:#x}, materials {:?}", g.name, g.header_materials);
    dump(b, h + 0x5f0, 0x2040 - 0x5f0, "header 0x5f0..");
    let lists: [(&str, fastfile_t7::walk::world::Span, usize); 4] =
        [("fog volume", g.fog_volumes, 624), ("lit fog volume", g.lit_fog_volumes, 360), ("lut volume", g.lut_volumes, 60), ("exposure volume", g.exposure_volumes, 108)];
    for (name, span, size) in lists {
        let Some((at, n)) = span else {
            println!("{name}s: none");
            continue;
        };
        println!("{name}s: {n}");
        for i in 0..n.min(6) {
            dump(b, at + size * i, size, &format!("{name} {i}"));
        }
    }
    for (count_at, _size) in [(0x1620usize, 144usize), (0x1630, 144), (0x1b40, 384), (0x1b50, 464), (0x1b60, 464), (0x1b90, 144), (0x1ba0, 144), (0x1ff0, 336), (0x2000, 688), (0x2010, 688), (0x2020, 164), (0x2030, 164)] {
        let n = u32::from_le_bytes(b[h + count_at..h + count_at + 4].try_into().unwrap()) as usize;
        println!("post list {count_at:#x}: {n}");
    }
}
