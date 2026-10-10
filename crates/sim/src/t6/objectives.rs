//! bo2mp: BO2's objectives (`objective_add` and the rest): the flags, bomb
//! sites, capture zones and carried things its HUD marks in the world and
//! on the compass. The engine keeps 32 (`level.objectives`, reset at each
//! level start); each client is sent the ones it may see
//! (`G_UpdateObjectivesForClient`), and BO2's own HUD scripts draw them.

use bevy_ecs::prelude::World;
use gsc_t6::{Value, Vm};

use super::{Zm, arg, entnum, int, num, text, vec3};

pub(crate) const MAX: usize = 32;

/// One objective (BO2's `objective_t`).
#[derive(Clone, Default)]
pub(crate) struct Objective {
    /// 0 empty, 1 active, 2 invisible.
    pub state: u8,
    pub origin: [f32; 3],
    /// The entity it follows (`objective_onentity`).
    pub entity: Option<u32>,
    pub size: [f32; 2],
    /// Its icon's material (`objective_icon`).
    pub icon: String,
    pub owner: Option<u32>,
    /// Its name (`&"dom_a"`; empty = none): a named objective goes to
    /// every client and the HUD picks who sees it.
    pub name: String,
    /// The teams whose HUD shows it (`objective_visibleteams`): bit
    /// `1 << team`.
    pub team_mask: i32,
    /// 0-255 (`objective_setprogress` x 255).
    pub progress: u8,
    /// The clients using it (`objective_setplayerusing`), a bit each.
    pub using: u64,
    /// The clients it is hidden from (`objective_setinvisibletoplayer`).
    pub hidden: u64,
    pub gamemode_flags: u8,
    /// Its team (`objective_team`): 0 none, 1 allies, 2 axis, 3-8, 9
    /// spectator.
    pub team: u8,
}

/// BO2's team number for a team name (`Scr_GetTeam`).
pub(crate) fn team_number(name: &str) -> Result<u8, String> {
    Ok(match name {
        "allies" => 1,
        "axis" => 2,
        "team3" => 3,
        "team4" => 4,
        "team5" => 5,
        "team6" => 6,
        "team7" => 7,
        "team8" => 8,
        "spectator" => 9,
        "neutral" | "none" | "free" => 0,
        _ => {
            return Err(format!(
                "'{name}' is an illegal team string. Must be allies, axis, team3, team4, team5, team6, team7, team8, none, or spectator."
            ));
        }
    })
}

fn index(a: &[Value]) -> Result<usize, String> {
    let i = int(a, 0)?;
    usize::try_from(i)
        .ok()
        .filter(|i| *i < MAX)
        .ok_or_else(|| format!("index {i} is an illegal objective index. Valid indexes are 0 to {}", MAX - 1))
}

fn state(vm: &Vm<World>, a: &[Value], i: usize) -> Result<u8, String> {
    match text(vm, a, i).as_str() {
        "empty" => Ok(0),
        "active" => Ok(1),
        "invisible" => Ok(2),
        s => Err(format!(
            "Illegal objective state \"{s}\". Valid states are \"empty\", \"invisible\", \"current\", \"active\""
        )),
    }
}

/// A client's bit in a use or hidden mask.
fn bit(vm: &Vm<World>, world: &World, v: &Value) -> Result<u64, String> {
    let c = super::client(vm, world, v).map_err(|_| "SetVisibleToPlayer called on non player entity".to_owned())?;
    Ok(1u64.checked_shl(c.0).unwrap_or(0))
}

fn obj(world: &mut World, i: usize) -> impl std::ops::DerefMut<Target = Objective> + '_ {
    bevy_ecs::prelude::Mut::map_unchanged(world.resource_mut::<Zm>(), move |zm| &mut zm.objectives[i])
}

/// Its origin rounded as the engine rounds it (toward zero).
fn rounded(v: [f32; 3]) -> [f32; 3] {
    v.map(f32::trunc)
}

