//! bo2mp: the free-for-all end screen in the modern HUD, after Modern
//! Warfare Remastered's: the world dimmed, VICTORY or DEFEAT in big red (or
//! cool blue) capitals over a thin rule, the end reason under it ("Score
//! limit reached"), then the top three, each row a step further right: its
//! place in a diamond ("1st", the winner's with chevrons after it), the
//! name, the score at the right, a thin line under it. The names and
//! scores decode in letter by letter. His own row follows the three when
//! he placed lower. Gun Game shows the gun reached; One in the Chamber,
//! Sharpshooter and the rest their points.
//!
//! The scripts publish `bo2mp_outcome` as the match ends (mp_globallogic.gsc
//! `bo2mp_ffaoutcome`); the screen shows until the final killcam starts and
//! does not come back after it (the scoreboard follows).

use bevy::prelude::*;
use net::{LocalPresentClient, PresentedSnapshot};

use crate::layers::GameUiFont;
use crate::shell::theme::ShellFonts;

/// One player's line of the result.
#[derive(Clone, Debug, PartialEq)]
pub(crate) struct Placed {
    pub num: i32,
    pub score: i32,
    pub name: String,
}

/// The result as the scripts publish it.
#[derive(Clone, Debug, PartialEq)]
pub(crate) struct Outcome {
    pub winner: i32,
    pub gametype: String,
    pub reason: String,
    pub placed: Vec<Placed>,
}

/// `"<winner>|<gametype>|<reason>|<num>,<score>,<name>;..."` (a name may
/// hold commas; it is the row's last field).
pub(crate) fn parse_outcome(s: &str) -> Option<Outcome> {
    let mut parts = s.splitn(4, '|');
    let winner = parts.next()?.trim().parse().ok()?;
    let gametype = parts.next()?.trim().to_owned();
    let reason = parts.next()?.trim().to_owned();
    let placed = parts
        .next()
        .unwrap_or("")
        .split(';')
        .filter_map(|row| {
            let mut f = row.splitn(3, ',');
            let num = f.next()?.trim().parse().ok()?;
            let score = f.next()?.trim().parse::<f32>().ok()? as i32;
            Some(Placed { num, score, name: f.next().unwrap_or("").to_owned() })
        })
        .collect();
    Some(Outcome { winner, gametype, reason, placed })
}

/// The end reason's words from its localized key (`MP_SCORE_LIMIT_REACHED`
/// -> "Score limit reached").
pub(crate) fn reason_text(key: &str) -> String {
    let k = key.trim().trim_start_matches('&').to_ascii_uppercase();
    let known = [
        ("SCORE_LIMIT_REACHED", "Score limit reached"),
        ("TIME_LIMIT_REACHED", "Time limit reached"),
        ("HOST_ENDED_GAME", "Host ended game"),
        ("ENDED_GAME", "Game ended"),
        ("ROUND_LIMIT_REACHED", "Round limit reached"),
        ("ENEMIES_ELIMINATED", "Enemies eliminated"),
        ("TARGET_DESTROYED", "Target destroyed"),
        ("PLAYERS_FORFEITED", "Players forfeited"),
    ];
    if let Some((_, t)) = known.iter().find(|(id, _)| k.ends_with(id)) {
        return (*t).to_owned();
    }
    if k.is_empty() || !k.contains('_') {
        return key.trim().to_owned();
    }
    // Another key: its words after the game's prefix, sentence case.
    let words = k.split('_').skip_while(|w| matches!(*w, "MP" | "MPUI" | "WAGER" | "GAME")).collect::<Vec<_>>().join(" ").to_lowercase();
    let mut c = words.chars();
    c.next().map_or_else(String::new, |f| f.to_uppercase().chain(c).collect())
}

/// "1st", "2nd", "3rd", "4th", ... "11th", "21st".
pub(crate) fn place_label(place: usize) -> String {
    let suffix = match (place % 10, place % 100) {
        (_, 11..=13) => "th",
        (1, _) => "st",
        (2, _) => "nd",
        (3, _) => "rd",
        _ => "th",
    };
    format!("{place}{suffix}")
}

/// The score as the row shows it.
pub(crate) fn score_text(gametype: &str, score: i32) -> String {
    match gametype {
        "gun" => format!("GUN {score}"),
        // (Resurgence: the squad's place.)
        "br" => format!("#{score}"),
        _ => score.to_string(),
    }
}

