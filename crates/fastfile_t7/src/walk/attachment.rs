//! Weapon attachments: `WeaponAttachment` (the `attachment` asset: one per
//! attachment kind, "acog", "extbarrel", ...) and `WeaponAttachmentUnique`
//! (the `attachmentunique` asset: an attachment on one weapon, with its
//! models, animations and overrides). Read on retail `core_common`, where
//! the first weapons load them where the weapon's lists point.
//!
//! ```text
//! WeaponAttachment 0x220: 0x00 char* name ("acog"); 0x08 char* displayName ("WEAPON_ACOG");
//!         0x1d8 char* (a UI3D reticle name, "MagnifierReticle_UI3D"); the rest numbers
//!         (type, flags, multipliers)
//! ```

use super::capture::FieldValue;
use super::weapon::ANIMS;
use crate::asset_type::AssetType;
use crate::cursor::{ReadError, Result};
use crate::stream::Stream;

pub fn attachment(s: &mut Stream, h: usize) -> Result<Option<String>> {
    s.no_loads(h, AssetType::Attachment.header_size(), &[0, 8, 0x1d8])?;
    let name = s.string(h)?;
    s.string(h + 8)?;
    s.string(h + 0x1d8)?;
    Ok(name)
}

/// `WeaponAttachmentUnique` pointer fields read so far, in stream order.
#[derive(Clone, Copy)]
enum U {
    Str,
    /// An asset of this type.
    A(AssetType),
    /// One struct of this size without pointers.
    Struct(usize),
    /// `n` f32s (0x3e0: location damage multipliers, 21).
    Floats(usize),
    /// 197 animation names.
    Anims,
    /// One struct of this size whose last 8 bytes are a pointer to an
    /// asset of this type, the rest numbers (0x248, 0x28: vec3 origin,
    /// angles, a word, then the model; `au_baseweapon_damage`'s
    /// `wpn_t7_base_dm_mag_view`, loaded there).
    Placed(usize, AssetType),
}

const MODEL: U = U::A(AssetType::XModel);
const IMAGE: U = U::A(AssetType::Image);
const ACV: U = U::A(AssetType::AttachmentCosmeticVariant);

/// Pointer fields: (offset, kind, the name the capture keeps). Strings at
/// every field `au_smg_standard_none` points at the zone's shared empty
/// string. 0x78..0x138: four attached models `{ XModel* model, model2;
/// char* tag; vec3 origin, angles }` (48).
const UNIQUE_FIELDS: &[(usize, U, &str)] = &[
    (0x0, U::Str, "name"),
    (0x8, U::Str, "+0x8"),
    (0x10, U::Str, "+0x10"),
    (0x18, IMAGE, "hudIcon"),
    (0x20, IMAGE, "icon"),
    (0x40, U::Str, "displayName"),
    (0x50, U::Str, "+0x50"),
    (0x60, U::Struct(0x80), "+0x60"),
    (0x70, ACV, "acv"),
    (0x78, MODEL, "model0"),
    (0x80, MODEL, "model0b"),
    (0x88, U::Str, "tag0"),
    (0xa8, MODEL, "model1"),
    (0xb0, MODEL, "model1b"),
    (0xb8, U::Str, "tag1"),
    (0xd8, MODEL, "model2"),
    (0xe0, MODEL, "model2b"),
    (0xe8, U::Str, "tag2"),
    (0x108, MODEL, "model3"),
    (0x110, MODEL, "model3b"),
    (0x118, U::Str, "tag3"),
    (0x138, U::Str, "+0x138"),
    (0x140, U::Str, "+0x140"),
    (0x148, U::Str, "+0x148"),
    (0x150, IMAGE, "+0x150 image"),
    (0x160, ACV, "acv2"),
    (0x178, U::Str, "+0x178"),
    (0x1a8, U::Str, "+0x1a8"),
    (0x1d8, U::Str, "+0x1d8"),
    (0x208, U::Str, "+0x208"),
    (0x228, U::Str, "+0x228"),
    (0x230, U::Str, "+0x230"),
    (0x238, U::Str, "+0x238"),
    (0x248, U::Placed(0x28, AssetType::XModel), "model4"),
    (0x358, U::A(AssetType::WeaponCamo), "camo"),
    (0x3b8, U::Anims, "anims"),
    (0x3e0, U::Floats(21), "locationDamageMultipliers"),
    (0x420, U::Str, "+0x420"),
    (0x428, U::Str, "+0x428"),
    (0x430, U::Str, "+0x430"),
    (0x438, U::Str, "+0x438"),
    (0x440, U::Str, "+0x440"),
    (0x448, U::Str, "+0x448"),
    (0x450, U::Str, "+0x450"),
    (0x458, U::Str, "+0x458"),
    (0x460, U::Str, "+0x460"),
    (0x468, U::Str, "+0x468"),
    (0x470, U::Str, "+0x470"),
    (0x478, U::Str, "+0x478"),
    (0x480, U::Str, "+0x480"),
    (0x488, U::Str, "+0x488"),
    (0x490, U::Str, "+0x490"),
    (0x498, U::Str, "+0x498"),
    (0x4a0, U::Str, "+0x4a0"),
    (0x4a8, U::Str, "+0x4a8"),
    (0x4b0, U::Str, "+0x4b0"),
    (0x4b8, U::Str, "+0x4b8"),
    (0x500, U::Str, "+0x500"),
    (0x508, U::Str, "+0x508"),
    (0x528, U::A(AssetType::Fx), "viewFlashEffect"),
    (0x530, U::A(AssetType::Fx), "worldFlashEffect"),
    (0x560, U::A(AssetType::Laser), "viewLaser"),
    (0x568, U::A(AssetType::Laser), "worldLaser"),
];

