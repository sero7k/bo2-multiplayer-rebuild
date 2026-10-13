//! Entities: the map's entities and structs at level start, `spawn`,
//! `getent(array)`, models, visibility, links, movers, triggers' hints.

use bevy_ecs::prelude::World;
use gsc_t6::{Key, ObjKind, ObjRef, Value, Vm};

use super::{Ent, Zm, arg, entnum, flag, is_player, list, num, origin_of, text, vec3};

type R = Result<Value, String>;

/// Map entity keys the engine types (the rest stay strings).
fn typed(vm: &mut Vm<World>, key: &str, val: &str, entity: bool) -> Value {
    let floats = |s: &str| -> Option<[f32; 3]> {
        let p: Vec<f32> = s
            .split_whitespace()
            .filter_map(|x| x.parse().ok())
            .collect();
        (p.len() == 3).then(|| [p[0], p[1], p[2]])
    };
    match key {
        // The engine's vector fields: a door's open turn (`script_angles`)
        // and a debris pile's slide (`script_vector`) go to rotateto/moveto.
        "origin" | "angles" | "script_angles" | "script_vector" => match floats(val) {
            Some(v) => Value::Vec3(v),
            None => vm.string(val),
        },
        "spawnflags" | "count" | "health" | "dmg" | "script_forcespawn" | "script_int"
            if entity =>
        {
            Value::Int(val.trim().parse().unwrap_or(0))
        }
        "radius" | "height" | "speed" | "wait" | "delay" if entity => {
            Value::Float(val.trim().parse().unwrap_or(0.0))
        }
        // Names stay text even when they look like numbers.
        "targetname" | "target" | "script_noteworthy" | "script_string" | "classname" | "model"
        | "script_flag" | "script_label" | "script_parameters" | "script_linkto"
<<<<<<< /home/claude/bo2-multiplayer-rebuild/crates/sim/src/t6/natives_ent.rs
        | "script_linkname" | "script_sound" | "script_fxid" | "script_animation" => vm.string(val),
=======
        | "script_linkname" | "script_sound" | "script_fxid" | "script_animation"
        | "script_location" => vm.string(val),
>>>>>>> /tmp/claude-0/zmhead/crates/sim/src/t6/natives_ent.rs
        // Numbers are numbers (BO2's scripts compare `self.zombie_cost` with
        // a score and halve it; `trigger.script_chance` with a randomfloat).
        _ => {
            let t = val.trim();
            if let Ok(i) = t.parse::<i32>() {
                Value::Int(i)
            } else if let Ok(f) = t.parse::<f32>()
                && t.contains('.')
            {
                Value::Float(f)
            } else {
                vm.string(val)
            }
        }
    }
}

/// Create the map's entities (`script_struct` blocks through
/// `codescripts/struct::createstruct`); path nodes are kept for the AI.
pub(super) fn spawn_map_entities(vm: &mut Vm<World>, world: &mut World, texts: &[String]) {
    let createstruct = vm
        .program
        .find(&vm.strings, "codescripts/struct", "createstruct");
    let struct_s = vm.intern("struct");
    let (mut ents, mut structs, mut nodes) = (0, 0, 0);
    for (text_index, text) in texts.iter().enumerate() {
        // bo2mp: each block's place in the map's text (blocks with a key,
        // as `asset_world` counts them for the session's own models).
        let mut place = 0u32;
        for e in asset_t6_entities(text) {
            let ordinal = (!e.is_empty()).then(|| {
                place += 1;
                place - 1
            });
            let ordinal = ordinal.filter(|_| text_index == 0);
            let cls = e
                .iter()
                .find(|(k, _)| k.eq_ignore_ascii_case("classname"))
                .map_or("", |(_, v)| v.as_str())
                .to_owned();
            if cls.starts_with("node_") {
                nodes += 1;
                continue;
            }
<<<<<<< /home/claude/bo2-multiplayer-rebuild/crates/sim/src/t6/natives_ent.rs
            // bo2mp: the map's north (getnorthyaw: the minimap's turn).
            if cls == "worldspawn" {
                if let Some((_, v)) = e.iter().find(|(k, _)| k.eq_ignore_ascii_case("northyaw")) {
                    vm.dvars.insert("northyaw".to_owned(), v.trim().to_owned());
                }
            }
=======
>>>>>>> /tmp/claude-0/zmhead/crates/sim/src/t6/natives_ent.rs
            if cls == "script_struct" {
                let Some(cs) = createstruct else { continue };
                let level = Value::Object(vm.level);
                vm.spawn(world, cs, level, vec![]);
                let Value::Array(a) = vm.raw_field(vm.level, struct_s) else {
                    continue;
                };
                let Some(Value::Object(s)) = a.get(&Key::Int(a.len() as i32 - 1)) else {
                    continue;
                };
                for (k, v) in &e {
                    let key = k.to_ascii_lowercase();
                    if key == "classname" {
                        continue;
                    }
                    let val = typed(vm, &key, v, false);
                    let f = vm.intern(&key);
                    vm.set_raw_field(s, f, val);
                }
                structs += 1;
                continue;
            }
            let n = world.resource_mut::<Zm>().alloc_entnum();
            let obj = vm.alloc_object(ObjKind::Entity(n));
            let mut ent = Ent {
                obj: Some(obj),
                classname: cls.clone(),
                map: true,
                map_ordinal: ordinal,
                solid: true,
                ..Default::default()
            };
<<<<<<< /home/claude/bo2-multiplayer-rebuild/crates/sim/src/t6/natives_ent.rs
            if let Some(ordinal) = ordinal {
                super::map_natives::note(world, ordinal, &cls, &e);
            }
=======
            let mut targetname = None;
>>>>>>> /tmp/claude-0/zmhead/crates/sim/src/t6/natives_ent.rs
            for (k, v) in &e {
                let key = k.to_ascii_lowercase();
                match key.as_str() {
                    "classname" => continue,
                    "targetname" => targetname = Some(v.clone()),
                    "origin" => {
                        if let Value::Vec3(p) = typed(vm, "origin", v, true) {
                            ent.origin = p;
                        }
                        continue;
                    }
                    "angles" => {
                        if let Value::Vec3(p) = typed(vm, "angles", v, true) {
                            ent.angles = p;
                        }
                        continue;
                    }
                    "angle" => {
                        ent.angles = [0.0, v.trim().parse().unwrap_or(0.0), 0.0];
                        continue;
                    }
                    "model" if v.starts_with('*') => ent.brush = Some(v.clone()),
                    "model" => {
                        ent.model = v.clone();
                        continue;
                    }
                    "radius" => ent.radius = v.trim().parse().unwrap_or(0.0),
                    "height" => ent.height = v.trim().parse().unwrap_or(0.0),
<<<<<<< /home/claude/bo2-multiplayer-rebuild/crates/sim/src/t6/natives_ent.rs
                    // bo2mp: breaks piece by piece (t6/destructible.rs).
                    "destructibledef" => ent.destructible = Some(v.trim().to_ascii_lowercase()),
=======
                    "destructibledef" => ent.destructible = v.to_ascii_lowercase(),
>>>>>>> /tmp/claude-0/zmhead/crates/sim/src/t6/natives_ent.rs
                    _ => {}
                }
                let val = typed(vm, &key, v, true);
                let f = vm.intern(&key);
                vm.set_raw_field(obj, f, val);
            }
            if ent.brush.is_some() {
                ent.spawn_angles = Some(ent.angles);
            }
            if cls.starts_with("zbarrier") {
<<<<<<< /home/claude/bo2-multiplayer-rebuild/crates/sim/src/t6/natives_ent.rs
                ent.zbarrier = Some(super::zbarrier::ZBarrier::from_keys(&e));
=======
                let mut z = super::zbarrier::ZBarrier::from_keys(&e);
                z.def = super::zbarrier::def_for(world, &e);
                ent.zbarrier = Some(z);
            }
            let mut zm = world.resource_mut::<Zm>();
            zm.ents.insert(n, ent);
            if let Some(t) = targetname {
                zm.nav.attach_movers(&t, n);
>>>>>>> /tmp/claude-0/zmhead/crates/sim/src/t6/natives_ent.rs
            }
            ents += 1;
        }
    }
    diag::info!(
        Sim,
        "bo2zm t6: map {ents} entities, {structs} structs, {nodes} path nodes (kept for the AI)"
    );
}

