//! Weapons: `WeaponVariantDef` (the `weapon` asset, 0x300) and the
//! `WeaponDef` (0x15f0) it points at. Field offsets and names from HydraX
//! (`weapon_fields`); the stream order read on retail `mp_common`.
//!
//! ```text
//! WeaponVariantDef 0x300: 0x00 char* name ("ar_standard_mp"); 0x08 char* (the WeaponDef's name,
//!         "ar_standard"); 0x18 WeaponDef*; 0x20 char* displayName; 0x28 altWeapon; 0x30 attachmentUnique;
//!         0x38 WeaponAttachment* attachments[64]*; 0x40 WeaponAttachmentUnique* uniques[128]*;
//!         0x48 char* anims[197]*; 0x50 u32 hideTags[32]* (script strings);
//!         0x58 XModel* attachViewModels[5]*; 0x60 XModel* attachWorldModels[5]*;
//!         0x68 char* attachViewModelTags[5]*; 0x70 char* attachWorldModelTags[5]*;
//!         0x1d0 ammoDisplayName, 0x1d8 ammoName, 0x1e8 clipName (char*);
//!         0x288, 0x290, 0x298 GfxImage* (ads overlays, dpad icon)
//! WeaponDef 0x15f0: strings (sound aliases, names) and asset pointers (`DEF_FIELDS`), and
//!         0x60 AttachmentCosmeticVariant* acvs[44]*; 0x1198, 0x11a0 f32[40]* (parallel and
//!         perpendicular bounce factors per surface); 0x13b0 f32[21]* (location damage multipliers);
//!         0x12d0, 0x12d8 accuracy graph names; 0x12e0, 0x12e8 knots, 0x12f0, 0x12f8 original knots
//!         (vec2; counts 0x1300 aiVsAi, 0x1304 aiVsPlayer, the originals use the same counts)
//! ```
//!
//! Stream order: the variant's name and def name, the `WeaponDef` struct
//! and its fields in offset order (each asset loaded where its pointer is,
//! the arrays where theirs are; the accuracy graphs as Black Ops II orders
//! them: each graph's name, knots and original knots), then the variant's other fields in offset
//! order. Arrays shared by several weapons (all-null model lists, the same
//! animation table) are references to the first copy.

use super::AssetFn;
use super::capture::{FieldValue, WeaponRec};
use super::weapon_fields::{ANIM_NAMES, DEF_FIELDS, VARIANT_FIELDS};
use crate::asset_type::AssetType;
use crate::cursor::Result;
use crate::ptr::Ptr;
use crate::stream::Stream;

pub(super) fn walker(ty: AssetType) -> Option<AssetFn> {
    match ty {
        AssetType::Weapon => Some(weapon),
        AssetType::Attachment => Some(super::attachment::attachment),
        AssetType::AttachmentUnique => Some(super::attachment::attachment_unique),
        AssetType::FlameTable => Some(super::flame::flame_table),
        _ => None,
    }
}

/// What a weapon field points at.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Kind {
    Str,
    Fx,
    TagFx,
    Model,
    Image,
    Tracer,
    Rumble,
    Bundle,
    SharedSounds,
    ImpactSounds,
    ImpactFx,
    FlameTable,
    Laser,
    Beam,
    XCam,
    Objective,
    Camo,
    Attachment,
    AttachmentUnique,
}

impl Kind {
    pub const fn asset_type(self) -> Option<AssetType> {
        use AssetType as T;
        Some(match self {
            Kind::Str => return None,
            Kind::Fx => T::Fx,
            Kind::TagFx => T::TagFx,
            Kind::Model => T::XModel,
            Kind::Image => T::Image,
            Kind::Tracer => T::Tracer,
            Kind::Rumble => T::Rumble,
            Kind::Bundle => T::ScriptBundle,
            Kind::SharedSounds => T::SharedWeaponSounds,
            Kind::ImpactSounds => T::ImpactSound,
            Kind::ImpactFx => T::ImpactFx,
            Kind::FlameTable => T::FlameTable,
            Kind::Laser => T::Laser,
            Kind::Beam => T::Beam,
            Kind::XCam => T::XCam,
            Kind::Objective => T::Objective,
            Kind::Camo => T::WeaponCamo,
            Kind::Attachment => T::Attachment,
            Kind::AttachmentUnique => T::AttachmentUnique,
        })
    }
}

pub const VARIANT: usize = 0x300;
pub const DEF: usize = 0x15f0;
const ATTACHMENTS: usize = 64;
const UNIQUES: usize = 128;
pub const ANIMS: usize = 197;
const HIDE_TAGS: usize = 32;
const ATTACH_MODELS: usize = 5;
const ACVS: usize = 44;
const BOUNCE: usize = 40 * 4;
const LOCATION_DAMAGE: usize = 21 * 4;

