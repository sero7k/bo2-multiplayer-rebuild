//! A zone's raw file names holding a word (and their sizes):
//! `t6_raw_names <zone.ff> [word] [out dir]` (with a folder: each written
//! there under its own name, slashes as `_`).
fn main() {
    let mut a = std::env::args().skip(1);
    let path = a.next().expect("zone");
    let word = a.next().unwrap_or_default();
    let out = a.next().map(std::path::PathBuf::from);
    let c = asset_t6::capture_zone(std::path::Path::new(&path)).expect("capture");
    for (name, bytes) in &c.raw_files {
        if name.contains(&word) {
            println!("{name} {}", bytes.len());
            if let Some(dir) = &out {
                let _ = std::fs::create_dir_all(dir);
                let _ = std::fs::write(dir.join(name.replace(['/', '\\'], "_")), bytes);
            }
        }
    }
}
