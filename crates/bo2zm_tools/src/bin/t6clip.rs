//! bo2zm: measure a Black Ops II map's collision, read-only.
//! `t6clip [zone.ff] [x y z ...]` (default Nuketown, at its player start)
//! counts brush contents bits, then lists every brush whose player-sized hull
//! holds each point: the brushes a player standing there would be inside.

use std::collections::BTreeMap;
use std::path::PathBuf;
use std::process::ExitCode;

use asset_t6::{ClipBrushRef, ClipRef, capture_zone};

const DEFAULT_ZONE: &str =
    r"C:\Program Files (x86)\Steam\steamapps\common\Call of Duty Black Ops II\zone\all\zm_nuked.ff";

/// IW4's standing player box.
const PLAYER_MINS: [f32; 3] = [-15.0, -15.0, 0.0];
const PLAYER_MAXS: [f32; 3] = [15.0, 15.0, 70.0];

/// Every plane of a brush: the six box planes, then the extra sides.
fn planes(b: &ClipBrushRef) -> Vec<[f32; 4]> {
    let mut p = vec![
        [1.0, 0.0, 0.0, b.maxs[0]],
        [-1.0, 0.0, 0.0, -b.mins[0]],
        [0.0, 1.0, 0.0, b.maxs[1]],
        [0.0, -1.0, 0.0, -b.mins[1]],
        [0.0, 0.0, 1.0, b.maxs[2]],
        [0.0, 0.0, -1.0, -b.mins[2]],
    ];
    p.extend(b.sides.iter().map(|&(n, d, _)| [n[0], n[1], n[2], d]));
    p
}

/// Whether the box at `origin` overlaps the brush: every plane pushed out by
/// the box's reach along its normal must still have the origin behind it.
fn box_inside(b: &ClipBrushRef, origin: [f32; 3]) -> bool {
    planes(b).iter().all(|p| {
        let reach: f32 = (0..3)
            .map(|i| {
                if p[i] < 0.0 {
                    p[i] * PLAYER_MINS[i]
                } else {
                    p[i] * PLAYER_MAXS[i]
                }
            })
            .sum();
        p[0] * origin[0] + p[1] * origin[1] + p[2] * origin[2] - (p[3] + reach) < 0.0
    })
}

/// The brushes listed under one leaf brush node (0 = none), by the same
/// visit rule the engine uses: brush lists at positive counts, else the
/// node after a negative count and each nonzero child offset.
fn leaf_brushes(clip: &ClipRef, root: i32) -> Vec<u16> {
    let mut out = Vec::new();
    if root <= 0 {
        return out;
    }
    let mut pending = vec![root as usize];
    while let Some(index) = pending.pop() {
        let Some(node) = clip.leafbrush_nodes.get(index) else {
            continue;
        };
        if node.leaf_brush_count > 0 {
            out.extend_from_slice(&node.brushes);
            continue;
        }
        if node.leaf_brush_count < 0 {
            pending.push(index + 1);
        }
        for off in node.child_offset {
            if off != 0 {
                pending.push(index + usize::from(off));
            }
        }
    }
    out
}

