//! Compiles GSC source files and reports errors: `gsc_compile_check <file>...`.
fn main() {
    for f in std::env::args().skip(1) {
        let src = std::fs::read_to_string(&f).unwrap();
        let name = f.rsplit_once("Core/").map_or(f.as_str(), |(_, n)| n).trim_end_matches(".gsc").to_owned();
        match gsc_t6::compile(&name, &src) {
            Ok(_) => println!("OK   {name}"),
            Err(e) => println!("FAIL {name}: {}", e.lines().next().unwrap_or("")),
        }
    }
}
