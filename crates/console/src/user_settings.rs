use std::{fs, path::PathBuf};

use bevy::{
    audio::{AudioSink, AudioSinkPlayback, GlobalVolume, Volume},
    input::{
        ButtonInput,
        keyboard::KeyCode,
        mouse::{MouseButton, MouseWheel},
    },
    prelude::*,
};

use crate::{BindButton, KeyBinds, binds::wheel_button, display_button};

#[derive(Resource, Default)]
pub(crate) struct PendingMenuBinding {
    id: Option<u32>,
    armed: bool,
}

#[derive(Resource, Default)]
pub(crate) struct UserSettingsPersistence {
    path: Option<PathBuf>,
    last_payload: Option<String>,
}

#[derive(Resource)]
pub(crate) struct AppliedMasterVolume(f32);

pub(crate) fn load_user_settings(
    identity: Res<ui::LaunchIdentity>,
    mut settings: ResMut<frame::GameSettings>,
    mut binds: ResMut<KeyBinds>,
    mut persistence: ResMut<UserSettingsPersistence>,
) {
    let Some(path) = settings_path(&identity.artifacts) else {
        warn!("no HOME or XDG_CONFIG_HOME; user settings are session-only");
        return;
    };
    persistence.path = Some(path.clone());
    match fs::read_to_string(&path) {
        Ok(source) => parse_settings(&source, &mut settings, &mut binds),
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => {}
        Err(error) => warn!("could not read {}: {error}", path.display()),
    }
    settings.sanitize();
    // bo2mp: the game always boots borderless fullscreen (Options can still
    // make this session windowed).
    settings.fullscreen = true;
    persistence.last_payload = Some(serialize_settings(&settings, &binds));
    // bo2mp: his friend code, made the first time (saved with the next
    // change of settings, and at once here).
    if settings.friend_code == 0 {
        settings.friend_code = new_friend_code();
        settings.touch();
        persistence.last_payload = None;
    }
}

/// bo2mp: a fresh 8-digit friend code (from the clock, the process and the
/// standard library's random hasher seed).
fn new_friend_code() -> u32 {
    use std::hash::{BuildHasher, Hasher};
    let mut h = std::collections::hash_map::RandomState::new().build_hasher();
    h.write_u128(std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).map_or(0, |d| d.as_nanos()));
    h.write_u32(std::process::id());
    10_000_000 + (h.finish() % 90_000_000) as u32
}

pub(crate) fn consume_menu_binding(
    mut intents: MessageReader<frame::UiBindRequest>,
    keys: Res<ButtonInput<KeyCode>>,
    mouse: Res<ButtonInput<MouseButton>>,
    mut wheel: MessageReader<MouseWheel>,
    (gamepads, active, mut settings): (
        Query<&bevy::input::gamepad::Gamepad>,
        Res<frame::ActivePad>,
        ResMut<frame::GameSettings>,
    ),
    mut pending: ResMut<PendingMenuBinding>,
    mut capture: ResMut<frame::UiBindingCapture>,
    mut binds: ResMut<KeyBinds>,
    mut view: ResMut<ui::BindingView>,
) {
    capture.consumed_input = false;
    if capture.command.is_none() {
        pending.id = None;
        view.listening = None;
    }
    let wheel_direction = wheel
        .read()
        .fold(None, |first, event| first.or_else(|| wheel_button(event.y)));
    let mut began = false;
    for intent in intents.read() {
        if let Some(id) = input_iw4::command_id_lookup(&intent.command) {
            capture.command = Some(intent.command.clone());
            pending.id = Some(id);
            view.listening = Some(id);
            view.revision = view.revision.wrapping_add(1);
            pending.armed = false;
            began = true;
        } else {
            capture.command = None;
            pending.id = None;
            view.listening = None;
        }
    }
    let Some(id) = pending.id else { return };
    if began || !pending.armed {
        pending.armed = true;
        return;
    }
    let pad = active.0.and_then(|entity| gamepads.get(entity).ok());
    let pad_start =
        pad.is_some_and(|pad| pad.just_pressed(bevy::input::gamepad::GamepadButton::Start));
    let pad_button = pad.and_then(|pad| {
        pad.get_just_pressed()
            .copied()
            .filter_map(crate::PadButton::from_gamepad_button)
            .find(|button| *button != crate::PadButton::Start)
    });
    if let Some(button) = pad_button.filter(|_| !pad_start) {
        capture.command = None;
        capture.consumed_input = true;
        binds.clear_command_on(id, true);
        binds.set(BindButton::Pad(button), id);
        if settings.pad_layout != frame::GameSettings::PAD_LAYOUT_CUSTOM {
            settings.pad_layout = frame::GameSettings::PAD_LAYOUT_CUSTOM;
            settings.touch();
        }
        pending.id = None;
        pending.armed = false;
        view.listening = None;
        view.revision = view.revision.wrapping_add(1);
        return;
    }
    if keys.just_pressed(KeyCode::Escape) || pad_start {
        capture.command = None;
        capture.consumed_input = true;
        pending.id = None;
        pending.armed = false;
        view.listening = None;
        view.revision = view.revision.wrapping_add(1);
        return;
    }
    let button = keys
        .get_just_pressed()
        .copied()
        .map(BindButton::Key)
        .next()
        .or_else(|| {
            mouse
                .get_just_pressed()
                .copied()
                .map(BindButton::Mouse)
                .next()
        })
        .or(wheel_direction);
    let Some(button) = button else { return };
    capture.command = None;
    capture.consumed_input = true;
    binds.clear_command_on(id, false);
    binds.set(button, id);
    pending.id = None;
    pending.armed = false;
    view.listening = None;
    view.revision = view.revision.wrapping_add(1);
}

