pub mod class_presets;
pub mod pad;
pub mod retire;
pub mod schedule;
pub mod script_entity_notify;
pub mod script_notify;
pub mod session;
pub mod settings;
pub mod ui;

pub use class_presets::{ClassPreset, pick_showcase, showcase_classes};
<<<<<<< /home/claude/bo2-multiplayer-rebuild/crates/frame/src/lib.rs
pub use pad::{ActivePad, InputDevices, PromptStyle, TestControllerRumble};
=======
pub use pad::{
    ActivePad, BO2_XBOX_GLYPHS, InputDevices, PromptStyle, TestControllerRumble, bo2_frontend,
    bo2_cycle_glyph, bo2_game, bo2_pad_glyph, bo2_zm_choice, bo2_zm_start, set_bo2_frontend, set_bo2_game,
    set_bo2_zm_start,
};
>>>>>>> /tmp/claude-0/zmhead/crates/frame/src/lib.rs
pub use retire::Retiring;
pub use schedule::{
    AUTHORITY_TOC, AuthorityBookkeeping, AuthorityEdge, AuthoritySet, CLIENT_TOC,
    ClassEquipResolved, ClientEdge, ClientSet, FxSoundPublished, LifeFrontPublished,
    ModelLightingSeated, PresentedPublished, RenderSet, SessionSwapApplied, WORKER_CMD_AFTER,
    WORKER_CMD_END_FENCE, WORKER_CMD_NOT_RENDER_THREAD, WORKER_CMD_RETAIL_NAMES, WORKER_CMD_TOC,
    WorkerCmdSet, authority_set_name, client_set_name, configure_authority_sets,
    configure_client_sets, configure_render_sets, configure_worker_cmd_sets, worker_cmd_name,
};
pub use script_entity_notify::{AbortKillcam, BeginKillcam, KillcamEnded, SpawnedPlayer};
<<<<<<< /home/claude/bo2-multiplayer-rebuild/crates/frame/src/lib.rs
pub use script_notify::{EndBoardShown, ExitLevelCalled, register_script_notify};
=======
pub use script_notify::{ExitLevelCalled, register_script_notify};
>>>>>>> /tmp/claude-0/zmhead/crates/frame/src/lib.rs
pub use session::{
    AdmissionKey, AppScreen, BotNavigationReady, CacWeaponOffer, ClassSelectHandoff, HasWorld,
    Headless, HostClassLoadouts, HostClassSlot, HudInputView, LaunchIdentity, LaunchReport,
    LifeEndCause, LifeEnded, LifeStartReason, LifeStarted, LocalLoadKey, MapLoadApproved,
    MapLoadFailed, MatchInstalled, MatchKey, MatchTornDown, ReturnedToMenu, RuntimeRole,
    TeardownReason, UiCamera, UiDraw, ViewSubject, WorldGeneration, WorldProducts,
};
pub use settings::{DisplayResolution, GameSettings};
pub use ui::{
<<<<<<< /home/claude/bo2-multiplayer-rebuild/crates/frame/src/lib.rs
    GamePaused, HostMatchRules, LuiMenus, MenuDims, MenuMusicScreen, WorldBlur, WorldDim, UiBindRequest, UiBindingCapture, UiExecCommand,
    UiMenuDvars, UiMenuKey, UiMenuRequest, UiPartyState, UiPlayMusic, UiPlaySound, UiStopMusic,
    register_ui_contracts,
=======
    GamePaused, HostMatchRules, LuiMenus, UiBindRequest, UiBindingCapture, UiExecCommand,
    ScriptScreenFilter, UiMenuDvars, UiMenuKey, UiMenuRequest, UiPartyState, UiPlayMusic,
    UiPlaySound, UiStopMusic, WorldBlur, register_ui_contracts,
>>>>>>> /tmp/claude-0/zmhead/crates/frame/src/lib.rs
};
