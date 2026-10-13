use std::collections::{HashMap, HashSet};
use std::sync::LazyLock;

use movement_iw4::{SURFACE_TYPE_NAMES, surface_type_index};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum StepGait {
    Sprint,
    Run,
    Walk,
    Prone,
}

impl StepGait {
    pub const ALL: [Self; 4] = [Self::Sprint, Self::Run, Self::Walk, Self::Prone];

    fn index(self) -> usize {
        match self {
            Self::Sprint => 0,
            Self::Run => 1,
            Self::Walk => 2,
            Self::Prone => 3,
        }
    }
}

#[derive(Clone, Copy)]
struct SurfacePick {
    alias: &'static str,
    fallback: &'static str,
}

const SURF_N: usize = SURFACE_TYPE_NAMES.len();

struct SurfaceBank {
    footstep: [[[[SurfacePick; SURF_N]; 2]; 2]; 4],
    land: [[[SurfacePick; SURF_N]; 2]; 2],
    by_alias: HashMap<&'static str, &'static [&'static str]>,
    quiet: HashMap<&'static str, &'static str>,
    world: HashMap<&'static str, &'static str>,
    all_names: Vec<&'static str>,
}

fn leak_str(s: String) -> &'static str {
    Box::leak(s.into_boxed_str())
}

