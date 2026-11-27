//! Unit tests of the turn of a Silhouette that turns with its facing, and of the halving of a blocked move
//! ([`World::turn`], [`World::free_part`]): without a game, then in one, a 56 × 40 unit on TurnRateMovement. The
//! scenario tests of the Seeker are `crates/sim/tests/oriented/`.

use super::*;
use crate::defs::test_defs::TestDef;
use crate::defs::{ContactStep, MovementAttrs, ObjectKind};
use crate::ids::{ObjectId, PlayerId};
use crate::map::Map;
use crate::movement::MoveCounters;
use crate::movement::space::Layer;
use crate::mover::{MoveStatus, Mover};
use crate::nav::{Nav, NavClass, NavGrid, NavRules};
use crate::orders::{OrderKind, Target};
use crate::scenario::SpawnAt;
use crate::shape;
use crate::testing::ScenarioBuilder;

/// A 56 × 40 Silhouette (the Seeker's and the Prayer Mobile's): 56 su along its facing.
const LONG: Vec2 = Vec2::new(28.0, 20.0);

/// A unit at `pos` whose Silhouette has half extents `half`, lying along `facing` (radians) if it turns with it.
fn body(id: u32, pos: Vec2, half: Vec2, facing: Option<f32>) -> Body {
    let oriented = facing.map(|facing| Oriented::facing(half, facing));
    Body {
        id: ObjectId(id),
        owner: None,
        pos,
        half: oriented.map_or(half, |o| o.bound_half()),
        oriented,
        velocity: Vec2::ZERO,
        heading: Vec2::ZERO,
        stationary: id != 1,
        progress: None,
        arrived_with: None,
        yields: false,
        gathers: false,
        leads: true,
        moved: 0,
    }
}

/// An idle Guard, 36 × 36, at `pos`.
fn guard(id: u32, pos: Vec2) -> Body {
    body(id, pos, Vec2::splat(18.0), None)
}

/// `f` with the world of `bodies[0]`'s move on `grid` (if any), the other bodies standing in its layer.
fn in_world<R>(grid: Option<&NavGrid>, bodies: &[Body], f: impl FnOnce(&mut World) -> R) -> R {
    let mut layer = Layer::default();
    layer.reset(32, 32);
    for (index, b) in bodies.iter().enumerate() {
        layer.insert(index as u32, b.pos, b.half, b.kinds());
    }
    let (mut lanes, mut counters) = (Vec::new(), MoveCounters::default());
    let mut world = World {
        grid,
        layer: Some(&layer),
        bodies,
        me: 0,
        contact: ContactStep::Diagonal,
        door: None,
        lanes: &mut lanes,
        counters: &mut counters,
        mine: bodies[0],
    };
    f(&mut world)
}

fn degrees(radians: f32) -> f32 {
    radians * (180.0 / core::f32::consts::PI)
}

/// T78 against the tiles: a 56 × 40 unit whose long side runs 4 su from a wall turns in place towards the wall's
/// side, or away from it (the other end swings in as far), up to contact and no further. Its lowest corner reaches
/// `y + 28 sin θ + 20 cos θ`, which meets the wall at θ = 8.688°. The turn of 90° is tried in 14 parts (its corners,
/// 34.4 su out, move 54 su); the second, to 12.9°, is blocked and halved five times, so the unit keeps 8.638°, 0.02 su
/// from the wall, and lies there for the rest of its tick.
#[test]
fn a_turn_beside_a_wall_stops_at_contact() {
    let rows = [".".repeat(32), ".".repeat(32), "M".repeat(32)];
    let map = Map::from_ascii(&rows.join("\n")).unwrap();
    let nav = Nav::build(&map, &[], &[(NavClass::GroundRugged, 1)], NavRules::default());
    let grid = nav.grid(NavClass::GroundRugged, 1).unwrap();
    // The wall begins at y = 128; the unit's long side, facing east, at 124.
    let pos = Vec2::new(400.0, 104.0);
    let unit = body(1, pos, LONG, Some(0.0));
    // A 32nd of a 14th of the turn: how close to contact the halving gets.
    let precision = core::f32::consts::FRAC_PI_2 / 14.0 / 32.0;
    // Halving the second part: 9.64° blocked, 8.04° free, 8.84° blocked, 8.44° free, 8.64° free.
    let kept = math::degrees_to_radians(90.0 / 14.0 * 1.34375);
    for (to, reached) in [
        (core::f32::consts::FRAC_PI_2, kept),
        (-core::f32::consts::FRAC_PI_2, -kept),
    ] {
        in_world(Some(grid), &[unit], |world| {
            let turned = world.turn(pos, 0.0, to);
            assert!(turned.cut && turned.blocker.is_none(), "{turned:?}");
            assert!((turned.facing - reached).abs() < 1e-6, "{}°", degrees(turned.facing));
            assert!(degrees(turned.facing.abs()) < 8.688, "{}°", degrees(turned.facing));
            // It lies as it turned, free, its box short of the wall by a fiftieth of a su.
            let lies = world.body().oriented.unwrap();
            assert_eq!(lies.facing, turned.facing);
            assert_eq!(
                world.body().half,
                shape::bounding_half(LONG, math::facing_vector(turned.facing))
            );
            assert!(world.is_free(pos));
            let gap = 128.0 - (pos.y + world.body().half.y);
            assert!(gap > 0.0 && gap < 0.03, "{gap} su from the wall");
            // As far again as the halving's precision is in the wall.
            let further = turned.facing + to.signum() * precision;
            world.mine = world.mine.turned(Oriented::facing(LONG, further));
            assert!(!world.is_free(pos));
        });
    }
    // A turn that stays clear of the wall is made whole.
    in_world(Some(grid), &[unit], |world| {
        let to = math::degrees_to_radians(5.0);
        assert_eq!(
            world.turn(pos, 0.0, to),
            Turned {
                facing: to,
                cut: false,
                blocker: None
            }
        );
        assert_eq!(world.body().oriented.unwrap().facing, to);
    });
}