/// 8-byte words that hold -1 ints (pairs of -1, or -1 next to -1000), not
/// pointers: `au_smg_standard_none` loads nothing for them (its name and
/// animation table fill the stream up to the next asset).
const UNIQUE_DATA_WORDS: &[usize] = &[0x5c0, 0x5d8, 0x5f0, 0x600, 0x608, 0x6b8, 0x728, 0x750, 0x780, 0x790];

/// `WeaponAttachmentUnique` (0x840, read on retail `core_common`): the
/// fields of `UNIQUE_FIELDS` in offset order; what they name is kept in
/// `Capture::links` (animations as `anim[slot]`).
pub fn attachment_unique(s: &mut Stream, h: usize) -> Result<Option<String>> {
    let mut known: Vec<usize> = UNIQUE_FIELDS.iter().map(|f| f.0).collect();
    known.extend(UNIQUE_DATA_WORDS);
    // The first field that loads data and is not known: the walk reads
    // the known fields before it, then stops there (`T7_AUPROBE=1` prints
    // where the stream is, the start of that field's data).
    let size = AssetType::AttachmentUnique.header_size();
    let unknown = (0..size / 8 * 8)
        .step_by(8)
        .find(|o| !known.contains(o) && s.ptr_at(h + o).is_ok_and(|p| p.is_inline()));
    let mut name = None;
    let mut links = Vec::new();
    let stop = |s: &Stream, u: usize, name: &Option<String>| {
        if std::env::var_os("T7_AUPROBE").is_some() {
            eprintln!("AUPROBE field {u:#x} of {name:?} data at {:#x}", s.pos());
        }
        Err(ReadError::Invalid("pointer field the walker does not know", h + u))
    };
    for &(off, kind, label) in UNIQUE_FIELDS {
        if let Some(u) = unknown.filter(|&u| u < off) {
            return stop(s, u, &name);
        }
        match kind {
            U::Str if off == 0 => name = s.string(h)?,
            U::Str => {
                if let Some(v) = s.string_named(h + off)?.filter(|v| !v.is_empty()) {
                    links.push((label.to_string(), FieldValue::Str(v)));
                }
            }
            U::A(ty) => {
                if let Some(n) = s.asset_named(ty, h + off)? {
                    links.push((label.to_string(), FieldValue::Asset { ty, name: n }));
                }
            }
            U::Struct(size) => {
                // u32s only: 4-aligned (later uniques' references to it land so).
                if let Some(t) = s.array(h + off, size, 3)? {
                    s.no_loads(t, size, &[])?;
                }
            }
            U::Placed(size, ty) => {
                if let Some(t) = s.array(h + off, size, 7)? {
                    s.no_loads(t, size - 8, &[])?;
                    if let Some(n) = s.asset_named(ty, t + size - 8)? {
                        links.push((label.to_string(), FieldValue::Asset { ty, name: n }));
                    }
                }
            }
            U::Floats(n) => {
                s.array(h + off, 4 * n, 3)?;
            }
            U::Anims => {
                if let Some(t) = s.records(h + off, ANIMS, 8, 7)? {
                    for i in 0..ANIMS {
                        if let Some(a) = s.string_named(t + 8 * i)?.filter(|a| !a.is_empty()) {
                            links.push((format!("anim[{i}]"), FieldValue::Str(a)));
                        }
                    }
                }
            }
        }
    }
    if let Some(u) = unknown.filter(|&u| u > UNIQUE_FIELDS[UNIQUE_FIELDS.len() - 1].0) {
        return stop(s, u, &name);
    }
    if let Some(n) = &name {
        s.capture.add_links(AssetType::AttachmentUnique, n, links);
    }
    Ok(name)
}