fn leak_slice(v: Vec<&'static str>) -> &'static [&'static str] {
    Box::leak(v.into_boxed_slice())
}

fn build_world_owned(alias: &str) -> Option<String> {
    let idx = alias.find("_plr_")?;
    let mut out = String::with_capacity(alias.len() - 4);
    out.push_str(&alias[..idx]);
    out.push('_');
    out.push_str(&alias[idx + 5..]);
    Some(out)
}

fn build_quiet_owned(alias: &str) -> Option<String> {
    if let Some(rest) = alias.strip_prefix("step_") {
        return Some(format!("qstep_{rest}"));
    }
    if let Some(rest) = alias.strip_prefix("Land_") {
        return Some(format!("qLand_{rest}"));
    }
    None
}

fn build_candidates_owned(alias: &str, fallback: &str) -> Vec<String> {
    let mut out = Vec::with_capacity(8);
    let mut push = |name: String| {
        if !name.is_empty() && !out.iter().any(|existing| existing == &name) {
            out.push(name);
        }
    };
    for seed in [alias, fallback] {
        push(seed.to_owned());
        if let Some(world) = build_world_owned(seed) {
            push(world);
        }
        if let Some(quiet) = build_quiet_owned(seed) {
            if let Some(quiet_world) = build_world_owned(&quiet) {
                push(quiet.clone());
                push(quiet_world);
            } else {
                push(quiet);
            }
        }
    }
    out
}

fn intern_pick(
    alias: String,
    fallback: String,
    names: &mut HashSet<&'static str>,
    quiet: &mut HashMap<&'static str, &'static str>,
    world: &mut HashMap<&'static str, &'static str>,
    by_alias: &mut HashMap<&'static str, &'static [&'static str]>,
) -> SurfacePick {
    let candidates_owned = build_candidates_owned(&alias, &fallback);
    let alias = leak_str(alias);
    let fallback = leak_str(fallback);
    let mut leaked = Vec::with_capacity(candidates_owned.len());
    for name in candidates_owned {
        let interned = leak_str(name);
        names.insert(interned);
        leaked.push(interned);
        if let Some(q) = build_quiet_owned(interned) {
            quiet.insert(interned, leak_str(q));
        }
        if let Some(w) = build_world_owned(interned) {
            world.insert(interned, leak_str(w));
        }
    }
    let candidates = leak_slice(leaked);
    by_alias.insert(alias, candidates);
    SurfacePick { alias, fallback }
}

fn build_bank() -> SurfaceBank {
    let mut names = HashSet::new();
    let mut quiet = HashMap::new();
    let mut world = HashMap::new();
    let mut by_alias = HashMap::new();
    let dummy = intern_pick(
        "step_run_default".to_owned(),
        "step_run_default".to_owned(),
        &mut names,
        &mut quiet,
        &mut world,
        &mut by_alias,
    );
    let mut footstep = [[[[dummy; SURF_N]; 2]; 2]; 4];
    let mut land = [[[dummy; SURF_N]; 2]; 2];
    for (gi, gait) in StepGait::ALL.into_iter().enumerate() {
        for local in [false, true] {
            let li = usize::from(local);
            for quieter in [false, true] {
                let qi = usize::from(quieter);
                for (si, surface) in SURFACE_TYPE_NAMES.iter().enumerate() {
                    let prefix = step_prefix(gait, local);
                    let mut alias = format!("{prefix}_{surface}");
                    let mut fallback = format!("{prefix}_default");
                    if quieter {
                        alias = build_quiet_owned(&alias).unwrap_or(alias);
                        fallback = build_quiet_owned(&fallback).unwrap_or(fallback);
                    }
                    footstep[gi][li][qi][si] = intern_pick(
                        alias,
                        fallback,
                        &mut names,
                        &mut quiet,
                        &mut world,
                        &mut by_alias,
                    );
                }
            }
        }
    }
    for local in [false, true] {
        let li = usize::from(local);
        for quieter in [false, true] {
            let qi = usize::from(quieter);
            for (si, surface) in SURFACE_TYPE_NAMES.iter().enumerate() {
                let mut alias = if local {
                    format!("Land_plr_{surface}")
                } else {
                    format!("Land_{surface}")
                };
                let mut fallback = if local {
                    "Land_plr_default".to_owned()
                } else {
                    "Land_default".to_owned()
                };
                if quieter {
                    alias = build_quiet_owned(&alias).unwrap_or(alias);
                    fallback = build_quiet_owned(&fallback).unwrap_or(fallback);
                }
                land[li][qi][si] = intern_pick(
                    alias,
                    fallback,
                    &mut names,
                    &mut quiet,
                    &mut world,
                    &mut by_alias,
                );
            }
        }
    }
    for gait in StepGait::ALL {
        for local in [false, true] {
            names.insert(gear_rattle_alias(gait, local));
        }
    }
    names.insert(gear_alias(true));
    names.insert(gear_alias(false));
    let mut all_names: Vec<&'static str> = names.into_iter().collect();
    all_names.sort_unstable();
    SurfaceBank {
        footstep,
        land,
        by_alias,
        quiet,
        world,
        all_names,
    }
}

static BANK: LazyLock<SurfaceBank> = LazyLock::new(build_bank);

fn surf_slot(surface_flags: u32) -> usize {
    let index = surface_type_index(surface_flags);
    if index < SURFACE_TYPE_NAMES.len() {
        index
    } else {
        0
    }
}

fn footstep_pick(
    gait: StepGait,
    surface_flags: u32,
    local_player: bool,
    quieter: bool,
) -> SurfacePick {
    BANK.footstep[gait.index()][usize::from(local_player)][usize::from(quieter)]
        [surf_slot(surface_flags)]
}

fn land_pick(surface_flags: u32, local_player: bool, quieter: bool) -> SurfacePick {
    BANK.land[usize::from(local_player)][usize::from(quieter)][surf_slot(surface_flags)]
}

pub fn step_prefix(gait: StepGait, local_player: bool) -> &'static str {
    match (gait, local_player) {
        (StepGait::Sprint, true) => "step_sprint_plr",
        (StepGait::Sprint, false) => "step_sprint",
        (StepGait::Run, true) => "step_run_plr",
        (StepGait::Run, false) => "step_run",
        (StepGait::Walk, true) => "step_walk_plr",
        (StepGait::Walk, false) => "step_walk",
        (StepGait::Prone, true) => "step_prone_plr",
        (StepGait::Prone, false) => "step_prone",
    }
}

pub fn footstep_aliases(
    gait: StepGait,
    surface_flags: u32,
    local_player: bool,
    quieter: bool,
) -> (&'static str, &'static str) {
    let pick = footstep_pick(gait, surface_flags, local_player, quieter);
    (pick.alias, pick.fallback)
}

pub fn world_surface_alias(alias: &str) -> Option<&'static str> {
    BANK.world.get(alias).copied()
}

pub fn quiet_surface_alias(alias: &str) -> Option<&'static str> {
    BANK.quiet.get(alias).copied()
}

