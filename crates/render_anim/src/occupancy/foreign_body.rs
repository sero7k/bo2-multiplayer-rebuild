//! bo2mp: on a map from another game, the players are that game's soldiers.
//!
//! A Modern Warfare 2, Modern Warfare 3 or Black Ops map's zone carries its
//! own two teams' soldiers (its `mp_body_*` / `*_mp_body_*` bodies and
//! heads, the engine's soldier kits); under Black Ops II's rules each
//! player wears his team's (the map's allies kit for BO2's allies, its axis
//! kit for BO2's axis) instead of the BO2 faction character, and moves by
//! that game's own player animations.
//!
//! What still comes from Black Ops II: the choice of animation (BO2's sim
//! picks it from `mp/playeranim.script` and sends it in `legs_anim` /
//! `torso_anim`), which is played as the map game's nearest animation (the
//! same name where it has one, else the closest by stance, movement,
//! direction and weapon: [`map_anims`]); the gun in his hands (a BO2
//! weapon); and the draw, by BO2's fallback pass (`t6_body`), which reads
//! another game's `mc_*` model materials and lights them with the map's
//! light grid (`foreign_t6_lighting`).
//!
//! Exactly one path draws a player: this is the BO2 body path wearing
//! another game's soldier; the engine's own MW2 remote bodies stay off under
//! BO2's rules (they read `legs_anim` / `torso_anim` as MW2 animation-tree
//! nodes, which BO2's numbers are not: see `body_path`).

use asset_core::AssetNamespace;

/// A game's name as the logs give it.
pub fn game_label(ns: Option<AssetNamespace>) -> &'static str {
    match ns {
        Some(AssetNamespace::Iw4) => "MW2",
        Some(AssetNamespace::Iw5) => "MW3",
        Some(AssetNamespace::T5) => "BO1",
        Some(AssetNamespace::T6) => "BO2",
        Some(AssetNamespace::T4) => "WaW",
        Some(AssetNamespace::Iw3) => "CoD4",
        Some(AssetNamespace::Iw6) => "Ghosts",
        None => "unknown",
    }
}

/// One team's soldier: body model and head (models in the body catalog).
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ForeignKit {
    pub body: String,
    pub head: Option<String>,
}

impl ForeignKit {
    pub fn label(&self) -> String {
        format!("{}+{}", self.body, self.head.as_deref().unwrap_or("-"))
    }
}

/// How the players are drawn this match.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum BodyStyle {
    /// Black Ops II's faction characters and animations (a BO2 map, or no
    /// other game's soldiers to wear).
    Bo2,
    /// The map game's soldiers: the kits, animated by `anim_ns`'s player
    /// animations (BO2's where none map).
    Foreign {
        allies: ForeignKit,
        axis: ForeignKit,
        anim_ns: AssetNamespace,
    },
}

/// The match's choice and why (fallbacks said in `notes`).
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct BodyPlan {
    pub map_game: Option<AssetNamespace>,
    pub style: BodyStyle,
    pub notes: Vec<String>,
}

impl BodyPlan {
    /// The match's log line: "bo2mp bodies: map game X → body path Y (allies kit .., axis kit ..)".
    pub fn line(&self) -> String {
        let game = game_label(self.map_game);
        let (path, kits) = match &self.style {
            BodyStyle::Bo2 => ("BO2 characters + BO2 animations (bo2 body path)".to_owned(), "BO2 faction scripts' characters".to_owned()),
            BodyStyle::Foreign { allies, axis, anim_ns } => (
                format!(
                    "{} soldiers + {} animations (bo2 body path, BO2 draw, map light grid)",
                    game,
                    game_label(Some(*anim_ns))
                ),
                format!("allies kit {}, axis kit {}", allies.label(), axis.label()),
            ),
        };
        let notes = if self.notes.is_empty() {
            String::new()
        } else {
            format!("; {}", self.notes.join("; "))
        };
        format!("bo2mp bodies: map game {game} → body path {path} ({kits}){notes}")
    }
}

