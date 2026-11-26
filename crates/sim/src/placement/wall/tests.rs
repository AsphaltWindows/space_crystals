//! The wall round a footprint (design change 0339): its points, the search along it and the way round.

use super::*;
use crate::units::TilePos;

/// A 2 × 2 footprint, a Storage's before design change 0380, on tiles (4, 4) to (5, 5): x and y 256..384, centred on
/// (320, 320).
fn storage() -> Placement {
    Placement {
        w: 2,
        h: 2,
        ..Placement::new(TilePos::new(4, 4), crate::defs::Footprint::Tiles { w: 1, h: 1 })
    }
}

/// A Recruit's wall round [`storage`]: its 24 su box 4 su off the footprint, the loop 64 + 12 + 4 = 80 su from the
/// centre along each axis, x and y 240..400.
fn wall() -> Wall {
    Wall::around(&storage(), Vec2::splat(12.0), 4.0)
}

/// The loop is counted from the middle of its east side, clockwise on screen (+y is south): its corners, a side's
/// length (160 su) apart, and back to the start 640 su on; a point on it is as far along as it was taken from.
#[test]
fn the_wall_is_counted_clockwise_from_the_east() {
    let w = wall();
    assert_eq!(w.length(), 640.0);
    let points = [0.0, 80.0, 240.0, 400.0, 560.0, 640.0].map(|t| w.point(t));
    assert_eq!(
        points,
        [
            (400.0, 320.0),
            (400.0, 400.0),
            (240.0, 400.0),
            (240.0, 240.0),
            (400.0, 240.0),
            (400.0, 320.0)
        ]
        .map(|(x, y)| Vec2::new(x, y))
    );
    for t in [0.0, 30.0, 80.0, 150.0, 300.0, 401.0, 600.0] {
        assert_eq!(w.along(w.point(t)), t, "{t}");
    }
}

/// The point of the wall nearest a unit: outside the loop, the unit brought onto it along each axis (beside a side,
/// straight in; past a corner, the corner); inside it, on the footprint too, its nearest side, of two as near the
/// first clockwise from the east.
#[test]
fn the_nearest_point_is_the_part_of_the_footprint_closest_to_the_unit() {
    let w = wall();
    let nearest = |x: f32, y: f32| w.point(w.along(Vec2::new(x, y)));
    assert_eq!(nearest(-500.0, 300.0), Vec2::new(240.0, 300.0), "far to the west");
    assert_eq!(
        nearest(100.0, 100.0),
        Vec2::new(240.0, 240.0),
        "past the north-west corner"
    );
    assert_eq!(
        nearest(500.0, 395.0),
        Vec2::new(400.0, 395.0),
        "east, by the south-east corner"
    );
    assert_eq!(
        nearest(260.0, 330.0),
        Vec2::new(240.0, 330.0),
        "on the footprint, by its west edge"
    );
    assert_eq!(
        nearest(330.0, 390.0),
        Vec2::new(330.0, 400.0),
        "between the footprint and the wall, south"
    );
    assert_eq!(nearest(320.0, 320.0), Vec2::new(400.0, 320.0), "the centre: east first");
}

/// The search (0339's wall-hug): the nearest point first, then a step further each way, clockwise first (north, on
/// the west side), then two steps, and so on up to halfway round, where the two ways meet once.
#[test]
fn the_search_goes_both_ways_from_the_nearest_point_clockwise_first() {
    let w = wall();
    let first: Vec<Vec2> = w.hug(Vec2::new(0.0, 320.0), 4.0).take(5).collect();
    assert_eq!(
        first,
        [
            (240.0, 320.0),
            (240.0, 316.0),
            (240.0, 324.0),
            (240.0, 312.0),
            (240.0, 328.0)
        ]
        .map(|(x, y)| Vec2::new(x, y))
    );
    let all: Vec<Vec2> = w.hug(Vec2::new(0.0, 320.0), 4.0).collect();
    assert_eq!(all.len(), 160, "1 + 2 × 80 steps less the halfway point, tried once");
    assert_eq!(
        all.last(),
        Some(&Vec2::new(400.0, 320.0)),
        "halfway round, the opposite side"
    );
    // Round the corner: from the south-west corner on clockwise, up the west side; the other way along the south.
    let corner: Vec<Vec2> = w.hug(Vec2::new(200.0, 440.0), 8.0).take(3).collect();
    assert_eq!(
        corner,
        [(240.0, 400.0), (240.0, 392.0), (248.0, 400.0)].map(|(x, y)| Vec2::new(x, y))
    );
}