pub fn surface_alias_candidates(alias: &str, _fallback: &str) -> &'static [&'static str] {
    BANK.by_alias.get(alias).copied().unwrap_or(&[])
}

pub fn gear_rattle_alias(gait: StepGait, local_player: bool) -> &'static str {
    match (gait, local_player) {
        (StepGait::Sprint, true) => "gear_rattle_plr_sprint",
        (StepGait::Sprint, false) => "gear_rattle_sprint",
        (StepGait::Run, true) => "gear_rattle_plr_run",
        (StepGait::Run, false) => "gear_rattle_run",
        (StepGait::Walk, true) => "gear_rattle_plr_walk",
        (StepGait::Walk, false) => "gear_rattle_walk",
        (StepGait::Prone, true) => "gear_rattle_plr_prone",
        (StepGait::Prone, false) => "gear_rattle_prone",
    }
}

pub fn gear_alias(local_player: bool) -> &'static str {
    if local_player {
        "gear_rattle_plr_mantle"
    } else {
        "gear_rattle_mantle"
    }
}

pub fn land_aliases(
    surface_flags: u32,
    local_player: bool,
    quieter: bool,
) -> (&'static str, &'static str) {
    let pick = land_pick(surface_flags, local_player, quieter);
    (pick.alias, pick.fallback)
}

/// bo2zm: a movement sound by Black Ops II's names, which its footstep
/// tables list per surface: steps `fly_step_<gait>_<plr|npc>_<surface>`,
/// landings `fly_land_<plr|npc>_<surface>`, gear `fly_gear_<gait>_<plr|npc>`.
/// BO2 keeps no quieter set; a quiet name maps to the plain one.
fn t6_movement_owned(alias: &str) -> Option<String> {
    let plain = alias
        .strip_prefix('q')
        .filter(|rest| rest.starts_with("step_") || rest.starts_with("Land_"))
        .unwrap_or(alias);
    if let Some(rest) = plain.strip_prefix("step_") {
        let (gait, rest) = rest.split_once('_')?;
        return Some(match rest.strip_prefix("plr_") {
            Some(surface) => format!("fly_step_{gait}_plr_{surface}"),
            None => format!("fly_step_{gait}_npc_{rest}"),
        });
    }
    if let Some(rest) = plain.strip_prefix("Land_") {
        return Some(match rest.strip_prefix("plr_") {
            Some(surface) => format!("fly_land_plr_{surface}"),
            None => format!("fly_land_npc_{rest}"),
        });
    }
    let rest = alias.strip_prefix("gear_rattle_")?;
<<<<<<< /home/claude/bo2-multiplayer-rebuild/crates/audio/src/aliases.rs
    // bo2mp: another player's mantle is BO2's prone-and-mantle gear (it has
    // no fly_gear_mantle_npc).
    if rest == "mantle" {
        return Some("fly_prone_n_mantle_npc".to_owned());
    }
=======
>>>>>>> /tmp/claude-0/zmhead/crates/audio/src/aliases.rs
    Some(match rest.strip_prefix("plr_") {
        Some(gait) => format!("fly_gear_{gait}_plr"),
        None => format!("fly_gear_{rest}_npc"),
    })
}

static T6_MOVEMENT: LazyLock<HashMap<&'static str, &'static str>> = LazyLock::new(|| {
    BANK.all_names
        .iter()
        .filter_map(|&name| Some((name, leak_str(t6_movement_owned(name)?))))
        .collect()
});

/// bo2zm: the Black Ops II name of a footstep, landing or gear sound.
pub fn t6_movement_alias(alias: &str) -> Option<&'static str> {
    T6_MOVEMENT.get(alias).copied()
}

/// bo2mp: Black Ops II's surface types by index. Under its rules every
/// map's surface flags are read in this list (an Infinity Ward map's slush
/// and riot shield are moved into it as the map loads), not in Modern
/// Warfare 2's: its index 30 is tall grass, not slush, and 31 the riot
/// shield.
pub const T6_STEP_SURFACES: [&str; 32] = [
    "default",
    "bark",
    "brick",
    "carpet",
    "cloth",
    "concrete",
    "dirt",
    "flesh",
    "foliage",
    "glass",
    "grass",
    "gravel",
    "ice",
    "metal",
    "mud",
    "paper",
    "plaster",
    "rock",
    "sand",
    "snow",
    "water",
    "wood",
    "asphalt",
    "ceramic",
    "plastic",
    "rubber",
    "cushion",
    "fruit",
    "paintedmetal",
    "player",
    "tallgrass",
    "riotshield",
];

