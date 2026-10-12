use std::collections::{BTreeSet, HashSet};

use asset_audio::SoundCatalog;
use asset_core::AssetNamespace;
use asset_game::WeaponRegistry;
use assets::{MapLoadProcess, MatchType10SoundHints, PreparedWeapons};
use bevy::prelude::*;
use frame::{ClientSet, LaunchIdentity, MatchTornDown, ReturnedToMenu};

use crate::aliases::movement_prepare_names;
use crate::ambient::{SoundBankCompose, SoundBankLoadAttempted, SoundBankNamespace};
use crate::clip_store::{ClipKey, ClipStore, clip_keys_for_alias};
use crate::playback::SoundBank;

#[derive(Resource, Default)]
pub struct AudioReady(pub bool);

#[derive(Resource)]
pub struct AudioSilent;

impl AudioSilent {
    pub fn active() -> bool {
        static SILENT: std::sync::OnceLock<bool> = std::sync::OnceLock::new();
        *SILENT.get_or_init(|| {
            frame::Headless::requested()
                || std::env::var("IW4L_SOUND").is_ok_and(|value| matches!(value.as_str(), "off" | "0"))
        })
    }

    /// IW4L_HEADLESS=1: no sound device was opened, so no voice ever gets a
    /// sink. Its starts, loops and stops still run and reach the log.
    pub fn no_device() -> bool {
        frame::Headless::requested()
    }

    /// Silent runs that log sound (IW4L_SOUND_LOG) still compose the map's
    /// sound bank and prepare its clips, so the real alias lookups, starts,
    /// loops and stops happen and reach the log; the PCM decoder hands the
    /// device zeros, so nothing is ever audible. Without the bank every
    /// lookup fell back to the iw4 namespace and no BO2 alias resolved.
    pub fn composes() -> bool {
        Self::active() && std::env::var_os("IW4L_SOUND_LOG").is_some()
    }
}

const MATCH_HUD_PULSE: &[&str] = &["ui_pulse_text_type", "ui_pulse_text_delete"];

const MENU_CODE: [&str; 2] = ["mouse_over", "mouse_click"];

const MATCH_CLOCK: &[&str] = &["ui_mp_timer_countdown"];

/// bo2mp: Black Ops II's hit sounds (`_damagefeedback.gsc`'s hit alerts and
/// the engine's headshot crack), ready before the first hit: a late one
/// left his first hits of a match silent.
const T6_HIT_SOUNDS: &[&str] = &[
    "mpl_hit_alert",
    "mpl_hit_alert_low",
    "prj_bullet_impact_headshot",
    "prj_bullet_impact_headshot_2d",
];

#[derive(Default)]
struct MatchRequests {
    required: HashSet<ClipKey>,
    missing: BTreeSet<String>,
    resolved_aliases: usize,
}

#[derive(Resource, Default)]
struct MatchClipPrep {
    submitted: bool,
    required: HashSet<ClipKey>,
    /// Clips this match still had to convert when the set was queued — not the
    /// alias count, and not the whole cache.
    total: usize,
    stage: Option<asset_transport::StageHandle>,
    /// Resident sample bytes this process had already produced when the set
    /// was queued. The decoders count for the life of the process, so what
    /// this match cost is the distance from here.
    sample_bytes_at_queue: u64,
}

/// Resident `f32` samples every decoder has produced so far.
fn prepared_sample_bytes() -> u64 {
    crate::clip_prep_cost()
        .paths
        .iter()
        .map(|(_, cost)| cost.sample_bytes)
        .sum()
}

pub(crate) fn register(app: &mut App) {
    if AudioSilent::no_device() {
        diag::info!(Audio, "audio: Silent (IW4L_HEADLESS=1, no sound device)");
        app.insert_resource(AudioSilent);
    } else if AudioSilent::active() {
        diag::info!(Audio, "audio: Silent (IW4L_SOUND=off)");
        app.insert_resource(AudioSilent);
    }
    app.init_resource::<AudioReady>()
        .init_resource::<MatchClipPrep>()
        .init_resource::<crate::match_voices::AnnouncerRoutes>()
        .add_systems(
            Update,
            (
                queue_match_clips.after(crate::ambient::install_sound_bank),
                poll_match_audio_ready.after(queue_match_clips),
                reset_match_audio_on_match_end,
            )
                .in_set(ClientSet::Load),
        );
}

