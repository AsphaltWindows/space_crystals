//! The search's own parts (`orders::aim`); its behaviour, in games: `tests/hand_of_heaven/hull.rs` (along a cliff),
//! `tests/vehicle_assembly/mortar.rs` (the Bike's close targets) and `tests/factory/raptor/close_range.rs`.

use super::*;
use crate::defs::ObjectKind;
use crate::map::Map;
use crate::nav::{NavClass, NavRules};

/// The ring search's points round `from`: on rings 16 su apart out to 4 grid units, the first point of each toward the
/// mark, the next ones further and further from it, of two as far the clockwise one first (the angle growing), never
/// more than a step apart along the ring.
#[test]
fn the_rings_go_out_from_the_unit_the_point_toward_the_mark_first() {
    let from = Vec2::new(640.0, 640.0);
    let points: Vec<Vec2> = rings(from, from + Vec2::new(0.0, -100.0)).collect();
    let first: Vec<Vec2> = points.iter().take(8).map(|&p| p - from).collect();
    let near = |a: Vec2, b: Vec2| a.distance(b) < 1e-3;
    let s = 16.0 * std::f32::consts::FRAC_1_SQRT_2;
    let expected = [
        Vec2::new(0.0, -16.0),
        Vec2::new(s, -s),
        Vec2::new(-s, -s),
        Vec2::new(16.0, 0.0),
        Vec2::new(-16.0, 0.0),
        Vec2::new(s, s),
        Vec2::new(-s, s),
        Vec2::new(0.0, 16.0),
    ];
    assert!(first.iter().zip(expected).all(|(&a, b)| near(a, b)), "{first:?}");
    let radii: Vec<f32> = points.iter().map(|p| p.distance(from)).collect();
    assert!(radii.windows(2).all(|w| w[0] <= w[1] + 1e-3), "the nearest rings first");
    assert!((radii[radii.len() - 1] - 256.0).abs() < 1e-3, "out to 4 grid units");
    assert!((radii[8] - 32.0).abs() < 1e-3, "the second ring");
    let second: Vec<Vec2> = points[8..]
        .iter()
        .take_while(|p| (p.distance(from) - 32.0).abs() < 1e-3)
        .copied()
        .collect();
    // 2π · 32 / 16 ≈ 12.6 steps round it: up to the even 14, 14.4 su apart.
    assert_eq!(second.len(), 14, "an even number of points, at most a step apart");
    assert!(std::f32::consts::TAU * 32.0 / second.len() as f32 <= 16.0);
    assert!(near(second[0] - from, Vec2::new(0.0, -32.0)), "toward the mark first");
    assert!(
        near(second[13] - from, Vec2::new(0.0, 32.0)),
        "the point away from it last"
    );
}

/// The Assault Bike turns on circles of its MinimumTurnRadius, 128 su, either side of it: a target inside one, or less
/// than the margin beyond its rim, it can't come round onto with its mortar, the margin a grid unit, and its mortar's
/// MinRange, a grid unit (design change 0460; `margin`). Facing east at (640, 640): its circles' centres are 128 su
/// north and south of it.
#[test]
fn a_wheeled_unit_can_t_come_round_onto_a_target_inside_its_circles_or_near_their_rims() {
    let bike = ObjectKind::AssaultBike.def().unit.as_ref().unwrap();
    let at = Vec2::new(640.0, 640.0);
    let mortar = bike.attack.as_ref().unwrap();
    let inside = |target: Vec2| inside_its_circles(bike, mortar, at, 0.0, target);
    assert!(inside(at + Vec2::new(0.0, -128.0)), "the north circle's centre");
    assert!(inside(at + Vec2::new(0.0, 48.0)), "beside it");
    assert!(
        inside(at + Vec2::new(0.0, -128.0 - 255.0)),
        "less than two grid units beyond the rim"
    );
    assert!(!inside(at + Vec2::new(0.0, -128.0 - 257.0)), "beyond them");
    assert!(!inside(at + Vec2::new(320.0, 0.0)), "ahead, out of both");
    assert!(
        inside(at + Vec2::new(200.0, 0.0)),
        "ahead, less than the margin beyond their rims"
    );
    let hand = ObjectKind::HandOfHeaven.def().unit.as_ref().unwrap();
    assert!(
        !inside_its_circles(hand, hand.attack.as_ref().unwrap(), at, 0.0, at + Vec2::new(0.0, 48.0)),
        "a tracked unit turns in place"
    );
}

/// Room to turn: no solid tile within the half diagonal of the unit's Silhouette of the spot, on the map. Tile (5, 3)
/// is cliff: a spot whose distance to its square is the room is clear, one nearer isn't, and nor is one whose disc runs
/// off the map.
#[test]
fn room_to_turn_is_a_disc_clear_of_solid_tiles() {
    let mut rows = vec![".".repeat(12); 8];
    rows[3] = ".....c......".to_owned();
    let map = Map::from_ascii(&rows.join("\n")).unwrap();
    let grid = NavGrid::build(&map, NavClass::Ground, 1, &[], NavRules::default());
    let room = Vec2::new(32.0, 20.0).length();
    // The cliff's square is [320, 384] × [192, 256].
    assert!(room_to_turn(&grid, Vec2::new(352.0, 256.0 + room), room));
    assert!(!room_to_turn(&grid, Vec2::new(352.0, 256.0 + room - 0.5), room));
    let corner = Vec2::new(384.0, 256.0) + Vec2::splat(room * std::f32::consts::FRAC_1_SQRT_2 + 0.01);
    assert!(room_to_turn(&grid, corner, room), "diagonally off its corner");
    assert!(
        !room_to_turn(&grid, Vec2::new(room - 1.0, 400.0), room),
        "off the map's west edge"
    );
    assert!(room_to_turn(&grid, Vec2::new(room, 400.0), room), "just on it");
}
