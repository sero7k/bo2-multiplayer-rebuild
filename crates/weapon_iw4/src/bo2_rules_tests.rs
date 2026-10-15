//! Black Ops II's weapon rules, driven the way the match drives them: one
//! command per 60 fps frame, the player's guns each keeping their own ammo
//! and "needs rechamber" mark (`sim::combat` does the same with
//! `ClientMatchState::rechamber_weapons`). The input timings are the ones
//! measured frame by frame from the original game's footage.

use std::vec::Vec;

use crate::offhand::with_test_rules;
use crate::tick::{
    BUTTON_ATTACK, BUTTON_RELOAD, WeaponCmd, WeaponCombatFacts, WeaponHandState, WeaponTickEvent,
    weapon_ordinary,
};
use crate::weaponstate::{FireType, WeaponState};

const SHOTGUN: u32 = 1;
const SNIPER: u32 = 2;
const SMG: u32 = 3;
const SMG_BURST: u32 = 4;

/// Remington 870 MCS: pump (a rechamber after every shot).
fn shotgun() -> WeaponCombatFacts {
    WeaponCombatFacts {
        fire_type: FireType::SingleShot as i32,
        fire_time_ms: 100,
        bolt_action: true,
        rechamber_time_ms: 700,
        rechamber_bolt_time_ms: 700,
        rechamber_bolt_delay_ms: 300,
        drop_time_ms: 250,
        raise_time_ms: 450,
        quick_drop_time_ms: 200,
        quick_raise_time_ms: 300,
        clip_size: 8,
        start_ammo: 32,
        ..WeaponCombatFacts::default()
    }
}

/// DSR 50: bolt-action sniper.
fn sniper() -> WeaponCombatFacts {
    WeaponCombatFacts {
        fire_type: FireType::SingleShot as i32,
        fire_time_ms: 150,
        bolt_action: true,
        rechamber_time_ms: 1100,
        rechamber_bolt_time_ms: 1100,
        rechamber_bolt_delay_ms: 500,
        drop_time_ms: 300,
        raise_time_ms: 450,
        quick_drop_time_ms: 250,
        quick_raise_time_ms: 300,
        clip_size: 5,
        start_ammo: 30,
        ..WeaponCombatFacts::default()
    }
}

/// MP7 with Select Fire: automatic, and its burst alternate.
fn smg(fire_type: FireType) -> WeaponCombatFacts {
    WeaponCombatFacts {
        fire_type: fire_type as i32,
        fire_time_ms: 75,
        burst_cooldown_ms: 200,
        drop_time_ms: 250,
        raise_time_ms: 300,
        quick_drop_time_ms: 200,
        quick_raise_time_ms: 250,
        clip_size: 40,
        start_ammo: 160,
        ..WeaponCombatFacts::default()
    }
}

fn facts_for(weapon: u32) -> WeaponCombatFacts {
    match weapon {
        SHOTGUN => shotgun(),
        SNIPER => sniper(),
        SMG => smg(FireType::FullAuto),
        SMG_BURST => smg(FireType::BurstFire3),
        _ => WeaponCombatFacts::default(),
    }
}

struct Gun {
    weapon: u32,
    clip: i32,
    stock: i32,
    needs_rechamber: bool,
}

/// A player holding some guns, stepped one 60 fps frame at a time.
struct Player {
    hand: WeaponHandState,
    guns: Vec<Gun>,
    selected: u32,
    buttons: u32,
    old_buttons: u32,
    frame: u32,
    log: Vec<(u32, WeaponTickEvent)>,
}

impl Player {
    fn new(held: u32, others: &[u32]) -> Self {
        let mut guns = Vec::new();
        for &w in core::iter::once(&held).chain(others) {
            let f = facts_for(w);
            guns.push(Gun { weapon: w, clip: f.clip_size, stock: f.start_ammo - f.clip_size, needs_rechamber: false });
        }
        let f = facts_for(held);
        Self {
            hand: WeaponHandState {
                weapon: held,
                weaponstate: WeaponState::Ready as i32,
                clip: f.clip_size,
                stock: f.start_ammo - f.clip_size,
                ..WeaponHandState::default()
            },
            guns,
            selected: held,
            buttons: 0,
            old_buttons: 0,
            frame: 0,
            log: Vec::new(),
        }
    }

