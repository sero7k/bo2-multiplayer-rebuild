//! ZBarriers: a map entity made of pieces (the magic box: the fake box,
//! the box, its lid, the weapon riser and the teddy), each a model with
//! two animations from the entity's own keys (`zbarrierboardmodelN`,
//! `zbarriertearanimN` = opening, `zbarrierboardanimN` = closing). A
//! piece's state is "closed", "opening", "open" or "closing"; opening and
//! closing play their animation and end in open / closed. Scripts number
//! pieces from 0, the keys from 1.

use bevy_ecs::prelude::World;
use gsc_t6::{Value, Vm};

use super::{Zm, arg, entnum, list};

#[derive(Clone, Debug, Default)]
pub(crate) struct Piece {
    pub model: String,
    pub open_anim: String,
    pub close_anim: String,
    pub state: String,
    pub shown: bool,
    /// The playing animation: (clip, start ms, length ms).
    pub playing: Option<(String, i64, i64)>,
    /// The engine shows cycling weapons on this piece while it opens.
    pub box_rise: bool,
    /// While a box-rise piece opens: the box's guns it cycles through
    /// (the gun, the left-hand gun for two-gun pistols), each with its parts.
    pub rise: Vec<(super::WeaponLook, Option<super::WeaponLook>)>,
}

#[derive(Clone, Debug, Default)]
pub(crate) struct ZBarrier {
    pub pieces: Vec<Piece>,
    /// The barrier type its `type` key names (window boards' tear states).
    pub def: Option<std::sync::Arc<super::T6ZBarrierDef>>,
}

/// The barrier type a zbarrier entity's `type` key names, if the zones
/// hold it.
pub(crate) fn def_for(
    world: &World,
    keys: &[(String, String)],
) -> Option<std::sync::Arc<super::T6ZBarrierDef>> {
    let ty = keys
        .iter()
        .find(|(n, _)| n.eq_ignore_ascii_case("type"))?
        .1
        .to_ascii_lowercase();
    let defs = &world.resource::<Zm>().zbarrier_defs;
    // The engine builds a zbarrier from its classname ("zbarrier_<def>");
    // the "type" key is only the script's name for it. TranZit's bus
    // windows say type zb_BusWindow1 but are class zbarrier_zb_BusWindow5.
    let class = keys
        .iter()
        .find(|(n, _)| n.eq_ignore_ascii_case("classname"))
        .and_then(|(_, v)| v.to_ascii_lowercase().strip_prefix("zbarrier_").map(str::to_owned));
    let ty = match class {
        Some(c) if defs.contains_key(&c) => c,
        _ => ty,
    };
    if let Some(def) = defs.get(&ty) {
        diag::info!(
            Sim,
            "bo2zm t6: zbarrier type {ty}: auto_hide {} taunts {} reach_through {} boards {}",
            def.auto_hide_open_pieces,
            def.taunts,
            def.reach_through,
            def.boards.len()
        );
        return Some(def.clone());
    }
    // A map may name another map's type (Verrückt's wall
    // windows ask for zmcore_prototype_wallbarrier; its zone holds only
    // zmcore_asylum_wallbarrier): take the one zone type of the same kind.
    let kind = |n: &str| n.rsplit('_').next().unwrap_or_default().to_owned();
    let same: Vec<_> = defs.iter().filter(|(n, _)| kind(n) == kind(&ty)).collect();
    if let [(name, def)] = same.as_slice() {
        diag::info!(
            Sim,
            "bo2zm t6: zbarrier type {ty} not in the zones, using {name}"
        );
        return Some((*def).clone());
    }
    diag::warn!(Sim, "bo2zm t6: zbarrier type {ty} not in the zones");
    None
}

