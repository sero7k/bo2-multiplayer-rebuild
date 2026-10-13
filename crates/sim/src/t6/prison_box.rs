//! Mob of the Dead's Mystery Box glow: the client half of
//! `_zm_magicbox_prison` (its .csc). Each box place gets an anchor (the
//! script's `fx_obj`, a tag_origin where the box stands) that its effects
//! hang on: the low glow and hum while the box is away, the bright one where
//! it stands, the locked glow while Brutus holds it, the burst as it opens
//! and as it leaves.

use std::collections::HashMap;

use bevy_ecs::prelude::{Resource, World};
use gsc_t6::{ObjKind, Vm};

use super::{Ent, Zm};
use crate::EventAudience;

const OPEN: &str = "maps/zombie_alcatraz/fx_alcatraz_magicbox_open";
const LOCKED: &str = "maps/zombie_alcatraz/fx_alcatraz_magicbox_lock";
const AWAY: &str = "maps/zombie_alcatraz/fx_alcatraz_magicbox_amb";
const HERE: &str = "maps/zombie_alcatraz/fx_alcatraz_magicbox_arrive";
const LEAVING: &str = "maps/zombie_alcatraz/fx_alcatraz_magicbox_leave";
const HUM_LOW: &str = "zmb_hellbox_amb_low";
const HUM_HIGH: &str = "zmb_hellbox_amb_high";

/// Each box place (its zbarrier) -> its anchor and the glow on it now.
#[derive(Resource, Default)]
struct Anchors(HashMap<u32, (u32, Option<&'static str>)>);

/// One of the box's client fields changed on zbarrier `n`.
pub(super) fn field(vm: &mut Vm<World>, world: &mut World, n: u32, name: &str, value: i64) {
    let Some((origin, angles)) = world.resource::<Zm>().ents.get(&n).map(|e| (e.origin, e.angles)) else {
        return;
    };
    let known = world
        .get_resource::<Anchors>()
        .and_then(|a| a.0.get(&n).copied());
    let (anchor, glow) = match known {
        Some(a) => a,
        None => {
            let a = world.resource_mut::<Zm>().alloc_entnum();
            let obj = vm.alloc_object(ObjKind::Entity(a));
            world.resource_mut::<Zm>().ents.insert(
                a,
                Ent {
                    obj: Some(obj),
                    classname: "script_model".into(),
                    model: "tag_origin".into(),
                    origin,
                    angles,
                    ..Default::default()
                },
            );
            world.init_resource::<Anchors>();
            world.resource_mut::<Anchors>().0.insert(n, (a, None));
            (a, None)
        }
    };
    match name {
        // magicbox_initial_closed_fx: only on a place not set up before.
        "magicbox_initial_fx" if known.is_none() && value == 1 => {
            set_glow(world, n, anchor, glow, AWAY, HUM_LOW);
        }
        "magicbox_amb_fx" => {
            let (fx, hum, sound) = match value {
                0 => (AWAY, HUM_LOW, Some("zmb_hellbox_leave")),
                1 => (HERE, HUM_HIGH, Some("zmb_hellbox_arrive")),
                2 => (LOCKED, HUM_HIGH, None),
                _ => return,
            };
            set_glow(world, n, anchor, glow, fx, hum);
            if let Some(alias) = sound {
                super::natives_fx::sound(world, EventAudience::All, alias, origin);
            }
        }
        "magicbox_open_fx" => super::clientfields::bolt_queued(world, anchor, OPEN, "tag_origin", value == 0),
        "magicbox_leaving_fx" if value == 1 => {
            super::clientfields::bolt_queued(world, anchor, LEAVING, "tag_origin", false);
        }
        _ => {}
    }
}

/// The glow on the anchor swapped for `fx`, and its hum.
fn set_glow(
    world: &mut World,
    n: u32,
    anchor: u32,
    old: Option<&'static str>,
    fx: &'static str,
    hum: &str,
) {
    if let Some(old) = old {
        super::clientfields::bolt_queued(world, anchor, old, "tag_origin", true);
    }
    super::clientfields::bolt_queued(world, anchor, fx, "tag_origin", false);
    if let Some(e) = world.resource_mut::<Zm>().ents.get_mut(&anchor) {
        e.loop_sound = Some(hum.to_owned());
    }
    world.resource_mut::<Anchors>().0.insert(n, (anchor, Some(fx)));
}
