//! bo2mp: Modern Warfare 2's and 3's weapon camos. Their game draws a camo
//! as another model of the gun (WeaponDef gunXModel / worldModel slots
//! 1-15: `viewmodel_m4_woodland`), the same shape with the camo's
//! materials. The weapon capture notes each gun's camo models here; the
//! match load pairs them with the gun's own models into material swaps
//! (`asset_core::weapon_camos`).

use std::collections::HashMap;
use std::sync::Mutex;

use asset_core::AssetNamespace;

/// A gun's camos: (slot, view model, world model).
pub type CamoModels = Vec<(u8, Option<String>, Option<String>)>;

/// (game, gun's view model) -> (its world model, its camos).
static NOTED: Mutex<Option<HashMap<(AssetNamespace, String), (Option<String>, CamoModels)>>> = Mutex::new(None);

/// A gun's camo models as its weapon capture read them.
pub fn note(ns: AssetNamespace, gun: &str, world: Option<String>, camos: CamoModels) {
    if camos.is_empty() {
        return;
    }
    if let Ok(mut n) = NOTED.lock() {
        n.get_or_insert_with(HashMap::new).insert((ns, gun.to_owned()), (world, camos));
    }
}

/// Every gun noted: (game, view model, world model, camos).
#[must_use]
pub fn all() -> Vec<(AssetNamespace, String, Option<String>, CamoModels)> {
    NOTED
        .lock()
        .map(|n| n.iter().flatten().map(|((ns, gun), (world, camos))| (*ns, gun.clone(), world.clone(), camos.clone())).collect())
        .unwrap_or_default()
}

/// Modern Warfare 2's camos by slot (its WeaponDef model slots 1-8), and
/// the menu picture of each.
pub const MW2_CAMOS: [(u8, &str, &str); 8] = [
    (1, "Woodland", "weapon_camo_menu_woodland"),
    (2, "Desert", "weapon_camo_menu_desert"),
    (3, "Arctic", "weapon_camo_menu_arctic"),
    (4, "Digital", "weapon_camo_menu_digital"),
    (5, "Urban", "weapon_camo_menu_red_urban"),
    (6, "Red Tiger", "weapon_camo_menu_red_tiger"),
    (7, "Blue Tiger", "weapon_camo_menu_blue_tiger"),
    (8, "Fall", "weapon_camo_menu_orange_fall"),
];

/// Modern Warfare 3's camos by slot (its WeaponDef model slots 1-13).
pub const MW3_CAMOS: [(u8, &str, &str); 13] = [
    (1, "Classic", "weapon_camo_menu_classic"),
    (2, "Snow", "weapon_camo_menu_snow"),
    (3, "Multicam", "weapon_camo_menu_multi"),
    (4, "Digital Urban", "weapon_camo_menu_d_urban"),
    (5, "Hex", "weapon_camo_menu_hex"),
    (6, "Choco", "weapon_camo_menu_choco"),
    (7, "Snake", "weapon_camo_menu_snake"),
    (8, "Blue", "weapon_camo_menu_blue"),
    (9, "Red", "weapon_camo_menu_red"),
    (10, "Autumn", "weapon_camo_menu_autumn"),
    (11, "Gold", "weapon_camo_menu_gold"),
    (12, "Marine", "weapon_camo_menu_marine"),
    (13, "Winter", "weapon_camo_menu_winter"),
];
