//! bo2mp: tells the audio crate's music director which front-end playlist
//! the shell's screen wants — the custom-game lobby (the lobby, its map and
//! mode pickers, and Create-a-Class opened from there) or the main screens.
//! Read-only over the shell; the shell itself is not touched.

use bevy::prelude::*;
use frame::{ClientSet, MenuMusicScreen};

use crate::shell::{Screen, Shell};

fn music_screen_of(shell: &Shell) -> MenuMusicScreen {
    let lobby = |s: Screen| matches!(s, Screen::Lobby | Screen::MapPick | Screen::ModePick);
    match shell.screen {
        s if lobby(s) => MenuMusicScreen::Lobby,
        Screen::Classes | Screen::ClassEdit | Screen::ItemPick if lobby(shell.cac_from) => {
            MenuMusicScreen::Lobby
        }
        _ => MenuMusicScreen::Main,
    }
}

fn publish_music_screen(shell: Option<Res<Shell>>, mut out: ResMut<MenuMusicScreen>) {
    let next = shell.map_or(MenuMusicScreen::Main, |shell| music_screen_of(&shell));
    if *out != next {
        *out = next;
    }
}

pub(crate) fn register(app: &mut App) {
    app.init_resource::<MenuMusicScreen>()
        .add_systems(Update, publish_music_screen.in_set(ClientSet::Ui));
}
