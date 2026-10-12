//! bo2mp: other games' guns as extra Black Ops II weapons ("guest guns").
//!
//! Each one gets a Create-a-Class row in mp/statsTable.csv (index from
//! `FIRST_INDEX`, reference `bo1_ak74u`) and the attachments BO2 lacks get
//! rows in mp/attachmentTable.csv; the game gives `bo1_ak74u_mp+reflex`,
//! which the weapon catalog names that game's own variant
//! (`ak74u_reflex_mp`). A game's guns are listed only once
//! `set_available` says its guns are there (the front end: the game is
//! installed; the match: its guns were loaded).

use std::sync::atomic::{AtomicU8, Ordering};

/// The first statstable index a guest gun takes (BO2's own end at 255).
pub const FIRST_INDEX: i32 = 300;

/// A guest gun: its game (`t5`), its own name there (`ak74u`), its name,
/// BO2's item group and slot, its attachments (as its game names them,
/// in the order Create-a-Class lists them) and its kill icon material.
#[derive(Clone, Copy, Debug)]
pub struct GuestGun {
    pub game: &'static str,
    pub gun: &'static str,
    pub name: &'static str,
    pub group: &'static str,
    pub slot: &'static str,
    pub attachments: &'static str,
    pub icon: &'static str,
}

const fn g(
    gun: &'static str,
    name: &'static str,
    group: &'static str,
    slot: &'static str,
    attachments: &'static str,
    icon: &'static str,
) -> GuestGun {
    GuestGun { game: "t5", gun, name, group, slot, attachments, icon }
}

const SMG: &str = "weapon_smg";
const AR: &str = "weapon_assault";
const LMG: &str = "weapon_lmg";
const SNIPER: &str = "weapon_sniper";
const CQB: &str = "weapon_cqb";
const PISTOL: &str = "weapon_pistol";
const LAUNCHER: &str = "weapon_launcher";
const SPECIAL: &str = "weapon_special";
const P: &str = "primary";
const S: &str = "secondary";

