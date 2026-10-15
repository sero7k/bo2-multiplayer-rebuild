//! bo2mp: the Warzone HUD, drawn instead of the modern HUD's score bar,
//! health line and ammo corner while Resurgence runs (other modes keep
//! theirs). Modern Warfare III's (2023) Warzone HUD in its layout, with its
//! own pictures read from the install (`jup_hud`) drawn in WARZONE's
//! colours (`shell::wz_theme::HUD`); a picture the install lacks is a plain
//! shape.
//!
//! - Top left, under the minimap (which rings the safe zone and the next
//!   one): the circle's panel (the circle and its time until it closes,
//!   its time closing, the infil's countdown, FINAL CIRCLE), then the
//!   squad's contract and its time.
//! - Top right: players and squads left, his kills, his cash.
//! - Bottom left: his squadmates (slot colour and number, name, armor and
//!   health, down / redeploying / out), then him: name, armor plates in
//!   segments over his health, plates carried, the self-revive kit.
//! - Bottom right: the gun's name, the clip in big figures over the rest,
//!   the ammo's picture, his grenades.
//! - Centre: the gas warning, the redeploy countdown in its ring (or
//!   redeploy disabled); hit markers on armor, a plate breaking, the kill
//!   marker and the ELIMINATED splash (`warzone_markers`).
//!
//! Everything is read from `bo2mp_br`, published to his client each tick
//! from the Resurgence script's state (`br_resurgence.gsc`
//! `br_hud_publish`).

use bevy::prelude::*;

use crate::layers::{GameUiFont, UiLayer, UiLayerVisibility};
use crate::shell::wz_theme::{HUD, HUD_WORDS};

/// One squadmate as the HUD shows him.
#[derive(Clone, Debug, Default, PartialEq)]
pub(crate) struct Mate {
    /// 'a' alive, 'r' redeploying, 'd' down (dead, not coming back on his
    /// own), 'o' out.
    pub state: char,
    pub health: i32,
    pub armor: i32,
    pub name: String,
}

/// The Resurgence values the script publishes for him (`bo2mp_br`).
#[derive(Clone, Debug, Default, PartialEq)]
pub(crate) struct Br {
    /// "lobby", "infil", "wait" (the circle holds), "close" (closing),
    /// "final".
    pub state: String,
    /// The circle (1-based) and how many there are.
    pub phase: i32,
    pub phases: i32,
    /// Milliseconds to the infil's end, the circle closing or closed.
    pub ms: i32,
    pub squads_left: i32,
    pub players_left: i32,
    pub my_squad: i32,
    pub cash: i32,
    pub kills: i32,
    pub armor: i32,
    pub plate_hp: i32,
    pub plate_slots: i32,
    pub plates_carried: i32,
    pub plates_max: i32,
    /// The redeploy wait now (ms; < 0: redeploys are off).
    pub redeploy_wait: i32,
    /// His own redeploy, ms left (< 0: not waiting).
    pub redeploy_in: i32,
    pub out: bool,
    pub self_revive: bool,
    pub in_gas: bool,
    /// How far outside the safe zone (world units).
    pub gas_distance: i32,
    pub gas_mask: bool,
    pub gas_mask_ms: i32,
    pub contract: String,
    pub contract_ms: i32,
    /// The safe zone (x, y, radius) and the next one.
    pub circle: Option<[f32; 3]>,
    pub next_circle: Option<[f32; 3]>,
    /// Counters of his hits on armor, plates broken and kills (a rise is
    /// a new one), and the last one he killed.
    pub armor_hits: i32,
    pub breaks: i32,
    pub kill_events: i32,
    pub victim: String,
    pub mates: Vec<Mate>,
}