pub(super) fn bind(vm: &mut Vm<World>) {
    macro_rules! f {
        ($name:literal, $body:expr) => {
            vm.bind($name, false, $body);
        };
    }
    f!("objective_add", |vm, world, _, a| {
        let i = index(a)?;
        let st = state(vm, a, 1)?;
        let origin = if a.len() > 2 { Some(rounded(vec3(a, 2)?)) } else { None };
        let name = (a.len() > 3).then(|| text(vm, a, 3));
        let owner = (a.len() > 4).then(|| entnum(vm, arg(a, 4))).flatten();
        let mut o = obj(world, i);
        o.hidden = 0;
        o.entity = None;
        o.state = st;
        if let Some(origin) = origin {
            o.origin = origin;
            if let Some(name) = name {
                o.name = name;
            }
            if a.len() > 4 {
                o.owner = owner;
            }
        }
        o.team = 0;
        Ok(Value::Undefined)
    });
    f!("objective_state", |vm, world, _, a| {
        let i = index(a)?;
        let st = state(vm, a, 1)?;
        let mut o = obj(world, i);
        o.state = st;
        if st != 1 {
            o.entity = None;
        }
        Ok(Value::Undefined)
    });
    f!("objective_delete", |_, world, _, a| {
        let i = index(a)?;
        let mut o = obj(world, i);
        o.entity = None;
        o.state = 0;
        Ok(Value::Undefined)
    });
    f!("objective_icon", |vm, world, _, a| {
        let i = index(a)?;
        let icon = text(vm, a, 1);
        if icon.len() > 63 {
            return Err(format!("Objective icon name is too long (> 63): {icon}"));
        }
        obj(world, i).icon = icon;
        Ok(Value::Undefined)
    });
    f!("objective_position", |_, world, _, a| {
        let i = index(a)?;
        let v = rounded(vec3(a, 1)?);
        obj(world, i).origin = v;
        Ok(Value::Undefined)
    });
    f!("objective_onentity", |vm, world, _, a| {
        let i = index(a)?;
        let n = entnum(vm, arg(a, 1)).ok_or("argument 2 is not an entity")?;
        obj(world, i).entity = Some(n);
        Ok(Value::Undefined)
    });
    f!("objective_clearentity", |_, world, _, a| {
        let i = index(a)?;
        obj(world, i).entity = None;
        Ok(Value::Undefined)
    });
    f!("objective_team", |vm, world, _, a| {
        let i = index(a)?;
        let t = team_number(&text(vm, a, 1))?;
        obj(world, i).team = t;
        Ok(Value::Undefined)
    });
    f!("objective_visibleteams", |_, world, _, a| {
        let i = index(a)?;
        let m = int(a, 1)?;
        obj(world, i).team_mask = i32::from(m as i16);
        Ok(Value::Undefined)
    });
    f!("objective_setvisibletoplayer", |vm, world, _, a| {
        let i = index(a)?;
        let b = bit(vm, world, arg(a, 1))?;
        obj(world, i).hidden &= !b;
        Ok(Value::Undefined)
    });
    f!("objective_setinvisibletoplayer", |vm, world, _, a| {
        let i = index(a)?;
        let b = bit(vm, world, arg(a, 1))?;
        obj(world, i).hidden |= b;
        Ok(Value::Undefined)
    });
    f!("objective_setvisibletoall", |_, world, _, a| {
        let i = index(a)?;
        obj(world, i).hidden = 0;
        Ok(Value::Undefined)
    });
    f!("objective_setinvisibletoall", |_, world, _, a| {
        let i = index(a)?;
        obj(world, i).hidden = u64::MAX;
        Ok(Value::Undefined)
    });
    f!("objective_setplayerusing", |vm, world, _, a| {
        let i = index(a)?;
        let b = bit(vm, world, arg(a, 1))?;
        obj(world, i).using |= b;
        Ok(Value::Undefined)
    });
    f!("objective_clearplayerusing", |vm, world, _, a| {
        let i = index(a)?;
        let b = bit(vm, world, arg(a, 1))?;
        obj(world, i).using &= !b;
        Ok(Value::Undefined)
    });
    f!("objective_clearallusing", |_, world, _, a| {
        let i = index(a)?;
        obj(world, i).using = 0;
        Ok(Value::Undefined)
    });
    f!("objective_setprogress", |_, world, _, a| {
        let i = index(a)?;
        let f = num(a, 1)? * 255.0;
        obj(world, i).progress = if f > 0.0 { f.min(255.0) as u8 } else { 0 };
        Ok(Value::Undefined)
    });
    f!("objective_setsize", |vm, world, _, a| {
        let i = index(a)?;
        let size = if a.len() == 2 {
            let n = entnum(vm, arg(a, 1)).ok_or("Illegal entity parameter for Objective_SetSize.")?;
            // Its bounds across: a box trigger's width and length, a
            // radius trigger's diameter.
            let zm = world.resource::<Zm>();
            let e = zm.ents.get(&n).ok_or("Illegal entity parameter for Objective_SetSize.")?;
            match e.box_dims {
                Some(d) => [d[0], d[1]],
                None => [e.radius * 2.0, e.radius * 2.0],
            }
        } else {
            [num(a, 1)?, num(a, 2)?]
        };
        obj(world, i).size = size;
        Ok(Value::Undefined)
    });
    f!("objective_setgamemodeflags", |_, world, _, a| {
        let i = index(a)?;
        let v = int(a, 1)?;
        obj(world, i).gamemode_flags = v as u8;
        Ok(Value::Undefined)
    });
    f!("objective_getgamemodeflags", |_, world, _, a| {
        let i = index(a)?;
        Ok(Value::Int(i32::from(world.resource::<Zm>().objectives[i].gamemode_flags)))
    });
}