/// One field of `kind` at `field`, kept in `rec` as `name`.
fn field(s: &mut Stream, rec: &mut WeaponRec, field: usize, kind: Kind, name: &str) -> Result<()> {
    if std::env::var_os("T7_WTRACE").is_some() && s.ptr_at(field)?.is_inline() {
        eprintln!("WFIELD {name} at {:#x}", s.pos());
    }
    let value = match kind.asset_type() {
        // An empty string (the zone's shared "") names nothing.
        None => s
            .string_named(field)?
            .filter(|v| !v.is_empty())
            .map(FieldValue::Str),
        Some(ty) => s.asset_named(ty, field)?.map(|name| FieldValue::Asset { ty, name }),
    };
    if let Some(v) = value {
        rec.fields.push((name.to_string(), v));
    }
    Ok(())
}

/// `count` pointers of `kind` at the array the pointer at `at` loads.
fn list(s: &mut Stream, rec: &mut WeaponRec, at: usize, count: usize, kind: Kind, name: &str) -> Result<()> {
    if std::env::var_os("T7_WTRACE").is_some() {
        eprintln!("WLIST {name} {:?} at {:#x}", s.ptr_at(at)?, s.pos());
    }
    let shared = s.ptr_at(at)?.virtual_offset().and_then(|v| s.vmap.file_pos(v));
    if let Some(t) = s.records(at, count, 8, 7)? {
        for i in 0..count {
            field(s, rec, t + 8 * i, kind, &format!("{name}[{i}]"))?;
        }
    } else if let (Some(t), None) = (shared, kind.asset_type()) {
        // bo2mp: a shared list of strings (`attachViewModelTag`): the
        // strings its entries named there.
        for i in 0..count {
            let slot = t + 8 * i;
            let text = match s.ptr_at(slot)? {
                Ptr::Null => continue,
                p @ Ptr::Ref { .. } => p.virtual_offset().and_then(|v| s.string_at(v)),
                // (Loaded by that variant: its own record has the text.)
                _ => continue,
            };
            if let Some(text) = text.map(crate::cursor::text).filter(|t| !t.is_empty()) {
                rec.fields.push((format!("{name}[{i}]"), FieldValue::Str(text)));
            }
        }
    } else if let (Some(t), Some(ty)) = (shared, kind.asset_type()) {
        // The list is another variant's (`ar_standard_mp`'s attachments are
        // `ar_standard_cp`'s array): what its entries named there.
        for i in 0..count {
            let slot = t + 8 * i;
            let virt = match s.ptr_at(slot)? {
                Ptr::Null => continue,
                p @ Ptr::Ref { .. } => p.virtual_offset(),
                _ => s.vmap.virt_of(slot),
            };
            let Some(v) = virt else { continue };
            let n = s.named_at(v).map_or_else(|| format!("<ref {v:#x}>"), |(_, n)| n.clone());
            rec.fields.push((format!("{name}[{i}]"), FieldValue::Asset { ty, name: n }));
        }
    }
    Ok(())
}

/// The accuracy graph knot arrays (`vec2`): (field, count field). Black
/// Ops II's order, kept here: each graph's name, knots, original knots.
const KNOTS: [(usize, usize); 4] = [(0x12e0, 0x1300), (0x12e8, 0x1304), (0x12f0, 0x1300), (0x12f8, 0x1304)];

/// Where a `WeaponDef` field loads: its offset, but for the accuracy graphs
/// (`aiVsAi` name, knots, original knots, then the `aiVsPlayer` three).
fn def_order(off: usize) -> usize {
    match off {
        0x12e0 => 0x12d1,
        0x12f0 => 0x12d2,
        0x12e8 => 0x12d9,
        0x12f8 => 0x12da,
        o => o,
    }
}

