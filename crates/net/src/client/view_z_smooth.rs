//! bo2zm: Black Ops II eases the camera over a stair step (its
//! `cg_viewZSmoothing*` dvars): the player moves up or down the step at once,
//! the eye follows over 0.1 s and never trails it by more than 16 units.

/// `cg_viewZSmoothingTime` (0.1 s).
pub const SMOOTH_MS: i32 = 100;

/// `cg_viewZSmoothingMin`: a smaller step is not eased.
pub const SMOOTH_MIN: f32 = 1.0;

/// `cg_viewZSmoothingMax`: the most the eye trails a step.
pub const SMOOTH_MAX: f32 = 16.0;

#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct ViewZSmoothing {
    /// The step the eye is easing over (up is positive).
    offset: f32,

    /// When that step was taken (command time, ms).
    time_ms: i32,

    /// Steps taken by moves predicted since the last `settle`.
    pending: f32,

    pending_time_ms: i32,
}

impl ViewZSmoothing {
    /// A newly predicted move stepped `rise` units at `command_time`.
    pub fn note_step(&mut self, rise: f32, command_time: i32) {
        if rise != 0.0 {
            self.pending += rise;
            self.pending_time_ms = command_time;
        }
    }

    /// After a frame's prediction at `now_ms`: ease over the new steps. Only
    /// a normal, noclip or ufo player (pm_type 0, 2 or 3) is eased; a new
    /// step adds what is left of the last one.
    pub fn settle(&mut self, now_ms: i32, pm_type: i32) {
        let change = core::mem::take(&mut self.pending);
        let since_last = now_ms.wrapping_sub(self.time_ms);
        if change != 0.0
            && self.pending_time_ms != self.time_ms
            && matches!(pm_type, 0 | 2 | 3)
            && change.abs() >= SMOOTH_MIN
        {
            let mut left = 0.0;
            if since_last < SMOOTH_MS {
                let gap = self.pending_time_ms.wrapping_sub(self.time_ms);
                if (0..SMOOTH_MS).contains(&gap) {
                    left = (1.0 - gap as f32 / SMOOTH_MS as f32) * self.offset;
                }
            }
            self.offset = (left + change).clamp(-SMOOTH_MAX, SMOOTH_MAX);
            self.time_ms = self.pending_time_ms;
        } else if since_last > SMOOTH_MS {
            self.offset = 0.0;
        }
    }

    /// How far below the player's real eye the camera sits at `now_ms`.
    pub fn camera_drop(&self, now_ms: i32) -> f32 {
        let elapsed = now_ms.wrapping_sub(self.time_ms);
        if self.offset == 0.0 || elapsed < 0 {
            return 0.0;
        }
        let done = if elapsed < SMOOTH_MS {
            elapsed as f32 / SMOOTH_MS as f32
        } else {
            1.0
        };
        (1.0 - done) * self.offset
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_step_up_eases_over_a_tenth_of_a_second() {
        let mut z = ViewZSmoothing::default();
        z.note_step(12.0, 1000);
        z.settle(1000, 0);
        assert_eq!(z.camera_drop(1000), 12.0);
        assert!((z.camera_drop(1050) - 6.0).abs() < 1e-5);
        assert_eq!(z.camera_drop(1100), 0.0);
    }

    #[test]
    fn small_steps_and_dead_players_snap() {
        let mut z = ViewZSmoothing::default();
        z.note_step(0.75, 1000);
        z.settle(1000, 0);
        assert_eq!(z.camera_drop(1000), 0.0);
        z.note_step(12.0, 1016);
        z.settle(1016, playerstate_iw4::PM_TYPE_DEAD);
        assert_eq!(z.camera_drop(1016), 0.0);
    }

    #[test]
    fn stairs_add_up_to_sixteen_at_most() {
        let mut z = ViewZSmoothing::default();
        z.note_step(12.0, 1000);
        z.settle(1000, 0);
        // Half way through the first step: 6 left + 12 = 18, held to 16.
        z.note_step(12.0, 1050);
        z.settle(1050, 0);
        assert_eq!(z.camera_drop(1050), SMOOTH_MAX);
        z.note_step(-12.0, 1300);
        z.settle(1300, 0);
        assert_eq!(z.camera_drop(1300), -12.0);
    }
}
