//! What a walk keeps besides the asset list: the weapons' records.

use crate::asset_type::AssetType;

/// What a weapon field names.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum FieldValue {
    /// An asset of `ty`: its name when it was loaded here or by an earlier
    /// asset the reference names (`<ref 0x...>` when the walk cannot tell).
    Asset { ty: AssetType, name: String },
    /// A string (sound alias, weapon name, tag, hint, ...).
    Str(String),
}

/// One weapon (`WeaponVariantDef` and its `WeaponDef`).
#[derive(Clone, Debug, Default)]
pub struct WeaponRec {
    /// The variant's name (`ar_standard_mp`) and its `WeaponDef`'s name field
    /// (`ar_standard`).
    pub name: String,
    pub base_name: String,
    pub display_name: String,
    /// File offsets of the variant header and of the `WeaponDef` struct.
    pub header_at: usize,
    pub def_at: Option<usize>,
    /// Every non-null pointer field, in stream order: (field name, value).
    /// Unnamed fields read as `def+0x...`/`variant+0x...`; arrays as
    /// `field[i]`.
    pub fields: Vec<(String, FieldValue)>,
    /// The animation table (197 slots): (slot, slot name, animation name).
    pub anims: Vec<(usize, String, String)>,
    /// `hideTags`: script string indices (32 slots, zero ones dropped).
    pub hide_tags: Vec<u32>,
}

impl WeaponRec {
    /// The first field called `name`.
    pub fn field(&self, name: &str) -> Option<&FieldValue> {
        self.fields.iter().find(|(n, _)| n == name).map(|(_, v)| v)
    }
}

#[derive(Clone, Debug, Default)]
pub struct Capture {
    pub weapons: Vec<WeaponRec>,
}