/// The surfaces nearest to `surface` underfoot, for one BO2's bank has no
/// step of its own: what the foot would sound like on it, never a loud
/// odd one.
fn t6_step_substitutes(surface: &str) -> &'static [&'static str] {
    match surface {
        "bark" | "paper" | "plastic" => &["wood"],
        "brick" | "plaster" | "ceramic" | "glass" | "rubber" => &["concrete"],
        "carpet" => &["cloth"],
        "cloth" | "cushion" => &["carpet"],
        "flesh" | "fruit" | "water" => &["mud", "dirt"],
        "foliage" | "tallgrass" => &["grass", "dirt"],
        "ice" => &["snow", "concrete"],
        "mud" | "sand" | "player" => &["dirt"],
        "rock" => &["gravel", "concrete"],
        "snow" => &["dirt"],
        "slush" => &["snow", "dirt"],
        "paintedmetal" | "riotshield" => &["metal"],
        _ => &[],
    }
}

/// A footstep or a landing, as BO2 names them by surface.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum T6StepKind {
    Step(StepGait),
    Land,
}

fn t6_step_name(kind: T6StepKind, local_player: bool, surface: &str) -> String {
    let who = if local_player { "plr" } else { "npc" };
    match kind {
        T6StepKind::Step(gait) => {
            let gait = match gait {
                StepGait::Sprint => "sprint",
                StepGait::Run => "run",
                StepGait::Walk => "walk",
                StepGait::Prone => "prone",
            };
            format!("fly_step_{gait}_{who}_{surface}")
        }
        T6StepKind::Land => format!("fly_land_{who}_{surface}"),
    }
}

/// bo2mp: the Black Ops II sound for a step or landing on surface type
/// `index` (BO2's list): its own when `has` (BO2's bank) has it, else the
/// nearest surface it has, else concrete, then dirt; BO2's `default` only
/// when none of those exists. With the surface it settled on.
pub fn t6_step_alias(
    has: impl Fn(&str) -> bool,
    kind: T6StepKind,
    index: usize,
    local_player: bool,
) -> Option<(String, &'static str)> {
    let surface = T6_STEP_SURFACES.get(index).copied().unwrap_or("default");
    let own = (index != 0).then_some(surface);
    own.into_iter()
        .chain(t6_step_substitutes(surface).iter().copied())
        .chain(["concrete", "dirt", "default"])
        .map(|s| (t6_step_name(kind, local_player, s), s))
        .find(|(name, _)| has(name))
}

/// Diag: each (surface, sound) pair once, the first time a step plays it.
pub fn note_t6_step(index: usize, alias: &str, settled: &str) {
    static SEEN: LazyLock<std::sync::Mutex<HashSet<(usize, String)>>> =
        LazyLock::new(|| std::sync::Mutex::new(HashSet::new()));
    let fresh = SEEN
        .lock()
        .map(|mut seen| seen.insert((index, alias.to_owned())))
        .unwrap_or(false);
    if fresh {
        let surface = T6_STEP_SURFACES.get(index).copied().unwrap_or("?");
        if surface == settled {
            diag::info!(Audio, "bo2 footsteps: surface {surface} (#{index}) plays {alias}");
        } else {
            diag::info!(
                Audio,
                "bo2 footsteps: surface {surface} (#{index}) has no BO2 step; plays {alias} ({settled})"
            );
        }
    }
}

pub fn movement_prepare_names() -> &'static [&'static str] {
    BANK.all_names.as_slice()
}

pub fn select_fire_alias<'a>(
    player_view: bool,
    fire: Option<&'a str>,
    fire_player: Option<&'a str>,
) -> Option<&'a str> {
    if player_view {
        if let Some(p) = fire_player.filter(|s| !s.is_empty()) {
            return Some(p);
        }
    }
    fire.filter(|s| !s.is_empty())
}

