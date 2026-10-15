//! bo2mp: Resurgence's opening sounds (the pre-lobby and drop music, the
//! plane, the island's infil strikes, the first announcer lines) are
//! Modern Warfare III (2023) clips read and decoded from the install's
//! packages when first asked for; they all come at once at the drop, the
//! music runs half a minute, and a busy machine let some of them expire
//! before they were ready. Once a match's bank holds those sounds, they are
//! queued for decoding ahead, during the pre-match countdown.

use asset_core::AssetNamespace;
use bevy::prelude::*;

use crate::backend::MatchEpoch;
use crate::clip_store::ClipStore;
use crate::playback::SoundBank;

fn warm(
    mut warmed: Local<Option<u64>>,
    epoch: Res<MatchEpoch>,
    bank: Option<Res<SoundBank>>,
    clips: Option<ResMut<ClipStore>>,
) {
    if *warmed == Some(epoch.0) || !sim::resurgence_on() {
        return;
    }
    let (Some(bank), Some(mut clips)) = (bank, clips) else {
        return;
    };
    if !asset_audio::has_jup_sounds(&bank.0) {
        return;
    }
    let mut queued = 0;
    for alias in sim::resurgence_opening_aliases() {
        queued += clips.request_alias(AssetNamespace::Jup, &asset_audio::jup_alias_key(alias));
    }
    *warmed = Some(epoch.0);
    diag::info!(
        Audio,
        "audio: Resurgence: {queued} opening MW3 (2023) clips decoding ahead"
    );
}

pub(crate) fn register(app: &mut App) {
    app.add_systems(Update, warm);
}
