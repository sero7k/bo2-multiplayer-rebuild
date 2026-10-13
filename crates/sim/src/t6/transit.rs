//! Tranzit (zm_transit): the bus is solid to players.
//!
//! The bus is a script vehicle (`veh_t6_civ_bus_zombie`) made a moving
//! platform by its scripts. Script models do not block player movement
//! here, so its body is kept as turned boxes (presence's blockers) taken
//! from the model's own collision: straight-down rays through it (t6clip
//! `T6CLIP_RAY`, runs/busray*.txt in the transit lane) for the floor, the
//! door wells' steps, the raised back, the dash and the rounded roof, and
//! its surfaces (t6props `T6PROPS_MODEL`) for the walls' inner faces at
//! y +-69 and the hatch at x 196..255, y +-29. The door gaps (x 37..81 and
//! 309..353 on its right, -y) and the hatch stay open here: their own
//! brush models (`bus_door_blocker`, the hatch) close them while the
//! scripts keep them solid.

/// The aisle floor's top in the bus's own space (its origin is at the road).
pub(crate) const BUS_FLOOR: f32 = 37.84;

/// The inside of the bus (forward x, left y) between its walls.
pub(crate) const BUS_INSIDE: ([f32; 2], [f32; 2]) = ([-89.0, -69.0], [375.0, 69.0]);

/// The door wells on its right (-y): forward ranges, and the aisle's edge
/// they are cut in from (its steps go down past it to the road).
const BUS_DOOR_WELLS: [[f32; 2]; 2] = [[37.0, 81.0], [309.0, 353.0]];
const BUS_AISLE_EDGE: f32 = -47.75;

/// A point of the bus's own space over a door well's steps.
pub(crate) fn in_door_well(l: [f32; 3]) -> bool {
    l[1] < BUS_AISLE_EDGE && BUS_DOOR_WELLS.iter().any(|w| l[0] >= w[0] && l[0] <= w[1])
}

type Box3 = ([f32; 3], [f32; 3]);

const BUS: &[Box3] = &[
    // Under the floor, but for the door wells.
    ([-96.0, -72.0, 0.0], [37.0, 72.0, BUS_FLOOR]),
    ([81.0, -72.0, 0.0], [309.0, 72.0, BUS_FLOOR]),
    ([353.0, -72.0, 0.0], [383.0, 72.0, BUS_FLOOR]),
    // The door wells: the aisle, then two steps down to the road.
    ([37.0, -47.75, 0.0], [81.0, 72.0, BUS_FLOOR]),
    ([37.0, -60.5, 0.0], [81.0, -47.75, 28.93]),
    ([37.0, -72.0, 0.0], [81.0, -60.5, 20.02]),
    ([309.0, -47.75, 0.0], [353.0, 72.0, BUS_FLOOR]),
    ([309.0, -60.5, 0.0], [353.0, -47.75, 28.93]),
    ([309.0, -72.0, 0.0], [353.0, -60.5, 20.02]),
    // The raised back (a jump up from the aisle, too low under the roof
    // to stand on past its edge), and the dash in front.
    ([-96.0, -69.0, BUS_FLOOR], [-47.0, 69.0, 66.8]),
    ([-96.0, -69.0, 66.8], [-64.0, 69.0, 81.0]),
    ([359.0, -69.0, BUS_FLOOR], [375.0, 69.0, 78.0]),
    // The hood in front.
    ([383.0, -72.0, 0.0], [449.0, 72.0, 51.0]),
    // Left wall; right wall around the doors, and over them: up to where
    // the roof curves down to its edge.
    ([-96.0, 69.0, BUS_FLOOR], [383.0, 72.0, 131.0]),
    ([-96.0, -72.0, BUS_FLOOR], [37.0, -69.0, 131.0]),
    ([81.0, -72.0, BUS_FLOOR], [309.0, -69.0, 131.0]),
    ([353.0, -72.0, BUS_FLOOR], [383.0, -69.0, 131.0]),
    ([37.0, -72.0, 120.0], [81.0, -69.0, 131.0]),
    ([309.0, -72.0, 120.0], [353.0, -69.0, 131.0]),
    // Back and front.
    ([-96.0, -69.0, BUS_FLOOR], [-89.0, 69.0, 131.0]),
    ([375.0, -69.0, BUS_FLOOR], [383.0, 69.0, 131.0]),
    // The roof, rounded: 147.6 along its middle (|y| < 38), 143.5 then
    // 136 toward its edges, lower over the back and the front; around the
    // hatch.
    ([-50.0, -38.0, 142.0], [196.0, 38.0, 147.6]),
    ([255.0, -38.0, 142.0], [354.0, 38.0, 147.6]),
    ([196.0, 29.0, 142.0], [255.0, 38.0, 147.6]),
    ([196.0, -38.0, 142.0], [255.0, -29.0, 147.6]),
    ([-50.0, 38.0, 140.0], [354.0, 56.0, 143.5]),
    ([-50.0, -56.0, 140.0], [354.0, -38.0, 143.5]),
    ([-50.0, 56.0, 129.0], [354.0, 72.0, 136.0]),
    ([-50.0, -72.0, 129.0], [354.0, -56.0, 136.0]),
    ([-70.0, -38.0, 136.0], [-50.0, 38.0, 141.0]),
    ([-70.0, 38.0, 134.0], [-50.0, 56.0, 139.0]),
    ([-70.0, -56.0, 134.0], [-50.0, -38.0, 139.0]),
    ([-70.0, 56.0, 128.0], [-50.0, 72.0, 133.0]),
    ([-70.0, -72.0, 128.0], [-50.0, -56.0, 133.0]),
    ([-96.0, -72.0, 122.0], [-70.0, 72.0, 128.0]),
    ([354.0, -38.0, 132.0], [383.0, 38.0, 138.0]),
    ([354.0, 38.0, 131.0], [383.0, 72.0, 136.0]),
    ([354.0, -72.0, 131.0], [383.0, -38.0, 136.0]),
];

