//! A Black Ops II zone's weapon camos and the guns that use them, and its
//! camo rows: `t6_camo_dump <zone.ff> [gun]`.
fn main() {
    let mut a = std::env::args().skip(1);
    let path = a.next().expect("zone");
    let gun = a.next().unwrap_or_else(|| "an94_mp".to_owned());
    let c = asset_t6::capture_zone(std::path::Path::new(&path)).expect("capture");
    println!("{} weapon camos", c.weapon_camos.len());
    for w in c.weapons.iter().filter(|w| w.name.contains(&gun)).take(3) {
        println!("weapon {} camo `{}` gun models {:?}", w.name, w.camo, w.gun_models.iter().take(2).collect::<Vec<_>>());
    }
    for cam in &c.weapon_camos {
        println!("camo {} solid {:?} pattern {:?}: {} sets, {} material sets", cam.name, cam.solid_base_image, cam.pattern_base_image, cam.sets.len(), cam.material_sets.len());
        for (i, s) in cam.sets.iter().enumerate().take(3) {
            println!("  set {i}: {s:?}");
        }
        for (i, s) in cam.material_sets.iter().enumerate().take(45) {
            println!("  material set {i}: {:?}", s.iter().map(|m| (m.base.clone(), m.camo.clone(), m.consts, m.replace_flags)).collect::<Vec<_>>());
        }
        break;
    }
    for t in c.string_tables.iter().filter(|t| t.name.to_ascii_lowercase().contains("attachment") || t.name.contains("weaponoption")) {
        println!("table {} {}x{}", t.name, t.rows, t.columns);
        for r in 0..t.rows {
            let row: Vec<&str> = (0..t.columns).map(|k| t.cells[r * t.columns + k].as_str()).collect();
            if row.get(1).is_some_and(|c| *c == "camo") || row.get(2).is_some_and(|c| *c == "camo") || row.iter().any(|c| c.contains("camo")) {
                println!("  {}", row.join(" | "));
            }
        }
    }
}
