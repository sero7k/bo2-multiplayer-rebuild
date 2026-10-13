//! bo2mp shell: OPTIONS > CONTROLLER's diagram of the pad: an Xbox
//! controller drawn in the shell's style with every button's action under
//! the chosen BUTTON LAYOUT (Black Ops II's DEFAULT, TACTICAL, LEFTY,
//! NUMBER, CHARLIE) and STICK LAYOUT. The layouts are `console::binds::
//! pad_layout`'s (the console crate depends on this one, so they are
//! listed again here; `layouts_match_the_console` keeps them in step by
//! hand-checked cases).

use bevy::prelude::*;
use frame::GameSettings;

use super::theme::*;
use super::view::{Ctx, Kids, abs};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum PadKey {
    A,
    B,
    X,
    Y,
    Lb,
    Rb,
    Lt,
    Rt,
    Ls,
    Rs,
    Dpad,
    View,
    Menu,
}

impl PadKey {
    fn glyph(self) -> &'static str {
        match self {
            Self::A => "A",
            Self::B => "B",
            Self::X => "X",
            Self::Y => "Y",
            Self::Lb => "LB",
            Self::Rb => "RB",
            Self::Lt => "LT",
            Self::Rt => "RT",
            Self::Ls => "LS",
            Self::Rs => "RS",
            Self::Dpad => "D-PAD",
            Self::View => "VIEW",
            Self::Menu => "MENU",
        }
    }

    /// The face buttons' Xbox colours.
    fn colour(self) -> Color {
        match self {
            Self::A => Color::srgb(0.38, 0.78, 0.30),
            Self::B => Color::srgb(0.88, 0.28, 0.24),
            Self::X => Color::srgb(0.22, 0.52, 0.95),
            Self::Y => Color::srgb(0.96, 0.78, 0.22),
            _ => KEYCAP,
        }
    }
}

/// Each button's action under `layout` (0 DEFAULT .. 4 CHARLIE; CUSTOM
/// shows DEFAULT's, his own binds changed from it).
pub(crate) fn actions(layout: u8) -> Vec<(PadKey, &'static str)> {
    use PadKey::*;
    let mut out = vec![
        (Lt, "AIM DOWN SIGHTS"),
        (Lb, "TACTICAL"),
        (Rt, "FIRE"),
        (Rb, "LETHAL"),
        (A, "JUMP"),
        (B, "CROUCH / PRONE"),
        (X, "RELOAD / USE"),
        (Y, "SWITCH WEAPON"),
        (Ls, "SPRINT / HOLD BREATH"),
        (Rs, "MELEE"),
        (Dpad, "SCORESTREAKS"),
        (View, "SCOREBOARD"),
        (Menu, "PAUSE"),
    ];
    let mut set = |button: PadKey, action: &'static str| {
        if let Some(row) = out.iter_mut().find(|(b, _)| *b == button) {
            row.1 = action;
        }
    };
    let layout = if layout == GameSettings::PAD_LAYOUT_CUSTOM { 0 } else { layout.min(4) };
    let tactical = |set: &mut dyn FnMut(PadKey, &'static str)| {
        set(B, "MELEE");
        set(Rs, "CROUCH / PRONE");
    };
    match layout {
        1 => tactical(&mut set),
        2 => {
            set(Lt, "FIRE");
            set(Rt, "AIM DOWN SIGHTS");
            set(Lb, "LETHAL");
            set(Rb, "TACTICAL");
        }
        3 | 4 => {
            set(Lb, "JUMP");
            set(A, "TACTICAL");
            if layout == 4 {
                tactical(&mut set);
            }
        }
        _ => {}
    }
    out
}

/// What each stick does under the stick layout.
fn sticks(stick_layout: u8) -> (&'static str, &'static str) {
    match stick_layout.min(3) {
        1 => ("LOOK", "MOVE"),
        2 => ("MOVE FORWARD · LOOK SIDEWAYS", "MOVE SIDEWAYS · LOOK UP"),
        3 => ("MOVE SIDEWAYS · LOOK UP", "MOVE FORWARD · LOOK SIDEWAYS"),
        _ => ("MOVE", "LOOK"),
    }
}

fn chip(p: &mut Kids, cx: &Ctx, b: PadKey, size: f32) {
    let round = matches!(b, PadKey::A | PadKey::B | PadKey::X | PadKey::Y | PadKey::Ls | PadKey::Rs);
    p.spawn((
        Node {
            min_width: cx.px(size),
            height: cx.px(size),
            padding: UiRect::horizontal(cx.px(if round { 0.0 } else { 8.0 })),
            justify_content: JustifyContent::Center,
            align_items: AlignItems::Center,
            border_radius: BorderRadius::all(if round { Val::Percent(50.0) } else { cx.px(6.0) }),
            ..default()
        },
        BackgroundColor(b.colour()),
    ))
    .with_child((Text::new(b.glyph()), cx.bold(size * 0.48), TextColor(WHITE)));
}

/// One button at a place on the drawn pad (design units inside the body).
fn on_body(p: &mut Kids, cx: &Ctx, b: PadKey, x: f32, y: f32, size: f32) {
    p.spawn(abs(Some(x - size / 2.0), Some(y - size / 2.0), None, None, cx)).with_children(|c| chip(c, cx, b, size));
}

