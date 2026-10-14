//! Vehicles: no x64-zt writer for IW7, so the order is inferred from
//! `VehicleDef` (x64-zt `iw7/structs.hpp`) and checked against the retail
//! data (`iw7_common_core_mp.ff`'s 12 vehicles). The pointers, in
//! declaration order: name and hint strings, the helicopter ground FX, the
//! spaceship anims (33 x 2 `XAnimParts`), thrust/juke FX, rumbles, the
//! sound aliases (plain strings in IW7), the turret weapon name and weapon,
//! trophy FX, compass icons, and the trailing strings. `VehiclePhysicsDef`
//! holds no pointers.

use super::weapon::fx_combined;
use super::{AssetId, Walker};
use crate::asset_type::AssetType;
use crate::cursor::Result;
use crate::layout as L;
use crate::stream::VIRTUAL;

fn dbg() -> bool {
    std::env::var_os("IW7_VEHDUMP").is_some()
}

pub(super) fn vehicle(w: &mut Walker, h: usize) -> Result<AssetId> {
    use AssetType as T;
    use L::VehicleDef as D;
    w.s.push(VIRTUAL);
    if dbg() {
        for o in (0..D::SIZE).step_by(8) {
            let v = w.s.u64_at(h + o)?;
            if v >> 32 == 0xFFFF_FFFF {
                eprintln!("  veh+{o:5} {v:#018x}");
            }
        }
    }
    let name = w.xstr(h + D::name)?.unwrap_or_default();
    if dbg() {
        eprintln!("vehicle {name} header {h:#x} pos {:#x}", w.s.pos());
    }
    w.xstr(h + D::useHintString)?;
    fx_combined(w, h + D::vehHelicopterGroundFx)?;
    fx_combined(w, h + D::vehHelicopterGroundWaterFx)?;
    for i in 0..33 * 2 {
        w.asset(T::XAnimParts, h + D::ssAnimParts + 8 * i)?;
    }
    fx_combined(w, h + D::ssThrustFxLoop)?;
    fx_combined(w, h + D::ssJukeFx)?;
    for f in [D::ssIdleRumble, D::ssSmallRumble, D::ssMedRumble, D::ssLargeRumble] {
        w.asset(T::Rumble, h + f)?;
    }
    for f in [
        D::rattleLoop,
        D::airLoop,
        D::engineLoop,
        D::hoverLoop,
        D::boostLoop,
        D::ssThrustLoop,
        D::boostStart,
        D::boostStop,
        D::boostDepleted,
        D::boostUnavailable,
        D::jukeLeft,
        D::jukeRight,
        D::jukeUpDown,
        D::jukeBack,
        D::jukeFront,
        D::flightOn,
        D::flightOff,
        D::hardCollision,
        D::softCollision,
    ] {
        w.xstr(h + f)?;
    }
    w.xstr(h + D::turretWeaponName)?;
    w.asset(T::Weapon, h + D::turretWeapon)?;
    w.xstr(h + D::turretSpinSnd)?;
    w.xstr(h + D::turretStopSnd)?;
    fx_combined(w, h + D::trophyExplodeFx)?;
    fx_combined(w, h + D::trophyFlashFx)?;
    for f in [D::compassFriendlyIcon, D::compassEnemyIcon, D::compassFriendlyAltIcon, D::compassEnemyAltIcon] {
        w.asset(T::Material, h + f)?;
    }
    for f in [
        D::idleLowSnd,
        D::idleHighSnd,
        D::engineLowSnd,
        D::engineHighSnd,
        D::engineStartUpSnd,
        D::engineShutdownSnd,
        D::engineIdleSnd,
        D::engineSustainSnd,
        D::engineRampUpSnd,
        D::engineRampDownSnd,
        D::suspensionSoftSnd,
        D::suspensionHardSnd,
        D::collisionSnd,
        D::speedSnd,
        D::surfaceSndName,
        D::soundTriggerOverrideZone,
        D::globalVisionSettings,
        D::mapVisionSettings,
    ] {
        w.xstr(h + f)?;
    }
    if dbg() {
        eprintln!("  vehicle end pos {:#x}", w.s.pos());
    }
    w.s.pop();
    Ok(w.name_only(T::Vehicle, name))
}
