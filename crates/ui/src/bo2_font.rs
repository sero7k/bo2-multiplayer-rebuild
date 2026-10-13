//! bo2zm: text in Black Ops II's own HUD fonts (`assets::T6HudFonts`): each
//! letter a piece of the font sheet, placed as the font's glyph table says
//! (its box against the pen and the baseline, then the pen moves on by the
//! letter's advance).

use std::collections::HashMap;

use bevy::prelude::*;

/// The sheet as an image and each font by the name the HUD's Lua scripts
/// give it (Default, Big, Morris, ...).
#[derive(Resource)]
pub(crate) struct Bo2Fonts {
    pub sheet: Handle<Image>,
    /// The sheet's pixels as plain RGBA8 (for blurring covered text).
    cpu: Option<std::sync::Arc<Image>>,
    fonts: HashMap<String, Bo2Font>,
    /// bo2zm: the PlayStation face buttons (□ ○ △ ×), drawn here: BO2 on PC
    /// has only Xbox button pictures and its fonts have no such shapes.
    icons: HashMap<char, Handle<Image>>,
}

/// The characters drawn as button icons: the PlayStation shapes and BO2's
/// Xbox button pictures (`frame::BO2_XBOX_GLYPHS`).
pub(crate) fn is_button_icon(c: char) -> bool {
    matches!(c, '□' | '○' | '△' | '×' | '\u{E100}'..='\u{E114}')
}

/// One PlayStation face button, 64 x 64: a dark disc with a light rim and
/// the shape in its own colour (square pink, circle red, triangle green,
/// cross blue), edges smoothed.
fn button_icon(c: char) -> Image {
    use bevy::asset::RenderAssetUsages;
    use bevy::render::render_resource::{Extent3d, TextureDimension, TextureFormat};
    const N: usize = 64;
    let (rgb, kind) = match c {
        '□' => ([0.95, 0.55, 0.82], 0),
        '○' => ([1.0, 0.33, 0.36], 1),
        '△' => ([0.3, 0.86, 0.72], 2),
        _ => ([0.48, 0.62, 1.0], 3),
    };
    let mut px = vec![0u8; N * N * 4];
    let edge = |d: f32| (0.5 - d).clamp(0.0, 1.0);
    for y in 0..N {
        for x in 0..N {
            let (fx, fy) = (x as f32 + 0.5 - 32.0, y as f32 + 0.5 - 32.0);
            let r = (fx * fx + fy * fy).sqrt();
            // The disc and its rim.
            let disc = edge(r - 30.5);
            let rim = edge((r - 29.0).abs() - 1.5);
            // The shape's stroke (distance from its outline, 4.5 wide).
            let d = match kind {
                0 => (fx.abs().max(fy.abs()) - 13.0).abs(),
                1 => (r - 14.0).abs(),
                2 => {
                    // An upward triangle, centre slightly low.
                    let (qx, qy) = (fx, fy - 3.0);
                    let k = 3f32.sqrt();
                    let a = (k * qx.abs() - qy) * 0.5 - 8.0;
                    let b = qy - 8.0;
                    a.max(b).abs()
                }
                _ => {
                    let a = (fx - fy).abs() / 2f32.sqrt();
                    let b = (fx + fy).abs() / 2f32.sqrt();
                    let inside = fx.abs().max(fy.abs()) <= 13.0;
                    if inside { a.min(b) } else { 99.0 }
                }
            };
            let shape = edge(d - 2.25);
            let base = [0.08, 0.08, 0.09];
            let mut col = [0.0f32; 3];
            for i in 0..3 {
                let with_rim = base[i] + (0.85 - base[i]) * rim * 0.6;
                col[i] = with_rim + (rgb[i] - with_rim) * shape;
            }
            let o = (y * N + x) * 4;
            for i in 0..3 {
                px[o + i] = (col[i].clamp(0.0, 1.0) * 255.0).round() as u8;
            }
            px[o + 3] = (disc * 255.0).round() as u8;
        }
    }
    Image::new(
        Extent3d { width: N as u32, height: N as u32, depth_or_array_layers: 1 },
        TextureDimension::D2,
        px,
        TextureFormat::Rgba8UnormSrgb,
        RenderAssetUsages::default(),
    )
}

struct Bo2Font {
    pixel_height: f32,
    /// Above and below the baseline, over every glyph's box.
    ascent: f32,
    descent: f32,
    glyphs: HashMap<char, Bo2Glyph>,
}

#[derive(Clone, Copy)]
struct Bo2Glyph {
    x0: f32,
    y0: f32,
    dx: f32,
    w: f32,
    h: f32,
    rect: Rect,
}

