//! bo2mp: the in-game HUD, in a clean modern layout over Black Ops II's game.
//!
//! - Top left: a square minimap that turns with him, a compass tape along
//!   its top edge (N, NE, E, ...), his team and the enemies his radar shows.
//! - Top centre: the score bar. His side's score in a blue box with its bar
//!   reaching left, the clock, the enemy's score in a red box with its bar
//!   reaching right, and WINNING / LOSING / TIED under it.
//! - Bottom left: his name over a thin progress bar (his score toward the
//!   limit).
//! - Bottom right: his three scorestreaks in a column of squares on the
//!   right edge (the earned one lit and pulsing, the next one filling up,
//!   used ones dimmed), the gun's name (his grenades beside it), then the
//!   gun's attachments in gold, one per line, left of a dark box with the
//!   clip in big figures, the rest of the ammo and the gun's picture.
//!
//! BO2's own ammo area, score corner, square minimap and scorestreak column
//! are still built but never drawn (`hks_t6::host`'s `hide_for_bo1_hud`);
//! the kill feed, hit marker, kill banner and scoreboard are `hud_match`'s
//! own; BO2's killcam and menus stay. `BO2_HUD=bo2` brings BO2's own HUD
//! back.

use std::collections::HashMap;
use std::sync::Mutex;
use std::time::Instant;

use bevy::asset::RenderAssetUsages;
use bevy::prelude::*;
use bevy::render::render_resource::{Extent3d, TextureDimension, TextureFormat};
use bevy::window::PrimaryWindow;
use frame::ClientSet;
use net::{LocalPresentClient, PresentedSnapshot};

use crate::layers::{GameUiFont, UiLayer, UiLayerVisibility, game_text_font};
use crate::shell::theme::{ACCENT, GOLD};

/// Free-for-all game types (one score per player, no teams).
const FFA_TYPES: [&str; 5] = ["dm", "gun", "oic", "sas", "shrp"];

const WHITE: Color = Color::srgb(1.0, 1.0, 1.0);
const DIM: Color = Color::srgba(1.0, 1.0, 1.0, 0.62);
const LOW: Color = Color::srgb(0.95, 0.28, 0.22);
const BLUE: Color = Color::srgb(0.26, 0.62, 0.95);
const RED: Color = Color::srgb(0.90, 0.26, 0.22);
const PANEL: Color = Color::srgba(0.04, 0.05, 0.06, 0.62);
const TRACK: Color = Color::srgba(0.0, 0.0, 0.0, 0.45);

/// A scorestreak square's side on a 1080-line screen.
const STREAK_PX: f32 = 62.0;

/// Where one of his scorestreaks stands.
#[derive(Clone, Copy, Debug, PartialEq)]
enum StreakState {
    /// Earned and not used yet: lit, pulsing.
    Ready,
    /// The next one he earns, and how far he is toward it (0 to 1).
    Next(f32),
    /// Further off than the next one.
    Locked,
    /// Earned this life and used.
    Used,
}

/// One of his scorestreaks in the column: BO2's HUD picture for it (the
/// statstable item's image, `hud_ks_*`) and its momentum cost.
#[derive(Clone, Debug, PartialEq)]
struct Streak {
    icon: String,
    cost: i32,
    state: StreakState,
}

/// His scorestreaks as the server publishes them (`bo2mp_streaks`:
/// `icon:cost[:held]` by `;`, dearest first, `held` 1 while his streak
/// weapon has one to use) against his momentum (`bo2mp_momentum`). Without
/// `held` (the default picks, before his class script gave him his own)
/// one is ready once his momentum reaches its cost, as BO2's column shows.
fn parse_streaks(list: &str, momentum: i32) -> Vec<Streak> {
    let picks: Vec<(String, i32, Option<bool>)> = list
        .split(';')
        .filter_map(|e| {
            let mut p = e.split(':');
            let icon = p.next()?.trim();
            let cost: i32 = p.next()?.trim().parse().ok()?;
            let held = p.next().map(|h| h.trim() == "1");
            (!icon.is_empty()).then(|| (icon.to_owned(), cost, held))
        })
        .collect();
    let ready = |cost: i32, held: Option<bool>| held.unwrap_or(momentum >= cost);
    // The next one: the cheapest he has not reached yet.
    let next = picks.iter().filter(|(_, c, h)| !ready(*c, *h) && momentum < *c).map(|(_, c, _)| *c).min();
    picks
        .into_iter()
        .map(|(icon, cost, held)| {
            let state = if ready(cost, held) {
                StreakState::Ready
            } else if momentum >= cost {
                StreakState::Used
            } else if Some(cost) == next {
                StreakState::Next((momentum.max(0) as f32 / cost.max(1) as f32).clamp(0.0, 1.0))
            } else {
                StreakState::Locked
            };
            Streak { icon, cost, state }
        })
        .collect()
}

/// An attachment's name when BO2's tables do not give one (its
/// mp/attachmenttable.csv reference; BO2's own English names).
fn attachment_fallback(reference: &str) -> String {
    match reference {
        "reflex" => "Reflex Sight",
        "acog" => "ACOG Sight",
        "holo" => "EOTech Sight",
        "mms" => "Millimeter Scanner",
        "ir" => "Dual Band Scope",
        "dualoptic" => "Hybrid Optic",
        "rangefinder" => "Target Finder",
        "vzoom" => "Variable Zoom",
        "silencer" => "Suppressor",
        "extclip" => "Extended Clip",
        "dualclip" => "Fast Mag",
        "fastads" => "Quickdraw Handle",
        "fmj" => "FMJ",
        "grip" => "Fore Grip",
        "gl" => "Launcher",
        "sf" => "Select Fire",
        "rf" => "Rapid Fire",
        "dw" => "Dual Wield",
        "steadyaim" => "Laser Sight",
        "stalker" => "Stalker Stock",
        "swayreduc" => "Ballistics CPU",
        "tacknife" => "Tactical Knife",
        "extbarrel" => "Long Barrel",
        "stackfire" => "Tri-Bolt",
        other => return other.replace('_', " ").to_uppercase(),
    }
    .to_owned()
}

pub(crate) fn modern_hud_enabled() -> bool {
    hks_t6::host::bo1_hud_on() && crate::lui_hud::lui_enabled()
}

#[derive(Component)]
struct HudRoot;

