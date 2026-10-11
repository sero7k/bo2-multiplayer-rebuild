use std::collections::HashMap;

use asset_core::AssetNamespace;
use bevy::prelude::Resource;

pub(crate) fn voice_prefixes_for_zone(
    catalog: Option<&asset_game::MenuCatalog>,
    bank: Option<&crate::SoundBank>,
    identity: &frame::LaunchIdentity,
) -> (Option<String>, Option<String>) {
    let Some(catalog) = catalog else {
        return (None, None);
    };
    let Some(table) = catalog.string_table(gamemode_iw4::FACTION_TABLE) else {
        return (None, None);
    };
    let (allies_cs, axis_cs) = arena_faction_charsets(catalog, bank, identity);
    let allies = table.lookup_col(&allies_cs, gamemode_iw4::FACTION_VOICE_PREFIX_COL);
    let axis = table.lookup_col(&axis_cs, gamemode_iw4::FACTION_VOICE_PREFIX_COL);
    (
        (!allies.is_empty()).then(|| allies.to_owned()),
        (!axis.is_empty()).then(|| axis.to_owned()),
    )
}

fn arena_faction_charsets(
    catalog: &asset_game::MenuCatalog,
    bank: Option<&crate::SoundBank>,
    identity: &frame::LaunchIdentity,
) -> (String, String) {
    let from_ui = catalog.rawfile_text("mp/basemaps.arena").map(str::to_owned);
    let from_bank = bank.and_then(|b| b.0.rawfile_text("mp/basemaps.arena").map(str::to_owned));
    let arena_text = from_ui
        .or(from_bank)
        .or_else(|| asset_game::read_basemaps_arena(&identity.games_root));
    let row = arena_text
        .as_deref()
        .and_then(|text| asset_game::arena_charsets(text, &identity.zone));
    let (allies, axis) = row.map_or((None, None), |row| (row.allieschar, row.axischar));
    (
        allies
            .filter(|s| !s.is_empty())
            .unwrap_or_else(|| gamemode_iw4::DEFAULT_ALLIES_CHARSET.to_owned()),
        axis.filter(|s| !s.is_empty())
            .unwrap_or_else(|| gamemode_iw4::DEFAULT_AXIS_CHARSET.to_owned()),
    )
}

pub(crate) fn team_voice_aliases(
    bank: &asset_audio::SoundCatalog,
    allies: Option<&str>,
    axis: Option<&str>,
) -> Vec<String> {
    let prefixes: Vec<&str> = [allies, axis]
        .into_iter()
        .flatten()
        .chain(["generic_death_"])
        .collect();
    let mut aliases: Vec<String> = (0..bank.sounds.len())
        .filter(|&index| bank.namespace_of_alias(index) == AssetNamespace::Iw4)
        .filter_map(|index| bank.name_at(index))
        .filter(|name| prefixes.iter().any(|prefix| name.starts_with(prefix)))
        .map(str::to_owned)
        .collect();
    for prefix in [allies, axis].into_iter().flatten() {
        for line in TEAM_MUSIC {
            let alias = format!("{prefix}{line}");
            if !aliases.contains(&alias) {
                aliases.push(alias);
            }
        }
    }
    aliases
}

/// bo2mp: every `game["voice"][team]` prefix Black Ops II's faction scripts
/// set (`MP/Factions/faction_*_mp/maps/mp/teams/_teamset_*.gsc:41`, and
/// `_teamset_multiteam.gsc`): seals, pmc, cd, fbi, isa, pla.
const T6_ANNOUNCER_PREFIXES: [&str; 6] = [
    "vox_st6_", "vox_pmc_", "vox_cda_", "vox_fbi_", "vox_isa_", "vox_pla_",
];

/// bo2mp: the announcer lines of the match's factions. BO2 plays
/// `game["voice"][team] + game["dialog"][line]` with `playlocalsound`
/// (`_globallogic_audio.gsc:523-527`), e.g. `vox_st6_timesup`. The map's
/// zones load only this match's two factions' language zones, so every
/// T6 alias with one of the six prefixes is one of theirs. They are not in
/// the weapon or map sets, so without this they were first decoded when the
/// script played them and missed the start deadline (a late line is dropped).
pub(crate) fn t6_announcer_aliases(bank: &asset_audio::SoundCatalog) -> Vec<String> {
    (0..bank.sounds.len())
        .filter(|&index| bank.namespace_of_alias(index) == AssetNamespace::T6)
        .filter_map(|index| bank.name_at(index))
        .filter(|name| {
            let lower = name.to_ascii_lowercase();
            T6_ANNOUNCER_PREFIXES
                .iter()
                .any(|prefix| lower.starts_with(prefix))
        })
        .map(str::to_owned)
        .collect()
}

