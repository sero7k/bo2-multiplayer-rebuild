//! bo2mp: engine functions only Black Ops II multiplayer's scripts call:
//! team scores, the match clock the HUD counts down, the map centre the
//! minimap turns about, the kill feed, and the small answers a local match
//! gives (no online services, no pregame lobby, no stats recorder).

use bevy_ecs::prelude::World;
use gsc_t6::{Array, Value, Vm};

use super::{Zm, arg, text, vec3};

pub(super) fn bind(vm: &mut Vm<World>) {
    macro_rules! f {
        ($name:literal, $body:expr) => {
            vm.bind($name, false, $body);
        };
    }
    macro_rules! m {
        ($name:literal, $body:expr) => {
            vm.bind($name, true, $body);
        };
    }
    f!("setslowmotion", |_, _, _, a| {
        let num = |i: usize| match arg(a, i) {
            Value::Int(n) => *n as f32,
            Value::Float(f) => *f,
            _ => 1.0,
        };
        super::slowmo::set(num(0), num(1), num(2));
        Ok(Value::Undefined)
    });
    // Nothing is restricted in a match of ours (Black Ops II's ranked
    // playlists' bans): every item a class names may be given.
    f!("isitemrestricted", |_, _, _, _| Ok(Value::Int(0)));
    // Two lethals at once is a Black Ops II console perk the game never
    // turns on in multiplayer.
    m!("dualgrenadesactive", |_, _, _, _| Ok(Value::Int(0)));
    // A vehicle's share of an explosion (`_globallogic_vehicle`): its health
    // goes down by the damage; at none it dies, as the engine's own damage
    // kills it.
    m!("finishvehicleradiusdamage", |vm, world, s, a| {
        let Some(obj) = s.as_obj() else { return Ok(Value::Undefined) };
        let damage = match arg(a, 2) {
            Value::Int(n) => *n,
            Value::Float(f) => *f as i32,
            _ => 0,
        };
        let hf = vm.intern("health");
        let health = match vm.get_field(world, obj, hf) {
            Value::Int(n) => n,
            Value::Float(f) => f as i32,
            _ => return Ok(Value::Undefined),
        };
        let left = (health - damage.max(0)).max(0);
        vm.set_field(world, obj, hf, Value::Int(left));
        if left == 0 && health > 0 {
            vm.notify_str(world, obj, "death", &[arg(a, 0).clone()]);
        }
        Ok(Value::Undefined)
    });
    // A round over (Search and Destroy, half time): the level starts again
    // at the next frame, `pers[]` kept when asked (`map_restart( 1 )`).
    f!("map_restart", |_, world, _, a| {
        world.resource_mut::<Zm>().pending_restart = Some(super::flag(a, 0, false));
        Ok(Value::Undefined)
    });
    // Team scores: what the score bar and the score limit read.
    f!("getteamscore", |vm, world, _, a| {
        let team = text(vm, a, 0);
        Ok(Value::Int(
            world
                .resource::<Zm>()
                .team_scores
                .get(&team)
                .copied()
                .unwrap_or(0),
        ))
    });
    f!("setteamscore", |vm, world, _, a| {
        let team = text(vm, a, 0);
        let score = arg(a, 1).as_int().unwrap_or(0);
        world.resource_mut::<Zm>().team_scores.insert(team, score);
        Ok(Value::Undefined)
    });
    // The server time the match ends at (0: no clock), for the HUD timer.
    f!("setgameendtime", |_, world, _, a| {
        world.resource_mut::<Zm>().game_end_time = arg(a, 0).as_int().unwrap_or(0);
        Ok(Value::Undefined)
    });
    f!("setmapcenter", |_, world, _, a| {
        world.resource_mut::<Zm>().map_center = vec3(a, 0)?;
        Ok(Value::Undefined)
    });
    // The kill feed: who killed whom with what (the HUD's obituary lines).
    f!("obituary", |vm, world, _, a| {
        let name = |vm: &Vm<World>, v: &Value| match super::entnum(vm, v) {
            Some(n) => format!("player {n}"),
            None => vm.to_text(v),
        };
        let victim = name(vm, arg(a, 0));
        let attacker = name(vm, arg(a, 1));
        let weapon = text(vm, a, 2);
        let mod_ = text(vm, a, 3);
        diag::info!(
            Sim,
            "bo2mp obituary: {attacker} killed {victim} ({weapon}, {mod_})"
        );
        // lane C: the kill feed his HUD shows (names, not numbers; "self"
        // when nobody else killed him: a fall, his own grenade).
        let (v, k) = (player_name(vm, world, arg(a, 0)), player_name(vm, world, arg(a, 1)));
        let own = super::entnum(vm, arg(a, 1)).is_none_or(|n| Some(n) == super::entnum(vm, arg(a, 0)));
        // The two sides, for the name colours (yours or the other team's).
        let (kt, vt) = (player_team(vm, arg(a, 1)), player_team(vm, arg(a, 0)));
        feed(world).obituary(format!("{k}|{v}|{weapon}|{mod_}|{}|{kt}|{vt}", if own { "self" } else { "" }));
        if let (Some(dead), Some(killer)) = (super::entnum(vm, arg(a, 0)), super::entnum(vm, arg(a, 1)))
            && dead != killer
        {
            feed(world).killers.insert(dead as u32, (killer as i32, false));
        }
        Ok(Value::Undefined)
    });
    // A string as a localized-string reference (`istring("mpl_...")`).
    f!("istring", |vm, _, _, a| {
        let s = text(vm, a, 0);
        let id = vm.intern(&s);
        Ok(Value::IStr(id))
    });
    // The asset a table names: his zones hold every table the scripts ask
    // for under its own name.
    f!("tablelookupfindcoreasset", |_, _, _, a| Ok(
        arg(a, 0).clone()
    ));
    f!("getvehicletreadfxarray", |_, _, _, _| Ok(Value::array(
        Array::new()
    )));
    // Split-screen only (BO2's engine asks whether the game mode is local
    // split-screen); a custom game is not, so the bots fill up to 18
    // players rather than 10.
    f!("islocalgame", |_, _, _, _| Ok(Value::Int(0)));
    f!("ispregamegamestarted", |_, _, _, _| Ok(Value::Int(0)));
    f!("setdvarint", |vm, _, _, a| {
        let name = text(vm, a, 0).to_ascii_lowercase();
        let v = vm.to_text(arg(a, 1));
        vm.dvars.insert(name, v);
        Ok(Value::Undefined)
    });
    f!("clientnotify", |vm, _, _, a| {
        let n = text(vm, a, 0);
        diag::info!(Sim, "bo2mp client notify: {n}");
        Ok(Value::Undefined)
    });
    for name in [
        "matchrecorderincrementheaderstat",
        "recordgameresult",
        "recordleaguewinner",
        "recordmatchbegin",
        "recordplayerstats",
        "skillupdate",
        "changeadvertisedstatus",
        "pcserverupdateplaylist",
        "sethostmigrationstatus",
        "announcement",
    ] {
        vm.bind(name, false, |_, _, _, _| Ok(Value::Undefined));
    }
    // Player side: the kill camera's field of view, the rank on the
    // scoreboard, the spawn the client streams ahead (nothing to stream).
    for name in [
        "setfovforkillcam",
        "setrank",
        "predictspawnpoint",
        "setpregameclass",
        "setpregameteam",
        "gamehistorystartmatch",
        "recordloadoutperksandkillstreaks",
        "bbclasschoice",
        "addgametypestat",
        "addbonuscardstat",
        // Spawn protection's on/off marks (the scripts time it themselves).
        "spawnprotectionactive",
        "spawnprotectioninactive",
        // A launcher's lock-on warnings (lane D: scorestreaks and launchers).
        "weaponlockfree",
        "weaponlocktargettooclose",
        "weaponlocknoclearance",
        "setweaponheatpercent",
        // Camo and reticle render options, the first raise animation.
        "setplayerrenderoptions",
        "initialweaponraise",
    ] {
        vm.bind(name, true, |_, _, _, _| Ok(Value::Undefined));
    }
    // BO2's placespawnpoint (every spawn point, as the match starts): a
    // player-sized box lifted up to 128 units (less under a ceiling), then
    // dropped straight down to the floor; the point moves there. A point the
    // map leaves in the air or inside a fence now stands on the ground, as a
    // player spawned there would. One still in solid gets BO2's warning.
    m!("placespawnpoint", |vm, world, s, _| {
        use crate::bullet_collision::{MASK_PLAYER_SOLID, PLAYER_MAXS, PLAYER_MINS};
        let Some(n) = super::entnum(vm, s) else {
            return Ok(Value::Undefined);
        };
        let Some(o) = world.resource::<Zm>().ents.get(&n).map(|e| e.origin) else {
            return Ok(Value::Undefined);
        };
        let f = super::frame(world);
        let up = f.trace_clip(o, [o[0], o[1], o[2] + 128.0], PLAYER_MINS, PLAYER_MAXS, MASK_PLAYER_SOLID);
        let top = [o[0], o[1], o[2] + 128.0 * up.fraction];
        let bottom = [o[0], o[1], top[2] - 262_144.0];
        let down = f.trace_clip(top, bottom, PLAYER_MINS, PLAYER_MAXS, MASK_PLAYER_SOLID);
        if down.fraction >= 1.0 {
            // No floor under it at all: leave it where the map put it.
            return Ok(Value::Undefined);
        }
        let floor = [o[0], o[1], top[2] - 262_144.0 * down.fraction];
        let stuck = f.trace_clip(floor, floor, PLAYER_MINS, PLAYER_MAXS, MASK_PLAYER_SOLID).startsolid != 0;
        drop(f);
        if stuck {
            diag::info!(Sim, "bo2mp spawn point {n} is in solid at {:.0} {:.0} {:.0}", floor[0], floor[1], floor[2]);
        }
        if (floor[2] - o[2]).abs() > 1.0 {
            diag::info!(Sim, "bo2mp spawn point {n} placed: z {:.0} -> {:.0} at {:.0} {:.0}", o[2], floor[2], o[0], o[1]);
        }
        if let Some(e) = world.resource_mut::<Zm>().ents.get_mut(&n) {
            e.origin = floor;
        }
        Ok(Value::Undefined)
    });
    // A weapon only some items use (none of a default class's).
    f!("isweaponspecificuse", |_, _, _, _| Ok(Value::Int(0)));
    // Camo, reticle and player render options packed for the client: none
    // (the default look) until the class data carries them.
    m!("calcplayeroptions", |_, _, _, _| Ok(Value::Int(0)));
    m!("calcweaponoptions", |_, _, _, _| Ok(Value::Int(0)));
    // A test script's prints (BO2MP_DEBUG_GSC) land in the log; BO2's own
    // scripts only print from developer blocks, which never run.
    f!("println", |vm, _, _, a| {
        let line: Vec<String> = a.iter().map(|v| vm.to_text(v)).collect();
        diag::info!(Sim, "bo2mp println: {}", line.join(" "));
        Ok(Value::Undefined)
    });
    // A trace through the map (as bullettrace, no entities): the result
    // array the scripts read (fraction, position, normal, surface).
    f!("worldtrace", |vm, world, _, a| {
        let (s, e) = (vec3(a, 0)?, vec3(a, 1)?);
        let t = super::frame(world).trace_static_world(
            s,
            e,
            [0.0; 3],
            [0.0; 3],
            crate::bullet_collision::MASK_SHOT,
        );
        let f = t.fraction.clamp(0.0, 1.0);
        let pos: [f32; 3] = std::array::from_fn(|i| s[i] + (e[i] - s[i]) * f);
        let mut arr = Array::new();
        let k = |vm: &mut Vm<World>, s: &str| gsc_t6::Key::Str(vm.intern(s));
        let surface = vm.string(if f < 1.0 { "default" } else { "none" });
        arr.set(k(vm, "fraction"), Value::Float(f));
        arr.set(k(vm, "position"), Value::Vec3(pos));
        arr.set(k(vm, "normal"), Value::Vec3(if f < 1.0 { t.normal } else { [0.0; 3] }));
        arr.set(k(vm, "surfacetype"), surface);
        Ok(Value::array(arr))
    });
    // refreshshieldattachment: t6::playeranim (the riot shield on the body).
    for name in ["seteverhadweaponall", "stoplocalsound"] {
        vm.bind(name, true, |_, _, _, _| Ok(Value::Undefined));
    }
    m!("isweaponviewonlylinked", |_, _, _, _| Ok(Value::Int(0)));
    // The spawn a player gets (`getsortedspawnpoints(point team, influencer
    // team, enemy mask, player, predicted)`): as BO2's engine, every spawn
    // point of the team is rated by the spawn influencers that count for
    // the influencer team (`influencers`), the points walked best first,
    // the best twenty at most, skipping dangerous ones (it would telefrag,
    // or a living enemy within 2600 units, spawnsystem_sight_check_max_
    // distance, sees it, eyes 65 units up) and the first safe one taken.
    // Points the influencers rate alike keep the farthest from the nearest
    // enemy first. (The scripts ask every half second for each dead
    // player's predicted spawn: that answer only tells the client what to
    // stream, and ours streams nothing, so it takes the best-rated point
    // and pays no sight traces.)
    f!("getsortedspawnpoints", |vm, world, _, a| {
        let team = text(vm, a, 0);
        let influencer_team = text(vm, a, 1);
        let player = arg(a, 3).clone();
        let predicted = super::flag(a, 4, false);
        let level = vm.level;
        // The game type's spawn point kinds: the team's (level.
        // spawn_point_team_class_names[team], e.g. mp_tdm_spawn), else every
        // kind it registered (level.spawn_point_class_names), else the
        // free-for-all kind.
        let strings = |vm: &mut Vm<World>, v: Value| -> Vec<String> {
            match v {
                Value::Array(list) => list
                    .read()
                    .values_in_order()
                    .filter_map(|v| match v {
                        Value::Str(s) => Some(vm.str(*s).to_ascii_lowercase()),
                        _ => None,
                    })
                    .collect(),
                _ => Vec::new(),
            }
        };
        let team_kinds = {
            let f = vm.intern("spawn_point_team_class_names");
            let key = gsc_t6::Key::Str(vm.intern(&team));
            let v = match vm.raw_field(level, f) {
                Value::Array(arr) => arr.get(&key).unwrap_or(Value::Undefined),
                _ => Value::Undefined,
            };
            strings(vm, v)
        };
        let kinds = if team_kinds.is_empty() {
            let f = vm.intern("spawn_point_class_names");
            let all = vm.raw_field(level, f);
            let v = strings(vm, all);
            if v.is_empty() { vec!["mp_dm_spawn".to_owned()] } else { v }
        } else {
            team_kinds
        };
        // Every map entity of those kinds.
        let of_kinds = |world: &World, kinds: &[String]| -> Vec<Value> {
            world
                .resource::<Zm>()
                .ents
                .values()
                .filter(|e| kinds.iter().any(|k| e.classname.eq_ignore_ascii_case(k)))
                .filter_map(|e| e.obj.map(Value::Object))
                .collect()
        };
        let mut points = of_kinds(world, &kinds);
        // bo2mp: a map from another game without the game type's kinds
        // (none of them in its entities, none copied in by the load):
        // Team Deathmatch's points, else free-for-all's, rather than none
        // (the scripts then spawn the player at the world origin).
        if points.is_empty() {
            for fallback in ["mp_tdm_spawn", "mp_dm_spawn"] {
                points = of_kinds(world, &[fallback.to_owned()]);
                if !points.is_empty() {
                    static WARNED: std::sync::atomic::AtomicBool =
                        std::sync::atomic::AtomicBool::new(false);
                    if !WARNED.swap(true, std::sync::atomic::Ordering::Relaxed) {
                        diag::warn!(
                            Sim,
                            "bo2mp spawn gap: no map spawn points of {kinds:?} (team {team}); \
                             {} {fallback} points used instead",
                            points.len()
                        );
                    }
                    break;
                }
            }
            if points.is_empty() {
                static EMPTY: std::sync::atomic::AtomicBool =
                    std::sync::atomic::AtomicBool::new(false);
                if !EMPTY.swap(true, std::sync::atomic::Ordering::Relaxed) {
                    diag::warn!(
                        Sim,
                        "bo2mp spawn gap: no map spawn points of {kinds:?} (team {team}) and none to \
                         fall back on: the player spawns where the scripts default to"
                    );
                }
            }
        }
        let me = super::entnum(vm, &player);
        let my_team = me.and_then(|n| {
            let p = world.resource::<Zm>().players.get(&n)?.obj;
            let f = vm.intern("team");
            Some(vm.to_text(&vm.raw_field(p, f)))
        });
        // Living enemies' eyes.
        let others: Vec<u32> = world.resource::<Zm>().players.keys().copied().collect();
        let mut enemies: Vec<[f32; 3]> = Vec::new();
        for n in others {
            if Some(n) == me {
                continue;
            }
            let Some(obj) = world.resource::<Zm>().players.get(&n).map(|p| p.obj) else {
                continue;
            };
            let f = vm.intern("team");
            let t = vm.to_text(&vm.raw_field(obj, f));
            if my_team.as_deref().is_some_and(|m| m != "free" && m == t) {
                continue;
            }
            if let Some(ps) = super::frame(world).player(crate::world::ClientId(n))
                && ps.health > 0
            {
                enemies.push([ps.origin[0], ps.origin[1], ps.origin[2] + SIGHT_HEIGHT]);
            }
        }
        // spawnsystem_sight_check_max_distance, spawnsystem_sight_height_offset.
        const SIGHT_RANGE: f32 = 2600.0;
        const SIGHT_HEIGHT: f32 = 65.0;
        // The influencer team's bit (level.spawnsystem.ispawn_teammask).
        let team_bit = {
            let ss = vm.intern("spawnsystem");
            let masks = vm.intern("ispawn_teammask");
            let key = gsc_t6::Key::Str(vm.intern(&influencer_team));
            match vm.raw_field(level, ss) {
                Value::Object(o) => match vm.raw_field(o, masks) {
                    Value::Array(arr) => arr.get(&key).and_then(|v| v.as_int()).unwrap_or(1),
                    _ => 1,
                },
                _ => 1,
            }
        };
        let mut rated: Vec<(f32, f32, [f32; 3], Value)> = Vec::new();
        for p in points {
            let Some(o) = super::origin_of(vm, world, &p).or_else(|| {
                let Value::Object(obj) = &p else { return None };
                let f = vm.intern("origin");
                vm.raw_field(*obj, f).as_vec3()
            }) else {
                continue;
            };
            let far = enemies
                .iter()
                .map(|e| ((e[0] - o[0]).powi(2) + (e[1] - o[1]).powi(2)).sqrt())
                .fold(4096.0f32, f32::min);
            rated.push((0.0, far, o, p));
        }
        // The influencers where they are now (one following an entity is
        // where that entity is; one whose entity is gone counts no more).
        let now = vm.time_ms;
        let infl = world.get_resource_mut::<super::influencers::SpawnInfluencers>().map(|mut s| {
            s.expire(now);
            s.list.clone()
        });
        if let Some(list) = infl {
            let mut at: Vec<Option<[f32; 3]>> = Vec::with_capacity(list.len());
            for i in &list {
                at.push(match &i.entity {
                    Some(e) => super::origin_of(vm, world, e),
                    None => Some(i.origin),
                });
            }
            let origin_now = |i: &super::influencers::Influencer| {
                list.iter().position(|x| x.id == i.id).and_then(|k| at[k])
            };
            let spots: Vec<[f32; 3]> = rated.iter().map(|r| r.2).collect();
            let scores = super::influencers::score_points(&list, team_bit, &spots, &origin_now);
            for (r, s) in rated.iter_mut().zip(scores) {
                r.0 = s;
            }
        }
        let rated: Vec<(f32, [f32; 3], Value)> = {
            rated.sort_by(|x, y| {
                y.0.partial_cmp(&x.0)
                    .unwrap_or(std::cmp::Ordering::Equal)
                    .then(y.1.partial_cmp(&x.1).unwrap_or(std::cmp::Ordering::Equal))
            });
            rated.into_iter().map(|(s, _, o, p)| (s, o, p)).collect()
        };
        if rated.is_empty() {
            return Ok(super::list(Vec::new()));
        }
        let f = super::frame(world);
        let alive: Vec<[f32; 3]> = f
            .client_ids_sorted()
            .into_iter()
            .filter_map(|id| f.player(id).filter(|ps| ps.health > 0).map(|ps| ps.origin))
            .collect();
        // Never on top of anyone alive (the spawn would telefrag him).
        let telefrag = |o: [f32; 3]| {
            alive.iter().any(|a| {
                ((a[0] - o[0]).powi(2) + (a[1] - o[1]).powi(2)).sqrt() < 48.0 && (a[2] - o[2]).abs() < 72.0
            })
        };
        let mut safe: Vec<usize> = Vec::new();
        let (tracing, traces) = (std::time::Instant::now(), std::cell::Cell::new(0u32));
        for (i, (_, o, _)) in rated.iter().enumerate().take(if predicted { 0 } else { 20 }) {
            if telefrag(*o) {
                continue;
            }
            let eye = [o[0], o[1], o[2] + SIGHT_HEIGHT];
            let seen = enemies.iter().any(|e| {
                (e[0] - o[0]).powi(2) + (e[1] - o[1]).powi(2) <= SIGHT_RANGE * SIGHT_RANGE
                    && {
                        traces.set(traces.get() + 1);
                        true
                    }
                    && f.trace_static_world(*e, eye, [0.0; 3], [0.0; 3], crate::bullet_collision::MASK_SHOT)
                        .fraction
                        >= 1.0
            });
            if !seen {
                safe.push(i);
                break;
            }
        }
        let ms = tracing.elapsed().as_secs_f64() * 1000.0;
        if ms > 10.0 {
            diag::warn!(Sim, "bo2mp spawn pick: {} sight traces took {ms:.1} ms", traces.get());
        }
        // All dangerous: the best that would not telefrag, else the best.
        let fallback = rated.iter().position(|(_, o, _)| !telefrag(*o)).unwrap_or(0);
        drop(f);
        let pick = safe.first().copied().unwrap_or(fallback);
        Ok(super::list(vec![rated[pick].2.clone()]))
    });
    // The death path's small answers: no revive, rank XP waits for the
    // profile (`cloneplayer` is below).
    // Grenades: the lethal goes in the primary offhand slot (G), the
    // tactical in the secondary; each slot's class comes from the weapon's
    // own equipment row (frag, smoke, flash...).
    fn offhand_class(world: &mut World, name: &str) -> i32 {
        let Ok(w) = super::weapon(world, name) else {
            return 0;
        };
        super::frame(world)
            .offhand_loadout_row(w)
            .map_or(0, |f| f.offhand_class)
    }
    m!("setoffhandsecondaryclass", |vm, world, s, a| {
        let id = super::client(vm, world, s)?;
        let name = text(vm, a, 0);
        let class = match offhand_class(world, &name) {
            0 => 3,
            c => c,
        };
        if let Some(ps) = super::frame(world).player_mut(id) {
            ps.offhand_secondary = class;
        }
        Ok(Value::Undefined)
    });
    m!("switchtooffhand", |vm, world, s, a| {
        let id = super::client(vm, world, s)?;
        let name = text(vm, a, 0);
        let class = offhand_class(world, &name);
        if class != 0
            && let Some(ps) = super::frame(world).player_mut(id)
        {
            ps.offhand_primary = class;
        }
        Ok(Value::Undefined)
    });
    // Every perk of a class at once (`setperks(array)`), as setperk each.
    m!("setperks", |vm, world, s, a| {
        let id = super::client(vm, world, s)?;
        let Value::Array(list) = arg(a, 0) else {
            return Ok(Value::Undefined);
        };
        let names: Vec<String> = list
            .read()
            .values_in_order()
            .map(|v| vm.to_text(v))
            .collect();
        for p in names {
            crate::script_player::set_perk(&mut super::frame(world), id, &p, true);
            if let Some(pl) = world.resource_mut::<Zm>().players.get_mut(&id.0) {
                pl.perks.insert(p);
            }
        }
        Ok(Value::Undefined)
    });
    // A gun's attachment by name (`weaponhasattachment("mk48_mp+fmj", "fmj")`).
    f!("weaponhasattachment", |vm, _, _, a| {
        let w = text(vm, a, 0);
        let att = text(vm, a, 1);
        Ok(Value::bool(w.split('+').skip(1).any(|x| x == att)))
    });
    m!("getstowedweapon", |vm, world, s, _| {
        let name = super::entnum(vm, s)
            .and_then(|n| super::playeranim::stowed_of(world, n))
            .unwrap_or_else(|| "none".to_owned());
        Ok(vm.string(&name))
    });
    m!("getcurrentweaponaltweapon", |vm, _, _, _| Ok(vm.string("none")));
    m!("getvehicleoccupied", |_, _, _, _| Ok(Value::Undefined));
    f!("doesweaponreplacespawnweapon", |_, _, _, _| Ok(Value::Int(0)));
    for name in ["gpsjammeractive", "gpsjammerinactive", "recordmultikill", "trackweaponfirenative"] {
        vm.bind(name, true, |_, _, _, _| Ok(Value::Undefined));
    }
    // The bomb timers, spawn influencer weights (objectives are
    // `objectives.rs`).
    for name in [
        "setbombtimer",
        "setspawnpointsbaseweight",
        "setinfluencerteammask",
    ] {
        vm.bind(name, false, |_, _, _, _| Ok(Value::Undefined));
    }
    for name in [
        "recordgameevent",
        "clientclaimtrigger",
        "clientreleasetrigger",
        "releaseclaimedtrigger",
        "allowtacticalinsertion",
        // Spawn logging flags (BO2's spawn telemetry).
        "setspawnclientflag",
    ] {
        vm.bind(name, true, |_, _, _, _| Ok(Value::Undefined));
    }
    m!("isempjammed", |_, _, _, _| Ok(Value::Int(0)));
    // The lethal slot by weapon (One in the Chamber, Sticks and Stones give
    // the tomahawk).
    m!("setoffhandprimaryclass", |vm, world, s, a| {
        let id = super::client(vm, world, s)?;
        let name = text(vm, a, 0);
        let class = match offhand_class(world, &name) {
            0 => 1,
            c => c,
        };
        if let Some(ps) = super::frame(world).player_mut(id) {
            ps.offhand_primary = class;
        }
        Ok(Value::Undefined)
    });
    // A random point inside an entity's bounds (fractions per axis): its
    // origin (our brush entities keep no bounds here).
    m!("getpointinbounds", |vm, world, s, _| Ok(super::origin_of(vm, world, s)
        .map_or(Value::Undefined, Value::Vec3)));
    // bo2mp: the engine's corpse of him, holding his death animation, and
    // an entity for the scripts (`self.body`).
    m!("cloneplayer", |vm, world, s, _| {
        let Some(n) = super::entnum(vm, s) else {
            return Ok(Value::Undefined);
        };
        Ok(super::playeranim::clone_player(vm, world, n))
    });
    m!("needsrevive", |_, _, _, _| Ok(Value::Int(0)));
    m!("getentitytype", |_, _, _, _| Ok(Value::Int(1)));
    m!("anyammoforweaponmodes", |_, _, _, _| Ok(Value::Int(1)));
    for name in ["enabledeathstreak", "luinotifyeventtospectators"] {
        vm.bind(name, true, |_, _, _, _| Ok(Value::Undefined));
    }
    f!("registerxp", |_, _, _, _| Ok(Value::Undefined));
    // A sound's length in ms, its longest variant's clip in his banks (the
    // announcer waits for one line before the next); -1 for one with no
    // known length, as BO2's (the announcer then waits 3.1 s, not 0.1).
    f!("soundgetplaybacktime", |vm, world, _, a| {
        let name = text(vm, a, 0).to_ascii_lowercase();
        Ok(Value::Int(
            world.resource::<Zm>().sound_lengths.get(&name).map_or(-1, |ms| *ms as i32),
        ))
    });
    // A weapon's blast radius (its weapon file's explosionRadius).
    f!("getweaponexplosionradius", |vm, world, _, a| {
        let w = super::weapon(world, &text(vm, a, 0)).unwrap_or(0);
        let r = if w == 0 {
            0
        } else {
            super::frame(world)
                .equipment_facts_for(w)
                .map_or(0, |f| f.explosion_radius.max(0))
        };
        Ok(Value::Float(r as f32))
    });
    bind_screen(vm);
}

