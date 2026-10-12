//! bo2mp: Black Ops' in-game HUD over Black Ops II's game.
//!
//! In a multiplayer match the ammo corner (weapon name, bullets, stock,
//! grenades) and the score corner (faction icon, clock, the two score bars)
//! are drawn here in Black Ops' layout and with its own pictures, read from
//! Black Ops' IWDs. BO2's own widgets for the same things (`AmmoArea`,
//! `ScoreBottomLeft`) are still built but never drawn (`hks_t6::host`'s
//! `hide_for_bo1_hud`). Everything else (scorestreaks, minimap, kill feed,
//! scoreboard, killcam, menus) stays BO2's. `BO2_HUD=bo2` brings BO2's back.
//!
//! The layout is Black Ops' `weaponinfo`, `ammo_counter` and
//! `scorebars_team_*` menus (640x480, bottom corners), scaled to the screen.
//! A picture Black Ops' files do not have is drawn as a plain shape, and the
//! names of the HUD pictures its IWDs do hold are written to the log once.

use std::collections::HashMap;
use std::sync::mpsc::{Receiver, channel};

use asset_core::AssetNamespace;
use bevy::asset::RenderAssetUsages;
use bevy::prelude::*;
use bevy::render::render_resource::{Extent3d, TextureDimension, TextureFormat};
use bevy::window::PrimaryWindow;
use frame::ClientSet;
use net::{LocalPresentClient, PresentedSnapshot};

use crate::layers::{GameUiFont, UiLayer, UiLayerVisibility, game_text_font};

/// Black Ops' pictures the HUD uses.
const PICTURES: &[&str] = &[
    // The score bars and the faction box.
    "score_bar_bg",
    "score_bar_allies",
    "score_bar_opfor",
    "hud_faction_backing",
    "hud_faction_back_light",
    "hud_frame_faction_lines",
    "hud_frame_faction_fade",
    // Team icons.
    "faction_128_specops",
    "faction_128_spetsnaz",
    "faction_128_marines",
    "faction_128_nva",
    "faction_128_op40",
    "faction_128_tropas",
    // The ammo corner.
    "hud_frame_bars",
    "hud_frame_bars_fade",
    "hud_frame_black_back",
    "hud_frame_black_back_fade",
    "hud_frame_line",
    "ammo_counter_bullet",
    "ammo_counter_riflebullet",
    "ammo_counter_beltbullet",
    "ammo_counter_shotgunshell",
    "ammo_counter_rocket",
    "ammo_counter_single",
    // Grenades and equipment.
    "hud_grenadeicon",
    "hud_us_grenade",
    "hud_icon_sticky_grenade",
    "hud_sticky_grenade",
    "hud_hatchet",
    "hud_claymore",
    "hud_c4",
    "hud_us_flashgrenade",
    "hud_us_stungrenade",
    "hud_willy_pete",
    "hud_tact_insert",
    "hud_tactical_insertion",
];

/// Black Ops' factions for Black Ops II's (the announcer's pairing):
/// SEALs, FBI and ISA are Black Ops' SOG / Black Ops; the PMC, the Cordis
/// Die and the PLA are the Spetsnaz.
const ALLIES_ICON: &str = "faction_128_specops";
const AXIS_ICON: &str = "faction_128_spetsnaz";

/// Free-for-all game types (one score per player, no teams).
const FFA_TYPES: [&str; 5] = ["dm", "gun", "oic", "sas", "shrp"];

const WHITE: Color = Color::srgb(1.0, 1.0, 1.0);
const DIM: Color = Color::srgba(1.0, 1.0, 1.0, 0.75);
const LOW: Color = Color::srgb(0.95, 0.25, 0.18);
/// Black Ops' own team colours on the bars (friendly blue, enemy red).
const FRIENDLY: Color = Color::srgb(0.33, 0.72, 0.95);
const ENEMY: Color = Color::srgb(0.88, 0.27, 0.18);
const BAR_BACK: Color = Color::srgba(0.0, 0.0, 0.0, 0.55);

pub(crate) fn bo1_hud_enabled() -> bool {
    hks_t6::host::bo1_hud_on() && crate::lui_hud::lui_enabled()
}

