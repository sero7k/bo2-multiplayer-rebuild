//! bo2mp: Infinite Warfare's Zombies maps (`cp_zmb`, `cp_rave`, ...) played
//! as Black Ops II Zombies. Not part of upstream IW4L.
//!
//! The map's own lane (`super::iw7`) reads the world and collision; here
//! its zombie setup (`fastfile_iw7::zombies::ZmLayout`: areas, spawners,
//! windows, debris, machines, Magic Wheel spots, Pack-a-Punch, wall
//! weapons, generators, portals) is written as the entities Black Ops II's
//! Zombies scripts look for, and Black Ops II's Zombies zones are loaded
//! beside it the way a Black Ops II Zombies map loads them. The scripts
//! that run it are Black Ops II's own; the map script is ours
//! (`sim::t6::foreign_zm`), and plays on Buried's Zombies kit (its zombies,
//! window barriers, Mystery Box, perk machines, wall weapons and box guns:
//! [`DONOR`]).
//!
//! What maps onto what:
//! - an area (`spawn_volume`) is a zone: an `info_volume` named after it
//!   with the area's trigger hulls (`?N`), its spawners as the zone's spawn
//!   structs (`riser_location` when they climb out of the ground, else
//!   `spawn_location`; one that comes in through a window names it);
//! - a window (`window_entrance`) is a boarded barrier: `exterior_goal`,
//!   its zbarrier (Buried's wooden boards) and the mantle traversal in;
//! - debris and team doors are `zombie_debris` (their cost; a team door,
//!   free in Infinite Warfare, costs 1000) whose flag opens the zones on
//!   both sides; their brushes are the clip;
//! - perk machines are Black Ops II's machines on their spots (by what
//!   the perk does; perks Black Ops II has none like are left out),
//!   Pack-a-Punch Black Ops II's;
//! - each Magic Wheel spot is a Mystery Box spot (the start one nearest the
//!   players' starts), each wall weapon a wall buy of a Black Ops II gun of
//!   its kind;
//! - generators are power switches (any one turns the power on);
//! - portals take players to the main street's portal, and that one to
//!   Pack-a-Punch's room (for 30 s) once the power is on.
//!
//! IW7 routes its AI over a nav mesh; Black Ops II's zombies walk path
//! nodes, laid out over the collision here (`asset_world::nav_soup`), with
//! each window's mantle and the map's own traversals as negotiation links.

use std::collections::{BTreeMap, HashMap};
use std::fmt::Write as _;
use std::sync::Mutex;

use asset_world::nav_soup::{WalkNode, WalkSoup};
use asset_world::world_iw7::SoupTri;
use fastfile_iw7::map::Iw7Map;
use fastfile_iw7::zombies::{BlockerKind, Spot, ZmLayout};

use super::LoadedWorld;

/// The Black Ops II Zombies map whose zones (its zombies, barriers, box,
/// machines and guns) these maps play with.
pub(crate) const DONOR: &str = "zm_buried";

/// The start location the map's structs name (`zclassic_perks_iw7`...);
/// the map script registers it (`sim::t6::foreign_zm`).
pub(crate) const LOCATION: &str = "iw7";

/// Walk graph node spacing and most nodes.
const NAV_SPACING: f32 = 96.0;
const NAV_MAX_NODES: usize = 40_000;

/// A Zombies map read by the lane, for the rules attached after it.
pub(crate) struct Pending {
    pub map: String,
    pub layout: ZmLayout,
    pub walk: Vec<WalkNode>,
    /// Where players land in Pack-a-Punch's room.
    pub pap_room: Vec<[f32; 3]>,
}

static PENDING: Mutex<Option<Pending>> = Mutex::new(None);

/// Other lanes' Zombies maps played the same way (registered by name).
static REGISTERED: Mutex<Vec<String>> = Mutex::new(Vec::new());

/// bo2mp: another lane's Zombies map (Black Ops III's) that plays through
/// this pipeline: [`prepare_layout`], then [`attach_t6_zm_rules`]. The
/// sim's side is `sim::register_foreign_zm_map`.
#[allow(dead_code)]
pub(crate) fn register_foreign_zm_map(name: &str) {
    let mut g = REGISTERED.lock().unwrap_or_else(std::sync::PoisonError::into_inner);
    if !g.iter().any(|n| n == name) {
        g.push(name.to_owned());
    }
}

/// An Infinite Warfare Zombies map (`cp_*`), or one registered.
pub(crate) fn is_iw7_zombies_map(map: &str) -> bool {
    map.starts_with("cp_") || REGISTERED.lock().unwrap_or_else(std::sync::PoisonError::into_inner).iter().any(|n| n == map)
}

/// The lane read a Zombies map: its layout and walk graph, kept for
/// [`attach_t6_zm_rules`]. Report lines.
pub(super) fn prepare(map_name: &str, map: &Iw7Map, tris: &[SoupTri]) -> Vec<String> {
    prepare_layout(map_name, ZmLayout::read(map), tris)
}

/// [`prepare`] for a layout another lane read (its own map's entities in
/// `ZmLayout`'s terms) over its collision triangles.
pub(super) fn prepare_layout(map_name: &str, layout: ZmLayout, tris: &[SoupTri]) -> Vec<String> {
    let started = std::time::Instant::now();
    let mut report = vec![format!("iw7 zombies: {}", layout.summary())];
    let soup = WalkSoup::new(tris);
    let mut seeds: Vec<[f32; 3]> = Vec::new();
    seeds.extend(layout.starts.iter().map(|s| s.origin));
    seeds.extend(layout.respawns.iter().map(|s| s.origin));
    for a in &layout.areas {
        seeds.extend(a.spawners.iter().map(|s| s.spot.origin));
        seeds.extend(a.spawners.iter().filter_map(|s| s.goal));
    }
    for w in &layout.windows {
        seeds.push(w.outside.origin);
        seeds.extend(w.inside.map(|s| s.origin));
    }
    seeds.extend(layout.perks.iter().map(|m| m.spot.origin));
    seeds.extend(layout.wheels.iter().map(|s| s.origin));
    seeds.extend(layout.wall_buys.iter().map(|w| w.spot.origin));
    for p in &layout.portals {
        seeds.push(p.spot.origin);
        seeds.extend(p.dests.iter().map(|d| d.origin));
    }
    for t in &layout.traversals {
        seeds.push(t.begin);
        seeds.push(t.end);
    }
    // Pack-a-Punch's room: a few floor spots beside the machine, toward the
    // room's middle.
    let mut pap_room = Vec::new();
    if let Some(pap) = layout.pap {
        let mid = layout.area_at(pap.origin).and_then(|a| a.boxes.first()).map(|(lo, hi)| std::array::from_fn::<f32, 3, _>(|k| (lo[k] + hi[k]) * 0.5));
        let toward = mid.map_or([1.0, 0.0], |m| {
            let (dx, dy) = (m[0] - pap.origin[0], m[1] - pap.origin[1]);
            let len = (dx * dx + dy * dy).sqrt().max(1.0);
            [dx / len, dy / len]
        });
        for (along, side) in [(96.0, 0.0), (96.0, 40.0), (96.0, -40.0), (136.0, 0.0)] {
            let p = [pap.origin[0] + toward[0] * along - toward[1] * side, pap.origin[1] + toward[1] * along + toward[0] * side, pap.origin[2] + 40.0];
            if let Some(z) = soup.floor_under(p, 160.0) {
                pap_room.push([p[0], p[1], z]);
            }
        }
        seeds.push(pap.origin);
        seeds.extend(pap_room.iter().copied());
    }
    let walk = soup.graph(&seeds, NAV_SPACING, NAV_MAX_NODES);
    report.push(format!(
        "iw7 zombies walk graph: {} nodes, {} links from {} seeds ({:.1}s); Pack-a-Punch room spots {}",
        walk.len(),
        walk.iter().map(|n| n.links.len()).sum::<usize>(),
        seeds.len(),
        started.elapsed().as_secs_f32(),
        pap_room.len()
    ));
    *PENDING.lock().unwrap_or_else(std::sync::PoisonError::into_inner) = Some(Pending { map: map_name.to_owned(), layout, walk, pap_room });
    report
}

