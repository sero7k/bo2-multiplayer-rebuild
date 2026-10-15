//! Resurgence: squads, a closing gas circle, redeploys while a squadmate
//! lives, cash and a buy station. The numbers live in one [`Rules`]
//! value; the match script reads them as dvars (`sim`'s `t6::br`).

pub const GAMETYPE_TOKEN: &str = "br";

/// The game type the match runs underneath (every player for himself; the
/// squads are kept by the Resurgence script).
pub const BASE_GAMETYPE: &str = "dm";

pub const DISPLAY_NAME: &str = "RESURGENCE";

pub const MAP: &str = "mp_jup_escape5";

pub const MAP_TITLE: &str = "REBIRTH ISLAND";

pub const MAX_SQUAD_SIZE: u8 = 4;

/// One circle: it waits, then closes to a smaller one over `close_s`.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct CirclePhase {
    pub wait_s: u16,
    pub close_s: u16,
    /// The radius it closes to, as a share of the first circle's.
    pub end_radius: f32,
    /// Gas damage a second outside it while this phase runs.
    pub gas_dps: u16,
    /// Redeploy wait while this phase runs; `None`: no redeploys (a dead
    /// player is out unless bought back).
    pub redeploy_s: Option<u16>,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum BuyKind {
    LoadoutDrop,
    SelfRevive,
    Uav,
    SquadBuyback,
    ArmorPlates,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct BuyItem {
    pub kind: BuyKind,
    pub key: &'static str,
    pub label: &'static str,
    pub price: u32,
}

/// A contract: a tablet on the map a squad takes, one at a time.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ContractKind {
    /// Kill a marked enemy before the time runs out.
    Bounty,
    /// Open three supply boxes, one after the other.
    Scavenger,
    /// Hold a spot to upload data: the next circle is shown.
    Recon,
    /// Reach a buy station in time: cash and a discount.
    SupplyRun,
    /// Survive while marked on everyone's map.
    MostWanted,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Contract {
    pub kind: ContractKind,
    pub key: &'static str,
    pub label: &'static str,
    /// Cash to every squadmate when it is done.
    pub reward: u32,
    /// Seconds to do it (0: no limit).
    pub time_s: u16,
    /// How many of its tablets are on the map.
    pub count: u8,
}

/// A ground loot rarity: how often it rolls (weights add up across tiers)
/// and how many attachments its gun carries.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct LootTier {
    pub key: &'static str,
    pub label: &'static str,
    pub weight: u16,
    pub attachments: u8,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Rules {
    pub squad_size: u8,
    /// Seconds from the match start to the first circle's wait.
    pub infil_s: u16,
    pub health: u16,
    pub plate_hp: u16,
    pub plate_slots: u8,
    pub plates_carried_max: u8,
    pub plate_insert_ms: u16,
    /// Plates he comes back with on a redeploy (and starts with).
    pub spawn_plates: u8,
    pub start_cash: u32,
    pub kill_cash: u32,
    /// A squadmate's kill takes this off the dead squadmates' redeploy wait.
    pub kill_redeploy_cut_s: u16,
    pub cash_stacks: &'static [u32],
    pub buy_station_count: u8,
    pub uav_s: u16,
    pub phases: &'static [CirclePhase],
    pub buy: &'static [BuyItem],
    pub loot: &'static [LootTier],
    pub loot_spots: u16,
    /// Seconds off a dead squadmate's redeploy wait when the squad breaks
    /// an enemy's armor, opens a supply box, or kills a bounty.
    pub armor_break_cut_s: u16,
    pub cache_cut_s: u16,
    pub bounty_cut_s: u16,
    /// Supply boxes on the map (cash, plates, a gun, a gas mask).
    pub caches: u16,
    /// Seconds of gas a gas mask holds off before it breaks.
    pub gas_mask_s: u16,
    /// Percent chance of the Heavy Armor event (one more plate slot all match).
    pub heavy_armor_pct: u8,
    pub contracts: &'static [Contract],
    /// Supply Run: the share off the squad's next purchase, in percent.
    pub supply_run_discount_pct: u8,
    /// A bounty's target that lives through it: his squad's reward.
    pub bounty_survive_cash: u32,
    /// Recon: ms to upload alone (more squadmates in it go faster).
    pub recon_ms: u32,
}

/// The numbers, from what Activision and the press published for Warzone's
/// Resurgence in the Modern Warfare III era (none was read from the game's
/// files: its game type keeps them in compiled scripts):
///
/// - Gas: 10 damage a tick ("Gas damage increased to 10 damage per tick, up
///   from 9", Warzone Season 1 patch notes, callofduty.com, 2023-12).
/// - Redeploy: off once Circle 3 has closed ("The Resurgence redeployment
///   mechanic will now be disabled at the end of Circle 3", same notes,
///   Urzikstan Resurgence); its wait 15 s in the first circle, 39 s after,
///   and a squadmate's kill takes 7 s off it (Charlie Intel, "How to get
///   your teammates to respawn faster in Warzone Rebirth Island
///   Resurgence", 2021).
/// - Armor: three 50-point plates (Heavy Armor "buffing the total number of
///   health points from 150 to 200", Warzone Season 3 patch notes,
///   callofduty.com, 2024-04); a plate costs $300 at a buy station ("down
///   from $500", same notes).
///
/// - Heavy Armor: a Rebirth Island public event with "a low chance of
///   occurring at the start of each match", one more plate for the whole
///   match (Season 3 patch notes, callofduty.com, 2024-04).
/// - Redeploy waits shortened by kills, breaking armor and opening supply
///   boxes, and off at the fourth circle (Pro Game Guides, "How to respawn
///   teammates faster in Resurgence mode in Warzone 2"); the 7 s for a kill
///   above, the others ours.
/// - Contracts (Bounty, Scavenger, Recon, Supply Run with its 80% discount,
///   Most Wanted): see [`CONTRACTS`].
///
/// Not published that we found, so still our own: the circles' waits,
/// closing times and sizes, the loadout drop's, self-revive kit's, UAV's
/// and buyback's prices on Rebirth Island, the cash amounts, the event's
/// chance, the gas mask's life and the number of supply boxes.
pub const PHASES: &[CirclePhase] = &[
    CirclePhase {
        wait_s: 120,
        close_s: 90,
        end_radius: 0.62,
        gas_dps: 10,
        redeploy_s: Some(15),
    },
    CirclePhase {
        wait_s: 75,
        close_s: 75,
        end_radius: 0.40,
        gas_dps: 10,
        redeploy_s: Some(39),
    },
    CirclePhase {
        wait_s: 60,
        close_s: 60,
        end_radius: 0.24,
        gas_dps: 10,
        redeploy_s: Some(39),
    },
    CirclePhase {
        wait_s: 50,
        close_s: 50,
        end_radius: 0.13,
        gas_dps: 10,
        redeploy_s: None,
    },
    CirclePhase {
        wait_s: 40,
        close_s: 40,
        end_radius: 0.06,
        gas_dps: 10,
        redeploy_s: None,
    },
    CirclePhase {
        wait_s: 30,
        close_s: 35,
        end_radius: 0.02,
        gas_dps: 10,
        redeploy_s: None,
    },
    CirclePhase {
        wait_s: 20,
        close_s: 30,
        end_radius: 0.0,
        gas_dps: 10,
        redeploy_s: None,
    },
];

pub const BUY: &[BuyItem] = &[
    BuyItem {
        kind: BuyKind::LoadoutDrop,
        key: "loadout",
        label: "LOADOUT DROP",
        price: 6000,
    },
    BuyItem {
        kind: BuyKind::SquadBuyback,
        key: "buyback",
        label: "SQUAD BUYBACK",
        price: 4000,
    },
    BuyItem {
        kind: BuyKind::SelfRevive,
        key: "selfrevive",
        label: "SELF-REVIVE KIT",
        price: 4500,
    },
    BuyItem {
        kind: BuyKind::Uav,
        key: "uav",
        label: "PERSONAL UAV",
        price: 4000,
    },
    // ($300 a plate.)
    BuyItem {
        kind: BuyKind::ArmorPlates,
        key: "plates",
        label: "ARMOR PLATES (3)",
        price: 900,
    },
];

pub const LOOT: &[LootTier] = &[
    LootTier {
        key: "common",
        label: "COMMON",
        weight: 45,
        attachments: 0,
    },
    LootTier {
        key: "uncommon",
        label: "UNCOMMON",
        weight: 28,
        attachments: 1,
    },
    LootTier {
        key: "rare",
        label: "RARE",
        weight: 16,
        attachments: 2,
    },
    LootTier {
        key: "epic",
        label: "EPIC",
        weight: 8,
        attachments: 3,
    },
    LootTier {
        key: "legendary",
        label: "LEGENDARY",
        weight: 3,
        attachments: 4,
    },
];

/// Contracts as Warzone's own guide describes them ("Contracts and In-Game
/// Events", callofduty.com strategy guide); the payouts and times are not
/// published ("a few minutes", "a few thousand" in cash), so ours.
pub const CONTRACTS: &[Contract] = &[
    Contract {
        kind: ContractKind::Bounty,
        key: "bounty",
        label: "BOUNTY",
        reward: 1500,
        time_s: 90,
        count: 2,
    },
    Contract {
        kind: ContractKind::Scavenger,
        key: "scavenger",
        label: "SCAVENGER",
        reward: 1200,
        time_s: 0,
        count: 2,
    },
    Contract {
        kind: ContractKind::Recon,
        key: "recon",
        label: "RECON",
        reward: 1000,
        time_s: 0,
        count: 1,
    },
    Contract {
        kind: ContractKind::SupplyRun,
        key: "supplyrun",
        label: "SUPPLY RUN",
        reward: 800,
        time_s: 60,
        count: 1,
    },
    Contract {
        kind: ContractKind::MostWanted,
        key: "mostwanted",
        label: "MOST WANTED",
        reward: 3000,
        time_s: 90,
        count: 1,
    },
];

pub const REBIRTH: Rules = Rules {
    squad_size: 4,
    infil_s: 15,
    health: 100,
    plate_hp: 50,
    plate_slots: 3,
    plates_carried_max: 8,
    plate_insert_ms: 1200,
    spawn_plates: 1,
    start_cash: 0,
    kill_cash: 400,
    kill_redeploy_cut_s: 7,
    cash_stacks: &[100, 200, 500, 1000],
    buy_station_count: 3,
    uav_s: 30,
    phases: PHASES,
    buy: BUY,
    loot: LOOT,
    loot_spots: 48,
    armor_break_cut_s: 2,
    cache_cut_s: 2,
    bounty_cut_s: 10,
    caches: 16,
    gas_mask_s: 20,
    heavy_armor_pct: 10,
    contracts: CONTRACTS,
    supply_run_discount_pct: 80,
    bounty_survive_cash: 1000,
    recon_ms: 30000,
};

/// When a phase runs, in ms from the match start: its wait begins, its
/// circle starts closing, it is closed.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct PhaseWindow {
    pub index: usize,
    pub wait_from_ms: u32,
    pub close_from_ms: u32,
    pub closed_ms: u32,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum Moment {
    Infil,
    Waiting { phase: usize, left_ms: u32 },
    Closing { phase: usize, t: f32 },
    Closed,
}

impl Rules {
    pub const fn with_squad_size(mut self, n: u8) -> Self {
        self.squad_size = if n == 0 {
            1
        } else if n > MAX_SQUAD_SIZE {
            MAX_SQUAD_SIZE
        } else {
            n
        };
        self
    }

    pub fn windows(&self) -> impl Iterator<Item = PhaseWindow> + '_ {
        let mut at = u32::from(self.infil_s) * 1000;
        self.phases.iter().enumerate().map(move |(index, p)| {
            let wait_from_ms = at;
            let close_from_ms = wait_from_ms + u32::from(p.wait_s) * 1000;
            let closed_ms = close_from_ms + u32::from(p.close_s) * 1000;
            at = closed_ms;
            PhaseWindow {
                index,
                wait_from_ms,
                close_from_ms,
                closed_ms,
            }
        })
    }

    pub fn moment(&self, ms: u32) -> Moment {
        if ms < u32::from(self.infil_s) * 1000 {
            return Moment::Infil;
        }
        for w in self.windows() {
            if ms < w.close_from_ms {
                return Moment::Waiting {
                    phase: w.index,
                    left_ms: w.close_from_ms - ms,
                };
            }
            if ms < w.closed_ms {
                let t =
                    (ms - w.close_from_ms) as f32 / (w.closed_ms - w.close_from_ms).max(1) as f32;
                return Moment::Closing { phase: w.index, t };
            }
        }
        Moment::Closed
    }

    /// The circle's radius as a share of the first one's.
    pub fn radius_share(&self, ms: u32) -> f32 {
        let start = |i: usize| {
            if i == 0 {
                1.0
            } else {
                self.phases[i - 1].end_radius
            }
        };
        match self.moment(ms) {
            Moment::Infil => 1.0,
            Moment::Waiting { phase, .. } => start(phase),
            Moment::Closing { phase, t } => {
                let a = start(phase);
                a + (self.phases[phase].end_radius - a) * t
            }
            Moment::Closed => self.phases.last().map_or(1.0, |p| p.end_radius),
        }
    }

    /// The redeploy wait at `ms`, `None` once redeploys are off.
    pub fn redeploy_s(&self, ms: u32) -> Option<u16> {
        match self.moment(ms) {
            Moment::Infil => self.phases.first().and_then(|p| p.redeploy_s),
            Moment::Waiting { phase, .. } | Moment::Closing { phase, .. } => {
                self.phases[phase].redeploy_s
            }
            Moment::Closed => None,
        }
    }

    pub fn gas_dps(&self, ms: u32) -> u16 {
        match self.moment(ms) {
            Moment::Infil => 0,
            Moment::Waiting { phase, .. } | Moment::Closing { phase, .. } => {
                self.phases[phase].gas_dps
            }
            Moment::Closed => self.phases.last().map_or(0, |p| p.gas_dps),
        }
    }

    pub fn buy_item(&self, kind: BuyKind) -> Option<&BuyItem> {
        self.buy.iter().find(|b| b.kind == kind)
    }

    /// The loot tier a roll in `0..total weight` lands on.
    pub fn loot_tier(&self, roll: u32) -> Option<&LootTier> {
        let mut acc = 0u32;
        for t in self.loot {
            acc += u32::from(t.weight);
            if roll < acc {
                return Some(t);
            }
        }
        self.loot.last()
    }

    pub fn loot_weight_total(&self) -> u32 {
        self.loot.iter().map(|t| u32::from(t.weight)).sum()
    }
}

