//! bo2mp shell: a preview of every screen with stand-in data, saved as
//! pictures (`cargo run -p ui --example shell_preview`): the shell's look
//! can be checked without the games' files.

use std::path::PathBuf;

use bevy::prelude::*;
use bevy::render::view::screenshot::{Screenshot, save_to_disk};

use super::bridge::{self, Catalog, MapEntry, ModeEntry, PlayerCard};
use super::classes::{self, ClassBook, ClassItem, ClassView};
use super::pics::ShellPics;
use super::{Screen, Shell, Tab, zombies};

#[derive(Resource)]
struct Plan {
    out: PathBuf,
    step: usize,
    frames: u32,
    pictures: Vec<(String, Image)>,
}

fn stand_in_catalog() -> Catalog {
    let bo2 = ["Aftermath", "Cargo", "Carrier", "Drone", "Express", "Hijacked", "Meltdown", "Overflow", "Plaza", "Raid", "Slums", "Standoff", "Turbine", "Yemen", "Nuketown 2025"];
    let classic = [("mp_paris", "Resistance", "Modern Warfare 3"), ("mp_seatown", "Seatown", "Modern Warfare 3"), ("mp_array", "Array", "Black Ops"), ("mp_highrise", "Highrise", "Modern Warfare 2"), ("mp_rust", "Rust", "Modern Warfare 2"), ("mp_terminal", "Terminal", "Modern Warfare 2"), ("mp_crash", "Crash", "Call of Duty 4")];
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
            group: g.to_uppercase(),
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
        player: PlayerCard {
            name: "simon".to_owned(),
            level: 72,
            prestige: 3,
            xp_frac: 0.42,
            rank_picture: String::new(),
            record: [("KILLS", "1,204"), ("DEATHS", "876"), ("K/D RATIO", "1.37"), ("WINS", "58"), ("LOSSES", "41"), ("HEADSHOTS", "212"), ("SCORE", "301,550"), ("TIME PLAYED", "14h 05m")]
                .iter()
                .map(|(a, b)| ((*a).to_owned(), (*b).to_owned()))
                .collect(),
            can_prestige: false,
            xp: 412_300,
            xp_next: 425_000,
        },
    }
}