fn take_pending(map: &str) -> Option<Pending> {
    let mut g = PENDING.lock().unwrap_or_else(std::sync::PoisonError::into_inner);
    if g.as_ref().is_some_and(|p| p.map == map) { g.take() } else { None }
}

// ---------------------------------------------------------------------------
// The entities.

/// An entity block being written.
#[derive(Default)]
struct Block(Vec<(String, String)>);

impl Block {
    fn new(classname: &str) -> Self {
        let mut b = Block::default();
        b.set("classname", classname);
        b
    }

    fn set(&mut self, k: &str, v: impl Into<String>) -> &mut Self {
        let v = v.into();
        match self.0.iter_mut().find(|(key, _)| key == k) {
            Some(slot) => slot.1 = v,
            None => self.0.push((k.to_owned(), v)),
        }
        self
    }

    fn remove(&mut self, k: &str) -> &mut Self {
        self.0.retain(|(key, _)| key != k);
        self
    }

    fn at(&mut self, origin: [f32; 3], yaw: f32) -> &mut Self {
        self.set("origin", vec3(origin));
        self.set("angles", format!("0 {} 0", num(yaw)))
    }

    fn get(&self, k: &str) -> Option<&str> {
        self.0.iter().find(|(key, _)| key == k).map(|(_, v)| v.as_str())
    }

    fn write(&self, out: &mut String) {
        out.push_str("{\n");
        for (k, v) in &self.0 {
            let _ = writeln!(out, "\"{k}\" \"{v}\"");
        }
        out.push_str("}\n");
    }
}

fn num(v: f32) -> String {
    let r = (v * 100.0).round() / 100.0;
    if r == r.trunc() { format!("{}", r as i64) } else { format!("{r}") }
}

fn vec3(p: [f32; 3]) -> String {
    format!("{} {} {}", num(p[0]), num(p[1]), num(p[2]))
}

/// A map entity text's blocks.
fn parse_blocks(text: &str) -> Vec<Block> {
    let mut out = Vec::new();
    let mut cur: Option<Block> = None;
    let mut quoted: Vec<String> = Vec::new();
    let mut chars = text.chars();
    while let Some(c) = chars.next() {
        match c {
            '{' => {
                cur = Some(Block::default());
                quoted.clear();
            }
            '}' => {
                out.extend(cur.take());
                quoted.clear();
            }
            '"' => {
                let s: String = chars.by_ref().take_while(|&c| c != '"').collect();
                quoted.push(s);
                if quoted.len() == 2 {
                    let v = quoted.pop().unwrap_or_default();
                    let k = quoted.pop().unwrap_or_default();
                    if let Some(b) = cur.as_mut() {
                        b.0.push((k, v));
                    }
                }
            }
            _ => {}
        }
    }
    out
}

/// What the donor map's entities lend: its zombie spawners and the
/// barrier and Mystery Box zbarriers' keys.
#[derive(Default)]
pub(crate) struct DonorKit {
    spawners: Vec<Block>,
    window: Option<Block>,
    box_barrier: Option<Block>,
}

impl DonorKit {
    pub(crate) fn from_entities(texts: &[String]) -> Self {
        let mut kit = DonorKit::default();
        for b in texts.iter().flat_map(|t| parse_blocks(t)) {
            let class = b.get("classname").unwrap_or("").to_owned();
            if class.starts_with("actor_")
                && b.get("script_noteworthy") == Some("zombie_spawner")
                && !class.contains("ghost")
                && !class.contains("sloth")
            {
                kit.spawners.push(b);
            } else if class.eq_ignore_ascii_case("zbarrier_zmcore_BasicWoodBarrier") && kit.window.is_none() {
                kit.window = Some(b);
            } else if class.eq_ignore_ascii_case("zbarrier_zmcore_MagicBox") && kit.box_barrier.is_none() {
                kit.box_barrier = Some(b);
            }
        }
        for b in kit.spawners.iter_mut().chain(kit.window.iter_mut()).chain(kit.box_barrier.iter_mut()) {
            b.remove("guid").remove("export").remove("script_location");
        }
        kit
    }
}

/// An Infinite Warfare perk machine as Black Ops II's machine that does
/// the same: (perk, machine model). `None`: Black Ops II has no such perk
/// on the donor's machines (Bombstoppers, Blue Bolts, Slappy Taffy, Trail
/// Blazers).
fn bo2_perk(iw7_machine: &str) -> Option<(&'static str, &'static str)> {
    Some(match iw7_machine.strip_prefix("perk_machine_")? {
        // Tuff 'Nuff: more health.
        "tough" => ("specialty_armorvest", "zombie_vending_jugg"),
        // Up 'n Atoms: revive.
        "revive" => ("specialty_quickrevive", "zombie_vending_revive"),
        // Quickies: faster reloads.
        "flash" => ("specialty_fastreload", "zombie_vending_sleight"),
        // Bang Bangs: faster fire.
        "rat_a_tat" => ("specialty_rof", "zombie_vending_doubletap2"),
        // Racin' Stripes: longer sprint.
        "run" => ("specialty_longersprint", "zombie_vending_marathon"),
        // Mule Munchies: a third weapon.
        "more" => ("specialty_additionalprimaryweapon", "zombie_vending_three_gun"),
        _ => return None,
    })
}

