//! Unit tests of the shapes: the axis-aligned rectangles of the square Silhouettes and the footprints, and (M5-V1) the
//! oriented Silhouettes, in rotated pairs touching, corner to edge and edge to edge.

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

/// A 56x40 Silhouette (the Seeker's) at `centre`, facing `degrees`.
fn seeker(centre: Vec2, degrees: f32) -> Shape {
    Shape::silhouette(ObjectKind::Seeker, centre, math::degrees_to_radians(degrees))
}

/// Its facing's direction and the direction across it (its Right), for a shape built by [`seeker`].
fn frame(degrees: f32) -> (Vec2, Vec2) {
    let u = math::facing_vector(math::degrees_to_radians(degrees));
    (u, Vec2::new(-u.y, u.x))
}

fn near(a: f32, b: f32) -> bool {
    (a - b).abs() < 1e-3
}

/// Plan decision T78, design change 0033 (`systems/units.md > UnitCollision > Silhouette Rotation`): only a Silhouette
/// that isn't square turns with the facing, its length along it. A square one is the same box whatever its facing.
#[test]
fn only_a_silhouette_that_isnt_square_turns_with_the_facing() {
    let at = Vec2::new(500.0, 500.0);
    let pk = Shape::silhouette(ObjectKind::Peacekeeper, at, 1.0);
    assert_eq!(pk, Shape::rect(at, Vec2::splat(12.0)), "square: never turned");
    assert!(!ObjectKind::Peacekeeper.def().silhouette_turns());
    for kind in [ObjectKind::Seeker, ObjectKind::PrayerMobile, ObjectKind::HandOfHeaven] {
        assert!(kind.def().silhouette_turns(), "{kind:?}");
    }
    // Facing east the Seeker is wide and flat, facing south tall and narrow; at 45° its box is 67.9 su square.
    assert_eq!(
        seeker(at, 0.0).bounds(),
        (at - Vec2::new(28.0, 20.0), at + Vec2::new(28.0, 20.0))
    );
    assert_eq!(seeker(at, 0.0).bound_half(), Vec2::new(28.0, 20.0));
    let south = seeker(at, 90.0).bound_half();
    assert!(near(south.x, 20.0) && near(south.y, 28.0), "{south}");
    let diagonal = seeker(at, 45.0).bound_half();
    assert!(near(diagonal.x, 48.0 * core::f32::consts::FRAC_1_SQRT_2) && near(diagonal.x, diagonal.y));
    assert!(near(diagonal.x, 33.941));
}

/// T78's geometry on rotated pairs, **edge to edge**: two Seekers side by side at 30°, their long sides facing each
/// other, are as far apart as their centres across the length less their widths; touching, they have no gap and
/// don't overlap, and a hundredth of a space unit nearer they do.
#[test]
fn two_turned_shapes_edge_to_edge() {
    let at = Vec2::new(500.0, 500.0);
    let (along, across) = frame(30.0);
    let a = seeker(at, 30.0);
    // 50 su apart across, and 10 su along: the long sides overlap over 46 su of their length, 10 su apart.
    let b = seeker(at + across * 50.0 + along * 10.0, 30.0);
    assert!(near(a.gap(&b), 10.0), "{}", a.gap(&b));
    assert!(near(b.gap(&a), 10.0), "the same from both sides");
    assert!(!a.overlaps(&b) && !b.overlaps(&a));
    // Touching along the whole side: 40 su apart across.
    let touching = seeker(at + across * 40.0, 30.0);
    assert!(a.gap(&touching) < 1e-3);
    assert!(!a.overlaps(&seeker(at + across * 40.01, 30.0)), "touching is free");
    assert!(a.overlaps(&seeker(at + across * 39.99, 30.0)));
    assert_eq!(a.gap(&seeker(at + across * 39.99, 30.0)), 0.0);
    // End to end along the length: 56 su apart.
    assert!(near(a.gap(&seeker(at + along * 66.0, 30.0)), 10.0));
}

