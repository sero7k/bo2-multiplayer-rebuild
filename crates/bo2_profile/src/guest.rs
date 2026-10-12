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

const fn gh(
    gun: &'static str,
    name: &'static str,
    group: &'static str,
    slot: &'static str,
    attachments: &'static str,
    icon: &'static str,
) -> GuestGun {
    GuestGun { game: "iw6", gun, name, group, slot, attachments, icon }
}

// Ghosts' attachments as its weapon definitions list them (scopes,
// under-barrels, others), one word per attachment: the per-class copies
// (`reflexsmg`, `eotechlmg`, `silencer03`, `griphide`, `firetypeburstlight`)
// are the one Create-a-Class picks. Guns whose definitions list no sights
// take their class's (the assault rifles' and SMGs' are the AK-12's).
const GH_AR: &str = "reflex eotech acog hybrid thermal tracker barrelrange flashsuppress silencer xmags fmj";
const GH_AR_UB: &str =
    "reflex eotech acog hybrid thermal tracker gl grip shotgun barrelrange flashsuppress silencer xmags fmj";
const GH_SMG: &str = "reflex eotech acog hybrid thermal tracker grip barrelrange flashsuppress silencer xmags fmj";
const GH_LMG: &str = "acog hybrid thermal tracker barrelrange flashsuppress silencer xmags fmj";
const GH_DMR: &str = "reflex eotech acog thermal tracker silencer xmags fmj";
const GH_SNIPER: &str = "acog thermal vzscope xmags fmj";

