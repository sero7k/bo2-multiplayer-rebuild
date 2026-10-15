mod bo2_font;
mod bo2_globe;
mod cheat_menu; // bo2zm: the solo cheat menu (Num 1)
pub mod classes;
mod frame_rate; // bo2zm: Draw FPS, Max FPS
pub mod frontend;
mod gap_hud;
mod launch_report;
mod layers;
mod load_table;
mod loading;
mod lui_hud; // bo2zm M4; bo2mp: the multiplayer front end and match HUD
mod lui_hud_zm; // bo2 merge: Zombies' own LUI front end and HUD (see lui_hud::zombies_turn)
mod menu;
mod menu_load;
mod options;
mod plugin;
mod screen;
mod scope_hint; // bo2mp scope lane
mod overhead_names; // bo2mp: names over heads in BO2's font
mod zm_hud; // bo2zm M4: the Zombies globe
mod foreign_maps; // bo2mp: other games' maps in the map list
mod game_folders; // bo2mp: the Game Folders panel
pub mod hud_modern; // bo2mp: the modern in-game HUD
pub mod hud_match; // bo2mp: our kill feed, hit marker, kill banner, scoreboard
pub mod hud_nuke; // bo2mp: the Tactical Nuke's KEM STRIKE INBOUND banner
pub mod hud_medals; // bo2mp: Black Ops' medal notification
pub(crate) mod hud_cards; // bo2mp: MW2/MW3 calling cards and emblems in the HUD
mod jup_cards; // bo2mp: Modern Warfare III's (2023) emblems and calling cards
mod picture_cache; // bo2mp: other games' menu pictures kept on disk
mod jup_hud; // bo2mp: Modern Warfare III's (2023) Warzone HUD pictures
pub(crate) mod hud_warzone; // bo2mp: the Warzone HUD (Resurgence)
pub(crate) mod hud_prematch; // bo2mp: the match's start countdown
pub mod hud_outcome; // bo2mp: the free-for-all end screen
pub(crate) mod online_match; // bo2mp: the front end's matches through the master
pub mod shell; // bo2mp: the client's own front end and loading screen
mod music_screen; // bo2mp: front-end music context for the audio director
mod load_info; // bo2mp: the loading screen's info bar
mod pad_glyphs; // bo2mp: Xbox button icons on a controller
mod lui_additive; // bo2mp scope: true additive HUD pictures
mod lui_frontend; // bo2mp lane C
mod lui_scene; // bo2mp menus: the front end's 3D backdrop
mod bo2mp_damage; // bo2mp lane C
pub use lui_frontend::{Bo2mpFrontend, Bo2mpFrontendAssets};
pub mod lui_omni_row; // bo2zm: Controls > Move "Omni Movement" (a tester 10-10)

pub use classes::equip_txn::{
    EquipTxnWatch, apply_pending_class_equip, resolve_class_equip_transaction,
    sync_class_change_allowed,
};
pub use classes::icons::{
    ClassSelectIconCache, UiAssetRoot, cac_attachment_image, cac_material_iwd_stem,
    cac_weapon_image, pretty_weapon_name,
};
pub use classes::select::{
    ClassChangeAllowed, ClassChangeBlockReason, ClassEquipRefusal, ClassEquipRequest,
    ClassSelectHighlight, ClassSelectOverlayOpen, ClassSelectPhase, ClassSelectStatus,
    PendingClassEquip, accept_class_equip, class_index_by_name, commit_class_equip,
    reject_class_equip,
};
pub use classes::setup::{ClassEditRow, ClassLoadoutCatalog, ClassPickerFolder, ClassSlotState};
pub use classes::store::SessionClassStore;
pub use frame::{AppScreen, LaunchIdentity, LaunchReport};
pub use frame::{ClassPreset, showcase_classes};
pub use gap_hud::GapHud;
pub use lui_additive::AdditiveUi;
pub use overhead_names::{OverheadNameRow, OverheadNameRows};
pub use launch_report::publish_gap_hud;
pub use layers::{
    ApplyUiLayers, GameUiFont, UiCamera, UiDraw, UiLayer, UiLayerVisibility, UiLayers,
    game_text_font,
};
pub use loading::{LoadProgress, LoadingPreviewSource, LoadingScreen};
pub use menu::MenuMapList;
pub use options::{BindingView, PresentModeOverride};
pub use plugin::UiPlugin;
pub use screen::{layers_for_screen, sync_ui_layers};

pub use menu::install_frontend_menus;
