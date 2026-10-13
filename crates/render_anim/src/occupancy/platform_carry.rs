//! bo2zm: riding a moving platform (the Tranzit bus) looks smooth.
//!
//! The server carries a rider with the bus each tick, so his place and his
//! turn arrive 20 times a second while the bus itself is drawn gliding on
//! its path. While he stands in the bus, his view keeps his place in the
//! bus as the server last sent it, and moves it with the bus as drawn.

use bevy::prelude::*;
use net::{FrameClock, LocalPresentClient, PresentedSnapshot};
use render_scene::WorldScriptModelInstance;

/// Where the bus the local player rides is: as the latest snapshot has it
/// (`snap_*`) and as it is drawn this frame (`drawn_*`). None when he
/// rides nothing.
#[derive(Resource, Default, Clone, Copy, Debug)]
pub struct PlatformCarry(pub Option<PlatformCarryPose>);

#[derive(Clone, Copy, Debug)]
pub struct PlatformCarryPose {
    pub snap_origin: [f32; 3],
    pub snap_angles: [f32; 3],
    pub drawn_origin: [f32; 3],
    pub drawn_angles: [f32; 3],
}

fn rotation(angles: [f32; 3]) -> Quat {
    let [pitch, yaw, roll] = angles;
    Quat::from_euler(
        EulerRot::ZYX,
        yaw.to_radians(),
        pitch.to_radians(),
        roll.to_radians(),
    )
}

impl PlatformCarryPose {
    /// A world place he holds in the snapshot's bus, moved to the drawn bus.
    pub fn carry_point(&self, point: [f32; 3]) -> [f32; 3] {
        let local = rotation(self.snap_angles).inverse()
            * (Vec3::from_array(point) - Vec3::from_array(self.snap_origin));
        (Vec3::from_array(self.drawn_origin) + rotation(self.drawn_angles) * local).to_array()
    }

    /// How far the drawn bus has turned past the snapshot's.
    pub fn yaw_lead(&self) -> f32 {
        math_iw4::angle_subtract(self.drawn_angles[1], self.snap_angles[1])
    }
}

pub fn track_platform_carry(
    clock: Res<FrameClock>,
    presented: Res<PresentedSnapshot>,
    local: Res<LocalPresentClient>,
    slots: Option<Res<net::CEntitySlots>>,
    runtimes: Query<&net::CEntityRuntime>,
    owners: Query<&WorldScriptModelInstance>,
    mut carry: ResMut<PlatformCarry>,
) {
    carry.0 = None;
    static OFF: std::sync::OnceLock<bool> = std::sync::OnceLock::new();
    if *OFF.get_or_init(|| std::env::var_os("IW4L_NO_CARRY").is_some()) {
        return;
    }
    let (Some(slots), Some(snapshot), Some(ps)) =
        (slots, presented.snapshot(), presented.player(local.0))
    else {
        return;
    };
    let snap_time = sim::level_time_ms(snapshot.tick);
    let drawn_time = presented.trajectory_time_ms(if clock.started() {
        clock.time()
    } else {
        snap_time
    });
    for owner in &owners {
        let model = &owner.current_model.0;
        let ride_box = sim::platform_ride_box(model);
        // A Die Rise car carries whoever stands on its floor or roof: his
        // ground entity is the car.
        let on_ground = sim::platform_rides_on_ground(model);
        if ride_box.is_none() && !on_ground {
            continue;
        }
        let Some(number) = owner.gentity_number else {
            continue;
        };
        let Some(mover) = snapshot
            .meta
            .script_movers
            .iter()
            .find(|m| m.state.number == i32::from(number))
        else {
            continue;
        };
        let Some(runtime) = slots
            .entity_for_number(number)
            .and_then(|entity| runtimes.get(entity).ok())
        else {
            continue;
        };
        // A bus eased from last tick's place stands, at the server, where
        // its easing ends: that is the bus his place was carried with.
        let pos = sim::pos_from_entity_state(&mover.state);
        let apos = sim::apos_from_entity_state(&mover.state);
        let end = |tr: &entity_iw4::Trajectory| {
            if tr.tr_type == entity_iw4::TR_LINEAR_STOP {
                tr.tr_time.wrapping_add(tr.tr_duration)
            } else {
                snap_time
            }
        };
        let pose = PlatformCarryPose {
            snap_origin: entity_iw4::evaluate_trajectory(&pos, end(&pos)),
            snap_angles: entity_iw4::evaluate_trajectory(&apos, end(&apos)),
            drawn_origin: entity_iw4::evaluate_trajectory(&runtime.current.pos, drawn_time),
            drawn_angles: entity_iw4::evaluate_trajectory(&runtime.current.apos, drawn_time),
        };
        let inside = match ride_box {
            Some((lo, hi)) => {
                let local_place = rotation(pose.snap_angles).inverse()
                    * (Vec3::from_array(ps.origin) - Vec3::from_array(pose.snap_origin));
                (0..3).all(|i| local_place[i] >= lo[i] && local_place[i] <= hi[i])
            }
            None => ps.ground_entity_num == i32::from(number),
        };
        if inside {
            carry.0 = Some(pose);
            carry_log(&pose, ps.origin, clock.time());
            carry_trace(&pose, ps.origin, ps.viewangles[1], clock.time(), snapshot.tick.0);
            return;
        }
    }
}

