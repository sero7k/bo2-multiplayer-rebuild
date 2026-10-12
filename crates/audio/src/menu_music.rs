//! The multiplayer front end's music: one main-menu track at a time, picked
//! at random from Black Ops II's, Black Ops', Modern Warfare 2's and Modern
//! Warfare 3's own, and a different one when it ends. It stops the moment a
//! match starts loading and comes back only once the game is back at the
//! menu; a game whose track is not installed is left out of the rotation.

use std::time::{Duration, Instant, SystemTime, UNIX_EPOCH};

use asset_core::AssetNamespace;
use bevy::audio::Volume;
use bevy::prelude::*;

use crate::backend::AudioScope;
use crate::clip_store::{ClipStore, clip_keys_for_alias};
use crate::frontend::FrontendAudio;
use crate::pcm::PcmAudio;
use crate::playback::SoundBank;
use frame::ReturnedToMenu;

/// Each game's main-menu track: its namespace, alias and name for the log.
const TRACKS: [(AssetNamespace, &str, &str); 4] = [
    (AssetNamespace::T6, "mus_mp_frontend", "Black Ops II"),
    (AssetNamespace::T5, "mus_mp_frontend", "Black Ops"),
    (AssetNamespace::Iw4, "music_mainmenu_mp", "Modern Warfare 2"),
    (AssetNamespace::Iw5, "music_mainmenu_mp", "Modern Warfare 3"),
];

const VOLUME: f32 = 0.55;

#[derive(Resource)]
pub(crate) struct MenuMusic {
    playing: Option<Playing>,
    /// The track waiting on its clip.
    next: Option<usize>,
    last: Option<usize>,
    unavailable: [bool; TRACKS.len()],
    rng: u64,
    /// A match load was asked for; silent until the game is back at the menu.
    loading: bool,
}

struct Playing {
    entity: Entity,
    track: usize,
    ends: Instant,
}

impl Default for MenuMusic {
    fn default() -> Self {
        let seed = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .map_or(0x9e37_79b9_7f4a_7c15, |d| d.as_nanos() as u64);
        Self {
            playing: None,
            next: None,
            last: None,
            unavailable: [false; TRACKS.len()],
            rng: seed | 1,
            loading: false,
        }
    }
}

impl MenuMusic {
    fn random(&mut self) -> u64 {
        self.rng ^= self.rng << 13;
        self.rng ^= self.rng >> 7;
        self.rng ^= self.rng << 17;
        self.rng
    }

    /// A track still in the rotation, not the one that just played when
    /// another is left.
    fn pick(&mut self) -> Option<usize> {
        let open: Vec<usize> = (0..TRACKS.len()).filter(|&i| !self.unavailable[i]).collect();
        let fresh: Vec<usize> = open.iter().copied().filter(|&i| Some(i) != self.last).collect();
        let from = if fresh.is_empty() { open } else { fresh };
        if from.is_empty() {
            return None;
        }
        let at = (self.random() % from.len() as u64) as usize;
        Some(from[at])
    }

    fn stop(&mut self, commands: &mut Commands) {
        if let Some(playing) = self.playing.take() {
            crate::backend::stop(commands, playing.entity);
        }
        self.next = None;
    }
}

pub(crate) fn update(
    mut commands: Commands,
    frontend: Option<Res<FrontendAudio>>,
    bank: Option<Res<SoundBank>>,
    mut clips: Option<ResMut<ClipStore>>,
    mut assets: ResMut<Assets<PcmAudio>>,
    mut music: ResMut<MenuMusic>,
    request: Option<Res<assets::MatchLoadRequest>>,
    accepted: Option<Res<assets::MatchLoadAccepted>>,
    mut returned: MessageReader<ReturnedToMenu>,
) {
    if returned.read().count() > 0 {
        music.loading = false;
    }
    if request.is_some() || accepted.is_some() {
        music.loading = true;
    }
    let in_menu = !music.loading
        && match (&frontend, &bank) {
            (Some(frontend), Some(bank)) => std::sync::Arc::ptr_eq(&frontend.bank, &bank.0),
            _ => false,
        };
    if !in_menu {
        music.stop(&mut commands);
        return;
    }
    if let Some(playing) = &music.playing {
        if Instant::now() < playing.ends {
            return;
        }
        let track = playing.track;
        music.stop(&mut commands);
        music.last = Some(track);
    }
    let (Some(bank), Some(clips)) = (bank, clips.as_mut()) else {
        return;
    };
    loop {
        let track = match music.next {
            Some(track) => track,
            None => match music.pick() {
                Some(track) => track,
                None => return,
            },
        };
        let (ns, alias, game) = TRACKS[track];
        let Some(key) = clip_keys_for_alias(&bank.0, ns, alias).into_iter().next() else {
            music.unavailable[track] = true;
            music.next = None;
            diag::info!(Audio, "audio: menu music: {game}'s track `{alias}` is not installed; left out");
            continue;
        };
        clips.request(key.clone());
        let Some(ready) = clips.ready(&key) else {
            music.next = Some(track);
            return;
        };
        music.next = None;
        let pcm = match ready {
            Ok(pcm) => pcm,
            Err(_) => {
                music.unavailable[track] = true;
                diag::warn!(Audio, "audio: menu music: {game}'s track `{alias}` did not decode; left out");
                continue;
            }
        };
        let frames = pcm.samples().len() / usize::from(pcm.channel_count().max(1));
        let length = Duration::from_secs_f64(frames as f64 / f64::from(pcm.rate().max(1)));
        let handle = assets.add(pcm);
        let entity = crate::backend::spawn_oneshot(
            &mut commands,
            handle,
            Volume::Linear(VOLUME),
            1.0,
            0,
            AudioScope::Menu,
        );
        music.playing = Some(Playing {
            entity,
            track,
            ends: Instant::now() + length,
        });
        diag::info!(
            Audio,
            "audio: menu music: {game} ({:.0} s)",
            length.as_secs_f64()
        );
        return;
    }
}
