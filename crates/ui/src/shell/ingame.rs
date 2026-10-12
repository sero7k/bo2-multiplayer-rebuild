//! bo2mp shell: the menus in a match - the pause menu (Esc / Start) and
//! Choose Class (the pause menu's, and the one the server opens when he
//! spawns), in the shell's look, instead of BO2's. Choosing a class answers
//! the server as BO2's menu did (`changeclass` `custom<n>`); END GAME is
//! BO2's `popup_leavegame` `endround`.

use std::sync::Mutex;
use std::sync::atomic::{AtomicBool, AtomicU8, Ordering};

use bevy::input::gamepad::{Gamepad, GamepadAxis, GamepadButton};
use bevy::prelude::*;
use bevy::window::PrimaryWindow;

use super::classes::{self, POINTS};
use super::pics::ShellPics;
use super::theme::*;
use super::view::{Ctx, header, menu_rows};
use super::{Act, Hit, Row, bridge, window_scale};

/// What is up: 0 nothing, 1 the pause menu, 2 Choose Class.
static UP: AtomicU8 = AtomicU8::new(0);
/// The server asked for Choose Class / for its menus closed.
static SERVER_OPEN: AtomicBool = AtomicBool::new(false);
static SERVER_CLOSE: AtomicBool = AtomicBool::new(false);
static RESPONSES: Mutex<Vec<(String, String)>> = Mutex::new(Vec::new());

/// A menu of the shell is up in the match (it holds his keys and mouse).
pub(crate) fn open() -> bool {
    bridge::enabled() && UP.load(Ordering::Relaxed) != 0
}

/// The server opened a menu on his screen (`openmenu`): Choose Class for
/// the class menu; true when the shell shows it (BO2's is not opened).
pub(crate) fn server_menu(menu: &str) -> bool {
    if !bridge::enabled() {
        return false;
    }
    if menu.contains("changeclass") || menu.eq_ignore_ascii_case("class") {
        SERVER_OPEN.store(true, Ordering::Relaxed);
        return true;
    }
    false
}

/// The server closed his menus.
pub(crate) fn server_close() {
    SERVER_CLOSE.store(true, Ordering::Relaxed);
}

/// His answers for the server, taken by the match HUD.
pub(crate) fn take_responses() -> Vec<(String, String)> {
    std::mem::take(&mut *RESPONSES.lock().unwrap_or_else(std::sync::PoisonError::into_inner))
}

fn respond(menu: &str, response: &str) {
    diag::info!(Ui, "bo2mp shell: menu response {menu} {response}");
    RESPONSES.lock().unwrap_or_else(std::sync::PoisonError::into_inner).push((menu.to_owned(), response.to_owned()));
}

#[derive(Default)]
pub(crate) struct InGame {
    focus: usize,
    /// END GAME was pressed once (again to confirm).
    confirm: bool,
    held: Option<i32>,
    next_at: f64,
}

fn rows(up: u8, st: &InGame) -> Vec<Row> {
    let row = |label: &str, sub: &str, act: Act| Row { label: label.to_owned(), sub: sub.to_owned(), value: None, act };
    match up {
        1 => vec![
            row("RESUME", "Back to the match.", Act::Choose(0)),
            row("CHOOSE CLASS", "Your next life spawns with it.", Act::Choose(1)),
            row("OPTIONS", "Graphics, sound and controls.", Act::Choose(3)),
            row(
                if st.confirm { "END GAME — PRESS AGAIN" } else { "END GAME" },
                "Ends the match for everyone and goes back to the lobby.",
                Act::Choose(2),
            ),
        ],
        2 => classes::book()
            .map(|b| {
                b.classes
                    .iter()
                    .map(|c| row(&c.name.to_uppercase(), &format!("{} / {POINTS} points", c.spent), Act::Choose(c.n as usize)))
                    .collect()
            })
            .unwrap_or_default(),
        3 => super::options::rows(OPT_TAB.load(Ordering::Relaxed), &super::options::snapshot()),
        _ => Vec::new(),
    }
}

/// The Options tab he is on in a match.
static OPT_TAB: std::sync::atomic::AtomicUsize = std::sync::atomic::AtomicUsize::new(0);

/// In a match: no front end, no load, a match set up.
fn in_match(front: bool, loading: bool, setup: bool) -> bool {
    !front && !loading && setup
}