/// bo2mp: Call of Duty: Ghosts' multiplayer guns (`iw6_<gun>_mp` in its
/// common_core_mp; Create-a-Class and the scripts call it `gh_<gun>`). Ghosts
/// builds a variant from attachment assets at run time; a guest variant
/// (`gh_ak12_mp+reflex`) is the base gun until those are applied. Pictures
/// are its ui_mp's `weapon_<gun>_360` (the `icon` names other ones). Its
/// statstable rows are numbered from `GHOSTS_FIRST_INDEX`.
pub const GHOSTS: &[GuestGun] = &[
    gh("ak12", "AK-12", AR, P, "reflex eotech acog hybrid thermal tracker gl grip shotgun barrelrange flashsuppress silencer xmags fmj firetypeburst firetypesingle", ""),
    gh("sc2010", "SC-2010", AR, P, "reflex eotech acog hybrid thermal tracker barrelrange flashsuppress silencer xmags fmj firetypeburst firetypesingle", ""),
    gh("honeybadger", "Honey Badger", AR, P, GH_AR, ""),
    gh("bren", "Bren", AR, P, GH_AR, ""),
    gh("msbs", "MSBS", AR, P, "reflex eotech acog hybrid thermal tracker gl grip shotgun barrelrange flashsuppress silencer xmags fmj firetypeauto firetypesingle", ""),
    gh("arx160", "ARX-160", AR, P, GH_AR_UB, ""),
    gh("fads", "FAD", AR, P, "reflex eotech acog hybrid thermal tracker gl shotgun barrelrange flashsuppress silencer xmags fmj", ""),
    gh("r5rgp", "Remington R5", AR, P, GH_AR, ""),
    gh("kriss", "Vector CRB", SMG, P, "reflex eotech acog hybrid thermal tracker grip barrelrange flashsuppress rof silencer xmags fmj", "weapon_kriss_360|hud_icon_kriss_iw6"),
    gh("pp19", "Bizon", SMG, P, GH_SMG, ""),
    gh("vepr", "Vepr", SMG, P, GH_SMG, ""),
    gh("microtar", "MTAR-X", SMG, P, GH_SMG, ""),
    gh("cbjms", "CBJ-MS", SMG, P, GH_SMG, ""),
    gh("k7", "K7", SMG, P, GH_SMG, "weapon_k7_ui_360|hud_icon_k7_ui"),
    gh("m27", "M27-IAR", LMG, P, "reflex eotech acog hybrid thermal tracker gl shotgun barrelrange flashsuppress silencer xmags fmj", ""),
    gh("ameli", "Ameli", LMG, P, GH_LMG, ""),
    gh("lsat", "LSAT", LMG, P, GH_LMG, ""),
    gh("kac", "Chain SAW", LMG, P, GH_LMG, ""),
    gh("vks", "VKS", SNIPER, P, "acog thermal vzscope barrelbored silencer xmags fmj", ""),
    gh("gm6", "Lynx", SNIPER, P, GH_SNIPER, ""),
    gh("usr", "USR", SNIPER, P, GH_SNIPER, ""),
    gh("l115a3", "L115", SNIPER, P, "acog thermal vzscope barrelbored silencer xmags fmj", ""),
    gh("imbel", "IA-2", SNIPER, P, "reflex eotech acog thermal tracker barrelrange flashsuppress silencer xmags fmj firetypeburst ironsight", ""),
    gh("svu", "SVU", SNIPER, P, GH_DMR, ""),
    gh("g28", "MR-28", SNIPER, P, GH_DMR, ""),
    gh("mk14", "MK14 EBR", SNIPER, P, GH_DMR, "weapon_mk14_360|hud_icon_mk14_iw6"),
    gh("mts255", "MTS-255", CQB, P, "reflex eotech grip ammoslug barrelrange silencer xmags", ""),
    gh("fp6", "FP6", CQB, P, "reflex eotech grip ammoslug barrelrange silencer xmags", ""),
    gh("maul", "Maul", CQB, P, "reflex eotech grip ammoslug barrelrange silencer xmags", ""),
    gh("uts15", "Bulldog", CQB, P, "reflex eotech grip barrelrange silencer xmags", ""),
    gh("p226", "P226", PISTOL, S, "reflex eotech akimbo tactical barrelrange flashsuppress silencer xmags fmj", ""),
    gh("m9a1", "M9A1", PISTOL, S, "reflex eotech xmags fmj", ""),
    gh("mp443", "MP-443 Grach", PISTOL, S, "reflex eotech xmags fmj", "weapon_mp443_ui_360|hud_icon_mp443_ui"),
    gh("magnum", ".44 Magnum", PISTOL, S, "reflex eotech acog barrelrange flashsuppress silencer xmags fmj", ""),
    gh("pdw", "PDW", PISTOL, S, "reflex eotech acog tactical xmags fmj", ""),
    gh("panzerfaust3", "Panzerfaust", LAUNCHER, S, "", ""),
    gh("mk32", "MK32", LAUNCHER, S, "", ""),
    gh("maaws", "MAAWS", LAUNCHER, S, "", "hud_icon_maaws"),
    gh("rgm", "Kastet", LAUNCHER, S, "", ""),
];

/// The first statstable index of Ghosts' guns (fixed, as MW3's).
pub const GHOSTS_FIRST_INDEX: i32 = 800;

const fn w3(
    gun: &'static str,
    name: &'static str,
    group: &'static str,
    slot: &'static str,
    attachments: &'static str,
    icon: &'static str,
) -> GuestGun {
    GuestGun { game: "iw5", gun, name, group, slot, attachments, icon }
}

const MW3_AR: &str = "reflex eotech acog hybrid thermal heartbeat gl shotgun silencer xmags";
const MW3_AR_ROF: &str = "reflex eotech acog hybrid thermal heartbeat gl shotgun silencer rof xmags";
const MW3_SMG: &str = "reflex eotech acog hamrhybrid thermal rof silencer xmags";
const MW3_LMG: &str = "reflex eotech acog thermal grip rof silencer xmags";
const MW3_LMG_HB: &str = "reflex eotech acog thermal heartbeat grip rof silencer xmags";
const MW3_SNIPER: &str = "acog vzscope thermal heartbeat silencer xmags";
const MW3_SHOTGUN: &str = "reflex eotech grip silencer xmags";
const MW3_MP: &str = "reflex eotech akimbo silencer xmags";
const MW3_PISTOL: &str = "tactical akimbo silencer xmags";

