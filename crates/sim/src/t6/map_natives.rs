//! bo2mp: a map from another game (MW2, MW3, CoD4, Black Ops) under Black
//! Ops II's rules. The session installs the map's own script models
//! (drawn from the map's catalog: bomb sites, flags, crates, Headquarters
//! radios) and script brush models (their collision and drawn brushes)
//! from its entity text, each under its block's place in that text. Black
//! Ops II's scripts see the same blocks as their map entities, and decide
//! what stays: `_gameobjects::main` deletes every entity whose
//! `script_gameobjectname` does not name the game type (Search and
//! Destroy's bomb sites in Team Deathmatch, Capture the Flag's flags in
//! Domination ...); scripts hide, show, re-model and unsolid others.
//!
//! Here the session's own rows follow the BO2 entity of the same place:
//! - gone or hidden (or given another model, which the BO2 entity's own
//!   presence draws): the map's model is hidden and lets bullets through,
//!   its brush is not drawn;
//! - a brush model's collision is the BO2 entity's own (`brushes`), which
//!   moves, unsolids and goes with it: the session's copy, standing where
//!   the map put it whatever the scripts did, stops blocking (it was an
//!   invisible wall wherever a game type deleted its brush: a Search and
//!   Destroy blocker, a bomb site's crate in Team Deathmatch).
//!
//! Without this, every game type showed every game type's objectives and
//! was blocked by their brushes.

use std::collections::BTreeMap;

use bevy_ecs::prelude::World;

use super::{Zm, frame};
use crate::ScriptModelId;

/// What a map entity was when the level started.
#[derive(Clone, Debug, Default)]
pub(crate) struct MapNative {
    pub classname: String,
    pub model: String,
    pub gameobject: String,
}

impl MapNative {
    fn brush(&self) -> bool {
        self.model.starts_with('*')
    }
}

/// What was last done to the session's row of a place.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub(crate) struct Applied {
    pub hidden: bool,
    /// A brush model's collision was handed to the BO2 entity.
    pub brush_handed: bool,
}

/// The session's row for the map entity at `ordinal` (as the session
/// names it: `install_script_model_id`).
fn session_id(ordinal: u32) -> ScriptModelId {
    ScriptModelId::from_authored_source_ordinal(ordinal)
}

/// A map entity as it spawns (`spawn_map_entities`).
pub(super) fn note(world: &mut World, ordinal: u32, classname: &str, keys: &[(String, String)]) {
    let get = |want: &str| {
        keys.iter()
            .find(|(k, _)| k.eq_ignore_ascii_case(want))
            .map_or_else(String::new, |(_, v)| v.clone())
    };
    if classname != "script_model" && classname != "script_brushmodel" {
        return;
    }
    world.resource_mut::<Zm>().map_natives.insert(
        ordinal,
        MapNative {
            classname: classname.to_owned(),
            model: get("model"),
            gameobject: get("script_gameobjectname"),
        },
    );
}

