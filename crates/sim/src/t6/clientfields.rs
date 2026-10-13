//! Client fields (`setclientfield` -> `codesetclientfield`,
//! `codesetworldclientfield`): Black Ops II's client scripts turn them into
//! effects and sounds on each client. The engine plays their half here, as
//! those scripts do: a zombie's eye glow, the dirt as it rises, a
//! power-up's glow, a perk machine's trail from the sky, a wall buy's gun.

use bevy_ecs::prelude::World;
use gsc_t6::{ObjKind, Value, Vm};

use super::{Zm, arg, entnum, origin_of};
use crate::EventAudience;

/// Effects waiting their time (or their entity's first showing).
#[derive(Clone, Debug, Default)]
pub(crate) struct ClientFx {
    pending: Vec<Pending>,
    /// Each entity's client field values as last set, for
    /// `codegetclientfield` (Vulture Aid's bit fields, `anim_rate`).
    values: std::collections::HashMap<(u32, String), i64>,
    /// Each player's script vision (`visionset_slot`'s name) and its lerp;
    /// keyed by the manager type too ("visionset", "overlay").
    visions: std::collections::HashMap<(u32, &'static str), (String, f32)>,
    /// `visionsetlaststand`: the vision a player down sees, the seconds it
    /// fades in over and when it was set (the bleed-out's `zombie_death`
    /// fades in from `zombie_last_stand`, the one before it).
    laststand: Option<LastStandVision>,
    /// Each player down: when he went down and the vision he is shown.
    down: std::collections::HashMap<u32, DownView>,
    /// The map's own vision while no script vision is up, as
    /// `bo2zm_vision` text (BO2's client fog-volume vision monitor: Die
    /// Rise's `zm_highrise_off` before the power, `zm_highrise_on` after);
    /// empty draws the map's own grade.
    map_vision: String,
}

#[derive(Clone, Debug)]
struct LastStandVision {
    name: String,
    secs: f32,
    set_ms: i64,
}

#[derive(Clone, Debug)]
struct DownView {
    since: i64,
    /// The `visionsetlaststand` he is shown (when it was set), its name,
    /// and the one he saw before it (it fades in from that).
    set_ms: Option<i64>,
    name: String,
    from: Option<String>,
}

impl ClientFx {
    /// An entity's client field as last set (0 before).
    pub(crate) fn value(&self, ent: u32, name: &str) -> i64 {
        self.values.get(&(ent, name.to_owned())).copied().unwrap_or(0)
    }
}

#[derive(Clone, Debug)]
struct Pending {
    due: i64,
    /// Given up after this (the entity never showed).
    until: i64,
    ent: u32,
    step: Step,
}

#[derive(Clone, Debug)]
enum Step {
    /// An effect where the entity stands, offset.
    At { fx: &'static str, offset: [f32; 3] },
    /// An effect on one of the entity's tags (or taken off it).
    Bolt {
        fx: &'static str,
        tag: &'static str,
        stop: bool,
    },
    /// A script's `playfxontag` on an entity not drawn yet: its effect and
    /// tag by name.
    BoltNamed { fx: String, tag: String },
}

// The client scripts' effects (clientscripts/mp/zombies/_zm.csc,
// _zm_powerups.csc, zm_nuked_fx.csc).
const RISE_BURST: &str = "maps/zombie/fx_mp_zombie_hand_dirt_burst";
const RISE_BILLOW: &str = "maps/zombie/fx_mp_zombie_body_dirt_billowing";
const RISE_DUST: &str = "maps/zombie/fx_mp_zombie_body_dust_falling";
const EYE_GLOW: &str = "misc/fx_zombie_eye_single";
/// zm_nuked.csc zombie_eye_clientfield: after the moon transmission
/// (round 25) every zombie's eyes glow blue.
const EYE_GLOW_BLUE: &str = "maps/zombie/fx_zombie_eye_single_blue";
const GUTS_EXPLOSION: &str = "maps/zombie/fx_zmb_tranzit_torso_explo";
/// A standing zombie's J_SpineLower above its feet.
const SPINE_LOWER: f32 = 40.0;
const PERK_METEOR: &str = "maps/zombie/fx_zmb_trail_perk_meteor";
const POWERUP: [&str; 4] = [
    "misc/fx_zombie_powerup_on",
    "misc/fx_zombie_powerup_solo_on",
    "misc/fx_zombie_powerup_on_red",
    "misc/fx_zombie_powerup_caution_on",
];

fn queue(world: &mut World, ent: u32, after_ms: i64, step: Step) {
    let mut zm = world.resource_mut::<Zm>();
    let due = zm.now_ms + after_ms;
    zm.client_fx.pending.push(Pending {
        due,
        until: due + 2000,
        ent,
        step,
    });
}

/// An effect bolted to entity `ent`'s tag (or, with `stop`, taken off it)
/// in turn with the others queued, once the entity shows.
pub(super) fn bolt_queued(world: &mut World, ent: u32, fx: &'static str, tag: &'static str, stop: bool) {
    queue(world, ent, 0, Step::Bolt { fx, tag, stop });
}

/// A script's `playfxontag` on entity `ent` once the entity shows (it was
/// spawned this tick: the script spawns a `tag_origin` model and bolts an
/// effect to it in the same frame).
pub(super) fn bolt_named_queued(world: &mut World, ent: u32, fx: &str, tag: &str) {
    queue(
        world,
        ent,
        0,
        Step::BoltNamed {
            fx: fx.to_owned(),
            tag: tag.to_owned(),
        },
    );
}

/// An effect bolted to entity `ent`'s tag after a moment (or once the
/// entity first shows): the gibs' neck spurt and blood trails.
pub(super) fn bolt_later(
    world: &mut World,
    ent: u32,
    after_ms: i64,
    fx: &'static str,
    tag: &'static str,
) {
    queue(
        world,
        ent,
        after_ms,
        Step::Bolt {
            fx,
            tag,
            stop: false,
        },
    );
}

fn rand_range(vm: &mut Vm<World>, lo: i32, hi: i32) -> f32 {
    (lo + (vm.rand_u32() % (hi - lo) as u32) as i32) as f32
}

/// An entity's field changed: its client script's effects.
fn entity_field(vm: &mut Vm<World>, world: &mut World, ent: &Value, name: &str, value: i64) {
    let Some(n) = entnum(vm, ent) else { return };
    world
        .resource_mut::<Zm>()
        .client_fx
        .values
        .insert((n, name.to_owned()), value);
    if name.starts_with("score_cf_") {
        score_field(world, n, name, value as i32);
        return;
    }
    match name {
        // handle_zombie_risers: a burst of dirt, a billow, then dust
        // falling off its back while it climbs out.
        "zombie_riser_fx" if value != 0 => {
            let origin = origin_of(vm, world, ent).unwrap_or([0.0; 3]);
            super::natives_fx::sound(world, EventAudience::All, "zmb_zombie_spawn", origin);
            let burst = [0.0, 0.0, rand_range(vm, 5, 10)];
            queue(
                world,
                n,
                0,
                Step::At {
                    fx: RISE_BURST,
                    offset: burst,
                },
            );
            let billow = [
                rand_range(vm, -10, 10),
                rand_range(vm, -10, 10),
                rand_range(vm, 5, 10),
            ];
            queue(
                world,
                n,
                250,
                Step::At {
                    fx: RISE_BILLOW,
                    offset: billow,
                },
            );
            let mut t = 0;
            while t < 5500 {
                queue(
                    world,
                    n,
                    2250 + t,
                    Step::Bolt {
                        fx: RISE_DUST,
                        tag: "j_spineupper",
                        stop: false,
                    },
                );
                t += 300;
            }
        }
        // Mob: pose_dead_body (zm_prison_weap_quest.csc) keeps the body
        // for the wolf head that eats it.
        "make_client_clone" if value != 0 => super::prison_wolves::clone_body(world, n),
        // zombie_gut_explosion_cb: a grenade's kill bursts the body at
        // J_SpineLower as the server ghosts it.
        "zombie_gut_explosion" if value != 0 => queue(
            world,
            n,
            0,
            Step::At {
                fx: GUTS_EXPLOSION,
                offset: [0.0, 0.0, SPINE_LOWER],
            },
        ),
        // zombie_wait_explode: the same burst two seconds on, where the
        // thrown body lies (mature only).
        "zombie_ragdoll_explode" if value != 0 && super::profile_mature() => queue(
            world,
            n,
            2000,
            Step::At {
                fx: GUTS_EXPLOSION,
                offset: [0.0, 0.0, SPINE_LOWER],
            },
        ),
        // createzombieeyes / deletezombieeyes with level._override_eye_fx:
        // blue_eyes from the csc main of TranZit, Die Rise and Buried
        // (orange_eyes once Buried's buried_sq_maxis_eye_glow_override is
        // set), and on Nuketown once the world field zombie_eye_change is.
        // The stop takes both glows off, whichever one this zombie got.
        "zombie_has_eyes" => {
            // zombie_eyes_clientfield_cb also sets the eye shader's brightness
            // (mapshaderconstant scriptVector2.y): get_eyeball_luminance()
            // alive (2; Origins' csc main overrides it to 3), 0.25 once the
            // eyes go out.
            let luminance = if value == 0 {
                0.25
            } else if world.resource::<Zm>().map == "zm_tomb" {
                3.0
            } else {
                2.0
            };
            if let Some(e) = world.resource_mut::<Zm>().ents.get_mut(&n) {
                e.eye_luminance = Some(luminance);
            }
            let zm = world.resource::<Zm>();
            let field = |name: &str| zm.wallbuys.world_fields.get(name) == Some(&1);
            let blue = match zm.map.as_str() {
                "zm_transit" | "zm_highrise" => true,
                "zm_buried" => !field("buried_sq_maxis_eye_glow_override"),
                _ => field("zombie_eye_change"),
            };
            let fxs: &[&'static str] = if value == 0 {
                &[EYE_GLOW, EYE_GLOW_BLUE]
            } else if blue {
                &[EYE_GLOW_BLUE]
            } else {
                &[EYE_GLOW]
            };
            for &fx in fxs {
                queue(
                    world,
                    n,
                    0,
                    Step::Bolt {
                        fx,
                        tag: "j_eyeball_le",
                        stop: value == 0,
                    },
                );
            }
        }
        // powerup_fx_callback: the glow by kind (no glow for 0).
        "powerup_fx" => {
            if let Some(fx) = usize::try_from(value - 1).ok().and_then(|i| POWERUP.get(i)) {
                queue(
                    world,
                    n,
                    0,
                    Step::Bolt {
                        fx,
                        tag: "tag_origin",
                        stop: false,
                    },
                );
            }
        }
        // Mob's Mystery Box glow (_zm_magicbox_prison.csc).
        "magicbox_initial_fx" | "magicbox_amb_fx" | "magicbox_open_fx" | "magicbox_leaving_fx" => {
            super::prison_box::field(vm, world, n, name, value);
        }
        // perk_meteor_fx: the trail while the machine falls.
        "clientfield_perk_intro_fx" => queue(
            world,
            n,
            0,
            Step::Bolt {
                fx: PERK_METEOR,
                tag: "tag_origin",
                stop: value == 0,
            },
        ),
        _ => {}
    }
}

/// The other toplayer fields BO2's client scripts hand to the HUD
/// (`setupclientfieldcodecallbacks("toplayer", ...)` in the maps' .csc
/// files), besides every `perk_` and `powerup_` one: Mob's Afterlife
/// (`player_lives`, `player_in_afterlife`, the float `player_afterlife_mana`:
/// its time bar, `afterlife_mana_watch` maps 200 mana to 1) and tomahawk,
/// the carried part (`buildable`: Tranzit, Die Rise, Buried; `craftable`:
/// Mob, Origins), Buried's Vulture Aid meter (float) and tablet, Origins'
/// challenges.
const HUD_FIELDS: [&str; 13] = [
    "player_lives",
    "player_in_afterlife",
    "player_afterlife_mana",
    "tomahawk_in_use",
    "upgraded_tomahawk_in_use",
    "buildable",
    "craftable",
    "vulture_perk_disease_meter",
    "player_tablet_state",
    "challenge_complete_1",
    "challenge_complete_2",
    "challenge_complete_3",
    "challenge_complete_4",
];

/// The world fields BO2's client scripts hand to the HUD
/// (`setupclientfieldcodecallbacks("world", ...)`): Mob's collected parts
/// (the plane's `quest_state1..5` and `quest_plane_craft_complete`, who
/// holds a part `piece_player1..4`, the shield's, the acid kit's, the
/// warden's key) and Buried's Time Bomb and special round.
const WORLD_HUD_FIELDS: [&str; 21] = [
    "quest_state1",
    "quest_state2",
    "quest_state3",
    "quest_state4",
    "quest_state5",
    "quest_plane_craft_complete",
    "piece_player1",
    "piece_player2",
    "piece_player3",
    "piece_player4",
    "piece_riotshield_clamp",
    "piece_riotshield_dolly",
    "piece_riotshield_door",
    "piece_packasplat_blood",
    "piece_packasplat_case",
    "piece_packasplat_fuse",
    "piece_key_warden",
    "time_bomb_hud_toggle",
    "time_bomb_lua_override",
    "time_bomb_saved_round_number",
    "sq_tpo_special_round_active",
];

/// The HUD's world fields as set (0 before) go to every player as
/// `bo2zm_worldcf` ("field:value,...", in `WORLD_HUD_FIELDS`' order); his
/// HUD fires each change as the field's own event, as with his own fields.
fn world_hud_fields(world: &mut World) {
    let (text, players) = {
        let zm = world.resource::<Zm>();
        let text = WORLD_HUD_FIELDS
            .iter()
            .filter_map(|f| {
                let v = *zm.wallbuys.world_fields.get(*f)?;
                (v != 0).then(|| format!("{f}:{v}"))
            })
            .collect::<Vec<_>>()
            .join(",");
        (text, zm.players.keys().copied().collect::<Vec<_>>())
    };
    for n in players {
        super::set_client_dvar(world, n, "bo2zm_worldcf", &text);
    }
}

/// A player's own field (`setclientfieldtoplayer`): the ones his HUD reads
/// (perks, power-ups, `HUD_FIELDS`) go to his client as `bo2zm_icons`
/// ("field:value,...", a float field's value as a float).
fn player_field(world: &mut World, n: u32, name: &str, fvalue: f32) {
    let value = fvalue as i32;
    // Buried's town lighting (its client script, solo or online): the sun
    // light and the sky's horizon brightness, as `bo2zm_sunlight`.
    if name == "clientfield_underground_lighting" {
        let text = if value != 0 { "18 10" } else { "13 1" };
        super::set_client_dvar(world, n, "bo2zm_sunlight", text);
        return;
    }
    // Origins' weather (its client script): fog bank 1 dry, 2 rain, 4 snow;
    // snow pales the sun colour (`bo2zm_suncolor`), the others keep the map's.
    if name == "player_weather_visionset" {
        let (bank, colour) = match value {
            1 => ("2", ""),
            2 => ("4", "0.769943 0.894102 1"),
            _ => ("", ""),
        };
        super::set_client_dvar(world, n, "bo2zm_fogbank", bank);
        super::set_client_dvar(world, n, "bo2zm_suncolor", colour);
        return;
    }
    if !(name.starts_with("perk_") || name.starts_with("powerup_") || HUD_FIELDS.contains(&name)) {
        return;
    }
    let list = {
        let mut zm = world.resource_mut::<Zm>();
        let Some(p) = zm.players.get_mut(&n) else {
            return;
        };
        let at = p.hud_fields.iter().position(|(k, _)| k == name);
        match (at, fvalue) {
            (Some(i), 0.0) => {
                p.hud_fields.remove(i);
            }
            (Some(i), v) if p.hud_fields[i].1 != v => p.hud_fields[i].1 = v,
            (Some(_), _) => return,
            (None, 0.0) => return,
            (None, v) => p.hud_fields.push((name.to_owned(), v)),
        }
        p.hud_fields
            .iter()
            .map(|(k, v)| format!("{k}:{v}"))
            .collect::<Vec<_>>()
            .join(",")
    };
    super::set_client_dvar(world, n, "bo2zm_icons", &list);
}

/// A player's points counter (`_zm_score::score_cf_increment_info` steps
/// `score_cf_damage`, `score_cf_death_head`, ... on each hit and kill, and
/// wraps): his client gets them as `bo2zm_scorecf` ("field:value,..."), and
/// his HUD flies one "+10" per step (hudcompetitivescoreboardzombie.lua).
fn score_field(world: &mut World, n: u32, name: &str, value: i32) {
    let list = {
        let mut zm = world.resource_mut::<Zm>();
        let Some(p) = zm.players.get_mut(&n) else {
            return;
        };
        match p.score_fields.iter_mut().find(|(k, _)| k == name) {
            Some((_, v)) if *v == value => return,
            Some((_, v)) => *v = value,
            None => p.score_fields.push((name.to_owned(), value)),
        }
        p.score_fields
            .iter()
            .map(|(k, v)| format!("{k}:{v}"))
            .collect::<Vec<_>>()
            .join(",")
    };
    super::set_client_dvar(world, n, "bo2zm_scorecf", &list);
}

/// The visionset manager's two client fields (maps/mp/_visionset_mgr.gsc
/// `update_clientfields`): `visionset_slot`, the player's top vision as its
/// index in `level.vsmgr["visionset"].sorted_name_keys` (the default
/// "none" draws the map's own), and `visionset_lerp`, how far in it is.
/// His client gets them as `bo2zm_vision` ("name lerp"); its vision file is
/// the info's name (Mob's `zm_afterlife` is `vision/zm_afterlife.vision`).
/// The overlay type's two (`overlay_slot`, `overlay_lerp`: Mob's Afterlife
/// screen filter `zm_afterlife_filter`) go as `bo2zm_overlay`.
fn vision_field(vm: &mut Vm<World>, world: &mut World, n: u32, field: &str, value: &Value) {
    let (kind, dvar) = if field.starts_with("overlay_") {
        ("overlay", "bo2zm_overlay")
    } else {
        ("visionset", "bo2zm_vision")
    };
    let (name, lerp) = world
        .resource::<Zm>()
        .client_fx
        .visions
        .get(&(n, kind))
        .cloned()
        .unwrap_or_else(|| (String::new(), 1.0));
    let (name, lerp) = if field.ends_with("_slot") {
        let slot = value.as_int().unwrap_or(0);
        (vision_slot_name(vm, kind, slot).unwrap_or_default(), lerp)
    } else {
        (name, value.as_float().unwrap_or(1.0))
    };
    let name = if name.eq_ignore_ascii_case("none") { String::new() } else { name };
    let text = if !name.is_empty() {
        format!("{name} {lerp}")
    } else if kind == "visionset" {
        world.resource::<Zm>().client_fx.map_vision.clone()
    } else {
        String::new()
    };
    let mut zm = world.resource_mut::<Zm>();
    let before = zm.client_fx.visions.insert((n, kind), (name.clone(), lerp));
    // Down, he sees the last stand's vision; this one comes back when he
    // is up (`laststand_visions`).
    let downed = zm.client_fx.down.get(&n).is_some_and(|v| v.set_ms.is_some());
    drop(zm);
    if kind == "overlay" {
        let before = before.map(|(name, _)| name).unwrap_or_default();
        if let Some((to, secs)) = overlay_blur_change(&before, &name) {
            overlay_blur(world, n, to, secs);
        }
    }
    if kind == "visionset" && downed {
        return;
    }
    super::set_client_dvar(world, n, dvar, &text);
}

/// The visionset manager's blur overlays (style 2,
/// `vsmgr_register_overlay_info_style_blur`): name, blur, seconds in,
/// seconds out. TranZit's Denizen blurs the screen of the player it holds
/// (`_zm_ai_screecher.csc`); no other map has one.
const BLUR_OVERLAYS: &[(&str, f32, f32, f32)] = &[("zm_ai_screecher_blur", 4.0, 0.1, 0.25)];

/// A player's overlay went from `before` to `now`: a blur overlay blurs his
/// screen to its strength over its seconds in, and back to none over its
/// seconds out when it goes (`overlay_update_cb`). The blur and the seconds.
fn overlay_blur_change(before: &str, now: &str) -> Option<(f32, f32)> {
    if before.eq_ignore_ascii_case(now) {
        return None;
    }
    let find = |name: &str| BLUR_OVERLAYS.iter().find(|b| b.0.eq_ignore_ascii_case(name));
    match (find(before), find(now)) {
        (_, Some(&(_, blur, secs_in, _))) => Some((blur, secs_in)),
        (Some(&(_, _, _, secs_out)), None) => Some((0.0, secs_out)),
        (None, None) => None,
    }
}

/// `setblurbylocalclientnum(blur, seconds)`: his screen blurs from where it
/// is to `to` (as `setblur`).
fn overlay_blur(world: &mut World, n: u32, to: f32, secs: f32) {
    let now = crate::level_time_ms(super::tick(world));
    let mut f = super::frame(world);
    let id = crate::world::ClientId(n);
    if f.client_meta(id).is_none() {
        return;
    }
    let view = &mut f.client_meta_mut(id).view_effects;
    view.blur = Some(crate::ScriptBlur {
        from: view.blur.map_or(0.0, |b| b.sample(now)),
        to,
        set_ms: now,
        duration_ms: (secs * 1000.0).round() as i32,
    });
}

/// A player went down (a lethal hit): a `visionsetlaststand` set from now
/// on is his to see.
pub(super) fn went_down(world: &mut World, n: u32) {
    let now = world.resource::<Zm>().now_ms;
    world
        .resource_mut::<Zm>()
        .client_fx
        .down
        .entry(n)
        .or_insert_with(|| DownView { since: now, set_ms: None, name: String::new(), from: None });
}

/// Each tick: a player down sees `visionsetlaststand`'s vision (BO2's grey
/// `zombie_last_stand`, then `zombie_death` half way to bleeding out),
/// faded in over its seconds - from the one he saw before it, if any. One
/// set before he went down is not his (Who's Who sets none). Up again, he
/// gets the visionset manager's vision back.
pub(super) fn laststand_visions(world: &mut World) {
    let now = world.resource::<Zm>().now_ms;
    let players: Vec<(u32, bool)> = world
        .resource::<Zm>()
        .players
        .iter()
        .map(|(c, p)| (*c, p.laststand))
        .collect();
    for (c, down) in players {
        let text = {
            let mut zm = world.resource_mut::<Zm>();
            let fx = &mut zm.client_fx;
            if !down {
                let Some(view) = fx.down.remove(&c) else { continue };
                if view.set_ms.is_none() {
                    continue;
                }
                let (name, lerp) = fx.visions.get(&(c, "visionset")).cloned().unwrap_or_default();
                if name.is_empty() { fx.map_vision.clone() } else { format!("{name} {lerp}") }
            } else {
                let Some(v) = fx.laststand.clone() else { continue };
                let view = fx.down.entry(c).or_insert_with(|| DownView {
                    since: now,
                    set_ms: None,
                    name: String::new(),
                    from: None,
                });
                if v.set_ms < view.since {
                    continue;
                }
                if view.set_ms != Some(v.set_ms) {
                    view.from = (!view.name.is_empty() && view.name != v.name).then(|| view.name.clone());
                    view.set_ms = Some(v.set_ms);
                    view.name.clone_from(&v.name);
                }
                let t = if v.secs > 0.0 {
                    ((now - v.set_ms) as f32 / (v.secs * 1000.0)).clamp(0.0, 1.0)
                } else {
                    1.0
                };
                // In steps of 5%: his client re-grades on each new value.
                let t = (t * 20.0).round() / 20.0;
                match &view.from {
                    Some(from) if t < 1.0 => format!("{} {t} {from}", v.name),
                    _ => format!("{} {t}", v.name),
                }
            }
        };
        super::set_client_dvar(world, c, "bo2zm_vision", &text);
    }
}

/// The map's own vision (`map_vision`), sent to each player who is up with
/// no script vision of his own (a player down, or in Who's Who's, gets it
/// when that ends).
pub(crate) fn set_map_vision(world: &mut World, text: &str) {
    let players: Vec<u32> = {
        let mut zm = world.resource_mut::<Zm>();
        text.clone_into(&mut zm.client_fx.map_vision);
        let fx = &zm.client_fx;
        zm.players
            .keys()
            .copied()
            .filter(|c| {
                !fx.down.get(c).is_some_and(|v| v.set_ms.is_some())
                    && !matches!(fx.visions.get(&(*c, "visionset")), Some((n, _)) if !n.is_empty())
            })
            .collect()
    };
    for c in players {
        super::set_client_dvar(world, c, "bo2zm_vision", text);
    }
}

fn vision_slot_name(vm: &mut Vm<World>, kind: &str, slot: i32) -> Option<String> {
    let vsmgr = vm.intern("vsmgr");
    let Value::Array(types) = vm.raw_field(vm.level, vsmgr) else {
        return None;
    };
    let visionset = gsc_t6::Key::Str(vm.intern(kind));
    let Some(Value::Object(info)) = types.get(&visionset) else {
        return None;
    };
    let keys = vm.intern("sorted_name_keys");
    let Value::Array(names) = vm.raw_field(info, keys) else {
        return None;
    };
    names.get(&gsc_t6::Key::Int(slot)).map(|v| vm.to_text(&v))
}

pub(super) fn bind(vm: &mut Vm<World>) {
    // A script camera: camerasetposition(entity), cameraactivate(on);
    // camerasetlookat() keeps the camera entity's own angles.
    vm.bind("camerasetposition", true, |vm, world, s, a| {
        if let (Some(n), Some(cam)) = (entnum(vm, s), entnum(vm, arg(a, 0)))
            && let Some(p) = world.resource_mut::<Zm>().players.get_mut(&n)
        {
            p.camera = Some(cam);
        }
        Ok(Value::Undefined)
    });
    vm.bind("cameraactivate", true, |vm, world, s, a| {
        let on = !matches!(arg(a, 0), Value::Int(0)) && !arg(a, 0).is_undefined();
        if let Some(n) = entnum(vm, s)
            && let Some(p) = world.resource_mut::<Zm>().players.get_mut(&n)
        {
            p.camera_on = on;
        }
        Ok(Value::Undefined)
    });
    vm.bind("camerasetlookat", true, |_, _, _, _| Ok(Value::Undefined));
    // setviewmodel(name): his first-person hands (Mob's Afterlife turns on
    // c_zom_ghost_viewhands, then puts his own back), sent as the client
    // dvar `bo2zm_viewhands`.
    vm.bind("setviewmodel", true, |vm, world, s, a| {
        if let Some(n) = entnum(vm, s) {
            let name = vm.to_text(arg(a, 0)).to_ascii_lowercase();
            super::set_client_dvar(world, n, "bo2zm_viewhands", &name);
        }
        Ok(Value::Undefined)
    });
    // setclientuivisibilityflag("hud_visible", 0): his HUD hides (the
    // game-over shot); script HUD text still shows.
    // bo2mp: setclientuivisibilityflag("g_compassShowEnemies", 1): a moment
    // of radar (the Lightning Strike's pick shows where enemies are).
    vm.bind("setclientuivisibilityflag", true, |vm, world, s, a| {
        let flag = vm.to_text(arg(a, 0));
        if flag == "hud_visible"
            && let Some(n) = entnum(vm, s)
        {
            let hidden = matches!(arg(a, 1), Value::Int(0));
            super::set_client_dvar(world, n, "bo2zm_hud_hidden", if hidden { "1" } else { "" });
        }
        if flag == "g_compassShowEnemies"
            && arg(a, 1).as_float().unwrap_or(0.0) != 0.0
            && let Some(n) = entnum(vm, s)
            && world.resource::<Zm>().mp
        {
            super::killstreaks::radar_pulse(world, n);
        }
        Ok(Value::Undefined)
    });
    // setclientammocounterhide(1) / setclientminiscoreboardhide(1) (the
    // game over): his ammo counter and points hide (BO2's HUD bit 12).
    fn hide(vm: &mut Vm<World>, world: &mut World, s: &Value, a: &[Value], key: &str) {
        if let Some(n) = entnum(vm, s) {
            let on = !matches!(arg(a, 0), Value::Int(0)) && !arg(a, 0).is_undefined();
            super::set_client_dvar(world, n, key, if on { "1" } else { "" });
        }
    }
    vm.bind("setclientammocounterhide", true, |vm, world, s, a| {
        hide(vm, world, s, a, "bo2zm_ammo_hide");
        Ok(Value::Undefined)
    });
    vm.bind("setclientminiscoreboardhide", true, |vm, world, s, a| {
        hide(vm, world, s, a, "bo2zm_score_hide");
        Ok(Value::Undefined)
    });
    vm.bind("codesetplayerstateclientfield", false, |vm, world, _, a| {
        if let Some(n) = entnum(vm, arg(a, 0)) {
            let name = vm.to_text(arg(a, 1));
            if matches!(
                name.as_str(),
                "visionset_slot" | "visionset_lerp" | "overlay_slot" | "overlay_lerp"
            ) {
                vision_field(vm, world, n, &name, arg(a, 2));
                return Ok(Value::Undefined);
            }
            let fvalue = arg(a, 2).as_float().unwrap_or(0.0);
            let value = fvalue as i32;
            if name == "clientfield_sq_vo" {
                super::highrise::maxis_vo(vm, world, n, value);
            }
            player_field(world, n, &name, fvalue);
        }
        Ok(Value::Undefined)
    });
    // maps/mp/_utility::setclientfield calls these.
    vm.bind("codesetworldclientfield", false, |vm, world, _, a| {
        let name = vm.to_text(arg(a, 0));
        let value = arg(a, 1).as_int().unwrap_or(0);
        super::wallbuys::world_field(vm, world, &name, value);
        super::prison_wolves::world_field(vm, world, &name, i64::from(value));
        if WORLD_HUD_FIELDS.contains(&name.as_str()) {
            world_hud_fields(world);
        }
        Ok(Value::Undefined)
    });
    // maps/mp/_utility::getclientfield on level: the world field as last
    // set (0 before; Origins' capture zones read theirs back).
    vm.bind("codegetworldclientfield", false, |vm, world, _, a| {
        let name = vm.to_text(arg(a, 0));
        let value = world
            .resource::<Zm>()
            .wallbuys
            .world_fields
            .get(&name)
            .copied()
            .unwrap_or(0);
        Ok(Value::Int(value))
    });
    // visionsetlaststand(name, seconds): the vision a player down sees
    // (maps/mp/zombies/_zm::player_laststand "zombie_last_stand" over 1 s;
    // _zm_laststand "zombie_death" over the bleed-out's second half), shown
    // by `laststand_visions`.
    vm.bind("visionsetlaststand", false, |vm, world, _, a| {
        let name = vm.to_text(arg(a, 0)).to_ascii_lowercase();
        let secs = arg(a, 1).as_float().unwrap_or(0.0).max(0.0);
        let mut zm = world.resource_mut::<Zm>();
        let set_ms = zm.now_ms;
        zm.client_fx.laststand = (!name.is_empty()).then_some(LastStandVision { name, secs, set_ms });
        Ok(Value::Undefined)
    });
    vm.bind("codesetclientfield", false, |vm, world, _, a| {
        let ent = arg(a, 0).clone();
        let name = vm.to_text(arg(a, 1));
        let value = arg(a, 2).as_int().map_or(0, i64::from);
        entity_field(vm, world, &ent, &name, value);
        Ok(Value::Undefined)
    });
    // maps/mp/_utility::getclientfield: the value as last set (0 before).
    vm.bind("codegetclientfield", false, |vm, world, _, a| {
        let name = vm.to_text(arg(a, 1));
        let value = entnum(vm, arg(a, 0)).map_or(0, |n| {
            world
                .resource::<Zm>()
                .client_fx
                .values
                .get(&(n, name))
                .copied()
                .unwrap_or(0)
        });
        Ok(Value::Int(value as i32))
    });
    // Scripts that call the method directly.
    vm.bind("setclientfield", true, |vm, world, s, a| {
        let name = vm.to_text(arg(a, 0));
        let value = arg(a, 1).as_int().unwrap_or(0);
        if matches!(s, Value::Object(o) if matches!(vm.kind(*o), Some(ObjKind::Level))) {
            super::wallbuys::world_field(vm, world, &name, value);
            super::prison_wolves::world_field(vm, world, &name, i64::from(value));
        } else {
            entity_field(vm, world, s, &name, i64::from(value));
        }
        Ok(Value::Undefined)
    });
}

/// Each tick (after the entities show): the effects that are due.
pub(crate) fn advance(world: &mut World, now: i64) {
    let due: Vec<Pending> = {
        let mut zm = world.resource_mut::<Zm>();
        if zm.client_fx.pending.is_empty() {
            return;
        }
        let (due, keep) = std::mem::take(&mut zm.client_fx.pending)
            .into_iter()
            .partition(|p| p.due <= now);
        zm.client_fx.pending = keep;
        due
    };
    for p in due {
        let shown = world
            .resource::<Zm>()
            .presences
            .by_ent
            .get(&p.ent)
            .map(|s| s.origin);
        let Some(origin) = shown else {
            // Not drawn yet (spawned this tick): try again until it is.
            if now < p.until {
                world
                    .resource_mut::<Zm>()
                    .client_fx
                    .pending
                    .push(Pending { due: now + 50, ..p });
            }
            continue;
        };
        match p.step {
            Step::At { fx, offset } => {
                let at = [
                    origin[0] + offset[0],
                    origin[1] + offset[1],
                    origin[2] + offset[2],
                ];
                super::natives_fx::effect(world, fx, at, [0.0, 0.0, 1.0]);
            }
            Step::Bolt { fx, tag, stop } => {
                super::natives_fx::bolt_effect(world, p.ent, fx, tag, stop);
            }
            Step::BoltNamed { fx, tag } => {
                super::natives_fx::bolt_effect(world, p.ent, &fx, &tag, false);
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_denizen_blurs_in_and_out() {
        assert_eq!(overlay_blur_change("", "zm_ai_screecher_blur"), Some((4.0, 0.1)));
        assert_eq!(overlay_blur_change("zm_ai_screecher_blur", ""), Some((0.0, 0.25)));
        assert_eq!(overlay_blur_change("zm_ai_screecher_blur", "zm_ai_screecher_blur"), None);
        assert_eq!(overlay_blur_change("", "zm_afterlife_filter"), None);
    }
}
