use bevy::{
    prelude::*,
    window::{MonitorSelection, PresentMode, PrimaryWindow},
};
use std::collections::BTreeMap;

#[derive(Resource, Clone, Debug, Default)]
pub struct BindingView {
    pub chords: BTreeMap<u32, String>,
    pub listening: Option<u32>,
    pub revision: u64,
}

impl BindingView {
    pub fn chord(&self, command_id: u32) -> &str {
        self.chords
            .get(&command_id)
            .map(String::as_str)
            .unwrap_or("UNBOUND")
    }
}

#[derive(Resource)]
pub struct PresentModeOverride(pub PresentMode);

pub(crate) fn apply_window_settings(
    settings: Res<frame::GameSettings>,
    present_override: Option<Res<PresentModeOverride>>,
    mut windows: Query<&mut Window, With<PrimaryWindow>>,
    mut applied: Local<Option<(frame::DisplayResolution, bool, PresentMode)>>,
) {
    let present_mode = present_override.map_or_else(
        || {
            if settings.vsync {
                PresentMode::AutoVsync
            } else {
                PresentMode::AutoNoVsync
            }
        },
        |mode| mode.0,
    );
    let display = (settings.resolution, settings.fullscreen, present_mode);
    if applied.as_ref() == Some(&display) {
        return;
    }
    let Ok(mut window) = windows.single_mut() else {
        return;
    };
    window
        .resolution
        .set_physical_resolution(settings.resolution.width, settings.resolution.height);
<<<<<<< /home/claude/bo2-multiplayer-rebuild/crates/ui/src/options.rs
    window.mode = if settings.fullscreen {
=======
    // IW4L_WINDOW_HIDDEN=1 (a test run with no window) never goes full
    // screen: a borderless full-screen window would cover his game.
    let hidden_test = std::env::var("IW4L_WINDOW_HIDDEN").is_ok_and(|v| v == "1");
    window.mode = if settings.fullscreen && !hidden_test {
>>>>>>> /tmp/claude-0/zmhead/crates/ui/src/options.rs
        bevy::window::WindowMode::BorderlessFullscreen(MonitorSelection::Current)
    } else {
        bevy::window::WindowMode::Windowed
    };
    window.present_mode = present_mode;
    *applied = Some(display);
}
