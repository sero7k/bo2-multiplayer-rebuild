//! bo2mp: controller button pictures.
//!
//! While he plays with a controller (its buttons or sticks were the last
//! thing he touched) BO2's menus and prompts are told so
//! (`Engine.LastInput_Gamepad`, `UIExpression.IsUsingGamepad`, the
//! `input_source_changed` event) and write their button glyphs
//! (`^BBUTTON_LUI_PRIMARY^`, ...), which are drawn with the Xbox 360 button
//! icons in `assets/ui/controller` (built into the game). A key or the mouse
//! puts the keyboard prompts back.

use std::sync::atomic::{AtomicBool, Ordering};

use bevy::input::gamepad::{Gamepad, GamepadAxis, GamepadButton};
use bevy::input::mouse::MouseMotion;
use bevy::prelude::*;
use frame::ClientSet;

static PAD: AtomicBool = AtomicBool::new(false);
static CHANGED: AtomicBool = AtomicBool::new(false);

/// The controller is what he plays with now.
pub(crate) fn pad_active() -> bool {
    PAD.load(Ordering::Relaxed)
}

/// The input source changed since the last call (once per change).
pub(crate) fn take_change() -> bool {
    CHANGED.swap(false, Ordering::Relaxed)
}

/// The icons, by material name (`xenonbutton_a`), as the game was built.
const ICONS: &[(&str, &[u8])] = &[
    ("xenon_controller_top", include_bytes!("../../../assets/ui/controller/xenon_controller_top.iwi")),
    ("xenonbutton_a", include_bytes!("../../../assets/ui/controller/xenonbutton_a.iwi")),
    ("xenonbutton_b", include_bytes!("../../../assets/ui/controller/xenonbutton_b.iwi")),
    ("xenonbutton_back", include_bytes!("../../../assets/ui/controller/xenonbutton_back.iwi")),
    ("xenonbutton_dpad_all", include_bytes!("../../../assets/ui/controller/xenonbutton_dpad_all.iwi")),
    ("xenonbutton_dpad_down", include_bytes!("../../../assets/ui/controller/xenonbutton_dpad_down.iwi")),
    ("xenonbutton_dpad_left", include_bytes!("../../../assets/ui/controller/xenonbutton_dpad_left.iwi")),
    ("xenonbutton_dpad_right", include_bytes!("../../../assets/ui/controller/xenonbutton_dpad_right.iwi")),
    ("xenonbutton_dpad_rl", include_bytes!("../../../assets/ui/controller/xenonbutton_dpad_rl.iwi")),
    ("xenonbutton_dpad_ud", include_bytes!("../../../assets/ui/controller/xenonbutton_dpad_ud.iwi")),
    ("xenonbutton_dpad_up", include_bytes!("../../../assets/ui/controller/xenonbutton_dpad_up.iwi")),
    ("xenonbutton_lb", include_bytes!("../../../assets/ui/controller/xenonbutton_lb.iwi")),
    ("xenonbutton_ls", include_bytes!("../../../assets/ui/controller/xenonbutton_ls.iwi")),
    ("xenonbutton_lt", include_bytes!("../../../assets/ui/controller/xenonbutton_lt.iwi")),
    ("xenonbutton_rb", include_bytes!("../../../assets/ui/controller/xenonbutton_rb.iwi")),
    ("xenonbutton_rs", include_bytes!("../../../assets/ui/controller/xenonbutton_rs.iwi")),
    ("xenonbutton_rt", include_bytes!("../../../assets/ui/controller/xenonbutton_rt.iwi")),
    ("xenonbutton_start", include_bytes!("../../../assets/ui/controller/xenonbutton_start.iwi")),
    ("xenonbutton_x", include_bytes!("../../../assets/ui/controller/xenonbutton_x.iwi")),
    ("xenonbutton_y", include_bytes!("../../../assets/ui/controller/xenonbutton_y.iwi")),
];

