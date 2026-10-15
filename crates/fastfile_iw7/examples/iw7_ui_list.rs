//! `iw7_ui_list <zone.ff> [word]...`: the pictures a zone holds (with
//! pixels or streams) whose names hold any of the words, with sizes.
fn main() {
    let a: Vec<String> = std::env::args().collect();
    let zone = fastfile_iw7::Zone::open(std::path::Path::new(&a[1])).expect("zone");
    let (cap, stop) = fastfile_iw7::walk::walk_zone(&zone);
    println!("{} images, stop {:?}", cap.images.len(), stop.map(|s| s.error));
    for im in &cap.images {
        if im.name.starts_with(',') || (im.pixels.is_none() && !im.streamed) {
            continue;
        }
        let n = im.name.to_ascii_lowercase();
        if a.len() > 2 && !a[2..].iter().any(|w| n.contains(w.as_str())) {
            continue;
        }
        println!("{} {}x{} fmt {} streams {:?}", im.name, im.width, im.height, im.format, im.streams);
    }
}
