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
use std::sync::Mutex;
use std::sync::mpsc::{Receiver, channel};
use std::time::Instant;

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
    // The round minimap's marks.
    "compassping_player",
    "compassping_friendly_mp",
    "compassping_enemy",
    "compassping_enemyfiring",
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

/// Black Ops' pictures: decoded off the main thread once, then kept; and
/// BO2's own by the same names, for what Black Ops lacks.
#[derive(Resource, Default)]
struct Bo1Pictures {
    started: bool,
    waiting: Mutex<Option<Receiver<Vec<(String, u32, u32, Vec<u8>)>>>>,
    ready: HashMap<String, Handle<Image>>,
    bo2: HashMap<String, Option<Handle<Image>>>,
}

impl Bo1Pictures {
    /// The first of `names` there is: Black Ops' first, then BO2's.
    fn pick(
        &mut self,
        names: &[&str],
        icons: Option<&assets::T6HudIcons>,
        images: &mut Assets<Image>,
    ) -> Option<Handle<Image>> {
        for name in names {
            if let Some(h) = self.ready.get(*name) {
                return Some(h.clone());
            }
        }
        for name in names {
            let h = self
                .bo2
                .entry((*name).to_owned())
                .or_insert_with(|| {
                    icons
                        .and_then(|i| i.0.iter().find(|(key, _)| key.as_str() == *name))
                        .map(|(_, img)| images.add((**img).clone()))
                })
                .clone();
            if h.is_some() {
                return h;
            }
        }
        None
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
    mut pictures: ResMut<Bo1Pictures>,
    identity: Option<Res<frame::LaunchIdentity>>,
    mut images: ResMut<Assets<Image>>,
) {
    if !bo1_hud_enabled() {
        return;
    }
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
        if let Ok(mut waiting) = pictures.waiting.lock() {
            *waiting = Some(rx);
        }
    }
    let decoded = {
        let Ok(mut waiting) = pictures.waiting.lock() else { return };
        let Some(rx) = waiting.as_ref() else { return };
        let Ok(decoded) = rx.try_recv() else { return };
        *waiting = None;
        decoded
    };
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
        let handle = images.add(image);
        pictures.ready.insert(name, handle);
    }
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
    /// The clip is at a quarter or less of the most it was seen holding.
    low_clip: bool,
    weapon: String,
    weapon_label: String,
    /// He holds a grenade or a scorestreak, not a gun.
    not_a_gun: bool,
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

/// The match's values while Black Ops' HUD shows: not in a killcam, at the
/// match's end, or under a menu (the pause menu, the scoreboard's menus,
/// Create-a-Class): BO2 hides its HUD then too.
fn match_values(
    presented: &PresentedSnapshot,
    local: &LocalPresentClient,
    menus: Option<&frame::LuiMenus>,
) -> Option<crate::lui_frontend::MatchValues> {
    if menus.is_some_and(|m| m.open) {
        return None;
    }
    let snap = presented.snapshot()?;
    let dvars = snap.meta.script_dvars(local.0);
    let mp = crate::lui_frontend::MatchValues::read(&dvars)?;
    if mp.hud_hidden || !mp.killcam.is_empty() {
        return None;
    }
    if mp.menu.split_once(':').is_some_and(|(_, m)| !m.trim().is_empty()) {
        return None;
    }
    Some(mp)
}

