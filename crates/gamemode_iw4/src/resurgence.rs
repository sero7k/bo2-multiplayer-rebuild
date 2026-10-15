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
/// Not published that we found, so still our own: the circles' waits,
/// closing times and sizes, the loadout drop's, self-revive kit's, UAV's
/// and buyback's prices on Rebirth Island, and the cash amounts.
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

/// What the match's sounds are, by what happens.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum BrSound {
    MatchStart,
    InfilMusic,
    CircleCollapsing,
    CircleClosed,
    FinalCircle,
    GasTick,
    GasEnter,
    GasLoop,
    RedeployCountdown,
    Redeployed,
    RedeployDisabled,
    SquadmateDown,
    SquadWiped,
    EnemySquadWiped,
    SquadsRemaining,
    TopSquads,
    Victory,
    Defeat,
    CashPickup,
    PlatePickup,
    PlateInsert,
    PlatesFull,
    ArmorBreak,
    BuyStationOpen,
    Purchase,
    PurchaseDenied,
    LoadoutDropIncoming,
    LoadoutDropLanded,
    UavOnline,
    BuybackComplete,
    SelfRevive,
    LootCommon,
    LootRare,
    LootLegendary,
    IslandAmbience,
    PrelobbyMusic,
    CircleMusicLow,
    CircleMusicMed,
    CircleMusicHigh,
    CircleMusicFinal,
    ExfilMusic,
}

/// One match sound: our event, its Modern Warfare III alias (played once
/// that game's sounds can be read) and Black Ops II's stand-in until then
/// (empty: silent).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct SoundRoute {
    pub event: BrSound,
    pub key: &'static str,
    pub jup: &'static str,
    pub t6: &'static str,
}

const fn route(
    event: BrSound,
    key: &'static str,
    jup: &'static str,
    t6: &'static str,
) -> SoundRoute {
    SoundRoute {
        event,
        key,
        jup,
        t6,
    }
}

pub const SOUNDS: &[SoundRoute] = &[
    route(BrSound::MatchStart, "match_start", "", "mpl_hit_alert"),
    route(
        BrSound::InfilMusic,
        "infil_music",
        "mus_mx_mp_br_ri_infil_1_intro",
        "",
    ),
    route(
        BrSound::CircleCollapsing,
        "circle_collapsing",
        "",
        "mpl_sd_bomb_planted",
    ),
    route(BrSound::CircleClosed, "circle_closed", "", ""),
    route(
        BrSound::FinalCircle,
        "final_circle",
        "",
        "mus_time_running_out",
    ),
    route(BrSound::GasTick, "gas_tick", "", "mpl_hit_alert_low"),
    route(BrSound::GasEnter, "gas_enter", "", ""),
    route(BrSound::GasLoop, "gas_loop", "", ""),
    route(
        BrSound::RedeployCountdown,
        "redeploy_tick",
        "",
        "mpl_ui_timer_countdown",
    ),
    route(
        BrSound::Redeployed,
        "redeployed",
        "",
        "mpl_killstreak_radar",
    ),
    route(BrSound::RedeployDisabled, "redeploy_disabled", "", ""),
    route(BrSound::SquadmateDown, "squadmate_down", "", ""),
    route(
        BrSound::SquadWiped,
        "squad_wiped",
        "",
        "mpl_flagcapture_sting_enemy",
    ),
    route(
        BrSound::EnemySquadWiped,
        "enemy_squad_wiped",
        "",
        "mpl_flagcapture_sting_friend",
    ),
    route(
        BrSound::SquadsRemaining,
        "squads_remaining",
        "",
        "uin_alert_slideout",
    ),
    route(BrSound::TopSquads, "top_squads", "", ""),
    route(BrSound::Victory, "victory", "", "mus_victory"),
    route(BrSound::Defeat, "defeat", "", "mus_loss"),
    route(BrSound::CashPickup, "cash_pickup", "", "mpl_turret_alert"),
    route(
        BrSound::PlatePickup,
        "plate_pickup",
        "",
        "uin_alert_slideout",
    ),
    route(
        BrSound::PlateInsert,
        "plate_insert",
        "",
        "fly_riotshield_impact_npc",
    ),
    route(BrSound::PlatesFull, "plates_full", "", ""),
    route(BrSound::ArmorBreak, "armor_break", "", "mpl_hit_alert_clip"),
    route(BrSound::BuyStationOpen, "buy_open", "", "uin_main_pause"),
    route(BrSound::Purchase, "purchase", "", "mpl_turret_alert"),
    route(
        BrSound::PurchaseDenied,
        "purchase_denied",
        "",
        "uin_unlock_denied",
    ),
    route(
        BrSound::LoadoutDropIncoming,
        "loadout_incoming",
        "",
        "mpl_killstreak_supply",
    ),
    route(
        BrSound::LoadoutDropLanded,
        "loadout_landed",
        "",
        "mpl_supply_crate_land",
    ),
    route(BrSound::UavOnline, "uav_online", "", "mpl_killstreak_radar"),
    route(
        BrSound::BuybackComplete,
        "buyback",
        "",
        "mpl_killstreak_radar",
    ),
    route(BrSound::SelfRevive, "self_revive", "", "mpl_hit_alert"),
    route(BrSound::LootCommon, "loot_common", "", ""),
    route(BrSound::LootRare, "loot_rare", "", ""),
    route(BrSound::LootLegendary, "loot_legendary", "", ""),
    route(BrSound::IslandAmbience, "ambience", "", ""),
    route(
        BrSound::PrelobbyMusic,
        "music_prelobby",
        "mus_mx_mp_br_ri_prelobby_outro_intro",
        "",
    ),
    route(
        BrSound::CircleMusicLow,
        "music_circle_low",
        "mus_mx_mp_br_ri_circle_close_low_1_intro",
        "",
    ),
    route(
        BrSound::CircleMusicMed,
        "music_circle_med",
        "mus_mx_mp_br_ri_circle_close_med_1_intro",
        "",
    ),
    route(
        BrSound::CircleMusicHigh,
        "music_circle_high",
        "mus_mx_mp_br_ri_circle_close_high_1_intro",
        "",
    ),
    route(
        BrSound::CircleMusicFinal,
        "music_circle_final",
        "mus_mx_mp_br_ri_circle_close_final_loop_1_intro",
        "mus_time_running_out",
    ),
    route(
        BrSound::ExfilMusic,
        "music_exfil",
        "mus_mx_mp_br_ri_exfil_1_intro",
        "mus_victory",
    ),
];

pub fn sound(event: BrSound) -> Option<&'static SoundRoute> {
    SOUNDS.iter().find(|r| r.event == event)
}