    fn gun(&mut self, weapon: u32) -> &mut Gun {
        self.guns.iter_mut().find(|g| g.weapon == weapon).unwrap()
    }

    /// One frame: 16 or 17 ms, like 60 fps.
    fn step(&mut self) -> Option<WeaponTickEvent> {
        let started = self.hand.weapon;
        // The match loads the held gun's own ammo and mark.
        let (clip, stock, mark) = {
            let g = self.gun(started);
            (g.clip, g.stock, g.needs_rechamber)
        };
        self.hand.clip = clip;
        self.hand.stock = stock;
        self.hand.rechamber_pending = mark;
        let facts = facts_for(started);
        let target = facts_for(self.selected);
        let target_clip = self.gun(self.selected).clip;
        let mut cmd = WeaponCmd {
            msec: if self.frame % 3 == 2 { 17 } else { 16 },
            buttons: self.buttons,
            old_buttons: self.old_buttons,
            cmd_weapon: self.selected as u16,
            cmd_weapon_owned: true,
            switch_raise_time_ms: target.raise_time_ms,
            switch_quick_raise_time_ms: target.quick_raise_time_ms,
            switch_raise_empty: target.empty_raise_time_ms > 0 && target_clip <= 0,
            ..WeaponCmd::default()
        };
        let ev = weapon_ordinary(&mut self.hand, &facts, &mut cmd);
        // ... and stores them back with the gun the frame began with.
        let (c, s, m) = (self.hand.clip, self.hand.stock, self.hand.rechamber_pending);
        let g = self.gun(started);
        g.clip = c;
        g.stock = s;
        g.needs_rechamber = m;
        self.old_buttons = self.buttons;
        if let Some(e) = ev {
            self.log.push((self.frame, e));
        }
        self.frame += 1;
        ev
    }

    fn frames(&mut self, n: u32) {
        for _ in 0..n {
            self.step();
        }
    }

    fn state(&self) -> WeaponState {
        WeaponState::from_i32(self.hand.weaponstate).unwrap()
    }

    fn shots_since(&self, frame: u32) -> usize {
        self.log
            .iter()
            .filter(|(f, e)| *f >= frame && matches!(e, WeaponTickEvent::ShotAccepted { .. }))
            .count()
    }

    /// Press the trigger for one frame.
    fn tap_fire(&mut self) {
        self.buttons |= BUTTON_ATTACK;
        self.step();
        self.buttons &= !BUTTON_ATTACK;
    }

    fn swap_to(&mut self, weapon: u32) {
        self.selected = weapon;
    }

    fn run_until(&mut self, max: u32, mut done: impl FnMut(&Self) -> bool) -> u32 {
        for n in 0..max {
            if done(self) {
                return n;
            }
            self.step();
        }
        panic!("not reached in {max} frames: {:?} t={} log {:?}", self.state(), self.hand.weapon_time, self.log);
    }
}

/// Footage: the 870 fires, swap 4 frames later; the pump never plays
/// before the drop, and it plays when the 870 next comes up.
#[test]
fn a_pump_skipped_by_a_quick_swap_plays_on_the_next_draw() {
    with_test_rules(true, || {
        let mut p = Player::new(SHOTGUN, &[SNIPER]);
        p.tap_fire();
        assert_eq!(p.gun(SHOTGUN).clip, 7);
        p.frames(3);
        p.swap_to(SNIPER);
        p.run_until(30, |p| p.state() == WeaponState::Dropping);
        assert_eq!(p.hand.weapon, SHOTGUN);
        assert!(!p.log.iter().any(|(_, e)| *e == WeaponTickEvent::RechamberWeapon), "no pump before the drop");
        p.run_until(120, |p| p.hand.weapon == SNIPER && p.state() == WeaponState::Ready);
        assert!(p.gun(SHOTGUN).needs_rechamber, "the 870 still needs its pump");
        p.swap_to(SHOTGUN);
        p.run_until(120, |p| p.hand.weapon == SHOTGUN && p.state() == WeaponState::Rechambering);
        p.run_until(120, |p| p.state() == WeaponState::Ready);
        assert!(!p.gun(SHOTGUN).needs_rechamber);
    });
}

