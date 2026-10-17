use playerstate_iw4::PlayerState;

use crate::Pml;

pub fn accelerate(ps: &mut PlayerState, pml: &Pml, wishdir: &[f32; 3], wishspeed: f32, accel: f32) {
    accelerate_velocity(
        &mut ps.velocity,
        ps.pm_flags,
        pml.frametime,
        wishdir,
        wishspeed,
        accel,
    );
}

#[allow(clippy::assign_op_pattern)]
fn accelerate_velocity(
    velocity: &mut [f32; 3],
    pm_flags: u32,
    frametime: f32,
    wishdir: &[f32; 3],
    mut wishspeed: f32,
    accel: f32,
) {
    if (pm_flags & 8) != 0 {
        let wish_x = wishspeed * wishdir[0];
        let wish_y = wishdir[1] * wishspeed;
        let wish_z = wishspeed * wishdir[2];
        let mut delta = [
            wish_x - velocity[0],
            wish_y - velocity[1],
            wish_z - velocity[2],
        ];
        let delta_speed = libm::sqrtf(delta[1] * delta[1] + delta[0] * delta[0] + delta[2] * delta[2]);
        let scale = 1.0_f32 / if delta_speed <= 0.0 { 1.0 } else { delta_speed };
        delta = [delta[0] * scale, delta[1] * scale, delta[2] * scale];
        let mut acceleration = frametime * accel * wishspeed;
        if delta_speed < acceleration {
            acceleration = delta_speed;
        }
        velocity[0] = delta[0] * acceleration + velocity[0];
        velocity[1] = delta[1] * acceleration + velocity[1];
        velocity[2] = acceleration * delta[2] + velocity[2];
        return;
    }

    // (The sums keep the order the speed is added in: the rounding differs.)
    let current_speed =
        wishdir[0] * velocity[0] + velocity[1] * wishdir[1] + velocity[2] * wishdir[2];
    let add_speed = wishspeed - current_speed;
    if add_speed <= 0.0 || add_speed.is_nan() {
        return;
    }

    if wishspeed < 100.0_f32 {
        wishspeed = 100.0_f32;
    }
    let mut acceleration = frametime * accel * wishspeed;
    if add_speed < acceleration {
        acceleration = add_speed;
    }
    velocity[0] = wishdir[0] * acceleration + velocity[0];
    velocity[1] = wishdir[1] * acceleration + velocity[1];
    velocity[2] = wishdir[2] * acceleration + velocity[2];
}
