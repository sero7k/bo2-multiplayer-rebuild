use bevy::prelude::Resource;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct DisplayResolution {
    pub width: u32,
    pub height: u32,
}

impl DisplayResolution {
    pub const HD: Self = Self {
        width: 1280,
        height: 720,
    };

    pub const fn new(width: u32, height: u32) -> Self {
        Self { width, height }
    }
}

impl core::fmt::Display for DisplayResolution {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        write!(f, "{}x{}", self.width, self.height)
    }
}

#[derive(Resource, Clone, Debug, PartialEq)]
pub struct GameSettings {
    pub resolution: DisplayResolution,
    pub fullscreen: bool,
    pub vsync: bool,
    pub fov: f32,
    pub master_volume: f32,
    /// The music director's level (menu, lobby and match music), under
    /// `master_volume`.
    pub music_volume: f32,
    /// Low ambient music during a match (the music director's match playlist).
    pub match_music: bool,
    pub brightness: f32,
    pub shadows: bool,
    pub depth_of_field: bool,
    pub bloom: bool,
    pub sensitivity: f32,
    pub invert_mouse: bool,
    pub player_name: String,

    pub pad_layout: u8,
    pub pad_stick_layout: u8,
    pub pad_sensitivity: f32,
    pub pad_ads_sensitivity: f32,
    pub pad_invert: bool,
    pub pad_curve: u8,
    pub pad_aim_assist: u8,
    pub pad_prompts: u8,
    pub pad_vibration: bool,
    pub pad_deadzone_left: f32,
    pub pad_deadzone_right: f32,

    /// bo2zm: `bo2_quality=fast` (Black Ops II maps: half-size textures,
    /// single-layer ground, props hidden sooner); `full` by default.
    pub bo2_fast: bool,
    /// bo2zm: Controls > Move > "Omni Movement": sprint in any direction.
    /// Not Black Ops II's (it sprints forward only), so off by default.
    pub omni_movement: bool,
    /// bo2mp: Graphics > "Player Models": on another game's map, its own
    /// soldiers and animations (true), or Black Ops II's characters.
    pub map_game_bodies: bool,
    /// bo2mp: his friend code (8 digits, made once on this PC; 0 until
    /// then), which others add him by.
    pub friend_code: u32,
    /// bo2mp: the friend codes he added.
    pub friends: Vec<u32>,

    pub revision: u64,
}

impl Default for GameSettings {
    fn default() -> Self {
        Self {
            resolution: DisplayResolution::HD,
            fullscreen: false,
            vsync: true,
            fov: Self::FOV_DEFAULT,
            master_volume: 1.0,
            music_volume: Self::MUSIC_VOLUME_DEFAULT,
            match_music: true,
            brightness: 0.0,
            shadows: true,
            depth_of_field: true,
            bloom: true,
            sensitivity: 5.0,
            invert_mouse: false,
            player_name: "Player".to_owned(),
            pad_layout: 0,
            pad_stick_layout: 0,
            pad_sensitivity: Self::PAD_SENSITIVITY_DEFAULT,
            pad_ads_sensitivity: 1.0,
            pad_invert: false,
            pad_curve: 0,
            pad_aim_assist: 0,
            pad_prompts: 0,
            pad_vibration: true,
            pad_deadzone_left: 0.12,
            pad_deadzone_right: 0.12,
            bo2_fast: false,
            omni_movement: false,
            map_game_bodies: true,
            friend_code: 0,
            friends: Vec::new(),
            revision: 0,
        }
    }
}

impl GameSettings {
    pub const FOV_DEFAULT: f32 = 65.0;
    pub const FOV_MIN: f32 = 65.0;
    pub const FOV_MAX: f32 = 120.0;
    pub const PAD_SENSITIVITY_DEFAULT: f32 = 3.0;
    pub const PAD_LAYOUT_CUSTOM: u8 = 255;
    pub const MUSIC_VOLUME_DEFAULT: f32 = 0.8;

