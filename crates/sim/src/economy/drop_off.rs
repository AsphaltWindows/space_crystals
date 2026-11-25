//! Where a drop-off takes a load (plan decision T34: "the drop-off rule comes from the target's def",
//! `StructureDef::drop_off`): anywhere at an Egress Node (`factions/colonists/economy.md > Egress Node > Drop Off`),
//! at the resource's hatch at a Tunnel (`factions/syndicate/tunnels.md > Tunnel > SymmetryType - ABCD`, design
//! changes 0248 and 0255; slice M4-D).
//!
//! **The hatch square** ([`hatch_offset`]). "Each hatch is 1 grid square long, near the middle of its side (exact
//! square: the art's choice, mirrored with the Tunnel)": the tile of the side's row nearest its middle, and of the two
//! middle tiles of a side with an even number of them (the Tunnel's 4) **the one farther from Side A**, the entrance
//! and exit, so that the Agents standing at a hatch keep off Side A's paths. The Director's choice, standing in for
//! the art's. It turns and flips with the structure, since it is read off the sides as placed (`crate::sides`); the
//! client draws the hatches on the same squares (`object_render/tunnel.rs`).
//!
//! DropOffReach is measured to that square (`factions/syndicate/units.md > Agent > Gathering > Space Crystals`:
//! "within 8 su of the crystal hatch on Side B"), and so is the distance that picks the nearest Tunnel (`> Gathering
//! rules`: "straight line to its crystal hatch").

use glam::Vec2;

use crate::defs::{DropOff, ObjectKind};
use crate::object::{Object, Placement};
use crate::shape::Shape;
use crate::sides::{DIRECTIONS, SideLetter, sides};
use crate::units::{SU_PER_GU, TilePos};

/// Where workers unload their Space Crystals at a drop-off.
#[derive(Copy, Clone, Debug, PartialEq)]
pub struct DropPoint {
    /// What DropOffReach is measured to: the structure, or the hatch square.
    pub shape: Shape,
    /// The middle of [`DropPoint::shape`]: what the nearest drop-off is nearest by.
    pub centre: Vec2,
    /// The side of the hatch, for a drop-off with hatches: one worker unloads there at a time. `None` for one that
    /// takes loads anywhere, from any number of workers.
    pub side: Option<SideLetter>,
}

/// What a worker delivers: each has a hatch of its own at a Tunnel (`defs::DropOff::Hatches`).
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum Resource {
    Crystals,
    Supplies,
}

/// The hatch square of `resource` at the drop-off `o`, a Tunnel as placed: the tile of the map it takes that resource
/// at (module docs; `tunnel::hatch_square`). `None` for an object that has no hatches. What the economy check measures
/// to (design change 0259: "from the patch's edge to the Tunnel's crystal hatch") and what the client draws.
pub fn hatch_square(o: &Object, resource: Resource) -> Option<TilePos> {
    let DropOff::Hatches { crystals, supplies } = rule(o.kind)? else {
        return None;
    };
    let side = match resource {
        Resource::Crystals => crystals,
        Resource::Supplies => supplies,
    };
    hatch_tile(o.kind, o.placement.as_ref()?, side)
}

/// The drop-off rule of `kind`'s type, if workers deliver to it.
pub fn rule(kind: ObjectKind) -> Option<DropOff> {
    kind.def().structure.as_ref()?.drop_off
}

/// The hatch square of the side lettered `side` of a structure of `w` by `h` tiles as placed, whose sides face
/// north, east, south and west as `letters` says (`crate::sides::sides`): its tile, counted from the footprint's
/// top-left one (module docs). `None` if no side has that letter.
pub fn hatch_offset(letters: [SideLetter; 4], side: SideLetter, w: u32, h: u32) -> Option<(i32, i32)> {
    let facing = |letter| (letters.iter().position(|&l| l == letter)).map(|i| DIRECTIONS[i]);
    let out = facing(side)?;
    // Away from Side A, along the hatch's side: 0 for a structure without one, or with it on the same axis.
    let entrance = facing(SideLetter::A).unwrap_or((0, 0));
    let middle = |n: i32, entrance: i64| if n % 2 == 1 || entrance < 0 { n / 2 } else { n / 2 - 1 };
    let (w, h) = (w.max(1) as i32, h.max(1) as i32);
    Some(match out {
        (0, -1) => (middle(w, entrance.0), 0),
        (0, _) => (middle(w, entrance.0), h - 1),
        (-1, _) => (0, middle(h, entrance.1)),
        _ => (w - 1, middle(h, entrance.1)),
    })
}

/// The tile of the hatch on the side lettered `side` of a `kind` structure placed at `placement` (module docs).
pub fn hatch_tile(kind: ObjectKind, placement: &Placement, side: SideLetter) -> Option<TilePos> {
    let letters = sides(kind, placement.orientation())?;
    let (dx, dy) = hatch_offset(letters, side, placement.w, placement.h)?;
    Some(TilePos::new(placement.origin.x + dx, placement.origin.y + dy))
}

