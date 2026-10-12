//! bo2mp shell: a preview of every screen with stand-in data, saved as
//! pictures (`cargo run -p ui --example shell_preview`): the shell's look
//! can be checked without the games' files.

use std::path::PathBuf;

use bevy::prelude::*;
use bevy::render::view::screenshot::{Screenshot, save_to_disk};

use super::bridge::{self, Catalog, MapEntry, ModeEntry, PlayerCard};
use super::pics::ShellPics;
use super::{Screen, Shell, Tab};

#[derive(Resource)]
struct Plan {
    out: PathBuf,
    step: usize,
    frames: u32,
    pictures: Vec<(String, Image)>,
}

fn stand_in_catalog() -> Catalog {
    let bo2 = ["Aftermath", "Cargo", "Carrier", "Drone", "Express", "Hijacked", "Meltdown", "Overflow", "Plaza", "Raid", "Slums", "Standoff", "Turbine", "Yemen", "Nuketown 2025"];
    let classic = [("mp_terminal", "Terminal", "Modern Warfare 2"), ("mp_highrise", "Highrise", "Modern Warfare 2"), ("mp_rust", "Rust", "Modern Warfare 2"), ("mp_paris", "Resistance", "Modern Warfare 3"), ("mp_seatown", "Seatown", "Modern Warfare 3"), ("mp_array", "Array", "Black Ops"), ("mp_crash", "Crash", "Call of Duty 4")];
    let mut maps: Vec<MapEntry> = bo2
        .iter()
        .map(|t| MapEntry {
            ui_name: format!("mp_{}", t.to_lowercase().replace(' ', "")),
            title: t.to_uppercase(),
            group: "BLACK OPS II".to_owned(),
            origin: "Black Ops II".to_owned(),
            pictures: vec!["stand_in_b".to_owned()],
        })
        .collect();
    for (k, (n, t, g)) in classic.iter().enumerate() {
        maps.push(MapEntry {
            ui_name: (*n).to_owned(),
            title: t.to_uppercase(),
            group: "CLASSIC MAPS".to_owned(),
            origin: (*g).to_owned(),
            pictures: vec![if k == 0 { "stand_in_a" } else { "stand_in_b" }.to_owned()],
        });
    }
    let modes = [
        ("tdm", "TEAM DEATHMATCH", "Work with your team to reach the score limit."),
        ("dm", "FREE-FOR-ALL", "Every player for themselves."),
        ("dom", "DOMINATION", "Capture and hold the flags."),
        ("sd", "SEARCH & DESTROY", "One life per round: plant or defuse the bomb."),
        ("gun", "GUN GAME", "A new gun with every kill."),
    ];
    Catalog {
        maps,
        modes: modes.iter().map(|(k, t, d)| ModeEntry { key: (*k).to_owned(), title: (*t).to_owned(), desc: (*d).to_owned() }).collect(),
        player: PlayerCard { name: "simon".to_owned(), level: 72, prestige: 3, xp_frac: 0.42, rank_picture: String::new() },
    }
}

fn step(
    mut commands: Commands,
    mut plan: ResMut<Plan>,
    mut shell: ResMut<Shell>,
    mut pics: ResMut<ShellPics>,
    mut images: ResMut<Assets<Image>>,
    mut exit: MessageWriter<AppExit>,
) {
    if !plan.pictures.is_empty() {
        for (name, img) in std::mem::take(&mut plan.pictures) {
            let h = images.add(img);
            pics.insert(&name, h);
        }
    }
    plan.frames += 1;
    if plan.frames < 40 {
        return;
    }
    plan.frames = 0;
    let shots: [(&str, fn(&mut Shell, &mut Commands)); 8] = [
        ("01_main", |s, _| *s = Shell::default()),
        ("02_multiplayer", |s, _| {
            s.screen = Screen::Multiplayer;
            s.focus = 0;
        }),
        ("03_lobby", |s, _| {
            s.screen = Screen::Lobby;
            s.tab = Tab::Lobby;
            s.map = "mp_terminal".to_owned();
            s.focus = 0;
        }),
        ("04_lobby_settings", |s, _| {
            s.tab = Tab::Settings;
            s.focus = 1;
        }),
        ("05_map_picker", |s, _| {
            s.screen = Screen::MapPick;
            s.page = 0;
            s.focus = 3;
        }),
        ("06_map_picker_classic", |s, _| {
            s.page = 1;
            s.focus = 0;
        }),
        ("07_mode_picker", |s, _| {
            s.screen = Screen::ModePick;
            s.focus = 2;
        }),
        ("08_loading", |_, c| {
            c.remove_resource::<crate::lui_frontend::Bo2mpFrontend>();
            c.insert_resource(session::Bo2mpMatchSetup { map: "mp_terminal".to_owned(), gametype: "gun".to_owned(), ..default() });
            c.insert_resource(assets::LoadingScreen::new(asset_transport::progress::LoadProgress::new(1), "mp_terminal", "gun"));
        }),
    ];
    // Each step: set it up, then a picture of it on the next turn.
    let i = plan.step / 2;
    if i >= shots.len() {
        exit.write(AppExit::Success);
        return;
    }
    let (name, set) = shots[i];
    if plan.step % 2 == 0 {
        set(&mut shell, &mut commands);
    } else {
        let path = plan.out.join(format!("{name}.png"));
        commands.spawn(Screenshot::primary_window()).observe(save_to_disk(path));
    }
    plan.step += 1;
}

/// The shell with stand-in data, saving a picture of each screen in `out`.
/// `pictures`: stand-ins for map pictures (`stand_in_a`, `stand_in_b`).
pub fn install(app: &mut App, pictures: Vec<(String, Image)>, out: PathBuf) {
    super::register_shell(app);
    bridge::set_preview_catalog(stand_in_catalog());
    app.insert_resource(crate::lui_frontend::Bo2mpFrontend { stats_path: PathBuf::new() });
    app.insert_resource(Plan { out, step: 0, frames: 0, pictures });
    app.add_systems(Update, step);
}
