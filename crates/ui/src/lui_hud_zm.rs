//! bo2 merge: the Zombies game's LUI driver (its front end, lobbies and
//! HUD), beside the multiplayer one in `lui_hud`; see
//! `lui_hud::zombies_turn` for which runs.
//!
//! bo2zm M4: Black Ops II's own Zombies HUD, run from the game's UI
//! scripts: the HavokScript VM (`hks_t6`) loads LUI and CoD's base, then
//! `ui_mp/t6/hud.lua`; the root opens its `HUD` menu as the engine does,
//! and the game's values reach it as the engine's events
//! (`hud_update_rounds_played`, ...). Each frame its elements (pictures and
//! text, in root units 720 high) are drawn as Bevy UI over the game.
//!
//! On unless `BO2ZM_LUI=0`; the hand-made HUD in `zm_hud` keeps what the
//! engine itself draws in BO2 (the use hint, script HUD text, the hurt
//! overlay).

use std::cell::RefCell;
use std::collections::HashMap;
use std::rc::Rc;

use bevy::prelude::*;
use bevy::window::PrimaryWindow;
use frame::ClientSet;
use hks_t6::host::{Drawn, Host};
use hks_t6::value::Value;
use net::{LocalPresentClient, PresentedSnapshot};

use crate::bo2_font::Bo2Fonts;
use crate::layers::{UiLayer, UiLayerVisibility};

/// Font metrics the scripts' text measure reads: font name -> (pixel
/// height, advance per letter, space advance).
type Metrics = Rc<RefCell<HashMap<String, (f32, HashMap<char, f32>, f32)>>>;

/// The scripts' font files and the names `Bo2Fonts` keeps them by
/// (`ui/t6/codbase.lua`'s `fonts/<res>/<file>`).
const FONT_FILES: [(&str, &str); 7] = [
    ("normalFont", "Default"),
    ("smallFont", "Condensed"),
    ("bigFont", "Big"),
    ("extraBigFont", "Morris"),
    ("extraSmallFont", "ExtraSmall"),
    ("italicFont", "Italic"),
    ("smallItalicFont", "SmallItalic"),
];

/// Black Ops II's build number as its menus show it, and where (root
/// units, right-aligned).
const VERSION_TEXT: &str = "43.1701.1";
const VERSION_RECT: [f32; 4] = [1000.0, 70.0, 1125.0, 84.0];

fn font_name(file: &str) -> &'static str {
    let stem = file.rsplit('/').next().unwrap_or(file);
    FONT_FILES
        .iter()
        .find(|(f, _)| f.eq_ignore_ascii_case(stem))
        .map_or("Default", |(_, n)| n)
}

/// On unless `BO2ZM_LUI=0` (then the hand-made HUD in `zm_hud` draws it all).
pub(crate) fn lui_enabled() -> bool {
    std::env::var("BO2ZM_LUI").map_or(true, |v| v != "0")
}