/// bo2mp: Modern Warfare 3's multiplayer guns, by their names in its
/// common_mp (`iw5_acr`; Create-a-Class and the scripts call it `mw3_acr`).
/// MW3 builds a variant from attachment configurations; the attachments
/// are its statstable's words for the gun (per-class sights and
/// suppressors are one word). Its statstable rows are numbered from
/// `MW3_FIRST_INDEX`, apart from the rosters listed before it.
pub const MW3: &[GuestGun] = &[
    w3("iw5_m4", "M4A1", AR, P, MW3_AR, "weapon_m4_short"),
    w3("iw5_m16", "M16A4", AR, P, MW3_AR_ROF, "weapon_m16a4"),
    w3("iw5_scar", "SCAR-L", AR, P, MW3_AR, "weapon_scar_h"),
    w3("iw5_cm901", "CM901", AR, P, MW3_AR, "weapon_cm901"),
    w3("iw5_type95", "Type 95", AR, P, MW3_AR_ROF, "weapon_type95"),
    w3("iw5_g36c", "G36C", AR, P, MW3_AR, "weapon_g36"),
    w3("iw5_acr", "ACR 6.8", AR, P, MW3_AR, "weapon_acr"),
    w3("iw5_mk14", "MK14", AR, P, MW3_AR_ROF, "weapon_mk14"),
    w3("iw5_ak47", "AK-47", AR, P, MW3_AR, "weapon_ak47"),
    w3("iw5_fad", "FAD", AR, P, MW3_AR, "weapon_fad"),
    w3("iw5_mp5", "MP5", SMG, P, MW3_SMG, "weapon_mp5k"),
    w3("iw5_ump45", "UMP45", SMG, P, MW3_SMG, "weapon_ump45"),
    w3("iw5_pp90m1", "PP90M1", SMG, P, MW3_SMG, "weapon_pp90m1"),
    w3("iw5_p90", "P90", SMG, P, MW3_SMG, "weapon_p90"),
    w3("iw5_m9", "PM-9", SMG, P, MW3_SMG, "weapon_mini_uzi"),
    w3("iw5_mp7", "MP7", SMG, P, MW3_SMG, "weapon_mp7"),
    w3("iw5_sa80", "L86 LSW", LMG, P, MW3_LMG_HB, "weapon_sa80"),
    w3("iw5_mg36", "MG36", LMG, P, MW3_LMG_HB, "weapon_mg36"),
    w3("iw5_pecheneg", "PKP Pecheneg", LMG, P, MW3_LMG, "weapon_pecheneg"),
    w3("iw5_mk46", "MK46", LMG, P, MW3_LMG_HB, "weapon_mk46"),
    w3("iw5_m60", "M60E4", LMG, P, MW3_LMG, "weapon_m60e4"),
    w3("iw5_barrett", "Barrett .50cal", SNIPER, P, MW3_SNIPER, "weapon_barrett"),
    w3("iw5_l96a1", "L118A", SNIPER, P, MW3_SNIPER, "weapon_l96a1"),
    w3("iw5_dragunov", "Dragunov", SNIPER, P, MW3_SNIPER, "weapon_dragunov"),
    w3("iw5_as50", "AS50", SNIPER, P, MW3_SNIPER, "weapon_as50"),
    w3("iw5_rsass", "RSASS", SNIPER, P, MW3_SNIPER, "weapon_rsass"),
    w3("iw5_msr", "MSR", SNIPER, P, MW3_SNIPER, "weapon_msr"),
    w3("iw5_usas12", "USAS 12", CQB, P, MW3_SHOTGUN, "weapon_usas12"),
    w3("iw5_ksg", "KSG 12", CQB, P, MW3_SHOTGUN, "weapon_ksg"),
    w3("iw5_spas12", "SPAS-12", CQB, P, MW3_SHOTGUN, "weapon_spas12"),
    w3("iw5_aa12", "AA-12", CQB, P, MW3_SHOTGUN, "weapon_aa12"),
    w3("iw5_striker", "Striker", CQB, P, MW3_SHOTGUN, "weapon_striker"),
    w3("iw5_1887", "Model 1887", CQB, P, "", "weapon_model1887"),
    w3("iw5_fmg9", "FMG9", PISTOL, S, MW3_MP, "weapon_fmg9"),
    w3("iw5_g18", "G18", PISTOL, S, MW3_MP, "weapon_glock"),
    w3("iw5_mp9", "MP9", PISTOL, S, MW3_MP, "weapon_mp9"),
    w3("iw5_skorpion", "Skorpion", PISTOL, S, MW3_MP, "weapon_skorpion"),
    w3("iw5_usp45", "USP .45", PISTOL, S, MW3_PISTOL, "weapon_usp_45"),
    w3("iw5_p99", "P99", PISTOL, S, MW3_PISTOL, "weapon_p99"),
    w3("iw5_mp412", "MP412", PISTOL, S, "tactical akimbo", "weapon_mp412"),
    w3("iw5_44magnum", ".44 Magnum", PISTOL, S, "tactical akimbo", "weapon_magnum"),
    w3("iw5_fnfiveseven", "Five Seven", PISTOL, S, MW3_PISTOL, "weapon_fnfiveseven"),
    w3("iw5_deserteagle", "Desert Eagle", PISTOL, S, "tactical akimbo", "weapon_desert_eagle"),
    w3("iw5_smaw", "SMAW", LAUNCHER, S, "", "weapon_smaw"),
    w3("stinger", "Stinger", LAUNCHER, S, "", "weapon_stinger"),
    w3("xm25", "XM25", LAUNCHER, S, "", "weapon_xm25"),
    w3("m320", "M320 GLM", LAUNCHER, S, "", "weapon_m320"),
    w3("rpg", "RPG-7", LAUNCHER, S, "", "weapon_rpg7"),
];

