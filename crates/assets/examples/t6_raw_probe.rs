//! Prints a T6 zone's raw files whose name holds a pattern:
//! `t6_raw_probe <zone.ff> <pattern> [print]`.
fn main() {
    let a: Vec<String> = std::env::args().collect();
    let c = asset_t6::capture_zone(std::path::Path::new(&a[1])).unwrap();
    for (name, bytes) in &c.raw_files {
        if name.contains(&a[2]) {
            println!("{name} ({} bytes)", bytes.len());
            if a.len() > 3 {
                println!("{}", String::from_utf8_lossy(bytes));
            }
        }
    }
}