fn read_values(
    presented: &PresentedSnapshot,
    local: &LocalPresentClient,
    menus: Option<&frame::LuiMenus>,
    weapons: Option<&assets::PreparedWeapons>,
    slot: Option<&crate::lui_hud::LuiHudSlot>,
    most: &mut HashMap<u32, i32>,
) -> Option<Bo1Values> {
    let mp = match_values(presented, local, menus)?;
    let snap = presented.snapshot()?;
    let dvars = snap.meta.script_dvars(local.0);
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
                .unwrap_or_else(|| base.replace('_', " ").to_ascii_uppercase());
            (base, label)
        }
        None => (String::new(), String::new()),
    };
    let not_a_gun = offhand_pictures(&format!("{weapon}_mp")).is_some()
        || ["killstreak", "remote", "_drop", "briefcase", "minigun", "m32", "inventory"]
            .iter()
            .any(|k| weapon.contains(k));
    let clip = meta.ammo_clip;
    let seen = most.entry(weapon_index).or_insert(0);
    *seen = (*seen).max(clip);
    let low_clip = *seen >= 4 && clip * 4 <= *seen;
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
        clip,
        clip_left: ps
            .filter(|p| p.last_weapon_hand == 1)
            .map(|p| clip_of(&p.ammoclip, p.weapon as i32, 1)),
        stock: meta.ammo_stock,
        low_clip,
        weapon,
        weapon_label,
        not_a_gun,
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
    (weapons, menus, icons, slot): (
        Option<Res<assets::PreparedWeapons>>,
        Option<Res<frame::LuiMenus>>,
        Option<Res<assets::T6HudIcons>>,
        NonSend<crate::lui_hud::LuiHudSlot>,
    ),
    mut images: ResMut<Assets<Image>>,
    mut pictures: ResMut<Bo1Pictures>,
    mut most: Local<HashMap<u32, i32>>,
    mut shown: Local<Option<Bo1Values>>,
) {
    if !bo1_hud_enabled() {
        return;
    }
    let values = match (presented.as_deref(), local.as_deref()) {
        (Some(p), Some(l)) => read_values(p, l, menus.as_deref(), weapons.as_deref(), Some(&*slot), &mut most),
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
    let icons = icons.as_deref();
    let mut pic = |names: &[&str]| pictures.pick(names, icons, &mut images);
    let (bullet, _) = bullet_of(&values.weapon);
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
                                row.spawn((
                                    BackgroundColor(BAR_BACK),
                                    Node { width: px(230.0), height: px(16.0), ..default() },
                                ))
                                .with_children(|bar| {
                                    if let Some(image) = bar_bg.clone() {
                                        bar.spawn((
                                            ImageNode { image, ..default() },
                                            Node {
                                                position_type: PositionType::Absolute,
                                                width: Val::Percent(100.0),
                                                height: Val::Percent(100.0),
                                                ..default()
                                            },
                                        ));
                                    }
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

            // --- The ammo corner (bottom right), Black Ops' `weaponinfo`:
            // the grenades, the gun's round, the clip in big figures and the
            // rest of his ammo in small ones, the gun's name above. ---
            let shows_ammo = !values.not_a_gun && (values.clip > 0 || values.stock > 0 || values.clip_left.is_some());
            root.spawn(Node {
                position_type: PositionType::Absolute,
                right: Val::Percent(3.0),
                bottom: Val::Percent(3.5),
                flex_direction: FlexDirection::Column,
                align_items: AlignItems::FlexEnd,
                row_gap: px(2.0),
                ..default()
            })
            .with_children(|corner| {
                if shows_ammo && !values.weapon_label.is_empty() {
                    corner.spawn(Node::default()).with_children(|c| {
                        text(c, bo2, &font, "Default", &values.weapon_label, 22.0 * k, DIM);
                    });
                }
                corner
                    .spawn(Node {
                        flex_direction: FlexDirection::Row,
                        align_items: AlignItems::FlexEnd,
                        column_gap: px(12.0),
                        padding: UiRect::axes(px(12.0), px(2.0)),
                        ..default()
                    })
                    .with_children(|row| {
                        // The dark slant behind it.
                        if let Some(image) = ammo_back.clone() {
                            row.spawn((
                                ImageNode { image, color: Color::srgba(1.0, 1.0, 1.0, 0.55), ..default() },
                                Node {
                                    position_type: PositionType::Absolute,
                                    left: px(-40.0),
                                    right: px(-8.0),
                                    top: px(-4.0),
                                    bottom: px(-6.0),
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
                                    margin: UiRect::bottom(px(6.0)),
                                    ..default()
                                })
                                .with_children(|g| {
                                    let tint = if *n == 0 { Color::srgba(1.0, 1.0, 1.0, 0.3) } else { WHITE };
                                    picture_or(
                                        g,
                                        picture.clone(),
                                        tint,
                                        Color::srgba(0.8, 0.8, 0.8, 0.5),
                                        Node { width: px(36.0), height: px(36.0), ..default() },
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
                        // The gun's round (one picture: rifle, shell, belt,
                        // rocket or bullet).
                        if let Some(image) = bullet_pic.clone() {
                            row.spawn((
                                ImageNode { image, ..default() },
                                Node {
                                    width: px(14.0),
                                    height: px(42.0),
                                    margin: UiRect::bottom(px(6.0)),
                                    ..default()
                                },
                            ));
                        }
                        let clip_color = if values.low_clip || values.clip == 0 { LOW } else { WHITE };
                        row.spawn(Node {
                            flex_direction: FlexDirection::Row,
                            align_items: AlignItems::FlexEnd,
                            column_gap: px(6.0),
                            ..default()
                        })
                        .with_children(|nums| {
                            // A second gun (dual wield): its clip first.
                            if let Some(left) = values.clip_left {
                                text(nums, bo2, &font, "Big", &left.to_string(), 56.0 * k, if left == 0 { LOW } else { WHITE });
                                text(nums, bo2, &font, "Big", "|", 40.0 * k, DIM);
                            }
                            text(nums, bo2, &font, "Big", &values.clip.to_string(), 56.0 * k, clip_color);
                            nums.spawn(Node { margin: UiRect::bottom(px(5.0)), ..default() })
                                .with_children(|c| {
                                    let stock_color = if values.stock == 0 { LOW } else { DIM };
                                    text(c, bo2, &font, "Big", &values.stock.to_string(), 30.0 * k, stock_color);
                                });
                        });
                    });
            });
        });
    *shown = Some(values);
}

// --- The round minimap (Black Ops' `compassround`, top left). ---

/// The minimap BO2's HUD worked out this frame (`lui_hud`), for the round
/// one: his view of the map, whether he is looking through a scope, and
/// when it came.
static MINIMAP: Mutex<Option<(crate::lui_frontend::MinimapView, bool, Instant)>> = Mutex::new(None);

pub(crate) fn publish_minimap(view: Option<&crate::lui_frontend::MinimapView>, scoped: bool) {
    if let Ok(mut m) = MINIMAP.lock() {
        *m = view.map(|v| (v.clone(), scoped, Instant::now()));
    }
}

#[derive(Component)]
struct Bo1CompassRoot;

/// The round map's size on a 1080-line screen (Black Ops' 104 of 480).
const COMPASS_PX: f32 = 236.0;

/// The map turned so he faces up, cut to a disc on a dark ground with a
/// thin rim.
fn round_map(view: &crate::lui_frontend::MinimapView, src: &Image, line: Option<&Image>, n: usize) -> Vec<u8> {
    let (sw, sh) = (src.texture_descriptor.size.width as usize, src.texture_descriptor.size.height as usize);
    let line = line.and_then(|i| {
        let (lw, lh) = (i.texture_descriptor.size.width as usize, i.texture_descriptor.size.height as usize);
        Some((i.data.as_deref()?, lw, lh))
    });
    let mut px = src.data.as_deref().map(|d| view.render(d, sw, sh, n, line)).unwrap_or_else(|| vec![0; n * n * 4]);
    let r = n as f32 * 0.5;
    for j in 0..n {
        for i in 0..n {
            let (dx, dy) = (i as f32 + 0.5 - r, j as f32 + 0.5 - r);
            let d = (dx * dx + dy * dy).sqrt();
            let o = (j * n + i) * 4;
            // Outside the disc: nothing (a pixel of soft edge).
            let inside = (r - d).clamp(0.0, 1.0);
            // The rim: a light ring two pixels in from the edge.
            let rim = (1.5 - (d - (r - 2.5)).abs()).clamp(0.0, 1.0);
            // Under the map, a dark ground.
            let (ma, ground) = (f32::from(px[o + 3]) / 255.0, 0.55);
            let a = ma + ground * (1.0 - ma);
            let mut rgb = [0.0f32; 3];
            for (k, c) in rgb.iter_mut().enumerate() {
                *c = if a > 0.0 { f32::from(px[o + k]) * ma / a } else { 0.0 };
            }
            for c in &mut rgb {
                *c = *c * (1.0 - rim * 0.7) + 200.0 * rim * 0.7;
            }
            let a = (a + rim * 0.7 * (1.0 - a)) * inside;
            for k in 0..3 {
                px[o + k] = rgb[k].round().clamp(0.0, 255.0) as u8;
            }
            px[o + 3] = (a * 255.0).round() as u8;
        }
    }
    px
}

#[allow(clippy::too_many_arguments)]
fn bo1_compass(
    mut commands: Commands,
    presented: Option<Res<PresentedSnapshot>>,
    local: Option<Res<LocalPresentClient>>,
    menus: Option<Res<frame::LuiMenus>>,
    icons: Option<Res<assets::T6HudIcons>>,
    window: Query<&Window, With<PrimaryWindow>>,
    roots: Query<Entity, With<Bo1CompassRoot>>,
    mut images: ResMut<Assets<Image>>,
    mut pictures: ResMut<Bo1Pictures>,
    mut map_image: Local<Option<Handle<Image>>>,
) {
    let view = MINIMAP
        .lock()
        .ok()
        .and_then(|m| m.clone())
        .filter(|(_, scoped, at)| !scoped && at.elapsed().as_secs_f32() < 0.5)
        .map(|(v, _, _)| v);
    let shown = bo1_hud_enabled()
        && match (presented.as_deref(), local.as_deref()) {
            (Some(p), Some(l)) => match_values(p, l, menus.as_deref()).is_some(),
            _ => false,
        };
    let (Some(view), true) = (view, shown) else {
        for e in &roots {
            commands.entity(e).despawn();
        }
        return;
    };
    let icons = icons.as_deref();
    let Some(src) = icons.and_then(|i| i.0.iter().find(|(k, _)| *k == view.material)).map(|(_, i)| i.clone()) else {
        return;
    };
    let line = view
        .line
        .and_then(|_| icons?.0.iter().find(|(k, _)| *k == "compass_radarline"))
        .map(|(_, i)| i.clone());
    let height = window.single().map_or(1080.0, Window::height);
    let k = (height / 1080.0).max(0.4);
    let size = COMPASS_PX * k;
    let n = (size.round() as usize).clamp(64, 400);
    let pixels = round_map(&view, &src, line.as_deref(), n);
    let mut made = Image::new(
        Extent3d { width: n as u32, height: n as u32, depth_or_array_layers: 1 },
        TextureDimension::D2,
        pixels,
        TextureFormat::Rgba8Unorm,
        RenderAssetUsages::default(),
    );
    made.sampler = bevy::image::ImageSampler::linear();
    let handle = match map_image.as_ref() {
        Some(h) if images.get(h).is_some() => {
            let h = h.clone();
            images.insert(&h, made).ok();
            h
        }
        _ => {
            let h = images.add(made);
            *map_image = Some(h.clone());
            h
        }
    };
    // His mark in the middle, his team's and the enemies the radar shows
    // around him, each turned to where he faces on the map.
    let mut marks: Vec<(Option<Handle<Image>>, [f32; 2], f32, f32, f32)> = Vec::new();
    let mark = (size * 0.12).max(10.0);
    let per_unit = size / view.range;
    let me_turn = view.heading(view.me.1);
    let r = size * 0.5 - mark * 0.35;
    let mut place = |names: &[&str], p: [f32; 2], yaw: f32, alpha: f32, marks: &mut Vec<_>| {
        let o = view.screen_offset(p);
        let at = [o[0] * per_unit, o[1] * per_unit];
        if at[0] * at[0] + at[1] * at[1] > r * r {
            return;
        }
        let picture = pictures.pick(names, icons, &mut images);
        marks.push((picture, at, view.heading(yaw) - me_turn, alpha, mark));
    };
    for (p, yaw, firing, alpha) in &view.enemies {
        let names: &[&str] = if *firing { &["compassping_enemyfiring", "compassping_enemy"] } else { &["compassping_enemy"] };
        place(names, *p, *yaw, *alpha, &mut marks);
    }
    for (p, yaw) in &view.friends {
        place(&["compassping_friendly_mp", "compassping_friendly"], *p, *yaw, 1.0, &mut marks);
    }
    place(&["compassping_player"], view.me.0, view.me.1, 1.0, &mut marks);

    let root = match roots.iter().next() {
        Some(e) => {
            commands.entity(e).despawn_related::<Children>();
            e
        }
        None => commands
            .spawn((
                Bo1CompassRoot,
                UiLayer::Hud,
                UiLayerVisibility,
                Node {
                    position_type: PositionType::Absolute,
                    left: Val::Percent(2.2),
                    top: Val::Percent(3.0),
                    width: Val::Px(size),
                    height: Val::Px(size),
                    ..default()
                },
            ))
            .id(),
    };
    commands.entity(root).insert(Node {
        position_type: PositionType::Absolute,
        left: Val::Percent(2.2),
        top: Val::Percent(3.0),
        width: Val::Px(size),
        height: Val::Px(size),
        ..default()
    });
    commands.entity(root).with_children(|c| {
        c.spawn((
            ImageNode::new(handle),
            Node {
                position_type: PositionType::Absolute,
                width: Val::Percent(100.0),
                height: Val::Percent(100.0),
                ..default()
            },
        ));
        for (picture, at, turn, alpha, side) in marks {
            let node = Node {
                position_type: PositionType::Absolute,
                left: Val::Px(size * 0.5 + at[0] - side * 0.5),
                top: Val::Px(size * 0.5 + at[1] - side * 0.5),
                width: Val::Px(side),
                height: Val::Px(side),
                ..default()
            };
            let turned = UiTransform { rotation: Rot2::radians(turn), ..UiTransform::IDENTITY };
            match picture {
                Some(image) => {
                    c.spawn((ImageNode { image, color: Color::srgba(1.0, 1.0, 1.0, alpha), ..default() }, node, turned));
                }
                None => {
                    c.spawn((BackgroundColor(Color::srgba(1.0, 0.85, 0.2, alpha)), node, turned));
                }
            }
        }
    });
}

pub(crate) fn register_bo1_hud_systems(app: &mut App) {
    app.init_resource::<Bo1Pictures>();
    app.add_systems(
        Update,
        (load_pictures, bo1_hud, bo1_compass)
            .chain()
            .after(crate::bo2_font::load_bo2_fonts)
            .in_set(ClientSet::Ui),
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