/// The diagram, at the right of the options rows from `top` down.
pub(crate) fn view(c: &mut Kids, cx: &mut Ctx, s: &GameSettings, top: f32) {
    let rows = actions(s.pad_layout);
    let (left_stick, right_stick) = sticks(s.pad_stick_layout);
    c.spawn((
        Node {
            width: cx.px(760.0),
            flex_direction: FlexDirection::Column,
            row_gap: cx.px(14.0),
            padding: UiRect::all(cx.px(22.0)),
            border: UiRect::top(cx.px(3.0)),
            ..abs(None, Some(top), Some(64.0), None, cx)
        },
        BackgroundColor(cx.pal.panel),
        BorderColor::all(cx.pal.accent),
    ))
    .with_children(|p| {
        p.spawn(Node { justify_content: JustifyContent::SpaceBetween, ..default() }).with_children(|t| {
            t.spawn((Text::new("BUTTON LAYOUT"), cx.bold(20.0), TextColor(FAINT)));
            t.spawn((Text::new(super::options::pad_layout_name(s.pad_layout)), cx.bold(24.0), TextColor(cx.pal.accent)));
        });
        // The pad: shoulders over a rounded body, sticks, d-pad, the face
        // buttons, View and Menu where an Xbox controller has them.
        p.spawn(Node { width: Val::Percent(100.0), height: cx.px(226.0), justify_content: JustifyContent::Center, ..default() })
            .with_children(|row| {
                row.spawn(Node { width: cx.px(440.0), height: cx.px(226.0), ..default() }).with_children(|pad| {
                    pad.spawn((
                        Node { width: cx.px(440.0), height: cx.px(180.0), border_radius: BorderRadius::all(cx.px(70.0)), ..abs(Some(0.0), Some(44.0), None, None, cx) },
                        BackgroundColor(Color::srgba(0.70, 0.78, 0.90, 0.10)),
                    ));
                    on_body(pad, cx, PadKey::Lt, 70.0, 14.0, 30.0);
                    on_body(pad, cx, PadKey::Lb, 120.0, 34.0, 30.0);
                    on_body(pad, cx, PadKey::Rt, 370.0, 14.0, 30.0);
                    on_body(pad, cx, PadKey::Rb, 320.0, 34.0, 30.0);
                    on_body(pad, cx, PadKey::Ls, 96.0, 106.0, 46.0);
                    on_body(pad, cx, PadKey::Dpad, 160.0, 176.0, 30.0);
                    on_body(pad, cx, PadKey::Rs, 280.0, 176.0, 46.0);
                    on_body(pad, cx, PadKey::View, 186.0, 100.0, 26.0);
                    on_body(pad, cx, PadKey::Menu, 254.0, 100.0, 26.0);
                    on_body(pad, cx, PadKey::Y, 346.0, 76.0, 32.0);
                    on_body(pad, cx, PadKey::X, 312.0, 108.0, 32.0);
                    on_body(pad, cx, PadKey::B, 380.0, 108.0, 32.0);
                    on_body(pad, cx, PadKey::A, 346.0, 140.0, 32.0);
                });
            });
        // Every button's action, in two columns: the left hand's, the right's.
        let left = [PadKey::Lt, PadKey::Lb, PadKey::Ls, PadKey::Dpad, PadKey::View];
        let right = [PadKey::Rt, PadKey::Rb, PadKey::A, PadKey::B, PadKey::X, PadKey::Y, PadKey::Rs, PadKey::Menu];
        p.spawn(Node { column_gap: cx.px(28.0), ..default() }).with_children(|cols| {
            for side in [&left[..], &right[..]] {
                cols.spawn(Node { flex_direction: FlexDirection::Column, row_gap: cx.px(4.0), flex_grow: 1.0, flex_basis: Val::Px(0.0), ..default() })
                    .with_children(|col| {
                        for b in side {
                            let action = rows.iter().find(|(x, _)| x == b).map_or("", |r| r.1);
                            col.spawn(Node { column_gap: cx.px(12.0), align_items: AlignItems::Center, ..default() }).with_children(|line| {
                                line.spawn(Node { width: cx.px(64.0), justify_content: JustifyContent::Center, ..default() })
                                    .with_children(|c| chip(c, cx, *b, 26.0));
                                line.spawn((Text::new(action), cx.bold(18.0), TextColor(TEXT)));
                            });
                        }
                    });
            }
        });
        p.spawn((Node { height: Val::Px(1.0), width: Val::Percent(100.0), ..default() }, BackgroundColor(cx.pal.line)));
        p.spawn((
            Text::new(format!(
                "LEFT STICK: {left_stick}   ·   RIGHT STICK: {right_stick}\nDEAD ZONES: LEFT {}  ·  RIGHT {}",
                super::options::percent(s.pad_deadzone_left),
                super::options::percent(s.pad_deadzone_right)
            )),
            cx.regular(18.0),
            TextColor(DIM),
        ));
    });
}

#[cfg(test)]
mod tests {
    use super::*;

    fn action(layout: u8, b: PadKey) -> &'static str {
        actions(layout).into_iter().find(|(x, _)| *x == b).unwrap().1
    }

    /// The same cases as `console::binds::pad_layout`.
    #[test]
    fn layouts_match_the_console() {
        assert_eq!(action(0, PadKey::Rt), "FIRE");
        assert_eq!(action(0, PadKey::B), "CROUCH / PRONE");
        assert_eq!(action(1, PadKey::B), "MELEE");
        assert_eq!(action(1, PadKey::Rs), "CROUCH / PRONE");
        assert_eq!(action(2, PadKey::Lt), "FIRE");
        assert_eq!(action(2, PadKey::Rb), "TACTICAL");
        assert_eq!(action(3, PadKey::Lb), "JUMP");
        assert_eq!(action(3, PadKey::A), "TACTICAL");
        assert_eq!(action(3, PadKey::B), "CROUCH / PRONE");
        assert_eq!(action(4, PadKey::B), "MELEE");
        assert_eq!(action(GameSettings::PAD_LAYOUT_CUSTOM, PadKey::A), "JUMP");
    }
}
