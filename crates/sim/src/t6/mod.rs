//! bo2zm M3: Black Ops II Zombies' own scripts and the engine side they call.
//!
//! A BO2 map hands its zones' compiled scripts to [`install`]; the `gsc_t6`
//! VM runs them, here, beside IW4L's GSC runtime (which then only keeps its
//! stub start-up: dvar `bo2zm_t6` set makes its connect callback step aside).
//! The level start follows the game's order: map entities and structs, the
//! game type's `main`, the map's `main`, `codecallback_startgametype`; then
//! every authority frame connects players and runs the scheduler.
//!
//! Engine state the scripts see lives in [`Zm`]: entities (map and spawned),
//! players, movers, tables. Natives are `fn(&mut Vm<World>, &mut World, self,
//! args)`. Not part of upstream IW4L.

use std::collections::{BTreeMap, BTreeSet};
use std::sync::Arc;

use bevy_ecs::prelude::{Resource, World};
use gsc_t6::{Hooks, ObjKind, ObjRef, Program, ScriptObject, Strings, Value, Vm};

use crate::frame::FrameWorld;
use crate::world::ClientId;

mod actors;
mod asd;
mod audio_triggers;
mod autoplay;
mod input_replay;
mod bots;
pub mod br;
mod autoplay_prison;
mod brushes;
mod buried_egg;
mod cheats; // bo2zm: the solo cheat menu (Num 1)
mod clientfields;
mod destructible;
mod engine_sounds;
mod darts;
mod bo1_maps;
// bo2mp: Zombies maps from other games (Infinite Warfare's).
pub mod foreign_zm;
mod fields;
mod flight;
mod fxanims;
mod globallogic;
mod grenades;
mod hud;
mod hurt;
mod influencers;
mod items;
mod killstreaks;
mod loadout;
mod map_natives;
mod movers;
pub(crate) mod mp;
pub(crate) mod trickshot;
mod natives_ai;
mod natives_ent;
mod highrise;
mod natives_fx;
mod natives_game;
pub mod slowmo; // bo2mp: setslowmotion, read by the final killcam
mod natives_mp;
mod natives_player;
mod nav;
mod objectives;
mod playeranim;
mod players;
mod presence;
pub mod stats;
mod prison_box;
mod prison_plane;
mod prison_retriever;
mod prison_wolves;
mod tomb_craft;
mod tomb_biplane;
mod tomb_mechz;
mod tomb_portal;
mod tomb_robot;
mod tomb_staffs;
mod tags;
mod tomb_tank;
mod tomb_test;
mod riders;
mod riotshield;
pub(crate) mod jetgun;
mod limits;
mod offhand;
pub(crate) use offhand::flashbang as t6_flashbang;
mod transit;
mod triggers;
mod turrets;
pub(crate) use players::weapon_notes;
pub(crate) use grenades::note_detonation;
pub(crate) use triggers::{DamageKind, damage_triggers};
mod use_test;
mod vehicles;
mod wallbuys;
mod zbarrier;

pub(crate) use items::collect_pickups as collect_item_pickups;

/// A string table (`mp/zombiemode.csv`, ...).
#[derive(Clone, Debug, Default)]
pub struct T6Table {
    pub columns: usize,
    pub rows: usize,
    pub cells: Vec<String>,
}

/// What a BO2 map gives the script runtime.
#[derive(Clone, Debug, Default)]
pub struct T6Install {
    pub map: String,
    pub gametype: String,
    /// The start location when chosen (Tranzit's `town`); empty = the map's own.
    pub location: String,
    /// Compiled script objects in zone load order (later replace earlier).
    pub objects: Vec<Vec<u8>>,
    /// String tables by lower-case name.
    pub tables: BTreeMap<String, T6Table>,
    /// The map's entity string(s).
    pub entities: Vec<String>,
    /// The map's AI path nodes, in node order.
    pub path_nodes: Vec<T6PathNode>,
    /// The map compiler's baked node sight and smooth tables.
    pub path_tables: T6PathTables,
    /// Animations the server times and moves actors by.
    pub anims: Vec<T6Anim>,
    /// Animation state definitions: (name, text).
    pub animstatedefs: Vec<(String, String)>,
    /// Skeletons and hit boxes of the models entities can wear, by name.
    pub models: Vec<(String, Arc<xmodel_runtime::RetainedModelCapability>)>,
    /// Clips actors are posed with for hits, by (lowercase) name.
    pub clips: Vec<(String, Arc<xmodel_runtime::AnimClip>)>,
    /// The English strings (`ZOMBIE_WEAPON_M14` -> its text).
    pub strings: std::collections::HashMap<String, String>,
    /// Every sound alias the banks hold (lower case).
    pub sound_aliases: BTreeSet<String>,
    /// bo2mp: the game type's settings (`getgametypesetting`), lower-case
    /// name -> value: `mp/gamesettings_default.cfg` then the game type's own.
    pub gamesettings: BTreeMap<String, String>,
    /// bo2mp: the lobby's bots (friends, enemies, difficulty 0-3).
    pub bots: Option<(u32, u32, u32)>,
    /// bo2mp: sound alias lengths in ms (lower-case names).
    pub sound_lengths: std::collections::HashMap<String, u32>,
    /// bo2mp: retrievable weapons (getretrievableweapons).
    pub retrievable_weapons: Vec<String>,
    /// bo2mp: each vehicle's turret and gunner weapons, and how it drives.
    pub vehicles: Vec<T6Vehicle>,
    /// bo2mp: BO2's third-person player animation script, parsed.
    pub playeranim: Option<Arc<crate::t6_playeranim::T6PlayerAnims>>,
    /// bo2mp: the destructibles the map's entities name (`destructibledef`).
    pub destructibles: Vec<Arc<xmodel_runtime::T5DestructibleDef>>,
    /// Barrier types (`ZBarrierDef`) the map's zbarriers name by `type`.
    pub zbarrier_defs: Vec<T6ZBarrierDef>,
}

/// A barrier type: the zombie animation states its windows ask for.
#[derive(Clone, Debug, Default)]
pub struct T6ZBarrierDef {
    pub name: String,
    pub taunts: bool,
    pub reach_through: bool,
    /// Pieces torn off (state "open") stop drawing (`autoHideOpenPieces`).
    pub auto_hide_open_pieces: bool,
    pub taunt_state: String,
    pub reach_through_state: String,
    pub num_attack_slots: u32,
    pub attack_spot_horz_offset: f32,
    /// Per board: (tear anim state, tear anim substate).
    pub boards: Vec<(String, String)>,
}

/// bo2mp: a vehicle def from his zones: its guns, how it drives and how
/// its driver sees it.
#[derive(Clone, Debug, Default)]
pub struct T6Vehicle {
    pub name: String,
    pub turret: String,
    pub gunners: Vec<String>,
    pub drive: T6VehicleDrive,
}

