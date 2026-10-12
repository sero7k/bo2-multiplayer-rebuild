//! bo2mp shell: Options - graphics, sound, mouse and keyboard, controller,
//! account - as left / right selectors on his settings
//! (`frame::GameSettings`). The screens only queue a change; `sync_settings`
//! (the one system that writes them) carries it out, and the systems that
//! watch his settings apply it: the window (`ui::options`), shadows, depth
//! of field, bloom and the master volume (`console::user_settings`), the
//! brightness (`film_vision_view`), the music (`audio::music`), the mouse
//! and the pad (`console`), the vibration (`audio::rumble`); the settings
//! file (`settings.cfg`) is saved by `console::user_settings` whenever they
//! change. Unlock All lives in his stats file (`bridge`).
//!
//! The same rows serve the front end and the pause menu in a match (which
//! leaves ACCOUNT out: its rows need the front end's scripts).

use frame::{DisplayResolution, GameSettings};

use super::{Act, Row, Setting};

pub(crate) const TABS: [&str; 5] = ["GRAPHICS", "SOUND", "MOUSE & KEYBOARD", "CONTROLLER", "ACCOUNT"];

/// The tabs a match's pause menu shows (all but ACCOUNT).
pub(crate) const MATCH_TABS: usize = 4;

/// Unlock All (every item free in Create-a-Class), as his stats hold it.
pub(crate) static UNLOCK_ALL: std::sync::atomic::AtomicBool = std::sync::atomic::AtomicBool::new(false);

/// ACCOUNT's rows: his name (typed in), Unlock All.
pub(crate) const ACCOUNT_TAB: usize = 4;

pub(crate) const RESOLUTIONS: [(u32, u32); 8] = [(1280, 720), (1366, 768), (1600, 900), (1920, 1080), (2560, 1080), (2560, 1440), (3440, 1440), (3840, 2160)];

pub(crate) const PAD_LAYOUTS: [&str; 5] = ["DEFAULT", "TACTICAL", "LEFTY", "NUMBER", "CHARLIE"];
pub(crate) const STICK_LAYOUTS: [&str; 4] = ["DEFAULT", "SOUTHPAW", "LEGACY", "LEGACY SOUTHPAW"];

fn on_off(v: bool) -> String {
    if v { "ON" } else { "OFF" }.to_owned()
}

/// A selector row: its label, what it shows, and a line about it.
fn row(label: &str, value: String, sub: &str, k: u8) -> Row {
    Row { label: label.to_owned(), sub: sub.to_owned(), value: Some(value), act: Act::Cycle(Setting::Option(k)) }
}

/// A 0..1 level as a percentage.
pub(crate) fn percent(v: f32) -> String {
    format!("{:.0}%", v * 100.0)
}

/// The button layout's name (one he bound himself is CUSTOM).
pub(crate) fn pad_layout_name(layout: u8) -> String {
    if layout == GameSettings::PAD_LAYOUT_CUSTOM { "CUSTOM".to_owned() } else { PAD_LAYOUTS[layout.min(4) as usize].to_owned() }
}

