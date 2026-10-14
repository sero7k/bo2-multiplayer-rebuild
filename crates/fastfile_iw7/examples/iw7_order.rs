//! The run of asset types in a zone's table (type x count, in order):
//! `iw7_order <zone.ff>`.
fn main() {
    let path = std::env::args().nth(1).expect("zone");
    let z = fastfile_iw7::Zone::open(std::path::Path::new(&path)).expect("open");
    let mut runs: Vec<(String, usize)> = Vec::new();
    for a in &z.assets.assets {
        let n = a.asset_type().map_or("?", |t| t.name()).to_owned();
        match runs.last_mut() {
            Some((t, c)) if *t == n => *c += 1,
            _ => runs.push((n, 1)),
        }
    }
    let mut at = 0;
    for (t, c) in runs {
        println!("{at:5} {t} x{c}");
        at += c;
    }
}