/// The `WeaponDef` at `d`.
fn weapon_def(s: &mut Stream, rec: &mut WeaponRec, d: usize) -> Result<()> {
    let mut specials = vec![0x60, 0x1198, 0x11a0, 0x13b0];
    specials.extend(KNOTS.iter().map(|k| k.0));
    let mut known: Vec<usize> = DEF_FIELDS.iter().map(|f| f.0).collect();
    known.extend(&specials);
    s.no_loads(d, DEF, &known)?;
    let mut order: Vec<(usize, Option<(Kind, &str)>)> = DEF_FIELDS.iter().map(|&(o, k, n)| (o, Some((k, n)))).collect();
    order.extend(specials.iter().map(|&o| (o, None)));
    order.sort_by_key(|f| def_order(f.0));
    for (off, what) in order {
        match (off, what) {
            (_, Some((kind, name))) => field(s, rec, d + off, kind, name)?,
            (0x60, None) => {
                if let Some(t) = s.records(d + 0x60, ACVS, 8, 7)? {
                    for i in 0..ACVS {
                        let ty = AssetType::AttachmentCosmeticVariant;
                        if let Some(name) = s.asset_named(ty, t + 8 * i)? {
                            rec.fields.push((format!("acv[{i}]"), FieldValue::Asset { ty, name }));
                        }
                    }
                }
            }
            (0x1198 | 0x11a0, None) => {
                s.array(d + off, BOUNCE, 3)?;
            }
            (0x13b0, None) => {
                rec.location_damage_at = s.records_at(d + off, LOCATION_DAMAGE, 1, 3)?;
            }
            (_, None) => {
                let count_at = KNOTS.iter().find(|k| k.0 == off).map_or(0, |k| k.1);
                let n = s.i32_at(d + count_at)?.max(0) as usize;
                s.records(d + off, n, 8, 3)?;
            }
        }
    }
    Ok(())
}

pub fn weapon(s: &mut Stream, h: usize) -> Result<Option<String>> {
    let mut known: Vec<usize> = vec![0, 8, 0x18, 0x38, 0x40, 0x48, 0x50, 0x58, 0x60, 0x68, 0x70];
    known.extend(VARIANT_FIELDS.iter().map(|f| f.0));
    s.no_loads(h, VARIANT, &known)?;
    let mut rec = WeaponRec { header_at: h, ..Default::default() };
    let name = s.string(h)?;
    rec.name = name.clone().unwrap_or_else(String::new);
    rec.base_name = s.string_named(h + 8)?.unwrap_or_else(String::new);
    rec.def_at = s.array(h + 0x18, DEF, 7)?;
    if let Some(d) = rec.def_at {
        weapon_def(s, &mut rec, d)?;
    } else if let Ptr::Ref { offset, .. } = s.ptr_at(h + 0x18)? {
        // A `WeaponDef` another variant loaded: name that variant.
        let owner = s
            .vmap
            .file_pos(offset)
            .and_then(|at| s.capture.weapons.iter().find(|w| w.def_at == Some(at)))
            .map_or_else(|| format!("<shared {offset:#x}>"), |w| w.name.clone());
        rec.def_shared_with = Some(owner.clone());
        rec.fields.push(("weaponDef".into(), FieldValue::Str(format!("shared with {owner}"))));
    }
    for &(off, kind, fname) in VARIANT_FIELDS.iter().filter(|f| f.0 < 0x38) {
        field(s, &mut rec, h + off, kind, fname)?;
    }
    if let Some(FieldValue::Str(d)) = rec.field("displayName") {
        rec.display_name = d.clone();
    }
    list(s, &mut rec, h + 0x38, ATTACHMENTS, Kind::Attachment, "attachments")?;
    list(s, &mut rec, h + 0x40, UNIQUES, Kind::AttachmentUnique, "attachmentUniques")?;
    if std::env::var_os("T7_WTRACE").is_some() {
        eprintln!("WLIST anims {:?} at {:#x}", s.ptr_at(h + 0x48)?, s.pos());
    }
    if let Some(t) = s.records(h + 0x48, ANIMS, 8, 7)? {
        for i in 0..ANIMS {
            if let Some(a) = s.string_named(t + 8 * i)?.filter(|a| !a.is_empty()) {
                let slot = ANIM_NAMES.iter().find(|n| n.0 == i).map_or_else(|| format!("anim{i}"), |n| n.1.to_string());
                rec.anims.push((i, slot, a));
            }
        }
    }
    if let Some(t) = s.records(h + 0x50, HIDE_TAGS, 4, 3)? {
        for i in 0..HIDE_TAGS {
            let tag = s.u32_at(t + 4 * i)?;
            if tag != 0 {
                rec.hide_tags.push(tag);
            }
        }
    }
    list(s, &mut rec, h + 0x58, ATTACH_MODELS, Kind::Model, "attachViewModel")?;
    list(s, &mut rec, h + 0x60, ATTACH_MODELS, Kind::Model, "attachWorldModel")?;
    list(s, &mut rec, h + 0x68, ATTACH_MODELS, Kind::Str, "attachViewModelTag")?;
    list(s, &mut rec, h + 0x70, ATTACH_MODELS, Kind::Str, "attachWorldModelTag")?;
    for &(off, kind, fname) in VARIANT_FIELDS.iter().filter(|f| f.0 >= 0x38) {
        field(s, &mut rec, h + off, kind, fname)?;
    }
    s.capture.weapons.push(rec);
    Ok(name)
}
