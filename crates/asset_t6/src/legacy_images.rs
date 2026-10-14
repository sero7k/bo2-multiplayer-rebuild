//! bo2zm: Black Ops II zones that name pictures no Black Ops II pack holds,
//! found in Black Ops' or World at War's IWDs instead.
//!
//! The Zombies Declassified "DLC5" map pack (Kino der Toten, Ascension,
//! Five, Shangri-La, Moon and the World at War maps, as Black Ops II
//! zones) was built from Black Ops' own art. Its zones stream their
//! pictures from packs it does not ship: some are in Black Ops II's packs
//! under the same names (`PackSet::locate` finds those by name), the rest
//! only in Black Ops' (and World at War's) `main/*.iwd` archives, where
//! these helpers find them by name. Read only.

use std::path::{Path, PathBuf};
use std::sync::OnceLock;

/// Black Ops' and World at War's `main` folders under `IW4L_GAMES`, found
/// once (either may be missing).
pub fn legacy_iwd_mains() -> &'static [PathBuf] {
    static MAINS: OnceLock<Vec<PathBuf>> = OnceLock::new();
    MAINS.get_or_init(|| {
        let Ok(games) = asset_transport::discover::games_root_from_env() else {
            return Vec::new();
        };
        let mut out: Vec<PathBuf> = asset_transport::NamespaceTrees::discover(&games)
            .main_for(asset_core::AssetNamespace::T5)
            .map(Path::to_path_buf)
            .into_iter()
            .collect();
        if let Some(main) =
            asset_transport::find_common_mp_for_envelope(&games, asset_transport::T4_ZONE_VERSION)
                .ok()
                .and_then(|zone| asset_transport::game_root_for_zone(&zone.path).ok())
                .map(|root| root.join("main"))
        {
            out.push(main);
        }
        out.retain(|m| m.is_dir());
        out.dedup();
        out
    })
}

/// The 8-hex-digit hash a cut name ends with (`...chair~90107ea7`), split
/// off: (the start, true); else (the name, false).
fn split_cut(name: &str) -> (&str, bool) {
    match name.rsplit_once('~') {
        Some((start, hash)) if hash.len() == 8 && hash.bytes().all(|b| b.is_ascii_hexdigit()) => (start, true),
        _ => (name, false),
    }
}

/// A normal map's name (`_n`, `_nml`, `_normal`, also as a merged
/// picture's part): those games store normals as DXT5 with X in alpha,
/// not as Black Ops II's shaders read them, so they are not taken.
pub fn is_normal_map_name(name: &str) -> bool {
    let lower = name.to_ascii_lowercase();
    let (base, _) = split_cut(&lower);
    ["_n", "_nml", "_normal"]
        .iter()
        .any(|s| base.ends_with(s) || base.contains(&format!("{s}-")))
}

/// The names a Black Ops II image name may have in those IWDs, most
/// likely first: itself; a merged picture's colour part
/// (`~~-gp_eb_china_cabinet_c-rgb&~37fd4b10` -> `~-gp_eb_china_cabinet_c`);
/// a gamma-marked name without its mark (`~-g`), a plain one with it.
/// Lower case.
pub fn legacy_names(name: &str) -> Vec<String> {
    let name = name.to_ascii_lowercase();
    let mut out = vec![name.clone()];
    if let Some(rest) = name.strip_prefix("~~-g")
        && let Some((base, _)) = rest.split_once("-rgb")
    {
        out.push(format!("~-g{base}"));
        out.push(base.to_owned());
    }
    if let Some(base) = name.strip_prefix("~-g") {
        out.push(base.to_owned());
    } else if !name.starts_with('~') {
        out.push(format!("~-g{name}"));
    }
    out
}

/// A name the zone tool cut to fit (`~~-gstatic_berlin_metal_chair~90107ea7`:
/// a start, `~`, a hash): among `sorted` (an archive's image names, lower
/// case, sorted) the one gamma-marked picture (`~-g`) with that start, or
/// among several the one colour map (`_c`).
pub fn legacy_cut_name(name: &str, sorted: &[String]) -> Option<String> {
    let name = name.to_ascii_lowercase();
    let (start, cut) = split_cut(&name);
    if !cut {
        return None;
    }
    let start = start.trim_start_matches("~~-g").trim_start_matches("~-g");
    // A merged picture's first part ends at its `-rgb...` (cut anywhere
    // after the `-`).
    let start = start.split_once('-').map_or(start, |(base, _)| base);
    if start.len() < 8 {
        return None;
    }
    let prefix = format!("~-g{start}");
    let at = sorted.partition_point(|n| n.as_str() < prefix.as_str());
    let hits: Vec<&String> = sorted[at..].iter().take_while(|n| n.starts_with(&prefix)).collect();
    match hits.as_slice() {
        [one] => Some((*one).clone()),
        many if many.iter().any(|n| **n == prefix) => Some(prefix),
        many => {
            let colour: Vec<&&String> = many.iter().filter(|n| n.ends_with("_c")).collect();
            (colour.len() == 1).then(|| (*colour[0]).clone())
        }
    }
}

/// Whether a name may be looked for there at all: not a normal map, not a
/// runtime picture (`*reflection_probe3`, `$white`).
pub fn wants_legacy(name: &str) -> bool {
    !(name.starts_with('*') || name.starts_with('$') || name.starts_with(',') || is_normal_map_name(name))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn merged_and_gamma_names_reduce_to_their_colour_part() {
        assert_eq!(
            legacy_names("~~-gp_eb_china_cabinet_c-rgb&~37fd4b10"),
            ["~~-gp_eb_china_cabinet_c-rgb&~37fd4b10", "~-gp_eb_china_cabinet_c", "p_eb_china_cabinet_c"]
        );
        assert_eq!(legacy_names("~-gV_opel_blitz_c"), ["~-gv_opel_blitz_c", "v_opel_blitz_c"]);
        assert_eq!(legacy_names("chalk_writing_powerpack_c"), ["chalk_writing_powerpack_c", "~-gchalk_writing_powerpack_c"]);
    }

    #[test]
    fn cut_names_find_their_one_picture() {
        let sorted: Vec<String> = ["~-gstatic_berlin_metal_chair_c", "~-gstatic_berlin_metal_chair_s", "~-gzz_other_c"]
            .map(str::to_owned)
            .to_vec();
        assert_eq!(
            legacy_cut_name("~~-gstatic_berlin_metal_chair~90107ea7", &sorted).as_deref(),
            Some("~-gstatic_berlin_metal_chair_c")
        );
        assert_eq!(legacy_cut_name("~-gstatic_berlin_metal_chair_c", &sorted), None);
        // Cut inside the merge's `-rgb`: the first part, whole.
        assert_eq!(
            legacy_cut_name("~~-gstatic_berlin_metal_chair_s-rg~9357aaf5", &sorted).as_deref(),
            Some("~-gstatic_berlin_metal_chair_s")
        );
    }

    #[test]
    fn normal_maps_and_runtime_pictures_are_left_alone() {
        assert!(!wants_legacy("p_eb_china_cabinet_n"));
        assert!(!wants_legacy("dlc_tanker_truck_nml"));
        assert!(!wants_legacy("*reflection_probe3"));
        assert!(wants_legacy("~-gv_opel_blitz_c"));
        assert!(wants_legacy("~~-gv_opel_blitz_s-rgb&~-rv_o~40a7bb79"));
    }
}
