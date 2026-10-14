//! Compiles a GSC source file and lists its functions: `gsc_compile <file.gsc> [name]`.
fn main() {
    let mut a = std::env::args().skip(1);
    let path = a.next().expect("file");
    let name = a.next().unwrap_or_else(|| "check".to_owned());
    let src = std::fs::read_to_string(&path).expect("read");
    match gsc_t6::compile(&name, &src) {
        Ok(s) => {
            for f in &s.functions {
                println!("{} ({} params)", f.name, f.params);
            }
        }
        Err(e) => {
            eprintln!("{e}");
            std::process::exit(1);
        }
    }
}
