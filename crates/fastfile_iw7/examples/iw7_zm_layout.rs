//! An Infinite Warfare Zombies map's layout (`fastfile_iw7::zombies`):
//! `iw7_zm_layout <cp_map.ff>...` prints its areas, windows, blockers,
//! machines, portals, generators, wall weapons and traversals.
use fastfile_iw7::map::{Iw7Map, MapSources};
use fastfile_iw7::zombies::ZmLayout;

fn main() {
    for path in std::env::args().skip(1) {
        let map = match Iw7Map::load(&MapSources::beside(std::path::Path::new(&path))) {
            Ok(m) => m,
            Err(e) => {
                println!("{path}: {e}");
                continue;
            }
        };
        let l = ZmLayout::read(&map);
        println!("== {}: {}", map.name, l.summary());
        let at = |p: [f32; 3]| l.nearest_area(p).map_or("-", |a| a.name.as_str()).to_owned();
        for a in &l.areas {
            let ground = a.spawners.iter().filter(|s| s.ground).count();
            let free = a.spawners.iter().filter(|s| s.no_boards).count();
            let ceiling = a.spawners.iter().filter(|s| s.ceiling).count();
            println!("area {} at {:?}: {} boxes, {} spawners ({ground} ground, {ceiling} ceiling, {free} no boards)", a.name, a.origin, a.boxes.len(), a.spawners.len());
        }
        for s in &l.starts {
            println!("start {:?} in {}", s.origin, at(s.origin));
        }
        for w in &l.windows {
            println!("window {:?} yaw {:.0} in {} inside {:?} boards {:?} attack {}", w.outside.origin, w.outside.angles[1], at(w.outside.origin), w.inside.map(|i| i.origin), w.boards, w.attack.len());
        }
        for b in &l.blockers {
            println!("blocker {} {:?} {} sides {:?} triggers {:?} brushes {:?}", b.group, b.kind, b.cost, b.sides.iter().map(|(s, a)| format!("{a}@{:?}", s.origin)).collect::<Vec<_>>(), b.triggers, b.brushes);
        }
        for m in &l.perks {
            println!("machine {} at {:?} yaw {:.0} in {}", m.name, m.spot.origin, m.spot.angles[1], at(m.spot.origin));
        }
        for w in &l.wheels {
            println!("wheel {:?} yaw {:.0} in {}", w.origin, w.angles[1], at(w.origin));
        }
        if let Some(p) = l.pap {
            println!("pack-a-punch {:?} in {}", p.origin, at(p.origin));
        }
        for p in &l.portals {
            println!("portal {:?} area {} dests {}", p.spot.origin, p.area, p.dests.len());
        }
        for g in &l.generators {
            println!("generator {:?} powers {:?} lever {:?}", g.spot.origin, g.areas, g.lever.as_ref().map(|l| &l.0));
        }
        for w in &l.wall_buys {
            println!("wall weapon {} at {:?} in {}", w.weapon, w.spot.origin, at(w.spot.origin));
        }
        let mut anims: Vec<&str> = l.traversals.iter().map(|t| t.animscript.as_str()).collect();
        anims.sort_unstable();
        anims.dedup();
        println!("traversal animscripts {anims:?}");
    }
}