struct LuiHud {
    host: Host,
    metrics: Metrics,
    /// bo2zm M4: Black Ops II's front end (main menu, lobby), not the HUD.
    frontend: bool,
    /// When the front end last had its lobby update (LUI ms).
    lobby_update_at: f64,
    /// What the last lobby update was sent for (open menus, party), and
    /// whether one more goes a second later (a menu opening late).
    lobby_sig: String,
    lobby_again: bool,
    /// Its loading screen is up (a map loads).
    loading_open: bool,
    /// The lobby it opens on (back from a game), else the main menu.
    return_menu: Option<&'static str>,
    /// The left stick's menu step (its direction and when it last stepped).
    stick: Option<(&'static str, f64)>,
    /// BO2's scoreboard is open (Tab held, or the game-over shot).
    board_open: bool,
    /// The scripts it was built from (their count).
    built_from: usize,
    opened: bool,
    /// The mouse in root units, as last sent.
    mouse_at: Option<Vec2>,
    /// A bind row asked for a key and the capture has not finished.
    binding: bool,
    /// His settings' revision last written into BO2's names, and what.
    settings_rev: Option<u64>,
    written: Vec<(String, String)>,
    /// BO2ZM_LUI_DUMP: how many elements the last dump listed.
    dumped: usize,
    /// BO2ZM_LUI_PAUSE_AT's progress, and when the HUD opened.
    test_step: usize,
    /// BO2ZM_LUI_ASKED's one report went out.
    asked_logged: bool,
    /// BO2ZM_LUI_OPEN_MENU's event went out.
    open_menu_sent: bool,
    opened_at: f64,
    /// The values last sent (events go out when one changes).
    sent: Option<Values>,
    aspect: f32,
    /// bo2zm H28: Public Match asked the relay for its games: the list's
    /// generation then, and when (LUI ms).
    public_search: Option<(u64, f64)>,
    /// bo2zm H28: the relay lobby's match started (the menu music stopped).
    relay_started: bool,
    /// bo2zm H28: the Friends list's lobbies, by the xuid each row shows
    /// (`joinplayersessionbyxuid <xuid>` joins that one).
    friend_ads: Vec<(String, net::MasterAdvert)>,
    /// The relay list's generation the Friends rows came from, and when
    /// (LUI ms) they last asked the relay again.
    friends_seen: (u64, f64),
    /// The last Friends row's xuid number (a lobby keeps its xuid).
    friend_next: u64,
    /// bo2zm H28: JOIN on a friend: his own relay lobby closes, then this
    /// one is joined; its lobby opens for him once he is in (LUI ms of the
    /// ask: it gives up after a while).
    pending_join: Option<(net::MasterAdvert, f64)>,
    /// bo2zm H28: his Custom Games lobby on the relay: the map and mode last
    /// asked for, and when (LUI ms).
    private_host: Option<(String, String, f64)>,
    /// bo2zm H28: the relay ids of the lobby's other players, in the order
    /// of `party` (row xuid N is the (N-2)th: `kickplayer <xuid>`).
    party_ids: Vec<net::MemberId>,
    /// bo2zm H28: his game and session modes before a Friends JOIN (where
    /// he goes back to if that lobby ends for him).
    pre_join_modes: Option<(Vec<f32>, Vec<f32>)>,
}

/// The HUD's VM lives on the main thread (its values are `Rc`); beside it,
/// the front end's lobby while a game it started runs.
#[derive(Default)]
pub(crate) struct LuiHudSlot(Option<LuiHud>, Option<FrontState>);

/// bo2zm M4: where the front end was when its game started (the playlist,
/// game and session modes, the playlist filter): after the game BO2 goes
/// back to that lobby.
struct FrontState {
    playlist: usize,
    game_modes: Vec<f32>,
    session_modes: Vec<f32>,
    filter: Option<String>,
}

#[derive(Component)]
struct LuiRoot;

/// What BO2's menus take in and hand out: his keys and mouse, the
/// classic-menu handshake, the pause, and the engine calls they make.
#[derive(bevy::ecs::system::SystemParam)]
struct LuiIo<'w, 's> {
    /// His monitors (BO2's display options count them).
    monitors: Query<'w, 's, &'static bevy::window::Monitor>,
    /// His controller (Start opens the pause menu; A, B, the d-pad and the
    /// shoulders work its menus).
    pads: Query<'w, 's, &'static bevy::input::gamepad::Gamepad>,
    keys: Res<'w, ButtonInput<KeyCode>>,
    mouse: Res<'w, ButtonInput<MouseButton>>,
    menus: ResMut<'w, frame::LuiMenus>,
    paused: ResMut<'w, frame::GamePaused>,
    /// bo2zm: the cheat menu (Num 1) holds the keys and the solo pause.
    cheat: Res<'w, crate::cheat_menu::CheatMenu>,
    exec: MessageWriter<'w, frame::UiExecCommand>,
    sounds: MessageWriter<'w, frame::UiPlaySound>,
    /// The front end's music (`Engine.PlayMenuMusic`), stopped at Start.
    music: MessageWriter<'w, frame::UiPlayMusic>,
    music_stop: MessageWriter<'w, frame::UiStopMusic>,
    inbox: Option<ResMut<'w, net::ClientActionInbox>>,
    ids: Option<ResMut<'w, net::ActionRequestIds>>,
    role: Option<Res<'w, frame::RuntimeRole>>,
    /// The key capture BO2's bind rows start (`Engine.BindCommand`).
    bind: MessageWriter<'w, frame::UiBindRequest>,
    capture: Res<'w, frame::UiBindingCapture>,
    /// His settings, shown and changed by BO2's Settings and Controls.
    settings: ResMut<'w, frame::GameSettings>,
    /// How much the menus blur the world (`Engine.BlurWorld`).
    blur: ResMut<'w, frame::WorldBlur>,
    /// His last input (pad or keys) and his pad's kind: BO2's prompts.
    devices: Option<Res<'w, frame::InputDevices>>,
    /// bo2zm H28: the relay's games (Public Match joins one or hosts its
    /// own), the lobby he is in there, and what the menus ask of it.
    master_browser: Option<Res<'w, net::MasterBrowser>>,
    master_bridge: Option<Res<'w, net::MasterBridge>>,
    master_action: Option<ResMut<'w, net::PendingMasterMenuAction>>,
}

/// The resolutions BO2's video mode offers (his own first if it is not
/// one of them).
const RESOLUTIONS: [&str; 6] = [
    "1280x720",
    "1366x768",
    "1600x900",
    "1920x1080",
    "2560x1440",
    "3840x2160",
];

/// The speakers this PC can play on, as Windows names them (Settings' Sound
/// Device row, "Speakers (Realtek USB Audio)"), the one it plays on first;
/// asked once.
fn sound_devices() -> Vec<String> {
    use cpal::traits::{DeviceTrait, HostTrait};
    static NAMES: std::sync::OnceLock<Vec<String>> = std::sync::OnceLock::new();
    NAMES
        .get_or_init(|| {
            // Windows' full name ("Speakers (Realtek USB Audio)"), not
            // just "Speakers".
            let named = |d: &cpal::Device| {
                let desc = d.description().ok()?;
                Some(desc.extended().first().map_or(desc.name(), |f| f.as_str()).to_owned())
            };
            let host = cpal::default_host();
            let mut names: Vec<String> = host.default_output_device().as_ref().and_then(named).into_iter().collect();
            if let Ok(all) = host.output_devices() {
                for d in all {
                    if let Some(n) = named(&d)
                        && !names.contains(&n)
                    {
                        names.push(n);
                    }
                }
            }
            names
        })
        .clone()
}

/// His settings as BO2's profile and hardware-profile names.
fn profile_of(s: &frame::GameSettings) -> Vec<(String, String)> {
    let b = |v: bool| if v { "1" } else { "0" }.to_owned();
    vec![
        ("r_mode".into(), s.resolution.to_string()),
        ("r_fullscreen".into(), b(s.fullscreen)),
        ("r_vsync".into(), b(s.vsync)),
        ("cg_fov_default".into(), format!("{}", s.fov.round())),
        ("sm_enable".into(), b(s.shadows)),
        // The Shadows row (off -1, low 0 .. high 2): ours are on or off.
        (
            "sm_spotQuality".into(),
            if s.shadows { "2" } else { "-1" }.to_owned(),
        ),
        ("cg_mature".into(), b(s.mature)),
        ("ai_corpseCount".into(), s.corpses.to_string()),
        (
            "cg_drawFPS".into(),
            if s.draw_fps { "Simple" } else { "Off" }.to_owned(),
        ),
        ("com_maxfps".into(), s.max_fps.to_string()),
        ("r_fxaa".into(), b(s.fxaa)),
        ("r_aaSamples".into(), s.aa_samples.to_string()),
        ("r_texFilterQuality".into(), s.tex_filter.to_string()),
        (
            "r_picmip".into(),
            if s.bo2_fast && s.tex_quality < 0 {
                3
            } else {
                s.tex_quality
            }
            .to_string(),
        ),
        // BO2 keeps whether texture quality was picked by hand (1) or is
        // automatic (0); its menu reads it.
        (
            "r_picmip_manual".into(),
            b(s.tex_quality >= 0 || s.bo2_fast),
        ),
        ("snd_menu_master".into(), format!("{:.2}", s.master_volume)),
        ("snd_menu_music".into(), format!("{:.2}", s.music_volume)),
        ("snd_menu_sfx".into(), format!("{:.2}", s.sfx_volume)),
        ("snd_menu_voice".into(), format!("{:.2}", s.voice_volume)),
        (
            "snd_menu_cinematic".into(),
            format!("{:.2}", s.cinematic_volume),
        ),
        // PC Controls' mouse slider (0.01 to 30, default 5: our scale).
        ("mouseSensitivity".into(), format!("{:.2}", s.sensitivity)),
        // The pad's look sensitivity (BO2 0.4 to 4, default 1; ours x3).
        (
            "input_viewSensitivity".into(),
            format!("{:.2}", s.pad_sensitivity / 3.0),
        ),
        // PC Controls' invert mouse (yes -0.022, no 0.022).
        (
            "m_pitch".into(),
            if s.invert_mouse { "-0.022" } else { "0.022" }.to_owned(),
        ),
        // The pad's rows: look inversion, vibration.
        ("input_invertpitch".into(), b(s.pad_invert)),
        ("gpad_rumble".into(), b(s.pad_vibration)),
        // The brightness screen's slider (0.5 to 1.5): his brightness.
        ("r_gamma".into(), format!("{:.2}", 1.0 + s.brightness * 2.5)),
        // His "Omni Movement" row (Controls > Move).
        crate::lui_omni_row::profile(s),
    ]
}

/// One of BO2's names, changed by its menus, into his settings.
fn apply_profile(s: &mut frame::GameSettings, key: &str, value: &str) {
    let on = value.trim() != "0" && !value.trim().is_empty();
    let num = value.trim().parse::<f32>().ok();
    match key {
        "r_mode" => {
            if let Some((w, h)) = value.trim().split_once('x')
                && let (Ok(w), Ok(h)) = (w.parse(), h.parse())
            {
                s.resolution = frame::DisplayResolution::new(w, h);
            }
        }
        "r_fullscreen" => s.fullscreen = on,
        "r_vsync" => s.vsync = on,
        "cg_fov_default" => s.fov = num.unwrap_or(s.fov),
        "sm_enable" => s.shadows = on,
        "snd_menu_master" => s.master_volume = num.unwrap_or(s.master_volume).clamp(0.0, 1.0),
        "snd_menu_music" => s.music_volume = num.unwrap_or(s.music_volume).clamp(0.0, 1.0),
        "snd_menu_sfx" => s.sfx_volume = num.unwrap_or(s.sfx_volume).clamp(0.0, 1.0),
        "snd_menu_voice" => s.voice_volume = num.unwrap_or(s.voice_volume).clamp(0.0, 1.0),
        "snd_menu_cinematic" => {
            s.cinematic_volume = num.unwrap_or(s.cinematic_volume).clamp(0.0, 1.0)
        }
        "mouseSensitivity" => s.sensitivity = num.unwrap_or(s.sensitivity).clamp(0.1, 30.0),
        "input_viewSensitivity" => {
            s.pad_sensitivity = num.map_or(s.pad_sensitivity, |v| (v * 3.0).clamp(1.0, 10.0));
        }
        "m_pitch" => s.invert_mouse = num.is_some_and(|v| v < 0.0),
        "input_invertpitch" => s.pad_invert = on,
        "gpad_rumble" => s.pad_vibration = on,
        "sm_spotQuality" => s.shadows = num.is_none_or(|v| v >= 0.0),
        "cg_mature" => s.mature = on,
        "ai_corpseCount" => s.corpses = num.map_or(s.corpses, |v| v as u8),
        "cg_drawFPS" => {
            s.draw_fps = !value.trim().eq_ignore_ascii_case("off") && value.trim() != "0"
        }
        "com_maxfps" => s.max_fps = num.map_or(s.max_fps, |v| v.max(0.0) as u16),
        "r_fxaa" => s.fxaa = on,
        k if k == crate::lui_omni_row::KEY => s.omni_movement = on,
        "r_aaSamples" => s.aa_samples = num.map_or(s.aa_samples, |v| v as u8),
        "r_texFilterQuality" => {
            s.tex_filter = num.map_or(s.tex_filter, |v| v.clamp(0.0, 2.0) as u8)
        }
        "r_picmip_manual" if !on => {
            s.tex_quality = -1;
            s.bo2_fast = false;
        }
        // Texture quality: low is the fast quality (from the next start).
        "r_picmip" => {
            if let Some(v) = num {
                s.tex_quality = v.clamp(-1.0, 3.0) as i8;
                s.bo2_fast = s.tex_quality == 3;
            }
        }
        "r_gamma" => {
            if let Some(g) = num {
                s.brightness = ((g - 1.0) / 2.5).clamp(-0.2, 0.2);
            }
        }
        _ => {}
    }
}

/// His settings <-> BO2's names: his go in when they change (not by the
/// menus), and a name the menus changed comes back into them.
fn settings_bridge(hud: &mut LuiHud, settings: &mut frame::GameSettings) {
    if hud.settings_rev != Some(settings.revision) {
        // What the game's own code follows straight away: gore and corpses
        // (the game's rules), texture filtering (the samplers).
        sim::set_t6_profile(settings.mature, settings.corpses);
        asset_core::t6_set_aniso(match settings.tex_filter {
            0 => 1,
            1 => 4,
            _ => 16,
        });
        let pairs = profile_of(settings);
        let mut v = hud.host.values.borrow_mut();
        // The name on his lobby row: a name his settings give (the family
        // launcher's profile), else the Steam account this PC signs into.
        v.player_name = settings.shown_name();
        for (k, x) in &pairs {
            v.profile.insert(k.clone(), x.clone());
            v.dvars.insert(k.clone(), x.clone());
        }
        let mut modes: Vec<String> = RESOLUTIONS.iter().map(|r| (*r).to_owned()).collect();
        let mine = settings.resolution.to_string();
        if !modes.contains(&mine) {
            modes.insert(0, mine);
        }
        v.enums.insert("r_mode".to_owned(), modes);
        hud.written = pairs;
        hud.settings_rev = Some(settings.revision);
        return;
    }
    let changed: Vec<(String, String)> = {
        let v = hud.host.values.borrow();
        hud.written
            .iter()
            .filter_map(|(k, old)| {
                let now = v
                    .profile
                    .get(k)
                    .filter(|x| *x != old)
                    .or_else(|| v.dvars.get(k).filter(|x| *x != old))?;
                Some((k.clone(), now.clone()))
            })
            .collect()
    };
    if changed.is_empty() {
        return;
    }
    for (k, x) in &changed {
        diag::info!(Ui, "bo2zm lui: setting {k} = {x}");
        apply_profile(settings, k, x);
    }
    settings.touch();
}

/// His keys as BO2's pad buttons (what its menus listen for).
/// (BO2 on PC turns menu tabs with D, the Settings and Controls pages; A goes back.)
const MENU_KEYS: [(KeyCode, &str); 9] = [
    (KeyCode::Escape, "secondary"),
    (KeyCode::Enter, "primary"),
    (KeyCode::NumpadEnter, "primary"),
    (KeyCode::ArrowUp, "up"),
    (KeyCode::ArrowDown, "down"),
    (KeyCode::ArrowLeft, "left"),
    (KeyCode::ArrowRight, "right"),
    (KeyCode::KeyA, "shoulderl"),
    (KeyCode::KeyD, "shoulderr"),
];

/// Keys and mouse into BO2's menus: Esc opens its pause menu (the HUD's
/// `open_ingame_menu`, menu `class`); with a menu open, keys become pad
/// buttons and the mouse its pointer (root units).
fn menu_input(
    hud: &mut LuiHud,
    io: &LuiIo<'_, '_>,
    console_open: bool,
    cursor: Option<Vec2>,
    scale: f32,
) {
    if console_open || io.cheat.open || io.cheat.ate_keys {
        return;
    }
    use bevy::input::gamepad::GamepadButton as P;
    let pad_start = io.pads.iter().any(|p| p.just_pressed(P::Start));
    // (The front end always has a menu up, inside its black cover.)
    if !hud.frontend && hud.host.open_menus().is_empty() {
        // (With the shell on, its own pause menu opens in a Zombies game.)
        if (io.keys.just_pressed(KeyCode::Escape) || pad_start) && !crate::shell::bridge::enabled() {
            hud.host
                .root_event("open_ingame_menu", &[("menuName", Value::str("class"))]);
        }
        return;
    }
    for (key, button) in MENU_KEYS {
        if io.keys.just_pressed(key) {
            if std::env::var_os("BO2ZM_LUI_LOG").is_some() {
                diag::info!(Ui, "bo2zm lui key {key:?} -> {button}");
                for line in hud.host.menu_report() {
                    diag::info!(Ui, "bo2zm lui menu {line}");
                }
            }
            hud.host.button(button, true);
        } else if io.keys.just_released(key) {
            hud.host.button(button, false);
        }
    }
    // Letter keys for the prompts that show one (F Friends).
    for key in io.keys.get_just_pressed() {
        let name = format!("{key:?}");
        if let Some(letter) = name.strip_prefix("Key").filter(|l| l.len() == 1) {
            hud.host.key_shortcut(letter);
        }
    }
    // The pad's buttons by BO2's names (Start backs out like Esc).
    for pad in &io.pads {
        for (b, name) in [
            (P::South, "primary"),
            (P::East, "secondary"),
            (P::Start, "secondary"),
            (P::West, "alt1"),
            (P::North, "alt2"),
            (P::DPadUp, "up"),
            (P::DPadDown, "down"),
            (P::DPadLeft, "left"),
            (P::DPadRight, "right"),
            (P::LeftTrigger, "shoulderl"),
            (P::RightTrigger, "shoulderr"),
        ] {
            if pad.just_pressed(b) {
                hud.host.button(name, true);
            } else if pad.just_released(b) {
                hud.host.button(name, false);
            }
        }
    }
    // The left stick moves through the menus like the d-pad (one step per
    // push; it repeats while held, as BO2's menus do).
    let stick = io.pads.iter().find_map(|pad| {
        let x = pad
            .get(bevy::input::gamepad::GamepadAxis::LeftStickX)
            .unwrap_or(0.0);
        let y = pad
            .get(bevy::input::gamepad::GamepadAxis::LeftStickY)
            .unwrap_or(0.0);
        let dir = if y > 0.6 {
            Some("up")
        } else if y < -0.6 {
            Some("down")
        } else if x < -0.6 {
            Some("left")
        } else if x > 0.6 {
            Some("right")
        } else {
            None
        };
        dir.or((x.abs() > 0.3 || y.abs() > 0.3).then_some("hold"))
    });
    let now = hud.host.now_ms();
    match stick {
        Some(dir) if dir != "hold" => {
            let fresh = hud.stick.is_none_or(|(d, _)| d != dir);
            let repeat = hud
                .stick
                .is_some_and(|(d, at)| d == dir && now - at > 220.0);
            if fresh || repeat {
                // The first repeat waits longer (a held stick, not a flick).
                let at = if fresh { now + 230.0 } else { now };
                hud.stick = Some((dir, at));
                hud.host.button(dir, true);
                hud.host.button(dir, false);
            }
        }
        Some(_) => {}
        None => hud.stick = None,
    }
    if let Some(c) = cursor {
        let p = c / scale.max(1e-3);
        if hud.mouse_at != Some(p) {
            hud.mouse_at = Some(p);
            hud.host.mouse("mousemove", p.x, p.y, "");
        }
        for (b, name) in [(MouseButton::Left, "left"), (MouseButton::Right, "right")] {
            if io.mouse.just_pressed(b) {
                hud.host.mouse("mousedown", p.x, p.y, name);
            }
            if io.mouse.just_released(b) {
                hud.host.mouse("mouseup", p.x, p.y, name);
            }
        }
    }
}

/// Carry out what the menus asked of the engine.
fn engine_calls(hud: &mut LuiHud, io: &mut LuiIo<'_, '_>, local: Option<&LocalPresentClient>) {
    for call in hud.host.take_calls() {
        diag::info!(Ui, "bo2zm lui engine call: {call:?}");
        match call {
            hks_t6::host::EngineCall::MenuResponse(menu, response) => {
                let (Some(inbox), Some(ids), Some(local)) =
                    (io.inbox.as_mut(), io.ids.as_mut(), local)
                else {
                    continue;
                };
                let (Some(m), Some(r)) = (
                    sim::menu_response_field(&menu),
                    sim::menu_response_field(&response),
                ) else {
                    diag::warn!(
                        Ui,
                        "bo2zm lui: menu response {menu} {response} does not fit"
                    );
                    continue;
                };
                let request_id = ids.allocate();
                if let Err(e) = inbox.push(
                    local.0,
                    sim::ClientAction::MenuResponse {
                        request_id,
                        menu: m,
                        response: r,
                    },
                ) {
                    diag::warn!(Ui, "bo2zm lui: menu response not queued: {e}");
                }
            }
            hks_t6::host::EngineCall::Exec(text) => {
                // BO2's commands as ours; the ones with nothing here to do
                // (pad rumble, party, on-screen keyboard) are dropped.
                let word = text.split_whitespace().next().unwrap_or("");
                // A dvar the menus set as a command (`ui_mapname
                // zm_transit`, Custom Games' default): kept, the map one
                // this game plays.
                if (word.starts_with("ui_") || word.starts_with("party_"))
                    && let Some(value) = text
                        .split_once(' ')
                        .map(|(_, v)| v.trim())
                        .filter(|v| !v.is_empty())
                {
                    let mut v = hud.host.values.borrow_mut();
                    let value = if word.eq_ignore_ascii_case("ui_mapname")
                        && !v.maps.iter().any(|m| m == value)
                    {
                        v.maps.first().cloned().unwrap_or_else(|| value.to_owned())
                    } else {
                        value.to_owned()
                    };
                    let key = v
                        .dvars
                        .keys()
                        .find(|k| k.eq_ignore_ascii_case(word))
                        .cloned()
                        .unwrap_or_else(|| word.to_owned());
                    v.dvars.insert(key, value);
                    if word.eq_ignore_ascii_case("ui_mapname") {
                        hks_t6::host::fit_start(&mut v);
                    }
                    continue;
                }
                // Custom Games' options back to the game's defaults.
                if word.eq_ignore_ascii_case("resetCustomGametype") {
                    let mut v = hud.host.values.borrow_mut();
                    v.settings = v.default_settings.clone();
                    continue;
                }
                let ours = match word {
                    "fast_restart" | "map_restart" => Some("map_restart".to_owned()),
                    "disconnect" | "quit" => Some(word.to_owned()),
                    // bo2zm M4: the playlist's party starts (Solo, Public
                    // Match): its rule is that everyone readies up (Start
                    // Match), as the engine sets from the playlist.
                    "xstartpartyhost" | "xstartparty" if hud.frontend => {
                        let now = hud.host.now_ms();
                        let mut v = hud.host.values.borrow_mut();
                        v.dvars
                            .insert("party_readyPercentRequired".to_owned(), "1".to_owned());
                        // Public Match (Solo hosts its own): the lobby looks
                        // for players, counts down, then starts with him.
                        if word == "xstartparty" {
                            v.public_lobby_since = Some(now);
                            hks_t6::host::party_size(&mut v);
                            // bo2zm H28: with a relay, the search asks it
                            // for the games it has (public_match).
                            if let Some(browser) = io.master_browser.as_ref() {
                                browser.refresh();
                                hud.public_search = Some((browser.snapshot().generation, now));
                                hud.relay_started = false;
                            }
                        }
                        None
                    }
                    // bo2zm H28: JOIN on a friend's row (CoD.joinPlayer): out
                    // of his own relay lobby, then into the friend's.
                    "joinplayersessionbyxuid" if hud.frontend => {
                        let xuid = text.split_whitespace().nth(1).unwrap_or("");
                        let ad = hud
                            .friend_ads
                            .iter()
                            .find(|(x, _)| x == xuid)
                            .map(|(_, a)| a.clone());
                        let Some(ad) = ad else {
                            diag::warn!(Ui, "bo2zm lui: friends join: no lobby for {xuid}");
                            continue;
                        };
                        diag::info!(
                            Ui,
                            "bo2zm lui: friends join {} ({}/{}) {} {}",
                            ad.name,
                            ad.players,
                            ad.max_players,
                            ad.map,
                            ad.mode
                        );
                        hud.public_search = None;
                        hud.private_host = None;
                        hud.pending_join = Some((ad, hud.host.now_ms()));
                        if let Some(action) = io.master_action.as_mut() {
                            action.0 = Some(net::MasterMenuAction::LeaveLobby);
                        }
                        continue;
                    }
                    // bo2zm H28: KICK on a player's row, Yes (KickPlayerPopup:
                    // BO2's UI_KickPlayer_f -> PartyHost_KickPlayer).
                    "kickplayer" if hud.frontend => {
                        let xuid = text.split_whitespace().nth(1).unwrap_or("");
                        let member = xuid
                            .parse::<usize>()
                            .ok()
                            .and_then(|n| n.checked_sub(2))
                            .and_then(|i| hud.party_ids.get(i).copied());
                        match (member, io.master_action.as_mut()) {
                            (Some(member_id), Some(action)) => {
                                diag::info!(Ui, "bo2zm lui: kick {xuid} from his lobby");
                                action.0 = Some(net::MasterMenuAction::Kick { member_id });
                            }
                            _ => diag::warn!(Ui, "bo2zm lui: kick: no player {xuid}"),
                        }
                        continue;
                    }
                    // The Friends list opened (or its tab): the relay's
                    // lobbies, asked again.
                    "updateInfoForInGameList" | "gameInvitesReceived" | "showParty"
                        if hud.frontend =>
                    {
                        if let Some(browser) = io.master_browser.as_ref() {
                            browser.refresh();
                            hud.friends_seen.1 = hud.host.now_ms();
                        }
                        continue;
                    }
                    // He left the Public Match lobby: no countdown runs.
                    "xstopparty" | "xstopallparties" | "xstoppartykeeptogether"
                    | "xstartprivateparty"
                        if hud.frontend =>
                    {
                        let mut v = hud.host.values.borrow_mut();
                        if v.public_lobby_since.take().is_some() {
                            hks_t6::host::party_size(&mut v);
                        }
                        // bo2zm H28: and out of the relay's lobby.
                        hud.public_search = None;
                        hud.private_host = None;
                        hud.pending_join = None;
                        v.party.clear();
                        v.party_follower = false;
                        if io.master_bridge.is_some()
                            && let Some(action) = io.master_action.as_mut()
                        {
                            action.0 = Some(net::MasterMenuAction::LeaveLobby);
                        }
                        None
                    }
                    // bo2zm M4: the front end's Start Match (the party is
                    // ready): load the map the game started for.
                    "xpartyready" | "xpartygo"
                        if hud.frontend
                            && (word == "xpartygo"
                                || text.split_whitespace().nth(1) == Some("1")) =>
                    {
                        hud.host.values.borrow_mut().public_lobby_since = None;
                        io.music_stop.write(frame::UiStopMusic);
                        // The game's settings as the menus left them (Custom
                        // Games' starting round, difficulty, magic,
                        // headshots only; dog rounds stay off here).
                        let v = hud.host.values.borrow();
                        let picked = ["startRound", "zmDifficulty", "magic", "headshotsonly"]
                            .iter()
                            .filter_map(|k| {
                                v.settings.get(*k).map(|x| ((*k).to_owned(), *x as i32))
                            })
                            .collect();
                        // The map, start location and game type the menus
                        // picked (Tranzit's Survival: Town loads zm_transit;
                        // Turned at the Diner its own map, zm_transit_dr).
                        let dvar = |k: &str| hks_t6::host::dvar_get(&v.dvars, k).unwrap_or_default();
                        let (map, loc, mode) = (
                            dvar("ui_mapname"),
                            dvar("ui_zm_mapstartlocation"),
                            dvar("ui_gametype"),
                        );
                        let map = hks_t6::host::start_map(&map, &loc, &mode);
                        drop(v);
                        sim::set_t6_game_settings(picked);
                        // bo2zm H28: in a relay lobby its host starts
                        // everyone's match (the relay tells the others,
                        // who load it as he does); a player who joined
                        // waits for it.
                        let relay = io.master_bridge.as_ref().map(|b| b.state());
                        match &relay {
                            Some(
                                net::MasterBridgeState::Joining { .. }
                                | net::MasterBridgeState::Joined { .. },
                            ) => {
                                diag::info!(Ui, "bo2zm lui: start match waits for the lobby's host");
                                continue;
                            }
                            Some(state @ net::MasterBridgeState::Hosting { .. }) if !map.is_empty() => {
                                let party = 1 + state.other_names().len();
                                diag::info!(
                                    Ui,
                                    "bo2zm lui: relay start match map={map} loc={loc} mode={mode} players={party}"
                                );
                                sim::set_t6_party_size(party);
                                frame::set_bo2_zm_start(&map, &loc, &mode);
                                let private = if state.mode().is_some_and(|m| m.ends_with(":private")) {
                                    ":private"
                                } else {
                                    ""
                                };
                                if let Some(action) = io.master_action.as_mut() {
                                    action.0 = Some(net::MasterMenuAction::StartMatch {
                                        map: format!("t6:{map}"),
                                        mode: format!("{mode}:{loc}{private}"),
                                    });
                                }
                                continue;
                            }
                            _ => sim::set_t6_party_size(0),
                        }
                        frame::bo2_frontend().map(|zone| {
                            if map.is_empty() {
                                return format!("map {zone}");
                            }
                            diag::info!(
                                Ui,
                                "bo2zm lui: start match map={map} loc={loc} mode={mode}"
                            );
                            frame::set_bo2_zm_start(&map, &loc, &mode);
                            let prefix = if zone.starts_with("t6:") { "t6:" } else { "" };
                            format!("map {prefix}{map}")
                        })
                    }
                    _ => None,
                };
                match ours {
                    Some(text) => {
                        io.exec.write(frame::UiExecCommand { text });
                    }
                    None => diag::info!(
                        Ui,
                        "bo2zm lui: engine command `{text}` has nothing to do here"
                    ),
                }
            }
            hks_t6::host::EngineCall::PlaySound(alias) => {
                io.sounds.write(frame::UiPlaySound { alias });
            }
            // bo2 merge: the multiplayer front end's leagues (none here).
            hks_t6::host::EngineCall::FetchLeagueTeams => {
                diag::info!(Ui, "bo2zm lui: league teams asked; Zombies has none");
            }
            hks_t6::host::EngineCall::PlayMusic(alias) => {
                io.music.write(frame::UiPlayMusic { alias });
            }
            hks_t6::host::EngineCall::BlurWorld(amount) => {
                io.blur.0 = amount.max(0.0);
            }
            hks_t6::host::EngineCall::BindCommand(command, _index) => {
                // The game's own capture binds the next key he presses.
                io.bind.write(frame::UiBindRequest { command });
                hud.binding = true;
            }
            hks_t6::host::EngineCall::Event(name, fields) => {
                let fields: Vec<(&str, Value)> = fields
                    .iter()
                    .map(|(k, n)| (k.as_str(), Value::Num(*n)))
                    .collect();
                hud.host.root_event(&name, &fields);
            }
        }
    }
}

/// bo2zm H28: how long Public Match waits for the relay's list of games
/// (ms) before it hosts its own.
const PUBLIC_LOOKUP_MS: f64 = 3000.0;

/// bo2zm H28: Public Match through the relay. Its search joins the first
/// lobby there playing the same map, start and mode with room left (and
/// nothing he lacks), else he hosts his own; the lobby then lists who is in
/// it, and its countdown starts again when someone joins.
fn public_match(hud: &mut LuiHud, io: &mut LuiIo<'_, '_>) {
    if !hud.frontend {
        return;
    }
    let now = hud.host.now_ms();
    if let Some((generation, at)) = hud.public_search {
        let Some(browser) = io.master_browser.as_ref() else {
            hud.public_search = None;
            return;
        };
        let list = browser.snapshot();
        if (list.generation <= generation || list.loading) && now - at < PUBLIC_LOOKUP_MS {
            return;
        }
        hud.public_search = None;
        let v = hud.host.values.borrow();
        let dvar = |k: &str| hks_t6::host::dvar_get(&v.dvars, k).unwrap_or_default();
        let (map, loc, mode) = (
            dvar("ui_mapname"),
            dvar("ui_zm_mapstartlocation"),
            dvar("ui_gametype"),
        );
        drop(v);
        if map.is_empty() {
            return;
        }
        let map = format!("t6:{}", hks_t6::host::start_map(&map, &loc, &mode));
        let token = format!("{mode}:{loc}");
        let open = list.adverts.iter().find(|a| {
            !a.in_match
                && !a.locked
                && a.players < a.max_players
                && a.missing.0 == 0
                && a.map == map
                && a.mode == token
        });
        let action = match open {
            Some(a) => {
                diag::info!(
                    Ui,
                    "bo2zm lui: public match joins {} ({}/{}) {map} {token}",
                    a.name,
                    a.players,
                    a.max_players
                );
                net::MasterMenuAction::Join {
                    advert_id: a.id,
                    map,
                    mode: token,
                }
            }
            None => {
                diag::info!(
                    Ui,
                    "bo2zm lui: public match hosts {map} {token} ({} games on the relay)",
                    list.adverts.len()
                );
                net::MasterMenuAction::Host { map, mode: token }
            }
        };
        if let Some(pending) = io.master_action.as_mut() {
            pending.0 = Some(action);
        }
        return;
    }
    // The host removed him from his lobby (BO2's PartyClient_HandleKicked).
    if hud.host.values.borrow().party_follower
        && io.master_bridge.as_ref().is_some_and(|b| b.state().kicked())
    {
        party_ended(hud, io, "@PLATFORM_KICKEDFROMPARTY");
        return;
    }
    // The host of his Custom Games lobby left: BO2 moves a party to a new
    // host only in public games (PartyMigrate_IsEnabled); this one ends.
    if hud.host.values.borrow().party_follower
        && hud.host.values.borrow().game_modes.contains(&1.0)
        && io.master_bridge.as_ref().is_some_and(|b| b.state().host_gone())
    {
        party_ended(hud, io, "@XBOXLIVE_PARTYENDED");
        return;
    }
    // Who is in his relay lobby (none once it closed or he left it).
    let (names, follower, in_match) = match io.master_bridge.as_ref() {
        Some(bridge) => {
            let state = bridge.state();
            let follower = matches!(
                state,
                net::MasterBridgeState::Joining { .. } | net::MasterBridgeState::Joined { .. }
            );
            hud.party_ids = state.other_ids();
            (state.other_names(), follower, state.in_match())
        }
        None => {
            hud.party_ids.clear();
            (Vec::new(), false, false)
        }
    };
    let (joined, host_changed) = {
        let mut v = hud.host.values.borrow_mut();
        if names != v.party || follower != v.party_follower {
            let joined = names.len() > v.party.len() || (follower && !v.party_follower);
            let host_changed = follower != v.party_follower;
            diag::info!(Ui, "bo2zm lui: relay lobby {names:?} (joined someone's: {follower})");
            v.party = names;
            v.party_follower = follower;
            // Someone came in: the countdown starts over.
            if joined && v.public_lobby_since.is_some() {
                v.public_lobby_since = Some(now - hks_t6::host::PUBLIC_SEARCH_MS);
            }
            (joined, host_changed)
        } else {
            (false, false)
        }
    };
    // The lobby's buttons follow (BO2's party_joined / party_update_host:
    // a player who joined someone's has no Start Match).
    if host_changed {
        hud.host.root_event(
            "party_update_host",
            &[
                ("controller", Value::Num(0.0)),
                ("isHost", Value::Bool(!follower)),
            ],
        );
    }
    if joined {
        hud.host
            .root_event("party_joined", &[("controller", Value::Num(0.0))]);
    }
    // The host started the match: the lobby's music stops as it loads.
    if in_match && !hud.relay_started {
        hud.relay_started = true;
        hud.host.values.borrow_mut().public_lobby_since = None;
        io.music_stop.write(frame::UiStopMusic);
    } else if !in_match {
        hud.relay_started = false;
    }
}

/// bo2zm H28: his friend's lobby ended for him (BO2's
/// PartyClient_DisconnectFromHost): he is out of it, back where he joined
/// from, and the notice says why (popup_xboxlive_partyended shows the
/// partyend_reason dvar).
fn party_ended(hud: &mut LuiHud, io: &mut LuiIo<'_, '_>, reason: &str) {
    diag::info!(Ui, "bo2zm lui: his friend's lobby ended for him ({reason})");
    {
        let mut v = hud.host.values.borrow_mut();
        v.party.clear();
        v.party_follower = false;
        v.dvars
            .insert("partyend_reason".to_owned(), reason.to_owned());
        match hud.pre_join_modes.take() {
            Some((game, session)) => {
                v.game_modes = game;
                v.session_modes = session;
            }
            None => v.game_modes.retain(|m| *m != 1.0),
        }
        hks_t6::host::party_size(&mut v);
    }
    hud.party_ids.clear();
    hud.pending_join = None;
    hud.private_host = None;
    if let Some(action) = io.master_action.as_mut() {
        action.0 = Some(net::MasterMenuAction::LeaveLobby);
    }
    for menu in ["MainLobby", "popup_xboxlive_partyended"] {
        hud.host.root_event(
            "open_menu",
            &[
                ("controller", Value::Num(0.0)),
                ("menuName", Value::str(menu)),
            ],
        );
    }
}

/// bo2zm H28: how long a Friends JOIN waits to be in the friend's lobby
/// (ms) before it gives up.
const JOIN_WAIT_MS: f64 = 15000.0;
/// ... how often the Friends rows ask the relay again (ms).
const FRIENDS_REFRESH_MS: f64 = 5000.0;
/// ... how long his Custom Games lobby waits for the relay (ms) before it
/// asks again.
const PRIVATE_HOST_RETRY_MS: f64 = 5000.0;

/// bo2zm H28: one Friends row for a lobby on the relay: its host, where it
/// is (the map, how full, started), and whether JOIN can take him there.
fn friend_row(v: &hks_t6::host::EngineValues, xuid: &str, a: &net::MasterAdvert) -> hks_t6::host::Friend {
    let load = a.map.strip_prefix("t6:").unwrap_or(&a.map);
    let mut status = format!(
        "{}  {}/{}",
        hks_t6::host::map_shown_name(v, load),
        a.players,
        a.max_players
    );
    if a.in_match {
        status.push_str("  In game");
    }
    hks_t6::host::Friend {
        xuid: xuid.to_owned(),
        name: a.name.clone(),
        status,
        joinable: !a.in_match && !a.locked && a.players < a.max_players && a.missing.0 == 0,
    }
}

/// bo2zm H28: the Friends rows: every other Zombies lobby on the relay.
fn friend_rows(
    hud: &mut LuiHud,
    browser: &net::MasterBrowser,
    state: Option<&net::MasterBridgeState>,
    now: f64,
) {
    if now - hud.friends_seen.1 > FRIENDS_REFRESH_MS {
        browser.refresh();
        hud.friends_seen.1 = now;
    }
    let list = browser.snapshot();
    if list.generation != hud.friends_seen.0 {
        hud.friends_seen.0 = list.generation;
        let own = state
            .filter(|s| !s.is_terminal())
            .map(|s| s.identity().room_id);
        let mut ads = Vec::new();
        for a in list
            .adverts
            .iter()
            .filter(|a| Some(a.id) != own && a.map.starts_with("t6:zm_"))
        {
            let xuid = match hud.friend_ads.iter().find(|(_, o)| o.id == a.id) {
                Some((x, _)) => x.clone(),
                None => {
                    hud.friend_next += 1;
                    hud.friend_next.to_string()
                }
            };
            ads.push((xuid, a.clone()));
        }
        let rows: Vec<_> = {
            let v = hud.host.values.borrow();
            ads.iter().map(|(x, a)| friend_row(&v, x, a)).collect()
        };
        hud.friend_ads = ads;
        let changed = {
            let mut v = hud.host.values.borrow_mut();
            let changed = v.friends != rows;
            v.friends = rows;
            changed
        };
        if changed {
            let names: Vec<_> = hud
                .friend_ads
                .iter()
                .map(|(x, a)| format!("{x} {} {}", a.name, a.mode))
                .collect();
            diag::info!(Ui, "bo2zm lui: friends {names:?}");
            hud.host
                .root_event("friends_updated", &[("controller", Value::Num(0.0))]);
        }
    }
}

/// bo2zm H28: Custom Games over the relay, BO2's LAN party. His Custom
/// Games lobby (ONLINE > CUSTOM GAMES) is a relay lobby; the Friends list
/// shows the others' lobbies; JOIN on one closes his own, joins it, and
/// opens that lobby for him (the host's Start Match starts them all).
fn private_party(hud: &mut LuiHud, io: &mut LuiIo<'_, '_>) {
    if !hud.frontend {
        return;
    }
    let now = hud.host.now_ms();
    let state = io.master_bridge.as_ref().map(|b| b.state());
    // The Friends rows (the relay's list is there only while he is in
    // no relay lobby; in a friend's, his lobby still follows it below).
    if let Some(browser) = io.master_browser.as_ref() {
        friend_rows(hud, browser, state.as_ref(), now);
    }
    let Some(pending) = io.master_action.as_mut() else {
        return;
    };
    // JOIN on a friend: once his own lobby is closed (the bridge gone and
    // the leave applied), join the friend's; once in, its lobby opens.
    if let Some((ad, at)) = hud.pending_join.clone() {
        match &state {
            None if pending.0.is_none() && hud.private_host.is_none() => {
                diag::info!(Ui, "bo2zm lui: friends join asks the relay for {}", ad.name);
                pending.0 = Some(net::MasterMenuAction::Join {
                    advert_id: ad.id,
                    map: ad.map.clone(),
                    mode: ad.mode.clone(),
                });
                // (Asked once: the mark keeps it from asking again.)
                hud.private_host = Some((ad.map.clone(), ad.mode.clone(), now));
            }
            Some(net::MasterBridgeState::Joined { map, mode, .. }) => {
                hud.pending_join = None;
                hud.private_host = None;
                let map = map.strip_prefix("t6:").unwrap_or(map).to_owned();
                let mut parts = mode.split(':');
                let (gametype, loc) = (
                    parts.next().unwrap_or("").to_owned(),
                    parts.next().unwrap_or("").to_owned(),
                );
                {
                    let mut v = hud.host.values.borrow_mut();
                    hud.pre_join_modes = Some((v.game_modes.clone(), v.session_modes.clone()));
                    // The globe's map (Turned at the Diner loads its own,
                    // zm_transit_dr: the lobby shows TranZit's).
                    let globe = v
                        .maps
                        .iter()
                        .find(|m| map == **m || map.starts_with(m.as_str()))
                        .cloned()
                        .unwrap_or_else(|| map.clone());
                    v.dvars.insert("ui_mapname".to_owned(), globe);
                    v.dvars.insert("ui_zm_mapstartlocation".to_owned(), loc);
                    v.dvars.insert("ui_gametype".to_owned(), gametype);
                    // An online private match (SwitchToPrivateLobby's modes).
                    v.game_modes = vec![1.0];
                    for m in [2.0, 3.0] {
                        if !v.session_modes.contains(&m) {
                            v.session_modes.push(m);
                        }
                    }
                    hks_t6::host::party_size(&mut v);
                }
                diag::info!(Ui, "bo2zm lui: friends join in: {} lobby opens", ad.name);
                hud.host.root_event(
                    "open_menu",
                    &[
                        ("controller", Value::Num(0.0)),
                        ("menuName", Value::str("PrivateOnlineGameLobby")),
                    ],
                );
            }
            Some(s) if s.is_terminal() || now - at > JOIN_WAIT_MS => {
                diag::warn!(Ui, "bo2zm lui: friends join {} failed ({s:?})", ad.name);
                hud.pending_join = None;
                hud.private_host = None;
            }
            None if now - at > JOIN_WAIT_MS => {
                diag::warn!(Ui, "bo2zm lui: friends join {} never started", ad.name);
                hud.pending_join = None;
                hud.private_host = None;
            }
            _ => {}
        }
        return;
    }
    // In a friend's lobby: it follows the host's map, start and mode.
    if let Some(net::MasterBridgeState::Joined { map, mode, .. }) = &state
        && mode.ends_with(":private")
    {
        let map = map.strip_prefix("t6:").unwrap_or(map);
        let mut parts = mode.split(':');
        let (gametype, loc) = (parts.next().unwrap_or(""), parts.next().unwrap_or(""));
        let mut v = hud.host.values.borrow_mut();
        let globe = v
            .maps
            .iter()
            .find(|m| map == m.as_str() || map.starts_with(m.as_str()))
            .cloned()
            .unwrap_or_else(|| map.to_owned());
        for (k, x) in [
            ("ui_mapname", globe.as_str()),
            ("ui_zm_mapstartlocation", loc),
            ("ui_gametype", gametype),
        ] {
            if hks_t6::host::dvar_get(&v.dvars, k).as_deref() != Some(x) {
                v.dvars.insert(k.to_owned(), x.to_owned());
            }
        }
        return;
    }
    // His Custom Games lobby (an online private match) is a relay lobby
    // the others' Friends lists show.
    let want = {
        let v = hud.host.values.borrow();
        let custom = v.game_modes.contains(&1.0)
            && v.session_modes.contains(&2.0)
            && !v.party_follower
            && v.public_lobby_since.is_none();
        let dvar = |k: &str| hks_t6::host::dvar_get(&v.dvars, k).unwrap_or_default();
        let (map, loc, mode) = (
            dvar("ui_mapname"),
            dvar("ui_zm_mapstartlocation"),
            dvar("ui_gametype"),
        );
        (custom && !map.is_empty()).then(|| {
            (
                format!("t6:{}", hks_t6::host::start_map(&map, &loc, &mode)),
                format!("{mode}:{loc}:private"),
            )
        })
    };
    let Some((map, mode)) = want else {
        hud.private_host = None;
        return;
    };
    if pending.0.is_some() {
        return;
    }
    let asked = hud
        .private_host
        .as_ref()
        .is_some_and(|(m, d, at)| *m == map && *d == mode && now - at < PRIVATE_HOST_RETRY_MS);
    match &state {
        Some(s @ net::MasterBridgeState::Hosting { .. }) => {
            if (s.map(), s.mode()) != (Some(map.as_str()), Some(mode.as_str())) && !asked {
                diag::info!(Ui, "bo2zm lui: custom games lobby now {map} {mode}");
                hud.private_host = Some((map.clone(), mode.clone(), now));
                pending.0 = Some(net::MasterMenuAction::UpdateLobby { map, mode });
            }
        }
        // A closed one goes first (a new lobby needs the relay's list).
        Some(s) if s.is_terminal() => {
            pending.0 = Some(net::MasterMenuAction::LeaveLobby);
        }
        Some(_) => {}
        None => {
            if !asked {
                diag::info!(Ui, "bo2zm lui: custom games hosts {map} {mode}");
                hud.private_host = Some((map.clone(), mode.clone(), now));
                pending.0 = Some(net::MasterMenuAction::Host { map, mode });
            }
        }
    }
}

/// One drawn element (its LUI id) and what it shows (rebuilt on change;
/// its alpha is set in place).
#[derive(Component)]
struct LuiNode {
    id: usize,
    shows: String,
    alpha: f32,
}

fn build(ui: &assets::T6Ui, frontend: bool) -> LuiHud {
    let metrics: Metrics = Rc::default();
    let m = metrics.clone();
    let measure = Box::new(move |text: &str, font: &str, h: f32| {
        let m = m.borrow();
        let Some((px, adv, space)) = m.get(font_name(font)) else {
            return text.chars().count() as f32 * h * 0.5;
        };
        let w: f32 = text
            .chars()
            .map(|c| {
                if c == ' ' {
                    *space
                } else {
                    adv.get(&c).copied().unwrap_or(*space)
                }
            })
            .sum();
        w * h / px.max(1.0)
    });
    let scripts = ui
        .scripts
        .iter()
        .map(|(n, b)| (n.clone(), b.to_vec()))
        .collect();
    let mut host = Host::new(scripts, measure);
    host.load_base();
    {
        let mut v = host.values.borrow_mut();
        v.localize = ui
            .strings
            .iter()
            .map(|(k, t)| (k.to_ascii_uppercase(), t.clone()))
            .collect();
        // BO2ZM_LUI_FINDSTR=<words,...>: every localized string whose text
        // holds one of the words (finding a key; debugging aid).
        if let Ok(words) = std::env::var("BO2ZM_LUI_FINDSTR") {
            let words: Vec<String> = words.split(',').map(str::to_ascii_lowercase).collect();
            let mut hits: Vec<_> = v
                .localize
                .iter()
                .filter(|(_, t)| {
                    let t = t.to_ascii_lowercase();
                    words.iter().any(|w| t.contains(w.as_str()))
                })
                .collect();
            hits.sort();
            for (k, t) in hits {
                diag::info!(Ui, "bo2zm lui findstr {k} = {t:?}");
            }
        }
        diag::info!(
            Ui,
            "bo2zm lui: string tables {}",
            ui.tables
                .iter()
                .map(|t| t.0.as_str())
                .collect::<Vec<_>>()
                .join(" ")
        );
        for (name, cols, _rows, cells) in ui.tables.iter() {
            let rows = cells
                .chunks((*cols).max(1))
                .map(<[String]>::to_vec)
                .collect();
            v.tables.insert(name.clone(), rows);
        }
        // The game settings' defaults (Custom Games' rows start on them).
        for (name, value) in ui.game_settings.iter() {
            v.settings.insert(name.clone(), *value);
        }
    }
    zombies_values(&mut host);
    {
        let mut v = host.values.borrow_mut();
        v.front_end = frontend;
        v.default_settings = v.settings.clone();
    }
    // BO2's front end (its own script state, as BO2 keeps it apart from
    // the game's HUD) out of a game; test aid: BO2ZM_LUI_FRONTEND=1 builds
    // it over a game too.
    if frontend || std::env::var_os("BO2ZM_LUI_FRONTEND").is_some() {
        host.require("T6.Main");
        // Its loading screen (the engine opens `Loading` while a map loads).
        host.require("T6.HUD.Loading");
    } else {
        host.require("T6.HUD");
    }
    // bo2zm: his "Omni Movement" row on the Controls menu (a tester 10-10).
    crate::lui_omni_row::add(&mut host);
    // Test aid: BO2ZM_LUI_REQUIRE=<module>[,<module>] loads more of BO2's UI
    // (the front end: `T6.Main`).
    if let Ok(more) = std::env::var("BO2ZM_LUI_REQUIRE") {
        for m in more.split(',').filter(|m| !m.is_empty()) {
            host.require(m.trim());
        }
    }
    LuiHud {
        host,
        metrics,
        frontend,
        lobby_update_at: 0.0,
        lobby_sig: String::new(),
        lobby_again: false,
        loading_open: false,
        return_menu: None,
        stick: None,
        built_from: ui.scripts.len(),
        opened: false,
        mouse_at: None,
        test_step: 0,
        asked_logged: false,
        open_menu_sent: false,
        dumped: 0,
        opened_at: 0.0,
        binding: false,
        settings_rev: None,
        written: Vec::new(),
        sent: None,
        aspect: 0.0,
        board_open: false,
        public_search: None,
        relay_started: false,
        friend_ads: Vec::new(),
        friends_seen: (0, 0.0),
        friend_next: 1000,
        pending_join: None,
        private_host: None,
        party_ids: Vec::new(),
        pre_join_modes: None,
    }
}

/// Nuketown Zombies as the engine reports it: survival, standard rules,
/// the HUD shown, one player on the allies.
/// The Black Ops maps on BO2's globe, in globe order, each near its place
/// (the maps table's globe columns: degrees west, degrees north, as BO2's
/// own rows give Mob of the Dead 122 35). Each sits at least as far from
/// its neighbours as BO2's own closest pair (Nuketown and Mob, 10.7
/// degrees), so every icon and NEW tag stands clear and can be picked.
const BO1_GLOBE: &[(&str, f32, f32)] = &[
    // Nacht der Untoten: west of Origins (11 east 49 north).
    ("zm_prototype", 6.0, 52.0),
    // Verrückt: north of Origins.
    ("zm_asylum", -13.0, 62.0),
    // Der Riese: Breslau, east of Origins.
    ("zm_factory", -28.0, 50.0),
    // Kino der Toten: south of Origins.
    ("zm_theater", -14.0, 37.0),
    // Shi No Numa: Japan.
    ("zm_sumpf", -138.0, 36.0),
    // Five: the Pentagon.
    ("zm_pentagon", 77.0, 39.0),
    // Ascension: Baikonur.
    ("zm_cosmodrome", -63.0, 46.0),
    // Shangri-La: the Himalayas.
    ("zm_temple", -86.0, 28.0),
    // Moon: south of Nuketown (110 40), clear of TranZit and Mob.
    ("zm_moon", 104.0, 27.0),
];

fn zombies_values(host: &mut Host) {
    let hud_bit = host
        .field("CoD", "BIT_HUD_VISIBLE")
        .as_num()
        .map_or(0, |n| n as i32);
    let allies = host
        .field("CoD", "TEAM_ALLIES")
        .as_num()
        .map_or(1, |n| n as i32);
    // An offline game (Restart Level in the pause menu).
    let offline = host.field("CoD", "SESSIONMODE_OFFLINE").as_num();
    let mut v = host.values.borrow_mut();
    for (k, x) in [
        ("r_fontResolution", "720"),
        ("ui_gametype", "zstandard"),
        ("g_gametype", "zstandard"),
        ("ui_zm_gamemodegroup", "zsurvival"),
        ("ui_zm_mapstartlocation", "nuked"),
        ("ui_mapname", "zm_nuked"),
        ("mapname", "zm_nuked"),
        // He hosts his own game (CoD.isHost: the leave-game popup ends it).
        ("sv_running", "1"),
        // The engine's own (the front end reads them).
        ("developer", "0"),
        // His party: eight in the public lobby, four in a game (BO2's
        // zombies party; `party_size` switches it), one at this PC.
        ("party_maxplayers", "8"),
        ("party_maxlocalplayers", "1"),
        // Each lobby kind's local players (one: this PC) and party size.
        ("party_maxlocalplayers_mainlobby", "1"),
        ("party_maxlocalplayers_playermatch", "1"),
        ("party_maxlocalplayers_privatematch", "1"),
        ("party_maxlocalplayers_theater", "1"),
        ("party_maxlocalplayers_systemlink", "1"),
        ("party_maxlocalplayers_local_splitscreen", "1"),
        ("party_maxplayers_privatematch", "4"),
        ("party_maxplayers_theater", "4"),
        ("party_maxplayers_systemlink", "4"),
        ("party_maxplayers_local_splitscreen", "1"),
        // The public Zombies lobby: "1 Player (8 Max)" in the real game.
        ("party_maxplayers_partylobby", "8"),
        ("party_playerCount", "1"),
        // Everyone in the party must be ready (he is: Start Match).
        ("party_readyPercentRequired", "1"),
        // Settings' Voice Chat page as the real game first opens it.
        ("cl_voice", "1"),
        // Settings' Sound Device row: the speakers this PC can play on.
        ("sd_can_switch_device", "1"),
    ] {
        v.dvars.insert(k.to_owned(), x.to_owned());
    }
    let speakers = sound_devices();
    if let Some(name) = speakers.first() {
        v.dvars.insert("sd_xa2_device_name".to_owned(), name.clone());
        v.dvars.insert("sd_xa2_num_devices".to_owned(), speakers.len().to_string());
        v.enums.insert("sd_xa2_device_name".to_owned(), speakers.clone());
    }
    // The volume sliders the profile keeps, as the real game first has
    // them (Codcaster's run 0 to 2).
    for (k, x) in [
        ("snd_shoutcast_game", "0.25"),
        ("snd_shoutcast_voip", "1"),
        ("snd_voicechat_volume", "1"),
        ("snd_voicechat_record_level", "1"),
    ] {
        v.profile.entry(k.to_owned()).or_insert_with(|| x.to_owned());
    }
    v.settings.insert("startRound".to_owned(), 1.0);
    // The maps this rebuild plays: every Zombies map on BO2's globe.
    v.maps = [
        "zm_transit",
        "zm_nuked",
        "zm_highrise",
        "zm_prison",
        "zm_buried",
        "zm_tomb",
    ]
    .map(str::to_owned)
    .to_vec();
    // The Black Ops maps, when their pack is there (its maps table rows join
    // BO2's): each on the globe at its own real-world place.
    let have: std::collections::BTreeSet<String> = v
        .tables
        .get("zm/mapstable.csv")
        .map(|rows| rows.iter().filter_map(|r| r.first().cloned()).collect())
        .unwrap_or_default();
    for (map, _, _) in BO1_GLOBE {
        if have.contains(*map) {
            v.maps.push((*map).to_owned());
        }
    }
    if let Some(rows) = v.tables.get_mut("zm/mapstable.csv") {
        for r in rows.iter_mut() {
            let Some((_, lon, lat)) = BO1_GLOBE.iter().find(|(m, ..)| r.first().is_some_and(|x| x == m)) else {
                continue;
            };
            if r.len() > 17 {
                r[16] = lon.to_string();
                r[17] = lat.to_string();
            }
        }
    }
    // In a game the front end started: the map, location and mode it picked
    // (the game-over title names them).
    if let Some((map, loc, mode)) = frame::bo2_zm_choice() {
        for (k, x) in [
            ("ui_mapname", &map),
            ("mapname", &map),
            ("ui_zm_mapstartlocation", &loc),
            ("ui_gametype", &mode),
            ("g_gametype", &mode),
        ] {
            v.dvars.insert(k.to_owned(), x.clone());
        }
        // Its group too: the HUD and, after the game, the lobby picture read it.
        if let Some(g) = hks_t6::host::mode_group(&v.tables, &mode) {
            v.dvars.insert("ui_zm_gamemodegroup".to_owned(), g);
        }
    }
    // One team (the scoreboard draws a team's rows only when it counts
    // it), and Zombies' columns (its `setscoreboardcolumns`).
    v.settings.insert("teamCount".to_owned(), 1.0);
    v.columns = [
        "MPUI_SCORE",
        "MPUI_KILLS",
        "MPUI_DOWNS",
        "MPUI_REVIVES",
        "MPUI_HEADSHOTS",
    ]
    .map(str::to_owned)
    .to_vec();
    v.bits.insert(hud_bit);
    v.team = allies;
    v.session_modes = offline.into_iter().collect();
}

/// The values the HUD shows, from the presented snapshot.
#[derive(Clone, Debug, Default, PartialEq)]
struct Values {
    round: i32,
    score: i32,
    weapon: i32,
    clip: i32,
    /// A dual-wield weapon's left clip.
    clip_left: Option<i32>,
    stock: i32,
    /// His grenades and mines (`name:count` by `;`).
    offhand: String,
    /// His HUD client fields (`field:value` by `,`: perks, power-ups).
    icons: String,
    /// The world's HUD client fields (`field:value` by `,`: Mob's parts,
    /// Buried's Time Bomb).
    worldcf: String,
    /// His points counters (`score_cf_damage:3,...`).
    scorecf: String,
    /// His d-pad slots (`slot:weapon` by `;`, the scripts' setactionslot).
    slots: String,
    /// The held weapon's name and display-name key (`ZOMBIE_WEAPON_...`).
    weapon_name: String,
    weapon_key: String,
    /// The held weapon's inventory type (0 gun, 1 offhand, 2 item - a perk
    /// bottle, the knuckle crack -, 3 alt mode), as the game's HUD reads it.
    weapon_inventory: i32,
    /// The held weapon's Black Ops II `weapType` (7 melee: no ammo counter).
    weapon_type: i32,
    /// His team as the engine keeps it (1 axis, 2 allies), from the
    /// scripts' `sessionteam`: a Turned zombie is axis.
    team: i32,
    /// The game type the game runs (`zcleansed` Turned): a test run starts
    /// without the menus, so the HUD learns it from the game.
    gametype: String,
    /// The scoreboard's rows (`clientnum:name:score:kills:downs:revives:
    /// headshots` by `;`).
    board: String,
    /// The scripts hid his HUD (the game over).
    hidden: bool,
    /// The scripts hid his ammo counter and points (the game over's
    /// setclientammocounterhide / setclientminiscoreboardhide).
    ammo_hide: bool,
    /// The game-over shot (he watches the intermission camera).
    game_over: bool,
}

/// A d-pad weapon's icon.
fn slot_icon(weapon: &str) -> Option<&'static str> {
    Some(match weapon {
        "claymore_zm" => "hud_icon_claymore",
        _ => return None,
    })
}