/// bo2mp lane C: what each player's own screen shows, for his client: the
/// menus the scripts open (BO2's class menu at spawn), the LUI notifies
/// (score popups, medals, rank), the kill feed. `publish_screen` sends them
/// each tick as client dvars: `bo2mp_menu` ("<n>:<menu>", "" menu =
/// closed), `bo2mp_lui` ("<n>|<event>|<arg>|..." by `;`, the last 12),
/// `bo2mp_feed` ("<n>|<attacker>|<victim>|<weapon>|<mod>|<self>|<attacker team>|<victim team>" by `;`, the last
/// 6), `bo2mp_scores` ("<allies>,<axis>"), `bo2mp_timeleft` (whole
/// seconds, "" no clock), `bo2mp_team`, `bo2mp_gametype`, `bo2mp_teams`
/// (everyone's: "<client>:<team>,..."), `bo2mp_cols` (the scoreboard's
/// columns). `<n>` counts up
/// so the client sends each once.
#[derive(bevy_ecs::prelude::Resource, Default)]
pub(crate) struct ScreenFeed {
    seq: u32,
    menus: std::collections::BTreeMap<u32, String>,
    lui: std::collections::BTreeMap<u32, Vec<String>>,
    feed: Vec<String>,
    /// The game type's scoreboard columns (`setscoreboardcolumns`).
    cols: Vec<String>,
    /// The minimap (`setminimap`): "<material>,<ul x>,<ul y>,<lr x>,<lr y>,
    /// <north yaw>".
    minimap: String,
    /// Whoever last killed each client (victim -> killer), until he is back
    /// in play: the card BO2 shows at his death, before the killcam.
    killers: std::collections::HashMap<u32, (i32, bool)>,
    /// Each client's killcam ("<kind>|<killer>") and its killer's perks
    /// (`perk_rows`), worked out once a killcam.
    killcam_perks: std::collections::HashMap<u32, (String, String)>,
}