/// The rows of a tab (`k`: the row's number in the tab).
pub(crate) fn rows(tab: usize, s: &GameSettings) -> Vec<Row> {
    match tab {
        0 => vec![
            row("RESOLUTION", s.resolution.to_string(), "The window's size (fullscreen fills the monitor at its own size).", 0),
            row("DISPLAY MODE", if s.fullscreen { "FULLSCREEN" } else { "WINDOWED" }.to_owned(), "Borderless fullscreen on the monitor the game is on, or a window.", 1),
            row("V-SYNC", on_off(s.vsync), "Lock the frame rate to the monitor's.", 2),
            row("FIELD OF VIEW", format!("{:.0}", s.fov), "How wide you see in a match (65 is Black Ops II's).", 3),
            row("BRIGHTNESS", format!("{:+.0}", s.brightness * 100.0), "Brighter or darker in a match (0 is the map's own).", 4),
            row("SHADOWS", on_off(s.shadows), "Shadow maps in a match.", 5),
            row("DEPTH OF FIELD", on_off(s.depth_of_field), "Blur behind the sights when aiming.", 6),
            row("BLOOM", on_off(s.bloom), "Glow around bright lights.", 7),
            row(
                "TEXTURE QUALITY",
                if s.bo2_fast { "PERFORMANCE" } else { "FULL" }.to_owned(),
                "Performance halves texture sizes and draws less far on Black Ops II maps (textures from the next map on).",
                8,
            ),
            row(
                "PLAYER MODELS",
                if s.map_game_bodies { "MAP'S GAME" } else { "BLACK OPS II" }.to_owned(),
                "On another game's map: that game's soldiers and animations, or Black Ops II's characters.",
                9,
            ),
        ],
        1 => vec![
            row("MASTER VOLUME", percent(s.master_volume), "Every sound in the game.", 0),
            row("MUSIC VOLUME", percent(s.music_volume), "Menu, lobby and match music.", 1),
            row("MUSIC IN MATCHES", on_off(s.match_music), "Quiet ambient music during matches.", 2),
        ],
        2 => vec![
            row("MOUSE SENSITIVITY", format!("{:.1}", s.sensitivity), "How fast the mouse turns you.", 0),
            row("INVERT MOUSE", on_off(s.invert_mouse), "Mouse up looks down.", 1),
            row("OMNI MOVEMENT", on_off(s.omni_movement), "Sprint in any direction (not Black Ops II's). From the next match on.", 2),
        ],
        3 => vec![
            row("LOOK SENSITIVITY", format!("{:.0}", s.pad_sensitivity), "1 is slow, 14 is the fastest.", 0),
            row("AIM SENSITIVITY", percent(s.pad_ads_sensitivity), "Turning while aiming down sights.", 1),
            row("INVERT LOOK", on_off(s.pad_invert), "Stick up looks down.", 2),
            row("VIBRATION", on_off(s.pad_vibration), "The controller rumbles.", 3),
            row("BUTTON LAYOUT", pad_layout_name(s.pad_layout), "Black Ops II's button layouts (CUSTOM: buttons you bound yourself).", 4),
            row("STICK LAYOUT", STICK_LAYOUTS[s.pad_stick_layout.min(3) as usize].to_owned(), "Which stick moves and which looks.", 5),
            row("LEFT STICK DEADZONE", percent(s.pad_deadzone_left), "How far the stick goes before it moves you.", 6),
            row("RIGHT STICK DEADZONE", percent(s.pad_deadzone_right), "How far the stick goes before it turns you.", 7),
        ],
        _ => vec![
            Row {
                label: "PLAYER NAME".to_owned(),
                sub: "Press Enter to type a new name (16 letters at most), Enter again to keep it.".to_owned(),
                value: Some(s.player_name.clone()),
                act: Act::TypeName,
            },
            row(
                "UNLOCK ALL",
                on_off(UNLOCK_ALL.load(std::sync::atomic::Ordering::Relaxed)),
                "Every weapon, attachment, perk and wildcard free in Create-a-Class, whatever your level.",
                1,
            ),
            Row {
                label: "FRIEND CODE".to_owned(),
                sub: "Give this to a friend so they can add you (FRIENDS on the main menu).".to_owned(),
                value: Some(GameSettings::friend_code_text(s.friend_code)),
                act: Act::Info,
            },
        ],
    }
}

/// `v` moved `by` steps of `d`, on the grid of `d`, kept in `lo..=hi`.
pub(crate) fn step(v: f32, by: i32, d: f32, lo: f32, hi: f32) -> f32 {
    ((v / d).round() + by as f32).mul_add(d, 0.0).clamp(lo, hi)
}