/// The HUD icon of a grenade the engine names in `hud_update_offhand`, and
/// its slot.
fn offhand_slot(weapon: &str) -> Option<(&'static str, &'static str)> {
    Some(match weapon {
        "frag_grenade_zm" => ("lethal", "hud_us_grenade"),
        "sticky_grenade_zm" => ("lethal", "hud_icon_sticky_grenade"),
        "cymbal_monkey_zm" => ("tactical", "hud_cymbal_monkey"),
        _ => return None,
    })
}

/// (field, value) pairs of `bo2zm_icons` (Mob's Afterlife mana is a
/// float, 0 to 1).
fn icon_fields(value: &str) -> Vec<(&str, f32)> {
    value
        .split(',')
        .filter_map(|e| {
            let (f, v) = e.split_once(':')?;
            Some((f, v.trim().parse().unwrap_or(0.0)))
        })
        .collect()
}

/// A clip from the player's clip table (rows of 12 bytes: the weapon, the
/// right hand's count, the left hand's).
fn clip_of(table: &[u8], weapon: i32, hand: usize) -> i32 {
    table
        .chunks_exact(12)
        .find(|row| i32::from_le_bytes([row[0], row[1], row[2], row[3]]) == weapon)
        .map_or(0, |row| {
            let o = 4 + hand.min(1) * 4;
            i32::from_le_bytes([row[o], row[o + 1], row[o + 2], row[o + 3]])
        })
}

