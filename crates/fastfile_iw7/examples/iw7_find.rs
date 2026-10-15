//! `iw7_find <zone.ff>... -- <name>...`: which zones define (not just
//! reference) an XModel or XModelSurfs of each name, with its LODs, and an
//! image (with pixels or streams) or a material (with a technique set).
use fastfile_iw7::walk::walk_zone;

fn main() {
    let args: Vec<String> = std::env::args().skip(1).collect();
    let split = args.iter().position(|a| a == "--").unwrap_or(args.len());
    let (zones, names) = (&args[..split], args.get(split + 1..).unwrap_or(&[]));
    for path in zones {
        let Ok(zone) = fastfile_iw7::Zone::open(std::path::Path::new(path)) else {
            println!("{path}: not opened");
            continue;
        };
        let (cap, stop) = walk_zone(&zone);
        println!("{path}: {} xmodels, {} surfs, stop {:?}", cap.xmodels.len(), cap.model_surfs.len(), stop.map(|s| s.error));
        for x in &cap.xmodels {
            if names.iter().any(|n| x.name.trim_start_matches(',') == n.as_str()) {
                println!(
                    "  xmodel {} lods {:?}",
                    x.name,
                    x.lods.iter().map(|l| (l.numsurfs, l.surfs.map(|s| cap.name_of(s).to_owned()))).collect::<Vec<_>>()
                );
            }
        }
        // Images with pixels (or streams) and materials with a technique
        // set of those names: where a map's references are held.
        let want = |n: &str| names.iter().any(|w| n.trim_start_matches(',').eq_ignore_ascii_case(w.trim_start_matches(',')));
        for im in &cap.images {
            if want(&im.name) && (im.pixels.is_some() || im.streamed) {
                println!("  image {} fmt {} {}x{} streamed {}", im.name, im.format, im.width, im.height, im.streamed);
            }
        }
        for m in &cap.materials {
            if want(&m.name) && m.techset.is_some() {
                println!("  material {} techset {}", m.name, m.techset.map_or("", |t| cap.name_of(t)));
            }
        }
        for m in &cap.model_surfs {
            if names.iter().any(|n| m.name.trim_start_matches(',') == n.as_str()) {
                println!("  surfs {} ({} surfaces, verts {:?})", m.name, m.surfs.len(), m.surfs.iter().map(|s| s.verts.is_some()).collect::<Vec<_>>());
            }
        }
    }
}
