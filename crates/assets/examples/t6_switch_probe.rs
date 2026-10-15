//! Prints the weapon timings Black Ops II's weapon switch, fire, rechamber
//! and reload rules read, for weapons whose name contains a pattern:
//! `t6_switch_probe <zone.ff> [name-pattern]`.
use fastfile_t6::layout::WeaponDef as d;
use fastfile_t6::layout::WeaponVariantDef as v;

fn main() {
    let a: Vec<String> = std::env::args().collect();
    let c = asset_t6::capture_zone(std::path::Path::new(&a[1])).unwrap();
    let pat = a.get(2).cloned().unwrap_or_default();
    for w in c.weapons.iter().filter(|w| w.name.contains(&pat)) {
        println!(
            "{:24} fireType {} fire {} fireDelay {} lastFire {} rechamber {} bolt {} boltAction {} shotsBeforeRechamber {} burstDelay {} \
             drop {} raise {} quickDrop {} quickRaise {} firstRaise {} emptyDrop {} emptyRaise {} altDrop {} altRaise {} \
             reload {} reloadEmpty {} reloadAdd {} reloadStart {} reloadStartAdd {} reloadEnd {} segmented {} clip {}",
            w.name,
            w.def_i32(d::fireType),
            w.def_i32(d::iFireTime),
            w.def_i32(d::iFireDelay),
            w.def_i32(d::iLastFireTime),
            w.def_i32(d::iRechamberTime),
            w.def_i32(d::iRechamberBoltTime),
            w.def_u8(d::bBoltAction),
            w.def_i32(d::iShotsBeforeRechamber),
            w.def_i32(d::iBurstDelayTime),
            w.def_i32(d::iDropTime),
            w.def_i32(d::iRaiseTime),
            w.def_i32(d::quickDropTime),
            w.def_i32(d::quickRaiseTime),
            w.def_i32(d::iFirstRaiseTime),
            w.def_i32(d::iEmptyDropTime),
            w.def_i32(d::iEmptyRaiseTime),
            w.def_i32(d::iAltDropTime),
            w.var_i32(v::iAltRaiseTime),
            w.var_i32(v::iReloadTime),
            w.var_i32(v::iReloadEmptyTime),
            w.def_i32(d::iReloadAddTime),
            w.def_i32(d::iReloadStartTime),
            w.def_i32(d::iReloadStartAddTime),
            w.def_i32(d::iReloadEndTime),
            w.def_u8(d::bSegmentedReload),
            w.var_i32(v::iClipSize),
        );
    }
}