/// The first statstable index of MW3's guns: a fixed number, not their
/// place in `all()`, so rosters listed before them can grow without moving
/// a saved class's MW3 gun.
pub const MW3_FIRST_INDEX: i32 = 500;

const fn t4(
    gun: &'static str,
    name: &'static str,
    group: &'static str,
    slot: &'static str,
    attachments: &'static str,
    icon: &'static str,
) -> GuestGun {
    GuestGun { game: "t4", gun, name, group, slot, attachments, icon }
}

const WAW_SMG: &str = "aperture extclip silencer";
const WAW_BOLT: &str = "scoped bayonet gl";

/// bo2mp: World at War's multiplayer guns (attachments as its variants in
/// common_mp carry them, `silenced` and `bigammo` as BO2's `silencer` and
/// `extclip`; the LMGs' bipods are deployed modes, not listed). Its
/// statstable rows are numbered from `WAW_FIRST_INDEX`.
pub const WAW: &[GuestGun] = &[
    t4("stg44", "STG-44", AR, P, "aperture telescopic flash", "hud_icon_stg44"),
    t4("svt40", "SVT-40", AR, P, "aperture telescopic flash", "hud_icon_svt40"),
    t4("gewehr43", "Gewehr 43", AR, P, "aperture telescopic gl silencer", "hud_icon_gewehr43"),
    t4("m1garand", "M1 Garand", AR, P, "scoped flash bayonet gl", "hud_icon_m1garand"),
    t4("m1carbine", "M1A1 Carbine", AR, P, "aperture flash bayonet extclip", "hud_icon_m1carbine"),
    t4("thompson", "Thompson", SMG, P, WAW_SMG, "hud_icon_thompson"),
    t4("mp40", "MP40", SMG, P, WAW_SMG, "hud_icon_mp40"),
    t4("type100smg", "Type 100", SMG, P, WAW_SMG, "hud_icon_type_100_smg"),
    t4("ppsh", "PPSh-41", SMG, P, "aperture extclip", "hud_icon_ppsh"),
    t4("bar", "BAR", LMG, P, "", "hud_icon_bar"),
    t4("fg42", "FG42", LMG, P, "telescopic", "hud_icon_fg42"),
    t4("type99lmg", "Type 99 LMG", LMG, P, "bayonet", "hud_icon_type_99_lmg"),
    t4("dp28", "DP-28", LMG, P, "", "hud_icon_dp28"),
    t4("mg42", "MG42", LMG, P, "", "hud_icon_mg42"),
    t4("30cal", "Browning M1919", LMG, P, "", "hud_icon_30cal"),
    t4("springfield", "Springfield", SNIPER, P, WAW_BOLT, "hud_icon_springfield_scoped"),
    t4("kar98k", "Kar98k", SNIPER, P, WAW_BOLT, "hud_icon_kar98k"),
    t4("mosinrifle", "Mosin-Nagant", SNIPER, P, WAW_BOLT, "hud_icon_mosin_rifle"),
    t4("type99rifle", "Arisaka", SNIPER, P, WAW_BOLT, "hud_icon_type_99_rifle"),
    t4("ptrs41", "PTRS-41", SNIPER, P, "", "hud_icon_ptrs41"),
    t4("shotgun", "M1897 Trench Gun", CQB, P, "bayonet grip", "hud_icon_shotgun"),
    t4("doublebarreledshotgun", "Double-Barreled Shotgun", CQB, P, "sawoff grip", "hud_icon_double_barrel"),
    t4("colt", "M1911 .45", PISTOL, S, "", "hud_icon_colt"),
    t4("nambu", "Nambu", PISTOL, S, "", "hud_icon_nambu"),
    t4("tokarev", "Tokarev TT-33", PISTOL, S, "", "hud_icon_tokarev"),
    t4("walther", "Walther P38", PISTOL, S, "", "hud_icon_walther"),
    t4("357magnum", ".357 Magnum", PISTOL, S, "", "hud_icon_357"),
    t4("bazooka", "M9A1 Bazooka", LAUNCHER, S, "", "hud_icon_bazooka"),
];

