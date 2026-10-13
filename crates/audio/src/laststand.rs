//! bo2zm: the sound of going down. Black Ops II's client script
//! (`clientscripts/mp/zombies/_zm.csc`, `last_stand_monitor` and
//! `last_stand_thread`) follows the server's "lsm" state. On "1" a
//! heartbeat starts (`chr_heart_beat`: the first at half volume, then each
//! one 5% later and 5% louder, from 0.5 s apart up to 2 s, up to full) and,
//! solo, a low loop fades in over 0.3 s (`evt_laststand_loop`). On "0" the
//! loop fades out over 0.7 s, `evt_laststand_in` plays, and 0.7 s later the
//! revive gasp (`revive_gasp`). The server sends the state as the client
//! dvar `bo2zm_lsm`.

use asset_core::AssetNamespace;
use bevy::audio::Volume;
use bevy::prelude::*;
use net::{LocalPresentClient, PresentedSnapshot};

use crate::backend::{AudioScope, MatchEpoch};
use crate::clip_store::{ClipStore, clip_keys_for_alias};
use crate::pcm::{LoopingPcmAudio, PcmAudio};
use crate::playback::{MissingAliasGaps, SoundBank};
use crate::{AliasCommand, PlayAlias};

const BEAT: &str = "chr_heart_beat";
const LOOP: &str = "evt_laststand_loop";
const LOOP_IN_SECS: f32 = 0.3;
const LOOP_OUT_SECS: f32 = 0.7;
const GASP_AFTER_SECS: f32 = 0.7;

#[derive(Default)]
pub(crate) struct LastStand {
    client: Option<u32>,
    epoch: u64,
    down: bool,
    next_beat: f32,
    pause: f32,
    vol: f32,
    gasp_at: Option<f32>,
    beat: Option<(Handle<PcmAudio>, f32)>,
    /// The low loop: its voice, its own volume, and its fade (start time,
    /// from, to, seconds).
    looper: Option<(Entity, f32)>,
    fade: (f32, f32, f32, f32),
}

fn t6(alias: &str) -> AliasCommand {
    AliasCommand::Play(PlayAlias {
        namespace: AssetNamespace::T6,
        alias: alias.to_owned(),
        fallback: None,
        origin_inches: None,
        snd_ent: None,
    })
}

/// The alias's first clip, decoded, and its own volume; asks for it the
/// first time (it is ready a few frames later).
fn clip(
    bank: &SoundBank,
    clips: &mut ClipStore,
    gaps: &mut MissingAliasGaps,
    alias: &str,
) -> Option<(PcmAudio, f32)> {
    let Some(key) = clip_keys_for_alias(&bank.0, AssetNamespace::T6, alias)
        .into_iter()
        .next()
    else {
        gaps.record(alias);
        return None;
    };
    clips.request(key.clone());
    let Some(Ok(pcm)) = clips.ready(&key) else {
        return None;
    };
    let volume = bank
        .0
        .sound_in(AssetNamespace::T6, alias)
        .and_then(|s| s.aliases.first())
        .map_or(0.8, |a| a.vol_min.max(0.0));
    Some((pcm, volume))
}