/// The rows shown: (place, row), the top three and then his own when he
/// placed lower.
pub(crate) fn shown_rows(o: &Outcome, me: i32) -> Vec<(usize, Placed)> {
    let mut rows: Vec<(usize, Placed)> = o.placed.iter().take(3).cloned().enumerate().map(|(i, p)| (i + 1, p)).collect();
    if let Some(at) = o.placed.iter().position(|p| p.num == me)
        && at >= 3
    {
        rows.push((at + 1, o.placed[at].clone()));
    }
    rows
}

/// What a decoding line reads `t` seconds in: the letters revealed so far,
/// the rest stand-in symbols (spaces stay spaces), all of it by `full`.
pub(crate) fn decoded(target: &str, t: f32, full: f32, seed: u32) -> String {
    if t >= full {
        return target.to_owned();
    }
    const GLYPHS: &[u8] = b"ABCDEFGHJKLMNPQRSTUVWXYZ0123456789#$%&";
    let n = target.chars().count().max(1);
    let shown = ((t.max(0.0) / full) * n as f32) as usize;
    let tick = (t * 30.0) as u32;
    target
        .chars()
        .enumerate()
        .map(|(i, ch)| {
            if i < shown || ch == ' ' {
                ch
            } else {
                let h = (i as u32).wrapping_mul(2_654_435_761) ^ tick.wrapping_mul(40_503) ^ seed;
                GLYPHS[(h % GLYPHS.len() as u32) as usize] as char
            }
        })
        .collect()
}

#[derive(Component)]
struct OutcomeRoot;

/// A line that decodes in: its words, when it starts (real seconds) and
/// how long it takes.
#[derive(Component)]
struct Decode {
    target: String,
    start: f64,
    full: f32,
    seed: u32,
}

#[derive(Default)]
struct State {
    /// The result being shown (or shown already) and when it was first seen.
    shown: Option<(String, f64)>,
    /// A killcam began after it: the screen is done.
    done: bool,
}

const DEFEAT_RED: Color = Color::srgb(0.86, 0.30, 0.18);
const VICTORY_BLUE: Color = Color::srgb(0.50, 0.82, 1.0);
const DRAW_GREY: Color = Color::srgb(0.80, 0.80, 0.82);

/// bo2mp: the end screen is up: it alone shows (VICTORY / DEFEAT / DRAW and
/// the places); the HUD, scoreboard and the rest are hidden meanwhile
/// (`screen::layers_for_screen`).
static OUTCOME_UP: std::sync::atomic::AtomicBool = std::sync::atomic::AtomicBool::new(false);

pub(crate) fn outcome_up() -> bool {
    OUTCOME_UP.load(std::sync::atomic::Ordering::Relaxed)
}

#[allow(clippy::too_many_arguments)]
fn outcome_screen(
    mut commands: Commands,
    presented: Option<Res<PresentedSnapshot>>,
    local: Option<Res<LocalPresentClient>>,
    (shell_fonts, game_font): (Option<Res<ShellFonts>>, Option<Res<GameUiFont>>),
    window: Query<&Window, With<bevy::window::PrimaryWindow>>,
    roots: Query<Entity, With<OutcomeRoot>>,
    mut lines: Query<(&mut Text, &Decode)>,
    time: Res<Time<Real>>,
    mut state: Local<State>,
) {
    let now = time.elapsed_secs_f64();
    let seen = match (presented.as_deref().and_then(PresentedSnapshot::snapshot), local.as_deref()) {
        (Some(snap), Some(l)) if crate::hud_modern::modern_hud_enabled() => {
            let d = snap.meta.script_dvars(l.0);
            let outcome = d.string("bo2mp_outcome").unwrap_or_default().to_owned();
            let killcam = d.string("bo2mp_killcam").unwrap_or_default();
            Some((outcome, killcam.starts_with("1|") || killcam.starts_with("2|"), l.0.0 as i32))
        }
        _ => None,
    };
    let clear = |commands: &mut Commands| {
        OUTCOME_UP.store(false, std::sync::atomic::Ordering::Relaxed);
        for e in &roots {
            commands.entity(e).despawn();
        }
    };
    let Some((raw, in_killcam, me)) = seen else {
        *state = State::default();
        clear(&mut commands);
        return;
    };
    if raw.is_empty() {
        // (A new match: ready for its result.)
        *state = State::default();
        clear(&mut commands);
        return;
    }
    if state.shown.as_ref().is_some_and(|(r, _)| *r != raw) {
        *state = State::default();
        clear(&mut commands);
    }
    if in_killcam && state.shown.is_some() {
        state.done = true;
    }
    if state.done || in_killcam {
        clear(&mut commands);
        return;
    }
    let Some(outcome) = parse_outcome(&raw) else { return };
    if state.shown.is_none() {
        state.shown = Some((raw.clone(), now));
    }
    let start = state.shown.as_ref().map_or(now, |(_, t)| *t);
    OUTCOME_UP.store(true, std::sync::atomic::Ordering::Relaxed);
    if !roots.is_empty() {
        for (mut text, d) in &mut lines {
            let v = decoded(&d.target, (now - d.start) as f32, d.full, d.seed);
            if text.0 != v {
                text.0 = v;
            }
        }
        return;
    }
    let Some(faces) = crate::hud_match::Faces::of(shell_fonts.as_deref(), game_font.as_deref()) else { return };
    let title_face = shell_fonts.as_deref().map_or_else(|| faces.bold.clone(), |s| s.black.clone());
    let h = window.single().map_or(1080.0, Window::height);
    spawn_screen(&mut commands, &faces, &title_face, (h / 1080.0).max(0.4), &outcome, me, start);
}

