//! What an Infinite Warfare Zombies map (`cp_*`) places for its game, read
//! from its entities by the names its scripts look them up by: the player
//! starts, the areas (`spawn_volume`s and the spawners they target), the
//! windows, the buyable debris and doors and the areas they open, the perk
//! machines, the Magic Wheel spots, Pack-a-Punch, the portals, the power
//! generators, the wall weapons and the AI traversals. Plain data in the
//! map's own terms; whoever plays the map with another game's rules maps it
//! onto theirs.

use crate::map::Iw7Map;

/// A map entity's keys, in order.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct Ent {
    pub keys: Vec<(String, String)>,
}

impl Ent {
    pub fn get(&self, k: &str) -> Option<&str> {
        self.keys.iter().find(|(key, _)| key == k).map(|(_, v)| v.as_str())
    }

    fn vec3(&self, k: &str) -> Option<[f32; 3]> {
        let v: Vec<f32> = self.get(k)?.split_whitespace().filter_map(|x| x.parse().ok()).collect();
        (v.len() == 3).then(|| [v[0], v[1], v[2]])
    }

    pub fn origin(&self) -> [f32; 3] {
        self.vec3("origin").unwrap_or([0.0; 3])
    }

    pub fn angles(&self) -> [f32; 3] {
        self.vec3("angles").unwrap_or([0.0; 3])
    }

    fn spot(&self) -> Spot {
        Spot { origin: self.origin(), angles: self.angles() }
    }

    fn is(&self, k: &str, v: &str) -> bool {
        self.get(k) == Some(v)
    }
}

/// The entities of a (named-key) entity string.
pub fn parse_entities(text: &str) -> Vec<Ent> {
    let mut out = Vec::new();
    let mut cur: Option<Ent> = None;
    for line in text.lines() {
        let t = line.trim();
        if t == "{" {
            cur = Some(Ent::default());
        } else if t == "}" {
            out.extend(cur.take());
        } else if let Some(e) = cur.as_mut()
            && let Some((k, v)) = t.split_once(' ')
        {
            e.keys.push((k.trim_matches('"').to_owned(), v.trim().trim_matches('"').to_owned()));
        }
    }
    out
}

#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct Spot {
    pub origin: [f32; 3],
    pub angles: [f32; 3],
}

/// Where zombies come from in an area.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct Spawner {
    pub spot: Spot,
    /// They climb out of the ground (`ground_spawn*`, `spawn_ground`).
    pub ground: bool,
    /// They drop from the ceiling (`spawn_ceiling`).
    pub ceiling: bool,
    /// They go straight for the players, no window on the way
    /// (`ground_spawn_no_boards`).
    pub no_boards: bool,
    /// The struct its `target` names: where they walk to first.
    pub goal: Option<[f32; 3]>,
}

/// An area of the park: a `spawn_volume` (its name is its link name less
/// the prefab prefix: `pf7_moon_rocket_space` -> `moon_rocket_space`), its
/// boxes in the world and its zombie spawners.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct Area {
    pub name: String,
    pub origin: [f32; 3],
    /// Its trigger model (`?<n>`): the hulls the boxes are made of.
    pub model: Option<u32>,
    /// World boxes (mins, maxs).
    pub boxes: Vec<([f32; 3], [f32; 3])>,
    pub spawners: Vec<Spawner>,
}

impl Area {
    pub fn contains(&self, p: [f32; 3], margin: f32) -> bool {
        self.boxes.iter().any(|(lo, hi)| (0..3).all(|k| p[k] >= lo[k] - margin && p[k] <= hi[k] + margin))
    }
}