impl ZBarrier {
    /// From the map entity's keys (lower-cased names).
    pub(crate) fn from_keys(keys: &[(String, String)]) -> Self {
        let get = |k: &str| {
            keys.iter()
                .find(|(n, _)| n.eq_ignore_ascii_case(k))
                .map(|(_, v)| v.clone())
                .unwrap_or_default()
        };
        let n: usize = get("zbarriernumboards").trim().parse().unwrap_or(0);
        let pieces = (1..=n)
            .map(|i| Piece {
                model: get(&format!("zbarrierboardmodel{i}")),
                open_anim: get(&format!("zbarriertearanim{i}")).to_ascii_lowercase(),
                close_anim: get(&format!("zbarrierboardanim{i}")).to_ascii_lowercase(),
                state: "closed".to_owned(),
<<<<<<< /home/claude/bo2-multiplayer-rebuild/crates/sim/src/t6/zbarrier.rs
                shown: false,
=======
                // Pieces start shown, as in the game: window boards stand
                // in their windows; the box's scripts hide the pieces
                // they don't want.
                shown: true,
>>>>>>> /tmp/claude-0/zmhead/crates/sim/src/t6/zbarrier.rs
                playing: None,
                box_rise: false,
                rise: Vec::new(),
            })
            .collect();
<<<<<<< /home/claude/bo2-multiplayer-rebuild/crates/sim/src/t6/zbarrier.rs
        Self { pieces }
=======
        Self { pieces, def: None }
>>>>>>> /tmp/claude-0/zmhead/crates/sim/src/t6/zbarrier.rs
    }
}

fn anim_ms(world: &World, anim: &str) -> i64 {
    world
        .resource::<Zm>()
        .anims
        .get(anim)
        .map_or(1000, super::actors::anim_length_ms)
}

/// A state's time scale (the scripts' third argument: Speed Cola's
/// 0.31 / 0.2112 repairs): BO2 keeps it in a byte between 0.1 and 2.0 and
/// plays the clip over its length times it.
fn scalar(v: &Value) -> f32 {
    let s = v.as_float().unwrap_or(1.0).clamp(0.1, 2.0);
    ((s - 0.1) / 1.9 * 255.0).round() / 255.0 * 1.9 + 0.1
}

/// Each tick: an opening or closing piece whose animation ended is open or
/// closed.
pub(crate) fn advance(world: &mut World, now: i64) {
    // IW4L_T6_ZBLOG: every 10 s, each zbarrier's shown pieces and states.
    if std::env::var("IW4L_T6_ZBLOG").is_ok() && now % 10_000 < i64::from(crate::MATCH_TICK_MS) {
        let zm = world.resource::<Zm>();
        for (n, e) in &zm.ents {
            let Some(z) = &e.zbarrier else { continue };
            let pieces: Vec<String> = z
                .pieces
                .iter()
                .enumerate()
                .map(|(i, p)| {
                    format!(
                        "{i}:{}{}({})",
                        p.state,
                        if p.shown { "*" } else { "" },
                        p.model
                    )
                })
                .collect();
            diag::info!(
                Sim,
<<<<<<< /home/claude/bo2-multiplayer-rebuild/crates/sim/src/t6/zbarrier.rs
                "bo2zm t6 zbarrier ent{n} at ({:.0} {:.0} {:.0}): {}",
=======
                "bo2zm t6 zbarrier ent{n} at ({:.0} {:.0} {:.0}) hide_open={:?}: {}",
>>>>>>> /tmp/claude-0/zmhead/crates/sim/src/t6/zbarrier.rs
                e.origin[0],
                e.origin[1],
                e.origin[2],
                z.def.as_ref().map(|d| d.auto_hide_open_pieces),
                pieces.join(" ")
            );
        }
    }
    let mut zm = world.resource_mut::<Zm>();
    for e in zm.ents.values_mut() {
        let Some(z) = e.zbarrier.as_mut() else {
            continue;
        };
        for p in &mut z.pieces {
            let Some((_, start, len)) = &p.playing else {
                continue;
            };
            if now - start >= *len {
                match p.state.as_str() {
                    "opening" => p.state = "open".to_owned(),
                    "closing" => p.state = "closed".to_owned(),
                    _ => {}
                }
            }
        }
    }
}

