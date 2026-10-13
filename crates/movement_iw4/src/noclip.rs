//! bo2zm: no clip (pm_type 2, the solo cheat menu's fly): the player flies
//! along his view, through walls and floors. Forward and back go along the
//! view (looking up flies up), the strafe keys sideways; sprint held flies
//! at sprint speed. No gravity, no collision, no fall.

use playerstate_iw4::{ENTITYNUM_NONE, PlayerState, UserCmd, buttons};

/// The move type that flies (as BO2 numbers it; 3 is ufo).
pub const PM_TYPE_NOCLIP: i32 = 2;

/// Sprint held: the player's sprint speed scale.
const SPRINT_SCALE: f32 = 1.5;

/// The velocity one command flies at, along the view's forward and right.
pub fn velocity(speed: f32, cmd: &UserCmd, forward: [f32; 3], right: [f32; 3]) -> [f32; 3] {
    let speed = speed
        * if cmd.buttons & buttons::SPRINT != 0 {
            SPRINT_SCALE
        } else {
            1.0
        };
    let (f, r) = (
        f32::from(cmd.forwardmove) / 127.0,
        f32::from(cmd.rightmove) / 127.0,
    );
    core::array::from_fn(|i| (forward[i] * f + right[i] * r) * speed)
}

/// One move: straight along the view for the command's time.
pub(crate) fn fly(
    ps: &mut PlayerState,
    cmd: &UserCmd,
    forward: [f32; 3],
    right: [f32; 3],
    frametime: f32,
) {
    ps.velocity = velocity(ps.speed as f32, cmd, forward, right);
    for i in 0..3 {
        ps.origin[i] += ps.velocity[i] * frametime;
    }
    ps.ground_entity_num = ENTITYNUM_NONE;
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn forward_flies_along_the_view_and_sprint_is_faster() {
        let mut cmd = UserCmd {
            forwardmove: 127,
            ..Default::default()
        };
        let up = [0.0, 0.0, 1.0];
        assert_eq!(
            velocity(190.0, &cmd, up, [0.0, -1.0, 0.0]),
            [0.0, 0.0, 190.0]
        );
        cmd.buttons = buttons::SPRINT;
        assert_eq!(velocity(190.0, &cmd, up, [0.0, -1.0, 0.0])[2], 285.0);
        cmd = UserCmd {
            forwardmove: -127,
            rightmove: 127,
            ..Default::default()
        };
        assert_eq!(
            velocity(100.0, &cmd, [1.0, 0.0, 0.0], [0.0, -1.0, 0.0]),
            [-100.0, -100.0, 0.0]
        );
    }
}
