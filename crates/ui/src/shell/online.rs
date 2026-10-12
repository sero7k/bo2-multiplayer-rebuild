//! bo2mp shell: the online panel's numbers, from the master server's list
//! of lobbies (`net::MasterBrowser`, refreshed every 15 seconds while the
//! front end is up). With no master server set (`IW4L_MASTER_ADDR`) the
//! panel says he is playing offline.

use bevy::prelude::*;

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash)]
pub(crate) enum Status {
    #[default]
    Offline,
    Connecting,
    Online,
    Unreachable,
}


#[derive(Resource, Clone, Debug, Default, PartialEq, Eq, Hash)]
pub(crate) struct OnlineInfo {
    pub status: Status,
    /// Everyone on the server: the players in its lobbies and him.
    pub players: u32,
    pub in_lobbies: u32,
    pub in_matches: u32,
    pub open_lobbies: u32,
    pub server: String,
}

pub(crate) fn poll_online(
    mut commands: Commands,
    browser: Option<Res<net::MasterBrowser>>,
    front: Option<Res<crate::lui_frontend::Bo2mpFrontend>>,
    info: Option<Res<OnlineInfo>>,
    time: Res<Time>,
    mut last_refresh: Local<f64>,
) {
    let now = time.elapsed_secs_f64();
    let server = std::env::var("IW4L_MASTER_SERVER_NAME").unwrap_or_default();
    let next = match browser.as_deref() {
        None => OnlineInfo {
            status: Status::Offline,
            players: 1,
            server: "LOCAL".to_owned(),
            ..default()
        },
        Some(b) => {
            if front.is_some() && now - *last_refresh > 15.0 {
                *last_refresh = now;
                b.refresh();
            }
            let snap = b.snapshot();
            super::friends::set_seen(snap.friends.clone());
            let in_lobbies: u32 = snap.adverts.iter().filter(|a| !a.in_match).map(|a| u32::from(a.players)).sum();
            let in_matches: u32 = snap.adverts.iter().filter(|a| a.in_match).map(|a| u32::from(a.players)).sum();
            let open = snap.adverts.iter().filter(|a| !a.locked && a.players < a.max_players).count() as u32;
            OnlineInfo {
                status: if snap.error.is_some() {
                    Status::Unreachable
                } else if snap.loading && snap.generation == 0 {
                    Status::Connecting
                } else {
                    Status::Online
                },
                players: in_lobbies + in_matches + 1,
                in_lobbies,
                in_matches,
                open_lobbies: open,
                server: if server.is_empty() { "MASTER".to_owned() } else { server.to_uppercase() },
            }
        }
    };
    if info.as_deref() != Some(&next) {
        commands.insert_resource(next);
    }
}
