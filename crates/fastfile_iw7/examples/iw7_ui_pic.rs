//! A picture from `ui.ff` as PNG: `iw7_ui_pic <zone dir> <out.png> <name>...`.
fn main() {
    let a: Vec<String> = std::env::args().collect();
    let (w, h, rgba) = fastfile_iw7::ui_image::read_ui_image(std::path::Path::new(&a[1]), &a[3..]).expect("no picture");
    let f = std::fs::File::create(&a[2]).expect("png");
    let mut e = png::Encoder::new(std::io::BufWriter::new(f), w, h);
    e.set_color(png::ColorType::Rgba);
    e.write_header().and_then(|mut wr| wr.write_image_data(&rgba)).expect("png");
    println!("{w}x{h}");
}