/// Which announcer a Black Ops II match speaks with. BO1's is the default;
/// `BO2_ANNOUNCER=bo2` keeps Black Ops II's own.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum T6Announcer {
    Bo1,
    Bo2,
}

impl T6Announcer {
    pub(crate) fn from_env() -> Self {
        match std::env::var("BO2_ANNOUNCER") {
            Ok(value) if value.eq_ignore_ascii_case("bo2") => Self::Bo2,
            _ => Self::Bo1,
        }
    }
}

/// BO2 faction voice → the BO1 voice that replaces it. The side BO2 puts a
/// faction on picks the voice: the American and allied factions speak as
/// Black Ops, their opponents as Spetsnaz.
const T6_TO_T5_VOICE: [(&str, &str); 6] = [
    ("vox_st6_", "vox_ops_"),
    ("vox_fbi_", "vox_ops_"),
    ("vox_isa_", "vox_ops_"),
    ("vox_pmc_", "vox_rus_"),
    ("vox_cda_", "vox_rus_"),
    ("vox_pla_", "vox_rus_"),
];

/// BO2 lines with no line of the same name in BO1, and the BO1 line said in
/// their place. Lines whose names match play as themselves; a line in
/// neither keeps BO2's voice.
const T6_TO_T5_LINES: &[(&str, &str)] = &[
    ("koth_start", "hq_start"),
    ("koth_located", "hq_located"),
    ("koth_online", "hq_online"),
    ("koth_secured", "hq_secured"),
    ("koth_captured", "hq_capture"),
    ("koth_lost", "hq_capture"),
    ("sd_bomb_taken", "sd_bomb_taken_taken"),
    ("kls_planemortar_ready", "kls_airstrike_ready"),
    ("kls_planemortar_used", "kls_airstrike_used"),
    ("kls_planemortar_enemy", "kls_airstrike_enemy"),
    ("kls_straferun_ready", "kls_napalm_ready"),
    ("kls_straferun_used", "kls_napalm_used"),
    ("kls_straferun_enemy", "kls_napalm_enemy"),
    ("kls_mgl_ready", "kls_grim_ready"),
    ("kls_mgl_used", "kls_grim_used"),
    ("kls_mgl_enemy", "kls_grim_enemy"),
    ("kls_predator_ready", "kls_tv_ready"),
    ("kls_predator_used", "kls_tv_used"),
    ("kls_predator_enemy", "kls_tv_enemy"),
    ("kls_littlebird_ready", "kls_cobra_ready"),
    ("kls_littlebird_used", "kls_cobra_used"),
    ("kls_littlebird_enemy", "kls_cobra_enemy"),
    ("kls_playerheli_ready", "kls_huey_ready"),
    ("kls_playerheli_used", "kls_huey_used"),
    ("kls_playerheli_enemy", "kls_huey_enemy"),
    ("kls_microwave_ready", "kls_turret_ready"),
    ("kls_microwave_used", "kls_turret_used"),
    ("kls_microwave_enemy", "kls_turret_enemy"),
];

/// Routes Black Ops II announcer aliases (`vox_<faction>_<line>`) to BO1's
/// lines in the T5 namespace, where the bank carries them.
pub(crate) fn t6_bo1_routes(
    bank: &asset_audio::SoundCatalog,
    t6_aliases: &[String],
) -> HashMap<String, (AssetNamespace, String)> {
    t6_aliases
        .iter()
        .filter_map(|alias| {
            let lower = alias.to_ascii_lowercase();
            let (t6_prefix, t5_prefix) = T6_TO_T5_VOICE
                .iter()
                .find(|(t6, _)| lower.starts_with(t6))?;
            let line = &lower[t6_prefix.len()..];
            let line = T6_TO_T5_LINES
                .iter()
                .find(|(from, _)| *from == line)
                .map_or(line, |(_, to)| to);
            let routed = format!("{t5_prefix}{line}");
            bank.index_in(AssetNamespace::T5, &routed)?;
            Some((lower, (AssetNamespace::T5, routed)))
        })
        .collect()
}

const TEAM_MUSIC: [&str; 5] = [
    "spawn_music",
    "victory_music",
    "defeat_music",
    "winning_music",
    "losing_music",
];

const IW4_LINE: &str = "1mc_";