/// T78's geometry, **corner to edge**: a Seeker at 45° whose corner points at a Guard's flat side, and a Guard whose
/// corner points at the Seeker's front edge. The gap is the corner's distance to the edge; touching, it is none.
#[test]
fn a_turned_shapes_corner_to_an_edge() {
    let at = Vec2::new(500.0, 500.0);
    let a = seeker(at, 45.0);
    let (along, across) = frame(45.0);
    // The Seeker's corner farthest east: its front corner on its left (`along × 28 − across × 20`).
    let corner = at + along * 28.0 - across * 20.0;
    assert!(near(corner.x - at.x, 33.941) && near(corner.y - at.y, 5.657));
    let guard_at = |gap: f32| unit(ObjectKind::Guard, corner.x + gap + 18.0, corner.y);
    assert!(near(a.gap(&guard_at(10.0)), 10.0), "{}", a.gap(&guard_at(10.0)));
    assert!(near(guard_at(10.0).gap(&a), 10.0));
    assert!(a.gap(&guard_at(0.0)) < 1e-3, "touching");
    assert!(!a.overlaps(&guard_at(0.01)) && a.overlaps(&guard_at(-0.01)));
    assert!(!guard_at(0.01).overlaps(&a) && guard_at(-0.01).overlaps(&a));
    // A Guard's corner 10 su in front of the middle of the Seeker's front edge, on its normal.
    let front = at + along * (28.0 + 10.0);
    let guard = unit(ObjectKind::Guard, front.x + 18.0, front.y + 18.0);
    assert!(near(a.gap(&guard), 10.0), "{}", a.gap(&guard));
    assert!(near(a.distance_to(front), 10.0));
}

/// Two long shapes crossed like a plus sign overlap although neither has a corner inside the other: the least corner
/// distance alone would see a gap. The separating-axis test sees no axis between them.
#[test]
fn crossed_shapes_overlap_with_no_corner_inside() {
    let at = Vec2::new(500.0, 500.0);
    let half = Vec2::new(32.0, 20.0);
    let a = Shape::oriented(at, half, math::facing_vector(math::degrees_to_radians(30.0)));
    let b = Shape::oriented(at, half, math::facing_vector(math::degrees_to_radians(120.0)));
    assert!(a.corners().iter().all(|&c| b.distance_to(c) > 0.0));
    assert!(b.corners().iter().all(|&c| a.distance_to(c) > 0.0));
    assert!(a.overlaps(&b) && b.overlaps(&a));
    assert_eq!(a.gap(&b), 0.0);
}

/// A point is measured from a turned shape's edge in the shape's own frame: in front of it, beside it, and off its
/// corner (3-4-5).
#[test]
fn a_point_is_measured_from_a_turned_shapes_edge() {
    let at = Vec2::new(500.0, 500.0);
    let a = seeker(at, 30.0);
    let (along, across) = frame(30.0);
    assert_eq!(a.distance_to(at + along * 10.0), 0.0, "inside");
    assert!(near(a.distance_to(at + along * 33.0), 5.0), "in front");
    assert!(near(a.distance_to(at - across * 25.0), 5.0), "beside");
    assert!(
        near(a.distance_to(at + along * 31.0 + across * 24.0), 5.0),
        "off the corner"
    );
}

/// Range from a turned target (`systems/combat.md > AttackAttributes > Range Measurement`, edge to edge): a Seeker
/// facing south is 40 su wide and 56 su long, so a footprint 256 su east of its long side and one 256 su north of its
/// short side are both 4 grid units from it, which they wouldn't be from the box it has facing east.
#[test]
fn a_turned_target_is_measured_from_its_long_side_and_its_short_side() {
    let at = Vec2::new(gu(20.0), gu(20.0));
    let south = Shape::oriented(at, Vec2::new(28.0, 20.0), Vec2::Y);
    let east_of = Shape::rect(Vec2::new(at.x + 20.0 + gu(4.0) + 32.0, at.y), Vec2::splat(32.0));
    let north_of = Shape::rect(Vec2::new(at.x, at.y - 28.0 - gu(4.0) - 32.0), Vec2::splat(32.0));
    assert_eq!(south.gap(&east_of), gu(4.0));
    assert_eq!(south.gap(&north_of), gu(4.0));
    let east = Shape::rect(at, Vec2::new(28.0, 20.0));
    assert_eq!(east.gap(&east_of), gu(4.0) - 8.0);
    assert_eq!(east.gap(&north_of), gu(4.0) + 8.0);
}

