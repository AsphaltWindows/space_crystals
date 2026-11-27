//! Unit tests of the detour round a row of standing units ([`detour`]).

use super::*;
use crate::ids::ObjectId;
use crate::map::Map;
use crate::movement::MoveCounters;
use crate::movement::space::{Body, Layer};
use crate::nav::{Nav, NavClass, NavGrid, NavRules};

/// A Peacekeeper-sized body (24 × 24 su) that stands for good, or is on its way and only held up.
fn body(id: u32, x: f32, y: f32, held_up: bool) -> Body {
    Body {
        id: ObjectId(id),
        owner: None,
        pos: Vec2::new(x, y),
        half: Vec2::splat(12.0),
        oriented: None,
        velocity: Vec2::ZERO,
        heading: Vec2::ZERO,
        stationary: true,
        progress: held_up.then_some(0),
        arrived_with: None,
        yields: false,
        gathers: false,
        leads: true,
        moved: 0,
        crushes: false,
        crushable: false,
    }
}

/// [`detour`] for `bodies[0]`, blocked by `bodies[1]`, on `grid`.
fn detour_of(bodies: &[Body], grid: Option<&NavGrid>, target: Vec2, other_end: bool) -> Vec<Vec2> {
    let mut layer = Layer::default();
    layer.reset(40, 12);
    for (index, b) in bodies.iter().enumerate() {
        layer.insert(index as u32, b.pos, b.half, b.kinds());
    }
    let world = World {
        grid,
        layer: Some(&layer),
        bodies,
        me: 0,
        contact: crate::defs::ContactStep::Sideways,
        door: None,
        lanes: &mut Vec::new(),
        counters: &mut MoveCounters::default(),
        mine: bodies[0],
    };
    detour(&world, 1, bodies[0].pos, target, other_end)
}

/// A firing line of three, a little out of line as units that stopped one after another are, with its target to
/// the west; the unit stands against the middle one's east face, in the pocket between the two that stick out.
/// The row's box is x 468.5 to 494.1 and y 204.7 to 291.1, grown by 14 (the unit's half extents and the
/// clearance).
fn firing_line() -> [Body; 4] {
    [
        body(1, 505.2, 245.0, false),
        body(2, 481.2, 245.7, false),
        body(3, 482.1, 216.7, false),
        body(4, 481.7, 279.1, false),
    ]
}

const TARGET: Vec2 = Vec2::new(100.0, 250.0);

/// The bug's geometry: out of the pocket by the east side, along it to the nearer end, and across that end to
/// the front, the side that faces the target.
#[test]
fn the_detour_goes_out_of_the_row_round_its_nearer_end_to_the_front() {
    let detour = detour_of(&firing_line(), None, TARGET, false);
    let (west, east, north) = (469.2 - 14.0, 494.1 + 14.0, 204.7 - 14.0);
    let expected = [Vec2::new(east, 245.0), Vec2::new(east, north), Vec2::new(west, north)];
    assert_eq!(detour.len(), 3, "{detour:?}");
    for (point, expected) in detour.iter().zip(expected) {
        assert!(point.distance(expected) < 1e-3, "{detour:?}");
    }
}

/// An attempt after one that didn't get the unit on goes round the other end.
#[test]
fn the_other_end_is_the_fronts_far_end() {
    let detour = detour_of(&firing_line(), None, TARGET, true);
    let south = 291.1 + 14.0;
    assert_eq!(detour.len(), 3, "{detour:?}");
    assert!(
        (detour[1].y - south).abs() < 1e-3 && (detour[2].y - south).abs() < 1e-3,
        "{detour:?}"
    );
    assert!(detour[2].x < detour[1].x, "and ends at the front: {detour:?}");
}

/// The row is the units that stand for good, as far as the moving unit doesn't fit between them: a unit that is
/// only held up (on its way, like the unit itself) isn't of it, nor is one a unit's width away.
#[test]
fn units_on_their_way_and_units_apart_are_not_of_the_row() {
    let mut bodies = firing_line().to_vec();
    // Right north of the row: held up, so the row ends where it did. And one 24 su south of its south end.
    bodies.push(body(5, 482.0, 190.0, true));
    bodies.push(body(6, 482.0, 291.1 + 24.0 + 12.0, false));
    assert_eq!(
        detour_of(&bodies, None, TARGET, false),
        detour_of(&firing_line(), None, TARGET, false)
    );
    // A hair nearer, the unit doesn't fit between: the row is longer to the south.
    bodies[5].pos.y -= 0.5;
    let longer = detour_of(&bodies, None, TARGET, true);
    assert!(longer[1].y > 291.1 + 14.0 + 30.0, "{longer:?}");
}

/// In a corridor a tile wide there is no way round: every way out of the row's box, and along it, runs into the
/// walls. No detour.
#[test]
fn between_walls_there_is_no_detour() {
    let rows = ["M".repeat(40), ".".repeat(40), "M".repeat(40)];
    let map = Map::from_ascii(&rows.join("\n")).unwrap();
    let nav = Nav::build(&map, &[], &[(NavClass::GroundRugged, 1)], NavRules::default());
    let grid = nav.grid(NavClass::GroundRugged, 1).unwrap();
    // The corridor is y 64 to 128. Two Guards (36 su) in it, the second behind the first.
    let guard = |id: u32, x: f32| Body {
        half: Vec2::splat(18.0),
        oriented: None,
        ..body(id, x, 96.0, false)
    };
    let bodies = [guard(1, 536.0), guard(2, 500.0)];
    assert_eq!(detour_of(&bodies, Some(grid), Vec2::new(100.0, 96.0), false), []);
    // In the open the same two have one.
    assert!(!detour_of(&bodies, None, Vec2::new(100.0, 96.0), false).is_empty());
}
