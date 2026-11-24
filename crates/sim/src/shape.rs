//! The shapes distances are measured between — `systems/combat.md > AttackAttributes > Range Measurement` (design
//! change 0031), `systems/controls/behaviors.md > MovingToObject > Proximity` (design change 0032).
//!
//! Range, MinRange and proximity are measured **edge to edge**: the shortest distance between two objects' shapes,
//! the gap between them. A unit's shape is its Silhouette (`systems/units.md > Unit > Silhouette`), a structure's
//! or map resource's its footprint.
//!
//! **Axis-aligned for now.** Design change 0033 makes Silhouettes rotate with the unit's facing
//! (`systems/units.md > UnitCollision > Silhouette Rotation`); that waits for the vehicles milestone, with oriented
//! collision. Until then a [`Shape`] is an axis-aligned rectangle, like the collision box (`docs/movement.md > 3`).
//! Everything that measures goes through [`Shape`]'s constructors and methods, and [`Shape::silhouette`] already
//! takes the facing, so oriented boxes change this module only.

use glam::Vec2;

use crate::defs::ObjectKind;
use crate::object::{Object, Placement};
use crate::units::SU_PER_GU;

/// An object's shape on the map, space units: a rectangle (axis-aligned for now, see the module docs).
#[derive(Copy, Clone, Debug, PartialEq)]
pub struct Shape {
    centre: Vec2,
    /// Half its width and height.
    half: Vec2,
}

impl Shape {
    /// The axis-aligned rectangle around `centre` with half extents `half`.
    pub fn rect(centre: Vec2, half: Vec2) -> Self {
        Self { centre, half }
    }

    /// The Silhouette of a unit of `kind` at `pos`, facing `facing` (radians). The facing is ignored until
    /// Silhouettes rotate (design change 0033, the vehicles milestone); a kind without a Silhouette is a point.
    pub fn silhouette(kind: ObjectKind, pos: Vec2, _facing: f32) -> Self {
        Self::rect(pos, kind.def().half_extents().unwrap_or(Vec2::ZERO))
    }

    /// The footprint of a tile-based object: its whole tiles.
    pub fn footprint(placement: &Placement) -> Self {
        let half = Vec2::new(placement.w as f32, placement.h as f32) * (SU_PER_GU / 2.0);
        Self::rect(placement.origin.corner() + half, half)
    }

    /// `o`'s shape: its footprint if it is tile-based, else its Silhouette where it stands.
    pub fn of(o: &Object) -> Self {
        match &o.placement {
            Some(placement) => Self::footprint(placement),
            None => Self::silhouette(o.kind, o.pos, o.facing),
        }
    }

    pub fn centre(&self) -> Vec2 {
        self.centre
    }

    /// The corners `(lo, hi)` of the axis-aligned box around the shape: for what works on tiles (vision).
    pub fn bounds(&self) -> (Vec2, Vec2) {
        (self.centre - self.half, self.centre + self.half)
    }

    /// `> Range Measurement`: the shortest distance between this shape and `other`, the gap between their edges.
    /// 0 when they touch or overlap.
    pub fn gap(&self, other: &Shape) -> f32 {
        let apart = (self.centre - other.centre).abs() - (self.half + other.half);
        apart.max(Vec2::ZERO).length()
    }

    /// `> Range Measurement` (ground-location targets): the shortest distance from this shape to `point`; 0 inside
    /// it.
    pub fn distance_to(&self, point: Vec2) -> f32 {
        let apart = (self.centre - point).abs() - self.half;
        apart.max(Vec2::ZERO).length()
    }
}