/// `key=value` fields by `|`; each `m=<state>,<health>,<armor>,<name>` is a
/// squadmate (the name may hold commas). `None` for "".
pub(crate) fn parse(s: &str) -> Option<Br> {
    if s.trim().is_empty() {
        return None;
    }
    let mut b = Br { redeploy_in: -1, contract_ms: -1, ..Br::default() };
    for field in s.split('|') {
        let Some((k, v)) = field.split_once('=') else { continue };
        let n = || v.trim().parse::<f32>().map_or(0, |f| f as i32);
        match k {
            "st" => b.state = v.to_owned(),
            "ph" => b.phase = n(),
            "np" => b.phases = n(),
            "ms" => b.ms = n(),
            "sq" => b.squads_left = n(),
            "pl" => b.players_left = n(),
            "my" => b.my_squad = n(),
            "ca" => b.cash = n(),
            "ki" => b.kills = n(),
            "ar" => b.armor = n(),
            "ah" => b.plate_hp = n(),
            "ps" => b.plate_slots = n(),
            "pc" => b.plates_carried = n(),
            "pm" => b.plates_max = n(),
            "rd" => b.redeploy_wait = n(),
            "rt" => b.redeploy_in = n(),
            "out" => b.out = n() != 0,
            "sr" => b.self_revive = n() != 0,
            "gas" => b.in_gas = n() != 0,
            "gd" => b.gas_distance = n(),
            "mask" => b.gas_mask = n() != 0,
            "mm" => b.gas_mask_ms = n(),
            "ct" => b.contract = v.to_owned(),
            "cm" => b.contract_ms = n(),
            "circle" => {
                let f: Vec<f32> = v.split(',').filter_map(|x| x.trim().parse().ok()).collect();
                if f.len() >= 3 {
                    b.circle = Some([f[0], f[1], f[2]]);
                }
                if f.len() >= 6 {
                    b.next_circle = Some([f[3], f[4], f[5]]);
                }
            }
            "eh" => b.armor_hits = n(),
            "eb" => b.breaks = n(),
            "ek" => b.kill_events = n(),
            "ev" => b.victim = v.to_owned(),
            "m" => {
                let mut p = v.splitn(4, ',');
                let state = p.next().and_then(|s| s.chars().next()).unwrap_or('a');
                let num = |x: Option<&str>| x.and_then(|x| x.trim().parse::<f32>().ok()).map_or(0, |f| f as i32);
                let health = num(p.next());
                let armor = num(p.next());
                b.mates.push(Mate { state, health, armor, name: p.next().unwrap_or("").to_owned() });
            }
            _ => {}
        }
    }
    Some(b)
}

/// "1:05"; "0:00" at or under nothing.
pub(crate) fn clock_ms(ms: i32) -> String {
    let s = (ms.max(0) + 999) / 1000;
    format!("{}:{:02}", s / 60, s % 60)
}

/// "$12,300".
pub(crate) fn money(cash: i32) -> String {
    let digits = cash.max(0).to_string();
    let mut out = String::new();
    for (i, c) in digits.chars().enumerate() {
        if i > 0 && (digits.len() - i) % 3 == 0 {
            out.push(',');
        }
        out.push(c);
    }
    format!("${out}")
}

/// Each plate segment's fill, 0..=1, for `armor` over `slots` plates of
/// `plate_hp` each.
pub(crate) fn plate_fill(armor: i32, plate_hp: i32, slots: i32) -> Vec<f32> {
    let hp = plate_hp.max(1) as f32;
    (0..slots.max(0)).map(|k| ((armor as f32 - k as f32 * hp) / hp).clamp(0.0, 1.0)).collect()
}

/// The circle panel's heading and time, and whether the gas is moving.
pub(crate) fn circle_line(b: &Br) -> (String, String, bool) {
    match b.state.as_str() {
        "lobby" => (HUD_WORDS.infil.to_owned(), String::new(), false),
        "infil" => (HUD_WORDS.infil.to_owned(), clock_ms(b.ms), false),
        "final" => (HUD_WORDS.final_circle.to_owned(), String::new(), false),
        "close" => (format!("{} {}", HUD_WORDS.circle_closing, b.phase), clock_ms(b.ms), true),
        _ => (format!("{} {}", HUD_WORDS.circle_closes, b.phase), clock_ms(b.ms), false),
    }
}

/// The contract's picture by its name (the script's label).
fn contract_picture(label: &str) -> &'static str {
    let l = label.to_ascii_lowercase();
    if l.contains("bounty") {
        "contract_bounty"
    } else if l.contains("recon") {
        "contract_recon"
    } else if l.contains("supply") {
        "contract_supply"
    } else if l.contains("wanted") {
        "contract_mostwanted"
    } else if l.contains("scav") {
        "contract_scavenger"
    } else {
        "contract"
    }
}

/// The ammo's picture for a gun.
fn ammo_picture(weapon: &str) -> &'static str {
    let w = weapon.to_ascii_lowercase();
    if ["870", "ksg", "saiga", "srm", "spas", "shotgun", "olympia", "striker", "_sh_"].iter().any(|k| w.contains(k)) {
        "ammo_shells"
    } else if ["dsr", "ballista", "svu", "as50", "sniper", "barrett", "l96", "_sn_"].iter().any(|k| w.contains(k)) {
        "ammo_sniper"
    } else if ["fiveseven", "fnp45", "beretta", "judge", "kard", "pistol", "_pi_", "colt", "makarov"].iter().any(|k| w.contains(k)) {
        "ammo_pistol"
    } else {
        "ammo_rifle"
    }
}