/// The screen, drawn for `outcome` as player `me` sees it (its lines start
/// decoding at `start`, real seconds).
fn spawn_screen(commands: &mut Commands, faces: &crate::hud_match::Faces, title_face: &Handle<Font>, k: f32, outcome: &Outcome, me: i32, start: f64) {
    let px = |v: f32| Val::Px(v * k);
    let font = |f: &Handle<Font>, size: f32| crate::shell::theme::font(f, size * k);
    let shade = |size: f32| TextShadow { offset: Vec2::splat((size * k / 16.0).max(1.0)), color: Color::srgba(0.0, 0.0, 0.0, 0.8), ..default() };
    let (title, colour) = match outcome.winner {
        -1 => ("DRAW", DRAW_GREY),
        w if w == me => ("VICTORY", VICTORY_BLUE),
        _ if outcome.gametype == "br" && outcome.placed.iter().any(|p| p.num == me && p.score == 1) => ("VICTORY", VICTORY_BLUE),
        _ => ("DEFEAT", DEFEAT_RED),
    };
    let rows = shown_rows(&outcome, me);
    let reason = reason_text(&outcome.reason);
    commands
        .spawn((
            OutcomeRoot,
            Node { position_type: PositionType::Absolute, width: Val::Percent(100.0), height: Val::Percent(100.0), ..default() },
            BackgroundColor(Color::srgba(0.0, 0.0, 0.0, 0.62)),
            GlobalZIndex(40),
            Pickable::IGNORE,
        ))
        .with_children(|root| {
            root.spawn(Node {
                position_type: PositionType::Absolute,
                width: Val::Percent(100.0),
                top: Val::Percent(21.0),
                flex_direction: FlexDirection::Column,
                align_items: AlignItems::Center,
                ..default()
            })
            .with_children(|col| {
                col.spawn((Text::new(title), font(title_face, 64.0), TextColor(colour), shade(64.0), TextLayout::linebreak(bevy::text::LineBreak::NoWrap)));
                col.spawn((Node { width: px(470.0), height: px(1.5), margin: UiRect::vertical(px(8.0)), ..default() }, BackgroundColor(Color::srgba(1.0, 1.0, 1.0, 0.35))));
                col.spawn((Text::new(reason), font(&faces.regular, 23.0), TextColor(Color::srgba(1.0, 1.0, 1.0, 0.9)), shade(23.0), TextLayout::linebreak(bevy::text::LineBreak::NoWrap)));
                col.spawn(Node { height: px(26.0), ..default() });
                for (i, (place, p)) in rows.iter().enumerate() {
                    let mine = p.num == me;
                    let first = *place == 1;
                    // Each row a step further right, its own row after a gap.
                    let step = i.min(2) as f32 * 26.0;
                    let row_start = start + 0.25 + i as f64 * 0.18;
                    col.spawn(Node {
                        width: px(860.0 - step * 2.0),
                        height: px(if first { 66.0 } else { 58.0 }),
                        margin: UiRect { left: px(step * 2.0), top: px(if *place > 3 { 14.0 } else { 0.0 }), ..default() },
                        align_items: AlignItems::Center,
                        border: UiRect::bottom(px(1.0)),
                        ..default()
                    })
                    .insert(BorderColor::all(Color::srgba(1.0, 1.0, 1.0, 0.18)))
                    .with_children(|r| {
                        // The place in its diamond.
                        r.spawn(Node { width: px(78.0), height: px(58.0), justify_content: JustifyContent::Center, align_items: AlignItems::Center, ..default() })
                            .with_children(|d| {
                                d.spawn((
                                    Node { position_type: PositionType::Absolute, width: px(40.0), height: px(40.0), border: UiRect::all(px(1.5)), ..default() },
                                    BorderColor::all(Color::srgba(1.0, 1.0, 1.0, if first { 0.75 } else { 0.45 })),
                                    BackgroundColor(Color::srgba(0.0, 0.0, 0.0, 0.25)),
                                    UiTransform { rotation: Rot2::degrees(45.0), ..UiTransform::IDENTITY },
                                ));
                                d.spawn((Text::new(place_label(*place)), font(&faces.bold, 16.0), TextColor(Color::WHITE), shade(16.0), TextLayout::linebreak(bevy::text::LineBreak::NoWrap)));
                            });
                        // The winner's chevrons.
                        r.spawn((
                            Text::new(if first { "\u{203A}\u{203A}" } else { "" }),
                            font(&faces.regular, 30.0),
                            TextColor(Color::srgba(1.0, 1.0, 1.0, 0.45)),
                            Node { width: px(54.0), margin: UiRect::left(px(6.0)), ..default() },
                        ));
                        let name_colour = if mine { colour } else { Color::srgba(1.0, 1.0, 1.0, if first { 1.0 } else { 0.85 }) };
                        r.spawn((
                            Text::new(""),
                            Decode { target: p.name.clone(), start: row_start, full: 0.55, seed: i as u32 * 7919 + 1 },
                            font(&faces.regular, if first { 34.0 } else { 27.0 }),
                            TextColor(name_colour),
                            shade(30.0),
                            TextLayout::linebreak(bevy::text::LineBreak::NoWrap),
                            Node { flex_grow: 1.0, ..default() },
                        ));
                        r.spawn((
                            Text::new(""),
                            Decode { target: score_text(&outcome.gametype, p.score), start: row_start + 0.1, full: 0.45, seed: i as u32 * 104_729 + 3 },
                            font(&faces.bold, if first { 26.0 } else { 23.0 }),
                            TextColor(Color::WHITE),
                            shade(24.0),
                            TextLayout::linebreak(bevy::text::LineBreak::NoWrap),
                            Node { margin: UiRect::right(px(12.0)), ..default() },
                        ));
                    });
                }
            });
        });
}