/// bo2mp: a vehicle's driving and camera values (`VehicleDef`); speeds in
/// units a second, angles in degrees.
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct T6VehicleDrive {
    /// 0: his view from the vehicle's `tag_player`; else third person.
    pub camera_mode: i32,
    pub max_speed: f32,
    pub max_speed_vertical: f32,
    pub accel: f32,
    pub accel_vertical: f32,
    /// Reversing, a share of `max_speed`.
    pub reverse_scale: f32,
    /// The third-person camera: how far back, how high it looks from, how
    /// far up and down it may look.
    pub camera_range: f32,
    pub camera_height: f32,
    pub camera_pitch: [f32; 2],
    /// 0: his own field of view.
    pub camera_fov: f32,
    /// How far he may look up and down from its gun (first person).
    pub turret_pitch: [f32; 2],
    /// `thirdPersonDriver`: the driver's seat views from outside.
    pub third_person_driver: i32,
    /// The commands of `moveUpButtonName`, `moveDownButtonName`,
    /// `switchSeatButtonName`, `attackButtonName`, `attackSecondaryButtonName`
    /// in BO2's default controller binds ("" for none).
    pub buttons: [&'static str; 5],
}

/// An AI path node and its links (node, distance, negotiation).
#[derive(Clone, Debug, Default)]
pub struct T6PathNode {
    pub ty: u32,
    pub spawnflags: u32,
    pub targetname: String,
    pub target: String,
    pub script_noteworthy: String,
    pub script_linkname: String,
    pub animscript: String,
    pub origin: [f32; 3],
    pub angle: f32,
    pub radius: f32,
    pub links: Vec<(u16, f32, bool)>,
}

/// The map compiler's baked path tables: which node pairs see each other
/// (`vis`, bit `node_count * i + j` for i < j) and which a straight walk
/// joins (`smooth`, bit `original_count * i + j`).
#[derive(Clone, Debug, Default)]
pub struct T6PathTables {
    pub node_count: u32,
    pub original_count: u32,
    pub vis: Vec<u8>,
    pub smooth: Vec<u8>,
}

/// An animation's timing, root motion and notetracks.
#[derive(Clone, Debug, Default)]
pub struct T6Anim {
    pub name: String,
    pub numframes: u16,
    pub framerate: f32,
    pub looping: bool,
    pub delta_trans: Vec<(u16, [f32; 3])>,
    /// The root's turn keys (frame, quaternion x y z w); empty: no turn.
    pub delta_quat: Vec<(u16, [f32; 4])>,
    pub notifies: Vec<(String, f32)>,
}

#[derive(Resource)]
pub(crate) struct T6Runtime {
    pub vm: Vm<World>,
    /// His "Omni Movement" setting when the game started.
    pub omni: bool,
}

/// An entity the scripts see (not a player).
#[derive(Clone, Debug, Default)]
pub(crate) struct Ent {
    pub obj: Option<ObjRef>,
    pub classname: String,
    pub origin: [f32; 3],
    pub angles: [f32; 3],
    pub model: String,
    pub hidden: bool,
    /// Hidden by `ghost`: not drawn, but still shot and hit (Origins' wind
    /// staff smoke detectors are ghosted brushes the staff's shots turn).
    pub ghosted: bool,
    pub solid: bool,
    /// From the map (not spawned by a script).
    pub map: bool,
    /// bo2mp: the block's place in the map's entity text (a map from
    /// another game: its own drawn model or brush, installed by the
    /// session under this place, follows this entity: `map_natives`).
    pub map_ordinal: Option<u32>,
    /// Brush model (`*N`) a map entity uses.
    pub brush: Option<String>,
    /// A brush entity's angles as the map placed it (its brushes are built
    /// that way; a trigger turned since turns them, `triggers::touches`).
    pub spawn_angles: Option<[f32; 3]>,
    /// Trigger radius/height (`trigger_radius*`) or brush extents.
    pub radius: f32,
    pub height: f32,
    /// A zbarrier's pieces (the magic box).
    pub zbarrier: Option<zbarrier::ZBarrier>,
    /// A box trigger (`trigger_box`, `trigger_box_use`): width (along its
    /// forward), length (right), height, centred on the origin and turned
    /// by its angles.
    pub box_dims: Option<[f32; 3]>,
    /// `usetriggerrequirelookat`: the player must also face the trigger's
    /// centre (`triggers::box_use_reach`), not only stand in it.
    pub look_at: bool,
    /// `sethintlowpriority(1)`: its hint yields to any other trigger he can
    /// use there (every buyable door, `_zm_blockers` door_think).
    pub hint_low: bool,
    pub hint: Option<String>,
    pub cursor_hint: Option<String>,
    pub link: Option<(u32, [f32; 3], [f32; 3])>,
    /// The parent's tag it rides (`linkto` with a tag; `tags`).
    pub link_tag: Option<String>,
    pub attached: Vec<(String, String)>,
    /// The parts Black Ops II hangs on a weapon's world model
    /// (`useweaponmodel`, a wall or box gun): magazine, scope, rocket, each
    /// placed at its offset from a tag of the gun.
    pub weapon_parts: Vec<(String, String, xmodel_runtime::TagOffset)>,
    pub loop_sound: Option<String>,
    pub can_damage: bool,
    /// Players a trigger (or model) is hidden from.
    pub invisible_to: BTreeSet<u32>,
    pub invisible_to_all: bool,
    /// `showtoplayer`: the players a model is drawn for (Kill Confirmed's
    /// tags: the enemy pair to one side, the friendly pair to the other).
    /// `None` is everyone; `hide` and `show` clear it.
    pub shown_to: Option<BTreeSet<u32>>,
    pub trigger_off: bool,
    /// bo2mp: the engine's dropped item (a gun, a scavenger bag) this is.
    pub item: Option<i32>,
    /// bo2mp: the destructible definition (`destructibledef`) a map entity
    /// breaks by, lower case.
    pub destructible: Option<String>,
    /// `setteamfortrigger` (the team, false) or `setexcludeteamfortrigger`
    /// (the team, true): which zombies a radius, box or use trigger answers
    /// (`triggers::actor_touches`).
    pub trigger_team: Option<(String, bool)>,
    /// `setmovingplatformenabled(1)`: players standing on it ride it
    /// (`riders`).
    pub platform: bool,
    /// Tags the scripts hid on its model (`hidepart`), lower case.
    pub hidden_parts: Vec<String>,
    /// A script model's animation (`setanim`): clip and when it started.
    pub anim: Option<(String, i64)>,
    /// Gone limp (`startragdoll`, thrown by `launchragdoll`): the clients
    /// drop the body under gravity.
    pub ragdoll: Option<xmodel_runtime::DObjRagdoll>,
    /// bo2zm round 1: its eye brightness (`mapshaderconstant` scriptVector2.y
    /// from zombie_eyes_clientfield_cb): 2 alive (Origins 3), 0.25 out.
    pub eye_luminance: Option<f32>,
}

#[derive(Clone, Debug)]
pub(crate) struct Player {
    pub obj: ObjRef,
    pub begun: bool,
    pub sessionstate: String,
    pub perks: BTreeSet<String>,
    /// The entity the player is linked to (`playerlinkto`): he goes where
    /// it goes.
    pub linked: Option<u32>,
    /// The knife the scripts gave him (named in a knife hit).
    pub melee_weapon: Option<String>,
    /// Down (last stand): he crawls until `reviveplayer` / `undolaststand`.
    pub laststand: bool,
    /// His weapon and weapon state last tick (weapon events).
    pub last_weapon: u32,
    pub last_wstate: i32,
    pub last_clip: i32,
    /// Fire held last tick, and when he raised the Zombie Shield (its
    /// plant is due).
    pub last_attack: bool,
    pub shield_raised: Option<i64>,
    /// His Jet Gun's spin and heat.
    pub jet: jetgun::JetState,
    /// `forcegrenadethrow`: let go of his grenade for him (t6::offhand).
    pub force_throw: bool,
    /// Whether he was diving / sprinting last tick (`dtp_start` /
    /// `dtp_end`, `sprint_begin` / `sprint_end`).
    pub last_dive: bool,
    pub last_sprint: bool,
    /// His scoreboard counts the engine keeps (headshots, downs, revives...).
    pub stats: BTreeMap<String, i32>,
    /// His HUD's client fields that are set (perks, power-ups, Mob's
    /// Afterlife lives and mana), in the order they were set.
    pub hud_fields: Vec<(String, f32)>,
    /// His points counters (`score_cf_damage`, ...: each hit and kill
    /// steps one; his HUD flies one popup per step).
    pub score_fields: Vec<(String, i32)>,
    /// A script camera (`camerasetposition` + `cameraactivate`): his view
    /// rides that entity (Nuketown's game-over rocket shot).
    pub camera: Option<u32>,
    pub camera_on: bool,
    /// bo2mp: a bot (`addtestclient`), and while its client has not come
    /// yet, when it was made.
    pub test_client: bool,
    pub pending_since: Option<i64>,
    /// bo2mp: the shellshock he is under (its name), when it ends (level
    /// ms) and the speed factor it put on `move_speed_scale_multiplier`
    /// (the shock file's `bg_shock_movement`; 1 when none).
    pub shock: Option<(String, i32)>,
    pub shock_factor: f32,
    /// His body model (`setmodel`; not drawn in third person yet): Mob's
    /// Afterlife copies it onto the body he leaves behind.
    pub model: String,
    /// `startpoisoning` / `stoppoisoning`: BO2's player-state poisoned flag
    /// (TranZit's Denizen grab and the fog's gas; the client draws it).
    pub poisoned: bool,
}

impl Player {
    pub(crate) fn new(obj: ObjRef) -> Self {
        Self {
            obj,
            begun: false,
            sessionstate: "spectator".to_owned(),
            perks: Default::default(),
            linked: None,
            melee_weapon: None,
            laststand: false,
            last_weapon: 0,
            last_wstate: 0,
            last_clip: 0,
            last_attack: false,
            shield_raised: None,
            jet: Default::default(),
            force_throw: false,
            last_dive: false,
            last_sprint: false,
            stats: Default::default(),
            hud_fields: Vec::new(),
            score_fields: Vec::new(),
            camera: None,
            camera_on: false,
            test_client: false,
            pending_since: None,
            shock: None,
            shock_factor: 1.0,
            model: String::new(),
            poisoned: false,
        }
    }
}

/// Engine state for the scripts.
#[derive(Resource, Default)]
pub(crate) struct Zm {
    pub map: String,
    pub ents: BTreeMap<u32, Ent>,
    pub next_entnum: u32,
    pub players: BTreeMap<u32, Player>,
    pub tables: Arc<BTreeMap<String, T6Table>>,
    pub fx: Vec<String>,
    /// Client systems by id (`clientsysregister`): "musicCmd", ...
    pub client_sys: Vec<String>,
    /// `spawnfx` effects: name, origin, forward, up (when given).
    pub fx_ents: BTreeMap<u32, (String, [f32; 3], [f32; 3], Option<[f32; 3]>)>,
    pub unbound: BTreeMap<String, u64>,
    pub started: bool,
    pub movers: movers::Movers,
    /// Notifies to deliver later: (server ms, object, name).
    pub timers: Vec<(i64, ObjRef, String)>,
    /// bo2mp: menu answers to deliver later as the player's `menuresponse`
    /// (server ms, player, menu, response).
    pub menu_replies: Vec<(i64, ObjRef, String, String)>,
    /// Script movers showing entities' models.
    pub presences: presence::Presences,
    /// The map's AI navigation graph.
    pub nav: nav::Nav,
    /// Brush entities' collision as last sent.
    pub brush_rows: BTreeMap<u32, brushes::BrushRow>,
    /// Path node script objects, by node index.
    pub node_objs: Vec<ObjRef>,
    /// Each path node object's node (the bot scripts name nodes by them).
    pub node_of: std::collections::HashMap<ObjRef, u32>,
    /// Animations by lower-case name.
    pub anims: std::collections::HashMap<String, T6Anim>,
    pub clips: std::collections::HashMap<String, Arc<xmodel_runtime::AnimClip>>,
    pub strings: std::collections::HashMap<String, String>,
    pub sound_aliases: BTreeSet<String>,
    /// bo2mp: the shellshocks the scripts started (and precached), in order (the index
    /// the player state carries is the position + 1).
    pub shock_names: Vec<String>,
    /// bo2mp: a multiplayer map (game type in `maps/mp/gametypes/`).
    pub mp: bool,
    /// bo2mp: a ranked match (BO2's `level.rankedmatch`: XP counts, the
    /// public class set; Combat Training), else a custom game.
    pub ranked_match: bool,
    /// bo2mp: the game type's settings (`getgametypesetting`).
    pub gamesettings: BTreeMap<String, String>,
    /// bo2mp: team scores (`setteamscore`), the match's end time on the
    /// server clock (`setgameendtime`, 0 = no clock), the minimap's centre.
    pub team_scores: BTreeMap<String, i32>,
    pub sound_lengths: std::collections::HashMap<String, u32>,
    /// bo2mp: retrievable weapons (getretrievableweapons).
    pub retrievable_weapons: Vec<String>,
    /// bo2mp: bots (the engine half of BO2's bot scripts), the bot clients
    /// asked for and not yet handed to the bot roster, bots to remove,
    /// bots throwing a grenade this tick, their difficulty (0 easy .. 3).
    pub bots: BTreeMap<u32, bots::Bot>,
    pub test_client_requests: Vec<u32>,
    pub bot_leaves: Vec<u32>,
    pub bot_throws: Vec<u32>,
    pub bot_difficulty: i32,
    /// bo2mp: path nodes marked dangerous (`setdangerous`): by node, a
    /// count per team (allies, axis), 0..15 like BO2's.
    pub node_danger: BTreeMap<u32, [u8; 2]>,
    /// bo2mp: items picked up last tick (engine records, see `items`).
    pub item_pickups: Vec<crate::ItemPickupRecord>,
    pub game_end_time: i32,
    pub map_center: [f32; 3],
    /// Barrier types by lower-case name.
    pub zbarrier_defs: std::collections::HashMap<String, Arc<T6ZBarrierDef>>,
    /// Script HUD elements (Game Over, Max Ammo...).
    pub huds: hud::Huds,
    /// Flying limbs and when they go.
    pub gibs: Vec<(u32, i64)>,
    /// Client-field effects waiting their time.
    pub client_fx: clientfields::ClientFx,
    /// Script vehicles on their paths (Nuketown's perk arrival).
    pub vehicles: vehicles::Vehicles,
    /// Players standing on moving platforms (Tranzit's bus).
    pub riders: riders::Riders,
    /// Wall buys' chalk and bought guns (BO2's client-script visuals).
    pub wallbuys: wallbuys::WallBuys,
    pub wallbuy_since: Option<i64>,
    /// The debris piles' path cuts are made (`brushes::cut_debris_paths`).
    pub debris_paths_cut: bool,
    /// What each player's HUD was last sent (round, hint, offhand,
    /// action slots).
    pub hud_sent: BTreeMap<u32, (String, String, String, String, String)>,
    /// bo2mp: mp/statstable.csv's items, read once.
    pub items: loadout::ItemCache,
    /// bo2zm M4: each player's d-pad slots (`setactionslot(slot, "weapon",
    /// name)`): (slot, weapon or kind).
    pub action_slots: BTreeMap<u32, Vec<(i32, String)>>,
    /// The test player's fire button, pressed every other tick.
    pub autoplay_fire: bool,
    /// When every expected player of a party was first playing (the test
    /// aids' party clock, `autoplay::party_clock`).
    pub party_in_at: Option<i64>,
    /// The round the scripts last reported (`setroundsplayed`).
    pub rounds_played: i32,
    /// Animation state definitions by file stem (`zm_nuked_basic`).
    pub asds: std::collections::HashMap<String, asd::AnimStateDef>,
    /// Actors (zombies) by entity number.
    pub actors: BTreeMap<u32, actors::Actor>,
    /// Server time of the last scheduler run.
    pub now_ms: i64,
    pub next_thread_dump: i64,
    /// Players holding use (for the press edge).
    pub use_held: BTreeSet<u32>,
    /// The hint of the use trigger each player faces.
    pub hints: BTreeMap<u32, String>,
    /// bo2mp: the destructible definitions by lower-case name.
    pub destructibles: std::collections::HashMap<String, Arc<xmodel_runtime::T5DestructibleDef>>,
    /// bo2mp: a round restart the scripts asked for (`map_restart`), run at
    /// the next frame's start: whether the players' `pers[]` carry over.
    pub pending_restart: Option<bool>,
    /// bo2mp: BO2's 32 objectives (`objective_add`), reset with the level.
    pub objectives: [objectives::Objective; objectives::MAX],
    /// bo2mp: players coming back from a round restart: their `pers[]` and
    /// whether each is a bot.
    pub rejoin: BTreeMap<u32, (Value, bool)>,
    /// The map's entities (their text), spawned again at a round restart.
    pub map_entities: Vec<String>,
    /// bo2mp: the map's entities by their place in its text (see
    /// `map_natives`), filled as they spawn.
    pub map_natives: BTreeMap<u32, map_natives::MapNative>,
    /// bo2mp: what `map_natives` last did to each one (kept over a round
    /// restart: the session's rows are not put back by one).
    pub map_natives_applied: BTreeMap<u32, map_natives::Applied>,
    /// bo2mp: the summary of `map_natives` was logged.
    pub map_natives_reported: bool,
    /// The map's moving props (washing lines, shutters, wires, dust devils).
    pub fxprops: fxanims::FxProps,
    /// Mob's wolf heads (the client script's half of feeding them).
    pub wolves: prison_wolves::Wolves,
    /// The map's sound triggers (bumping a car, the truck bed).
    pub audio_trigs: audio_triggers::AudioTrigs,
    /// Players' thrown grenades the scripts watch (the Monkey Bomb).
    pub thrown: Vec<grenades::Thrown>,
    /// Gone-off grenades freed a tick later.
    pub thrown_free: Vec<(ObjRef, i64)>,
    /// Detonators fired since last tick (client, weapon): `detonate`.
    pub detonated: Vec<(u32, u32)>,
    /// Clients whose gun fired a shot since last tick: `weapon_fired`.
    pub shots: Vec<u32>,
    /// Where tracked shots went off this tick (their `projectile_impact`).
    pub detonations: Vec<(crate::ProjectileId, [f32; 3])>,
}

/// Entity numbers: players are their client number; the rest start here.
pub(crate) const FIRST_ENTNUM: u32 = 64;

impl Zm {
    pub(crate) fn alloc_entnum(&mut self) -> u32 {
        if self.next_entnum < FIRST_ENTNUM {
            self.next_entnum = FIRST_ENTNUM;
        }
        let n = self.next_entnum;
        self.next_entnum += 1;
        n
    }
}

/// Install a BO2 map's scripts (before [`start`]).
/// bo2zm: Black Ops II's hit locations by its models' part classes (T6
/// added torso_mid at 5, so from there on IW4's names were one off: a lower
/// torso hit reached the scripts as right_arm_upper, a left foot as gun).
pub(crate) const T6_HITLOC_NAMES: [&str; 21] = [
    "none",
    "helmet",
    "head",
    "neck",
    "torso_upper",
    "torso_mid",
    "torso_lower",
    "right_arm_upper",
    "left_arm_upper",
    "right_arm_lower",
    "left_arm_lower",
    "right_hand",
    "left_hand",
    "right_leg_upper",
    "left_leg_upper",
    "right_leg_lower",
    "left_leg_lower",
    "right_foot",
    "left_foot",
    "gun",
    "riotshield",
];

/// bo2zm: his profile's choices the scripts and the engine read (BO2's
/// Settings menu sets them): mature content (`getlocalprofileint
/// ("cg_mature")`: heads and limbs come off) and how many corpses stay
/// (`ai_corpseCount`: past it the oldest goes).
static MATURE: std::sync::atomic::AtomicBool = std::sync::atomic::AtomicBool::new(true);
static CORPSES: std::sync::atomic::AtomicU8 = std::sync::atomic::AtomicU8::new(16);

pub use natives_game::set_t6_game_settings;
pub use cheats::{Cheat, cheat_on, cheat_pick, cheats_allowed};
pub use highrise::platform_rides_on_ground;
pub use transit::platform_ride_box;

pub fn set_t6_profile(mature: bool, corpses: u8) {
    MATURE.store(mature, std::sync::atomic::Ordering::Relaxed);
    CORPSES.store(corpses.max(1), std::sync::atomic::Ordering::Relaxed);
}

pub(crate) fn profile_mature() -> bool {
    MATURE.load(std::sync::atomic::Ordering::Relaxed)
}

pub(crate) fn corpse_limit() -> usize {
    usize::from(CORPSES.load(std::sync::atomic::Ordering::Relaxed))
}

/// The party a Public Match lobby started with (0 = none): wins over
/// IW4L_T6_PLAYERS.
static PARTY: std::sync::atomic::AtomicI32 = std::sync::atomic::AtomicI32::new(0);

/// bo2mp: the bots Custom Games adds to the next Zombies game (0-3).
static ZM_BOTS: std::sync::atomic::AtomicI32 = std::sync::atomic::AtomicI32::new(0);

/// bo2mp: Custom Games' bots for the next Zombies game.
pub fn set_zm_bots(n: u32) {
    ZM_BOTS.store(n.min(3) as i32, std::sync::atomic::Ordering::Relaxed);
}

/// bo2mp: the bots the Zombies game being started has.
pub(crate) fn zm_bots() -> i32 {
    ZM_BOTS.load(std::sync::atomic::Ordering::Relaxed)
}

/// bo2mp: Multiplayer's custom guns (other games' guns) in the next Zombies
/// game's Mystery Box (Custom Games' CUSTOM WEAPONS).
static ZM_GUEST_GUNS: std::sync::atomic::AtomicBool = std::sync::atomic::AtomicBool::new(true);

/// bo2mp: whether the next Zombies game's Mystery Box has the custom guns.
pub fn set_zm_guest_guns(on: bool) {
    ZM_GUEST_GUNS.store(on, std::sync::atomic::Ordering::Relaxed);
}

/// bo2mp: the Zombies game being started has the custom guns in its box
/// (`BO2MP_ZM_CUSTOM_WEAPONS=0|1` overrides the lobby, a test aid).
pub(crate) fn zm_guest_guns() -> bool {
    match std::env::var("BO2MP_ZM_CUSTOM_WEAPONS") {
        Ok(v) => v.trim() != "0",
        Err(_) => ZM_GUEST_GUNS.load(std::sync::atomic::Ordering::Relaxed),
    }
}

/// bo2mp: the custom guns this map's Mystery Box can give, as
/// `bo2mp/zm_guest_guns` reads them (`name:vox:upgrade`, space-separated):
/// the loaded guest guns a player can be given, each family once and none
/// of a family the map's own Zombies weapons have
/// (`bo2_profile::guest::zm_box_guns`), each with its Pack-a-Punch upgrade
/// (`bo2_profile::guest::zm_upgrade_of`, made by the match load; empty when
/// it has none loaded).
fn zm_box_guest_list(world: &mut World) -> String {
    let f = frame(world);
    let names = f.weapon_script_names();
    let zm: std::collections::HashSet<String> = names
        .iter()
        .filter(|n| {
            let n = n.split('+').next().unwrap_or(n);
            // (The guest guns' own upgrades, `bo1_ak74u_upgraded_zm_mp`,
            // are no family of the map's.)
            !bo2_profile::guest::is_guest_name(n) && n.strip_suffix("_mp").unwrap_or(n).ends_with("_zm")
        })
        .map(|n| bo2_profile::guest::family_of_zm_weapon(n))
        .collect();
    let guns = bo2_profile::guest::zm_box_guns(&zm, |g| {
        f.weapon_index_by_script_name(&bo2_profile::guest::script_name(g))
            .is_some_and(|w| w != 0 && f.combat_facts_for(w).is_some())
    });
    let loaded = |name: &str| f.weapon_index_by_script_name(name).is_some_and(|w| w != 0 && f.combat_facts_for(w).is_some());
    guns.iter()
        .map(|g| {
            let name = bo2_profile::guest::script_name(g);
            let upgrade = bo2_profile::guest::zm_upgrade_of(&name).map(|u| u.upgraded).filter(|u| loaded(u)).unwrap_or_default();
            format!("{name}:{}:{upgrade}", bo2_profile::guest::zm_vox(g))
        })
        .collect::<Vec<_>>()
        .join(" ")
}

/// bo2zm H28: the players in the lobby that starts the next game (the host
/// and those who joined it), 1-4; 0 = no lobby (his own game).
pub fn set_t6_party_size(n: usize) {
    PARTY.store(n.min(4) as i32, std::sync::atomic::Ordering::Relaxed);
}

/// How many players the host's party is (IW4L_T6_PLAYERS, 1-4, default 1):
/// BO2 waits for them all before the first round, and a party of one is a
/// solo game.
pub(crate) fn expected_players() -> i32 {
    let party = PARTY.load(std::sync::atomic::Ordering::Relaxed);
    if party > 0 {
        return party;
    }
    static N: std::sync::OnceLock<i32> = std::sync::OnceLock::new();
    *N.get_or_init(|| {
        std::env::var("IW4L_T6_PLAYERS")
            .ok()
            .and_then(|v| v.trim().parse::<i32>().ok())
            .unwrap_or(1)
            .clamp(1, 4)
    })
}

/// Every engine builtin, bound on a linked program (`mp`: multiplayer's too).
pub(crate) fn bind_natives(vm: &mut Vm<World>, mp: bool) {
    gsc_t6::natives::bind_core(vm);
    natives_game::bind(vm);
    natives_ent::bind(vm);
    natives_player::bind(vm);
    if !mp {
        globallogic::bind(vm);
    }
    // Sounds and effects replace the quiet stubs bound above.
    natives_fx::bind(vm);
    natives_ai::bind(vm);
    zbarrier::bind(vm);
    darts::bind(vm);
    clientfields::bind(vm);
    brushes::bind(vm);
    vehicles::bind(vm);
    turrets::bind(vm);
    riotshield::bind(vm);
    jetgun::bind(vm);
    offhand::bind(vm);
    limits::bind(vm);
    stats::bind(vm);
    hud::bind(vm);
    if mp {
        influencers::bind(vm);
        natives_mp::bind(vm);
        bots::bind(vm);
        killstreaks::bind(vm);
        loadout::bind(vm);
        objectives::bind(vm);
        items::bind(vm);
        // bo2mp lane F: the body's animation natives (the death path).
        playeranim::bind(vm);
        // bo2mp: Plutonium's additions his trickshot script calls, and the
        // engine builtins it shares with BO2's scripts (suicide, kick...).
        trickshot::bind(vm);
    }
}

pub(crate) fn install(world: &mut World, inst: T6Install) -> Result<(), String> {
    cheats::reset();
    let mp = mp::is_mp(&inst.map);
    let mut objects = Vec::with_capacity(inst.objects.len() + 1);
    for bytes in &inst.objects {
        objects.push(ScriptObject::parse(bytes)?);
    }
    // bo2mp: the scripts the engine provides (multiplayer's _globallogic).
    if mp {
        objects.extend(mp::engine_scripts()?);
    } else {
        // bo2mp: Custom Games' Zombies bots.
        objects.push(gsc_t6::compile("bo2mp/zm_bots", mp::ZM_BOTS)?);
        // bo2mp: Multiplayer's custom guns in the Mystery Box.
        objects.push(gsc_t6::compile("bo2mp/zm_guest_guns", mp::ZM_GUEST_GUNS)?);
        // bo2mp: a Zombies map from another game: our map script.
        if let Some(script) = foreign_zm::map_script(&inst.map, &objects)? {
            objects.push(script);
        }
    }
    let mut strings = Strings::default();
    // bo2mp: Black Ops II's linker binds a bare call to the engine's own
    // function when the engine has one, before the script's or its
    // includes'. These are the multiplayer names a script also defines
    // (_spawning's clearspawnpoints() must not run _spawnlogic's, which
    // empties the spawn lists).
    const ENGINE_FIRST: &[(&str, bool)] = &[
        ("addspawnpoints", false),
        ("clearspawnpoints", false),
        ("vectorcross", false),
        ("suicide", true),
    ];
    let program = Program::link_with(objects, &mut strings, if mp { ENGINE_FIRST } else { &[] })?;
    let functions = program.functions.len();
    let builtins = program.builtins.len();
    let mut vm: Vm<World> = Vm::new(program, strings);
    // A new game rolls new dice (which perk falls next, box guns, drops): the
    // scripts' random numbers start from the clock. IW4L_T6_SEED=<n> fixes
    // them for a test that must repeat.
    vm.rng = std::env::var("IW4L_T6_SEED")
        .ok()
        .and_then(|s| s.trim().parse::<u64>().ok())
        .unwrap_or_else(|| {
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .map_or(1, |d| d.as_nanos() as u64)
        })
        .wrapping_mul(0x9e37_79b9_7f4a_7c15)
        | 1;
    bind_natives(&mut vm, mp);
    // IW4L_T6_TRACE=name,name: log those builtins' calls (debugging).
    if let Ok(names) = std::env::var("IW4L_T6_TRACE") {
        vm.trace_builtins(&names, 5000);
    }
    // IW4L_T6_FTRACE=name,name: log those script functions' calls.
    if let Ok(names) = std::env::var("IW4L_T6_FTRACE") {
        vm.trace_functions(&names);
    }
    vm.hooks = Hooks {
        get_field: Some(fields::get),
        set_field: Some(fields::set),
        unbound: Some(unbound),
    };
    // bo2mp: a multiplayer match: 18 players (a full lobby), no online
    // services, ranked play off (a local match, as Combat Training).
    // BO2MP_BOTS=<friends>,<enemies>[,<difficulty 0-3>]: bots, as BO2's
    // bot setup screen sets them (a test aid until the lobby does).
    // A script thread that runs over 250 ms names itself (a freeze he
    // would feel); IW4L_T6_PROF=1: over 20 ms.
    vm.slow_thread_ms = if prof_on() { 20.0 } else { 250.0 };
    // Profiling: which scripts and builtins the ticks spend on.
    if prof_on() {
        vm.profile = Some(gsc_t6::vm::ScriptProfile::default());
    }
    let mut bot_difficulty = 1;
    let lobby = inst.bots.map(|(f, e, d)| format!("{f},{e},{d}"));
    if mp && let Some(spec) = std::env::var("BO2MP_BOTS").ok().or(lobby) {
        let v: Vec<i32> = spec.split(',').filter_map(|s| s.trim().parse().ok()).collect();
        let (friends, enemies) = (v.first().copied().unwrap_or(0), v.get(1).copied().unwrap_or(0));
        bot_difficulty = v.get(2).copied().unwrap_or(1);
        for (k, n) in [("bot_friends", friends), ("bot_enemies", enemies), ("bot_difficulty", bot_difficulty)] {
            vm.dvars.insert(k.to_owned(), n.to_string());
        }
    }
    if mp {
        for (k, v) in [
            ("sv_maxclients", "18"),
            ("party_maxplayers", "18"),
            ("xblive_wagermatch", "0"),
            ("xblive_basictraining", "0"),
            ("xblive_privatematch", "1"),
            ("scr_xpscale", "1"),
        ] {
            vm.dvars.insert(k.to_owned(), v.to_owned());
        }
        // bo2mp: the optional M.O.A.B. (`bo2mp/moab`): the lobby's choice,
        // or BO2MP_MOAB=<kills>; 0 keeps Black Ops II as it is.
        let moab = std::env::var("BO2MP_MOAB").ok().and_then(|v| v.trim().parse::<u32>().ok()).unwrap_or_else(mp::moab_kills);
        if moab > 0 {
            vm.dvars.insert("scr_moab".to_owned(), "1".to_owned());
            vm.dvars.insert("scr_moab_kills".to_owned(), moab.to_string());
        }
        // A new match plays at normal speed.
        slowmo::reset();
        // bo2mp: TRICKSHOT MODE (his script is linked only when on).
        let ts = if trickshot::enabled() { "1" } else { "0" };
        vm.dvars.insert("bo2mp_trickshot".to_owned(), ts.to_owned());
        // bo2mp: Resurgence (`bo2mp/resurgence`): the lobby's WARZONE, or
        // BO2MP_BR=<squad size> with the free-for-all.
        if let Some(rules) = br::rules() {
            vm.dvars.extend(br::dvars(&rules));
        }
        // bo2mp: BO2's own HUD asked for (`BO2_HUD=bo2`): its prematch
        // countdown shows instead of the modern HUD's.
        let bo2_hud = std::env::var("BO2_HUD").is_ok_and(|v| v.trim().eq_ignore_ascii_case("bo2"));
        vm.dvars.insert("bo2mp_bo2hud".to_owned(), if bo2_hud { "1" } else { "0" }.to_owned());
        // Combat Training (ranked, lane C): BO2's public basic-training
        // match. BO2's ranked bot branch (_bot.gsc: one bot per human
        // player, "comp stomp") stays off (sv_botsoak, its own switch): the
        // stand-in globallogic fills the teams with the playlist's bots
        // (`combattrainingbots`).
        if stats::ranked() {
            for (k, v) in [("xblive_basictraining", "1"), ("xblive_privatematch", "0"), ("sv_botsoak", "1")] {
                vm.dvars.insert(k.to_owned(), v.to_owned());
            }
        }
        // The engine's perk dvars at their registered defaults. The scripts
        // read them (Hardline: a kill's momentum x perk_killstreakMomentumMultiplier
        // in giveplayermomentumnotification); unset they read 0, so Hardline
        // gave 0 momentum a kill and no scorestreak was ever earned.
        for (k, v) in [
            ("perk_weapswitchmultiplier", "0.5"),
            ("perk_weapspreadmultiplier", "0.65"),
            ("perk_weapreloadmultiplier", "0.5"),
            ("perk_weapratemultiplier", "0.75"),
            ("perk_weapadsmultiplier", "0.5"),
            ("perk_weapmeleemultiplier", "0.5"),
            ("perk_weaptossmultiplier", "0.5"),
            ("perk_weapequipmentusemultiplier", "0.6"),
            ("perk_extrabreath", "5"),
            ("perk_bulletpenetrationmultiplier", "2"),
            ("perk_sprintmultiplier", "2"),
            ("perk_grenadetossbacktimer", "2500"),
            ("perk_killstreakreduction", "1"),
            ("perk_killstreakmomentummultiplier", "1.2"),
            ("perk_killstreakdeathpenaltymultiplier", "0.5"),
            ("perk_delayexplosivetime", "0.9"),
            ("perk_disarmexplosivetime", "2"),
            ("perk_speedmultiplier", "1.07"),
            ("perk_deathstreakcountrequired", "3"),
            ("perk_sprintrecoverymultiplier", "0.3"),
            ("perk_healthregenmultiplier", "1.5"),
            ("perk_damagekickreduction", "0.25"),
            ("perk_shellshockreduction", "0.5"),
            ("perk_mantlereduction", "0.4"),
            ("perk_interactspeedmultiplier", "1.15"),
            ("perk_throwbackinnerradius", "120"),
            ("perk_flakjacket", "35"),
            ("perk_fireproof", "95"),
            ("perk_armorpiercing", "95"),
            ("perk_dogsattackghost", "1"),
            ("perk_blackbirdshowsgpsjammer", "1"),
            ("perk_fastladderclimbmultiplier", "1.75"),
            ("perk_marksmanenemynamefadein", "5"),
            ("perk_marksmanenemynamefadeout", "5"),
            ("perk_nottargetedbyai_min_speed", "0"),
            ("perk_nottargetedbyai_graceperiod", "2000"),
        ] {
            vm.dvars.entry(k.to_owned()).or_insert_with(|| v.to_owned());
        }
    }
    for (k, v) in [
        ("mapname", inst.map.as_str()),
        ("g_gametype", inst.gametype.as_str()),
        ("ui_gametype", inst.gametype.as_str()),
        ("ui_zm_gamemodegroup", zm_gamemodegroup(&inst.gametype)),
        (
            "ui_zm_mapstartlocation",
            if inst.location.is_empty() {
                location(&inst.map)
            } else {
                inst.location.as_str()
            },
        ),
        ("zm_gamemodegroup", zm_gamemodegroup(&inst.gametype)),
        ("sv_maxclients", if mp { "18" } else { "4" }),
        ("party_maxplayers", if mp { "18" } else { "4" }),
        ("onlinegame", "0"),
        ("systemlink", "0"),
        ("splitscreen", "0"),
        ("xblive_privatematch", "0"),
        ("developer", "0"),
        ("developer_script", "0"),
        ("zombie_cheat", "0"),
        ("sv_cheats", "0"),
        ("g_gameskill", "1"),
        ("scr_zm_enable_bots", "0"),
        // The box may move (teddy bear): the magic box reads it and no
        // script or config of his sets it, so the engine's own default.
        ("magic_chest_movable", "1"),
        // Client scripts own the map's placed effects and exploders (our
        // client draws the placed effects itself).
        ("cg_usingclientscripts", "1"),
        // The engine's default gravity: Ascension's Matryoshka dolls divide
        // by it at init (their launch peak time).
        ("bg_gravity", "800"),
    ] {
        vm.dvars.insert(k.to_owned(), v.to_owned());
    }
    // bo2mp: Custom Games' Zombies bots (`bo2mp/zm_bots`).
    if !mp {
        vm.dvars.insert("bo2mp_zm_bots".to_owned(), zm_bots().to_string());
        // bo2mp: Multiplayer's custom guns in the Mystery Box
        // (`bo2mp/zm_guest_guns`): on or off, and which.
        let on = zm_guest_guns();
        let list = if on { zm_box_guest_list(world) } else { String::new() };
        diag::info!(
            Sim,
            "bo2mp zombies custom guns: {} ({} guns): {list}",
            if on { "in the box" } else { "off" },
            list.split_whitespace().count()
        );
        vm.dvars.insert("bo2mp_zm_guest_guns".to_owned(), if on { "1" } else { "0" }.to_owned());
        vm.dvars.insert("bo2mp_zm_guest_list".to_owned(), list);
    }
    diag::info!(
        Sim,
        "bo2zm t6 scripts: {} objects, {functions} functions, {builtins} builtins, {} unbound",
        inst.objects.len(),
        vm.unbound_builtins().len()
    );
    // IW4L_T6_UNBOUND=1: every builtin the scripts name that has no native.
    if std::env::var("IW4L_T6_UNBOUND").is_ok() {
        let mut names: Vec<String> = vm
            .unbound_builtins()
            .into_iter()
            .map(|(n, m)| if m { format!(".{n}") } else { n })
            .collect();
        names.sort();
        diag::info!(Sim, "bo2zm t6 unbound: {}", names.join(" "));
    }
    // Path nodes are stored where the designer placed them (Nuketown's
    // town grid floats at z 0, the ground is near -60); BO2 drops them to
    // the floor. Ours did not: every "can it walk straight there" sweep ran
    // 60 units up, over low walls, fences and cars, and zombies took those
    // routes into them and stood there (M4 retest 1).
    let mut nodes = inst.path_nodes.clone();
    actors::drop_nodes_to_floor(world, &mut nodes);
    let node_objs = node_objects(&mut vm, &nodes);
    let baked = nav::Baked::new(&inst.path_tables, nodes.len());
    let linked: Vec<(u32, u32)> = nodes
        .iter()
        .enumerate()
        .flat_map(|(i, n)| n.links.iter().map(move |l| (i as u32, u32::from(l.0))))
        .collect();
    let seen = linked.iter().filter(|&&(i, j)| baked.visible(i, j) == Some(true)).count();
    let smooth = linked.iter().filter(|&&(i, j)| baked.smooth(i, j)).count();
    diag::info!(
        Sim,
        "bo2mp path tables: {} nodes ({} compiled), sight table {} bytes (side {}), smooth table {} bytes (side {}); of {} links, {} see each other, {} walk straight",
        inst.path_tables.node_count,
        inst.path_tables.original_count,
        inst.path_tables.vis.len(),
        baked.sides().0,
        inst.path_tables.smooth.len(),
        baked.sides().1,
        linked.len(),
        seen,
        smooth,
    );
    let anims: std::collections::HashMap<String, T6Anim> = inst
        .anims
        .into_iter()
        .map(|a| (a.name.to_ascii_lowercase(), a))
        .collect();
    let asds = inst
        .animstatedefs
        .iter()
        .map(|(name, text)| {
            let stem = name
                .rsplit('/')
                .next()
                .unwrap_or(name)
                .trim_end_matches(".asd")
                .to_ascii_lowercase();
            (stem, asd::AnimStateDef::parse(text))
        })
        .collect::<std::collections::HashMap<String, asd::AnimStateDef>>();
    // IW4L_T6_ASDLOG=<part of a def name>: each state of those animstatedefs,
    // its notify and animations (test aid).
    if let Ok(want) = std::env::var("IW4L_T6_ASDLOG") {
        for (stem, def) in asds.iter().filter(|(k, _)| k.contains(want.as_str())) {
            let mut names: Vec<&String> = def.states.keys().collect();
            names.sort();
            for k in names {
                let st = &def.states[k];
                diag::info!(
                    Sim,
                    "bo2zm t6 asd {stem} {k}: notify {} flags {:?} anims {:?}",
                    st.notify,
                    st.flags,
                    st.substates
                );
            }
        }
    }
    // IW4L_T6_ANIM=<part of a name>: those animations' timing and root motion.
    if let Ok(want) = std::env::var("IW4L_T6_ANIM") {
        let mut names: Vec<&String> = anims.keys().filter(|k| k.contains(want.as_str())).collect();
        names.sort();
        for k in names.into_iter().take(40) {
            let a = &anims[k];
            diag::info!(
                Sim,
                "bo2zm t6 anim {k}: {} frames at {} fps, looping {}, {} root keys {:?} .. {:?}, speed {:.1}, notes {:?}",
                a.numframes,
                a.framerate,
                a.looping,
                a.delta_trans.len(),
                a.delta_trans.first(),
                a.delta_trans.last(),
                actors::anim_speed(a),
                a.notifies
            );
        }
    }
    diag::info!(
        Sim,
        "bo2zm t6: {} path nodes, {} animations, {} animstatedefs",
        nodes.len(),
        anims.len(),
        inst.animstatedefs.len()
    );
    diag::info!(
        Sim,
        "bo2zm t6: {} models with hit boxes, {} actor clips",
        inst.models
            .iter()
            .filter(|(_, c)| c.bone_collision.iter().any(Option::is_some))
            .count(),
        inst.clips.len()
    );
    frame(world).add_model_capabilities(inst.models);
    world.insert_resource(Zm {
        map: inst.map.clone(),
        clips: inst.clips.into_iter().collect(),
        // Keys upper-cased: BO2 looks them up without case (the B23R's
        // chalk asks for "ZOMBIE_WEAPON_BERETTA93r", fix list 3).
        strings: inst
            .strings
            .into_iter()
            .map(|(k, v)| (k.to_ascii_uppercase(), v))
            .collect(),
        sound_aliases: inst.sound_aliases,
        mp,
        ranked_match: mp && stats::ranked(),
        bot_difficulty,
        sound_lengths: inst.sound_lengths,
        retrievable_weapons: inst.retrievable_weapons,
        gamesettings: inst.gamesettings,
        // bo2mp: the rank tables carried on to the combined levels and
        // prestiges (bo2_profile's progression) before a script reads them.
        tables: Arc::new(
            inst.tables
                .into_iter()
                .map(|(name, mut t)| {
                    if bo2_profile::progression::extend_table(&name, &mut t.columns, &mut t.rows, &mut t.cells) {
                        diag::info!(Sim, "bo2mp progression: {name} extended to {} rows", t.rows);
                    }
                    (name, t)
                })
                .collect(),
        ),
        zbarrier_defs: inst
            .zbarrier_defs
            .into_iter()
            .map(|d| (d.name.to_ascii_lowercase(), Arc::new(d)))
            .collect(),
        next_entnum: FIRST_ENTNUM,
        nav: nav::Nav::new(nodes).with_baked(baked),
        node_of: node_objs.iter().enumerate().map(|(i, o)| (*o, i as u32)).collect(),
        node_objs,
        anims,
        asds,
        destructibles: inst
            .destructibles
            .into_iter()
            .map(|d| (d.name.to_ascii_lowercase(), d))
            .collect(),
        ..Default::default()
    });
    killstreaks::install(world, inst.vehicles);
    world.insert_resource(T6Runtime { vm, omni: crate::step::omni_movement_setting() });
    world.insert_resource(playeranim::Bodies::new(inst.playeranim));
    world.insert_resource(PendingEntities(inst.entities));
    Ok(())
}

/// The path nodes' script objects, by node index.
fn node_objects(vm: &mut Vm<World>, nodes: &[T6PathNode]) -> Vec<ObjRef> {
    let mut node_objs = Vec::with_capacity(nodes.len());
    for n in nodes {
        let o = vm.alloc_object(ObjKind::Struct);
        let kind = nav::NODE_TYPE_NAMES.get(n.ty as usize).copied().unwrap_or("Path");
        let fields: [(&str, Value); 4] = [
            ("origin", Value::Vec3(n.origin)),
            ("angles", Value::Vec3([0.0, n.angle, 0.0])),
            ("spawnflags", Value::Int(n.spawnflags as i32)),
            ("radius", Value::Float(n.radius)),
        ];
        for (k, v) in fields {
            let f = vm.intern(k);
            vm.set_raw_field(o, f, v);
        }
        for (k, v) in [
            ("type", kind),
            ("targetname", n.targetname.as_str()),
            ("target", n.target.as_str()),
            ("script_noteworthy", n.script_noteworthy.as_str()),
            ("script_linkname", n.script_linkname.as_str()),
            ("animscript", n.animscript.as_str()),
        ] {
            if !v.is_empty() {
                let f = vm.intern(k);
                let s = vm.string(v);
                vm.set_raw_field(o, f, s);
            }
        }
        node_objs.push(o);
    }
    node_objs
}

#[derive(Resource)]
struct PendingEntities(Vec<String>);

/// The zombies game type a map starts in from Solo: Nuketown is
/// `zstandard` only; the other Black Ops II maps play their full mode,
/// `zclassic` (each map's `gamemode_callback_setup` registers it).
pub fn zm_gametype(map: &str) -> &'static str {
    match map {
        "zm_nuked" => "zstandard",
        _ => "zclassic",
    }
}

