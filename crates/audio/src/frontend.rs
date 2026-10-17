use std::sync::Arc;

use asset_audio::{SoundCatalog, load_mp_sound_bank};
use asset_transport::GamesRoot;
use assets::{NamespaceSoundIwd, NamespaceTrees};
use bevy::{
    prelude::*,
    tasks::{AsyncComputeTaskPool, Task, TaskPool, futures_lite::future},
};
use frame::{
    ClientSet, LaunchIdentity, ReturnedToMenu, UiPlaySound,
    register_ui_contracts,
};

use crate::{
    ClipStore, SoundClass,
    ambient::SoundIwd,
    clip_store::PendingStarts,
    pcm::PcmAudio,
    playback::{MissingAliasGaps, SharedPlayAssets, SoundBank, SoundPickState, play_alias_oneshot},
    start::StartDecisions,
};

#[derive(Resource, Clone)]
pub struct FrontendAudio {
    pub bank: Arc<SoundCatalog>,
    pub iwd: Arc<NamespaceSoundIwd>,
}

const FRONTEND_SOUND_ZONE: &str = "iw4:code_post_gfx_mp";

#[derive(Resource)]
struct FrontendAudioPrepare(Task<FrontendAudioWalked>);

/// bo2zm: the frontend bank did not build once; do not walk it again every
/// frame (a Black Ops II-only install has no MW2 frontend zone).
#[derive(Resource)]
struct FrontendAudioGap;

struct FrontendAudioWalked {
    bank: Option<Arc<SoundCatalog>>,
    iwd: Arc<NamespaceSoundIwd>,
    lines: Vec<String>,
}

pub(crate) fn register_frontend_audio(app: &mut App) {
    register_ui_contracts(app);
    // Music (the menus' own beds included) is the music director's
    // (`crate::music`).
    app.add_message::<ReturnedToMenu>()
        .add_systems(
            Update,
            (
                start_frontend_audio_prepare,
                install_frontend_audio_prepare.after(start_frontend_audio_prepare),
                restore_frontend_audio_on_menu
                    .after(install_frontend_audio_prepare)
                    .after(crate::ambient::stop_map_ambient_on_match_end),
            )
                .in_set(ClientSet::Load),
        )
        .add_systems(
            Update,
            (
                play_ui_sound_messages.after(crate::voice::reclaim_finished_voices),
            )
                .in_set(ClientSet::Effects),
        );
}

fn start_frontend_audio_prepare(
    frontend: Option<Res<FrontendAudio>>,
    running: Option<Res<FrontendAudioPrepare>>,
    identity: Option<Res<LaunchIdentity>>,
    silent: Option<Res<crate::AudioSilent>>,
    gap: Option<Res<FrontendAudioGap>>,
    loading: (
        Option<Res<assets::MatchLoadBusy>>,
        Option<Res<assets::MatchLoadRequest>>,
        Option<Res<assets::MatchLoadAccepted>>,
        Option<Res<assets::PreparedMatchReady>>,
        Option<Res<crate::ambient::SoundBankCompose>>,
    ),
    mut commands: Commands,
) {
    if frontend.is_some() || running.is_some() || silent.is_some() || gap.is_some() {
        return;
    }
    let (busy, request, accepted, ready, walk) = loading;
    if busy.is_some_and(|busy| busy.0)
        || request.is_some()
        || accepted.is_some()
        || ready.is_some()
        || walk.is_some()
    {
        return;
    }
    let Some(identity) = identity else {
        return;
    };
    if identity.games_root.as_os_str().is_empty() {
        return;
    }
    let games = GamesRoot(identity.games_root.clone());
    let pool = AsyncComputeTaskPool::get_or_init(TaskPool::default);
    let task = pool.spawn(async move {
        let mut lines = Vec::new();
        let bank = match load_mp_sound_bank(&games, FRONTEND_SOUND_ZONE) {
            Ok(loaded) => {
                lines.extend(loaded.gap_lines());
                Some(Arc::new(loaded.catalog))
            }
            Err(error) => {
                lines.push(format!("frontend sound bank: {error}"));
                None
            }
        };
        let (indices, open_lines) = NamespaceSoundIwd::open(&NamespaceTrees::discover(&games));
        lines.extend(open_lines);
        FrontendAudioWalked {
            bank,
            iwd: Arc::new(indices),
            lines,
        }
    });
    commands.insert_resource(FrontendAudioPrepare(task));
}

fn install_frontend_audio_prepare(
    mut prepare: Option<ResMut<FrontendAudioPrepare>>,
    mut commands: Commands,
) {
    let Some(prepare) = prepare.as_deref_mut() else {
        return;
    };
    let Some(walked) = future::block_on(future::poll_once(&mut prepare.0)) else {
        return;
    };
    commands.remove_resource::<FrontendAudioPrepare>();
    for line in walked.lines {
        diag::info!(Audio, "audio: frontend {line}");
    }
    let Some(bank) = walked.bank else {
        diag::warn!(
            Audio,
            "audio: frontend sound bank did not build — the menu stays silent (typed gap)"
        );
        commands.insert_resource(FrontendAudioGap);
        return;
    };
    commands.insert_resource(FrontendAudio {
        bank,
        iwd: walked.iwd,
    });
    diag::info!(Audio, "audio: frontend sound resident");
}