/// The map's entity blocks as key/value lists.
fn asset_t6_entities(text: &str) -> Vec<Vec<(String, String)>> {
    let mut out = Vec::new();
    let mut cur: Option<Vec<(String, String)>> = None;
    let mut quoted = Vec::new();
    let mut chars = text.chars().peekable();
    while let Some(c) = chars.next() {
        match c {
            '{' => {
                cur = Some(Vec::new());
                quoted.clear();
            }
            '}' => {
                if let Some(e) = cur.take() {
                    out.push(e);
                }
                quoted.clear();
            }
            '"' => {
                let mut s = String::new();
                for c in chars.by_ref() {
                    if c == '"' {
                        break;
                    }
                    s.push(c);
                }
                quoted.push(s);
                if quoted.len() == 2 {
                    let v = quoted.pop().unwrap();
                    let k = quoted.pop().unwrap();
                    if let Some(e) = cur.as_mut() {
                        e.push((k, v));
                    }
                }
            }
            _ => {}
        }
    }
    out
}

/// Entities whose `key` equals `value` (all with no key).
fn matching(vm: &mut Vm<World>, world: &mut World, a: &[Value]) -> Vec<Value> {
    let want = match arg(a, 0) {
        Value::Undefined => None,
        v => Some(vm.to_text(v)),
    };
    let key = match arg(a, 1) {
        Value::Undefined => None,
        v => Some(vm.to_text(v).to_ascii_lowercase()),
    };
    let ents: Vec<(u32, ObjRef, String)> = world
        .resource::<Zm>()
        .ents
        .iter()
        .filter_map(|(n, e)| e.obj.map(|o| (*n, o, e.classname.clone())))
        .collect();
    let players: Vec<ObjRef> = world
        .resource::<Zm>()
        .players
        .values()
        .map(|p| p.obj)
        .collect();
    let mut out = Vec::new();
    let (Some(want), Some(key)) = (want, key) else {
        return ents.iter().map(|(_, o, _)| Value::Object(*o)).collect();
    };
    if key == "classname" {
        if want == "player" {
            return players.into_iter().map(Value::Object).collect();
        }
        for (_, o, c) in &ents {
            if *c == want && vm.alive(*o) {
                out.push(Value::Object(*o));
            }
        }
        return out;
    }
    let Some(k) = vm.strings.find(&key) else {
        return out;
    };
    let Some(w) = vm.strings.find(&want) else {
        return out;
    };
    for (_, o, _) in ents {
        if !vm.alive(o) {
            continue;
        }
        if let Value::Str(s) = vm.raw_field(o, k)
            && s == w
        {
            out.push(Value::Object(o));
        }
    }
    out
}

fn ent_mut<'a>(world: &'a mut World, n: u32) -> Option<bevy_ecs::prelude::Mut<'a, Zm>> {
    let zm = world.resource_mut::<Zm>();
    zm.ents.contains_key(&n).then_some(zm)
}