pub(crate) fn sync_binding_view(
    binds: Res<KeyBinds>,
    devices: Res<frame::InputDevices>,
    mut view: ResMut<ui::BindingView>,
    mut dvars: ResMut<frame::UiMenuDvars>,
    mut style: Local<Option<frame::PromptStyle>>,
) {
    if !binds.is_changed() && *style == Some(devices.style) {
        return;
    }
    *style = Some(devices.style);
    let mut chords = std::collections::BTreeMap::<u32, (Vec<String>, Vec<String>)>::new();
    for (button, id) in binds.iter() {
        let (keys, buttons) = chords.entry(id).or_default();
        match button {
            BindButton::Pad(pad) => buttons.push(pad.prompt(devices.style).to_owned()),
            _ => keys.push(display_button(button)),
        }
    }
    view.chords.clear();
    for (id, (mut keys, mut buttons)) in chords {
        keys.sort();
        keys.dedup();
        buttons.sort();
        buttons.dedup();
        let text = match (keys.is_empty(), buttons.is_empty()) {
            (false, false) => format!("{} | {}", keys.join(" OR "), buttons.join(" OR ")),
            (false, true) => keys.join(" OR "),
            (true, false) => buttons.join(" OR "),
            (true, true) => continue,
        };
        view.chords.insert(id, text);
    }
    // Publish every command so removing its last binding clears the old label.
    for (id, command) in input_iw4::INPUT_COMMAND_NAMES.iter().enumerate().skip(1) {
        dvars.set(&format!("ui_bind_{command}"), view.chord(id as u32));
    }
    view.revision = view.revision.wrapping_add(1);
}

pub(crate) fn apply_master_volume(
    settings: Res<frame::GameSettings>,
    mut global: ResMut<GlobalVolume>,
    mut sinks: Query<(Entity, &mut AudioSink, &bevy::audio::PlaybackSettings)>,
    mut muted_volumes: Local<std::collections::HashMap<Entity, f32>>,
    applied: Option<ResMut<AppliedMasterVolume>>,
    mut commands: Commands,
) {
    if !settings.is_changed() {
        return;
    }
    let previous = applied.as_ref().map_or(1.0, |value| value.0);
    let next = settings.master_volume;
    global.volume = Volume::Linear(next);
    muted_volumes.retain(|entity, _| sinks.contains(*entity));
    for (entity, mut sink, playback) in &mut sinks {
        let base = if previous > f32::EPSILON {
            sink.volume().to_linear() / previous
        } else {
            muted_volumes
                .get(&entity)
                .copied()
                .unwrap_or(playback.volume.to_linear())
        };
        if next == 0.0 {
            muted_volumes.insert(entity, base);
        }
        sink.set_volume(Volume::Linear(base * next));
    }
    if next > 0.0 {
        muted_volumes.clear();
    }
    if let Some(mut applied) = applied {
        applied.0 = next;
    } else {
        commands.insert_resource(AppliedMasterVolume(next));
    }
}

pub(crate) fn sync_player_name(
    settings: Res<frame::GameSettings>,
    generation: Res<frame::WorldGeneration>,
    has_world: Res<frame::HasWorld>,
    role: Res<frame::RuntimeRole>,
    local: Option<Res<net::LocalPresentClient>>,
    link: Option<Res<net::UdpClientLink>>,
    mut inbox: Option<ResMut<net::ClientActionInbox>>,
    mut seq: ResMut<net::ActionRequestIds>,
    mut sent: Local<Option<(frame::WorldGeneration, sim::ClientId, [u8; 16])>>,
) {
    if !has_world.0 || *role == frame::RuntimeRole::Replay {
        *sent = None;
        return;
    }
    let (Some(local), Some(inbox)) = (local, inbox.as_deref_mut()) else {
        return;
    };
    if let Some(link) = link
        && (link.connection.is_none()
            || link.assigned_client != Some(local.0)
            || !link.has_entered_match())
    {
        *sent = None;
        return;
    }
<<<<<<< /home/claude/bo2-multiplayer-rebuild/crates/console/src/user_settings.rs
    let name = entity_iw4::pack_client_state_name(&settings.player_name);
=======
    let name = entity_iw4::pack_client_state_name(&settings.shown_name());
>>>>>>> /tmp/claude-0/zmhead/crates/console/src/user_settings.rs
    let next = (*generation, local.0, name);
    if sent.as_ref() == Some(&next) {
        return;
    }
    let request_id = seq.allocate();
    if let Err(error) = inbox.push(local.0, sim::ClientAction::SetName { request_id, name }) {
        diag::warn!(
            Console,
            "name: request_id={request_id} not queued — {error}"
        );
    } else {
        *sent = Some(next);
    }
}

