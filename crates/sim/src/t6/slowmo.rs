//! bo2mp: `setslowmotion( start, end, seconds )`: the game's time scale,
//! ramped from `start` to `end` over `seconds` (Black Ops II's final killcam
//! slows to a quarter on the killing shot, then back). The killcam's replay
//! reads it and falls behind live time by what the slow motion takes.

use std::sync::Mutex;
use std::time::Instant;

struct Ramp {
    start: f32,
    end: f32,
    seconds: f32,
    at: Instant,
}

static RAMP: Mutex<Option<Ramp>> = Mutex::new(None);

pub(super) fn set(start: f32, end: f32, seconds: f32) {
    if let Ok(mut r) = RAMP.lock() {
        *r = Some(Ramp { start: start.clamp(0.05, 4.0), end: end.clamp(0.05, 4.0), seconds: seconds.max(0.0), at: Instant::now() });
    }
}

/// The time scale now (1 when nothing slowed it).
pub fn scale() -> f32 {
    let Ok(r) = RAMP.lock() else { return 1.0 };
    let Some(r) = r.as_ref() else { return 1.0 };
    if r.seconds <= 0.0 {
        return r.end;
    }
    let t = (r.at.elapsed().as_secs_f32() / r.seconds).min(1.0);
    r.start + (r.end - r.start) * t
}

/// Back to normal speed (a new match).
pub fn reset() {
    if let Ok(mut r) = RAMP.lock() {
        *r = None;
    }
}