/// The animation namespaces to try for a map game's soldiers, nearest
/// first: its own, then the nearest loaded game's (MW3's skeleton is MW2's;
/// Black Ops, World at War and Black Ops II share Treyarch's; CoD4's is
/// MW2's).
fn anim_order(game: AssetNamespace) -> &'static [AssetNamespace] {
    use AssetNamespace::*;
    match game {
        Iw4 => &[Iw4],
        Iw5 => &[Iw5, Iw4],
        T5 => &[T5, T6],
        T4 => &[T4, T5, T6],
        Iw3 => &[Iw3, Iw4],
        T6 => &[T6],
        Iw6 => &[Iw6, Iw5, Iw4],
    }
}

/// The choice. `kits`: the map zone's soldier kits (allies, axis) as the
/// body catalog chose them; `player_anims(ns)`: how many player animations
/// (`pb_*`) of that game the match loaded. `force_bo2`: `BO2MP_BODIES=bo2`.
pub fn choose_plan(
    map_game: Option<AssetNamespace>,
    t6_rules: bool,
    kits: (Option<ForeignKit>, Option<ForeignKit>),
    player_anims: impl Fn(AssetNamespace) -> usize,
    force_bo2: bool,
) -> BodyPlan {
    let mut notes = Vec::new();
    let bo2 = |notes: Vec<String>| BodyPlan {
        map_game,
        style: BodyStyle::Bo2,
        notes,
    };
    let Some(game) = map_game else {
        return bo2(vec!["map game unknown: BO2 characters".to_owned()]);
    };
    if game == AssetNamespace::T6 || !t6_rules {
        return bo2(notes);
    }
    if force_bo2 {
        return bo2(vec!["BO2MP_BODIES=bo2: BO2 characters on every map".to_owned()]);
    }
    let (allies, axis) = match kits {
        (Some(a), Some(x)) => (a, x),
        (Some(a), None) => {
            notes.push(format!("{} map has one team's soldier only: both teams wear {}", game_label(Some(game)), a.body));
            (a.clone(), a)
        }
        (None, Some(x)) => {
            notes.push(format!("{} map has one team's soldier only: both teams wear {}", game_label(Some(game)), x.body));
            (x.clone(), x)
        }
        (None, None) => {
            let why = match game {
                AssetNamespace::Iw3 | AssetNamespace::T4 => format!(
                    "FALLBACK: {} soldiers are not read yet (its reader takes the world and guns only), and MW2's soldiers live in MW2 map zones, not loaded here: BO2 characters instead",
                    game_label(Some(game))
                ),
                _ => format!(
                    "FALLBACK: the {} map's zone brought no soldier kit: BO2 characters instead",
                    game_label(Some(game))
                ),
            };
            return bo2(vec![why]);
        }
    };
    let order = anim_order(game);
    let anim_ns = order.iter().copied().find(|ns| player_anims(*ns) > 0);
    let Some(anim_ns) = anim_ns else {
        return bo2(vec![format!(
            "FALLBACK: no {} player animations loaded (nor {}): BO2 characters instead",
            game_label(Some(game)),
            order[1..].iter().map(|ns| game_label(Some(*ns))).collect::<Vec<_>>().join(", ")
        )]);
    };
    if anim_ns != game {
        notes.push(format!(
            "FALLBACK: {} player animations not loaded, its soldiers move by {}'s",
            game_label(Some(game)),
            game_label(Some(anim_ns))
        ));
    }
    BodyPlan {
        map_game,
        style: BodyStyle::Foreign { allies, axis, anim_ns },
        notes,
    }
}