pub(crate) fn save_user_settings(
    settings: Res<frame::GameSettings>,
    binds: Res<KeyBinds>,
    mut persistence: ResMut<UserSettingsPersistence>,
) {
    if !settings.is_changed() && !binds.is_changed() {
        return;
    }
    let payload = serialize_settings(&settings, &binds);
    if persistence.last_payload.as_deref() == Some(payload.as_str()) {
        return;
    }
    let Some(path) = persistence.path.clone() else {
        return;
    };
    let Some(parent) = path.parent() else { return };
    if let Err(error) = fs::create_dir_all(parent) {
        warn!("could not create {}: {error}", parent.display());
        return;
    }
    let temporary = path.with_extension("cfg.tmp");
    if let Err(error) =
        fs::write(&temporary, payload.as_bytes()).and_then(|()| fs::rename(&temporary, &path))
    {
        warn!("could not atomically save {}: {error}", path.display());
        return;
    }
    persistence.last_payload = Some(payload);
}

fn settings_path(artifacts: &std::path::Path) -> Option<PathBuf> {
    if let Some(path) = std::env::var_os("IW4L_SETTINGS_PATH") {
        return Some(PathBuf::from(path));
    }
    if cfg!(windows) {
        return Some(artifacts.join("settings.cfg"));
    }
    std::env::var_os("XDG_CONFIG_HOME")
        .map(PathBuf::from)
        .or_else(|| std::env::var_os("HOME").map(|path| PathBuf::from(path).join(".config")))
        .map(|path| path.join("iw4l/settings.cfg"))
}

pub(crate) fn apply_pad_layout_setting(
    settings: Res<frame::GameSettings>,
    mut binds: ResMut<KeyBinds>,
    mut seen: Local<Option<u8>>,
) {
    let layout = settings.pad_layout;
    let previous = seen.replace(layout);
    if previous.is_none()
        || previous == Some(layout)
        || layout == frame::GameSettings::PAD_LAYOUT_CUSTOM
    {
        return;
    }
    binds.apply_pad_layout(usize::from(layout));
}

<<<<<<< /home/claude/bo2-multiplayer-rebuild/crates/console/src/user_settings.rs
/// bo2zm: the "Omni Movement" setting, for the next game this PC starts
=======
/// bo2zm: his "Omni Movement" setting, for the next game this PC starts
>>>>>>> /tmp/claude-0/zmhead/crates/console/src/user_settings.rs
/// (a game reads it once, when it starts; a second PC gets the host's).
pub(crate) fn push_omni_movement_setting(settings: Res<frame::GameSettings>) {
    if settings.is_changed() {
        sim::set_omni_movement(settings.omni_movement);
    }
}

fn parse_into<T: std::str::FromStr>(value: &str, slot: &mut T) {
    if let Ok(value) = value.parse() {
        *slot = value;
    }
}

