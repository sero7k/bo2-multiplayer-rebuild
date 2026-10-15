//! bo2mp: Modern Warfare III (2023)'s gun names, by hand.
//!
//! The game's display names are localized strings it keeps obfuscated in
//! its zones, and those are not read; this table gives each gun the
//! public name it has in the game, by its internal name
//! (`jup_jp01_ar_golf36` is the Holger 556). Only guns whose public name is
//! known for certain are listed; the others are left out rather than
//! guessed. Each row: the internal name without its `jup_` prefix, the
//! public name, and Black Ops II's Create-a-Class group for it (battle
//! rifles as assault rifles, marksman rifles as sniper rifles, as Black
//! Ops II has no class of either), and its Create-a-Class slot.

/// (internal name without `jup_`, public name, BO2 item group, slot).
pub const JUP_NAMES: &[(&str, &str, &str, &str)] = &[
    // Assault rifles.
    ("jp01_ar_golf36", "Holger 556", "weapon_assault", "primary"),
    ("jp19_ar_acharlie", "MCW", "weapon_assault", "primary"),
    ("jp36_ar_anov94", "SVA 545", "weapon_assault", "primary"),
    ("jp44_ar_stango44", "STG44", "weapon_assault", "primary"),
    ("jp08_ar_qbravo97", "DG-56", "weapon_assault", "primary"),
    // Battle rifles.
    ("cp08_br_xmike5", "Sidewinder", "weapon_assault", "primary"),
    ("jp19_br_acharlie450", "MCW 6.8", "weapon_assault", "primary"),
    // Submachine guns.
    ("jp04_sm_umike", "Striker", "weapon_smg", "primary"),
    ("jp07_sm_uzulu", "WSP Swarm", "weapon_smg", "primary"),
    // Light machine guns.
    ("jp01_lm_mgolf36", "Holger 26", "weapon_lmg", "primary"),
    ("jp06_lm_pkilop", "Pulemyot 762", "weapon_lmg", "primary"),
    ("jp08_lm_qbravo95lsw", "DG-58 LSW", "weapon_lmg", "primary"),
    // Shotguns.
    ("jp09_sh_oromeo12", "Riveter", "weapon_cqb", "primary"),
    ("jp16_sh_recho870", "Lockwood 680", "weapon_cqb", "primary"),
    ("jp38_sh_spapa12", "Reclaimer 18", "weapon_cqb", "primary"),
    // Marksman rifles.
    ("jp37_dm_kappa98", "KAR98K", "weapon_sniper", "primary"),
    // Pistols.
    ("jp12_pi_mike93", "Renetti", "weapon_pistol", "secondary"),
];

/// A Modern Warfare III gun's public name by its internal name, with or
/// without `jup_` and `_mp` (`jup_jp01_ar_golf36_mp` -> `Holger 556`).
#[must_use]
pub fn public_name(internal: &str) -> Option<&'static str> {
    let n = internal.to_ascii_lowercase();
    let n = n.strip_prefix("jup_").unwrap_or(&n);
    let n = n.strip_suffix("_mp").unwrap_or(n);
    JUP_NAMES.iter().find(|(code, ..)| *code == n).map(|(_, name, ..)| *name)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn names_by_internal_name() {
        assert_eq!(public_name("jup_jp01_ar_golf36_mp"), Some("Holger 556"));
        assert_eq!(public_name("jp01_lm_mgolf36"), Some("Holger 26"));
        assert_eq!(public_name("jup_jp99_ar_unknown"), None);
    }

    #[test]
    fn each_gun_once() {
        let mut seen = std::collections::HashSet::new();
        for (code, name, ..) in JUP_NAMES {
            assert!(seen.insert(*code), "{code} twice");
            assert!(seen.insert(*name), "{name} twice");
        }
    }
}
