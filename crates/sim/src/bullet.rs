use weapon_iw4::WeaponCombatFacts;

pub fn bullet_damage_at_distance(facts: &WeaponCombatFacts, dist: f32) -> i32 {
    if let Some(d) = damage_over_points(&facts.damage_points, dist) {
        return d.max(0);
    }
    let base = facts.damage;
    let min = if facts.min_damage > 0 {
        facts.min_damage
    } else {
        base
    };
    let mut damage = base;
    let range = facts.min_damage_range - facts.max_damage_range;
    if base != min && range != 0.0 {
        if facts.max_damage_range <= dist {
            if facts.min_damage_range <= dist {
                damage = min;
            } else {
                let lerp = (dist - facts.max_damage_range) / range;
                let lerp = lerp.clamp(0.0, 1.0);
                damage = ((base as f32) * (1.0 - lerp) + (min as f32) * lerp) as i32;
            }
        }
    }
    damage.max(0)
}

/// Black Ops II's damage at a distance from its (range, damage) points
/// (points with no range are unused): the first point's damage up to its
/// range, straight between neighbouring points, the last's beyond. None
/// when the gun has no points (the other games' guns).
pub fn damage_over_points(points: &[[f32; 2]; 6], dist: f32) -> Option<i32> {
    let mut used = points.iter().filter(|p| p[0] > 0.0);
    let first = used.next()?;
    if dist <= first[0] {
        return Some(first[1] as i32);
    }
    let mut prev = *first;
    for p in used {
        if dist <= p[0] {
            let span = p[0] - prev[0];
            let t = if span > 0.0 { (dist - prev[0]) / span } else { 1.0 };
            return Some((prev[1] + (p[1] - prev[1]) * t) as i32);
        }
        prev = *p;
    }
    Some(prev[1] as i32)
}

pub fn angles_to_forward(angles: [f32; 3]) -> [f32; 3] {
    let yaw = angles[1].to_radians();
    let pitch = angles[0].to_radians();
    let cy = yaw.cos();
    let sy = yaw.sin();
    let sp = pitch.sin();
    let cp = pitch.cos();
    [cp * cy, cp * sy, -sp]
}

#[cfg(test)]
mod tests {
    use super::damage_over_points;

    /// The MP7's points as Black Ops II's common_mp has them.
    const MP7: [[f32; 2]; 6] =
        [[800.0, 33.0], [801.0, 22.0], [1100.0, 22.0], [0.0, 0.0], [0.0, 0.0], [1100.0, 18.0]];
    /// The SWAT-556's (burst): five tiers.
    const SARITCH: [[f32; 2]; 6] =
        [[1000.0, 50.0], [1001.0, 50.0], [1500.0, 50.0], [1501.0, 49.0], [3000.0, 49.0], [3001.0, 40.0]];

    #[test]
    fn black_ops_ii_damage_steps_between_its_points() {
        assert_eq!(damage_over_points(&MP7, 10.0), Some(33));
        assert_eq!(damage_over_points(&MP7, 800.0), Some(33));
        assert_eq!(damage_over_points(&MP7, 900.0), Some(22));
        assert_eq!(damage_over_points(&MP7, 1099.0), Some(22));
        assert_eq!(damage_over_points(&MP7, 5000.0), Some(18));
        assert_eq!(damage_over_points(&SARITCH, 1400.0), Some(50));
        assert_eq!(damage_over_points(&SARITCH, 2000.0), Some(49));
        assert_eq!(damage_over_points(&SARITCH, 4000.0), Some(40));
        assert_eq!(damage_over_points(&[[0.0; 2]; 6], 100.0), None);
    }
}