fn read_values(presented: &PresentedSnapshot, local: &LocalPresentClient) -> Option<Values> {
    let snap = presented.snapshot()?;
    let dvars = snap.meta.script_dvars(local.0);
    let hidden = dvars.string("bo2zm_hud_hidden") == Some("1");
    let round = dvars.string("bo2zm_round")?.trim().parse().ok()?;
    let meta = snap.meta.for_client(local.0)?;
    let ps = presented.player(local.0);
    Some(Values {
        round,
        score: meta.score,
        weapon: ps.map_or(0, |p| p.weapon as i32),
        clip: meta.ammo_clip,
        clip_left: ps
            .filter(|p| p.last_weapon_hand == 1)
            .map(|p| clip_of(&p.ammoclip, p.weapon as i32, 1)),
        stock: meta.ammo_stock,
        offhand: dvars.string("bo2zm_offhand").unwrap_or_default().to_owned(),
        icons: dvars.string("bo2zm_icons").unwrap_or_default().to_owned(),
        worldcf: dvars.string("bo2zm_worldcf").unwrap_or_default().to_owned(),
        scorecf: dvars.string("bo2zm_scorecf").unwrap_or_default().to_owned(),
        slots: dvars
            .string("bo2zm_actionslots")
            .unwrap_or_default()
            .to_owned(),
        weapon_name: String::new(),
        weapon_key: String::new(),
        weapon_inventory: 0,
        weapon_type: 0,
        team: meta.client_state_team,
        gametype: dvars.string("bo2zm_gametype").unwrap_or_default().to_owned(),
        board: dvars
            .string("bo2zm_scoreboard")
            .unwrap_or_default()
            .to_owned(),
        hidden,
        ammo_hide: dvars.string("bo2zm_ammo_hide") == Some("1")
            || dvars.string("bo2zm_score_hide") == Some("1"),
        game_over: ps.is_some_and(|p| p.pm_type == playerstate_iw4::PM_TYPE_INTERMISSION),
    })
}

