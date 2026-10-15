//! Lists a map zone's cube pictures (the sky's, the reflection probes'):
//! `jup_sky <zone image (.zone, decompressed)>`.

use fastfile_jup::image;
use fastfile_jup::map::ZoneImage;

fn main() {
    let path = std::env::args().nth(1).expect("usage: jup_sky <file.zone>");
    let zone = ZoneImage::parse(std::fs::read(&path).expect("read")).expect("zone");
    let z = &zone.bytes;
    let imgs = zone.types.iter().filter(|&&t| t == 21).count();
    let Some(run) = image::ImageRun::find(z, imgs) else {
        println!("no image run");
        return;
    };
    println!("{} images", run.at.len());
    let mut shapes: std::collections::BTreeMap<(u16, u16, u8), usize> = Default::default();
    for &at in &run.at {
        let Ok(Some(r)) = image::parse(z, at) else { continue };
        *shapes.entry((r.depth, r.array, r.format)).or_default() += 1;
        let latlong = r.width == 2 * r.height && r.width >= 1024;
        let rare = !matches!(r.format, 0x24 | 0x25 | 0x2a | 0x2f | 0x30);
        if r.array == 6 || r.depth == 6 || latlong || rare {
            println!(
                "{:#x} {:016x} {}x{} depth {} array {} format {:#x} usage {:#x} flags {:#x} levels {} groups {} size {} inline {}",
                at,
                r.hash,
                r.width,
                r.height,
                r.depth,
                r.array,
                r.format,
                r.usage,
                r.flags,
                r.levels,
                r.groups.len(),
                r.data_size,
                r.inline.is_some()
            );
        }
    }
    println!("(depth, array, format) counts: {shapes:?}");
}