/// The first statstable index of World at War's guns (see
/// `MW3_FIRST_INDEX`).
pub const WAW_FIRST_INDEX: i32 = 600;

const fn c4(
    gun: &'static str,
    name: &'static str,
    group: &'static str,
    slot: &'static str,
    attachments: &'static str,
    icon: &'static str,
) -> GuestGun {
    GuestGun { game: "iw3", gun, name, group, slot, attachments, icon }
}

const COD4_AR: &str = "reflex acog gl silencer";
const COD4_SMG: &str = "reflex acog silencer";
const COD4_LMG: &str = "reflex acog grip";

/// bo2mp: Call of Duty 4's multiplayer guns (attachments as its variants
/// in common_mp carry them). Its statstable rows are numbered from
/// `COD4_FIRST_INDEX`.
pub const COD4: &[GuestGun] = &[
    c4("m16", "M16A4", AR, P, COD4_AR, "hud_icon_m16a4_grenade"),
    c4("ak47", "AK-47", AR, P, COD4_AR, "hud_icon_ak47"),
    c4("m4", "M4 Carbine", AR, P, COD4_AR, "hud_icon_m4carbine"),
    c4("g3", "G3", AR, P, COD4_AR, "hud_icon_g3"),
    c4("g36c", "G36C", AR, P, COD4_AR, "hud_icon_g36c_mp"),
    c4("m14", "M14", AR, P, COD4_AR, "hud_icon_m14"),
    c4("mp44", "MP44", AR, P, "", "hud_icon_mp44"),
    c4("mp5", "MP5", SMG, P, COD4_SMG, "hud_icon_mp5"),
    c4("skorpion", "Skorpion", SMG, P, COD4_SMG, "hud_icon_skorpian"),
    c4("uzi", "Mini-Uzi", SMG, P, COD4_SMG, "hud_icon_mini_uzi"),
    c4("ak74u", "AK-74u", SMG, P, COD4_SMG, "hud_icon_ak74u"),
    c4("p90", "P90", SMG, P, COD4_SMG, "hud_icon_p90"),
    c4("saw", "M249 SAW", LMG, P, COD4_LMG, "hud_icon_m249saw"),
    c4("rpd", "RPD", LMG, P, COD4_LMG, "hud_icon_rpd"),
    c4("m60e4", "M60E4", LMG, P, COD4_LMG, "hud_icon_m60e4"),
    c4("m40a3", "M40A3", SNIPER, P, "acog", "hud_icon_m40a3"),
    c4("m21", "M21", SNIPER, P, "acog", "hud_icon_m14_scoped"),
    c4("dragunov", "Dragunov", SNIPER, P, "acog", "hud_icon_dragunov"),
    c4("remington700", "R700", SNIPER, P, "acog", "hud_icon_remington700"),
    c4("barrett", "Barrett .50cal", SNIPER, P, "acog", "hud_icon_barrett50cal"),
    c4("winchester1200", "W1200", CQB, P, "reflex grip", "hud_icon_winchester_1200"),
    c4("m1014", "M1014", CQB, P, "reflex grip", "hud_icon_benelli_m4"),
    c4("beretta", "M9", PISTOL, S, "silencer", "hud_icon_m9beretta"),
    c4("usp", "USP .45", PISTOL, S, "silencer", "hud_icon_usp_45"),
    c4("colt45", "M1911 .45", PISTOL, S, "silencer", "hud_icon_colt_45"),
    c4("deserteagle", "Desert Eagle", PISTOL, S, "", "hud_icon_desert_eagle"),
    c4("rpg", "RPG-7", LAUNCHER, S, "", "hud_icon_rpg"),
];

