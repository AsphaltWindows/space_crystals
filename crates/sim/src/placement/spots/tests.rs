//! The spots around a footprint (plan decision T62): M4-S1's eight for the Laser Turret's Agent, unchanged, and the
//! rings that hold more.

use super::*;
use crate::paths::boxes_overlap;
use crate::units::TilePos;

/// A footprint `w` × `h` tiles whose top-left tile is (4, 4).
fn footprint(w: u32, h: u32) -> Placement {
    Placement {
        w,
        h,
        ..Placement::new(TilePos::new(4, 4), crate::defs::Footprint::Tiles { w: 1, h: 1 })
    }
}

/// M4-S1's eight directions out of a footprint, clockwise on screen from the east.
const BESIDE: [(f32, f32); 8] = [
    (1.0, 0.0),
    (1.0, 1.0),
    (0.0, 1.0),
    (-1.0, 1.0),
    (-1.0, 0.0),
    (-1.0, -1.0),
    (0.0, -1.0),
    (1.0, -1.0),
];

/// Design change 0298 point 7 (M4-S1's test, moved here): the spots beside a footprint are tried "in a fixed order
/// starting from the side it came from", then further and further round, of two equally far the lower number first,
/// and the opposite side last; a direction between two goes to the nearer. Each order tries all eight.
#[test]
fn the_spots_beside_are_tried_from_the_side_it_came_from() {
    let beside_order = |came_from: Vec2| Round::BothWays.order(8, eighth(came_from));
    assert_eq!(beside_order(Vec2::new(1.0, 0.0)), [0, 1, 7, 2, 6, 3, 5, 4]);
    assert_eq!(beside_order(Vec2::new(-1.0, 1.0)), [3, 2, 4, 1, 5, 0, 6, 7]);
    assert_eq!(beside_order(Vec2::new(0.0, -1.0)), [6, 5, 7, 0, 4, 1, 3, 2]);
    assert_eq!(beside_order(Vec2::new(-1.0, 0.0)), [4, 3, 5, 2, 6, 1, 7, 0]);
    assert_eq!(beside_order(Vec2::new(-1.0, -0.0)), [4, 3, 5, 2, 6, 1, 7, 0]);
    assert_eq!(beside_order(Vec2::new(1.0, 0.3))[0], 0);
    assert_eq!(beside_order(Vec2::new(1.0, 0.5))[0], 1);
    for i in 0..8 {
        let mut order = beside_order(math::facing_vector(i as f32 * PI / 4.0 + 0.1));
        assert_eq!(order[0], i);
        order.sort_unstable();
        assert_eq!(order, [0, 1, 2, 3, 4, 5, 6, 7]);
    }
}

/// The Laser Turret's Agent (a 1 × 1 footprint, a 36 × 36 box): ring 0 has no room between a side's middle and its
/// corners, so it is M4-S1's eight spots exactly, each against the footprint, numbered as before, and facing out the
/// same way.
#[test]
fn a_turret_ring_is_the_eight_directions_out_of_it() {
    let placement = footprint(1, 1);
    let half = Vec2::splat(18.0);
    let ring = Spots::around(&placement, half).ring(0);
    let reach = Vec2::splat(SU_PER_GU / 2.0) + half + Vec2::splat(WALL_CLEARANCE);
    let expected: Vec<(Vec2, Vec2)> = (BESIDE.iter())
        .map(|&direction| {
            let direction = Vec2::from(direction);
            (placement.center() + direction * reach, direction.normalize())
        })
        .collect();
    assert_eq!(ring.spots, expected);
    assert_eq!(ring.anchors, [0, 1, 2, 3, 4, 5, 6, 7]);
}

