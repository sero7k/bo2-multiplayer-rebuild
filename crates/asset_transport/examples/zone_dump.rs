fn main() {
    let mut a = std::env::args().skip(1);
    let src = a.next().unwrap();
    let dst = a.next().unwrap();
    let img = asset_transport::open_zone(&src).expect("open");
    std::fs::write(&dst, &img.bytes).unwrap();
    println!("{:?} {}", img.game, img.bytes.len());
}
