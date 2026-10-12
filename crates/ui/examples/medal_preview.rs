//! Black Ops' medal notification with real medal rows, saved as pictures:
//! `cargo run -p ui --example medal_preview -- <out dir>` (Black Ops'
//! text is read when `IW4L_GAMES` is set).

use bevy::prelude::*;

fn main() {
    let out = std::path::PathBuf::from(
        std::env::args()
            .nth(1)
            .unwrap_or_else(|| "medal_preview".to_owned()),
    );
    std::fs::create_dir_all(&out).expect("out dir");
    let mut app = App::new();
    app.add_plugins(DefaultPlugins.set(WindowPlugin {
        primary_window: Some(Window {
            resolution: (1920, 1080).into(),
            ..default()
        }),
        ..default()
    }));
    app.add_systems(Startup, |mut commands: Commands| {
        commands.spawn(Camera2d);
    });
    ui::hud_medals::preview::install(&mut app, out);
    app.run();
}