/// The game mode group of a game type: `_zm_utility::is_classic` reads
/// `ui_zm_gamemodegroup`, so a `zclassic` map must say `zclassic`; and
/// `is_encounter` wants `zencounter` (Grief and Turned: without it no
/// player gets a team, and Grief says "another chance" at the start).
fn zm_gamemodegroup(gametype: &str) -> &'static str {
    match gametype {
        "zclassic" => "zclassic",
        "zgrief" | "zcleansed" | "zmeat" => "zencounter",
        _ => "zsurvival",
    }
}

fn location(map: &str) -> &'static str {
    match map {
        "zm_nuked" => "nuked",
        "zm_transit" => "transit",
        "zm_highrise" => "rooftop",
        "zm_prison" => "prison",
        "zm_buried" => "processing",
        "zm_tomb" => "tomb",
        // Black Ops 1 maps: the map's own default_start_location (its Mule Kick
        // is "zclassic_perks_default"; zzz_zm_location.gsc).
        "zm_prototype" => "default",
        _ => "",
    }
}

/// bo2mp: Black Ops II's killcam (`_killcam.gsc`) puts a dead player in
/// spectator with the fields the engine replays from: whom to watch
/// (`spectatorclient`), the entity that killed (`killcamentity`), how far
/// back (`archivetime`), the time offset and the length. The same seats
/// MW2's scripts fill, so the engine's own killcam replay plays it.
pub(crate) fn killcam_seats(world: &World) -> Vec<(ClientId, crate::ScriptSeat)> {
    let (Some(zm), Some(rt)) = (world.get_resource::<Zm>(), world.get_resource::<T6Runtime>()) else {
        return Vec::new();
    };
    if !zm.mp {
        return Vec::new();
    }
    let vm = &rt.vm;
    let field = |o: ObjRef, name: &str| -> Option<Value> {
        let f = vm.strings.find(name)?;
        Some(vm.raw_field(o, f))
    };
    let num = |o: ObjRef, name: &str| field(o, name).and_then(|v| v.as_float());
    // BO2's final replay (_killcam.gsc dofinalkillcam) seats every player,
    // the living too: the match end freezes them, it does not kill them.
    let final_kill = field(vm.level, "infinalkillcam").and_then(|v| v.as_int()).is_some_and(|v| v != 0);
    let mut out = Vec::new();
    for (client, p) in &zm.players {
        if p.sessionstate != "spectator" {
            continue;
        }
        let archive = num(p.obj, "archivetime").unwrap_or(0.0);
        let spectator = num(p.obj, "spectatorclient").map_or(-1, |v| v as i32);
        if archive <= 0.0 || spectator < 0 {
            continue;
        }
        let seat = crate::ScriptSeat {
            spectator_client: spectator,
            kill_cam_entity: num(p.obj, "killcamentity").map_or(-1, |v| v as i32),
            look_at_entity: num(p.obj, "killcamentitylookat").map_or(-1, |v| v as i32),
            archive_ms: (archive * 1000.0).round() as i32,
            ps_offset_ms: num(p.obj, "psoffsettime").map_or(0, |v| v as i32),
            length_ms: (num(p.obj, "killcamlength").unwrap_or(0.0) * 1000.0).round() as i32,
            final_kill,
        };
        out.push((ClientId(*client), seat));
    }
    out
}