/// The way round: from the west side to the north side it turns at the north-west corner; to the east side, the
/// shorter way, by the north-west and then the north-east corner, or the south ones, by where the unit is; along one
/// side it doesn't turn. Of two ways as long, clockwise.
#[test]
fn the_way_round_turns_first_at_the_nearest_corner_the_shorter_way() {
    let w = wall();
    let v = Vec2::new;
    assert_eq!(w.next_corner(v(0.0, 300.0), v(300.0, 240.0)), Some(v(240.0, 240.0)));
    assert_eq!(w.next_corner(v(0.0, 300.0), v(400.0, 280.0)), Some(v(240.0, 240.0)));
    assert_eq!(w.next_corner(v(0.0, 340.0), v(400.0, 360.0)), Some(v(240.0, 400.0)));
    assert_eq!(
        w.next_corner(v(240.0, 240.0), v(400.0, 280.0)),
        Some(v(400.0, 240.0)),
        "on the corner, the next"
    );
    assert_eq!(w.next_corner(v(0.0, 300.0), v(240.0, 380.0)), None, "one side");
    assert_eq!(
        w.next_corner(v(0.0, 320.0), v(400.0, 320.0)),
        Some(v(240.0, 240.0)),
        "halfway: clockwise"
    );
}

/// The straight way crosses a rectangle only through its inside: along its edge, touching a corner, or wholly outside
/// is clear; from inside it, or through it, it crosses.
#[test]
fn a_way_crosses_a_rectangle_only_through_its_inside() {
    let (centre, half) = (Vec2::new(320.0, 320.0), Vec2::splat(76.0));
    let crosses = |a: (f32, f32), b: (f32, f32)| crosses(Vec2::new(a.0, a.1), Vec2::new(b.0, b.1), centre, half);
    assert!(crosses((0.0, 320.0), (600.0, 320.0)), "through");
    assert!(crosses((320.0, 320.0), (600.0, 600.0)), "from inside");
    assert!(crosses((0.0, 300.0), (300.0, 260.0)), "cutting the corner");
    assert!(!crosses((0.0, 300.0), (300.0, 230.0)), "passing the corner");
    assert!(!crosses((0.0, 244.0), (600.0, 244.0)), "along the north edge");
    assert!(!crosses((0.0, 200.0), (300.0, 200.0)), "outside");
    assert!(!crosses((200.0, 288.0), (288.0, 200.0)), "touching the corner");
    assert!(!crosses((240.0, 300.0), (240.0, 300.0)), "standing still outside");
}

/// The row of a point: how many steps further out its wall is, 0 on the wall or inside it, a corner's as a side's.
#[test]
fn a_point_is_on_the_row_whose_wall_goes_through_it() {
    let (w, step) = (wall(), Vec2::splat(32.0));
    assert_eq!(w.row_of(Vec2::new(240.0, 300.0), step), 0);
    assert_eq!(w.row_of(Vec2::new(176.0, 300.0), step), 2, "two rows out, west");
    assert_eq!(w.row_of(Vec2::new(432.0, 432.0), step), 1, "a corner one row out");
    assert_eq!(w.row_of(Vec2::new(320.0, 330.0), step), 0, "on the footprint");
}

/// The walls further out are all off a map once their rectangle reaches its farthest edge along both axes.
#[test]
fn the_walls_further_out_end_off_the_map() {
    let w = wall();
    // From the centre (320, 320) to the farthest edges of a 1280 × 640 map: 960 and 320; the wall is 80 out.
    let step = Vec2::splat(32.0);
    assert_eq!(
        w.off_map(step, Vec2::new(1280.0, 640.0)),
        28,
        "(960 − 80) / 32, rounded up"
    );
    assert_eq!(w.out(step * 28.0).half, Vec2::splat(976.0));
    let past = w.out(Vec2::splat(300.0));
    assert_eq!(
        past.off_map(step, Vec2::splat(640.0)),
        0,
        "a wall already past both edges"
    );
}