/// The engine's events for what changed since `old` (everything the first
/// time).
fn send_changes(host: &mut Host, old: Option<&Values>, new: &Values) {
    if old != Some(new) && std::env::var_os("BO2ZM_LUI_LOG").is_some() {
        diag::info!(Ui, "bo2zm lui values at {:.0} ms {new:?}", host.now_ms());
    }
    let changed = |f: fn(&Values) -> i64| old.is_none_or(|o| f(o) != f(new));
    if old.is_none_or(|o| o.board != new.board) {
        let mut v = host.values.borrow_mut();
        v.players.clear();
        v.clients.clear();
        for row in new.board.split(';').filter(|r| !r.is_empty()) {
            let mut cells = row.split(':');
            let client = cells.next().and_then(|c| c.parse().ok()).unwrap_or(0);
            v.clients.push(client);
            v.players.push(cells.map(str::to_owned).collect());
        }
    }
    if old.is_some_and(|o| o.ammo_hide != new.ammo_hide) {
        // BO2's HUD bit 12 (CoD.BIT_AMMO_COUNTER_HIDE): its ammo area and
        // score listen for hud_update_bit_12 and read the bit.
        let bit = host
            .field("CoD", "BIT_AMMO_COUNTER_HIDE")
            .as_num()
            .map_or(12, |n| n as i32);
        {
            let mut v = host.values.borrow_mut();
            if new.ammo_hide {
                v.bits.insert(bit);
            } else {
                v.bits.remove(&bit);
            }
        }
        host.root_event(
            &format!("hud_update_bit_{bit}"),
            &[("newValue", Value::Num(f32::from(u8::from(new.ammo_hide))))],
        );
    }
    if changed(|v| i64::from(v.round)) {
        host.root_event(
            "hud_update_rounds_played",
            &[
                ("roundsPlayed", Value::Num(new.round as f32)),
                ("wasDemoJump", Value::Bool(false)),
            ],
        );
    }
    // His points counters first (as BO2 runs client fields before the
    // scoreboard's update): each step flies its own "+10", and the score's
    // change then flies only what they did not cover (spending, bonuses).
    if let Some(o) = old
        && o.scorecf != new.scorecf
    {
        let was = icon_fields(&o.scorecf);
        for (f, v) in icon_fields(&new.scorecf) {
            let before = was.iter().find(|(g, _)| *g == f).map_or(0.0, |(_, x)| *x);
            if before != v {
                if std::env::var_os("IW4L_T6_HUDLOG").is_some() {
                    diag::info!(Ui, "bo2zm lui points step {f} {before} -> {v}");
                }
                host.root_event(
                    f,
                    &[
                        ("name", Value::str(f)),
                        ("entNum", Value::Num(0.0)),
                        ("newValue", Value::Num(v)),
                        ("oldValue", Value::Num(before)),
                    ],
                );
            }
        }
    }
    if changed(|v| i64::from(v.score)) {
        // One row per player (here the local one, client 0), ours first.
        let rows = hks_t6::value::Table::new_ref();
        let row = hks_t6::value::Table::new_ref();
        row.borrow_mut()
            .set_str("score", Value::Num(new.score as f32));
        row.borrow_mut().set_str("clientNum", Value::Num(0.0));
        rows.borrow_mut().set(Value::Num(1.0), Value::Table(row));
        host.root_event(
            "hud_update_competitive_scoreboard",
            &[
                ("competitivescores", Value::Table(rows)),
                ("selfindex", Value::Num(1.0)),
                ("bWasDemoJump", Value::Bool(false)),
            ],
        );
    }
    // The game type the game runs: the HUD's Turned and Grief checks read
    // `ui_gametype` (BO2's zombie hides his score and d-pad area only in
    // Turned). Then his team again, now that the checks see it.
    let mode_changed =
        !new.gametype.is_empty() && old.is_none_or(|o| o.gametype != new.gametype);
    if mode_changed {
        let group = hks_t6::host::mode_group(&host.values.borrow().tables, &new.gametype);
        let mut v = host.values.borrow_mut();
        for k in ["ui_gametype", "g_gametype"] {
            v.dvars.insert(k.to_owned(), new.gametype.clone());
        }
        if let Some(g) = group {
            v.dvars.insert("ui_zm_gamemodegroup".to_owned(), g);
        }
    }
    // His team (BO2's `hud_update_team_change`): a Turned zombie (axis)
    // has no d-pad area or score; Grief's scoreboard knows his side.
    if mode_changed || changed(|v| i64::from(v.team)) {
        let name = match new.team {
            1 => Some("TEAM_AXIS"),
            2 => Some("TEAM_ALLIES"),
            _ => None,
        };
        if let Some(team) = name.and_then(|n| host.field("CoD", n).as_num()) {
            host.values.borrow_mut().team = team as i32;
            host.root_event("hud_update_team_change", &[("team", Value::Num(team))]);
        }
    }
    if changed(|v| i64::from(v.weapon)) {
        host.values
            .borrow_mut()
            .weapon_types
            .insert(new.weapon_name.clone(), new.weapon_type);
        host.root_event(
            "hud_update_weapon",
            &[
                ("weapon", Value::str(&new.weapon_name)),
                ("inventorytype", Value::Num(new.weapon_inventory as f32)),
            ],
        );
        // A switch shows the new gun's name (not the first one he spawns with).
        if old.is_some() && !new.weapon_key.is_empty() {
            host.root_event(
                "hud_update_weapon_select",
                &[("weaponDisplayName", Value::str(&new.weapon_key))],
            );
        }
    }
    if changed(|v| {
        i64::from(v.clip)
            | (i64::from(v.stock) << 20)
            | (i64::from(v.clip_left.unwrap_or(-1)) << 40)
    }) {
        let mut fields = vec![
            ("ammoInClip", Value::Num(new.clip as f32)),
            ("ammoStock", Value::Num(new.stock as f32)),
            ("lowClip", Value::Bool(false)),
        ];
        if let Some(left) = new.clip_left {
            fields.push(("ammoInDWClip", Value::Num(left as f32)));
        }
        host.root_event("hud_update_ammo", &fields);
    }
    if old.is_none_or(|o| o.offhand != new.offhand) {
        // Each slot: { material = <the icon>, ammo = <count> }.
        let mut fields = Vec::new();
        for e in new.offhand.split(';') {
            let Some((w, n)) = e.split_once(':') else {
                continue;
            };
            let Some((slot, icon)) = offhand_slot(w) else {
                continue;
            };
            let n: f32 = n.trim().parse().unwrap_or(0.0);
            if n <= 0.0 {
                continue;
            }
            let t = hks_t6::value::Table::new_ref();
            t.borrow_mut().set_str("material", host.material(icon));
            t.borrow_mut().set_str("ammo", Value::Num(n));
            fields.push((slot, Value::Table(t)));
        }
        host.root_event("hud_update_offhand", &fields);
    }
    if old.is_none_or(|o| o.slots != new.slots || o.offhand != new.offhand) {
        // actionSlotData[slot] = { material, ammo, aspectRatio }.
        let data = hks_t6::value::Table::new_ref();
        for e in new.slots.split(';') {
            let Some((n, w)) = e.split_once(':') else {
                continue;
            };
            let (Ok(n), Some(icon)) = (n.trim().parse::<f32>(), slot_icon(w)) else {
                continue;
            };
            let ammo: f32 = new
                .offhand
                .split(';')
                .find_map(|o| {
                    o.split_once(':')
                        .filter(|(k, _)| *k == w)
                        .and_then(|(_, c)| c.trim().parse().ok())
                })
                .unwrap_or(0.0);
            let t = hks_t6::value::Table::new_ref();
            t.borrow_mut().set_str("material", host.material(icon));
            t.borrow_mut().set_str("ammo", Value::Num(ammo));
            t.borrow_mut().set_str("aspectRatio", Value::Num(1.0));
            data.borrow_mut().set(Value::Num(n), Value::Table(t));
        }
        host.root_event(
            "hud_update_actionslots",
            &[("actionSlotData", Value::Table(data))],
        );
    }
    if old.is_none_or(|o| o.icons != new.icons) {
        field_events(host, old.map(|o| o.icons.as_str()), &new.icons);
    }
    if old.is_none_or(|o| o.worldcf != new.worldcf) {
        field_events(host, old.map(|o| o.worldcf.as_str()), &new.worldcf);
    }
}

/// A client field's own event (`perk_juggernaut`, ...) for each that
/// changed from `old` to `new` (`field:value` lists); one gone from the
/// list is back to 0.
fn field_events(host: &mut Host, old: Option<&str>, new: &str) {
    let now = icon_fields(new);
    if let Some(o) = old {
        for (f, _) in icon_fields(o) {
            if !now.iter().any(|(g, _)| *g == f) {
                host.root_event(f, &[("newValue", Value::Num(0.0))]);
            }
        }
    }
    for (f, v) in now {
        let was = old.and_then(|o| {
            icon_fields(o)
                .into_iter()
                .find(|(g, _)| *g == f)
                .map(|(_, x)| x)
        });
        if was != Some(v) {
            host.root_event(
                f,
                &[
                    ("newValue", Value::Num(v)),
                    ("oldValue", Value::Num(was.unwrap_or(0.0))),
                ],
            );
        }
    }
}

/// BO2's button token (`BUTTON_LUI_PRIMARY`, the `CoD.buttonStrings`) as
/// the pad button `frame::bo2_pad_glyph` names.
fn lui_button(token: &str) -> Option<&'static str> {
    Some(match token.strip_prefix("BUTTON_LUI_")? {
        "PRIMARY" => "south",
        "SECONDARY" => "east",
        "ALT1" => "west",
        "ALT2" => "north",
        "SELECT" => "select",
        "START" => "start",
        "SHOULDERL" => "lb",
        "SHOULDERR" => "rb",
        "LEFT_TRIGGER" => "lt",
        "RIGHT_TRIGGER" => "rt",
        "LEFT_STICK" | "LEFT_STICK_UP" => "ls",
        "RIGHT_STICK" => "rs",
        "DPAD_ALL" => "dpad_all",
        "DPAD_UD" => "dpad_ud",
        "DPAD_RL" | "DPAD_LR" => "dpad_rl",
        "DPAD_U" | "DPAD_UP" => "dpad_up",
        "DPAD_D" | "DPAD_DOWN" => "dpad_down",
        "DPAD_L" | "DPAD_LEFT" => "dpad_left",
        "DPAD_R" | "DPAD_RIGHT" => "dpad_right",
        _ => return None,
    })
}

/// `plain`, but with a pad's style BO2's button tokens become its button
/// pictures (the BO2 font draws them).
fn plain_with(text: &str, pad: Option<frame::PromptStyle>) -> String {
    let mut out = String::with_capacity(text.len());
    let mut it = text.chars().peekable();
    while let Some(c) = it.next() {
        if c == '^' && it.peek().is_some_and(char::is_ascii_digit) {
            it.next();
            continue;
        }
        if c == '^' && it.peek() == Some(&'B') {
            it.next();
            let mut token = String::new();
            for d in it.by_ref() {
                if d == '^' {
                    break;
                }
                token.push(d);
            }
            if let Some(glyph) = pad
                .and_then(|style| lui_button(&token).and_then(|b| frame::bo2_pad_glyph(b, style)))
                .or_else(|| frame::bo2_cycle_glyph(&token))
            {
                out.push_str(glyph);
            }
            continue;
        }
        out.push(c);
    }
    out.trim().to_owned()
}

