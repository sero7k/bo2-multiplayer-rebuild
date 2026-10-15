//! bo2mp: the Tactical Nuke's banner, after Modern Warfare III's (2023)
//! DEFCON MGB INBOUND: a red band across the top of the screen, a dark
//! slanted panel in its middle with "⚠ KEM STRIKE INBOUND ⚠" between two
//! pairs of red slashes, a thinner strip under it with WARNING either side
//! of the countdown (tenths of a second), and the caller's name with
//! "CALLED A KEM STRIKE!" at the right. It glitches in (grey blocks
//! opening from the middle), flashes, its red pulses while it counts down,
//! and it fades and breaks up at zero. The score bar and the clock hide
//! while the nuke is inbound and settling (`hud_modern`), and come back
//! after.
//!
//! The server sends `bo2mp_nuke` = `<id>|<countdown ms>|<caller>` (the
//! `bo2mp/moab` script's `level.moab_banner*`); the countdown runs on this
//! client's clock from when the id is first seen.

use bevy::asset::RenderAssetUsages;
use bevy::prelude::*;
use bevy::render::render_resource::{Extent3d, TextureDimension, TextureFormat};
use bevy::window::PrimaryWindow;
use frame::ClientSet;
use net::{LocalPresentClient, PresentedSnapshot};

use crate::layers::{GameUiFont, UiLayer, UiLayerVisibility};
use crate::shell::theme::ShellFonts;

#[derive(Component)]
struct NukeRoot;

#[derive(Default)]
struct NukeState {
    /// The nuke shown: its id, when it was first seen (real s), its
    /// countdown (s) and caller.
    shown: Option<(i64, f64, f32, String)>,
    /// The warning triangle picture.
    triangle: Option<Handle<Image>>,
}

/// The banner's reds and dark, as the reference: a salmon red, a deeper
/// red for the right side and the strip's words, the charcoal panel.
const RED: (f32, f32, f32) = (0.96, 0.45, 0.38);
const RED_DEEP: (f32, f32, f32) = (0.78, 0.26, 0.22);
const WORD_RED: Color = Color::srgb(0.47, 0.13, 0.11);
const DARK: (f32, f32, f32) = (0.17, 0.14, 0.14);

/// The bands' heights on a 1080-line screen.
const BAND_PX: f32 = 68.0;
const STRIP_PX: f32 = 30.0;

/// The glitch in, the flash, the pulse and the break-up (s).
const GLITCH_IN: f32 = 0.16;
const SETTLE: f32 = 0.30;
const FLASH: f32 = 1.0;
const PULSE: f32 = 1.4;
const FADE_OUT: f32 = 0.5;
const BREAK_UP: f32 = 0.18;

fn rgba((r, g, b): (f32, f32, f32), a: f32) -> Color {
    Color::srgba(r, g, b, a.clamp(0.0, 1.0))
}

/// The warning triangle: salmon, an exclamation mark cut out of it in the
/// panel's dark.
fn triangle_image() -> Image {
    let (w, h) = (64usize, 56usize);
    let mut px = vec![0u8; w * h * 4];
    let red = [(RED.0 * 255.0) as u8, (RED.1 * 255.0) as u8, (RED.2 * 255.0) as u8];
    let dark = [(DARK.0 * 255.0) as u8, (DARK.1 * 255.0) as u8, (DARK.2 * 255.0) as u8];
    for y in 0..h {
        for x in 0..w {
            // Inside the triangle (apex at the top middle), a little
            // antialiasing at its sides.
            let fy = (y as f32 + 0.5) / h as f32;
            let half = fy * (w as f32 / 2.0);
            let dx = ((x as f32 + 0.5) - w as f32 / 2.0).abs();
            let cover = (half - dx + 0.5).clamp(0.0, 1.0);
            if cover <= 0.0 || y + 2 >= h {
                continue;
            }
            let cx = x as f32 + 0.5 - w as f32 / 2.0;
            let bang = cx.abs() < 3.6 && ((fy > 0.34 && fy < 0.70) || (fy > 0.77 && fy < 0.88));
            let c = if bang { dark } else { red };
            let o = (y * w + x) * 4;
            px[o..o + 3].copy_from_slice(&c);
            px[o + 3] = (cover * 255.0) as u8;
        }
    }
    Image::new(
        Extent3d { width: w as u32, height: h as u32, depth_or_array_layers: 1 },
        TextureDimension::D2,
        px,
        TextureFormat::Rgba8UnormSrgb,
        RenderAssetUsages::default(),
    )
}

