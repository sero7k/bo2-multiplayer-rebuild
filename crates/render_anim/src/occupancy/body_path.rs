//! bo2mp: which system draws a player's body, so every player has exactly
//! one.
//!
//! Two systems can draw a player (and his corpse and the gun in his hands):
//!
//! - Black Ops II's: `occupancy::t6_body` (the faction script's character,
//!   head and gear, BO2's animations, the BO2 world gun), drawn by the
//!   fallback pass (`render::extract::extract_t6_dynamic`).
//! - The engine's own (Modern Warfare 2's `CG_Player`):
//!   `occupancy::remote_body`, the soldier kit the map's zone carries (the
//!   map game's own body and head, its world gun), drawn by the exact
//!   path's model lane with its model lighting.
//!
//! A Black Ops II map carries no such kit, so only BO2's ever drew there.
//! A Modern Warfare 2/3 or Black Ops map does (its zone holds its game's
//! soldiers), and under Black Ops II's rules both ran: two bodies in one
//! place, the map game's soldier through BO2's character, each with a gun.
//! Under BO2's rules (or whenever BO2's player animation script is loaded,
//! which the BO2 path needs) the BO2 path is the only one; the engine's
//! own draws only in a match by another game's rules.

use bevy::prelude::*;

/// What draws the players this match.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum PlayerBodyPath {
    /// Black Ops II's characters by the fallback pass (`t6_body`); the
    /// engine's remote bodies never run.
    Bo2Pass,
    /// The engine's own remote bodies (a match by another game's rules).
    Legacy,
    /// Neither has what it needs (no BO2 script, no soldier kit).
    Neither,
}

impl PlayerBodyPath {
    pub fn label(self) -> &'static str {
        match self {
            Self::Bo2Pass => "bo2 fallback pass (t6_body)",
            Self::Legacy => "engine remote body (map game's soldier kit)",
            Self::Neither => "none",
        }
    }
}

/// The choice: under Black Ops II's rules, or with BO2's player animation
/// script loaded, BO2's path only, whatever soldiers the map's zone holds.
pub fn player_body_path(t6_rules: bool, bo2_script: bool, legacy_kits: bool) -> PlayerBodyPath {
    if t6_rules || bo2_script {
        PlayerBodyPath::Bo2Pass
    } else if legacy_kits {
        PlayerBodyPath::Legacy
    } else {
        PlayerBodyPath::Neither
    }
}

/// Whether the engine's own remote bodies (and their corpses and guns) may
/// run this frame.
pub fn legacy_bodies_allowed(bo2_script: Option<&sim::t6_playeranim::T6PlayerAnimsRes>) -> bool {
    let script = bo2_script.is_some_and(|s| s.0.is_some());
    // The kit is not looked at: any value other than Bo2Pass leaves the
    // remote-body systems to their own checks.
    player_body_path(assets::t6_rules(), script, true) != PlayerBodyPath::Bo2Pass
}

/// One player (or corpse) the BO2 path drew this frame.
#[derive(Clone, Debug)]
pub struct Bo2BodyDrawn {
    pub number: u16,
    pub corpse: bool,
    /// Draw items (models: body, head, gear, gun, attachments).
    pub items: usize,
    /// Surfaces with a material over those items.
    pub surfaces: usize,
    pub body: String,
}

/// This frame's player draws by both paths, for the census log.
#[derive(Resource, Default)]
pub struct PlayerBodyCensus {
    pub bo2: Vec<Bo2BodyDrawn>,
}

/// How many frames with players in them are logged after a match loads.
const CENSUS_FRAMES: u32 = 3;

#[derive(Default)]
pub(crate) struct CensusLocals {
    owner: Option<usize>,
    logged: u32,
}