#[allow(clippy::too_many_arguments)]
pub(crate) fn update(
    mut commands: Commands,
    presented: Option<Res<PresentedSnapshot>>,
    local: Option<Res<LocalPresentClient>>,
    bank: Option<Res<SoundBank>>,
    mut clips: Option<ResMut<ClipStore>>,
    mut pcm_assets: ResMut<Assets<PcmAudio>>,
    mut looping_assets: ResMut<Assets<LoopingPcmAudio>>,
    mut gaps: ResMut<MissingAliasGaps>,
    epoch: Res<MatchEpoch>,
    time: Res<Time>,
    mut st: Local<LastStand>,
    mut sinks: Query<&mut bevy::audio::AudioSink>,
    mut play: MessageWriter<AliasCommand>,
) {
    use bevy::audio::AudioSinkPlayback;
    let now = time.elapsed_secs();
    let id = local.as_ref().map(|l| l.0);
    let client = id.map(|c| c.0);
    if st.client != client || st.epoch != epoch.0 {
        // A new match or a new view: the old voices went with it.
        if let Some((e, _)) = st.looper.take() {
            crate::backend::stop(&mut commands, e);
        }
        *st = LastStand {
            client,
            epoch: epoch.0,
            ..default()
        };
    }
    let down = match (presented.as_ref(), id) {
        (Some(p), Some(c)) => p
            .snapshot()
            .is_some_and(|snap| snap.meta.script_dvars(c).string("bo2zm_lsm") == Some("1")),
        _ => false,
    };
    if down && !st.down {
        st.down = true;
        st.next_beat = now;
        st.pause = 0.5;
        st.vol = 0.5;
        st.gasp_at = None;
        st.fade = (now, 0.0, 1.0, LOOP_IN_SECS);
        diag::info!(Audio, "audio: down, the heartbeat starts");
    } else if !down && st.down {
        st.down = false;
        let from = fade_gain(st.fade, now);
        st.fade = (now, from, 0.0, LOOP_OUT_SECS);
        play.write(t6("evt_laststand_in"));
        st.gasp_at = Some(now + GASP_AFTER_SECS);
        diag::info!(Audio, "audio: up again, the heartbeat stops");
    }
    if let Some(at) = st.gasp_at
        && now >= at
    {
        play.write(t6("revive_gasp"));
        st.gasp_at = None;
        diag::info!(Audio, "audio: revive gasp");
    }
    // The beat's timing runs with the sound off too (a test game builds no
    // sound bank); only the playing needs the bank.
    if st.down && now >= st.next_beat {
        if st.beat.is_none()
            && let (Some(bank), Some(clips)) = (bank.as_ref(), clips.as_mut())
        {
            st.beat = clip(bank, clips, &mut gaps, BEAT).map(|(pcm, v)| (pcm_assets.add(pcm), v));
        }
        if let Some((handle, own)) = st.beat.clone() {
            let gain = own * st.vol * crate::bus_gain::sfx();
            crate::backend::spawn_oneshot(
                &mut commands,
                handle,
                Volume::Linear(gain),
                1.0,
                epoch.0,
                AudioScope::Match,
            );
        }
        diag::info!(
            Audio,
            "audio: heartbeat at {:.2} (next in {:.2} s{})",
            st.vol,
            st.pause,
            if st.beat.is_some() { "" } else { ", clip not ready" }
        );
        st.next_beat = now + st.pause;
        st.pause = (st.pause * 1.05).min(2.0);
        st.vol = (st.vol * 1.05).min(1.0);
    }
    let (Some(bank), Some(clips)) = (bank, clips.as_mut()) else {
        return;
    };
    // The low loop, faded in and out with his sound slider over it.
    if st.down
        && st.looper.is_none()
        && let Some((pcm, own)) = clip(&bank, clips, &mut gaps, LOOP)
    {
        let handle = looping_assets.add(pcm.into_looping());
        let e = crate::backend::spawn_loop(
            &mut commands,
            handle,
            Volume::Linear(0.0),
            epoch.0,
            AudioScope::Match,
        );
        st.looper = Some((e, own));
        diag::info!(Audio, "audio: down loop `{LOOP}` at {own:.2}");
    }
    if let Some((e, own)) = st.looper {
        let g = fade_gain(st.fade, now);
        if !st.down && g <= 0.0 {
            crate::backend::stop(&mut commands, e);
            st.looper = None;
        } else if let Ok(mut sink) = sinks.get_mut(e) {
            sink.set_volume(Volume::Linear(own * g * crate::bus_gain::sfx()));
        }
    }
}

/// Where a fade is at `now`: (start, from, to, seconds).
fn fade_gain((start, from, to, secs): (f32, f32, f32, f32), now: f32) -> f32 {
    let t = if secs > 0.0 { ((now - start) / secs).clamp(0.0, 1.0) } else { 1.0 };
    from + (to - from) * t
}
