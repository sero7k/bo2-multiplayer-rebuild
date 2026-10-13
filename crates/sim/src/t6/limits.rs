//! What the scripts allow a player's body, as BO2's engine keeps it.
//! TranZit's bus takes sprint and prone from its riders, a zombie clinging
//! to the bus takes crouch, a Denizen on his head halves his speed, and a
//! death in the lava or on the moving bus leaves him lying down. The
//! switches (`allowsprint`, `allowprone`, `allowcrouch`, `allowstand`,
//! `allowmelee`, `allowads`) are the player's `ScriptControls`
//! (t6::natives_player; they reach his PC too, so his own guess of his moves
//! agrees), and `script_player::constrain_cmd` lets go of the taken moves
//! with BO2's Zombies stance rule. Here: `setmovespeedscale`, and a sprint
//! already going ends when sprint is taken.

use bevy_ecs::prelude::World;
use gsc_t6::{Value, Vm};
use playerstate_iw4::buttons;
use playerstate_iw4::pm_flags::SPRINTING;

use super::{Zm, arg, client, frame};
use crate::world::ClientId;

/// His switches (all allowed by default and after a spawn).
fn controls(world: &mut World, c: u32) -> crate::match_state::ScriptControls {
    frame(world)
        .client_meta(ClientId(c))
        .map(|m| m.controls)
        .unwrap_or_default()
}

pub(super) fn bind(vm: &mut Vm<World>) {
    // player setmovespeedscale(scale): his run speed times scale.
    vm.bind("setmovespeedscale", true, |vm, world, s, a| {
        let id = client(vm, world, s)?;
        let scale = arg(a, 0).as_float().unwrap_or(1.0).max(0.0);
        if let Some(ps) = frame(world).player_mut(id) {
            ps.move_speed_scale_multiplier = scale;
        }
        Ok(Value::Undefined)
    });
}

/// Each tick, before the move step: a sprint already going ends once
/// sprint is taken (letting go of sprint does not end one; the move step's
/// constrain_cmd lets go of the taken buttons).
pub(super) fn tick(world: &mut World) {
    press_test(world);
    let clients: Vec<u32> = world.resource::<Zm>().players.keys().copied().collect();
    let no_sprint: Vec<u32> = clients
        .into_iter()
        .filter(|c| controls(world, *c).sprint_disabled)
        .collect();
    let mut f = frame(world);
    for c in no_sprint {
        if let Some(ps) = f.player_mut(ClientId(c)) {
            if ps.pm_flags & SPRINTING != 0 {
                ps.pm_flags &= !SPRINTING;
                ps.sprint_delay = 0;
                ps.last_sprint_end = ps.command_time;
            }
        }
    }
}

/// IW4L_T6_PRESS="sprint+forward@30-33,prone@34-37,melee@50-50.3,..."
/// (seconds): player 0 holds those buttons then (tests that the bus takes
/// sprint and prone from its riders, and that melee taps knock a Denizen
/// off his head); once a second it logs what he kept and how he stands.
/// `use` holds the use key (TranZit's navcard pickup and navcomputer),
/// `attack` the fire key (a Jet Gun spun at the navcomputer's tower).
fn press_test(world: &mut World) {
    type Press = Vec<(u32, bool, i64, i64)>;
    static PRESS: std::sync::OnceLock<Option<Press>> = std::sync::OnceLock::new();
    let Some(press) = PRESS.get_or_init(|| {
        let v = std::env::var("IW4L_T6_PRESS").ok()?;
        let mut out = Vec::new();
        for part in v.split(',') {
            let Some((names, when)) = part.split_once('@') else {
                continue;
            };
            let Some((a, z)) = when.split_once('-') else {
                continue;
            };
            let (Ok(a), Ok(z)) = (a.trim().parse::<f32>(), z.trim().parse::<f32>()) else {
                continue;
            };
            let mut b = 0;
            let mut forward = false;
            for n in names.split('+') {
                match n.trim() {
                    "sprint" => b |= buttons::SPRINT,
                    "prone" => b |= buttons::PRONE,
                    "crouch" => b |= buttons::CROUCH,
                    "melee" => b |= buttons::MELEE_CHARGE,
                    "jump" => b |= buttons::JUMP,
                    "use" => b |= buttons::USE,
                    "attack" => b |= buttons::ATTACK,
                    "frag" => b |= buttons::FRAG,
                    "smoke" => b |= buttons::SMOKE,
                    "forward" => forward = true,
                    _ => {}
                }
            }
            out.push((b, forward, (a * 1000.0) as i64, (z * 1000.0) as i64));
        }
        Some(out)
    }) else {
        return;
    };
    let now = world.resource::<Zm>().now_ms;
    let Some(&(b, forward, ..)) = press.iter().find(|p| now >= p.2 && now < p.3) else {
        return;
    };
    let id = ClientId(0);
    {
        let mut req = world.resource_mut::<crate::step::StepRequest>();
        for (who, cmd) in &mut req.input.cmds {
            if *who == id {
                cmd.buttons |= b;
                if forward {
                    cmd.forwardmove = 127;
                }
            }
        }
    }
    if now % 1000 >= i64::from(crate::MATCH_TICK_MS) {
        return;
    }
    let l = controls(world, id.0);
    if let Some(ps) = frame(world).player(id) {
        use playerstate_iw4::pm_flags;
        // What the move step keeps of his press (BO2's Zombies stance rule,
        // then the taken moves let go).
        let mut c = playerstate_iw4::UserCmd {
            buttons: b,
            ..Default::default()
        };
        crate::script_player::bo2_zombies_stance(&l, ps.pm_flags, &mut c);
        crate::script_player::apply_allow_controls(&l, ps.pm_flags, &mut c);
        let kept = c.buttons;
        diag::info!(
            Sim,
            "bo2zm t6 limits: player 0 pressed {b:#x} kept {kept:#x} (no sprint {}, no prone {}): sprinting {}, prone {}, crouched {}",
            l.sprint_disabled,
            l.prone_disabled,
            ps.pm_flags & SPRINTING != 0,
            ps.pm_flags & pm_flags::PRONE != 0,
            ps.pm_flags & pm_flags::CROUCH != 0
        );
    }
}
