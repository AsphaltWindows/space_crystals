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
//! DropOffReach is measured to that square (`factions/syndicate/units/agent.md > Gathering > Space Crystals`:
//! "within 8 su of the crystal hatch on Side B"). Each resource has its point ([`drop_point`]): a load of Supplies is
//! taken at the Supplies hatch on Side D (design change 0248; slice M4-I).
//!
//! **Which drop-off** ([`nearest`]): the nearest is measured to its crystal point, whatever the load: a Tunnel's
//! crystal hatch (`factions/syndicate/units/agent.md > Gathering rules`: "straight line to its crystal hatch"), an
//! Egress Node's centre. A worker carrying Supplies picks its Tunnel by the same measure, and delivers at the Supplies
//! hatch all the same (design change 0292 point 2: "'nearest Tunnel' for Supplies is measured to the crystal hatch,
//! the same measure as for crystals"; `factions/syndicate/units/agent.md > Gathering > Supplies`: "even though Supplies
//! are delivered at Side D").

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
    /// The middle of [`DropPoint::shape`]. A crystal point's is what the nearest drop-off is nearest by, whatever the
    /// load ([`nearest`]).
    pub centre: Vec2,
    /// The side of the hatch, for a drop-off with hatches: one worker unloads there at a time. `None` for one that
    /// takes loads anywhere, from any number of workers.
    pub side: Option<SideLetter>,
}

/// What a worker delivers: each has a hatch of its own at a Tunnel (`defs::DropOff::Hatches`).
pub use crate::defs::Resource;

/// The hatch square of `resource` at the drop-off `o`, a Tunnel as placed: the tile of the map it takes that resource
/// at (module docs; `tunnel::hatch_square`). `None` for an object that has no hatches. What the economy check measures
/// to (design change 0259: "from the patch's edge to the Tunnel's crystal hatch") and what the client draws.
pub fn hatch_square(o: &Object, resource: Resource) -> Option<TilePos> {
    let side = rule(o.kind)?.hatch(resource)?;
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

/// Where workers unload `resource` at a `kind` structure at `pos`, whose shape is `whole`, placed at `placement`:
/// the whole structure, or the resource's hatch square (module docs); `None` for a type that takes no loads.
pub fn drop_point(
    kind: ObjectKind,
    pos: Vec2,
    whole: Shape,
    placement: Option<&Placement>,
    resource: Resource,
) -> Option<DropPoint> {
    let anywhere = DropPoint {
        shape: whole,
        centre: pos,
        side: None,
    };
    let Some(side) = rule(kind)?.hatch(resource) else {
        return Some(anywhere);
    };
    let hatch = placement.and_then(|placement| hatch_tile(kind, placement, side));
    // A hatch structure without a footprint doesn't exist; it would take loads anywhere.
    Some(hatch.map_or(anywhere, |tile| DropPoint {
        shape: Shape::rect(tile.center(), Vec2::splat(SU_PER_GU / 2.0)),
        centre: tile.center(),
        side: Some(side),
    }))
}

/// [`drop_point`] of object `o`.
pub fn drop_point_of(o: &Object, resource: Resource) -> Option<DropPoint> {
    drop_point(o.kind, o.pos, Shape::of(o), o.placement.as_ref(), resource)
}

/// Where workers unload Space Crystals at a `kind` structure ([`drop_point`]).
pub fn crystal_point(kind: ObjectKind, pos: Vec2, whole: Shape, placement: Option<&Placement>) -> Option<DropPoint> {
    drop_point(kind, pos, whole, placement, Resource::Crystals)
}

/// [`crystal_point`] of object `o`.
pub fn crystal_point_of(o: &Object) -> Option<DropPoint> {
    drop_point_of(o, Resource::Crystals)
}

/// The nearest to `pos` of `drop_offs`, each given with its [`crystal_point`], for a load of either resource (module
/// docs, design change 0292 point 2): by the straight line to that point's centre, the first among equals, so the
/// lowest id of drop-offs given in id order. The one rule of the return leg's choice (`orders::gather`) and of the
/// removal path's (`Sim::reassign_drop_offs`).
pub fn nearest<T>(pos: Vec2, drop_offs: impl IntoIterator<Item = (T, DropPoint)>) -> Option<T> {
    (drop_offs.into_iter())
        .map(|(drop_off, crystals)| (drop_off, crystals.centre.distance_squared(pos)))
        .min_by(|a, b| a.1.total_cmp(&b.1))
        .map(|(drop_off, _)| drop_off)
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

    /// Plan decision T34: an Egress Node takes loads anywhere, a Tunnel at its crystal hatch on Side B and its
    /// Supplies hatch on Side D, and nothing else takes any.
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
        // Slice M4-I: Supplies at the hatch on Side D, tile (10, 22).
        let point = drop_point(
            ObjectKind::Tunnel,
            placement.center(),
            whole,
            Some(&placement),
            Resource::Supplies,
        );
        let tile = TilePos::new(10, 22);
        assert_eq!(
            point.map(|point| (point.side, point.centre, point.shape)),
            Some((Some(D), tile.center(), Shape::rect(tile.center(), Vec2::splat(32.0))))
        );
        let node = Placement::new(TilePos::new(4, 4), ObjectKind::EgressNode.def().footprint);
        let whole = Shape::footprint(&node);
        for resource in [Resource::Crystals, Resource::Supplies] {
            let point = drop_point(ObjectKind::EgressNode, node.center(), whole, Some(&node), resource).unwrap();
            assert_eq!((point.side, point.centre, point.shape), (None, node.center(), whole));
        }
    }

    /// [`nearest`]: by the straight line to each crystal point's centre, the first of equals (the lowest id, in id
    /// order), none of none.
    #[test]
    fn the_nearest_drop_off_is_by_its_crystal_points_centre_the_first_of_equals() {
        let point = |x: f32| {
            let centre = Vec2::new(x, 0.0);
            DropPoint {
                shape: Shape::rect(centre, Vec2::splat(32.0)),
                centre,
                side: Some(B),
            }
        };
        let near = |drop_offs: &[(u32, f32)]| nearest(Vec2::ZERO, drop_offs.iter().map(|&(id, x)| (id, point(x))));
        assert_eq!(near(&[(1, 100.0), (2, -50.0), (3, 60.0)]), Some(2));
        assert_eq!(near(&[(3, 50.0), (4, -50.0)]), Some(3), "the first of equals");
        assert_eq!(near(&[]), None);
    }
}
