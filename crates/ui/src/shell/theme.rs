//! bo2mp shell: the one look every screen shares - the front end, the
//! loading screen and the in-game HUD (`hud_modern` uses the same blue and
//! panels): near-black navy panels, one blue accent, white text that dims
//! when it is not the thing in focus, and a condensed sans.

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

const REGULAR: &[u8] = include_bytes!("../../assets/BarlowCondensed-400.ttf");
const SEMIBOLD: &[u8] = include_bytes!("../../assets/BarlowCondensed-600.ttf");

/// The shell's two weights (the in-game HUD's `GameUiFont` is the semibold).
#[derive(Resource, Clone)]
pub(crate) struct ShellFonts {
    pub regular: Handle<Font>,
    pub bold: Handle<Font>,
}

pub(crate) fn load_fonts(mut commands: Commands, mut fonts: ResMut<Assets<Font>>) {
    commands.insert_resource(ShellFonts {
        regular: fonts.add(Font::from_bytes(REGULAR.to_vec())),
        bold: fonts.add(Font::from_bytes(SEMIBOLD.to_vec())),
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