/// An Infinite Warfare wall weapon as a Black Ops II wall buy of its kind:
/// (weapon, world model).
fn bo2_wall_weapon(iw7_weapon: &str) -> Option<(&'static str, &'static str)> {
    let base = iw7_weapon.strip_prefix("iw7_")?.split(['+']).next()?;
    let base = base.rsplit_once("_zm").map_or(base, |(b, _)| b);
    Some(match base {
        // The start room's: the revolver and the M1 as Black Ops II's
        // start wall guns.
        "revolver" => ("rottweil72_zm", "t6_wpn_shotty_olympia_world"),
        "m1c" => ("m14_zm", "t6_wpn_ar_m14_world"),
        // Pistols.
        "g18" | "g18c" | "mag" => ("beretta93r_zm", "t6_wpn_pistol_b2023r_world"),
        // Shotguns: pump and auto, double barrel.
        "sonic" | "spas" | "spasc" | "sdfshotty" => ("870mcs_zm", "t6_wpn_shotty_870mcs_world"),
        "devastator" | "crdb" => ("rottweil72_zm", "t6_wpn_shotty_olympia_world"),
        // Rifles.
        "m4" => ("m16_zm", "t6_wpn_ar_m16a2_world"),
        "nrg" | "ar57" | "arclassic" | "sdfar" | "m8" => ("an94_zm", "t6_wpn_ar_an94_world"),
        // Submachine guns.
        "erad" | "fmg" => ("pdw57_zm", "t6_wpn_smg_pdw57_world"),
        "ake" | "ripper" => ("ak74u_zm", "t6_wpn_smg_ak74u_world"),
        "crb" | "ump45" | "ump45c" | "udm45" | "mp28" => ("mp5k_zm", "t6_wpn_smg_mp5_world"),
        // Machine guns.
        "rvn" => ("lsat_zm", "t6_wpn_lmg_lsat_world"),
        // Marksman and sniper rifles, the Forge Freeze.
        "kbs" | "cheytacc" | "longshot" | "chargeshot" | "forgefreeze" => ("svu_zm", "t6_wpn_sniper_svu_world"),
        // Melee weapons, launchers and the maps' own specials: none.
        _ => return None,
    })
}