/// Once per match load: which path draws the players and what the map's
/// zone brought (the soldiers the engine's path would draw); then, for the
/// first frames with players, each player's draws by each path: one BO2
/// body each and no engine body is right.
pub(crate) fn log_player_body_census(
    census: Res<PlayerBodyCensus>,
    bodies: Option<Res<assets::PreparedBodies>>,
    script: Option<Res<sim::t6_playeranim::T6PlayerAnimsRes>>,
    legacy_plan: Res<crate::RemoteBodyDrawPlan>,
    legacy_roots: Query<&net::CEntity, With<crate::occupancy::remote_body::RemotePlayer>>,
    mut locals: Local<CensusLocals>,
) {
    let Some(bodies) = bodies else {
        return;
    };
    let owner = std::sync::Arc::as_ptr(&bodies.0) as usize;
    let bo2_script = script.as_ref().is_some_and(|s| s.0.is_some());
    let kits = bodies.0.kits();
    let legacy_kits = kits.allies.is_some() || kits.axis.is_some();
    let path = player_body_path(assets::t6_rules(), bo2_script, legacy_kits);
    if locals.owner != Some(owner) {
        locals.owner = Some(owner);
        locals.logged = 0;
        let kit = |k: Option<&asset_model::SoldierKit>| {
            k.map_or_else(
                || "-".to_owned(),
                |k| format!("{}+{}", k.body, k.head.as_deref().unwrap_or("-")),
            )
        };
        diag::info!(
            World,
            "player bodies: drawn by {} (t6_rules={} bo2_playeranim={}); map zone soldier kits allies={} axis={}{}",
            path.label(),
            assets::t6_rules(),
            bo2_script,
            kit(kits.allies.as_ref()),
            kit(kits.axis.as_ref()),
            if path == PlayerBodyPath::Bo2Pass && !bo2_script {
                " — NO BO2 PLAYER ANIM SCRIPT: no player body is drawn"
            } else if path == PlayerBodyPath::Bo2Pass && legacy_kits {
                " (not drawn: the engine's remote bodies are off under BO2's rules)"
            } else {
                ""
            }
        );
    }
    let legacy_draws = legacy_plan.draws.len();
    let legacy_players = legacy_roots.iter().count();
    if locals.logged >= CENSUS_FRAMES || (census.bo2.is_empty() && legacy_draws == 0 && legacy_players == 0) {
        return;
    }
    locals.logged += 1;
    let mut legacy_by_ent: std::collections::BTreeMap<u32, usize> = std::collections::BTreeMap::new();
    for d in &legacy_plan.draws {
        *legacy_by_ent.entry(d.scene_entnum.unwrap_or(u32::MAX)).or_default() += 1;
    }
    let bo2: Vec<String> = census
        .bo2
        .iter()
        .take(24)
        .map(|b| {
            format!(
                "#{}{} {} items/{} surfaces ({})",
                b.number,
                if b.corpse { " corpse" } else { "" },
                b.items,
                b.surfaces,
                b.body
            )
        })
        .collect();
    let mut twice: Vec<u16> = Vec::new();
    for (i, b) in census.bo2.iter().enumerate() {
        if census.bo2[..i].iter().any(|o| o.number == b.number) && !twice.contains(&b.number) {
            twice.push(b.number);
        }
    }
    let doubled = legacy_draws > 0 || !twice.is_empty();
    let line = format!(
        "player bodies frame {}/{CENSUS_FRAMES}: path={} bo2 bodies={} [{}]; engine remote bodies={} draws={} by entity {:?}{}",
        locals.logged,
        path.label(),
        census.bo2.len(),
        bo2.join(", "),
        legacy_players,
        legacy_draws,
        legacy_by_ent,
        if doubled {
            format!(" — DOUBLE BODY: a player drawn by more than one path (twice in bo2: {twice:?})")
        } else {
            String::new()
        }
    );
    if doubled {
        diag::warn!(World, "{line}");
    } else {
        diag::info!(World, "{line}");
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn bo2_rules_never_draw_the_map_games_soldier() {
        // A MW2/MW3/BO1 map under BO2's rules: its zone brought soldiers.
        assert_eq!(player_body_path(true, true, true), PlayerBodyPath::Bo2Pass);
        assert_eq!(player_body_path(true, false, true), PlayerBodyPath::Bo2Pass);
        // BO2's script loaded (a BO2 map, Zombies) whatever the rules flag.
        assert_eq!(player_body_path(false, true, true), PlayerBodyPath::Bo2Pass);
        assert_eq!(player_body_path(true, true, false), PlayerBodyPath::Bo2Pass);
    }

    #[test]
    fn other_rules_keep_the_engines_bodies() {
        assert_eq!(player_body_path(false, false, true), PlayerBodyPath::Legacy);
        assert_eq!(player_body_path(false, false, false), PlayerBodyPath::Neither);
    }
}