/// Black Ops' multiplayer guns (attachments as its variants in common_mp
/// carry them).
pub const BO1: &[GuestGun] = &[
    g("mp5k", "MP5K", SMG, P, "acog elbit reflex extclip rf silencer", "hud_icon_mp5"),
    g("skorpion", "Skorpion", SMG, P, "extclip grip rf silencer", "hud_icon_skorpion"),
    g("mac11", "MAC11", SMG, P, "elbit reflex extclip grip rf silencer", "hud_icon_mac11"),
    g("ak74u", "AK74u", SMG, P, "acog elbit reflex extclip dualclip gl grip rf silencer", "hud_icon_ak74u"),
    g("uzi", "Uzi", SMG, P, "acog elbit reflex extclip grip rf silencer", "hud_icon_uzi"),
    g("pm63", "PM63", SMG, P, "extclip grip rf", "hud_icon_pm63"),
    g("mpl", "MPL", SMG, P, "acog elbit reflex dualclip grip rf silencer", "hud_icon_mpl"),
    g("spectre", "Spectre", SMG, P, "acog elbit reflex extclip grip rf silencer", "hud_icon_spectre"),
    g("kiparis", "Kiparis", SMG, P, "acog elbit reflex extclip grip rf silencer", "hud_icon_kiparis"),
    g("m16", "M16", AR, P, "acog elbit reflex ir extclip dualclip gl mk ft silencer", "hud_icon_m16a4"),
    g("enfield", "Enfield", AR, P, "acog elbit reflex ir extclip dualclip gl mk ft silencer", ""),
    g("m14", "M14", AR, P, "acog elbit reflex ir extclip gl mk ft grip silencer", "hud_icon_m14"),
    g("famas", "FAMAS", AR, P, "acog elbit reflex ir extclip dualclip gl mk ft silencer", "hud_icon_famas"),
    g("galil", "Galil", AR, P, "acog elbit reflex ir extclip dualclip gl mk ft silencer", "hud_icon_galil"),
    g("aug", "AUG", AR, P, "acog elbit reflex ir extclip dualclip gl mk ft silencer", "hud_icon_aug"),
    g("fnfal", "FN FAL", AR, P, "acog elbit reflex ir extclip dualclip gl mk ft silencer", "hud_icon_fnfal"),
    g("ak47", "AK47", AR, P, "acog elbit reflex ir extclip dualclip gl mk ft silencer", "hud_icon_ak47"),
    g("commando", "Commando", AR, P, "acog elbit reflex ir extclip dualclip gl mk ft silencer", ""),
    g("g11", "G11", AR, P, "lps vzoom", "hud_icon_g11"),
    g("rottweil72", "Olympia", CQB, P, "", ""),
    g("ithaca", "Stakeout", CQB, P, "grip", ""),
    g("spas", "SPAS-12", CQB, P, "silencer", "hud_icon_spas12"),
    g("hs10", "HS10", CQB, P, "", ""),
    g("hk21", "HK21", LMG, P, "acog elbit reflex ir extclip", ""),
    g("rpk", "RPK", LMG, P, "acog elbit reflex ir extclip dualclip", ""),
    g("m60", "M60", LMG, P, "acog elbit reflex ir extclip grip", "hud_icon_m60e4"),
    g("stoner63", "Stoner63", LMG, P, "acog elbit reflex ir extclip", "hud_icon_stoner63a"),
    g("dragunov", "Dragunov", SNIPER, P, "acog ir vzoom extclip silencer", "hud_icon_dragunov"),
    g("wa2000", "WA2000", SNIPER, P, "acog ir vzoom extclip silencer", "hud_icon_waw2000"),
    g("l96a1", "L96A1", SNIPER, P, "acog ir vzoom extclip silencer", ""),
    g("psg1", "PSG1", SNIPER, P, "acog ir vzoom extclip silencer", ""),
    g("asp", "ASP", PISTOL, S, "", ""),
    g("m1911", "M1911", PISTOL, S, "upgradesight extclip silencer", ""),
    g("makarov", "Makarov", PISTOL, S, "upgradesight extclip silencer", "hud_icon_makarov"),
    g("python", "Python", PISTOL, S, "acog snub speed", "hud_icon_colt"),
    g("cz75", "CZ75", PISTOL, S, "upgradesight auto extclip silencer", "hud_icon_cz75"),
    g("m72_law", "M72 LAW", LAUNCHER, S, "", ""),
    g("rpg", "RPG", LAUNCHER, S, "", "hud_icon_rpg"),
    g("strela", "Strela-3", LAUNCHER, S, "", ""),
    g("china_lake", "China Lake", LAUNCHER, S, "", ""),
    g("crossbow_explosive", "Crossbow", SPECIAL, S, "", "hud_icon_crossbow"),
    g("knife_ballistic", "Ballistic Knife", SPECIAL, S, "", ""),
];

const fn m(
    gun: &'static str,
    name: &'static str,
    group: &'static str,
    slot: &'static str,
    attachments: &'static str,
) -> GuestGun {
    GuestGun { game: "iw4", gun, name, group, slot, attachments, icon: "" }
}

const MW2_AR: &str = "reflex eotech acog thermal heartbeat gl shotgun silencer fmj xmags";
const MW2_SMG: &str = "reflex eotech acog thermal rof akimbo silencer fmj xmags";
const MW2_LMG: &str = "reflex eotech acog thermal heartbeat grip silencer fmj xmags";
const MW2_SNIPER: &str = "acog thermal heartbeat silencer fmj xmags";
const MW2_MP: &str = "reflex eotech akimbo silencer fmj xmags";
const MW2_SHOTGUN: &str = "reflex eotech grip silencer fmj xmags";
const MW2_PISTOL: &str = "tactical akimbo silencer fmj xmags";