/// A turn that a unit stops names it as the blocker (`super::super::follow::blocked_turn` asks it to yield). A
/// Guard stands just off the unit's front corner on its right: turning clockwise (right, towards the south) the
/// corner swings into it at about 4.2°; turning the other way the unit swings clear of it, all the way.
#[test]
fn a_turn_into_a_unit_stops_there_and_names_it() {
    let pos = Vec2::new(400.0, 400.0);
    // The unit spans x 372..428 and y 380..420; the Guard x 422..458 and y 422..458.
    let bodies = [body(1, pos, LONG, Some(0.0)), guard(2, Vec2::new(440.0, 440.0))];
    in_world(None, &bodies, |world| {
        let turned = world.turn(pos, 0.0, core::f32::consts::FRAC_PI_2);
        assert!(turned.cut, "{turned:?}");
        assert_eq!(turned.blocker, Some(1));
        assert!(
            turned.facing > 0.0 && degrees(turned.facing) < 4.3,
            "{}°",
            degrees(turned.facing)
        );
        assert!(world.is_free(pos));
        assert!(!overlaps(world.body(), pos, &bodies[1]));
    });
    in_world(None, &bodies, |world| {
        let to = -core::f32::consts::FRAC_PI_2;
        let turned = world.turn(pos, 0.0, to);
        assert!(!turned.cut && turned.facing == to, "{turned:?}");
        assert!(world.is_free(pos));
    });
}

/// A square Silhouette is the same box at every facing: its turn is never tried, and its body stays as it was.
#[test]
fn a_square_silhouette_turns_freely() {
    let pos = Vec2::new(400.0, 400.0);
    // A Guard right against it on the east: a box that turned would meet it.
    let bodies = [body(1, pos, Vec2::splat(18.0), None), guard(2, Vec2::new(436.0, 400.0))];
    in_world(None, &bodies, |world| {
        let turned = world.turn(pos, 0.0, core::f32::consts::PI);
        assert_eq!(
            turned,
            Turned {
                facing: core::f32::consts::PI,
                cut: false,
                blocker: None
            }
        );
        assert!(world.body().oriented.is_none());
        assert_eq!(world.body().half, Vec2::splat(18.0));
    });
}

/// Two idle Guards 48 su apart: a 56 × 40 unit fits between them pointed through the gap (40 su across it), and
/// not lying across it (56 su).
fn gap() -> [Body; 2] {
    // y 340..376 and 424..460, both x 482..518.
    [guard(2, Vec2::new(500.0, 358.0)), guard(3, Vec2::new(500.0, 442.0))]
}

