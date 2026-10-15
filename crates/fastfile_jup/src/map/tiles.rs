//! A big map's streamed world zones. Rebirth Island (`mp_jup_escape5`)
//! keeps most of its world out of the map zone: each surface of the map's
//! `gfx_map` names the world zone its vertices are in, 0 for the map
//! zone's own `gfx_map_trzone`, k for the k-th of the map's streamed zones
//! at their finest level (`<map>_<x>_<y>_0_0_tr`, the swap zones
//! `<map>_<part>_swap_<state>_0_tr`), counted in name order.
//!
//! Checked on `mp_jup_escape5` (35 zones): every zone's triangle bytes are
//! exactly what the surfaces that name it add up to.
//!
//! Left out: other maps whose name starts with this one's (`_hell` is a
//! map of its own), the coarse levels (`_3_tr`), and the collision grid
//! zones (`_cg_`).

use super::MapSources;

/// The map's streamed world zones (names without `.ff`), in the order the
/// surface list counts them (index 0 here is world zone 1).
pub fn names(src: &MapSources) -> Vec<String> {
    let dir = src.install.join("zone");
    let Ok(read) = std::fs::read_dir(&dir) else {
        return Vec::new();
    };
    let prefix = format!("{}_", src.name);
    let mut all = Vec::new();
    for e in read.flatten() {
        let name = e.file_name().to_string_lossy().to_ascii_lowercase();
        if let Some(stem) = name.strip_suffix(".ff")
            && stem.starts_with(&prefix)
        {
            all.push(stem.to_owned());
        }
    }
    filter(&src.name, &all)
}

/// The finest-level world zones of `map` among `stems`, in name order.
pub fn filter(map: &str, stems: &[String]) -> Vec<String> {
    let prefix = format!("{map}_");
    // Maps whose name extends this one's (`<map>_hell`): their zones are
    // theirs.
    let others: Vec<String> = stems
        .iter()
        .filter(|s| !s.ends_with("_tr"))
        .map(|s| format!("{s}_"))
        .collect();
    let mut out: Vec<String> = stems
        .iter()
        .filter(|s| {
            s.starts_with(&prefix)
                && s.ends_with("_0_tr")
                && !s[prefix.len()..].starts_with("cg_")
                && !others.iter().any(|o| s.starts_with(o.as_str()))
        })
        .cloned()
        .collect();
    out.sort();
    out.dedup();
    out
}

#[cfg(test)]
mod tests {
    #[test]
    fn rebirth_order() {
        let stems: Vec<String> = [
            "mp_jup_escape5_hell",
            "mp_jup_escape5_hell_0_0_0_0_tr",
            "mp_jup_escape5_0_0_0_3_tr",
            "mp_jup_escape5_lighthouse_swap_dflt_0_tr",
            "mp_jup_escape5_0_0_0_0_tr",
            "mp_jup_escape5_cg_00_0462_tr",
            "mp_jup_escape5_-1563_-313_0_0_tr",
            "mp_jup_escape5_-313_-313_0_0_tr",
        ]
        .map(String::from)
        .to_vec();
        assert_eq!(
            super::filter("mp_jup_escape5", &stems),
            [
                "mp_jup_escape5_-1563_-313_0_0_tr",
                "mp_jup_escape5_-313_-313_0_0_tr",
                "mp_jup_escape5_0_0_0_0_tr",
                "mp_jup_escape5_lighthouse_swap_dflt_0_tr",
            ]
        );
    }
}