/// The icon a BO2 button glyph name (`BUTTON_LUI_PRIMARY`, `BUTTON_USE`,
/// without the `^B` / `^`) is drawn with on an Xbox controller (BO2's
/// default layout).
pub(crate) fn icon_for(button: &str) -> Option<&'static str> {
    let b = button.to_ascii_uppercase();
    let b = b.strip_prefix("BBUTTON").map_or(b.clone(), |rest| format!("BUTTON{rest}"));
    let b = b.strip_prefix("BUTTON_").unwrap_or(&b);
    let b = b.strip_prefix("LUI_").unwrap_or(b);
    Some(match b.trim_end_matches("_ACTIVE") {
        "PRIMARY" | "A" | "JUMP" | "GOSTAND" => "xenonbutton_a",
        "SECONDARY" | "B" | "STANCE" | "CROUCH" | "PRONE" => "xenonbutton_b",
        "ALT1" | "ALT" | "X" | "USE" | "USERELOAD" | "RELOAD" | "ACTIVATE" => "xenonbutton_x",
        "ALT2" | "Y" | "WEAPNEXT" | "SWITCH" => "xenonbutton_y",
        "SHOULDERL" | "LSHLDR" | "LB" | "CYCLE_LEFT" | "SMOKE" | "SECONDARY_OFFHAND" => "xenonbutton_lb",
        "SHOULDERR" | "RSHLDR" | "RB" | "CYCLE_RIGHT" | "FRAG" | "THROW" => "xenonbutton_rb",
        "LEFT_TRIGGER" | "LTRIG" | "LT" | "ADS" | "SPEED_THROW" | "TOGGLEADS_THROW" => "xenonbutton_lt",
        "RIGHT_TRIGGER" | "RTRIG" | "RT" | "ATTACK" | "FIRE" => "xenonbutton_rt",
        "LEFT_STICK" | "LEFT_STICK_UP" | "LSTICK" | "LS" | "SPRINT" | "BREATH_SPRINT" => "xenonbutton_ls",
        "RIGHT_STICK" | "RSTICK" | "RS" | "MELEE" => "xenonbutton_rs",
        "START" | "MENU" | "PAUSE" => "xenonbutton_start",
        "SELECT" | "BACK" | "SCORES" | "SCOREBOARD" => "xenonbutton_back",
        "DPAD_ALL" => "xenonbutton_dpad_all",
        "DPAD_U" | "DPAD_UP" | "ACTIONSLOT1" => "xenonbutton_dpad_up",
        "DPAD_D" | "DPAD_DOWN" | "ACTIONSLOT2" => "xenonbutton_dpad_down",
        "DPAD_L" | "DPAD_LEFT" | "ACTIONSLOT3" => "xenonbutton_dpad_left",
        "DPAD_R" | "DPAD_RIGHT" | "ACTIONSLOT4" => "xenonbutton_dpad_right",
        "DPAD_UD" => "xenonbutton_dpad_ud",
        "DPAD_RL" | "DPAD_LR" => "xenonbutton_dpad_rl",
        _ => return None,
    })
}

/// An icon's picture (decoded on first use, then kept).
pub(crate) fn icon_image(name: &str) -> Option<Image> {
    let (_, bytes) = ICONS.iter().find(|(n, _)| *n == name)?;
    match asset_material::decode_iwi_rgba(bytes) {
        Ok((w, h, rgba)) => Some(Image::new(
            bevy::render::render_resource::Extent3d { width: w, height: h, depth_or_array_layers: 1 },
            bevy::render::render_resource::TextureDimension::D2,
            rgba,
            bevy::render::render_resource::TextureFormat::Rgba8UnormSrgb,
            bevy::asset::RenderAssetUsages::default(),
        )),
        Err(error) => {
            diag::warn!(Ui, "controller icon `{name}` did not decode: {error}");
            None
        }
    }
}

/// Which input he used last: a pad's button or a stick pushed past its
/// dead zone, or a key, a mouse button or the mouse moving.
fn track_input(
    pads: Query<&Gamepad>,
    keys: Res<ButtonInput<KeyCode>>,
    mouse: Res<ButtonInput<MouseButton>>,
    mut motion: MessageReader<MouseMotion>,
) {
    let moved: f32 = motion.read().map(|m| m.delta.length()).sum();
    let pad_used = pads.iter().any(|p| {
        p.get_just_pressed().next().is_some()
            || [GamepadAxis::LeftStickX, GamepadAxis::LeftStickY, GamepadAxis::RightStickX, GamepadAxis::RightStickY]
                .iter()
                .any(|a| p.get(*a).is_some_and(|v| v.abs() > 0.5))
            || p.pressed(GamepadButton::LeftTrigger2)
            || p.pressed(GamepadButton::RightTrigger2)
    });
    let desk_used = keys.get_just_pressed().next().is_some() || mouse.get_just_pressed().next().is_some() || moved > 4.0;
    let now = if pad_used {
        true
    } else if desk_used {
        false
    } else {
        pad_active()
    };
    if now != pad_active() {
        PAD.store(now, Ordering::Relaxed);
        CHANGED.store(true, Ordering::Relaxed);
        hks_t6::host::set_gamepad_active(now);
        diag::info!(Ui, "input: {}", if now { "controller" } else { "keyboard and mouse" });
    }
}

pub(crate) fn register_pad_glyphs(app: &mut App) {
    app.add_systems(Update, track_input.in_set(ClientSet::Ui));
}