/// The entity number of `self` (not a player), or an error.
fn me(vm: &Vm<World>, world: &World, s: &Value) -> Result<u32, String> {
    let n = entnum(vm, s).ok_or("not an entity")?;
    if world.resource::<Zm>().ents.contains_key(&n) {
        Ok(n)
    } else if world.resource::<Zm>().players.contains_key(&n) {
        Err("a player, not an entity".into())
    } else {
        Err("deleted entity".into())
    }
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
    f!("getent", |vm, world, _, a| {
        let v = matching(vm, world, a);
        Ok(v.into_iter().next().unwrap_or_default())
    });
    f!("getentarray", |vm, world, _, a| {
<<<<<<< /home/claude/bo2-multiplayer-rebuild/crates/sim/src/t6/natives_ent.rs
        // An undefined class name: BO2's _spawning re-adds its spawn points
        // that way after clearing them (updateallspawnpoints reads the list
        // it just cleared); an empty answer leaves no spawn points and aborts
        // the level. The answer is every spawn point of the classes the game
        // type registered (level.spawn_point_class_names), not every entity:
        // path and launch points (the Warthog's, Hellstorm's) put the map
        // centre (findboxcenter of the spawn bounds) 5800 units up.
        if matches!(arg(a, 0), Value::Undefined)
            && !matches!(arg(a, 1), Value::Undefined)
            && vm.to_text(arg(a, 1)).eq_ignore_ascii_case("classname")
        {
            let f = vm.intern("spawn_point_class_names");
            let names: Vec<String> = match vm.raw_field(vm.level, f) {
                Value::Array(arr) => {
                    let vals: Vec<Value> = arr.read().values_in_order().cloned().collect();
                    vals.iter()
                        .filter(|v| !matches!(v, Value::Undefined))
                        .map(|v| vm.to_text(v))
                        .collect()
                }
                _ => Vec::new(),
            };
            if !names.is_empty() {
                // They come back not yet set up (`inited` cleared): the
                // clear before this emptied level.spawnpoints, and
                // addspawnpointsinternal only re-adds points it sets up, so
                // set-up ones would leave the bots' wander list (and the
                // spawn bounds) empty.
                let inited = vm.intern("inited");
                let mut v = Vec::new();
                let mut seen = std::collections::BTreeSet::new();
                for name in names {
                    let q = [vm.string(&name), vm.string("classname")];
                    for e in matching(vm, world, &q) {
                        if let Value::Object(o) = &e {
                            if !seen.insert((o.index, o.generation)) {
                                continue;
                            }
                            vm.set_raw_field(*o, inited, Value::Undefined);
                        }
                        v.push(e);
                    }
                }
                return Ok(list(v));
            }
        }
=======
>>>>>>> /tmp/claude-0/zmhead/crates/sim/src/t6/natives_ent.rs
        let v = matching(vm, world, a);
        Ok(list(v))
    });
    // Spawners only (the `actor_*` entities), matched like getentarray:
    // Buried's fuel step spawns its ghosts from these.
    f!("getspawnerarray", |vm, world, _, a| {
        let v = matching(vm, world, a);
        let zm = world.resource::<Zm>();
        let v = v
            .into_iter()
            .filter(|v| {
                entnum(vm, v)
                    .and_then(|n| zm.ents.get(&n))
                    .is_some_and(|e| e.classname.starts_with("actor_"))
            })
            .collect();
        Ok(list(v))
    });
    f!("spawn", |vm, world, _, a| {
        let cls = text(vm, a, 0);
        let origin = arg(a, 1).as_vec3().unwrap_or([0.0; 3]);
        let n = world.resource_mut::<Zm>().alloc_entnum();
        let obj = vm.alloc_object(ObjKind::Entity(n));
        let mut ent = Ent {
            obj: Some(obj),
            classname: cls.clone(),
            origin,
            solid: true,
            ..Default::default()
        };
        if cls.starts_with("trigger_radius") {
            ent.radius = arg(a, 3).as_float().unwrap_or(0.0);
            ent.height = arg(a, 4).as_float().unwrap_or(0.0);
            // bo2mp: a use trigger spawned with no size (a turret's or a
            // claymore's pick-up, `spawn("trigger_radius_use", pos)`) is
            // reached from the player's use radius.
            if cls == "trigger_radius_use" && ent.radius <= 0.0 && world.resource::<Zm>().mp {
                let reach = vm
                    .dvars
                    .get("player_useradius")
                    .and_then(|v| v.parse::<f32>().ok())
                    .unwrap_or(128.0);
                ent.radius = reach;
                ent.height = reach;
            }
        }
        if cls.starts_with("trigger_box") {
            ent.box_dims = Some([
                arg(a, 3).as_float().unwrap_or(64.0),
                arg(a, 4).as_float().unwrap_or(64.0),
                arg(a, 5).as_float().unwrap_or(64.0),
            ]);
        }
        if std::env::var_os("IW4L_T6_BOUNDSLOG").is_some() && cls.starts_with("trigger") {
            diag::info!(
                Sim,
                "bo2zm t6 spawned {cls} at {origin:?} box {:?} radius {}",
                ent.box_dims,
                ent.radius
            );
        }
        world.resource_mut::<Zm>().ents.insert(n, ent);
        Ok(Value::Object(obj))
    });
    m!("delete", |vm, world, s, _| {
        let Some(n) = entnum(vm, s) else {
            return Ok(Value::Undefined);
        };
        if is_player(vm, world, s) {
            return Err("cannot delete a player".into());
        }
        let obj = s.as_obj();
<<<<<<< /home/claude/bo2-multiplayer-rebuild/crates/sim/src/t6/natives_ent.rs
        let item = {
=======
        // The engine notifies "death" on a deleted entity before freeing it
        // (ScrCmd_Delete): its endons end now, its `waittill("death")`
        // threads run after it is freed (Tranzit's Denizens count down when
        // one burrows away, powerups leave level.random_weapon_powerups; Die
        // Rise's corpse-room count sees `self` undefined and returns).
        if let Some(o) = obj
            && vm.alive(o)
        {
            vm.notify_queued(world, o, "death", &[]);
        }
        {
>>>>>>> /tmp/claude-0/zmhead/crates/sim/src/t6/natives_ent.rs
            let mut zm = world.resource_mut::<Zm>();
            zm.ents.remove(&n);
            zm.movers.list.remove(&n);
<<<<<<< /home/claude/bo2-multiplayer-rebuild/crates/sim/src/t6/natives_ent.rs
            zm.ents.remove(&n).and_then(|e| e.item)
        };
        if let Some(number) = item {
            super::items::forget(world, number);
=======
>>>>>>> /tmp/claude-0/zmhead/crates/sim/src/t6/natives_ent.rs
        }
        if let Some(o) = obj {
            // bo2mp: a deleted entity tells its waiters `death` first; BO2's
            // multiplayer scripts count on it to free a scorestreak's rule
            // (the care package's helicopter is deleted once it has flown
            // off, and only its `death` ends the drop's count).
            if world.resource::<Zm>().mp && vm.alive(o) {
                vm.notify_str(world, o, "death", &[]);
            }
            vm.free_object(world, o);
        }
        Ok(Value::Undefined)
    });
    // physicslaunch(point, force): BO2 makes the model a physics object
    // that falls, comes to rest and is notified `stationary` (a ballistic
    // knife left in a zombie that died, `force_drop_knives_to_ground_on_death`,
    // then moves its pickup trigger onto it). Here it falls straight down
    // under the engine's gravity to the solid floor below and is notified
    // `stationary` there. PLACEHOLDER: the launch force is not applied
    // (the model's mass is not known here).
    m!("physicslaunch", |vm, world, s, _| {
        let Ok(n) = me(vm, world, s) else {
            return Ok(Value::Undefined);
        };
        let (from, linked, now) = {
            let zm = world.resource::<Zm>();
            let e = &zm.ents[&n];
            (e.origin, e.link.is_some(), zm.now_ms)
        };
        let Some(obj) = s.as_obj() else {
            return Ok(Value::Undefined);
        };
        if linked {
            return Ok(Value::Undefined);
        }
        let down = [from[0], from[1], from[2] - 2000.0];
        let t = super::frame(world).trace_static_world(
            from,
            down,
            [0.0; 3],
            [0.0; 3],
            crate::bullet_collision::CONTENTS_SOLID,
        );
        let drop = if t.startsolid == 0 && t.fraction < 1.0 {
            (from[2] - down[2]) * t.fraction
        } else {
            0.0
        };
        let secs = (2.0 * drop / super::movers::GRAVITY).sqrt();
        if secs > 0.0 {
            world
                .resource_mut::<Zm>()
                .movers
                .gravity(n, from, [0.0; 3], now, secs);
        }
        super::natives_game::notify_later(vm, world, obj, "stationary", (secs * 1000.0) as i64);
        Ok(Value::Undefined)
    });
    m!("setmodel", |vm, world, s, a| {
<<<<<<< /home/claude/bo2-multiplayer-rebuild/crates/sim/src/t6/natives_ent.rs
        // bo2mp: a player's body (BO2's faction class scripts) goes to every
        // client, which draws him with it.
        if is_player(vm, world, s) {
            if let Some(n) = entnum(vm, s) {
                let model = text(vm, a, 0);
                super::playeranim::edit_body(world, n, |l| l.model = model);
=======
        // A player's body model (third person) is not drawn here yet, but
        // kept: `self.model` is what Mob's Afterlife body is made from.
        let model = text(vm, a, 0);
        if is_player(vm, world, s) {
            if let Some(n) = entnum(vm, s)
                && let Some(p) = world.resource_mut::<Zm>().players.get_mut(&n)
            {
                p.model = model;
>>>>>>> /tmp/claude-0/zmhead/crates/sim/src/t6/natives_ent.rs
            }
            return Ok(Value::Undefined);
        }
        let n = me(vm, world, s)?;
        if let Some(mut zm) = ent_mut(world, n) {
            let e = zm.ents.get_mut(&n).unwrap();
            // A gun's parts fit only that gun: the box's teddy bear, set on
            // the gun it floated, wears no magazine.
            if e.model != model {
                e.weapon_parts.clear();
            }
            e.model = model;
        }
        Ok(Value::Undefined)
    });
    m!("attach", |vm, world, s, a| {
<<<<<<< /home/claude/bo2-multiplayer-rebuild/crates/sim/src/t6/natives_ent.rs
        if is_player(vm, world, s) {
            if let Some(n) = entnum(vm, s) {
                let added = (text(vm, a, 0), text(vm, a, 1));
                super::playeranim::edit_body(world, n, |l| l.attached.push(added));
            }
            return Ok(Value::Undefined);
        }
=======
>>>>>>> /tmp/claude-0/zmhead/crates/sim/src/t6/natives_ent.rs
        let Ok(n) = me(vm, world, s) else {
            return Ok(Value::Undefined);
        };
        let model = text(vm, a, 0);
        // BO2 tags are case-blind (the Panzer attaches at "J_Helmet"); bones
        // are stored lowercase, so its hit boxes only build with this.
        let tag = text(vm, a, 1).to_ascii_lowercase();
        if let Some(mut zm) = ent_mut(world, n) {
            zm.ents.get_mut(&n).unwrap().attached.push((model, tag));
        }
        Ok(Value::Undefined)
    });
    m!("detach", |vm, world, s, a| {
<<<<<<< /home/claude/bo2-multiplayer-rebuild/crates/sim/src/t6/natives_ent.rs
        if is_player(vm, world, s) {
            if let Some(n) = entnum(vm, s) {
                let gone = text(vm, a, 0);
                super::playeranim::edit_body(world, n, |l| {
                    if let Some(i) = l.attached.iter().position(|(m, _)| *m == gone) {
                        l.attached.remove(i);
                    }
                });
            }
            return Ok(Value::Undefined);
        }
=======
>>>>>>> /tmp/claude-0/zmhead/crates/sim/src/t6/natives_ent.rs
        let Ok(n) = me(vm, world, s) else {
            return Ok(Value::Undefined);
        };
        let model = text(vm, a, 0);
        if let Some(mut zm) = ent_mut(world, n) {
            let e = zm.ents.get_mut(&n).unwrap();
            if let Some(i) = e.attached.iter().position(|(m, _)| *m == model) {
                e.attached.remove(i);
            }
        }
        Ok(Value::Undefined)
    });
    m!("detachall", |vm, world, s, _| {
<<<<<<< /home/claude/bo2-multiplayer-rebuild/crates/sim/src/t6/natives_ent.rs
        if is_player(vm, world, s) {
            if let Some(n) = entnum(vm, s) {
                super::playeranim::detach_all(world, n);
            }
            return Ok(Value::Undefined);
        }
=======
>>>>>>> /tmp/claude-0/zmhead/crates/sim/src/t6/natives_ent.rs
        let Ok(n) = me(vm, world, s) else {
            return Ok(Value::Undefined);
        };
        if let Some(mut zm) = ent_mut(world, n) {
            zm.ents.get_mut(&n).unwrap().attached.clear();
        }
        Ok(Value::Undefined)
    });
    macro_rules! flagset {
        ($($name:literal => |$e:ident| $set:expr),* $(,)?) => {$(
            vm.bind($name, true, |vm, world, s, _| {
                let Ok(n) = me(vm, world, s) else { return Ok(Value::Undefined) };
                if let Some(mut zm) = ent_mut(world, n) {
                    let $e = zm.ents.get_mut(&n).unwrap();
                    $set;
                }
                Ok(Value::Undefined)
            });
        )*};
    }
    flagset!(
        "hide" => |e| {
            e.hidden = true;
<<<<<<< /home/claude/bo2-multiplayer-rebuild/crates/sim/src/t6/natives_ent.rs
            e.shown_to = None;
        },
        "show" => |e| {
            e.hidden = false;
            e.shown_to = None;
        },
        "ghost" => |e| e.hidden = true,
=======
            e.ghosted = false;
        },
        "show" => |e| {
            e.hidden = false;
            e.ghosted = false;
        },
        "ghost" => |e| {
            e.hidden = true;
            e.ghosted = true;
        },
>>>>>>> /tmp/claude-0/zmhead/crates/sim/src/t6/natives_ent.rs
        "notsolid" => |e| e.solid = false,
        "solid" => |e| e.solid = true,
        "unlink" => |e| {
            e.link = None;
            e.link_tag = None;
        },
        "stoploopsound" => |e| e.loop_sound = None,
    );
    // Parts of its model hidden or shown again by tag (a buildable table's
    // parts appear as they are added); drawn by `presence`.
    fn part(vm: &mut Vm<World>, world: &mut World, s: &Value, a: &[Value], hide: bool) {
        let Ok(n) = me(vm, world, s) else { return };
        let tag = text(vm, a, 0).to_ascii_lowercase();
        if let Some(mut zm) = ent_mut(world, n) {
            let parts = &mut zm.ents.get_mut(&n).unwrap().hidden_parts;
            parts.retain(|p| *p != tag);
            if hide {
                parts.push(tag);
            }
        }
    }
    m!("hidepart", |vm, world, s, a| {
        part(vm, world, s, a, true);
        Ok(Value::Undefined)
    });
    m!("showpart", |vm, world, s, a| {
        part(vm, world, s, a, false);
        Ok(Value::Undefined)
    });
    // Origins' crafted staffs on their pedestals.
    m!("showallparts", |vm, world, s, _| {
        if let Ok(n) = me(vm, world, s)
            && let Some(mut zm) = ent_mut(world, n)
        {
            zm.ents.get_mut(&n).unwrap().hidden_parts.clear();
        }
        Ok(Value::Undefined)
    });
    // A weapon's world model on an entity (the box's floating gun, a wall
    // gun, Origins' staffs): the gun (or the model given), the parts BO2
    // hangs on it (magazine, scope, rocket) and its hide tags hidden, the
    // tags hidden before shown again.
    m!("useweaponmodel", |vm, world, s, a| {
        let Ok(n) = me(vm, world, s) else {
            return Ok(Value::Undefined);
        };
        let w = super::weapon(world, &text(vm, a, 0)).unwrap_or(0);
        let look = super::weapon_look(world, w).unwrap_or_default();
        let mut model = text(vm, a, 1);
        if model.is_empty() {
            model = look.model;
        }
        if let Some(mut zm) = ent_mut(world, n) {
            let e = zm.ents.get_mut(&n).unwrap();
            e.model = model;
            e.weapon_parts = look.parts;
            e.hidden_parts = look.hide;
        }
        Ok(Value::Undefined)
    });
    // The weapon's hide tags hidden on the model (the tags hidden before
    // shown again); the parts it wears stay (`useweaponmodel`'s).
    m!("useweaponhidetags", |vm, world, s, a| {
        let Ok(n) = me(vm, world, s) else {
            return Ok(Value::Undefined);
        };
        let w = super::weapon(world, &text(vm, a, 0)).unwrap_or(0);
        let hide = super::weapon_look(world, w).map(|l| l.hide).unwrap_or_default();
        if let Some(mut zm) = ent_mut(world, n) {
            zm.ents.get_mut(&n).unwrap().hidden_parts = hide;
        }
        Ok(Value::Undefined)
    });
    // A player unlinks too (`playerlinkto`).
    m!("unlink", |vm, world, s, _| {
        let Some(n) = entnum(vm, s) else {
            return Ok(Value::Undefined);
        };
        let mut zm = world.resource_mut::<Zm>();
        if let Some(p) = zm.players.get_mut(&n) {
            p.linked = None;
        } else if let Some(e) = zm.ents.get_mut(&n) {
            e.link = None;
            e.link_tag = None;
        }
        Ok(Value::Undefined)
    });
    m!("setcandamage", |vm, world, s, a| {
        let Ok(n) = me(vm, world, s) else {
            return Ok(Value::Undefined);
        };
        let on = flag(a, 0, true);
        if let Some(mut zm) = ent_mut(world, n) {
            zm.ents.get_mut(&n).unwrap().can_damage = on;
        }
        Ok(Value::Undefined)
    });
    m!("sethintstring", |vm, world, s, a| {
        let Ok(n) = me(vm, world, s) else {
            return Ok(Value::Undefined);
        };
        // The text as the player reads it: the language string with its
        // arguments filled in (`&&1`, `&&2`).
        let args = super::name_players(vm, world, a.get(1..).unwrap_or(&[]));
        let h = super::localize(vm, world, arg(a, 0), &args);
        if let Some(mut zm) = ent_mut(world, n) {
            zm.ents.get_mut(&n).unwrap().hint = (!h.is_empty()).then_some(h);
        }
        Ok(Value::Undefined)
    });
    // sethintlowpriority(on): a low priority trigger's hint shows only when
    // no other usable trigger is there (`triggers::dispatch`).
    m!("sethintlowpriority", |vm, world, s, a| {
        let Ok(n) = me(vm, world, s) else {
            return Ok(Value::Undefined);
        };
        let on = !matches!(arg(a, 0), Value::Int(0) | Value::Undefined);
        if let Some(mut zm) = ent_mut(world, n) {
            zm.ents.get_mut(&n).unwrap().hint_low = on;
        }
        Ok(Value::Undefined)
    });
    m!("setcursorhint", |vm, world, s, a| {
        let Ok(n) = me(vm, world, s) else {
            return Ok(Value::Undefined);
        };
        let h = text(vm, a, 0);
        if let Some(mut zm) = ent_mut(world, n) {
            zm.ents.get_mut(&n).unwrap().cursor_hint = Some(h);
        }
        Ok(Value::Undefined)
    });
    m!("playloopsound", |vm, world, s, a| {
<<<<<<< /home/claude/bo2-multiplayer-rebuild/crates/sim/src/t6/natives_ent.rs
        if std::env::var_os("BO2MP_SNDLOG").is_some() {
            let alias = text(vm, a, 0);
            let on = me(vm, world, s);
            diag::info!(Sim, "bo2mp sndlog loop {alias} on {on:?}");
        }
=======
>>>>>>> /tmp/claude-0/zmhead/crates/sim/src/t6/natives_ent.rs
        let Ok(n) = me(vm, world, s) else {
            return Ok(Value::Undefined);
        };
        let alias = text(vm, a, 0);
        if let Ok(want) = std::env::var("IW4L_T6_SNDLOG")
            && (want == "1" || want.split(',').any(|w| alias.contains(w)))
        {
            let at = world.resource::<Zm>().ents.get(&n).map(|e| e.origin);
            diag::info!(Sim, "bo2zm t6 loop sound {alias} at {at:?}");
        }
        if let Some(mut zm) = ent_mut(world, n) {
            zm.ents.get_mut(&n).unwrap().loop_sound = Some(alias);
        }
        Ok(Value::Undefined)
    });
    m!("linkto", |vm, world, s, a| {
        let Ok(n) = me(vm, world, s) else {
            return Ok(Value::Undefined);
        };
        let Some(parent) = entnum(vm, arg(a, 0)) else {
            return Err("linkto needs an entity".into());
        };
        let off = arg(a, 2).as_vec3();
        let aoff = arg(a, 3).as_vec3().unwrap_or([0.0; 3]);
        // On a tag of the parent's model (where the posed skeleton has it
        // this tick), else on the parent itself.
        let tag = match arg(a, 1) {
            Value::Undefined => None,
            v => Some(vm.to_text(v)).filter(|t| !t.is_empty()),
        };
        let tag_pose = tag
            .as_deref()
            .and_then(|t| super::tags::tag_world(world, parent, t))
            .map(|(o, axes)| (o, super::tags::axis_to_angles(axes)));
        let parent_pose = tag_pose.or_else(|| {
            let p = arg(a, 0).clone();
            let o = origin_of(vm, world, &p);
            let ang = world
                .resource::<Zm>()
                .ents
                .get(&parent)
                .map(|e| e.angles)
                .unwrap_or([0.0; 3]);
            o.map(|o| (o, ang))
        });
        let (my, my_angles) = world
            .resource::<Zm>()
            .ents
            .get(&n)
            .map(|e| (e.origin, e.angles))
            .unwrap_or(([0.0; 3], [0.0; 3]));
        // No offsets given: BO2 keeps its turn relative to the parent too
        // (a Die Rise perk machine stays facing out of its car's door).
        let aoff = match (&off, parent_pose) {
            (None, Some((_, pa))) => [0, 1, 2].map(|i| my_angles[i] - pa[i]),
            _ => aoff,
        };
        let off = off.unwrap_or_else(|| match parent_pose {
            // Keep where it is relative to the parent.
            Some((po, pa)) => {
                let (f, r, u) = gsc_t6::math::angle_vectors(pa);
                let d = gsc_t6::math::sub(my, po);
                [
                    gsc_t6::math::dot(d, f),
                    -gsc_t6::math::dot(d, r),
                    gsc_t6::math::dot(d, u),
                ]
            }
            None => [0.0; 3],
        });
        if let Some(mut zm) = ent_mut(world, n) {
            let e = zm.ents.get_mut(&n).unwrap();
            e.link = Some((parent, off, aoff));
            e.link_tag = tag.filter(|_| tag_pose.is_some());
        }
        Ok(Value::Undefined)
    });
    m!("enablelinkto", |_, _, _, _| Ok(Value::Undefined));
    // Players standing on it ride it (`riders`: Tranzit's bus).
    m!("setmovingplatformenabled", |vm, world, s, a| {
        let Ok(n) = me(vm, world, s) else {
            return Ok(Value::Undefined);
        };
        let on = arg(a, 0).as_float().is_none_or(|v| v != 0.0);
        if let Some(mut zm) = ent_mut(world, n) {
            zm.ents.get_mut(&n).unwrap().platform = on;
        }
        Ok(Value::Undefined)
    });
    m!("getorigin", |vm, world, s, _| Ok(
        origin_of(vm, world, s).map_or(Value::Undefined, Value::Vec3)
    ));
    m!("getentitynumber", |vm, _, s, _| Ok(
        entnum(vm, s).map_or(Value::Undefined, |n| Value::Int(n as i32))
    ));
    // The engine's entity type, as the scripts' bookmarks read it: a
    // player 1, a thrown grenade (a missile) 4, anything else 0.
    m!("getentitytype", |vm, world, s, _| {
        let n = entnum(vm, s);
        let zm = world.resource::<Zm>();
        let player = zm.players.values().any(|p| Some(p.obj) == s.as_obj());
        let missile = n.and_then(|n| zm.ents.get(&n)).is_some_and(|e| e.classname == "grenade");
        Ok(Value::Int(if player { 1 } else if missile { 4 } else { 0 }))
    });
    // Where the tag is on its posed model (`tags`); the entity's own
    // origin and angles for a tag its model lacks.
    m!("gettagorigin", |vm, world, s, a| {
        let tag = vm.to_text(arg(a, 0));
        if let Some(n) = entnum(vm, s)
            && let Some((o, _)) = super::tags::tag_world(world, n, &tag)
        {
            return Ok(Value::Vec3(o));
        }
        Ok(origin_of(vm, world, s).map_or(Value::Undefined, Value::Vec3))
    });
    m!("gettagangles", |vm, world, s, a| {
        let n = entnum(vm, s).unwrap_or(u32::MAX);
        let tag = vm.to_text(arg(a, 0));
        if let Some((_, axes)) = super::tags::tag_world(world, n, &tag) {
            return Ok(Value::Vec3(super::tags::axis_to_angles(axes)));
        }
        Ok(world
            .resource::<Zm>()
            .ents
            .get(&n)
            .map_or(Value::Vec3([0.0; 3]), |e| Value::Vec3(e.angles)))
    });
    // An entity's bounds: its model's (the file's own bounds), a player's
    // or an actor's body; abs = turned by its angles and placed.
    m!("getmins", |vm, world, s, _| Ok(Value::Vec3(
        local_bounds(vm, world, s).0
    )));
    m!("getmaxs", |vm, world, s, _| Ok(Value::Vec3(
        local_bounds(vm, world, s).1
    )));
    m!("getabsmins", |vm, world, s, _| Ok(Value::Vec3(
        abs_bounds(vm, world, s).0
    )));
    m!("getabsmaxs", |vm, world, s, _| {
        let (lo, hi) = abs_bounds(vm, world, s);
        // IW4L_T6_BOUNDSLOG=1: each asked entity's model, angles and bounds.
        if std::env::var_os("IW4L_T6_BOUNDSLOG").is_some() {
            let (llo, lhi) = local_bounds(vm, world, s);
            let e = entnum(vm, s).and_then(|n| world.resource::<Zm>().ents.get(&n).cloned());
            if let Some(e) = e {
                diag::info!(
                    Sim,
                    "bo2zm t6 bounds {} at {:?} angles {:?}: local {llo:?}..{lhi:?} abs size {:?}",
                    e.model,
                    e.origin,
                    e.angles,
                    [hi[0] - lo[0], hi[1] - lo[1], hi[2] - lo[2]]
                );
            }
        }
        Ok(Value::Vec3(hi))
    });
    m!("moveto", |vm, world, s, a| {
        let n = me(vm, world, s)?;
        let to = vec3(a, 0)?;
        let t = num(a, 1)?;
        let (acc, dec) = (
            arg(a, 2).as_float().unwrap_or(0.0),
            arg(a, 3).as_float().unwrap_or(0.0),
        );
        let mut zm = world.resource_mut::<Zm>();
        let now = zm.now_ms;
        let from = zm.ents[&n].origin;
        zm.movers.move_to(n, from, to, now, t, acc, dec);
        Ok(Value::Undefined)
    });
    m!("movex", |vm, world, s, a| move_axis(vm, world, s, a, 0));
    m!("movey", |vm, world, s, a| move_axis(vm, world, s, a, 1));
    m!("movez", |vm, world, s, a| move_axis(vm, world, s, a, 2));
    m!("movegravity", |vm, world, s, a| {
        let n = me(vm, world, s)?;
        let vel = vec3(a, 0)?;
        let t = num(a, 1)?;
        let mut zm = world.resource_mut::<Zm>();
        let now = zm.now_ms;
        let from = zm.ents[&n].origin;
        zm.movers.gravity(n, from, vel, now, t);
        Ok(Value::Undefined)
    });
    m!("rotateto", |vm, world, s, a| {
        let n = me(vm, world, s)?;
        let to = vec3(a, 0)?;
        let t = num(a, 1)?;
        let (acc, dec) = (
            arg(a, 2).as_float().unwrap_or(0.0),
            arg(a, 3).as_float().unwrap_or(0.0),
        );
        let mut zm = world.resource_mut::<Zm>();
        let now = zm.now_ms;
        let from = zm.ents[&n].angles;
        // Shortest way round, as the engine turns.
        let mut target = to;
        for i in 0..3 {
            let d = gsc_t6::math::angle_clamp180(to[i] - from[i]);
            target[i] = from[i] + d;
        }
        zm.movers.rotate_to(n, from, target, now, t, acc, dec);
        Ok(Value::Undefined)
    });
    // vibrate(dir, degrees, period, secs): rocks about its origin toward
    // `dir`, fading out (a landed perk machine).
    m!("vibrate", |vm, world, s, a| {
        let n = me(vm, world, s)?;
        let dir = vec3(a, 0)?;
        let (amp, period, secs) = (num(a, 1)?, num(a, 2)?, num(a, 3)?);
        let mut zm = world.resource_mut::<Zm>();
        let now = zm.now_ms;
        let base = zm.ents[&n].angles;
        zm.movers.vibrate(n, base, dir, amp, period, now, secs);
        Ok(Value::Undefined)
    });
    m!("rotateyaw", |vm, world, s, a| rotate_axis(
        vm, world, s, a, 1
    ));
    m!("rotatepitch", |vm, world, s, a| rotate_axis(
        vm, world, s, a, 0
    ));
    m!("rotateroll", |vm, world, s, a| rotate_axis(
        vm, world, s, a, 2
    ));
<<<<<<< /home/claude/bo2-multiplayer-rebuild/crates/sim/src/t6/natives_ent.rs
    // `a istouching(b)`: whichever of the two has a volume (a brush or a
    // trigger cylinder) against the other's box (a player's, an actor's,
=======
    // `a istouching(b)`: whichever of the two has a volume (a brush, a
    // trigger cylinder or a spawned trigger box) against the other's box (a player's, an actor's,
>>>>>>> /tmp/claude-0/zmhead/crates/sim/src/t6/natives_ent.rs
    // else a point).
    m!("istouching", |vm, world, s, a| {
        let other = arg(a, 0).clone();
        let (Some(sn), Some(on)) = (entnum(vm, s), entnum(vm, &other)) else {
            return Ok(Value::Int(0));
        };
        let volume_of = |world: &World, n: u32| -> Option<Ent> {
            world
                .resource::<Zm>()
                .ents
                .get(&n)
<<<<<<< /home/claude/bo2-multiplayer-rebuild/crates/sim/src/t6/natives_ent.rs
                .filter(|e| super::triggers::brush_index(e).is_some() || e.radius > 0.0)
=======
                .filter(|e| {
                    super::triggers::brush_index(e).is_some()
                        || e.radius > 0.0
                        || e.box_dims.is_some()
                })
>>>>>>> /tmp/claude-0/zmhead/crates/sim/src/t6/natives_ent.rs
                .cloned()
        };
        let (vol, toucher) = match (volume_of(world, on), volume_of(world, sn)) {
            (Some(v), _) => (v, sn),
            (None, Some(v)) => (v, on),
            (None, None) => return Ok(Value::Int(0)),
        };
        // A script_model is as big as its model: BO2's setmodel sets its
        // bounds from the model and istouching places them at its origin,
        // unturned (a turbine set down in the tower counts for Maxis even
        // though its origin is on the floor, below the tower's volume).
        let model = world
            .resource::<Zm>()
            .ents
            .get(&toucher)
            .filter(|o| o.classname == "script_model" && !o.model.is_empty() && !o.model.starts_with('*'))
            .map(|o| (o.origin, o.model.clone()));
        let model_box = model.and_then(|(p, m)| {
            let (mid, half) = super::frame(world).model_capability(&m).flatten()?.bounds?;
            Some((
                std::array::from_fn(|i| p[i] + mid[i] - half[i]),
                std::array::from_fn(|i| p[i] + mid[i] + half[i]),
            ))
        });
        let bounds = if world.resource::<Zm>().players.contains_key(&toucher) {
            super::triggers::player_box(world, toucher)
        } else if model_box.is_some() {
            model_box
        } else {
            let zm = world.resource::<Zm>();
            zm.ents.get(&toucher).map(|o| {
                let p = o.origin;
                match zm.actors.get(&toucher) {
                    Some(act) => (
                        [p[0] - act.radius, p[1] - act.radius, p[2]],
                        [p[0] + act.radius, p[1] + act.radius, p[2] + act.height],
                    ),
                    None => (
                        [p[0] - 1.0, p[1] - 1.0, p[2] - 1.0],
                        [p[0] + 1.0, p[1] + 1.0, p[2] + 1.0],
                    ),
                }
            })
        };
        let Some((mut mins, mut maxs)) = bounds else {
            return Ok(Value::Int(0));
        };
        // istouching(b, pad): the box grows by `pad` on every side.
        if a.len() > 1 {
            let pad = vec3(a, 1)?;
            for i in 0..3 {
                mins[i] -= pad[i];
                maxs[i] += pad[i];
            }
        }
        let hit = super::triggers::touches(world, &vol, mins, maxs);
        // IW4L_T6_TOUCHLOG=<targetname>[,...]: each test against a volume
        // with that name, its brush bounds and the answer (Die Rise refuses
        // Who's Who's body inside an elevator volume).
        if let Ok(names) = std::env::var("IW4L_T6_TOUCHLOG") {
            let tn = vm.intern("targetname");
            let name = vol.obj.map(|o| vm.to_text(&vm.raw_field(o, tn))).unwrap_or_default();
            if names.split(',').any(|n| n == name) {
                let bb = super::triggers::brush_bounds(world, &vol);
                diag::info!(
                    Sim,
                    "bo2zm t6 touch {name} ({} {:?}) at {:?} bounds {bb:?} vs {mins:?}..{maxs:?}: {hit}",
                    vol.classname,
                    vol.brush,
                    vol.origin
                );
            }
        }
        Ok(Value::bool(hit))
    });
    m!("setinvisibletoplayer", |vm, world, s, a| {
        let Ok(n) = me(vm, world, s) else {
            return Ok(Value::Undefined);
        };
        let Some(p) = entnum(vm, arg(a, 0)) else {
            return Ok(Value::Undefined);
        };
        let hide = flag(a, 1, true);
        if let Some(mut zm) = ent_mut(world, n) {
            let e = zm.ents.get_mut(&n).unwrap();
            if hide {
                e.invisible_to.insert(p);
            } else {
                shown_to_one(e, p);
            }
        }
        Ok(Value::Undefined)
    });
    m!("showtoplayer", |vm, world, s, a| {
        let Ok(n) = me(vm, world, s) else {
            return Ok(Value::Undefined);
        };
        let Some(p) = entnum(vm, arg(a, 0)) else {
            return Ok(Value::Undefined);
        };
        if let Some(mut zm) = ent_mut(world, n) {
            let e = zm.ents.get_mut(&n).unwrap();
            e.hidden = false;
            e.shown_to
                .get_or_insert_with(std::collections::BTreeSet::new)
                .insert(p);
        }
        Ok(Value::Undefined)
    });
    m!("setvisibletoplayer", |vm, world, s, a| {
        let Ok(n) = me(vm, world, s) else {
            return Ok(Value::Undefined);
        };
        let Some(p) = entnum(vm, arg(a, 0)) else {
            return Ok(Value::Undefined);
        };
        if let Some(mut zm) = ent_mut(world, n) {
            shown_to_one(zm.ents.get_mut(&n).unwrap(), p);
        }
        Ok(Value::Undefined)
    });
    m!("setinvisibletoall", |vm, world, s, _| {
        let Ok(n) = me(vm, world, s) else {
            return Ok(Value::Undefined);
        };
        if let Some(mut zm) = ent_mut(world, n) {
            zm.ents.get_mut(&n).unwrap().invisible_to_all = true;
        }
        Ok(Value::Undefined)
    });
    m!("setvisibletoall", |vm, world, s, _| {
        let Ok(n) = me(vm, world, s) else {
            return Ok(Value::Undefined);
        };
        if let Some(mut zm) = ent_mut(world, n) {
            let e = zm.ents.get_mut(&n).unwrap();
            e.invisible_to_all = false;
            e.invisible_to.clear();
        }
        Ok(Value::Undefined)
    });
    m!("triggerenable", |vm, world, s, a| {
        let Ok(n) = me(vm, world, s) else {
            return Ok(Value::Undefined);
        };
        let on = flag(a, 0, true);
        if let Some(mut zm) = ent_mut(world, n) {
            zm.ents.get_mut(&n).unwrap().trigger_off = !on;
        }
        Ok(Value::Undefined)
    });
    // BO2 (GScr_SetTeamForTrigger, GScr_SetExcludeTeamForTrigger): the team
    // a radius, box or use trigger answers, or the one it ignores (a
    // claymore's trigger ignores its owner's side).
    m!("setteamfortrigger", |vm, world, s, a| {
        let Ok(n) = me(vm, world, s) else {
            return Ok(Value::Undefined);
        };
        let team = vm.to_text(arg(a, 0));
        if let Some(mut zm) = ent_mut(world, n) {
            zm.ents.get_mut(&n).unwrap().trigger_team = Some((team, false));
        }
        Ok(Value::Undefined)
    });
    m!("setexcludeteamfortrigger", |vm, world, s, a| {
        let Ok(n) = me(vm, world, s) else {
            return Ok(Value::Undefined);
        };
        let team = vm.to_text(arg(a, 0));
        if let Some(mut zm) = ent_mut(world, n) {
            zm.ents.get_mut(&n).unwrap().trigger_team = Some((team, true));
        }
        Ok(Value::Undefined)
    });
    // A wall buy's trigger is the weapon's own small box on the wall: the
    // player uses it by looking at it from close by, not by standing in
    // it (seats and clip keep him a body away from the truck's chalk).
    m!("usetriggerrequirelookat", |vm, world, s, _| {
        let Ok(n) = me(vm, world, s) else {
            return Ok(Value::Undefined);
        };
        if let Some(mut zm) = ent_mut(world, n) {
            zm.ents.get_mut(&n).unwrap().look_at = true;
        }
        Ok(Value::Undefined)
    });
    for name in [
        "triggerignoreteam",
        "setvisibletoallexceptteam",
<<<<<<< /home/claude/bo2-multiplayer-rebuild/crates/sim/src/t6/natives_ent.rs
        "setexcludeteamfortrigger",
        "setmovingplatformenabled",
=======
>>>>>>> /tmp/claude-0/zmhead/crates/sim/src/t6/natives_ent.rs
        "setowner",
        "setteam",
        "setphysparams",
        "setturretcarried",
        "makegrenadedud",
        "setclientfield",
        "setclientfieldtoplayer",
        "stopsounds",
        "playsound",
        "playsoundtoplayer",
        "playsoundtoteam",
        "playsoundontag",
        "playsoundasmaster",
        "playlocalsound",
        "useanimtree",
        "setanim",
        "clearanim",
        "notsolidcapsule",
        "dontinterpolate",
        "setscale",
        "setzbarrierpiecestate",
        "showzbarrierpiece",
        "hidezbarrierpiece",
        "zbarrierpieceuseboxriselogic",
        "zbarrierpieceusedefaultmodel",
        "zbarrierpieceuseupgradedmodel",
        // client draw hints (Origins' giant robots, the crystal biplane)
        "setforcenocull",
        "setlightintensity",
        "setplayercollision",
    ] {
        vm.bind(name, true, |_, _, _, _| Ok(Value::Undefined));
    }
    // A limp body (a ragdoll): every zombie death goes limp part way through
    // its death anim (zm_death's `waitforragdoll`), the Trample Steam throws
    // its kills (`launchragdoll`). Here it is a mark on the body and its last
    // throw; each client drops the body from the pose it had, under gravity
    // (render_anim's ragdoll). As in Black Ops II, a living body goes limp
    // too (the traps throw theirs, then kill them), and a throw only moves a
    // body that is limp already.
    m!("startragdoll", |vm, world, s, _| {
        if let Some(n) = entnum(vm, s) {
            go_limp(world, n);
        }
        Ok(Value::Undefined)
    });
    m!("launchragdoll", |vm, world, s, a| {
        let throw = vec3(a, 0)?;
        if let Some(n) = entnum(vm, s)
            && let Some(ragdoll) = world
                .resource_mut::<Zm>()
                .into_inner()
                .ents
                .get_mut(&n)
                .and_then(|e| e.ragdoll.as_mut())
        {
            ragdoll.launch = throw;
            ragdoll.launches = ragdoll.launches.wrapping_add(1);
        }
        Ok(Value::Undefined)
    });
    m!("isragdoll", |vm, world, s, _| {
        let limp = entnum(vm, s)
            .and_then(|n| world.resource::<Zm>().ents.get(&n))
            .is_some_and(|e| e.ragdoll.is_some());
        Ok(Value::Int(i32::from(limp)))
    });
    // A script model's animation (the Monkey Bomb's clap): actors play
    // theirs through their state machine, not this.
    m!("setanim", |vm, world, s, a| {
        let Ok(n) = me(vm, world, s) else {
            return Ok(Value::Undefined);
        };
        let Value::Anim(i) = arg(a, 0) else {
            return Ok(Value::Undefined);
        };
        let Some(&(_, name)) = vm.program.anims.get(*i as usize) else {
            return Ok(Value::Undefined);
        };
        let clip = vm.strings.get(name).to_ascii_lowercase();
        let mut zm = world.resource_mut::<Zm>();
        let now = zm.now_ms;
        if let Some(e) = zm.ents.get_mut(&n)
            && e.anim.as_ref().is_none_or(|(c, _)| *c != clip)
        {
            e.anim = Some((clip, now));
        }
        Ok(Value::Undefined)
    });
    m!("clearanim", |vm, world, s, _| {
        if let Ok(n) = me(vm, world, s)
            && let Some(e) = world.resource_mut::<Zm>().ents.get_mut(&n)
        {
            e.anim = None;
        }
        Ok(Value::Undefined)
    });
    m!("resetmissiledetonationtime", |vm, world, s, a| {
        if let Ok(n) = me(vm, world, s) {
            let secs = arg(a, 0).as_float();
            let now = world.resource::<Zm>().now_ms;
            super::grenades::reset_fuse(world, n, secs, now);
        }
        Ok(Value::Undefined)
    });
    // A planted claymore a zombie walked into (or one planted where it
    // cannot stay): it goes off now.
    m!("detonate", |vm, world, s, _| {
        if let Ok(n) = me(vm, world, s) {
            let now = world.resource::<Zm>().now_ms;
            super::grenades::detonate_now(world, n, now);
        }
        Ok(Value::Undefined)
    });
    m!("playsoundwithnotify", |vm, world, s, a| {
        // The sound system is not wired to scripts yet: notify after a short
        // while so waiting scripts go on.
        let name = text(vm, a, 1);
        if let Some(o) = s.as_obj() {
            super::natives_game::notify_later(vm, world, o, &name, 500);
        }
        Ok(Value::Undefined)
    });
}

/// BO2's `setvisibletoplayer` clears this player's bit only
/// (ScrCmd_SetVisibleToPlayer): after `setinvisibletoall` (every bit set) the
/// others still do not see it (Mob's ghost-only props, one ghost at a time).
fn shown_to_one(e: &mut Ent, p: u32) {
    if e.invisible_to_all {
        e.invisible_to_all = false;
        e.invisible_to.extend(0..32);
    }
    e.invisible_to.remove(&p);
}

fn move_axis(vm: &mut Vm<World>, world: &mut World, s: &Value, a: &[Value], axis: usize) -> R {
    let n = me(vm, world, s)?;
    let d = num(a, 0)?;
    let t = num(a, 1)?;
    let (acc, dec) = (
        arg(a, 2).as_float().unwrap_or(0.0),
        arg(a, 3).as_float().unwrap_or(0.0),
    );
    let mut zm = world.resource_mut::<Zm>();
    let now = zm.now_ms;
    let from = zm.ents[&n].origin;
    let mut to = from;
    to[axis] += d;
    zm.movers.move_to(n, from, to, now, t, acc, dec);
    Ok(Value::Undefined)
}

fn rotate_axis(vm: &mut Vm<World>, world: &mut World, s: &Value, a: &[Value], axis: usize) -> R {
    let n = me(vm, world, s)?;
    let d = num(a, 0)?;
    let t = num(a, 1)?;
    let (acc, dec) = (
        arg(a, 2).as_float().unwrap_or(0.0),
        arg(a, 3).as_float().unwrap_or(0.0),
    );
    let mut zm = world.resource_mut::<Zm>();
    let now = zm.now_ms;
    let from = zm.ents[&n].angles;
    let mut to = from;
    to[axis] += d;
    zm.movers.rotate_to(n, from, to, now, t, acc, dec);
    Ok(Value::Undefined)
}

/// Model-space bounds of an entity (mins, maxs).
fn local_bounds(vm: &Vm<World>, world: &mut World, s: &Value) -> ([f32; 3], [f32; 3]) {
    const BODY: ([f32; 3], [f32; 3]) = ([-15.0, -15.0, 0.0], [15.0, 15.0, 72.0]);
    let Some(n) = entnum(vm, s) else { return BODY };
    if is_player(vm, world, s) || world.resource::<Zm>().actors.contains_key(&n) {
        return BODY;
    }
    let Some(e) = world.resource::<Zm>().ents.get(&n).cloned() else {
        return ([0.0; 3], [0.0; 3]);
    };
    if let Some([w, l, h]) = e.box_dims {
        return ([-w * 0.5, -l * 0.5, -h * 0.5], [w * 0.5, l * 0.5, h * 0.5]);
    }
    if e.radius > 0.0 {
        return ([-e.radius, -e.radius, 0.0], [e.radius, e.radius, e.height]);
    }
    if e.model.is_empty() || e.model.starts_with('*') {
        return ([0.0; 3], [0.0; 3]);
    }
    let f = super::frame(world);
    match f
        .model_capability(&e.model)
        .flatten()
        .and_then(|c| c.bounds)
    {
        Some((mid, half)) => (
            std::array::from_fn(|i| mid[i] - half[i]),
            std::array::from_fn(|i| mid[i] + half[i]),
        ),
        None => ([0.0; 3], [0.0; 3]),
    }
}

/// World bounds, the way BO2 links an entity: a player or zombie is its box
/// placed (never turned); a brush model is its box turned, placed; anything
/// else (a script model, a trigger, a vehicle) is a cube as wide as its box's
/// radius every way, whatever way it faces. All grow by 1. The wall buys
/// size their use box from a gun model's cube (`getabsmins`), so turning the
/// gun's own box instead gave a chalk on a wall running along x a box 3
/// units thick: a tester could not buy the Bus Depot shotgun unless he stood
/// against it.
fn abs_bounds(vm: &Vm<World>, world: &mut World, s: &Value) -> ([f32; 3], [f32; 3]) {
    let (lo, hi) = local_bounds(vm, world, s);
    let origin = origin_of(vm, world, s).unwrap_or([0.0; 3]);
    let ent = entnum(vm, s).and_then(|n| {
        let zm = world.resource::<Zm>();
        let sentient = zm.players.contains_key(&n) || zm.actors.contains_key(&n);
        Some((sentient, zm.ents.get(&n)?.clone()))
    });
    let brush = ent.as_ref().is_some_and(|(_, e)| e.model.starts_with('*'));
    if ent.as_ref().is_none_or(|(sentient, _)| *sentient) {
        return (
            std::array::from_fn(|i| origin[i] + lo[i] - 1.0),
            std::array::from_fn(|i| origin[i] + hi[i] + 1.0),
        );
    }
    if !brush {
        let r = (0..3)
            .map(|i| lo[i].abs().max(hi[i].abs()).powi(2))
            .sum::<f32>()
            .sqrt()
            + 1.0;
        return (origin.map(|o| o - r), origin.map(|o| o + r));
    }
    let angles = ent.map_or([0.0; 3], |(_, e)| e.angles);
    let (f, r, u) = gsc_t6::math::angle_vectors(angles);
    let mut mins = [f32::MAX; 3];
    let mut maxs = [f32::MIN; 3];
    for c in 0..8 {
        let p = [
            if c & 1 == 0 { lo[0] } else { hi[0] },
            if c & 2 == 0 { lo[1] } else { hi[1] },
            if c & 4 == 0 { lo[2] } else { hi[2] },
        ];
        for i in 0..3 {
            // local y is left (the right vector's opposite).
            let w = origin[i] + f[i] * p[0] - r[i] * p[1] + u[i] * p[2];
            mins[i] = mins[i].min(w);
            maxs[i] = maxs[i].max(w);
        }
    }
    (mins.map(|v| v - 1.0), maxs.map(|v| v + 1.0))
}

/// Mark entity `n` limp (`startragdoll`), once: its fall starts now. `None`
/// for a living actor or a gone entity.
fn go_limp(world: &mut World, n: u32) {
    let zm = world.resource_mut::<Zm>().into_inner();
    let start_ms = zm.now_ms as i32;
    if let Some(ent) = zm.ents.get_mut(&n) {
        ent.ragdoll.get_or_insert(xmodel_runtime::DObjRagdoll {
            start_ms,
            launch: [0.0; 3],
            launches: 0,
        });
    }
}