fn reset_match_audio_on_match_end(
    mut torn: MessageReader<MatchTornDown>,
    mut returned: MessageReader<ReturnedToMenu>,
    mut ready: ResMut<AudioReady>,
    mut prep: ResMut<MatchClipPrep>,
    mut announcer: ResMut<crate::match_voices::AnnouncerRoutes>,
) {
    if torn.read().count() == 0 && returned.read().count() == 0 {
        return;
    }
    *ready = AudioReady(false);
    if let Some(stage) = prep.stage.take() {
        stage.cancel();
    }
    *prep = MatchClipPrep::default();
    *announcer = Default::default();
}

fn queue_match_clips(
    attempted: Res<SoundBankLoadAttempted>,
    walk: Option<Res<SoundBankCompose>>,
    mut clips: Option<ResMut<ClipStore>>,
    weapons: Option<Res<PreparedWeapons>>,
    type10: Option<Res<MatchType10SoundHints>>,
    bank: Option<Res<SoundBank>>,
    identity: Option<Res<LaunchIdentity>>,
    catalog: Option<Res<asset_game::MenuCatalog>>,
    script_sound: Option<Res<asset_audio::SessionMapScriptSound>>,
    teams: Option<Res<asset_game::SessionTeamSettings>>,
    namespace: Option<Res<SoundBankNamespace>>,
    loading: Option<Res<MapLoadProcess>>,
    mut prep: ResMut<MatchClipPrep>,
    mut announcer: ResMut<crate::match_voices::AnnouncerRoutes>,
    mut ready: ResMut<AudioReady>,
    silent: Option<Res<AudioSilent>>,
) {
    if ready.0 || prep.submitted {
        return;
    }
    if silent.is_some() && !AudioSilent::composes() {
        ready.0 = true;
        if let Some(loading) = loading.as_ref() {
            loading
                .progress
                .record_skipped(asset_transport::StageId::Audio);
        }
        diag::info!(Audio, "audio: AudioReady — Silent, no clips prepared");
        return;
    }
    if !attempted.0 {
        return;
    }
    if walk.is_some() {
        return;
    }
    let Some(clips) = clips.as_mut() else {
        ready.0 = true;
        if let Some(loading) = loading.as_ref() {
            loading
                .progress
                .record_skipped(asset_transport::StageId::Audio);
        }
        diag::info!(
            Audio,
            "audio: AudioReady skipped — sound bank did not install"
        );
        return;
    };
    let Some(weapons) = weapons else {
        return;
    };
    let Some(type10) = type10 else {
        return;
    };
    let Some(bank) = bank else {
        return;
    };
    let Some(namespace) = namespace else {
        return;
    };
    let Some(script_sound) = script_sound else {
        return;
    };
    let mut set = MatchRequests::default();
    let mut aliases = 0usize;
    for weapon in 1..=weapons.0.len() as u32 {
        aliases += request_weapon_aliases(clips, &bank.0, &weapons.0, weapon, &mut set);
    }
    for alias in movement_prepare_names() {
        aliases += 1;
        request_named(clips, &bank.0, AssetNamespace::Iw4, alias, &mut set);
    }
    if let Some(identity) = identity.as_deref() {
        let ns = namespace.namespace;
        let zone = namespace.zone.as_str();
        if zone != identity.zone {
            return;
        }
        for emitter in bank.0.createfx_loop_sounds(ns, zone) {
            aliases += 1;
            request_named(clips, &bank.0, ns, &emitter.soundalias, &mut set);
        }
    }
    if let Some(identity) = identity.as_deref() {
        let (allies, axis) = crate::match_voices::voice_prefixes_for_zone(
            catalog.as_deref(),
            Some(bank.as_ref()),
            identity,
        );
        let voices =
            crate::match_voices::team_voice_aliases(&bank.0, allies.as_deref(), axis.as_deref());
        let native = teams.as_deref().map(|teams| &teams.0);
        *announcer = crate::match_voices::AnnouncerRoutes::build(
            &bank.0,
            namespace.namespace,
            [allies.as_deref(), axis.as_deref()],
            native,
            &voices,
        );
        diag::info!(
            Audio,
            "audio: announcer: {} of {} team voice aliases play as {:?}",
            announcer.len(),
            voices.len(),
            namespace.namespace
        );
        for alias in &voices {
            aliases += 1;
            let (ns, alias) = announcer.route(alias);
            request_named(clips, &bank.0, ns, alias, &mut set);
        }
    }
    if let Some(alias) = script_sound.0.ambient_alias.as_deref() {
        aliases += 1;
        request_named(clips, &bank.0, namespace.namespace, alias, &mut set);
    }
    if let Some(catalog) = catalog.as_deref() {
        for alias in catalog.played_sound_aliases().into_iter().chain(MENU_CODE) {
            aliases += 1;
            request_named(clips, &bank.0, AssetNamespace::Iw4, alias, &mut set);
        }
    }
    for alias in MATCH_HUD_PULSE
        .iter()
        .chain([&"motiontracker_ping", &"motiontracker_pong"])
        .chain(crate::weapon_lock::ALIASES.iter())
        .chain([&gamemode_iw4::damage_feedback::HIT_ALERT_ALIAS])
        .chain(MATCH_CLOCK)
    {
        aliases += 1;
        request_named(clips, &bank.0, AssetNamespace::Iw4, alias, &mut set);
    }
    // bo2mp: a map from another game (MW2, MW3, Black Ops, CoD4) played by
    // Black Ops II's rules runs BO2's scripts, which play BO2's sounds
    // (`t6:` aliases): its hit sounds, interface and announcer are set up as
    // on a BO2 map. Keyed off the map's sound namespace alone, those
    // matches skipped this whole block and the BO1 announcer routes were
    // never built (the voice went silent on every foreign map).
    let t6_match = namespace.namespace == AssetNamespace::T6 || assets::t6_rules();
    if t6_match {
        for alias in T6_HIT_SOUNDS {
            aliases += 1;
            request_named(clips, &bank.0, AssetNamespace::T6, alias, &mut set);
        }
        // bo2mp: the team announcer (`vox_<faction>_<line>`: match ending
        // soon, lead taken/lost, ...), ready before the script plays it.
        let mut announcer_lines = crate::match_voices::t6_announcer_aliases(&bank.0);
        let voice = crate::match_voices::T6Announcer::from_env();
        if voice == crate::match_voices::T6Announcer::Bo1 {
            let candidates = crate::match_voices::t6_announcer_candidates(&bank.0);
            let routes = crate::match_voices::t6_bo1_routes(&bank.0, &candidates);
            diag::info!(
                Audio,
                "audio: announcer: Black Ops 1 voice for {} of {} Black Ops II lines (the rest keep Black Ops II's voice)",
                routes.len(),
                announcer_lines.len()
            );
            diag::info!(
                Audio,
                "audio: announcer: BO1 routing on `{}` (sound namespace {:?}, t6 rules {}): {} BO2 lines in the bank, {} candidates, {} routed (t5 {}, iw5 {}); e.g. vox_pmc_tdm_start -> {:?}",
                namespace.zone,
                namespace.namespace,
                assets::t6_rules(),
                announcer_lines.len(),
                candidates.len(),
                routes.len(),
                routes.values().filter(|(ns, _)| *ns == AssetNamespace::T5).count(),
                routes.values().filter(|(ns, _)| *ns == AssetNamespace::Iw5).count(),
                routes.get("vox_pmc_tdm_start"),
            );
            // The lines the scripts can play: the bank's own BO2 lines and
            // every BO2 alias a BO1 line now stands in for.
            let known: std::collections::HashSet<String> =
                announcer_lines.iter().map(|a| a.to_ascii_lowercase()).collect();
            let mut extra: Vec<String> =
                routes.keys().filter(|k| !known.contains(*k)).cloned().collect();
            extra.sort_unstable();
            announcer_lines.extend(extra);
            if routes.is_empty() {
                diag::warn!(
                    Audio,
                    "audio: announcer: no Black Ops 1 lines in the sound bank — is BO1 under IW4L_GAMES (its zone/English/en_{}.ff)?",
                    asset_audio::BO1_ANNOUNCER_ZONE
                );
            }
            announcer.extend(routes);
        }
        diag::info!(
            Audio,
            "audio: announcer: {} Black Ops II team voice lines queued ({voice:?} voice)",
            announcer_lines.len()
        );
        // The interface sounds standing in for BO2's (Black Ops' menu
        // sounds, the pre-match countdown).
        for (from, _) in crate::ui_sounds::UI_ROUTES {
            if let Some((ns, to)) = crate::ui_sounds::ui_sound(&bank.0, from) {
                aliases += 1;
                request_named(clips, &bank.0, ns, to, &mut set);
            }
        }
        for alias in &announcer_lines {
            aliases += 1;
            let lower = alias.to_ascii_lowercase();
            let (ns, alias) = announcer
                .routed(AssetNamespace::T6, &lower)
                .unwrap_or((AssetNamespace::T6, alias.as_str()));
            request_named(clips, &bank.0, ns, alias, &mut set);
        }
    }
    for alias in &type10.0 {
        aliases += 1;
        request_fx_type10(clips, &bank.0, alias, &mut set);
    }
    if !set.missing.is_empty() {
        let names: Vec<&str> = set.missing.iter().map(String::as_str).collect();
        diag::warn!(
            Audio,
            "audio: match-set gap: {} aliases name no loaded sound: {}",
            names.len(),
            names.join(" ")
        );
    }
    prep.total = set.required.len();
    prep.required = set.required;
    prep.submitted = true;
    if let Some(loading) = loading {
        prep.sample_bytes_at_queue = prepared_sample_bytes();
        if prep.total == 0 && set.resolved_aliases > 0 {
            loading
                .progress
                .record_reused_scoped(asset_transport::StageId::Audio, "clips");
        } else {
            prep.stage = Some(
                loading
                    .progress
                    .begin(asset_transport::StageId::Audio, Some(prep.total as u64)),
            );
        }
    }
    diag::info!(
        Audio,
        "audio: match-set queued {} aliases ({} type-10), {} clips still converting ({} workers); resident reused={} clips/{}B",
        aliases,
        type10.0.len(),
        prep.total,
        clips.workers(),
        clips.reused_resident().0,
        clips.reused_resident().1,
    );
    if prep.total == 0 {
        mark_ready(&mut ready, &mut prep, Some(&mut **clips));
    }
}