/// BO2's picture for a grenade or piece of equipment, and whether it is
/// lethal.
fn offhand_picture(weapon: &str) -> Option<(bool, &'static str)> {
    Some(match weapon {
        "frag_grenade_mp" => (true, "hud_grenadeicon"),
        "sticky_grenade_mp" => (true, "hud_icon_sticky_grenade"),
        "hatchet_mp" => (true, "hud_hatchet"),
        "claymore_mp" => (true, "hud_icon_claymore"),
        "satchel_charge_mp" => (true, "hud_icon_satchelcharge"),
        "bouncingbetty_mp" => (true, "hud_bounce_betty"),
        "flash_grenade_mp" => (false, "hud_us_flashgrenade"),
        "concussion_grenade_mp" => (false, "hud_us_stungrenade"),
        "willy_pete_mp" => (false, "hud_willy_pete"),
        "emp_grenade_mp" => (false, "hud_empgrenade"),
        "trophy_system_mp" => (false, "hud_trophy_system"),
        "tactical_insertion_mp" => (false, "hud_tact_insert"),
        "sensor_grenade_mp" => (false, "hud_sensor_grenade"),
        "proximity_grenade_mp" => (false, "hud_proximity_grenade"),
        _ => return None,
    })
}

#[derive(Clone, Debug, Default, PartialEq)]
struct HudValues {
    ffa: bool,
    axis: bool,
    yours: i32,
    enemy: i32,
    limit: i32,
    /// Seconds left, when the mode has a clock.
    timeleft: Option<i32>,
    name: String,
    score: i32,
    clip: i32,
    clip_left: Option<i32>,
    stock: i32,
    low_clip: bool,
    weapon: String,
    weapon_label: String,
    /// He holds a grenade or a scorestreak, not a gun.
    not_a_gun: bool,
    /// (lethal?, the weapon, count).
    offhand: Vec<(bool, String, u32)>,
    /// The held gun's attachments, by name (`m16_mp+extclip+silencer`).
    attachments: Vec<String>,
    /// His three scorestreaks, dearest first.
    streaks: Vec<Streak>,
    height: u32,
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

/// The match's values while the HUD shows: not in a killcam, at the match's
/// end, or under a menu (BO2 hides its HUD then too).
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
) -> Option<HudValues> {
    let mp = match_values(presented, local, menus)?;
    let snap = presented.snapshot()?;
    let dvars = snap.meta.script_dvars(local.0);
    let meta = snap.meta.for_client(local.0)?;
    let ps = presented.player(local.0);
    let ffa = FFA_TYPES.contains(&mp.gametype.as_str()) || mp.team == "free";
    let (yours, enemy) = if ffa {
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
        let (allies, axis) = mp.scores.split_once(',').unwrap_or(("0", "0"));
        let (allies, axis) = (allies.trim().parse().unwrap_or(0), axis.trim().parse().unwrap_or(0));
        if mp.team == "axis" { (axis, allies) } else { (allies, axis) }
    };
    let limit = mp
        .settings
        .split(',')
        .find_map(|kv| kv.split_once('=').filter(|(k, _)| k.trim() == "scorelimit"))
        .and_then(|(_, v)| v.trim().parse::<f32>().ok())
        .map_or(0, |v| v as i32);
    let timeleft = mp.timeleft.trim().parse::<f32>().ok().map(|t| t.max(0.0) as i32);
    let name = String::from_utf8_lossy(&meta.name).trim_end_matches(char::from(0)).to_owned();
    let weapon_index = ps.map_or(0, |p| p.weapon);
    let mut attachments = Vec::new();
    let (weapon, weapon_label) = match weapons {
        Some(w) => {
            let full = w.0.name_of(weapon_index);
            let base = full.split('+').next().unwrap_or(full).to_owned();
            attachments = full
                .split('+')
                .skip(1)
                .map(str::trim)
                .filter(|a| !a.is_empty() && *a != "none")
                .map(|a| slot.and_then(|s| s.attachment_name(a)).unwrap_or_else(|| attachment_fallback(a)))
                .collect();
            let key = w.0.display_name_key_of(weapon_index).unwrap_or_default().to_owned();
            let label = (!key.is_empty())
                .then(|| slot.and_then(|s| s.localize(&key)))
                .flatten()
                .unwrap_or_else(|| base.trim_end_matches("_mp").replace('_', " ").to_ascii_uppercase());
            (base, label)
        }
        None => (String::new(), String::new()),
    };
    let not_a_gun = offhand_picture(&weapon).is_some()
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
            let (lethal, _) = offhand_picture(w)?;
            Some((lethal, w.to_owned(), n.trim().parse().unwrap_or(0)))
        })
        .collect();
    Some(HudValues {
        ffa,
        axis: mp.team == "axis",
        yours,
        enemy,
        limit,
        timeleft,
        name,
        score: meta.score,
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
        attachments,
        streaks: parse_streaks(
            dvars.string("bo2mp_streaks").unwrap_or_default(),
            dvars.string("bo2mp_momentum").and_then(|m| m.trim().parse().ok()).unwrap_or(0),
        ),
        height: 0,
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
            bo2.spawn_line(c, face, &[(s.to_owned(), color)], px, (px / 22.0).max(1.0));
        }
        None => {
            c.spawn((
                Text::new(s),
                game_text_font(&font.0, px * 0.8),
                TextColor(color),
                TextShadow { offset: Vec2::splat((px / 22.0).max(1.0)), color: Color::srgba(0.0, 0.0, 0.0, 0.75), ..default() },
            ));
        }
    }
}

/// A BO2 HUD picture by name, once per name.
fn picture(
    cache: &mut HashMap<String, Option<Handle<Image>>>,
    icons: Option<&assets::T6HudIcons>,
    images: &mut Assets<Image>,
    name: &str,
) -> Option<Handle<Image>> {
    cache
        .entry(name.to_owned())
        .or_insert_with(|| {
            icons
                .and_then(|i| i.0.iter().find(|(key, _)| key.as_str() == name))
                .map(|(_, img)| images.add((**img).clone()))
        })
        .clone()
}

/// The gun's picture (its kill icon, `killicon:<weapon>:<ratio>`) and how
/// wide it is for its height.
fn gun_picture(
    cache: &mut HashMap<String, Option<Handle<Image>>>,
    icons: Option<&assets::T6HudIcons>,
    images: &mut Assets<Image>,
    weapon: &str,
) -> Option<(Handle<Image>, f32)> {
    let icons_ref = icons?;
    let prefix = format!("killicon:{weapon}:");
    let (key, wide) = icons_ref.0.iter().find_map(|(k, _)| {
        let ratio = k.strip_prefix(&prefix)?;
        Some((
            k.clone(),
            match ratio {
                "1" => 2.0,
                "2" => 4.0,
                _ => 1.0,
            },
        ))
    })?;
    Some((picture(cache, icons, images, &key)?, wide))
}

