//! bo2zm: list every asset in Black Ops II zones by type and name.
//!
//! `t6list <zone.ff>...` prints one line per asset: zone, type, name (the
//! string most asset headers start with). Sound banks add their alias
//! count and bank file names. Nothing is written anywhere.

use std::path::PathBuf;
use std::process::ExitCode;

use asset_transport::{T6ZoneMemory, ZoneGame, open_zone};
use fastfile_t6::layout as l;
use fastfile_t6::{AssetType, Loaded, Ptr, WalkSink, Walker, ZonePtr, ZoneStream, open_asset_table};

fn deref(s: &ZoneStream<'_>, p: Ptr, off: usize) -> fastfile_t6::Result<Option<Ptr>> {
    Ok(match s.ptr_at(p, off)? {
        ZonePtr::Offset(q) => Some(s.resolve_alias(q)),
        _ => None,
    })
}

fn string_field(s: &ZoneStream<'_>, p: Ptr, off: usize) -> fastfile_t6::Result<String> {
    Ok(match deref(s, p, off)? {
        Some(q) => String::from_utf8_lossy(s.cstr_bytes(q)?).into_owned(),
        None => String::new(),
    })
}

fn fixed_string(s: &ZoneStream<'_>, p: Ptr, off: usize, len: usize) -> fastfile_t6::Result<String> {
    let raw = s.slice_at(p, off, len)?;
    let end = raw.iter().position(|&b| b == 0).unwrap_or(raw.len());
    Ok(String::from_utf8_lossy(&raw[..end]).into_owned())
}

struct Lister {
    zone: String,
    lines: Vec<String>,
}

/// Asset types whose header does not start with a name pointer.
fn named_at_zero(ty: AssetType) -> bool {
    !matches!(
        ty,
        AssetType::Localize
    )
}