fn poll_match_audio_ready(
    mut clips: Option<ResMut<ClipStore>>,
    mut prep: ResMut<MatchClipPrep>,
    mut ready: ResMut<AudioReady>,
) {
    if ready.0 || !prep.submitted {
        return;
    }
    let Some(clips) = clips.as_mut() else {
        return;
    };
    prep.required.retain(|key| clips.ready(key).is_none());
    let done = prep.total.saturating_sub(prep.required.len());
    let decoded = prepared_sample_bytes().saturating_sub(prep.sample_bytes_at_queue);
    if let Some(stage) = prep.stage.as_ref() {
        stage.set_completed(done as u64);
        stage.set_bytes(decoded);
    }
    if prep.required.is_empty() {
        mark_ready(&mut ready, &mut prep, Some(&mut **clips));
    }
}

fn mark_ready(ready: &mut AudioReady, prep: &mut MatchClipPrep, clips: Option<&mut ClipStore>) {
    if let Some(stage) = prep.stage.take() {
        stage.set_completed(prep.total as u64);
        stage.set_bytes(prepared_sample_bytes().saturating_sub(prep.sample_bytes_at_queue));
        stage.done();
    }
    if !ready.0 {
        ready.0 = true;
        diag::info!(Audio, "audio: AudioReady ({} clips prepared)", prep.total);
        if let Some(clips) = clips {
            clips.arm_match_live();
        }
    }
}