#[allow(clippy::too_many_arguments)]
fn modern_hud(
    mut commands: Commands,
    presented: Option<Res<PresentedSnapshot>>,
    local: Option<Res<LocalPresentClient>>,
    font: Option<Res<GameUiFont>>,
    bo2: Option<Res<crate::bo2_font::Bo2Fonts>>,
    window: Query<&Window, With<PrimaryWindow>>,
    roots: Query<Entity, With<HudRoot>>,
    (weapons, menus, icons, slot): (
        Option<Res<assets::PreparedWeapons>>,
        Option<Res<frame::LuiMenus>>,
        Option<Res<assets::T6HudIcons>>,
        NonSend<crate::lui_hud::LuiHudSlot>,
    ),
    mut images: ResMut<Assets<Image>>,
    mut cache: Local<HashMap<String, Option<Handle<Image>>>>,
    mut most: Local<HashMap<u32, i32>>,
    mut shown: Local<Option<HudValues>>,
) {
    if !modern_hud_enabled() {
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
    let gun = gun_picture(&mut cache, icons, &mut images, &values.weapon);
    let offhands: Vec<(bool, Option<Handle<Image>>, u32)> = values
        .offhand
        .iter()
        .filter_map(|(lethal, w, n)| {
            let (_, name) = offhand_picture(w)?;
            Some((*lethal, picture(&mut cache, icons, &mut images, name), *n))
        })
        .collect();
    let streak_pics: Vec<Option<Handle<Image>>> =
        values.streaks.iter().map(|s| picture(&mut cache, icons, &mut images, &s.icon)).collect();

    // The two sides' Black Ops emblems (Black Ops and Spetsnaz, read from
    // Black Ops' own files; plain squares until they are in).
    let mut emblem = |name: &str| -> Option<Handle<Image>> {
        if let Some(Some(h)) = cache.get(name) {
            return Some(h.clone());
        }
        let owned = name.to_owned();
        let img = crate::foreign_maps::picture_async(name, true, move || {
            crate::foreign_maps::read_game_image("t5", &[owned])
        })?;
        let h = images.add((*img).clone());
        cache.insert(name.to_owned(), Some(h.clone()));
        Some(h)
    };
    let emblems = if values.ffa {
        (None, None)
    } else if values.axis {
        (emblem("faction_128_spetsnaz"), emblem("faction_128_specops"))
    } else {
        (emblem("faction_128_specops"), emblem("faction_128_spetsnaz"))
    };
    commands
        .spawn((
            HudRoot,
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
            // --- Top centre: the score bar. His side on the left, theirs on
            // the right: score, bar (filling from the middle out), team name
            // over it, the team's Black Ops emblem, and the clock between
            // the emblems with WINNING / LOSING / TIED under it. ---
            root.spawn(Node {
                position_type: PositionType::Absolute,
                top: Val::Percent(1.4),
                width: Val::Percent(100.0),
                flex_direction: FlexDirection::Column,
                align_items: AlignItems::Center,
                row_gap: px(4.0),
                ..default()
            })
            .with_children(|top| {
                top.spawn(Node {
                    flex_direction: FlexDirection::Row,
                    align_items: AlignItems::FlexEnd,
                    column_gap: px(8.0),
                    ..default()
                })
                .with_children(|bar| {
                    let limit = values.limit.max(1) as f32;
                    let part = |score: i32| {
                        if values.limit > 0 { (score as f32 / limit).clamp(0.0, 1.0) } else { 0.0 }
                    };
                    let (mine_name, theirs_name) = if values.ffa {
                        ("YOU", "LEADER")
                    } else if values.axis {
                        ("SPETSNAZ", "BLACK OPS")
                    } else {
                        ("BLACK OPS", "SPETSNAZ")
                    };
                    let side = |bar: &mut ChildSpawnerCommands, mine: bool| {
                        let (score, name, color) = if mine {
                            (values.yours, mine_name, BLUE)
                        } else {
                            (values.enemy, theirs_name, RED)
                        };
                        bar.spawn(Node {
                            flex_direction: FlexDirection::Column,
                            align_items: if mine { AlignItems::FlexEnd } else { AlignItems::FlexStart },
                            row_gap: px(4.0),
                            ..default()
                        })
                        .with_children(|col| {
                            col.spawn(Node::default()).with_children(|n| {
                                text(n, bo2, &font, "Default", name, 22.0 * k, WHITE);
                            });
                            col.spawn(Node {
                                flex_direction: if mine { FlexDirection::Row } else { FlexDirection::RowReverse },
                                align_items: AlignItems::Center,
                                column_gap: px(10.0),
                                ..default()
                            })
                            .with_children(|row| {
                                row.spawn(Node::default()).with_children(|n| {
                                    text(n, bo2, &font, "Default", &score.to_string(), 22.0 * k, WHITE);
                                });
                                row.spawn((
                                    BackgroundColor(Color::srgba(0.35, 0.37, 0.4, 0.75)),
                                    Node {
                                        width: px(260.0),
                                        height: px(12.0),
                                        justify_content: if mine { JustifyContent::FlexEnd } else { JustifyContent::FlexStart },
                                        ..default()
                                    },
                                ))
                                .with_children(|t| {
                                    t.spawn((
                                        BackgroundColor(color),
                                        Node {
                                            width: Val::Percent(part(score) * 100.0),
                                            height: Val::Percent(100.0),
                                            ..default()
                                        },
                                    ));
                                });
                            });
                        });
                    };
                    let emblem = |bar: &mut ChildSpawnerCommands, pic: Option<Handle<Image>>, color: Color| {
                        bar.spawn((
                            BackgroundColor(PANEL),
                            Node {
                                width: px(58.0),
                                height: px(58.0),
                                justify_content: JustifyContent::Center,
                                align_items: AlignItems::Center,
                                ..default()
                            },
                        ))
                        .with_children(|b| match pic {
                            Some(image) => {
                                b.spawn((ImageNode::new(image), Node { width: px(52.0), height: px(52.0), ..default() }));
                            }
                            None => {
                                b.spawn((BackgroundColor(color), Node { width: px(30.0), height: px(30.0), ..default() }));
                            }
                        });
                    };
                    side(&mut *bar, true);
                    emblem(&mut *bar, emblems.0.clone(), BLUE);
                    bar.spawn((
                        BackgroundColor(PANEL),
                        Node {
                            min_width: px(120.0),
                            height: px(58.0),
                            justify_content: JustifyContent::Center,
                            align_items: AlignItems::Center,
                            ..default()
                        },
                    ))
                    .with_children(|c| {
                        let t = values.timeleft.map_or_else(String::new, clock);
                        let color = if values.timeleft.is_some_and(|t| t <= 30) { LOW } else { WHITE };
                        text(c, bo2, &font, "Default", &t, 38.0 * k, color);
                    });
                    emblem(&mut *bar, emblems.1.clone(), RED);
                    side(&mut *bar, false);
                });
                let (status, color) = match values.yours.cmp(&values.enemy) {
                    std::cmp::Ordering::Greater => ("WINNING", BLUE),
                    std::cmp::Ordering::Less => ("LOSING", RED),
                    std::cmp::Ordering::Equal => ("TIED", DIM),
                };
                top.spawn(Node::default()).with_children(|c| {
                    text(c, bo2, &font, "Default", status, 20.0 * k, color);
                });
            });

            // --- Bottom left: his name over a thin bar. ---
            root.spawn(Node {
                position_type: PositionType::Absolute,
                left: Val::Percent(1.2),
                bottom: Val::Percent(2.6),
                flex_direction: FlexDirection::Column,
                row_gap: px(4.0),
                ..default()
            })
            .with_children(|c| {
                c.spawn(Node::default()).with_children(|n| {
                    text(n, bo2, &font, "Default", &values.name, 20.0 * k, DIM);
                });
                let part = if values.limit > 0 {
                    (values.score as f32 / values.limit.max(1) as f32).clamp(0.0, 1.0)
                } else {
                    1.0
                };
                c.spawn((BackgroundColor(TRACK), Node { width: px(210.0), height: px(4.0), ..default() }))
                    .with_children(|t| {
                        t.spawn((
                            BackgroundColor(WHITE),
                            Node {
                                width: Val::Percent(part * 100.0),
                                height: Val::Percent(100.0),
                                ..default()
                            },
                        ));
                    });
            });

            // --- Bottom right: scorestreaks, gun name, attachments, ammo. ---
            let corner = Corner { values: &values, gun: gun.clone(), offhands: offhands.clone(), streak_pics: streak_pics.clone() };
            spawn_corner(root, bo2, &font, k, &corner);
        });
    *shown = Some(values);
}

// --- The bottom right corner. ---

/// What the bottom right corner draws: the values and their pictures.
struct Corner<'a> {
    values: &'a HudValues,
    gun: Option<(Handle<Image>, f32)>,
    offhands: Vec<(bool, Option<Handle<Image>>, u32)>,
    /// Each scorestreak's picture, in `values.streaks`' order.
    streak_pics: Vec<Option<Handle<Image>>>,
}