/// A Warzone HUD picture as a handle: loaded off the main thread once
/// (`None` until it is in, and for good when the install lacks it).
pub(crate) fn picture(
    cache: &mut std::collections::HashMap<String, Option<Handle<Image>>>,
    images: &mut Assets<Image>,
    name: &str,
) -> Option<Handle<Image>> {
    let key = format!("jup:hud:{name}");
    if let Some(Some(h)) = cache.get(&key) {
        return Some(h.clone());
    }
    let owned = name.to_owned();
    let img = crate::foreign_maps::picture_async(&key, true, move || crate::jup_hud::read_picture(&owned))?;
    let h = images.add((*img).clone());
    cache.insert(key, Some(h.clone()));
    Some(h)
}

/// What the Warzone HUD draws beside the script's values: his own from his
/// player state and his gun.
pub(crate) struct Inputs<'a> {
    pub br: &'a Br,
    pub name: &'a str,
    /// His health, 0..=100 % (0 when dead).
    pub health: i32,
    pub weapon: &'a str,
    pub weapon_label: &'a str,
    pub not_a_gun: bool,
    pub clip: i32,
    pub stock: i32,
    pub low_clip: bool,
    /// (lethal?, BO2's picture, count).
    pub offhands: Vec<(bool, Option<Handle<Image>>, u32)>,
    /// The screen's height.
    pub height: f32,
}

type TextFn<'a> = &'a mut dyn FnMut(&mut ChildSpawnerCommands, &str, f32, Color);

/// A picture `side` square tinted `color`, else a plain shape of that
/// colour (`round`: a dot).
fn glyph(c: &mut ChildSpawnerCommands, pic: Option<Handle<Image>>, side: f32, color: Color, round: bool) {
    match pic {
        Some(image) => {
            c.spawn((
                ImageNode { image, color, ..default() },
                Node { width: Val::Px(side), height: Val::Px(side), flex_shrink: 0.0, ..default() },
            ));
        }
        None => {
            let s = side * 0.6;
            c.spawn(Node {
                width: Val::Px(side),
                height: Val::Px(side),
                justify_content: JustifyContent::Center,
                align_items: AlignItems::Center,
                flex_shrink: 0.0,
                ..default()
            })
            .with_children(|b| {
                b.spawn((
                    BackgroundColor(color),
                    Node {
                        width: Val::Px(s),
                        height: Val::Px(s),
                        border_radius: BorderRadius::all(if round { Val::Percent(50.0) } else { Val::Px(s * 0.15) }),
                        ..default()
                    },
                ));
            });
        }
    }
}

/// A bar `w` x `h`: its track and `part` of it filled.
fn bar(c: &mut ChildSpawnerCommands, w: f32, h: f32, part: f32, fill: Color) {
    c.spawn((BackgroundColor(HUD.track), Node { width: Val::Px(w), height: Val::Px(h), flex_shrink: 0.0, ..default() }))
        .with_children(|t| {
            t.spawn((
                BackgroundColor(fill),
                Node { width: Val::Percent(part.clamp(0.0, 1.0) * 100.0), height: Val::Percent(100.0), ..default() },
            ));
        });
}

/// The plates as segments across `w`, `gap` apart.
fn plate_bar(c: &mut ChildSpawnerCommands, b: &Br, armor: i32, w: f32, h: f32, gap: f32) {
    let fills = plate_fill(armor, b.plate_hp, b.plate_slots);
    let n = fills.len().max(1) as f32;
    let seg = ((w - gap * (n - 1.0)) / n).max(1.0);
    c.spawn(Node { flex_direction: FlexDirection::Row, column_gap: Val::Px(gap), ..default() }).with_children(|r| {
        for f in fills {
            bar(r, seg, h, f, HUD.armor);
        }
    });
}

/// A squad slot's square: its colour and number.
fn slot(c: &mut ChildSpawnerCommands, text: TextFn, n: usize, side: f32, k: f32) {
    let color = HUD.squad[n % HUD.squad.len()];
    c.spawn((
        BackgroundColor(color),
        Node {
            width: Val::Px(side),
            height: Val::Px(side),
            justify_content: JustifyContent::Center,
            align_items: AlignItems::Center,
            flex_shrink: 0.0,
            ..default()
        },
    ))
    .with_children(|s| text(s, &(n + 1).to_string(), side * 0.8, Color::srgb(0.05, 0.05, 0.04)));
    let _ = k;
}