/// Modern Warfare 2's multiplayer guns (attachments as its variants in
/// common_mp carry them).
pub const MW2: &[GuestGun] = &[
    m("m4", "M4A1", AR, P, MW2_AR),
    m("famas", "FAMAS", AR, P, MW2_AR),
    m("scar", "SCAR-H", AR, P, MW2_AR),
    m("tavor", "TAR-21", AR, P, MW2_AR),
    m("fal", "FAL", AR, P, MW2_AR),
    m("m16", "M16A4", AR, P, MW2_AR),
    m("masada", "ACR", AR, P, MW2_AR),
    m("fn2000", "F2000", AR, P, MW2_AR),
    m("ak47", "AK-47", AR, P, MW2_AR),
    m("mp5k", "MP5K", SMG, P, MW2_SMG),
    m("ump45", "UMP45", SMG, P, MW2_SMG),
    m("kriss", "Vector", SMG, P, MW2_SMG),
    m("p90", "P90", SMG, P, MW2_SMG),
    m("uzi", "Mini-Uzi", SMG, P, MW2_SMG),
    m("sa80", "L86 LSW", LMG, P, MW2_LMG),
    m("rpd", "RPD", LMG, P, MW2_LMG),
    m("mg4", "MG4", LMG, P, MW2_LMG),
    m("aug", "AUG HBAR", LMG, P, MW2_LMG),
    m("m240", "M240", LMG, P, MW2_LMG),
    m("cheytac", "Intervention", SNIPER, P, MW2_SNIPER),
    m("barrett", "Barrett .50cal", SNIPER, P, MW2_SNIPER),
    m("wa2000", "WA2000", SNIPER, P, MW2_SNIPER),
    m("m21", "M21 EBR", SNIPER, P, MW2_SNIPER),
    m("spas12", "SPAS-12", CQB, P, MW2_SHOTGUN),
    m("aa12", "AA-12", CQB, P, MW2_SHOTGUN),
    m("striker", "Striker", CQB, P, MW2_SHOTGUN),
    m("m1014", "M1014", CQB, P, MW2_SHOTGUN),
    m("ranger", "Ranger", CQB, S, "akimbo fmj"),
    m("model1887", "Model 1887", CQB, S, "akimbo fmj"),
    m("pp2000", "PP2000", PISTOL, S, MW2_MP),
    m("glock", "G18", PISTOL, S, MW2_MP),
    m("beretta393", "M93 Raffica", PISTOL, S, MW2_MP),
    m("tmp", "TMP", PISTOL, S, MW2_MP),
    m("usp", "USP .45", PISTOL, S, MW2_PISTOL),
    m("coltanaconda", ".44 Magnum", PISTOL, S, "tactical akimbo fmj"),
    m("beretta", "M9", PISTOL, S, MW2_PISTOL),
    m("deserteagle", "Desert Eagle", PISTOL, S, "tactical akimbo fmj"),
    m("at4", "AT4-HS", LAUNCHER, S, ""),
    m("rpg", "RPG-7", LAUNCHER, S, ""),
    m("stinger", "Stinger", LAUNCHER, S, ""),
    m("javelin", "Javelin", LAUNCHER, S, ""),
    m("m79", "Thumper", LAUNCHER, S, ""),
];

const fn gh(gun: &'static str, name: &'static str, group: &'static str, slot: &'static str) -> GuestGun {
    GuestGun { game: "iw6", gun, name, group, slot, attachments: "", icon: "" }
}