#[allow(clippy::too_many_arguments)]
pub(crate) fn ingame_input(
    front: Option<Res<crate::lui_frontend::Bo2mpFrontend>>,
    loading: Option<Res<assets::LoadingScreen>>,
    setup: Option<Res<session::Bo2mpMatchSetup>>,
    keys: Res<ButtonInput<KeyCode>>,
    mouse: Res<ButtonInput<MouseButton>>,
    pads: Query<&Gamepad>,
    hits: Query<(&GameHit, &Interaction)>,
    hud_keys: Option<Res<frame::HudInputView>>,
    time: Res<Time>,
    mut st: Local<InGame>,
    mut sounds: MessageWriter<frame::UiPlaySound>,
) {
    if !bridge::enabled() || !in_match(front.is_some(), loading.is_some(), setup.is_some()) {
        UP.store(0, Ordering::Relaxed);
        SERVER_OPEN.store(false, Ordering::Relaxed);
        SERVER_CLOSE.store(false, Ordering::Relaxed);
        return;
    }
    let now = time.elapsed_secs_f64();
    let mut up = UP.load(Ordering::Relaxed);
    if SERVER_OPEN.swap(false, Ordering::Relaxed) {
        up = 2;
        st.focus = 0;
    }
    if SERVER_CLOSE.swap(false, Ordering::Relaxed) && up == 2 {
        up = 0;
    }
    classes::want(up == 2);
    if hud_keys.as_deref().is_some_and(|k| k.console_open) {
        publish(&st);
        UP.store(up, Ordering::Relaxed);
        return;
    }
    let pad = |b: GamepadButton| pads.iter().any(|p| p.just_pressed(b));
    let esc = keys.just_pressed(KeyCode::Escape) || pad(GamepadButton::Start);
    if up == 0 {
        if esc {
            up = 1;
            st.focus = 0;
            st.confirm = false;
            sounds.write(frame::UiPlaySound { alias: "uin_main_pause".to_owned() });
        }
        publish(&st);
        UP.store(up, Ordering::Relaxed);
        return;
    }
    // The menu sounds: a move (focus, tab, a setting changed), a choice, a
    // step back; the last one this frame is heard.
    let mut sound: Option<&'static str> = None;
    let rows = rows(up, &st);
    let n = rows.len().max(1);
    st.focus = st.focus.min(n - 1);
    let stick = pads.iter().find_map(|p| {
        let y = p.get(GamepadAxis::LeftStickY).unwrap_or(0.0);
        if y > 0.6 { Some(-1) } else if y < -0.6 { Some(1) } else { None }
    });
    let mut dir = 0;
    match stick {
        Some(d) if st.held != Some(d) => {
            st.held = Some(d);
            st.next_at = now + 0.38;
            dir = d;
        }
        Some(d) if now >= st.next_at => {
            st.next_at = now + 0.12;
            dir = d;
        }
        Some(_) => {}
        None => st.held = None,
    }
    if keys.just_pressed(KeyCode::ArrowUp) || keys.just_pressed(KeyCode::KeyW) || pad(GamepadButton::DPadUp) {
        dir = -1;
    }
    if keys.just_pressed(KeyCode::ArrowDown) || keys.just_pressed(KeyCode::KeyS) || pad(GamepadButton::DPadDown) {
        dir = 1;
    }
    if dir != 0 {
        let was = st.focus;
        st.focus = ((st.focus as i32 + dir).rem_euclid(n as i32)) as usize;
        st.confirm = false;
        if st.focus != was {
            sound = Some("uin_navigation_click");
        }
    }
    if up == 3 {
        let tabs = super::options::TABS.len() as i32;
        let mut tab = OPT_TAB.load(Ordering::Relaxed) as i32;
        if keys.just_pressed(KeyCode::KeyQ) || pad(GamepadButton::LeftTrigger) {
            tab = (tab - 1).rem_euclid(tabs);
            st.focus = 0;
            sound = Some("uin_navigation_click");
        }
        if keys.just_pressed(KeyCode::KeyE) || pad(GamepadButton::RightTrigger) {
            tab = (tab + 1).rem_euclid(tabs);
            st.focus = 0;
            sound = Some("uin_navigation_click");
        }
        OPT_TAB.store(tab as usize, Ordering::Relaxed);
        let side = if keys.just_pressed(KeyCode::ArrowLeft) || keys.just_pressed(KeyCode::KeyA) || pad(GamepadButton::DPadLeft) {
            -1
        } else if keys.just_pressed(KeyCode::ArrowRight) || keys.just_pressed(KeyCode::KeyD) || pad(GamepadButton::DPadRight) {
            1
        } else {
            0
        };
        if side != 0
            && let Some(Act::Cycle(super::Setting::Option(k))) = rows.get(st.focus).map(|r| r.act)
        {
            super::options::queue(tab as usize, k, side);
            sound = Some("uin_navigation_click");
        }
    }
    let mut pick = keys.just_pressed(KeyCode::Enter) || keys.just_pressed(KeyCode::Space) || pad(GamepadButton::South);
    for (hit, i) in &hits {
        if *i != Interaction::None {
            if st.focus != hit.0 {
                st.focus = hit.0;
                st.confirm = false;
                sound = Some("uin_navigation_click");
            }
            if mouse.just_pressed(MouseButton::Left) {
                pick = true;
            }
        }
    }
    let back = esc || keys.just_pressed(KeyCode::Backspace) || pad(GamepadButton::East) || mouse.just_pressed(MouseButton::Right);
    if pick && up == 3 {
        if let Some(Act::Cycle(super::Setting::Option(k))) = rows.get(st.focus).map(|r| r.act) {
            super::options::queue(OPT_TAB.load(Ordering::Relaxed), k, 1);
            sound = Some("uin_navigation_select_main");
        }
    } else if pick && let Some(r) = rows.get(st.focus) {
        let Act::Choose(k) = r.act else {
            if let Some(alias) = sound {
                sounds.write(frame::UiPlaySound { alias: alias.to_owned() });
            }
            return;
        };
        sound = Some("uin_navigation_select_main");
        match up {
            1 => match k {
                0 => up = 0,
                1 => {
                    up = 2;
                    st.focus = 0;
                }
                3 => {
                    up = 3;
                    st.focus = 0;
                    OPT_TAB.store(0, Ordering::Relaxed);
                }
                _ if st.confirm => {
                    respond("popup_leavegame", "endround");
                    up = 0;
                }
                _ => st.confirm = true,
            },
            _ => {
                respond("changeclass", &format!("custom{k}"));
                up = 0;
            }
        }
    } else if back {
        let was = up;
        up = if up == 1 { 0 } else { 1 };
        st.focus = if was == 3 { 2 } else if up == 1 { 1 } else { 0 };
        st.confirm = false;
        sound = Some("cac_cmn_backout");
    }
    if let Some(alias) = sound {
        sounds.write(frame::UiPlaySound { alias: alias.to_owned() });
    }
    publish(&st);
    UP.store(up, Ordering::Relaxed);
}

