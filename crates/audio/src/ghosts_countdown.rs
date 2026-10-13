//! bo2mp: the match start countdown ("5, 4, 3, 2, 1") ticks with Ghosts'
//! own countdown sound (`match_countdown_tick`, a loaded PCM clip in its
//! `code_post_gfx_mp`) when Ghosts is installed. Black Ops II's script
//! asks for `uin_start_count_down` each second (`ui_sounds` routes it to
//! BO2's or Modern Warfare 3's tick); `playback` plays this clip in its
//! place. The zone is read once, off the main thread, at start-up.

use std::sync::{Arc, Once, OnceLock};

use bevy::prelude::*;
use frame::LaunchIdentity;

use crate::pcm::PcmAudio;

static CLIP: OnceLock<Option<PcmAudio>> = OnceLock::new();

/// The countdown tick's names, as Black Ops II's script and the UI routes
/// give them.
pub(crate) fn is_countdown(alias: &str) -> bool {
    alias.eq_ignore_ascii_case("uin_start_count_down") || alias.eq_ignore_ascii_case("match_countdown_tick")
}

/// Ghosts' tick, once read (None without Ghosts or while reading).
pub(crate) fn clip() -> Option<PcmAudio> {
    CLIP.get().cloned().flatten()
}

fn warm(identity: Option<Res<LaunchIdentity>>) {
    static STARTED: Once = Once::new();
    let root = identity.map(|i| i.games_root.clone());
    STARTED.call_once(move || {
        std::thread::spawn(move || {
            let read = crate::music::ghosts::install(root.as_deref()).map(|install| {
                fastfile_iw6::ui_sound::loaded_pcm(
                    &install,
                    fastfile_iw6::ui_sound::COUNTDOWN_ZONE,
                    fastfile_iw6::ui_sound::COUNTDOWN_ALIAS,
                )
            });
            let clip = match read {
                Some(Ok(pcm)) => {
                    diag::info!(
                        Audio,
                        "audio: match countdown plays Ghosts' `{}` ({:.2} s)",
                        fastfile_iw6::ui_sound::COUNTDOWN_ALIAS,
                        pcm.samples.len() as f32 / f32::from(pcm.channels.max(1)) / pcm.sample_rate.max(1) as f32
                    );
                    PcmAudio::from_prepared(Arc::from(pcm.samples), pcm.channels, pcm.sample_rate)
                }
                Some(Err(e)) => {
                    diag::warn!(Audio, "audio: Ghosts' countdown tick not read ({e}); BO2's plays");
                    None
                }
                None => {
                    diag::info!(Audio, "audio: no Ghosts install; the match countdown plays BO2's tick");
                    None
                }
            };
            let _ = CLIP.set(clip);
        });
    });
}

pub(crate) fn register(app: &mut App) {
    app.add_systems(Update, warm);
}