pub(crate) fn register_outcome_hud(app: &mut App) {
    app.add_systems(Update, outcome_screen.in_set(frame::ClientSet::Ui));
}

pub mod preview {
    //! The end screen with stand-in results over a picture, saved as
    //! pictures (`cargo run -p ui --example hud_preview -- <out dir>
    //! outcome [size] [background.png]`).

    use std::path::PathBuf;

    use bevy::image::{CompressedImageFormats, ImageSampler, ImageType};
    use bevy::render::view::screenshot::{Screenshot, save_to_disk};

    use super::*;

    #[derive(Resource)]
    struct Plan {
        out: PathBuf,
        background: Option<PathBuf>,
        step: usize,
        frames: u32,
        at: f64,
    }

    #[derive(Component)]
    struct Shot;

    /// (file name, published result, who is looking).
    fn case(i: usize) -> Option<(&'static str, &'static str, i32)> {
        Some(match i {
            0 => ("01_ffa_defeat", "3|dm|MP_SCORE_LIMIT_REACHED|3,1500,simon;1,1450,egs;0,1200,reaaL;2,900,Jimbo;", 1),
            1 => ("02_ffa_victory", "3|dm|MP_SCORE_LIMIT_REACHED|3,1500,simon;1,1450,egs;0,1200,reaaL;2,900,Jimbo;", 3),
            2 => ("03_gun_game_fifth", "4|gun|MP_SCORE_LIMIT_REACHED|4,20,Sass;1,18,egs;0,17,reaaL;2,15,Jimbo;3,12,simon;", 3),
            3 => ("04_oic_time", "1|oic|MP_TIME_LIMIT_REACHED|1,9,egs;3,7,simon;0,4,JTAG;", 3),
            _ => return None,
        })
    }

