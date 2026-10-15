use asset_game::WeaponAnimSlot;

use crate::{EventResult, ViewmodelController, ViewmodelEvent, WeaponState};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum PresentFpvEvent {
    Fire,

    ReleaseAttack,
    RaiseFirst,
    Raise,
    Reload,
    ReloadEmpty,
    ReloadStart,
    ReloadEnd,
    Rechamber,
    Drop,
    QuickDrop,
    QuickRaise,
    /// bo2mp: Black Ops II's drop and raise of a gun with an empty clip.
    EmptyDrop,
    EmptyRaise,
    SprintIn,
    SprintLoop,
    SprintOut,
}

/// The engine's weapAnim numbers for the drops and raises (`weap_anim.rs`):
/// first raise, and Black Ops II's empty-clip drop and raise.
const WEAP_ANIM_FIRST_RAISE: u32 = 0xc;
const WEAP_ANIM_EMPTY_DROP: u32 = weapon_iw4::weap_anim_event::EMPTY_DROP;
const WEAP_ANIM_EMPTY_RAISE: u32 = weapon_iw4::weap_anim_event::EMPTY_RAISE;

/// bo2mp: the drop or raise a weapAnim number (restart bit masked off) asks
/// for. The alternate-mode swap (0x11, 0x12) plays through `szXAnims`.
pub fn switch_event_for_weap_anim(masked: u32) -> Option<PresentFpvEvent> {
    use weapon_iw4::weap_anim_event as event_id;
    Some(match masked {
        event_id::DROP => PresentFpvEvent::Drop,
        event_id::RAISE => PresentFpvEvent::Raise,
        WEAP_ANIM_FIRST_RAISE => PresentFpvEvent::RaiseFirst,
        event_id::QUICK_DROP => PresentFpvEvent::QuickDrop,
        event_id::QUICK_RAISE => PresentFpvEvent::QuickRaise,
        WEAP_ANIM_EMPTY_DROP => PresentFpvEvent::EmptyDrop,
        WEAP_ANIM_EMPTY_RAISE => PresentFpvEvent::EmptyRaise,
        _ => return None,
    })
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct AuthorityFpvCues {
    pub shot_accepted: bool,
    pub spawned: bool,
    pub attack_released: bool,
}

#[derive(Clone, Debug, PartialEq)]
pub struct FpvPoseSample {
    pub state: WeaponState,

    pub clip_name: Option<String>,
    pub clip_time: f32,
    pub fire_weight: f32,
    pub idle_weight: f32,
}

pub fn map_authority_to_fpv(cues: AuthorityFpvCues) -> Vec<PresentFpvEvent> {
    let mut out = Vec::new();

    if cues.shot_accepted {
        out.push(PresentFpvEvent::Fire);
    }
    if cues.attack_released {
        out.push(PresentFpvEvent::ReleaseAttack);
    }

    out
}

fn viewmodel_event_for(event: PresentFpvEvent) -> Option<ViewmodelEvent> {
    Some(match event {
        PresentFpvEvent::Fire => ViewmodelEvent::Fire,
        PresentFpvEvent::ReleaseAttack => return None,
        PresentFpvEvent::RaiseFirst => ViewmodelEvent::Raise {
            first: true,
            quick: false,
        },
        PresentFpvEvent::Raise => ViewmodelEvent::Raise {
            first: false,
            quick: false,
        },
        PresentFpvEvent::Reload => ViewmodelEvent::Reload { empty: false },
        PresentFpvEvent::ReloadEmpty => ViewmodelEvent::Reload { empty: true },
        PresentFpvEvent::ReloadStart => ViewmodelEvent::ReloadStart,
        PresentFpvEvent::ReloadEnd => ViewmodelEvent::ReloadEnd,
        PresentFpvEvent::Rechamber => ViewmodelEvent::Rechamber,
        PresentFpvEvent::Drop => ViewmodelEvent::Drop,
        PresentFpvEvent::QuickDrop => ViewmodelEvent::QuickDrop,
        PresentFpvEvent::QuickRaise => ViewmodelEvent::Raise {
            first: false,
            quick: true,
        },
        PresentFpvEvent::EmptyDrop => ViewmodelEvent::EmptyDrop,
        PresentFpvEvent::EmptyRaise => ViewmodelEvent::EmptyRaise,
        PresentFpvEvent::SprintIn => ViewmodelEvent::SprintIn,
        PresentFpvEvent::SprintLoop => ViewmodelEvent::SprintLoop,
        PresentFpvEvent::SprintOut => ViewmodelEvent::SprintOut,
    })
}

fn apply_present_events(controller: &mut ViewmodelController, events: &[PresentFpvEvent]) {
    for event in events {
        match viewmodel_event_for(*event) {
            None => controller.settle_fire_to_idle(),
            Some(vm) => report_fpv_event_result(*event, controller.handle(vm)),
        }
    }
}

pub fn is_predicted_fire_weap_anim(masked: u32) -> bool {
    use weapon_iw4::weap_anim_event as event_id;
    matches!(
        masked,
        event_id::FIRE
            | event_id::LASTSHOT
            | event_id::ADS_FIRE
            | event_id::ADS_LASTSHOT
            | event_id::FIRE_INTRO
            | event_id::ADS_FIRE_INTRO
    )
}

fn report_dispatch_slot(slot: usize, result: EventResult) {
    match result {
        EventResult::Started(_) => {}
        EventResult::IgnoredState(state) => {
            diag::warn!(Fpv, "fpv: szXAnims[{slot}] ignored in state {state:?}");
        }
        EventResult::IgnoredMissingClip(named) => {
            diag::warn!(
                Fpv,
                "fpv: szXAnims[{slot}] ({named:?}) missing — not started"
            );
        }
    }
}

fn report_fpv_event_result(event: PresentFpvEvent, result: EventResult) {
    match result {
        EventResult::Started(_) => {}
        EventResult::IgnoredState(state) => {
            diag::warn!(Fpv, "fpv: {event:?} ignored in state {state:?}");
        }
        EventResult::IgnoredMissingClip(slot) => {
            diag::warn!(Fpv, "fpv: {event:?} ignored — szXAnims[{slot:?}] missing");
        }
    }
}

pub fn sample_pose(controller: &ViewmodelController) -> FpvPoseSample {
    let active = controller.active_anims().max_by(|a, b| {
        a.weight
            .partial_cmp(&b.weight)
            .unwrap_or(std::cmp::Ordering::Equal)
    });
    FpvPoseSample {
        state: controller.state(),
        clip_name: active.map(|a| a.clip.name.clone()),
        clip_time: active.map(|a| a.time).unwrap_or(0.0),
        fire_weight: controller.animation_weight(WeaponAnimSlot::Fire),
        idle_weight: controller.animation_weight(WeaponAnimSlot::Idle),
    }
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct FpvAuthoritySample {
    pub tick: u32,
    pub weaponstate: i32,
    pub cues: AuthorityFpvCues,

    pub sprinting: bool,

    pub ads_frac: f32,

    pub weap_anim: i32,

    /// The sim's timer on its weapon state (ms left), primary hand.
    pub weapon_time: i32,

    pub weap_anim_secondary: i32,

    pub last_weapon_hand: i32,

    pub perks0: u32,

    pub clip_ammo: Option<i32>,

    pub left_clip_ammo: Option<i32>,
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct LocalShotIdentity {
    pub life_sequence: u32,
    pub hand: u8,
    pub weapon_shot_count: i32,

    pub weap_anim_restart: bool,
}

#[derive(Clone, Debug, Default)]
pub struct LocalShotCursor {
    last: Option<LocalShotIdentity>,
}

impl LocalShotCursor {
    pub fn clear(&mut self) {
        self.last = None;
    }

    pub fn observe(&mut self, identity: LocalShotIdentity) -> bool {
        match self.last {
            None => {
                self.last = Some(identity);
                false
            }
            Some(prev) if prev == identity => false,
            Some(prev) => {
                let count_edge = identity.weapon_shot_count != prev.weapon_shot_count;
                let restart_edge = identity.weap_anim_restart != prev.weap_anim_restart
                    && identity.weapon_shot_count == prev.weapon_shot_count;
                self.last = Some(identity);
                count_edge || restart_edge
            }
        }
    }

    pub fn last(&self) -> Option<LocalShotIdentity> {
        self.last
    }
}

pub fn local_shot_identity(
    life_sequence: u32,
    hand: u8,
    weapon_shot_count: i32,
    weap_anim: i32,
) -> LocalShotIdentity {
    LocalShotIdentity {
        life_sequence,
        hand,
        weapon_shot_count,
        weap_anim_restart: (weap_anim as u32 & 0x200) != 0,
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum WeapAnimEdge {
    Unchanged,

    Idle,

    ForceIdle,

    Dispatch(usize),

    /// bo2mp: a drop or raise; `restart_only`: the same one again with only
    /// the restart bit flipped.
    Switch {
        event: PresentFpvEvent,
        restart_only: bool,
    },
}

#[derive(Clone, Debug, Default)]
pub struct FpvPresentState {
    pub last_tick: Option<u32>,

    pub last_ads_frac: Option<f32>,

    pub last_weap_anim: Option<i32>,

    pub last_weap_anim_secondary: Option<i32>,

    pub local_shot: LocalShotCursor,
}

impl FpvPresentState {
    pub fn clear(&mut self) {
        self.last_tick = None;
        self.last_ads_frac = None;
        self.last_weap_anim = None;
        self.last_weap_anim_secondary = None;
        self.local_shot.clear();
    }

    pub fn forget_weap_anim(&mut self) {
        self.last_weap_anim = None;
        self.last_weap_anim_secondary = None;
    }

    pub fn observe_weap_anim_edge(&mut self, raw: i32) -> WeapAnimEdge {
        observe_weap_anim_edge_on(&mut self.last_weap_anim, raw)
    }

    pub fn observe_weap_anim_secondary_edge(&mut self, raw: i32) -> WeapAnimEdge {
        observe_weap_anim_edge_on(&mut self.last_weap_anim_secondary, raw)
    }
}

#[derive(Debug)]
pub struct EquippedFpv {
    pub gun_xmodel: String,
    pub gun_index: assets::FpvMeshIndex,
    pub hands_index: assets::FpvMeshIndex,
    pub namespace: asset_core::AssetNamespace,
    pub hands: asset_model::FpvHands,
    pub controller: ViewmodelController,

    pub left: Option<ViewmodelController>,
}

impl EquippedFpv {
    pub fn new(
        gun_xmodel: impl Into<String>,
        gun_index: assets::FpvMeshIndex,
        hands_index: assets::FpvMeshIndex,
        namespace: asset_core::AssetNamespace,
        hands: asset_model::FpvHands,
        controller: ViewmodelController,
        left: Option<ViewmodelController>,
    ) -> Self {
        Self {
            gun_xmodel: gun_xmodel.into(),
            gun_index,
            hands_index,
            namespace,
            hands,
            controller,
            left,
        }
    }
}

pub fn events_for_authority_tick(
    present: &mut FpvPresentState,
    sample: FpvAuthoritySample,
) -> Vec<PresentFpvEvent> {
    if present.last_tick == Some(sample.tick) {
        return Vec::new();
    }
    let events = map_authority_to_fpv(sample.cues);
    present.last_tick = Some(sample.tick);
    present.last_ads_frac = Some(sample.ads_frac);
    events
}

pub fn tick_equipped_fpv_with_predicted_fire(
    equipped: &mut EquippedFpv,
    present: &mut FpvPresentState,
    sample: Option<FpvAuthoritySample>,
    predicted_local_fire: bool,
    dt_secs: f32,
) -> (FpvPoseSample, Vec<String>) {
    tick_equipped_fpv_with_extra_events(
        equipped,
        present,
        sample,
        predicted_local_fire,
        &[],
        dt_secs,
    )
}

pub fn tick_equipped_fpv_with_extra_events(
    equipped: &mut EquippedFpv,
    present: &mut FpvPresentState,
    sample: Option<FpvAuthoritySample>,
    predicted_local_fire: bool,
    extra_events: &[PresentFpvEvent],
    dt_secs: f32,
) -> (FpvPoseSample, Vec<String>) {
    let mut events = match sample {
        Some(sample) => events_for_authority_tick(present, sample),
        None => Vec::new(),
    };
    if predicted_local_fire && !events.contains(&PresentFpvEvent::Fire) {
        events.push(PresentFpvEvent::Fire);
    }
    for event in extra_events {
        if !events.contains(event) {
            events.push(*event);
        }
    }
    if let Some(sample) = sample {
        equipped.controller.set_predicted_perks(sample.perks0);
    }
    apply_present_events(&mut equipped.controller, &events);
    if let Some(sample) = sample {
        let edge = present.observe_weap_anim_edge(sample.weap_anim);
        let empty_mag = sample.clip_ammo == Some(0);
        let sim = Some((sample.weaponstate, sample.weapon_time));
        if !apply_weap_anim_edge(&mut equipped.controller, edge, empty_mag, sim) {
            present.last_weap_anim = None;
        }
    }
    let crate::AdvanceResult { notifies } = equipped.controller.advance(dt_secs);
    let ads_frac = sample
        .map(|s| s.ads_frac)
        .or(present.last_ads_frac)
        .unwrap_or(0.0);
    equipped.controller.apply_ads_overlay_frame(ads_frac);
    if let (Some(left), Some(sample)) = (equipped.left.as_mut(), sample) {
        if sample.last_weapon_hand == 1 {
            left.set_predicted_perks(sample.perks0);
            let edge = present.observe_weap_anim_secondary_edge(sample.weap_anim_secondary);
            // The secondary hand's timer is not in the sample: its drop and
            // raise play over the gun's own times.
            if !apply_weap_anim_edge(left, edge, sample.left_clip_ammo == Some(0), None) {
                present.last_weap_anim_secondary = None;
            }
            let _ = left.advance(dt_secs);

            left.apply_ads_overlay_frame(0.0);
        }
    }
    (sample_pose(&equipped.controller), notifies)
}

/// Apply one weapAnim edge to a controller; false: the idle edge waits for
/// the playing clip (observe it again next frame). `sim`: the sim's weapon
/// state and its timer (ms left), when known.
pub fn apply_weap_anim_edge(
    controller: &mut ViewmodelController,
    edge: WeapAnimEdge,
    empty_mag: bool,
    sim: Option<(i32, i32)>,
) -> bool {
    match edge {
        WeapAnimEdge::Unchanged => {}
        WeapAnimEdge::Idle => return controller.apply_idle_weap_anim(empty_mag),
        WeapAnimEdge::ForceIdle => controller.apply_force_idle_weap_anim(empty_mag),
        WeapAnimEdge::Dispatch(slot) => {
            report_dispatch_slot(slot, controller.dispatch_sz_xanim_index(slot));
        }
        WeapAnimEdge::Switch {
            event,
            restart_only,
        } => {
            // bo2mp: the same drop or raise with only its restart bit
            // flipped is a prediction reconciled against the server's
            // (their toggles disagreed), not a new drop or raise: the sim
            // never starts one drop on another or one raise on another.
            // Restarting it put the gun back at the top of the drop (or the
            // bottom of the raise) and stretched the swap.
            let playing = controller.switch_in_progress();
            let same_kind = match (event, playing) {
                (
                    PresentFpvEvent::Drop
                    | PresentFpvEvent::QuickDrop
                    | PresentFpvEvent::EmptyDrop,
                    Some(WeaponState::Dropping),
                ) => true,
                (
                    PresentFpvEvent::Raise
                    | PresentFpvEvent::RaiseFirst
                    | PresentFpvEvent::QuickRaise
                    | PresentFpvEvent::EmptyRaise,
                    Some(WeaponState::Raising { .. }),
                ) => true,
                _ => false,
            };
            if !(restart_only && same_kind) {
                apply_present_events(controller, &[event]);
            }
        }
    }
    // On the edge only: the sim's timer steps a tick at a time, and
    // re-timing every frame would wobble the clip's speed.
    if edge != WeapAnimEdge::Unchanged
        && let Some((weaponstate, weapon_time)) = sim
    {
        sync_switch_with_sim(controller, weaponstate, weapon_time);
    }
    true
}

/// bo2mp: a drop or raise playing while the sim is in that drop or raise
/// plays out over the sim's timer: the drop is down when the sim's drop
/// ends, the raise up when the sim's raise does.
fn sync_switch_with_sim(controller: &mut ViewmodelController, weaponstate: i32, weapon_time: i32) {
    use weapon_iw4::WeaponState as Sim;
    let Ok(sim) = Sim::from_i32(weaponstate) else {
        return;
    };
    let agrees = match controller.switch_in_progress() {
        Some(WeaponState::Dropping) => matches!(
            sim,
            Sim::Dropping | Sim::DroppingQuick | Sim::DroppingAltswitch | Sim::OffhandInit
        ),
        Some(WeaponState::Raising { .. }) => matches!(sim, Sim::Raising | Sim::RaisingAltswitch),
        _ => false,
    };
    if agrees {
        controller.sync_switch_to_sim(weapon_time);
    }
}

fn observe_weap_anim_edge_on(previous: &mut Option<i32>, raw: i32) -> WeapAnimEdge {
    let last = previous.replace(raw);
    if last == Some(raw) {
        return WeapAnimEdge::Unchanged;
    }
    let masked = raw as u32 & weapon_iw4::WEAP_ANIM_EVENT_MASK;
    if let Some(event) = switch_event_for_weap_anim(masked) {
        let restart_only = last.is_some_and(|last| {
            (last as u32 ^ raw as u32) == weapon_iw4::WEAP_ANIM_RESTART_BIT
        });
        return WeapAnimEdge::Switch {
            event,
            restart_only,
        };
    }
    match masked {
        0 => WeapAnimEdge::Idle,
        1 => WeapAnimEdge::ForceIdle,
        _ => match weapon_iw4::slot_for_weap_anim_event(masked) {
            Some(slot) => WeapAnimEdge::Dispatch(slot),
            None => WeapAnimEdge::Idle,
        },
    }
}

#[cfg(test)]
mod tests {
    use std::sync::Arc;

    use asset_anim::xanim_clip::{AnimClip, Translation};
    use asset_game::WeaponAnimations;
    use weapon_iw4::{WEAP_ANIM_RESTART_BIT, weap_anim_event as event_id};

    use super::*;

    fn clip(name: &str, frames: u16, looping: bool) -> Arc<AnimClip> {
        Arc::new(AnimClip {
            name: name.to_owned(),
            framerate: 30.0,
            numframes: frames,
            looping,
            tracks: Vec::new(),
            notifies: Vec::new(),
            delta_translation: Translation::Default,
        })
    }

    /// 15-frame (0.5 s) drop and raise; drop over 600 ms, raise over 800 ms.
    fn gun() -> ViewmodelController {
        let mut gun = WeaponAnimations::empty("dsr50_mp")
            .with_clip(WeaponAnimSlot::Idle, clip("idle", 30, true))
            .with_clip(WeaponAnimSlot::Drop, clip("drop", 15, false))
            .with_clip(WeaponAnimSlot::Raise, clip("raise", 15, false))
            .with_switch_timers(600, 0, 0);
        gun.raise_time_ms = 800;
        gun.empty_drop_time_ms = 300;
        ViewmodelController::new(gun)
    }

    const SIM_RAISING: i32 = 0x1;
    const SIM_DROPPING: i32 = 0x3;
    const DOWN: f32 = 0.5 - 1e-4;

    fn edge(
        present: &mut FpvPresentState,
        c: &mut ViewmodelController,
        raw: u32,
        sim: (i32, i32),
    ) -> bool {
        let edge = present.observe_weap_anim_edge(raw as i32);
        apply_weap_anim_edge(c, edge, false, Some(sim))
    }

    #[test]
    fn the_switch_weap_anims_map_to_present_events() {
        let map = switch_event_for_weap_anim;
        assert_eq!(map(event_id::DROP), Some(PresentFpvEvent::Drop));
        assert_eq!(map(event_id::RAISE), Some(PresentFpvEvent::Raise));
        assert_eq!(map(0xc), Some(PresentFpvEvent::RaiseFirst));
        assert_eq!(map(event_id::QUICK_DROP), Some(PresentFpvEvent::QuickDrop));
        assert_eq!(map(event_id::QUICK_RAISE), Some(PresentFpvEvent::QuickRaise));
        assert_eq!(map(0x15), Some(PresentFpvEvent::EmptyDrop));
        assert_eq!(map(0x16), Some(PresentFpvEvent::EmptyRaise));
        assert_eq!(map(event_id::FIRE), None);
        assert_eq!(
            viewmodel_event_for(PresentFpvEvent::EmptyDrop),
            Some(ViewmodelEvent::EmptyDrop)
        );
        assert_eq!(
            viewmodel_event_for(PresentFpvEvent::EmptyRaise),
            Some(ViewmodelEvent::EmptyRaise)
        );
    }

    #[test]
    fn the_empty_drop_number_plays_over_the_empty_drop_time() {
        let mut present = FpvPresentState::default();
        let mut c = gun();
        edge(&mut present, &mut c, 0, (0, 0));
        edge(&mut present, &mut c, 0x15 | WEAP_ANIM_RESTART_BIT, (SIM_DROPPING, 300));
        assert_eq!(c.state(), WeaponState::Dropping);
        assert_eq!(c.animation_weight(WeaponAnimSlot::Drop), 1.0);
        c.advance(0.29);
        assert!(c.animation_time(WeaponAnimSlot::Drop) < DOWN);
        c.advance(0.011);
        assert!(c.animation_time(WeaponAnimSlot::Drop) >= DOWN);
    }

    #[test]
    fn the_new_gun_raises_from_the_first_raising_tick() {
        // The viewmodel is rebuilt for the new gun (`forget_weap_anim`): its
        // first sample is the sim's first Raising tick.
        let mut present = FpvPresentState {
            last_weap_anim: Some((event_id::DROP | WEAP_ANIM_RESTART_BIT) as i32),
            ..Default::default()
        };
        present.forget_weap_anim();
        let mut c = gun();
        edge(&mut present, &mut c, event_id::RAISE, (SIM_RAISING, 800));
        assert!(matches!(c.state(), WeaponState::Raising { .. }));
        assert_eq!(c.animation_weight(WeaponAnimSlot::Raise), 1.0);
        assert_eq!(c.animation_weight(WeaponAnimSlot::Idle), 0.0);
        assert_eq!(c.animation_time(WeaponAnimSlot::Raise), 0.0);
        c.advance(0.79);
        assert!(c.animation_time(WeaponAnimSlot::Raise) < DOWN);
        c.advance(0.011);
        assert!(c.animation_time(WeaponAnimSlot::Raise) >= DOWN, "up with the sim");
        // The sim's raise ends: idle at once.
        let idle = event_id::IDLE | WEAP_ANIM_RESTART_BIT;
        assert!(edge(&mut present, &mut c, idle, (0, 0)));
        assert_eq!(c.state(), WeaponState::Ready);
    }

    #[test]
    fn a_raise_seen_late_catches_up_with_the_sim() {
        let mut present = FpvPresentState::default();
        let mut c = gun();
        // The sim is 300 ms into its 800 ms raise when the edge is seen.
        edge(&mut present, &mut c, event_id::RAISE, (SIM_RAISING, 500));
        c.advance(0.49);
        assert!(c.animation_time(WeaponAnimSlot::Raise) < DOWN);
        c.advance(0.011);
        assert!(c.animation_time(WeaponAnimSlot::Raise) >= DOWN);
    }

    #[test]
    fn a_reconciled_restart_bit_does_not_restart_the_drop() {
        let mut present = FpvPresentState::default();
        let mut c = gun();
        let flipped = event_id::DROP | WEAP_ANIM_RESTART_BIT;
        edge(&mut present, &mut c, event_id::DROP, (SIM_DROPPING, 600));
        c.advance(0.3);
        let t = c.animation_time(WeaponAnimSlot::Drop);
        assert!(t > 0.2);
        edge(&mut present, &mut c, flipped, (SIM_DROPPING, 300));
        assert_eq!(c.animation_time(WeaponAnimSlot::Drop), t, "not restarted");
        // A swap back readies the gun at once; a new swap drops it afresh.
        edge(&mut present, &mut c, event_id::IDLE, (0, 0));
        assert_eq!(c.state(), WeaponState::Ready);
        assert_eq!(c.animation_weight(WeaponAnimSlot::Idle), 1.0);
        edge(&mut present, &mut c, flipped, (SIM_DROPPING, 600));
        assert_eq!(c.state(), WeaponState::Dropping);
        assert_eq!(c.animation_time(WeaponAnimSlot::Drop), 0.0);
    }

    #[test]
    fn a_swap_while_raising_drops_at_once_over_the_full_drop() {
        let mut present = FpvPresentState::default();
        let mut c = gun();
        edge(&mut present, &mut c, event_id::RAISE, (SIM_RAISING, 800));
        c.advance(0.2);
        let drop = event_id::DROP | WEAP_ANIM_RESTART_BIT;
        edge(&mut present, &mut c, drop, (SIM_DROPPING, 600));
        assert_eq!(c.state(), WeaponState::Dropping);
        assert_eq!(c.animation_weight(WeaponAnimSlot::Raise), 0.0);
        c.advance(0.59);
        assert!(c.animation_time(WeaponAnimSlot::Drop) < DOWN);
        c.advance(0.011);
        assert!(c.animation_time(WeaponAnimSlot::Drop) >= DOWN);
    }
}
