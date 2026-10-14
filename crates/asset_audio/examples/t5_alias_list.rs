//! Lists a Black Ops zone's sound aliases matching any of some words (and
//! how each clip is stored): `t5_alias_list <zone.ff> [word ...]`.
fn main() {
    let mut a = std::env::args().skip(1);
    let path = a.next().expect("zone");
    let words: Vec<String> = a.map(|w| w.to_ascii_lowercase()).collect();
    let cat = asset_audio::load_sound_catalog_t5(std::path::Path::new(&path)).expect("read");
    for s in &cat.sounds {
        let name = s.name.to_ascii_lowercase();
        if !words.is_empty() && !words.iter().any(|w| name.contains(w.as_str())) {
            continue;
        }
        let kinds: Vec<String> = s
            .aliases
            .iter()
            .map(|al| match &al.streamed {
                Some((dir, f)) => format!("{} {dir}/{f}", al.file_kind()),
                None => format!("{} {}", al.file_kind(), al.loaded_name.as_deref().or(al.file_name.as_deref()).unwrap_or("")),
            })
            .collect();
        println!("{}: {}", s.name, kinds.join(", "));
    }
}
