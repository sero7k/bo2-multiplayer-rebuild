//! Materials of a zone whose textures name an image holding a word, and
//! the zone's images holding it: `t6_image_users <zone.ff> <word>`.
fn main() {
    let mut a = std::env::args().skip(1);
    let path = a.next().expect("zone");
    let word = a.next().expect("word");
    let c = asset_t6::capture_zone(std::path::Path::new(&path)).expect("capture");
    for img in c.images.iter().filter(|i| i.name.contains(&word)) {
        println!("image {} {}x{}", img.name, img.width, img.height);
    }
    for m in &c.materials {
        let hits: Vec<String> = m
            .textures
            .iter()
            .filter_map(|t| t.image.and_then(|k| c.images.get(k.index)).map(|i| (t.semantic, i.name.clone())))
            .filter(|(_, n)| n.contains(&word))
            .map(|(s, n)| format!("{s}:{n}"))
            .collect();
        if !hits.is_empty() {
            println!("material {} {:?}", m.name, hits);
        }
    }
}