/// Squads still in it: a squad is out once none of its players is alive or
/// waiting on a redeploy. `alive_or_pending[squad]` counts those players.
pub fn squads_standing(alive_or_pending: &[u8]) -> usize {
    alive_or_pending.iter().filter(|&&n| n > 0).count()
}

/// One match sound: the event the match script plays it for, Modern
/// Warfare III's aliases for it (one picked at random each time it plays),
/// what follows it, and Black Ops II's stand-in for an install without
/// that game ("" for none).
///
/// An alias is its name where the name is known (the music, and the
/// sounds whose names were found in the zones or by `jup_alias_brute`),
/// else `#<hash>`: the hashed announcer lines of `eng_ingame_br` and
/// `eng_mp_jup_escape5`, sorted into events by what they say (transcribed
/// from the install's own streams, see docs/JUP.md), and Rebirth Island's
/// ambience beds of `mp_jup_escape5` (its long stereo `wz23` streams).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct SoundRoute {
    pub key: &'static str,
    pub jup: &'static [&'static str],
    /// The route played next and after how many ms (music: an intro into
    /// its main part; a loop into itself).
    pub then: Option<(&'static str, u32)>,
    pub t6: &'static str,
}

pub fn sound(key: &str) -> Option<&'static SoundRoute> {
    SOUNDS.iter().find(|r| r.key == key)
}