pub fn select_cg_fire_alias<'a>(
    last_shot: bool,
    player_view: bool,
    fire: Option<&'a str>,
    fire_player: Option<&'a str>,
    fire_last: Option<&'a str>,
    fire_last_player: Option<&'a str>,
) -> Option<&'a str> {
    let current = select_fire_alias(player_view, fire, fire_player);
    if !last_shot {
        return current;
    }
    if player_view {
        if let Some(p) = fire_last_player.filter(|s| !s.is_empty()) {
            return Some(p);
        }
    }
    if let Some(w) = fire_last.filter(|s| !s.is_empty()) {
        return Some(w);
    }
    current
}

#[cfg(test)]
mod t6_movement_tests {
    use super::*;

    #[test]
    fn movement_sounds_take_black_ops_ii_names() {
        let (step, _) = footstep_aliases(StepGait::Run, 22 << 20, true, false);
        assert_eq!(t6_movement_alias(step), Some("fly_step_run_plr_asphalt"));
        let (step, _) = footstep_aliases(StepGait::Sprint, 5 << 20, false, true);
        assert_eq!(t6_movement_alias(step), Some("fly_step_sprint_npc_concrete"));
        let (land, fallback) = land_aliases(22 << 20, true, false);
        assert_eq!(t6_movement_alias(land), Some("fly_land_plr_asphalt"));
        assert_eq!(t6_movement_alias(fallback), Some("fly_land_plr_default"));
        assert_eq!(
            t6_movement_alias(gear_rattle_alias(StepGait::Sprint, true)),
            Some("fly_gear_sprint_plr")
        );
        assert_eq!(t6_movement_alias(gear_alias(true)), Some("fly_gear_mantle_plr"));
    }
}

#[cfg(test)]
mod t6_step_tests {
    use super::*;

    fn bo2_bank(name: &str) -> bool {
        // A bank with a few surfaces, as BO2's: no plaster, no tall grass.
        [
            "fly_step_run_plr_default",
            "fly_step_run_plr_concrete",
            "fly_step_run_plr_dirt",
            "fly_step_run_plr_grass",
            "fly_step_run_plr_metal",
            "fly_step_run_npc_metal",
            "fly_land_plr_concrete",
        ]
        .contains(&name)
    }

    #[test]
    fn a_surface_bo2_has_plays_its_own_step() {
        let got = t6_step_alias(bo2_bank, T6StepKind::Step(StepGait::Run), 13, true);
        assert_eq!(got, Some(("fly_step_run_plr_metal".to_owned(), "metal")));
        let got = t6_step_alias(bo2_bank, T6StepKind::Step(StepGait::Run), 13, false);
        assert_eq!(got, Some(("fly_step_run_npc_metal".to_owned(), "metal")));
    }

    #[test]
    fn a_surface_bo2_lacks_plays_the_nearest_not_the_default() {
        // Black Ops' tall grass (30) walks as grass, not as MW2's slush.
        let got = t6_step_alias(bo2_bank, T6StepKind::Step(StepGait::Run), 30, true);
        assert_eq!(got, Some(("fly_step_run_plr_grass".to_owned(), "grass")));
        // Plaster (16) as concrete; painted metal (28) as metal.
        let got = t6_step_alias(bo2_bank, T6StepKind::Step(StepGait::Run), 16, true);
        assert_eq!(got.map(|g| g.1), Some("concrete"));
        let got = t6_step_alias(bo2_bank, T6StepKind::Step(StepGait::Run), 28, true);
        assert_eq!(got.map(|g| g.1), Some("metal"));
        // Unknown or default ground: concrete.
        let got = t6_step_alias(bo2_bank, T6StepKind::Step(StepGait::Run), 0, true);
        assert_eq!(got.map(|g| g.1), Some("concrete"));
        let got = t6_step_alias(bo2_bank, T6StepKind::Land, 40, true);
        assert_eq!(got, Some(("fly_land_plr_concrete".to_owned(), "concrete")));
    }

    #[test]
    fn nothing_in_the_bank_plays_nothing() {
        assert_eq!(t6_step_alias(|_| false, T6StepKind::Land, 5, true), None);
    }
}
