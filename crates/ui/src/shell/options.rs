//! bo2mp shell: Options - graphics, sound, mouse and keyboard, controller -
//! as left / right selectors on his settings (`frame::GameSettings`, saved
//! and applied as every other change to them is). The same rows serve the
//! front end and the pause menu in a match.

use frame::{DisplayResolution, GameSettings};

use super::{Act, Row, Setting};

pub(crate) const TABS: [&str; 4] = ["GRAPHICS", "SOUND", "MOUSE & KEYBOARD", "CONTROLLER"];

const RESOLUTIONS: [(u32, u32); 8] = [(1280, 720), (1366, 768), (1600, 900), (1920, 1080), (2560, 1080), (2560, 1440), (3440, 1440), (3840, 2160)];

fn on_off(v: bool) -> String {
    if v { "ON" } else { "OFF" }.to_owned()
}

/// A row's label, what it shows, and a line about it.
fn row(label: &str, value: String, sub: &str, k: u8) -> Row {
    Row { label: label.to_owned(), sub: sub.to_owned(), value: Some(value), act: Act::Cycle(Setting::Option(k)) }
}

/// The rows of a tab (`k`: the row's number in the tab).
pub(crate) fn rows(tab: usize, s: &GameSettings) -> Vec<Row> {
    match tab {
        0 => vec![
            row("RESOLUTION", s.resolution.to_string(), "The window's size.", 0),
            row("DISPLAY MODE", if s.fullscreen { "FULLSCREEN" } else { "WINDOWED" }.to_owned(), "", 1),
            row("V-SYNC", on_off(s.vsync), "Lock the frame rate to the monitor's.", 2),
            row("FIELD OF VIEW", format!("{:.0}", s.fov), "How wide you see (65 is Black Ops II's).", 3),
            row("BRIGHTNESS", format!("{:+.0}", s.brightness * 100.0), "", 4),
            row("SHADOWS", on_off(s.shadows), "", 5),
            row("DEPTH OF FIELD", on_off(s.depth_of_field), "Blur behind the sights when aiming.", 6),
            row("BLOOM", on_off(s.bloom), "", 7),
            row("TEXTURE QUALITY", if s.bo2_fast { "PERFORMANCE" } else { "FULL" }.to_owned(), "Performance halves texture sizes on Black Ops II maps.", 8),
        ],
        1 => vec![row("MASTER VOLUME", format!("{:.0}", s.master_volume * 100.0), "", 0)],
        2 => vec![
            row("MOUSE SENSITIVITY", format!("{:.1}", s.sensitivity), "", 0),
            row("INVERT MOUSE", on_off(s.invert_mouse), "", 1),
            row("OMNI MOVEMENT", on_off(s.omni_movement), "Sprint in any direction (not Black Ops II's).", 2),
        ],
        _ => vec![
            row("LOOK SENSITIVITY", format!("{:.0}", s.pad_sensitivity), "1 is slow, 14 is the fastest.", 0),
            row("AIM SENSITIVITY", format!("{:.0}%", s.pad_ads_sensitivity * 100.0), "Turning while aiming down sights.", 1),
            row("INVERT LOOK", on_off(s.pad_invert), "", 2),
            row("VIBRATION", on_off(s.pad_vibration), "", 3),
            row("BUTTON LAYOUT", ["DEFAULT", "TACTICAL", "LEFTY", "NUMBER", "CHARLIE"][s.pad_layout.min(4) as usize].to_owned(), "", 4),
            row("STICK LAYOUT", ["DEFAULT", "SOUTHPAW", "LEGACY", "LEGACY SOUTHPAW"][s.pad_stick_layout.min(3) as usize].to_owned(), "", 5),
            row("LEFT STICK DEADZONE", format!("{:.0}%", s.pad_deadzone_left * 100.0), "", 6),
            row("RIGHT STICK DEADZONE", format!("{:.0}%", s.pad_deadzone_right * 100.0), "", 7),
        ],
    }
}

fn step(v: f32, by: i32, d: f32, lo: f32, hi: f32) -> f32 {
    ((v + by as f32 * d) / d).round().mul_add(d, 0.0).clamp(lo, hi)
}

/// Change a row by one step (`by` -1 or +1).
pub(crate) fn change(tab: usize, k: u8, by: i32, s: &mut GameSettings) {
    match (tab, k) {
        (0, 0) => {
            let at = RESOLUTIONS.iter().position(|(w, h)| *w == s.resolution.width && *h == s.resolution.height).unwrap_or(0) as i32;
            let (w, h) = RESOLUTIONS[(at + by).rem_euclid(RESOLUTIONS.len() as i32) as usize];
            s.resolution = DisplayResolution::new(w, h);
        }
        (0, 1) => s.fullscreen = !s.fullscreen,
        (0, 2) => s.vsync = !s.vsync,
        (0, 3) => s.fov = step(s.fov, by, 5.0, GameSettings::FOV_MIN, GameSettings::FOV_MAX),
        (0, 4) => s.brightness = step(s.brightness, by, 0.04, -0.2, 0.2),
        (0, 5) => s.shadows = !s.shadows,
        (0, 6) => s.depth_of_field = !s.depth_of_field,
        (0, 7) => s.bloom = !s.bloom,
        (0, 8) => s.bo2_fast = !s.bo2_fast,
        (1, 0) => s.master_volume = step(s.master_volume, by, 0.05, 0.0, 1.0),
        (2, 0) => s.sensitivity = step(s.sensitivity, by, 0.5, 0.5, 30.0),
        (2, 1) => s.invert_mouse = !s.invert_mouse,
        (2, 2) => s.omni_movement = !s.omni_movement,
        (3, 0) => s.pad_sensitivity = step(s.pad_sensitivity, by, 1.0, 1.0, 14.0),
        (3, 1) => s.pad_ads_sensitivity = step(s.pad_ads_sensitivity, by, 0.05, 0.5, 1.5),
        (3, 2) => s.pad_invert = !s.pad_invert,
        (3, 3) => s.pad_vibration = !s.pad_vibration,
        (3, 4) => s.pad_layout = ((s.pad_layout.min(4) as i32 + by).rem_euclid(5)) as u8,
        (3, 5) => s.pad_stick_layout = ((s.pad_stick_layout.min(3) as i32 + by).rem_euclid(4)) as u8,
        (3, 6) => s.pad_deadzone_left = step(s.pad_deadzone_left, by, 0.02, 0.0, 0.4),
        (3, 7) => s.pad_deadzone_right = step(s.pad_deadzone_right, by, 0.02, 0.0, 0.4),
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

pub(crate) fn sync_settings(settings: Option<bevy::prelude::ResMut<GameSettings>>) {
    let Some(mut s) = settings else { return };
    let queued = QUEUED.lock().map(|mut q| std::mem::take(&mut *q)).unwrap_or_default();
    for (tab, k, by) in queued {
        change(tab, k, by, &mut *s);
    }
    if let Ok(mut g) = NOW.lock()
        && g.as_ref().is_none_or(|old| old.revision != s.revision || old != &*s)
    {
        *g = Some((*s).clone());
    }
}