    /// bo2mp: an 8-digit friend code.
    pub const fn is_friend_code(code: u32) -> bool {
        code >= 10_000_000 && code <= 99_999_999
    }

    /// bo2mp: a code as shown, `1234 5678`.
    pub fn friend_code_text(code: u32) -> String {
        if Self::is_friend_code(code) {
            format!("{} {:04}", code / 10_000, code % 10_000)
        } else {
            "-".to_owned()
        }
    }

    pub fn touch(&mut self) {
        self.revision = self.revision.wrapping_add(1);
    }

    pub fn sanitize(&mut self) {
        self.resolution.width = self.resolution.width.clamp(640, 7680);
        self.resolution.height = self.resolution.height.clamp(480, 4320);
        self.fov = if self.fov.is_finite() {
            self.fov.clamp(Self::FOV_MIN, Self::FOV_MAX)
        } else {
            Self::FOV_DEFAULT
        };
        self.brightness = if self.brightness.is_finite() {
            self.brightness.clamp(-0.2, 0.2)
        } else {
            0.0
        };
        self.master_volume = self.master_volume.clamp(0.0, 1.0);
        self.music_volume = if self.music_volume.is_finite() {
            self.music_volume.clamp(0.0, 1.0)
        } else {
            Self::MUSIC_VOLUME_DEFAULT
        };
        self.sensitivity = self.sensitivity.clamp(0.1, 30.0);
        if self.pad_layout != Self::PAD_LAYOUT_CUSTOM {
            self.pad_layout = self.pad_layout.min(4);
        }
        self.pad_stick_layout = self.pad_stick_layout.min(3);
        self.pad_curve = self.pad_curve.min(2);
        self.pad_aim_assist = 0;
        self.pad_prompts = self.pad_prompts.min(3);
        let finite = |v: f32, lo: f32, hi: f32, default: f32| {
            if v.is_finite() {
                v.clamp(lo, hi)
            } else {
                default
            }
        };
        self.pad_sensitivity = finite(
            self.pad_sensitivity,
            1.0,
            // BO2's Look Sensitivity runs to 14 (Ultra).
            14.0,
            Self::PAD_SENSITIVITY_DEFAULT,
        );
        self.pad_ads_sensitivity = finite(self.pad_ads_sensitivity, 0.5, 1.5, 1.0);
        self.pad_deadzone_left = finite(self.pad_deadzone_left, 0.0, 0.4, 0.12);
        self.pad_deadzone_right = finite(self.pad_deadzone_right, 0.0, 0.4, 0.12);
        if !Self::is_friend_code(self.friend_code) {
            self.friend_code = 0;
        }
        let mine = self.friend_code;
        let mut seen = Vec::new();
        self.friends.retain(|&c| Self::is_friend_code(c) && c != mine && !seen.contains(&c) && {
            seen.push(c);
            true
        });
        self.friends.truncate(100);
        self.player_name = self.player_name.trim().chars().take(16).collect();
        if self.player_name.is_empty() {
            self.player_name = "Player".to_owned();
        }
    }
}

#[cfg(test)]
mod friend_tests {
    use super::GameSettings;

    #[test]
    fn friend_codes_are_eight_digits_and_kept_tidy() {
        let mut s = GameSettings { friend_code: 48_213_907, friends: vec![12_345_678, 12_345_678, 48_213_907, 5, 99_999_999], ..GameSettings::default() };
        s.sanitize();
        assert_eq!(s.friends, vec![12_345_678, 99_999_999]);
        assert_eq!(GameSettings::friend_code_text(48_213_907), "4821 3907");
        assert_eq!(GameSettings::friend_code_text(0), "-");
        let mut bad = GameSettings { friend_code: 123, ..GameSettings::default() };
        bad.sanitize();
        assert_eq!(bad.friend_code, 0);
    }
}