/// The Moni NAC: swap during the 870's pump and back before the drop ends:
/// the 870 is ready that frame and its pump starts again from the top, and
/// it can be chained.
#[test]
fn swapping_away_mid_pump_and_back_restarts_the_pump() {
    with_test_rules(true, || {
        let mut p = Player::new(SHOTGUN, &[SNIPER]);
        p.tap_fire();
        p.run_until(30, |p| p.state() == WeaponState::Rechambering);
        for _ in 0..3 {
            p.frames(9);
            p.swap_to(SNIPER);
            p.step();
            assert_eq!(p.state(), WeaponState::Dropping);
            p.frames(8);
            p.swap_to(SHOTGUN);
            p.step();
            assert_eq!(p.hand.weapon, SHOTGUN);
            assert_eq!(p.state(), WeaponState::Rechambering, "the swap-back frame restarts the pump");
            assert_eq!(p.hand.weapon_time, shotgun().rechamber_time_ms);
        }
        assert_eq!(p.gun(SHOTGUN).clip, 7, "no ammo changes");
        // Left alone it finishes and clears.
        p.run_until(60, |p| p.state() == WeaponState::Ready);
        assert!(!p.gun(SHOTGUN).needs_rechamber);
    });
}

/// Past the bolt the pump still counts as not done: a swap there and back
/// replays it (Infinity Ward's rules cleared the mark at the bolt).
#[test]
fn the_mark_stays_until_the_whole_pump_has_played() {
    with_test_rules(true, || {
        let mut p = Player::new(SHOTGUN, &[SNIPER]);
        p.tap_fire();
        p.run_until(30, |p| p.state() == WeaponState::Rechambering);
        p.run_until(40, |p| p.hand.weapon_delay == 0);
        assert!(p.hand.rechamber_pending);
        p.swap_to(SNIPER);
        p.frames(4);
        p.swap_to(SHOTGUN);
        p.step();
        assert_eq!(p.state(), WeaponState::Rechambering);
    });
}

/// The instaswap: the sniper is going down (swapped off it), the trigger is
/// held and swap is pressed back: the sniper is ready and fires on that
/// same frame, with no raise.
#[test]
fn a_swap_back_with_the_trigger_held_fires_at_once() {
    with_test_rules(true, || {
        let mut p = Player::new(SNIPER, &[SHOTGUN]);
        p.swap_to(SHOTGUN);
        p.frames(12);
        assert_eq!(p.state(), WeaponState::Dropping);
        p.buttons |= BUTTON_ATTACK;
        p.step();
        assert_eq!(p.shots_since(0), 0, "no shot while going down");
        p.swap_to(SNIPER);
        let f = p.frame;
        p.step();
        assert_eq!(p.shots_since(f), 1, "fires on the swap-back frame");
        assert_eq!(p.gun(SNIPER).clip, 4);
    });
}

/// Footage: shotgun fired, swap to the sniper, swap back off it while it is
/// still up, then swap to it again holding fire: it fires 1 frame later.
#[test]
fn fire_swap_swap_fire_chain() {
    with_test_rules(true, || {
        let mut p = Player::new(SHOTGUN, &[SNIPER]);
        p.tap_fire();
        p.frames(4);
        p.swap_to(SNIPER);
        p.run_until(120, |p| p.hand.weapon == SNIPER && p.state() == WeaponState::Ready);
        p.swap_to(SHOTGUN);
        p.frames(11);
        assert_eq!(p.hand.weapon, SNIPER);
        p.buttons |= BUTTON_ATTACK;
        p.swap_to(SNIPER);
        let f = p.frame;
        p.step();
        assert_eq!(p.shots_since(f), 1);
    });
}