impl Bo2Fonts {
    fn font(&self, name: &str) -> Option<&Bo2Font> {
        self.fonts.get(name).or_else(|| self.fonts.get("Default"))
    }

    /// Each font's pixel height, letter advances and space advance (the
    /// LUI scripts' text measure).
    pub fn advances(&self) -> HashMap<String, (f32, HashMap<char, f32>, f32)> {
        self.fonts
            .iter()
            .map(|(name, f)| {
                let space = f.glyphs.get(&' ').map_or(f.pixel_height * 0.3, |g| g.dx);
<<<<<<< /home/claude/bo2-multiplayer-rebuild/crates/ui/src/bo2_font.rs
                (name.clone(), (f.pixel_height, f.glyphs.iter().map(|(c, g)| (*c, g.dx)).collect(), space))
=======
                let mut adv: HashMap<char, f32> = f.glyphs.iter().map(|(c, g)| (*c, g.dx)).collect();
                // Button pictures: a little wider than the font is tall.
                for c in self.icons.keys() {
                    adv.insert(*c, f.pixel_height * 1.1);
                }
                (name.clone(), (f.pixel_height, adv, space))
>>>>>>> /tmp/claude-0/zmhead/crates/ui/src/bo2_font.rs
            })
            .collect()
    }

    /// How wide `text` is in `font` at `px` (as `spawn_line` lays it out).
    pub fn line_width(&self, font: &str, text: &str, px: f32) -> f32 {
        let Some(f) = self.font(font) else {
            return 0.0;
        };
        let s = px / f.pixel_height.max(1.0);
        let space = f.glyphs.get(&' ').map_or(f.pixel_height * 0.3, |g| g.dx);
        text.chars()
            .map(|c| match f.glyphs.get(&c).or_else(|| f.glyphs.get(&'?')) {
                _ if is_button_icon(c) => f.pixel_height * 1.1,
                Some(g) if c != ' ' => g.dx,
                _ => space,
            })
            .sum::<f32>()
            * s
    }

<<<<<<< /home/claude/bo2-multiplayer-rebuild/crates/ui/src/bo2_font.rs
    /// bo2mp: the glyph coverage of `lines` (text, left edge in units from the
    /// block's left) in `font` at `px_u` units tall, laid out as `spawn_line`
    /// does one line under the other, as a white RGBA8 picture whose alpha is
    /// the coverage: `w_u` x block height units plus `pad` units clear all
    /// round, `res` texels to the unit. Returns it with its size in texels
    /// and the block's height in units; the caller blurs it, as BO2's engine
    /// blurs the menu layer it has drawn the text into.
    pub fn coverage(&self, font: &str, lines: &[(String, f32)], px_u: f32, w_u: f32, pad: f32, res: f32) -> Option<(Vec<u8>, usize, usize, f32)> {
        let f = self.font(font)?;
        let sheet = self.cpu.as_ref()?;
        let data = sheet.data.as_ref()?;
        let (sw, sh) = (sheet.width() as usize, sheet.height() as usize);
        if data.len() < sw * sh * 4 {
            return None;
        }
        let s = px_u / f.pixel_height.max(1.0);
        let line_h = (f.ascent + f.descent) * s;
        let block_h = line_h * lines.len() as f32;
        let tw = (((w_u + 2.0 * pad) * res).ceil() as usize).max(1);
        let th = (((block_h + 2.0 * pad) * res).ceil() as usize).max(1);
        let mut px = vec![255u8; tw * th * 4];
        for p in px.chunks_exact_mut(4) {
            p[3] = 0;
        }
        let space = f.glyphs.get(&' ').map_or(f.pixel_height * 0.3, |g| g.dx);
        let at = |x: f32, y: f32| -> f32 {
            let (xi, yi) = (x.floor() as isize, y.floor() as isize);
            if xi < 0 || yi < 0 || xi as usize >= sw || yi as usize >= sh {
                return 0.0;
            }
            f32::from(data[(yi as usize * sw + xi as usize) * 4 + 3]) / 255.0
        };
        for (i, (text, left)) in lines.iter().enumerate() {
            let mut pen = 0.0f32;
            for c in text.chars() {
                let g = match f.glyphs.get(&c).or_else(|| f.glyphs.get(&'?')) {
                    Some(g) if c != ' ' => g,
                    _ => {
                        pen += space;
                        continue;
                    }
                };
                // The glyph's box in units, then in texels of the block.
                let gx = left + (pen + g.x0) * s;
                let gy = i as f32 * line_h + (f.ascent + g.y0) * s;
                let (gw, gh) = ((g.w * s).max(1e-3), (g.h * s).max(1e-3));
                let tx0 = ((gx + pad) * res).floor().max(0.0) as usize;
                let ty0 = ((gy + pad) * res).floor().max(0.0) as usize;
                let tx1 = (((gx + gw + pad) * res).ceil() as usize).min(tw);
                let ty1 = (((gy + gh + pad) * res).ceil() as usize).min(th);
                for ty in ty0..ty1 {
                    for tx in tx0..tx1 {
                        // Each texel averages a 3 x 3 sampling of the sheet over its footprint.
                        let mut sum = 0.0f32;
                        for sy in 0..3 {
                            for sx in 0..3 {
                                let ux = (tx as f32 + (sx as f32 + 0.5) / 3.0) / res - pad;
                                let uy = (ty as f32 + (sy as f32 + 0.5) / 3.0) / res - pad;
                                let (fx, fy) = ((ux - gx) / gw, (uy - gy) / gh);
                                if !(0.0..1.0).contains(&fx) || !(0.0..1.0).contains(&fy) {
                                    continue;
                                }
                                sum += at(g.rect.min.x + fx * g.rect.width(), g.rect.min.y + fy * g.rect.height());
                            }
                        }
                        let a = (sum / 9.0 * 255.0).round() as u32;
                        let o = (ty * tw + tx) * 4 + 3;
                        px[o] = (u32::from(px[o]) + a).min(255) as u8;
                    }
                }
                pen += g.dx;
            }
        }
        Some((px, tw, th, block_h))
    }

