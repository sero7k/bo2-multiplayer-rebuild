//! A zone's technique sets: `t7_techsets <zone.ff> [substring]` prints
//! each with how many of its 12 techniques have programs this walk could
//! follow, and the texture slots its pixel programs bind.
fn main() {
    let a: Vec<String> = std::env::args().collect();
    let zone = fastfile_t7::Zone::open(std::path::Path::new(&a[1])).expect("zone");
    let want = a.get(2).cloned().unwrap_or_default();
    let w = fastfile_t7::walk::walk(&zone);
    println!("{} technique sets; walk {:?}", w.capture.techsets.len(), w.stop);
    for ts in w.capture.techsets.iter().filter(|t| t.name.contains(&want)) {
        let n = ts.programs.iter().filter(|p| p.is_some()).count();
        let binds: Vec<String> = ts
            .programs
            .iter()
            .flatten()
            .filter_map(|p| p[1])
            .filter_map(|(at, len)| zone.image.get(at..at + len))
            .filter_map(fastfile_t7::dxbc::reflect)
            .flat_map(|r| r.textures.into_iter().map(|t| t.0))
            .fold(Vec::new(), |mut v, t| {
                if !v.contains(&t) {
                    v.push(t);
                }
                v
            });
        println!("{n:2} {} {binds:?}", ts.name);
    }
}
