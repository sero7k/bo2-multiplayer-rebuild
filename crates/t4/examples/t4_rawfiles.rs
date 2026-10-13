//! A World at War map zone's raw files (names and sizes; `<word>` prints
//! those whose name holds it): `t4_rawfiles <zone.ff> [word]`.
fn main() {
    let mut a = std::env::args().skip(1);
    let path = a.next().expect("zone");
    let word = a.next();
    let map = t4::load_map(std::path::Path::new(&path)).expect("read");
    println!("stopped at: {:?}", map.zone.stats.stopped_at);
    for asset in &map.zone.assets {
        if let iw3::zone::Asset::RawFile(r) = asset {
            println!("{} ({} bytes)", r.name, r.data.len());
            if word.as_deref().is_some_and(|w| r.name.contains(w)) {
                println!("{}", String::from_utf8_lossy(&r.data));
            }
        }
    }
}