/// `bo2mp_nuke` as (id, countdown s, caller); none when no nuke is up.
fn read_banner(presented: &PresentedSnapshot, local: &LocalPresentClient) -> Option<(i64, f32, String)> {
    let snap = presented.snapshot()?;
    let dvars = snap.meta.script_dvars(local.0);
    let v = dvars.string("bo2mp_nuke")?;
    let mut p = v.splitn(3, '|');
    let id: i64 = p.next()?.trim().parse().ok()?;
    let ms: f32 = p.next()?.trim().parse().ok()?;
    let name = p.next().unwrap_or("").trim().to_owned();
    Some((id, (ms / 1000.0).max(0.1), name))
}

/// "0:09.8".
fn countdown(left: f32) -> String {
    let tenths = (left.max(0.0) * 10.0).ceil() as i32;
    format!("{}:{:02}.{}", tenths / 600, (tenths / 10) % 60, tenths % 10)
}

/// A small, steady shake for the break-up (no randomness: the same frame
/// looks the same).
fn jitter(t: f32, seed: f32) -> f32 {
    ((t * 91.0 + seed * 13.7).sin() * 43758.547).fract() * 2.0 - 1.0
}

#[allow(clippy::too_many_arguments)]
fn nuke_banner(
    mut commands: Commands,
    presented: Option<Res<PresentedSnapshot>>,
    local: Option<Res<LocalPresentClient>>,
    shell_fonts: Option<Res<ShellFonts>>,
    game_font: Option<Res<GameUiFont>>,
    window: Query<&Window, With<PrimaryWindow>>,
    roots: Query<Entity, With<NukeRoot>>,
    time: Res<Time<Real>>,
    mut images: ResMut<Assets<Image>>,
    mut st: Local<NukeState>,
) {
    for e in &roots {
        commands.entity(e).despawn();
    }
    let banner = match (presented.as_deref(), local.as_deref()) {
        (Some(p), Some(l)) if crate::hud_modern::modern_hud_enabled() => read_banner(p, l),
        _ => None,
    };
    let now = time.elapsed_secs_f64();
    let Some((id, length, name)) = banner else {
        st.shown = None;
        return;
    };
    if st.shown.as_ref().is_none_or(|s| s.0 != id) {
        diag::info!(Ui, "bo2mp hud: KEM STRIKE INBOUND banner ({name}, {length:.1} s)");
        st.shown = Some((id, now, length, name.clone()));
    }
    let Some((_, start, length, name)) = st.shown.clone() else { return };
    let t = (now - start) as f32;
    let left = length - t;
    // (Past zero and its break-up: the banner is gone; the score bar stays
    // hidden until the server clears the nuke.)
    if left < -BREAK_UP {
        return;
    }
    let (Some(bold), regular) = (
        shell_fonts.as_deref().map(|f| f.bold.clone()).or_else(|| game_font.as_deref().map(|g| g.0.clone())),
        shell_fonts.as_deref().map(|f| f.regular.clone()).or_else(|| game_font.as_deref().map(|g| g.0.clone())),
    ) else {
        return;
    };
    let regular = regular.unwrap_or_else(|| bold.clone());
    let triangle = st.triangle.get_or_insert_with(|| images.add(triangle_image())).clone();
    let size = window.single().map_or(Vec2::new(1920.0, 1080.0), |w| Vec2::new(w.width(), w.height()));
    spawn_at(&mut commands, &bold, &regular, &triangle, size, t, length, &name);
}

