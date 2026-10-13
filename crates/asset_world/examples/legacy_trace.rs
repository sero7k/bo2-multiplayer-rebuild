//! bo2mp offline: a CoD4 / World at War map's brush model bounds and a drop
//! straight down: `legacy_trace <zone.ff> <model#> <x> <y>`.
fn main() {
    let a: Vec<String> = std::env::args().collect();
    let zone = t4::load_map(std::path::Path::new(&a[1])).unwrap().zone;
    let clip = zone.clip_map().unwrap();
    let c = asset_world::world_iw3::build_iw3_clip_collision(clip).unwrap();
    let m: usize = a[2].parse().unwrap();
    if let Some(cm) = c.cmodels.get(m) {
        println!("cmodel {m}: mins {:?} maxs {:?} brushes {}", cm.mins, cm.maxs, cm.num_brushes);
    }
    let (x, y): (f32, f32) = (a[3].parse().unwrap(), a[4].parse().unwrap());
    let mut z = 3000.0;
    for _ in 0..8 {
        let h = c.sweep_box([x, y, z], [x, y, -4000.0], [-15.0, -15.0, 0.0], [15.0, 15.0, 70.0], asset_world::MASK_PLAYER_SOLID);
        if h.fraction >= 1.0 {
            println!("nothing below {z}");
            break;
        }
        println!("floor at z {:.1} (from {z})", h.endpos[2]);
        z = h.endpos[2] - 80.0;
    }
}