/// The acceptance's gap, through [`World::resolve`]: pointed through it the unit goes straight on, 4 su a tick, and
/// is never slowed; lying across it the unit meets both Guards where its front reaches their faces (x = 462) and
/// gets no further, whatever it tries, and overlaps neither.
#[test]
fn a_long_unit_passes_a_48_su_gap_pointed_through_it_and_not_across_it() {
    let [a, b] = gap();
    let d = Vec2::new(4.0, 0.0);
    let (mut pos, mut mover) = (Vec2::new(380.0, 400.0), Mover::default());
    for tick in 1..=60 {
        let bodies = [body(1, pos, LONG, Some(0.0)), a, b];
        let moved = in_world(None, &bodies, |world| {
            world.resolve(pos, d, None, &mut mover, tick, &mut Vec::new())
        });
        assert!(moved.forward && moved.blocker.is_none(), "tick {tick}: {moved:?}");
        assert_eq!(moved.pos, pos + d, "tick {tick}");
        pos = moved.pos;
    }
    assert_eq!(pos, Vec2::new(620.0, 400.0));
    let north = -core::f32::consts::FRAC_PI_2;
    let (mut pos, mut mover) = (Vec2::new(440.0, 400.0), Mover::default());
    for tick in 1..=10 {
        let bodies = [body(1, pos, LONG, Some(north)), a, b];
        let moved = in_world(None, &bodies, |world| {
            world.resolve(pos, d, None, &mut mover, tick, &mut Vec::new())
        });
        let lies = body(1, moved.pos, LONG, Some(north));
        assert!(
            !overlaps(&lies, moved.pos, &a) && !overlaps(&lies, moved.pos, &b),
            "tick {tick}: {moved:?}"
        );
        assert!(moved.pos.x <= 462.0 + 1e-3, "tick {tick}: {moved:?}");
        pos = moved.pos;
    }
    // The same where it stands in the gap: pointed through, it fits; turned across it, it doesn't.
    let at = Vec2::new(500.0, 400.0);
    in_world(None, &[body(1, at, LONG, Some(0.0)), a, b], |world| {
        assert!(world.is_free(at));
        world.mine = world.mine.turned(Oriented::facing(LONG, north));
        assert!(!world.is_free(at));
    });
}

/// T78's halving of a blocked step, for the move up to a sweep's contact ([`World::free_part`]): a step whose end
/// overlaps keeps the free part of it, the move halved five times; one that ends touching is made whole.
#[test]
fn a_blocked_move_keeps_its_free_part() {
    let pos = Vec2::new(400.0, 400.0);
    // The unit's front at x 428, the Guard's west face at 442: the 20 su step east meets it at 0.7 of the way.
    let bodies = [body(1, pos, LONG, Some(0.0)), guard(2, Vec2::new(460.0, 400.0))];
    let d = Vec2::new(20.0, 0.0);
    in_world(None, &bodies, |world| {
        let at = world.free_part(pos, d, 1.0);
        assert_eq!(
            at,
            pos + d * 0.6875,
            "free at 0.5, 0.625 and 0.6875, not at 0.75 nor 0.71875"
        );
        assert!(world.is_free(at));
        assert_eq!(
            world.free_part(pos, d, 0.7),
            Vec2::new(414.0, 400.0),
            "touching is free"
        );
        // Nothing of a step straight into the Guard is free.
        let touching = Vec2::new(414.0, 400.0);
        assert_eq!(world.free_part(touching, d, 1.0), touching);
    });
}

// In a game: a 56 × 40 unit on TurnRateMovement, the acceptance's test unit. No type of the design is one yet (the
// Seeker's own turning isn't built: `movement::turn_rate`), so it is the Seeker given a TurnRate of 15° a tick.

const P1: PlayerId = PlayerId(1);
const P2: PlayerId = PlayerId(2);
const UNIT: ObjectId = ObjectId(1);

/// The Seeker, 56 × 40, turning 15° a tick and driving 8 su a tick.
fn turning_seeker() -> TestDef {
    TestDef::unit(ObjectKind::Seeker, |unit| {
        unit.movement = MovementAttrs::TurnRate {
            turn_rate: math::degrees_to_radians(15.0),
            acceleration: None,
            deceleration: None,
            max_speed: 8.0,
        };
    })
}

