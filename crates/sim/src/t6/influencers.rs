//! bo2mp: Black Ops II's spawn influencers.
//!
//! Black Ops II picks a spawn point by the influencers its scripts place
//! (`maps/mp/gametypes/_spawning`): each living player's enemy sphere
//! (-150 within 2600 units, linear) and friendly sphere, dead friends,
//! grenades, claymores, artillery, vehicles, dogs, recently spawned
//! enemies, the game mode's own (flags, zones, radios) and the map's.
//! Each has a shape (sphere, or cylinder along its up axis), a score, a
//! score curve, the teams it counts for (a mask: free 1, then 1 << n for the
//! teams in `level.teams`), an optional timeout and an optional entity it
//! follows. The engine sums every enabled influencer that counts for the
//! asking team at each of the team's points and walks them best first
//! (`getsortedspawnpoints`, in natives_mp).
//!
//! Verified from the scripts: the natives' arguments, the curve and shape
//! numbers, every default score, radius and curve. The curves' exact
//! shapes are the engine's and are not read yet; here: constant 1, linear
//! 1 - t, steep (1 - t)^2, inverse linear t, negative-to-positive
//! 2t - 1 (t: distance over radius, 0 at the centre).

use bevy_ecs::prelude::{Resource, World};
use gsc_t6::{Value, Vm};

/// One influencer, as its add native made it.
#[derive(Clone, Debug)]
pub(crate) struct Influencer {
    pub id: i32,
    pub cylinder: Option<Cylinder>,
    pub origin: [f32; 3],
    pub radius: f32,
    pub score: f32,
    pub team_mask: i32,
    pub curve: i32,
    /// Game time (ms) it ends; None: until removed.
    pub expires_ms: Option<i64>,
    pub enabled: bool,
    /// The entity it follows (its origin replaces `origin`).
    pub entity: Option<Value>,
}

/// A cylinder's axis (`up`, unit) and length from its origin along it.
#[derive(Clone, Copy, Debug)]
pub(crate) struct Cylinder {
    pub up: [f32; 3],
    pub length: f32,
}

#[derive(Resource, Default)]
pub(crate) struct SpawnInfluencers {
    next: i32,
    pub list: Vec<Influencer>,
}

impl SpawnInfluencers {
    pub fn add(&mut self, mut inf: Influencer) -> i32 {
        self.next += 1;
        inf.id = self.next;
        let id = inf.id;
        self.list.push(inf);
        id
    }

    pub fn remove(&mut self, id: i32) {
        self.list.retain(|i| i.id != id);
    }

    pub fn get_mut(&mut self, id: i32) -> Option<&mut Influencer> {
        self.list.iter_mut().find(|i| i.id == id)
    }

    /// Drops the ones whose time is up.
    pub fn expire(&mut self, now_ms: i64) {
        self.list.retain(|i| i.expires_ms.is_none_or(|t| now_ms < t));
    }
}

/// A score curve at `t` (distance over radius, 0..1).
pub(crate) fn curve(kind: i32, t: f32) -> f32 {
    let t = t.clamp(0.0, 1.0);
    match kind {
        1 => 1.0 - t,
        2 => (1.0 - t) * (1.0 - t),
        3 => t,
        4 => 2.0 * t - 1.0,
        _ => 1.0,
    }
}

/// An influencer's share at a point (its origin as given), or None when the
/// point is outside it.
pub(crate) fn influence_at(inf: &Influencer, origin: [f32; 3], p: [f32; 3]) -> Option<f32> {
    if inf.radius <= 0.0 {
        return None;
    }
    let d = [p[0] - origin[0], p[1] - origin[1], p[2] - origin[2]];
    let dist = match inf.cylinder {
        None => (d[0] * d[0] + d[1] * d[1] + d[2] * d[2]).sqrt(),
        Some(c) => {
            let along = d[0] * c.up[0] + d[1] * c.up[1] + d[2] * c.up[2];
            if along < 0.0 || along > c.length {
                return None;
            }
            let r = [d[0] - c.up[0] * along, d[1] - c.up[1] * along, d[2] - c.up[2] * along];
            (r[0] * r[0] + r[1] * r[1] + r[2] * r[2]).sqrt()
        }
    };
    (dist <= inf.radius).then(|| inf.score * curve(inf.curve, dist / inf.radius))
}

/// Every enabled influencer counting for `team_bit` summed at each point.
/// `origin_now`: an influencer's current origin (its entity's, or its own).
pub(crate) fn score_points(
    list: &[Influencer],
    team_bit: i32,
    points: &[[f32; 3]],
    origin_now: &dyn Fn(&Influencer) -> Option<[f32; 3]>,
) -> Vec<f32> {
    let active: Vec<(&Influencer, [f32; 3])> = list
        .iter()
        .filter(|i| i.enabled && i.team_mask & team_bit != 0)
        .filter_map(|i| origin_now(i).map(|o| (i, o)))
        .collect();
    points
        .iter()
        .map(|&p| active.iter().filter_map(|(i, o)| influence_at(i, *o, p)).sum())
        .collect()
}

fn unit(v: [f32; 3]) -> Option<[f32; 3]> {
    let l = (v[0] * v[0] + v[1] * v[1] + v[2] * v[2]).sqrt();
    (l > 1e-6).then(|| [v[0] / l, v[1] / l, v[2] / l])
}

