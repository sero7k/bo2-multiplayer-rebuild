//! Lists Black Ops II's Select Fire pairs in a zone: each `au_<gun>_sf`
//! unique's alternate weapon and that weapon's fire type, fire and burst
//! times, alternate raise / drop: `t6_sf_probe <common_mp.ff>`.
use fastfile_t6::layout::WeaponDef as d;
use fastfile_t6::layout::WeaponVariantDef as v;

fn main() {
    let args: Vec<String> = std::env::args().collect();
    let c = asset_t6::capture_zone(std::path::Path::new(&args[1])).unwrap();
    for u in c.attachment_uniques.iter().filter(|u| u.name.ends_with("_sf")) {
        let w = c.weapons.iter().find(|w| w.name == u.alt_weapon_name);
        match w {
            Some(w) => println!(
                "{} -> {} fireType {} fire {} burstDelay {} altRaise {} altDrop {}",
                u.name,
                w.name,
                w.def_i32(d::fireType),
                w.def_i32(d::iFireTime),
                w.def_i32(d::iBurstDelayTime),
                w.var_i32(v::iAltRaiseTime),
                w.def_i32(d::iAltDropTime)
            ),
            None => println!("{} -> {} (not in this zone)", u.name, u.alt_weapon_name),
        }
    }
}
