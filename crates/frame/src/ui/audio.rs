use bevy::prelude::*;

#[derive(Message, Clone, Debug)]
pub struct UiPlaySound {
    pub alias: String,
}

#[derive(Message, Clone, Debug)]
pub struct UiPlayMusic {
    pub alias: String,
}

#[derive(Message, Clone, Debug, Default)]
pub struct UiStopMusic;

/// Which front-end music the menus want: the main screens' playlist, or the
/// custom-game lobby's (the lobby, its map and mode pickers). The ui shell
/// publishes it every frame; the audio crate's music director reads it.
#[derive(Resource, Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum MenuMusicScreen {
    #[default]
    Main,
    Lobby,
}
