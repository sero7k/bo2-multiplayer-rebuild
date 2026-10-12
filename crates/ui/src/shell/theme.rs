//! bo2mp shell: the one look every screen shares - the front end, the
//! loading screen and the in-game HUD (`hud_modern` uses the same blue and
//! panels): near-black navy panels, one blue accent, white text that dims
//! when it is not the thing in focus, and a condensed sans. The Zombies
//! screens are the same screens in red (`Palette::ZOMBIES`).

use bevy::prelude::*;

pub(crate) const ACCENT: Color = Color::srgb(0.26, 0.62, 0.95);
pub(crate) const ACCENT_SOFT: Color = Color::srgba(0.26, 0.62, 0.95, 0.30);
pub(crate) const ENEMY: Color = Color::srgb(0.90, 0.26, 0.22);
pub(crate) const WHITE: Color = Color::srgb(1.0, 1.0, 1.0);
pub(crate) const TEXT: Color = Color::srgba(0.92, 0.94, 0.97, 0.92);
pub(crate) const DIM: Color = Color::srgba(0.80, 0.84, 0.90, 0.58);
pub(crate) const FAINT: Color = Color::srgba(0.80, 0.84, 0.90, 0.32);
pub(crate) const GOLD: Color = Color::srgb(0.98, 0.72, 0.30);
pub(crate) const GREEN: Color = Color::srgb(0.36, 0.86, 0.48);
pub(crate) const PANEL: Color = Color::srgba(0.025, 0.04, 0.07, 0.82);
pub(crate) const PANEL_SOFT: Color = Color::srgba(0.025, 0.04, 0.07, 0.55);
pub(crate) const LINE: Color = Color::srgba(0.70, 0.80, 0.95, 0.14);
pub(crate) const KEYCAP: Color = Color::srgba(0.70, 0.78, 0.90, 0.22);

/// The colours a front-end screen is drawn in: Multiplayer's blue, or the
/// Zombies screens' red (`palette`, the one place the look switches). The
/// loading screen and the HUD keep the blue constants above.
#[derive(Clone, Copy, Debug, PartialEq)]
pub(crate) struct Palette {
    pub accent: Color,
    pub accent_soft: Color,
    pub panel: Color,
    pub panel_soft: Color,
    pub line: Color,
    /// The footer bar and the map tiles' name strip.
    pub footer: Color,
    pub strip: Color,
    /// An empty picture box.
    pub picture_bg: Color,
    /// The backdrop: the dark it fades into (rgb), and the tint over it.
    pub shade: [f32; 3],
    pub tint: Color,
}

impl Palette {
    pub const MULTIPLAYER: Palette = Palette {
        accent: ACCENT,
        accent_soft: ACCENT_SOFT,
        panel: PANEL,
        panel_soft: PANEL_SOFT,
        line: LINE,
        footer: Color::srgba(0.01, 0.02, 0.04, 0.72),
        strip: Color::srgba(0.0, 0.02, 0.05, 0.88),
        picture_bg: Color::srgb(0.05, 0.07, 0.10),
        shade: [0.008, 0.016, 0.035],
        tint: Color::srgba(0.02, 0.06, 0.14, 0.30),
    };

    /// Zombies: the same screens in blood red over near-black.
    pub const ZOMBIES: Palette = Palette {
        accent: Color::srgb(0.88, 0.17, 0.13),
        accent_soft: Color::srgba(0.88, 0.17, 0.13, 0.30),
        panel: Color::srgba(0.06, 0.018, 0.018, 0.84),
        panel_soft: Color::srgba(0.06, 0.018, 0.018, 0.56),
        line: Color::srgba(0.95, 0.70, 0.68, 0.14),
        footer: Color::srgba(0.035, 0.008, 0.008, 0.74),
        strip: Color::srgba(0.05, 0.0, 0.0, 0.88),
        picture_bg: Color::srgb(0.10, 0.04, 0.04),
        shade: [0.030, 0.006, 0.006],
        tint: Color::srgba(0.16, 0.01, 0.01, 0.32),
    };

    /// The accent at another opacity.
    pub fn accent_a(&self, a: f32) -> Color {
        self.accent.with_alpha(a)
    }
}

/// The palette a screen is drawn in.
pub(crate) fn palette(screen: super::Screen) -> Palette {
    if screen.is_zombies() { Palette::ZOMBIES } else { Palette::MULTIPLAYER }
}

const REGULAR: &[u8] = include_bytes!("../../assets/BarlowCondensed-400.ttf");
const SEMIBOLD: &[u8] = include_bytes!("../../assets/BarlowCondensed-600.ttf");
/// The countdown's thin numerals.
const LIGHT: &[u8] = include_bytes!("../../assets/BarlowCondensed-200.ttf");
/// The main menu's CALL OF DUTY and the end screen's VICTORY / DEFEAT.
const BLACK: &[u8] = include_bytes!("../../assets/BarlowCondensed-900.ttf");

/// The shell's weights (the in-game HUD's `GameUiFont` is the semibold;
/// the thin one is the lobby countdown's, the black one the logo's).
#[derive(Resource, Clone)]
pub(crate) struct ShellFonts {
    pub regular: Handle<Font>,
    pub bold: Handle<Font>,
    pub light: Handle<Font>,
    pub black: Handle<Font>,
}

pub(crate) fn load_fonts(mut commands: Commands, mut fonts: ResMut<Assets<Font>>) {
    commands.insert_resource(ShellFonts {
        regular: fonts.add(Font::from_bytes(REGULAR.to_vec())),
        bold: fonts.add(Font::from_bytes(SEMIBOLD.to_vec())),
        light: fonts.add(Font::from_bytes(LIGHT.to_vec())),
        black: fonts.add(Font::from_bytes(BLACK.to_vec())),
    });
}

pub(crate) fn font(h: &Handle<Font>, px: f32) -> TextFont {
    TextFont {
        font: h.clone().into(),
        font_size: bevy::text::FontSize::Px(px.max(1.0)),
        ..default()
    }
}

/// A gradient from `from` on the left to clear on the right.
pub(crate) fn fade_right(from: Color, to: Color) -> BackgroundGradient {
    BackgroundGradient::from(LinearGradient::to_right(vec![
        ColorStop::new(from, Val::Percent(0.0)),
        ColorStop::new(to, Val::Percent(100.0)),
    ]))
}