/// The sweep of a turned shape (`Shape::sweep`): it meets a standing box where the gap closes, not before; sliding
/// along it, or leaving it, isn't contact; pushing into it meets it at once. A Seeker facing south (exactly, so that
/// touching is exact) and one at 30°.
#[test]
fn a_turned_shape_sweeps_into_a_box_where_the_gap_closes() {
    let at = Vec2::new(500.0, 500.0);
    let south = Shape::oriented(at, Vec2::new(28.0, 20.0), Vec2::Y);
    let guard = |x: f32| unit(ObjectKind::Guard, at.x + 20.0 + x + 18.0, at.y);
    let (t, n) = south.sweep(Vec2::new(10.0, 0.0), &guard(5.0)).unwrap();
    assert_eq!(t, 0.5);
    assert_eq!(n.abs(), Vec2::X, "on the Guard's west face");
    assert_eq!(south.sweep(Vec2::new(4.0, 0.0), &guard(5.0)), None, "short of it");
    assert_eq!(south.sweep(Vec2::new(0.0, 10.0), &guard(0.0)), None, "sliding along");
    assert_eq!(south.sweep(Vec2::new(-10.0, 3.0), &guard(0.0)), None, "leaving");
    assert_eq!(
        south.sweep(Vec2::new(10.0, 0.0), &guard(0.0)).map(|(t, _)| t),
        Some(0.0),
        "pushing in"
    );
    assert_eq!(south.sweep(Vec2::ZERO, &guard(0.0)), None, "standing");
    // At 30° its farthest corner east reaches 28 cos 30° + 20 sin 30° = 34.25 su: it meets a Guard 5 su beyond.
    let turned = seeker(at, 30.0);
    let reach = turned.bound_half().x;
    assert!(near(reach, 34.249));
    let guard = unit(ObjectKind::Guard, at.x + reach + 5.0 + 18.0, at.y);
    let (t, _) = turned.sweep(Vec2::new(10.0, 0.0), &guard).unwrap();
    assert!(near(t, 0.5), "{t}");
    let met = turned.at(at + Vec2::new(10.0 * t, 0.0));
    assert!(met.gap(&guard) < 1e-3);
    assert!(!turned.at(at + Vec2::new(10.0 * t - 0.01, 0.0)).overlaps(&guard));
}

/// A turned Silhouette against the tiles is tested as it lies, not by its bounding box (`Shape::clear_on`,
/// `Shape::tile_contact`): a Seeker at 45° (facing south-east) whose front edge passes a solid tile's corner is clear
/// of it, though its box (67.9 su square) overlaps the tile, and 6 su further along its facing the corner is inside
/// it. Its move meets the tile where the front edge reaches the corner, and the map's edge where its rear corner
/// does: off the map is solid too.
#[test]
fn a_turned_shape_is_tested_against_the_tiles_as_it_lies() {
    use crate::nav::{Nav, NavClass, NavRules};
    let map = Map::from_ascii("....\n.M..\n....\n....").unwrap();
    let nav = Nav::build(&map, &[], &[(NavClass::Ground, 1)], NavRules::default());
    let grid = nav.grid(NavClass::Ground, 1).unwrap();
    // Tile (1, 1) is x 64..128 and y 64..128; its corner (64, 64) is 33.9 su along the facing from (40, 40), past
    // the front edge at 28.
    let shape = seeker(Vec2::new(40.0, 40.0), 45.0);
    assert!(
        !nav::box_clear(grid, shape.bound_half(), shape.centre()),
        "its box overlaps the tile"
    );
    assert!(shape.clear_on(grid));
    assert!(!shape.at(Vec2::new(46.0, 46.0)).clear_on(grid));
    // Along its facing the front edge closes the 5.94 su to the corner at 0.525 of an 11.3 su move.
    let t = shape.tile_contact(Vec2::new(8.0, 8.0), grid).unwrap();
    assert!(near(t, (33.941 - 28.0) / 11.314), "{t}");
    // Backing off, its rear corner, at y 6.06, leaves the map at 0.757 of the move.
    let t = shape.tile_contact(Vec2::new(-8.0, -8.0), grid).unwrap();
    assert!(near(t, 6.059 / 8.0), "{t}");
    // Across its facing it meets nothing.
    assert_eq!(shape.tile_contact(Vec2::new(4.0, -4.0), grid), None);
    // A square Silhouette is its box.
    let guard = unit(ObjectKind::Guard, 40.0, 40.0);
    assert!(guard.clear_on(grid) && nav::box_clear(grid, guard.bound_half(), guard.centre()));
    assert!(!guard.at(Vec2::new(50.0, 50.0)).clear_on(grid));
}