#[derive(Component)]
struct Bo1HudRoot;

/// Black Ops' pictures: decoded off the main thread once, then kept.
#[derive(Default)]
struct Pictures {
    started: bool,
    waiting: Option<Receiver<Vec<(String, u32, u32, Vec<u8>)>>>,
    ready: HashMap<String, Handle<Image>>,
}

impl Pictures {
    fn get(&self, name: &str) -> Option<Handle<Image>> {
        self.ready.get(name).cloned()
    }
}

/// Decodes `PICTURES` from Black Ops' IWDs, and logs the HUD-looking
/// picture names those hold.
fn decode_pictures(games_root: std::path::PathBuf) -> Vec<(String, u32, u32, Vec<u8>)> {
    let trees = asset_transport::NamespaceTrees::discover(&asset_transport::GamesRoot(games_root));
    let Some(main) = trees.main_for(AssetNamespace::T5) else {
        diag::warn!(Ui, "bo1 hud: no Black Ops install under the games folder; plain shapes only");
        return Vec::new();
    };
    if let Ok(index) = asset_transport::IwdIndex::open(main) {
        let mut names: Vec<&str> = index
            .image_names()
            .filter(|n| {
                ["hud_", "score", "faction", "ammo_counter", "compass", "scorebar", "dpad"]
                    .iter()
                    .any(|k| n.contains(k))
            })
            .collect();
        names.sort_unstable();
        diag::info!(Ui, "bo1 hud: {} HUD pictures in Black Ops' IWDs: {}", names.len(), names.join(" "));
    }
    let mut out = Vec::new();
    let mut missing = Vec::new();
    for name in PICTURES {
        match asset_material::decode_ui_image_from_main(main, name) {
            Ok(Some((w, h, rgba))) => out.push(((*name).to_owned(), w, h, rgba)),
            Ok(None) => missing.push(*name),
            Err(e) => {
                diag::warn!(Ui, "bo1 hud: `{name}` did not decode: {e}");
                missing.push(*name);
            }
        }
    }
    diag::info!(
        Ui,
        "bo1 hud: {} of {} pictures from Black Ops; not found: {}",
        out.len(),
        PICTURES.len(),
        missing.join(" ")
    );
    out
}

fn load_pictures(
    pictures: &mut Pictures,
    identity: Option<&frame::LaunchIdentity>,
    images: &mut Assets<Image>,
) {
    if !pictures.started {
        let Some(identity) = identity else { return };
        pictures.started = true;
        let root = identity.games_root.clone();
        let (tx, rx) = channel();
        std::thread::Builder::new()
            .name("bo1-hud-pictures".into())
            .spawn(move || {
                let _ = tx.send(decode_pictures(root));
            })
            .ok();
        pictures.waiting = Some(rx);
    }
    let Some(rx) = &pictures.waiting else { return };
    let Ok(decoded) = rx.try_recv() else { return };
    for (name, w, h, rgba) in decoded {
        let image = Image::new(
            Extent3d {
                width: w,
                height: h,
                depth_or_array_layers: 1,
            },
            TextureDimension::D2,
            rgba,
            TextureFormat::Rgba8UnormSrgb,
            RenderAssetUsages::default(),
        );
        pictures.ready.insert(name, images.add(image));
    }
    pictures.waiting = None;
}

/// Black Ops' round picture for a gun (`ammo_counter` draws one per round
/// in the clip) and its width for a 1080-line screen (the height is 36).
fn bullet_of(weapon: &str) -> (&'static str, f32) {
    const SNIPERS: [&str; 6] = ["dsr50", "ballista", "svu", "as50", "l96a1", "wa2000"];
    const SHOTGUNS: [&str; 6] = ["870mcs", "saiga12", "ksg", "srm1216", "spas", "olympia"];
    const LAUNCHERS: [&str; 5] = ["smaw", "fhj18", "usrpg", "rpg", "m72_law"];
    const LMGS: [&str; 6] = ["mk48", "qbb95", "lsat", "hamr", "rpk", "m60"];
    let has = |list: &[&str]| list.iter().any(|w| weapon.starts_with(w));
    if has(&LAUNCHERS) {
        ("ammo_counter_rocket", 30.0)
    } else if has(&SNIPERS) {
        ("ammo_counter_riflebullet", 13.0)
    } else if has(&SHOTGUNS) {
        ("ammo_counter_shotgunshell", 15.0)
    } else if has(&LMGS) {
        ("ammo_counter_beltbullet", 8.0)
    } else {
        ("ammo_counter_bullet", 9.0)
    }
}

