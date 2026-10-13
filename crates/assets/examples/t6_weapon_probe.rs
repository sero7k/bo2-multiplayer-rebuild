//! Prints T6 weapons' damage tiers and a few timings:
//! `t6_weapon_probe <zone.ff> [name-pattern]`.
use fastfile_t6::layout::WeaponDef as d;

fn main() {
    let a: Vec<String> = std::env::args().collect();
    let c = asset_t6::capture_zone(std::path::Path::new(&a[1])).unwrap();
    let pat = a.get(2).cloned().unwrap_or_default();
    for w in c.weapons.iter().filter(|w| w.name.contains(&pat)) {
        let dmg: Vec<i32> = (0..6).map(|i| w.def_i32(d::damage + i * 4)).collect();
        let rng: Vec<f32> = (0..6).map(|i| w.def_f32(d::damageRange + i * 4)).collect();
        println!(
            "{:28} dmg {:?} ranges {:?} fire {} burstDelay {} minPlayer {}",
            w.name,
            dmg,
            rng,
            w.def_i32(d::iFireTime),
            w.def_i32(d::iBurstDelayTime),
            w.def_i32(d::minPlayerDamage)
        );
    }
}