#[derive(Component)]
pub(crate) struct GameHit(usize);

#[derive(Component)]
pub(crate) struct GameRoot;

#[allow(clippy::too_many_arguments)]
pub(crate) fn ingame_view(
    mut commands: Commands,
    fonts: Option<Res<ShellFonts>>,
    mut pics: ResMut<ShellPics>,
    icons: Option<Res<assets::T6HudIcons>>,
    mut images: ResMut<Assets<Image>>,
    roots: Query<Entity, With<GameRoot>>,
    windows: Query<&Window, With<PrimaryWindow>>,
    hits: Query<(&GameHit, &Interaction)>,
    mut drawn: Local<String>,
) {
    let up = if open() { UP.load(Ordering::Relaxed) } else { 0 };
    let Some(fonts) = fonts.filter(|_| up != 0) else {
        for e in &roots {
            commands.entity(e).despawn();
        }
        drawn.clear();
        return;
    };
    // (The row under the mouse is the one in focus: input set it.)
    let focus = hits.iter().find(|(_, i)| **i != Interaction::None).map(|(h, _)| h.0);
    let st_rows = rows(up, &InGame::default());
    let book = classes::book();
    let (area, s) = window_scale(&windows);
    let pad = crate::pad_glyphs::pad_active();
    let key = format!(
        "{up}|{focus:?}|{}|{area:?}|{pad}|{}|{}|{}|{}",
        FOCUS.load(Ordering::Relaxed),
        OPT_TAB.load(Ordering::Relaxed),
        super::options::snapshot().revision,
        book.as_ref().map_or(0, |b| std::sync::Arc::as_ptr(b) as usize),
        pics.count()
    );
    if *drawn == key && !roots.is_empty() {
        return;
    }
    *drawn = key;
    for e in &roots {
        commands.entity(e).despawn();
    }
    let focus = FOCUS.load(Ordering::Relaxed) as usize;
    let mut cx = Ctx { s, w: area.x / s, h: area.y / s, fonts: &fonts, pics: &mut pics, icons: icons.as_deref(), images: &mut images, pad };
    commands
        .spawn((
            GameRoot,
            GlobalZIndex(15_000),
            Node { position_type: PositionType::Absolute, width: Val::Percent(100.0), height: Val::Percent(100.0), ..default() },
            BackgroundGradient::from(LinearGradient::to_right(vec![
                ColorStop::new(Color::srgba(0.008, 0.016, 0.035, 0.94), Val::Percent(0.0)),
                ColorStop::new(Color::srgba(0.008, 0.016, 0.035, 0.72), Val::Percent(45.0)),
                ColorStop::new(Color::srgba(0.008, 0.016, 0.035, 0.45), Val::Percent(100.0)),
            ])),
        ))
        .with_children(|c| {
            header(c, &cx, &[match up {
                1 => "PAUSED",
                3 => "OPTIONS",
                _ => "CHOOSE CLASS",
            }]);
            if up == 3 {
                let labels: Vec<String> = super::options::TABS.iter().map(|t| (*t).to_owned()).collect();
                super::view::tabs(c, &mut cx, &labels, OPT_TAB.load(Ordering::Relaxed), Hit::Page);
            }
            c.spawn(Node {
                position_type: PositionType::Absolute,
                left: cx.px(72.0),
                top: cx.px(if up == 3 { 196.0 } else { 176.0 }),
                flex_direction: FlexDirection::Column,
                row_gap: cx.px(8.0),
                ..default()
            })
            .with_children(|col| {
                let mut rows = st_rows.clone();
                if up == 1 && CONFIRM.load(Ordering::Relaxed) {
                    rows[3].label = "END GAME — PRESS AGAIN".to_owned();
                }
                if up == 3 {
                    for r in &mut rows {
                        r.sub.clear();
                    }
                }
                // (Rows are hit by `GameHit`, not the front end's `Hit`.)
                menu_rows(col, &cx, &rows, focus, if up == 3 { 760.0 } else { 520.0 }, false);
            });
            if up == 2
                && let Some(b) = book.as_ref()
                && let Some(class) = b.classes.get(focus)
            {
                super::cac::class_card(c, &mut cx, b, class, 640.0, 176.0, 1208.0);
            }
            c.spawn(Node {
                position_type: PositionType::Absolute,
                left: cx.px(72.0),
                bottom: cx.px(40.0),
                column_gap: cx.px(28.0),
                ..default()
            })
            .with_children(|f| {
                let hints: &[(&str, &str)] = if up == 3 {
                    &[("ESC", "BACK"), ("A / D", "CHANGE"), ("Q / E", "TABS")]
                } else {
                    &[("ESC", if up == 1 { "RESUME" } else { "BACK" }), ("ENTER", "SELECT")]
                };
                for &(k, what) in hints {
                    f.spawn(Node { column_gap: cx.px(8.0), align_items: AlignItems::Center, ..default() }).with_children(|h| {
                        h.spawn((Node { padding: UiRect::axes(cx.px(7.0), cx.px(1.0)), ..default() }, BackgroundColor(KEYCAP)))
                            .with_child((Text::new(k), cx.bold(17.0), TextColor(WHITE)));
                        h.spawn((Text::new(what), cx.bold(18.0), TextColor(DIM)));
                    });
                }
            });
        });
}

/// The row in focus and END GAME's confirm, as input left them (for the
/// view).
static FOCUS: std::sync::atomic::AtomicUsize = std::sync::atomic::AtomicUsize::new(0);
static CONFIRM: AtomicBool = AtomicBool::new(false);

/// The rows' hits are the front end's `Hit::Row`; in a match they are read
/// as `GameHit` (added beside them).
pub(crate) fn tag_hits(mut commands: Commands, rows: Query<(Entity, &Hit), (Added<Hit>, Without<GameHit>)>) {
    if !open() {
        return;
    }
    for (e, hit) in &rows {
        if let Hit::Row(i) = hit {
            commands.entity(e).insert(GameHit(*i));
        }
    }
}

pub(crate) fn publish(st: &InGame) {
    FOCUS.store(st.focus, Ordering::Relaxed);
    CONFIRM.store(st.confirm, Ordering::Relaxed);
}

/// The preview: a menu up as if he had opened it.
pub(crate) fn preview_open(up: u8) {
    UP.store(up, Ordering::Relaxed);
}