/// The banner as it looks `t` seconds into a `length` second countdown;
/// none once it has broken up.
#[allow(clippy::too_many_arguments)]
pub(crate) fn spawn_at(
    commands: &mut Commands,
    bold: &Handle<Font>,
    regular: &Handle<Font>,
    triangle: &Handle<Image>,
    size: Vec2,
    t: f32,
    length: f32,
    name: &str,
) -> Option<Entity> {
    let left = length - t;
    if left < -BREAK_UP {
        return None;
    }
    let k = (size.y / 1080.0).max(0.4);
    let (band, strip) = ((BAND_PX * k).round(), (STRIP_PX * k).round());

    // The animation's parts for now.
    let glitching = t < GLITCH_IN;
    let shown = ((t - GLITCH_IN) / (SETTLE - GLITCH_IN)).clamp(0.0, 1.0);
    let flash = (1.0 - (t - GLITCH_IN) / FLASH).clamp(0.0, 1.0);
    let pulse = 0.5 + 0.5 * (t * std::f32::consts::TAU / PULSE).cos();
    // The red fades over the last half second; the rest breaks up at zero.
    let red_out = (left / FADE_OUT).clamp(0.0, 1.0);
    let breaking = left < 0.0;
    let alpha = if breaking { (1.0 + left / BREAK_UP).clamp(0.0, 1.0) * (0.55 + 0.45 * jitter(t, 1.0).abs()) } else { shown };
    let shake = if breaking || left < 0.08 { jitter(t, 2.0) * 10.0 * k } else { 0.0 };
    let red_a = alpha * red_out * (0.82 + 0.10 * pulse + 0.2 * flash);
    let red_b = alpha * red_out * (0.66 + 0.10 * pulse + 0.2 * flash);
    let bright = |c: (f32, f32, f32)| (c.0 + (1.0 - c.0) * 0.25 * flash, c.1 + (1.0 - c.1) * 0.15 * flash, c.2 + (1.0 - c.2) * 0.15 * flash);

    let root = commands
        .spawn((
            NukeRoot,
            UiLayer::Hud,
            UiLayerVisibility,
            GlobalZIndex(140),
            Node {
                position_type: PositionType::Absolute,
                left: Val::Px(0.0),
                top: Val::Px(0.0),
                width: Val::Percent(100.0),
                height: Val::Px(band + strip),
                ..default()
            },
        ))
        .id();

    if glitching {
        // Grey blocks opening from the middle, as the reference's first
        // frames.
        let open = (t / GLITCH_IN).clamp(0.0, 1.0);
        commands.entity(root).with_children(|r| {
            for (w, a, dy) in [(0.18 + 0.42 * open, 0.55, 0.0f32), (0.10 + 0.55 * open, 0.25, 0.18), (0.06 + 0.2 * open, 0.4, -0.1)] {
                r.spawn((
                    Node {
                        position_type: PositionType::Absolute,
                        left: Val::Percent(50.0 - w * 50.0),
                        top: Val::Px(band * (0.08 + dy.max(0.0))),
                        width: Val::Percent(w * 100.0),
                        height: Val::Px(band * 0.72),
                        ..default()
                    },
                    BackgroundColor(Color::srgba(0.22, 0.20, 0.20, a)),
                ));
                r.spawn((
                    Node {
                        position_type: PositionType::Absolute,
                        left: Val::Percent(50.0 - w * 40.0),
                        top: Val::Px(band * 0.8),
                        width: Val::Percent(w * 80.0),
                        height: Val::Px(strip * 0.8),
                        ..default()
                    },
                    BackgroundColor(rgba(RED, a * 0.6)),
                ));
            }
        });
        return Some(root);
    }

    commands.entity(root).with_children(|r| {
        // --- The band: red either side, the dark slanted panel between. ---
        r.spawn((
            Node {
                position_type: PositionType::Absolute,
                left: Val::Px(shake),
                top: Val::Px(0.0),
                width: Val::Percent(100.0),
                height: Val::Px(band),
                overflow: Overflow::clip(),
                ..default()
            },
            BackgroundColor(Color::NONE),
        ))
        .with_children(|b| {
            // The reds: brighter at the left, deeper at the right.
            b.spawn((
                Node { position_type: PositionType::Absolute, left: Val::Px(0.0), top: Val::Px(0.0), width: Val::Percent(50.0), height: Val::Percent(100.0), ..default() },
                BackgroundColor(rgba(bright(RED), red_a)),
            ));
            b.spawn((
                Node { position_type: PositionType::Absolute, left: Val::Percent(50.0), top: Val::Px(0.0), width: Val::Percent(50.0), height: Val::Percent(100.0), ..default() },
                BackgroundColor(rgba(bright(RED_DEEP), red_b)),
            ));
            // The dark panel and its slanted ends (`/` both sides).
            let dark = rgba(DARK, 0.9 * alpha);
            b.spawn((
                Node { position_type: PositionType::Absolute, left: Val::Percent(21.0), top: Val::Px(0.0), width: Val::Percent(58.0), height: Val::Percent(100.0), ..default() },
                BackgroundColor(dark),
            ));
            let slant = |left: f32, width: f32, color: Color| {
                (
                    Node {
                        position_type: PositionType::Absolute,
                        left: Val::Percent(left),
                        top: Val::Px(-band),
                        width: Val::Percent(width),
                        height: Val::Px(band * 3.0),
                        ..default()
                    },
                    BackgroundColor(color),
                    UiTransform { rotation: Rot2::degrees(28.0), ..UiTransform::IDENTITY },
                )
            };
            b.spawn(slant(19.6, 2.8, dark));
            b.spawn(slant(77.6, 2.8, dark));
            // The red slashes inside the panel's ends.
            let stripe = rgba(bright(RED), alpha * (0.85 + 0.15 * flash));
            for x in [24.6, 26.6, 71.6, 73.6] {
                b.spawn(slant(x, 1.05, stripe));
            }
            // Thin scan lines over the whole band.
            let mut y = 0.0;
            while y < band {
                b.spawn((
                    Node { position_type: PositionType::Absolute, left: Val::Px(0.0), top: Val::Px(y), width: Val::Percent(100.0), height: Val::Px(1.0), ..default() },
                    BackgroundColor(Color::srgba(0.0, 0.0, 0.0, 0.10 * alpha)),
                ));
                y += (3.0 * k).max(2.0);
            }
            // ⚠ KEM STRIKE INBOUND ⚠
            b.spawn(Node {
                position_type: PositionType::Absolute,
                left: Val::Px(0.0),
                top: Val::Px(0.0),
                width: Val::Percent(100.0),
                height: Val::Percent(100.0),
                flex_direction: FlexDirection::Row,
                align_items: AlignItems::Center,
                justify_content: JustifyContent::Center,
                column_gap: Val::Px(18.0 * k),
                ..default()
            })
            .with_children(|row| {
                let tri = |row: &mut ChildSpawnerCommands| {
                    row.spawn((
                        ImageNode { image: triangle.clone(), color: Color::srgba(1.0, 1.0, 1.0, alpha), ..default() },
                        Node { width: Val::Px(52.0 * k), height: Val::Px(46.0 * k), ..default() },
                    ));
                };
                tri(row);
                // (The reference's lettering is wide: Barlow, stretched.)
                row.spawn((
                    Node { margin: UiRect::horizontal(Val::Px(132.0 * k)), ..default() },
                    UiTransform { scale: Vec2::new(1.55, 1.0), ..UiTransform::IDENTITY },
                ))
                .with_children(|w| {
                    w.spawn((
                        Text::new("KEM STRIKE INBOUND"),
                        crate::shell::theme::font(bold, 52.0 * k),
                        TextColor(rgba(bright(RED), alpha)),
                        TextLayout::linebreak(bevy::text::LineBreak::NoWrap),
                    ));
                });
                tri(row);
            });
            // The caller, at the right.
            b.spawn(Node {
                position_type: PositionType::Absolute,
                right: Val::Px(14.0 * k),
                top: Val::Px(0.0),
                height: Val::Percent(100.0),
                flex_direction: FlexDirection::Column,
                align_items: AlignItems::FlexEnd,
                justify_content: JustifyContent::Center,
                ..default()
            })
            .with_children(|c| {
                c.spawn((
                    Text::new(name.to_uppercase()),
                    crate::shell::theme::font(&bold, 24.0 * k),
                    TextColor(Color::srgba(1.0, 0.86, 0.82, alpha)),
                    TextLayout::linebreak(bevy::text::LineBreak::NoWrap),
                ));
                c.spawn((
                    Text::new("CALLED A KEM STRIKE!"),
                    crate::shell::theme::font(&bold, 16.0 * k),
                    TextColor(Color::srgba(1.0, 1.0, 1.0, alpha)),
                    TextLayout::linebreak(bevy::text::LineBreak::NoWrap),
                ));
            });
        });
        // --- The strip: WARNING, the countdown, WARNING. ---
        r.spawn((
            Node {
                position_type: PositionType::Absolute,
                left: Val::Px(-shake),
                top: Val::Px(band),
                width: Val::Percent(100.0),
                height: Val::Px(strip),
                flex_direction: FlexDirection::Row,
                align_items: AlignItems::Center,
                border: UiRect::top(Val::Px((2.0 * k).max(1.0))),
                ..default()
            },
            BackgroundColor(rgba((0.90, 0.42, 0.40), 0.58 * alpha * red_out.max(0.5))),
            BorderColor::all(rgba(RED_DEEP, alpha)),
        ))
        .with_children(|s| {
            let word = |s: &mut ChildSpawnerCommands, at: f32, text: String, color: Color, px: f32| {
                s.spawn(Node {
                    position_type: PositionType::Absolute,
                    left: Val::Percent(at - 10.0),
                    width: Val::Percent(20.0),
                    height: Val::Percent(100.0),
                    justify_content: JustifyContent::Center,
                    align_items: AlignItems::Center,
                    ..default()
                })
                .with_children(|w| {
                    w.spawn((
                        Text::new(text),
                        crate::shell::theme::font(&regular, px * k),
                        TextColor(color),
                        TextLayout::linebreak(bevy::text::LineBreak::NoWrap),
                    ));
                });
            };
            let word_red = WORD_RED.with_alpha(alpha);
            word(s, 32.0, "WARNING".to_owned(), word_red, 23.0);
            word(s, 50.7, countdown(left), Color::srgba(1.0, 1.0, 1.0, alpha), 23.0);
            word(s, 68.3, "WARNING".to_owned(), word_red, 23.0);
        });
    });
    Some(root)
}