impl ScreenFeed {
    fn next(&mut self) -> u32 {
        self.seq += 1;
        self.seq
    }

    fn obituary(&mut self, line: String) {
        let n = self.next();
        self.feed.push(format!("{n}|{line}"));
        let len = self.feed.len();
        self.feed.drain(..len.saturating_sub(6));
    }
}

fn feed(world: &mut World) -> bevy_ecs::prelude::Mut<'_, ScreenFeed> {
    if !world.contains_resource::<ScreenFeed>() {
        world.insert_resource(ScreenFeed::default());
    }
    world.resource_mut::<ScreenFeed>()
}

/// A player's team (`self.team`: "allies", "axis", ...), "" for no player.
fn player_team(vm: &mut Vm<World>, v: &Value) -> String {
    match v {
        Value::Object(o) if super::entnum(vm, v).is_some() => {
            let f = vm.intern("team");
            vm.to_text(&vm.raw_field(*o, f))
        }
        _ => String::new(),
    }
    .replace(['|', ';'], "")
}

/// A player's name (`self.name`), else the value as text.
fn player_name(vm: &mut Vm<World>, world: &mut World, v: &Value) -> String {
    match v {
        // (`name` is the engine's field on a player, not a script one.)
        Value::Object(o) if super::entnum(vm, v).is_some() => {
            let f = vm.intern("name");
            let n = vm.get_field(world, *o, f);
            vm.to_text(&n)
        }
        other => vm.to_text(other),
    }
    .replace(['|', ';'], " ")
}