/// bo2mp: Call of Duty: Ghosts' multiplayer guns (`iw6_<gun>_mp` in its
/// common_core_mp; no attachments yet: Ghosts builds its variants at run
/// time from attachment assets). Listed after every other game's so the
/// statstable indices before them never move.
pub const GHOSTS: &[GuestGun] = &[
    gh("ak12", "AK-12", AR, P),
    gh("sc2010", "SC-2010", AR, P),
    gh("honeybadger", "Honey Badger", AR, P),
    gh("bren", "Bren", AR, P),
    gh("msbs", "MSBS", AR, P),
    gh("arx160", "ARX-160", AR, P),
    gh("fads", "FAD", AR, P),
    gh("r5rgp", "Remington R5", AR, P),
    gh("kriss", "Vector CRB", SMG, P),
    gh("pp19", "Bizon", SMG, P),
    gh("vepr", "Vepr", SMG, P),
    gh("microtar", "MTAR-X", SMG, P),
    gh("cbjms", "CBJ-MS", SMG, P),
    gh("k7", "K7", SMG, P),
    gh("m27", "M27-IAR", LMG, P),
    gh("ameli", "Ameli", LMG, P),
    gh("lsat", "LSAT", LMG, P),
    gh("kac", "Chain SAW", LMG, P),
    gh("vks", "VKS", SNIPER, P),
    gh("gm6", "Lynx", SNIPER, P),
    gh("usr", "USR", SNIPER, P),
    gh("l115a3", "L115", SNIPER, P),
    gh("imbel", "IA-2", SNIPER, P),
    gh("svu", "SVU", SNIPER, P),
    gh("g28", "MR-28", SNIPER, P),
    gh("mk14", "MK14 EBR", SNIPER, P),
    gh("mts255", "MTS-255", CQB, P),
    gh("fp6", "FP6", CQB, P),
    gh("maul", "Maul", CQB, P),
    gh("uts15", "Bulldog", CQB, P),
    gh("p226", "P226", PISTOL, S),
    gh("m9a1", "M9A1", PISTOL, S),
    gh("mp443", "MP-443 Grach", PISTOL, S),
    gh("magnum", ".44 Magnum", PISTOL, S),
    gh("pdw", "PDW", PISTOL, S),
    gh("panzerfaust3", "Panzerfaust", LAUNCHER, S),
    gh("mk32", "MK32", LAUNCHER, S),
    gh("maaws", "MAAWS", LAUNCHER, S),
    gh("rgm", "Kastet", LAUNCHER, S),
];

/// Attachments the guest guns have and BO2's table lacks: reference, slot,
/// name, picture.
const EXTRA_ATTACHMENTS: &[(&str, &str, &str, &str)] = &[
    ("elbit", "top", "Red Dot Sight", "cac_mods_red_dot"),
    ("lps", "top", "Low Power Scope", "cac_mods_var_zoom"),
    ("upgradesight", "top", "Upgraded Iron Sights", "cac_mods_ironsight"),
    ("ft", "bottom", "Flamethrower", "t5:menu_mp_weapons_attach_ft_hirez"),
    ("auto", "trigger", "Full Auto Upgrade", "cac_mods_trigger_group"),
    ("snub", "muzzle", "Snub Nose", "cac_mods_barrel_extend"),
    ("speed", "trigger", "Speed Reloader", "cac_mods_dual_clip"),
    ("eotech", "top", "Holographic Sight", "cac_mods_holographic"),
    ("thermal", "top", "Thermal Scope", "cac_mods_dual_band"),
    ("heartbeat", "bottom", "Heartbeat Sensor", "cac_mods_rangefinder"),
    ("shotgun", "bottom", "Shotgun", "cac_mods_trigger_group"),
    ("xmags", "trigger", "Extended Mags", "cac_mods_extended_mag"),
    ("rof", "trigger", "Rapid Fire", "cac_mods_rapid_fire"),
    ("akimbo", "bottom", "Akimbo", "cac_mods_dual_wield"),
    ("tactical", "bottom", "Tactical Knife", "cac_mods_tact_knife"),
];

/// The prefix a game's guns carry (`t5` -> `bo1_`).
#[must_use]
pub fn prefix(game: &str) -> &'static str {
    match game {
        "t5" => "bo1_",
        "iw4" => "mw2_",
        "iw5" => "mw3_",
        "iw6" => "gh_",
        _ => "",
    }
}

const GAMES: [&str; 4] = ["t5", "iw4", "iw5", "iw6"];
static AVAILABLE: AtomicU8 = AtomicU8::new(0);

fn bit(game: &str) -> u8 {
    GAMES.iter().position(|g| *g == game).map_or(0, |i| 1 << i)
}

