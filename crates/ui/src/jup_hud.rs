//! bo2mp: Modern Warfare III's (2023) Warzone HUD pictures, read from the
//! owner's install at runtime for the Warzone HUD (`hud_warzone`). Nothing
//! of the game is copied or kept on disk.
//!
//! The pictures' names are hashes in the game's zones (its UI scripts that
//! name and place them are compiled and are not read): each one here was
//! picked by eye from contact sheets of the in-game zones' pictures
//! (`fastfile_jup` example `jup_hud_pics`) and is kept by its zone and name
//! hash. They are white (or grey) glyphs drawn tinted in the Warzone HUD's
//! colours (`shell::wz_theme::HUD`); one the install lacks is drawn as a
//! plain shape instead.
//!
//! The install is the emblems' (`jup_cards::install`): `$BO2_JUP_INSTALL`,
//! else the game folders file's `jup` line.

use std::collections::HashMap;
use std::sync::OnceLock;

use fastfile_jup::map::materials::{MapImage, Packages};

/// The zones the pictures are in.
const ZONES: &[&str] = &["ingame", "ingame_br", "ingame_shared_mp", "ingame_base_mp"];

/// The largest side a picture is decoded at.
const MAX_SIDE: u32 = 256;

/// A HUD picture: our name for it, its zone (in `ZONES`) and its name
/// hash there. (What each one shows, as seen on the contact sheets.)
pub(crate) const PICTURES: &[(&str, &str, u64)] = &[
    // Plates, health, armor.
    ("plate", "ingame_shared_mp", 0x7152_b586_6253_24d7),           // a ballistic plate, flat white
    ("plate_carrier", "ingame_br", 0x6ee9_d5b2_3804_6e98),          // a vest in a hexagon
    ("plate_bar", "ingame_base_mp", 0x0924_1024_ad50_aba2),         // three bar segments
    ("armor_break", "ingame_base_mp", 0x0606_294e_7363_0ca3),       // a plate breaking
    ("armor_hit_glow", "ingame_base_mp", 0x3cf4_6547_8f40_8543),    // a soft glow
    ("self_revive", "ingame_br", 0x68bd_d0ba_9aa6_8cbf),            // a medic cross in a hexagon
    ("revive", "ingame_base_mp", 0x148c_50fa_6ab2_213e),            // a heart in a map pin
    // Cash and buying.
    ("cash", "ingame_shared_mp", 0x4891_1602_9e8c_451b),            // "$"
    ("cash_chrome", "ingame_br", 0x7cd4_c6a4_8c93_bde3),            // a chrome "$"
    ("money_bag", "ingame_shared_mp", 0x176e_9c4c_108f_b360),
    ("buy_station", "ingame", 0x3f78_9aac_2253_591c),               // "$" on a screen
    ("buy_cart", "ingame_br", 0x44e3_b151_7243_a342),
    ("loadout_drop", "ingame_br", 0x56cc_82a0_51ed_e86c),           // parachutes
    ("loadout_crate", "ingame", 0x3d31_9c51_c3e3_80ce),             // "$" and guns in a box
    ("uav", "ingame_shared_mp", 0x301b_2945_faa4_1b06),             // a dish
    // The squad.
    ("squad", "ingame_br", 0x22cb_b629_d6fb_24b4),                  // three people
    ("players", "ingame_base_mp", 0x411f_f6de_971b_3e50),           // one person in a circle
    ("downed", "ingame", 0x2ed0_0f76_22f8_1893),                    // a man lying down
    ("skull", "ingame_shared_mp", 0x4162_5dbb_9df4_d954),
    ("eliminated", "ingame_br", 0x35a6_bde2_cd52_66d2),             // skull and scythes
    ("redeploy", "ingame_shared_mp", 0x51bf_5736_3efe_3898),        // an arrow up out of a star
    ("redeploy_shield", "ingame_br", 0x43c3_48a9_0478_d125),
    ("kill_marker", "ingame_shared_mp", 0x5207_c773_b427_5cdb),     // a skull over a chevron
    // The circle and the gas.
    ("gas", "ingame_br", 0x6345_f610_3e5e_2c63),                    // a skull in a hazard sign
    ("gas_mask", "ingame_br", 0x7d19_a6d6_24a9_4b32),
    ("circle_timer", "ingame_base_mp", 0x1788_9934_64ad_41fb),      // a ring in segments
    ("ring", "ingame_shared_mp", 0x55b7_11df_3ff1_a4bf),
    ("arc", "ingame_shared_mp", 0x293a_1291_f471_6355),             // a quarter ring
    ("clock_wipe", "ingame_br", 0x4a90_9c8d_ceab_3438),
    ("dashed_ring", "ingame_br", 0x028a_f3c5_1bba_7daa),
    ("plane", "ingame_shared_mp", 0x3458_941e_ccdc_3a54),           // the infil plane
    // Contracts.
    ("contract", "ingame", 0x163a_1a0e_2d6d_8e2a),                  // the tablet
    ("contract_bounty", "ingame_br", 0x1c2a_89f1_86f5_358d),        // crosshair on a shield
    ("contract_recon", "ingame_br", 0x3f26_4810_b0fd_f3dc),         // magnifier on a shield
    ("contract_supply", "ingame_br", 0x072c_e8f2_b3c8_d17c),        // stopwatch on a shield
    ("contract_mostwanted", "ingame_br", 0x6116_1b22_0587_80bf),    // crown on a shield
    ("contract_scavenger", "ingame", 0x7c30_1f12_4202_839b),        // a "?" box
    // Pings, markers, hits.
    ("ping", "ingame_br", 0x3f9d_0c15_5cb1_2227),                   // a white flag on a ring
    ("ping_danger", "ingame_shared_mp", 0x3039_9ec5_df0c_22ed),     // a red diamond
    ("player_arrow", "ingame_br", 0x3242_ca65_a1a9_773f),
    ("hitmarker", "ingame_base_mp", 0x3429_ec5c_987c_3664),         // an X with ends
    ("hitmarker_kill", "ingame_shared_mp", 0x23ba_5104_1bf9_db30),  // four corner strokes
    // Ammo.
    ("ammo_rifle", "ingame_shared_mp", 0x01b4_9bdc_91cc_aab9),      // four rounds
    ("ammo_shells", "ingame_shared_mp", 0x1262_5fee_24b4_8bfd),
    ("ammo_pistol", "ingame_shared_mp", 0x2bb6_c1d6_4113_7f88),
    ("ammo_sniper", "ingame_shared_mp", 0x75aa_3799_54b6_b1a4),     // one round
];