/// The ready scorestreak's square: its glow pulses (`pulse_ready`).
#[derive(Component)]
struct ReadyPulse;

/// The bottom right corner: his scorestreak column on the right edge, the
/// gun's name (his grenades beside it), then its attachments in gold, one
/// per line, left of the ammo box.
fn spawn_corner(
    root: &mut ChildSpawnerCommands,
    bo2: Option<&crate::bo2_font::Bo2Fonts>,
    font: &GameUiFont,
    k: f32,
    c: &Corner,
) {
    let px = |v: f32| Val::Px(v * k);
    let values = c.values;
    let shows_ammo = !values.not_a_gun && (values.clip > 0 || values.stock > 0 || values.clip_left.is_some());
    root.spawn(Node {
        position_type: PositionType::Absolute,
        right: Val::Percent(1.2),
        bottom: Val::Percent(2.6),
        flex_direction: FlexDirection::Column,
        align_items: AlignItems::FlexEnd,
        row_gap: px(6.0),
        ..default()
    })
    .with_children(|corner| {
        // The scorestreak column: dearest at the top, as BO2's.
        if !values.streaks.is_empty() {
            corner
                .spawn(Node {
                    flex_direction: FlexDirection::Column,
                    align_items: AlignItems::FlexEnd,
                    row_gap: px(5.0),
                    margin: UiRect::bottom(px(4.0)),
                    ..default()
                })
                .with_children(|col| {
                    for (s, pic) in values.streaks.iter().zip(c.streak_pics.iter()) {
                        streak_square(col, k, s, pic.clone());
                    }
                });
        }
        // The gun's name, his grenades to its left.
        let name = shows_ammo && !values.weapon_label.is_empty();
        if name || !c.offhands.is_empty() {
            corner
                .spawn(Node {
                    flex_direction: FlexDirection::Row,
                    align_items: AlignItems::Center,
                    column_gap: px(12.0),
                    ..default()
                })
                .with_children(|line| {
                    if !c.offhands.is_empty() {
                        offhand_row(line, bo2, font, k, &c.offhands);
                    }
                    if name {
                        line.spawn(Node::default()).with_children(|n| {
                            text(n, bo2, font, "Default", &values.weapon_label, 22.0 * k, WHITE);
                        });
                    }
                });
        }
        if !shows_ammo {
            return;
        }
        // The attachments (gold, one per line, right-aligned) beside the
        // ammo box.
        corner
            .spawn(Node {
                flex_direction: FlexDirection::Row,
                align_items: AlignItems::Center,
                column_gap: px(12.0),
                ..default()
            })
            .with_children(|row| {
                if !values.attachments.is_empty() {
                    row.spawn(Node {
                        flex_direction: FlexDirection::Column,
                        align_items: AlignItems::FlexEnd,
                        row_gap: px(1.0),
                        ..default()
                    })
                    .with_children(|list| {
                        for a in &values.attachments {
                            list.spawn(Node::default()).with_children(|n| {
                                text(n, bo2, font, "Default", a, 19.0 * k, GOLD);
                            });
                        }
                    });
                }
                row.spawn((
                    BackgroundColor(PANEL),
                    Node {
                        flex_direction: FlexDirection::Row,
                        align_items: AlignItems::Center,
                        column_gap: px(14.0),
                        padding: UiRect::axes(px(12.0), px(6.0)),
                        ..default()
                    },
                ))
                .with_children(|row| {
                    let clip_color = if values.low_clip || values.clip == 0 { LOW } else { WHITE };
                    if let Some(left) = values.clip_left {
                        text(row, bo2, font, "Default", &left.to_string(), 44.0 * k, if left == 0 { LOW } else { WHITE });
                    }
                    text(row, bo2, font, "Default", &values.clip.to_string(), 44.0 * k, clip_color);
                    let stock_color = if values.stock == 0 { LOW } else { DIM };
                    text(row, bo2, font, "Default", &values.stock.to_string(), 26.0 * k, stock_color);
                    if let Some((image, wide)) = c.gun.clone() {
                        row.spawn((
                            ImageNode::new(image),
                            Node { width: px(36.0 * wide), height: px(36.0), ..default() },
                        ));
                    }
                });
            });
    });
}