/// The resolution `by` steps from `now` in the list (one not in it starts
/// from the nearest).
pub(crate) fn next_resolution(now: DisplayResolution, by: i32) -> DisplayResolution {
    let pixels = |(w, h): (u32, u32)| i64::from(w) * i64::from(h);
    let mine = pixels((now.width, now.height));
    let at = RESOLUTIONS.iter().position(|(w, h)| *w == now.width && *h == now.height);
    let i = match at {
        Some(i) => (i as i32 + by).rem_euclid(RESOLUTIONS.len() as i32) as usize,
        None => {
            let near = RESOLUTIONS.iter().enumerate().min_by_key(|(_, r)| (pixels(**r) - mine).abs()).map_or(0, |(i, _)| i);
            if by > 0 && pixels(RESOLUTIONS[near]) <= mine {
                (near + 1) % RESOLUTIONS.len()
            } else if by < 0 && pixels(RESOLUTIONS[near]) >= mine {
                (near + RESOLUTIONS.len() - 1) % RESOLUTIONS.len()
            } else {
                near
            }
        }
    };
    let (w, h) = RESOLUTIONS[i];
    DisplayResolution::new(w, h)
}

/// Change a row by one step (`by` -1 or +1).
pub(crate) fn change(tab: usize, k: u8, by: i32, s: &mut GameSettings) {
    match (tab, k) {
        (0, 0) => s.resolution = next_resolution(s.resolution, by),
        (0, 1) => s.fullscreen = !s.fullscreen,
        (0, 2) => s.vsync = !s.vsync,
        (0, 3) => s.fov = step(s.fov, by, 5.0, GameSettings::FOV_MIN, GameSettings::FOV_MAX),
        (0, 4) => s.brightness = step(s.brightness, by, 0.04, -0.2, 0.2),
        (0, 5) => s.shadows = !s.shadows,
        (0, 6) => s.depth_of_field = !s.depth_of_field,
        (0, 7) => s.bloom = !s.bloom,
        (0, 8) => s.bo2_fast = !s.bo2_fast,
        (0, 9) => s.map_game_bodies = !s.map_game_bodies,
        (1, 0) => s.master_volume = step(s.master_volume, by, 0.05, 0.0, 1.0),
        (1, 1) => s.music_volume = step(s.music_volume, by, 0.05, 0.0, 1.0),
        (1, 2) => s.match_music = !s.match_music,
        (2, 0) => s.sensitivity = step(s.sensitivity, by, 0.5, 0.5, 30.0),
        (2, 1) => s.invert_mouse = !s.invert_mouse,
        (2, 2) => s.omni_movement = !s.omni_movement,
        (3, 0) => s.pad_sensitivity = step(s.pad_sensitivity, by, 1.0, 1.0, 14.0),
        (3, 1) => s.pad_ads_sensitivity = step(s.pad_ads_sensitivity, by, 0.05, 0.5, 1.5),
        (3, 2) => s.pad_invert = !s.pad_invert,
        (3, 3) => s.pad_vibration = !s.pad_vibration,
        // (From CUSTOM, right is DEFAULT and left CHARLIE.)
        (3, 4) => {
            let at = if s.pad_layout == GameSettings::PAD_LAYOUT_CUSTOM { if by > 0 { -1 } else { 0 } } else { i32::from(s.pad_layout.min(4)) };
            s.pad_layout = (at + by).rem_euclid(PAD_LAYOUTS.len() as i32) as u8;
        }
        (3, 5) => s.pad_stick_layout = ((s.pad_stick_layout.min(3) as i32 + by).rem_euclid(4)) as u8,
        (3, 6) => s.pad_deadzone_left = step(s.pad_deadzone_left, by, 0.02, 0.0, 0.4),
        (3, 7) => s.pad_deadzone_right = step(s.pad_deadzone_right, by, 0.02, 0.0, 0.4),
        (4, 1) => {
            let on = !UNLOCK_ALL.load(std::sync::atomic::Ordering::Relaxed);
            super::bridge::request(super::bridge::Request::UnlockAll(on));
            return;
        }
        _ => return,
    }
    s.sanitize();
    s.touch();
}

/// His settings as of this frame, and the changes the screens asked for
/// (carried out by `sync_settings`, the one system that writes them).
static NOW: std::sync::Mutex<Option<GameSettings>> = std::sync::Mutex::new(None);
static QUEUED: std::sync::Mutex<Vec<(usize, u8, i32)>> = std::sync::Mutex::new(Vec::new());