/// A boarded window: the outside spot zombies come to (`window_entrance`,
/// its yaw pointing inside), its two attack spots, the inside spot the
/// player repairs from (the `secure_window` interaction) and where the
/// boards are (that interaction's target).
#[derive(Clone, Debug, Default, PartialEq)]
pub struct Window {
    pub outside: Spot,
    pub attack: Vec<[f32; 3]>,
    pub inside: Option<Spot>,
    pub boards: Option<[f32; 3]>,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum BlockerKind {
    /// `debris_<cost>`: bought, it goes.
    Debris,
    /// `team_door_switch`: held open by the team.
    TeamDoor,
}

/// A buyable blocker: its interactions (one per side, each naming the area
/// it opens), its use trigger, and the brushes that block until it goes.
#[derive(Clone, Debug, PartialEq)]
pub struct Blocker {
    pub kind: BlockerKind,
    pub cost: u32,
    /// (where the player stands, the area it opens).
    pub sides: Vec<(Spot, String)>,
    /// The use triggers (`door_buy`, one per side): origin and trigger
    /// model `?N`.
    pub triggers: Vec<([f32; 3], u32)>,
    /// The blocking brushes: `*N` and the entity's origin.
    pub brushes: Vec<(u32, [f32; 3])>,
    /// The name its pieces share (the interactions' and trigger's target).
    pub group: String,
    /// Its props (`find_props`): what is seen of it, its brushes being clip
    /// only.
    pub props: Vec<BlockerProp>,
}

/// A placed model that is part of a blocker: its index among the map's
/// placed models (`Iw7Map::static_models`) and where it stands.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct BlockerProp {
    pub index: usize,
    pub origin: [f32; 3],
    /// Rows: the model's forward, left and up axes (scaled by its scale).
    pub axis: [[f32; 3]; 3],
}



/// A machine by its IW7 interaction name (`perk_machine_tough`).
#[derive(Clone, Debug, PartialEq)]
pub struct Machine {
    pub name: String,
    pub spot: Spot,
}

/// A portal (`fast_travel`): where it is, its area and the spots it sends
/// players to.
#[derive(Clone, Debug, PartialEq)]
pub struct Portal {
    pub spot: Spot,
    pub area: String,
    pub dests: Vec<Spot>,
}

/// A power generator: the areas it powers, and its lever model.
#[derive(Clone, Debug, PartialEq)]
pub struct Generator {
    pub spot: Spot,
    pub areas: Vec<String>,
    pub lever: Option<(String, Spot)>,
}

/// A wall weapon: IW7's weapon name and the spot of its chalk.
#[derive(Clone, Debug, PartialEq)]
pub struct WallBuy {
    pub weapon: String,
    pub spot: Spot,
    /// The weapon model's spot (the interaction's target).
    pub model_spot: Option<Spot>,
}

/// An AI traversal: from its begin node to its end node, with the
/// traverse's animscript.
#[derive(Clone, Debug, PartialEq)]
pub struct Traversal {
    pub begin: [f32; 3],
    pub end: [f32; 3],
    pub animscript: String,
}

#[derive(Clone, Debug, Default, PartialEq)]
pub struct ZmLayout {
    pub starts: Vec<Spot>,
    pub respawns: Vec<Spot>,
    pub areas: Vec<Area>,
    pub windows: Vec<Window>,
    pub blockers: Vec<Blocker>,
    pub perks: Vec<Machine>,
    pub wheels: Vec<Spot>,
    pub pap: Option<Spot>,
    pub portals: Vec<Portal>,
    pub generators: Vec<Generator>,
    pub wall_buys: Vec<WallBuy>,
    pub dog_spots: Vec<Spot>,
    pub traversals: Vec<Traversal>,
}

fn model_index(model: &str, prefix: char) -> Option<u32> {
    model.strip_prefix(prefix)?.parse().ok()
}

/// `pf7_moon_rocket_space` -> `moon_rocket_space`.
fn area_name(link: &str) -> String {
    match link.split_once('_') {
        Some((pf, rest)) if pf.starts_with("pf") && pf[2..].chars().all(|c| c.is_ascii_digit()) => rest.to_owned(),
        _ => link.to_owned(),
    }
}

impl ZmLayout {
    /// The layout of a map read by [`Iw7Map::load`].
    pub fn read(map: &Iw7Map) -> Self {
        let mut out = Self::from_parts(&parse_entities(&map.entities), &map.trigger_hulls, &map.path_nodes);
        out.find_props(map);
        out
    }

    /// Each blocker's props: the scriptables its group names
    /// (`cp_zmb_barrier_*`: Spaceland's park wall panels; Rave's
    /// `cp_rave_doorbuy_*` boards and rocks), drawn as placed models by
    /// `Iw7Map::load`. IW7's blocker brushes (`script_brushmodel` of the
    /// group) hold no surfaces: they only clip; the scripts take the
    /// scriptable away with the clip.
    pub fn find_props(&mut self, map: &Iw7Map) {
        for b in &mut self.blockers {
            b.props = map
                .static_models
                .iter()
                .enumerate()
                .filter(|(_, sm)| sm.scriptable.and_then(|k| map.scriptables.get(k as usize)).is_some_and(|sc| sc.targetname == b.group))
                .map(|(index, sm)| BlockerProp { index, origin: sm.origin, axis: sm.axis.map(|r| r.map(|v| v * sm.scale)) })
                .collect();
        }
    }