pub(crate) fn register(app: &mut App) {
    app.add_systems(Update, nuke_banner.in_set(ClientSet::Ui).run_if(crate::hud_modern::not_warzone_match));
}

#[cfg(test)]
mod tests {
    #[test]
    fn countdown_reads_as_the_reference() {
        assert_eq!(super::countdown(9.8), "0:09.8");
        assert_eq!(super::countdown(9.75), "0:09.8");
        assert_eq!(super::countdown(0.04), "0:00.1");
        assert_eq!(super::countdown(0.0), "0:00.0");
        assert_eq!(super::countdown(10.0), "0:10.0");
    }

    #[test]
    fn the_triangle_is_drawn() {
        let img = super::triangle_image();
        let data = img.data.unwrap();
        assert!(data.chunks(4).filter(|p| p[3] > 200).count() > 800);
    }
}

pub mod preview {
    //! The banner at points through its life, over a backdrop picture (a
    //! PNG, e.g. a frame of the reference), saved as pictures:
    //! `cargo run -p ui --example hud_preview -- <out dir> nuke 1280x720 <backdrop.png>`.

    use std::path::PathBuf;

    use bevy::render::view::screenshot::{Screenshot, save_to_disk};

    use super::*;

    /// The moments shown (s into a 10 s countdown).
    const TIMES: [f32; 12] = [0.03, 0.08, 0.13, 0.2, 0.26, 0.6, 1.4, 5.0, 9.6, 9.95, 10.06, 10.14];