/// His grenades and equipment: each picture and its count.
fn offhand_row(
    line: &mut ChildSpawnerCommands,
    bo2: Option<&crate::bo2_font::Bo2Fonts>,
    font: &GameUiFont,
    k: f32,
    offhands: &[(bool, Option<Handle<Image>>, u32)],
) {
    let px = |v: f32| Val::Px(v * k);
    line.spawn((
        BackgroundColor(PANEL),
        Node {
            flex_direction: FlexDirection::Row,
            align_items: AlignItems::Center,
            column_gap: px(10.0),
            padding: UiRect::axes(px(10.0), px(4.0)),
            ..default()
        },
    ))
    .with_children(|row| {
        for lethal in [false, true] {
            for (_, pic, n) in offhands.iter().filter(|(l, _, _)| *l == lethal) {
                row.spawn(Node {
                    flex_direction: FlexDirection::Row,
                    align_items: AlignItems::Center,
                    column_gap: px(3.0),
                    ..default()
                })
                .with_children(|g| {
                    let tint = if *n == 0 { Color::srgba(1.0, 1.0, 1.0, 0.3) } else { WHITE };
                    match pic.clone() {
                        Some(image) => {
                            g.spawn((
                                ImageNode { image, color: tint, ..default() },
                                Node { width: px(30.0), height: px(30.0), ..default() },
                            ));
                        }
                        None => {
                            g.spawn((
                                BackgroundColor(tint.with_alpha(0.4)),
                                Node { width: px(22.0), height: px(22.0), ..default() },
                            ));
                        }
                    }
                    text(g, bo2, font, "Default", &n.to_string(), 18.0 * k, DIM);
                });
            }
        }
    });
}

/// One scorestreak's square: a dark translucent square with its picture.
/// Ready: lit, an accent edge and a pulsing accent glow. Next: a pale fill
/// rising from the bottom as he earns toward it, and a thin accent bar
/// along the bottom. Further off: faint. Used: fainter still.
fn streak_square(col: &mut ChildSpawnerCommands, k: f32, s: &Streak, pic: Option<Handle<Image>>) {
    let px = |v: f32| Val::Px(v * k);
    let side = STREAK_PX * k;
    let (icon_alpha, edge) = match s.state {
        StreakState::Ready => (1.0, ACCENT),
        StreakState::Next(_) => (0.62, Color::srgba(1.0, 1.0, 1.0, 0.16)),
        StreakState::Locked => (0.32, Color::srgba(1.0, 1.0, 1.0, 0.08)),
        StreakState::Used => (0.16, Color::NONE),
    };
    let mut square = col.spawn((
        BackgroundColor(PANEL),
        BorderColor::all(edge),
        Node {
            width: Val::Px(side),
            height: Val::Px(side),
            border: UiRect::all(px(if s.state == StreakState::Ready { 2.0 } else { 1.0 })),
            justify_content: JustifyContent::Center,
            align_items: AlignItems::Center,
            overflow: Overflow::clip(),
            ..default()
        },
    ));
    square.with_children(|sq| {
        match s.state {
            StreakState::Ready => {
                // The glow under the picture (`pulse_ready` breathes it).
                sq.spawn((
                    ReadyPulse,
                    BackgroundColor(ACCENT.with_alpha(0.3)),
                    Node {
                        position_type: PositionType::Absolute,
                        left: Val::Px(0.0),
                        top: Val::Px(0.0),
                        width: Val::Percent(100.0),
                        height: Val::Percent(100.0),
                        ..default()
                    },
                ));
            }
            StreakState::Next(part) => {
                sq.spawn((
                    BackgroundGradient::from(LinearGradient::to_top(vec![
                        ColorStop::new(Color::srgba(1.0, 1.0, 1.0, 0.24), Val::Percent(0.0)),
                        ColorStop::new(Color::srgba(1.0, 1.0, 1.0, 0.08), Val::Percent(100.0)),
                    ])),
                    Node {
                        position_type: PositionType::Absolute,
                        left: Val::Px(0.0),
                        bottom: Val::Px(0.0),
                        width: Val::Percent(100.0),
                        height: Val::Percent(part * 100.0),
                        ..default()
                    },
                ));
                sq.spawn((
                    BackgroundColor(ACCENT),
                    Node {
                        position_type: PositionType::Absolute,
                        left: Val::Px(0.0),
                        bottom: Val::Px(0.0),
                        width: Val::Percent(part * 100.0),
                        height: px(3.0),
                        ..default()
                    },
                ));
            }
            StreakState::Locked | StreakState::Used => {}
        }
        let tint = Color::srgba(1.0, 1.0, 1.0, icon_alpha);
        match pic {
            Some(image) => {
                sq.spawn((
                    ImageNode { image, color: tint, ..default() },
                    Node { width: Val::Percent(84.0), height: Val::Percent(84.0), ..default() },
                ));
            }
            None => {
                sq.spawn((
                    BackgroundColor(tint.with_alpha(icon_alpha * 0.35)),
                    Node { width: Val::Percent(46.0), height: Val::Percent(46.0), ..default() },
                ));
            }
        }
    });
}

/// The ready scorestreak's glow breathes (a 1.2-second beat).
fn pulse_ready(time: Res<Time<Real>>, mut q: Query<&mut BackgroundColor, With<ReadyPulse>>) {
    let beat = (time.elapsed_secs() * std::f32::consts::TAU / 1.2).sin() * 0.5 + 0.5;
    for mut bg in &mut q {
        bg.0 = ACCENT.with_alpha(0.14 + 0.30 * beat);
    }
}

// --- The minimap (top left), square, with a compass tape on top. ---

/// The minimap BO2's HUD worked out this frame (`lui_hud`): his view of the
/// map, whether he is looking through a scope, and when it came.
static MINIMAP: Mutex<Option<(crate::lui_frontend::MinimapView, bool, Instant)>> = Mutex::new(None);

pub(crate) fn publish_minimap(view: Option<&crate::lui_frontend::MinimapView>, scoped: bool) {
    if let Ok(mut m) = MINIMAP.lock() {
        *m = view.map(|v| (v.clone(), scoped, Instant::now()));
    }
}

#[derive(Component)]
struct MinimapRoot;

/// The minimap's side on a 1080-line screen.
const MAP_PX: f32 = 236.0;