/// The first statstable index of Call of Duty 4's guns (see
/// `MW3_FIRST_INDEX`).
pub const COD4_FIRST_INDEX: i32 = 700;

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
    // (Modern Warfare 3's.)
    ("hybrid", "top", "Hybrid Sight", "cac_mods_holographic"),
    ("hamrhybrid", "top", "HAMR Scope", "cac_mods_holographic"),
    ("vzscope", "top", "Variable Zoom", "cac_mods_var_zoom"),
    // (World at War's.)
    ("aperture", "top", "Aperture Sight", "cac_mods_ironsight"),
    ("telescopic", "top", "Telescopic Sight", "cac_mods_var_zoom"),
    ("scoped", "top", "Sniper Scope", "cac_mods_var_zoom"),
    ("bayonet", "bottom", "Bayonet", "cac_mods_tact_knife"),
    ("flash", "muzzle", "Flash Hider", "cac_mods_barrel_extend"),
    ("sawoff", "muzzle", "Sawed-Off", "cac_mods_barrel_extend"),
    // (Ghosts'.)
    ("tracker", "top", "Tracker Sight", "iw6:scope_targetfinder_360"),
    ("ironsight", "top", "Iron Sights", "iw6:iron_sight_360"),
    ("barrelrange", "muzzle", "Long Barrel", "cac_mods_barrel_extend"),
    ("barrelbored", "muzzle", "Bored Barrel", "cac_mods_barrel_extend"),
    ("flashsuppress", "muzzle", "Muzzle Brake", "iw6:muzzle_break_360"),
    ("firetypeburst", "trigger", "Burst Fire", "iw6:burst_fire_360"),
    ("firetypesingle", "trigger", "Single Fire", "iw6:single_fire_360"),
    ("firetypeauto", "trigger", "Full Auto", "iw6:full_auto_360"),
    ("ammoslug", "trigger", "Slug Rounds", "iw6:slug_round_360"),
];

/// The prefix a game's guns carry (`t5` -> `bo1_`).
#[must_use]
pub fn prefix(game: &str) -> &'static str {
    match game {
        "t5" => "bo1_",
        "iw4" => "mw2_",
        "iw5" => "mw3_",
        "iw6" => "gh_",
        "t4" => "waw_",
        "iw3" => "cod4_",
        _ => "",
    }
}

