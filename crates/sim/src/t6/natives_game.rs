//! Game-wide builtins: tables, session and game-mode queries, precache,
//! players lists, HUD element stand-ins, traces, and quiet stubs for what
//! has no engine side yet (client fields, effects, sounds) - those are
//! listed once in the log by `unbound` when a script reaches one not here.

use std::collections::BTreeMap;

use bevy_ecs::prelude::World;
use gsc_t6::{ObjKind, ObjRef, Value, Vm};

use super::{Zm, arg, entnum, frame, int, is_player, list, num, text, vec3};
use crate::world::ClientId;

/// Notify `name` on `obj` after `ms` (server time).
pub(super) fn notify_later(
    _vm: &mut Vm<World>,
    world: &mut World,
    obj: ObjRef,
    name: &str,
    ms: i64,
) {
    let mut zm = world.resource_mut::<Zm>();
    let at = zm.now_ms + ms;
    zm.timers.push((at, obj, name.to_owned()));
}

/// `notify_later`, unless the same notify is already on its way to `obj`.
/// BO2's engine raises `bad_path` from the actor's path state, at most once a
/// frame; one per failed `setgoalpos` call multiplied on Origins: a zombie on
/// the tank runs several breadcrumb threads (`set_zombie_on_tank` threads
/// `find_flesh` again), each woken by every `bad_path`, each queueing another,
/// till one tick ran 85 s of script.
pub(super) fn notify_later_once(
    vm: &mut Vm<World>,
    world: &mut World,
    obj: ObjRef,
    name: &str,
    ms: i64,
) {
    let pending = world
        .resource::<Zm>()
        .timers
        .iter()
        .any(|t| t.1 == obj && t.2 == name);
    if !pending {
        notify_later(vm, world, obj, name, ms);
    }
}

/// Deliver due `notify_later`s.
pub(super) fn deliver_timers(vm: &mut Vm<World>, world: &mut World, now: i64) {
    let due: Vec<(i64, ObjRef, String)> = {
        let mut zm = world.resource_mut::<Zm>();
        let (due, keep): (Vec<_>, Vec<_>) = zm.timers.drain(..).partition(|t| t.0 <= now);
        zm.timers = keep;
        due
    };
    for (_, o, n) in due {
        if vm.alive(o) {
            vm.notify_str(world, o, &n, &[]);
        }
    }
    let replies: Vec<(i64, ObjRef, String, String)> = {
        let mut zm = world.resource_mut::<Zm>();
        let (due, keep): (Vec<_>, Vec<_>) = zm.menu_replies.drain(..).partition(|t| t.0 <= now);
        zm.menu_replies = keep;
        due
    };
    for (_, o, menu, response) in replies {
        if vm.alive(o) {
            let (m, r) = (vm.string(&menu), vm.string(&response));
            diag::info!(Sim, "bo2mp menu response (test aid): {menu} {response}");
            vm.notify_str(world, o, "menuresponse", &[m, r]);
        }
    }
}

fn table_cell(world: &World, table: &str, row: usize, col: usize) -> Option<String> {
    let zm = world.resource::<Zm>();
    let t = zm.tables.get(&table.to_ascii_lowercase())?;
    if row >= t.rows || col >= t.columns {
        return None;
    }
    t.cells.get(row * t.columns + col).cloned()
}

fn table_find(world: &World, table: &str, col: usize, key: &str) -> Option<usize> {
    let zm = world.resource::<Zm>();
    let t = zm.tables.get(&table.to_ascii_lowercase())?;
    (0..t.rows).find(|&r| {
        t.cells
            .get(r * t.columns + col)
            .is_some_and(|c| c.eq_ignore_ascii_case(key))
    })
}

/// bo2mp: a multiplayer game type setting from the settings files; one
/// they do not name is 0 (as the engine's default for an unset setting).
/// BO2MP_SETTING_<name>=<value> overrides one (a test aid: a short time
/// limit, a low score limit).
fn mp_setting(settings: &BTreeMap<String, String>, name: &str) -> Value {
    let key = name.to_ascii_lowercase();
    let over = std::env::var(format!("BO2MP_SETTING_{key}")).ok();
    let raw = over.as_deref().or_else(|| super::br::setting(&key)).or(settings.get(&key).map(String::as_str));
    match raw {
        Some(v) if v.contains('.') => Value::Float(v.parse().unwrap_or(0.0)),
        Some(v) => Value::Int(v.parse().unwrap_or(0)),
        // BO2 plays its announcer unless a game type turns it off:
        // `_globallogic_audio.gsc:496,525` skip every line when
        // `level.allowannouncer` (this setting) is 0, so an unnamed one must
        // not read as off.
        None if key == "allowannouncer" => Value::Int(1),
        None => Value::Int(0),
    }
}

/// bo2zm M4: the game settings BO2's front end picked for this game (Custom
/// Games: starting round, difficulty, magic, headshots only), by lower-case
/// name; they win over the defaults below.
static FRONT_END_SETTINGS: std::sync::Mutex<Vec<(String, i32)>> = std::sync::Mutex::new(Vec::new());

/// bo2zm M4: the front end's settings for the next game (all of them; an
/// empty list = the defaults).
pub fn set_t6_game_settings(settings: Vec<(String, i32)>) {
    if let Ok(mut s) = FRONT_END_SETTINGS.lock() {
        *s = settings
            .into_iter()
            .map(|(k, v)| (k.to_ascii_lowercase(), v))
            .collect();
    }
}

/// Game type settings Zombies reads (`getgametypesetting`); others are 0.
fn gametype_setting(name: &str) -> Value {
    let key = name.to_ascii_lowercase();
    // (The test aid's starting round still wins.)
    if !(key == "startround" && std::env::var_os("IW4L_T6_START_ROUND").is_some())
        && let Some(v) = FRONT_END_SETTINGS
            .lock()
            .ok()
            .and_then(|s| s.iter().find(|(k, _)| *k == key).map(|(_, v)| *v))
    {
        return Value::Int(v);
    }
    match key.as_str() {
        "teamcount" => Value::Int(1),
        "roundlimit" => Value::Int(1),
        "playernumlives" => Value::Int(0),
        "allowhitmarkers" => Value::Int(1),
        "allowfinalkillcam" => Value::Int(0),
        // IW4L_T6_START_ROUND=<n>: start there (a test aid: runners and
        // sprinters at once).
        "startround" => Value::Int(
            std::env::var("IW4L_T6_START_ROUND")
                .ok()
                .and_then(|v| v.parse().ok())
                .unwrap_or(1),
        ),
        "zmdifficulty" => Value::Int(1),
        "magic" => Value::Int(1),
        // His zm/gamesettings_zstandard.cfg (and _zclassic): no hellhound
        // rounds in Nuketown (the scripts' dog AI never ran here: dog rounds
        // stood two still dogs far off for minutes).
        "allowdogs" => Value::Int(0),
        "headshotsonly" => Value::Int(0),
        "allowpowerups" => Value::Int(1),
        "cleansedloadout" => Value::Int(0),
        _ => Value::Int(0),
    }
}