/// Each client's objectives (`bo2mp_objectives`), as the engine sends
/// them: not empty, and named, or of no team or his team and not hidden
/// from him. One row each, `;` between:
/// `index,state,x,y,z,entity,icon,name,teammask,progress,flags,team,using`
/// (entity -1 = none; its position is the entity's own, a player's his
/// HUD takes from its snapshot; `using` the clients using it, a hex bit
/// mask).
pub(super) fn publish(world: &mut World, clients: &[(u32, String)]) {
    let rows: Vec<(usize, Objective, [f32; 3])> = {
        let objectives: Vec<(usize, Objective)> = world
            .resource::<Zm>()
            .objectives
            .iter()
            .enumerate()
            .filter(|(_, o)| o.state != 0)
            .map(|(i, o)| (i, o.clone()))
            .collect();
        objectives
            .into_iter()
            .map(|(i, o)| {
                let at = o.entity.and_then(|n| entity_origin(world, n)).unwrap_or(o.origin);
                (i, o, at)
            })
            .collect()
    };
    let clean = |s: &str| s.replace([',', ';'], "_");
    for (c, team) in clients {
        let team = team_number(team).unwrap_or(0);
        let me = 1u64.checked_shl(*c).unwrap_or(0);
        let line = rows
            .iter()
            .filter(|(_, o, _)| !o.name.is_empty() || ((o.team == 0 || o.team == team) && o.hidden & me == 0))
            .map(|(i, o, at)| {
                format!(
                    "{i},{},{},{},{},{},{},{},{},{},{},{},{:x}",
                    o.state,
                    at[0],
                    at[1],
                    at[2],
                    o.entity.map_or(-1, i64::from),
                    clean(&o.icon),
                    clean(&o.name),
                    o.team_mask,
                    o.progress,
                    o.gamemode_flags,
                    o.team,
                    o.using
                )
            })
            .collect::<Vec<_>>()
            .join(";");
        super::set_client_dvar(world, *c, "bo2mp_objectives", &line);
    }
}

/// Where an objective on an entity is: a player's is his own (the HUD
/// follows him in its snapshot, so the row does not change as he runs);
/// another entity's, its origin.
fn entity_origin(world: &mut World, n: u32) -> Option<[f32; 3]> {
    if world.resource::<Zm>().players.contains_key(&n) {
        return None;
    }
    world.resource::<Zm>().ents.get(&n).map(|e| e.origin)
}