/// The Warzone HUD under the modern HUD's root (`hud_modern`, which keeps
/// the minimap and draws this instead of its own score bar, health line and
/// ammo corner while Resurgence runs).
pub(crate) fn spawn(
    root: &mut ChildSpawnerCommands,
    v: &Inputs,
    k: f32,
    pic: &mut dyn FnMut(&str) -> Option<Handle<Image>>,
    text: TextFn,
) {
    let b = v.br;
    let px = |x: f32| Val::Px(x * k);
    let alive = v.health > 0;

    // --- Top left, under the minimap: the circle, the contract. ---
    let map_bottom = v.height * 0.02 + (236.0 + 22.0 + 8.0) * k;
    let (heading, time, closing) = circle_line(b);
    let circle_pic = if b.state == "infil" { pic("plane") } else { pic("circle_timer") };
    let gas_pic = pic("gas");
    let contract_pic = (!b.contract.is_empty()).then(|| pic(contract_picture(&b.contract)));
    root.spawn(Node {
        position_type: PositionType::Absolute,
        left: Val::Percent(1.2),
        top: Val::Px(map_bottom),
        width: px(236.0),
        flex_direction: FlexDirection::Column,
        row_gap: px(4.0),
        ..default()
    })
    .with_children(|col| {
        col.spawn((
            BackgroundColor(HUD.panel),
            Node {
                flex_direction: FlexDirection::Row,
                align_items: AlignItems::Center,
                column_gap: px(8.0),
                padding: UiRect::axes(px(8.0), px(5.0)),
                border: UiRect::left(px(3.0)),
                ..default()
            },
            BorderColor::all(if closing { HUD.gas } else { HUD.accent }),
        ))
        .with_children(|row| {
            // The ring, the gas's skull in it while the circle closes.
            row.spawn(Node {
                width: px(34.0),
                height: px(34.0),
                justify_content: JustifyContent::Center,
                align_items: AlignItems::Center,
                ..default()
            })
            .with_children(|ring| {
                ring.spawn(Node { position_type: PositionType::Absolute, ..default() })
                    .with_children(|r| glyph(r, circle_pic.clone(), 34.0 * k, if closing { HUD.gas } else { HUD.accent }, true));
                if closing {
                    glyph(ring, gas_pic.clone(), 18.0 * k, HUD.gas, true);
                }
            });
            row.spawn(Node { flex_direction: FlexDirection::Column, ..default() }).with_children(|t| {
                text(t, &heading, 15.0 * k, HUD.dim);
                if !time.is_empty() {
                    text(t, &time, 26.0 * k, if closing { HUD.gas } else { HUD.text });
                }
            });
        });
        if let Some(cp) = contract_pic {
            col.spawn((
                BackgroundColor(HUD.panel),
                Node {
                    flex_direction: FlexDirection::Row,
                    align_items: AlignItems::Center,
                    column_gap: px(8.0),
                    padding: UiRect::axes(px(8.0), px(4.0)),
                    ..default()
                },
            ))
            .with_children(|row| {
                glyph(row, cp, 28.0 * k, HUD.accent, false);
                row.spawn(Node { flex_direction: FlexDirection::Column, ..default() }).with_children(|t| {
                    text(t, &b.contract.to_uppercase(), 15.0 * k, HUD.accent);
                    if b.contract_ms >= 0 {
                        text(t, &clock_ms(b.contract_ms), 18.0 * k, HUD.text);
                    }
                });
            });
        }
    });

    // --- Top right: players and squads left, kills, cash. ---
    let (players_pic, squad_pic, skull_pic, cash_pic) = (pic("players"), pic("squad"), pic("skull"), pic("cash"));
    root.spawn(Node {
        position_type: PositionType::Absolute,
        right: Val::Percent(1.6),
        top: Val::Percent(2.0),
        flex_direction: FlexDirection::Column,
        align_items: AlignItems::FlexEnd,
        row_gap: px(6.0),
        ..default()
    })
    .with_children(|col| {
        col.spawn((
            BackgroundColor(HUD.panel),
            Node {
                flex_direction: FlexDirection::Row,
                align_items: AlignItems::Center,
                column_gap: px(6.0),
                padding: UiRect::axes(px(10.0), px(4.0)),
                ..default()
            },
        ))
        .with_children(|row| {
            let mut counter = |row: &mut ChildSpawnerCommands, p: Option<Handle<Image>>, n: i32, label: &str| {
                row.spawn(Node { flex_direction: FlexDirection::Row, align_items: AlignItems::Center, column_gap: px(4.0), ..default() })
                    .with_children(|c| {
                        glyph(c, p, 22.0 * k, HUD.text, true);
                        c.spawn(Node { flex_direction: FlexDirection::Column, align_items: AlignItems::FlexStart, ..default() })
                            .with_children(|t| {
                                text(t, &n.to_string(), 24.0 * k, HUD.text);
                                text(t, label, 11.0 * k, HUD.dim);
                            });
                    });
            };
            counter(row, players_pic.clone(), b.players_left, HUD_WORDS.players_left);
            row.spawn((BackgroundColor(HUD.dim), Node { width: Val::Px(1.0), height: px(28.0), ..default() }));
            counter(row, squad_pic.clone(), b.squads_left, HUD_WORDS.squads_left);
            row.spawn((BackgroundColor(HUD.dim), Node { width: Val::Px(1.0), height: px(28.0), ..default() }));
            counter(row, skull_pic.clone(), b.kills, HUD_WORDS.kills);
        });
        col.spawn(Node { flex_direction: FlexDirection::Row, align_items: AlignItems::Center, column_gap: px(4.0), ..default() })
            .with_children(|row| {
                glyph(row, cash_pic.clone(), 24.0 * k, HUD.cash, false);
                text(row, &money(b.cash).trim_start_matches('$').to_owned(), 28.0 * k, HUD.cash);
            });
    });

    // --- Bottom left: the squad, then him. ---
    let (downed_pic, redeploy_pic, out_pic) = (pic("downed"), pic("redeploy"), pic("eliminated"));
    let (plate_pic, revive_pic) = (pic("plate"), pic("self_revive"));
    root.spawn(Node {
        position_type: PositionType::Absolute,
        left: Val::Percent(1.2),
        bottom: Val::Percent(2.6),
        flex_direction: FlexDirection::Column,
        row_gap: px(6.0),
        ..default()
    })
    .with_children(|col| {
        for (i, m) in b.mates.iter().enumerate() {
            let dead = m.state != 'a';
            col.spawn((
                BackgroundColor(HUD.panel),
                Node {
                    flex_direction: FlexDirection::Row,
                    align_items: AlignItems::Center,
                    column_gap: px(6.0),
                    padding: UiRect::all(px(4.0)),
                    ..default()
                },
            ))
            .with_children(|row| {
                slot(row, text, i + 1, 22.0 * k, k);
                row.spawn(Node { flex_direction: FlexDirection::Column, row_gap: px(2.0), ..default() }).with_children(|c| {
                    text(c, &m.name, 15.0 * k, if dead { HUD.dim } else { HUD.text });
                    if !dead {
                        plate_bar(c, b, m.armor, 150.0 * k, 3.0 * k, 2.0 * k);
                        bar(c, 150.0 * k, 3.0 * k, m.health as f32 / 100.0, if m.health <= 33 { HUD.low } else { HUD.health });
                    }
                });
                match m.state {
                    'r' => glyph(row, redeploy_pic.clone(), 22.0 * k, HUD.redeploy, false),
                    'd' => glyph(row, downed_pic.clone(), 22.0 * k, HUD.down, false),
                    'o' => glyph(row, out_pic.clone(), 22.0 * k, HUD.dim, false),
                    _ => {}
                }
            });
        }
        // Him.
        col.spawn((
            BackgroundColor(HUD.panel),
            Node {
                flex_direction: FlexDirection::Row,
                align_items: AlignItems::Center,
                column_gap: px(8.0),
                padding: UiRect::all(px(6.0)),
                ..default()
            },
        ))
        .with_children(|row| {
            slot(row, text, 0, 30.0 * k, k);
            row.spawn(Node { flex_direction: FlexDirection::Column, row_gap: px(3.0), ..default() }).with_children(|c| {
                text(c, v.name, 18.0 * k, HUD.text);
                plate_bar(c, b, if alive { b.armor } else { 0 }, 230.0 * k, 9.0 * k, 3.0 * k);
                bar(c, 230.0 * k, 6.0 * k, v.health as f32 / 100.0, if v.health <= 33 { HUD.low } else { HUD.health });
            });
            row.spawn(Node { flex_direction: FlexDirection::Column, align_items: AlignItems::Center, ..default() }).with_children(|c| {
                c.spawn(Node { flex_direction: FlexDirection::Row, align_items: AlignItems::Center, column_gap: px(2.0), ..default() })
                    .with_children(|r| {
                        glyph(r, plate_pic.clone(), 24.0 * k, if b.plates_carried > 0 { HUD.armor } else { HUD.dim }, false);
                        text(r, &b.plates_carried.to_string(), 20.0 * k, HUD.text);
                    });
                text(c, HUD_WORDS.plates, 10.0 * k, HUD.dim);
            });
            if b.self_revive {
                glyph(row, revive_pic.clone(), 26.0 * k, HUD.accent, false);
            }
        });
    });

    // --- Bottom right: the gun, its ammo, his grenades. ---
    let ammo_pic = pic(ammo_picture(v.weapon));
    root.spawn(Node {
        position_type: PositionType::Absolute,
        right: Val::Percent(1.6),
        bottom: Val::Percent(2.6),
        flex_direction: FlexDirection::Column,
        align_items: AlignItems::FlexEnd,
        row_gap: px(2.0),
        ..default()
    })
    .with_children(|col| {
        if alive && !v.weapon_label.is_empty() {
            text(col, &v.weapon_label.to_uppercase(), 17.0 * k, HUD.dim);
        }
        if alive && !v.not_a_gun {
            col.spawn((
                BackgroundColor(HUD.panel),
                Node {
                    flex_direction: FlexDirection::Row,
                    align_items: AlignItems::FlexEnd,
                    column_gap: px(8.0),
                    padding: UiRect::axes(px(12.0), px(4.0)),
                    border: UiRect::right(px(3.0)),
                    ..default()
                },
                BorderColor::all(HUD.accent),
            ))
            .with_children(|row| {
                text(row, &v.clip.to_string(), 46.0 * k, if v.low_clip { HUD.low } else { HUD.text });
                row.spawn(Node { padding: UiRect::bottom(px(6.0)), ..default() })
                    .with_children(|s| text(s, &format!("/ {}", v.stock), 22.0 * k, HUD.dim));
                row.spawn(Node { padding: UiRect::bottom(px(6.0)), ..default() })
                    .with_children(|s| glyph(s, ammo_pic.clone(), 34.0 * k, HUD.text, false));
            });
        }
        if alive && !v.offhands.is_empty() {
            col.spawn(Node { flex_direction: FlexDirection::Row, column_gap: px(12.0), ..default() }).with_children(|row| {
                for (lethal, p, n) in &v.offhands {
                    row.spawn(Node { flex_direction: FlexDirection::Row, align_items: AlignItems::Center, column_gap: px(3.0), ..default() })
                        .with_children(|c| {
                            let color = if *n == 0 { HUD.dim } else if *lethal { HUD.text } else { HUD.accent };
                            glyph(c, p.clone(), 24.0 * k, color, true);
                            text(c, &n.to_string(), 17.0 * k, color);
                        });
                }
            });
        }
    });

    // --- Centre: the gas, the redeploy. ---
    if alive && b.in_gas {
        let mask_pic = pic("gas_mask");
        root.spawn(Node {
            position_type: PositionType::Absolute,
            top: Val::Percent(20.0),
            width: Val::Percent(100.0),
            flex_direction: FlexDirection::Column,
            align_items: AlignItems::Center,
            ..default()
        })
        .with_children(|c| {
            c.spawn(Node { flex_direction: FlexDirection::Row, align_items: AlignItems::Center, column_gap: px(8.0), ..default() })
                .with_children(|r| {
                    glyph(r, gas_pic.clone(), 40.0 * k, HUD.gas, true);
                    text(r, HUD_WORDS.in_gas, 26.0 * k, HUD.gas);
                });
            if b.gas_mask {
                c.spawn(Node { flex_direction: FlexDirection::Row, align_items: AlignItems::Center, column_gap: px(6.0), ..default() })
                    .with_children(|r| {
                        glyph(r, mask_pic, 22.0 * k, HUD.text, true);
                        text(r, &format!("{} {}", HUD_WORDS.gas_mask, clock_ms(b.gas_mask_ms)), 17.0 * k, HUD.text);
                    });
            } else {
                // (World units are inches.)
                let m = (b.gas_distance.max(0) as f32 * 0.0254).round() as i32;
                text(c, &format!("{m}m"), 17.0 * k, HUD.text);
            }
        });
    }
    if !alive && (b.redeploy_in >= 0 || b.out) && b.state != "lobby" {
        let ring_pic = pic("circle_timer");
        root.spawn(Node {
            position_type: PositionType::Absolute,
            top: Val::Percent(56.0),
            width: Val::Percent(100.0),
            flex_direction: FlexDirection::Column,
            align_items: AlignItems::Center,
            row_gap: px(6.0),
            ..default()
        })
        .with_children(|c| {
            if b.redeploy_in >= 0 {
                text(c, HUD_WORDS.redeploying, 18.0 * k, HUD.dim);
                c.spawn(Node {
                    width: px(96.0),
                    height: px(96.0),
                    justify_content: JustifyContent::Center,
                    align_items: AlignItems::Center,
                    ..default()
                })
                .with_children(|ring| {
                    ring.spawn(Node { position_type: PositionType::Absolute, ..default() })
                        .with_children(|r| glyph(r, ring_pic, 96.0 * k, HUD.redeploy, true));
                    text(ring, &((b.redeploy_in.max(0) + 999) / 1000).to_string(), 42.0 * k, HUD.text);
                });
            } else {
                text(c, HUD_WORDS.redeploy_off, 24.0 * k, HUD.down);
                text(c, HUD_WORDS.buy_back, 16.0 * k, HUD.dim);
            }
        });
    }
}

