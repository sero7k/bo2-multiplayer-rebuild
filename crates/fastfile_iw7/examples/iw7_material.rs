//! A material's definition: `iw7_material <zone.ff>... -- <material name>...`
//! (names without the comma; a substring matches too). Materials are
//! defined in the `techsets_<zone>.ff` zones; the other zones hold only
//! comma-named references to them. Each texture's image is resolved to the
//! zone (of those given) that defines it, with its stream ranges.
use fastfile_iw7::walk::{AssetId, Capture, walk_zone};
use std::collections::HashMap;

fn main() {
    let args: Vec<String> = std::env::args().skip(1).collect();
    let split = args.iter().position(|a| a == "--").unwrap_or(args.len());
    let (zones, names) = (&args[..split], args.get(split + 1..).unwrap_or(&[]));
    let mut caps: Vec<(String, Capture)> = Vec::new();
    for path in zones {
        let zone = fastfile_iw7::Zone::open(std::path::Path::new(path)).expect("zone");
        let (cap, stop) = walk_zone(&zone);
        let real = cap.materials.iter().filter(|m| !m.name.starts_with(',')).count();
        eprintln!("{path}: walked {} of {}{}, {real} materials defined", cap.walked, zone.assets.assets.len(), stop.map_or(String::new(), |s| format!(" (stopped: {})", s.error)));
        let label = std::path::Path::new(path).file_stem().unwrap().to_string_lossy().trim_start_matches("iw7_").to_owned();
        caps.push((label, cap));
    }
    // Real (comma-less) images by name.
    let mut images: HashMap<&str, (usize, usize)> = HashMap::new();
    for (z, (_, c)) in caps.iter().enumerate() {
        for (i, im) in c.images.iter().enumerate() {
            if !im.name.starts_with(',') {
                images.entry(im.name.as_str()).or_insert((z, i));
            }
        }
    }
    for (z, (label, c)) in caps.iter().enumerate() {
        for m in c.materials.iter().filter(|m| !m.name.starts_with(',')) {
            if !names.iter().any(|n| m.name == *n || m.name.contains(n.as_str())) {
                continue;
            }
            let ts = m.techset.map_or("<none>", |t| c.name_of(t));
            println!("material {} [{label}] techset {ts} type {} sort {} camera {} state_flags {:#x}", m.name, m.material_type, m.sort_key, m.camera_region, m.state_flags);
            for t in &m.textures {
                let iname = t.image.map_or("<none>", |id: AssetId| c.name_of(id).trim_start_matches(','));
                let def = images.get(iname).map(|&(iz, ii)| {
                    let im = &caps[iz].1.images[ii];
                    let top = im.streams.iter().rposition(|s| s.0 != 0).map(|k| im.streams[k]);
                    format!("{} {}x{} fmt {} [{}]", if im.streamed { "streamed" } else { "inline" }, top.map_or(im.width, |s| s.0), top.map_or(im.height, |s| s.1), im.format, caps[iz].0)
                });
                let slot = fastfile_iw7::texture::sampler_name(t.name_hash).map_or_else(|| format!("{:08x}", t.name_hash), str::to_owned);
                println!(
                    "  texture {slot:12} semantic {:2} sampler {:#04x} -> {iname} ({})",
                    t.semantic,
                    t.sampler_state,
                    def.unwrap_or_else(|| "not defined in the given zones".into())
                );
            }
            for k in &m.constants {
                println!("  constant {:12} {:08x} {:?}", k.name, k.name_hash, k.literal);
            }
            let _ = z;
        }
    }
}