pub(super) fn bind(vm: &mut Vm<World>) {
    use super::{arg, int, num, vec3};
    // addsphereinfluencer(type, origin, radius, score, team mask, name,
    // curve, [timeout seconds], [entity])
    vm.bind("addsphereinfluencer", false, |vm, world, _, a| {
        let timeout = num(a, 7).unwrap_or(0.0);
        let inf = Influencer {
            id: 0,
            cylinder: None,
            origin: vec3(a, 1).unwrap_or([0.0; 3]),
            radius: num(a, 2).unwrap_or(0.0),
            score: num(a, 3).unwrap_or(0.0),
            team_mask: int(a, 4).unwrap_or(0),
            curve: int(a, 6).unwrap_or(0),
            expires_ms: (timeout > 0.0).then(|| vm.time_ms + (timeout * 1000.0) as i64),
            enabled: true,
            entity: Some(arg(a, 8).clone()).filter(|v| !matches!(v, Value::Undefined)),
        };
        Ok(Value::Int(world.get_resource_or_insert_with(SpawnInfluencers::default).add(inf)))
    });
    // addcylinderinfluencer(type, origin, forward, up, radius, length,
    // score, team mask, name, curve, [timeout], [entity])
    vm.bind("addcylinderinfluencer", false, |vm, world, _, a| {
        let timeout = num(a, 10).unwrap_or(0.0);
        let up = unit(vec3(a, 3).unwrap_or([0.0; 3])).unwrap_or([0.0, 0.0, 1.0]);
        let inf = Influencer {
            id: 0,
            cylinder: Some(Cylinder { up, length: num(a, 5).unwrap_or(0.0) }),
            origin: vec3(a, 1).unwrap_or([0.0; 3]),
            radius: num(a, 4).unwrap_or(0.0),
            score: num(a, 6).unwrap_or(0.0),
            team_mask: int(a, 7).unwrap_or(0),
            curve: int(a, 9).unwrap_or(0),
            expires_ms: (timeout > 0.0).then(|| vm.time_ms + (timeout * 1000.0) as i64),
            enabled: true,
            entity: Some(arg(a, 11).clone()).filter(|v| !matches!(v, Value::Undefined)),
        };
        Ok(Value::Int(world.get_resource_or_insert_with(SpawnInfluencers::default).add(inf)))
    });
    vm.bind("removeinfluencer", false, |_, world, _, a| {
        if let (Ok(id), Some(mut s)) = (int(a, 0), world.get_resource_mut::<SpawnInfluencers>()) {
            s.remove(id);
        }
        Ok(Value::Undefined)
    });
    vm.bind("enableinfluencer", false, |_, world, _, a| {
        if let (Ok(id), Some(mut s)) = (int(a, 0), world.get_resource_mut::<SpawnInfluencers>())
            && let Some(i) = s.get_mut(id)
        {
            i.enabled = super::flag(a, 1, true);
        }
        Ok(Value::Undefined)
    });
    vm.bind("setinfluencerteammask", false, |_, world, _, a| {
        if let (Ok(id), Ok(mask), Some(mut s)) =
            (int(a, 0), int(a, 1), world.get_resource_mut::<SpawnInfluencers>())
            && let Some(i) = s.get_mut(id)
        {
            i.team_mask = mask;
        }
        Ok(Value::Undefined)
    });
}

#[cfg(test)]
mod tests {
    use super::*;

    fn sphere(origin: [f32; 3], radius: f32, score: f32, mask: i32, curve: i32) -> Influencer {
        Influencer {
            id: 0,
            cylinder: None,
            origin,
            radius,
            score,
            team_mask: mask,
            curve,
            expires_ms: None,
            enabled: true,
            entity: None,
        }
    }

    #[test]
    fn an_enemy_sphere_lowers_the_points_near_him_for_the_other_team() {
        // An axis player's enemy influencer counts for allies (bit 2), not axis (4).
        let list = vec![sphere([0.0; 3], 2600.0, -150.0, 2, 1)];
        let points = [[100.0, 0.0, 0.0], [2000.0, 0.0, 0.0], [3000.0, 0.0, 0.0]];
        let at = |i: &Influencer| Some(i.origin);
        let allies = score_points(&list, 2, &points, &at);
        assert!(allies[0] < allies[1] && allies[1] < allies[2]);
        assert_eq!(allies[2], 0.0);
        assert_eq!(score_points(&list, 4, &points, &at), vec![0.0; 3]);
    }

    #[test]
    fn a_cylinder_counts_along_its_axis_only() {
        let mut c = sphere([0.0, 0.0, -2000.0], 1200.0, -600.0, 2, 0);
        c.cylinder = Some(Cylinder { up: [0.0, 0.0, 1.0], length: 5000.0 });
        assert_eq!(influence_at(&c, c.origin, [100.0, 0.0, 0.0]), Some(-600.0));
        assert_eq!(influence_at(&c, c.origin, [100.0, 0.0, 4000.0]), None);
        assert_eq!(influence_at(&c, c.origin, [1300.0, 0.0, 0.0]), None);
    }

    #[test]
    fn timed_influencers_expire_and_disabled_ones_do_not_count() {
        let mut s = SpawnInfluencers::default();
        let mut a = sphere([0.0; 3], 100.0, -100.0, 2, 0);
        a.expires_ms = Some(1000);
        s.add(a);
        let id = s.add(sphere([0.0; 3], 100.0, -50.0, 2, 0));
        s.get_mut(id).unwrap().enabled = false;
        s.expire(1500);
        assert_eq!(s.list.len(), 1);
        assert_eq!(score_points(&s.list, 2, &[[0.0; 3]], &|i| Some(i.origin)), vec![0.0]);
    }
}