pub const SOUNDS: &[SoundRoute] = &[
    // before the drop
    SoundRoute {
        key: "music_prelobby",
        jup: &["mus_mx_mp_br_ri_prelobby_outro_intro"],
        then: None,
        t6: "",
    },
    // the drop: its intro, then its main part
    SoundRoute {
        key: "music_infil",
        jup: &["mus_mx_mp_br_ri_infil_1_intro"],
        then: Some(("music_infil_main", 32500)),
        t6: "",
    },
    // the drop's main part, then its end
    SoundRoute {
        key: "music_infil_main",
        jup: &["mus_mx_mp_br_ri_infil_1_main_0"],
        then: Some(("music_infil_exit", 16000)),
        t6: "",
    },
    // the drop's music ends
    SoundRoute {
        key: "music_infil_exit",
        jup: &["mus_mx_mp_br_ri_infil_1_exit"],
        then: None,
        t6: "",
    },
    // a circle closes (early)
    SoundRoute {
        key: "music_circle_low",
        jup: &[
            "mus_mx_mp_br_ri_circle_close_low_1_intro",
            "mus_mx_mp_br_ri_circle_close_low_2_intro",
            "mus_mx_mp_br_ri_circle_close_low_3_intro",
            "mus_mx_mp_br_ri_circle_close_low_4_intro",
        ],
        then: None,
        t6: "",
    },
    // a circle closes (mid match)
    SoundRoute {
        key: "music_circle_med",
        jup: &[
            "mus_mx_mp_br_ri_circle_close_med_1_intro",
            "mus_mx_mp_br_ri_circle_close_med_2_intro",
            "mus_mx_mp_br_ri_circle_close_med_3_intro",
            "mus_mx_mp_br_ri_circle_close_med_4_intro",
        ],
        then: None,
        t6: "",
    },
    // a circle closes (late)
    SoundRoute {
        key: "music_circle_high",
        jup: &[
            "mus_mx_mp_br_ri_circle_close_high_1_intro",
            "mus_mx_mp_br_ri_circle_close_high_2_intro",
            "mus_mx_mp_br_ri_circle_close_high_3_intro",
            "mus_mx_mp_br_ri_circle_close_high_4_intro",
        ],
        then: None,
        t6: "",
    },
    // the last circle closes: its intro, then its loop
    SoundRoute {
        key: "music_circle_final",
        jup: &["mus_mx_mp_br_ri_circle_close_final_loop_1_intro"],
        then: Some(("music_circle_final_main", 5330)),
        t6: "mus_time_running_out",
    },
    // the last circle's loop (again and again)
    SoundRoute {
        key: "music_circle_final_main",
        jup: &["mus_mx_mp_br_ri_circle_close_final_loop_1_main_0"],
        then: Some(("music_circle_final_main", 21340)),
        t6: "",
    },
    // the last circle's loop ends
    SoundRoute {
        key: "music_circle_final_exit",
        jup: &["mus_mx_mp_br_ri_circle_close_final_loop_1_exit"],
        then: None,
        t6: "",
    },
    // the last circle has closed: the second loop
    SoundRoute {
        key: "music_circle_last",
        jup: &["mus_mx_mp_br_ri_circle_close_final_loop_2_intro"],
        then: Some(("music_circle_last_main", 14240)),
        t6: "",
    },
    // the second loop (again and again)
    SoundRoute {
        key: "music_circle_last_main",
        jup: &["mus_mx_mp_br_ri_circle_close_final_loop_2_main_0"],
        then: Some(("music_circle_last_main", 32540)),
        t6: "",
    },
    // the second loop ends
    SoundRoute {
        key: "music_circle_last_exit",
        jup: &["mus_mx_mp_br_ri_circle_close_final_loop_2_exit"],
        then: None,
        t6: "",
    },
    // the winners' exfil: its intro, then its main part
    SoundRoute {
        key: "music_exfil",
        jup: &["mus_mx_mp_br_ri_exfil_1_intro"],
        then: Some(("music_exfil_main", 36120)),
        t6: "mus_victory",
    },
    // the exfil's main part
    SoundRoute {
        key: "music_exfil_main",
        jup: &["mus_mx_mp_br_ri_exfil_1_main_0"],
        then: Some(("music_exfil_end", 9800)),
        t6: "",
    },
    // the exfil's end
    SoundRoute {
        key: "music_exfil_end",
        jup: &["mus_mx_mp_br_ri_exfil_1_exit"],
        then: None,
        t6: "",
    },
    // the drop plane goes over
    SoundRoute {
        key: "infil_flyby",
        jup: &["veh_br_c17_flyby", "mp_slam_c130"],
        then: None,
        t6: "",
    },
    // Rebirth's infil strikes (lighthouse, water tower)
    SoundRoute {
        key: "infil_strike",
        jup: &[
            "cine_escape5_infil_explo_lighthouse",
            "cine_escape5_infil_explo_watertower",
            "cine_escape5_infil_missile",
        ],
        then: None,
        t6: "",
    },
    // the island's ambience beds (Rebirth Island's zone, ~90 s each, one after another)
    SoundRoute {
        key: "ambience",
        jup: &[
            "#0b85df63711b6c25",
            "#1740cda558b73462",
            "#1c3c12510cdd12f5",
            "#1e711604b7b87bd4",
            "#2a16fdc5cae19fb4",
            "#2e87cfa90f7bc594",
            "#3068371160a34e15",
            "#34ac32fcb1d68ba1",
            "#48fcdb304b39437b",
            "#519dd91b00264af4",
            "#5a35711feeb25069",
            "#638efce63f506f42",
            "#73a6206f4062bd69",
            "#73c4af10805ea64b",
            "#74e9e33c5bffd0a5",
            "#7afeda88f3fa4fe0",
            "#84c0753396b10c61",
            "#894345bfdeb97ed9",
            "#8a9aa15638ab09d1",
            "#8b744b5fa97d8cf7",
            "#9601a8aafd1072c2",
            "#9be75829a4aefd9f",
            "#a00417eee4d578ef",
            "#a4ef7104de2bc725",
            "#a810e6cc08333dc0",
            "#af6b82978df810c3",
            "#b9c4b790b784c4aa",
            "#be3b21a8b50ad091",
            "#c06dd8c8cadd4482",
            "#c5a3217fbcb4f5bd",
            "#cfb26cde6bee574f",
            "#e0b5d056c6cb9709",
            "#e11bb8a331558775",
            "#e24ecdd6dd97021b",
            "#e61fb0ff317787a2",
            "#ea1bfdc53f4f89a2",
            "#f3bf37164140cec5",
            "#fcf4b7eff95ba28a",
        ],
        then: Some(("ambience", 90000)),
        t6: "",
    },
    // the island's emitter loops, each on a spot of the map
    SoundRoute {
        key: "amb_loops",
        jup: &[
            "emt_escape5_air_steam_lp",
            "emt_escape5_big_fan_lp",
            "emt_escape5_generator_lg_lp",
            "emt_escape5_steam_lp",
            "emt_escape5_steam_lp_01",
            "emt_escape5_steam_lp_02",
            "emt_escape5_steam_lp_03",
            "emt_escape5_vent_large_lp",
            "emt_escape5_vent_med_lp",
            "emt_escape5_vent_small_lp",
            "emt_escape5_water_pipe_lp",
            "emt_escape5_fan_industrial_lg",
            "emt_seagull_circling_sfx",
            "emt_distant_thunder",
            "emt_escape5_fire_medium",
            "emt_escape5_fire_small",
            "emt_escape5_rock_debris",
            "emt_escape5_tv_static",
        ],
        then: None,
        t6: "",
    },
    // first spawn
    SoundRoute {
        key: "match_start",
        jup: &[],
        then: None,
        t6: "mpl_hit_alert",
    },
    // a circle starts closing
    SoundRoute {
        key: "circle_collapsing",
        jup: &["br_circle_closing"],
        then: None,
        t6: "mpl_sd_bomb_planted",
    },
    // the last seconds before a circle closes
    SoundRoute {
        key: "circle_tick",
        jup: &["uin_iw9_gulag_countdown_timer"],
        then: None,
        t6: "mpl_ui_timer_countdown",
    },
    // a circle has closed
    SoundRoute {
        key: "circle_closed",
        jup: &[],
        then: None,
        t6: "",
    },
    // the last circle, or redeploys off
    SoundRoute {
        key: "final_circle",
        jup: &[],
        then: None,
        t6: "",
    },
    // into the gas
    SoundRoute {
        key: "gas_enter",
        jup: &[],
        then: None,
        t6: "",
    },
    // a second of gas
    SoundRoute {
        key: "gas_tick",
        jup: &[],
        then: None,
        t6: "mpl_hit_alert_low",
    },
    // near the gas wall
    SoundRoute {
        key: "gas_wall",
        jup: &["br_circle_proximity_lp"],
        then: None,
        t6: "",
    },
    // gas mask on
    SoundRoute {
        key: "gas_mask_on",
        jup: &["br_gas_mask_on_plr"],
        then: None,
        t6: "",
    },
    // gas mask off
    SoundRoute {
        key: "gas_mask_off",
        jup: &["br_gas_mask_off_plr"],
        then: None,
        t6: "",
    },
    // gas mask broken
    SoundRoute {
        key: "gas_mask_break",
        jup: &["br_gas_mask_break_plr"],
        then: None,
        t6: "",
    },
    // a gas mask picked up
    SoundRoute {
        key: "mask_pickup",
        jup: &["br_gas_mask_on_plr"],
        then: None,
        t6: "uin_alert_slideout",
    },
    // the redeploy's last seconds
    SoundRoute {
        key: "redeploy_tick",
        jup: &["uin_iw9_gulag_countdown_timer"],
        then: None,
        t6: "mpl_ui_timer_countdown",
    },
    // back in
    SoundRoute {
        key: "redeployed",
        jup: &["br_splash_redeploy"],
        then: None,
        t6: "mpl_killstreak_radar",
    },
    // a redeploy's flare
    SoundRoute {
        key: "redeploy_flare",
        jup: &[
            "br_obj_flare_launch",
            "br_obj_flare_launch_dist",
            "br_obj_flare_burst",
            "br_obj_flare_revive_burst",
            "eqp_flaregun_flare_burst",
        ],
        then: None,
        t6: "",
    },
    // dead with redeploys off
    SoundRoute {
        key: "redeploy_disabled",
        jup: &[],
        then: None,
        t6: "",
    },
    // a squadmate died
    SoundRoute {
        key: "squadmate_down",
        jup: &[],
        then: None,
        t6: "",
    },
    // his squad is out
    SoundRoute {
        key: "squad_wiped",
        jup: &[],
        then: None,
        t6: "mpl_flagcapture_sting_enemy",
    },
    // another squad is out
    SoundRoute {
        key: "enemy_squad_wiped",
        jup: &[],
        then: None,
        t6: "mpl_flagcapture_sting_friend",
    },
    // squads left
    SoundRoute {
        key: "squads_remaining",
        jup: &[],
        then: None,
        t6: "uin_alert_slideout",
    },
    // three or two squads left
    SoundRoute {
        key: "top_squads",
        jup: &[],
        then: None,
        t6: "",
    },
    // his squad won
    SoundRoute {
        key: "victory",
        jup: &["uin_iw9_br_victory_slam", "uin_iw9_br_elite_victory_slam"],
        then: None,
        t6: "mus_victory",
    },
    // his squad lost (the after-action screen)
    SoundRoute {
        key: "defeat",
        jup: &["uin_iw9_br_aar_begin"],
        then: None,
        t6: "mus_loss",
    },
    // cash
    SoundRoute {
        key: "cash_pickup",
        jup: &["br_reward_mission_cash"],
        then: None,
        t6: "mpl_turret_alert",
    },
    // plates picked up
    SoundRoute {
        key: "plate_pickup",
        jup: &["perk_scavenger_pickup_plr"],
        then: None,
        t6: "uin_alert_slideout",
    },
    // a plate in
    SoundRoute {
        key: "plate_insert",
        jup: &[],
        then: None,
        t6: "fly_riotshield_impact_npc",
    },
    // plates full
    SoundRoute {
        key: "plates_full",
        jup: &[],
        then: None,
        t6: "",
    },
    // an enemy's plate broke
    SoundRoute {
        key: "armor_break",
        jup: &[],
        then: None,
        t6: "mpl_hit_alert_clip",
    },
    // the Heavy Armor event
    SoundRoute {
        key: "heavy_armor",
        jup: &["br_reward_mission_misc"],
        then: None,
        t6: "",
    },
    // a contract tablet put away
    SoundRoute {
        key: "biometric_close",
        jup: &["evt_biometric_scanner_close"],
        then: None,
        t6: "",
    },
    // a common or uncommon gun
    SoundRoute {
        key: "loot_common",
        jup: &[],
        then: None,
        t6: "",
    },
    // a rare gun
    SoundRoute {
        key: "loot_rare",
        jup: &[],
        then: None,
        t6: "",
    },
    // an epic or legendary gun
    SoundRoute {
        key: "loot_legendary",
        jup: &["br_legendary_loot_drop"],
        then: None,
        t6: "",
    },
    // a supply box opened
    SoundRoute {
        key: "cache_open",
        jup: &["br_loot_cache_chest_open"],
        then: None,
        t6: "",
    },
    // a Scavenger box opened
    SoundRoute {
        key: "supply_chest_open",
        jup: &["br_loot_supply_chest_open"],
        then: None,
        t6: "",
    },
    // at a buy station
    SoundRoute {
        key: "buy_open",
        jup: &[],
        then: None,
        t6: "uin_main_pause",
    },
    // bought
    SoundRoute {
        key: "purchase",
        jup: &[],
        then: None,
        t6: "mpl_turret_alert",
    },
    // not enough cash
    SoundRoute {
        key: "purchase_denied",
        jup: &[],
        then: None,
        t6: "uin_unlock_denied",
    },
    // a loadout drop called
    SoundRoute {
        key: "loadout_incoming",
        jup: &["iw9_carepackage_parachute_drop"],
        then: None,
        t6: "mpl_killstreak_supply",
    },
    // the loadout crate is down
    SoundRoute {
        key: "loadout_landed",
        jup: &[],
        then: None,
        t6: "mpl_supply_crate_land",
    },
    // his UAV
    SoundRoute {
        key: "uav_online",
        jup: &["ui_operation_scan_active_lr"],
        then: None,
        t6: "mpl_killstreak_radar",
    },
    // a squadmate bought back
    SoundRoute {
        key: "buyback",
        jup: &[],
        then: None,
        t6: "mpl_killstreak_radar",
    },
    // the self-revive kit used
    SoundRoute {
        key: "self_revive",
        jup: &["br_player_revived"],
        then: None,
        t6: "mpl_hit_alert",
    },
    // a contract taken (the tablet, then the splash)
    SoundRoute {
        key: "contract_take",
        jup: &["br_splash_mission_accept"],
        then: None,
        t6: "uin_alert_slideout",
    },
    // a contract tablet used
    SoundRoute {
        key: "contract_tablet",
        jup: &["evt_biometric_scanner_open"],
        then: None,
        t6: "",
    },
    // a contract paid
    SoundRoute {
        key: "contract_reward",
        jup: &["br_splash_mission_complete"],
        then: None,
        t6: "mpl_turret_alert",
    },
    // a contract failed
    SoundRoute {
        key: "contract_failed",
        jup: &["br_splash_mission_failure"],
        then: None,
        t6: "",
    },
    // Most Wanted taken
    SoundRoute {
        key: "contract_take_elite",
        jup: &["br_splash_legendary_mission_accept"],
        then: None,
        t6: "uin_alert_slideout",
    },
    // Most Wanted paid
    SoundRoute {
        key: "contract_reward_elite",
        jup: &["br_splash_legendary_mission_complete"],
        then: None,
        t6: "mpl_turret_alert",
    },
    // Most Wanted failed
    SoundRoute {
        key: "contract_failed_elite",
        jup: &["br_splash_legendary_mission_failure"],
        then: None,
        t6: "",
    },
    // a bounty's or most-wanted target killed
    SoundRoute {
        key: "target_down",
        jup: &["br_splash_vip_eliminated"],
        then: None,
        t6: "",
    },
    // a contract's cash
    SoundRoute {
        key: "reward_cash",
        jup: &["br_reward_mission_cash"],
        then: None,
        t6: "",
    },
    // Recon: uploading
    SoundRoute {
        key: "recon_upload",
        jup: &["dmz_upload_station_enter", "dmz_upload_station_close_lp"],
        then: None,
        t6: "",
    },
    // Recon: off the spot
    SoundRoute {
        key: "recon_exit",
        jup: &["dmz_upload_station_exit"],
        then: None,
        t6: "",
    },
    // his squad bought him back
    SoundRoute {
        key: "vo_bought_back",
        jup: &["#85027b95e65552e2", "#c112cf4c46cb2043"],
        then: None,
        t6: "",
    },
    // his squad killed its bounty
    SoundRoute {
        key: "vo_bounty_done",
        jup: &[
            "#08d32f447c418d4c",
            "#4a45e186afb3a143",
            "#77e3ba4687a372d7",
            "#8732229b77932c86",
            "#c4ce0f5b5d2ca678",
            "#d0af90679e5174e4",
            "#d648c40fb83639c2",
            "#d67f6bde305760c4",
        ],
        then: None,
        t6: "",
    },
    // a bounty ran out
    SoundRoute {
        key: "vo_bounty_failed",
        jup: &[
            "#05766629b88e507a",
            "#08d348447c41b7c7",
            "#871a259b777e9fbe",
            "#ab9d4344d7f4ea85",
            "#c4ce125b5d2cab91",
            "#d67550de304ede5e",
            "#f6f1f013411a80c8",
        ],
        then: None,
        t6: "",
    },
    // his squad is someone's bounty
    SoundRoute {
        key: "vo_bounty_on_you",
        jup: &[
            "#198849d5e945f099",
            "#2ea3be05fd329024",
            "#b311bd5011c9d02c",
            "#be35e03dafa506b0",
            "#c6ace6fecef658c7",
            "#d65a9fc273d3997b",
            "#e4d699af5e507ea0",
        ],
        then: None,
        t6: "",
    },
    // his squad took a bounty
    SoundRoute {
        key: "vo_bounty_start",
        jup: &[
            "#01f830de4902fee9",
            "#5d0a797d2fb59c88",
            "#6cf0a79cc0a7c602",
            "#6cf0a89cc0a7c7b5",
            "#6cf0a99cc0a7c968",
            "#8732329b779347b6",
            "#95704df165a79e31",
            "#959d3bf165ce6d86",
        ],
        then: None,
        t6: "",
    },
    // his squad lived through a bounty
    SoundRoute {
        key: "vo_bounty_survived",
        jup: &["#24bf31f446c8b83b", "#e7847b9a1f8aade0"],
        then: None,
        t6: "",
    },
    // at a buy station
    SoundRoute {
        key: "vo_buy_station",
        jup: &[
            "#31480d5ff4a7b594",
            "#3224dc6480c3b703",
            "#49bb447f173a96af",
            "#49c5637f17431fe1",
            "#5b55927f21472c30",
            "#74e21e7f2fa04e5b",
            "#8cdc2560eb4ee466",
            "#8cdc2660eb4ee619",
            "#9e066839886c00ba",
            "#ca74df7ecfe45912",
        ],
        then: None,
        t6: "",
    },
    // the circle is closing
    SoundRoute {
        key: "vo_circle_closing",
        jup: &[
            "#0d55a80bd0507cfe",
            "#170b835b7644e20c",
            "#356ba027f529a23c",
            "#3a34b1c0544f847e",
            "#72ba53d2206227b6",
        ],
        then: None,
        t6: "",
    },
    // the circle closes soon
    SoundRoute {
        key: "vo_circle_soon",
        jup: &["#0b1173c268db1b85"],
        then: None,
        t6: "",
    },
    // a contract done
    SoundRoute {
        key: "vo_contract_done",
        jup: &[
            "#03204bf179316b57",
            "#2488f68ac1a5d8cd",
            "#3b3019a3854db452",
            "#773a76c11bb993c7",
            "#a4a93c8788806015",
            "#c936d52f5cfefc15",
            "#ed4b9dfb48fba45c",
            "#f2726d26d163fe33",
            "#f62c0766360acef6",
        ],
        then: None,
        t6: "",
    },
    // a contract failed
    SoundRoute {
        key: "vo_contract_failed",
        jup: &[
            "#03203ff1793156f3",
            "#0a8c419a1e7f155c",
            "#0ab9db799e396ddb",
            "#0ede27c40742e401",
            "#229f35043382e352",
            "#b6bbf59710a6234b",
            "#ed4b9afb48fb9f43",
            "#ed6d88fb49186713",
            "#f3a5e36dbf7abe8f",
            "#f62ff366360e95d1",
            "#f66d146636427c02",
        ],
        then: None,
        t6: "",
    },
    // a timed contract is running out
    SoundRoute {
        key: "vo_contract_hurry",
        jup: &[
            "#4318330ec3f93468",
            "#ae96e96bdd1326f8",
            "#ed6d89fb491868c6",
            "#f62ffb66360ea369",
            "#f81761fb21d88463",
        ],
        then: None,
        t6: "",
    },
    // a contract taken
    SoundRoute {
        key: "vo_contract_take",
        jup: &[
            "#585ad871add2d02b",
            "#cdd0da3aa2c73ace",
            "#ed4b8dfb48fb892c",
            "#f53203b82aee41e9",
            "#f62bfa66360ab8df",
        ],
        then: None,
        t6: "",
    },
    // the number 0 (a countdown)
    SoundRoute {
        key: "vo_count_0",
        jup: &["#0094be760ee41cdd", "#9827d3dbed3f1baa"],
        then: None,
        t6: "",
    },
    // the number 1 (a countdown)
    SoundRoute {
        key: "vo_count_1",
        jup: &["#0094bd760ee41b2a", "#9827d4dbed3f1d5d"],
        then: None,
        t6: "",
    },
    // the number 10 (a countdown)
    SoundRoute {
        key: "vo_count_10",
        jup: &["#975bcbdbec91a7ae"],
        then: None,
        t6: "",
    },
    // the number 2 (a countdown)
    SoundRoute {
        key: "vo_count_2",
        jup: &["#9827d1dbed3f1844"],
        then: None,
        t6: "",
    },
    // the number 3 (a countdown)
    SoundRoute {
        key: "vo_count_3",
        jup: &["#0094bb760ee417c4", "#9827d2dbed3f19f7"],
        then: None,
        t6: "",
    },
    // the number 4 (a countdown)
    SoundRoute {
        key: "vo_count_4",
        jup: &["#0094ba760ee41611", "#9827cfdbed3f14de"],
        then: None,
        t6: "",
    },
    // the number 5 (a countdown)
    SoundRoute {
        key: "vo_count_5",
        jup: &["#0094b9760ee4145e", "#9827d0dbed3f1691"],
        then: None,
        t6: "",
    },
    // the number 6 (a countdown)
    SoundRoute {
        key: "vo_count_6",
        jup: &["#0094b8760ee412ab", "#9827cddbed3f1178"],
        then: None,
        t6: "",
    },
    // the number 7 (a countdown)
    SoundRoute {
        key: "vo_count_7",
        jup: &["#0094b7760ee410f8", "#9827cedbed3f132b"],
        then: None,
        t6: "",
    },
    // the number 8 (a countdown)
    SoundRoute {
        key: "vo_count_8",
        jup: &["#9827cbdbed3f0e12"],
        then: None,
        t6: "",
    },
    // the number 9 (a countdown)
    SoundRoute {
        key: "vo_count_9",
        jup: &["#9827ccdbed3f0fc5"],
        then: None,
        t6: "",
    },
    // his squad lost
    SoundRoute {
        key: "vo_defeat",
        jup: &[
            "#0a36b46d491e3e45",
            "#4691e412785f1df1",
            "#493154e1fd746b4a",
            "#62eb9db0ee8ac3a8",
            "#74a325e21619ff25",
            "#75bbf0edbbd3fc9b",
            "#bc84303fa26c8c55",
            "#dc308dd92481abc9",
            "#e4e1880c42828293",
            "#fd6b988d47c9ef8a",
        ],
        then: None,
        t6: "",
    },
    // an enemy UAV is up
    SoundRoute {
        key: "vo_enemy_uav",
        jup: &[
            "#02d51d9239bdd127",
            "#3060773a353ccaab",
            "#325ffb7a619f62e0",
            "#62fae5dc4f03185d",
            "#721418c4f50cb4ce",
            "#76e466ddac0ada77",
            "#b5f530cb949324ce",
            "#bc62c051e6d332a7",
        ],
        then: None,
        t6: "",
    },
    // the last circle
    SoundRoute {
        key: "vo_final_circle",
        jup: &[
            "#5db30efe6bd8fe09",
            "#6ab43ef5781bde83",
            "#c19ac5d1a4fdb431",
        ],
        then: None,
        t6: "",
    },
    // Heavy Armor
    SoundRoute {
        key: "vo_heavy_armor",
        jup: &["#23ca765e293353d1"],
        then: None,
        t6: "",
    },
    // he is in the gas
    SoundRoute {
        key: "vo_in_gas",
        jup: &["#956304b9a5b05fdf", "#c68ed099744d5b28"],
        then: None,
        t6: "",
    },
    // the drop
    SoundRoute {
        key: "vo_infil",
        jup: &[
            "#53f4a097daef8b40",
            "#53f4a197daef8cf3",
            "#53f4a297daef8ea6",
            "#72f3b2f053196f21",
            "#9191474a7af30714",
            "#9191484a7af308c7",
            "#b48c05ad5718a1a3",
            "#d489516df5b61c16",
            "#e203b224052738af",
            "#f561db7c3dcfae7e",
            "#fec492aec55afafd",
        ],
        then: None,
        t6: "",
    },
    // his kill (praise)
    SoundRoute {
        key: "vo_kill",
        jup: &[
            "#158e349a2db0f4a5",
            "#3583b9e8089370a5",
            "#42d50a5adf61b17e",
            "#42d5105adf61bbb0",
            "#4eda2108fdfce795",
            "#4fd6f9d736838420",
            "#517f1542f5252db9",
            "#51fe68b3d75eb52c",
            "#5a2436a8fa4fcb03",
            "#5ebd327ed521506f",
            "#60527670b0fb6cf4",
            "#69257096cd4f284a",
            "#7429438c37cf89bf",
            "#754cac1138407422",
            "#92118bc05f15ab7e",
            "#a98ccaf05d21530e",
            "#b66cfde6b6145709",
            "#c3008dcdf6d8a7cf",
            "#c35c6c497e441bfe",
            "#c7bf40656402805b",
            "#f1b21554fa9d41e8",
            "#fd2b0cf551b3954e",
        ],
        then: None,
        t6: "",
    },
    // on the ground
    SoundRoute {
        key: "vo_landed",
        jup: &[
            "#4c17a4aeb0498948",
            "#4c17a7aeb0498e61",
            "#6f2f195ca36d564c",
            "#6f2f1a5ca36d57ff",
            "#6f2f1b5ca36d59b2",
            "#ba8b32ddbc014ca8",
            "#c64c189a787cab50",
            "#ec8a81cda04419fa",
        ],
        then: None,
        t6: "",
    },
    // he is the last of his squad alive
    SoundRoute {
        key: "vo_last_alive",
        jup: &[
            "#209e91a13cbdc6af",
            "#359a16a87e7ef79d",
            "#39915dc7c89d1b98",
            "#531555cc0b4e29b9",
            "#5b60b1942d0ad99f",
            "#9901ec3f39e43332",
            "#a01667f48dc49e5f",
            "#a03930c9c01c12b1",
            "#a0d506e74581072e",
        ],
        then: None,
        t6: "",
    },
    // a loadout drop is on the way
    SoundRoute {
        key: "vo_loadout_incoming",
        jup: &[
            "#0f269fa7e3de2337",
            "#219deb3a41f464b0",
            "#5fee309c2e143c5a",
            "#6ca834b303eccd24",
            "#7e997b95bd7083ae",
            "#d68737953aa6d1f9",
            "#db4c0a74fedd5182",
            "#df9bca211c9b09c9",
            "#ea59685609ffd5ed",
        ],
        then: None,
        t6: "",
    },
    // the loadout crate is down
    SoundRoute {
        key: "vo_loadout_landed",
        jup: &["#bece3aab4b2741e1"],
        then: None,
        t6: "",
    },
    // a squadmate is down
    SoundRoute {
        key: "vo_mate_down",
        jup: &["#8cf9d8bd4413c7d6", "#b1ae590d23bbebf7"],
        then: None,
        t6: "",
    },
    // a squadmate is redeploying
    SoundRoute {
        key: "vo_mate_pending",
        jup: &["#bb3f5e3ad9dbfcc4"],
        then: None,
        t6: "",
    },
    // a squadmate is back
    SoundRoute {
        key: "vo_mate_redeployed",
        jup: &["#2c82ebc9107ef641", "#5589c49527a20eb1"],
        then: None,
        t6: "",
    },
    // the mode's name
    SoundRoute {
        key: "vo_mode_name",
        jup: &["#85038d3500619077", "#ecefce23fee9f747"],
        then: None,
        t6: "",
    },
    // 10 players left
    SoundRoute {
        key: "vo_players_left_10",
        jup: &["#4fcf3578fd34504b", "#c336e86ab9fca2c9"],
        then: None,
        t6: "",
    },
    // 25 players left
    SoundRoute {
        key: "vo_players_left_25",
        jup: &[
            "#2ef311ba0eafaba1",
            "#59bd43f38a076de7",
            "#b57e99e58c47937b",
            "#f4ba3161d45faa70",
        ],
        then: None,
        t6: "",
    },
    // 50 players left
    SoundRoute {
        key: "vo_players_left_50",
        jup: &["#264e9e9c5ad9ae17", "#c5ab28674bfce615"],
        then: None,
        t6: "",
    },
    // waiting for the drop
    SoundRoute {
        key: "vo_prelobby",
        jup: &[
            "#c5fea84b8729b489",
            "#d8c399fdfb41ff05",
            "#eedac1b8e64b682d",
            "#efa31ca679f0728b",
        ],
        then: None,
        t6: "",
    },
    // the drop begins
    SoundRoute {
        key: "vo_prelobby_over",
        jup: &["#864c16de4cecc2ac", "#f8ba959bd3fab459"],
        then: None,
        t6: "",
    },
    // the data is uploaded: the next circle is marked
    SoundRoute {
        key: "vo_recon_done",
        jup: &[
            "#9bf06da24a47b451",
            "#f0d8af5e4a191ee3",
            "#f64c816e09b1a3fe",
        ],
        then: None,
        t6: "",
    },
    // Recon: the upload stopped
    SoundRoute {
        key: "vo_recon_lost",
        jup: &["#e6142023a3ca7050"],
        then: None,
        t6: "",
    },
    // his squad took a Recon
    SoundRoute {
        key: "vo_recon_start",
        jup: &[
            "#6219dca075c616fa",
            "#6a24ea3c4609ba0d",
            "#93d32dd1a392451b",
            "#b0371510f5ad7ad9",
            "#bf7abf5ab2871fea",
            "#fb242ef0aa55a595",
        ],
        then: None,
        t6: "",
    },
    // Recon: uploading
    SoundRoute {
        key: "vo_recon_upload",
        jup: &[
            "#6a54ee3c4632e49b",
            "#936d39d1a33ba64d",
            "#bf14a35ab2303d24",
            "#fb2420f0aa558dcb",
        ],
        then: None,
        t6: "",
    },
    // redeploys end with this circle
    SoundRoute {
        key: "vo_redeploy_closing",
        jup: &[
            "#175d329496c94880",
            "#3f2988c988a14261",
            "#a9c4d786f954e139",
        ],
        then: None,
        t6: "",
    },
    // a squadmate's redeploy wait got shorter
    SoundRoute {
        key: "vo_redeploy_cut",
        jup: &["#08ca3a230faa0be3", "#4f7590a6ce708766"],
        then: None,
        t6: "",
    },
    // redeploys are off (this circle on)
    SoundRoute {
        key: "vo_redeploy_off",
        jup: &["#228972949dd3c6a5", "#3f297ec988a13163"],
        then: None,
        t6: "",
    },
    // dead: a redeploy is coming
    SoundRoute {
        key: "vo_redeploy_pending",
        jup: &[
            "#0e950452c1f1dcac",
            "#153a27a0484b76e4",
            "#17495d9496b8e887",
            "#5a33424f696cea7a",
            "#7f32a2031e2755fb",
            "#85466b95e68f047e",
            "#c13baf4c46edeef7",
            "#e83fecc95751b92f",
        ],
        then: None,
        t6: "",
    },
    // back in after a redeploy
    SoundRoute {
        key: "vo_redeployed",
        jup: &[
            "#05f5f45ae7d78c4b",
            "#20001fd90d554491",
            "#7371b9578f9ad36c",
            "#9d453dd18fef04c8",
            "#dc0b71d924625ff8",
        ],
        then: None,
        t6: "",
    },
    // Resurgence's rule, said on landing
    SoundRoute {
        key: "vo_rules",
        jup: &[
            "#223e86949d93fd27",
            "#85407c35009521b2",
            "#e835cac957492ae4",
            "#ecb9ce23febc67d7",
        ],
        then: None,
        t6: "",
    },
    // far outside the circle
    SoundRoute {
        key: "vo_safe_far",
        jup: &["#95631db9a5b08a5a", "#c68ec999744d4f43"],
        then: None,
        t6: "",
    },
    // the third supply box
    SoundRoute {
        key: "vo_scav_done",
        jup: &["#2a0cf6910f708b28", "#a1396c7522560c88"],
        then: None,
        t6: "",
    },
    // a supply box opened, the next marked
    SoundRoute {
        key: "vo_scav_next",
        jup: &[
            "#29c0d799435d9e63",
            "#3ed632de0cfdf018",
            "#63e23044fa0a424e",
            "#a83755aa24d482c5",
            "#b7778f8a85b4c459",
            "#e35de751f4b25bf9",
            "#e5e8bb5cc4e9ec30",
        ],
        then: None,
        t6: "",
    },
    // his squad took a Scavenger
    SoundRoute {
        key: "vo_scav_start",
        jup: &["#2a0ce6910f706ff8", "#a13963752255fd3d"],
        then: None,
        t6: "",
    },
    // his whole squad is inside the circle
    SoundRoute {
        key: "vo_squad_safe",
        jup: &["#956317b9a5b08028", "#c68ed199744d5cdb"],
        then: None,
        t6: "",
    },
    // 10 squads left
    SoundRoute {
        key: "vo_squads_left_10",
        jup: &["#4bc1e49606fa9429", "#6a1b729424930848"],
        then: None,
        t6: "",
    },
    // 2 squads left
    SoundRoute {
        key: "vo_squads_left_2",
        jup: &["#14563fa6ff66830a", "#56958913e080f160"],
        then: None,
        t6: "",
    },
    // 3 squads left
    SoundRoute {
        key: "vo_squads_left_3",
        jup: &["#dd2bb5c0cfaf0f79"],
        then: None,
        t6: "",
    },
    // 4 squads left
    SoundRoute {
        key: "vo_squads_left_4",
        jup: &["#039f1de149aa0bf9"],
        then: None,
        t6: "",
    },
    // 5 squads left
    SoundRoute {
        key: "vo_squads_left_5",
        jup: &["#d32bff6f57bb9e83"],
        then: None,
        t6: "",
    },
    // the squad made it to the buy station
    SoundRoute {
        key: "vo_supply_done",
        jup: &[
            "#0f9c20adbcb34c55",
            "#d72c6a71c447d8e1",
            "#d7556171c46aceaa",
        ],
        then: None,
        t6: "",
    },
    // a squadmate is still outside the circle
    SoundRoute {
        key: "vo_team_outside",
        jup: &["#95631bb9a5b086f4", "#c68ed299744d5e8e"],
        then: None,
        t6: "",
    },
    // his squad finished in the top three
    SoundRoute {
        key: "vo_third",
        jup: &["#00d2d624ae7764d9", "#83371fea398d4dd8"],
        then: None,
        t6: "",
    },
    // his UAV is up
    SoundRoute {
        key: "vo_uav_online",
        jup: &[
            "#046ff630f7096a72",
            "#2d5377602888332c",
            "#6425986c74a5f465",
            "#67e413dda433ff13",
            "#6e233bc461cb7051",
            "#6e234ac461cb89ce",
            "#720cf8c4f5065fc0",
            "#720d08c4f5067af0",
            "#76e462ddac0ad3ab",
            "#a87d8cdd378e7429",
            "#b65c76436af1a918",
        ],
        then: None,
        t6: "",
    },
    // his squad won
    SoundRoute {
        key: "vo_victory",
        jup: &[
            "#2f90a6492a670ed3",
            "#36eb2da6606b0ee8",
            "#662fc0f2367468db",
            "#be33446bd7d9032f",
            "#be33456bd7d904e2",
            "#c42ca8844eb0ba5a",
        ],
        then: None,
        t6: "",
    },
    // his squad lived through Most Wanted
    SoundRoute {
        key: "vo_wanted_done",
        jup: &[
            "#4a82cf86afe730cb",
            "#537143691f82a366",
            "#de44ab49c363be33",
        ],
        then: None,
        t6: "",
    },
    // another squad is Most Wanted
    SoundRoute {
        key: "vo_wanted_enemy",
        jup: &["#3eef40e98442a8a0"],
        then: None,
        t6: "",
    },
    // his squad's most-wanted man died
    SoundRoute {
        key: "vo_wanted_failed",
        jup: &["#535345691f69a93d", "#d06881679e1558c4"],
        then: None,
        t6: "",
    },
    // his squad killed a most-wanted enemy
    SoundRoute {
        key: "vo_wanted_killed",
        jup: &[
            "#53274e691f447d9d",
            "#a4a92d8788804698",
            "#d08083679e29ee0b",
        ],
        then: None,
        t6: "",
    },
    // his squadmate is Most Wanted
    SoundRoute {
        key: "vo_wanted_mate",
        jup: &[
            "#41bdd56836fc2943",
            "#4a82cb86afe729ff",
            "#537144691f82a519",
            "#cde0062d5e13150b",
        ],
        then: None,
        t6: "",
    },
    // he is Most Wanted
    SoundRoute {
        key: "vo_wanted_you",
        jup: &[
            "#a4a93b8788805e62",
            "#d08d89679e348299",
            "#de85bc49c39b720b",
            "#de85c049c39b78d7",
        ],
        then: None,
        t6: "",
    },
];

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn sound_routes_are_one_per_key_and_chain_to_routes() {
        for (i, r) in SOUNDS.iter().enumerate() {
            assert!(
                SOUNDS[i + 1..].iter().all(|o| o.key != r.key),
                "{} twice",
                r.key
            );
            if let Some((next, ms)) = r.then {
                assert!(sound(next).is_some(), "{} -> {next}", r.key);
                assert!(ms > 0, "{}", r.key);
            }
            for a in r.jup {
                assert!(!a.is_empty() && !a.contains(' '), "{}: `{a}`", r.key);
                if let Some(h) = a.strip_prefix('#') {
                    assert!(h.len() == 16 && u64::from_str_radix(h, 16).is_ok(), "{a}");
                }
            }
        }
    }

    #[test]
    fn redeploys_end_with_circle_three() {
        assert_eq!(REBIRTH.windows().count(), REBIRTH.phases.len());
        let first = REBIRTH.windows().next().unwrap();
        let fourth = REBIRTH.windows().nth(3).unwrap();
        assert_eq!(first.wait_from_ms, u32::from(REBIRTH.infil_s) * 1000);
        assert_eq!(REBIRTH.redeploy_s(first.wait_from_ms), Some(15));
        assert!(REBIRTH.redeploy_s(fourth.wait_from_ms).is_none());
    }
}
