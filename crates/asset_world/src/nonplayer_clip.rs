//! bo2mp (Black Ops maps: invisible walls): clip brushes that only block
//! vehicles, missiles, AI or sight must not stop a player.

use crate::{ClipCollision, MASK_PLAYER_SOLID};

/// What [`strip_nonplayer_clip_contents`] found and did.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct NonPlayerClipReport {
    /// Contents bits in the player mask that only clip materials which do
    /// not block players carry (vehicle, missile, AI, sight ... clips).
    pub stripped_bits: u32,
    /// Brushes that blocked players only through those bits (and now do
    /// not), and brushes that keep blocking.
    pub brushes_freed: usize,
    pub brushes_player_solid: usize,
    /// Mesh triangles likewise.
    pub tris_freed: usize,
    /// Each clip material (`clip*`) and its contents, as the map has them.
    pub clip_materials: Vec<(String, u32)>,
}

impl NonPlayerClipReport {
    pub fn report_line(&self) -> String {
        format!(
            "clip contents: player mask {MASK_PLAYER_SOLID:#010x}; bits only non-player clips carry \
             {:#010x}: {} brushes and {} mesh triangles no longer block players ({} brushes still \
             do); clip materials {:?}",
            self.stripped_bits,
            self.brushes_freed,
            self.tris_freed,
            self.brushes_player_solid,
            self.clip_materials
                .iter()
                .map(|(name, contents)| format!("{name}={contents:#x}"))
                .collect::<Vec<_>>()
        )
    }
}

/// A clip material that never blocks a player (named for what it does
/// block: vehicles, missiles, AI, sight, items).
fn nonplayer_clip_name(name: &str) -> bool {
    let name = name.to_ascii_lowercase();
    [
        "clip_vehicle",
        "clip_missile",
        "clip_ai",
        "clip_monster",
        "clip_nosight",
        "clip_sight",
        "clip_item",
        "clip_corpse",
        "clip_shot",
    ]
    .iter()
    .any(|n| name.contains(n))
        && !name.contains("player")
}

/// A map's brushes carry their side materials' contents, and the player
/// trace masks them with [`MASK_PLAYER_SOLID`]. A contents bit that only
/// vehicle, missile, AI or sight clip materials carry (no material that
/// blocks players has it) must not stop a player: such a bit inside the
/// player mask (a game's own layout of the bits differing from the mask's)
/// is cleared from the brushes and mesh triangles. Bits any other material
/// carries are left alone, so this never opens a wall a player clip,
/// `clip` or a visible surface makes.
pub fn strip_nonplayer_clip_contents(clip: &mut ClipCollision) -> NonPlayerClipReport {
    let mut nonplayer = 0u32;
    let mut player = 0u32;
    let mut clip_materials = Vec::new();
    for m in &clip.materials {
        if m.name.to_ascii_lowercase().contains("clip") {
            clip_materials.push((m.name.clone(), m.content_flags));
        }
        if nonplayer_clip_name(&m.name) {
            nonplayer |= m.content_flags;
        } else {
            player |= m.content_flags;
        }
    }
    // Solid and glass always block (and are every visible surface's).
    let stripped_bits = nonplayer & MASK_PLAYER_SOLID & !player & !0x11;
    let mut report = NonPlayerClipReport {
        stripped_bits,
        clip_materials,
        ..NonPlayerClipReport::default()
    };
    for brush in &mut clip.brushes {
        if brush.contents & MASK_PLAYER_SOLID == 0 {
            continue;
        }
        if stripped_bits != 0 && brush.contents & MASK_PLAYER_SOLID & !stripped_bits == 0 {
            report.brushes_freed += 1;
        }
        brush.contents &= !stripped_bits;
        if brush.contents & MASK_PLAYER_SOLID != 0 {
            report.brushes_player_solid += 1;
        }
    }
    if stripped_bits != 0 {
        let mesh = std::sync::Arc::make_mut(&mut clip.mesh);
        for flags in &mut mesh.tri_content_flags {
            if *flags & MASK_PLAYER_SOLID != 0 && *flags & MASK_PLAYER_SOLID & !stripped_bits == 0 {
                report.tris_freed += 1;
            }
            *flags &= !stripped_bits;
        }
    }
    report
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{ClipBrush, ClipMapMaterial};

    fn clip_with(materials: &[(&str, u32)], brushes: &[u32]) -> ClipCollision {
        ClipCollision {
            materials: materials
                .iter()
                .map(|(name, contents)| ClipMapMaterial {
                    name: (*name).to_owned(),
                    surface_flags: 0,
                    content_flags: *contents,
                })
                .collect(),
            brushes: brushes
                .iter()
                .map(|contents| ClipBrush {
                    planes: Vec::new(),
                    contents: *contents,
                    plane_surface_flags: Vec::new(),
                    glass_encoded: 0,
                })
                .collect(),
            ..ClipCollision::default()
        }
    }

    #[test]
    fn a_bit_only_vehicle_clip_carries_stops_blocking() {
        // A vehicle clip whose bit sits inside the player mask.
        let mut clip = clip_with(
            &[
                ("clip_player", 0x1_0000),
                ("clip_vehicle", 0x80_0000),
                ("concrete", 0x1),
            ],
            &[0x80_0000, 0x1_0000, 0x1, 0x80_0001],
        );
        let r = strip_nonplayer_clip_contents(&mut clip);
        assert_eq!(r.stripped_bits, 0x80_0000);
        assert_eq!(r.brushes_freed, 1);
        assert_eq!(r.brushes_player_solid, 3);
        assert_eq!(clip.brushes[0].contents & MASK_PLAYER_SOLID, 0);
        assert_eq!(clip.brushes[3].contents, 0x1);
    }

    #[test]
    fn a_bit_a_player_clip_shares_is_kept() {
        let mut clip = clip_with(
            &[("clip_player", 0x1_0000), ("clip_vehicle", 0x1_0200)],
            &[0x1_0200],
        );
        let r = strip_nonplayer_clip_contents(&mut clip);
        assert_eq!(r.stripped_bits, 0);
        assert_eq!(clip.brushes[0].contents, 0x1_0200);
    }
}