/// bo2zm test aid (IW4L_CARRY_LOG=1): how far the snapshot bus and the drawn
/// bus are apart while he rides, every 120 frames.
fn carry_log(pose: &PlatformCarryPose, origin: [f32; 3], time: i32) {
    use std::sync::atomic::{AtomicU32, Ordering};
    static ON: std::sync::OnceLock<bool> = std::sync::OnceLock::new();
    static N: AtomicU32 = AtomicU32::new(0);
    if !*ON.get_or_init(|| std::env::var_os("IW4L_CARRY_LOG").is_some()) {
        return;
    }
    if N.fetch_add(1, Ordering::Relaxed) % 120 != 0 {
        return;
    }
    let gap = Vec3::from_array(pose.drawn_origin) - Vec3::from_array(pose.snap_origin);
    diag::info!(
        World,
        "carry: t={time} gap={:.1} yaw_lead={:.2} at {:?} -> {:?}",
        gap.length(),
        pose.yaw_lead(),
        origin.map(|v| v.round()),
        pose.carry_point(origin).map(|v| v.round()),
    );
}

/// bo2zm test aid (IW4L_CARRY_TRACE=1): every riding frame, up to 4,000:
/// time, snapshot tick, his place, the snapshot bus and the drawn bus.
fn carry_trace(pose: &PlatformCarryPose, origin: [f32; 3], yaw: f32, time: i32, tick: u32) {
    use std::sync::atomic::{AtomicU32, Ordering};
    static ON: std::sync::OnceLock<bool> = std::sync::OnceLock::new();
    static N: AtomicU32 = AtomicU32::new(0);
    if !*ON.get_or_init(|| std::env::var_os("IW4L_CARRY_TRACE").is_some()) {
        return;
    }
    if N.fetch_add(1, Ordering::Relaxed) >= 4000 {
        return;
    }
    let r = |v: [f32; 3]| v.map(|x| (x * 10.0).round() / 10.0);
    diag::info!(
        World,
        "ctrace t={time} tick={tick} ps={:?} yaw={yaw:.1} snap={:?} {:.1} drawn={:?} {:.1}",
        r(origin),
        r(pose.snap_origin),
        pose.snap_angles[1],
        r(pose.drawn_origin),
        pose.drawn_angles[1],
    );
}

/// bo2zm test aid (IW4L_CAMERA_JITTER=1): how far the camera strays, frame
/// to frame, from where its last frame's speed would put it (a smooth ride
/// strays little; 20 Hz steps stray a lot; speed is per ms): mean and largest every 600
/// frames, and whether he was riding.
pub fn camera_jitter(time: i32, origin: [f32; 3], riding: bool) {
    use std::sync::Mutex;
    static ON: std::sync::OnceLock<bool> = std::sync::OnceLock::new();
    if !*ON.get_or_init(|| std::env::var_os("IW4L_CAMERA_JITTER").is_some()) {
        return;
    }
    #[derive(Default)]
    struct Tally {
        last: Option<(i32, Vec3, Vec3)>,
        sum: f32,
        max: f32,
        max_time: i32,
        n: u32,
        riding: u32,
    }
    static TALLY: Mutex<Option<Tally>> = Mutex::new(None);
    let mut guard = TALLY.lock().unwrap();
    let t = guard.get_or_insert_with(Tally::default);
    let now = Vec3::from_array(origin);
    match t.last {
        Some((last_time, last, speed)) if time > last_time => {
            let dt = (time - last_time) as f32;
            let stray = (now - (last + speed * dt)).length();
            t.sum += stray;
            if stray > 30.0 {
                diag::info!(World, "camera jump: t={time} dt={dt} stray {stray:.1} riding {riding}");
            }
            if stray > t.max {
                t.max = stray;
                t.max_time = time;
            }
            t.n += 1;
            t.riding += u32::from(riding);
            if t.n >= 600 {
                diag::info!(
                    World,
                    "camera jitter: mean {:.2} max {:.1} (t={}) riding {}/{}",
                    t.sum / t.n as f32,
                    t.max,
                    t.max_time,
                    t.riding,
                    t.n
                );
                t.sum = 0.0;
                t.max = 0.0;
                t.n = 0;
                t.riding = 0;
            }
            t.last = Some((time, now, (now - last) / dt));
        }
        Some((last_time, ..)) if time == last_time => {}
        _ => t.last = Some((time, now, Vec3::ZERO)),
    }
}