    pub fn from_parts(ents: &[Ent], trigger_hulls: &[Vec<([f32; 3], [f32; 3])>], nodes: &[crate::map::PathNode]) -> Self {
        let mut out = ZmLayout::default();
        let named = |name: &str| named(ents, name);
        let first_target = |e: &Ent| first_target(ents, e);

        out.starts = named("default_player_start").into_iter().map(Ent::spot).collect();
        out.respawns = named("player_respawn_loc").into_iter().map(Ent::spot).collect();

        for v in named("spawn_volume").into_iter().filter(|e| e.is("classname", "info_volume")) {
            let origin = v.origin();
            let name = area_name(v.get("script_linkname").unwrap_or(""));
            if name.is_empty() {
                continue;
            }
            let model = v.get("model").and_then(|m| model_index(m, '?'));
            let boxes = model
                .and_then(|n| trigger_hulls.get(n as usize))
                .map(|hulls| {
                    hulls
                        .iter()
                        .map(|(mid, half)| (std::array::from_fn(|k| origin[k] + mid[k] - half[k]), std::array::from_fn(|k| origin[k] + mid[k] + half[k])))
                        .collect()
                })
                .unwrap_or_default();
            let spawners = v
                .get("target")
                .map(|t| {
                    named(t)
                        .into_iter()
                        .map(|s| {
                            let params = s.get("script_parameters").unwrap_or("");
                            // How they come in (60661): out of the ground,
                            // the ceiling or a wall; else they walk in.
                            let how = s.get("_k60661").unwrap_or("");
                            Spawner {
                                spot: s.spot(),
                                ground: how == "spawn_ground" || (how.is_empty() && params.starts_with("ground_spawn")),
                                ceiling: how == "spawn_ceiling",
                                no_boards: params.contains("no_boards"),
                                goal: first_target(s).map(Ent::origin),
                            }
                        })
                        .collect()
                })
                .unwrap_or_default();
            out.areas.push(Area { name, origin, model, boxes, spawners });
        }

        for w in named("window_entrance") {
            let attack = w.get("target").map(|t| named(t).into_iter().map(Ent::origin).collect()).unwrap_or_default();
            // The repair spot: the nearest `secure_window` interaction.
            let o = w.origin();
            let inside = ents
                .iter()
                .filter(|e| e.is("targetname", "interaction") && e.is("script_noteworthy", "secure_window"))
                .min_by(|a, b| dist(a.origin(), o).total_cmp(&dist(b.origin(), o))).filter(|i| dist(i.origin(), o) < 160.0);
            out.windows.push(Window {
                outside: w.spot(),
                attack,
                inside: inside.map(Ent::spot),
                boards: inside.and_then(first_target).map(Ent::origin),
            });
        }

        // Blockers, by the name their pieces share.
        let mut groups: Vec<String> = Vec::new();
        for e in ents {
            let note = e.get("script_noteworthy").unwrap_or("");
            let blocker = note.starts_with("debris_") || note == "team_door_switch";
            if blocker && (e.is("targetname", "interaction") || e.is("targetname", "door_buy"))
                && let Some(t) = e.get("target")
                && !groups.iter().any(|g| g == t)
            {
                groups.push(t.to_owned());
            }
        }
        for g in groups {
            let pieces: Vec<&Ent> = ents.iter().filter(|e| e.get("target") == Some(g.as_str()) && e.get("classname") != Some("script_brushmodel")).collect();
            let note = pieces.iter().find_map(|e| e.get("script_noteworthy").filter(|n| n.starts_with("debris_") || *n == "team_door_switch")).unwrap_or("");
            let (kind, cost) = match note.strip_prefix("debris_") {
                Some(c) => (BlockerKind::Debris, c.parse().unwrap_or(1000)),
                None => (BlockerKind::TeamDoor, 0),
            };
            let sides = pieces
                .iter()
                .filter(|e| e.is("targetname", "interaction"))
                .map(|e| (e.spot(), e.get("_k60666").unwrap_or("").to_owned()))
                .collect();
            let triggers = pieces
                .iter()
                .filter(|e| e.is("targetname", "door_buy"))
                .filter_map(|e| Some((e.origin(), model_index(e.get("model")?, '?')?)))
                .collect();
            let brushes = named(&g)
                .into_iter()
                .filter(|e| e.is("classname", "script_brushmodel"))
                .filter_map(|e| Some((model_index(e.get("model")?, '*')?, e.origin())))
                .collect();
            out.blockers.push(Blocker { kind, cost, sides, triggers, brushes, group: g, props: Vec::new() });
        }

        for e in ents.iter().filter(|e| e.is("targetname", "interaction")) {
            let note = e.get("script_noteworthy").unwrap_or("");
            if note.starts_with("perk_machine_") {
                out.perks.push(Machine { name: note.to_owned(), spot: e.spot() });
            } else if note == "pap_upgrade" && out.pap.is_none() {
                out.pap = Some(e.spot());
            } else if note == "fast_travel" {
                let dests = e.get("target").map(|t| named(t).into_iter().filter(|d| d.is("classname", "script_struct")).map(Ent::spot).collect()).unwrap_or_default();
                out.portals.push(Portal { spot: e.spot(), area: e.get("_k60666").unwrap_or("").to_owned(), dests });
            } else if note == "generator" {
                let lever = first_target(e).and_then(|l| Some((l.get("model")?.to_owned(), l.spot())));
                let areas = e.get("script_parameters").unwrap_or("").split(',').map(str::trim).filter(|a| !a.is_empty()).map(str::to_owned).collect();
                out.generators.push(Generator { spot: e.spot(), areas, lever });
            } else if e.get("_k502") == Some("wall_buy") && note.starts_with("iw7_") {
                out.wall_buys.push(WallBuy { weapon: note.to_owned(), spot: e.spot(), model_spot: first_target(e).map(Ent::spot) });
            }
        }
        out.wheels = ents.iter().filter(|e| e.is("script_noteworthy", "spinner") && e.is("classname", "script_struct")).map(Ent::spot).collect();
        out.dog_spots = named("dog_spawner").into_iter().map(Ent::spot).collect();

        // Traversals: a node's negotiation links, begin to end.
        for n in nodes {
            for &(to, _, negotiation) in &n.links {
                if let (true, Some(end)) = (negotiation, nodes.get(usize::from(to))) {
                    out.traversals.push(Traversal { begin: n.origin, end: end.origin, animscript: n.animscript.clone() });
                }
            }
        }
        out
    }

