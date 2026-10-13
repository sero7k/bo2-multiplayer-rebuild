//! bo2zm: his own dive and voice. Black Ops II's player plays the
//! dive-to-prone launch whoosh when the dive starts, the landing thud
//! plus a grunt (the client script's `dtplandsoundplayer` exert) when he
//! touches down, and a pain grunt when he is hurt. Our movement keeps the
//! dive in his playerstate flags, so this watches those flags (and his
//! health) and plays BO2's own aliases, 2D, as the player-only `_plr`
//! sounds are.

use bevy::prelude::*;
use net::{LocalPresentClient, PresentedSnapshot};

use crate::{AliasCommand, PlayAlias};

use movement_iw4::{PMF_DIVE, PMF_DIVE_SLIDE};

/// A pain grunt at most this often, as BO2's exert cooldown.
const PAIN_GAP_SECS: f32 = 0.6;

#[derive(Default)]
pub(crate) struct LastSeen {
    client: Option<u32>,
    flags: u32,
    health: i32,
    pain_at: f32,
}

fn t6(alias: &str) -> AliasCommand {
    AliasCommand::Play(PlayAlias {
        namespace: asset_core::AssetNamespace::T6,
        alias: alias.to_owned(),
        fallback: None,
        origin_inches: None,
        snd_ent: None,
    })
}

pub(crate) fn update(
    presented: Option<Res<PresentedSnapshot>>,
    local: Option<Res<LocalPresentClient>>,
    time: Res<Time>,
    mut last: Local<LastSeen>,
    mut play: MessageWriter<AliasCommand>,
) {
    let (Some(presented), Some(local)) = (presented, local) else {
        last.client = None;
        return;
    };
    let Some(ps) = presented.player(local.0) else {
        last.client = None;
        return;
    };
    let now = time.elapsed_secs();
    if last.client != Some(local.0.0) {
        // A new match or a new view: take the state as it is, no sounds.
        *last = LastSeen {
            client: Some(local.0.0),
            flags: ps.pm_flags,
            health: ps.health,
            pain_at: now,
        };
        return;
    }
    let rose = |flag: u32| ps.pm_flags & flag != 0 && last.flags & flag == 0;
    if rose(PMF_DIVE) {
        // Its secondary is the character's launch effort.
        play.write(t6("fly_dtp_launch_plr"));
        diag::info!(Audio, "audio: dive launch");
    }
    if rose(PMF_DIVE_SLIDE) {
        play.write(t6("fly_dtp_land_plr_default"));
        play.write(t6("vox_exert_generic_grunt"));
        diag::info!(Audio, "audio: dive land");
    }
    if ps.health < last.health && last.health > 0 && now - last.pain_at >= PAIN_GAP_SECS {
        play.write(t6("vox_exert_generic_pain"));
        diag::info!(Audio, "audio: pain grunt ({} -> {})", last.health, ps.health);
        last.pain_at = now;
    }
    last.flags = ps.pm_flags;
    last.health = ps.health;
}
