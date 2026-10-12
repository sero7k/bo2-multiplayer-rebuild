//! The music director: separate menu, lobby and in-match playlists, built
//! from the games' own music (each game's main-menu track) and the user's
//! files (`<exe dir>/music/{menu,lobby,match}`), with fades and crossfades
//! between tracks and contexts. See `config.rs` for `music.cfg`.

mod config;
mod director;
mod logic;
mod manifest;
mod stream;

pub use logic::{Action, Core, Envelope, FadeTimings, MusicContext, Playlist, TrackId};
pub use director::MusicVoice;
pub use stream::StreamedMusic;

pub(crate) use director::register;