/// Without the swap-back, a trigger held through a raise does not fire:
/// the semi-automatic waits for it to come up and go down again.
#[test]
fn a_trigger_held_through_a_raise_does_not_fire() {
    with_test_rules(true, || {
        let mut p = Player::new(SHOTGUN, &[SNIPER]);
        p.buttons |= BUTTON_ATTACK;
        p.step();
        assert_eq!(p.shots_since(0), 1);
        p.swap_to(SNIPER);
        p.frames(90);
        assert_eq!(p.hand.weapon, SNIPER);
        assert_eq!(p.shots_since(1), 0, "held trigger: no shot");
        p.buttons = 0;
        p.frames(3);
        p.buttons |= BUTTON_ATTACK;
        let f = p.frame;
        p.frames(2);
        assert_eq!(p.shots_since(f), 1, "a fresh pull fires");
    });
}

/// Semi-automatic rate stays one shot per pull under the new fire rule.
#[test]
fn a_held_trigger_fires_a_semi_automatic_once() {
    with_test_rules(true, || {
        let mut p = Player::new(SHOTGUN, &[]);
        p.buttons |= BUTTON_ATTACK;
        p.frames(200);
        assert_eq!(p.shots_since(0), 1);
    });
}

/// Select Fire in full-auto: a swap pressed while the trigger is held
/// starts the drop between two shots.
#[test]
fn an_automatic_swaps_between_shots() {
    with_test_rules(true, || {
        let mut p = Player::new(SMG, &[SNIPER]);
        p.buttons |= BUTTON_ATTACK;
        p.frames(20);
        p.swap_to(SNIPER);
        let n = p.run_until(10, |p| p.state() == WeaponState::Dropping);
        assert!(n <= 5, "drop within one shot's time, took {n} frames");
    });
}

/// In burst the swap waits for the burst to finish: nothing is checked
/// while a burst is going out.
#[test]
fn a_burst_finishes_before_the_swap() {
    with_test_rules(true, || {
        let mut p = Player::new(SMG_BURST, &[SNIPER]);
        p.tap_fire();
        p.swap_to(SNIPER);
        p.run_until(30, |p| p.state() == WeaponState::Dropping);
        assert_eq!(p.gun(SMG_BURST).clip, 37, "the whole burst went out first");
    });
}

/// A burst gun needs a fresh pull after its cooldown.
#[test]
fn a_held_burst_trigger_fires_one_burst() {
    with_test_rules(true, || {
        let mut p = Player::new(SMG_BURST, &[]);
        p.buttons |= BUTTON_ATTACK;
        p.frames(120);
        assert_eq!(p.shots_since(0), 3);
        p.buttons = 0;
        p.frames(2);
        p.buttons |= BUTTON_ATTACK;
        p.frames(40);
        assert_eq!(p.shots_since(0), 6);
    });
}

/// Reload pressed after a swap-back: the 870 reloads that frame (the change
/// check comes before the reload check).
#[test]
fn reload_right_after_a_swap_back() {
    with_test_rules(true, || {
        let mut p = Player::new(SNIPER, &[SHOTGUN]);
        p.gun(SNIPER).clip = 3;
        p.hand.clip = 3;
        p.swap_to(SHOTGUN);
        p.frames(6);
        p.swap_to(SNIPER);
        p.buttons |= BUTTON_RELOAD;
        let ev = p.step();
        assert_eq!(ev, Some(WeaponTickEvent::ReloadStarted));
    });
}

/// Infinity Ward's rules are unchanged.
#[test]
fn infinity_ward_rules_unchanged() {
    with_test_rules(false, || {
        let mut p = Player::new(SNIPER, &[SHOTGUN]);
        p.swap_to(SHOTGUN);
        p.frames(12);
        p.buttons |= BUTTON_ATTACK;
        p.step();
        p.swap_to(SNIPER);
        let f = p.frame;
        p.step();
        assert_eq!(p.shots_since(f), 0, "needs a fresh pull");
    });
}