/// The acceptance's "turns in place beside a wall and stops at contact" in a game: the unit, its long side 4 su from a
/// wall, is sent north, a quarter turn to its left. In its first tick it turns 15° in three tries of 5°: the second
/// is blocked (its rear swings into the wall at 8.69°) and halved five times, so it stops at 8.59°, 0.04 su from the
/// wall. A turn cut short is a blocked move (design change 0035): the unit doesn't stand and wait, it takes its step
/// north all the same (`follow::creep_on`), away from the wall, and turns on from there; its turn is cut once more,
/// at 33.3°, and it creeps again. Clear of the wall, it turns the rest of the way and drives to its point. `TestSim`
/// checks the invariants after every tick: it never overlaps the wall.
#[test]
fn in_a_game_a_turn_beside_a_wall_stops_at_contact() {
    let _def = turning_seeker();
    let map = [".".repeat(16), ".".repeat(16), "M".repeat(16)].join("\n");
    let pos = Vec2::new(400.0, 104.0);
    let mut t = ScenarioBuilder::new(&map)
        .spawn(ObjectKind::Seeker, Some(P1), SpawnAt::Point(pos))
        .start();
    let point = Vec2::new(400.0, 40.0);
    t.order_at(2, P1, &[UNIT], OrderKind::Move(Target::Point(point)), false)
        .run_until(2);
    let unit = t.sim().object(UNIT).unwrap();
    assert!(
        (degrees(unit.facing) - -8.59375).abs() < 1e-3,
        "{}°",
        degrees(unit.facing)
    );
    assert_eq!(unit.pos, pos - Vec2::new(0.0, 8.0), "it crept on, north");
    let (mut cut, mut facing) = (1, unit.facing);
    while t.sim().tick() < 40 && t.status(1) == MoveStatus::Moving {
        t.step();
        let unit = t.sim().object(UNIT).unwrap();
        // A tick that turned it less than its TurnRate, short of north, was cut.
        let turned = degrees(facing - unit.facing);
        cut += u32::from(unit.facing > -core::f32::consts::FRAC_PI_2 + 1e-4 && turned < 15.0 - 1e-3);
        facing = unit.facing;
    }
    assert_eq!((t.status(1), t.pos(1)), (MoveStatus::Arrived, point));
    assert!(
        (facing + core::f32::consts::FRAC_PI_2).abs() < 1e-5,
        "facing north: {}°",
        degrees(facing)
    );
    assert_eq!(cut, 2);
}

/// The acceptance's gap in a game: two of player 2's Guards on Hold 48 su apart. Sent east through the gap, the unit
/// is pointed through it, 40 su across, and drives through at full speed, never held up. Standing in the gap and sent
/// north, it can't turn across it: its turn is cut where its long sides meet the Guards' corners, at 11.3°, and it
/// never lies more across the gap while it is in it. It scrapes out along the Guards and gets there.
#[test]
fn in_a_game_a_long_unit_passes_a_48_su_gap_pointed_through_it_and_cant_turn_across_it() {
    let _def = turning_seeker();
    let game = |at: Vec2| {
        let mut t = ScenarioBuilder::new(&format!("{}\n", ".".repeat(24)).repeat(16))
            .spawn(ObjectKind::Seeker, Some(P1), SpawnAt::Point(at))
            .spawn(ObjectKind::Guard, Some(P2), SpawnAt::Point(Vec2::new(500.0, 358.0)))
            .spawn(ObjectKind::Guard, Some(P2), SpawnAt::Point(Vec2::new(500.0, 442.0)))
            .start();
        t.without_auto_targeting()
            .order_at(2, P2, &[ObjectId(2), ObjectId(3)], OrderKind::Hold, false);
        t
    };
    let east = Vec2::new(800.0, 400.0);
    let mut t = game(Vec2::new(300.0, 400.0));
    t.order_at(2, P1, &[UNIT], OrderKind::Move(Target::Point(east)), false);
    while t.sim().tick() < 100 && t.status(1) != MoveStatus::Arrived {
        let before = t.pos(1);
        t.step();
        assert_eq!(t.pos(1).y, 400.0);
        assert!(t.pos(1).x - before.x <= 8.0 + 1e-3);
        assert_eq!(t.mover(1).blocked_ticks, 0, "tick {}: never held up", t.sim().tick());
    }
    assert_eq!((t.status(1), t.pos(1)), (MoveStatus::Arrived, east));
    assert_eq!((t.pos(2), t.pos(3)), (Vec2::new(500.0, 358.0), Vec2::new(500.0, 442.0)));
    let at = Vec2::new(500.0, 400.0);
    let mut t = game(at);
    let north = OrderKind::Move(Target::Point(Vec2::new(500.0, 150.0)));
    t.order_at(2, P1, &[UNIT], north, false).run_until(2);
    let facing = t.sim().object(UNIT).unwrap().facing;
    assert!(
        degrees(facing) < -11.15 && degrees(facing) > -11.33,
        "{}°",
        degrees(facing)
    );
    while t.sim().tick() < 400 && t.status(1) == MoveStatus::Moving {
        t.step();
        let unit = t.sim().object(UNIT).unwrap();
        // Its centre between the Guards, its sides within their reach.
        let between = (unit.pos.x - 500.0).abs() < 18.0 && (unit.pos.y - 400.0).abs() < 24.0;
        assert!(
            !between || degrees(unit.facing.abs()) < 11.33,
            "tick {}: at {} facing {}°",
            t.sim().tick(),
            unit.pos,
            degrees(unit.facing)
        );
    }
    assert_eq!((t.status(1), t.pos(1)), (MoveStatus::Arrived, Vec2::new(500.0, 150.0)));
}