fn serialize_settings(settings: &frame::GameSettings, binds: &KeyBinds) -> String {
    let safe_name = settings.player_name.replace(['\n', '\r', '='], " ");
    let mut lines = vec![
        "// IW4L user settings v2".to_owned(),
        format!(
            "resolution={}x{}",
            settings.resolution.width, settings.resolution.height
        ),
        format!("fullscreen={}", settings.fullscreen),
        format!("vsync={}", settings.vsync),
        format!("master_volume={:.3}", settings.master_volume),
        format!("music_volume={:.3}", settings.music_volume),
<<<<<<< /home/claude/bo2-multiplayer-rebuild/crates/console/src/user_settings.rs
        format!("match_music={}", settings.match_music),
=======
        format!("sfx_volume={:.3}", settings.sfx_volume),
        format!("voice_volume={:.3}", settings.voice_volume),
        format!("cinematic_volume={:.3}", settings.cinematic_volume),
>>>>>>> /tmp/claude-0/zmhead/crates/console/src/user_settings.rs
        format!("brightness={:.3}", settings.brightness),
        format!("fov={:.0}", settings.fov),
        format!("shadows={}", settings.shadows),
        format!("depth_of_field={}", settings.depth_of_field),
        format!("bloom={}", settings.bloom),
        format!("sensitivity={:.3}", settings.sensitivity),
        format!("invert_mouse={}", settings.invert_mouse),
        format!("player_name={safe_name}"),
        format!("pad_layout={}", settings.pad_layout),
        format!("pad_stick_layout={}", settings.pad_stick_layout),
        format!("pad_sensitivity={:.2}", settings.pad_sensitivity),
        format!("pad_ads_sensitivity={:.2}", settings.pad_ads_sensitivity),
        format!("pad_invert={}", settings.pad_invert),
        format!("pad_curve={}", settings.pad_curve),
        format!("pad_aim_assist={}", settings.pad_aim_assist),
        format!("pad_prompts={}", settings.pad_prompts),
        format!("pad_vibration={}", settings.pad_vibration),
        format!("pad_deadzone_left={:.2}", settings.pad_deadzone_left),
        format!("pad_deadzone_right={:.2}", settings.pad_deadzone_right),
        format!(
            "bo2_quality={}",
            if settings.bo2_fast { "fast" } else { "full" }
        ),
<<<<<<< /home/claude/bo2-multiplayer-rebuild/crates/console/src/user_settings.rs
        format!("omni_movement={}", settings.omni_movement),
        format!("map_game_bodies={}", settings.map_game_bodies),
        format!("friend_code={}", settings.friend_code),
        format!(
            "friends={}",
            settings.friends.iter().map(u32::to_string).collect::<Vec<_>>().join(",")
        ),
=======
        format!("mature={}", settings.mature),
        format!("corpses={}", settings.corpses),
        format!("draw_fps={}", settings.draw_fps),
        format!("max_fps={}", settings.max_fps),
        format!("fxaa={}", settings.fxaa),
        format!("aa_samples={}", settings.aa_samples),
        format!("tex_filter={}", settings.tex_filter),
        format!("tex_quality={}", settings.tex_quality),
        format!("omni_movement={}", settings.omni_movement),
>>>>>>> /tmp/claude-0/zmhead/crates/console/src/user_settings.rs
        format!("binds_layout={BINDS_LAYOUT}"),
        "unbindall".to_owned(),
    ];
    lines.extend(binds.list_lines());
    lines.push(String::new());
    lines.join("\n")
}

<<<<<<< /home/claude/bo2-multiplayer-rebuild/crates/console/src/user_settings.rs
/// bo2zm: the saved binds' layout. 4 = Black Ops II's own PC keys
/// (`default_controls.cfg`, from the retail bindings_mp.bdg); a file with an
/// older layout (or none, layout 1) is moved over on load.
const BINDS_LAYOUT: u32 = 4;