#[allow(clippy::too_many_arguments)]
fn lui_hud(
    mut commands: Commands,
    mut slot: NonSendMut<LuiHudSlot>,
    (ui, front_ui, screen, mut loading): (
        Option<Res<assets::T6Ui>>,
        Option<Res<assets::T6ZmFrontend>>,
        Res<frame::AppScreen>,
        Option<ResMut<assets::LoadingScreen>>,
    ),
    fonts: Option<Res<Bo2Fonts>>,
    icons: Option<Res<assets::T6HudIcons>>,
    (mut image_assets, mut globes, globe_nodes): (
        ResMut<Assets<Image>>,
        ResMut<Assets<crate::bo2_globe::GlobeMaterial>>,
        Query<&bevy::ui_render::prelude::MaterialNode<crate::bo2_globe::GlobeMaterial>>,
    ),
    presented: Option<Res<PresentedSnapshot>>,
    local: Option<Res<LocalPresentClient>>,
    (window, time, mp_front, identity): (
        Query<&Window, With<PrimaryWindow>>,
        Res<Time>,
        Option<Res<crate::lui_frontend::Bo2mpFrontend>>,
        Option<Res<frame::LaunchIdentity>>,
    ),
    (keys, weapons, actions): (
        Option<Res<frame::HudInputView>>,
        Option<Res<assets::PreparedWeapons>>,
        Option<Res<net::ClientActionInput>>,
    ),
    mut io: LuiIo,
    roots: Query<Entity, With<LuiRoot>>,
    mut nodes: Query<(Entity, &mut LuiNode, &mut Node, &mut ZIndex)>,
    (mut image_nodes, mut xforms): (Query<&mut ImageNode>, Query<&mut UiTransform>),
    (children, computed): (
        Query<&Children>,
        Query<(&ComputedNode, &UiGlobalTransform, &InheritedVisibility)>,
    ),
    (mut pictures, mut logged): (Local<HashMap<String, Handle<Image>>>, Local<usize>),
) {
    if !lui_enabled() {
        return;
    }
    // bo2 merge: the multiplayer driver (`lui_hud`) has this frame: this
    // one's root goes; the shared menu state is left to that one.
    if !crate::lui_hud::zombies_turn(mp_front.is_some(), identity.as_deref()) {
        for e in &roots {
            commands.entity(e).try_despawn();
        }
        frame::set_bo2_game(false);
        return;
    }
    // bo2 merge: Zombies' LUI answers hold this frame.
    hks_t6::lui::set_multiplayer(false);
    let values = match (presented.as_deref(), local.as_deref()) {
        (Some(p), Some(l)) => read_values(p, l),
        _ => None,
    };
    // The held weapon's names from the weapon table.
    let values = values.map(|mut v| {
        if let Some(w) = weapons.as_deref() {
            v.weapon_name =
                w.0.name_of(v.weapon as u32)
                    .trim_end_matches("_mp")
                    .to_owned();
            v.weapon_key =
                w.0.display_name_key_of(v.weapon as u32)
                    .unwrap_or_default()
                    .to_owned();
            v.weapon_inventory =
                w.0.facts_of(v.weapon as u32).map_or(0, |f| f.inventory_type);
            v.weapon_type =
                w.0.facts_of(v.weapon as u32).map_or(0, |f| f.t6_weap_type);
        }
        v
    });
    // A Black Ops II game runs while its HUD has values: the pad's aim
    // assist is BO2's there.
    frame::set_bo2_game(values.is_some());
    // bo2zm M4: out of a game, on the menu screen, a game started on BO2's
    // own front end shows it (its scripts and pictures read at the start).
    let front = values.is_none()
        && frame::bo2_frontend().is_some()
        && matches!(
            *screen,
            frame::AppScreen::MainMenu | frame::AppScreen::Loading
        );
    let ui: Option<&assets::T6Ui> = if front {
        front_ui.as_deref().map(|f| &f.ui)
    } else {
        ui.as_deref()
    };
    let icons: Option<&assets::T6HudIcons> = if front {
        front_ui.as_deref().map(|f| &f.icons)
    } else {
        icons.as_deref()
    };
    let values = if front {
        Some(Values::default())
    } else {
        values
    };
    let (Some(ui), Some(values)) = (ui, values) else {
        for e in &roots {
            commands.entity(e).despawn();
        }
        let idle = frame::LuiMenus::default();
        if *io.menus != idle {
            *io.menus = idle;
        }
        if io.paused.0 && !io.cheat.open {
            io.paused.0 = false;
        }
        return;
    };
    // (Re)build when the map's scripts arrive.
    if slot
        .0
        .as_ref()
        .is_none_or(|h| h.built_from != ui.scripts.len() || h.frontend != front)
        && !ui.scripts.is_empty()
    {
        let t = std::time::Instant::now();
        // The front end hands over to the game: keep its lobby.
        let slot = &mut *slot;
        if !front && let Some(old) = slot.0.as_mut().filter(|h| h.frontend) {
            let filter = old
                .host
                .field("CoD", "PlaylistCategoryFilter")
                .as_str()
                .map(str::to_owned);
            let v = old.host.values.borrow();
            let state = FrontState {
                playlist: v.playlist,
                game_modes: v.game_modes.clone(),
                session_modes: v.session_modes.clone(),
                filter,
            };
            drop(v);
            slot.1 = Some(state);
        }
        let mut hud = build(ui, front);
        // Back from a game: the lobby it started from.
        if front && let Some(st) = slot.1.take() {
            {
                let mut v = hud.host.values.borrow_mut();
                v.playlist = st.playlist;
                v.game_modes = st.game_modes;
                v.session_modes = st.session_modes;
            }
            if let Some(f) = &st.filter {
                hud.host
                    .set_global_field("CoD", "PlaylistCategoryFilter", Value::str(f));
            }
            let modes = hud.host.values.borrow().game_modes.clone();
            hud.return_menu = Some(if modes.contains(&0.0) {
                "PublicGameLobby"
            } else if modes.contains(&1.0) {
                "PrivateOnlineGameLobby"
            } else {
                "MainLobby"
            });
        }
        diag::info!(
            Ui,
            "bo2zm lui: {} scripts, base + {} in {:.0} ms; {} script errors; missing modules {:?}",
            ui.scripts.len(),
            if front {
                "T6.Main (front end)"
            } else {
                "T6.HUD"
            },
            t.elapsed().as_secs_f64() * 1000.0,
            hud.host.errors.len(),
            hud.host.missing.borrow()
        );
        slot.0 = Some(hud);
        pictures.clear();
        // The old screen goes (the front end's, when the game's HUD takes
        // over); drawing starts next frame on a new root, never this one's.
        if !roots.is_empty() {
            for e in &roots {
                commands.entity(e).despawn();
            }
            return;
        }
    }
    let Some(hud) = slot.0.as_mut() else {
        return;
    };
    // Font metrics for the scripts' text measure, once the fonts load.
    if hud.metrics.borrow().is_empty()
        && let Some(f) = fonts.as_deref()
    {
        *hud.metrics.borrow_mut() = f.advances();
    }
    // The keys his commands are bound to (the d-pad's key prompts).
    if let Some(k) = keys.as_deref() {
        let mut v = hud.host.values.borrow_mut();
        if v.binds.len() != k.binding_keys_all.len()
            || k.binding_keys_all
                .iter()
                .any(|(c, keys)| v.binds.get(c) != Some(keys))
        {
            v.binds = k
                .binding_keys_all
                .iter()
                .map(|(c, keys)| (c.clone(), keys.clone()))
                .collect();
        }
    }
    // IW4L_HEADLESS=1: no window. The scripts still run, on BO2's own
    // 1280x720 screen with no pointer, so their errors and engine calls
    // still reach the log; nothing is drawn.
    let headless = frame::Headless::requested();
    let (ww, wh, pointer) = match window.single() {
        Ok(win) => (win.width(), win.height(), win.cursor_position()),
        Err(_) if headless => (1280.0, 720.0, None),
        Err(_) => return,
    };
    let aspect = ww / wh.max(1.0);
    let scale = wh / 720.0;
    if (hud.aspect - aspect).abs() > 1e-3 {
        hud.aspect = aspect;
        hud.host.root(aspect);
    }
    if !hud.opened {
        hud.opened = true;
        hud.opened_at = time.elapsed_secs_f64() * 1000.0;
        // Test aid: BO2ZM_LUI_FIRST_MENU=<menu> opens that menu instead of
        // the HUD (the front end: `main`).
        let first = std::env::var("BO2ZM_LUI_FIRST_MENU")
            .unwrap_or_else(|_| if hud.frontend { "main" } else { "HUD" }.to_owned());
        hud.host.open_menu(&first);
        if hud.frontend {
            // The engine's `open_menu`: BO2's black cover (`main`) opens
            // the menu it names on it (after a game, its lobby).
            let menu = hud.return_menu.unwrap_or("MainMenu");
            hud.host.root_event(
                "open_menu",
                &[
                    ("controller", Value::Num(0.0)),
                    ("menuName", Value::str(menu)),
                ],
            );
        } else {
            hud.host
                .root_event("first_snapshot", &[("controller", Value::Num(0.0))]);
            hud.host.root_event("hud_update_refresh", &[]);
        }
    }
    // The map loads after Start Match: BO2's loading screen over the menus.
    if hud.frontend && *screen == frame::AppScreen::Loading && !hud.loading_open {
        hud.loading_open = true;
        hks_t6::host::set_loading_dvars(&mut hud.host.values.borrow_mut());
        hud.host.open_menu("Loading");
        // The engine starts it once its screen is up: the map name, place,
        // game mode and picture fade in one after another
        // (ui_mp_t6_hud_loading.lua). BO2 loads on another thread so its
        // screen keeps drawing; ours stops drawing once the map installs
        // (about 2.5 s in, measured load146), which froze the screen before
        // the picture's turn. So all four fades start together and the
        // whole screen is up within a second.
        hud.host.root_event("start_loading", &[]);
        for event in ["fade_in_map_location", "fade_in_gametype", "fade_in_map_image"] {
            hud.host.root_event(event, &[]);
        }
    }
    // (The loading screen can arrive a frame after the menu opens.)
    if hud.frontend
        && hud.loading_open
        && *screen == frame::AppScreen::Loading
        && let Some(loading) = loading.as_deref_mut()
    {
        loading.hold_once(std::time::Duration::from_millis(4500));
    }
    if hud.frontend {
        hud.host.solo_lobby_start();
    }
    if !hud.frontend {
        send_changes(&mut hud.host, hud.sent.as_ref(), &values);
    } else if hud.host.now_ms() - hud.lobby_update_at > 1000.0
        && let sig = lobby_sig(&hud.host)
        && (sig != hud.lobby_sig || std::mem::take(&mut hud.lobby_again))
    {
        // The engine's lobby updates (it sends them as the party changes,
        // not on a clock: each one rebuilds the player rows, restarting
        // their "Searching for player" dots): the lobbies refresh their
        // titles and Start Match from them.
        hud.lobby_update_at = hud.host.now_ms();
        if sig != hud.lobby_sig {
            if std::env::var_os("BO2ZM_LUI_LOG").is_some() {
                let v = hud.host.values.borrow();
                let game: Vec<_> = ["ui_mapname", "ui_zm_mapstartlocation", "ui_gametype"]
                    .iter()
                    .map(|k| hks_t6::host::dvar_get(&v.dvars, k))
                    .collect();
                diag::info!(Ui, "bo2zm lui: lobby update {game:?}");
            }
            hud.lobby_again = true;
            hud.lobby_sig = sig;
        }
        // Their members: him (the party panel's card).
        let members = hks_t6::host::lobby_members(&hud.host.values.borrow());
        hud.host.root_event(
            "gamelobby_update",
            &[
                ("controller", Value::Num(0.0)),
                ("members", members.clone()),
            ],
        );
        hud.host.root_event(
            "partylobby_update",
            &[("controller", Value::Num(0.0)), ("members", members)],
        );
    }
    // Test aid: BO2ZM_LUI_OPEN_MENU=<menu>: a second after the first menu
    // opens, the engine's `open_menu` event asks for that menu (BO2's front
    // end: its black cover opens `MainMenu` on it).
    if let Ok(menu) = std::env::var("BO2ZM_LUI_OPEN_MENU")
        && hud.host.now_ms() - hud.opened_at > 1000.0
        && !hud.open_menu_sent
    {
        hud.open_menu_sent = true;
        hud.host.root_event(
            "open_menu",
            &[
                ("controller", Value::Num(0.0)),
                ("menuName", Value::str(&menu)),
            ],
        );
    }
    // Test aid: BO2ZM_LUI_ASKED=1 logs, 8 s after the menus open, every
    // engine field the scripts asked for that nothing answers (counted).
    // (BO2ZM_LUI_ASKED=<ms>: that long after instead.)
    if let Some(after) = std::env::var("BO2ZM_LUI_ASKED").ok().map(|v| {
        v.parse::<f64>()
            .ok()
            .filter(|ms| *ms > 1.0)
            .unwrap_or(8000.0)
    }) && hud.host.now_ms() - hud.opened_at > after
        && !hud.asked_logged
    {
        hud.asked_logged = true;
        let asked = hud.host.asked.borrow();
        let list: Vec<String> = asked.iter().map(|(k, n)| format!("{k} x{n}")).collect();
        diag::info!(Ui, "bo2zm lui asked ({}): {}", list.len(), list.join(", "));
        diag::info!(
            Ui,
            "bo2zm lui missing widget setups: {:?}",
            hks_t6::lui::missing_methods()
        );
    }
    // BO2's scoreboard: open while Tab (+scores) is held and through the
    // game-over shot (the engine opens it; its scripts refresh it).
    let scores_down = actions
        .as_deref()
        .is_some_and(|a| a.client.kb.scores.active);
    // (With the shell on, its own scoreboard and Game Over show instead.)
    let want_board = !hud.frontend && (scores_down || values.game_over) && !crate::shell::zm_board::board_ours();
    if want_board != hud.board_open {
        hud.board_open = want_board;
        let event = if want_board {
            "open_scoreboard_menu"
        } else {
            "close_scoreboard_menu"
        };
        hud.host
            .root_event(event, &[("controller", Value::Num(0.0))]);
    }
    let hidden = values.hidden;
    hud.sent = Some(values);
    // BO2ZM_LUI_PAUSE_AT=<ms after the HUD opened>[,<button>@<ms>...]: open
    // the pause menu then, and press pad buttons later (test aid). `-`
    // first: no pause menu (the front end's menus are up already).
    if let Ok(plan) = std::env::var("BO2ZM_LUI_PAUSE_AT") {
        let now = hud.host.now_ms() - hud.opened_at;
        let mut steps = plan.split(',');
        let first = steps.next().map_or("", str::trim);
        let at = if first == "-" {
            Some(0.0)
        } else {
            first.parse::<f64>().ok()
        };
        if let Some(at) = at
            && now >= at
            && hud.test_step == 0
        {
            hud.test_step = 1;
            if first != "-" {
                hud.host
                    .root_event("open_ingame_menu", &[("menuName", Value::str("class"))]);
            }
        }
        for (i, step) in steps.enumerate() {
            let Some((button, at)) = step.split_once('@') else {
                continue;
            };
            if hud.test_step == i + 1 && at.trim().parse::<f64>().is_ok_and(|t| now >= t) {
                hud.test_step = i + 2;
                diag::info!(Ui, "bo2zm lui test: {button}");
                // `click:x:y` = the mouse there, pressed and let go (root
                // units); else a pad button.
                let mut c = button.trim().split(':');
                let kind = c.next();
                if kind == Some("key") {
                    // `key:F` = that letter key (a prompt's shortcut).
                    hud.host.key_shortcut(c.next().unwrap_or(""));
                } else if kind == Some("move") {
                    // `move:x:y` = the mouse there, no click.
                    let x: f32 = c.next().and_then(|v| v.parse().ok()).unwrap_or(0.0);
                    let y: f32 = c.next().and_then(|v| v.parse().ok()).unwrap_or(0.0);
                    hud.host.mouse("mousemove", x, y, "");
                } else if kind == Some("click") {
                    let x: f32 = c.next().and_then(|v| v.parse().ok()).unwrap_or(0.0);
                    let y: f32 = c.next().and_then(|v| v.parse().ok()).unwrap_or(0.0);
                    hud.host.mouse("mousemove", x, y, "");
                    hud.host.mouse("mousedown", x, y, "left");
                    hud.host.mouse("mouseup", x, y, "left");
                } else if let Some(b) = button.trim().strip_suffix('+') {
                    // `right+` presses and holds, `right-` lets go.
                    hud.host.button(b, true);
                } else if let Some(b) = button.trim().strip_suffix('-') {
                    hud.host.button(b, false);
                } else {
                    hud.host.button(button.trim(), true);
                    hud.host.button(button.trim(), false);
                }
            }
        }
    }
    // His keys and mouse, then what the menus asked of the engine. While
    // a bind row waits for a key, the key is the capture's; when it is
    // done the rows show the new keys (`key_bound`).
    let console_open = keys.as_deref().is_some_and(|k| k.console_open);
    let capturing = io.capture.command.is_some() || io.capture.consumed_input;
    if hud.binding && !capturing {
        hud.binding = false;
        hud.host.root_event("key_bound", &[]);
    }
    if !capturing {
        menu_input(hud, &io, console_open, pointer, scale);
    }
    // The shell's pause menu's answers (END GAME), as BO2's menus send them.
    if !hud.frontend {
        for (menu, response) in crate::shell::ingame::take_responses() {
            hud.host.push_call(hks_t6::host::EngineCall::MenuResponse(menu, response));
        }
    }
    engine_calls(hud, &mut io, local.as_deref());
    public_match(hud, &mut io);
    private_party(hud, &mut io);
    let monitors = io.monitors.iter().count().max(1);
    {
        let mut v = hud.host.values.borrow_mut();
        v.dvars
            .insert("r_monitorCount".to_owned(), monitors.to_string());
        v.dvars
            .entry("r_monitor".to_owned())
            .or_insert_with(|| "0".to_owned());
    }
    settings_bridge(hud, &mut io.settings);
    // His last input: BO2's prompts follow it (`Engine.LastInput_Gamepad`,
    // `input_source_changed`: source 0 the pad, 1 keys and mouse).
    let pad_style = io
        .devices
        .as_deref()
        .filter(|d| d.pad_prompts)
        .map(|d| d.style);
    let gamepad = pad_style.is_some();
    if hud.host.values.borrow().gamepad != gamepad {
        hud.host.values.borrow_mut().gamepad = gamepad;
        hud.host.root_event(
            "input_source_changed",
            &[
                ("controller", Value::Num(0.0)),
                ("source", Value::Num(if gamepad { 0.0 } else { 1.0 })),
            ],
        );
    }
    let open = !hud.host.open_menus().is_empty() || io.cheat.open;
    let state = frame::LuiMenus { active: true, open };
    if *io.menus != state {
        *io.menus = state;
    }
    // A solo game pauses under its menus (Black Ops II's solo pause).
    let pause = (open || crate::shell::ingame::open()) && !hud.frontend && io.role.as_deref() == Some(&frame::RuntimeRole::Listen);
    if io.paused.0 != pause {
        io.paused.0 = pause;
        diag::info!(
            Ui,
            "bo2zm lui: game {}",
            if pause { "paused" } else { "resumed" }
        );
    }
    hud.host.frame(time.elapsed_secs_f64() * 1000.0);
    if hud.host.errors.len() > *logged {
        for e in &hud.host.errors[*logged..] {
            diag::warn!(Ui, "bo2zm lui script error: {e}");
        }
        *logged = hud.host.errors.len();
    }
    if headless {
        return;
    }
    // BO2ZM_LUI_DUMP=1: every element the scripts have, each time their
    // number changes (debugging aid).
    if std::env::var_os("BO2ZM_LUI_DUMP").is_some() {
        let all = hud.host.drawn();
        if all.len() != hud.dumped {
            hud.dumped = all.len();
            for d in &all {
                diag::info!(
                    Ui,
                    "bo2zm lui dump {}: {} {:?} rect {:?} alpha {:.2} rgb {:?} text {:?} align {} anchors {:?}",
                    d.id,
                    d.kind,
                    d.material,
                    d.rect,
                    d.alpha,
                    d.rgb,
                    d.text,
                    d.alignment,
                    d.anchors
                );
            }
        }
    }
    // His HUD hidden (the game over): only the scoreboard shows.
    let only = if hidden {
        Some(
            hud.host
                .menu_element_ids("Menu.Scoreboard")
                .unwrap_or_default(),
        )
    } else {
        None
    };
    let mut drawn: Vec<Drawn> = hud
        .host
        .drawn()
        .into_iter()
        .filter(|d| {
            only.as_ref().is_none_or(|ids| ids.contains(&d.id))
                && d.alpha > 0.004
                && (d.blur
                    || d.material.is_some()
                    || d.kind == "image"
                    || d.text.as_deref().is_some_and(|t| !t.trim().is_empty())
                    || ((d.kind == "dashes" || d.kind == "meter") && d.dashes.0 > 0))
        })
        .collect();
    // BO2's loading screen for Die Rise, Mob of the Dead, Buried and Origins
    // sets `ui_zm_useloadingmovie` and plays the map's loading movie on a
    // `black` (then `webm_720p`) picture (ui_mp_t6_hud_loading.lua). Until
    // the movies play here, the map's own loading picture stands in for
    // them (it drew as a plain white sheet).
    if hud.loading_open {
        let v = hud.host.values.borrow();
        let dvar = |k: &str| hks_t6::host::dvar_get(&v.dvars, k).unwrap_or_default();
        let movie = dvar("ui_zm_useloadingmovie");
        if !movie.is_empty() && movie != "0" {
            let (map, loc, mode) = (dvar("ui_mapname"), dvar("ui_zm_mapstartlocation"), dvar("ui_gametype"));
            let still = [
                format!("loadscreen_{map}_{mode}_{loc}"),
                format!("loadscreen_{map}_{mode}"),
                format!("loadscreen_{map}"),
            ]
            .into_iter()
            .find(|w| icons.is_some_and(|i| i.0.iter().any(|(n, _)| n == w)));
            for d in drawn.iter_mut() {
                if matches!(d.material.as_deref(), Some("black" | "webm_720p")) {
                    d.material = still.clone().or(d.material.take());
                }
            }
        }
    }
    // The engine's build number, small at the top right of every menu
    // screen (no script draws it; the loading screen has none).
    if hud.frontend && !hud.loading_open {
        drawn.push(Drawn {
            id: usize::MAX,
            kind: "text",
            rect: VERSION_RECT,
            alpha: 1.0,
            rgb: [1.0, 1.0, 1.0],
            material: None,
            text: Some(VERSION_TEXT.to_owned()),
            font: None,
            alignment: 3,
            anchors: (false, true),
            z_rot: 0.0,
            x_rot: 0.0,
            y_rot: 0.0,
            dashes: (0, 0, 0),
            dash_pitch: 0.0,
            tiles: 0.0,
            shader: [[0.0; 4]; 4],
            blur: false,
            behind: false,
            clip: None,
        });
    }

    let root = match roots.iter().next() {
        Some(r) => r,
        None => {
            let root = commands
                .spawn((
                    LuiRoot,
                    // The front end draws over the menu and loading screens.
                    if hud.frontend {
                        UiLayer::Overlay
                    } else {
                        UiLayer::Hud
                    },
                    UiLayerVisibility,
                    Node {
                        position_type: PositionType::Absolute,
                        width: Val::Percent(100.0),
                        height: Val::Percent(100.0),
                        ..default()
                    },
                ))
                .id();
            if hud.frontend {
                // Above the engine's own loading screen.
                commands.entity(root).insert(GlobalZIndex(1_000_000));
            }
            root
        }
    };
    let mut existing: HashMap<usize, Entity> = HashMap::new();
    for (e, n, ..) in &nodes {
        existing.insert(n.id, e);
    }
    let mut keep: std::collections::HashSet<usize> = std::collections::HashSet::new();
    // BO2's popup blur smears the menu under it until its words are gone
    // (Theater under Select Film); the veil below only darkens, so words
    // behind the top blur are not drawn.
    let last_blur = drawn.iter().rposition(|d| d.blur && d.material.is_none());
    for (order, d) in drawn.iter().enumerate() {
        if last_blur.is_some_and(|b| order < b) && d.text.as_deref().is_some_and(|t| !t.trim().is_empty()) {
            continue;
        }
        let [x0, y0, x1, y1] = d.rect;
        // Clipped by a stencil ancestor (setUseStencil): wholly outside its
        // box is not drawn; a picture across its edge is cropped below.
        let crop = d.clip.map(|c| {
            [
                x0.min(x1).max(c[0]),
                y0.min(y1).max(c[1]),
                x0.max(x1).min(c[2]),
                y0.max(y1).min(c[3]),
            ]
        });
        if crop.is_some_and(|v| v[2] <= v[0] || v[3] <= v[1]) {
            continue;
        }
        // The scripts' colours are linear (BO2 draws into an sRGB target:
        // the chalk's 0.21 red shows as a mid blood red).
        let color = Color::linear_rgba(d.rgb[0], d.rgb[1], d.rgb[2], d.alpha.clamp(0.0, 1.0));
        // The engine's plain `black` is black whatever its tint (as a block
        // of its colour it drew white).
        let color = if d.material.as_deref() == Some("black") {
            Color::linear_rgba(0.0, 0.0, 0.0, color.alpha())
        } else {
            color
        };
        let z = ZIndex(order as i32 + 1);
        if d.blur && d.material.is_none() {
            // A popup's blur of what is behind it: a dark veil over the
            // whole screen (BO2 blurs and darkens everything under the
            // popup, not just the popup's own box).
            keep.insert(d.id);
            let node = Node {
                position_type: PositionType::Absolute,
                left: Val::Px(0.0),
                top: Val::Px(0.0),
                width: Val::Percent(100.0),
                height: Val::Percent(100.0),
                ..default()
            };
            let veil = Color::linear_rgba(0.0, 0.0, 0.0, BLUR_VEIL * d.alpha.clamp(0.0, 1.0));
            let shows = "blur".to_owned();
            if let Some(&e) = existing.get(&d.id)
                && let Ok((_, n, mut nn, mut zi)) = nodes.get_mut(e)
                && n.shows == shows
            {
                *nn = node;
                *zi = z;
                commands.entity(e).insert(BackgroundColor(veil));
                continue;
            }
            if let Some(&e) = existing.get(&d.id) {
                commands.entity(e).despawn();
            }
            let e = commands
                .spawn((
                    LuiNode {
                        id: d.id,
                        shows,
                        alpha: veil.alpha(),
                    },
                    node,
                    BackgroundColor(veil),
                    z,
                ))
                .id();
            commands.entity(root).add_child(e);
        } else if d.kind == "dashes" || d.kind == "meter" {
            // A slider's bar: `count` dashes across its rectangle, the lit
            // ones filled (BO2's engine draws them). The voice level meter
            // (Settings' Level Indicator) is the same with square dashes.
            keep.insert(d.id);
            let (count, lit, _) = d.dashes;
            // Dashes one pitch apart, as the real ones sit (measured): a
            // slider's 5 units in from its bar's left and 5 above its
            // middle; the meter's centred across its box, also 5 up.
            let pitch = d.dash_pitch * scale;
            let inset = if d.kind == "meter" {
                (((x1 - x0).abs() * scale - pitch * count as f32) / 2.0).max(0.0)
            } else {
                5.0 * scale
            };
            let node = Node {
                position_type: PositionType::Absolute,
                left: Val::Px(x0 * scale + inset),
                top: Val::Px((y0 - 5.0) * scale),
                width: Val::Px(pitch * count as f32),
                height: Val::Px((y1 - y0).abs() * scale),
                align_items: AlignItems::Center,
                ..default()
            };
            let shows = format!(
                "dashes {count} {lit} {:.2} {:.2} {:.2} {:.2}",
                d.rgb[0], d.rgb[1], d.rgb[2], d.alpha
            );
            if let Some(&e) = existing.get(&d.id)
                && let Ok((_, n, mut nn, mut zi)) = nodes.get_mut(e)
                && n.shows == shows
            {
                *nn = node;
                *zi = z;
                continue;
            }
            if let Some(&e) = existing.get(&d.id) {
                commands.entity(e).despawn();
            }
            let e = commands
                .spawn((
                    LuiNode {
                        id: d.id,
                        shows,
                        alpha: color.alpha(),
                    },
                    node,
                    z,
                ))
                .with_children(|c| {
                    // As BO2 draws them (measured, 720 high): 11 tall, 5
                    // wide, 8 apart. Lit: grey, its top 3 rows lighter and
                    // its bottom 3 a little; unlit: a faint outline. Not
                    // tinted by the bar's colour (the real ones look the
                    // same focused or not). The colours are the real
                    // screen's (94, 66, 76 of 255) through our output's
                    // curve (about a 1.2 power), so they come out the same.
                    // The level meter's are 11 square, 14 apart.
                    let a = color.alpha();
                    let unit = scale;
                    let wide = if d.kind == "meter" { 11.0 } else { 5.0 } * unit;
                    let grey = |v: f32| BackgroundColor(Color::linear_rgba(v, v, v, a));
                    for i in 0..count {
                        let dash = Node {
                            width: Val::Px(wide),
                            height: Val::Px(11.0 * unit),
                            margin: UiRect::right(Val::Px(pitch - wide)),
                            flex_direction: FlexDirection::Column,
                            justify_content: JustifyContent::SpaceBetween,
                            flex_shrink: 0.0,
                            ..default()
                        };
                        if i < lit {
                            c.spawn((dash, grey(0.33))).with_children(|d| {
                                let band = || Node {
                                    width: Val::Percent(100.0),
                                    height: Val::Px(3.0 * unit),
                                    ..default()
                                };
                                d.spawn((band(), grey(0.44)));
                                d.spawn((band(), grey(0.37)));
                            });
                        } else {
                            c.spawn((
                                Node {
                                    border: UiRect::all(Val::Px(unit.max(1.0))),
                                    ..dash
                                },
                                BorderColor::all(Color::linear_rgba(1.0, 1.0, 1.0, 0.05 * a)),
                            ));
                        }
                    }
                })
                .id();
            commands.entity(root).add_child(e);
        } else if d.kind == "globe" {
            // bo2zm M4: the Zombies globe, by its own shader (its mesh and
            // day map are both globe_map_zm, the material's picture; turned
            // by shader vector 2, revealed by vector 0).
            let Some(map) = d
                .material
                .as_deref()
                .and_then(|m| picture(&mut pictures, icons, &mut image_assets, m))
            else {
                continue;
            };
            let (mesh, day) = (map.clone(), map);
            keep.insert(d.id);
            let node = Node {
                position_type: PositionType::Absolute,
                left: Val::Px(x0 * scale),
                top: Val::Px(y0 * scale),
                width: Val::Px((x1 - x0).abs() * scale),
                height: Val::Px((y1 - y0).abs() * scale),
                ..default()
            };
            let params = crate::bo2_globe::GlobeParams {
                v0: Vec4::from_array(d.shader[0]),
                v2: Vec4::from_array(d.shader[2]),
                color: Vec4::new(d.rgb[0], d.rgb[1], d.rgb[2], d.alpha.clamp(0.0, 1.0)),
                blur: Vec4::new(
                    if last_blur.is_some_and(|b| order < b) {
                        BLUR_UNITS * 2.0 / (x1 - x0).abs().max(1.0)
                    } else {
                        0.0
                    },
                    0.0,
                    0.0,
                    0.0,
                ),
            };
            let shows = "globe".to_owned();
            if let Some(&e) = existing.get(&d.id)
                && let Ok((_, n, mut nn, mut zi)) = nodes.get_mut(e)
                && n.shows == shows
            {
                *nn = node;
                *zi = z;
                if let Ok(m) = globe_nodes.get(e)
                    && let Some(mut mat) = globes.get_mut(&m.0)
                    && mat.params != params
                {
                    mat.params = params;
                }
                continue;
            }
            if let Some(&e) = existing.get(&d.id) {
                commands.entity(e).despawn();
            }
            let e = commands
                .spawn((
                    LuiNode {
                        id: d.id,
                        shows,
                        alpha: color.alpha(),
                    },
                    node,
                    crate::bo2_globe::globe_node(&mut globes, params, mesh, day),
                    z,
                ))
                .id();
            commands.entity(root).add_child(e);
        } else if d.kind == "image"
            && d.material.as_deref().is_none_or(|m| {
                m == "white" || picture(&mut pictures, icons, &mut image_assets, m).is_none()
            })
        {
            // A picture with no material (or the engine's plain `white`) is
            // a block of its colour: BO2's dim behind its menus.
            keep.insert(d.id);
            // BO2ZM_LUI_LOG: each picture a menu asks for that the front end
            // lacks, once (it shows as a block there).
            if let Some(mat) = d.material.as_ref().filter(|m| *m != "white")
                && std::env::var_os("BO2ZM_LUI_LOG").is_some()
            {
                static MISSED: std::sync::Mutex<Vec<String>> = std::sync::Mutex::new(Vec::new());
                if let Ok(mut m) = MISSED.lock()
                    && !m.contains(mat)
                {
                    m.push(mat.clone());
                    diag::info!(Ui, "bo2zm lui no picture `{mat}` for #{}", d.id);
                }
            }
            let node = Node {
                position_type: PositionType::Absolute,
                left: Val::Px(x0 * scale),
                top: Val::Px(y0 * scale),
                width: Val::Px((x1 - x0).abs() * scale),
                height: Val::Px((y1 - y0).abs() * scale),
                ..default()
            };
            let shows = "block".to_owned();
            if let Some(&e) = existing.get(&d.id)
                && let Ok((_, n, mut nn, mut zi)) = nodes.get_mut(e)
                && n.shows == shows
            {
                *nn = node;
                *zi = z;
                commands.entity(e).insert(BackgroundColor(color));
                continue;
            }
            if let Some(&e) = existing.get(&d.id) {
                commands.entity(e).despawn();
            }
            let e = commands
                .spawn((
                    LuiNode {
                        id: d.id,
                        shows,
                        alpha: color.alpha(),
                    },
                    node,
                    BackgroundColor(color),
                    z,
                ))
                .id();
            commands.entity(root).add_child(e);
        } else if let Some(mat) = &d.material {
            // BO2ZM_LUI_FORCEMAT=<id>:<material> draws that element with
            // another picture (debugging aid).
            let forced = std::env::var("BO2ZM_LUI_FORCEMAT")
                .ok()
                .and_then(|v| {
                    v.split_once(':')
                        .map(|(i, m)| (i.parse::<usize>().ok(), m.to_owned()))
                })
                .filter(|(i, _)| *i == Some(d.id))
                .map(|(_, m)| m);
            let mat = forced.as_ref().unwrap_or(mat);
            // Under the top popup's blur: the picture smeared (BO2 blurs the
            // whole menu behind a popup; the Options menu's logo and sky).
            let behind = last_blur.is_some_and(|b| order < b);
            let size = ((x1 - x0).abs(), (y1 - y0).abs());
            let found = if behind {
                blurred_picture(&mut pictures, icons, &mut image_assets, mat, size)
            } else {
                picture(&mut pictures, icons, &mut image_assets, mat).map(|h| (h, mat.clone(), Vec2::ZERO))
            };
            let Some((handle, key, pad)) = found else {
                continue;
            };
            // A smeared picture's box takes in the border it smears into.
            let (px, py) = ((x1 - x0) * pad.x, (y1 - y0) * pad.y);
            let [x0, y0, x1, y1] = [x0 - px, y0 - py, x1 + px, y1 + py];
            keep.insert(d.id);
            // Cropped to its clip: the visible part of the box, and the
            // same part of the picture (unturned pictures only).
            let (bx0, by0, bx1, by1) = (x0.min(x1), y0.min(y1), x0.max(x1), y0.max(y1));
            let (vis, part) = match crop {
                Some(v) if d.z_rot.abs() < 0.01 && v != [bx0, by0, bx1, by1] => {
                    let size = image_assets
                        .get(&handle)
                        .map_or(Vec2::ONE, |i| i.size().as_vec2());
                    let (bw, bh) = ((bx1 - bx0).max(1e-3), (by1 - by0).max(1e-3));
                    let part = bevy::math::Rect::new(
                        (v[0] - bx0) / bw * size.x,
                        (v[1] - by0) / bh * size.y,
                        (v[2] - bx0) / bw * size.x,
                        (v[3] - by0) / bh * size.y,
                    );
                    (v, Some(part))
                }
                _ => ([x0, y0, x1, y1], None),
            };
            // A loader (`lui_loader_32`, the lobby's "Searching for player"
            // spinner) is a strip of frames its shader steps through, one
            // turn a second, started at shader vector 0's x: one frame. The
            // 32 one holds 21 square frames squeezed into 512 pixels.
            let part = if mat.starts_with("lui_loader")
                && let Some(size) = image_assets.get(&handle).map(|i| i.size().as_vec2())
                && size.x >= size.y * 2.0
            {
                let frames = if mat == "lui_loader_32" { 21.0 } else { (size.x / size.y).floor() };
                let wide = size.x / frames;
                let turn = hud.host.now_ms() / 1000.0 + f64::from(d.shader[0][0]);
                let frame = ((turn.fract() * f64::from(frames)).floor() as f32).min(frames - 1.0);
                Some(bevy::math::Rect::new(frame * wide, 0.0, (frame + 1.0) * wide, size.y))
            } else {
                part
            };
            let node = Node {
                position_type: PositionType::Absolute,
                left: Val::Px(vis[0] * scale),
                top: Val::Px(vis[1] * scale),
                width: Val::Px((vis[2] - vis[0]).abs() * scale),
                height: Val::Px((vis[3] - vis[1]).abs() * scale),
                ..default()
            };
            // setZRot: the picture turned (BO2's right bracket is the left
            // one turned 180 degrees).
            let turn = UiTransform {
                rotation: Rot2::degrees(-d.z_rot),
                ..default()
            };
            let shows = format!("image {key}");
            if std::env::var("BO2ZM_LUI_CHAIN")
                .ok()
                .and_then(|v| v.parse::<usize>().ok())
                == Some(d.id)
            {
                diag::info!(
                    Ui,
                    "bo2zm lui chain #{}: {:?}",
                    d.id,
                    hud.host.chain_of(d.id)
                );
            }
            if std::env::var("BO2ZM_LUI_WATCH")
                .ok()
                .and_then(|v| v.parse::<usize>().ok())
                == Some(d.id)
            {
                diag::info!(
                    Ui,
                    "bo2zm lui watch #{} at {:.0} ms: rect {:?} colour {:?} entity {:?} z {} computed {:?}",
                    d.id,
                    hud.host.now_ms(),
                    d.rect,
                    color,
                    existing.get(&d.id),
                    order,
                    existing
                        .get(&d.id)
                        .and_then(|e| computed.get(*e).ok())
                        .map(|(c, t, v)| (c.size(), t.translation, v.get()))
                );
            }
            if let Some(&e) = existing.get(&d.id)
                && let Ok((_, n, mut nn, mut zi)) = nodes.get_mut(e)
                && n.shows == shows
            {
                *nn = node;
                *zi = z;
                if let Ok(mut img) = image_nodes.get_mut(e) {
                    img.color = color;
                    if img.rect != part {
                        img.rect = part;
                    }
                }
                if let Ok(mut x) = xforms.get_mut(e) {
                    *x = turn;
                }
                continue;
            }
            if std::env::var_os("BO2ZM_LUI_LOG").is_some() {
                diag::info!(Ui, "bo2zm lui image #{} {shows} at {:?}", d.id, d.rect);
            }
            if let Some(&e) = existing.get(&d.id) {
                commands.entity(e).despawn();
            }
            let e = commands
                .spawn((
                    LuiNode {
                        id: d.id,
                        shows,
                        alpha: color.alpha(),
                    },
                    node,
                    // LUI stretches a picture over its whole box (a 64x64
                    // bracket in a 64x30 row is squeezed, not shrunk).
                    ImageNode {
                        image: handle,
                        color,
                        rect: part,
                        image_mode: bevy::ui::widget::NodeImageMode::Stretch,
                        ..default()
                    },
                    turn,
                    z,
                ))
                .id();
            commands.entity(root).add_child(e);
        } else if let (Some(text), Some(bo2)) = (&d.text, fonts.as_deref()) {
            keep.insert(d.id);
            let text = plain_with(text, pad_style);
            let font = font_name(d.font.as_deref().unwrap_or("normalFont"));
            let px = (y1 - y0).abs() * scale;
            let width = bo2.line_width(font, &text, px);
            // LUI.Alignment: 1 left, 2 centre, 3 right; none is centred in
            // its box (BO2's Options menu words, in a box wider than the
            // word, and MFTabManager's tab titles), except a box held to one
            // side only, drawn from that side's edge (the ammo count, held
            // right, grows left, clear of the pad's arrows beside it).
            let align = match (d.alignment, d.anchors) {
                (1..=3, _) => d.alignment,
                (_, (false, false) | (true, true)) => 2,
                (_, (false, true)) => 3,
                _ => 1,
            };
            // A line longer than its box goes onto more lines, broken at
            // spaces, as LUI wraps text to its width (descriptions). A text
            // with no alignment set keeps one line past its box (the lobby's
            // "Waiting for 3 more players.", in a box 100 wide).
            let box_w = (x1 - x0).abs() * scale;
            let lines = if (1..=3).contains(&d.alignment) && box_w > px * 2.0 && width > box_w + 1.0 {
                wrap_lines(bo2, font, &text, px, box_w)
            } else {
                vec![text.clone()]
            };
            let node = if lines.len() > 1 {
                Node {
                    position_type: PositionType::Absolute,
                    left: Val::Px(x0.min(x1) * scale),
                    top: Val::Px(y0 * scale),
                    width: Val::Px(box_w),
                    flex_direction: FlexDirection::Column,
                    ..default()
                }
            } else {
                let left = match align {
                    2 => (x0 + x1) * 0.5 * scale - width * 0.5,
                    3 => x1 * scale - width,
                    _ => x0 * scale,
                };
                Node {
                    position_type: PositionType::Absolute,
                    left: Val::Px(left),
                    top: Val::Px(y0 * scale),
                    ..default()
                }
            };
            let shows = format!(
                "text {text}|{font}|{px:.1}|{:.2}|{:.2}|{:.2}|{}",
                d.rgb[0],
                d.rgb[1],
                d.rgb[2],
                lines.len()
            );
            if let Some(&e) = existing.get(&d.id)
                && let Ok((_, mut n, mut nn, mut zi)) = nodes.get_mut(e)
                && n.shows == shows
            {
                *nn = node;
                *zi = z;
                // A fade: every letter's alpha.
                if (n.alpha - color.alpha()).abs() > 1e-3 {
                    n.alpha = color.alpha();
                    for g in children.iter_descendants(e) {
                        if let Ok(mut img) = image_nodes.get_mut(g) {
                            img.color.set_alpha(color.alpha());
                        }
                    }
                }
                continue;
            }
            if std::env::var_os("BO2ZM_LUI_LOG").is_some() {
                diag::info!(Ui, "bo2zm lui text #{} {shows} at {:?}", d.id, d.rect);
            }
            if let Some(&e) = existing.get(&d.id) {
                commands.entity(e).despawn();
            }
            let e = commands
                .spawn((
                    LuiNode {
                        id: d.id,
                        shows,
                        alpha: color.alpha(),
                    },
                    node,
                    z,
                ))
                .with_children(|c| {
                    if lines.len() > 1 {
                        let justify = match align {
                            2 => JustifyContent::Center,
                            3 => JustifyContent::FlexEnd,
                            _ => JustifyContent::FlexStart,
                        };
                        for line in &lines {
                            c.spawn(Node {
                                width: Val::Percent(100.0),
                                height: Val::Px(px),
                                justify_content: justify,
                                ..default()
                            })
                            .with_children(|l| {
                                bo2.spawn_line(l, font, &[(line.clone(), color)], px, 0.0);
                            });
                        }
                    } else {
                        bo2.spawn_line(c, font, &[(text.clone(), color)], px, 0.0);
                    }
                })
                .id();
            commands.entity(root).add_child(e);
        }
    }
    for (id, e) in existing {
        if !keep.contains(&id) {
            commands.entity(e).despawn();
        }
    }
}