/// A player's perks 1 to 3 of his class as the HUD draws them:
/// `icon|localized name` by `;` (the killcam's perk list, `setperks`, and
/// the modern HUD's killcam, `bo2mp_killcam_perks`).
fn perk_rows(vm: &mut Vm<World>, world: &mut World, who: &Value) -> String {
    let class = match who {
        Value::Object(p) => {
            let f = vm.intern("class_num");
            vm.raw_field(*p, f).as_int().unwrap_or(0)
        }
        _ => 0,
    };
    let card = super::loadout::perk_card(vm, world, who, class);
    let mut rows = Vec::new();
    for row in card.split(';').filter(|r| !r.is_empty()) {
        let (icon, name) = row.split_once('|').unwrap_or(("", row));
        let key = vm.string(name);
        rows.push(format!("{icon}|{}", super::localize(vm, world, &key, &[]).replace(['|', ';'], " ")));
    }
    rows.join(";")
}

fn bind_screen(vm: &mut Vm<World>) {
    // The text effects (decode, pulse, typewriter):
    // elem setpulsefx(ms per letter, decay start ms, decay ms). The letters
    // come in, then the text decays away: BO2 takes down its objective
    // text and outcome title this way, not by alpha. (The decode's
    // scrambled letters come in as plain ones; redact is not drawn.)
    for name in ["setcod7decodefx", "setpulsefx", "settypewriterfx"] {
        vm.bind(name, true, |_, world, s, a| {
            let n = |i: usize| arg(a, i).as_float().unwrap_or(0.0) as i64;
            let fx = [world.resource::<Zm>().now_ms, n(0), n(1), n(2)];
            if let Value::Object(o) = s
                && let Some(e) = world.resource_mut::<Zm>().huds.elems.get_mut(&o.index)
            {
                e.fx = Some(fx);
            }
            Ok(Value::Undefined)
        });
    }
    vm.bind("setredactfx", true, |_, _, _, _| Ok(Value::Undefined));
    // elem setperks(player): the engine draws that player's perk icons in
    // the element (the killcam's killer card). Not drawn yet: the element
    // shows nothing (it was the "white" square createloadouticon starts
    // it as).
    vm.bind("setperks", true, |vm, world, s, a| {
        // The list rides in the element's text as `#perks:icon|name;...`
        // (the names localized here), drawn by the client.
        // (BO2's engine draws the perks of the player his view follows: in
        // a killcam (`showperks` as it starts) the killer's, not his own.)
        let mut who = arg(a, 0).clone();
        if let Value::Object(p) = &who {
            let (kc, sc) = (vm.intern("killcam"), vm.intern("spectatorclient"));
            let watched = vm.raw_field(*p, sc).as_float().filter(|c| *c >= 0.0).map(|c| c as u32);
            if gsc_t6::truthy(&vm.raw_field(*p, kc))
                && let Some(obj) = watched.and_then(|c| world.resource::<Zm>().players.get(&c).map(|q| q.obj))
            {
                who = Value::Object(obj);
            }
        }
        let rows = perk_rows(vm, world, &who);
        if let Value::Object(o) = s
            && let Some(e) = world.resource_mut::<Zm>().huds.elems.get_mut(&o.index)
        {
            e.shader = None;
            e.text = if rows.is_empty() { String::new() } else { format!("#perks:{rows}") };
        }
        Ok(Value::Undefined)
    });
    // elem setplayernamestring(player): his name.
    vm.bind("setplayernamestring", true, |vm, world, s, a| {
        let name = player_name(vm, world, arg(a, 0));
        if let Value::Object(o) = s
            && let Some(e) = world.resource_mut::<Zm>().huds.elems.get_mut(&o.index)
        {
            e.text = name;
        }
        Ok(Value::Undefined)
    });
    // getnorthyaw(): the map's north (its worldspawn `northyaw`).
    vm.bind("getnorthyaw", false, |vm, _, _, _| {
        Ok(Value::Float(vm.dvars.get("northyaw").and_then(|v| v.parse().ok()).unwrap_or(0.0)))
    });
    // setminimap(material, ul x, ul y, lr x, lr y): his minimap's picture
    // and the world corners it covers.
    vm.bind("setminimap", false, |vm, world, _, a| {
        let material = text(vm, a, 0);
        let n = |i: usize| arg(a, i).as_float().unwrap_or(0.0);
        let north = vm.dvars.get("northyaw").cloned().unwrap_or_else(|| "0".to_owned());
        let line = format!("{material},{},{},{},{},{north}", n(1), n(2), n(3), n(4));
        diag::info!(Sim, "bo2mp minimap: {line}");
        // Lane D: where a scorestreak's map pick lands.
        super::killstreaks::set_minimap(world, [n(1), n(2)], [n(3), n(4)], north.parse().unwrap_or(0.0));
        feed(world).minimap = line;
        Ok(Value::Undefined)
    });
    // setscoreboardcolumns(col, ...): his scoreboard's columns.
    vm.bind("setscoreboardcolumns", false, |vm, world, _, a| {
        let cols: Vec<String> = a.iter().map(|v| vm.to_text(v)).filter(|c| !c.is_empty()).collect();
        feed(world).cols = cols;
        Ok(Value::Undefined)
    });
    // openmenu(menu): BO2's own menu opens on his screen (lane C's client);
    // BO2MP_AUTOCLASS=<class> still answers the class menu a second later
    // (a test aid: hidden test matches have no one at the keyboard).
    vm.bind("openmenu", true, |vm, world, s, a| {
        let menu = text(vm, a, 0);
        let client = super::entnum(vm, s).unwrap_or(u32::MAX);
        diag::info!(Sim, "bo2mp menu open: player {client} {menu}");
        let mut f = feed(world);
        let n = f.next();
        f.menus.insert(client, format!("{n}:{menu}"));
        if let (Value::Object(o), Ok(class)) = (s, std::env::var("BO2MP_AUTOCLASS"))
            && menu.contains("changeclass")
        {
            let mut zm = world.resource_mut::<Zm>();
            let due = zm.now_ms + 1000;
            zm.menu_replies.push((due, *o, menu, class));
        }
        Ok(Value::Undefined)
    });
    for name in ["closemenu", "closeingamemenu"] {
        vm.bind(name, true, |vm, world, s, _| {
            let client = super::entnum(vm, s).unwrap_or(u32::MAX);
            let mut f = feed(world);
            let n = f.next();
            f.menus.insert(client, format!("{n}:"));
            Ok(Value::Undefined)
        });
    }
    // self luinotifyevent(&"event", argc, args...): an event his HUD's
    // scripts handle (`score_event`, medals, rank up).
    for name in ["luinotifyevent", "luinotifyeventtoplayer"] {
        vm.bind(name, true, |vm, world, s, a| {
            let client = super::entnum(vm, s).unwrap_or(u32::MAX);
            let mut parts = vec![text(vm, a, 0)];
            parts.extend(a.iter().skip(2).map(|v| vm.to_text(v).replace(['|', ';'], " ")));
            let mut f = feed(world);
            let n = f.next();
            let list = f.lui.entry(client).or_default();
            list.push(format!("{n}|{}", parts.join("|")));
            let len = list.len();
            list.drain(..len.saturating_sub(12));
            Ok(Value::Undefined)
        });
    }
}

