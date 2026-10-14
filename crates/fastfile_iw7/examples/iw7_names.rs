//! Asset names of a type: `iw7_names <zone.ff> <type name> [substring]`.
fn main() {
    let a: Vec<String> = std::env::args().collect();
    let zone = fastfile_iw7::Zone::open(std::path::Path::new(&a[1])).expect("zone");
    let ty = fastfile_iw7::AssetType::ALL.iter().copied().find(|t| t.name() == a[2]).expect("type");
    let (cap, _) = fastfile_iw7::walk::walk_zone(&zone);
    for n in cap.names(ty) {
        if a.get(3).is_none_or(|s| n.contains(s.as_str())) {
            println!("{n}");
        }
    }
}