/// Older layouts' keyboard keys go back to Black Ops II's PC defaults (his
/// 10-07 "use black ops defaults": the old E use / F frag were our own
/// defaults, not his picks). Saved pad buttons stay.
fn migrate_binds(script: &str, from_layout: u32) -> String {
    if from_layout >= BINDS_LAYOUT {
        return script.to_owned();
    }
    let mut out = String::from("unbindall\n");
    let defaults = crate::binds::DEFAULT_CONTROLS
        .lines()
        .map(str::trim)
        .filter(|l| l.starts_with("bind "));
    let pad = script.lines().map(str::trim).filter(|l| {
        let mut parts = l.split(' ');
        parts.next() == Some("bind")
            && parts.next().is_some_and(|k| {
                let k = k.to_ascii_uppercase();
                k.starts_with("BUTTON_") || k.starts_with("DPAD_")
            })
    });
    for line in defaults.chain(pad) {
        out.push_str(line);
=======
/// bo2zm: the saved binds' layout. 2 = Black Ops II's own PC keys for
/// grenades and the d-pad (G frag, 4 monkey bomb, 3 claymore, 2 and 5 the
/// other slots); 3 = the rest of its keys (F use, C crouch and prone, CTRL
/// prone toggle, wheel up next gun, middle mouse frag). An older file is
/// moved over on load.
const BINDS_LAYOUT: u32 = 3;

/// A key still on an older layout's default, its old and new command;
/// `None` unbinds it (Black Ops II's key does something we do not have).
type Move = (&'static str, &'static str, Option<&'static str>);

/// Layout 1 (ours) to 2: Black Ops II's grenade and d-pad keys.
const MOVES_2: [Move; 4] = [
    ("g", "+smoke", Some("+frag")),
    ("4", "+actionslot 4", Some("+smoke")),
    ("3", "+right", Some("+actionslot 4")),
    ("2", "+left", Some("+actionslot 2")),
];

/// Layout 2 to 3: the rest of Black Ops II's default_mp_controls.cfg. E
/// is its lean, which we do not have.
const MOVES_3: [Move; 9] = [
    ("f", "+frag", Some("+activate")),
    ("e", "+activate", None),
    ("c", "+movedown", Some("+stance")),
    ("ctrl", "+prone", Some("toggleprone")),
    ("shift", "+sprint", Some("+breath_sprint")),
    ("mouse3", "weapnext", Some("+frag")),
    ("mwheelup", "weapprev", Some("weapnext")),
    ("mwheeldown", "weapnext", Some("weapprev")),
    ("1", "weapnext", Some("+actionslot 1")),
];

/// The saved keys of layout `from` as the current layout's (Black Ops
/// II's): a key still on the old default moves; one he bound himself stays.
fn migrate_binds(script: &str, from: u32) -> String {
    let mut out = script.to_owned();
    if from < 2 {
        out = apply_moves(&out, &MOVES_2);
        if !out.lines().any(|l| l.split(' ').nth(1) == Some("5")) {
            out.push_str("bind 5 +actionslot 3\n");
        }
    }
    if from < 3 {
        out = apply_moves(&out, &MOVES_3);
    }
    out
}

fn apply_moves(script: &str, moves: &[Move]) -> String {
    let mut out = String::new();
    for line in script.lines() {
        let mut parts = line.splitn(3, ' ');
        let (cmd, key, rest) = (parts.next(), parts.next(), parts.next());
        let moved = match (cmd, key, rest) {
            (Some("bind"), Some(k), Some(r)) => moves
                .iter()
                .find(|(mk, from, _)| mk.eq_ignore_ascii_case(k) && r.trim().eq_ignore_ascii_case(from))
                .map(|(_, _, to)| to.map(|to| format!("bind {k} {to}"))),
            _ => None,
        };
        match moved {
            Some(None) => continue,
            Some(Some(line)) => out.push_str(&line),
            None => out.push_str(line),
        }
>>>>>>> /tmp/claude-0/zmhead/crates/console/src/user_settings.rs
        out.push('\n');
    }
    out
}

fn parse_settings(source: &str, settings: &mut frame::GameSettings, binds: &mut KeyBinds) {
    let mut bind_script = String::new();
    let mut layout = 1u32;
    for raw in source.lines() {
        let line = raw.trim();
        if line.is_empty() || line.starts_with("//") {
            continue;
        }
        if line.starts_with("bind ") || line == "unbindall" {
            bind_script.push_str(line);
            bind_script.push('\n');
            continue;
        }
        let Some((key, value)) = line.split_once('=') else {
            warn!("ignored malformed setting line: {line}");
            continue;
        };
        match key {
            "binds_layout" => layout = value.trim().parse().unwrap_or(1),
            "resolution" => {
                if let Some((w, h)) = value.split_once('x')
                    && let (Ok(width), Ok(height)) = (w.parse(), h.parse())
                {
                    settings.resolution = frame::DisplayResolution::new(width, height);
                }
            }
            "fullscreen" => {
                if let Ok(value) = value.parse() {
                    settings.fullscreen = value;
                }
            }
            "vsync" => {
                if let Ok(value) = value.parse() {
                    settings.vsync = value;
                }
            }
            "fov" => {
                if let Ok(v) = value.parse() {
                    settings.fov = v;
                }
            }
            "brightness" => {
                if let Ok(v) = value.parse() {
                    settings.brightness = v;
                }
            }
            "shadows" => {
                if let Ok(v) = value.parse() {
                    settings.shadows = v;
                }
            }
            "depth_of_field" => {
                if let Ok(v) = value.parse() {
                    settings.depth_of_field = v;
                }
            }
            "bloom" => {
                if let Ok(v) = value.parse() {
                    settings.bloom = v;
                }
            }
            "master_volume" => {
                if let Ok(value) = value.parse() {
                    settings.master_volume = value;
                }
            }
            "music_volume" => parse_into(value, &mut settings.music_volume),
            "match_music" => parse_into(value, &mut settings.match_music),
            "sensitivity" => {
                if let Ok(value) = value.parse() {
                    settings.sensitivity = value;
                }
            }
            "invert_mouse" => {
                if let Ok(value) = value.parse() {
                    settings.invert_mouse = value;
                }
            }
            "player_name" => settings.player_name = value.to_owned(),
            "music_volume" => parse_into(value, &mut settings.music_volume),
            "sfx_volume" => parse_into(value, &mut settings.sfx_volume),
            "voice_volume" => parse_into(value, &mut settings.voice_volume),
            "cinematic_volume" => parse_into(value, &mut settings.cinematic_volume),
            "pad_layout" => parse_into(value, &mut settings.pad_layout),
            "pad_stick_layout" => parse_into(value, &mut settings.pad_stick_layout),
            "pad_sensitivity" => parse_into(value, &mut settings.pad_sensitivity),
            "pad_ads_sensitivity" => parse_into(value, &mut settings.pad_ads_sensitivity),
            "pad_invert" => parse_into(value, &mut settings.pad_invert),
            "pad_curve" => parse_into(value, &mut settings.pad_curve),
            "pad_aim_assist" => parse_into(value, &mut settings.pad_aim_assist),
            "pad_prompts" => parse_into(value, &mut settings.pad_prompts),
            "pad_vibration" => parse_into(value, &mut settings.pad_vibration),
            "omni_movement" => parse_into(value, &mut settings.omni_movement),
<<<<<<< /home/claude/bo2-multiplayer-rebuild/crates/console/src/user_settings.rs
            "map_game_bodies" => parse_into(value, &mut settings.map_game_bodies),
            "friend_code" => parse_into(value, &mut settings.friend_code),
            "friends" => {
                settings.friends = value.split(',').filter_map(|c| c.trim().parse().ok()).collect();
            }
=======
>>>>>>> /tmp/claude-0/zmhead/crates/console/src/user_settings.rs
            "pad_deadzone_left" => parse_into(value, &mut settings.pad_deadzone_left),
            "pad_deadzone_right" => parse_into(value, &mut settings.pad_deadzone_right),
            // bo2zm: Black Ops II quality, `full` or `fast`.
            "bo2_quality" => {
                settings.bo2_fast = value.trim().eq_ignore_ascii_case("fast");
                asset_core::t6_set_fast(settings.bo2_fast);
            }
            "mature" => parse_into(value, &mut settings.mature),
            "corpses" => parse_into(value, &mut settings.corpses),
            "draw_fps" => parse_into(value, &mut settings.draw_fps),
            "max_fps" => parse_into(value, &mut settings.max_fps),
            "fxaa" => parse_into(value, &mut settings.fxaa),
            "aa_samples" => parse_into(value, &mut settings.aa_samples),
            // Texture filtering: the samplers made at load.
            "tex_filter" => {
                parse_into(value, &mut settings.tex_filter);
                asset_core::t6_set_aniso(match settings.tex_filter {
                    0 => 1,
                    1 => 4,
                    _ => 16,
                });
            }
            "tex_quality" => parse_into(value, &mut settings.tex_quality),
            _ => warn!("ignored unknown setting `{key}`"),
        }
    }
    if !bind_script.is_empty() && layout < BINDS_LAYOUT {
        bind_script = migrate_binds(&bind_script, layout);
<<<<<<< /home/claude/bo2-multiplayer-rebuild/crates/console/src/user_settings.rs
        info!("saved binds moved to Black Ops II's keys (layout {layout} -> {BINDS_LAYOUT})");
=======
        info!("saved binds moved to Black Ops II's keys (layout {layout} to {BINDS_LAYOUT})");
>>>>>>> /tmp/claude-0/zmhead/crates/console/src/user_settings.rs
    }
    if !bind_script.is_empty() {
        let warnings = binds.apply_config_script(&bind_script);
        for warning in warnings {
            warn!("settings bind: {warning}");
        }
    }
    if !binds.has_pad_binds() {
        binds.apply_pad_layout(usize::from(settings.pad_layout.min(4)));
    }
    if source
        .lines()
        .next()
        .is_some_and(|line| line.trim() == "// IW4L user settings v1")
        && binds.get(BindButton::Key(KeyCode::Digit4)).is_none()
        && !binds.iter().any(|(_, id)| id == 21)
    {
        binds.set(BindButton::Key(KeyCode::Digit4), 21);
    }
}

pub(crate) fn native_menu_settings(
    mut events: MessageReader<crate::ConsoleCommand>,
    mut settings: ResMut<frame::GameSettings>,
    mut dvars: ResMut<frame::UiMenuDvars>,
    mut shadows: ResMut<render_frontend::prepare::scene::view_parms::SmEnableDvar>,
    mut dof: ResMut<render_frontend::assemble::drawsurf::dof::DofDvars>,
    mut glow: ResMut<render_frontend::assemble::drawsurf::dof::GlowDvars>,
    mut test_rumble: MessageWriter<frame::TestControllerRumble>,
) {
    for command in events.read() {
        if !matches!(command.name.as_str(), "set" | "seta") {
            continue;
        }
        let [name, value, ..] = command.args.as_slice() else {
            continue;
        };
        match name.as_str() {
            "ui_r_mode" => {
                if let Some((w, h)) = value.split_once('x')
                    && let (Ok(w), Ok(h)) = (w.parse(), h.parse())
                {
                    settings.resolution = frame::DisplayResolution::new(w, h);
                }
            }
            "ui_r_displayMode" => settings.fullscreen = value == "1",
            "ui_r_vsync" => settings.vsync = value == "1",
            "ui_volume" => {
                if let Ok(v) = value.parse::<f32>()
                    && v.is_finite()
                {
                    settings.master_volume = v;
                }
            }
            "ui_player_name" => settings.player_name = value.clone(),
            "ui_fov" => {
                if let Ok(v) = value.parse::<f32>() {
                    settings.fov = v;
                }
            }
            "ui_brightness" => {
                if let Ok(v) = value.parse::<f32>()
                    && v.is_finite()
                {
                    settings.brightness = v;
                }
            }
            "ui_shadows" => settings.shadows = value == "1",
            "ui_dof" => settings.depth_of_field = value == "1",
            "ui_bloom" => settings.bloom = value == "1",
            "ui_pad_layout" => parse_into(value, &mut settings.pad_layout),
            "ui_pad_stick_layout" => parse_into(value, &mut settings.pad_stick_layout),
            "ui_pad_sensitivity" => parse_into(value, &mut settings.pad_sensitivity),
            "ui_pad_ads_sensitivity" => parse_into(value, &mut settings.pad_ads_sensitivity),
            "ui_pad_invert" => settings.pad_invert = value == "1",
            "ui_pad_curve" => parse_into(value, &mut settings.pad_curve),
            "ui_pad_aim_assist" => parse_into(value, &mut settings.pad_aim_assist),
            "ui_pad_prompts" => parse_into(value, &mut settings.pad_prompts),
            "ui_pad_test_rumble" => {
                if value == "1" {
                    test_rumble.write(frame::TestControllerRumble);
                }
            }
            "ui_pad_vibration" => settings.pad_vibration = value == "1",
            "ui_pad_deadzone_left" => parse_into(value, &mut settings.pad_deadzone_left),
            "ui_pad_deadzone_right" => parse_into(value, &mut settings.pad_deadzone_right),
            _ => continue,
        }
        settings.sanitize();
        settings.touch();
    }
    if settings.is_changed() {
        shadows.enabled = Some(settings.shadows);
        dof.enable = settings.depth_of_field;
        glow.enable = settings.bloom;
    }
    dvars.set("ui_volume", settings.master_volume.to_string());
    dvars.set("ui_brightness", settings.brightness.to_string());
    dvars.set("ui_fov", settings.fov.to_string());
    dvars.set("ui_player_name", settings.player_name.clone());
    dvars.set("ui_shadows", if settings.shadows { "1" } else { "0" });
    dvars.set("ui_dof", if settings.depth_of_field { "1" } else { "0" });
    dvars.set("ui_bloom", if settings.bloom { "1" } else { "0" });
    dvars.set("ui_r_mode", settings.resolution.to_string());
    dvars.set(
        "ui_r_displayMode",
        if settings.fullscreen { "1" } else { "0" },
    );
    dvars.set("ui_r_vsync", if settings.vsync { "1" } else { "0" });
    dvars.set("ui_pad_layout", settings.pad_layout.to_string());
    dvars.set("ui_pad_stick_layout", settings.pad_stick_layout.to_string());
    dvars.set("ui_pad_sensitivity", settings.pad_sensitivity.to_string());
    dvars.set(
        "ui_pad_ads_sensitivity",
        settings.pad_ads_sensitivity.to_string(),
    );
    dvars.set("ui_pad_invert", if settings.pad_invert { "1" } else { "0" });
    dvars.set("ui_pad_curve", settings.pad_curve.to_string());
    dvars.set("ui_pad_aim_assist", settings.pad_aim_assist.to_string());
    dvars.set("ui_pad_prompts", settings.pad_prompts.to_string());
    dvars.set(
        "ui_pad_vibration",
        if settings.pad_vibration { "1" } else { "0" },
    );
    dvars.set(
        "ui_pad_deadzone_left",
        settings.pad_deadzone_left.to_string(),
    );
    dvars.set(
        "ui_pad_deadzone_right",
        settings.pad_deadzone_right.to_string(),
    );
}

#[cfg(test)]
mod tests {
    use super::migrate_binds;

<<<<<<< /home/claude/bo2-multiplayer-rebuild/crates/console/src/user_settings.rs
    /// His saved binds (10-07, layout 2): the old E use / F frag keyboard
    /// keys go to Black Ops II's; his pad buttons stay.
    #[test]
    fn old_layout_gets_bo2_defaults() {
        let old = [
            "unbindall",
            "bind BUTTON_A +gostand",
            "bind DPAD_UP +actionslot 1",
=======
    fn has(script: &str, want: &str) -> bool {
        script.lines().any(|l| l == want)
    }

    /// His saved binds (10-04): G throws the monkey bomb, right mouse he
    /// rebound to ADS hold. Layout 1 moves through 2 to 3.
    #[test]
    fn old_layout_moves_to_bo2s() {
        let old = [
            "unbindall",
            "bind 1 weapnext",
            "bind 2 +left",
            "bind 3 +right",
            "bind 4 +actionslot 4",
>>>>>>> /tmp/claude-0/zmhead/crates/console/src/user_settings.rs
            "bind MOUSE2 +speed_throw",
            "bind e +activate",
            "bind f +frag",
            "bind g +smoke",
        ]
        .join("
");
        let new = migrate_binds(&old, 1);
        for want in [
            "bind g +frag",
            "bind f +activate",
            "bind 4 +smoke",
            "bind 3 +actionslot 4",
            "bind 2 +actionslot 2",
            "bind 5 +actionslot 3",
            "bind MOUSE2 +speed_throw",
            "bind 1 +actionslot 1",
        ] {
            assert!(has(&new, want), "missing {want:?} in {new}");
        }
        assert!(!new.contains("bind g +smoke") && !new.contains("bind e "), "{new}");
    }

    /// Layout 2's defaults move to Black Ops II's; a key he bound himself
    /// stays.
    #[test]
    fn layout_two_moves_to_bo2s_other_keys() {
        let old = [
            "unbindall",
            "bind 1 weapnext",
            "bind c +movedown",
            "bind CTRL +prone",
            "bind SHIFT +sprint",
            "bind MOUSE3 weapnext",
            "bind MWHEELUP weapprev",
            "bind MWHEELDOWN weapnext",
            "bind e +activate",
            "bind f +smoke",
            "bind g +frag",
            "bind 5 +actionslot 3",
        ]
        .join("
");
        let new = migrate_binds(&old, 2);
        for want in [
            "bind 1 +actionslot 1",
            "bind c +stance",
            "bind CTRL toggleprone",
            "bind SHIFT +breath_sprint",
            "bind MOUSE3 +frag",
            "bind MWHEELUP weapnext",
            "bind MWHEELDOWN weapprev",
            "bind f +smoke",
            "bind g +frag",
        ] {
            assert!(has(&new, want), "missing {want:?} in {new}");
        }
        assert!(!new.contains("bind e "), "{new}");
        assert_eq!(new.matches("bind 5 ").count(), 1);
    }

    /// The shipped defaults are Black Ops II's, and none of them is on an
    /// old layout's default, so moving them changes nothing.
    #[test]
    fn defaults_are_bo2s_keys() {
        let d = crate::DEFAULT_CONTROLS;
        for want in [
            "bind F +activate",
            "bind G +frag",
            "bind 4 +smoke",
            "bind C +stance",
            "bind CTRL toggleprone",
<<<<<<< /home/claude/bo2-multiplayer-rebuild/crates/console/src/user_settings.rs
            "bind SHIFT +breath_sprint",
            "bind MOUSE2 +speed_throw",
            "bind BUTTON_A +gostand",
            "bind DPAD_UP +actionslot 1",
        ] {
            assert!(new.lines().any(|l| l == want), "missing {want:?} in {new}");
        }
        for gone in ["bind e +activate", "bind f +frag", "bind c +movedown"] {
            assert!(!new.lines().any(|l| l == gone), "kept {gone:?} in {new}");
=======
            "bind MWHEELUP weapnext",
        ] {
            assert!(has(d, want), "missing {want:?}");
>>>>>>> /tmp/claude-0/zmhead/crates/console/src/user_settings.rs
        }
        assert_eq!(migrate_binds(d, 1), d);
    }

    /// His whole saved settings file (10-04, layout 1) through the load the
    /// game runs: G throws a grenade, 4 the monkey bomb, right mouse aims,
    /// F uses (his 10-10 "Keep BO2's F = use"); E is BO2's lean, unbound.
    #[test]
<<<<<<< /home/claude/bo2-multiplayer-rebuild/crates/console/src/user_settings.rs
    fn current_layout_is_kept() {
        let saved = "unbindall
bind e +activate";
        assert_eq!(migrate_binds(saved, super::BINDS_LAYOUT), saved);
=======
    fn his_saved_file_loads_with_bo2_keys() {
        use crate::{BindButton, KeyBinds};
        use bevy::input::keyboard::KeyCode;
        use bevy::input::mouse::MouseButton;
        let file = "// IW4L user settings v2\nresolution=1920x1080\nfullscreen=true\nunbindall\n\
                    bind 1 weapnext\nbind 2 +left\nbind 3 +right\nbind 4 +actionslot 4\n\
                    bind MOUSE1 +attack\nbind MOUSE2 +speed_throw\nbind e +activate\nbind f +frag\n\
                    bind g +smoke\nbind w +forward\n";
        let mut settings = frame::GameSettings::default();
        let mut binds = KeyBinds::default();
        super::parse_settings(file, &mut settings, &mut binds);
        let id = |c: &str| input_iw4::command_id_lookup(c);
        assert_eq!(binds.get(BindButton::Key(KeyCode::KeyG)), id("+frag"));
        assert_eq!(binds.get(BindButton::Key(KeyCode::KeyF)), id("+activate"));
        assert_eq!(binds.get(BindButton::Key(KeyCode::Digit4)), id("+smoke"));
        assert_eq!(binds.get(BindButton::Key(KeyCode::Digit3)), id("+actionslot 4"));
        assert_eq!(binds.get(BindButton::Key(KeyCode::Digit5)), id("+actionslot 3"));
        assert_eq!(binds.get(BindButton::Key(KeyCode::Digit1)), id("+actionslot 1"));
        assert_eq!(binds.get(BindButton::Mouse(MouseButton::Right)), id("+speed_throw"));
        assert_eq!(binds.get(BindButton::Key(KeyCode::KeyE)), None);
>>>>>>> /tmp/claude-0/zmhead/crates/console/src/user_settings.rs
    }
}
