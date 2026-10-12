use std::collections::{HashMap, HashSet, VecDeque};

use asset_game::MenuCatalog;
use assets::{PreparedLocalizedStrings, PreparedWeapons};
use bevy::prelude::*;
use hud_iw4::{
    GAME_MSG_WIN0_HORZ_ALIGN, GAME_MSG_WIN0_LINE_COUNT, GAME_MSG_WIN0_MSG_TIME_MS,
    GAME_MSG_WIN0_TEXT_SCALE, GAME_MSG_WIN0_TEXT_STYLE, GAME_MSG_WIN0_VERT_ALIGN, GAME_MSG_WIN0_X,
    KILLICON_DIED, game_msg_win0_line_y, gamenotify_line, killicon_stretch_uv,
    killicon_virtual_size, normalized_text_scale, obituary_mod, obituary_mod_killicon,
};
use net::LocalPresentClient;

use crate::chrome::text_width;
use crate::draw2d::{Draw2dCmd, Draw2dList, Draw2dOp, Draw2dProvenance, tessellate_fonts};
use crate::font_overlay::HUD_SMALL_FONT;
use crate::gaps::{GapCause, HudGap, HudPresentationGaps};
use crate::gpu_list::{HudTessPass, TessJob};
use crate::images::HudImages;
use crate::scorebar::milliseconds;

#[derive(Clone, Debug)]
enum KillfeedLine {
    Obituary {
        start_ms: i32,
        icon: String,
        icon_namespace: asset_core::AssetNamespace,
        attacker: String,
        has_attacker: bool,
        victim: String,
        attacker_team: i32,
        victim_team: i32,

        kill_icon_ratio: i32,

        flip_kill_icon: bool,
    },
    Notify {
        start_ms: i32,
        text: String,
        name_empty: bool,
    },
}

impl KillfeedLine {
    fn start_ms(&self) -> i32 {
        match self {
            Self::Obituary { start_ms, .. } | Self::Notify { start_ms, .. } => *start_ms,
        }
    }
}

struct KillIconPick {
    stem: String,

    namespace: asset_core::AssetNamespace,
    ratio: i32,
    flip: bool,
}

#[derive(Resource, Default)]
pub(crate) struct KillfeedWindow {
    lines: VecDeque<KillfeedLine>,
    bold: VecDeque<(i32, String)>,
    seen: HashSet<u32>,
}

const BOLD_LINE_COUNT: usize = 3;
const BOLD_MSG_TIME_MS: i32 = 3000;
const BOLD_TEXT_SCALE: f32 = 0.5;
const BOLD_Y: f32 = 100.0;

#[derive(Component)]
pub(crate) struct KillfeedRaster;

pub(crate) fn spawn_killfeed(root: &mut ChildSpawnerCommands) {
    crate::font_overlay::spawn_overlay(root, KillfeedRaster);
}

fn hide(pass: &mut HudTessPass) {
    pass.killfeed = TessJob::Hide;
}

fn pick_kill_icon(
    payload: &sim::EntityEventPayload,
    weapons: Option<&PreparedWeapons>,
) -> KillIconPick {
    let chrome = crate::images::HUD_CHROME_NAMESPACE;

    if let Some(mod_) = obituary_mod(payload.event_parm) {
        let Some(stem) = obituary_mod_killicon(mod_) else {
            return KillIconPick {
                stem: String::new(),
                namespace: chrome,
                ratio: 0,
                flip: false,
            };
        };
        return KillIconPick {
            stem: stem.to_owned(),
            namespace: chrome,
            ratio: 0,
            flip: false,
        };
    }
    let weapon = payload.event_parm as u32;
    let (ratio, flip) = match weapons.and_then(|reg| reg.0.facts_of(weapon)) {
        Some(facts) => (facts.kill_icon_ratio, facts.flip_kill_icon),
        None => (0, false),
    };
    let (stem, namespace) = if let Some(reg) = weapons {
        let ns = reg.0.namespace_of(weapon).unwrap_or(chrome);
        if let Some(image) = reg.0.kill_icon_image_of(weapon) {
            (image.to_owned(), ns)
        } else if let Some(name) = reg.0.kill_icon_of(weapon) {
            (name.to_owned(), ns)
        } else {
            (KILLICON_DIED.to_owned(), chrome)
        }
    } else {
        (KILLICON_DIED.to_owned(), chrome)
    };
    KillIconPick {
        stem,
        namespace,
        ratio,
        flip,
    }
}

