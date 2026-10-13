//! Lists a Ghosts zone's sound aliases matching a word (and how each is
//! stored): `iw6_alias_list <zone.ff> [word]`.
fn main() {
    let mut a = std::env::args().skip(1);
    let path = a.next().expect("zone");
    let word = a.next().unwrap_or_default().to_ascii_lowercase();
    let zone = fastfile_iw6::zone::Zone::open(std::path::Path::new(&path)).expect("open");
    let scan = zone.sounds();
    for l in &scan.lists {
        let name = l.name.display();
        if !name.to_ascii_lowercase().contains(&word) {
            continue;
        }
        let kinds: Vec<String> = l
            .aliases
            .iter()
            .map(|al| match al.sound.get().map(|f| &f.data) {
                Some(fastfile_iw6::sound::SoundData::Loaded(fastfile_iw6::sound::LoadedRef::Inline(s))) => format!("loaded {} fmt{} {}Hz {}ch {}b", s.name.display(), s.format, s.sample_rate, s.channels, s.data_len),
                Some(fastfile_iw6::sound::SoundData::Loaded(_)) => "loaded (shared)".into(),
                Some(d) => format!("{}", if d.stream().is_some() { "streamed" } else { "other" }),
                None => "unresolved".into(),
            })
            .collect();
        println!("{name}: {}", kinds.join(", "));
    }
}
