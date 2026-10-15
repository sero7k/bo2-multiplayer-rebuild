use bevy::prelude::*;

use crate::playback::PlayerSoundPlugin;

pub struct AudioPlugin;

impl Plugin for AudioPlugin {
    fn build(&self, app: &mut App) {
        crate::backend::register(app);
        app.add_plugins(PlayerSoundPlugin);
        crate::match_set::register(app);
        crate::frontend::register_frontend_audio(app);
        crate::music::register(app);
        crate::entity_events::register_entity_event_audio(app);
        crate::rumble::register(app);
        crate::ghosts_countdown::register(app);
        // bo2mp: Resurgence plays Modern Warfare III's (2023) sounds once
        // its install's sounds are read.
        sim::set_jup_sound_lookup(Some(asset_audio::jup_has));
    }
}