/// A traversal's Black Ops II animscript by how far it climbs or drops
/// (the donor's `maps/mp/animscripts/traverse/zm_*` and patch_zm's mantle).
fn bo2_traverse(dz: f32) -> &'static str {
    const DOWN: [(f32, &str); 8] = [
        (48.0, "zm_jump_down_48"),
        (72.0, "zm_jump_down_72"),
        (96.0, "zm_jump_down_96"),
        (127.0, "zm_jump_down_127"),
        (176.0, "zm_jump_down_176"),
        (190.0, "zm_jump_down_190"),
        (222.0, "zm_jump_down_222"),
        (240.0, "zm_jump_down_240"),
    ];
    const UP: [(f32, &str); 4] = [(96.0, "zm_jump_up_96"), (127.0, "zm_jump_up_127"), (154.0, "zm_jump_up_154"), (222.0, "zm_jump_up_222")];
    let nearest = |list: &[(f32, &'static str)], h: f32| list.iter().min_by(|a, b| (a.0 - h).abs().total_cmp(&(b.0 - h).abs())).map_or("zm_mantle_over_40", |e| e.1);
    if dz < -36.0 {
        nearest(&DOWN, -dz)
    } else if dz > 72.0 {
        nearest(&UP, dz)
    } else {
        "zm_mantle_over_40"
    }
}

fn yaw_dir(yaw: f32) -> [f32; 3] {
    let r = yaw.to_radians();
    [r.cos(), r.sin(), 0.0]
}

fn add(a: [f32; 3], b: [f32; 3], s: f32) -> [f32; 3] {
    [a[0] + b[0] * s, a[1] + b[1] * s, a[2] + b[2] * s]
}

fn dist(a: [f32; 3], b: [f32; 3]) -> f32 {
    ((a[0] - b[0]).powi(2) + (a[1] - b[1]).powi(2) + (a[2] - b[2]).powi(2)).sqrt()
}

/// The translation: the entity text Black Ops II's scripts read and the
/// AI's path nodes.
pub(crate) struct Translation {
    pub entities: String,
    pub path_nodes: Vec<asset_t6::PathNodeRef>,
    pub report: Vec<String>,
}

/// Path nodes being laid out: the walk graph, then negotiation pairs
/// joined to it.
struct Nodes {
    nodes: Vec<asset_t6::PathNodeRef>,
    grid: HashMap<(i32, i32), Vec<u32>>,
}

const GRID: f32 = 128.0;
const NODE_PATH: u32 = 1;
const NODE_BEGIN: u32 = 17;
const NODE_END: u32 = 18;

impl Nodes {
    fn from_walk(walk: &[WalkNode]) -> Self {
        let mut out = Nodes { nodes: Vec::with_capacity(walk.len()), grid: HashMap::new() };
        for (i, n) in walk.iter().enumerate().take(usize::from(u16::MAX) - 512) {
            let links = n
                .links
                .iter()
                .filter(|&&l| (l as usize) < usize::from(u16::MAX) - 512)
                .map(|&l| (l as u16, dist(n.origin, walk[l as usize].origin), false))
                .collect();
            out.nodes.push(asset_t6::PathNodeRef { ty: NODE_PATH, origin: n.origin, links, ..Default::default() });
            out.grid.entry(Self::cell(n.origin)).or_default().push(i as u32);
        }
        out
    }

    fn cell(p: [f32; 3]) -> (i32, i32) {
        ((p[0] / GRID).floor() as i32, (p[1] / GRID).floor() as i32)
    }

    /// The nearest walk nodes to `p` within `reach` (and a step's height),
    /// on the side `side` points to (when given), nearest first.
    fn near(&self, p: [f32; 3], reach: f32, side: Option<[f32; 3]>, n: usize) -> Vec<u32> {
        let (cx, cy) = Self::cell(p);
        let r = (reach / GRID).ceil() as i32;
        let mut found: Vec<(f32, u32)> = Vec::new();
        for x in cx - r..=cx + r {
            for y in cy - r..=cy + r {
                for &i in self.grid.get(&(x, y)).into_iter().flatten() {
                    let o = self.nodes[i as usize].origin;
                    let d = dist(o, p);
                    if self.nodes[i as usize].ty != NODE_PATH || d > reach || (o[2] - p[2]).abs() > 48.0 {
                        continue;
                    }
                    if let Some(s) = side
                        && (o[0] - p[0]) * s[0] + (o[1] - p[1]) * s[1] < 0.0
                    {
                        continue;
                    }
                    found.push((d, i));
                }
            }
        }
        found.sort_by(|a, b| a.0.total_cmp(&b.0));
        found.into_iter().take(n).map(|(_, i)| i).collect()
    }

    fn link(&mut self, a: u32, b: u32, negotiation: bool) {
        let d = dist(self.nodes[a as usize].origin, self.nodes[b as usize].origin);
        let from = &mut self.nodes[a as usize];
        if !from.links.iter().any(|l| u32::from(l.0) == b) {
            from.links.push((b as u16, d, negotiation));
        }
    }

    /// A negotiation pair (begin -> end with `animscript`), joined to the
    /// walk nodes before its begin and after its end. False when the node
    /// table is full.
    fn negotiation(&mut self, name: &str, begin: [f32; 3], end: [f32; 3], animscript: &str) -> bool {
        if self.nodes.len() + 2 > usize::from(u16::MAX) {
            return false;
        }
        let dir = {
            let (dx, dy) = (end[0] - begin[0], end[1] - begin[1]);
            let len = (dx * dx + dy * dy).sqrt().max(1e-3);
            [dx / len, dy / len, 0.0]
        };
        let before = self.near(begin, 160.0, Some([-dir[0], -dir[1], 0.0]), 3);
        let after = self.near(end, 160.0, Some(dir), 3);
        let b = self.nodes.len() as u32;
        let yaw = dir[1].atan2(dir[0]).to_degrees();
        self.nodes.push(asset_t6::PathNodeRef {
            ty: NODE_BEGIN,
            targetname: name.to_owned(),
            target: format!("{name}_end"),
            animscript: animscript.to_owned(),
            origin: begin,
            angle: yaw,
            ..Default::default()
        });
        let e = b + 1;
        self.nodes.push(asset_t6::PathNodeRef { ty: NODE_END, targetname: format!("{name}_end"), origin: end, angle: yaw, ..Default::default() });
        self.link(b, e, true);
        for n in before {
            self.link(n, b, false);
            self.link(b, n, false);
        }
        for n in after {
            self.link(e, n, false);
            self.link(n, e, false);
        }
        true
    }
}

/// The map's layout as Black Ops II Zombies' entities and path nodes.
pub(crate) fn translate(p: &Pending, kit: &DonorKit) -> Translation {
    let l = &p.layout;
    let mut report = Vec::new();
    let mut ents: Vec<Block> = Vec::new();
    let mut nodes = Nodes::from_walk(&p.walk);
    let loc = LOCATION;
    let zone_of = |at: [f32; 3]| l.nearest_area(at).map(|a| a.name.clone());

    let mut world = Block::new("worldspawn");
    world.set("origin", "0 0 0");
    ents.push(world);

    // The zombies (Buried's), placed where the map's first start is (they
    // spawn from the zones' structs).
    let base = l.starts.first().map_or([0.0; 3], |s| s.origin);
    for (i, s) in kit.spawners.iter().enumerate() {
        let mut b = Block(s.0.clone());
        b.set("origin", vec3([base[0] + 32.0 * i as f32, base[1], base[2] - 2000.0]));
        ents.push(b);
    }
    if kit.spawners.is_empty() {
        report.push(format!("iw7 zombies gap: no zombie spawners in {DONOR}'s entities: no zombies"));
    }

    // Player starts and respawns.
    for (i, s) in l.starts.iter().enumerate() {
        let mut b = Block::new("script_struct");
        b.set("targetname", "initial_spawn_points").set("script_noteworthy", "initial_spawn").set("radius", "32").set("script_int", (i % 2 + 1).to_string());
        b.at(s.origin, s.angles[1]);
        ents.push(b);
        let mut e = Block::new("info_player_start");
        e.at(s.origin, s.angles[1]);
        if i == 0 {
            ents.push(e);
        }
    }

    // Windows.
    let mut windows: Vec<(String, [f32; 3])> = Vec::new();
    for (i, w) in l.windows.iter().enumerate() {
        let id = format!("iw7_window_{i}");
        let yaw = w.outside.angles[1];
        let dir = yaw_dir(yaw);
        let boards = w.boards.map_or(add(w.outside.origin, dir, 30.0), |b| [b[0], b[1], w.outside.origin[2] + 4.0]);
        let inside = w.inside.map_or(add(boards, dir, 44.0), |s| s.origin);
        let Some(template) = &kit.window else {
            continue;
        };
        let mut goal = Block::new("script_struct");
        goal.set("targetname", "exterior_goal").set("target", id.as_str()).set("script_string", id.as_str());
        goal.at([boards[0] - dir[0] * 3.0, boards[1] - dir[1] * 3.0, w.outside.origin[2]], yaw);
        ents.push(goal);
        let mut spot = Block::new("script_struct");
        spot.set("targetname", id.as_str()).set("script_string", id.as_str()).set("radius", "36").set("height", "64");
        spot.at(inside, yaw);
        ents.push(spot);
        let mut zb = Block(template.0.clone());
        zb.set("targetname", id.as_str()).set("script_string", id.as_str());
        zb.at(boards, yaw);
        ents.push(zb);
        nodes.negotiation(&id, boards, add(boards, dir, 80.0), "zm_mantle_over_40");
        windows.push((id, w.outside.origin));
    }
    if kit.window.is_none() && !l.windows.is_empty() {
        report.push(format!("iw7 zombies gap: no barrier zbarrier in {DONOR}'s entities: {} windows left open", l.windows.len()));
    }
    let nearest_window = |at: [f32; 3]| -> Option<&str> {
        windows.iter().map(|(id, o)| (id.as_str(), dist(*o, at))).filter(|(_, d)| *d < 900.0).min_by(|a, b| a.1.total_cmp(&b.1)).map(|(id, _)| id)
    };

    // Zones and their spawners.
    let mut zones: Vec<&str> = Vec::new();
    let (mut risers, mut fallers, mut walkers, mut through_windows) = (0, 0, 0, 0);
    for a in &l.areas {
        let Some(model) = a.model else {
            report.push(format!("iw7 zombies: area {} has no trigger model: not a zone", a.name));
            continue;
        };
        zones.push(a.name.as_str());
        let spawners = format!("{}_spawners", a.name);
        let mut v = Block::new("info_volume");
        v.set("targetname", a.name.as_str()).set("target", spawners.as_str()).set("script_noteworthy", "player_volume").set("model", format!("?{model}"));
        v.set("origin", vec3(a.origin));
        ents.push(v);
        for s in &a.spawners {
            let mut b = Block::new("script_struct");
            let kind = if s.ground {
                "riser_location"
            } else if s.ceiling {
                "faller_location"
            } else {
                "spawn_location"
            };
            let via = if s.no_boards { None } else { nearest_window(s.spot.origin) };
            match via {
                Some(_) => through_windows += 1,
                None if s.ground => risers += 1,
                None if s.ceiling => fallers += 1,
                None => walkers += 1,
            }
            b.set("targetname", spawners.as_str()).set("script_noteworthy", kind).set("script_string", via.unwrap_or("find_flesh"));
            b.at(s.spot.origin, s.spot.angles[1]);
            ents.push(b);
        }
    }
    report.push(format!(
        "iw7 zombies: {} zones; spawners: {through_windows} through windows, {risers} rising, {fallers} dropping from ceilings, {walkers} walking in",
        zones.len()
    ));

    // Which zones join which, and the flag that opens the way.
    let mut links: Vec<(String, String, String)> = Vec::new();
    let known = |n: &str| zones.contains(&n);
    for b in &l.blockers {
        let mut sides: Vec<&str> = b.sides.iter().map(|(_, a)| a.as_str()).filter(|a| known(a)).collect();
        sides.sort_unstable();
        sides.dedup();
        for i in 0..sides.len() {
            for j in i + 1..sides.len() {
                links.push((sides[i].to_owned(), sides[j].to_owned(), b.group.clone()));
            }
        }
    }
    // Areas whose boxes meet and no blocker stands between: open from the
    // start.
    let meets = |a: &fastfile_iw7::zombies::Area, b: &fastfile_iw7::zombies::Area| {
        a.boxes.iter().any(|(alo, ahi)| b.boxes.iter().any(|(blo, bhi)| (0..3).all(|k| alo[k] <= bhi[k] + 16.0 && blo[k] <= ahi[k] + 16.0)))
    };
    let zone_areas: Vec<&fastfile_iw7::zombies::Area> = l.areas.iter().filter(|a| known(&a.name)).collect();
    for i in 0..zone_areas.len() {
        for j in i + 1..zone_areas.len() {
            let (a, b) = (zone_areas[i], zone_areas[j]);
            let doored = links.iter().any(|(x, y, _)| (x == &a.name && y == &b.name) || (x == &b.name && y == &a.name));
            if !doored && meets(a, b) {
                links.push((a.name.clone(), b.name.clone(), "always_on".to_owned()));
            }
        }
    }
    for (a, b, flag) in &links {
        let mut s = Block::new("script_struct");
        s.set("targetname", "iw7_zone_link").set("script_noteworthy", format!("{a} {b}")).set("script_flag", flag.as_str());
        s.set("origin", "0 0 0");
        ents.push(s);
    }
    let mut start_zones: Vec<String> = l.starts.iter().filter_map(|s| zone_of(s.origin)).collect();
    start_zones.sort();
    start_zones.dedup();
    for z in &start_zones {
        let mut s = Block::new("script_struct");
        s.set("targetname", "iw7_start_zone").set("script_noteworthy", z.as_str()).set("origin", "0 0 0");
        ents.push(s);
    }
    report.push(format!(
        "iw7 zombies: {} zone links ({} through doors and debris, {} open), start zones {:?}",
        links.len(),
        links.iter().filter(|l| l.2 != "always_on").count(),
        links.iter().filter(|l| l.2 == "always_on").count(),
        start_zones
    ));

    // Respawn points: per zone, the map's respawn spots in it.
    let mut by_zone: BTreeMap<String, Vec<Spot>> = BTreeMap::new();
    for r in &l.respawns {
        if let Some(z) = zone_of(r.origin).filter(|z| known(z)) {
            by_zone.entry(z).or_default().push(*r);
        }
    }
    for (z, spots) in &by_zone {
        let target = format!("{z}_player_spawns");
        let mut p = Block::new("script_struct");
        p.set("targetname", "player_respawn_point").set("script_noteworthy", z.as_str()).set("target", target.as_str()).set("radius", "1000").set("script_int", "1000");
        p.at(spots[0].origin, 0.0);
        ents.push(p);
        for (i, s) in spots.iter().enumerate() {
            let mut b = Block::new("script_struct");
            b.set("targetname", target.as_str()).set("radius", "32").set("script_int", (i % 2 + 1).to_string());
            b.at(s.origin, s.angles[1]);
            ents.push(b);
        }
    }

    // Debris and doors.
    for b in &l.blockers {
        let cost = match b.kind {
            BlockerKind::Debris => b.cost.max(250),
            BlockerKind::TeamDoor => 1000,
        };
        let flag = b.group.as_str();
        let trigger = |t: &mut Block| {
            t.set("targetname", "zombie_debris").set("target", flag).set("zombie_cost", cost.to_string()).set("script_flag", flag);
        };
        if b.triggers.is_empty() {
            for (spot, _) in &b.sides {
                let mut t = Block::new("trigger_radius_use");
                trigger(&mut t);
                t.set("radius", "72").set("height", "96");
                t.set("origin", vec3([spot.origin[0], spot.origin[1], spot.origin[2] - 16.0]));
                ents.push(t);
            }
        } else {
            for (origin, model) in &b.triggers {
                let mut t = Block::new("trigger_use");
                trigger(&mut t);
                t.set("model", format!("?{model}")).set("origin", vec3(*origin));
                ents.push(t);
            }
        }
        for (model, origin) in &b.brushes {
            let mut c = Block::new("script_brushmodel");
            c.set("targetname", flag).set("script_noteworthy", "clip").set("spawnflags", "1").set("model", format!("*{model}"));
            c.set("origin", vec3(*origin));
            ents.push(c);
        }
    }
    report.push(format!(
        "iw7 zombies: {} debris piles and doors ({} team doors sold at 1000)",
        l.blockers.len(),
        l.blockers.iter().filter(|b| b.kind == BlockerKind::TeamDoor).count()
    ));

    // Perk machines and Pack-a-Punch.
    let mut perks = Vec::new();
    let mut unmapped = Vec::new();
    for m in &l.perks {
        let Some((perk, model)) = bo2_perk(&m.name) else {
            unmapped.push(m.name.as_str());
            continue;
        };
        let mut s = Block::new("script_struct");
        s.set("targetname", "zm_perk_machine").set("script_noteworthy", perk).set("model", model).set("script_string", format!("zclassic_perks_{loc}"));
        s.at(m.spot.origin, m.spot.angles[1]);
        ents.push(s);
        perks.push(perk);
    }
    if let Some(pap) = l.pap {
        let mut s = Block::new("script_struct");
        s.set("targetname", "zm_perk_machine")
            .set("script_noteworthy", "specialty_weapupgrade")
            .set("model", "p6_anim_zm_buildable_pap")
            .set("script_string", format!("zclassic_perks_{loc}"))
            .set("target", "iw7_pap_sign");
        s.at(pap.origin, pap.angles[1]);
        ents.push(s);
        let mut sign = Block::new("script_struct");
        sign.set("targetname", "iw7_pap_sign").set("model", "zombie_sign_please_wait");
        sign.at(add(pap.origin, [0.0, 0.0, 50.0], 1.0), pap.angles[1] + 180.0);
        ents.push(sign);
        for o in &p.pap_room {
            let mut r = Block::new("script_struct");
            r.set("targetname", "iw7_pap_room");
            r.at(*o, pap.angles[1]);
            ents.push(r);
        }
    }
    report.push(format!(
        "iw7 zombies: perk machines {perks:?}; left out (no Black Ops II perk like them): {unmapped:?}; Pack-a-Punch {}",
        if l.pap.is_some() { "placed" } else { "missing" }
    ));

    // The Mystery Box at each Magic Wheel spot; it starts at the one
    // nearest the players' starts.
    let start_box = l
        .wheels
        .iter()
        .enumerate()
        .min_by(|a, b| dist(a.1.origin, base).total_cmp(&dist(b.1.origin, base)))
        .map(|(i, _)| i);
    if let Some(template) = &kit.box_barrier {
        for (i, w) in l.wheels.iter().enumerate() {
            let name = if Some(i) == start_box { "start_chest".to_owned() } else { format!("iw7_chest{i}") };
            let mut c = Block::new("script_struct");
            c.set("targetname", "treasure_chest_use").set("script_noteworthy", name.as_str()).set("zombie_cost", "950");
            c.at(w.origin, w.angles[1]);
            ents.push(c);
            let mut zb = Block(template.0.clone());
            zb.set("script_noteworthy", format!("{name}_zbarrier"));
            zb.at(w.origin, w.angles[1]);
            ents.push(zb);
        }
    } else if !l.wheels.is_empty() {
        report.push(format!("iw7 zombies gap: no Mystery Box zbarrier in {DONOR}'s entities: no box"));
    }

    // Wall buys.
    let mut walls = 0;
    let mut wall_gaps = Vec::new();
    for (i, w) in l.wall_buys.iter().enumerate() {
        let Some((weapon, model)) = bo2_wall_weapon(&w.weapon) else {
            wall_gaps.push(w.weapon.as_str());
            continue;
        };
        let at = w.model_spot.unwrap_or(w.spot);
        let target = format!("iw7_wallbuy_{i}");
        let mut s = Block::new("script_struct");
        s.set("targetname", "weapon_upgrade").set("zombie_weapon_upgrade", weapon).set("script_noteworthy", format!("zclassic_{loc}")).set("target", target.as_str());
        s.at(at.origin, at.angles[1]);
        ents.push(s);
        let mut m = Block::new("script_struct");
        m.set("targetname", target.as_str()).set("model", model);
        m.at(at.origin, at.angles[1]);
        ents.push(m);
        walls += 1;
    }
    report.push(format!("iw7 zombies: {walls} wall buys; left out: {wall_gaps:?}; {} Mystery Box spots", l.wheels.len()));

    // Power switches and portals (the map script runs them).
    for g in &l.generators {
        let mut s = Block::new("script_struct");
        s.set("targetname", "iw7_power_switch").set("script_noteworthy", g.areas.join(" "));
        s.at(g.spot.origin, g.spot.angles[1]);
        ents.push(s);
    }
    for (i, portal) in l.portals.iter().enumerate() {
        let dests = format!("iw7_portal_{i}_dest");
        let mut s = Block::new("script_struct");
        s.set("targetname", "iw7_portal").set("script_noteworthy", portal.area.as_str()).set("target", dests.as_str());
        s.at(portal.spot.origin, portal.spot.angles[1]);
        ents.push(s);
        for d in &portal.dests {
            let mut b = Block::new("script_struct");
            b.set("targetname", dests.as_str());
            b.at(d.origin, d.angles[1]);
            ents.push(b);
        }
    }

    // The map's own traversals (jumps, climbs) with Black Ops II's.
    let mut traversals = 0;
    for (i, t) in l.traversals.iter().enumerate() {
        if nodes.negotiation(&format!("iw7_traverse_{i}"), t.begin, t.end, bo2_traverse(t.end[2] - t.begin[2])) {
            traversals += 1;
        }
    }
    report.push(format!(
        "iw7 zombies: {} path nodes ({} walk graph, {} window mantles, {traversals} traversals)",
        nodes.nodes.len(),
        p.walk.len().min(usize::from(u16::MAX) - 512),
        windows.len()
    ));

    let mut entities = String::new();
    for b in &ents {
        b.write(&mut entities);
    }
    Translation { entities, path_nodes: nodes.nodes, report }
}

// ---------------------------------------------------------------------------
// Black Ops II's Zombies rules on the map.

/// bo2mp: Black Ops II Zombies on an Infinite Warfare Zombies map: the
/// donor's Zombies zones as a Black Ops II Zombies map loads them (common
/// and patch zones, the map's, its mode's, its patch), the map's layout
/// as their entities and path nodes, and our map script
/// (`sim::t6::foreign_zm`).
pub(crate) fn attach_t6_zm_rules(loaded: &mut LoadedWorld, map: &str) {
    super::t6::foreign_t6_lighting(loaded, map);
    if let Some(grade) = super::bo2_look::take_pending_grade() {
        asset_world::install_t6_grade(Some(grade));
    } else if asset_world::install_film_map_grade(loaded.world.film_vision.as_ref()) {
        loaded.report.push(format!("bo2 zombies rules grade: {map} graded by its own game's film vision"));
    }
    let Some(pending) = take_pending(map) else {
        loaded.report.push(format!("bo2 zombies rules gap: {map}'s Zombies layout was not read"));
        return;
    };
    let mut report = Vec::new();
    let dir = match asset_transport::games_root_from_env().and_then(|root| asset_transport::find_zone_file(&root, "t6:patch_zm")) {
        Ok(found) => match found.path.parent() {
            Some(dir) => dir.to_path_buf(),
            None => {
                loaded.report.push("bo2 zombies rules gap: patch_zm has no folder".to_owned());
                return;
            }
        },
        Err(error) => {
            loaded.report.push(format!("bo2 zombies rules gap: no Black Ops II Zombies zones ({error})"));
            return;
        }
    };
    let donor_path = dir.join(format!("{DONOR}.ff"));
    let donor = match asset_t6::capture_zone(&donor_path) {
        Ok(c) => c,
        Err(error) => {
            loaded.report.push(format!("bo2 zombies rules gap: {DONOR} not read ({error})"));
            return;
        }
    };
    let packs = if frame::Headless::requested() {
        None
    } else {
        match asset_t6::PackSet::open_dir(&dir) {
            Ok(packs) => Some(packs),
            Err(error) => {
                report.push(format!("bo2 zombies rules: t6 image packs: {error}"));
                None
            }
        }
    };
    let previous_ns = loaded.materials.capture_ns();
    loaded.materials.set_capture_ns(asset_core::AssetNamespace::T6);
    let mut combat = super::t6_m2::load_t6_combat(&donor_path, &donor, &mut loaded.materials, packs.as_ref());
    drop(donor);
    loaded.materials.set_capture_ns(previous_ns);
    report.append(&mut combat.report);

    // The donor's entities lend their zombies and barriers; the map's own
    // layout replaces them, and its path nodes the donor's.
    let kit = DonorKit::from_entities(&combat.scripts.entities);
    let t = translate(&pending, &kit);
    report.extend(t.report);
    combat.scripts.entities = vec![t.entities];
    combat.scripts.path_nodes = t.path_nodes;
    combat.scripts.path_tables = Default::default();

    let (origin, yaw) = pending.layout.starts.first().map_or(([0.0; 3], 0.0), |s| (s.origin, s.angles[1]));
    let mut scripts = super::t6_scripts::start_scripts(origin, yaw, true);
    loaded.spawns = pending
        .layout
        .starts
        .iter()
        .map(|s| asset_world::SpawnPoint {
            classname: "info_player_start".to_owned(),
            origin: s.origin,
            angles: s.angles,
            script_linkto: String::new(),
            script_destructable_area: String::new(),
        })
        .collect();

    let prepare = super::t6::script_sound_aliases(&combat.scripts);
    loaded.facts.script_sound = asset_audio::MapScriptSoundFacts {
        ambient_alias: combat.room_tone.clone(),
        prepare,
        ..Default::default()
    };
    loaded.facts.t6_footsteps = combat.footsteps;
    loaded.facts.t6_hud_icons = combat.hud_icons;
    loaded.facts.t6_hud_fonts = combat.hud_fonts;
    loaded.facts.t6_ui = combat.ui;
    loaded.facts.t6_playeranim = combat.scripts.playeranim.clone();
    loaded.facts.t6_ragdoll = combat.scripts.ragdoll.clone();
    loaded.facts.t6_pen_table = combat.pen_table.clone();
    loaded.facts.t6_screen_filters = combat.screen_filters;
    loaded.facts.t6_anim_speeds = combat
        .scripts
        .anims
        .iter()
        .filter(|a| a.name.starts_with("pb_") || a.name.starts_with("pl_"))
        .filter_map(|a| {
            let len = f32::from(a.numframes) / a.framerate;
            let (first, last) = (a.delta_trans.first()?.1, a.delta_trans.last()?.1);
            let v = [last[0] - first[0], last[1] - first[1], last[2] - first[2]];
            let d = if v[0] + v[1] <= v[2] * 0.8 { (v[0] * v[0] + v[1] * v[1] + v[2] * v[2]).sqrt() } else { (v[0] * v[0] + v[1] * v[1]).sqrt() };
            (len > 0.0 && d > 1.0).then(|| (a.name.to_ascii_lowercase(), d / len))
        })
        .collect();
    scripts.t6 = Some(combat.scripts);
    loaded.scripts = scripts;

    loaded.weapons.absorb(combat.weapons);
    loaded.fpv_meshes.absorb(combat.fpv);
    loaded.world_weapons.absorb(combat.world_weapons);
    loaded.xanims.absorb_local(combat.xanims);
    loaded.tracers.absorb(combat.tracers);
    loaded.world.fx.absorb_missing(combat.fx);
    loaded.world.fx_models.absorb(combat.fx_models);
    if combat.impact_fx.is_some() {
        loaded.world.impact_fx = combat.impact_fx;
    }
    if combat.pen_table.is_some() {
        loaded.pen_table = combat.pen_table;
    }
    loaded.sound = Some(match (combat.sound, loaded.sound.take()) {
        (Ok(mut bo2), Some(Ok(map_sound))) => {
            bo2.absorb_missing_aliases_unresolved(map_sound);
            Ok(bo2)
        }
        (Ok(bo2), _) => Ok(bo2),
        (Err(error), Some(Ok(map_sound))) => {
            report.push(format!("bo2 zombies rules: BO2 sound gap: {error}"));
            Ok(map_sound)
        }
        (Err(error), other) => other.unwrap_or(Err(error)),
    });
    report.push(format!("bo2 zombies rules: {map} plays Black Ops II Zombies on {DONOR}'s kit"));
    for line in &report {
        diag::info!(World, "{line}");
    }
    loaded.report.extend(report);
}

#[cfg(test)]
mod tests {
    use super::*;
    use fastfile_iw7::zombies::{Area, Blocker, Machine, Spawner, Traversal, WallBuy, Window};

    fn spot(x: f32, y: f32, yaw: f32) -> Spot {
        Spot { origin: [x, y, 0.0], angles: [0.0, yaw, 0.0] }
    }

    fn layout() -> ZmLayout {
        let room = |name: &str, x0: f32, x1: f32, model: u32| Area {
            name: name.to_owned(),
            origin: [(x0 + x1) * 0.5, 0.0, 0.0],
            model: Some(model),
            boxes: vec![([x0, -256.0, -64.0], [x1, 256.0, 192.0])],
            spawners: vec![
                Spawner { spot: spot(x0 + 64.0, -300.0, 90.0), ground: false, ceiling: false, no_boards: false, goal: None },
                Spawner { spot: spot(x0 + 128.0, 0.0, 0.0), ground: true, ceiling: false, no_boards: true, goal: None },
                Spawner { spot: spot(x0 + 160.0, 0.0, 0.0), ground: false, ceiling: true, no_boards: true, goal: None },
            ],
        };
        ZmLayout {
            starts: vec![spot(64.0, 0.0, 0.0), spot(96.0, 0.0, 0.0)],
            respawns: vec![spot(80.0, 32.0, 0.0)],
            areas: vec![room("front_gate", 0.0, 512.0, 3), room("moon", 512.0, 1024.0, 4), room("mars", 2048.0, 2560.0, 5)],
            windows: vec![Window { outside: spot(64.0, -290.0, 90.0), attack: Vec::new(), inside: Some(spot(64.0, -240.0, 90.0)), boards: Some([64.0, -260.0, 4.0]) }],
            blockers: vec![Blocker {
                kind: BlockerKind::Debris,
                cost: 1250,
                sides: vec![(spot(1000.0, 0.0, 0.0), "moon".to_owned()), (spot(2100.0, 0.0, 180.0), "mars".to_owned())],
                triggers: vec![([1500.0, 0.0, 30.0], 9)],
                brushes: vec![(12, [1500.0, 0.0, 40.0])],
                group: "pf7_auto6".to_owned(),
                props: Vec::new(),
            }],
            perks: vec![
                Machine { name: "perk_machine_tough".to_owned(), spot: spot(200.0, 200.0, 270.0) },
                Machine { name: "perk_machine_zap".to_owned(), spot: spot(300.0, 200.0, 270.0) },
            ],
            wheels: vec![spot(900.0, 0.0, 0.0), spot(100.0, 100.0, 90.0)],
            pap: Some(spot(2300.0, 0.0, 0.0)),
            wall_buys: vec![
                WallBuy { weapon: "iw7_m1c_zm".to_owned(), spot: spot(10.0, 10.0, 0.0), model_spot: None },
                WallBuy { weapon: "iw7_erad_zm+eradscope".to_owned(), spot: spot(20.0, 10.0, 0.0), model_spot: None },
                WallBuy { weapon: "iw7_unknown_zm".to_owned(), spot: spot(30.0, 10.0, 0.0), model_spot: None },
            ],
            traversals: vec![Traversal { begin: [256.0, 0.0, 0.0], end: [320.0, 0.0, -96.0], animscript: "jump_down_56".to_owned() }],
            ..Default::default()
        }
    }

    fn kit() -> DonorKit {
        DonorKit::from_entities(&[r#"{
"classname" "actor_zm_buried_basic_01_char_01"
"script_noteworthy" "zombie_spawner"
"count" "9999"
"guid" "1"
}
{
"classname" "actor_zm_buried_sloth"
"script_noteworthy" "zombie_spawner"
}
{
"classname" "zbarrier_zmcore_BasicWoodBarrier"
"zbarriernumboards" "6"
"type" "zmcore_BasicWoodBarrier"
"script_string" "candy"
}
{
"classname" "zbarrier_zmcore_MagicBox"
"type" "zmcore_MagicBox"
"script_location" "processing street"
}
"#
        .to_owned()])
    }

    fn walk() -> Vec<WalkNode> {
        let mut nodes = Vec::new();
        for i in 0..40 {
            for j in -4..=4 {
                nodes.push(WalkNode { origin: [i as f32 * 64.0, j as f32 * 64.0, 0.0], links: Vec::new() });
            }
        }
        let n = nodes.len();
        for a in 0..n {
            for b in 0..n {
                if a != b && dist(nodes[a].origin, nodes[b].origin) < 70.0 {
                    nodes[a].links.push(b as u32);
                }
            }
        }
        nodes
    }

    fn blocks(text: &str) -> Vec<Block> {
        parse_blocks(text)
    }

    #[test]
    fn the_donor_lends_its_zombies_and_barriers() {
        let k = kit();
        assert_eq!(k.spawners.len(), 1, "the sloth is no zombie spawner here");
        assert!(k.spawners[0].get("guid").is_none());
        assert!(k.window.is_some() && k.box_barrier.is_some());
        assert!(k.box_barrier.as_ref().unwrap().get("script_location").is_none());
    }

    #[test]
    fn the_layout_becomes_bo2_zombies_entities() {
        let p = Pending { map: "cp_test".to_owned(), layout: layout(), walk: walk(), pap_room: vec![[2400.0, 0.0, 0.0]] };
        let t = translate(&p, &kit());
        let ents = blocks(&t.entities);
        let named = |n: &str| ents.iter().filter(|b| b.get("targetname") == Some(n)).collect::<Vec<_>>();
        // Zones with their trigger models and spawners.
        let gate = named("front_gate");
        assert_eq!(gate.len(), 1);
        assert_eq!(gate[0].get("classname"), Some("info_volume"));
        assert_eq!(gate[0].get("model"), Some("?3"));
        let spawners = named("front_gate_spawners");
        assert_eq!(spawners.len(), 3);
        assert_eq!(spawners[2].get("script_noteworthy"), Some("faller_location"));
        assert_eq!(spawners[0].get("script_noteworthy"), Some("spawn_location"));
        assert_eq!(spawners[0].get("script_string"), Some("iw7_window_0"), "it comes in through the window");
        assert_eq!(spawners[1].get("script_noteworthy"), Some("riser_location"));
        assert_eq!(spawners[1].get("script_string"), Some("find_flesh"));
        // The window: its goal, its boards (the donor's) and its mantle.
        let goal = named("exterior_goal");
        assert_eq!(goal[0].get("target"), Some("iw7_window_0"));
        let parts = named("iw7_window_0");
        assert!(parts.iter().any(|b| b.get("classname") == Some("zbarrier_zmcore_BasicWoodBarrier") && b.get("script_string") == Some("iw7_window_0")));
        let begin = t.path_nodes.iter().position(|n| n.targetname == "iw7_window_0").unwrap();
        assert_eq!(t.path_nodes[begin].ty, NODE_BEGIN);
        assert_eq!(t.path_nodes[begin].animscript, "zm_mantle_over_40");
        assert!(t.path_nodes[begin].links.iter().any(|l| l.2 && t.path_nodes[l.0 as usize].ty == NODE_END));
        // The debris: its trigger, its clip and the zones it opens.
        let debris = named("zombie_debris");
        assert_eq!(debris.len(), 1);
        assert_eq!(debris[0].get("model"), Some("?9"));
        assert_eq!(debris[0].get("zombie_cost"), Some("1250"));
        assert_eq!(debris[0].get("script_flag"), Some("pf7_auto6"));
        assert!(named("pf7_auto6").iter().any(|b| b.get("model") == Some("*12") && b.get("script_noteworthy") == Some("clip")));
        let links: Vec<(&str, &str)> = named("iw7_zone_link").iter().map(|b| (b.get("script_noteworthy").unwrap(), b.get("script_flag").unwrap())).collect();
        assert!(links.contains(&("mars moon", "pf7_auto6")));
        assert!(links.contains(&("front_gate moon", "always_on")), "{links:?}");
        assert!(!links.iter().any(|(z, _)| *z == "front_gate mars"));
        assert_eq!(named("iw7_start_zone")[0].get("script_noteworthy"), Some("front_gate"));
        // Machines: Tuff 'Nuff is Juggernog; Blue Bolts has no like.
        let perks = named("zm_perk_machine");
        assert!(perks.iter().any(|b| b.get("script_noteworthy") == Some("specialty_armorvest") && b.get("model") == Some("zombie_vending_jugg")));
        assert!(perks.iter().any(|b| b.get("script_noteworthy") == Some("specialty_weapupgrade")));
        assert_eq!(perks.len(), 2);
        assert!(perks.iter().all(|b| b.get("script_string") == Some("zclassic_perks_iw7")));
        // The box: it starts at the spot nearest the starts.
        let chests = named("treasure_chest_use");
        assert_eq!(chests.len(), 2);
        assert_eq!(chests[1].get("script_noteworthy"), Some("start_chest"));
        assert!(ents.iter().any(|b| b.get("script_noteworthy") == Some("start_chest_zbarrier")));
        // Wall buys of their kind.
        let walls = named("weapon_upgrade");
        assert_eq!(walls.iter().map(|b| b.get("zombie_weapon_upgrade").unwrap()).collect::<Vec<_>>(), vec!["m14_zm", "pdw57_zm"]);
        assert_eq!(walls[0].get("script_noteworthy"), Some("zclassic_iw7"));
        // Starts, respawns, the zombies.
        assert_eq!(named("initial_spawn_points").len(), 2);
        assert_eq!(named("player_respawn_point")[0].get("script_noteworthy"), Some("front_gate"));
        assert!(ents.iter().any(|b| b.get("classname") == Some("actor_zm_buried_basic_01_char_01")));
        // The map's own traversal, a drop of 96.
        let tr = t.path_nodes.iter().find(|n| n.targetname == "iw7_traverse_0").unwrap();
        assert_eq!(tr.animscript, "zm_jump_down_96");
    }

    #[test]
    fn traversals_pick_the_nearest_height() {
        assert_eq!(bo2_traverse(-50.0), "zm_jump_down_48");
        assert_eq!(bo2_traverse(-130.0), "zm_jump_down_127");
        assert_eq!(bo2_traverse(10.0), "zm_mantle_over_40");
        assert_eq!(bo2_traverse(128.0), "zm_jump_up_127");
    }

    #[test]
    fn wall_weapons_by_kind() {
        assert_eq!(bo2_wall_weapon("iw7_revolver_zm"), Some(("rottweil72_zm", "t6_wpn_shotty_olympia_world")));
        assert_eq!(bo2_wall_weapon("iw7_forgefreeze_zm+forgefreezealtfire").map(|w| w.0), Some("svu_zm"));
        assert_eq!(bo2_wall_weapon("iw7_ake_zml").map(|w| w.0), Some("ak74u_zm"));
        assert_eq!(bo2_wall_weapon("iw7_sonic_zmr").map(|w| w.0), Some("870mcs_zm"));
        assert_eq!(bo2_wall_weapon("iw7_g18c_zm").map(|w| w.0), Some("beretta93r_zm"));
        assert_eq!(bo2_wall_weapon("iw7_golf_club_mp"), None);
        assert_eq!(bo2_wall_weapon("iw7_harpoon_zm"), None);
    }
}