    #[derive(Resource)]
    struct Plan {
        out: PathBuf,
        backdrop: Option<PathBuf>,
        step: usize,
        frames: u32,
    }

    #[derive(Component)]
    struct Shot;

    fn load_png(path: &std::path::Path) -> Option<Image> {
        let bytes = std::fs::read(path).ok()?;
        let img = image::load_from_memory(&bytes).ok()?.to_rgba8();
        let (w, h) = img.dimensions();
        Some(Image::new(
            Extent3d { width: w, height: h, depth_or_array_layers: 1 },
            TextureDimension::D2,
            img.into_raw(),
            TextureFormat::Rgba8UnormSrgb,
            RenderAssetUsages::default(),
        ))
    }

    #[allow(clippy::too_many_arguments)]
    fn step(
        mut commands: Commands,
        mut plan: ResMut<Plan>,
        fonts: Option<Res<ShellFonts>>,
        window: Query<&Window, With<PrimaryWindow>>,
        shots: Query<Entity, With<Shot>>,
        mut images: ResMut<Assets<Image>>,
        mut exit: MessageWriter<AppExit>,
    ) {
        let Some(fonts) = fonts else { return };
        plan.frames += 1;
        if plan.frames < 20 {
            return;
        }
        plan.frames = 0;
        let i = plan.step / 2;
        let Some(&t) = TIMES.get(i) else {
            exit.write(AppExit::Success);
            return;
        };
        if plan.step % 2 == 1 {
            let path = plan.out.join(format!("nuke_{:02}_{:05.2}s.png", i, t));
            commands.spawn(Screenshot::primary_window()).observe(save_to_disk(path));
            plan.step += 1;
            return;
        }
        for e in &shots {
            commands.entity(e).despawn();
        }
        let size = window.single().map_or(Vec2::new(1920.0, 1080.0), |w| Vec2::new(w.width(), w.height()));
        let bg = plan
            .backdrop
            .as_deref()
            .and_then(load_png)
            .map(|img| images.add(img));
        let back = match bg {
            Some(bg) => commands.spawn((Shot, ImageNode::new(bg), Node { position_type: PositionType::Absolute, width: Val::Percent(100.0), height: Val::Percent(100.0), ..default() }, GlobalZIndex(-1))).id(),
            None => commands.spawn((Shot, BackgroundColor(Color::srgb(0.62, 0.72, 0.88)), Node { position_type: PositionType::Absolute, width: Val::Percent(100.0), height: Val::Percent(100.0), ..default() }, GlobalZIndex(-1))).id(),
        };
        let _ = back;
        let triangle = images.add(triangle_image());
        if let Some(e) = spawn_at(&mut commands, &fonts.bold, &fonts.regular, &triangle, size, t, 10.0, "JKuikki") {
            commands.entity(e).insert(Shot);
        }
        plan.step += 1;
    }

    pub fn install(app: &mut App, out: PathBuf, backdrop: Option<PathBuf>) {
        app.add_systems(Startup, crate::shell::theme::load_fonts);
        app.insert_resource(Plan { out, backdrop, step: 0, frames: 0 });
        app.add_systems(Update, step);
    }
}