impl WalkSink for Lister {
    fn asset_loaded(&mut self, s: &ZoneStream<'_>, loaded: Loaded) -> fastfile_t6::Result<()> {
        let body = loaded.body;
        let name = if named_at_zero(loaded.ty) {
            string_field(s, body, 0).unwrap_or_default()
        } else if loaded.ty == AssetType::Localize {
            // A localized string: { value, name } - its key, then its text.
            let key = string_field(s, body, 4).unwrap_or_default();
            format!("{key} = {}", string_field(s, body, 0).unwrap_or_default())
        } else {
            String::new()
        };
        let mut extra = String::new();
        match loaded.ty {
            AssetType::Sound => {
                let aliases = s.u32_at(body, l::SndBank::aliasCount)?;
                let stream_file = fixed_string(
                    s,
                    body.at(l::SndBank::streamAssetBank),
                    l::SndRuntimeAssetBank::filename,
                    256,
                )?;
                let load_file = fixed_string(
                    s,
                    body.at(l::SndBank::loadAssetBank),
                    l::SndRuntimeAssetBank::filename,
                    256,
                )?;
                let loaded_count =
                    s.u32_at(body.at(l::SndBank::loadedAssets), l::SndLoadedAssets::loadedCount)?;
                let entry_count =
                    s.u32_at(body.at(l::SndBank::loadedAssets), l::SndLoadedAssets::entryCount)?;
                let data_size =
                    s.u32_at(body.at(l::SndBank::loadedAssets), l::SndLoadedAssets::dataSize)?;
                extra = format!(
                    " aliases={aliases} stream_bank='{stream_file}' load_bank='{load_file}' loaded={loaded_count}/{entry_count} data={data_size}"
                );
            }
            AssetType::Weapon => {
                let display = string_field(s, body, l::WeaponVariantDef::szDisplayName)?;
                let clip = s.i32_at(body, l::WeaponVariantDef::iClipSize)?;
                let def = deref(s, body, l::WeaponVariantDef::weapDef)?;
                let (wname, ty, class, inv) = match def {
                    Some(d) => (
                        string_field(s, d, l::WeaponDef::szOverlayName).unwrap_or_default(),
                        s.u32_at(d, l::WeaponDef::weapType)?,
                        s.u32_at(d, l::WeaponDef::weapClass)?,
                        s.u32_at(d, l::WeaponDef::inventoryType)?,
                    ),
                    None => (String::new(), 0, 0, 0),
                };
                extra = format!(
                    " display='{display}' clip={clip} type={ty} class={class} inv={inv} overlay='{wname}'"
                );
                // T6LIST_WEAPON=name,name: also its firing and turret numbers.
                let want = std::env::var("T6LIST_WEAPON").unwrap_or_default();
                if let Some(d) = def.filter(|_| want.split(',').any(|w| w == name)) {
                    let f = |off| s.f32_at(d, off);
                    let i = |off| s.i32_at(d, off);
                    let fx = |off| -> fastfile_t6::Result<String> {
                        Ok(match deref(s, d, off)? {
                            Some(p) => string_field(s, p, 0)?,
                            None => String::new(),
                        })
                    };
                    let w = l::WeaponDef::damage;
                    let r = l::WeaponDef::damageRange;
                    extra += &format!(
                        " fire_ms={} damage=[{} {} {} {} {} {}] ranges=[{} {} {} {} {} {}] arcs(l r t b)=[{} {} {} {}] \
                         max_range={} turn_speed=[{} {}]/[{} {}] converge(p y)=[{} {}] accuracy={} ai_spread={} \
                         player_spread={} suppress={} flash='{}' tracer='{}' fire_sound='{}'",
                        i(l::WeaponDef::iFireTime)?,
                        i(w)?, i(w + 4)?, i(w + 8)?, i(w + 12)?, i(w + 16)?, i(w + 20)?,
                        f(r)?, f(r + 4)?, f(r + 8)?, f(r + 12)?, f(r + 16)?, f(r + 20)?,
                        f(l::WeaponDef::leftArc)?, f(l::WeaponDef::rightArc)?,
                        f(l::WeaponDef::topArc)?, f(l::WeaponDef::bottomArc)?,
                        f(l::WeaponDef::maxRange)?,
                        f(l::WeaponDef::minTurnSpeed)?, f(l::WeaponDef::minTurnSpeed + 4)?,
                        f(l::WeaponDef::maxTurnSpeed)?, f(l::WeaponDef::maxTurnSpeed + 4)?,
                        f(l::WeaponDef::pitchConvergenceTime)?, f(l::WeaponDef::yawConvergenceTime)?,
                        f(l::WeaponDef::accuracy)?, f(l::WeaponDef::aiSpread)?,
                        f(l::WeaponDef::playerSpread)?, f(l::WeaponDef::suppressTime)?,
                        fx(l::WeaponDef::worldFlashEffect)?, fx(l::WeaponDef::tracerType)?,
                        string_field(s, d, l::WeaponDef::fireSound)?,
                    );
                }
            }
            // bo2mp lane D: a vehicle's guns (its turret and gunner
            // weapons, by name). Speeds as loaded: units a second (the
            // vehicle files' miles per hour times 17.6; Buried's ghost
            // lantern is 88 = 5 mph).
            AssetType::VehicleDef => {
                let turret = string_field(s, body, l::VehicleDef::turretWeapon).unwrap_or_default();
                let gunners: Vec<String> = (0..4)
                    .map(|i| string_field(s, body, l::VehicleDef::gunnerWeapon + i * 4).unwrap_or_default())
                    .filter(|g| !g.is_empty())
                    .collect();
                let f = |off| s.f32_at(body, off);
                extra = format!(
                    " turret='{turret}' gunners='{}' type={} maxSpeed={} maxSpeedVertical={} accel={} accelVertical={} rotAccel={} decelFwd={} decelSide={} decelUp={} quadRotor={} radius={}",
                    gunners.join(","),
                    s.i16_at(body, l::VehicleDef::r#type)?,
                    f(l::VehicleDef::maxSpeed)?,
                    f(l::VehicleDef::maxSpeedVertical)?,
                    f(l::VehicleDef::accel)?,
                    f(l::VehicleDef::accelVertical)?,
                    f(l::VehicleDef::rotAccel)?,
                    f(l::VehicleDef::vehHelicopterDecelerationFwd)?,
                    f(l::VehicleDef::vehHelicopterDecelerationSide)?,
                    f(l::VehicleDef::vehHelicopterDecelerationUp)?,
                    s.i32_at(body, l::VehicleDef::vehHelicopterQuadRotor)?,
                    f(l::VehicleDef::radius)?,
                );
            }
            _ => {}
        }
        self.lines
            .push(format!("{}\t{}\t{}{}", self.zone, loaded.ty.name(), name, extra));
        Ok(())
    }
}

fn main() -> ExitCode {
    let files: Vec<PathBuf> = std::env::args().skip(1).map(PathBuf::from).collect();
    if files.is_empty() {
        eprintln!("usage: t6list <zone.ff>...");
        return ExitCode::FAILURE;
    }
    let mut failed = false;
    for path in files {
        let zone = path
            .file_stem()
            .map_or_else(String::new, |s| s.to_string_lossy().into_owned());
        let run = || -> Result<Vec<String>, String> {
            let image = open_zone(&path).map_err(|e| e.to_string())?;
            if image.game != ZoneGame::T6 {
                return Err("not a Black Ops II zone".into());
            }
            let header = image.t6_header().map_err(|e| e.to_string())?;
            let mut memory = T6ZoneMemory::for_header(&header);
            let mut stream = memory.stream(&image.bytes).map_err(|e| e.to_string())?;
            let table = open_asset_table(&mut stream).map_err(|e| e.to_string())?;
            let mut lister = Lister {
                zone: zone.clone(),
                lines: Vec::new(),
            };
            Walker::new(&mut stream, &mut lister)
                .walk_assets(&table)
                .map_err(|e| e.to_string())?;
            Ok(lister.lines)
        };
        match run() {
            Ok(lines) => {
                for line in lines {
                    println!("{line}");
                }
            }
            Err(error) => {
                eprintln!("{zone}: {error}");
                failed = true;
            }
        }
    }
    if failed {
        ExitCode::FAILURE
    } else {
        ExitCode::SUCCESS
    }
}