fn main() -> ExitCode {
    let mut args = std::env::args().skip(1).peekable();
    let path = match args.peek() {
        Some(a) if a.ends_with(".ff") => PathBuf::from(args.next().unwrap_or_default()),
        _ => PathBuf::from(DEFAULT_ZONE),
    };
    let nums: Vec<f32> = args.filter_map(|a| a.parse().ok()).collect();
    let capture = match capture_zone(&path) {
        Ok(c) => c,
        Err(e) => {
            eprintln!("t6clip: {e}");
            return ExitCode::FAILURE;
        }
    };
    // T6CLIP_RAY=<model>@x,y[;x,y...]: where a straight-up line at (x, y) in
    // that model's own space crosses its collision triangles, bottom to top
    // (the bus floor, its ceiling and roof top).
    if let Some((want, spots)) = std::env::var("T6CLIP_RAY")
        .ok()
        .and_then(|v| v.split_once('@').map(|(m, s)| (m.to_owned(), s.to_owned())))
    {
        for m in capture.xmodels.iter().filter(|m| m.name == want) {
            for spot in spots.split(';') {
                let xy: Vec<f32> = spot.split(',').filter_map(|v| v.trim().parse().ok()).collect();
                let [x, y] = xy[..] else { continue };
                let mut hits = Vec::new();
                for c in &m.coll_surfs {
                    for t in &c.tris {
                        let [plane, s, tv] = *t;
                        if plane[2].abs() < 1e-4 {
                            continue;
                        }
                        let z = (plane[3] - plane[0] * x - plane[1] * y) / plane[2];
                        let p = [x, y, z];
                        let d = |v: [f32; 4]| v[0] * p[0] + v[1] * p[1] + v[2] * p[2] - v[3];
                        let (a, bb) = (d(s), d(tv));
                        if a >= 0.0 && bb >= 0.0 && a + bb <= 1.0 {
                            hits.push((z, plane[2], c.surf_flags));
                        }
                    }
                }
                hits.sort_by(|a, b| a.0.total_cmp(&b.0));
                println!("{} at {x},{y}: {:?}", m.name, hits);
            }
        }
        return ExitCode::SUCCESS;
    }
    if let Ok(out) = std::env::var("T6CLIP_ENTS") {
        let text: String = capture.map_ents.concat();
        if let Err(e) = std::fs::write(&out, text) {
            eprintln!("t6clip: {out}: {e}");
        }
    }
    let Some(clip) = capture.clip.as_ref() else {
        eprintln!("t6clip: {} has no clipMap", path.display());
        return ExitCode::FAILURE;
    };

    let mut bits: BTreeMap<u32, usize> = BTreeMap::new();
    for b in &clip.brushes {
        for bit in 0..32 {
            if (b.contents as u32) & (1 << bit) != 0 {
                *bits.entry(1 << bit).or_default() += 1;
            }
        }
    }
    println!("{} brushes; contents bits:", clip.brushes.len());
    for (bit, n) in &bits {
        println!("   {bit:#010x}  {n}");
    }
    let mut mats: BTreeMap<u32, usize> = BTreeMap::new();
    for m in &clip.materials {
        *mats.entry(m.content_flags as u32).or_default() += 1;
    }
    println!("{} clip materials; content flags:", clip.materials.len());
    for (flags, n) in &mats {
        println!("   {flags:#010x}  {n}");
    }

    let mut points: Vec<[f32; 3]> = nums.chunks_exact(3).map(|c| [c[0], c[1], c[2]]).collect();
    // T6CLIP_FIND=<text>: every map entity with a value holding that text,
    // all its keys; for a brush entity also its model's box placed at its
    // origin, and whether a player box at each point is inside that box or
    // inside one of the model's own brushes (which brush model a trigger
    // uses, where it sits, and whether a point is really in it).
    if let Ok(want) = std::env::var("T6CLIP_FIND") {
        for text in &capture.map_ents {
            for e in asset_t6::parse_entities(text) {
                if !e.fields.iter().any(|(_, v)| v.contains(want.as_str())) {
                    continue;
                }
                let kv: Vec<String> = e.fields.iter().map(|(k, v)| format!("{k}={v}")).collect();
                println!("ent {}", kv.join(" "));
                let model = e.fields.iter().find(|(k, _)| k == "model").and_then(|(_, v)| {
                    v.strip_prefix('*')?.parse::<usize>().ok()
                });
                let (Some(n), Some(m)) = (model, model.and_then(|n| clip.cmodels.get(n))) else {
                    continue;
                };
                let o = e.vec3("origin").unwrap_or([0.0; 3]);
                let list = leaf_brushes(clip, m.leaf.leaf_brush_node);
                let lo: [f32; 3] = std::array::from_fn(|i| o[i] + m.mins[i]);
                let hi: [f32; 3] = std::array::from_fn(|i| o[i] + m.maxs[i]);
                println!(
                    "   *{n}: {} brushes (map info {}), box [{:.0} {:.0} {:.0}]..[{:.0} {:.0} {:.0}]",
                    list.len(),
                    m.shares_map_info,
                    lo[0],
                    lo[1],
                    lo[2],
                    hi[0],
                    hi[1],
                    hi[2]
                );
                for p in &points {
                    let in_box = (0..3).all(|i| {
                        p[i] + PLAYER_MAXS[i] >= lo[i] && p[i] + PLAYER_MINS[i] <= hi[i]
                    });
                    let local: [f32; 3] = std::array::from_fn(|i| p[i] - o[i]);
                    let in_brush = list
                        .iter()
                        .filter_map(|&b| clip.brushes.get(usize::from(b)))
                        .any(|b| box_inside(b, local));
                    println!(
                        "      at [{:.0} {:.0} {:.0}]: in box {in_box}, in a brush {in_brush}",
                        p[0], p[1], p[2]
                    );
                }
            }
        }
    }
    if points.is_empty() {
        for text in &capture.map_ents {
            for e in asset_t6::parse_entities(text) {
                if e.classname() == "info_player_start"
                    && let Some(o) = e.vec3("origin")
                {
                    points.push(o);
                    points.push([o[0], o[1], o[2] + 228.0]);
                }
            }
        }
    }
    // Leaves the tree reaches from node 0; a negative child c is leaf -1-c.
    let mut reach = vec![false; clip.leaves.len()];
    let mut stack = if clip.nodes.is_empty() {
        vec![]
    } else {
        vec![0i32]
    };
    let mut seen_nodes = vec![false; clip.nodes.len()];
    while let Some(c) = stack.pop() {
        if c < 0 {
            if let Some(r) = reach.get_mut((-1 - c) as usize) {
                *r = true;
            }
            continue;
        }
        let i = c as usize;
        if i >= clip.nodes.len() || seen_nodes[i] {
            continue;
        }
        seen_nodes[i] = true;
        stack.extend(clip.nodes[i].children.map(i32::from));
    }
    println!(
        "tree reaches {} of {} nodes and {} of {} leaves",
        seen_nodes.iter().filter(|v| **v).count(),
        clip.nodes.len(),
        reach.iter().filter(|v| **v).count(),
        clip.leaves.len()
    );
    let mut world = vec![false; clip.brushes.len()];
    let mut world_reached = vec![false; clip.brushes.len()];
    for (li, leaf) in clip.leaves.iter().enumerate() {
        for b in leaf_brushes(clip, leaf.leaf_brush_node) {
            if let Some(w) = world.get_mut(usize::from(b)) {
                *w = true;
            }
            if reach[li]
                && let Some(w) = world_reached.get_mut(usize::from(b))
            {
                *w = true;
            }
        }
    }
    println!(
        "brushes listed by reached leaves: {}; by any leaf: {}",
        world_reached.iter().filter(|v| **v).count(),
        world.iter().filter(|v| **v).count()
    );
    let mut owner: BTreeMap<usize, Vec<usize>> = BTreeMap::new();
    for (i, m) in clip.cmodels.iter().enumerate() {
        for b in leaf_brushes(clip, m.leaf.leaf_brush_node) {
            owner.entry(usize::from(b)).or_default().push(i);
        }
    }
    println!(
        "tree: {} nodes, {} leaves, {} leaf brush nodes, {} brush models ({} share the map's info); {} brushes listed by leaves",
        clip.nodes.len(),
        clip.leaves.len(),
        clip.leafbrush_nodes.len(),
        clip.cmodels.len(),
        clip.cmodels.iter().filter(|m| m.shares_map_info).count(),
        world.iter().filter(|w| **w).count()
    );
    if let Some(m) = clip.cmodels.first() {
        println!(
            "model 0: {} brushes, mins [{:.0}, {:.0}, {:.0}] maxs [{:.0}, {:.0}, {:.0}]",
            leaf_brushes(clip, m.leaf.leaf_brush_node).len(),
            m.mins[0],
            m.mins[1],
            m.mins[2],
            m.maxs[0],
            m.maxs[1],
            m.maxs[2]
        );
    }
    for p in points {
        let hits: Vec<usize> = clip
            .brushes
            .iter()
            .enumerate()
            .filter(|(_, b)| box_inside(b, p))
            .map(|(i, _)| i)
            .collect();
        println!(
            "player box at [{:.1}, {:.1}, {:.1}]: inside {} brushes",
            p[0],
            p[1],
            p[2],
            hits.len()
        );
        for &i in hits.iter().take(20) {
            let b = &clip.brushes[i];
            let whose = format!(
                "{}{}",
                match (world_reached[i], world[i]) {
                    (true, _) => "WORLD WALL",
                    (false, true) => "entity leaf only",
                    (false, false) => "no leaf",
                },
                owner
                    .get(&i)
                    .map_or_else(String::new, |m| format!(", brush model {m:?}"))
            );
            println!(
                "   brush {i} ({whose}): contents {:#010x}, {} extra sides, mins [{:.0}, {:.0}, {:.0}] maxs [{:.0}, {:.0}, {:.0}]",
                b.contents as u32,
                b.sides.len(),
                b.mins[0],
                b.mins[1],
                b.mins[2],
                b.maxs[0],
                b.maxs[1],
                b.maxs[2]
            );
            println!(
                "      table contents {:#x?}",
                clip.brush_contents.get(i).map(|c| *c as u32)
            );
            println!(
                "      axial cflags {:x?} sflags {:x?}; side sflags {:x?}",
                b.axial_cflags,
                b.axial_sflags,
                b.sides.iter().map(|s| s.2).collect::<Vec<_>>()
            );
        }
    }
    let mut same = 0;
    let mut differ: BTreeMap<(u32, u32, bool), usize> = BTreeMap::new();
    for (i, b) in clip.brushes.iter().enumerate() {
        let t = clip.brush_contents.get(i).copied().unwrap_or(-1);
        if t == b.contents {
            same += 1;
        } else {
            *differ
                .entry((b.contents as u32, t as u32, owner.contains_key(&i)))
                .or_default() += 1;
        }
    }
    println!(
        "table vs brush contents: {} brushes, {} table entries, {same} same",
        clip.brushes.len(),
        clip.brush_contents.len()
    );
    for ((b, t, owned), n) in differ {
        println!("   brush {b:#010x} table {t:#010x} model-owned {owned}: {n}");
    }
    let owned: usize = owner.len();
    let both = owner.keys().filter(|&&i| world[i]).count();
    println!("{owned} brushes owned by brush models; {both} of them also in world leaves");
    for (i, m) in clip.cmodels.iter().enumerate().take(12) {
        let list = leaf_brushes(clip, m.leaf.leaf_brush_node);
        println!(
            "   model {i}: {} brushes {:?}.., contents {:#x}, mins [{:.0}, {:.0}, {:.0}] maxs [{:.0}, {:.0}, {:.0}]",
            list.len(),
            &list[..list.len().min(4)],
            m.leaf.brush_contents as u32,
            m.mins[0],
            m.mins[1],
            m.mins[2],
            m.maxs[0],
            m.maxs[1],
            m.maxs[2]
        );
    }
    // T6CLIP_MODELS=229,231: every brush of those brush models, with its
    // contents and surface flags (which ones block, which can be climbed).
    let picked: Vec<usize> = std::env::var("T6CLIP_MODELS")
        .unwrap_or_default()
        .split(',')
        .filter_map(|s| s.trim().parse().ok())
        .collect();
    for i in picked {
        let Some(m) = clip.cmodels.get(i) else {
            println!("model {i}: none");
            continue;
        };
        println!(
            "model {i}: contents {:#x}, mins [{:.0}, {:.0}, {:.0}] maxs [{:.0}, {:.0}, {:.0}]",
            m.leaf.brush_contents as u32,
            m.mins[0],
            m.mins[1],
            m.mins[2],
            m.maxs[0],
            m.maxs[1],
            m.maxs[2]
        );
        for j in leaf_brushes(clip, m.leaf.leaf_brush_node) {
            let b = &clip.brushes[usize::from(j)];
            println!(
                "   brush {j}: contents {:#010x}, mins [{:.0}, {:.0}, {:.0}] maxs [{:.0}, {:.0}, {:.0}], axial sflags {:x?}, side sflags {:x?}",
                b.contents as u32,
                b.mins[0],
                b.mins[1],
                b.mins[2],
                b.maxs[0],
                b.maxs[1],
                b.maxs[2],
                b.axial_sflags,
                b.sides.iter().map(|s| s.2).collect::<Vec<_>>()
            );
        }
    }
    // T6CLIP_SFLAGS=8: how many brushes carry each surface flag bit, and
    // the first ones with the asked bits (where the map's ladders are).
    if let Some(want) = std::env::var("T6CLIP_SFLAGS")
        .ok()
        .and_then(|v| u32::from_str_radix(v.trim_start_matches("0x"), 16).ok())
    {
        let mut sbits: BTreeMap<u32, usize> = BTreeMap::new();
        let mut shown = 0;
        for (i, b) in clip.brushes.iter().enumerate() {
            let all = b
                .axial_sflags
                .iter()
                .flatten()
                .map(|f| *f as u32)
                .chain(b.sides.iter().map(|s| s.2 as u32))
                .fold(0, |a, f| a | f);
            for bit in 0..32 {
                if all & (1 << bit) != 0 {
                    *sbits.entry(1 << bit).or_default() += 1;
                }
            }
            if all & want != 0 && shown < 60 {
                shown += 1;
                println!(
                    "   brush {i}: contents {:#010x}, sflags {all:#x}, mins [{:.0}, {:.0}, {:.0}] maxs [{:.0}, {:.0}, {:.0}], models {:?}",
                    b.contents as u32,
                    b.mins[0],
                    b.mins[1],
                    b.mins[2],
                    b.maxs[0],
                    b.maxs[1],
                    b.maxs[2],
                    owner.get(&i)
                );
            }
        }
        for m in &clip.materials {
            if m.surface_flags as u32 & want != 0 {
                println!(
                    "   clip material {}: surface flags {:#x}, contents {:#x}",
                    m.name, m.surface_flags as u32, m.content_flags as u32
                );
            }
        }
        println!("brushes by surface flag bit:");
        for (bit, n) in &sbits {
            println!("   {bit:#010x}  {n}");
        }
    }
    ExitCode::SUCCESS
}