/// Whether a name is a player animation another game could play for one
/// of BO2's (`pb_*` body, `pt_*` torso, `p_*`, `mp_mantle_*`).
pub fn is_player_anim(name: &str) -> bool {
    name.starts_with("pb_") || name.starts_with("pt_") || name.starts_with("p_") || name.starts_with("mp_mantle")
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum Stance {
    Stand,
    Crouch,
    Prone,
    LastStand,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum Kind {
    Idle,
    Walk,
    Run,
    Sprint,
    Crawl,
    Turn,
    Transition,
    Takeoff,
    Land,
    Dive,
    Climb,
    Mantle,
    Death,
    Pain,
    Stumble,
    Shellshock,
    Fire,
    Reload,
    Melee,
    Throw,
    Prime,
    Raise,
    Putaway,
    Plant,
    Vehicle,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum Dir {
    F,
    B,
    L,
    R,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum Weap {
    Pistol,
    Rpg,
    Akimbo,
    Shield,
    Mg,
    Grenade,
    Knife,
    Item,
    Sniper,
}

#[derive(Clone, Debug)]
struct Feats {
    torso: bool,
    stance: Stance,
    kind: Kind,
    dir: Option<Dir>,
    weapon: Option<Weap>,
    ads: bool,
    /// `crouch2prone` and the like.
    transition: Option<String>,
    tokens: Vec<String>,
}

/// Splits `90left` / `l90` into `90`, `left`.
fn tokens_of(name: &str) -> Vec<String> {
    let mut out = Vec::new();
    for raw in name.split('_').filter(|t| !t.is_empty()) {
        // A transition (`crouch2prone`) stays whole.
        if is_transition(raw) {
            out.push(raw.to_owned());
            continue;
        }
        let mut cur = String::new();
        let mut digit = None;
        for ch in raw.chars() {
            let d = ch.is_ascii_digit();
            if digit.is_some_and(|was| was != d) && !cur.is_empty() {
                out.push(std::mem::take(&mut cur));
            }
            digit = Some(d);
            cur.push(ch);
        }
        if !cur.is_empty() {
            out.push(cur);
        }
    }
    out
}

fn is_transition(token: &str) -> bool {
    let Some((a, b)) = token.split_once('2') else { return false };
    a.len() >= 3 && b.len() >= 3 && a.chars().all(|c| c.is_ascii_alphabetic()) && b.chars().all(|c| c.is_ascii_alphabetic())
}

fn feats(name: &str) -> Feats {
    let name = name.to_ascii_lowercase();
    let torso = name.starts_with("pt_");
    let body = name
        .strip_prefix("pb_")
        .or_else(|| name.strip_prefix("pt_"))
        .or_else(|| name.strip_prefix("p_"))
        .unwrap_or(&name);
    let tokens = tokens_of(body);
    let has = |t: &str| tokens.iter().any(|x| x == t);
    let any = |ts: &[&str]| tokens.iter().any(|x| ts.contains(&x.as_str()));
    let starts = |p: &str| tokens.iter().any(|x| x.starts_with(p));
    let transition = tokens.iter().find(|t| is_transition(t)).cloned();
    let stance = if (starts("laststand") && transition.is_none()) || has("deadhands") {
        Stance::LastStand
    } else if transition.is_some() {
        Stance::Stand
    } else if starts("prone") {
        Stance::Prone
    } else if starts("crouch") {
        Stance::Crouch
    } else {
        Stance::Stand
    };
    let kind = if name.starts_with("mp_mantle") {
        Kind::Mantle
    } else if tokens.iter().any(|t| t.contains("death")) {
        Kind::Death
    } else if any(&["pain", "flinch", "paina", "painb"]) {
        Kind::Pain
    } else if has("stumble") {
        Kind::Stumble
    } else if has("shellshock") {
        Kind::Shellshock
    } else if any(&["climb", "climbup", "climbdown", "ladder", "zipline"]) || starts("climb") {
        Kind::Climb
    } else if any(&["huey", "gunner", "jeep", "tank"]) || starts("standmg42") || starts("pronemg42") || starts("crouchsaw") {
        Kind::Vehicle
    } else if has("dive") && has("land") {
        Kind::Land
    } else if has("dive") {
        Kind::Dive
    } else if has("takeoff") {
        Kind::Takeoff
    } else if has("land") {
        Kind::Land
    } else if any(&["reload", "reloadcrouchwalk"]) || starts("reload") {
        Kind::Reload
    } else if any(&["fire", "shoot"]) && !any(&["walk", "run"]) {
        // (MW2's `pb_stand_shoot_walk_*` are its walks while aiming.)
        Kind::Fire
    } else if any(&["melee", "swipe", "thrust", "stab", "bash"]) {
        Kind::Melee
    } else if has("throw") {
        Kind::Throw
    } else if any(&["pullpin", "prime", "pullback", "setup"]) {
        Kind::Prime
    } else if any(&["raise", "firstraise", "pullout"]) {
        Kind::Raise
    } else if any(&["putaway", "drop"]) {
        Kind::Putaway
    } else if any(&["bombplant", "plant", "defuse"]) {
        Kind::Plant
    } else if transition.as_deref().is_some_and(|t| t.ends_with("2idle"))
        || tokens.windows(2).any(|w| w[0] == "2" && w[1] == "idle")
    {
        // `run2idle`, `run_b_2_idle`: a stop.
        Kind::Idle
    } else if transition.is_some() {
        Kind::Transition
    } else if has("turn") || starts("turn") || starts("crouchturn") {
        Kind::Turn
    } else if has("sprint") {
        Kind::Sprint
    } else if has("crawl") {
        Kind::Crawl
    } else if any(&["run", "combatrun", "fast", "crouchrun", "slide"]) {
        Kind::Run
    } else if any(&["walk", "combatwalk", "shuffle", "crouchwalk"]) {
        Kind::Walk
    } else {
        Kind::Idle
    };
    let mut dir = None;
    for (i, t) in tokens.iter().enumerate() {
        let next_knife = tokens.get(i + 1).is_some_and(|n| n == "knife");
        let d = match t.as_str() {
            "forward" | "front" | "fwd" | "f" => Some(Dir::F),
            "back" | "backward" | "backwards" => Some(Dir::B),
            "b" if !next_knife => Some(Dir::B),
            "left" | "l" => Some(Dir::L),
            "right" | "r" => Some(Dir::R),
            _ => None,
        };
        if d.is_some() {
            dir = d;
            break;
        }
    }
    let weapon = if any(&["dw", "akimbo"]) {
        Some(Weap::Akimbo)
    } else if any(&["shield", "riotshield", "riot"]) {
        Some(Weap::Shield)
    } else if has("pistol") {
        Some(Weap::Pistol)
    } else if any(&["rpg", "launcher", "bazooka"]) {
        Some(Weap::Rpg)
    } else if any(&["mg", "lmg", "minigun", "saw"]) {
        Some(Weap::Mg)
    } else if any(&["knife", "tomahawk"]) && kind != Kind::Melee {
        Some(Weap::Knife)
    } else if any(&["grenade", "stickgrenade", "satchel", "c4"]) && !matches!(kind, Kind::Throw | Kind::Prime) {
        Some(Weap::Grenade)
    } else if any(&["hold", "unarmed", "briefcase", "tablet", "radio", "rc", "remotecontroller", "handheldradio"]) {
        Some(Weap::Item)
    } else if has("sniper") {
        Some(Weap::Sniper)
    } else {
        None
    };
    let ads = has("ads");
    // Words for the same thing, one word (for the tokens they share).
    let tokens = tokens
        .into_iter()
        .map(|t| match t.as_str() {
            "alert" | "cac" | "aim" | "ready" => "idle".to_owned(),
            "fire" => "shoot".to_owned(),
            "front" | "fwd" | "f" => "forward".to_owned(),
            "backward" | "backwards" => "back".to_owned(),
            "combatrun" => "run".to_owned(),
            "dw" => "akimbo".to_owned(),
            "riotshield" | "riot" => "shield".to_owned(),
            "pullout" | "firstraise" => "raise".to_owned(),
            _ => t,
        })
        .collect();
    Feats {
        torso,
        stance,
        kind,
        dir,
        weapon,
        ads,
        transition,
        tokens,
    }
}

/// The stance a transition ends in (`crouch2prone`: prone).
fn transition_end(transition: &str) -> Stance {
    let to = transition.rsplit('2').next().unwrap_or("");
    if to.starts_with("prone") {
        Stance::Prone
    } else if to.starts_with("crouch") {
        Stance::Crouch
    } else if to.starts_with("laststand") {
        Stance::LastStand
    } else {
        Stance::Stand
    }
}

/// What playing `cand` instead of `kind` costs (`None`: never).
fn kind_cost(src: Kind, cand: Kind) -> Option<f32> {
    use Kind::*;
    if src == cand {
        return Some(0.0);
    }
    let torso_actions = [Fire, Reload, Melee, Throw, Prime, Raise, Putaway, Plant];
    if src == Death || cand == Death {
        return None;
    }
    if torso_actions.contains(&src) || torso_actions.contains(&cand) {
        return match (src, cand) {
            (Throw, Prime) | (Prime, Throw) => Some(8.0),
            (Plant, Idle) => Some(20.0),
            _ => None,
        };
    }
    Some(match (src, cand) {
        (Run, Sprint) | (Sprint, Run) => 6.0,
        (Walk, Run) | (Run, Walk) => 4.0,
        (Crawl, Walk) | (Walk, Crawl) => 10.0,
        (Crawl, Run) | (Run, Crawl) => 12.0,
        (Stumble, Run) => 8.0,
        (Stumble, Walk) => 9.0,
        (Turn, Idle) => 6.0,
        (Shellshock, Idle) | (Pain, Idle) => 12.0,
        (Mantle, Climb) | (Climb, Mantle) => 12.0,
        (Dive, Land) | (Land, Dive) => 14.0,
        (Transition, Idle) | (Takeoff, Idle) | (Land, Idle) | (Dive, Idle) | (Climb, Idle) | (Mantle, Idle)
        | (Vehicle, Idle) => 25.0,
        (Stumble, Idle) | (Sprint, Idle) => 30.0,
        _ => return None,
    })
}

fn stance_cost(src: Stance, cand: Stance) -> f32 {
    if src == cand {
        0.0
    } else if src == Stance::Prone || cand == Stance::Prone || src == Stance::LastStand || cand == Stance::LastStand {
        20.0
    } else {
        12.0
    }
}

/// How well `cand` stands in for `src` (higher is better; `None`: never).
fn score(src: &Feats, src_torso: bool, cand: &Feats) -> Option<f32> {
    // A torso-only animation never drives the legs.
    if cand.torso && !src_torso {
        return None;
    }
    let mut s = -kind_cost(src.kind, cand.kind)?;
    if src_torso && !cand.torso {
        s -= 3.0;
    }
    s -= stance_cost(src.stance, cand.stance);
    match (&src.transition, &cand.transition) {
        (Some(a), Some(b)) if a == b => s += 10.0,
        (Some(_), Some(_)) => s -= 10.0,
        _ => {}
    }
    match (src.dir, cand.dir) {
        (Some(a), Some(b)) if a == b => s += 6.0,
        (Some(_), Some(_)) => s -= 10.0,
        (Some(_), None) => s -= 3.0,
        (None, Some(_)) => s -= 2.0,
        (None, None) => {}
    }
    // Another game's shield, akimbo pair, knife, grenade or carried item
    // in hands that hold none: never.
    if let Some(w) = cand.weapon
        && matches!(w, Weap::Shield | Weap::Akimbo | Weap::Knife | Weap::Grenade | Weap::Item)
        && src.weapon != Some(w)
    {
        return None;
    }
    match (src.weapon, cand.weapon) {
        (Some(a), Some(b)) if a == b => s += 4.0,
        (Some(_), Some(_)) => s -= 6.0,
        (Some(_), None) => s -= 2.0,
        (None, Some(_)) => s -= 5.0,
        (None, None) => {}
    }
    s += if src.ads == cand.ads { 1.0 } else { -1.0 };
    // Words in common count; the candidate's own other words (a special
    // pose: `remotecontroller`, `bombplant`, `chicken_dance`) count against.
    let shared = cand.tokens.iter().filter(|t| src.tokens.contains(t)).count();
    s += shared as f32 * 0.5;
    s -= (cand.tokens.len() - shared) as f32 * 0.3;
    Some(s)
}

/// BO2's animations as another game's: per entry of `script_anims` (BO2's
/// list, with whether the script plays it on the torso only), the
/// candidate of the same name, else the best stand-in, else none (that
/// part then plays BO2's own clip, or the legs' for the torso).
#[derive(Clone, Debug, Default)]
pub struct AnimMap {
    pub names: Vec<Option<String>>,
    pub exact: usize,
    pub near: usize,
    pub missing: Vec<String>,
}

pub fn map_anims(script_anims: &[(String, bool)], candidates: &[&str]) -> AnimMap {
    let cands: Vec<(&str, Feats)> = candidates
        .iter()
        .copied()
        .filter(|n| is_player_anim(n))
        .map(|n| (n, feats(n)))
        .collect();
    let have: std::collections::HashSet<&str> = cands.iter().map(|(n, _)| *n).collect();
    let mut out = AnimMap::default();
    for (name, torso_only) in script_anims {
        let lower = name.to_ascii_lowercase();
        if have.contains(lower.as_str()) {
            out.exact += 1;
            out.names.push(Some(lower));
            continue;
        }
        let mut src = feats(&lower);
        // A stance change the game has no animation for: the stance it
        // ends in.
        if let Some(t) = src.transition.clone()
            && !cands.iter().any(|(_, f)| f.transition.as_deref() == Some(t.as_str()))
        {
            src.kind = Kind::Idle;
            src.stance = transition_end(&t);
            src.transition = None;
        }
        let torso = *torso_only || src.torso;
        let best = cands
            .iter()
            .filter_map(|(n, f)| score(&src, torso, f).map(|s| (s, *n)))
            .max_by(|a, b| a.0.total_cmp(&b.0).then_with(|| b.1.cmp(a.1)));
        match best {
            Some((_, n)) => {
                out.near += 1;
                out.names.push(Some(n.to_owned()));
            }
            None => {
                out.missing.push(lower);
                out.names.push(None);
            }
        }
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    fn kit(b: &str) -> Option<ForeignKit> {
        Some(ForeignKit {
            body: b.to_owned(),
            head: Some(format!("{b}_head")),
        })
    }

    #[test]
    fn each_map_game_wears_its_own_soldiers() {
        let all = |_: AssetNamespace| 100;
        for game in [AssetNamespace::Iw4, AssetNamespace::Iw5, AssetNamespace::T5] {
            let plan = choose_plan(Some(game), true, (kit("a"), kit("x")), all, false);
            assert_eq!(
                plan.style,
                BodyStyle::Foreign {
                    allies: kit("a").unwrap(),
                    axis: kit("x").unwrap(),
                    anim_ns: game
                }
            );
            assert!(plan.notes.is_empty(), "{:?}", plan.notes);
        }
        // A BO2 map: BO2's characters.
        assert_eq!(choose_plan(Some(AssetNamespace::T6), true, (kit("a"), kit("x")), all, false).style, BodyStyle::Bo2);
        // Another game's rules: not this path's business.
        assert_eq!(choose_plan(Some(AssetNamespace::Iw4), false, (kit("a"), kit("x")), all, false).style, BodyStyle::Bo2);
        // Forced.
        assert_eq!(choose_plan(Some(AssetNamespace::Iw4), true, (kit("a"), kit("x")), all, true).style, BodyStyle::Bo2);
    }

    #[test]
    fn fallbacks_are_said() {
        // MW3 without its own player animations: MW2's.
        let mw2_only = |ns: AssetNamespace| usize::from(ns == AssetNamespace::Iw4) * 500;
        let plan = choose_plan(Some(AssetNamespace::Iw5), true, (kit("a"), kit("x")), mw2_only, false);
        assert!(matches!(plan.style, BodyStyle::Foreign { anim_ns: AssetNamespace::Iw4, .. }));
        assert!(plan.line().contains("FALLBACK"), "{}", plan.line());
        // BO1 without its own: BO2's (Treyarch's skeleton).
        let t6_only = |ns: AssetNamespace| usize::from(ns == AssetNamespace::T6) * 500;
        let plan = choose_plan(Some(AssetNamespace::T5), true, (kit("a"), kit("x")), t6_only, false);
        assert!(matches!(plan.style, BodyStyle::Foreign { anim_ns: AssetNamespace::T6, .. }));
        // CoD4 / WaW: no soldiers read: BO2's, said.
        for game in [AssetNamespace::Iw3, AssetNamespace::T4] {
            let plan = choose_plan(Some(game), true, (None, None), |_| 500, false);
            assert_eq!(plan.style, BodyStyle::Bo2);
            assert!(plan.line().contains("FALLBACK"), "{}", plan.line());
        }
        // One team's kit: both wear it.
        let plan = choose_plan(Some(AssetNamespace::Iw4), true, (kit("a"), None), |_| 500, false);
        assert!(matches!(&plan.style, BodyStyle::Foreign { allies, axis, .. } if allies == axis));
    }

    #[test]
    fn line_names_the_game_path_and_kits() {
        let plan = choose_plan(Some(AssetNamespace::Iw4), true, (kit("mp_body_tf141"), kit("mp_body_op")), |_| 5, false);
        let line = plan.line();
        assert!(line.starts_with("bo2mp bodies: map game MW2 → body path MW2 soldiers + MW2 animations"), "{line}");
        assert!(line.contains("allies kit mp_body_tf141+mp_body_tf141_head, axis kit mp_body_op+mp_body_op_head"), "{line}");
    }

    const IW4: &[&str] = &[
        "pb_stand_alert",
        "pb_stand_alert_pistol",
        "pb_stand_alert_akimbo",
        "pb_crouch_alert",
        "pb_crouch_alert_pistol",
        "pb_prone_aim",
        "pb_prone_crawl",
        "pb_prone_crawl_back",
        "pb_prone_crawl_left",
        "pb_prone_crawl_right",
        "pb_combatrun_forward_loop",
        "pb_combatrun_back_loop",
        "pb_combatrun_left_loop",
        "pb_combatrun_right_loop",
        "pb_combatrun_forward_loop_pistol",
        "pb_crouch_run_forward",
        "pb_crouch_run_back",
        "pb_crouch_walk_forward_pistol",
        "pb_stand_shoot_walk_forward",
        "pb_stand_shoot_walk_back",
        "pb_stand_shoot_walk_left",
        "pb_sprint",
        "pb_sprint_pistol",
        "pb_runjump_takeoff",
        "pb_runjump_land",
        "pb_standjump_takeoff",
        "pb_standjump_land",
        "pb_crouch2prone",
        "pb_prone2crouch",
        "pb_stand_death_chest_blowback",
        "pb_stand_death_tumbleback",
        "pb_crouch_death_fetal",
        "pb_prone_death_quickdeath",
        "pb_death_run_forward_crumple",
        "pb_climbup",
        "pb_climbdown",
        "pt_stand_shoot",
        "pt_stand_shoot_pistol",
        "pt_crouch_shoot",
        "pt_prone_shoot",
        "pt_reload_stand_rifle",
        "pt_reload_crouch_rifle",
        "pt_reload_prone_rifle",
        "pt_reload_stand_pistol",
        "pt_stand_grenade_throw",
        "pt_crouch_grenade_throw",
        "pt_melee_right2right_1",
        "pt_melee_prone",
        "pt_stand_pullout_pose",
        "pt_flinch_forward",
        "pb_stand_grenade_pullpin",
        "mp_mantle_up_51",
    ];

    fn one(name: &str, torso: bool) -> Option<String> {
        map_anims(&[(name.to_owned(), torso)], IW4).names.remove(0)
    }

    #[test]
    fn same_name_first() {
        assert_eq!(one("pb_sprint", false).as_deref(), Some("pb_sprint"));
        assert_eq!(one("mp_mantle_up_51", false).as_deref(), Some("mp_mantle_up_51"));
    }

    #[test]
    fn bo2_moves_become_mw2_moves() {
        let cases = [
            ("pb_rifle_stand_cac_idle", false, "pb_stand_alert"),
            ("pb_stand_alert_", false, "pb_stand_alert"),
            ("pb_pistol_stand_cac_idle", false, "pb_stand_alert_pistol"),
            ("pb_crouch_alert_", false, "pb_crouch_alert"),
            ("pb_prone_aim_", false, "pb_prone_aim"),
            ("pb_combatrun_forward_", false, "pb_combatrun_forward_loop"),
            ("pb_combatrun_back_", false, "pb_combatrun_back_loop"),
            ("pb_combatrun_left_", false, "pb_combatrun_left_loop"),
            ("pb_crouch_run_forward_", false, "pb_crouch_run_forward"),
            ("pb_walk_back_", false, "pb_stand_shoot_walk_back"),
            ("pb_sprint_", false, "pb_sprint"),
            ("pb_smg_sprint", false, "pb_sprint"),
            ("pb_prone_crawl_left", false, "pb_prone_crawl_left"),
            ("pb_dw_run_forward", false, "pb_combatrun_forward_loop"),
            ("pb_rifle_stand_run2idle", false, "pb_stand_alert"),
            ("pb_hold_crouch2prone", false, "pb_crouch2prone"),
            ("pb_rpg_standjump_takeoff", false, "pb_standjump_takeoff"),
            ("pb_knife_runjump_land", false, "pb_runjump_land"),
            ("pb_riot_climbup", false, "pb_climbup"),
            ("pb_turn_90left", false, "pb_stand_alert"),
            ("pb_crouchturn_90right", false, "pb_crouch_alert"),
        ];
        for (src, torso, want) in cases {
            assert_eq!(one(src, torso).as_deref(), Some(want), "{src}");
        }
    }

    #[test]
    fn deaths_stay_deaths_and_torso_actions_stay_torso() {
        let death = one("pb_death_headshot_back", false).unwrap();
        assert!(death.contains("death"), "{death}");
        let death = one("pb_crouch_death_headshot_left", false).unwrap();
        assert!(death.starts_with("pb_crouch_death"), "{death}");
        let death = one("pb_prone_death_quickdeath_v2", false).unwrap();
        assert_eq!(death, "pb_prone_death_quickdeath");
        assert_eq!(one("pt_stand_shoot_auto_", true).as_deref(), Some("pt_stand_shoot"));
        assert_eq!(one("pt_reload_prone_", true).as_deref(), Some("pt_reload_prone_rifle"));
        assert_eq!(one("pt_pistol_crouch_reload", true).as_deref(), Some("pt_reload_crouch_rifle"));
        assert_eq!(one("pb_rifle_stand_run_b_2_idle", false).as_deref(), Some("pb_stand_alert"));
        assert_eq!(one("pt_grenade_stand_throw", true).as_deref(), Some("pt_stand_grenade_throw"));
        assert_eq!(one("pt_knife_stand_melee_swipe", true).as_deref(), Some("pt_melee_right2right_1"));
        assert_eq!(one("pt_pistol_stand_raise", true).as_deref(), Some("pt_stand_pullout_pose"));
        // A torso-only animation never drives the legs; a legs one with no
        // stand-in stays unmapped (BO2's own then plays).
        assert!(score(&feats("pb_stand_shoot_walk_forward"), false, &feats("pt_stand_shoot")).is_none());
    }

    #[test]
    fn tokens_split_digits_and_keep_transitions() {
        assert_eq!(tokens_of("crouchturn_90left"), ["crouchturn", "90", "left"]);
        assert_eq!(tokens_of("stand_turn_l90"), ["stand", "turn", "l", "90"]);
        assert_eq!(tokens_of("hold_crouch2prone"), ["hold", "crouch2prone"]);
        assert_eq!(feats("pb_b_knife_run_back").dir, Some(Dir::B));
        assert_eq!(feats("pb_b_knife_run_back").weapon, Some(Weap::Knife));
        assert_eq!(feats("pb_b_knife_stand_idle").dir, None);
    }
}

#[cfg(test)]
mod probe_tmp {
    #[test]
    fn probe_real_lists() {
        let Ok(dir) = std::env::var("PROBE_DIR") else { return };
        let read = |f: &str| std::fs::read_to_string(format!("{dir}/{f}")).unwrap();
        let script: Vec<(String, bool)> = read("t6_script.anims").lines().map(|l| (l.to_owned(), l.starts_with("pt_"))).collect();
        for g in ["iw4", "t5"] {
            let c = read(&format!("{g}.anims"));
            let cands: Vec<&str> = c.lines().collect();
            let m = super::map_anims(&script, &cands);
            let mut out = String::new();
            for ((s, _), n) in script.iter().zip(&m.names) {
                out += &format!("{s} -> {}\n", n.as_deref().unwrap_or("-"));
            }
            std::fs::write(format!("{dir}/map_{g}.txt"), out).unwrap();
            println!("{g}: exact {} near {} missing {}", m.exact, m.near, m.missing.len());
        }
    }
}