const GAMES: [&str; 6] = ["t5", "iw4", "iw5", "iw6", "t4", "iw3"];
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
    BO1.iter().chain(MW2.iter()).chain(GHOSTS.iter()).chain(MW3.iter()).chain(WAW.iter()).chain(COD4.iter())
}

/// The statstable index of the guest gun at `position` in `all()`: a
/// roster with its own first index (MW3's) numbers from it by its place in
/// that roster; the others by their place in `all()` from `FIRST_INDEX`.
fn stats_index(position: usize, gun: &GuestGun) -> i32 {
    let own = [(MW3, MW3_FIRST_INDEX), (WAW, WAW_FIRST_INDEX), (COD4, COD4_FIRST_INDEX), (GHOSTS, GHOSTS_FIRST_INDEX)];
    for (roster, first) in own {
        if let Some(k) = roster.iter().position(|g| g.game == gun.game && g.gun == gun.gun) {
            return first + k as i32;
        }
    }
    FIRST_INDEX + position as i32
}

impl GuestGun {
    /// The statstable reference (`bo1_ak74u`; MW3's `iw5_acr` is `mw3_acr`).
    #[must_use]
    pub fn reference(&self) -> String {
        let gun = if self.game == "iw5" { self.gun.strip_prefix("iw5_").unwrap_or(self.gun) } else { self.gun };
        format!("{}{}", prefix(self.game), gun)
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
            "t4" => "Call of Duty: World at War",
            "iw3" => "Call of Duty 4",
            _ => "another game",
        }
    }

    /// Create-a-Class pictures to try: the game's own menu picture, then
    /// its kill icon (`t5:menu_mp_weapons_ak74u|hud_icon_ak74u`).
    fn picture(&self) -> String {
        match self.game {
            // (MW2's Create-a-Class pictures are `weapon_<gun>`.)
            "iw4" => format!("iw4:weapon_{0}|hud_icon_{0}", self.gun),
            // (MW3's are its statstable's, `weapon_acr`.)
            "iw5" => format!("iw5:{}", self.icon),
            // (World at War's are Treyarch's `menu_mp_weapons_<gun>`,
            // CoD4's `weapon_<gun>`; else the kill icon.)
            "t4" => format!("t4:menu_mp_weapons_{}|{}", self.gun, self.icon),
            "iw3" => format!("iw3:weapon_{}|{}", self.gun, self.icon),
            // (Ghosts' are its ui_mp's `weapon_<gun>_360`, then the HUD icon.)
            "iw6" if self.icon.is_empty() => format!("iw6:weapon_{0}_360|hud_icon_{0}", self.gun),
            "iw6" => format!("iw6:{}", self.icon),
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
        row[0] = stats_index(k, gun).to_string();
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
        // (And the rosters numbered from their own first index.)
        for game in ["iw5", "t4", "iw3", "iw6"] {
            set_available(game, true);
        }
        assert!(extend_stats_rows(&mut rows));
        let refs: Vec<&String> = rows.iter().map(|r| &r[4]).collect();
        assert!(refs.iter().any(|r| *r == "mw2_m4"));
        assert!(refs.iter().any(|r| *r == "mw3_acr"));
        assert_eq!(rows.iter().find(|r| r[4] == "mw3_m4").map(|r| r[0].as_str()), Some("500"));
        assert_eq!(rows.iter().find(|r| r[4] == "waw_stg44").map(|r| r[0].as_str()), Some("600"));
        assert_eq!(rows.iter().find(|r| r[4] == "cod4_m16").map(|r| r[0].as_str()), Some("700"));
        assert_eq!(rows.iter().find(|r| r[4] == "gh_ak12").map(|r| r[0].as_str()), Some("800"));
        assert!(rows.iter().find(|r| r[4] == "gh_k7").is_some_and(|r| r[6] == "iw6:weapon_k7_ui_360|hud_icon_k7_ui"));
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