/// The map turned so he faces up, on a dark ground with a thin edge.
fn square_map(view: &crate::lui_frontend::MinimapView, src: &Image, line: Option<&Image>, n: usize) -> Vec<u8> {
    let (sw, sh) = (src.texture_descriptor.size.width as usize, src.texture_descriptor.size.height as usize);
    let line = line.and_then(|i| {
        let (lw, lh) = (i.texture_descriptor.size.width as usize, i.texture_descriptor.size.height as usize);
        Some((i.data.as_deref()?, lw, lh))
    });
    let mut px = src.data.as_deref().map(|d| view.render(d, sw, sh, n, line)).unwrap_or_else(|| vec![0; n * n * 4]);
    for j in 0..n {
        for i in 0..n {
            let o = (j * n + i) * 4;
            let edge = i == 0 || j == 0 || i + 1 == n || j + 1 == n;
            let ma = f32::from(px[o + 3]) / 255.0;
            let ground = 0.6;
            let a = ma + ground * (1.0 - ma);
            for k in 0..3 {
                let c = if a > 0.0 { f32::from(px[o + k]) * ma / a } else { 0.0 };
                // A cooler, darker map than BO2's (the reference's grey).
                let c = c * 0.85;
                px[o + k] = if edge { 150 } else { c.round().clamp(0.0, 255.0) as u8 };
            }
            px[o + 3] = if edge { 200 } else { (a * 255.0).round() as u8 };
        }
    }
    px
}

/// A map picture BO2's zones do not carry (a map from another game: its
/// `compass_map_<map>`), read from the other games' IWDs off the main
/// thread; `None` until it is in (or when no game has it).
fn foreign_picture(material: &str) -> Option<std::sync::Arc<Image>> {
    use std::sync::Arc;
    static READ: Mutex<Option<HashMap<String, Option<Arc<Image>>>>> = Mutex::new(None);
    let mut guard = READ.lock().ok()?;
    let read = guard.get_or_insert_with(HashMap::new);
    if let Some(found) = read.get(material) {
        return found.clone();
    }
    read.insert(material.to_owned(), None);
    let name = material.to_owned();
    std::thread::Builder::new()
        .name("minimap-picture".into())
        .spawn(move || {
            let found = asset_transport::games_root_from_env().ok().and_then(|root| {
                let trees = asset_transport::NamespaceTrees::discover(&root);
                [asset_core::AssetNamespace::Iw4, asset_core::AssetNamespace::Iw5, asset_core::AssetNamespace::T5]
                    .into_iter()
                    .filter_map(|ns| trees.main_for(ns))
                    .find_map(|main| asset_material::decode_ui_image_from_main(main, &name).ok().flatten())
            });
            diag::info!(Ui, "minimap: `{name}` {}", if found.is_some() { "read from its game" } else { "not found" });
            let image = found.map(|(w, h, rgba)| {
                Arc::new(Image::new(
                    Extent3d { width: w, height: h, depth_or_array_layers: 1 },
                    TextureDimension::D2,
                    rgba,
                    TextureFormat::Rgba8Unorm,
                    RenderAssetUsages::default(),
                ))
            });
            if let Ok(mut guard) = READ.lock()
                && let Some(read) = guard.as_mut()
            {
                read.insert(name, image);
            }
        })
        .ok();
    None
}

/// The compass points along the tape, clockwise from north.
const POINTS: [&str; 8] = ["N", "NE", "E", "SE", "S", "SW", "W", "NW"];

#[allow(clippy::too_many_arguments)]
fn minimap(
    mut commands: Commands,
    presented: Option<Res<PresentedSnapshot>>,
    local: Option<Res<LocalPresentClient>>,
    menus: Option<Res<frame::LuiMenus>>,
    icons: Option<Res<assets::T6HudIcons>>,
    font: Option<Res<GameUiFont>>,
    bo2: Option<Res<crate::bo2_font::Bo2Fonts>>,
    window: Query<&Window, With<PrimaryWindow>>,
    roots: Query<Entity, With<MinimapRoot>>,
    mut images: ResMut<Assets<Image>>,
    mut cache: Local<HashMap<String, Option<Handle<Image>>>>,
    mut map_image: Local<Option<Handle<Image>>>,
) {
    let view = MINIMAP
        .lock()
        .ok()
        .and_then(|m| m.clone())
        .filter(|(_, scoped, at)| !scoped && at.elapsed().as_secs_f32() < 0.5)
        .map(|(v, _, _)| v);
    let shown = modern_hud_enabled()
        && match (presented.as_deref(), local.as_deref()) {
            (Some(p), Some(l)) => match_values(p, l, menus.as_deref()).is_some(),
            _ => false,
        };
    let (Some(view), true, Some(font)) = (view, shown, font) else {
        for e in &roots {
            commands.entity(e).despawn();
        }
        return;
    };
    let icons = icons.as_deref();
    let bo2 = bo2.as_deref();
    let height = window.single().map_or(1080.0, Window::height);
    let k = (height / 1080.0).max(0.4);
    let size = MAP_PX * k;
    let n = (size.round() as usize).clamp(64, 400);
    // The map picture (when BO2's HUD pictures carry it; a map from another
    // game shows its marks on the dark ground until its own is read).
    let src = icons
        .and_then(|i| i.0.iter().find(|(k, _)| *k == view.material))
        .map(|(_, i)| i.clone())
        .or_else(|| foreign_picture(&view.material));
    let line = view
        .line
        .and_then(|_| icons?.0.iter().find(|(k, _)| *k == "compass_radarline"))
        .map(|(_, i)| i.clone());
    let pixels = match src.as_deref() {
        Some(src) => square_map(&view, src, line.as_deref(), n),
        None => square_map(&view, &Image::default(), None, n),
    };
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
    // The marks: enemies the radar shows (red), his team, him.
    let mark = (size * 0.085).max(8.0);
    let per_unit = size / view.range;
    let me_turn = view.heading(view.me.1);
    let half = size * 0.5 - mark * 0.5;
    let mut marks: Vec<(Option<Handle<Image>>, [f32; 2], f32, f32)> = Vec::new();
    let mut place = |name: &str, p: [f32; 2], yaw: f32, alpha: f32, marks: &mut Vec<_>| {
        let o = view.screen_offset(p);
        let at = [o[0] * per_unit, o[1] * per_unit];
        if at[0].abs() > half || at[1].abs() > half {
            return;
        }
        let pic = picture(&mut cache, icons, &mut images, name);
        marks.push((pic, at, view.heading(yaw) - me_turn, alpha));
    };
    for (p, yaw, firing, alpha) in &view.enemies {
        let name = if *firing { "compassping_enemyfiring" } else { "compassping_enemy" };
        place(name, *p, *yaw, *alpha, &mut marks);
    }
    for (p, yaw) in &view.friends {
        place("compassping_friendly_mp", *p, *yaw, 1.0, &mut marks);
    }
    place("compassping_player", view.me.0, view.me.1, 1.0, &mut marks);

    let node = Node {
        position_type: PositionType::Absolute,
        left: Val::Percent(1.2),
        top: Val::Percent(2.0),
        width: Val::Px(size),
        height: Val::Px(size + 22.0 * k),
        ..default()
    };
    let root = match roots.iter().next() {
        Some(e) => {
            commands.entity(e).despawn_related::<Children>();
            commands.entity(e).insert(node);
            e
        }
        None => commands.spawn((MinimapRoot, UiLayer::Hud, UiLayerVisibility, node)).id(),
    };
    let tape = 22.0 * k;
    commands.entity(root).with_children(|c| {
        // The compass tape: the points within 75 degrees of where he faces,
        // spread across the map's width.
        c.spawn((
            BackgroundColor(PANEL),
            Node {
                position_type: PositionType::Absolute,
                left: Val::Px(0.0),
                top: Val::Px(0.0),
                width: Val::Px(size),
                height: Val::Px(tape),
                overflow: Overflow::clip(),
                ..default()
            },
        ))
        .with_children(|t| {
            let span = 75f32.to_radians();
            for (i, label) in POINTS.iter().enumerate() {
                let dir = i as f32 * std::f32::consts::FRAC_PI_4;
                let mut d = dir - me_turn;
                while d > std::f32::consts::PI {
                    d -= std::f32::consts::TAU;
                }
                while d < -std::f32::consts::PI {
                    d += std::f32::consts::TAU;
                }
                if d.abs() > span {
                    continue;
                }
                let x = size * 0.5 + d / span * size * 0.5;
                t.spawn(Node {
                    position_type: PositionType::Absolute,
                    left: Val::Px(x - 12.0 * k),
                    top: Val::Px(2.0 * k),
                    width: Val::Px(24.0 * k),
                    justify_content: JustifyContent::Center,
                    ..default()
                })
                .with_children(|l| {
                    let color = if *label == "N" { WHITE } else { DIM };
                    text(l, bo2, &font, "Default", label, 15.0 * k, color);
                });
            }
            // The centre tick: where he faces.
            t.spawn((
                BackgroundColor(WHITE),
                Node {
                    position_type: PositionType::Absolute,
                    left: Val::Px(size * 0.5 - 1.0),
                    bottom: Val::Px(0.0),
                    width: Val::Px(2.0),
                    height: Val::Px(5.0 * k),
                    ..default()
                },
            ));
        });
        c.spawn((
            ImageNode::new(handle),
            Node {
                position_type: PositionType::Absolute,
                left: Val::Px(0.0),
                top: Val::Px(tape),
                width: Val::Px(size),
                height: Val::Px(size),
                ..default()
            },
        ));
        for (pic, at, turn, alpha) in marks {
            let node = Node {
                position_type: PositionType::Absolute,
                left: Val::Px(size * 0.5 + at[0] - mark * 0.5),
                top: Val::Px(tape + size * 0.5 + at[1] - mark * 0.5),
                width: Val::Px(mark),
                height: Val::Px(mark),
                ..default()
            };
            let turned = UiTransform { rotation: Rot2::radians(turn), ..UiTransform::IDENTITY };
            match pic {
                Some(image) => {
                    c.spawn((ImageNode { image, color: Color::srgba(1.0, 1.0, 1.0, alpha), ..default() }, node, turned));
                }
                None => {
                    c.spawn((BackgroundColor(Color::srgba(1.0, 0.3, 0.25, alpha)), node, turned));
                }
            }
        }
    });
}