/// Black Ops' picture for a Black Ops II grenade or piece of equipment
/// (BO2's own when Black Ops had none like it), and whether it is lethal.
fn offhand_pictures(weapon: &str) -> Option<(bool, &'static [&'static str])> {
    Some(match weapon {
        "frag_grenade_mp" => (true, &["hud_grenadeicon", "hud_us_grenade"]),
        "sticky_grenade_mp" => (true, &["hud_icon_sticky_grenade", "hud_sticky_grenade"]),
        "hatchet_mp" => (true, &["hud_hatchet"]),
        "claymore_mp" => (true, &["hud_claymore", "hud_icon_claymore"]),
        "satchel_charge_mp" => (true, &["hud_c4", "hud_icon_satchelcharge"]),
        "bouncingbetty_mp" => (true, &["hud_bounce_betty"]),
        "flash_grenade_mp" => (false, &["hud_us_flashgrenade"]),
        "concussion_grenade_mp" => (false, &["hud_us_stungrenade"]),
        "willy_pete_mp" => (false, &["hud_willy_pete"]),
        "emp_grenade_mp" => (false, &["hud_empgrenade"]),
        "trophy_system_mp" => (false, &["hud_trophy_system"]),
        "tactical_insertion_mp" => (false, &["hud_tact_insert", "hud_tactical_insertion"]),
        "sensor_grenade_mp" => (false, &["hud_sensor_grenade"]),
        "proximity_grenade_mp" => (false, &["hud_proximity_grenade"]),
        _ => return None,
    })
}

#[derive(Clone, Debug, Default, PartialEq)]
struct Bo1Values {
    ffa: bool,
    axis: bool,
    yours: i32,
    enemy: i32,
    limit: i32,
    /// Seconds left, when the mode has a clock.
    timeleft: Option<i32>,
    clip: i32,
    clip_left: Option<i32>,
    stock: i32,
    weapon: String,
    weapon_label: String,
    /// (lethal?, the weapon, count).
    offhand: Vec<(bool, String, u32)>,
    /// The screen's height, and how many pictures are in.
    height: u32,
    pictures: usize,
}

fn clip_of(table: &[u8], weapon: i32, hand: usize) -> i32 {
    table
        .chunks_exact(12)
        .find(|row| i32::from_le_bytes([row[0], row[1], row[2], row[3]]) == weapon)
        .map_or(0, |row| {
            let o = 4 + hand.min(1) * 4;
            i32::from_le_bytes([row[o], row[o + 1], row[o + 2], row[o + 3]])
        })
}