/// Path nodes whose `key` (targetname by default) is `value`.
fn nodes_matching(vm: &mut Vm<World>, world: &mut World, a: &[Value]) -> Vec<Value> {
    let want = text(vm, a, 0);
    let key = match arg(a, 1) {
        Value::Undefined => "targetname".to_owned(),
        v => vm.to_text(v).to_ascii_lowercase(),
    };
    let k = vm.intern(&key);
    let objs: Vec<ObjRef> = world.resource::<Zm>().node_objs.clone();
    objs.into_iter()
        .filter(|o| matches!(vm.raw_field(*o, k), Value::Str(s) if vm.str(s) == want))
        .map(Value::Object)
        .collect()
}

/// Vehicle path nodes (`info_vehicle_node`) whose `key` is `value`.
fn vehicle_nodes(vm: &mut Vm<World>, world: &mut World, a: &[Value]) -> Vec<Value> {
    let want = text(vm, a, 0);
    let key = match arg(a, 1) {
        Value::Undefined => "targetname".to_owned(),
        v => vm.to_text(v).to_ascii_lowercase(),
    };
    let k = vm.intern(&key);
    let nodes: Vec<ObjRef> = world
        .resource::<Zm>()
        .ents
        .values()
        .filter(|e| e.classname.starts_with("info_vehicle_node"))
        .filter_map(|e| e.obj)
        .collect();
    nodes
        .into_iter()
        .filter(|o| matches!(vm.raw_field(*o, k), Value::Str(s) if vm.str(s) == want))
        .map(Value::Object)
        .collect()
}

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
    f!("tablelookup", |vm, world, _, a| {
        let table = text(vm, a, 0);
        let col = int(a, 1)? as usize;
        let key = text(vm, a, 2);
        let out = int(a, 3)? as usize;
        let v = table_find(world, &table, col, &key)
            .and_then(|r| table_cell(world, &table, r, out))
            .unwrap_or_default();
        Ok(vm.string(&v))
    });
    f!("tablelookupistring", |vm, world, _, a| {
        let table = text(vm, a, 0);
        let col = int(a, 1)? as usize;
        let key = text(vm, a, 2);
        let out = int(a, 3)? as usize;
        let v = table_find(world, &table, col, &key)
            .and_then(|r| table_cell(world, &table, r, out))
            .unwrap_or_default();
        Ok(Value::IStr(vm.intern(&v)))
    });
    f!("tablelookuprownum", |vm, world, _, a| {
        let table = text(vm, a, 0);
        let col = int(a, 1)? as usize;
        let key = text(vm, a, 2);
        Ok(Value::Int(
            table_find(world, &table, col, &key).map_or(-1, |r| r as i32),
        ))
    });
    f!("tablelookupcolumnforrow", |vm, world, _, a| {
        let table = text(vm, a, 0);
        let row = int(a, 1)? as usize;
        let col = int(a, 2)? as usize;
        let v = table_cell(world, &table, row, col).unwrap_or_default();
        Ok(vm.string(&v))
    });
    f!("tablelookuprowcount", |vm, world, _, a| {
        let table = text(vm, a, 0).to_ascii_lowercase();
        Ok(Value::Int(
            world
                .resource::<Zm>()
                .tables
                .get(&table)
                .map_or(0, |t| t.rows as i32),
        ))
    });
    f!("getgametypesetting", |vm, world, _, a| {
        let name = text(vm, a, 0);
        // bo2mp: a multiplayer game type's own settings (his files'
        // mp/gamesettings_default.cfg then mp/gamesettings_<type>.cfg).
        let zm = world.resource::<Zm>();
        if zm.mp {
            return Ok(mp_setting(&zm.gamesettings, &name));
        }
        // bo2zm M4: Zombies' (the front end's picks, then the defaults).
        Ok(gametype_setting(&name))
    });
    // bo2mp: a script's own limits (his trickshot script's time and score
    // limits): later reads (`updategametypedvars`) see them.
    f!("setgametypesetting", |vm, world, _, a| {
        let name = text(vm, a, 0).to_ascii_lowercase();
        let value = vm.to_text(arg(a, 1));
        let mut zm = world.resource_mut::<Zm>();
        if zm.mp && !name.is_empty() {
            zm.gamesettings.insert(name, value);
        }
        Ok(Value::Undefined)
    });
    f!("sessionmodeisonlinegame", |_, _, _, _| Ok(Value::Int(0)));
    f!("sessionmodeisprivate", |_, _, _, _| Ok(Value::Int(0)));
    f!("sessionmodeisprivateonlinegame", |_, _, _, _| Ok(
        Value::Int(0)
    ));
    f!("sessionmodeissystemlink", |_, _, _, _| Ok(Value::Int(0)));
    f!("sessionmodeiszombiesgame", |_, world, _, _| Ok(Value::Int(
        i32::from(!world.resource::<Zm>().mp)
    )));
    f!("sessionmodeiscampaigngame", |_, _, _, _| Ok(Value::Int(0)));
    f!("gamemodeisusingxp", |_, _, _, _| Ok(Value::Int(0)));
    f!("gamemodeisusingstats", |_, _, _, _| Ok(Value::Int(0)));
    f!("gamemodeismode", |_, _, _, _| Ok(Value::Int(0)));
    f!("ispregame", |_, _, _, _| Ok(Value::Int(0)));
    // Zombies runs on the multiplayer side (_createfx's fx_init sets up
    // the exploders only there).
    f!("ismp", |_, _, _, _| Ok(Value::Int(1)));
    f!("issplitscreen", |_, _, _, _| Ok(Value::Int(0)));
    f!("isonlinegame", |_, _, _, _| Ok(Value::Int(0)));
    // The party size, or everyone already in (onallplayersready waits until
    // this many are playing; one means solo).
    // bo2zm: Zombies waits for the host's whole party; bo2mp: multiplayer
    // keeps 1 (its scripts do not wait on a party).
    f!("getnumexpectedplayers", |_, world, _, _| Ok(Value::Int(
        if world.resource::<Zm>().mp {
            1
        } else {
            super::expected_players().max(world.resource::<Zm>().players.len() as i32)
        }
    )));
    f!("getnumconnectedplayers", |_, world, _, _| Ok(Value::Int(
        world.resource::<Zm>().players.len() as i32
    )));
    // His profile: mature content (BO2's default on; is_mature() gates
    // heads popping off on headshots and limbs flying off).
    f!("getlocalprofileint", |vm, _, _, a| Ok(Value::Int(
        i32::from(
            matches!(vm.to_text(arg(a, 0)).as_str(), "cg_mature" | "cg_blood")
                && super::profile_mature()
        )
    )));
    f!("hasdlcavailable", |_, _, _, _| Ok(Value::Int(1)));
    // _zm_weapons::is_weapon_included asks the player too.
    m!("hasdlcavailable", |_, _, _, _| Ok(Value::Int(1)));
    // No god mode (TranZit's bus checks for it before hurting him).
    f!("isgodmode", |_, _, _, _| Ok(Value::Int(0)));
    f!("getnorthyaw", |_, _, _, _| Ok(Value::Float(0.0)));
    f!("getaitriggerflags", |_, _, _, _| Ok(Value::Int(2)));
    f!("getvehicletriggerflags", |_, _, _, _| Ok(Value::Int(4)));
    f!("getfreeactorcount", |_, _, _, _| Ok(Value::Int(32)));
    f!("aretexturesloaded", |_, _, _, _| Ok(Value::Int(1)));
    f!("getutc", |_, _, _, _| {
        let s = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map_or(0, |d| d.as_secs());
        Ok(Value::Int(s as i32))
    });
    f!("getminbitcountfornum", |_, _, _, a| {
        let n = int(a, 0)?.max(0) as u32;
        Ok(Value::Int((32 - n.leading_zeros()) as i32))
    });
    f!("loadfx", |vm, world, _, a| {
        let name = text(vm, a, 0);
        let mut zm = world.resource_mut::<Zm>();
        if let Some(i) = zm.fx.iter().position(|f| *f == name) {
            return Ok(Value::Int(i as i32 + 1));
        }
        zm.fx.push(name);
        Ok(Value::Int(zm.fx.len() as i32))
    });
    // Only players in the game: one still connecting (before `begin`) is not
    // listed, as in the engine (_visionset_mgr::monitor reads every listed
    // player's _player_entnum, set on "connected", which follows `begin`).
    f!("getplayers", |_, world, _, _| {
        let ps: Vec<Value> = world
            .resource::<Zm>()
            .players
            .values()
            .filter(|p| p.begun)
            .map(|p| Value::Object(p.obj))
            .collect();
        Ok(list(ps))
    });
    f!("isplayer", |vm, world, _, a| Ok(Value::bool(is_player(
        vm,
        world,
        arg(a, 0)
    ))));
    f!("isai", |_, _, _, _| Ok(Value::Int(0)));
    f!("isalive", |vm, world, _, a| {
        let v = arg(a, 0);
        if !vm.is_defined(v) {
            return Ok(Value::Int(0));
        }
        if let Some(n) = entnum(vm, v)
            && let Some(p) = world.resource::<Zm>().players.get(&n).cloned()
        {
            let h = frame(world).player(ClientId(n)).map_or(0, |ps| ps.health);
            return Ok(Value::bool(p.sessionstate == "playing" && h > 0));
        }
        Ok(Value::bool(entnum(vm, v).is_some()))
    });
    // An animation's length in seconds, and the times (0..1 of its length)
    // a notetrack of it fires (Buried's head chopper slices by them).
    f!("getanimlength", |vm, world, _, a| {
        let len = anim_of(vm, world, arg(a, 0)).map_or(0.0, |x| {
            if x.framerate > 0.0 {
                f32::from(x.numframes) / x.framerate
            } else {
                0.0
            }
        });
        Ok(Value::Float(len))
    });
    f!("getnotetracktimes", |vm, world, _, a| {
        let note = text(vm, a, 1);
        let times = anim_of(vm, world, arg(a, 0))
            .map(|x| {
                x.notifies
                    .iter()
                    .filter(|(n, _)| n.eq_ignore_ascii_case(&note))
                    .map(|(_, t)| Value::Float(*t))
                    .collect()
            })
            .unwrap_or_default();
        Ok(list(times))
    });
    f!("getwatcherweapons", |_, _, _, _| Ok(list(Vec::new())));
    // Weapons a player picks back up (their weapon file's bRetrievable).
    f!("getretrievableweapons", |vm, world, _, _| {
        // bo2mp: multiplayer's, as `<name>_mp` (BO2's _weaponobjects strips
        // the suffix itself).
        if world.resource::<Zm>().mp {
            let names = world.resource::<Zm>().retrievable_weapons.clone();
            let v = names.iter().map(|n| vm.string(n)).collect();
            return Ok(list(v));
        }
        // bo2zm: the weapons BO2 marks retrievable (placed equipment,
        // knives): their names, so _weaponobjects makes each a watcher
        // ("equip_turbine").
        let f = frame(world);
        let ws: Vec<u32> = (1..f.weapon_combat_len() as u32)
            .filter(|&w| f.weapon_combat_row(w).is_some_and(|r| r.retrievable))
            .collect();
        drop(f);
        let names = ws
            .into_iter()
            .map(|w| vm.string(&super::weapon_text(world, w)))
            .collect();
        Ok(list(names))
    });
    f!("getplayerspawnid", |vm, _, _, a| Ok(
        entnum(vm, arg(a, 0)).map_or(Value::Int(0), |n| Value::Int(n as i32))
    ));
    f!("getweaponindexfromname", |vm, world, _, a| {
        let n = text(vm, a, 0);
        Ok(Value::Int(super::weapon(world, &n).map_or(0, |w| w as i32)))
    });
    for name in [
        "getweaponfiresound",
        "getweaponfiresoundplayer",
        "getweaponpickupsound",
        "getweaponpickupsoundplayer",
    ] {
        vm.bind(name, false, |vm, _, _, _| Ok(vm.string("")));
    }
    f!("iprintln", |vm, _, _, a| {
        let t = text(vm, a, 0);
        diag::info!(Sim, "bo2zm t6 iprintln: {t}");
        Ok(Value::Undefined)
    });
    f!("iprintlnbold", |vm, _, _, a| {
        let t = text(vm, a, 0);
        diag::info!(Sim, "bo2zm t6 iprintlnbold: {t}");
        Ok(Value::Undefined)
    });
    for method in [false, true] {
        vm.bind("blank", method, |_, _, _, _| Ok(Value::Undefined));
        vm.bind("issplitscreen", method, |_, _, _, _| Ok(Value::Int(0)));
    }
    for name in [
        "setspawnpointrandomvariation",
        "setentertime",
        "setclientcompass",
        "setclienthudhardcore",
        "setclientplayerpushamount",
        "setclientaimlockonpitchstrength",
        "setclientplayersprinttime",
        "setclientnumlives",
        "disabledeathstreak",
    ] {
        vm.bind(name, false, |_, _, _, _| Ok(Value::Undefined));
        vm.bind(name, true, |_, _, _, _| Ok(Value::Undefined));
    }
    f!("getvehiclenode", |vm, world, _, a| {
        let v = vehicle_nodes(vm, world, a);
        Ok(v.into_iter().next().unwrap_or_default())
    });
    f!("getvehiclenodearray", |vm, world, _, a| {
        let v = vehicle_nodes(vm, world, a);
        Ok(list(v))
    });
    f!("weaponfuellife", |_, _, _, _| Ok(Value::Int(0)));
    // earthquake(scale, seconds, origin, radius): the screen shake the
    // engine's scene effects present (as the IW games' scripts get it).
    f!("earthquake", |_, world, _, a| {
        let now = crate::level_time_ms(world.resource::<crate::step::StepRequest>().tick);
        let mut quake = crate::ScriptEarthquake {
            id: 0,
            scale: num(a, 0).unwrap_or(0.0),
            duration_ms: (num(a, 1).unwrap_or(0.0) * 1000.0).round() as i32,
            origin: vec3(a, 2).unwrap_or([0.0; 3]),
            radius: num(a, 3).unwrap_or(0.0),
            start_ms: now,
        };
        if quake.valid() && quake.duration_ms > 0 {
            let mut runtime = world.resource_mut::<crate::script::Runtime>();
            let engine = &mut runtime.engine;
            engine.earthquakes.retain(|q| q.active(now));
            if engine.earthquakes.len() < crate::MAX_SCRIPT_EARTHQUAKES {
                quake.id = engine.next_earthquake;
                engine.next_earthquake = engine.next_earthquake.wrapping_add(1);
                engine.earthquakes.push(quake);
            }
        }
        Ok(Value::Undefined)
    });
    f!("numremoteclients", |_, _, _, _| Ok(Value::Int(0)));
    f!("addboxinfluencer", |_, _, _, _| Ok(Value::Int(1)));
    for name in [
        "adddemobookmark",
        "setinitialplayersconnected",
        "addinfluencerspawnpoints",
        "addspawnpoints",
        "clearspawnpoints",
        "setenableinfluencer",
        "spawnpointsetrandomvariation",
    ] {
        vm.bind(name, false, |_, _, _, _| Ok(Value::Undefined));
    }
    for name in [
        "stopburning",
        "playerknockback",
        "cameraactivate",
        "camerasetposition",
        "camerasetlookat",
        "logstring",
    ] {
        vm.bind(name, true, |_, _, _, _| Ok(Value::Undefined));
    }
    vm.bind("enableusability", true, |vm, world, s, _| {
        if let Ok(id) = super::client(vm, world, s) {
            let mut f = frame(world);
            if f.client_meta(id).is_some() {
                f.client_meta_mut(id).controls.usability_disabled = false;
            }
        }
        Ok(Value::Undefined)
    });
    vm.bind("disableusability", true, |vm, world, s, _| {
        if let Ok(id) = super::client(vm, world, s) {
            let mut f = frame(world);
            if f.client_meta(id).is_some() {
                f.client_meta_mut(id).controls.usability_disabled = true;
            }
        }
        Ok(Value::Undefined)
    });
    f!("getnode", |vm, world, _, a| {
        let v = nodes_matching(vm, world, a);
        Ok(v.into_iter().next().unwrap_or_default())
    });
    f!("getnodearray", |vm, world, _, a| {
        let v = nodes_matching(vm, world, a);
        Ok(list(v))
    });
    // Path node links the scripts change (Die Rise's escape pod, ffotd fixes).
    f!("linknodes", |_, world, _, a| {
        if let Some((x, y)) = node_pair(world, a) {
            world.resource_mut::<Zm>().nav.link(x, y);
        }
        Ok(Value::Undefined)
    });
    f!("unlinknodes", |_, world, _, a| {
        if let Some((x, y)) = node_pair(world, a) {
            world.resource_mut::<Zm>().nav.unlink(x, y);
        }
        Ok(Value::Undefined)
    });
    f!("nodesarelinked", |_, world, _, a| {
        let linked = node_pair(world, a).is_some_and(|(x, y)| world.resource::<Zm>().nav.linked(x, y));
        Ok(Value::Int(linked as i32))
    });
    f!("deletepathnode", |_, world, _, a| {
        if let Some(n) = node_index(world, arg(a, 0)) {
            world.resource_mut::<Zm>().nav.delete(n);
        }
        Ok(Value::Undefined)
    });
    f!("getallnodes", |_, world, _, _| {
        let v = world
            .resource::<Zm>()
            .node_objs
            .iter()
            .map(|o| Value::Object(*o))
            .collect();
        Ok(list(v))
    });
    // The node nearest a point (Buried's ghosts and Leroy path from it).
    f!("getnearestnode", |_, world, _, a| {
        let p = arg(a, 0).as_vec3().unwrap_or([0.0; 3]);
        let zm = world.resource::<Zm>();
        Ok(zm
            .nav
            .nearest(p)
            .and_then(|i| zm.node_objs.get(i as usize).copied())
            .map_or(Value::Undefined, Value::Object))
    });
    // BO2's GetNodesInRadius: (origin, radius, min radius, height = 512,
    // kind, at most = 256). The nodes within the radius across and the
    // height up or down, of the kind ("Cover": every cover kind; none:
    // every kind but a bad node); at most that many (BO2 stops in its node
    // tree's order, ours in node order); for the sorted one nearest first
    // (3D); only then are those inside the min radius (across), without
    // links or bad dropped.
    vm.bind("getnodesinradius", false, |vm, world, _, a| nodes_in_radius(vm, world, a, false));
    vm.bind("getnodesinradiussorted", false, |vm, world, _, a| nodes_in_radius(vm, world, a, true));
    fn nodes_in_radius(vm: &mut Vm<World>, world: &mut World, a: &[Value], sorted: bool) -> Result<Value, String> {
        use super::nav::{NODE_COVER_KINDS, NODE_TYPE_NAMES};
        let center = arg(a, 0).as_vec3().unwrap_or([0.0; 3]);
        let radius = arg(a, 1).as_float().unwrap_or(0.0);
        let min = arg(a, 2).as_float().unwrap_or(0.0);
        let height = arg(a, 3).as_float().unwrap_or(512.0);
        let kinds: u32 = match arg(a, 4) {
            Value::Undefined => !1,
            _ => {
                let k = text(vm, a, 4);
                if k.eq_ignore_ascii_case("Cover") {
                    NODE_COVER_KINDS
                } else {
                    NODE_TYPE_NAMES
                        .iter()
                        .position(|n| n.eq_ignore_ascii_case(&k))
                        .map_or(!1, |i| 1 << i)
                }
            }
        };
        let most = match arg(a, 5) {
            Value::Undefined => 256,
            v => v.as_int().unwrap_or(256).clamp(1, 256) as usize,
        };
        let zm = world.resource::<Zm>();
        let mut hits: Vec<(f32, u32)> = zm
            .nav
            .nodes
            .iter()
            .enumerate()
            .filter(|(_, n)| {
                let d = gsc_t6::math::sub(n.origin, center);
                d[0] * d[0] + d[1] * d[1] <= radius * radius
                    && kinds & (1 << (n.ty & 31)) != 0
                    && (n.spawnflags & 1 == 0 || kinds & 1 != 0)
                    && d[2] * d[2] <= height * height
            })
            .take(most)
            .map(|(i, n)| {
                let d = gsc_t6::math::sub(n.origin, center);
                (d[0] * d[0] + d[1] * d[1] + d[2] * d[2], i as u32)
            })
            .collect();
        if sorted {
            hits.sort_by(|x, y| x.0.total_cmp(&y.0));
        }
        let out = hits
            .into_iter()
            .filter(|&(_, i)| {
                let n = &zm.nav.nodes[i as usize];
                let d = gsc_t6::math::sub(n.origin, center);
                (min * min == 0.0 || min * min <= d[0] * d[0] + d[1] * d[1]) && !n.links.is_empty() && n.ty != 0
            })
            .filter_map(|(_, i)| zm.node_objs.get(i as usize).map(|o| Value::Object(*o)))
            .collect();
        Ok(list(out))
    }
    // Every path node within a sphere (zm_prison::title_update_main_end
    // marks the ones by the docks `no_teleport`).
    f!("getanynodearray", |_, world, _, a| {
        let center = arg(a, 0).as_vec3().unwrap_or([0.0; 3]);
        let radius = arg(a, 1).as_float().unwrap_or(0.0);
        let zm = world.resource::<Zm>();
        let v = zm
            .nav
            .nodes
            .iter()
            .zip(&zm.node_objs)
            .filter(|(n, _)| gsc_t6::math::length(gsc_t6::math::sub(n.origin, center)) <= radius)
            .map(|(_, o)| Value::Object(*o))
            .collect();
        Ok(list(v))
    });
    // bo2zm M3: weapon facts the scripts ask (class, fire type).
    f!("weaponclass", |vm, world, _, a| {
        let w = super::weapon(world, &text(vm, a, 0)).unwrap_or(0);
        let class = super::frame(world)
            .combat_facts_for(w)
            .map_or(-1, |f| f.weap_class);
        // (The class is IW4-numbered, as the catalog keeps every game's.)
        let name = if class < 0 { "none" } else { crate::t6_playeranim::t6_weapon_class_name(class) };
        Ok(vm.string(name))
    });
    // A weapon's world model (the box's floating gun, a wall buy's).
    f!("getweaponmodel", |vm, world, _, a| {
        let w = super::weapon(world, &text(vm, a, 0)).unwrap_or(0);
        let model = super::frame(world)
            .weapon_world_model(w)
            .map(|(m, _)| m.to_owned())
            .unwrap_or_default();
        Ok(vm.string(&model))
    });
    // A weapon's localized name (a dug-up gun's pickup hint).
    f!("getweapondisplayname", |vm, world, _, a| {
        let w = super::weapon(world, &text(vm, a, 0)).ok();
        let key = w.and_then(|w| {
            super::frame(world)
                .weapon_display_name(w)
                .map(str::to_owned)
        });
        Ok(match key {
            Some(k) => Value::IStr(vm.intern(&k)),
            None => vm.string(""),
        })
    });
    // Weapon facts the scripts ask about a gun by name.
    fn facts(
        vm: &Vm<World>,
        world: &mut World,
        a: &[Value],
    ) -> Option<weapon_iw4::WeaponCombatFacts> {
        let w = super::weapon(world, &text(vm, a, 0)).ok()?;
        super::frame(world).combat_facts_for(w)
    }
    f!("isweaponprimary", |vm, world, _, a| {
        Ok(Value::bool(
            facts(vm, world, a).is_some_and(|f| f.inventory_type == 0),
        ))
    });
    f!("isweaponequipment", |vm, world, _, a| {
        Ok(Value::bool(
            facts(vm, world, a).is_some_and(|f| f.inventory_type == 2),
        ))
    });
    f!("weaponclipsize", |vm, world, _, a| {
        Ok(Value::Int(facts(vm, world, a).map_or(0, |f| f.clip_size)))
    });
    f!("weaponmaxammo", |vm, world, _, a| {
        Ok(Value::Int(facts(vm, world, a).map_or(0, |f| f.max_ammo)))
    });
    f!("weaponstartammo", |vm, world, _, a| {
        Ok(Value::Int(facts(vm, world, a).map_or(0, |f| f.start_ammo)))
    });
    f!("weaponfiretime", |vm, world, _, a| {
        Ok(Value::Float(
            facts(vm, world, a).map_or(0.0, |f| f.fire_time_ms as f32 / 1000.0),
        ))
    });
    f!("weaponreloadtime", |vm, world, _, a| {
        Ok(Value::Float(
            facts(vm, world, a).map_or(0.0, |f| f.reload_time_ms as f32 / 1000.0),
        ))
    });
    f!("weaponisboltaction", |vm, world, _, a| {
        Ok(Value::bool(
            facts(vm, world, a).is_some_and(|f| f.bolt_action),
        ))
    });
    f!("weaponaltweaponname", |vm, world, _, a| {
        let alt = facts(vm, world, a).map_or(0, |f| f.alternate_weapon);
        let name = if alt == 0 {
            "none".to_owned()
        } else {
            super::weapon_text(world, alt)
        };
        Ok(vm.string(&name))
    });
    // A Black Ops II dual-wield weapon's left-hand weapon ("none").
    f!("weapondualwieldweaponname", |vm, world, _, a| {
        let w = super::weapon(world, &text(vm, a, 0)).unwrap_or(0);
        let left = frame(world)
            .combat_facts_for(w)
            .map_or(0, |f| f.dual_wield_weapon);
        if left == 0 {
            return Ok(vm.string("none"));
        }
        let name = super::weapon_text(world, left);
        Ok(vm.string(&name))
    });
    f!("weaponisdualwield", |vm, world, _, a| {
        Ok(Value::bool(
            facts(vm, world, a).is_some_and(|f| f.dual_wield_weapon != 0),
        ))
    });
    f!("weaponisgasweapon", |_, _, _, _| Ok(Value::Int(0)));
    f!("weapontype", |vm, world, _, a| {
        let w = super::weapon(world, &text(vm, a, 0)).unwrap_or(0);
        let ty = super::frame(world)
            .combat_facts_for(w)
            .map_or(-1, |f| f.weap_type);
        let name = match ty {
            0 => "bullet",
            1 => "grenade",
            2 => "projectile",
            3 => "riotshield",
            _ => "none",
        };
        Ok(vm.string(name))
    });
    f!("weaponinventorytype", |vm, world, _, a| {
        let w = super::weapon(world, &text(vm, a, 0)).unwrap_or(0);
        let ty = super::frame(world)
            .combat_facts_for(w)
            .map_or(-1, |f| f.inventory_type);
        let name = match ty {
            0 => "primary",
            1 => "offhand",
            2 => "item",
            3 => "altmode",
            _ => "none",
        };
        Ok(vm.string(name))
    });
    f!("weaponissemiauto", |vm, world, _, a| {
        let w = super::weapon(world, &text(vm, a, 0)).unwrap_or(0);
        let semi = super::frame(world)
            .combat_facts_for(w)
            .is_some_and(|f| f.fire_type == 1);
        Ok(Value::bool(semi))
    });
    f!("setroundsplayed", |vm, world, _, a| {
        let n = arg(a, 0).as_int().unwrap_or(0);
        world.resource_mut::<Zm>().rounds_played = n;
        let _ = vm;
        Ok(Value::Undefined)
    });
    for name in [
        "reportmtu",
        "recordzombieroundstart",
        "recordzombieroundend",
        "stopallrumbles",
        "recordmatchsummaryzombieendgamedata",
        "recordplayermatchend",
        "recordzombiezone",
        "uploadstats",
        "setmatchtalkflag",
        "setmatchflag",
    ] {
        vm.bind(name, false, |_, _, _, _| Ok(Value::Undefined));
    }
    // HUD elements start with the engine's defaults (scripts do
    // `elem.y = elem.y - 130`).
    fn hud(vm: &mut Vm<World>) -> Value {
        let o = vm.alloc_object(ObjKind::HudElem(0));
        for (k, v) in [
            ("x", Value::Int(0)),
            ("y", Value::Int(0)),
            ("z", Value::Int(0)),
            ("alpha", Value::Float(1.0)),
            ("fontscale", Value::Float(1.0)),
            ("sort", Value::Int(0)),
            ("color", Value::Vec3([1.0, 1.0, 1.0])),
            ("glowalpha", Value::Float(0.0)),
            ("glowcolor", Value::Vec3([0.0, 0.0, 0.0])),
            ("width", Value::Int(0)),
            ("height", Value::Int(0)),
        ] {
            let f = vm.intern(k);
            vm.set_raw_field(o, f, v);
        }
        Value::Object(o)
    }
    for name in [
        "newhudelem",
        "newclienthudelem",
        "newteamhudelem",
        "newscorehudelem",
        "newdebughudelem",
        "newdamageindicatorhudelem",
    ] {
        vm.bind(name, false, |vm, _, _, _| Ok(hud(vm)));
    }
    m!("destroy", |vm, world, s, _| {
        if let Some(o) = s.as_obj()
            && matches!(vm.kind(o), Some(ObjKind::HudElem(_)))
        {
            vm.free_object(world, o);
        }
        Ok(Value::Undefined)
    });
    for name in [
        "settext",
        "setvalue",
        "setshader",
        "settimer",
        "settimerup",
        "settenthstimer",
        "settenthstimerup",
        "setclock",
        "setclockup",
        "fadeovertime",
        "moveovertime",
        "changefontscaleovertime",
        "setwaypoint",
        "settargetent",
        "cleartargetent",
        "setpulsefx",
        "setcod7decodefx",
        "setredeemfx",
        "reset",
    ] {
        vm.bind(name, true, |_, _, _, _| Ok(Value::Undefined));
    }
    // Traces against the map (entities aside): bullettrace and
    // groundtrace give the engine's result array, the physics traces the
    // stopping point, the "passed" ones whether nothing was in the way.
    fn world_trace(
        world: &mut World,
        start: [f32; 3],
        end: [f32; 3],
        mask: u32,
    ) -> (f32, [f32; 3], [f32; 3]) {
        let t = frame(world).trace_static_world(start, end, [0.0; 3], [0.0; 3], mask);
        let f = t.fraction.clamp(0.0, 1.0);
        let pos = std::array::from_fn(|i| start[i] + (end[i] - start[i]) * f);
        (f, pos, t.normal)
    }
    fn trace_array(
        vm: &mut Vm<World>,
        (fraction, position, normal): (f32, [f32; 3], [f32; 3]),
    ) -> Value {
        let mut arr = gsc_t6::Array::new();
        let k = |vm: &mut Vm<World>, s: &str| gsc_t6::Key::Str(vm.intern(s));
        let surface = vm.string(if fraction < 1.0 { "default" } else { "none" });
        arr.set(k(vm, "fraction"), Value::Float(fraction));
        arr.set(k(vm, "position"), Value::Vec3(position));
        arr.set(
            k(vm, "normal"),
            Value::Vec3(if fraction < 1.0 { normal } else { [0.0; 3] }),
        );
        arr.set(k(vm, "surfacetype"), surface);
        Value::array(arr)
    }
    const SHOT: u32 = crate::bullet_collision::MASK_SHOT;
    const SOLID: u32 = crate::bullet_collision::MASK_PLAYER_SOLID;
    f!("bullettrace", |vm, world, _, a| {
        let (s, e) = (vec3(a, 0)?, vec3(a, 1)?);
        let r = world_trace(world, s, e, SHOT);
        Ok(trace_array(vm, r))
    });
    f!("groundtrace", |vm, world, _, a| {
        let (s, e) = (vec3(a, 0)?, vec3(a, 1)?);
        let r = world_trace(world, s, e, SOLID);
        Ok(trace_array(vm, r))
    });
    f!("bullettracepassed", |_, world, _, a| {
        let (s, e) = (vec3(a, 0)?, vec3(a, 1)?);
        Ok(Value::bool(world_trace(world, s, e, SHOT).0 >= 1.0))
    });
    f!("sighttracepassed", |_, world, _, a| {
        let (s, e) = (vec3(a, 0)?, vec3(a, 1)?);
        Ok(Value::bool(world_trace(world, s, e, SHOT).0 >= 1.0))
    });
    // ent damageconetrace(from[, ignore, dir, cone_deg]): 1 when a damage
    // trace from `from` reaches the body's middle or a point 15 units off
    // it (side to side, then up and down), else 0. With `dir` a point only
    // counts inside the cone (65 degrees unless given). The Paralyzer and
    // claymores ask this before they hurt anything.
    m!("damageconetrace", |vm, world, s, a| {
        let from = vec3(a, 0)?;
        let Some(o) = super::origin_of(vm, world, s) else {
            return Ok(Value::Float(0.0));
        };
        let body = entnum(vm, s).is_some_and(|n| {
            let zm = world.resource::<Zm>();
            zm.players.contains_key(&n) || zm.actors.contains_key(&n)
        });
        let mid = [o[0], o[1], o[2] + if body { 36.0 } else { 0.0 }];
        let (dx, dy) = (from[0] - mid[0], from[1] - mid[1]);
        let len = (dx * dx + dy * dy).sqrt();
        let side = if len < 0.001 { [0.0, 1.0] } else { [-dy / len, dx / len] };
        let points = [
            mid,
            [mid[0] + side[0] * 15.0, mid[1] + side[1] * 15.0, mid[2]],
            [mid[0] - side[0] * 15.0, mid[1] - side[1] * 15.0, mid[2]],
            [mid[0], mid[1], mid[2] + 15.0],
            [mid[0], mid[1], mid[2] - 15.0],
        ];
        let cone = vec3(a, 2).ok().map(|d| {
            let deg = match a.get(3) {
                Some(Value::Float(f)) => *f,
                Some(Value::Int(i)) => *i as f32,
                _ => 65.0,
            };
            (d, deg.to_radians().cos())
        });
        for p in points {
            if let Some((d, cos)) = cone {
                let v = [p[0] - from[0], p[1] - from[1], p[2] - from[2]];
                let (vl, dl) = (
                    (v[0] * v[0] + v[1] * v[1] + v[2] * v[2]).sqrt(),
                    (d[0] * d[0] + d[1] * d[1] + d[2] * d[2]).sqrt(),
                );
                if vl > 0.001 && dl > 0.001 && (v[0] * d[0] + v[1] * d[1] + v[2] * d[2]) / (vl * dl) < cos {
                    continue;
                }
            }
            if world_trace(world, from, p, SHOT).0 >= 1.0 {
                return Ok(Value::Float(1.0));
            }
        }
        Ok(Value::Float(0.0))
    });
    // physicstrace(start, end[, mins, maxs, ignore]): a trace array like
    // bullettrace's (BO2's scripts read trace["position"]), swept with the
    // box when one is given.
    f!("physicstrace", |vm, world, _, a| {
        let (s, e) = (vec3(a, 0)?, vec3(a, 1)?);
        let (mins, maxs) = (
            vec3(a, 2).unwrap_or([0.0; 3]),
            vec3(a, 3).unwrap_or([0.0; 3]),
        );
        let t = frame(world).trace_static_world(s, e, mins, maxs, SOLID);
        let f = if t.startsolid != 0 {
            0.0
        } else {
            t.fraction.clamp(0.0, 1.0)
        };
        let pos = std::array::from_fn(|i| s[i] + (e[i] - s[i]) * f);
        Ok(trace_array(vm, (f, pos, t.normal)))
    });
    f!("playerphysicstrace", |_, world, _, a| {
        let (s, e) = (vec3(a, 0)?, vec3(a, 1)?);
        let t = frame(world).trace_static_world(
            s,
            e,
            crate::bullet_collision::PLAYER_MINS,
            crate::bullet_collision::PLAYER_MAXS,
            SOLID,
        );
        let f = if t.startsolid != 0 {
            0.0
        } else {
            t.fraction.clamp(0.0, 1.0)
        };
        Ok(Value::Vec3(std::array::from_fn(|i| {
            s[i] + (e[i] - s[i]) * f
        })))
    });
    // bo2mp: a spawn on top of a living player (within a player's width,
    // a player's height) would telefrag him: the spawn scripts pick another.
    f!("positionwouldtelefrag", |_, world, _, a| {
        let p = arg(a, 0).as_vec3().unwrap_or([0.0; 3]);
        Ok(Value::bool(telefrag(world, p)))
    });
    f!("boundswouldtelefrag", |_, world, _, a| {
        let p = arg(a, 0).as_vec3().unwrap_or([0.0; 3]);
        Ok(Value::bool(telefrag(world, p)))
    });
    fn telefrag(world: &mut World, p: [f32; 3]) -> bool {
        let ids: Vec<u32> = world.resource::<Zm>().players.keys().copied().collect();
        let f = frame(world);
        ids.into_iter().any(|n| {
            f.player(ClientId(n)).is_some_and(|ps| {
                ps.health > 0
                    && ((ps.origin[0] - p[0]).powi(2) + (ps.origin[1] - p[1]).powi(2)).sqrt() < 32.0
                    && (ps.origin[2] - p[2]).abs() < 72.0
            })
        })
    }
    // bo2mp: `setmatchflag("game_ended", 1)`: every client's `bo2mp_game_ended`
    // dvar, which opens the scoreboard once his HUD is back (the intermission).
    f!("setmatchflag", |vm, world, _, a| {
        // bo2mp: `round_end_killcam` (the final killcam of a round that is
        // not the match's last, `_killcam::finalkillcam`): the modern HUD's
        // killcam is titled ROUND WINNING KILLCAM then (`bo2mp_round_killcam`).
        if text(vm, a, 0) == "round_end_killcam" {
            let on = if int(a, 1).unwrap_or(0) != 0 { "1" } else { "" };
            let ids: Vec<u32> = world.resource::<Zm>().players.keys().copied().collect();
            for n in ids {
                super::set_client_dvar(world, n, "bo2mp_round_killcam", on);
            }
        }
        if text(vm, a, 0) == "game_ended" {
            let on = if int(a, 1).unwrap_or(0) != 0 { "1" } else { "" };
            let ids: Vec<u32> = world.resource::<Zm>().players.keys().copied().collect();
            for n in ids {
                super::set_client_dvar(world, n, "bo2mp_game_ended", on);
            }
        }
        Ok(Value::Undefined)
    });
    // bo2mp: a script's vision (mpOutro at the match's end, then the map's
    // own): the frame's grade switches to it at once (its fade is not run).
    // A vision no zone holds (another game's map names its own) brings the
    // map's own grade back, so `mpIntro`'s greyscale does not stay.
    f!("visionsetnaked", |vm, _, _, a| {
        let name = text(vm, a, 0);
        let found = asset_world::set_t6_vision(&name);
        if found {
            diag::info!(Sim, "bo2mp: visionsetnaked {name} (vision found: true)");
        } else {
            diag::info!(Sim, "bo2mp: visionsetnaked {name} (vision found: false; the map's own grade, none for another game's map)");
        }
        Ok(Value::Undefined)
    });
    // The game's end (end_game): the session takes it from here.
    f!("exitlevel", |_, world, _, _| {
        world.resource_mut::<crate::script::Runtime>().exit_level = true;
        Ok(Value::Undefined)
    });
    for name in [
        "precachemodel",
        "precacheshader",
        "precachestring",
        "precacheitem",
        "precacherumble",
        "precachemenu",
        "precacheshellshock",
        "precacheanimstatedef",
        "precachevehicle",
        "precacheleaderboards",
        "precachestatusicon",
        "precacheheadicon",
        "precachelocationselector",
        "precacheturret",
        "registerclientfield",
        "codesetclientfield",
        "codesetworldclientfield",
        "codesetplayerstateclientfield",
        "makedvarserverinfo",
        "setmatchflag",
        "setmatchtalkflag",
        "setscoreboardcolumns",
        "setdemointermissionpoint",
        "setminimap",
        "visionsetnaked",
        "visionsetnight",
        "setvisionsetnaked",
        "setsunlight",
        "resetsunlight",
        "setexpfog",
        "setvolfog",
        "logstring",
        "logprint",
        "recordmatchbegin",
        "gamehistorystartmatch",
        "gamehistoryfinishmatch",
        "bbpostdemostreamstatsforround",
        "uploadstats",
        "recordplayerstats",
        "resettimeout",
        "setslowmotion",
        "objective_add",
        "objective_state",
        "objective_position",
        "objective_onentity",
        "objective_delete",
        "objective_visibleteams",
        "objective_icon",
        "objective_team",
        // the capture bar's progress and who is capturing (_gameobjects,
        // Origins' capture zones): the compass draws no objectives.
        "objective_setprogress",
        "objective_setplayerusing",
        "objective_clearplayerusing",
        "objective_setgamemodeflags",
        // the MP match clock (_globallogic, Turned's zcleansed): zombies
        // shows none.
        "setgameendtime",
        "playfx",
        "playfxontag",
        "playfxontagforclients",
        "stopfx",
        "playsoundatposition",
        "playsound",
        "physicsexplosionsphere",
        "physicsexplosioncylinder",
        "playrumbleonposition",
        "setclientnamemode",
        "setvoipattachment",
        "deactivateclientexploder",
        "scriptmodelsuseanimtree",
        "disablegrenadesuicide",
        "enablezombies",
        "disablezombies",
        "setdvarbool",
        "sethideonclientwhendead",
        // a client draw distance (map mains, e.g. Tranzit)
        "setculldist",
    ] {
        vm.bind(name, false, |_, _, _, _| Ok(Value::Undefined));
    }
}

/// The animation a script `%name` value names (its xanim's timing).
fn anim_of(vm: &Vm<World>, world: &World, v: &Value) -> Option<super::T6Anim> {
    let Value::Anim(i) = v else {
        return None;
    };
    let (_, name) = *vm.program.anims.get(*i as usize)?;
    world.resource::<Zm>().anims.get(vm.str(name)).cloned()
}

/// The path node index a script node object stands for.
fn node_index(world: &World, v: &Value) -> Option<u32> {
    let o = v.as_obj()?;
    let zm = world.resource::<Zm>();
    zm.node_objs.iter().position(|n| *n == o).map(|i| i as u32)
}

/// The two path nodes a link builtin names.
fn node_pair(world: &World, a: &[Value]) -> Option<(u32, u32)> {
    Some((node_index(world, arg(a, 0))?, node_index(world, arg(a, 1))?))
}