/// Whether `game`'s guns are listed (`t5`, `iw4`, `iw5`, `iw6`).
pub fn set_available(game: &str, on: bool) {
    if on {
        AVAILABLE.fetch_or(bit(game), Ordering::Relaxed);
    } else {
        AVAILABLE.fetch_and(!bit(game), Ordering::Relaxed);
    }
}

#[must_use]
pub fn available(game: &str) -> bool {
    AVAILABLE.load(Ordering::Relaxed) & bit(game) != 0
}

/// Every guest gun, available or not, in table order.
pub fn all() -> impl Iterator<Item = &'static GuestGun> {
    BO1.iter().chain(MW2.iter()).chain(GHOSTS.iter())
}

impl GuestGun {
    /// The statstable reference (`bo1_ak74u`).
    #[must_use]
    pub fn reference(&self) -> String {
        format!("{}{}", prefix(self.game), self.gun)
    }

    fn key(&self) -> String {
        format!("WEAPON_{}", self.reference().to_ascii_uppercase())
    }

    fn game_title(&self) -> &'static str {
        match self.game {
            "t5" => "Call of Duty: Black Ops",
            "iw4" => "Modern Warfare 2",
            "iw5" => "Modern Warfare 3",
            "iw6" => "Call of Duty: Ghosts",
            _ => "another game",
        }
    }

    /// Create-a-Class pictures to try: the game's own menu picture, then
    /// its kill icon (`t5:menu_mp_weapons_ak74u|hud_icon_ak74u`).
    fn picture(&self) -> String {
        match self.game {
            // (MW2's Create-a-Class pictures are `weapon_<gun>`.)
            "iw4" => format!("iw4:weapon_{0}|hud_icon_{0}", self.gun),
            _ => {
                let mut p = format!("{}:menu_mp_weapons_{}", self.game, self.gun);
                if !self.icon.is_empty() {
                    p.push('|');
                    p.push_str(self.icon);
                }
                p
            }
        }
    }
}

/// The text keys and words the guest rows use (key, text).
#[must_use]
pub fn localize() -> Vec<(String, String)> {
    let mut out = Vec::new();
    for gun in all() {
        out.push((gun.key(), gun.name.to_owned()));
        out.push((format!("{}_DESC", gun.key()), format!("The {} from {}.", gun.name, gun.game_title())));
    }
    for (reference, _, name, _) in EXTRA_ATTACHMENTS {
        let key = format!("MPUI_BO2MP_{}", reference.to_ascii_uppercase());
        out.push((key.clone(), (*name).to_owned()));
        out.push((format!("{key}_DESC"), (*name).to_owned()));
    }
    out.push(("MPUI_MK".to_owned(), "Masterkey".to_owned()));
    out
}

/// Whether `extend_table` adds guest rows to this table (name any case).
#[must_use]
pub fn is_guest_table(name: &str) -> bool {
    name.eq_ignore_ascii_case("mp/statsTable.csv") || name.eq_ignore_ascii_case("mp/attachmentTable.csv")
}

/// mp/statsTable.csv: a row per available guest gun (none twice).
pub fn extend_stats_rows(rows: &mut Vec<Vec<String>>) -> bool {
    let width = rows.iter().map(Vec::len).max().unwrap_or(19).max(19);
    let mut changed = false;
    let mut sort: std::collections::HashMap<&str, i32> = std::collections::HashMap::new();
    for (k, gun) in all().enumerate() {
        if !available(gun.game) {
            continue;
        }
        let reference = gun.reference();
        if rows.iter().any(|r| r.get(4).is_some_and(|c| *c == reference)) {
            continue;
        }
        let order = sort.entry(gun.group).or_insert(20);
        *order += 1;
        let mut row = vec![String::new(); width];
        row[0] = (FIRST_INDEX + k as i32).to_string();
        row[2] = gun.group.to_owned();
        row[3] = gun.key();
        row[4] = reference;
        if gun.group == LAUNCHER {
            row[5] = "1".to_owned();
        }
        row[6] = gun.picture();
        row[7] = format!("{}_DESC", gun.key());
        row[8] = gun.attachments.to_owned();
        row[9] = order.to_string();
        // (Unlocked from the start, a point each in Pick 10.)
        row[10] = "0".to_owned();
        row[12] = "1".to_owned();
        row[13] = gun.slot.to_owned();
        rows.push(row);
        changed = true;
    }
    changed
}