/// Where workers unload Space Crystals at a `kind` structure at `pos`, whose shape is `whole`, placed at
/// `placement`; `None` for a type that takes no loads.
pub fn crystal_point(kind: ObjectKind, pos: Vec2, whole: Shape, placement: Option<&Placement>) -> Option<DropPoint> {
    let anywhere = DropPoint {
        shape: whole,
        centre: pos,
        side: None,
    };
    match rule(kind)? {
        DropOff::Anywhere => Some(anywhere),
        DropOff::Hatches { crystals, .. } => {
            let hatch = placement.and_then(|placement| hatch_tile(kind, placement, crystals));
            // A hatch structure without a footprint doesn't exist; it would take loads anywhere.
            Some(hatch.map_or(anywhere, |tile| DropPoint {
                shape: Shape::rect(tile.center(), Vec2::splat(SU_PER_GU / 2.0)),
                centre: tile.center(),
                side: Some(crystals),
            }))
        }
    }
}

/// [`crystal_point`] of object `o`.
pub fn crystal_point_of(o: &Object) -> Option<DropPoint> {
    crystal_point(o.kind, o.pos, Shape::of(o), o.placement.as_ref())
}

#[cfg(test)]
mod tests {
    use super::SideLetter::{A, B, C, D};
    use super::*;
    use crate::object::Rotation;

    fn tunnel(rotation: Rotation, flip_h: bool) -> Placement {
        let unturned = Placement::new(TilePos::new(10, 20), ObjectKind::Tunnel.def().footprint);
        Placement {
            rotation,
            flip_h,
            ..unturned
        }
    }

    /// Design changes 0248 and 0255 point 6: the crystal hatch is one square of Side B, the Supplies hatch one of Side
    /// D, each the middle square farther from Side A; turned or flipped, the squares go with the sides.
    #[test]
    fn the_hatches_are_middle_squares_that_turn_and_flip_with_the_tunnel() {
        let hatch = |rotation, flip_h, side| {
            let tile = hatch_tile(ObjectKind::Tunnel, &tunnel(rotation, flip_h), side).unwrap();
            (tile.x - 10, tile.y - 20)
        };
        // Unturned: A on top, B on the right, C at the bottom, D on the left.
        assert_eq!(
            (hatch(Rotation::R0, false, B), hatch(Rotation::R0, false, D)),
            ((3, 2), (0, 2))
        );
        // Turned 90° clockwise: A on the right, B at the bottom, D on top.
        assert_eq!(
            (hatch(Rotation::R90, false, B), hatch(Rotation::R90, false, D)),
            ((1, 3), (1, 0))
        );
        assert_eq!(
            (hatch(Rotation::R180, false, B), hatch(Rotation::R180, false, D)),
            ((0, 1), (3, 1))
        );
        assert_eq!(
            (hatch(Rotation::R270, false, B), hatch(Rotation::R270, false, D)),
            ((2, 0), (2, 3))
        );
        // Flipped left and right: B on the left, D on the right.
        assert_eq!(
            (hatch(Rotation::R0, true, B), hatch(Rotation::R0, true, D)),
            ((0, 2), (3, 2))
        );
        assert_eq!(hatch(Rotation::R90, true, B), (1, 0));
        // The entrance and the back wall have squares of their own by the same rule; an odd side has one middle.
        assert_eq!(
            (hatch(Rotation::R0, false, A), hatch(Rotation::R0, false, C)),
            ((1, 0), (1, 3))
        );
        assert_eq!(hatch_offset([A, B, C, D], B, 3, 3), Some((2, 1)));
        assert_eq!(hatch_offset([A, A, A, A], B, 4, 4), None);
    }

    /// Plan decision T34: an Egress Node takes loads anywhere, a Tunnel at its crystal hatch on Side B, and nothing
    /// else takes any.
    #[test]
    fn only_the_egress_node_and_the_tunnel_take_loads() {
        for &kind in ObjectKind::ALL {
            let expected = match kind {
                ObjectKind::EgressNode => Some(DropOff::Anywhere),
                ObjectKind::Tunnel => Some(DropOff::Hatches {
                    crystals: B,
                    supplies: D,
                }),
                _ => None,
            };
            assert_eq!(rule(kind), expected, "{kind:?}");
        }
        let placement = tunnel(Rotation::R0, false);
        let whole = Shape::footprint(&placement);
        let point = crystal_point(ObjectKind::Tunnel, placement.center(), whole, Some(&placement)).unwrap();
        let tile = TilePos::new(13, 22);
        assert_eq!(
            (point.side, point.centre, point.shape),
            (Some(B), tile.center(), Shape::rect(tile.center(), Vec2::splat(32.0)))
        );
        let node = Placement::new(TilePos::new(4, 4), ObjectKind::EgressNode.def().footprint);
        let whole = Shape::footprint(&node);
        let point = crystal_point(ObjectKind::EgressNode, node.center(), whole, Some(&node)).unwrap();
        assert_eq!((point.side, point.centre, point.shape), (None, node.center(), whole));
    }
}
