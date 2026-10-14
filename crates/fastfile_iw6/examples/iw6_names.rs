//! Lists a Ghosts zone's materials and images whose names hold a word:
//! `iw6_names <zone.ff> <word>`.
fn main() {
    let mut a = std::env::args().skip(1);
    let path = a.next().expect("zone");
    let word = a.next().unwrap_or_default().to_ascii_lowercase();
    let zone = fastfile_iw6::Zone::open(std::path::Path::new(&path)).expect("open");
    let (cap, stop) = fastfile_iw6::walk::walk_zone(&zone);
    println!("walked {}/{} {:?}", cap.walked, zone.assets.assets.len(), stop.map(|s| s.error));
    for m in cap.materials.iter().filter(|m| m.name.to_ascii_lowercase().contains(&word)) {
        let images: Vec<String> = m.textures.iter().filter_map(|t| t.image).map(|i| cap.name_of(i).to_owned()).collect();
        println!("material {} -> {images:?}", m.name);
    }
    for i in cap.images.iter().filter(|i| i.name.to_ascii_lowercase().contains(&word)) {
        println!("image {} {}x{}x{} fmt {} map_type {} levels {} streamed {} pixels {:?} streams {:?}", i.name, i.width, i.height, i.depth, i.format, i.map_type, i.level_count, i.streamed, i.pixels.map(|p| p.len), i.streams);
    }
}
