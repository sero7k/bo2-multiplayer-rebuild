//! Prints a T6 zone's string table: `t6_table_probe <zone.ff> <name>`.
fn main() {
    let a: Vec<String> = std::env::args().collect();
    let c = asset_t6::capture_zone(std::path::Path::new(&a[1])).unwrap();
    for t in c.string_tables.iter().filter(|t| t.name.contains(&a[2])) {
        println!("== {} ({}x{})", t.name, t.rows, t.columns);
        for r in 0..t.rows {
            let row: Vec<&str> = (0..t.columns).map(|k| t.cells.get(r * t.columns + k).map_or("", |s| s.as_str())).collect();
            println!("{}", row.join(" | "));
        }
    }
}