    /// The area a point is in (the smallest box holding it).
    pub fn area_at(&self, p: [f32; 3]) -> Option<&Area> {
        let vol = |(lo, hi): &([f32; 3], [f32; 3])| (hi[0] - lo[0]) * (hi[1] - lo[1]) * (hi[2] - lo[2]);
        self.areas
            .iter()
            .filter_map(|a| a.boxes.iter().filter(|b| (0..3).all(|k| p[k] >= b.0[k] && p[k] <= b.1[k])).map(vol).reduce(f32::min).map(|v| (a, v)))
            .min_by(|a, b| a.1.total_cmp(&b.1))
            .map(|(a, _)| a)
    }

    /// The area a point is in, else the one whose boxes come nearest it
    /// (a player start or a perk machine just outside every box).
    pub fn nearest_area(&self, p: [f32; 3]) -> Option<&Area> {
        if let Some(a) = self.area_at(p) {
            return Some(a);
        }
        let gap = |(lo, hi): &([f32; 3], [f32; 3])| -> f32 {
            (0..3).map(|k| (lo[k] - p[k]).max(p[k] - hi[k]).max(0.0).powi(2)).sum::<f32>()
        };
        self.areas
            .iter()
            .filter_map(|a| a.boxes.iter().map(gap).reduce(f32::min).map(|d| (a, d)))
            .min_by(|a, b| a.1.total_cmp(&b.1))
            .map(|(a, _)| a)
    }

    /// One line: what it found.
    pub fn summary(&self) -> String {
        format!(
            "{} player starts, {} respawns, {} areas ({} spawners), {} windows, {} debris/doors ({} placed models in them), {} perk machines, {} wheel spots, pack-a-punch {}, {} portals, {} generators, {} wall weapons, {} dog spots, {} traversals",
            self.starts.len(),
            self.respawns.len(),
            self.areas.len(),
            self.areas.iter().map(|a| a.spawners.len()).sum::<usize>(),
            self.windows.len(),
            self.blockers.len(),
            self.blockers.iter().map(|b| b.props.len()).sum::<usize>(),
            self.perks.len(),
            self.wheels.len(),
            if self.pap.is_some() { "yes" } else { "no" },
            self.portals.len(),
            self.generators.len(),
            self.wall_buys.len(),
            self.dog_spots.len(),
            self.traversals.len(),
        )
    }
}