pub(crate) fn register_modern_hud_systems(app: &mut App) {
    app.add_systems(
        Update,
        (modern_hud, minimap, pulse_ready)
            .after(crate::bo2_font::load_bo2_fonts)
            .in_set(ClientSet::Ui),
    );
}

pub mod preview {
    //! The bottom right corner (scorestreaks, gun name, attachments, ammo)
    //! with stand-in values, saved as pictures
    //! (`cargo run -p ui --example hud_preview -- <out dir> streaks`).

    use std::path::PathBuf;

    use bevy::asset::RenderAssetUsages;
    use bevy::render::render_resource::{Extent3d, TextureDimension, TextureFormat};
    use bevy::render::view::screenshot::{Screenshot, save_to_disk};

    use super::*;

    #[derive(Resource)]
    struct Plan {
        out: PathBuf,
        step: usize,
        frames: u32,
    }

    #[derive(Component)]
    struct Shot;

    /// A stand-in picture: a white silhouette on clear (0 a plane, 1 a
    /// crate, 2 a bolt, 3 a gun).
    fn stand_in(kind: u8, w: u32, h: u32) -> Image {
        let mut px = vec![0u8; (w * h * 4) as usize];
        for y in 0..h {
            for x in 0..w {
                let (u, v) = (x as f32 / w as f32, y as f32 / h as f32);
                let on = match kind {
                    0 => {
                        let wing = (v - 0.5).abs() < 0.07 && (u - 0.5).abs() < 0.42;
                        let body = (u - 0.5).abs() < 0.06 && v > 0.15 && v < 0.85;
                        let tail = (v - 0.8).abs() < 0.05 && (u - 0.5).abs() < 0.18;
                        wing || body || tail
                    }
                    1 => {
                        let edge = (u - 0.5).abs().max((v - 0.5).abs());
                        edge < 0.36 && (edge > 0.28 || (u - v).abs() < 0.05 || (u + v - 1.0).abs() < 0.05)
                    }
                    2 => {
                        let a = (v > 0.12 && v < 0.55) && ((u - (0.62 - v * 0.5)).abs() < 0.09);
                        let b = (v >= 0.45 && v < 0.9) && ((u - (0.78 - v * 0.5)).abs() < 0.09);
                        a || b
                    }
                    _ => {
                        let body = v > 0.3 && v < 0.55 && u > 0.05 && u < 0.95;
                        let grip = v >= 0.55 && v < 0.95 && u > 0.55 && u < 0.68;
                        let mag = v >= 0.55 && v < 0.85 && u > 0.35 && u < 0.45;
                        body || grip || mag
                    }
                };
                if on {
                    let o = ((y * w + x) * 4) as usize;
                    px[o..o + 4].copy_from_slice(&[255, 255, 255, 255]);
                }
            }
        }
        Image::new(Extent3d { width: w, height: h, depth_or_array_layers: 1 }, TextureDimension::D2, px, TextureFormat::Rgba8UnormSrgb, RenderAssetUsages::default())
    }

