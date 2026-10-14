//! String tables: `iw7_tables <zone.ff>... -- <substring>` prints the rows
//! of every table holding the substring (in any cell).
fn main() {
    let a: Vec<String> = std::env::args().skip(1).collect();
    let split = a.iter().position(|x| x == "--").unwrap_or(a.len());
    let pat = a.get(split + 1).cloned().unwrap_or_default();
    for p in &a[..split] {
        let zone = fastfile_iw7::Zone::open(std::path::Path::new(p)).expect("zone");
        let (cap, _) = fastfile_iw7::walk::walk_zone(&zone);
        for t in &cap.string_tables {
            for r in 0..t.rows {
                let row: Vec<&str> = (0..t.columns).map(|c| t.cell(r, c)).collect();
                if row.iter().any(|c| c.contains(&pat)) {
                    println!("{} [{r}] {}", t.name, row.join(" | "));
                }
            }
        }
    }
}