fn named<'a>(ents: &'a [Ent], name: &str) -> Vec<&'a Ent> {
    ents.iter().filter(|e| e.is("targetname", name)).collect()
}

fn first_target<'a>(ents: &'a [Ent], e: &Ent) -> Option<&'a Ent> {
    e.get("target").and_then(|t| named(ents, t).into_iter().next())
}

fn dist(a: [f32; 3], b: [f32; 3]) -> f32 {
    ((a[0] - b[0]).powi(2) + (a[1] - b[1]).powi(2) + (a[2] - b[2]).powi(2)).sqrt()
}

#[cfg(test)]
mod tests {
    use super::*;

    const TEXT: &str = r#"{
"classname" "info_volume"
"targetname" "spawn_volume"
"script_linkname" "pf7_moon"
"target" "moon_spawner"
"origin" "100 0 0"
"model" "?0"
}
{
"classname" "script_struct"
"targetname" "moon_spawner"
"script_parameters" "ground_spawn_no_boards"
"origin" "120 10 0"
}
{
"classname" "script_struct"
"targetname" "moon_spawner"
"target" "goal1"
"origin" "150 10 0"
}
{
"classname" "script_struct"
"targetname" "moon_spawner"
"script_parameters" "ground_spawn_no_boards"
"_k60661" "spawn_ceiling"
"origin" "130 10 200"
}
{
"classname" "script_struct"
"targetname" "goal1"
"origin" "160 10 0"
}
{
"classname" "script_struct"
"targetname" "interaction"
"script_noteworthy" "debris_1250"
"_k60666" "moon"
"target" "pf7_auto9"
"origin" "10 20 0"
}
{
"classname" "trigger_use"
"targetname" "door_buy"
"script_noteworthy" "debris_1250"
"target" "pf7_auto9"
"model" "?1"
"origin" "12 22 0"
}
{
"classname" "script_brushmodel"
"targetname" "pf7_auto9"
"target" "pf7_auto9"
"model" "*9"
"origin" "12 24 0"
}
{
"classname" "script_struct"
"targetname" "interaction"
"script_noteworthy" "perk_machine_tough"
"origin" "1 2 3"
"angles" "0 90 0"
}
{
"classname" "script_struct"
"script_noteworthy" "spinner"
"origin" "5 5 5"
}
"#;

    #[test]
    fn reads_areas_blockers_and_machines() {
        let ents = parse_entities(TEXT);
        let hulls = vec![vec![([0.0; 3], [50.0, 50.0, 50.0])]];
        let l = ZmLayout::from_parts(&ents, &hulls, &[]);
        assert_eq!(l.areas.len(), 1);
        let a = &l.areas[0];
        assert_eq!(a.name, "moon");
        assert_eq!(a.boxes, vec![([50.0, -50.0, -50.0], [150.0, 50.0, 50.0])]);
        assert_eq!(a.spawners.len(), 3);
        assert!(a.spawners[0].ground && a.spawners[0].no_boards);
        assert!(a.spawners[2].ceiling && !a.spawners[2].ground);
        assert_eq!(a.spawners[1].goal, Some([160.0, 10.0, 0.0]));
        assert_eq!(l.area_at([120.0, 0.0, 0.0]).map(|a| a.name.as_str()), Some("moon"));
        assert_eq!(a.model, Some(0));
        assert!(l.area_at([400.0, 0.0, 0.0]).is_none());
        assert_eq!(l.nearest_area([400.0, 0.0, 0.0]).map(|a| a.name.as_str()), Some("moon"));
        assert_eq!(l.blockers.len(), 1);
        let b = &l.blockers[0];
        assert_eq!((b.kind, b.cost), (BlockerKind::Debris, 1250));
        assert_eq!(b.sides[0].1, "moon");
        assert_eq!(b.triggers, vec![([12.0, 22.0, 0.0], 1)]);
        assert_eq!(b.brushes, vec![(9, [12.0, 24.0, 0.0])]);
        assert_eq!(l.perks[0].name, "perk_machine_tough");
        assert_eq!(l.wheels.len(), 1);
    }

    #[test]
    fn area_names_lose_the_prefab_prefix() {
        assert_eq!(area_name("pf7_moon_rocket_space"), "moon_rocket_space");
        assert_eq!(area_name("hidden_room"), "hidden_room");
    }
}