/// How far (on a 1080-line screen) the kill feed rises over the modern
/// HUD's name line to clear the squad list: tenths of a pixel, 0 outside
/// Resurgence.
static FEED_LIFT: std::sync::atomic::AtomicU32 = std::sync::atomic::AtomicU32::new(0);

/// The squad list's height over the modern HUD's name line, for `b` (or
/// none outside Resurgence).
pub(crate) fn set_feed_lift(b: Option<&Br>) {
    let lift = b.map_or(0.0, |b| 30.0 + b.mates.len() as f32 * 46.0);
    FEED_LIFT.store((lift * 10.0) as u32, std::sync::atomic::Ordering::Relaxed);
}

/// See `set_feed_lift`.
pub(crate) fn feed_lift() -> f32 {
    FEED_LIFT.load(std::sync::atomic::Ordering::Relaxed) as f32 / 10.0
}

// --- Markers: hits on armor, plates breaking, kills. ---

#[derive(Component)]
struct MarkerRoot;

#[derive(Default)]
struct Seen {
    counts: Option<(i32, i32, i32)>,
    /// (kind, when, the victim) of the newest marker.
    last: Option<(Marker, f64, String)>,
}

#[derive(Clone, Copy, PartialEq)]
enum Marker {
    ArmorHit,
    Break,
    Kill,
}

