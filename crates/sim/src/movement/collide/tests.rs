//! Unit tests of one unit's move against other units ([`World::resolve`]).

use super::*;

#[test]
fn right_is_clockwise_on_screen() {
    let east = Vec2::new(4.0, 0.0);
    assert_eq!(turned_90(east, Side::Right), Vec2::new(0.0, 4.0), "south");
    assert_eq!(turned_90(east, Side::Left), Vec2::new(0.0, -4.0), "north");
    let south_east = turned_45(east, Side::Right);
    assert!(south_east.x > 0.0 && south_east.y > 0.0 && (south_east.length() - 4.0).abs() < 1e-5);
    let north_east = turned_45(east, Side::Left);
    assert!(north_east.x > 0.0 && north_east.y < 0.0 && (north_east.length() - 4.0).abs() < 1e-5);
    // Twice 45° is 90°.
    let twice = turned_45(south_east, Side::Right);
    assert!((twice - turned_90(east, Side::Right)).length() < 1e-5);
}

/// `docs/movement.md > 6.3`, rule 4: a unit going round a blocker on the side it leans to keeps that side when
/// its own step along the face turns the contact head-on, so it never steps back. Pressed against an idle Guard,
/// a unit whose lean was about [`LEAN`] stepped north as leaning, then south as head-on with `avoid_side` Left,
/// for good (two groups crossing in the lanes of the magic box found it, `> 6.4`).
#[test]
fn a_unit_going_round_on_its_lean_side_never_steps_back() {
    let body = |id: u32, pos: Vec2, half: f32| Body {
        id: crate::ids::ObjectId(id),
        owner: None,
        pos,
        half: Vec2::splat(half),
        velocity: Vec2::ZERO,
        heading: Vec2::ZERO,
        stationary: id == 2,
        progress: None,
        arrived_with: None,
        yields: false,
        gathers: false,
        leads: true,
    };
    // An idle Guard, x 502..538 and y 372..408. A Peacekeeper against its east face heads for a point north-west
    // of it; a head-on contact would make it step south.
    let guard = body(2, Vec2::new(520.0, 390.0), 18.0);
    let target = Vec2::new(416.0, 352.0);
    let mut pos = Vec2::new(550.0, 395.9);
    let mut mover = Mover {
        avoid_side: Side::Left,
        ..Mover::default()
    };
    let (mut layer, mut counters, mut lanes) = (Layer::default(), MoveCounters::default(), Vec::new());
    for tick in 100..130 {
        let bodies = [body(1, pos, 12.0), guard];
        layer.reset(16, 12);
        for (index, b) in bodies.iter().enumerate() {
            layer.insert(index as u32, b.pos, b.half, b.kinds());
        }
        let mut world = World {
            grid: None,
            layer: Some(&layer),
            bodies: &bodies,
            me: 0,
            door: None,
            lanes: &mut lanes,
            counters: &mut counters,
        };
        let d = (target - pos).clamp_length_max(4.0);
        let moved = world.resolve(pos, d, None, &mut mover, tick, &mut Vec::new());
        assert!(moved.pos.y <= pos.y, "tick {tick}: from {pos} back to {}", moved.pos);
        pos = moved.pos;
    }
    // Round the Guard's north-east corner and on west.
    assert!(pos.x < 500.0 && pos.y < 360.0, "{pos}");
}

/// `systems/units.md > UnitCollision > Units that already overlap` (design change 0106), for a pair that has no
/// Drone in it: "The overlap rule is general". A Peacekeeper inside a Guard walks out of it at full speed, straight
/// through, and the Guard isn't moved (nothing writes another unit's position). Once the two are apart the Guard
/// is solid: sent back, the Peacekeeper goes round it and never enters its box again.
#[test]
fn a_unit_inside_another_walks_out_unhindered_and_is_blocked_once_apart() {
    let body = |id: u32, pos: Vec2, half: f32| Body {
        id: crate::ids::ObjectId(id),
        owner: None,
        pos,
        half: Vec2::splat(half),
        velocity: Vec2::ZERO,
        heading: Vec2::ZERO,
        stationary: id == 2,
        progress: None,
        arrived_with: None,
        yields: false,
        gathers: false,
        leads: true,
    };
    let guard = body(2, Vec2::new(520.0, 390.0), 18.0);
    let mut pos = Vec2::new(514.0, 392.0);
    let mut mover = Mover::default();
    let (mut layer, mut counters, mut lanes) = (Layer::default(), MoveCounters::default(), Vec::new());
    let mut step = |pos: Vec2, d: Vec2, mover: &mut Mover, tick: Tick| {
        let bodies = [body(1, pos, 12.0), guard];
        layer.reset(16, 12);
        for (index, b) in bodies.iter().enumerate() {
            layer.insert(index as u32, b.pos, b.half, b.kinds());
        }
        let mut world = World {
            grid: None,
            layer: Some(&layer),
            bodies: &bodies,
            me: 0,
            door: None,
            lanes: &mut lanes,
            counters: &mut counters,
        };
        world.resolve(pos, d, None, mover, tick, &mut Vec::new())
    };
    let inside = |pos: Vec2| overlaps(pos, Vec2::splat(12.0), &guard);
    // Out to the east, 4 su a tick: 30 su apart is touching, from 6 su west of the Guard's centre.
    let east = Vec2::new(4.0, 0.0);
    let mut tick = 100;
    while inside(pos) {
        let moved = step(pos, east, &mut mover, tick);
        assert_eq!(
            (moved.pos, moved.forward, moved.blocker),
            (pos + east, true, None),
            "tick {tick}"
        );
        pos = moved.pos;
        tick += 1;
    }
    assert_eq!((tick, pos), (109, Vec2::new(550.0, 392.0)));
    // And back west, at the Guard: blocked at once, and never inside it again as it goes round.
    assert_eq!(step(pos, -east, &mut mover, tick).blocker, Some(1));
    for tick in tick..tick + 40 {
        let moved = step(pos, -east, &mut mover, tick);
        assert!(!inside(moved.pos), "tick {tick}: {}", moved.pos);
        pos = moved.pos;
    }
}