const T5_LINES: &[(&str, &str)] = &[
    ("acheive_bomb", "sd_bomb_taken_taken"),
    ("bomb_defused", "sd_bomb_defused"),
    ("bomb_planted", "sd_bomb_planted"),
    ("bomb_taken", "sd_bomb_drop"),
    ("boost", "generic_boost"),
    ("capture_a", "dom_capture_a"),
    ("capture_b", "dom_capture_c"),
    ("capture_c", "dom_capture_b"),
    ("capturing_a", "dom_capturing_a"),
    ("capturing_b", "dom_capturing_b"),
    ("capturing_c", "dom_capturing_c"),
    ("captureflag", "ctf_start"),
    ("demolition", "demo_start"),
    ("dest_sentrygun", "dest_sentry"),
    ("domination", "dom_start"),
    ("freeforall", "ffa_start"),
    ("headquarters", "hq_start"),
    ("hq_captured", "hq_capture"),
    ("hq_destroyed", "hq_defend"),
    ("losing_a", "dom_losing_a"),
    ("losing_b", "dom_losing_b"),
    ("losing_c", "dom_losing_c"),
    ("lost_a", "dom_lost_a"),
    ("lost_b", "dom_lost_b"),
    ("lost_c", "dom_lost_c"),
    ("obj_defend", "defend_start"),
    ("obj_destroy", "destroy_start"),
    ("positions_lock", "dom_lock_wetake"),
    ("sabotage", "sab_start"),
    ("searchdestroy", "sd_start"),
    ("secure_a", "dom_secured_a"),
    ("secure_b", "dom_secured_b"),
    ("secure_c", "dom_secured_c"),
    ("securing_a", "dom_securing_a"),
    ("securing_b", "dom_securing_b"),
    ("securing_c", "dom_securing_c"),
    ("switching", "switchingsides"),
    ("take_positions", "dom_lock_theytake"),
    ("tm_death", "tdm_start"),
];

#[derive(Resource, Default)]
pub(crate) struct AnnouncerRoutes(HashMap<String, (AssetNamespace, String)>);

impl AnnouncerRoutes {
    pub(crate) fn route<'a>(&'a self, alias: &'a str) -> (AssetNamespace, &'a str) {
        self.0
            .get(alias)
            .map_or((AssetNamespace::Iw4, alias), |(ns, routed)| (*ns, routed))
    }

    pub(crate) fn len(&self) -> usize {
        self.0.len()
    }

    /// The route for a script-played alias of `namespace`, if one is set.
    /// Black Ops II's routes are keyed by bare alias name.
    pub(crate) fn routed(
        &self,
        namespace: AssetNamespace,
        alias: &str,
    ) -> Option<(AssetNamespace, &str)> {
        if namespace != AssetNamespace::T6 {
            return None;
        }
        self.0
            .get(alias)
            .map(|(ns, routed)| (*ns, routed.as_str()))
    }

    pub(crate) fn extend(&mut self, routes: HashMap<String, (AssetNamespace, String)>) {
        self.0.extend(routes);
    }

    pub(crate) fn build(
        bank: &asset_audio::SoundCatalog,
        namespace: AssetNamespace,
        iw4: [Option<&str>; 2],
        native: Option<&asset_game::MapTeamSettings>,
        aliases: &[String],
    ) -> Self {
        let lines: &[(&str, &str)] = match namespace {
            AssetNamespace::Iw4 => return Self::default(),
            AssetNamespace::T5 => T5_LINES,
            AssetNamespace::Iw5 => &[],
            // bo2zm: Black Ops II announcer lines are not mapped yet.
            AssetNamespace::T6 => &[],
        };
        let Some(native) = native else {
            return Self::default();
        };
        let teams: Vec<(&str, Option<&str>, &asset_game::TeamMusic)> = iw4
            .into_iter()
            .zip([
                (native.allies_voice.as_deref(), &native.allies_music),
                (native.axis_voice.as_deref(), &native.axis_music),
            ])
            .filter_map(|(iw4, (voice, music))| Some((iw4?, voice, music)))
            .collect();
        let routes = aliases
            .iter()
            .filter_map(|alias| {
                teams.iter().find_map(|&(iw4, voice, music)| {
                    let line = alias.strip_prefix(iw4)?;
                    let routed = match line.strip_prefix(IW4_LINE) {
                        Some(line) => {
                            let line = lines
                                .iter()
                                .find(|(from, _)| *from == line)
                                .map_or(line, |(_, to)| to);
                            format!("{}{line}", voice?)
                        }
                        None => match line {
                            "spawn_music" => music.spawn.clone(),
                            "victory_music" => music.victory.clone(),
                            "defeat_music" => music.defeat.clone(),
                            "winning_music" => music.winning.clone(),
                            "losing_music" => music.losing.clone(),
                            _ => None,
                        }?,
                    };
                    bank.index_in(namespace, &routed)?;
                    Some((alias.clone(), (namespace, routed)))
                })
            })
            .collect();
        Self(routes)
    }
}