/// mp/attachmentTable.csv: the guest attachments BO2's table lacks.
pub fn extend_attachment_rows(rows: &mut Vec<Vec<String>>) -> bool {
    if !GAMES.iter().any(|g| available(g)) {
        return false;
    }
    let width = rows.iter().map(Vec::len).max().unwrap_or(20).max(20);
    let all_refs: Vec<&str> = EXTRA_ATTACHMENTS.iter().map(|a| a.0).collect();
    let mut changed = false;
    for (k, (reference, slot, _, image)) in EXTRA_ATTACHMENTS.iter().enumerate() {
        if rows.iter().any(|r| r.get(2).is_some_and(|c| c == "attachment") && r.get(4).is_some_and(|c| c == reference)) {
            continue;
        }
        let key = format!("MPUI_BO2MP_{}", reference.to_ascii_uppercase());
        let mut row = vec![String::new(); width];
        row[0] = (100 + k).to_string();
        row[1] = (*slot).to_owned();
        row[2] = "attachment".to_owned();
        row[3] = key.clone();
        row[4] = (*reference).to_owned();
        row[6] = (*image).to_owned();
        row[7] = format!("{key}_DESC");
        row[8] = reference.to_ascii_uppercase();
        row[9] = (40 + k).to_string();
        row[11] = format!(
            "acog dualclip extclip gl grip ir reflex rf silencer vzoom mk {}",
            all_refs.iter().filter(|r| *r != reference).copied().collect::<Vec<_>>().join(" ")
        );
        row[12] = "1".to_owned();
        row[18] = "0".to_owned();
        rows.push(row);
        changed = true;
    }
    changed
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn rows_only_for_available_games_and_once() {
        set_available("t5", false);
        set_available("iw4", false);
        let mut rows = vec![vec!["0".to_owned(); 19]];
        assert!(!extend_stats_rows(&mut rows));
        set_available("t5", true);
        set_available("iw4", true);
        assert!(extend_stats_rows(&mut rows));
        let refs: Vec<&String> = rows.iter().map(|r| &r[4]).collect();
        assert!(refs.iter().any(|r| *r == "mw2_m4"));
        let mut indices: Vec<&String> = rows.iter().map(|r| &r[0]).collect();
        indices.sort();
        indices.dedup();
        assert_eq!(indices.len(), rows.len(), "indices unique");
        let ak = rows.iter().find(|r| r[4] == "bo1_ak74u").expect("ak74u row");
        assert_eq!(ak[2], "weapon_smg");
        assert_eq!(ak[13], "primary");
        assert!(ak[6].starts_with("t5:menu_mp_weapons_ak74u|"));
        assert!(ak[0].parse::<i32>().unwrap() >= FIRST_INDEX);
        let n = rows.len();
        assert!(!extend_stats_rows(&mut rows));
        assert_eq!(rows.len(), n);
        let mut atts = vec![vec![String::new(); 20]];
        assert!(extend_attachment_rows(&mut atts));
        assert!(atts.iter().any(|r| r[4] == "elbit" && r[2] == "attachment"));
        // Every guest gun's attachment is BO2's or one added here.
        let bo2 = ["acog", "reflex", "ir", "extclip", "dualclip", "gl", "mk", "grip", "rf", "silencer", "vzoom", "fmj"];
        for gun in all() {
            for a in gun.attachments.split_whitespace() {
                assert!(bo2.contains(&a) || atts.iter().any(|r| r[4] == a), "{} {a}", gun.gun);
            }
        }
    }
}