impl Marker {
    /// How long its hit marker shows, and its splash (kills).
    fn span(self) -> (f64, f64) {
        match self {
            Marker::ArmorHit => (0.25, 0.0),
            Marker::Break => (0.45, 0.0),
            Marker::Kill => (0.45, 2.2),
        }
    }
}

/// The hit markers and the kill splash: a new one when the script's
/// counters rise, faded out by the frame.
#[allow(clippy::too_many_arguments)]
fn warzone_markers(
    mut commands: Commands,
    presented: Option<Res<net::PresentedSnapshot>>,
    local: Option<Res<net::LocalPresentClient>>,
    menus: Option<Res<frame::LuiMenus>>,
    font: Option<Res<GameUiFont>>,
    bo2: Option<Res<crate::bo2_font::Bo2Fonts>>,
    window: Query<&Window, With<bevy::window::PrimaryWindow>>,
    roots: Query<Entity, With<MarkerRoot>>,
    time: Res<Time<Real>>,
    mut images: ResMut<Assets<Image>>,
    mut cache: Local<std::collections::HashMap<String, Option<Handle<Image>>>>,
    mut seen: Local<Seen>,
) {
    let now = time.elapsed_secs_f64();
    let br = match (presented.as_deref(), local.as_deref()) {
        (Some(p), Some(l)) if crate::hud_modern::modern_hud_enabled() => crate::hud_modern::match_values(p, l, menus.as_deref())
            .and_then(|_| p.snapshot().and_then(|s| parse(s.meta.script_dvars(l.0).string("bo2mp_br").unwrap_or_default()))),
        _ => None,
    };
    let clear = |commands: &mut Commands| {
        for e in &roots {
            commands.entity(e).despawn();
        }
    };
    let Some(br) = br else {
        *seen = Seen::default();
        clear(&mut commands);
        return;
    };
    let counts = (br.armor_hits, br.breaks, br.kill_events);
    if let Some((h, bk, kl)) = seen.counts {
        let kind = if counts.2 > kl {
            Some(Marker::Kill)
        } else if counts.1 > bk {
            Some(Marker::Break)
        } else if counts.0 > h {
            Some(Marker::ArmorHit)
        } else {
            None
        };
        if let Some(kind) = kind {
            seen.last = Some((kind, now, br.victim.clone()));
        }
    }
    seen.counts = Some(counts);
    clear(&mut commands);
    let Some((kind, at, victim)) = seen.last.clone() else { return };
    let (mark_span, splash_span) = kind.span();
    let age = now - at;
    if age > mark_span.max(splash_span) {
        seen.last = None;
        return;
    }
    let Some(font) = font else { return };
    let height = window.single().map_or(1080.0, Window::height);
    let k = (height / 1080.0).max(0.4);
    let fade = |span: f64| (1.0 - (age / span).powi(2)).clamp(0.0, 1.0) as f32;
    let (name, color, side) = match kind {
        Marker::ArmorHit => ("hitmarker", HUD.armor_hit, 40.0),
        Marker::Break => ("armor_break", HUD.armor_break, 58.0),
        Marker::Kill => ("hitmarker_kill", HUD.enemy, 64.0),
    };
    let mark = picture(&mut cache, &mut images, name);
    let kill_pic = (kind == Marker::Kill).then(|| picture(&mut cache, &mut images, "kill_marker")).flatten();
    let bo2 = bo2.as_deref();
    let write = |c: &mut ChildSpawnerCommands, s: &str, px: f32, color: Color| crate::hud_modern::text(c, bo2, &font, "Default", s, px, color);
    commands
        .spawn((
            MarkerRoot,
            UiLayer::Hud,
            UiLayerVisibility,
            Node { position_type: PositionType::Absolute, width: Val::Percent(100.0), height: Val::Percent(100.0), ..default() },
        ))
        .with_children(|root| {
            if age < mark_span {
                let s = side * k;
                root.spawn(Node {
                    position_type: PositionType::Absolute,
                    left: Val::Px(0.0),
                    top: Val::Px(0.0),
                    width: Val::Percent(100.0),
                    height: Val::Percent(100.0),
                    justify_content: JustifyContent::Center,
                    align_items: AlignItems::Center,
                    ..default()
                })
                .with_children(|c| glyph(c, mark, s, color.with_alpha(fade(mark_span)), kind == Marker::Kill));
            }
            if kind == Marker::Kill && age < splash_span {
                let a = fade(splash_span);
                root.spawn(Node {
                    position_type: PositionType::Absolute,
                    top: Val::Percent(60.0),
                    width: Val::Percent(100.0),
                    flex_direction: FlexDirection::Column,
                    align_items: AlignItems::Center,
                    ..default()
                })
                .with_children(|c| {
                    c.spawn(Node { flex_direction: FlexDirection::Row, align_items: AlignItems::Center, column_gap: Val::Px(8.0 * k), ..default() })
                        .with_children(|r| {
                            glyph(r, kill_pic, 30.0 * k, HUD.enemy.with_alpha(a), true);
                            write(r, HUD_WORDS.eliminated, 22.0 * k, HUD.accent.with_alpha(a));
                        });
                    if !victim.is_empty() {
                        write(c, &victim, 18.0 * k, HUD.text.with_alpha(a));
                    }
                });
            }
        });
}