    /// bo2mp: one line's height in units at `px_u` (what `coverage` stacks).
    pub fn coverage_height(&self, font: &str, px_u: f32) -> f32 {
        self.font(font).map_or(0.0, |f| (f.ascent + f.descent) * px_u / f.pixel_height.max(1.0))
    }

=======
>>>>>>> /tmp/claude-0/zmhead/crates/ui/src/bo2_font.rs
    /// One line of coloured runs in `font`, `px` tall for the font's own
    /// pixel height, as a node of its size holding a glyph image per letter
    /// (a dark copy under each, offset by `shadow` pixels, when it is not
    /// zero). Returns the line's size.
    pub fn spawn_line(
        &self,
        parent: &mut ChildSpawnerCommands,
        font: &str,
        runs: &[(String, Color)],
        px: f32,
        shadow: f32,
    ) -> Vec2 {
        self.spawn_line_shadowed(parent, font, runs, px, shadow, 0.75)
    }

    /// bo2mp: `font`'s own pixel height (the em its glyphs were cut at).
    pub fn pixel_height(&self, font: &str) -> f32 {
        self.font(font).map_or(0.0, |f| f.pixel_height)
    }

    /// bo2mp: `spawn_line` with the shadow's alpha as a share of the text's
    /// (BO2's engine draws a hudelem's drop shadow at the text's own alpha).
    pub fn spawn_line_shadowed(
        &self,
        parent: &mut ChildSpawnerCommands,
        font: &str,
        runs: &[(String, Color)],
        px: f32,
        shadow: f32,
        shadow_alpha: f32,
    ) -> Vec2 {
        let Some(f) = self.font(font) else {
            return Vec2::ZERO;
        };
        let s = px / f.pixel_height.max(1.0);
        let space = f.glyphs.get(&' ').map_or(f.pixel_height * 0.3, |g| g.dx);
        let mut placed: Vec<(Bo2Glyph, f32, Color)> = Vec::new();
        let mut icons: Vec<(char, f32)> = Vec::new();
        let mut pen = 0.0f32;
        for (text, color) in runs {
            for c in text.chars() {
                if is_button_icon(c) && self.icons.contains_key(&c) {
                    icons.push((c, pen));
                    pen += f.pixel_height * 1.1;
                    continue;
                }
                match f.glyphs.get(&c).or_else(|| f.glyphs.get(&'?')) {
                    Some(g) if c != ' ' => {
                        placed.push((*g, pen, *color));
                        pen += g.dx;
                    }
                    _ => pen += space,
                }
            }
        }
        let size = Vec2::new(pen * s, (f.ascent + f.descent) * s);
        let sheet = self.sheet.clone();
        parent
            .spawn(Node {
                width: Val::Px(size.x),
                height: Val::Px(size.y),
                ..default()
            })
            .with_children(|line| {
                let mut glyph = |g: &Bo2Glyph, pen: f32, color: Color, off: f32| {
                    line.spawn((
                        Node {
                            position_type: PositionType::Absolute,
                            left: Val::Px((pen + g.x0) * s + off),
                            top: Val::Px((f.ascent + g.y0) * s + off),
                            width: Val::Px(g.w * s),
                            height: Val::Px(g.h * s),
                            ..default()
                        },
                        ImageNode {
                            image: sheet.clone(),
                            rect: Some(g.rect),
                            color,
                            ..default()
                        },
                    ));
                };
                if shadow != 0.0 {
                    for (g, pen, color) in &placed {
                        let a = color.alpha() * shadow_alpha;
                        glyph(g, *pen, Color::srgba(0.0, 0.0, 0.0, a), shadow);
                    }
                }
                for (g, pen, color) in &placed {
                    glyph(g, *pen, *color, 0.0);
                }
                // Button icons: a font's height, centred on the line.
                let side = f.pixel_height * s;
                for (c, pen) in &icons {
                    let Some(image) = self.icons.get(c) else { continue };
                    line.spawn((
                        Node {
                            position_type: PositionType::Absolute,
                            left: Val::Px(pen * s + side * 0.05),
                            top: Val::Px(((f.ascent + f.descent) * s - side) * 0.5),
                            width: Val::Px(side),
                            height: Val::Px(side),
                            ..default()
                        },
                        ImageNode::new(image.clone()),
                    ));
                }
            });
        size
    }
}

