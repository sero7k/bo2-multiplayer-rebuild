//! Writes a zone's inflated image to a file: `zone_image_dump <zone.ff> <out>`.
fn main() {
    let a: Vec<String> = std::env::args().collect();
    let z = asset_transport::open_zone(&a[1]).unwrap();
    std::fs::write(&a[2], &z.bytes[..]).unwrap();
}