fn snapshot_client_team(presented: &net::PresentedSnapshot, client: i32) -> i32 {
    if client < 0 {
        return 0;
    }
    presented
        .snapshot()
        .and_then(|snap| snap.meta.for_client(sim::ClientId(client as u32)))
        .map_or(0, |meta| meta.client_state_team)
}

fn obituary_name_color(local_team: i32, team: i32) -> [f32; 4] {
    if !matches!(local_team, 1 | 2) || !matches!(team, 1 | 2) {
        return [1.0, 1.0, 1.0, 1.0];
    }
    let rgb = if team == local_team {
        gamemode_iw4::TEAM_COLOR_MY_TEAM
    } else {
        gamemode_iw4::TEAM_COLOR_ENEMY_TEAM
    };
    let mut color = [1.0; 4];
    for (slot, value) in color.iter_mut().zip(rgb.split_whitespace()) {
        *slot = value.parse().unwrap_or(1.0);
    }
    color
}

fn snapshot_client_name(presented: &net::PresentedSnapshot, client: i32) -> String {
    if client < 0 {
        return String::new();
    }
    let Some(snap) = presented.snapshot() else {
        return String::new();
    };
    let Some(meta) = snap.meta.for_client(sim::ClientId(client as u32)) else {
        return String::new();
    };
    match entity_iw4::client_state_name(&meta.name) {
        Some(s) => s.to_owned(),
        None => String::new(),
    }
}

pub(crate) fn obituary(
    obituary: On<net::EntityObituary>,
    mut window: ResMut<KillfeedWindow>,
    weapons: Option<Res<PreparedWeapons>>,
    presented: Res<net::PresentedSnapshot>,
) {
    if obituary.in_killcam {
        return;
    }
    let payload = obituary.event.payload;
    let pick = pick_kill_icon(&payload, weapons.as_deref());
    let attacker = snapshot_client_name(&presented, payload.attacker_entity_num);
    let victim = snapshot_client_name(&presented, payload.other_entity_num);
    let attacker_team = snapshot_client_team(&presented, payload.attacker_entity_num);
    let victim_team = snapshot_client_team(&presented, payload.other_entity_num);
    let now = milliseconds() as i32;
    window.lines.push_back(KillfeedLine::Obituary {
        start_ms: now,
        icon: pick.stem,
        icon_namespace: pick.namespace,
        attacker,
        has_attacker: (0..18).contains(&payload.attacker_entity_num)
            && payload.attacker_entity_num != payload.other_entity_num,
        victim,
        attacker_team,
        victim_team,
        kill_icon_ratio: pick.ratio,
        flip_kill_icon: pick.flip,
    });
    while window.lines.len() > GAME_MSG_WIN0_LINE_COUNT {
        window.lines.pop_front();
    }
}

fn text_cmd(
    x: f32,
    y: f32,
    cmd_w: f32,
    cmd_h: f32,
    material: String,
    text: String,
    color: [f32; 4],
    site: &'static str,
) -> Draw2dCmd {
    Draw2dCmd {
        material_namespace: crate::images::HUD_CHROME_NAMESPACE,
        x: (x + 0.5).floor(),
        y: (y + 0.5).floor(),
        w: cmd_w,
        h: cmd_h,
        s0: 0.0,
        t0: 0.0,
        s1: 1.0,
        t1: 1.0,
        color,
        material,
        op: Draw2dOp::TextRun {
            font: HUD_SMALL_FONT.to_owned(),
            scale: cmd_w,
            text,
            loc_key: String::new(),

            style: GAME_MSG_WIN0_TEXT_STYLE,
            fx: None,
            glow: None,
        },
        provenance: Draw2dProvenance::CgDraw { site },
        layer: 1,
    }
}

