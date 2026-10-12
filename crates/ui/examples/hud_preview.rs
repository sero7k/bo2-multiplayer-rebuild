//! The match HUD's own kill feed, hit marker, kill banner and scoreboard
//! with stand-in data, saved as pictures:
//! `cargo run -p ui --example hud_preview -- <out dir>`; with `streaks`
//! after the dir, the modern HUD's bottom right corner (scorestreaks,
//! attachments, ammo) instead.

use bevy::prelude::*;

fn main() {
    let out = std::path::PathBuf::from(std::env::args().nth(1).unwrap_or_else(|| "hud_preview".to_owned()));
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
    if std::env::args().nth(2).as_deref() == Some("streaks") {
        ui::hud_modern::preview::install(&mut app, out);
    } else {
        ui::hud_match::preview::install(&mut app, out);
    }
    app.run();
}