/// Every picture, decoded once (`None`: no install, or not there).
fn decoded() -> &'static HashMap<&'static str, (u32, u32, Vec<u8>)> {
    static ALL: OnceLock<HashMap<&'static str, (u32, u32, Vec<u8>)>> = OnceLock::new();
    ALL.get_or_init(|| {
        let t = std::time::Instant::now();
        let out = read_all().unwrap_or_default();
        diag::info!(
            Ui,
            "bo2mp Warzone HUD: {} of {} Modern Warfare III pictures read ({:?})",
            out.len(),
            PICTURES.len(),
            t.elapsed()
        );
        out
    })
}

fn read_all() -> Option<HashMap<&'static str, (u32, u32, Vec<u8>)>> {
    let install = crate::jup_cards::install()?;
    let oodle = match fastfile_jup::oodle::load(&install) {
        Ok(o) => o,
        Err(e) => {
            diag::warn!(Ui, "bo2mp Warzone HUD: no Oodle ({e}); simple shapes stand in");
            return None;
        }
    };
    let packages = Packages::open(&install);
    let mut out = HashMap::new();
    for zone_name in ZONES {
        let wanted: Vec<&(&str, &str, u64)> = PICTURES.iter().filter(|p| p.1 == *zone_name).collect();
        if wanted.is_empty() {
            continue;
        }
        let path = install.join("zone").join(format!("{zone_name}.ff"));
        let zone = match fastfile_jup::read_zone(&path, oodle.as_ref()) {
            Ok((_, z)) => z,
            Err(e) => {
                diag::warn!(Ui, "bo2mp Warzone HUD: {zone_name}.ff not read: {e}");
                continue;
            }
        };
        let index = fastfile_jup::zone::HashIndex::build(&zone, wanted.iter().map(|p| p.2), 8);
        for &&(name, _, hash) in &wanted {
            let rec = index.plain(hash).iter().find_map(|&at| match fastfile_jup::image::parse(&zone, at) {
                Ok(Some(r)) if r.hash == hash && r.depth == 1 && r.array == 1 => Some(r),
                _ => None,
            });
            let Some(rec) = rec else {
                diag::warn!(Ui, "bo2mp Warzone HUD: `{name}` ({hash:016x}) is not in {zone_name}");
                continue;
            };
            let img = MapImage {
                hash: rec.hash,
                width: rec.width,
                height: rec.height,
                format: rec.format,
                groups: rec.groups.clone(),
                inline: rec.inline.map(|(o, n)| zone[o..o + n].to_vec()),
            };
            let pic = packages.level(&img, MAX_SIDE, oodle.as_ref()).ok().and_then(|l| {
                let px = fastfile_jup::cards::decode_bcn(l.dxgi, l.width, l.height, &l.data)?;
                Some((l.width, l.height, px))
            });
            match pic {
                Some(p) => {
                    out.insert(name, p);
                }
                None => diag::warn!(Ui, "bo2mp Warzone HUD: `{name}`'s pixels are not in the install"),
            }
        }
    }
    Some(out)
}

/// A picture by our name, as RGBA. Reads the install the first time: call
/// it off the main thread.
pub(crate) fn read_picture(name: &str) -> Option<(u32, u32, Vec<u8>)> {
    decoded().get(name).cloned()
}

#[cfg(test)]
mod tests {
    #[test]
    fn names_are_unique_and_zones_known() {
        let mut seen = std::collections::HashSet::new();
        for (name, zone, hash) in super::PICTURES {
            assert!(seen.insert(*name), "{name} twice");
            assert!(super::ZONES.contains(zone), "{name}: {zone}");
            assert_eq!(hash >> 63, 0, "{name}: a name hash has its top bit clear");
        }
    }
}