/// bo2mp: bot clients asked for (`addtestclient`), drained.
pub(crate) fn take_test_clients(world: &mut World) -> Vec<u32> {
    world
        .get_resource_mut::<Zm>()
        .map(|mut z| std::mem::take(&mut z.test_client_requests))
        .unwrap_or_default()
}

/// bo2mp: bots to remove (`botleavegame`), drained.
pub(crate) fn take_bot_leaves(world: &mut World) -> Vec<u32> {
    world
        .get_resource_mut::<Zm>()
        .map(|mut z| std::mem::take(&mut z.bot_leaves))
        .unwrap_or_default()
}

/// Run `f` with the VM out of the world.
pub(crate) fn with_vm<R>(
    world: &mut World,
    f: impl FnOnce(&mut Vm<World>, &mut World) -> R,
) -> Option<R> {
    if !world.contains_resource::<T6Runtime>() {
        return None;
    }
    Some(
        world.resource_scope(|world, mut rt: bevy_ecs::prelude::Mut<T6Runtime>| {
            f(&mut rt.vm, world)
        }),
    )
}

/// bo2zm M4: a player's answer from a BO2 menu (`Engine.SendMenuResponse`:
/// "popup_leavegame" "endround", ...) reaches the scripts as his
/// `menuresponse` notify. False when no BO2 map runs.
pub(crate) fn menu_response(world: &mut World, client: u32, menu: &str, response: &str) -> bool {
    // bo2mp: a client command he typed (`cmd drop`): Plutonium's
    // notifyonplayercommand listeners hear it.
    if menu == CLIENT_COMMAND_MENU {
        if world.contains_resource::<T6Runtime>() {
            trickshot::player_command(world, client, response);
        }
        return true;
    }
    // bo2mp: choosing a class reads his classes again (he may have changed
    // them since the match began).
    if response.starts_with("custom") {
        stats::refresh_classes(world, client);
    }
    with_vm(world, |vm, world| {
        let Some(p) = world.resource::<Zm>().players.get(&client).map(|p| p.obj) else {
            return;
        };
        let (m, r) = (vm.string(menu), vm.string(response));
        diag::info!(
            Sim,
            "bo2zm t6 menuresponse from {client}: {menu} {response}"
        );
        vm.notify_str(world, p, "menuresponse", &[m, r]);
    })
    .is_some()
}

/// bo2mp: the menu name a client command travels under (the console's
/// `cmd <text>`, and any command it does not know).
pub const CLIENT_COMMAND_MENU: &str = "bo2mp_cmd";

/// The level start: map entities and structs, then the game type's and
/// the map's `main` and `codecallback_startgametype`.
pub(crate) fn start(world: &mut World) {
    let Some(PendingEntities(texts)) = world.remove_resource::<PendingEntities>() else {
        return;
    };
    let map = world.resource::<Zm>().map.clone();
    let mp_map = world.resource::<Zm>().mp;
    let gametype = with_vm(world, |vm, _| {
        vm.dvars.get("g_gametype").cloned().unwrap_or_default()
    })
    .unwrap_or_default();
    with_vm(world, |vm, world| {
        let level = Value::Object(vm.level);
        vm.spawn_named(
            world,
            "codescripts/struct",
            "initstructs",
            level.clone(),
            vec![],
        );
        natives_ent::spawn_map_entities(vm, world, &texts);
        // The engine's calls, in order. Functions no script calls are the
        // engine's (measured: zm_nuked::gamemode_callback_setup registers
        // the map's game modes, which the game type's start needs;
        // _zm::post_main; codecallback_finalizeinitialization). A map main
        // that calls _zm::post_main itself (Buried) gets it once: twice ran
        // init_custom_ai_type twice (two Leroys, doubled spawn functions).
        let post_main_scripted = !mp_map
            && vm
                .program
                .find(&vm.strings, "maps/mp/zombies/_zm", "post_main")
                .is_some_and(|fi| vm.program.script_refers_to(&format!("maps/mp/{map}"), fi));
        // bo2mp: a multiplayer map starts its game type, the map, then the
        // match.
        let calls: Vec<(String, &str, bool)> = if mp_map {
            // The map's two factions first (bodies, arms, voices per team:
            // game["set_player_model"]): no script starts their teamset;
            // the engine does, for each faction zone it loaded.
            let mut v: Vec<(String, &str, bool)> = vm
                .program
                .scripts
                .iter()
                .map(|s| s.name.clone())
                .filter(|n| {
                    n.starts_with("maps/mp/teams/_teamset_") && n != "maps/mp/teams/_teamset_multiteam"
                })
                .map(|n| (n, "main", false))
                .collect();
            // bo2mp: a map from another game has no Black Ops II map script;
            // what one does first (`maps\mp\_load::main`) still runs.
            let map_script = format!("maps/mp/{map}");
            let own = vm.program.scripts.iter().any(|s| s.name == map_script);
            v.extend([
                (format!("maps/mp/gametypes/{gametype}"), "main", true),
                if own {
                    (map_script, "main", true)
                } else {
                    ("maps/mp/_load".to_owned(), "main", true)
                },
                (
                    "maps/mp/gametypes/_callbacksetup".to_owned(),
                    "codecallback_startgametype",
                    true,
                ),
            ]);
            v
        } else {
            vec![
                (format!("maps/mp/{map}"), "gamemode_callback_setup", false),
                (format!("maps/mp/gametypes_zm/{gametype}"), "main", true),
                (format!("maps/mp/{map}"), "main", true),
                ("maps/mp/zombies/_zm".to_owned(), "post_main", false),
                (
                    "maps/mp/gametypes_zm/_callbacksetup".to_owned(),
                    "codecallback_startgametype",
                    true,
                ),
                (
                    "maps/mp/gametypes_zm/_callbacksetup".to_owned(),
                    "codecallback_finalizeinitialization",
                    false,
                ),
            ]
        };
        // bo2mp: the test aid's script last (BO2MP_DEBUG_GSC).
        let mut calls = calls;
        if !mp_map {
            calls.push(("bo2mp/zm_bots".to_owned(), "main", false));
            calls.push(("bo2mp/zm_guest_guns".to_owned(), "main", false));
        }
        if mp_map {
            // bo2mp: the optional M.O.A.B. (off unless scr_moab 1).
            calls.push(("bo2mp/moab".to_owned(), "main", false));
            calls.push(("bo2mp/resurgence".to_owned(), "main", false));
            // bo2mp: his trickshot script, as Plutonium runs a custom
            // script: `main` before the level loads, `init` once the game
            // type has started (TRICKSHOT MODE only).
            if vm.program.scripts.iter().any(|s| s.name == trickshot::NAME) {
                calls.insert(0, (trickshot::NAME.to_owned(), "main", false));
                calls.push((trickshot::NAME.to_owned(), "init", false));
            }
            calls.push(("bo2mp/debug".to_owned(), "main", false));
        }
        for (script, func, required) in calls {
            if func == "post_main" && post_main_scripted {
                continue;
            }
            if vm
                .spawn_named(world, &script, func, level.clone(), vec![])
                .is_none()
                && required
            {
                diag::warn!(Sim, "bo2zm t6: no {script}::{func}");
            }
        }
        // bo2mp: a map from another game: its minimap from its own corner
        // entities (`minimap_corner`), as its map script would set it up.
        let map_script = format!("maps/mp/{map}");
        if mp_map && !vm.program.scripts.iter().any(|s| s.name == map_script) {
            let material = Value::Str(vm.intern(&format!("compass_map_{map}")));
            vm.spawn_named(world, "maps/mp/_compass", "setupminimap", level.clone(), vec![material]);
        }
        diag::info!(
            Sim,
            "bo2zm t6: level started, {} threads",
            vm.thread_count()
        );
    });
    let mut zm = world.resource_mut::<Zm>();
    zm.started = true;
    zm.map_entities = texts;
    if mp_map {
        map_natives::report_spawns(world, &gametype);
    }
    bo1_maps::start(world, &map);
    report(world);
}

/// bo2mp: Black Ops II's `map_restart` (a round of Search and Destroy over,
/// Demolition's half time): the level starts again in the same match. Every
/// script thread and object goes, and all the level put in the world (its
/// entities, grenades in the air, dropped guns, bodies); `game[]`, the
/// dvars and the loaded assets stay, and each player comes back in with his
/// `pers[]` when `persist` (`map_restart( 1 )`).
fn restart(world: &mut World, tick: crate::Tick, persist: bool) {
    killstreaks::restart(world);
    presence::retire_all(world);
    brushes::retire_all(world);
    let rejoin: BTreeMap<u32, (Value, bool)> = with_vm(world, |vm, world| {
        let pers = vm.intern("pers");
        let rejoin = world
            .resource::<Zm>()
            .players
            .iter()
            .map(|(c, p)| {
                let kept = if persist { vm.raw_field(p.obj, pers) } else { Value::Undefined };
                (*c, (kept, p.test_client))
            })
            .collect();
        vm.restart();
        rejoin
    })
    .unwrap_or_default();
    {
        let mut f = frame(world);
        for p in crate::frame::collect_projectiles(f.ecs()) {
            f.remove_projectile_by_number(p.entnum);
        }
        for number in f.dropped_item_numbers_sorted() {
            if f.remove_dropped_item_by_number(number).is_some() {
                f.free_dynamic_entity_number(number);
            }
        }
        *f.corpses_mut() = crate::PlayerCorpsePool::default();
        for id in f.client_ids_sorted() {
            let (origin, angles) = f
                .player(id)
                .map_or(([0.0; 3], [0.0; 3]), |ps| (ps.origin, ps.viewangles));
            crate::script_player::spawn(&mut f, tick, id, origin, angles, "spectator");
        }
    }
    let old = std::mem::take(&mut *world.resource_mut::<Zm>());
    let baked = old.nav.baked;
    let nodes = old.nav.nodes;
    let node_objs = with_vm(world, |vm, _| node_objects(vm, &nodes)).unwrap_or_default();
    let texts = old.map_entities.clone();
    let players = rejoin.len();
    // What the level loaded and the client was last sent stay (the HUD
    // goes by what it was sent; the models' serials keep counting).
    world.insert_resource(Zm {
        map: old.map,
        tables: old.tables,
        fx: old.fx,
        client_sys: old.client_sys,
        unbound: old.unbound,
        presences: old.presences,
        nav: nav::Nav::new(nodes).with_baked(baked),
        brush_rows: old.brush_rows,
        node_of: node_objs.iter().enumerate().map(|(i, o)| (*o, i as u32)).collect(),
        node_objs,
        anims: old.anims,
        clips: old.clips,
        strings: old.strings,
        sound_aliases: old.sound_aliases,
        shock_names: old.shock_names,
        mp: old.mp,
        ranked_match: old.ranked_match,
        gamesettings: old.gamesettings,
        team_scores: old.team_scores,
        sound_lengths: old.sound_lengths,
        retrievable_weapons: old.retrievable_weapons,
        bots: old.bots.into_iter().map(|(n, b)| (n, b.restarted())).collect(),
        test_client_requests: old.test_client_requests,
        bot_leaves: old.bot_leaves,
        bot_difficulty: old.bot_difficulty,
        game_end_time: old.game_end_time,
        map_center: old.map_center,
        hud_sent: old.hud_sent,
        autoplay_fire: old.autoplay_fire,
        rounds_played: old.rounds_played,
        asds: old.asds,
        now_ms: old.now_ms,
        next_thread_dump: old.next_thread_dump,
        destructibles: old.destructibles,
        zbarrier_defs: old.zbarrier_defs,
        map_natives_applied: old.map_natives_applied,
        map_natives_reported: old.map_natives_reported,
        next_entnum: FIRST_ENTNUM,
        rejoin,
        map_entities: old.map_entities,
        ..Default::default()
    });
    world.insert_resource(engine_sounds::EngineSounds::default());
    let script = world.get_resource::<playeranim::Bodies>().and_then(|b| b.script.clone());
    world.insert_resource(playeranim::Bodies::new(script));
    world.insert_resource(PendingEntities(texts));
    diag::info!(Sim, "bo2mp map_restart({persist}): the level starts again, {players} players come back");
}