fn read_values(
    presented: &PresentedSnapshot,
    local: &LocalPresentClient,
    weapons: Option<&assets::PreparedWeapons>,
    slot: Option<&crate::lui_hud::LuiHudSlot>,
) -> Option<Bo1Values> {
    let snap = presented.snapshot()?;
    let dvars = snap.meta.script_dvars(local.0);
    let mp = crate::lui_frontend::MatchValues::read(&dvars)?;
    // Gone with BO2's HUD: a killcam, the match's end, a menu he opened.
    if mp.hud_hidden || !mp.killcam.is_empty() {
        return None;
    }
    if mp.menu.split_once(':').is_some_and(|(_, m)| !m.trim().is_empty()) {
        return None;
    }
    let meta = snap.meta.for_client(local.0)?;
    let ps = presented.player(local.0);
    let ffa = FFA_TYPES.contains(&mp.gametype.as_str()) || mp.team == "free";
    let axis = mp.team == "axis";
    let (yours, enemy) = if ffa {
        // His score against the best of the others.
        let me = local.0.0;
        let best = snap
            .meta
            .clients
            .iter()
            .filter(|(c, _)| c.0 != me)
            .map(|(_, m)| m.score)
            .max()
            .unwrap_or(0);
        (meta.score, best)
    } else {
        let (allies, axis_score) = mp.scores.split_once(',').unwrap_or(("0", "0"));
        let (allies, axis_score) = (allies.trim().parse().unwrap_or(0), axis_score.trim().parse().unwrap_or(0));
        if axis { (axis_score, allies) } else { (allies, axis_score) }
    };
    let limit = mp
        .settings
        .split(',')
        .find_map(|kv| kv.split_once('=').filter(|(k, _)| k.trim() == "scorelimit"))
        .and_then(|(_, v)| v.trim().parse::<f32>().ok())
        .map_or(0, |v| v as i32);
    let timeleft = mp.timeleft.trim().parse::<f32>().ok().map(|t| t.max(0.0) as i32);
    let weapon_index = ps.map_or(0, |p| p.weapon);
    let (weapon, weapon_label) = match weapons {
        Some(w) => {
            let name = w.0.name_of(weapon_index);
            let base = name.split('+').next().unwrap_or(name).trim_end_matches("_mp").to_owned();
            let key = w.0.display_name_key_of(weapon_index).unwrap_or_default().to_owned();
            let label = (!key.is_empty())
                .then(|| slot.and_then(|s| s.localize(&key)))
                .flatten()
                .unwrap_or_else(|| base.to_ascii_uppercase());
            (base, label)
        }
        None => (String::new(), String::new()),
    };
    let offhand = dvars
        .string("bo2zm_offhand")
        .unwrap_or_default()
        .split(';')
        .filter_map(|e| {
            let (w, n) = e.split_once(':')?;
            let (lethal, _) = offhand_pictures(w)?;
            Some((lethal, w.to_owned(), n.trim().parse().unwrap_or(0)))
        })
        .collect();
    Some(Bo1Values {
        ffa,
        axis,
        yours,
        enemy,
        limit,
        timeleft,
        clip: meta.ammo_clip,
        clip_left: ps
            .filter(|p| p.last_weapon_hand == 1)
            .map(|p| clip_of(&p.ammoclip, p.weapon as i32, 1)),
        stock: meta.ammo_stock,
        weapon,
        weapon_label,
        offhand,
        height: 0,
        pictures: 0,
    })
}

fn clock(seconds: i32) -> String {
    format!("{}:{:02}", seconds / 60, seconds % 60)
}

/// A line of text: in BO2's own font when it loaded, else the game font.
fn text(
    c: &mut ChildSpawnerCommands,
    bo2: Option<&crate::bo2_font::Bo2Fonts>,
    font: &GameUiFont,
    face: &str,
    s: &str,
    px: f32,
    color: Color,
) {
    match bo2 {
        Some(bo2) => {
            bo2.spawn_line(c, face, &[(s.to_owned(), color)], px, (px / 18.0).max(1.0));
        }
        None => {
            c.spawn((Text::new(s), game_text_font(&font.0, px * 0.8), TextColor(color)));
        }
    }
}

fn picture_or(
    c: &mut ChildSpawnerCommands,
    picture: Option<Handle<Image>>,
    tint: Color,
    fallback: Color,
    node: Node,
) {
    match picture {
        Some(image) => {
            c.spawn((ImageNode { image, color: tint, ..default() }, node));
        }
        None => {
            c.spawn((BackgroundColor(fallback), node));
        }
    }
}