fn stand_in_book() -> ClassBook {
    let mut items = Vec::new();
    let mut add = |index: i32, name: &str, slot: &str, group: &str, cost: i32, level: i32, image: &str, attachments: &[&str]| {
        items.push(ClassItem {
            index,
            reference: name.to_lowercase().replace(' ', "_"),
            name: name.to_owned(),
            image: image.to_owned(),
            desc: format!("{name}: a stand-in description of what it does."),
            cost,
            group: group.to_owned(),
            slot: slot.to_owned(),
            level,
            locked: level > 30,
            attachments: attachments.iter().enumerate().map(|(k, a)| (k as i32 + 1, (*a).to_owned(), String::new())).collect(),
        });
    };
    let att = ["Reflex Sight", "Suppressor", "Extended Clip", "Fast Mag", "Grip", "ACOG"];
    for (i, n) in ["MP7", "PDW-57", "Vector K10", "MSMC", "Chicom CQB", "Skorpion EVO", "Peacekeeper"].iter().enumerate() {
        add(10 + i as i32, n, "primary", "weapon_smg", 1, 1 + i as i32 * 6, "stand_in_c", &att);
    }
    for (i, n) in ["MTAR", "Type 25", "SWAT-556", "FAL OSW", "M27", "SCAR-H", "SMR", "M8A1", "AN-94"].iter().enumerate() {
        add(30 + i as i32, n, "primary", "weapon_assault", 1, 1 + i as i32 * 5, "stand_in_c", &att);
    }
    for (i, n) in ["SVU-AS", "DSR 50", "Ballista", "XPR-50"].iter().enumerate() {
        add(50 + i as i32, n, "primary", "weapon_sniper", 1, 1, "stand_in_c", &att);
    }
    for (i, n) in ["Five-seven", "Tac-45", "B23R", "Executioner", "KAP-40"].iter().enumerate() {
        add(60 + i as i32, n, "secondary", "weapon_pistol", 1, 1, "stand_in_c", &att[..4]);
    }
    for (i, n) in ["Lightweight", "Hardline", "Blind Eye", "Flak Jacket", "Ghost"].iter().enumerate() {
        add(80 + i as i32, n, "specialty1", "specialty", 1, 1, "", &[]);
    }
    for (i, n) in ["Toughness", "Cold Blooded", "Fast Hands", "Hard Wired", "Scavenger"].iter().enumerate() {
        add(90 + i as i32, n, "specialty2", "specialty", 1, 1, "", &[]);
    }
    for (i, n) in ["Dexterity", "Extreme Conditioning", "Engineer", "Tactical Mask", "Dead Silence", "Awareness"].iter().enumerate() {
        add(100 + i as i32, n, "specialty3", "specialty", 1, 1, "", &[]);
    }
    for (i, n) in ["Frag", "Semtex", "Combat Axe", "Bouncing Betty", "C4", "Claymore"].iter().enumerate() {
        add(110 + i as i32, n, "primarygrenade", "weapon_grenade", 1, 1, "", &[]);
    }
    for (i, n) in ["Concussion", "Smoke Grenade", "Flashbang", "EMP Grenade", "Shock Charge"].iter().enumerate() {
        add(120 + i as i32, n, "specialgrenade", "weapon_grenade", 1, 1, "", &[]);
    }
    for (i, n) in ["Primary Gunfighter", "Perk 1 Greed", "Overkill", "Danger Close"].iter().enumerate() {
        add(130 + i as i32, n, "bonuscard", "bonuscard", 1, 1, "", &[]);
    }
    for (i, n) in ["UAV", "RC-XD", "Hunter Killer", "Care Package", "Counter-UAV", "Guardian"].iter().enumerate() {
        add(140 + i as i32, n, "killstreak", "killstreak", 0, 1, "", &[]);
    }
    let class = |n: i32, name: &str, primary: i32| {
        let mut slots = std::collections::HashMap::new();
        for (k, v) in [
            ("primary", primary),
            ("primaryattachment1", 1),
            ("primaryattachment2", 2),
            ("secondary", 60),
            ("specialty1", 80),
            ("specialty2", 90),
            ("specialty3", 100),
            ("primarygrenade", 110),
            ("primarygrenadecount", 1),
            ("specialgrenade", 120),
            ("specialgrenadecount", 2),
            ("bonuscard1", 130),
            ("killstreak1", 140),
            ("killstreak2", 142),
            ("killstreak3", 143),
        ] {
            slots.insert(k.to_owned(), v);
        }
        ClassView { n, name: name.to_owned(), slots, spent: 0 }
    };
    let mut b = ClassBook {
        classes: vec![class(0, "Custom 1", 10), class(1, "Custom 2", 30), class(2, "Custom 3", 50), class(3, "Custom 4", 31), class(4, "Custom 5", 50)],
        items,
    };
    let spends: Vec<i32> = b.classes.iter().map(|c| classes::spent(&b, &c.slots)).collect();
    for (c, s) in b.classes.iter_mut().zip(spends) {
        c.spent = s;
    }
    b
}