#[allow(clippy::too_many_arguments)]
pub(crate) fn update_killfeed(
    surface: Res<crate::surface::Hud2dSurface>,
    presented: Res<net::PresentedSnapshot>,
    local: Res<LocalPresentClient>,
    catalog: Option<Res<MenuCatalog>>,
    strings: Option<Res<PreparedLocalizedStrings>>,
    mut hud_images: ResMut<HudImages>,
    mut images: ResMut<Assets<Image>>,
    mut gaps: ResMut<HudPresentationGaps>,
    mut window: ResMut<KillfeedWindow>,
    mut pass: ResMut<HudTessPass>,
    mut notifies: MessageReader<net::SvcGameNotify>,
    view: Option<Res<frame::ViewSubject>>,
) {
    if !surface.is_ready() {
        hide(&mut pass);
        return;
    }
    if presented.player(local.0).is_none() {
        window.lines.clear();
        window.bold.clear();
        window.seen.clear();
        gaps.clear(HudGap::Obituary);
        hide(&mut pass);
        return;
    }

    let now = milliseconds() as i32;
    for cmd in notifies.read() {
        if !window.seen.insert(cmd.id) {
            continue;
        }
        let Some(template) = strings
            .as_ref()
            .and_then(|s| crate::hudelem::resolve_hud_text(s, &cmd.key))
        else {
            gaps.raise(GapCause::LocalizedRowMissing {
                key: cmd.key.clone(),
            });
            continue;
        };
        let args: Vec<String> = cmd
            .name
            .split(sim::HUD_PRINT_ARG_SEPARATOR)
            .map(|arg| {
                strings
                    .as_ref()
                    .and_then(|s| crate::hudelem::resolve_hud_text(s, arg))
                    .unwrap_or_else(|| arg.to_owned())
            })
            .collect();
        let text = if template.contains("&&2") {
            args.iter().enumerate().fold(template, |line, (i, arg)| {
                line.replace(&format!("&&{}", i + 1), arg)
            })
        } else {
            gamenotify_line(&template, &args[0])
        };
        if cmd.tag == net::SVC_PRINT_BOLD {
            window.bold.push_back((now, text));
            while window.bold.len() > BOLD_LINE_COUNT {
                window.bold.pop_front();
            }
            continue;
        }
        window.lines.push_back(KillfeedLine::Notify {
            start_ms: now,
            text,
            name_empty: cmd.tag == net::SVC_DISCONNECT_NOTIFY && cmd.name.is_empty(),
        });
        while window.lines.len() > GAME_MSG_WIN0_LINE_COUNT {
            window.lines.pop_front();
        }
    }
    window
        .lines
        .retain(|line| now.saturating_sub(line.start_ms()) < GAME_MSG_WIN0_MSG_TIME_MS);
    window
        .bold
        .retain(|(start, _)| now.saturating_sub(*start) < BOLD_MSG_TIME_MS);
    if window.lines.is_empty() && window.bold.is_empty() {
        gaps.clear(HudGap::Obituary);
        hide(&mut pass);
        return;
    }
    if view.is_some_and(|v| v.in_killcam()) {
        hide(&mut pass);
        return;
    }

    let names_ok = match window.lines.back() {
        None => true,
        Some(KillfeedLine::Obituary {
            has_attacker,
            attacker,
            victim,
            ..
        }) => (!*has_attacker || !attacker.is_empty()) && !victim.is_empty(),
        Some(KillfeedLine::Notify { text, .. }) => !text.is_empty(),
    };
    match window.lines.back() {
        None => {}
        Some(KillfeedLine::Obituary { .. }) => {
            if !names_ok {
                gaps.raise(GapCause::ObituaryNoClientInfo);
            } else {
                gaps.clear(HudGap::Obituary);
            }
        }
        Some(KillfeedLine::Notify { name_empty, .. }) => {
            if *name_empty {
                gaps.raise(GapCause::GameNotifyNoClientInfo);
            } else if names_ok {
                gaps.clear(HudGap::Obituary);
            }
        }
    }

    let local_team = snapshot_client_team(&presented, local.0.0 as i32);
    let font = catalog.as_ref().and_then(|c| c.font(HUD_SMALL_FONT));
    let (nscale, font_material) = match font {
        Some(def) => (
            normalized_text_scale(def.pixel_height, GAME_MSG_WIN0_TEXT_SCALE),
            asset_core::AssetRef::bare_name(&def.material).to_owned(),
        ),
        None => (0.0, String::new()),
    };
    if names_ok && font.is_none() {
        gaps.raise(GapCause::ObituaryNoClientInfo);
    }

    let miss = hud_images.miss_reason();
    let mut cmds = Vec::new();
    let mut fonts = HashMap::new();
    let font_tex_ok = if let Some(def) = font {
        fonts.insert(HUD_SMALL_FONT.to_owned(), def);
        hud_images
            .get(
                crate::images::HUD_CHROME_NAMESPACE,
                &font_material,
                &mut images,
            )
            .is_some()
    } else {
        false
    };
    if names_ok && font.is_some() && !font_tex_ok {
        let image = hud_images
            .zone_image_name(&font_material)
            .map(str::to_owned);
        gaps.raise(GapCause::FontAtlasMissing {
            material: font_material.clone(),
            image,
        });
    }

    for (i, line) in window.lines.iter().rev().enumerate() {
        let first_cmd = cmds.len();
        let age = now.saturating_sub(line.start_ms());

        let alpha = (age as f32 / 250.0).clamp(0.0, 1.0)
            * ((GAME_MSG_WIN0_MSG_TIME_MS - age) as f32 / 500.0).clamp(0.0, 1.0);
        let y_virtual = game_msg_win0_line_y(i);
        match line {
            KillfeedLine::Notify { text, .. } => {
                if font.is_some() && font_tex_ok {
                    let applied = surface.apply_rect(
                        GAME_MSG_WIN0_X,
                        y_virtual,
                        nscale,
                        nscale,
                        GAME_MSG_WIN0_HORZ_ALIGN,
                        GAME_MSG_WIN0_VERT_ALIGN,
                    );
                    cmds.push(text_cmd(
                        applied.x,
                        applied.y,
                        applied.w,
                        applied.h,
                        font_material.clone(),
                        text.clone(),
                        [1.0; 4],
                        "killfeed_game_msg",
                    ));
                }
            }
            KillfeedLine::Obituary {
                icon,
                icon_namespace,
                attacker,
                has_attacker,
                victim,
                attacker_team,
                victim_team,
                kill_icon_ratio,
                flip_kill_icon,
                ..
            } => {
                let mut x_virtual = GAME_MSG_WIN0_X;
                let line_names = (!*has_attacker || !attacker.is_empty())
                    && !victim.is_empty()
                    && font.is_some()
                    && font_tex_ok;
                if line_names && *has_attacker {
                    if let Some(def) = font {
                        let attacker_w = text_width(def, attacker) as f32 * nscale;
                        let applied = surface.apply_rect(
                            x_virtual,
                            y_virtual,
                            nscale,
                            nscale,
                            GAME_MSG_WIN0_HORZ_ALIGN,
                            GAME_MSG_WIN0_VERT_ALIGN,
                        );
                        cmds.push(text_cmd(
                            applied.x,
                            applied.y,
                            applied.w,
                            applied.h,
                            font_material.clone(),
                            attacker.clone(),
                            obituary_name_color(local_team, *attacker_team),
                            "killfeed_obituary",
                        ));
                        x_virtual += attacker_w + GAME_MSG_WIN0_TEXT_SCALE * 4.0;
                    }
                }

                let (icon_vw, icon_vh) = killicon_virtual_size(*kill_icon_ratio);
                let (s0, s1) = killicon_stretch_uv(*flip_kill_icon);
                let placed = surface.apply_rect(
                    x_virtual,
                    y_virtual - icon_vh,
                    icon_vw,
                    icon_vh,
                    GAME_MSG_WIN0_HORZ_ALIGN,
                    GAME_MSG_WIN0_VERT_ALIGN,
                );
                let icon_ok = hud_images.get(*icon_namespace, icon, &mut images).is_some();
                if !icon_ok {
                    gaps.raise(GapCause::ObituaryKillIconMissing {
                        name: icon.clone(),
                        miss,
                    });
                } else {
                    cmds.push(Draw2dCmd {
                        material_namespace: *icon_namespace,
                        x: (placed.x + 0.5).floor(),
                        y: (placed.y + 0.5).floor(),
                        w: placed.w,
                        h: placed.h,
                        s0,
                        t0: 0.0,
                        s1,
                        t1: 1.0,
                        color: [1.0, 1.0, 1.0, 1.0],
                        material: icon.clone(),
                        op: Draw2dOp::StretchPic,
                        provenance: Draw2dProvenance::CgDraw {
                            site: "killfeed_obituary",
                        },
                        layer: 1,
                    });
                }
                if line_names {
                    x_virtual += icon_vw + GAME_MSG_WIN0_TEXT_SCALE * 4.0;
                    let applied = surface.apply_rect(
                        x_virtual,
                        y_virtual,
                        nscale,
                        nscale,
                        GAME_MSG_WIN0_HORZ_ALIGN,
                        GAME_MSG_WIN0_VERT_ALIGN,
                    );
                    cmds.push(text_cmd(
                        applied.x,
                        applied.y,
                        applied.w,
                        applied.h,
                        font_material.clone(),
                        victim.clone(),
                        obituary_name_color(local_team, *victim_team),
                        "killfeed_obituary",
                    ));
                }
            }
        }
        for cmd in &mut cmds[first_cmd..] {
            cmd.color[3] *= alpha;
        }
    }
    if let Some(def) = font.filter(|_| font_tex_ok) {
        let scale = normalized_text_scale(def.pixel_height, BOLD_TEXT_SCALE);
        for (i, (start, text)) in window.bold.iter().enumerate() {
            let age = now.saturating_sub(*start);
            let alpha = ((BOLD_MSG_TIME_MS - age) as f32 / 500.0).clamp(0.0, 1.0);
            let width = text_width(def, text) as f32 * scale;
            let line_h = def.pixel_height as f32 * scale;
            let applied = surface.apply_rect(
                -width / 2.0,
                BOLD_Y + line_h * i as f32,
                scale,
                scale,
                hud_iw4::ALIGN_CENTER,
                hud_iw4::ALIGN_VIEWABLE,
            );
            cmds.push(text_cmd(
                applied.x,
                applied.y,
                applied.w,
                applied.h,
                font_material.clone(),
                text.clone(),
                [1.0, 1.0, 1.0, alpha],
                "killfeed_bold_msg",
            ));
        }
    }

    let list = Draw2dList { cmds };
    let (quads, _) = tessellate_fonts(&list, &fonts);
    if quads.is_empty() {
        hide(&mut pass);
        return;
    }
    pass.killfeed = TessJob::Quads(quads);
}

