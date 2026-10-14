//! Localized strings: `iw7_localize <zone.ff>... -- <substring>` (key or text).
fn main() {
    let a: Vec<String> = std::env::args().skip(1).collect();
    let split = a.iter().position(|x| x == "--").unwrap_or(a.len());
    let pat = a.get(split + 1).cloned().unwrap_or_default();
    for p in &a[..split] {
        let zone = fastfile_iw7::Zone::open(std::path::Path::new(p)).expect("zone");
        let (cap, stop) = fastfile_iw7::walk::walk_zone(&zone);
        eprintln!("{p}: {} strings{}", cap.localize.len(), stop.map(|s| format!(" (stopped: {})", s.error)).unwrap_or_default());
        for (k, v) in &cap.localize {
            if k.contains(&pat) || v.contains(&pat) {
                println!("{k} = {v}");
            }
        }
    }
}
