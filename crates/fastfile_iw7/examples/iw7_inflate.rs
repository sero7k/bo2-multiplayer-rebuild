//! Debug: writes a zone's inflated image, and (given its copied shared
//! ranges) its inflated shared VIRTUAL data:
//! `iw7_inflate <zone.ff> <out.bin> [<shared ranges.bin> <shared out.bin>]`.
fn main() {
    let a: Vec<String> = std::env::args().collect();
    let file = std::fs::read(&a[1]).expect("read zone");
    let (h, image) = fastfile_iw7::inflate_zone(&file).expect("inflate");
    std::fs::write(&a[2], &image).expect("write");
    println!("zone image {:#x} bytes", image.len());
    if a.len() > 4 {
        let raw = std::fs::read(&a[3]).expect("read shared");
        let sv = fastfile_iw7::pak::SharedVirtual::from_concatenated(&raw, &h.shared_files).expect("shared");
        std::fs::write(&a[4], &sv.data).expect("write");
        println!("shared {:#x} bytes (header says {:#x})", sv.data.len(), h.shared_virtual);
    }
}