/// What the front end's lobby updates depend on: the open menus and the
/// party (its size, team and mode settings).
fn lobby_sig(host: &Host) -> String {
    let v = host.values.borrow();
    let mut party: Vec<_> = v
        .dvars
        .iter()
        .filter(|(k, _)| k.starts_with("party") || k.starts_with("lobby"))
        .collect();
    // And the game it is set to: the engine sends an update when the map,
    // start or mode changes (Grief picked after its start: the lobby's
    // picture is the start's Grief one, `menu_zm_prison_zencounter_cellblock`).
    let game: Vec<_> = ["ui_mapname", "ui_zm_mapstartlocation", "ui_gametype"]
        .iter()
        .map(|k| hks_t6::host::dvar_get(&v.dvars, k))
        .collect();
    party.sort();
    // And who is in his relay lobby (bo2zm H28).
    format!(
        "{:?}|{party:?}|{game:?}|{}|{:?}|{}|{:?}|{}",
        host.open_menus(),
        v.team,
        v.game_modes,
        v.player_name,
        v.party,
        v.party_follower
    )
}

/// `text` broken at spaces into lines no wider than `width` pixels (a word
/// wider than that keeps a line to itself).
fn wrap_lines(bo2: &Bo2Fonts, font: &str, text: &str, px: f32, width: f32) -> Vec<String> {
    let mut lines = Vec::new();
    let mut line = String::new();
    for word in text.split(' ') {
        let candidate = if line.is_empty() {
            word.to_owned()
        } else {
            format!("{line} {word}")
        };
        if !line.is_empty() && bo2.line_width(font, &candidate, px) > width {
            lines.push(std::mem::take(&mut line));
            line = word.to_owned();
        } else {
            line = candidate;
        }
    }
    if !line.is_empty() {
        lines.push(line);
    }
    lines
}