/// Each frame: the session's rows follow their BO2 entities.
pub(super) fn sync(world: &mut World, now_ms: i64) {
    let wanted: Vec<(u32, bool, bool, String)> = {
        let zm = world.resource::<Zm>();
        if zm.map_natives.is_empty() {
            return;
        }
        let alive: BTreeMap<u32, &super::Ent> = zm
            .ents
            .values()
            .filter_map(|e| Some((e.map_ordinal?, e)))
            .collect();
        zm.map_natives
            .iter()
            .map(|(ordinal, native)| {
                let hidden = match alive.get(ordinal) {
                    None => true,
                    Some(e) => {
                        e.hidden
                            || e.invisible_to_all
                            || (!native.brush() && !e.model.eq_ignore_ascii_case(&native.model))
                    }
                };
                let why = match alive.get(ordinal) {
                    None => "deleted",
                    Some(e) if e.hidden || e.invisible_to_all => "hidden",
                    Some(_) if hidden => "re-modelled",
                    Some(_) => "shown",
                };
                (*ordinal, hidden, native.brush(), why.to_owned())
            })
            .collect()
    };
    let gametype = super::with_vm(world, |vm, _| {
        vm.dvars.get("g_gametype").cloned().unwrap_or_default()
    })
    .unwrap_or_default();
    let mut changes: Vec<(u32, Applied, String)> = Vec::new();
    {
        let zm = world.resource::<Zm>();
        for (ordinal, hidden, brush, why) in wanted {
            let was = zm
                .map_natives_applied
                .get(&ordinal)
                .copied()
                .unwrap_or_default();
            let now = Applied {
                hidden,
                brush_handed: brush,
            };
            if was != now {
                changes.push((ordinal, now, why));
            }
        }
    }
    // Once, a few seconds in (after the game type's start-up deletes): what
    // the game type kept and removed, by gameobject tag.
    let report_due = !world.resource::<Zm>().map_natives_reported && now_ms >= 3000;
    if changes.is_empty() {
        if report_due {
            report(world, &gametype);
        }
        return;
    }
    static LOGGED: std::sync::atomic::AtomicU32 = std::sync::atomic::AtomicU32::new(0);
    for (ordinal, applied, why) in changes {
        let id = session_id(ordinal);
        let mut f = frame(world);
        let mut touched = false;
        if let Some(row) = f.collision_owner_mut(id) {
            row.hidden = applied.hidden;
            // A brush's collision is the BO2 entity's (`brushes`); a
            // model's goes with it.
            row.solid = !applied.hidden && !applied.brush_handed;
            touched = true;
        }
        if let Some(number) = f.gentity_number(id)
            && let Some(mover) = f.script_mover_mut_by_number(number)
        {
            if applied.hidden {
                mover.state.e_flags |= entity_iw4::CG_SCRIPT_MOVER_NODRAW;
            } else {
                mover.state.e_flags &= !entity_iw4::CG_SCRIPT_MOVER_NODRAW;
            }
            // (A model's mover carries no collision; its row above does.)
            if applied.brush_handed {
                mover.nonsolid = true;
            }
            touched = true;
        }
        drop(f);
        let mut zm = world.resource_mut::<Zm>();
        zm.map_natives_applied.insert(ordinal, applied);
        if touched
            && applied.hidden
            && LOGGED.fetch_add(1, std::sync::atomic::Ordering::Relaxed) < 96
        {
            let native = zm.map_natives.get(&ordinal).cloned().unwrap_or_default();
            diag::info!(
                Sim,
                "bo2mp map entity #{ordinal} {} {} [{}] {why} under {gametype}: the map's own {} is hidden{}",
                native.classname,
                native.model,
                native.gameobject,
                if native.brush() { "brush" } else { "model" },
                if native.brush() {
                    " (its collision went with it)"
                } else {
                    ""
                }
            );
        }
    }
    if report_due {
        report(world, &gametype);
    }
}

fn report(world: &mut World, gametype: &str) {
    let mut zm = world.resource_mut::<Zm>();
    zm.map_natives_reported = true;
    let mut kept: BTreeMap<String, u32> = BTreeMap::new();
    let mut removed: BTreeMap<String, u32> = BTreeMap::new();
    let (mut models, mut brushes, mut hidden_models, mut hidden_brushes) = (0u32, 0u32, 0u32, 0u32);
    for (ordinal, native) in &zm.map_natives {
        let hidden = zm
            .map_natives_applied
            .get(ordinal)
            .is_some_and(|a| a.hidden);
        match (native.brush(), hidden) {
            (true, true) => hidden_brushes += 1,
            (true, false) => brushes += 1,
            (false, true) => hidden_models += 1,
            (false, false) => models += 1,
        }
        if !native.gameobject.is_empty() {
            let tag = native.gameobject.clone();
            *if hidden { &mut removed } else { &mut kept }
                .entry(tag)
                .or_default() += 1;
        }
    }
    diag::info!(
        Sim,
        "bo2mp map entities under {gametype}: models {models} shown / {hidden_models} hidden, \
         brushes {brushes} kept / {hidden_brushes} removed (brush collision from BO2's entities); \
         gameobjects kept {kept:?} removed {removed:?}"
    );
}

/// At level start: the spawn points of each class the BO2 scripts can see
/// (a game type whose classes are missing spawned players at the origin).
pub(super) fn report_spawns(world: &mut World, gametype: &str) {
    let zm = world.resource::<Zm>();
    let mut counts: BTreeMap<&str, u32> = BTreeMap::new();
    for e in zm.ents.values() {
        if e.classname.starts_with("mp_") && e.classname.contains("_spawn") {
            *counts.entry(e.classname.as_str()).or_default() += 1;
        }
    }
    let at_origin = zm
        .ents
        .values()
        .filter(|e| e.classname.starts_with("mp_") && e.classname.contains("_spawn"))
        .filter(|e| e.origin == [0.0; 3])
        .count();
    diag::info!(
        Sim,
        "bo2mp spawns at level start ({gametype}): {counts:?}{}",
        if at_origin > 0 {
            format!(" — {at_origin} at the world origin")
        } else {
            String::new()
        }
    );
}