pub(crate) fn restore_frontend_audio_on_menu(
    mut returned: MessageReader<ReturnedToMenu>,
    frontend: Option<Res<FrontendAudio>>,
    live: Option<Res<SoundBank>>,
    silent: Option<Res<crate::AudioSilent>>,
    mut commands: Commands,
) {
    if returned.read().count() == 0 || silent.is_some() {
        return;
    }
    let Some(frontend) = frontend else {
        diag::warn!(
            Audio,
            "audio: no resident frontend bank — the menu runs without sound (typed gap)"
        );
        return;
    };
    if live.is_some_and(|live| Arc::ptr_eq(&live.0, &frontend.bank)) {
        return;
    }
    commands.insert_resource(PendingStarts::default());
    commands.insert_resource(SharedPlayAssets::default());
    commands.insert_resource(SoundBank(Arc::clone(&frontend.bank)));
    commands.insert_resource(SoundIwd(Arc::clone(&frontend.iwd)));
    commands.insert_resource(ClipStore::start(
        Arc::clone(&frontend.bank),
        Some(Arc::clone(&frontend.iwd)),
    ));
    diag::info!(Audio, "audio: frontend sound restored");
}

/// The namespace a menu's alias lives in: the bank's own game (bo2mp:
/// Black Ops II's front end and match banks are T6; looked up as IW4 every
/// menu sound was missing), IW4 when no game has it.
fn alias_namespace(bank: &SoundCatalog, alias: &str) -> asset_core::AssetNamespace {
    use asset_core::AssetNamespace as Ns;
    [Ns::Iw4, Ns::T6, Ns::T5, Ns::Iw5]
        .into_iter()
        .find(|ns| bank.index_in(*ns, alias).is_some())
        .unwrap_or(Ns::Iw4)
}

fn play_ui_sound_messages(
    mut events: MessageReader<UiPlaySound>,
    mut commands: Commands,
    mut pcm_assets: ResMut<Assets<PcmAudio>>,
    mut shared: ResMut<SharedPlayAssets>,
    mut pick: ResMut<SoundPickState>,
    mut gaps: ResMut<MissingAliasGaps>,
    mut clips: Option<ResMut<ClipStore>>,
    mut pending: ResMut<PendingStarts>,
    mut occupancy: ResMut<crate::VoiceOccupancy>,
    mut decisions: ResMut<StartDecisions>,
    bank: Option<Res<SoundBank>>,
    iwd: Option<Res<SoundIwd>>,
    epoch: Res<crate::backend::MatchEpoch>,
    settings: Option<Res<frame::GameSettings>>,
) {
    // bo2mp: the lobby's countdown ticks with Ghosts' sound, as the match's
    // does (`playback`), when Ghosts is installed.
    let ghosts_tick = crate::ghosts_countdown::clip();
    let volume = settings.as_deref().map_or(1.0, |s| s.master_volume) * crate::bus_gain::sfx();
    let mut tick = |commands: &mut Commands, pcm_assets: &mut Assets<PcmAudio>, alias: &str| -> bool {
        let Some(clip) = ghosts_tick.clone().filter(|_| crate::ghosts_countdown::is_countdown(alias)) else { return false };
        let handle = pcm_assets.add(clip);
        crate::backend::spawn_oneshot(commands, handle, bevy::audio::Volume::Linear(volume.max(0.0)), 1.0, epoch.0, crate::backend::AudioScope::Menu);
        true
    };
    let Some(bank) = bank else {
        for event in events.read().filter(|_| !crate::AudioSilent::active()) {
            if tick(&mut commands, &mut pcm_assets, &event.alias) {
                continue;
            }
            diag::warn!(
                Audio,
                "audio: UiPlaySound `{}` dropped — no SoundBank (typed gap)",
                event.alias
            );
        }
        return;
    };
    let iwd = iwd.as_ref().map(|s| &s.0);
    for event in events.read() {
        if event.alias.is_empty() || tick(&mut commands, &mut pcm_assets, &event.alias) {
            continue;
        }
        let (namespace, alias) = match crate::ui_sounds::ui_sound(&bank.0, &event.alias) {
            Some((ns, to)) => (ns, to),
            None => (alias_namespace(&bank.0, &event.alias), event.alias.as_str()),
        };
        let outcome = play_alias_oneshot(
            &mut commands,
            &mut pcm_assets,
            &mut shared,
            &bank.0,
            iwd.map(|a| a.as_ref()),
            namespace,
            alias,
            None,
            None,
            &mut pick,
            clips.as_deref_mut(),
            &mut pending,
            &mut occupancy,
            &mut decisions,
            Some(crate::SND_ENT_LOCAL),
            SoundClass::Ui,
            epoch.0,
        );
        if outcome.allows_binding_fallback() {
            gaps.record(&event.alias);
        }
    }
}