<<<<<<< /home/claude/bo2-multiplayer-rebuild/crates/sim/src/t6/zbarrier.rs
/// What a piece shows: (model, clip, fraction, rate) at `now`.
pub(crate) fn piece_pose(p: &Piece, now: i64) -> (String, Option<(String, f32, f32)>) {
    let anim = p.playing.as_ref().map(|(clip, start, len)| {
        let frac = ((now - start) as f32 / (*len).max(1) as f32).clamp(0.0, 1.0);
        let rate = if frac >= 1.0 {
=======
/// A clip held at its end: just short of 1.0, so a held pose never lands
/// on a wrap back to the clip's first frame.
const HELD_END: f32 = 0.999;

/// What a piece shows: (model, clip, fraction, rate) at `now`. A window
/// board (`boards`: its barrier type hides torn pieces) that is not moving
/// holds its repair animation's last frame when closed: the board in the
/// window, not the model's rest pose under it.
pub(crate) fn piece_pose(
    p: &Piece,
    now: i64,
    boards: bool,
) -> (String, Option<(String, f32, f32)>) {
    if p.playing.is_none() && boards && p.state == "closed" && !p.close_anim.is_empty() {
        return (p.model.clone(), Some((p.close_anim.clone(), HELD_END, 0.0)));
    }
    let anim = p.playing.as_ref().map(|(clip, start, len)| {
        let frac = ((now - start) as f32 / (*len).max(1) as f32).clamp(0.0, HELD_END);
        let rate = if frac >= HELD_END {
>>>>>>> /tmp/claude-0/zmhead/crates/sim/src/t6/zbarrier.rs
            0.0
        } else {
            1000.0 / (*len).max(1) as f32
        };
        (clip.clone(), frac, rate)
    });
    (p.model.clone(), anim)
}

/// The gun a rising box piece shows at `now` and where (its offset from
/// the box, in the box's frame), or none. BO2's engine spins the box's guns
/// on its weapon-rise pieces (the scripts' `zbarrierpieceuseboxriselogic`)
/// while they open: the piece's opening clip (o_zombie_magic_box_weapon_rise,
/// 3.9 s) carries the rise in its root motion and a "switch" note at each
/// change of gun (fast, then slower: the server's own 0.05 / 0.1 / 0.2 /
/// 0.3 s waits before it picks). The two-gun piece (the dual rise) shows
/// only the left gun of a two-gun pistol. Each gun wears its parts
/// (magazine, scope, rocket), as BO2's box guns do.
pub(crate) fn rise_gun(
    p: &Piece,
    anim: Option<&super::T6Anim>,
    now: i64,
) -> Option<(super::WeaponLook, [f32; 3])> {
    let (_, start, len) = p.playing.as_ref()?;
    if !p.box_rise || !p.shown || p.state != "opening" || p.rise.is_empty() {
        return None;
    }
    let frac = ((now - start) as f32 / (*len).max(1) as f32).clamp(0.0, 1.0);
    let step = anim.map_or(0, |a| {
        a.notifies
            .iter()
            .filter(|(n, t)| n == "switch" && *t <= frac)
            .count()
    });
    // A fixed shuffle per opening (its start time), so every tick of one
    // step agrees.
    let mut h = (*start as u64) ^ (step as u64).wrapping_mul(0x9e37_79b9_7f4a_7c15);
    h ^= h >> 33;
    h = h.wrapping_mul(0xff51_afd7_ed55_8ccd);
    h ^= h >> 33;
    let (gun, left) = &p.rise[(h % p.rise.len() as u64) as usize];
    let model = if p.open_anim.contains("dual") {
        left.clone()?
    } else {
        gun.clone()
    };
    let offset = anim.map_or([0.0, 0.0, 40.0 * frac], |a| root_offset(a, frac));
    Some((model, offset))
}

/// A clip's root motion at `frac` (keys by frame, straight between): where
/// it has moved its model from the entity, in the entity's frame.
pub(crate) fn root_offset(a: &super::T6Anim, frac: f32) -> [f32; 3] {
    let f = frac * f32::from(a.numframes);
    let keys = &a.delta_trans;
    let after = keys.iter().position(|(k, _)| f32::from(*k) >= f);
    match after {
        None => keys.last().map_or([0.0; 3], |k| k.1),
        Some(0) => keys.first().map_or([0.0; 3], |k| k.1),
        Some(i) => {
            let ((k0, a0), (k1, a1)) = (keys[i - 1], keys[i]);
            let t = (f - f32::from(k0)) / f32::from(k1.saturating_sub(k0).max(1));
            std::array::from_fn(|c| a0[c] + (a1[c] - a0[c]) * t)
        }
    }
}

/// The clip's root turn at `frac` (quaternion x y z w, unit length): the
/// keys around it blended (the shorter way), or the one key, or none.
pub(crate) fn root_turn(a: &super::T6Anim, frac: f32) -> Option<[f32; 4]> {
    let keys = &a.delta_quat;
    let f = frac * f32::from(a.numframes);
    let q = match keys.iter().position(|(k, _)| f32::from(*k) >= f) {
        None => keys.last()?.1,
        Some(0) => keys.first()?.1,
        Some(i) => {
            let ((k0, q0), (k1, q1)) = (keys[i - 1], keys[i]);
            let t = (f - f32::from(k0)) / f32::from(k1.saturating_sub(k0).max(1));
            let sign = if q0.iter().zip(q1).map(|(a, b)| a * b).sum::<f32>() < 0.0 {
                -1.0
            } else {
                1.0
            };
            std::array::from_fn(|c| q0[c] + (sign * q1[c] - q0[c]) * t)
        }
    };
    let len = q.iter().map(|c| c * c).sum::<f32>().sqrt();
    (len > 1e-4).then(|| q.map(|c| c / len))
}

/// How a piece is turned at `now`: BO2 turns it by its clip's root turn as
/// it moves it by the root motion (`piece_origin`); a window board's pull
/// clip spins the plank as it throws it out. The barrier's own angles when
/// the clip does not turn its root.
pub(crate) fn piece_angles(
    zm: &Zm,
    e: &super::Ent,
    z: &ZBarrier,
    p: &Piece,
    now: i64,
) -> [f32; 3] {
    let Some((clip, frac, _)) = piece_pose(p, now, has_boards(z)).1 else {
        return e.angles;
    };
    match zm.anims.get(&clip).and_then(|a| root_turn(a, frac)) {
        Some(q) => turned(e.angles, q),
        None => e.angles,
    }
}

/// `angles` turned by `q` (x y z w) in its own frame: the turn's local axes
/// (its rotation's columns) placed on forward, left and up.
fn turned(angles: [f32; 3], [x, y, z, w]: [f32; 4]) -> [f32; 3] {
    let cols = [
        [
            1.0 - 2.0 * (y * y + z * z),
            2.0 * (x * y + w * z),
            2.0 * (x * z - w * y),
        ],
        [
            2.0 * (x * y - w * z),
            1.0 - 2.0 * (x * x + z * z),
            2.0 * (y * z + w * x),
        ],
        [
            2.0 * (x * z + w * y),
            2.0 * (y * z - w * x),
            1.0 - 2.0 * (x * x + y * y),
        ],
    ];
    let axis = math_iw4::angles_to_axis(angles);
    let world = |c: [f32; 3]| -> [f32; 3] {
        std::array::from_fn(|i| c[0] * axis[0][i] + c[1] * axis[1][i] + c[2] * axis[2][i])
    };
    math_iw4::axis_to_angles([world(cols[0]), world(cols[1]), world(cols[2])])
}

/// Window boards (a barrier type zombies tear at). Nacht's wall barrier
/// keeps torn boards on show, still boards.
pub(crate) fn has_boards(z: &ZBarrier) -> bool {
    z.def
        .as_ref()
        .is_some_and(|d| d.auto_hide_open_pieces || d.taunts || d.reach_through)
}

/// Where a piece stands at `now`: BO2 draws every piece at its barrier and
/// lets its clip's root motion carry it. A window board's repair clip flies
/// it from the ground outside (about 98 units out) into its own slot in
/// the window, and its pull clip throws it back out; the board model is one
/// plank at its origin, so without the root motion all six sat on one spot.
pub(crate) fn piece_origin(
    zm: &Zm,
    e: &super::Ent,
    z: &ZBarrier,
    p: &Piece,
    now: i64,
) -> [f32; 3] {
    let Some((clip, frac, _)) = piece_pose(p, now, has_boards(z)).1 else {
        return e.origin;
    };
    let Some(off) = zm
        .anims
        .get(&clip)
        .filter(|a| !a.delta_trans.is_empty())
        .map(|a| root_offset(a, frac))
    else {
        return e.origin;
    };
    let (f, r, u) = gsc_t6::math::angle_vectors(e.angles);
    std::array::from_fn(|c| e.origin[c] + f[c] * off[0] - r[c] * off[1] + u[c] * off[2])
}

/// The box's guns (`level.zombie_weapons` marked `is_in_box`): world
/// models with their parts, and the left gun for a two-gun pistol.
fn box_guns(
    vm: &mut Vm<World>,
    world: &mut World,
) -> Vec<(super::WeaponLook, Option<super::WeaponLook>)> {
    let (all_f, in_box_f) = (vm.intern("zombie_weapons"), vm.intern("is_in_box"));
    let Value::Array(all) = vm.raw_field(vm.level, all_f) else {
        return Vec::new();
    };
    let all = all.snapshot();
    let names: Vec<String> = all
        .keys()
        .filter_map(|k| {
            let gsc_t6::Key::Str(name) = k else {
                return None;
            };
            let Some(Value::Object(o)) = all.get(&k) else {
                return None;
            };
            vm.raw_field(*o, in_box_f)
                .as_int()
                .is_some_and(|v| v != 0)
                .then(|| vm.str(name).to_owned())
        })
        .collect();
    let mut out = Vec::new();
    for name in names {
        let Ok(w) = super::weapon(world, &name) else {
            continue;
        };
        let Some(gun) = super::weapon_look(world, w) else {
            continue;
        };
        let left = super::frame(world)
            .combat_facts_for(w)
            .map_or(0, |c| c.dual_wield_weapon);
        let left = (left != 0).then(|| super::weapon_look(world, left)).flatten();
        out.push((gun, left));
    }
    out
}

fn with_piece<R>(
    vm: &Vm<World>,
    world: &mut World,
    s: &Value,
    a: &[Value],
    f: impl FnOnce(&mut Piece) -> R,
) -> Option<R> {
    let n = entnum(vm, s)?;
    let i = arg(a, 0).as_int()?;
    let mut zm = world.resource_mut::<Zm>();
    let z = zm.ents.get_mut(&n)?.zbarrier.as_mut()?;
    z.pieces.get_mut(usize::try_from(i).ok()?).map(f)
}

pub(super) fn bind(vm: &mut Vm<World>) {
    macro_rules! m {
        ($name:literal, $body:expr) => {
            vm.bind($name, true, $body);
        };
    }
    m!("iszbarrier", |vm, world, s, _| {
        let is = entnum(vm, s).is_some_and(|n| {
            world
                .resource::<Zm>()
                .ents
                .get(&n)
                .is_some_and(|e| e.zbarrier.is_some())
        });
        Ok(Value::bool(is))
    });
    // Every zbarrier in the level, in entity order (Tranzit's survival
    // start hides the bus's window boards through it).
    vm.bind("getzbarrierarray", false, |vm, world, _, _| {
        let mut found: Vec<(u32, gsc_t6::ObjRef)> = world
            .resource::<Zm>()
            .ents
            .iter()
            .filter(|(_, e)| e.zbarrier.is_some())
            .filter_map(|(n, e)| e.obj.map(|o| (*n, o)))
            .filter(|(_, o)| vm.alive(*o))
            .collect();
        found.sort_by_key(|(n, _)| *n);
        Ok(list(
            found.into_iter().map(|(_, o)| Value::Object(o)).collect(),
        ))
    });
    m!("getnumzbarrierpieces", |vm, world, s, _| {
        let n = entnum(vm, s)
            .and_then(|n| {
                world
                    .resource::<Zm>()
                    .ents
                    .get(&n)
                    .and_then(|e| e.zbarrier.as_ref().map(|z| z.pieces.len()))
            })
            .unwrap_or(0);
        Ok(Value::Int(n as i32))
    });
    m!("setzbarrierpiecestate", |vm, world, s, a| {
        let state = vm.to_text(arg(a, 1));
        let now = world.resource::<Zm>().now_ms;
        let anims = with_piece(vm, world, s, a, |p| {
            (p.open_anim.clone(), p.close_anim.clone())
        });
        let Some((open, close)) = anims else {
            return Ok(Value::Undefined);
        };
        let k = scalar(arg(a, 2));
        let scaled = |ms: i64| ((ms as f32 * k) as i64).max(1);
        let (open_ms, close_ms) = (scaled(anim_ms(world, &open)), scaled(anim_ms(world, &close)));
        // IW4L_T6_REPAIR (use_test.rs): each board BO2's repair loop puts back.
        if state == "closing" && std::env::var_os("IW4L_T6_REPAIR").is_some() {
            diag::info!(
                Sim,
                "bo2zm t6 repair: ent{:?} piece {} closing at t{now} over {close_ms} ms ({close})",
                entnum(vm, s),
                vm.to_text(arg(a, 0))
            );
        }
        let rises = state == "opening" && with_piece(vm, world, s, a, |p| p.box_rise) == Some(true);
        let guns = if rises {
            box_guns(vm, world)
        } else {
            Vec::new()
        };
        if rises {
            diag::info!(
                Sim,
                "bo2zm t6 box spin: {} guns {:?}",
                guns.len(),
                guns.iter()
                    .take(4)
                    .map(|(g, _)| (g.model.as_str(), g.parts.len()))
                    .collect::<Vec<_>>()
            );
        }
        with_piece(vm, world, s, a, |p| {
            if rises {
                p.rise = guns;
            }
            match state.as_str() {
                "opening" => p.playing = Some((open, now, open_ms)),
                "closing" => p.playing = Some((close, now, close_ms)),
                // Settled states show the end of the animation that led
                // there; a board a zombie has picked stays closed until it
                // pulls (BO2 sets it as it sets closed).
                "open" => p.playing = Some((open, now - open_ms, open_ms)),
                "closed" | "targetted_by_zombie" => {
                    p.playing = Some((close, now - close_ms, close_ms))
                }
                _ => {}
            }
            p.state = state;
        });
        Ok(Value::Undefined)
    });
    m!("getzbarrierpiecestate", |vm, world, s, a| {
        let st = with_piece(vm, world, s, a, |p| p.state.clone()).unwrap_or_default();
        Ok(vm.string(&st))
    });
    m!("showzbarrierpiece", |vm, world, s, a| {
        with_piece(vm, world, s, a, |p| p.shown = true);
        Ok(Value::Undefined)
    });
    m!("hidezbarrierpiece", |vm, world, s, a| {
        with_piece(vm, world, s, a, |p| p.shown = false);
        Ok(Value::Undefined)
    });
    m!("zbarrierpieceuseboxriselogic", |vm, world, s, a| {
        with_piece(vm, world, s, a, |p| p.box_rise = true);
        Ok(Value::Undefined)
    });
    m!("getzbarrierpieceindicesinstate", |vm, world, s, a| {
        let want = vm.to_text(arg(a, 0));
        let idx: Vec<Value> = entnum(vm, s)
            .and_then(|n| {
                world.resource::<Zm>().ents.get(&n).and_then(|e| {
                    e.zbarrier.as_ref().map(|z| {
                        z.pieces
                            .iter()
                            .enumerate()
                            .filter(|(_, p)| p.state == want)
                            .map(|(i, _)| Value::Int(i as i32))
                            .collect()
                    })
                })
            })
            .unwrap_or_default();
        Ok(list(idx))
    });
    m!("getzbarrierpieceanimlengthforstate", |vm, world, s, a| {
        let state = vm.to_text(arg(a, 1));
        let anims = with_piece(vm, world, s, a, |p| {
            (p.open_anim.clone(), p.close_anim.clone())
        });
        let Some((open, close)) = anims else {
            return Ok(Value::Float(0.0));
        };
        let anim = if state.starts_with("clos") {
            close
        } else {
            open
        };
        Ok(Value::Float(
            anim_ms(world, &anim) as f32 * scalar(arg(a, 2)) / 1000.0,
        ))
    });
    for name in [
        "zbarrierpieceusedefaultmodel",
        "zbarrierpieceusealternatemodel",
        "zbarrierpieceuseupgradedmodel",
        "setzbarriercolmodel",
    ] {
        vm.bind(name, true, |_, _, _, _| Ok(Value::Undefined));
    }
<<<<<<< /home/claude/bo2-multiplayer-rebuild/crates/sim/src/t6/zbarrier.rs
    for name in [
        "zbarriersupportszombietaunts",
        "zbarriersupportszombiereachthroughattacks",
        "getzbarriernumattackslots",
    ] {
        vm.bind(name, true, |_, _, _, _| Ok(Value::Int(0)));
    }
    m!("getzbarrierattackslothorzoffset", |_, _, _, _| Ok(
        Value::Float(0.0)
    ));
    for name in [
        "getzbarriertauntanimstate",
        "getzbarrierreachthroughattackanimstate",
        "getzbarrierpieceanimstate",
        "getzbarrierpieceanimsubstate",
    ] {
        vm.bind(name, true, |vm, _, _, _| Ok(vm.string("")));
=======
    // The barrier type's facts (zero / "" when the zones lack the type).
    m!("zbarriersupportszombietaunts", |vm, world, s, _| Ok(
        Value::Int(i32::from(def(vm, world, s).is_some_and(|d| d.taunts)))
    ));
    m!(
        "zbarriersupportszombiereachthroughattacks",
        |vm, world, s, _| Ok(Value::Int(i32::from(
            def(vm, world, s).is_some_and(|d| d.reach_through)
        )))
    );
    m!("getzbarriernumattackslots", |vm, world, s, _| Ok(
        Value::Int(def(vm, world, s).map_or(0, |d| d.num_attack_slots as i32))
    ));
    m!("getzbarrierattackslothorzoffset", |vm, world, s, _| Ok(
        Value::Float(def(vm, world, s).map_or(0.0, |d| d.attack_spot_horz_offset))
    ));
    m!("getzbarriertauntanimstate", |vm, world, s, _| {
        let t = def(vm, world, s).map(|d| d.taunt_state.clone());
        Ok(vm.string(&t.unwrap_or_default()))
    });
    m!(
        "getzbarrierreachthroughattackanimstate",
        |vm, world, s, _| {
            let t = def(vm, world, s).map(|d| d.reach_through_state.clone());
            Ok(vm.string(&t.unwrap_or_default()))
        }
    );
    m!("getzbarrierpieceanimstate", |vm, world, s, a| {
        let t = board(vm, world, s, a).map(|b| b.0);
        Ok(vm.string(&t.unwrap_or_default()))
    });
    m!("getzbarrierpieceanimsubstate", |vm, world, s, a| {
        let t = board(vm, world, s, a).map(|b| b.1);
        Ok(vm.string(&t.unwrap_or_default()))
    });
}

fn def(vm: &Vm<World>, world: &World, s: &Value) -> Option<std::sync::Arc<super::T6ZBarrierDef>> {
    let n = entnum(vm, s)?;
    world
        .resource::<Zm>()
        .ents
        .get(&n)?
        .zbarrier
        .as_ref()?
        .def
        .clone()
}

/// Piece `a[0]`'s board in the barrier type: (tear state, tear substate).
fn board(vm: &Vm<World>, world: &World, s: &Value, a: &[Value]) -> Option<(String, String)> {
    let i = usize::try_from(arg(a, 0).as_int()?).ok()?;
    def(vm, world, s)?.boards.get(i).cloned()
}

#[cfg(test)]
mod tests {
    use super::*;

    fn close(a: [f32; 3], b: [f32; 3]) -> bool {
        a.iter().zip(b).all(|(x, y)| {
            let d = (x - y).rem_euclid(360.0);
            d.min(360.0 - d) < 0.05
        })
    }

    /// A board clip's root turn: a quarter turn left about up (the delta
    /// part's quat2: z = sin 45, w = cos 45) adds 90 to the barrier's yaw;
    /// halfway through it is 45; none leaves the barrier's angles.
    #[test]
    fn a_piece_turns_with_its_clips_root() {
        let h = std::f32::consts::FRAC_1_SQRT_2;
        let anim = super::super::T6Anim {
            numframes: 10,
            delta_quat: vec![(0, [0.0, 0.0, 0.0, 1.0]), (10, [0.0, 0.0, h, h])],
            ..Default::default()
        };
        let end = root_turn(&anim, 1.0).unwrap();
        assert!(close(turned([0.0, 30.0, 0.0], end), [0.0, 120.0, 0.0]));
        let mid = root_turn(&anim, 0.5).unwrap();
        assert!(close(turned([0.0, 0.0, 0.0], mid), [0.0, 45.0, 0.0]));
        let still = super::super::T6Anim::default();
        assert!(root_turn(&still, 0.5).is_none());
>>>>>>> /tmp/claude-0/zmhead/crates/sim/src/t6/zbarrier.rs
    }
}