    /// A stand-in scene: a pale concrete floor under a darker sky (the
    /// reference's daylight), so the corner is seen over something bright.
    fn scene() -> Image {
        let (w, h) = (480u32, 270u32);
        let mut px = vec![0u8; (w * h * 4) as usize];
        for y in 0..h {
            for x in 0..w {
                let (u, v) = (x as f32 / w as f32, y as f32 / h as f32);
                let n = ((x * 7919 + y * 104_729) % 97) as f32 / 97.0;
                let c = if v < 0.38 {
                    [0.36 + 0.1 * u, 0.40 + 0.1 * u, 0.46 + 0.08 * u]
                } else {
                    let g = 0.52 + 0.18 * (v - 0.38) + 0.05 * n;
                    [g, g * 0.99, g * 0.97]
                };
                let o = ((y * w + x) * 4) as usize;
                for k in 0..3 {
                    px[o + k] = (c[k].clamp(0.0, 1.0) * 255.0) as u8;
                }
                px[o + 3] = 255;
            }
        }
        Image::new(Extent3d { width: w, height: h, depth_or_array_layers: 1 }, TextureDimension::D2, px, TextureFormat::Rgba8UnormSrgb, RenderAssetUsages::default())
    }

    fn values(i: usize) -> HudValues {
        let base = HudValues {
            weapon: "m8a1_mp".into(),
            weapon_label: "M8A1".into(),
            clip: 45,
            stock: 180,
            offhand: vec![(true, "frag_grenade_mp".into(), 1), (false, "concussion_grenade_mp".into(), 2)],
            attachments: vec!["Suppressor".into(), "Extended Clip".into()],
            ..default()
        };
        match i {
            // Earning toward the first one.
            0 => HudValues { streaks: parse_streaks("ks_c:750:0;ks_b:525:0;ks_a:425:0", 300), ..base },
            // The first ready, the second filling up.
            1 => HudValues {
                streaks: parse_streaks("ks_c:750:0;ks_b:525:0;ks_a:425:1", 480),
                attachments: vec!["Reflex Sight".into(), "Fast Mag".into(), "Fore Grip".into()],
                ..base
            },
            // Two used, the dearest ready; a low clip.
            2 => HudValues {
                streaks: parse_streaks("ks_c:750:1;ks_b:525:0;ks_a:425:0", 800),
                clip: 6,
                low_clip: true,
                ..base
            },
            // Holding a scorestreak's remote: no ammo, the column stays.
            _ => HudValues {
                streaks: parse_streaks("ks_c:750:0;ks_b:525:1;ks_a:425:0", 610),
                not_a_gun: true,
                weapon: "killstreak_remote_turret_mp".into(),
                ..base
            },
        }
    }

    fn step(
        mut commands: Commands,
        mut plan: ResMut<Plan>,
        fonts: Option<Res<crate::shell::theme::ShellFonts>>,
        shots: Query<Entity, With<Shot>>,
        mut images: ResMut<Assets<Image>>,
        mut exit: MessageWriter<AppExit>,
    ) {
        let Some(fonts) = fonts else { return };
        plan.frames += 1;
        if plan.frames < 30 {
            return;
        }
        plan.frames = 0;
        let i = plan.step / 2;
        let names = ["01_earning", "02_ready_and_next", "03_used_and_low_clip", "04_holding_streak"];
        if i >= names.len() {
            exit.write(AppExit::Success);
            return;
        }
        if plan.step % 2 == 1 {
            let path = plan.out.join(format!("{}.png", names[i]));
            commands.spawn(Screenshot::primary_window()).observe(save_to_disk(path));
            plan.step += 1;
            return;
        }
        for e in &shots {
            commands.entity(e).despawn();
        }
        let font = GameUiFont(fonts.bold.clone());
        let values = values(i);
        let pic = |images: &mut Assets<Image>, kind: u8| Some(images.add(stand_in(kind, 64, 64)));
        let streak_pics = values
            .streaks
            .iter()
            .map(|s| match s.icon.as_str() {
                "ks_a" => pic(&mut images, 0),
                "ks_b" => pic(&mut images, 1),
                _ => pic(&mut images, 2),
            })
            .collect();
        let corner = Corner {
            values: &values,
            gun: Some((images.add(stand_in(3, 128, 64)), 2.0)),
            offhands: values.offhand.iter().map(|(l, _, n)| (*l, pic(&mut images, 1), *n)).collect(),
            streak_pics,
        };
        let bg = images.add(scene());
        commands
            .spawn((
                Shot,
                ImageNode::new(bg),
                Node { position_type: PositionType::Absolute, width: Val::Percent(100.0), height: Val::Percent(100.0), ..default() },
            ))
            .with_children(|root| spawn_corner(root, None, &font, 1.0, &corner));
        plan.step += 1;
    }

    /// The corner with stand-in values, a picture of each saved in `out`.
    pub fn install(app: &mut App, out: PathBuf) {
        app.add_systems(Startup, crate::shell::theme::load_fonts);
        app.insert_resource(Plan { out, step: 0, frames: 0 });
        app.add_systems(Update, (step, pulse_ready).chain());
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn streaks_read_ready_next_used() {
        // Dearest first; the cheapest held, the middle one next.
        let s = parse_streaks("hud_ks_c:750:0;hud_ks_b:525:0;hud_ks_a:425:1", 480);
        assert_eq!(s.len(), 3);
        assert_eq!(s[0].state, StreakState::Locked);
        assert!(matches!(s[1].state, StreakState::Next(p) if (p - 480.0 / 525.0).abs() < 1e-4));
        assert_eq!(s[2].state, StreakState::Ready);
        // Earned and used: past its cost and not held.
        let s = parse_streaks("c:750:1;b:525:0;a:425:0", 800);
        assert_eq!(s[0].state, StreakState::Ready);
        assert_eq!(s[1].state, StreakState::Used);
        assert_eq!(s[2].state, StreakState::Used);
        // The default picks (no `held`): ready once his momentum is in.
        let s = parse_streaks("c:750;b:525;a:425", 500);
        assert_eq!(s[2].state, StreakState::Ready);
        assert!(matches!(s[1].state, StreakState::Next(_)));
        assert!(parse_streaks("", 0).is_empty());
    }

    #[test]
    fn attachments_have_names() {
        assert_eq!(attachment_fallback("extclip"), "Extended Clip");
        assert_eq!(attachment_fallback("silencer"), "Suppressor");
        assert_eq!(attachment_fallback("new_thing"), "NEW THING");
    }

    #[test]
    fn clock_reads_minutes_and_seconds() {
        assert_eq!(clock(550), "9:10");
        assert_eq!(clock(5), "0:05");
    }

    #[test]
    fn equipment_has_a_slot() {
        assert_eq!(offhand_picture("frag_grenade_mp").map(|p| p.0), Some(true));
        assert_eq!(offhand_picture("flash_grenade_mp").map(|p| p.0), Some(false));
        assert!(offhand_picture("an94_mp").is_none());
    }
}