/// Once the map's fonts arrive: the sheet as an image, each font's glyphs.
pub(crate) fn load_bo2_fonts(
    mut commands: Commands,
    staged: Option<Res<assets::T6HudFonts>>,
    hud_icons: Option<Res<assets::T6HudIcons>>,
    mut images: ResMut<Assets<Image>>,
) {
    let Some(staged) = staged else {
        return;
    };
<<<<<<< /home/claude/bo2-multiplayer-rebuild/crates/ui/src/bo2_font.rs
    if !staged.is_changed() {
=======
    if !staged.is_changed() && !hud_icons.as_ref().is_some_and(|i| i.is_changed()) {
>>>>>>> /tmp/claude-0/zmhead/crates/ui/src/bo2_font.rs
        return;
    }
    let Some(sheet) = staged.sheet.as_ref() else {
        commands.remove_resource::<Bo2Fonts>();
        return;
    };
    let w = sheet.texture_descriptor.size.width as f32;
    let h = sheet.texture_descriptor.size.height as f32;
    let fonts = staged
        .fonts
        .iter()
        .map(|font| {
            let glyphs: HashMap<char, Bo2Glyph> = font
                .glyphs
                .iter()
                .filter_map(|(letter, m, st)| {
                    let c = char::from_u32(u32::from(*letter))?;
                    Some((
                        c,
                        Bo2Glyph {
                            x0: f32::from(m[0]),
                            y0: f32::from(m[1]),
                            dx: f32::from(m[2]),
                            w: f32::from(m[3]),
                            h: f32::from(m[4]),
                            rect: Rect::new(st[0] * w, st[1] * h, st[2] * w, st[3] * h),
                        },
                    ))
                })
                .collect();
<<<<<<< /home/claude/bo2-multiplayer-rebuild/crates/ui/src/bo2_font.rs
            // The glyph tables' y0 is against the line's reference, which
            // the engine puts one pixelHeight below the line's top (the line
            // step of `RB_DrawText` and `R_TextHeight` is the font's
            // pixelHeight): ExtraSmall's tallest glyph
            // reaches 21 over a 20 px font and simply stands out of its box.
            let ascent = font.pixel_height;
=======
            let ascent = glyphs
                .values()
                .map(|g| -g.y0)
                .fold(font.pixel_height * 0.8, f32::max);
>>>>>>> /tmp/claude-0/zmhead/crates/ui/src/bo2_font.rs
            let descent = glyphs.values().map(|g| g.y0 + g.h).fold(0.0, f32::max);
            (
                font.name.clone(),
                Bo2Font {
                    pixel_height: font.pixel_height,
                    ascent,
                    descent,
                    glyphs,
                },
            )
        })
        .collect();
    let mut icons: HashMap<char, Handle<Image>> = ['□', '○', '△', '×']
        .into_iter()
        .map(|c| (c, images.add(button_icon(c))))
        .collect();
    // BO2's own Xbox button pictures, once the HUD's pictures are in.
    if let Some(hud) = hud_icons.as_deref() {
        for (c, name) in frame::BO2_XBOX_GLYPHS {
            if let Some((_, image)) = hud.0.iter().find(|(n, _)| n == name) {
                icons.insert(c, images.add((**image).clone()));
            }
        }
    }
    commands.insert_resource(Bo2Fonts {
        sheet: images.add((**sheet).clone()),
        cpu: assets::plain_rgba8(sheet).map(std::sync::Arc::new),
        fonts,
        icons,
    });
}