/// bo2mp: kill feed pictures for other games' guns in a Black Ops II-rules
/// match (BO1 and MW2 rows as extra BO2 weapons). Black Ops II's own come
/// from its zones (`assets::lane::t6_m2::hud_icons`) as
/// `killicon:<base>:<ratio>`; the BO2 HUD finds a line's picture by the
/// weapon's script name less any `+attachments` and its `_mp`
/// (`lui_frontend::feed_line`). A foreign row's script name is
/// `bo1_<name>_mp` / `mw2_<name>_mp` (`mw3_` for MW3), so its key is
/// `killicon:bo1_<name>:<ratio>`; the picture is its WeaponDef kill icon,
/// read from that game's own IWDs. Once per weapon catalog.
pub(crate) fn foreign_kill_icons(
    weapons: Option<Res<PreparedWeapons>>,
    icons: Option<ResMut<assets::T6HudIcons>>,
    mut hud_images: ResMut<HudImages>,
    mut done: Local<usize>,
) {
    let (Some(weapons), Some(mut icons)) = (weapons, icons) else {
        return;
    };
    let key = std::sync::Arc::as_ptr(&weapons.0) as usize;
    if *done == key || !assets::t6_rules() || !hud_images.has_games_root() {
        return;
    }
    *done = key;
    let reg = &weapons.0;
    let (mut added, mut missing) = (0usize, Vec::new());
    for id in 1..=reg.len() as u32 {
        let Some(ns) = reg.namespace_of(id) else { continue };
        let Some(base) = foreign_kill_icon_base(ns, reg.name_of(id)) else {
            continue;
        };
        let ratio = reg.facts_of(id).map_or(0, |f| f.kill_icon_ratio);
        let prefix = format!("killicon:{base}:");
        if icons.0.iter().any(|(k, _)| k.starts_with(&prefix)) {
            continue;
        }
        let stems: Vec<String> = [reg.kill_icon_image_of(id), reg.kill_icon_of(id)]
            .into_iter()
            .flatten()
            .map(str::to_owned)
            .collect();
        let found = stems.iter().find_map(|stem| {
            hud_images.ensure_rgba(ns, stem);
            hud_images.rgba(ns, stem).cloned()
        });
        let Some((w, h, rgba)) = found.filter(|(w, h, rgba)| *w > 0 && *h > 0 && rgba.len() == (*w * *h * 4) as usize)
        else {
            if missing.len() < 12 {
                missing.push(format!("{base} ({})", stems.join("|")));
            }
            continue;
        };
        let image = Image::new(
            bevy::render::render_resource::Extent3d {
                width: w,
                height: h,
                depth_or_array_layers: 1,
            },
            bevy::render::render_resource::TextureDimension::D2,
            rgba,
            bevy::render::render_resource::TextureFormat::Rgba8UnormSrgb,
            bevy::asset::RenderAssetUsages::default(),
        );
        icons.0.push((format!("{prefix}{ratio}"), std::sync::Arc::new(image)));
        added += 1;
    }
    if added > 0 || !missing.is_empty() {
        diag::info!(
            Ui,
            "bo2 foreign kill icons: {added} added (keys killicon:<bo1_|mw2_|mw3_><name>:<ratio>); no picture for {}: {missing:?}",
            missing.len()
        );
    }
}