/// Each authority frame: connect players, move movers, run the scheduler.
/// IW4L_T6_PROF=1: where a tick's time goes (milliseconds per tick, every
/// 5 s of game), and each slow tick's parts. Without it, only a stall
/// (over STALL_TICK_MS) logs its parts.
#[derive(Default)]
struct Prof {
    ticks: u32,
    parts: Vec<(&'static str, f64)>,
    /// This tick's parts so far.
    tick: Vec<(&'static str, f64)>,
}

/// A tick slower than this logs its parts (ms).
const SLOW_TICK_MS: f64 = 30.0;
/// Profiling or not, a tick slower than this logs its parts (ms): a freeze
/// he would feel names its cause.
const STALL_TICK_MS: f64 = 250.0;

fn prof_on() -> bool {
    static ON: std::sync::OnceLock<bool> = std::sync::OnceLock::new();
    *ON.get_or_init(|| std::env::var_os("IW4L_T6_PROF").is_some())
}

thread_local! {
    /// The thread's CPU time when this tick began (a slow tick says how
    /// much of it was work).
    static TICK_CPU: std::cell::Cell<Option<f64>> = const { std::cell::Cell::new(None) };
    /// The profile line just went out: the scripts' own follows it.
    static SCRIPT_REPORT: std::cell::Cell<bool> = const { std::cell::Cell::new(false) };
}

fn prof_add(name: &'static str, since: std::time::Instant) -> std::time::Instant {
    static PROF: std::sync::Mutex<Option<Prof>> = std::sync::Mutex::new(None);
    let now = std::time::Instant::now();
    if let Ok(mut p) = PROF.lock() {
        let p = p.get_or_insert_with(Prof::default);
        if name == "tick" {
            let ms = (now - since).as_secs_f64() * 1000.0;
            let slow = if prof_on() { SLOW_TICK_MS } else { STALL_TICK_MS };
            if ms > slow {
                let parts: Vec<String> = p
                    .tick
                    .iter()
                    .filter(|(n, _)| *n != "total")
                    .map(|(n, ms)| format!("{n} {ms:.1}"))
                    .collect();
                let busy = diag::busy_note(TICK_CPU.get());
                diag::warn!(Sim, "bo2mp slow tick: {ms:.1} ms ({}){busy}", parts.join(", "));
            }
            p.tick.clear();
            p.ticks += 1;
            if !prof_on() {
                p.parts.clear();
                p.ticks = 0;
            } else if p.ticks >= 100 {
                let line: Vec<String> = p
                    .parts
                    .iter()
                    .map(|(n, ms)| format!("{n} {:.2}", ms / f64::from(p.ticks)))
                    .collect();
                diag::info!(Sim, "bo2zm t6 prof (ms per tick): {}", line.join(", "));
                *p = Prof::default();
                SCRIPT_REPORT.set(true);
            }
        } else {
            let ms = (now - since).as_secs_f64() * 1000.0;
            match p.parts.iter_mut().find(|(n, _)| *n == name) {
                Some(row) => row.1 += ms,
                None => p.parts.push((name, ms)),
            }
            p.tick.push((name, ms));
        }
    }
    now
}

pub(crate) fn advance(world: &mut World) {
    if !world.contains_resource::<T6Runtime>() {
        return;
    }
    // Always timed (a few clock reads a tick): a stall names its part.
    let t0 = std::time::Instant::now();
    TICK_CPU.set(diag::thread_cpu_ms());
    advance_inner(world);
    if prof_on() {
        prof_add("total", t0);
    }
    prof_add("tick", t0);
    if SCRIPT_REPORT.replace(false)
        && let Some(Some(line)) = with_vm(world, |vm, _| vm.profile_report(10))
    {
        diag::info!(Sim, "bo2mp script prof (ms over 100 ticks): {line}");
    }
}

/// One step's sub-part, timed when profiling.
/// bo2zm: the hang tracer. Each sim phase names itself; a watcher thread
/// prints the phase (to stderr, which a stuck main thread cannot hold up)
/// when no tick or client frame has gone by for 5 s.
static PHASE: std::sync::Mutex<&'static str> = std::sync::Mutex::new("idle");
static TICKS: std::sync::atomic::AtomicU64 = std::sync::atomic::AtomicU64::new(0);
static FRAMES: std::sync::atomic::AtomicU64 = std::sync::atomic::AtomicU64::new(0);

pub(crate) fn phase(name: &'static str) {
    if let Ok(mut p) = PHASE.lock() {
        *p = name;
    }
}

/// His client drew a frame (the hang tracer's other clock).
pub fn client_frame_heartbeat() {
    FRAMES.fetch_add(1, std::sync::atomic::Ordering::Relaxed);
}

fn start_hang_tracer() {
    static ONCE: std::sync::Once = std::sync::Once::new();
    ONCE.call_once(|| {
        let _ = std::thread::Builder::new().name("bo2zm-hang-tracer".into()).spawn(|| {
            use std::sync::atomic::Ordering::Relaxed;
            let (mut ticks, mut frames, mut told) = (0, 0, 0);
            loop {
                std::thread::sleep(std::time::Duration::from_secs(5));
                let (t, f) = (TICKS.load(Relaxed), FRAMES.load(Relaxed));
                let stuck_sim = t == ticks && t > 0;
                let stuck_client = f == frames && f > 0;
                // Told up to 6 times (every 5 s), so a script loop shows its
                // instruction count moving and a stuck builtin shows not.
                if (stuck_sim || stuck_client) && told < 6 {
                    let p = PHASE.lock().map(|p| *p).unwrap_or("?");
                    eprintln!(
                        "bo2zm hang: sim ticks {} ({t}), client frames {} ({f}); sim phase {p}; {}",
                        if stuck_sim { "STOPPED" } else { "going" },
                        if stuck_client { "STOPPED" } else { "going" },
                        gsc_t6::hang_report(),
                    );
                    diag::info!(Sim, "bo2zm hang: sim stuck {stuck_sim}, client stuck {stuck_client}, phase {p}");
                    told += 1;
                } else if !(stuck_sim || stuck_client) {
                    told = 0;
                }
                (ticks, frames) = (t, f);
            }
        });
    });
}

fn timed(name: &'static str, world: &mut World, f: impl FnOnce(&mut World)) {
    phase(name);
    let t0 = std::time::Instant::now();
    f(world);
    prof_add(name, t0);
}

fn advance_inner(world: &mut World) {
    let (advances, tick) = {
        let request = world.resource::<crate::step::StepRequest>();
        (request.reason.advances_authority_world(), request.tick)
    };
    if !advances {
        return;
    }
    // bo2mp: a round restart (`map_restart`) asked for last frame: the
    // level starts again before anything else runs.
    let restart_asked = world.resource_mut::<Zm>().pending_restart.take();
    if let Some(persist) = restart_asked {
        restart(world, tick, persist);
    }
    if world.contains_resource::<PendingEntities>() {
        start(world);
    }
    let now = i64::from(tick.0) * i64::from(crate::MATCH_TICK_MS);
    bo1_maps::advance(world, now);
    world.resource_mut::<Zm>().now_ms = now;
    start_hang_tracer();
    TICKS.fetch_add(1, std::sync::atomic::Ordering::Relaxed);
    timed("players", world, |world| {
        // (Its parts timed too: "p." in the profile line.)
        timed("p.sync", world, players::sync);
        timed("p.weapons", world, players::weapon_events);
        timed("p.shock", world, players::shock_tick);
        // bo2mp: each player's third-person animations, as BO2's script picks.
        if world.resource::<Zm>().mp {
            timed("p.anim", world, playeranim::advance);
        }
        // bo2zm: the riot shield's plant.
        timed("p.shield", world, riotshield::deploy_events);
        timed("p.moves", world, players::movement_events);
        if !world.resource::<Zm>().debris_paths_cut {
            brushes::cut_debris_paths(world);
        }
    });
    timed("bots", world, |world| {
        // The test player's buttons go in before the triggers read them.
        autoplay::drive(world, now);
        // bo2mp: a timed input script played as his commands (test aid).
        input_replay::drive(world, now);
        // bo2zm: the solo cheat menu's picks.
        cheats::apply(world);
        cheats::hold(world);
        // bo2mp: the bots' commands too.
        if world.resource::<Zm>().mp {
            bots::drive(world, now);
            // bo2mp: buttons bound with notifyonplayercommand ("+frag").
            trickshot::buttons(world);
        }
    });
    // Scorestreaks once every command is in (a bot capturing a crate
    // holds use): thrown and fired projectiles reach the scripts,
    // crafts fly, turrets aim.
    if world.resource::<Zm>().mp {
        timed("streaks", world, killstreaks::advance);
    }
    timed("tests", world, |world| autoplay_tests(world, now));
    // bo2zm: Die Rise's map parts (every map; they return at once elsewhere).
    timed("highrise", world, |world| highrise::tick(world, now));
    timed("weapons", world, |world| {
        darts::advance(world, now);
        darts::log(world, now);
        grenades::advance(world, now);
        // limits first: its IW4L_T6_PRESS holds are test aids too.
        limits::tick(world);
        // After the test aids: their held fire spins it too.
        jetgun::tick(world, crate::MATCH_TICK_MS as f32);
        offhand::tick(world);
    });
    // Each timed apart: a stall names the one.
    timed("vehicles", world, |world| vehicles::advance(world, crate::MATCH_TICK_MS as f32 / 1000.0));
    timed("movers", world, |world| {
        movers::advance(world, now);
        riders::advance(world);
        turrets::advance(world);
        let moved = {
            let mut zm = world.resource_mut::<Zm>();
            let zm = &mut *zm;
            let ents = &zm.ents;
            zm.nav
                .follow_movers(|n| ents.get(&n).map(|e| (e.origin, e.angles)))
        };
        // Their script `origin` too (Die Rise's escape pod links its door
        // nodes to the nodes near node.origin once it has crashed).
        if !moved.is_empty() {
            let objs: Vec<(ObjRef, [f32; 3])> = {
                let zm = world.resource::<Zm>();
                moved
                    .iter()
                    .filter_map(|&(n, p)| zm.node_objs.get(n as usize).map(|&o| (o, p)))
                    .collect()
            };
            with_vm(world, |vm, _| {
                let f = vm.intern("origin");
                for (o, p) in objs {
                    vm.set_raw_field(o, f, Value::Vec3(p));
                }
            });
        }
    });
    timed("zbarriers", world, |world| zbarrier::advance(world, now));
    timed("wallbuys", world, |world| wallbuys::advance(world, now));
    if world.resource::<Zm>().mp {
        timed("items", world, items::settle);
    }
    timed("triggers", world, triggers::dispatch);
    // Death barriers: the map's trigger_hurt volumes and the fall kill plane.
    timed("barriers", world, |world| hurt::advance(world, now));
    timed("hud", world, |world| {
        publish_hud(world);
        hud::publish(world);
        // bo2mp lane C: his multiplayer screen's values.
        if world.resource::<Zm>().mp {
            natives_mp::publish_screen(world);
            stats::save(world, false);
        }
    });
    timed("scripts", world, |world| {
        with_vm(world, |vm, world| {
            if vm.time_ms < now {
                vm.time_ms = now;
            }
            natives_game::deliver_timers(vm, world, now);
            vm.run_due(world);
        });
    });
    timed("actors", world, |world| actors::think(world, now));
    phase("gibs");
    // Limbs that have lain long enough go.
    let gone: Vec<u32> = {
        let mut zm = world.resource_mut::<Zm>();
        let (old, keep): (Vec<(u32, i64)>, Vec<(u32, i64)>) =
            zm.gibs.iter().partition(|(_, t)| *t <= now);
        zm.gibs = keep;
        old.into_iter().map(|(n, _)| n).collect()
    };
    for n in gone {
        let obj = world
            .resource_mut::<Zm>()
            .ents
            .remove(&n)
            .and_then(|e| e.obj);
        if let Some(o) = obj {
            with_vm(world, |vm, world| vm.free_object(world, o));
        }
    }
    timed("presence", world, |world| {
        presence::sync(world, now);
        destructible::flush(world);
        fxanims::advance(world, now);
        prison_wolves::advance(world, now);
        audio_triggers::advance(world, now);
        brushes::sync(world, now);
        map_natives::sync(world, now);
        players::follow_sync(world);
        publish_melee_targets(world);
    });
    timed("loops+fx", world, |world| {
        presence::publish_loops(world);
        clientfields::advance(world, now);
    });
    phase("report");
    report(world);
    phase("idle");
}

/// The test runs' hooks (each does nothing unless its test is on).
fn autoplay_tests(world: &mut World, now: i64) {
    autoplay::buy_test(world, now);
    autoplay::open_all_test(world, now);
    autoplay::census_test(world, now);
    autoplay::roam_test(world, now);
    autoplay::glide_test(world, now);
    autoplay::only_spawn_test(world);
    autoplay::box_test(world, now);
    autoplay_prison::shock_test(world, now);
    autoplay_prison::door_sweep(world, now);
    autoplay_prison::lives_watch(world, now);
    autoplay_prison::brutus_test(world, now);
    autoplay_prison::gondola_test(world, now);
    autoplay_prison::zone_probe(world, now);
    autoplay_prison::ghost_door_test(world, now);
    prison_plane::plane_test(world, now);
    prison_retriever::retriever_test(world, now);
    autoplay::box_move_test(world, now);
    autoplay::dual_wield_test(world, now);
    autoplay::mannequin_test(world, now);
    autoplay::knife_at_test(world, now);
    autoplay::view_test(world, now);
    autoplay::nohud_test(world);
    autoplay::bears_test(world, now);
    autoplay::perk_drop_test(world, now);
    autoplay::egg1_test(world, now);
    buried_egg::lantern_test(world, now);
    autoplay::game_over_test(world, now);
    autoplay::down_test(world, now);
    autoplay::corevive_test(world, now);
    autoplay::nades_test(world, now);
    autoplay::press_test(world, now);
    autoplay::give_test(world, now);
    autoplay::sprint_log(world, now);
    autoplay::door_test(world, now);
    autoplay::use_test(world, now);
    autoplay::walk_test(world, now);
    autoplay::ent_log(world, now);
    autoplay::floor_map(world, now);
    autoplay::kite_test(world, now);
    autoplay::face_test(world);
    autoplay::zombie_block_test(world, now);
    autoplay::solid_walk_test(world, now);
    autoplay::perk_test(world, now);
    use_test::use_test(world, now);
    use_test::build_test(world, now);
    use_test::place_test(world, now);
    autoplay::pap_test(world, now);
    autoplay::powerup_test(world, now);
    autoplay::fx_test(world, now);
    autoplay::look_test(world);
    autoplay::gib_test(world, now);
    tomb_test::generator_test(world, now);
    tomb_test::dig_test(world, now);
    tomb_test::recapture_test(world, now);
    tomb_mechz::mechz_test(world, now);
    tomb_robot::robot_test(world, now);
    tomb_biplane::biplane_test(world, now);
    tomb_portal::portal_test(world, now);
    tomb_craft::craft_test(world, now);
    tomb_staffs::staff_puzzle_test(world, now);
    tomb_tank::tank_test(world, now);
}

/// bo2mp: he steers a scorestreak from where he stands (a vehicle he
/// drives or rides, the Lodestar's view).
pub(crate) fn remote_control(world: &World, client: u32) -> bool {
    world.get_resource::<Zm>().is_some_and(|z| z.mp) && killstreaks::remote_control(world, client)
}

/// bo2mp: his command's switch to a scorestreak weapon BO2's scripts would
/// refuse (not earned, airspace full, last stand, ...) never starts.
pub(crate) fn gate_streak_switch(world: &mut World, client: u32, cmd: &mut playerstate_iw4::UserCmd) {
    if world.get_resource::<Zm>().is_some_and(|z| z.mp) {
        killstreaks::gate::gate_cmd(world, client, cmd);
    }
}

/// bo2mp: he pressed a d-pad slot's key (`+actionslot`, 0-based slot).
pub(crate) fn action_slot_pressed(world: &mut World, client: u32, slot: u8) {
    if world.get_resource::<Zm>().is_some_and(|z| z.mp) {
        killstreaks::action_slot_pressed(world, client, slot);
        trickshot::action_slot(world, client, slot);
    }
}

/// bo2mp: a player picked a spot on the map for a scorestreak (or backed
/// out); BO2's scripts hear it. False when no BO2 match runs.
pub(crate) fn location_pick(world: &mut World, client: u32, picked: [u8; 3], confirm: bool) -> bool {
    if !world.get_resource::<Zm>().is_some_and(|z| z.mp) {
        return false;
    }
    killstreaks::location_pick(world, client, picked, confirm);
    true
}

/// bo2mp: the console's `give killstreak`, to BO2's own scripts. False when
/// no BO2 match runs.
pub(crate) fn give_killstreak(world: &mut World, client: u32, menu_name: &str) -> bool {
    if !world.get_resource::<Zm>().is_some_and(|z| z.mp) {
        return false;
    }
    // bo2mp: the Tactical Nuke (`give killstreak nuke`): ready to call in,
    // as if earned (the `bo2mp/moab` script announces it).
    if matches!(menu_name.to_ascii_lowercase().as_str(), "nuke" | "kem" | "tactical_nuke" | "moab" | "kem_strike") {
        let given = with_vm(world, |vm, world| {
            let obj = world.resource::<Zm>().players.get(&client)?.obj;
            let (ready, earned) = (vm.intern("moab_ready"), vm.intern("moab_earned"));
            // (Asked again while he has one: it is called in.)
            if vm.raw_field(obj, ready).as_int() == Some(1) {
                let now = vm.intern("moab_call_now");
                vm.set_raw_field(obj, now, gsc_t6::Value::Int(1));
            }
            vm.set_raw_field(obj, ready, gsc_t6::Value::Int(1));
            vm.set_raw_field(obj, earned, gsc_t6::Value::Int(1));
            Some(())
        })
        .flatten()
        .is_some();
        diag::info!(Sim, "bo2mp: give killstreak nuke to client {client}: {}", if given { "ready" } else { "no player" });
        return true;
    }
    killstreaks::give(world, client, menu_name);
    true
}

/// bo2zm: the living zombies near each player (`n x y z;...`: entity
/// number and feet, nearest first, at most 24 within 2000 units), for his
/// client's knife lunge (BO2 lunges at a zombie in reach and stabs; IW4's
/// lunge only looked for players) and the pad's aim assist (BO2 assists onto
/// zombies; IW4's only onto players).
fn publish_melee_targets(world: &mut World) {
    const REACH: f32 = 2000.0;
    let players: Vec<u32> = world.resource::<Zm>().players.keys().copied().collect();
    for n in players {
        let Some(me) = frame(world).player(ClientId(n)).map(|ps| ps.origin) else {
            continue;
        };
        let list = {
            let zm = world.resource::<Zm>();
            let mut near: Vec<(f32, u32, [f32; 3])> = zm
                .actors
                .iter()
                .filter(|(_, a)| a.alive)
                .filter_map(|(k, _)| zm.ents.get(k).map(|e| (*k, e.origin)))
                .map(|(k, o)| {
                    let d: [f32; 3] = std::array::from_fn(|i| o[i] - me[i]);
                    (d[0] * d[0] + d[1] * d[1] + d[2] * d[2], k, o)
                })
                .filter(|(d2, _, _)| *d2 <= REACH * REACH)
                .collect();
            near.sort_by(|a, b| a.0.total_cmp(&b.0));
            near.iter()
                .take(24)
                .map(|(_, k, o)| format!("{k} {:.0} {:.0} {:.0}", o[0], o[1], o[2]))
                .collect::<Vec<_>>()
                .join(";")
        };
        set_client_dvar(world, n, "bo2zm_zombies", &list);
    }
}

/// A client dvar for one player's client (its HUD, its music).
pub(crate) fn set_client_dvar(world: &mut World, client: u32, key: &str, value: &str) {
    let mut f = frame(world);
    if f.client_meta(crate::world::ClientId(client)).is_none() {
        return;
    }
    let dvars = &mut f
        .client_meta_mut(crate::world::ClientId(client))
        .client_dvars;
    match dvars.iter_mut().find(|(k, _)| k == key) {
        Some(row) => value.clone_into(&mut row.1),
        None => dvars.push((key.to_owned(), value.to_owned())),
    }
}

fn report(world: &mut World) {
    let msgs = with_vm(world, |vm, _| std::mem::take(&mut vm.messages)).unwrap_or_default();
    for m in msgs {
        diag::warn!(Sim, "bo2zm t6: {m}");
    }
    // IW4L_T6_THREADS=<seconds>: list every waiting thread that often.
    let every = std::env::var("IW4L_T6_THREADS")
        .ok()
        .and_then(|s| s.parse::<i64>().ok());
    if let Some(every) = every {
        let now = world.resource::<Zm>().now_ms;
        let due = {
            let mut zm = world.resource_mut::<Zm>();
            if now >= zm.next_thread_dump {
                zm.next_thread_dump = now + every.max(1) * 1000;
                true
            } else {
                false
            }
        };
        if due {
            let lines = with_vm(world, |vm, _| vm.describe_threads()).unwrap_or_default();
            diag::info!(Sim, "bo2zm t6 threads at {}s: {}", now / 1000, lines.len());
            let mut counts: BTreeMap<String, usize> = BTreeMap::new();
            for l in lines {
                *counts.entry(l).or_default() += 1;
            }
            for (l, n) in counts {
                diag::info!(Sim, "bo2zm t6 thread x{n}: {l}");
            }
        }
    }
    // IW4L_T6_ROUND_JUMP=<n>[@<secs>]: 10 s in (or then), set
    // level.round_number to n (a test aid for things that only start in a
    // round-1 game, like the Nuketown transmission and blue eyes; or that
    // wait rounds, like TranZit's Avogadro in his cloud: `5@300`).
    static ROUND_JUMPED: std::sync::atomic::AtomicBool = std::sync::atomic::AtomicBool::new(false);
    if let Some((n, at)) = std::env::var("IW4L_T6_ROUND_JUMP").ok().and_then(|v| {
        let (n, at) = v.split_once('@').unwrap_or((&v, "10"));
        Some((n.trim().parse::<i32>().ok()?, at.trim().parse::<f32>().ok()?))
    }) && world.resource::<Zm>().now_ms >= (at * 1000.0) as i64
        && !ROUND_JUMPED.swap(true, std::sync::atomic::Ordering::Relaxed)
    {
        with_vm(world, |vm, world| {
            let f = vm.intern("round_number");
            let level = vm.level;
            vm.set_field(world, level, f, Value::Int(n));
        });
        diag::info!(Sim, "bo2zm t6: round jumped to {n}");
    }
    // IW4L_T6_KILLS_ADD=<n>: 10 s in, n more kills on the books
    // (level.total_zombies_killed): the population sign counts them down.
    // IW4L_T6_DOOR_GRAB_AT=<n>: when the sign reads n, the player steps onto
    // the power-up behind the door (Nuketown's first song egg).
    static KILLS_ADDED: std::sync::atomic::AtomicBool = std::sync::atomic::AtomicBool::new(false);
    if let Some(n) = std::env::var("IW4L_T6_KILLS_ADD").ok().and_then(|v| v.parse::<i32>().ok())
        && world.resource::<Zm>().now_ms >= 10_000
        && !KILLS_ADDED.swap(true, std::sync::atomic::Ordering::Relaxed)
    {
        with_vm(world, |vm, world| {
            let f = vm.intern("total_zombies_killed");
            let level = vm.level;
            let had = match vm.get_field(world, level, f) {
                Value::Int(i) => i,
                _ => 0,
            };
            vm.set_field(world, level, f, Value::Int(had + n));
        });
        diag::info!(Sim, "bo2zm t6: {n} kills added");
    }
    if let Some(want) = std::env::var("IW4L_T6_DOOR_GRAB_AT").ok().and_then(|v| v.parse::<i32>().ok()) {
        let at = with_vm(world, |vm, world| {
            let level = vm.level;
            let f = vm.intern("population_count");
            let pop = vm.get_field(world, level, f);
            let f = vm.intern("door_powerup");
            let pu = match vm.get_field(world, level, f) {
                Value::Object(o) if vm.alive(o) => Some(o),
                _ => None,
            };
            let f = vm.intern("origin");
            let origin = pu.map(|o| vm.get_field(world, o, f));
            (pop, origin)
        });
        if let Some((Value::Int(pop), Some(Value::Vec3(o)))) = at
            && pop == want
        {
            static SAID: std::sync::atomic::AtomicBool = std::sync::atomic::AtomicBool::new(false);
            if !SAID.swap(true, std::sync::atomic::Ordering::Relaxed) {
                diag::info!(Sim, "bo2zm t6 test: sign at {pop}, grabbing the door power-up at {o:?}");
            }
            teleport_player(world, crate::world::ClientId(0), [o[0], o[1], o[2] - 40.0]);
        }
    }
    // IW4L_T6_SETLEVEL=field=int,...: set on level at 15 s (a test aid:
    // Die Rise's next_leaper_round=2 brings a Jumping Jack round early).
    if let Ok(sets) = std::env::var("IW4L_T6_SETLEVEL") {
        let now = world.resource::<Zm>().now_ms;
        if (15000..15000 + i64::from(crate::MATCH_TICK_MS)).contains(&now) {
            with_vm(world, |vm, _| {
                for (k, v) in sets.split(',').filter_map(|kv| kv.split_once('=')) {
                    let Ok(v) = v.parse::<i32>() else {
                        continue;
                    };
                    let f = vm.intern(k);
                    vm.set_raw_field(vm.level, f, Value::Int(v));
                    diag::info!(Sim, "bo2zm t6 setlevel {k} = {v}");
                }
            });
        }
    }
    // IW4L_T6_EVAL=level.a.b,...: those values every 5 s (debugging; roots level, anim, player).
    if let Ok(paths) = std::env::var("IW4L_T6_EVAL") {
        let now = world.resource::<Zm>().now_ms;
        if now % 5000 < i64::from(crate::MATCH_TICK_MS) {
            let lines = with_vm(world, |vm, world| {
                let mut out = Vec::new();
                for path in paths.split(',') {
                    let mut parts = path.split('.');
                    let mut v = match parts.next() {
                        Some("level") => Value::Object(vm.level),
                        Some("anim") => Value::Object(vm.anim),
                        Some("player") => match world.resource::<Zm>().players.get(&0) {
                            Some(p) => Value::Object(p.obj),
                            None => continue,
                        },
                        _ => continue,
                    };
                    for p in parts {
                        v = match &v {
                            Value::Object(o) => {
                                let f = vm.intern(p);
                                vm.get_field(world, *o, f)
                            }
                            Value::Array(a) => {
                                let k = match p.parse::<i32>() {
                                    Ok(i) => gsc_t6::Key::Int(i),
                                    Err(_) => gsc_t6::Key::Str(vm.intern(p)),
                                };
                                a.get(&k).unwrap_or_default()
                            }
                            _ => Value::Undefined,
                        };
                    }
                    let text = match &v {
                        Value::Array(a) => {
                            let snap = a.snapshot();
                            let items: Vec<String> = snap
                                .keys()
                                .map(|k| {
                                    let val = snap.get(&k).cloned().unwrap_or_default();
                                    format!("{}={}", vm.to_text(&k.value()), vm.to_text(&val))
                                })
                                .collect();
                            format!("[{}]", items.join(", "))
                        }
                        other => vm.to_text(other),
                    };
                    out.push(format!("{path} = {text}"));
                }
                out
            })
            .unwrap_or_default();
            for l in lines {
                diag::info!(Sim, "bo2zm t6 eval at {}s: {l}", now / 1000);
            }
        }
    }
    // IW4L_T6_ACTORS=<ms>: every actor's place, animscript and animation.
    let every = std::env::var("IW4L_T6_ACTORS")
        .ok()
        .and_then(|s| s.parse::<i64>().ok());
    if let Some(every) = every {
        let zm = world.resource::<Zm>();
        let now = zm.now_ms;
        if now % every.max(50) < i64::from(crate::MATCH_TICK_MS) {
            for (n, a) in &zm.actors {
                let e = zm.ents.get(n);
                let o = e.map_or([0.0; 3], |e| e.origin);
                let anim = a.playing.as_ref().map_or("-".to_owned(), |p| {
                    format!("{}/{}:{}", p.state, p.substate, p.anim)
                });
                let next = a.path.get(a.path_i).copied().unwrap_or([0.0; 3]);
                let goal = a.goal.unwrap_or([0.0; 3]);
                diag::info!(
                    Sim,
                    "bo2zm t6 actor {n} at {}ms: ({:.0} {:.0} {:.0}) yaw {:.0} script {} anim {} hp {} alive {} path {}/{} next ({:.0} {:.0} {:.0}) goal ({:.0} {:.0} {:.0}) scripted {} traverse {} model {}",
                    now,
                    o[0],
                    o[1],
                    o[2],
                    e.map_or(0.0, |e| e.angles[1]),
                    a.script,
                    anim,
                    a.health,
                    a.alive,
                    a.path_i,
                    a.path.len(),
                    next[0],
                    next[1],
                    next[2],
                    goal[0],
                    goal[1],
                    goal[2],
                    a.scripted.is_some(),
                    a.traverse.is_some(),
                    e.map_or("", |e| e.model.as_str())
                );
            }
            // IW4L_T6_ACTOR_FIELDS=a,b,...: those script fields of each actor too.
            if let Ok(fields) = std::env::var("IW4L_T6_ACTOR_FIELDS") {
                let objs: Vec<(u32, ObjRef)> = zm
                    .actors
                    .iter()
                    .filter_map(|(n, a)| a.obj.map(|o| (*n, o)))
                    .collect();
                let lines = with_vm(world, |vm, world| {
                    objs.iter()
                        .map(|(n, o)| {
                            let vals: Vec<String> = fields
                                .split(',')
                                .map(|f| {
                                    let id = vm.intern(f);
                                    let v = vm.get_field(world, *o, id);
                                    format!("{f}={}", vm.to_text(&v))
                                })
                                .collect();
                            format!("{n}: {}", vals.join(" "))
                        })
                        .collect::<Vec<_>>()
                })
                .unwrap_or_default();
                for l in lines {
                    diag::info!(Sim, "bo2zm t6 actor fields at {now}ms {l}");
                }
            }
        }
    }
}

fn unbound(vm: &mut Vm<World>, world: &mut World, id: u32, _self: &Value, args: &[Value]) -> Value {
    let (name, method) = vm.program.builtins[id as usize];
    let n = format!("{}{}", if method { "." } else { "" }, vm.str(name));
    let mut zm = world.resource_mut::<Zm>();
    let count = zm.unbound.entry(n.clone()).or_insert(0);
    *count += 1;
    if *count == 1 {
        let a: Vec<String> = args.iter().map(|a| vm.to_text(a)).collect();
        diag::warn!(Sim, "bo2zm t6: unbound builtin {n}({})", a.join(", "));
    }
    Value::Undefined
}

/// bo2zm M3: engine damage on a player (fall, explosion, ...) goes to
/// the BO2 scripts' CodeCallback_PlayerDamage; false when they don't run.
pub(crate) fn player_hit(world: &mut World, hit: &crate::script_player::Hit) -> bool {
    if !world.contains_resource::<T6Runtime>() {
        return false;
    }
    // A bullet or a knife the riot shield stopped: BO2's scripts know the
    // hit location as "riotshield".
    let hitloc = if hit.hitloc == crate::bullet_collision::HITLOC_SHIELD {
        "riotshield"
    } else {
        T6_HITLOC_NAMES
            .get(usize::from(hit.hitloc))
            .copied()
            .unwrap_or("none")
    };
    let mut wname = weapon_text(world, hit.weapon);
    let mut amount = hit.amount;
    // bo2mp: BO2's melee button slashes with the knife every class carries
    // (knife_mp, given as the melee weapon; its file's iMeleeDamage 150 - a
    // kill), whatever gun is up (the guns' own iMeleeDamage is 25); the riot
    // shield bashes and a held knife stabs with their own.
    if hit.means == "MOD_MELEE" && world.resource::<Zm>().mp {
        // The gun's row already melees with the knife's damage (the combat
        // table's melee weapon); the scripts are told it was the knife.
        let f = frame(world);
        if let Some(facts) = f.combat_facts_for(hit.weapon)
            && facts.knife_model != 0
            && facts.knife_model != hit.weapon
        {
            amount = amount.max(facts.melee_damage);
            wname = crate::script_player::weapon_name(&f, facts.knife_model);
        }
    }
    if std::env::var_os("BO2MP_HITLOG").is_some() {
        diag::info!(
            Sim,
            "bo2mp hit: player {} by {:?} with {wname} ({}): {hitloc} for {amount}",
            hit.victim.0,
            hit.attacker.map(|a| a.0),
            hit.means
        );
    }
    with_vm(world, |vm, world| {
        let attacker = hit
            .attacker
            .and_then(|a| {
                world
                    .resource::<Zm>()
                    .players
                    .get(&a.0)
                    .map(|p| Value::Object(p.obj))
            })
            .unwrap_or(Value::Undefined);
        natives_ai::player_damage(
            vm,
            world,
            hit.victim.0,
            attacker.clone(),
            attacker,
            amount,
            hit.flags,
            hit.means,
            &wname,
            hit.point,
            hit.dir,
            hitloc,
        );
    });
    true
}

/// A broken prop's piece (a mannequin's head, BO2's stage spawn model)
/// flies off its bone, tumbling, lands on the floor below and lies there
/// 20 s, as BO2's physics debris does.
pub(crate) fn throw_debris(
    world: &mut World,
    model: &str,
    from: [f32; 3],
    direction: [f32; 3],
    floor: f32,
) {
    if !world.contains_resource::<T6Runtime>() {
        return;
    }
    let now = world.resource::<Zm>().now_ms;
    let g = world.resource_mut::<Zm>().alloc_entnum();
    // Out that way, a little to one side (by its number), and up: a short
    // hop that lands within about 40 units.
    let (fx, fy) = (direction[0], direction[1]);
    let len = (fx * fx + fy * fy).sqrt().max(0.001);
    let side = (g % 7) as f32 / 3.0 - 1.0;
    let (hx, hy) = (fx / len - fy / len * side * 0.3, fy / len + fx / len * side * 0.3);
    let vel = [hx * 55.0, hy * 55.0, 160.0];
    // Flight time to the floor: z(t) = z0 + vz*t - 400*t^2.
    let drop = (from[2] - floor).max(0.0);
    let secs = ((vel[2] + (vel[2] * vel[2] + 1600.0 * drop).sqrt()) / 800.0).clamp(0.2, 3.0);
    let yaw = fy.atan2(fx).to_degrees();
    let obj = with_vm(world, |vm, _| vm.alloc_object(ObjKind::Entity(g)));
    let Some(obj) = obj else {
        return;
    };
    let spin = if g % 2 == 0 { 1.0 } else { -1.0 };
    let mut zm = world.resource_mut::<Zm>();
    zm.ents.insert(
        g,
        Ent {
            obj: Some(obj),
            classname: "script_model".into(),
            origin: from,
            angles: [0.0, yaw, 0.0],
            model: model.to_owned(),
            ..Default::default()
        },
    );
    zm.movers.gravity(g, from, vel, now, secs);
    zm.movers.rotate_to(g, [0.0, yaw, 0.0], [90.0 * spin, yaw + 200.0 * spin, 0.0], now, secs, 0.0, 0.0);
    zm.gibs.push((g, now + (secs * 1000.0) as i64 + 20000));
    let at = [from[0] + vel[0] * secs, from[1] + vel[1] * secs, floor];
    diag::info!(Sim, "bo2zm t6 debris {model} thrown from {from:?}, lands in {secs:.2}s at {at:?}");
}

/// bo2zm M3: a bullet (or a knife, a blast) on a script model that shows a
/// BO2 entity: an actor takes it through CodeCallback_ActorDamage with the
/// hit location of the bone it struck (the weapon's location multiplier
/// applied, as the engine does); false when the model isn't one of ours.
/// A map prop's breakable piece broke (`destructibledef` stage with a
/// break notify): the engine's `codecallback_destructibleevent("broken",
/// notify, attacker, weapon)` on the prop (the Nuketown mannequin heads
/// are "headless").
pub(crate) fn destructible_broken(
    world: &mut World,
    target: crate::ScriptModelId,
    notify: &str,
    attacker: Option<ClientId>,
    weapon: u32,
) {
    if !world.contains_resource::<T6Runtime>() {
        return;
    }
    let Some(n) = world
        .resource::<Zm>()
        .presences
        .by_ent
        .iter()
        .find(|(_, s)| s.id == target)
        .map(|(n, _)| *n)
    else {
        return;
    };
    let Some(obj) = world.resource::<Zm>().ents.get(&n).and_then(|e| e.obj) else {
        return;
    };
    let w = weapon_text(world, weapon);
    diag::info!(Sim, "bo2zm t6 destructible ent{n} broken {notify}");
    with_vm(world, |vm, world| {
        let att = attacker
            .and_then(|a| {
                world
                    .resource::<Zm>()
                    .players
                    .get(&a.0)
                    .map(|p| Value::Object(p.obj))
            })
            .unwrap_or(Value::Undefined);
        let args = vec![vm.string("broken"), vm.string(notify), att, vm.string(&w)];
        vm.spawn_named(
            world,
            "maps/mp/zombies/_zm",
            "codecallback_destructibleevent",
            Value::Object(obj),
            args,
        );
    });
}

pub(crate) fn entity_hit(world: &mut World, hit: &crate::script::EntityHit) -> bool {
    if !world.contains_resource::<T6Runtime>() {
        return false;
    }
    let Some(n) = world
        .resource::<Zm>()
        .presences
        .by_ent
        .iter()
        .find(|(_, s)| s.id == hit.target)
        .map(|(n, _)| *n)
    else {
        return false;
    };
    let alive = world
        .resource::<Zm>()
        .actors
        .get(&n)
        .is_some_and(|a| a.alive);
    if !alive {
        // bo2mp: a map's breakable (a Nuketown mannequin, a car) takes a
        // blast or a knife by the piece's own scale.
        let breakable = world
            .resource::<Zm>()
            .ents
            .get(&n)
            .is_some_and(|e| e.destructible.is_some());
        // bo2mp: a scorestreak's plane, chopper, drone, vehicle or turret:
        // its scripts hear the hit.
        if !breakable && killstreaks::damage_target(world, n) {
            let weapon = weapon_text(world, hit.weapon);
            killstreaks::damage_entity(
                world,
                n,
                hit.amount,
                hit.attacker,
                hit.means,
                &weapon,
                hit.point,
                hit.dir,
                hit.flags,
            );
            return true;
        }
        if breakable {
            let t = tick(world);
            crate::t5_destructible::apply_damage(
                &mut frame(world),
                t,
                crate::AuthorityModelOwner::ScriptModel(hit.target),
                hit.bone.and_then(|b| u16::try_from(b).ok()),
                hit.amount,
                destructible::kind(hit.means),
                Some(hit.dir),
                hit.attacker,
                hit.weapon,
            );
        }
        return true;
    }
    // BO2's G_DamageActor takes no damage below 1 (a Paralyzer shot: its
    // script does the harm), so no callback and no "damage".
    if hit.amount <= 0 {
        return true;
    }
    // The struck bone's hit location.
    let part = hit.bone.and_then(|bone| {
        let f = frame(world);
        f.entity_collision_capabilities()
            .iter()
            .find(|row| row.owner.script_model() == Some(hit.target))
            .and_then(|row| row.dobj.as_ref())
            .and_then(|d| d.current_collision.as_ref())
            .and_then(|c| c.bones.iter().find(|b| usize::from(b.bone) == bone))
            .map(|b| b.part_classification)
    });
    // A blast strikes no bone: BO2 calls its location "none" and its point
    // is the blast's centre, so the scripts tear off the limb nearest the
    // blast (a grenade at the feet takes the legs: a crawler).
    let blast = part.is_none() && hit.flags & 1 != 0;
    let part = part.unwrap_or(4);
    let hitloc = if blast {
        "none"
    } else {
        T6_HITLOC_NAMES
            .get(usize::from(part))
            .copied()
            .unwrap_or("none")
    };
    let point = if blast {
        std::array::from_fn(|i| hit.point[i] - hit.dir[i])
    } else {
        hit.point
    };
    // The weapon's location multiplier (not for a knife).
    let melee = hit.means == "MOD_MELEE";
    let scale = if melee {
        1.0
    } else {
        let f = frame(world);
        f.combat_facts_for(hit.weapon)
            .map_or(1.0, |facts| facts.location_scale(part))
    };
    let amount = ((hit.amount as f32) * scale).round().max(1.0) as i32;
    // A knife hit names the knife the scripts gave him, and cuts as hard as
    // that knife: BO2 stabs with the melee weapon he holds (the Bowie Knife
    // harder than the knife). A gun that is its own knife keeps its own.
    let held_knife = if melee {
        hit.attacker.and_then(|a| {
            world
                .resource::<Zm>()
                .players
                .get(&a.0)
                .and_then(|p| p.melee_weapon.clone())
        })
    } else {
        None
    };
    let amount = match held_knife.as_deref().map(|k| weapon(world, k)) {
        Some(Ok(knife)) if knife != 0 => {
            let f = frame(world);
            let own = f.weapon_melee_damage(knife);
            match f.combat_facts_for(hit.weapon) {
                Some(gun) if gun.knife_model != 0 && own > 0 => {
                    (amount - gun.melee_damage + own).max(1)
                }
                _ => amount,
            }
        }
        _ => amount,
    };
    let weapon = if melee {
        held_knife.unwrap_or_else(|| "knife_zm".to_owned())
    } else {
        weapon_text(world, hit.weapon)
    };
    if melee || std::env::var("IW4L_T6_HITLOG").is_ok() {
        diag::info!(
            Sim,
            "bo2zm t6 hit ent{n}: {} {weapon} {hitloc} x{scale:.2} = {amount}",
            hit.means
        );
    }
    with_vm(world, |vm, world| {
        let attacker = hit
            .attacker
            .and_then(|a| {
                world
                    .resource::<Zm>()
                    .players
                    .get(&a.0)
                    .map(|p| Value::Object(p.obj))
            })
            .unwrap_or(Value::Undefined);
        actors::damage(
            vm,
            world,
            n,
            attacker.clone(),
            attacker,
            amount,
            hit.flags,
            hit.means,
            &weapon,
            point,
            hit.dir,
            hitloc,
        );
    });
    true
}

/// A script string as the player reads it: a localized key (`&"KEY"` or a
/// bare `KEY`) becomes its English text with `&&1`.. filled from `args`
/// (themselves localized when they are keys); other values print as is.
pub(crate) fn localize(vm: &Vm<World>, world: &World, v: &Value, args: &[Value]) -> String {
    let zm = world.resource::<Zm>();
    let raw = vm.to_text(v);
    let key = raw.trim_start_matches('&');
    let mut out = match zm.strings.get(&key.to_ascii_uppercase()) {
        Some(t) => t.clone(),
        None if matches!(v, Value::IStr(_)) => key.to_owned(),
        None => raw.clone(),
    };
    for (i, a) in args.iter().enumerate() {
        let text = match a {
            Value::IStr(_) => {
                let k = vm.to_text(a);
                zm.strings
                    .get(&k.trim_start_matches('&').to_ascii_uppercase())
                    .cloned()
                    .unwrap_or(k)
            }
            other => vm.to_text(other),
        };
        out = out.replace(&format!("&&{}", i + 1), &text);
    }
    out
}

/// The arguments of a localized string with each player swapped for his
/// name, as BO2 prints one (Scr_ConstructMessageString: a player entity
/// becomes its display name), e.g. "&&1 is reviving you".
pub(crate) fn name_players(vm: &mut Vm<World>, world: &mut World, args: &[Value]) -> Vec<Value> {
    args.iter()
        .map(|a| match client(vm, world, a) {
            Ok(id) => {
                let name = frame(world)
                    .client_meta(id)
                    .map(|m| {
                        let end = m.name.iter().position(|&b| b == 0).unwrap_or(m.name.len());
                        String::from_utf8_lossy(&m.name[..end]).into_owned()
                    })
                    .filter(|n| !n.is_empty())
                    .unwrap_or_else(|| "Player".to_owned());
                vm.string(&name)
            }
            Err(_) => a.clone(),
        })
        .collect()
}

/// The grenades and mines a player carries, as the HUD shows them (BO2's
/// offhand icons, one per grenade): `name:count` by `;`, lethal first.
fn offhand_counts(world: &mut World, c: u32) -> String {
    const OFFHANDS: [&str; 16] = [
        "frag_grenade_zm",
        "sticky_grenade_zm",
        "cymbal_monkey_zm",
        "claymore_zm",
        // bo2mp: a multiplayer class's lethal then tactical equipment.
        "frag_grenade_mp",
        "sticky_grenade_mp",
        "hatchet_mp",
        "claymore_mp",
        "satchel_charge_mp",
        "bouncingbetty_mp",
        "flash_grenade_mp",
        "concussion_grenade_mp",
        "willy_pete_mp",
        "emp_grenade_mp",
        "trophy_system_mp",
        "tactical_insertion_mp",
    ];
    let f = frame(world);
    let id = crate::world::ClientId(c);
    let held = crate::script_player::weapons(&f, id, crate::script_player::WeaponList::All);
    let mut out: Vec<String> = Vec::new();
    for want in OFFHANDS {
        for &w in &held {
            let name = crate::script_player::weapon_name(&f, w);
            if name == want || name.trim_end_matches("_mp") == want {
                let (clip, stock) = (
                    crate::script_player::ammo_clip(&f, id, w),
                    crate::script_player::ammo_stock(&f, id, w),
                );
                if std::env::var_os("IW4L_T6_HINTLOG").is_some() {
                    diag::info!(Sim, "bo2zm t6 offhand {want}: clip {clip} stock {stock}");
                }
                out.push(format!("{want}:{}", clip + stock));
            }
        }
    }
    out.join(";")
}

/// bo2zm: the scoreboard BO2's menus show (Tab, and the game over): every
/// player as `clientnum:name:score:kills:downs:revives:headshots`, by `;`
/// (the columns Zombies' `setscoreboardcolumns` names, in its order).
fn scoreboard_rows(world: &mut World) -> String {
    let clients: Vec<u32> = world.resource::<Zm>().players.keys().copied().collect();
    let mut rows = Vec::new();
    for c in clients {
        let (name, score, kills) = {
            let f = frame(world);
            let Some(m) = f.client_meta(crate::world::ClientId(c)) else {
                continue;
            };
            let end = m.name.iter().position(|&b| b == 0).unwrap_or(m.name.len());
            let name = String::from_utf8_lossy(&m.name[..end]).replace([':', ';'], " ");
            (
                if name.is_empty() {
                    "Player".to_owned()
                } else {
                    name
                },
                m.score,
                m.kills,
            )
        };
        let zm = world.resource::<Zm>();
        let stat = |k: &str| {
            zm.players
                .get(&c)
                .and_then(|p| p.stats.get(k).copied())
                .unwrap_or(0)
        };
        rows.push(format!(
            "{c}:{name}:{score}:{kills}:{}:{}:{}",
            stat("downs"),
            stat("revives"),
            stat("headshots")
        ));
    }
    rows.join(";")
}

/// A bot (`addtestclient`) has no screen: the server builds it no HUD
/// (BO2 sends a test client none).
pub(crate) fn is_bot(world: &World, c: u32) -> bool {
    world.resource::<Zm>().players.get(&c).is_some_and(|p| p.test_client)
}

/// Each player's HUD values the scripts own go out as client dvars: the
/// round (`bo2zm_round`), the hint of the use trigger he faces
/// (`bo2zm_hint`), his grenades (`bo2zm_offhand`) and the scoreboard
/// (`bo2zm_scoreboard`); sent when they change.
fn publish_hud(world: &mut World) {
    let round = with_vm(world, |vm, _| {
        let f = vm.intern("round_number");
        vm.raw_field(vm.level, f).as_int().unwrap_or(0)
    })
    .unwrap_or(0)
    .to_string();
    // The game type (the HUD's Turned and Grief checks read it).
    let gametype = with_vm(world, |vm, _| vm.dvars.get("g_gametype").cloned())
        .flatten()
        .unwrap_or_default();
    let board = scoreboard_rows(world);
    let clients: Vec<u32> = world.resource::<Zm>().players.keys().copied().collect();
    // bo2mp: the scorestreak column's default picks.
    let picks = loadout::streak_picks(world);
    for c in clients {
        if is_bot(world, c) {
            continue;
        }
        let hint = world
            .resource::<Zm>()
            .hints
            .get(&c)
            .cloned()
            .unwrap_or_default();
        let offhand = offhand_counts(world, c);
        let slots = world
            .resource::<Zm>()
            .action_slots
            .get(&c)
            .map(|v| v.iter().map(|(n, w)| format!("{n}:{w}")).collect::<Vec<_>>().join(";"))
            .unwrap_or_default();
        // bo2mp: the scorestreak column: his picks (once) and his momentum.
        let momentum = world
            .resource::<Zm>()
            .players
            .get(&c)
            .and_then(|p| p.stats.get("momentum").copied())
            .unwrap_or(0)
            .to_string();
        // His own picks, each lit while he holds it; the defaults until his
        // class script has given him his. Sent every tick it changes, an
        // empty list too (no streaks: the column goes), so nothing of a
        // last life or class stays on his screen.
        let mut picks = with_vm(world, |vm, world| loadout::his_streaks(vm, world, c))
            .flatten()
            .unwrap_or_else(|| picks.clone());
        // bo2mp: the Tactical Nuke, once earned (`self.moab_ready`), at the
        // top of the column (dearest), lit until he calls it in.
        let nuke = with_vm(world, |vm, world| {
            let obj = world.resource::<Zm>().players.get(&c)?.obj;
            let f = vm.intern("moab_ready");
            Some(vm.raw_field(obj, f).as_int() == Some(1))
        })
        .flatten()
        .unwrap_or(false);
        if nuke {
            if !picks.is_empty() {
                picks.push(';');
            }
            picks.push_str("iw4:specialty_nuke:99999:1");
        }
        // bo2mp: the Tactical Nuke's banner: `<id>|<countdown ms>|<caller>`
        // while it is inbound and settling, else empty.
        let banner = with_vm(world, |vm, _| {
            let (b, id, ms) = (vm.intern("moab_banner"), vm.intern("moab_banner_id"), vm.intern("moab_banner_ms"));
            let name = vm.raw_field(vm.level, b);
            if name.is_undefined() {
                return Some(String::new());
            }
            let name = vm.to_text(&name);
            let id = vm.raw_field(vm.level, id).as_int().unwrap_or(0);
            let ms = vm.raw_field(vm.level, ms).as_int().unwrap_or(10_000);
            Some(format!("{id}|{ms}|{name}"))
        })
        .flatten()
        .unwrap_or_default();
        {
            let mut f = frame(world);
            if f.client_meta(crate::world::ClientId(c)).is_some() {
                let dvars = &mut f.client_meta_mut(crate::world::ClientId(c)).client_dvars;
                match dvars.iter_mut().find(|(k, _)| k == "bo2mp_nuke") {
                    Some(row) if row.1 != banner => row.1 = banner.clone(),
                    Some(_) => {}
                    None if !banner.is_empty() => dvars.push(("bo2mp_nuke".to_owned(), banner.clone())),
                    None => {}
                }
                let shown = dvars.iter().any(|(k, _)| k == "bo2mp_streaks");
                if !picks.is_empty() || shown {
                    for (name, value) in [("bo2mp_streaks", picks.clone()), ("bo2mp_momentum", momentum)] {
                        match dvars.iter_mut().find(|(k, _)| k == name) {
                            Some(row) if row.1 != value => row.1 = value,
                            Some(_) => {}
                            None => dvars.push((name.to_owned(), value)),
                        }
                    }
                }
            }
        }
        let sent = world.resource::<Zm>().hud_sent.get(&c).cloned();
        if sent.as_ref().is_some_and(|(r, h, o, a, b)| {
            *r == round && *h == hint && *o == offhand && *a == slots && *b == board
        }) {
            continue;
        }
        let mut f = frame(world);
        if f.client_meta(crate::world::ClientId(c)).is_none() {
            continue;
        }
        if std::env::var_os("IW4L_T6_HINTLOG").is_some() {
            diag::info!(
                Sim,
                "bo2zm t6 hud out for {c}: round {round:?} hint {hint:?} offhand {offhand:?} scoreboard {board:?}"
            );
        }
        let dvars = &mut f.client_meta_mut(crate::world::ClientId(c)).client_dvars;
        for (name, value) in [
            ("bo2zm_round", round.clone()),
            ("bo2zm_hint", hint.clone()),
            ("bo2zm_offhand", offhand.clone()),
            ("bo2zm_actionslots", slots.clone()),
            ("bo2zm_scoreboard", board.clone()),
            ("bo2zm_gametype", gametype.clone()),
        ] {
            match dvars.iter_mut().find(|(k, _)| k == name) {
                Some(row) => row.1 = value,
                None => dvars.push((name.to_owned(), value)),
            }
        }
        world
            .resource_mut::<Zm>()
            .hud_sent
            .insert(c, (round.clone(), hint, offhand, slots, board.clone()));
    }
}

/// Put a player somewhere from the server (a script's `setorigin`, a
/// link): the teleport bit flips so his client jumps there too.
pub(crate) fn teleport_player(world: &mut World, client: crate::world::ClientId, origin: [f32; 3]) {
    let mut f = frame(world);
    f.set_origin(client, origin);
    if let Some(ps) = f.player_mut(client) {
        ps.e_flags ^= playerstate_iw4::eflags::TELEPORT;
    }
}

/// Turn a player's view from the server (`setplayerangles`): the delta
/// under his last command's angles, so his client's view turns too.
pub(crate) fn set_player_view(world: &mut World, client: crate::world::ClientId, view: [f32; 3]) {
    let mut f = frame(world);
    let cmd = f
        .old_cmd_angles_mut()
        .iter()
        .find(|(c, _)| *c == client)
        .map(|(_, a)| *a);
    if let Some(ps) = f.player_mut(client) {
        if let Some(cmd) = cmd {
            ps.delta_angles =
                std::array::from_fn(|i| view[i] - (cmd[i] as u16) as f32 * (360.0 / 65536.0));
        }
        ps.viewangles = view;
    }
}

/// bo2zm M3: the living zombies a blast at `origin` reaches: (their shown
/// model, their middle, distance), nearest first. As BO2's engine does, the
/// distance is to the nearest point of each one's box, not its middle: the
/// Sliquifier's bolt (radius 25) goes off on the floor at a zombie's feet.
pub(crate) fn radius_targets(
    world: &mut World,
    origin: [f32; 3],
    radius: f32,
) -> Vec<(crate::ScriptModelId, [f32; 3], f32)> {
    if !world.contains_resource::<T6Runtime>() {
        return Vec::new();
    }
    let zm = world.resource::<Zm>();
    let mut out: Vec<(crate::ScriptModelId, [f32; 3], f32)> = zm
        .actors
        .iter()
        .filter(|(_, a)| a.alive)
        .filter_map(|(n, a)| {
            let e = zm.ents.get(n)?;
            let id = zm.presences.by_ent.get(n)?.id;
            let o = e.origin;
            let mid = [o[0], o[1], o[2] + a.height * 0.5];
            let lo = [o[0] - a.radius, o[1] - a.radius, o[2]];
            let hi = [o[0] + a.radius, o[1] + a.radius, o[2] + a.height];
            let near: [f32; 3] = std::array::from_fn(|i| origin[i].clamp(lo[i], hi[i]));
            let d = gsc_t6::math::length(gsc_t6::math::sub(near, origin));
            (d < radius).then_some((id, mid, d))
        })
        .collect();
    // bo2mp: and the map's breakables, measured to their nearest part.
    out.extend(destructible::radius_targets(world, origin, radius));
    // bo2mp: and the scorestreaks' planes, drones, vehicles and turrets.
    let streak_ents: Vec<(crate::ScriptModelId, [f32; 3], f32)> = {
        let zm = world.resource::<Zm>();
        zm.presences
            .by_ent
            .iter()
            .filter(|(n, _)| !zm.actors.contains_key(n) && killstreaks::damage_target(world, **n))
            .filter_map(|(n, p)| {
                let at = zm.ents.get(n)?.origin;
                let d = gsc_t6::math::length(gsc_t6::math::sub(at, origin));
                (d < radius).then_some((p.id, at, d))
            })
            .collect()
    };
    out.extend(streak_ents);
    out.sort_by(|a, b| a.2.total_cmp(&b.2));
    out
}

// ---- helpers for natives -------------------------------------------------

pub(crate) fn arg(a: &[Value], i: usize) -> &Value {
    a.get(i).unwrap_or(&Value::Undefined)
}

pub(crate) fn num(a: &[Value], i: usize) -> Result<f32, String> {
    arg(a, i).as_float().ok_or_else(|| {
        format!(
            "argument {} is {}, not a number",
            i + 1,
            arg(a, i).type_name()
        )
    })
}

pub(crate) fn int(a: &[Value], i: usize) -> Result<i32, String> {
    arg(a, i).as_int().ok_or_else(|| {
        format!(
            "argument {} is {}, not a number",
            i + 1,
            arg(a, i).type_name()
        )
    })
}

pub(crate) fn vec3(a: &[Value], i: usize) -> Result<[f32; 3], String> {
    arg(a, i).as_vec3().ok_or_else(|| {
        format!(
            "argument {} is {}, not a vector",
            i + 1,
            arg(a, i).type_name()
        )
    })
}

pub(crate) fn text(vm: &Vm<World>, a: &[Value], i: usize) -> String {
    match arg(a, i) {
        Value::Undefined => String::new(),
        v => vm.to_text(v),
    }
}

pub(crate) fn flag(a: &[Value], i: usize, default: bool) -> bool {
    match arg(a, i) {
        Value::Undefined => default,
        v => gsc_t6::truthy(v),
    }
}

pub(crate) fn list(vals: Vec<Value>) -> Value {
    let mut a = gsc_t6::Array::new();
    for v in vals {
        a.push(v);
    }
    Value::array(a)
}

/// The entity number of an entity object.
pub(crate) fn entnum(vm: &Vm<World>, v: &Value) -> Option<u32> {
    match v {
        Value::Object(o) => match vm.kind(*o)? {
            ObjKind::Entity(n) => Some(n),
            _ => None,
        },
        _ => None,
    }
}

/// The client of a player object.
pub(crate) fn client(vm: &Vm<World>, world: &World, v: &Value) -> Result<ClientId, String> {
    let n = entnum(vm, v).ok_or("not an entity")?;
    if world.resource::<Zm>().players.contains_key(&n) {
        Ok(ClientId(n))
    } else {
        Err("not a player".into())
    }
}

pub(crate) fn is_player(vm: &Vm<World>, world: &World, v: &Value) -> bool {
    entnum(vm, v).is_some_and(|n| world.resource::<Zm>().players.contains_key(&n))
}

pub(crate) fn tick(world: &World) -> crate::Tick {
    world.resource::<crate::step::StepRequest>().tick
}

pub(crate) fn frame(world: &mut World) -> FrameWorld<'_> {
    FrameWorld::from_world(world)
}

/// An entity's origin (players from the sim).
pub(crate) fn origin_of(vm: &Vm<World>, world: &mut World, v: &Value) -> Option<[f32; 3]> {
    let n = entnum(vm, v)?;
    if world.resource::<Zm>().players.contains_key(&n) {
        return frame(world).player(ClientId(n)).map(|ps| ps.origin);
    }
    world.resource::<Zm>().ents.get(&n).map(|e| e.origin)
}

/// Weapon index by script name (`m1911_zm`; IW4L registers `_mp` names).
pub(crate) fn weapon(world: &mut World, name: &str) -> Result<u32, String> {
    resolve_weapon(world, name).map(|(w, _)| w)
}

/// `weapon`, and when the weapon is not the one the name asks for, what
/// stood in for it and why.
///
/// bo2mp: a gun with attachments (`mk48_mp+reflex+fmj`) is that name when
/// the catalog has it, else its base gun with its sight, if it has one
/// (`mk48_mp+reflex`, the scope lane's catalog weapon: sight model, ADS,
/// zoom, overlay; its other attachments are not drawn or modelled yet),
/// else the base gun. Another game's gun (`bo1_ak74u_mp+reflex+grip`,
/// `gh_ak12_mp+acog`): its own variant with those attachments, in any
/// order, else the nearest one, else its base gun. Only weapons a player
/// can be given (with combat data) stand in.
pub(crate) fn resolve_weapon(world: &mut World, name: &str) -> Result<(u32, Option<String>), String> {
    if name == "none" || name.is_empty() {
        return Ok((0, None));
    }
    let f = frame(world);
    let usable = |w: u32| w != 0 && (f.combat_facts_for(w).is_some() || f.weapon_is_melee_only(w));
    let Some((base, atts)) = name.split_once('+') else {
        return crate::script_player::weapon_named(&f, &format!("{name}_mp"))
            .or_else(|_| crate::script_player::weapon_named(&f, name))
            .map(|w| (w, None));
    };
    let exact = crate::script_player::weapon_named(&f, name).ok();
    if let Some(w) = exact.filter(|w| usable(*w)) {
        return Ok((w, None));
    }
    let base = if base.ends_with("_mp") { base.to_owned() } else { format!("{base}_mp") };
    let why = match exact {
        Some(_) => "it has no combat data",
        None => "the catalog has no such variant",
    };
    if bo2_profile::guest::is_guest_name(name) {
        if let Some(w) = f.guest_weapon_index_where(name, usable) {
            let stand_in = crate::script_player::weapon_name(&f, w);
            return Ok((w, Some(format!("`{name}`: {why}; nearest variant `{stand_in}`"))));
        }
        return Err(format!("unknown weapon '{name}' (no variant of `{base}` can be given)"));
    }
    // bo2mp: Select Fire with a sight (`an94_mp+sf+reflex`, any order):
    // the catalog names it `<gun>+<sight>+sf` (or `<gun>+sf`).
    let parts: Vec<&str> = atts.split('+').filter(|a| !a.is_empty()).collect();
    if parts.contains(&"sf") {
        let mut tries: Vec<String> = parts
            .iter()
            .filter(|a| **a != "sf")
            .map(|a| format!("{base}+{a}+sf"))
            .collect();
        tries.push(format!("{base}+sf"));
        for t in tries {
            if let Ok(w) = crate::script_player::weapon_named(&f, &t)
                && usable(w)
            {
                let note = (t != name).then(|| format!("`{name}`: {why}; its Select Fire variant `{t}`"));
                return Ok((w, note));
            }
        }
    }
    for att in parts {
        let sight = format!("{base}+{att}");
        if let Ok(w) = crate::script_player::weapon_named(&f, &sight)
            && usable(w)
        {
            return Ok((w, Some(format!("`{name}`: {why}; its `{att}` variant `{sight}`"))));
        }
    }
    match crate::script_player::weapon_named(&f, &base).ok().filter(|w| usable(*w)) {
        Some(w) => Ok((w, Some(format!("`{name}`: {why}; the base gun `{base}`")))),
        None => Err(format!("unknown weapon '{name}'")),
    }
}

/// A weapon's world model as everyone else sees it (on a wall, in the
/// box): the gun, the parts Black Ops II hangs on it (magazine, scope,
/// rocket: its `attachWorldModel`s) and the tags it hides, lower case.
#[derive(Clone, Debug, Default, PartialEq)]
pub(crate) struct WeaponLook {
    pub model: String,
    pub parts: Vec<(String, String, xmodel_runtime::TagOffset)>,
    pub hide: Vec<String>,
}

pub(crate) fn weapon_look(world: &mut World, w: u32) -> Option<WeaponLook> {
    let f = frame(world);
    let (model, hide) = f.weapon_world_model(w)?;
    if model.is_empty() {
        return None;
    }
    Some(WeaponLook {
        model: model.to_owned(),
        // Only parts the map loaded: one it lacks would lose the whole gun.
        parts: f
            .weapon_world_parts(w)
            .iter()
            .filter(|p| f.model_capability(&p.model).flatten().is_some())
            .map(|p| {
                let at = xmodel_runtime::TagOffset::new(p.offset, p.rotation);
                (p.model.clone(), p.tag.clone(), at)
            })
            .collect(),
        hide: hide.iter().map(|t| t.to_ascii_lowercase()).collect(),
    })
}

/// A weapon's script name: Zombies' `m1911_zm` (the engine's `_mp`
/// dropped); multiplayer's own names end in `_mp` (`mp7_mp`, `radar_mp`),
/// as BO2's scripts compare them.
pub(crate) fn weapon_text(world: &mut World, w: u32) -> String {
    let mp = world.get_resource::<Zm>().is_some_and(|z| z.mp);
    let f = frame(world);
    let n = crate::script_player::weapon_name(&f, w);
    if mp {
        return n;
    }
    n.strip_suffix("_mp").map_or(n.clone(), str::to_owned)
}

pub(crate) fn names(set: &BTreeSet<String>) -> String {
    set.iter().cloned().collect::<Vec<_>>().join(", ")
}

/// bo2mp: a map with no path nodes of its own (one from another game:
/// MW2's and MW3's multiplayer maps carry none) takes the walk graph the
/// engine baked for it as Black Ops II path nodes, so BO2's bots can route
/// over it. Nothing changes when the map has its own. True when installed.
pub(crate) fn adopt_path_nodes(world: &mut World, nodes: Vec<T6PathNode>) -> bool {
    if nodes.is_empty() || world.get_resource::<Zm>().is_none_or(|zm| !zm.nav.nodes.is_empty()) {
        return false;
    }
    let node_objs = with_vm(world, |vm, _| node_objects(vm, &nodes)).unwrap_or_default();
    let count = nodes.len();
    let mut zm = world.resource_mut::<Zm>();
    zm.nav = nav::Nav::new(nodes);
    zm.node_of = node_objs.iter().enumerate().map(|(i, o)| (*o, i as u32)).collect();
    zm.node_objs = node_objs;
    diag::info!(Sim, "bo2mp nav: {count} path nodes from the engine's walk graph (the map has none of its own)");
    true
}

/// bo2mp: BO2's scripts run on this map and it has no path nodes yet.
pub(crate) fn needs_path_nodes(world: &World) -> bool {
    world.get_resource::<Zm>().is_some_and(|zm| zm.nav.nodes.is_empty())
}