/// The Armory (3 × 2) and a Recruit (24 × 24): ring 0 has the middles and the corners, two more spots on each half of
/// a long side and three on each half of a short one: 28, numbered clockwise from the middle of the east side. Every box
/// stands outside the footprint, against it or at a corner, and no two overlap.
#[test]
fn an_armory_ring_holds_twenty_eight_recruits() {
    let placement = footprint(3, 2);
    let half = Vec2::splat(12.0);
    let spots = Spots::around(&placement, half);
    let ring = spots.ring(0);
    assert_eq!(ring.spots.len(), 28);
    // Short sides (east, west): 2 + 2 between the middle and the corners; long ones: 3 + 3.
    assert_eq!(ring.anchors, [0, 3, 7, 11, 14, 17, 21, 25]);
    let (centre, size) = (placement.center(), Vec2::new(192.0, 128.0) / 2.0);
    for (i, &(spot, out)) in ring.spots.iter().enumerate() {
        assert!(
            !boxes_overlap(spot, half, centre, size),
            "spot {i} overlaps the footprint"
        );
        let gap = (spot - centre).abs() - size - half;
        assert!(
            gap.max_element() > 0.0 && gap.max_element() < 1.0,
            "spot {i} isn't against it: {gap}"
        );
        assert!((out.length() - 1.0).abs() < 1e-6);
        for (j, &(other, _)) in ring.spots.iter().enumerate().skip(i + 1) {
            assert!(!boxes_overlap(spot, half, other, half), "spots {i} and {j} overlap");
        }
    }
    // Clockwise on screen: the angle round the centre grows from the east, once round.
    let angles: Vec<f32> = (ring.spots.iter())
        .map(|&(spot, _)| math::vector_angle(spot - centre).rem_euclid(2.0 * PI))
        .collect();
    assert!(angles.windows(2).all(|w| w[0] < w[1]), "{angles:?}");
    assert_eq!(ring.spots[ring.anchors[4]].1, Vec2::new(-1.0, 0.0), "the west middle");
}

/// Each ring is a box further out than the one before, so no spot of one overlaps a spot of the other.
#[test]
fn the_next_ring_is_a_box_further_out() {
    let placement = footprint(3, 2);
    let half = Vec2::splat(12.0);
    let spots = Spots::around(&placement, half);
    let (inner, outer) = (spots.ring(0), spots.ring(1));
    let step = 24.0 + WALL_CLEARANCE;
    assert_eq!(outer.spots[0].0, inner.spots[0].0 + Vec2::new(step, 0.0));
    assert!(outer.spots.len() > inner.spots.len());
    for &(a, _) in &inner.spots {
        for &(b, _) in &outer.spots {
            assert!(!boxes_overlap(a, half, b, half));
        }
    }
}

/// Both ways round from the first spot, the opposite last, each spot once.
#[test]
fn the_round_tries_every_spot_once() {
    assert_eq!(
        Round::BothWays.order(5, 3),
        [3, 2, 4, 0, 1],
        "of two equally far, the lower number first"
    );
    assert_eq!(Round::BothWays.order(1, 0), [0]);
    assert_eq!(Round::BothWays.order(0, 0), Vec::<usize>::new());
}

/// A search tries a ring from its spot in the direction given and goes on round, then the next ring: with the west
/// middle taken, the next spot round from it, of the two as near the lower numbered (clockwise from the east), south
/// of it; with all of ring 0 taken, ring 1's west middle.
#[test]
fn a_search_goes_round_then_out() {
    let placement = footprint(3, 2);
    let spots = Spots::around(&placement, Vec2::splat(12.0));
    let (inner, outer) = (spots.ring(0), spots.ring(1));
    let west = Vec2::new(-1.0, 0.0);
    let found = spots.first(0..=3, west, Round::BothWays, |_| true);
    assert_eq!(found, Some(inner.spots[14]));
    let taken = inner.spots[14].0;
    let found = spots.first(0..=3, west, Round::BothWays, |spot| spot != taken);
    assert_eq!(found, Some(inner.spots[13]));
    assert!(found.unwrap().0.y > taken.y, "south of it");
    let ring_0: Vec<Vec2> = inner.spots.iter().map(|&(spot, _)| spot).collect();
    let found = spots.first(0..=3, west, Round::BothWays, |spot| !ring_0.contains(&spot));
    assert_eq!(found, Some(outer.spots[outer.anchors[4]]));
    assert_eq!(
        spots.first(0..=0, west, Round::BothWays, |spot| !ring_0.contains(&spot)),
        None
    );
}
