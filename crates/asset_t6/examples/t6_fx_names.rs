//! A Black Ops II zone's effects whose names hold a word:
//! `t6_fx_names <zone.ff> [word]` (their element counts too).
fn main() {
    let mut a = std::env::args().skip(1);
    let path = a.next().expect("zone");
    let word = a.next().unwrap_or_default().to_ascii_lowercase();
    let c = asset_t6::capture_zone(std::path::Path::new(&path)).expect("capture");
    for e in &c.fx {
        if e.name.to_ascii_lowercase().contains(&word) {
            println!("{} ({} elems)", e.name, e.elems.len());
        }
    }
}