fn request_named(
    clips: &mut ClipStore,
    bank: &SoundCatalog,
    ns: AssetNamespace,
    alias: &str,
    set: &mut MatchRequests,
) {
    let keys = clip_keys_for_alias(bank, ns, alias);
    if keys.is_empty() {
        set.missing.insert(alias.to_owned());
    } else {
        set.resolved_aliases += 1;
    }
    for key in keys {
        clips.request(key.clone());
        if clips.ready(&key).is_none() {
            set.required.insert(key);
        }
    }
}

fn request_fx_type10(
    clips: &mut ClipStore,
    bank: &SoundCatalog,
    alias: &str,
    set: &mut MatchRequests,
) {
    let ns = bank
        .index_in(AssetNamespace::Iw4, alias)
        .or_else(|| bank.index_unique(alias))
        .map(|index| bank.namespace_of_alias(index))
        .unwrap_or(AssetNamespace::Iw4);
    request_named(clips, bank, ns, alias, set);
}

fn request_weapon_aliases(
    clips: &mut ClipStore,
    bank: &SoundCatalog,
    registry: &WeaponRegistry,
    weapon: u32,
    set: &mut MatchRequests,
) -> usize {
    let sounds = registry.sounds_of(weapon);
    let ns = registry.namespace_of(weapon).unwrap_or(AssetNamespace::Iw4);
    let mut n = 0usize;
    if let Some(aliases) = sounds {
        for alias in aliases.reachable_aliases() {
            n += 1;
            request_named(clips, bank, ns, alias, set);
        }
    }
    let mut seen = HashSet::new();
    for alias in registry.notetrack_sound_aliases_of(weapon) {
        if seen.insert(alias) {
            n += 1;
            request_named(clips, bank, ns, alias, set);
        }
    }
    n
}