pub(crate) fn register(app: &mut App) {
    app.add_systems(Update, warzone_markers.after(crate::bo2_font::load_bo2_fonts).in_set(frame::ClientSet::Ui));
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_scripts_values_parse() {
        let b = parse("v=1|st=close|ph=2|np=5|ms=61500|sq=3|pl=9|my=1|ca=1250|ki=4|hp=80|ar=120|ah=50|ps=3|pc=2|pm=8|rd=20000|rt=-1|out=0|sr=1|gas=1|gd=400|mask=0|mm=0|ct=Bounty|cm=45000|circle=10,20,3000,15,25,1500|eh=3|eb=1|ek=2|ev=Bot, Jr|m=a,100,150,Alpha|m=r,0,0,Bravo, the 2nd").unwrap();
        assert_eq!(b.state, "close");
        assert_eq!((b.phase, b.phases, b.ms), (2, 5, 61500));
        assert_eq!((b.cash, b.kills, b.armor, b.plates_carried), (1250, 4, 120, 2));
        assert!(b.self_revive && b.in_gas && !b.out);
        assert_eq!(b.contract, "Bounty");
        assert_eq!(b.circle, Some([10.0, 20.0, 3000.0]));
        assert_eq!(b.next_circle, Some([15.0, 25.0, 1500.0]));
        assert_eq!(b.victim, "Bot, Jr");
        assert_eq!(b.mates.len(), 2);
        assert_eq!(b.mates[1], Mate { state: 'r', health: 0, armor: 0, name: "Bravo, the 2nd".into() });
        assert!(parse("").is_none());
    }

    #[test]
    fn plates_clocks_and_money() {
        assert_eq!(plate_fill(120, 50, 3), vec![1.0, 1.0, 0.4]);
        assert_eq!(plate_fill(0, 50, 3), vec![0.0, 0.0, 0.0]);
        assert_eq!(clock_ms(61_500), "1:02");
        assert_eq!(clock_ms(-5), "0:00");
        assert_eq!(money(1_234_567), "$1,234,567");
        assert_eq!(money(400), "$400");
        let b = Br { state: "wait".into(), phase: 3, ms: 5000, ..Br::default() };
        assert_eq!(circle_line(&b), ("CIRCLE CLOSES 3".into(), "0:05".into(), false));
        assert_eq!(ammo_picture("ksg_mp"), "ammo_shells");
        assert_eq!(ammo_picture("an94_mp"), "ammo_rifle");
        assert_eq!(contract_picture("Most Wanted"), "contract_mostwanted");
    }
}