    #[allow(clippy::too_many_arguments)]
    fn step(
        mut commands: Commands,
        mut plan: ResMut<Plan>,
        fonts: Option<Res<ShellFonts>>,
        window: Query<&Window, With<bevy::window::PrimaryWindow>>,
        shots: Query<Entity, With<Shot>>,
        roots: Query<Entity, With<OutcomeRoot>>,
        mut lines: Query<(&mut Text, &Decode)>,
        mut images: ResMut<Assets<Image>>,
        time: Res<Time<Real>>,
        mut exit: MessageWriter<AppExit>,
    ) {
        let Some(fonts) = fonts else { return };
        let now = time.elapsed_secs_f64();
        // (The lines decode on the real clock; the shots wait them out.)
        for (mut text, d) in &mut lines {
            text.0 = decoded(&d.target, (now - d.start) as f32, d.full, d.seed);
        }
        plan.frames += 1;
        if plan.frames < 40 || now - plan.at < 1.6 {
            return;
        }
        plan.frames = 0;
        let i = plan.step / 2;
        let Some((name, raw, me)) = case(i) else {
            exit.write(AppExit::Success);
            return;
        };
        let height = window.single().map_or(1080.0, Window::height);
        if plan.step % 2 == 1 {
            commands.spawn(Screenshot::primary_window()).observe(save_to_disk(plan.out.join(format!("{name}_{}p.png", height as u32))));
            plan.step += 1;
            return;
        }
        for e in shots.iter().chain(roots.iter()) {
            commands.entity(e).despawn();
        }
        if let Some(path) = &plan.background
            && let Ok(bytes) = std::fs::read(path)
            && let Ok(img) = Image::from_buffer(&bytes, ImageType::Extension("png"), CompressedImageFormats::NONE, true, ImageSampler::Default, Default::default())
        {
            commands.spawn((
                Shot,
                ImageNode::new(images.add(img)),
                Node { position_type: PositionType::Absolute, width: Val::Percent(100.0), height: Val::Percent(100.0), ..default() },
                GlobalZIndex(0),
            ));
        }
        let faces = crate::hud_match::Faces { regular: fonts.regular.clone(), bold: fonts.bold.clone() };
        let outcome = parse_outcome(raw).expect("case");
        spawn_screen(&mut commands, &faces, &fonts.black, (height / 1080.0).max(0.4), &outcome, me, now);
        plan.at = now;
        plan.step += 1;
    }

    pub fn install(app: &mut App, out: PathBuf, background: Option<PathBuf>) {
        app.add_systems(Startup, crate::shell::theme::load_fonts);
        app.insert_resource(Plan { out, background, step: 0, frames: 0, at: 0.0 });
        app.add_systems(Update, step);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reads_the_published_result() {
        let o = parse_outcome("3|dm|MP_SCORE_LIMIT_REACHED|3,1500,simon;1,1450,egs;0,1200,rea,aL;").unwrap();
        assert_eq!(o.winner, 3);
        assert_eq!(o.gametype, "dm");
        assert_eq!(o.placed.len(), 3);
        assert_eq!(o.placed[2], Placed { num: 0, score: 1200, name: "rea,aL".to_owned() });
        assert_eq!(reason_text(&o.reason), "Score limit reached");
        assert!(parse_outcome("").is_none());
    }

    #[test]
    fn places_and_scores_read_as_words() {
        assert_eq!(place_label(1), "1st");
        assert_eq!(place_label(2), "2nd");
        assert_eq!(place_label(3), "3rd");
        assert_eq!(place_label(11), "11th");
        assert_eq!(place_label(22), "22nd");
        assert_eq!(score_text("gun", 18), "GUN 18");
        assert_eq!(score_text("oic", 9), "9");
        assert_eq!(reason_text("MP_TIME_LIMIT_REACHED"), "Time limit reached");
        assert_eq!(reason_text("MP_SOMETHING_ELSE"), "Something else");
    }

    #[test]
    fn his_row_follows_the_top_three() {
        let o = parse_outcome("1|dm|x|1,9,a;2,8,b;3,7,c;4,6,d;5,5,e;").unwrap();
        let rows = shown_rows(&o, 5);
        assert_eq!(rows.iter().map(|(p, r)| (*p, r.num)).collect::<Vec<_>>(), vec![(1, 1), (2, 2), (3, 3), (5, 5)]);
        assert_eq!(shown_rows(&o, 2).len(), 3);
    }

    #[test]
    fn lines_decode_in() {
        assert_eq!(decoded("simon", 1.0, 0.5, 1), "simon");
        let mid = decoded("simon", 0.25, 0.5, 1);
        assert_eq!(mid.chars().count(), 5);
        assert!(mid.starts_with("si"));
        assert_eq!(decoded("a b", 0.0, 0.5, 1).chars().nth(1), Some(' '));
    }
}
