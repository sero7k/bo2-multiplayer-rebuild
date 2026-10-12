//! The shell's screens with stand-in data, saved as pictures:
//! `cargo run -p ui --example shell_preview -- <out dir> [picture a.png] [picture b.png]`.

use bevy::asset::RenderAssetUsages;
use bevy::image::{CompressedImageFormats, ImageSampler, ImageType};
use bevy::prelude::*;

fn main() {
    let mut args = std::env::args().skip(1);
    let out = std::path::PathBuf::from(args.next().unwrap_or_else(|| "shell_preview".to_owned()));
    std::fs::create_dir_all(&out).expect("out dir");
    let mut pictures = Vec::new();
    for (name, path) in ["stand_in_a", "stand_in_b", "stand_in_c"].into_iter().zip(args) {
        let bytes = std::fs::read(&path).expect("picture");
        let img = Image::from_buffer(&bytes, ImageType::Extension("png"), CompressedImageFormats::NONE, true, ImageSampler::Default, RenderAssetUsages::default())
            .expect("png");
        pictures.push((name.to_owned(), img));
    }
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
    ui::shell::preview::install(&mut app, pictures, out);
    app.run();
}
