//! bo2mp online: Black Ops II's front end through the master server.
//!
//! With a server configured (`online.cfg` -> `IW4L_MASTER_ADDR`), START in
//! the lobby does not load the map here: it opens a room on the server
//! (`MasterMenuAction::Host`), then starts its match (`StartMatch`), and
//! everyone in the room, him included, loads the map from the server's
//! offer (`session::follow_master_match`; a joiner's `Bo2mpMatchSetup`
//! comes from the offer there). A friend's lobby is joined from FRIENDS
//! (`MasterMenuAction::Join`); his front end closes when that match begins.
//!
//! When the server cannot host (unreachable, refused, or no room within
//! `HOST_WAIT_S`), the match starts on this PC as before.

use std::sync::Mutex;
use std::sync::atomic::{AtomicBool, Ordering};

use bevy::prelude::*;
use net::{MasterBridgeState, MasterMenuAction};

/// How long START waits for the server's room before playing here.
const HOST_WAIT_S: f64 = 15.0;

/// A server is configured (set every frame).
static AVAILABLE: AtomicBool = AtomicBool::new(false);

#[derive(Clone, Debug)]
struct Start {
    zone: String,
    mode: String,
    asked: Option<f64>,
}

static START: Mutex<Option<Start>> = Mutex::new(None);
static JOIN: Mutex<Option<MasterMenuAction>> = Mutex::new(None);
/// He left the room he joined (Esc on the waiting room).
static LEAVE: AtomicBool = AtomicBool::new(false);

/// The room he joined, while he waits for its host to start the match.
#[derive(Clone, Debug, Default, PartialEq, Eq, Hash)]
pub(crate) struct RoomView {
    pub name: String,
    pub map: String,
    pub mode: String,
    pub players: Vec<String>,
    pub max_players: u8,
}

static ROOM: Mutex<Option<RoomView>> = Mutex::new(None);
static ROOM_PREVIEW: Mutex<Option<RoomView>> = Mutex::new(None);

/// The room he waits in (the shell shows it instead of its menus).
pub(crate) fn room() -> Option<RoomView> {
    if let Some(p) = ROOM_PREVIEW.lock().ok().and_then(|p| p.clone()) {
        return Some(p);
    }
    ROOM.lock().ok().and_then(|r| r.clone())
}

pub(crate) fn preview_room(room: Option<RoomView>) {
    if let Ok(mut p) = ROOM_PREVIEW.lock() {
        *p = room;
    }
}

/// Esc in the waiting room: leave it.
pub(crate) fn queue_leave() {
    LEAVE.store(true, Ordering::Relaxed);
}

pub(crate) fn online_available() -> bool {
    AVAILABLE.load(Ordering::Relaxed)
}

/// START in the lobby with a server configured: this match through it.
pub(crate) fn queue_online_start(zone: String, mode: String) {
    if let Ok(mut s) = START.lock() {
        *s = Some(Start { zone, mode, asked: None });
    }
}

/// FRIENDS: join this lobby.
pub(crate) fn queue_join(advert: &net::master_protocol::Advert) {
    queue_join_room(advert.id, advert.map.clone(), advert.mode.clone());
}

/// FRIENDS / FIND A LOBBY: join this room.
pub(crate) fn queue_join_room(advert_id: net::master_protocol::AdvertId, map: String, mode: String) {
    if let Ok(mut j) = JOIN.lock() {
        *j = Some(MasterMenuAction::Join { advert_id, map, mode });
    }
}

#[allow(clippy::too_many_arguments)]
fn drive_online(
    mut commands: Commands,
    intent: Option<Res<net::MasterLaunchIntent>>,
    pending: Option<ResMut<net::PendingMasterMenuAction>>,
    bridge: Option<Res<net::MasterBridge>>,
    swap: Option<ResMut<session::SessionSwapRequest>>,
    front: Option<Res<crate::lui_frontend::Bo2mpFrontend>>,
    time: Res<Time<Real>>,
) {
    AVAILABLE.store(intent.as_deref().is_some_and(net::MasterLaunchIntent::configured), Ordering::Relaxed);
    let now = time.elapsed_secs_f64();
    let state = bridge.as_deref().map(net::MasterBridge::state);
    // A joiner's front end closes once his match begins (the session loads
    // it from the server's offer).
    if front.is_some() && matches!(state, Some(MasterBridgeState::Joined { in_match: true, .. })) {
        diag::info!(Ui, "bo2mp online: the host started the match; front end closed");
        commands.remove_resource::<crate::lui_frontend::Bo2mpFrontend>();
    }
    let waiting = match &state {
        Some(MasterBridgeState::Joined { name, map, mode, members, member_names, max_players, in_match: false, .. }) => {
            Some(RoomView {
                name: name.clone(),
                map: map.clone(),
                mode: mode.clone(),
                players: members.iter().map(|m| member_names.get(m).cloned().unwrap_or_else(|| "PLAYER".to_owned())).collect(),
                max_players: *max_players,
            })
        }
        _ => None,
    };
    if let Ok(mut r) = ROOM.lock()
        && *r != waiting
    {
        *r = waiting;
    }
    let Some(mut pending) = pending else { return };
    if LEAVE.swap(false, Ordering::Relaxed) {
        diag::info!(Ui, "bo2mp online: left the room");
        pending.0 = Some(MasterMenuAction::LeaveLobby);
        return;
    }
    if pending.0.is_none()
        && let Some(join) = JOIN.lock().ok().and_then(|mut j| j.take())
    {
        diag::info!(Ui, "bo2mp online: {join:?}");
        pending.0 = Some(join);
        return;
    }
    let Ok(mut slot) = START.lock() else { return };
    let Some(start) = slot.as_mut() else { return };
    let play_here = |reason: &str, start: &Start, swap: Option<ResMut<session::SessionSwapRequest>>| {
        diag::warn!(Ui, "bo2mp online: {reason}; `{}` starts on this PC", start.zone);
        if let Some(mut swap) = swap
            && let Err(e) = swap.request_zone(start.zone.clone())
        {
            diag::warn!(Ui, "bo2mp online: `{}` not started: {e}", start.zone);
        }
    };
    match state {
        Some(MasterBridgeState::Hosting { in_match: false, .. }) => {
            if pending.0.is_none() {
                diag::info!(Ui, "bo2mp online: room open; starting `{}` ({})", start.zone, start.mode);
                pending.0 = Some(MasterMenuAction::StartMatch { map: start.zone.clone(), mode: start.mode.clone() });
                *slot = None;
            }
        }
        Some(MasterBridgeState::Failed { .. } | MasterBridgeState::Closed { .. }) => {
            play_here("the server could not host", start, swap);
            pending.0 = Some(MasterMenuAction::LeaveLobby);
            *slot = None;
        }
        _ => match start.asked {
            None if bridge.is_none() && pending.0.is_none() => {
                diag::info!(Ui, "bo2mp online: opening a room for `{}` ({})", start.zone, start.mode);
                pending.0 = Some(MasterMenuAction::Host { map: start.zone.clone(), mode: start.mode.clone() });
                start.asked = Some(now);
            }
            Some(at) if now - at > HOST_WAIT_S => {
                play_here("no room from the server in time", start, swap);
                if bridge.is_some() {
                    pending.0 = Some(MasterMenuAction::LeaveLobby);
                }
                *slot = None;
            }
            _ => {}
        },
    }
}

pub(crate) fn register_online_match(app: &mut App) {
    app.add_systems(Update, drive_online.in_set(frame::ClientSet::Ui));
}