/// A stand-in Zombies game: the Black Ops maps on, and the Zombies
/// rebuild's screenshots from `BO2MP_ZOMBIES_SHOTS` (its `docs/screenshots`)
/// when that is set.
fn stand_in_zombies() -> zombies::Install {
    zombies::Install {
        exe: PathBuf::from("C:/Users/you/Documents/Call of Duty Claude/zombies/play/iw4l.exe"),
        dir: PathBuf::from("C:/Users/you/Documents/Call of Duty Claude/zombies/play"),
        extra: vec![PathBuf::from("C:/Users/you/Documents/zombies_declassified")],
        shots: std::env::var_os("BO2MP_ZOMBIES_SHOTS").map(PathBuf::from),
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
    let shots: &[(&str, fn(&mut Shell, &mut Commands))] = &[
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
        ("08_classes", |s, _| {
            s.screen = Screen::Classes;
            s.focus = 0;
        }),
        ("09_class_edit", |s, _| {
            s.screen = Screen::ClassEdit;
            s.class = 0;
            s.focus = 0;
        }),
        ("10_item_pick", |s, _| {
            s.screen = Screen::ItemPick;
            s.slot = 0;
            s.page = 1;
            s.focus = 2;
        }),
        ("11_item_pick_perk", |s, _| {
            s.screen = Screen::ItemPick;
            s.slot = 13;
            s.page = 0;
            s.focus = 1;
        }),
        ("12a_options", |s, _| {
            s.screen = Screen::Options;
            s.page = 0;
            s.focus = 3;
        }),
        ("12b_options_pad", |s, _| {
            s.page = 3;
            s.focus = 0;
        }),
        ("12c_barracks", |s, _| {
            s.screen = Screen::Barracks;
            s.focus = 0;
        }),
        // Zombies: the same screens in red.
        ("zm1_lobby", |s, _| {
            zombies::set_preview(Ok(stand_in_zombies()), None);
            s.screen = Screen::Zombies;
            s.zm_map = "zm_transit".to_owned();
            s.zm_start = 3;
            s.zm_round = 2;
            s.focus = 0;
        }),
        ("zm2_lobby_round", |s, _| {
            s.focus = 3;
        }),
        ("zm3_map_picker", |s, _| {
            s.screen = Screen::ZmMapPick;
            s.page = 0;
            s.focus = 3;
        }),
        ("zm4_map_picker_bo1", |s, _| {
            s.page = 1;
            s.focus = 4;
        }),
        ("zm5_start_picker", |s, _| {
            s.screen = Screen::ZmStartPick;
            s.page = 0;
            s.focus = 6;
        }),
        ("zm6_in_zombies", |s, _| {
            zombies::set_preview(Ok(stand_in_zombies()), Some("TOWN  ·  GRIEF on TRANZIT"));
            s.screen = Screen::Zombies;
            s.focus = 0;
        }),
        ("zm7_not_installed", |s, _| {
            zombies::set_preview(
                Err("The Zombies game is not built yet. It is looked for at C:\\Users\\you\\Documents\\Call of Duty Claude\\zombies\\play\\iw4l.exe. Run run-bo2.cmd again: it builds Zombies (DaffyDabz/bo2-zombies-rebuild) into the project's zombies folder.".to_owned()),
                None,
            );
            s.zm_map = "zm_prison".to_owned();
            s.zm_start = 0;
            s.zm_round = 0;
            s.focus = 0;
        }),
        ("zm8_main_after", |s, _| {
            zombies::set_preview(Ok(stand_in_zombies()), None);
            s.screen = Screen::Main;
            s.focus = 1;
        }),
        ("12_loading", |_, c| {
            c.remove_resource::<crate::lui_frontend::Bo2mpFrontend>();
            c.insert_resource(session::Bo2mpMatchSetup { map: "mp_terminal".to_owned(), gametype: "gun".to_owned(), ..default() });
            c.insert_resource(assets::LoadingScreen::new(asset_transport::progress::LoadProgress::new(1), "mp_terminal", "gun"));
        }),
        ("13_ingame_class", |_, c| {
            c.remove_resource::<assets::LoadingScreen>();
            super::ingame::preview_open(2);
        }),
    ];
    // Each step: set it up, then a picture of it on the next turn.
    // (BO2MP_PREVIEW_ONLY=<prefix>: only the pictures named so are saved;
    // every step is still set up, in order.)
    let only = std::env::var("BO2MP_PREVIEW_ONLY").unwrap_or_default();
    let i = plan.step / 2;
    if plan.step % 2 == 0 && i < shots.len() && !shots[i].0.starts_with(only.as_str()) {
        (shots[i].1)(&mut shell, &mut commands);
        plan.step += 2;
        plan.frames = 39;
        return;
    }
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
    app.add_message::<frame::UiPlaySound>();
    super::register_shell(app);
    bridge::set_preview_catalog(stand_in_catalog());
    classes::set_preview_book(stand_in_book());
    app.insert_resource(crate::lui_frontend::Bo2mpFrontend { stats_path: PathBuf::new() });
    app.insert_resource(Plan { out, step: 0, frames: 0, pictures });
    zombies::set_preview(Ok(stand_in_zombies()), None);
    app.add_systems(Update, step);
}
