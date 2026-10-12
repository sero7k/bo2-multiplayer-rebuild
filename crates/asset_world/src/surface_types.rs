//! bo2mp: other games' surface types read in Black Ops II's list.
//!
//! Every Call of Duty here numbers its surfaces alike from `default` (0) to
//! `paintedmetal` (28). After that they part: Modern Warfare 2 and 3 have
//! `riotshield` (29) and `slush` (30); Black Ops and Black Ops II have
//! `player` (29), `tallgrass` (30) and `riotshield` (31); Call of Duty 4 and
//! World at War stop at 28. Under Black Ops II's rules every map's surface
//! flags are read in BO2's list (footsteps, impacts, penetration), so an
//! Infinity Ward map's last two are moved to BO2's: its riot shield to
//! BO2's `riotshield`, its slush (BO2 has none) to `snow`; read as they
//! were, slush walked as tall grass and a shield as a body.

use crate::ClipCollision;

const SURF_TYPE_SHIFT: u32 = 20;
const SURF_TYPE_MASK: u32 = 0x1f << SURF_TYPE_SHIFT;

const IW_RIOTSHIELD: u32 = 29;
const IW_SLUSH: u32 = 30;
const T6_SNOW: u32 = 19;
const T6_RIOTSHIELD: u32 = 31;

/// An Infinity Ward map's surface flags in Black Ops II's surface list.
#[must_use]
pub fn iw_surface_flags_to_t6(flags: u32) -> u32 {
    let ty = (flags & SURF_TYPE_MASK) >> SURF_TYPE_SHIFT;
    let to = match ty {
        IW_RIOTSHIELD => T6_RIOTSHIELD,
        IW_SLUSH => T6_SNOW,
        _ => return flags,
    };
    (flags & !SURF_TYPE_MASK) | (to << SURF_TYPE_SHIFT)
}

/// Every surface flag of the map's collision through `map` (brush sides,
/// the mesh's triangles, its materials, static models' collision); how many
/// changed.
pub fn remap_clip_surface_flags(clip: &mut ClipCollision, map: impl Fn(u32) -> u32) -> usize {
    let mut changed = 0usize;
    let mut apply = |f: &mut u32| {
        let to = map(*f);
        if to != *f {
            *f = to;
            changed += 1;
        }
    };
    for brush in &mut clip.brushes {
        brush.plane_surface_flags.iter_mut().for_each(&mut apply);
    }
    for material in &mut clip.materials {
        apply(&mut material.surface_flags);
    }
    let needs_mesh = clip.mesh.tri_surface_flags.iter().any(|&f| map(f) != f);
    if needs_mesh {
        std::sync::Arc::make_mut(&mut clip.mesh)
            .tri_surface_flags
            .iter_mut()
            .for_each(&mut apply);
    }
    for model in &mut clip.static_models {
        for surf in &mut model.model.coll.surfs {
            apply(&mut surf.surf_flags);
        }
    }
    changed
}

/// How many of the map's collision surfaces carry each surface type (diag).
#[must_use]
pub fn clip_surface_type_census(clip: &ClipCollision) -> [usize; 32] {
    let mut out = [0usize; 32];
    let mut count = |f: u32| out[((f & SURF_TYPE_MASK) >> SURF_TYPE_SHIFT) as usize] += 1;
    clip.brushes
        .iter()
        .flat_map(|b| b.plane_surface_flags.iter())
        .for_each(|&f| count(f));
    clip.mesh.tri_surface_flags.iter().for_each(|&f| count(f));
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shared_surfaces_stay() {
        for ty in 0..=28u32 {
            let f = (ty << 20) | 0x0200_0010;
            assert_eq!(iw_surface_flags_to_t6(f), f);
        }
    }

    #[test]
    fn infinity_ward_slush_and_shield_move_to_black_ops_ii_names() {
        let other_bits = 0x0400_0004;
        assert_eq!(iw_surface_flags_to_t6((30 << 20) | other_bits), (19 << 20) | other_bits);
        assert_eq!(iw_surface_flags_to_t6((29 << 20) | other_bits), (31 << 20) | other_bits);
    }
}