/// The kill feed's key for another game's gun (see `foreign_kill_icons`):
/// `bo1_ak74u` for BO1's `ak74u_mp`, `mw2_m4` for MW2's `m4_mp`; a name
/// that already carries its prefix keeps it. `None` for BO2's own.
pub fn foreign_kill_icon_base(ns: asset_core::AssetNamespace, name: &str) -> Option<String> {
    let prefix = match ns {
        asset_core::AssetNamespace::T5 => "bo1_",
        asset_core::AssetNamespace::Iw4 => "mw2_",
        asset_core::AssetNamespace::Iw5 => "mw3_",
        asset_core::AssetNamespace::Iw6 => "gh_",
        asset_core::AssetNamespace::T6 => return None,
    };
    let base = name.split('+').next().unwrap_or(name).trim_end_matches("_mp");
    if base.is_empty() || base == "none" {
        return None;
    }
    Some(if base.starts_with(prefix) {
        base.to_owned()
    } else {
        format!("{prefix}{base}")
    })
}

#[cfg(test)]
mod foreign_kill_icon_tests {
    use super::foreign_kill_icon_base;
    use asset_core::AssetNamespace;

    #[test]
    fn keys_match_the_feed_rule() {
        assert_eq!(foreign_kill_icon_base(AssetNamespace::T5, "ak74u_mp").as_deref(), Some("bo1_ak74u"));
        assert_eq!(foreign_kill_icon_base(AssetNamespace::Iw4, "m4_mp").as_deref(), Some("mw2_m4"));
        assert_eq!(foreign_kill_icon_base(AssetNamespace::T5, "bo1_ak74u_mp").as_deref(), Some("bo1_ak74u"));
        assert_eq!(foreign_kill_icon_base(AssetNamespace::Iw4, "m4_reflex_mp").as_deref(), Some("mw2_m4_reflex"));
        assert_eq!(foreign_kill_icon_base(AssetNamespace::T6, "an94_mp"), None);
    }
}