#[allow(clippy::too_many_arguments)]
fn bo1_hud(
    mut commands: Commands,
    presented: Option<Res<PresentedSnapshot>>,
    local: Option<Res<LocalPresentClient>>,
    font: Option<Res<GameUiFont>>,
    bo2: Option<Res<crate::bo2_font::Bo2Fonts>>,
    window: Query<&Window, With<PrimaryWindow>>,
    roots: Query<Entity, With<Bo1HudRoot>>,
    (weapons, identity, icons, slot): (
        Option<Res<assets::PreparedWeapons>>,
        Option<Res<frame::LaunchIdentity>>,
        Option<Res<assets::T6HudIcons>>,
        NonSend<crate::lui_hud::LuiHudSlot>,
    ),
    mut images: ResMut<Assets<Image>>,
    mut pictures: Local<Pictures>,
    mut bo2_icons: Local<HashMap<String, Option<Handle<Image>>>>,
    mut shown: Local<Option<Bo1Values>>,
) {
    if !bo1_hud_enabled() {
        return;
    }
    load_pictures(&mut pictures, identity.as_deref(), &mut images);
    let values = match (presented.as_deref(), local.as_deref()) {
        (Some(p), Some(l)) => read_values(p, l, weapons.as_deref(), Some(&*slot)),
        _ => None,
    };
    let height = window.single().map_or(1080.0, Window::height);
    let Some(mut values) = values else {
        for e in &roots {
            commands.entity(e).despawn();
        }
        *shown = None;
        return;
    };
    let Some(font) = font else { return };
    values.height = height as u32;
    values.pictures = pictures.ready.len();
    if shown.as_ref() == Some(&values) && !roots.is_empty() {
        return;
    }
    for e in &roots {
        commands.entity(e).despawn();
    }
    let k = (height / 1080.0).max(0.4);
    let px = |v: f32| Val::Px(v * k);
    let bo2 = bo2.as_deref();
    // A picture: Black Ops' first, then BO2's own by the same names.
    let mut pic = |names: &[&str]| -> Option<Handle<Image>> {
        for name in names {
            if let Some(h) = pictures.get(name) {
                return Some(h);
            }
        }
        for name in names {
            let h = bo2_icons
                .entry((*name).to_owned())
                .or_insert_with(|| {
                    icons
                        .as_deref()
                        .and_then(|i| i.0.iter().find(|(key, _)| key.as_str() == *name))
                        .map(|(_, img)| images.add((**img).clone()))
                })
                .clone();
            if h.is_some() {
                return h;
            }
        }
        None
    };
    let (bullet, bullet_w) = bullet_of(&values.weapon);
    let bullet_pic = pic(&[bullet, "ammo_counter_bullet"]);
    let offhands: Vec<(bool, Option<Handle<Image>>, u32)> = values
        .offhand
        .iter()
        .filter_map(|(lethal, w, n)| {
            let (_, names) = offhand_pictures(w)?;
            Some((*lethal, pic(names), *n))
        })
        .collect();
    let team_icon = if values.ffa {
        None
    } else {
        pic(&[if values.axis { AXIS_ICON } else { ALLIES_ICON }])
    };
    let backing = pic(&["hud_faction_backing"]);
    let bar_bg = pic(&["score_bar_bg"]);
    let (bar_you, bar_them) = if values.axis {
        (pic(&["score_bar_opfor"]), pic(&["score_bar_allies"]))
    } else {
        (pic(&["score_bar_allies"]), pic(&["score_bar_opfor"]))
    };
    let ammo_back = pic(&["hud_frame_black_back_fade", "hud_frame_black_back"]);
    let ammo_lines = pic(&["hud_frame_bars_fade", "hud_frame_bars"]);

    commands
        .spawn((
            Bo1HudRoot,
            UiLayer::Hud,
            UiLayerVisibility,
            Node {
                position_type: PositionType::Absolute,
                width: Val::Percent(100.0),
                height: Val::Percent(100.0),
                ..default()
            },
        ))
        .with_children(|root| {
            // --- The score corner (bottom left): faction, clock, bars. ---
            root.spawn(Node {
                position_type: PositionType::Absolute,
                left: Val::Percent(3.0),
                bottom: Val::Percent(3.5),
                flex_direction: FlexDirection::Row,
                align_items: AlignItems::FlexEnd,
                column_gap: px(8.0),
                ..default()
            })
            .with_children(|corner| {
                if !values.ffa {
                    corner
                        .spawn(Node {
                            width: px(84.0),
                            height: px(84.0),
                            justify_content: JustifyContent::Center,
                            align_items: AlignItems::Center,
                            ..default()
                        })
                        .with_children(|b| {
                            if let Some(image) = backing.clone() {
                                b.spawn((
                                    ImageNode { image, color: Color::srgba(1.0, 1.0, 1.0, 0.85), ..default() },
                                    Node {
                                        position_type: PositionType::Absolute,
                                        width: Val::Percent(100.0),
                                        height: Val::Percent(100.0),
                                        ..default()
                                    },
                                ));
                            }
                            picture_or(
                                b,
                                team_icon.clone(),
                                WHITE,
                                if values.axis { ENEMY } else { FRIENDLY },
                                Node { width: px(70.0), height: px(70.0), ..default() },
                            );
                        });
                }
                corner
                    .spawn(Node {
                        flex_direction: FlexDirection::Column,
                        row_gap: px(4.0),
                        ..default()
                    })
                    .with_children(|col| {
                        if let Some(t) = values.timeleft {
                            let color = if t <= 30 { LOW } else { WHITE };
                            col.spawn(Node::default()).with_children(|c| {
                                text(c, bo2, &font, "Big", &clock(t), 30.0 * k, color);
                            });
                        }
                        let limit = values.limit.max(1) as f32;
                        for (score, fill, color) in [
                            (values.yours, bar_you.clone(), FRIENDLY),
                            (values.enemy, bar_them.clone(), ENEMY),
                        ] {
                            col.spawn(Node {
                                flex_direction: FlexDirection::Row,
                                align_items: AlignItems::Center,
                                column_gap: px(8.0),
                                ..default()
                            })
                            .with_children(|row| {
                                row.spawn(Node { width: px(230.0), height: px(16.0), ..default() })
                                    .with_children(|bar| {
                                        let full = Node {
                                            position_type: PositionType::Absolute,
                                            width: Val::Percent(100.0),
                                            height: Val::Percent(100.0),
                                            ..default()
                                        };
                                        picture_or(bar, bar_bg.clone(), WHITE, BAR_BACK, full);
                                        let part = if values.limit > 0 {
                                            (score as f32 / limit).clamp(0.0, 1.0)
                                        } else {
                                            0.0
                                        };
                                        picture_or(
                                            bar,
                                            fill,
                                            color,
                                            color,
                                            Node {
                                                position_type: PositionType::Absolute,
                                                width: Val::Percent(part * 100.0),
                                                height: Val::Percent(100.0),
                                                ..default()
                                            },
                                        );
                                    });
                                text(row, bo2, &font, "Default", &score.to_string(), 26.0 * k, WHITE);
                            });
                        }
                    });
            });

            // --- The ammo corner (bottom right). ---
            let shows_ammo = values.clip > 0 || values.stock > 0 || values.clip_left.is_some();
            root.spawn(Node {
                position_type: PositionType::Absolute,
                right: Val::Percent(3.0),
                bottom: Val::Percent(3.5),
                flex_direction: FlexDirection::Column,
                align_items: AlignItems::FlexEnd,
                row_gap: px(4.0),
                ..default()
            })
            .with_children(|corner| {
                if !values.weapon_label.is_empty() {
                    corner.spawn(Node::default()).with_children(|c| {
                        text(c, bo2, &font, "Default", &values.weapon_label, 22.0 * k, DIM);
                    });
                }
                corner
                    .spawn(Node {
                        flex_direction: FlexDirection::Row,
                        align_items: AlignItems::FlexEnd,
                        column_gap: px(14.0),
                        padding: UiRect::axes(px(10.0), px(4.0)),
                        ..default()
                    })
                    .with_children(|row| {
                        // The frame behind it all.
                        if let Some(image) = ammo_back.clone() {
                            row.spawn((
                                ImageNode { image, color: Color::srgba(1.0, 1.0, 1.0, 0.6), ..default() },
                                Node {
                                    position_type: PositionType::Absolute,
                                    left: px(-30.0),
                                    right: px(-6.0),
                                    top: px(-6.0),
                                    bottom: px(-6.0),
                                    ..default()
                                },
                            ));
                        }
                        if let Some(image) = ammo_lines.clone() {
                            row.spawn((
                                ImageNode { image, color: Color::srgba(1.0, 1.0, 1.0, 0.5), ..default() },
                                Node {
                                    position_type: PositionType::Absolute,
                                    left: px(-30.0),
                                    right: px(-6.0),
                                    bottom: px(-8.0),
                                    height: px(6.0),
                                    ..default()
                                },
                            ));
                        }
                        // Grenades: tactical then lethal, a count by each.
                        for lethal in [false, true] {
                            for (_, picture, n) in offhands.iter().filter(|(l, _, _)| *l == lethal) {
                                row.spawn(Node {
                                    flex_direction: FlexDirection::Row,
                                    align_items: AlignItems::Center,
                                    column_gap: px(2.0),
                                    ..default()
                                })
                                .with_children(|g| {
                                    let tint = if *n == 0 { Color::srgba(1.0, 1.0, 1.0, 0.3) } else { WHITE };
                                    picture_or(
                                        g,
                                        picture.clone(),
                                        tint,
                                        Color::srgba(0.8, 0.8, 0.8, 0.5),
                                        Node { width: px(40.0), height: px(40.0), ..default() },
                                    );
                                    if *n > 1 {
                                        text(g, bo2, &font, "Default", &n.to_string(), 20.0 * k, WHITE);
                                    }
                                });
                            }
                        }
                        if !shows_ammo {
                            return;
                        }
                        // The second gun's clip (dual wield), as a number.
                        if let Some(left) = values.clip_left {
                            text(row, bo2, &font, "Big", &left.to_string(), 34.0 * k, if left == 0 { LOW } else { WHITE });
                        }
                        // One round picture per round in the clip, newest on
                        // the right, in rows of 30 stacked upward.
                        let per_row = if bullet_w >= 20.0 { 10 } else { 30 };
                        let rounds = values.clip.clamp(0, 150) as usize;
                        row.spawn(Node {
                            flex_direction: FlexDirection::ColumnReverse,
                            align_items: AlignItems::FlexEnd,
                            row_gap: px(2.0),
                            ..default()
                        })
                        .with_children(|bullets| {
                            let mut left = rounds;
                            while left > 0 {
                                let n = left.min(per_row);
                                left -= n;
                                bullets
                                    .spawn(Node {
                                        flex_direction: FlexDirection::Row,
                                        column_gap: px(1.0),
                                        ..default()
                                    })
                                    .with_children(|line| {
                                        for _ in 0..n {
                                            picture_or(
                                                line,
                                                bullet_pic.clone(),
                                                WHITE,
                                                Color::srgb(0.85, 0.78, 0.55),
                                                Node {
                                                    width: px(bullet_w),
                                                    height: px(if bullet_pic.is_some() { 36.0 } else { 26.0 }),
                                                    ..default()
                                                },
                                            );
                                        }
                                    });
                            }
                        });
                        // The stock.
                        let low = values.stock == 0;
                        text(row, bo2, &font, "Big", &values.stock.to_string(), 40.0 * k, if low { LOW } else { WHITE });
                    });
            });
        });
    *shown = Some(values);
}

pub(crate) fn register_bo1_hud_systems(app: &mut App) {
    app.add_systems(
        Update,
        bo1_hud.after(crate::bo2_font::load_bo2_fonts).in_set(ClientSet::Ui),
    );
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn rounds_by_gun() {
        assert_eq!(bullet_of("dsr50").0, "ammo_counter_riflebullet");
        assert_eq!(bullet_of("ksg").0, "ammo_counter_shotgunshell");
        assert_eq!(bullet_of("smaw").0, "ammo_counter_rocket");
        assert_eq!(bullet_of("mk48").0, "ammo_counter_beltbullet");
        assert_eq!(bullet_of("an94").0, "ammo_counter_bullet");
    }

    #[test]
    fn clock_reads_like_black_ops() {
        assert_eq!(clock(272), "4:32");
        assert_eq!(clock(5), "0:05");
    }

    #[test]
    fn equipment_has_a_slot() {
        assert_eq!(offhand_pictures("frag_grenade_mp").map(|p| p.0), Some(true));
        assert_eq!(offhand_pictures("flash_grenade_mp").map(|p| p.0), Some(false));
        assert!(offhand_pictures("an94_mp").is_none());
    }
}