/// Where a player rides a platform model, in its own space (forward, left,
/// up): the bus from its back to its hood, wall to wall, its floor to
/// above its roof (he rides on the roof too). His client keeps his view
/// with the platform it draws while he stands there.
pub fn platform_ride_box(model: &str) -> Option<([f32; 3], [f32; 3])> {
    match model {
        "veh_t6_civ_bus_zombie" => Some(([-96.0, -76.0, 0.0], [449.0, 76.0, 240.0])),
        _ => None,
    }
}

/// A platform model's solid boxes in its own space (forward, left, up).
pub(crate) fn platform_boxes(model: &str) -> &'static [Box3] {
    match model {
        "veh_t6_civ_bus_zombie" => BUS,
        _ => &[],
    }
}

/// The bus ladder's model, shown on the bus once the ladder is built
/// (`zm_transit_openings` busladderthink; it is notsolid there).
const BUS_LADDER_MODEL: &str = "com_stepladder_large_closed";

/// The bus ladder in the bus's space: on the left side's outer face at the
/// back, where `tag_ladder_attach` (-48.6, 72.8, 53.5) and the
/// `ladder_mantle` clip are (that clip goes when the ladder is built), as
/// wide as the stepladder, from the road to the roof's top. Neither the map
/// nor the bus or ladder models carry a ladder surface, yet the scripts
/// count a player on it (`isonladder` in busupdateplayers): this is it.
/// Climbing past its top steps him onto the roof.
const BUS_LADDER: Box3 = ([-66.0, 72.0, 0.0], [-38.0, 76.0, 152.0]);

/// A ladder face's surface flags: the ladder bit, sounding as metal.
const LADDER_FACE: u32 = movement_iw4::SURF_LADDER | movement_iw4::LADDER_SURFACE_FLAGS;

/// The bus's ladder box once a shown stepladder is linked to it.
pub(crate) fn platform_ladder(zm: &super::Zm, bus: u32, model: &str) -> Option<(Box3, u32)> {
    if model != "veh_t6_civ_bus_zombie" {
        return None;
    }
    zm.ents
        .values()
        .any(|e| e.model == BUS_LADDER_MODEL && !e.hidden && e.link.is_some_and(|l| l.0 == bus))
        .then_some((BUS_LADDER, LADDER_FACE))
}