/// `systems/combat.md > AttackAttributes > Range Measurement`: the gap between two shapes ([`Shape::gap`]).
pub fn gap(a: &Shape, b: &Shape) -> f32 {
    a.gap(b)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::defs::Footprint;
    use crate::units::{TilePos, gu};

    fn unit(kind: ObjectKind, x: f32, y: f32) -> Shape {
        Shape::silhouette(kind, Vec2::new(x, y), 0.0)
    }

    #[test]
    fn the_gap_runs_between_the_nearest_edges() {
        // A Peacekeeper (24 su) and a Guard (36 su), centres 100 su apart: 100 − 12 − 18.
        let (pk, guard) = (
            unit(ObjectKind::Peacekeeper, 0.0, 0.0),
            unit(ObjectKind::Guard, 100.0, 0.0),
        );
        assert_eq!(pk.gap(&guard), 70.0);
        assert_eq!(gap(&guard, &pk), 70.0, "the same from both sides");
        // Past each other's corners: 3-4-5 between the corners.
        let diagonal = unit(ObjectKind::Guard, 30.0 + 30.0, 30.0 + 40.0);
        assert_eq!(pk.gap(&diagonal), 50.0);
        // Overlapping on one axis, the other alone counts.
        assert_eq!(pk.gap(&unit(ObjectKind::Guard, 5.0, 50.0)), 20.0);
        // Touching and overlapping shapes have no gap.
        assert_eq!(pk.gap(&unit(ObjectKind::Guard, 30.0, 0.0)), 0.0);
        assert_eq!(pk.gap(&unit(ObjectKind::Guard, 10.0, 3.0)), 0.0);
    }

    /// Design change 0031's Verification: a unit attacking a 4x4 structure stops 5 tiles from its wall, not from
    /// its centre, and a big attacker's own body doesn't eat into its range. No structure can be attacked yet, so
    /// this checks the measure on a 4x4 footprint.
    #[test]
    fn a_footprint_is_measured_from_its_wall() {
        let big = Shape::footprint(&Placement::new(TilePos::new(10, 10), Footprint::Tiles { w: 4, h: 4 }));
        assert_eq!(big.centre(), Vec2::new(gu(12.0), gu(12.0)));
        assert_eq!(big.bounds(), (Vec2::splat(gu(10.0)), Vec2::splat(gu(14.0))));
        // A Peacekeeper whose east edge is 5 tiles west of the west wall.
        let pk = unit(ObjectKind::Peacekeeper, gu(5.0) - 12.0, gu(12.0));
        assert_eq!(pk.gap(&big), gu(5.0));
        assert_eq!(
            big.gap(&pk),
            gu(5.0),
            "and the big shape reaches as far from its own wall"
        );
        // Along the wall the gap stays the same; past its corner it grows.
        assert_eq!(
            unit(ObjectKind::Peacekeeper, gu(5.0) - 12.0, gu(10.5)).gap(&big),
            gu(5.0)
        );
        assert!(unit(ObjectKind::Peacekeeper, gu(5.0) - 12.0, gu(9.0)).gap(&big) > gu(5.0));
        // Melee is a gap of about 0: touching the wall.
        assert_eq!(unit(ObjectKind::Peacekeeper, gu(10.0) - 12.0, gu(11.0)).gap(&big), 0.0);
    }

    #[test]
    fn a_point_is_measured_from_the_shapes_edge() {
        let guard = unit(ObjectKind::Guard, 100.0, 100.0);
        assert_eq!(guard.distance_to(Vec2::new(100.0, 100.0)), 0.0, "inside");
        assert_eq!(guard.distance_to(Vec2::new(118.0, 90.0)), 0.0, "on its edge");
        assert_eq!(guard.distance_to(Vec2::new(168.0, 100.0)), 50.0);
        assert_eq!(
            guard.distance_to(Vec2::new(100.0 - 18.0 - 3.0, 100.0 - 18.0 - 4.0)),
            5.0
        );
    }

    #[test]
    fn an_objects_shape_is_its_footprint_or_its_silhouette() {
        let patch = Placement::new(TilePos::new(2, 3), Footprint::Tiles { w: 1, h: 1 });
        assert_eq!(
            Shape::footprint(&patch).bounds(),
            (Vec2::new(128.0, 192.0), Vec2::new(192.0, 256.0))
        );
        assert_eq!(
            unit(ObjectKind::Peacekeeper, 50.0, 60.0).bounds(),
            (Vec2::new(38.0, 48.0), Vec2::new(62.0, 72.0))
        );
    }
}