pub(crate) fn snapshot() -> GameSettings {
    NOW.lock().ok().and_then(|g| g.clone()).unwrap_or_default()
}

pub(crate) fn queue(tab: usize, k: u8, by: i32) {
    if let Ok(mut q) = QUEUED.lock() {
        q.push((tab, k, by));
    }
}

/// A new name he typed.
static NAME: std::sync::Mutex<Option<String>> = std::sync::Mutex::new(None);

pub(crate) fn set_name(name: String) {
    if let Ok(mut n) = NAME.lock() {
        *n = Some(name);
    }
}

pub(crate) fn sync_settings(settings: Option<bevy::prelude::ResMut<GameSettings>>, mut fast: bevy::prelude::Local<Option<bool>>) {
    let Some(mut s) = settings else { return };
    if let Some(name) = NAME.lock().ok().and_then(|mut n| n.take()) {
        s.player_name = name;
        s.sanitize();
        s.touch();
        diag::info!(Ui, "bo2mp shell: player name now `{}`", s.player_name);
    }
    // (Friends added or removed on the FRIENDS screen.)
    if super::friends::apply_edits(&mut s) {
        s.sanitize();
        s.touch();
    }
    let queued = QUEUED.lock().map(|mut q| std::mem::take(&mut *q)).unwrap_or_default();
    for (tab, k, by) in queued {
        change(tab, k, by, &mut s);
    }
    // (Texture quality: the Black Ops II renderer reads it as it draws and
    // loads; the settings file only sets it at start.)
    if *fast != Some(s.bo2_fast) {
        if fast.is_some() {
            asset_core::t6_set_fast(s.bo2_fast);
            diag::info!(Ui, "bo2mp shell: Black Ops II quality {}", if s.bo2_fast { "fast" } else { "full" });
        }
        *fast = Some(s.bo2_fast);
    }
    // (Player models: the body path reads it as it draws.)
    assets::set_map_game_bodies(s.map_game_bodies);
    // (Who he is to his friends, and whom he looks for, for the master.)
    net::set_friend_identity(s.friend_code, &s.player_name);
    net::set_friends(&s.friends);
    if let Ok(mut g) = NOW.lock()
        && g.as_ref().is_none_or(|old| old.revision != s.revision || old != &*s)
    {
        *g = Some((*s).clone());
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn value(tab: usize, k: usize, s: &GameSettings) -> String {
        rows(tab, s)[k].value.clone().unwrap_or_default()
    }

    /// Every selector row of every tab changes something (none is only
    /// drawn), and a step there and back leaves it as it was.
    #[test]
    fn every_row_works_and_steps_back() {
        for tab in 0..ACCOUNT_TAB {
            let base = GameSettings::default();
            for (k, r) in rows(tab, &base).iter().enumerate() {
                let Act::Cycle(Setting::Option(id)) = r.act else { panic!("{} is not a selector", r.label) };
                assert_eq!(id as usize, k, "{}: row number", r.label);
                // (A level at its top by default steps down instead.)
                let by = {
                    let mut t = base.clone();
                    change(tab, id, 1, &mut t);
                    if value(tab, k, &t) == value(tab, k, &base) { -1 } else { 1 }
                };
                let mut s = base.clone();
                change(tab, id, by, &mut s);
                assert_ne!(s, base, "{} does nothing", r.label);
                assert_ne!(value(tab, k, &s), value(tab, k, &base), "{} shows no change", r.label);
                assert!(s.revision > base.revision, "{} is not marked changed (not saved)", r.label);
                change(tab, id, -by, &mut s);
                let shown = |s: &GameSettings| rows(tab, s).into_iter().map(|r| r.value).collect::<Vec<_>>();
                assert_eq!(shown(&s), shown(&base), "{} does not step back", r.label);
            }
        }
    }

    #[test]
    fn values_read_his_settings() {
        let mut s = GameSettings::default();
        s.resolution = DisplayResolution::new(2560, 1440);
        s.fullscreen = true;
        s.fov = 90.0;
        s.brightness = -0.08;
        s.master_volume = 0.35;
        s.music_volume = 0.0;
        s.match_music = false;
        s.sensitivity = 7.5;
        s.pad_ads_sensitivity = 1.25;
        s.pad_deadzone_left = 0.2;
        s.pad_stick_layout = 1;
        s.player_name = "sero".to_owned();
        assert_eq!(value(0, 0, &s), "2560x1440");
        assert_eq!(value(0, 1, &s), "FULLSCREEN");
        assert_eq!(value(0, 3, &s), "90");
        assert_eq!(value(0, 4, &s), "-8");
        assert_eq!(value(1, 0, &s), "35%");
        assert_eq!(value(1, 1, &s), "0%");
        assert_eq!(value(1, 2, &s), "OFF");
        assert_eq!(value(2, 0, &s), "7.5");
        assert_eq!(value(3, 1, &s), "125%");
        assert_eq!(value(3, 5, &s), "SOUTHPAW");
        assert_eq!(value(3, 6, &s), "20%");
        assert_eq!(value(4, 0, &s), "sero");
        assert_eq!(rows(4, &s)[0].act, Act::TypeName);
    }

    #[test]
    fn steps_stay_in_range() {
        let mut s = GameSettings { master_volume: 1.0, ..GameSettings::default() };
        change(1, 0, 1, &mut s);
        assert_eq!(s.master_volume, 1.0);
        for _ in 0..40 {
            change(1, 0, -1, &mut s);
        }
        assert_eq!(s.master_volume, 0.0);
        for _ in 0..30 {
            change(0, 3, 1, &mut s);
            change(0, 4, 1, &mut s);
            change(3, 0, 1, &mut s);
        }
        assert_eq!(s.fov, GameSettings::FOV_MAX);
        assert!((s.brightness - 0.2).abs() < 1e-6);
        assert_eq!(s.pad_sensitivity, 14.0);
        // (Off the grid - an old file's 0.333 - lands back on it.)
        assert!((step(0.333, 1, 0.05, 0.0, 1.0) - 0.4).abs() < 1e-6);
        assert!((step(0.333, -1, 0.05, 0.0, 1.0) - 0.3).abs() < 1e-6);
    }

    #[test]
    fn resolutions_cycle_from_any() {
        assert_eq!(next_resolution(DisplayResolution::new(1920, 1080), 1), DisplayResolution::new(2560, 1080));
        assert_eq!(next_resolution(DisplayResolution::new(3840, 2160), 1), DisplayResolution::new(1280, 720));
        assert_eq!(next_resolution(DisplayResolution::new(1280, 720), -1), DisplayResolution::new(3840, 2160));
        // (His own 1680x1050, not listed: the next bigger / smaller.)
        assert_eq!(next_resolution(DisplayResolution::new(1680, 1050), 1), DisplayResolution::new(1920, 1080));
        assert_eq!(next_resolution(DisplayResolution::new(1680, 1050), -1), DisplayResolution::new(1600, 900));
    }

    #[test]
    fn custom_button_layout() {
        let mut s = GameSettings { pad_layout: GameSettings::PAD_LAYOUT_CUSTOM, ..GameSettings::default() };
        assert_eq!(value(3, 4, &s), "CUSTOM");
        change(3, 4, 1, &mut s);
        assert_eq!(s.pad_layout, 0);
        s.pad_layout = GameSettings::PAD_LAYOUT_CUSTOM;
        change(3, 4, -1, &mut s);
        assert_eq!(s.pad_layout, 4);
    }

    /// The ACCOUNT rows are not settings-file ones: the name is typed, and
    /// Unlock All goes to his stats (no settings change).
    #[test]
    fn account_rows() {
        let base = GameSettings::default();
        let mut s = base.clone();
        change(4, 0, 1, &mut s);
        assert_eq!(s, base);
        assert_eq!(MATCH_TABS, ACCOUNT_TAB);
    }
}
