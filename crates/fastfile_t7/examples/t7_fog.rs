//! bo2mp: the words a Black Ops III map keeps for its fog and its post
//! effects, to read their layout: `t7_fog <zone.ff>` prints the GfxWorld
//! header's inline settings (0x5f0..0x2040, words that are not zero, as
//! floats where they look like one), each fog volume (624), lit fog volume
//! (360), LUT volume (60) and exposure volume (108), and how many records
//! each post-effect list holds; with `<out.png>`, the LUT material's
//! pictures (dimensions, format) and the first one's top level as a PNG.
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
    let paks = fastfile_t7::xpak::XpakSet::open_dir(path.parent().unwrap()).ok().map(|p| p.0);
    for m in g.header_materials.iter().flatten() {
        let Some(mat) = w.capture.material(m.trim_start_matches(',')) else {
            println!("lut material {m}: not in the zone");
            continue;
        };
        println!("lut material {m}: techset {:?}", mat.techset);
        for (i, (hash, name)) in mat.textures.iter().enumerate() {
            let im = w.capture.image(name.trim_start_matches(','));
            println!("  texture {hash:#010x} {name}: {:?}", im.map(|im| (im.width, im.height, im.depth, im.levels, im.format, fastfile_t7::texture::dxgi_name(im.format))));
            let (Some(im), Some(paks), Some(out)) = (im, paks.as_ref(), a.get(2)) else { continue };
            if i > 0 {
                continue;
            }
            match fastfile_t7::texture::image_levels(im, &z.image, paks) {
                Ok(l) => {
                    let depth = usize::from(im.depth.max(1));
                    let (w0, h0) = (l.width as usize, l.height as usize);
                    // A volume's slices one after another: the top level's
                    // bytes are depth times a slice's.
                    let slice = fastfile_t7::texture::level_size(l.format, l.width, l.height).unwrap_or(0);
                    let mut rows: Vec<u8> = Vec::new();
                    for d in 0..depth {
                        let one = fastfile_t7::texture::ImageLevels { data: l.data[d * slice..((d + 1) * slice).min(l.data.len())].to_vec(), levels: 1, ..l.clone() };
                        if one.format == 24 {
                            // R10G10B10A2: ten bits a channel, red lowest.
                            for w in one.data.chunks_exact(4).take(w0 * h0) {
                                let v = u32::from_le_bytes([w[0], w[1], w[2], w[3]]);
                                for k in 0..3 {
                                    rows.push(((((v >> (10 * k)) & 1023) as f32) * 255.0 / 1023.0).round() as u8);
                                }
                                rows.push(255);
                            }
                        } else if let Some((_, _, px)) = fastfile_t7::texture::level_rgba(&one, 0) {
                            rows.extend_from_slice(&px);
                        }
                    }
                    let (pw, ph) = (w0, rows.len() / 4 / w0.max(1));
                    let at = |x: usize, y: usize| rows.get((y * pw + x) * 4..(y * pw + x) * 4 + 4).map(|p| [p[0], p[1], p[2], p[3]]);
                    println!("  decoded {pw}x{ph} ({depth} slices of {w0}x{h0}): corners {:?} {:?} {:?} {:?}", at(0, 0), at(pw - 1, 0), at(0, ph - 1), at(pw - 1, ph - 1));
                    let f = std::fs::File::create(out).expect("png");
                    let mut e = png::Encoder::new(std::io::BufWriter::new(f), pw as u32, ph as u32);
                    e.set_color(png::ColorType::Rgba);
                    e.set_depth(png::BitDepth::Eight);
                    e.write_header().and_then(|mut w| w.write_image_data(&rows[..pw * ph * 4])).expect("png write");
                }
                Err(e) => println!("  pixels: {e}"),
            }
        }
    }
    for (count_at, _size) in [(0x1620usize, 144usize), (0x1630, 144), (0x1b40, 384), (0x1b50, 464), (0x1b60, 464), (0x1b90, 144), (0x1ba0, 144), (0x1ff0, 336), (0x2000, 688), (0x2010, 688), (0x2020, 164), (0x2030, 164)] {
        let n = u32::from_le_bytes(b[h + count_at..h + count_at + 4].try_into().unwrap()) as usize;
        println!("post list {count_at:#x}: {n}");
    }
}