/// How dark a popup's blur leaves the menu behind it (0 clear, 1 black).
const BLUR_VEIL: f32 = 0.6;
/// How far a popup's blur smears the menu behind it, in the menus'
/// 1280-wide units.
const BLUR_UNITS: f32 = 14.0;

/// A material's picture smeared as BO2's popup blur smears it, for a box
/// `size` units across: shrunk until one pixel is about a blur's width,
/// softened, and drawn stretched (made once per picture and strength).
/// Returns the image, the name it is kept under and how far its border
/// reaches past the box, as a share of the box.
fn blurred_picture(
    cache: &mut HashMap<String, Handle<Image>>,
    icons: Option<&assets::T6HudIcons>,
    images: &mut Assets<Image>,
    name: &str,
    size: (f32, f32),
) -> Option<(Handle<Image>, String, Vec2)> {
    let img = icons?.0.iter().find(|(k, _)| k == name)?.1.clone();
    let (w, h) = (img.texture_descriptor.size.width as usize, img.texture_descriptor.size.height as usize);
    let f = (blur_step(w, size.0) / 2).max(1);
    let pad = Vec2::new(4.0 / w.div_ceil(f).max(1) as f32, 4.0 / h.div_ceil(f).max(1) as f32);
    let step = blur_step(w, size.0);
    let key = format!("{name}#blur{step}");
    if let Some(h) = cache.get(&key) {
        return Some((h.clone(), key, pad));
    }
    let rgba = picture_rgba(&img)?;
    // Shrink by half the blur's width (alpha-weighted, so see-through
    // pixels do not darken the edges), then three soft passes.
    let (sw, sh) = (w.div_ceil(f).max(1), h.div_ceil(f).max(1));
    let mut small = vec![[0f32; 4]; sw * sh];
    for y in 0..h {
        for x in 0..w {
            let p = &rgba[(y * w + x) * 4..(y * w + x) * 4 + 4];
            let a = f32::from(p[3]) / 255.0;
            let c = &mut small[(y / f) * sw + x / f];
            for i in 0..3 {
                c[i] += f32::from(p[i]) / 255.0 * a;
            }
            c[3] += a;
        }
    }
    for (i, c) in small.iter_mut().enumerate() {
        let (sx, sy) = (i % sw, i / sw);
        let n = ((w - sx * f).min(f) * (h - sy * f).min(f)) as f32;
        for v in c.iter_mut() {
            *v /= n;
        }
    }
    // A border the smear spreads into, so a picture's box edge is soft too:
    // see-through, or the edge carried on for a picture solid to its edge
    // (a backdrop does not fade at the screen's edge).
    const PAD: usize = 4;
    let border: Vec<f32> = (0..sw)
        .flat_map(|x| [small[x][3], small[(sh - 1) * sw + x][3]])
        .chain((0..sh).flat_map(|y| [small[y * sw][3], small[y * sw + sw - 1][3]]))
        .collect();
    let solid = border.iter().sum::<f32>() / border.len().max(1) as f32 > 0.95;
    let (iw, ih) = (sw, sh);
    let (sw, sh) = (iw + PAD * 2, ih + PAD * 2);
    let mut small = {
        let inner = small;
        let mut out = vec![[0f32; 4]; sw * sh];
        for y in 0..sh {
            for x in 0..sw {
                let (ix, iy) = (x as i32 - PAD as i32, y as i32 - PAD as i32);
                let inside = (0..iw as i32).contains(&ix) && (0..ih as i32).contains(&iy);
                if inside || solid {
                    let (cx, cy) = (ix.clamp(0, iw as i32 - 1) as usize, iy.clamp(0, ih as i32 - 1) as usize);
                    out[y * sw + x] = inner[cy * iw + cx];
                }
            }
        }
        out
    };
    let soften = |px: &mut Vec<[f32; 4]>, horizontal: bool| {
        let src = px.clone();
        for y in 0..sh {
            for x in 0..sw {
                let mut acc = [0f32; 4];
                for (d, wt) in [(-1i32, 0.25f32), (0, 0.5), (1, 0.25)] {
                    let (nx, ny) = if horizontal {
                        ((x as i32 + d).clamp(0, sw as i32 - 1) as usize, y)
                    } else {
                        (x, (y as i32 + d).clamp(0, sh as i32 - 1) as usize)
                    };
                    let s = src[ny * sw + nx];
                    for i in 0..4 {
                        acc[i] += s[i] * wt;
                    }
                }
                px[y * sw + x] = acc;
            }
        }
    };
    for _ in 0..3 {
        soften(&mut small, true);
        soften(&mut small, false);
    }
    let mut out = vec![0u8; sw * sh * 4];
    for (i, c) in small.iter().enumerate() {
        let a = c[3].max(1e-6);
        for k in 0..3 {
            out[i * 4 + k] = ((c[k] / a).clamp(0.0, 1.0) * 255.0).round() as u8;
        }
        out[i * 4 + 3] = (c[3].clamp(0.0, 1.0) * 255.0).round() as u8;
    }
    use bevy::asset::RenderAssetUsages;
    use bevy::render::render_resource::{Extent3d, TextureDimension, TextureFormat};
    // Stored values shown as they are, as `picture` does.
    let small = Image::new(
        Extent3d { width: sw as u32, height: sh as u32, depth_or_array_layers: 1 },
        TextureDimension::D2,
        out,
        TextureFormat::Rgba8Unorm,
        RenderAssetUsages::default(),
    );
    let h = images.add(small);
    cache.insert(key.clone(), h.clone());
    Some((h, key, pad))
}

/// The popup blur's width in a picture's own pixels (`w` wide, drawn
/// `drawn` units across), in steps so a picture that grows or shrinks is
/// not remade every frame.
fn blur_step(w: usize, drawn: f32) -> usize {
    let r = BLUR_UNITS * w as f32 / drawn.max(1.0);
    [1usize, 2, 3, 4, 6, 8, 12, 16, 24, 32, 48, 64]
        .into_iter()
        .min_by(|a, b| (*a as f32 - r).abs().total_cmp(&(*b as f32 - r).abs()))
        .unwrap_or(1)
}

/// A picture's top level as plain RGBA bytes (its stored values).
fn picture_rgba(image: &Image) -> Option<Vec<u8>> {
    use bevy::render::render_resource::TextureFormat as F;
    let w = image.texture_descriptor.size.width as usize;
    let h = image.texture_descriptor.size.height as usize;
    let data = image.data.as_ref()?;
    let (block, decode): (usize, fn(&[u8], &mut [u8], usize)) = match image.texture_descriptor.format {
        F::Bc1RgbaUnormSrgb | F::Bc1RgbaUnorm => (8, bcdec_rs::bc1),
        F::Bc2RgbaUnormSrgb | F::Bc2RgbaUnorm => (16, bcdec_rs::bc2),
        F::Bc3RgbaUnormSrgb | F::Bc3RgbaUnorm => (16, bcdec_rs::bc3),
        F::Bc7RgbaUnormSrgb | F::Bc7RgbaUnorm => (16, bcdec_rs::bc7),
        F::Rgba8UnormSrgb | F::Rgba8Unorm => return data.get(..w * h * 4).map(<[u8]>::to_vec),
        _ => return None,
    };
    let mut rgba = vec![0u8; w * h * 4];
    let (bw, bh) = (w.div_ceil(4), h.div_ceil(4));
    let mut tile = [0u8; 64];
    for by in 0..bh {
        for bx in 0..bw {
            let at = (by * bw + bx) * block;
            decode(data.get(at..at + block)?, &mut tile, 16);
            for y in 0..4 {
                for x in 0..4 {
                    let (px, py) = (bx * 4 + x, by * 4 + y);
                    if px < w && py < h {
                        let d = (py * w + px) * 4;
                        let s = (y * 4 + x) * 4;
                        rgba[d..d + 4].copy_from_slice(&tile[s..s + 4]);
                    }
                }
            }
        }
    }
    Some(rgba)
}

/// A material's picture as an image handle (made once).
fn picture(
    cache: &mut HashMap<String, Handle<Image>>,
    icons: Option<&assets::T6HudIcons>,
    images: &mut Assets<Image>,
    name: &str,
) -> Option<Handle<Image>> {
    if let Some(h) = cache.get(name) {
        return Some(h.clone());
    }
    let img = icons?.0.iter().find(|(k, _)| k == name)?.1.clone();
    // BO2ZM_LUI_PICPROFILE=<material>: its size, format and each pixel
    // column's strongest alpha (where a picture's ink sits; debugging aid).
    if std::env::var("BO2ZM_LUI_PICPROFILE").is_ok_and(|v| v == name) {
        let size = img.size();
        let cols: Vec<u8> = img.data.as_ref().map_or(Vec::new(), |d| {
            let (w, h) = (size.x as usize, size.y as usize);
            (0..w)
                .map(|x| {
                    (0..h)
                        .map(|y| d.get((y * w + x) * 4 + 3).copied().unwrap_or(0))
                        .max()
                        .unwrap_or(0)
                })
                .collect()
        });
        diag::info!(
            Ui,
            "bo2zm lui picture {name}: {size:?} {:?} column alpha {cols:?}",
            img.texture_descriptor.format
        );
    }
    // The 2D layer's target shows what is written as it is, so a picture
    // is read as stored (BO2's menus draw it so); read as sRGB it showed
    // about three times too dark (the Zombies menus' sky, globe and maps).
    let mut img = (*img).clone();
    use bevy::render::render_resource::TextureFormat as F;
    let raw = match img.texture_descriptor.format {
        F::Rgba8UnormSrgb => Some(F::Rgba8Unorm),
        F::Bc1RgbaUnormSrgb => Some(F::Bc1RgbaUnorm),
        F::Bc2RgbaUnormSrgb => Some(F::Bc2RgbaUnorm),
        F::Bc3RgbaUnormSrgb => Some(F::Bc3RgbaUnorm),
        F::Bc7RgbaUnormSrgb => Some(F::Bc7RgbaUnorm),
        _ => None,
    };
    if let Some(f) = raw {
        img.texture_descriptor.format = f;
    }
    // Menu pictures stop at their edges: wrapped, the loading screen's top
    // and bottom shades drew a dark line across the map picture.
    img.sampler = bevy::image::ImageSampler::linear();
    let h = images.add(img);
    cache.insert(name.to_owned(), h.clone());
    Some(h)
}

pub(crate) fn register_lui_hud_systems(app: &mut App) {
    app.insert_non_send(LuiHudSlot::default());
    app.add_systems(
        Update,
        lui_hud
            .after(crate::bo2_font::load_bo2_fonts)
            .in_set(ClientSet::Ui),
    );
}
