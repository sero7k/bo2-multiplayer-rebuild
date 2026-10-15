//! bo2mp: a CoD4 / World at War zone's materials whose names (or technique
//! sets, or images) hold a pattern, each with its whole texture table:
//! `legacy_mat_dump <iw3|t4> <zone.ff> <pattern>...`.
fn main() {
    let a: Vec<String> = std::env::args().collect();
    let path = std::path::Path::new(&a[2]);
    let z = if a[1] == "t4" {
        t4::load_map(path).unwrap().zone
    } else {
        iw3::zone::Zone::parse(&iw3::fastfile::load(path).unwrap(), iw3::zone::ParseOptions::default()).unwrap()
    };
    let pats: Vec<String> = a[3..].iter().map(|p| p.to_ascii_lowercase()).collect();
    for asset in &z.assets {
        let iw3::zone::Asset::Material(m) = asset else { continue };
        let ts = m.technique_set.and_then(|t| z.technique_set(t)).map_or("", |t| t.name.as_str());
        let tex: Vec<String> = m
            .textures
            .iter()
            .map(|t| {
                let img = t.image.and_then(|i| z.image(i));
                format!(
                    "{:08x}/{:?}={}{}",
                    t.name_hash,
                    t.semantic,
                    img.map_or("-", |i| i.name.as_str()),
                    img.map_or(String::new(), |i| format!(
                        "({:?} {}x{} sem{} cat{}{})",
                        i.map_type,
                        i.width,
                        i.height,
                        i.semantic,
                        i.category,
                        if i.load_def.as_ref().is_some_and(|d| !d.data.is_empty()) { " inzone" } else { "" }
                    ))
                )
            })
            .collect();
        let all = format!("{} {ts} {}", m.name, tex.join(" ")).to_ascii_lowercase();
        if pats.iter().any(|p| all.contains(p.as_str())) {
            // The colour map as the lane decodes it: mean colour and alpha.
            let stats = m
                .textures
                .iter()
                .find(|t| t.semantic == iw3::zone::TextureSemantic::Color)
                .and_then(|t| t.image.and_then(|i| z.image(i)))
                .and_then(|i| {
                    let main = asset_transport::game_root_for_zone(path).ok()?.join("main");
                    asset_material::decode_ui_image_from_main(&main, &i.name).ok().flatten()
                })
                .map(|(w, h, px)| {
                    let n = (px.len() / 4).max(1) as f64;
                    let mean = |c: usize| px.chunks_exact(4).map(|p| f64::from(p[c])).sum::<f64>() / n;
                    let opaque = px.chunks_exact(4).filter(|p| p[3] >= 128).count() as f64 / n;
                    format!("{w}x{h} mean rgba {:.0} {:.0} {:.0} {:.0}, {:.0}% alpha>=128", mean(0), mean(1), mean(2), mean(3), opaque * 100.0)
                });
            println!("  colour: {stats:?}");
            println!("{} [{ts}] sort={} consts={:?}\n    {}", m.name, m.sort_key, m.constants.iter().map(|c| (&c.name, c.literal)).collect::<Vec<_>>(), tex.join("\n    "));
        }
    }
}