/// Each tick: every player's screen values (see `ScreenFeed`) as client
/// dvars.
/// The BO2MP_SETTING_<name> test overrides, read once (not each tick).
fn setting_overrides() -> &'static [(String, String)] {
    static OVERRIDES: std::sync::OnceLock<Vec<(String, String)>> = std::sync::OnceLock::new();
    OVERRIDES.get_or_init(|| {
        std::env::vars()
            .filter_map(|(k, v)| {
                let name = k.get(..14).filter(|p| p.eq_ignore_ascii_case("BO2MP_SETTING_")).map(|_| k[14..].to_owned())?;
                Some((name, v))
            })
            .collect()
    })
}

pub(super) fn publish_screen(world: &mut World) {
    let (scores, timeleft, clients) = {
        let zm = world.resource::<Zm>();
        let score = |t: &str| zm.team_scores.get(t).copied().unwrap_or(0);
        let left = (zm.game_end_time > 0).then(|| ((i64::from(zm.game_end_time) - zm.now_ms).max(0) / 1000).to_string());
        let clients: Vec<(u32, gsc_t6::ObjRef)> = zm.players.iter().map(|(c, p)| (*c, p.obj)).collect();
        (format!("{},{}", score("allies"), score("axis")), left.unwrap_or_default(), clients)
    };
    // bo2mp: the match's start countdown: the milliseconds until play
    // starts while the scripts hold the prematch period
    // (`level.bo2mp_prematchend`, set as the countdown starts), "" else.
    let now_ms = world.resource::<Zm>().now_ms;
    let prematch = super::with_vm(world, |vm, _| {
        let (on, end) = (vm.intern("inprematchperiod"), vm.intern("bo2mp_prematchend"));
        let on = gsc_t6::truthy(&vm.raw_field(vm.level, on));
        let end = vm.raw_field(vm.level, end).as_int();
        end.filter(|_| on).map(|end| (i64::from(end) - now_ms).max(0).to_string())
    })
    .flatten()
    .unwrap_or_default();
    // bo2mp: the free-for-all result for the modern HUD's end screen
    // (`level.bo2mp_outcome`, see mp_globallogic.gsc), "" until the end.
    let outcome = super::with_vm(world, |vm, _| {
        let f = vm.intern("bo2mp_outcome");
        match vm.raw_field(vm.level, f) {
            gsc_t6::Value::Undefined => String::new(),
            v => vm.to_text(&v),
        }
    })
    .unwrap_or_default();
    // The game type's settings the HUD reads (score limit, rounds), with
    // the BO2MP_SETTING_<name> test overrides the server uses.
    let settings = {
        let zm = world.resource::<Zm>();
        let overrides = setting_overrides();
        zm.gamesettings
            .iter()
            .map(|(k, v)| {
                let v = overrides.iter().find(|(n, _)| n.eq_ignore_ascii_case(k)).map_or(v, |(_, o)| o);
                format!("{k}={v}")
            })
            .collect::<Vec<_>>()
            .join(",")
    };
    let teams: Vec<(u32, String, String)> = super::with_vm(world, |vm, _| {
        let f = vm.intern("team");
        let gametype = vm.dvars.get("g_gametype").cloned().unwrap_or_default();
        clients.iter().map(|(c, o)| (*c, vm.to_text(&vm.raw_field(*o, f)), gametype.clone())).collect()
    })
    .unwrap_or_default();
    let (menus, lui, feed_line, cols, minimap) = {
        let f = feed(world);
        (f.menus.clone(), f.lui.clone(), f.feed.join(";"), f.cols.join(","), f.minimap.clone())
    };
    // The minimap's range: the world distance from its middle to its edge
    // (the map's compassmaxrange, the distance across the box; none set: 2048, measured against real BO2 on Aftermath).
    let range = super::with_vm(world, |vm, _| {
        vm.dvars.iter().find(|(k, _)| k.eq_ignore_ascii_case("compassmaxrange")).map(|(_, v)| v.clone())
    })
    .flatten()
    .unwrap_or_else(|| "2048".to_owned());
    let minimap = if minimap.is_empty() { minimap } else { format!("{minimap},{range}") };
    // The killcam (`_killcam.gsc` sets `self.killcam` while one plays and
    // `level.infinalkillcam` for the final one): "<kind>|<killer>" with kind
    // 1 (killcam) or 2 (final killcam), "" for none. BO2's engine sets
    // BIT_IN_KILLCAM / BIT_FINAL_KILLCAM from it and the HUD's killcam
    // widget shows the card of `GetPredictedClientNum` (the killer).
    // Kind 3 is the card at his death before the killcam opens: he is out
    // of play, his killer known, no killcam yet.
    let killers = feed(world).killers.clone();
    let spectating: std::collections::HashSet<u32> =
        world.resource::<Zm>().players.iter().filter(|(_, p)| p.sessionstate == "spectator").map(|(c, _)| *c).collect();
    // `sessionstate` is a built-in player field the sim keeps in `Zm`, not a raw script field
    // (reading it off the object gave "undefined", so the death card never ended at the respawn).
    let states: std::collections::HashMap<u32, String> =
        world.resource::<Zm>().players.iter().map(|(c, p)| (*c, p.sessionstate.clone())).collect();
    let killcams: std::collections::HashMap<u32, String> = super::with_vm(world, |vm, _| {
        let (kc, sc, fin) = (vm.intern("killcam"), vm.intern("spectatorclient"), vm.intern("infinalkillcam"));
        let (at, ge) = (vm.intern("archivetime"), vm.intern("gameended"));
        let final_on = gsc_t6::truthy(&vm.raw_field(vm.level, fin));
        clients
            .iter()
            .filter_map(|(c, o)| {
                if gsc_t6::truthy(&vm.raw_field(*o, kc)) {
                    let killer = vm.raw_field(*o, sc).as_float().map_or(-1, |v| v as i32);
                    return Some((*c, format!("{}|{killer}", if final_on { 2 } else { 1 })));
                }
                // The engine's own replay (a spectator with an archive time and
                // someone to watch) is a killcam even where the script left
                // `killcam` unset: the final one at the end of the match.
                let wc = vm.raw_field(*o, sc).as_float().map_or(-1, |v| v as i32);
                if spectating.contains(c) && wc >= 0 && vm.raw_field(*o, at).as_float().unwrap_or(0.0) > 0.0 {
                    let ended = gsc_t6::truthy(&vm.raw_field(vm.level, ge));
                    return Some((*c, format!("{}|{wc}", if final_on || ended { 2 } else { 1 })));
                }
                // (The intermission is no card: his death card goes with the match.)
                let out = !matches!(states.get(c).map_or("playing", String::as_str), "playing" | "intermission");
                killers.get(c).filter(|_| out).map(|(k, _)| (*c, format!("3|{k}")))
            })
            .collect()
    })
    .unwrap_or_default();
    // Once he was seen out of play and is back, the death card is over.
    {
        let mut f = feed(world);
        for (c, (_, seen)) in f.killers.iter_mut() {
            if killcams.contains_key(c) {
                *seen = true;
            }
        }
        f.killers.retain(|c, (_, seen)| killcams.contains_key(c) || !*seen);
    }
    // bo2mp: the killer's perks for the modern HUD's killcam (BO2 draws
    // them bottom right, `showperks`): `bo2mp_killcam_perks`, "icon|name"
    // by `;`, "" with no killcam.
    let mut killcam_perks: std::collections::HashMap<u32, String> = std::collections::HashMap::new();
    for (c, kc) in &killcams {
        let Some(killer) = kc
            .strip_prefix("1|")
            .or_else(|| kc.strip_prefix("2|"))
            .and_then(|k| k.parse::<i32>().ok())
            .filter(|k| *k >= 0)
        else {
            continue;
        };
        if let Some((seen, rows)) = feed(world).killcam_perks.get(c)
            && seen == kc
        {
            killcam_perks.insert(*c, rows.clone());
            continue;
        }
        let obj = world.resource::<Zm>().players.get(&(killer as u32)).map(|p| p.obj);
        let rows = obj.and_then(|o| super::with_vm(world, |vm, world| perk_rows(vm, world, &Value::Object(o)))).unwrap_or_default();
        diag::info!(Sim, "bo2mp killcam perks: client {c} watches {killer}: {rows:?}");
        feed(world).killcam_perks.insert(*c, (kc.clone(), rows.clone()));
        killcam_perks.insert(*c, rows);
    }
    feed(world).killcam_perks.retain(|c, _| killcams.contains_key(c));
    // bo2mp: every player's scoreboard values past the snapshot's score,
    // kills and deaths: out of play (dead, or watching a killcam), his
    // assists and the game type's own columns (`setscoreboardcolumns`:
    // plants, defuses, captures, defends, ...), the engine's counts or else
    // the scripts' fields: `bo2mp_board`, "<client>|<out>|<col>=<n>,..." by `;`.
    let board = {
        let mut names: Vec<String> = vec!["assists".to_owned()];
        for col in cols.split(',').map(str::trim) {
            if !col.is_empty() && !matches!(col, "score" | "kills" | "deaths" | "kdratio") && !names.iter().any(|n| n == col) {
                names.push(col.to_owned());
            }
        }
        super::with_vm(world, |vm, world| {
            let zm = world.resource::<Zm>();
            clients
                .iter()
                .map(|(c, o)| {
                    let p = zm.players.get(c);
                    let out = p.is_some_and(|p| matches!(p.sessionstate.as_str(), "dead" | "spectator"));
                    let values = names
                        .iter()
                        .map(|k| {
                            let v = p.and_then(|p| p.stats.get(k.as_str()).copied()).or_else(|| {
                                let f = vm.intern(k);
                                vm.raw_field(*o, f).as_float().map(|v| v as i32)
                            });
                            format!("{k}={}", v.unwrap_or(0))
                        })
                        .collect::<Vec<_>>()
                        .join(",");
                    format!("{c}|{}|{values}", u8::from(out))
                })
                .collect::<Vec<_>>()
                .join(";")
        })
        .unwrap_or_default()
    };
    // bo2mp: Resurgence's HUD values for each player (`self.bo2mp_br`, see
    // br_resurgence.gsc `br_hud_publish`), "" outside Resurgence.
    let br: std::collections::HashMap<u32, String> = super::with_vm(world, |vm, _| {
        let f = vm.intern("bo2mp_br");
        clients
            .iter()
            .filter_map(|(c, o)| match vm.raw_field(*o, f) {
                gsc_t6::Value::Undefined => None,
                v => Some((*c, vm.to_text(&v))),
            })
            .collect()
    })
    .unwrap_or_default();
    // The objectives each player's HUD marks (bots have no HUD).
    let views: Vec<(u32, String)> =
        teams.iter().filter(|(c, _, _)| !super::is_bot(world, *c)).map(|(c, t, _)| (*c, t.clone())).collect();
    super::objectives::publish(world, &views);
    // Everyone's team (the scoreboard's sides): "<client>:<team>,...".
    let all_teams =teams.iter().map(|(c, t, _)| format!("{c}:{t}")).collect::<Vec<_>>().join(",");
    for (c, team, gametype) in teams {
        if super::is_bot(world, c) {
            continue;
        }
        for (k, v) in [
            ("bo2mp_scores", scores.clone()),
            ("bo2mp_timeleft", timeleft.clone()),
            ("bo2mp_team", team),
            ("bo2mp_gametype", gametype),
            ("bo2mp_menu", menus.get(&c).cloned().unwrap_or_default()),
            ("bo2mp_lui", lui.get(&c).map(|l| l.join(";")).unwrap_or_default()),
            ("bo2mp_feed", feed_line.clone()),
            ("bo2mp_teams", all_teams.clone()),
            ("bo2mp_cols", cols.clone()),
            ("bo2mp_settings", settings.clone()),
            ("bo2mp_minimap", minimap.clone()),
            ("bo2mp_killcam", killcams.get(&c).cloned().unwrap_or_default()),
            ("bo2mp_killcam_perks", killcam_perks.get(&c).cloned().unwrap_or_default()),
            ("bo2mp_board", board.clone()),
            ("bo2mp_prematch", prematch.clone()),
            ("bo2mp_outcome", outcome.clone()),
            ("bo2mp_br", br.get(&c).cloned().unwrap_or_default()),
        ] {
            super::set_client_dvar(world, c, k, &v);
        }
    }
}
