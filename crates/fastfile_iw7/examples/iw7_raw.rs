//! Prints raw files of a zone: `iw7_raw <zone.ff> <name substring>...`.
fn main() {
    let a: Vec<String> = std::env::args().collect();
    let zone = fastfile_iw7::Zone::open(std::path::Path::new(&a[1])).expect("zone");
    let (w, _) = fastfile_iw7::walk::walk_fitted(&zone);
    for r in &w.cap.raw_files {
        if a[2..].iter().any(|s| r.name.contains(s.as_str())) {
            let body = fastfile_iw7::map::raw_file(&w.cap, &zone.image, &r.name).unwrap_or_default();
            println!("==== {} ({} bytes)\n{}", r.name, body.len(), String::from_utf8_lossy(&body));
        }
    }
}
