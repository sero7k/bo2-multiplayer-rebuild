use playerstate_iw4::{PlayerState, pm_flags};

use crate::{Bo2Feel, Pml};

pub fn friction(ps: &mut PlayerState, pml: &Pml, feel: Bo2Feel) {
    friction_scaled(ps, pml, feel.jump_slowdown, |_| 1.0);
}

/// bo2zm: `ground_scale` scales the ground's friction (a dive's slide); it
/// is asked only when that friction applies, after a hard landing's or a
/// jump's own scale. `jump_slowdown`: a landed jump grips harder for a
/// moment (BO2's jump_slowdownEnable; off, none of it).
#[allow(clippy::assign_op_pattern)]
pub(crate) fn friction_scaled(
    ps: &mut PlayerState,
    pml: &Pml,
    jump_slowdown: bool,
    ground_scale: impl FnOnce(&mut PlayerState) -> f32,
) {
    // Walking, the speed is measured along the ground.
    let speed = if pml.walking != 0 {
        velocity_speed(&[ps.velocity[0], ps.velocity[1], 0.0])
    } else {
        velocity_speed(&ps.velocity)
    };
    if stop_slow_velocity(&mut ps.velocity, speed) {
        return;
    }

    let flags = ps.pm_flags;
    let mut drop = 0.0_f32;
    if (flags & 0x10000) == 0 {
        if pml.walking != 0 && (pml.ground_trace[4] & 2) == 0 && (flags & 0x100) == 0 {
            let mut control = speed;
            if speed < 100.0 {
                control = 100.0;
            }

            if (flags & 0x80) != 0 {
                // A hard landing slides instead of stopping dead.
                control *= crate::feel::HARD_LANDING_FRICTION_SCALE;
            } else if (flags & 0x2000) != 0 {
                control *= walking_scale(ps, jump_slowdown);
            }
            control *= ground_scale(ps);

            drop = control * 5.5;
            drop = drop * pml.frametime + 0.0;
        }
        // Slick ground still slows you.
        if (pml.ground_trace[4] & 2) != 0 {
            drop += crate::feel::SLIDING_FRICTION * speed * pml.frametime;
        }
    } else {
        drop = (speed / (ps.melee_charge_time as f32 * 0.001)) * pml.frametime;
    }

    if ps.pm_type == 5 {
        drop = speed * 5.0 * pml.frametime + drop;
    }

    let mut new_speed = speed - drop;
    if new_speed < 0.0 {
        new_speed = 0.0;
    }
    let scale = new_speed / speed;
    ps.velocity[0] = scale * ps.velocity[0];
    ps.velocity[1] = ps.velocity[1] * scale;
    ps.velocity[2] = scale * ps.velocity[2];
}

fn walking_scale(ps: &mut PlayerState, jump_slowdown: bool) -> f32 {
    if ps.pm_time < 0x709 {
        // A landed jump grips harder only while the jump slowdown is on.
        if jump_slowdown {
            if ps.pm_time > 0x6a3 {
                return 2.5;
            }

            return (ps.pm_time as f32 * 1.5 * 0.0005882352706976235) + 1.0;
        }
    } else {
        ps.pm_flags &= !pm_flags::JUMPING;
        ps.jump_origin_z = 0.0;
    }
    1.0
}

fn velocity_speed(velocity: &[f32; 3]) -> f32 {
    let speed_squared =
        velocity[0] * velocity[0] + velocity[1] * velocity[1] + velocity[2] * velocity[2];
    libm::sqrtf(speed_squared)
}

fn stop_slow_velocity(velocity: &mut [f32; 3], speed: f32) -> bool {
    if speed < 1.0 {
        *velocity = [0.0, 0.0, 0.0];
        true
    } else {
        false
    }
}
