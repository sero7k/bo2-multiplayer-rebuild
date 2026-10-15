//! bo2mp: Resurgence on Black Ops II's free-for-all (`bo2mp/resurgence`).
//!
//! The rules are `gamemode_iw4::resurgence`'s table; the match script reads
//! them as `scr_br_*` dvars written here, so the numbers have one home.
//! A sound is played by its Modern Warfare III alias (`jup:<alias>`) once a
//! reader for that game's sounds says it has it, else by Black Ops II's
//! stand-in, else not at all.

use std::sync::Mutex;

use gamemode_iw4::resurgence::{self, Rules};

/// The script (`bo2mp/resurgence`).
pub(super) const SCRIPT: &str = include_str!("scripts/br_resurgence.gsc");

static SETUP: Mutex<Option<Rules>> = Mutex::new(None);

/// Whether Modern Warfare III's sounds hold an alias (set by its audio reader).
static JUP_SOUND: Mutex<Option<fn(&str) -> bool>> = Mutex::new(None);

/// bo2mp: the next match is Resurgence with squads of `squad_size` (`None`:
/// a normal match). Read when the match starts.
pub fn set_resurgence(squad_size: Option<u8>) {
    let rules = squad_size.map(|n| resurgence::REBIRTH.with_squad_size(n));
    if let Ok(mut s) = SETUP.lock() {
        *s = rules;
    }
}

pub fn resurgence_on() -> bool {
    SETUP.lock().is_ok_and(|s| s.is_some())
}

/// bo2mp: how to ask Modern Warfare III's sounds whether an alias is there.
pub fn set_jup_sound_lookup(lookup: Option<fn(&str) -> bool>) {
    if let Ok(mut s) = JUP_SOUND.lock() {
        *s = lookup;
    }
}

pub(super) fn rules() -> Option<Rules> {
    if let Ok(v) = std::env::var("BO2MP_BR") {
        let n = v
            .trim()
            .parse::<u8>()
            .unwrap_or(resurgence::REBIRTH.squad_size);
        return (n > 0).then(|| resurgence::REBIRTH.with_squad_size(n));
    }
    SETUP.lock().ok().and_then(|s| *s)
}

fn jup_has(alias: &str) -> bool {
    !alias.is_empty()
        && JUP_SOUND
            .lock()
            .ok()
            .and_then(|f| *f)
            .is_some_and(|f| f(alias))
}

/// The alias the script plays for a sound route ("" for none).
pub(super) fn sound_alias(route: &resurgence::SoundRoute) -> String {
    if jup_has(route.jup) {
        format!("jup:{}", route.jup)
    } else {
        route.t6.to_owned()
    }
}

/// The game type settings Resurgence holds whatever the free-for-all's
/// files say: no clock or score to end it, one life (the script grants the
/// redeploys), no final killcam.
pub(super) const SETTINGS: &[(&str, &str)] = &[
    ("timelimit", "0"),
    ("scorelimit", "0"),
    ("playernumlives", "1"),
    ("allowfinalkillcam", "0"),
    ("playerrespawndelay", "0"),
    ("waverespawndelay", "0"),
];

pub(super) fn setting(name: &str) -> Option<&'static str> {
    rules()?;
    SETTINGS
        .iter()
        .find(|(k, _)| k.eq_ignore_ascii_case(name))
        .map(|(_, v)| *v)
}

/// The `scr_br_*` dvars for `rules`.
pub(super) fn dvars(rules: &Rules) -> Vec<(String, String)> {
    let mut out: Vec<(String, String)> = Vec::new();
    let mut put = |k: &str, v: String| out.push((format!("scr_br{k}"), v));
    put("", "1".into());
    put("_squad", rules.squad_size.to_string());
    put("_infil_ms", (u32::from(rules.infil_s) * 1000).to_string());
    put("_health", rules.health.to_string());
    put("_plate_hp", rules.plate_hp.to_string());
    put("_plate_slots", rules.plate_slots.to_string());
    put("_plates_max", rules.plates_carried_max.to_string());
    put("_plate_ms", rules.plate_insert_ms.to_string());
    put("_spawn_plates", rules.spawn_plates.to_string());
    put("_start_cash", rules.start_cash.to_string());
    put("_kill_cash", rules.kill_cash.to_string());
    put(
        "_kill_cut_ms",
        (u32::from(rules.kill_redeploy_cut_s) * 1000).to_string(),
    );
    put(
        "_stacks",
        rules
            .cash_stacks
            .iter()
            .map(u32::to_string)
            .collect::<Vec<_>>()
            .join(" "),
    );
    put("_stations", rules.buy_station_count.to_string());
    put("_uav_ms", (u32::from(rules.uav_s) * 1000).to_string());
    put("_loot_spots", rules.loot_spots.to_string());
    put("_phases", rules.phases.len().to_string());
    for (w, p) in rules.windows().zip(rules.phases) {
        put(
            &format!("_phase_{}", w.index),
            format!(
                "{} {} {} {} {} {}",
                w.wait_from_ms,
                w.close_from_ms,
                w.closed_ms,
                p.end_radius,
                p.gas_dps,
                p.redeploy_s.map_or(-1, i32::from) * 1000
            ),
        );
    }
    put("_buys", rules.buy.len().to_string());
    for (i, b) in rules.buy.iter().enumerate() {
        put(
            &format!("_buy_{i}"),
            format!("{}|{}|{}", b.key, b.label, b.price),
        );
    }
    put("_tiers", rules.loot.len().to_string());
    for (i, t) in rules.loot.iter().enumerate() {
        put(
            &format!("_tier_{i}"),
            format!("{}|{}|{}|{}", t.key, t.label, t.weight, t.attachments),
        );
    }
    for r in resurgence::SOUNDS {
        put(&format!("_snd_{}", r.key), sound_alias(r));
    }
    out
}

#[cfg(test)]
mod tests {
    #[test]
    fn the_resurgence_script_compiles_with_its_parts() {
        let s =
            gsc_t6::compile("bo2mp/resurgence", super::SCRIPT).expect("br_resurgence.gsc compiles");
        for f in [
            "main",
            "br_onplayerdamage",
            "br_onplayerkilled",
            "br_circle",
            "br_redeploy",
            "br_buy_station",
            "br_bot_think",
            "br_check_squads",
        ] {
            assert!(s.functions.iter().any(|x| x.name == f), "{f}");
        }
    }

    #[test]
    fn every_phase_and_sound_reaches_the_script() {
        let r = gamemode_iw4::resurgence::REBIRTH;
        let d = super::dvars(&r);
        let get = |k: &str| d.iter().find(|(n, _)| n == k).map(|(_, v)| v.clone());
        assert_eq!(get("scr_br_phases"), Some(r.phases.len().to_string()));
        assert!(get(&format!("scr_br_phase_{}", r.phases.len() - 1)).is_some());
        assert_eq!(get("scr_br_snd_victory").as_deref(), Some("mus_victory"));
        let last = get("scr_br_phase_4").unwrap();
        assert!(last.ends_with(" -1000"), "{last}");
    }
}
