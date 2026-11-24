//! Unit tests of one unit's tick, without other units or tiles. The scenario tests in `crates/sim/tests/` cover
//! collision.

use super::*;

const WALKER: MovementAttrs = MovementAttrs::TurnRate {
    turn_rate: math::PI,
    acceleration: None,
    deceleration: None,
    max_speed: 4.0,
};

fn walking(path: &[Vec2]) -> Mover {
    let mut mover = Mover::default();
    mover.go_to(MoveGoal::Point(*path.last().unwrap()), None, 1);
    mover.follow(path.to_vec(), *path.last().unwrap(), 0);
    mover
}

/// One tick of a lone unit that collides with nothing (as an air unit). Returns whether its move failed.
fn tick(pos: &mut Vec2, facing: &mut f32, mover: &mut Mover, movement: &MovementAttrs, tick: Tick) -> bool {
    turning_tick(pos, facing, mover, movement, tick).0
}

/// [`tick`], and the angle the unit turned: what it leaves of its TurnRate for the aim in phase 5e ([`Turns`]).
fn turning_tick(
    pos: &mut Vec2,
    facing: &mut f32,
    mover: &mut Mover,
    movement: &MovementAttrs,
    tick: Tick,
) -> (bool, f32) {
    let bodies = [Body {
        id: ObjectId(1),
        owner: None,
        pos: *pos,
        half: Vec2::splat(12.0),
        velocity: mover.velocity,
        heading: Vec2::ZERO,
        stationary: mover.is_stationary(),
        progress: None,
        arrived_with: None,
        yields: false,
        gathers: false,
        leads: true,
    }];
    let mut counters = MoveCounters::default();
    let mut world = World {
        grid: None,
        layer: None,
        bodies: &bodies,
        me: 0,
        door: None,
        lanes: &mut Vec::new(),
        counters: &mut counters,
    };
    let mut turned = 0.0;
    let failed = step_unit(
        pos,
        facing,
        &mut turned,
        mover,
        movement,
        &mut world,
        &mut Vec::new(),
        &mut Vec::new(),
        None,
        &[],
        tick,
    );
    (failed, turned)
}

#[test]
fn passes_waypoints_and_snaps_onto_the_goal() {
    let path = [Vec2::new(0.0, 0.0), Vec2::new(6.0, 0.0), Vec2::new(6.0, 9.0)];
    let mut mover = walking(&path);
    let (mut pos, mut facing) = (path[0], 0.0);
    tick(&mut pos, &mut facing, &mut mover, &WALKER, 2);
    assert_eq!(pos, Vec2::new(4.0, 0.0));
    assert_eq!(
        (facing, mover.velocity, mover.locomotion),
        (0.0, Vec2::new(4.0, 0.0), Locomotion::Moving)
    );
    assert_eq!(mover.orientation, Orientation::Maintaining);
    // The corner waypoint is within one step: the unit heads for the next one without touching it.
    tick(&mut pos, &mut facing, &mut mover, &WALKER, 3);
    assert_eq!(mover.next, 2);
    assert!(
        (pos.distance(Vec2::new(4.0, 0.0)) - 4.0).abs() < 1e-5 && pos.y > 0.0,
        "{pos}"
    );
    assert_eq!(mover.orientation, Orientation::Turning(path[2]));
    tick(&mut pos, &mut facing, &mut mover, &WALKER, 4);
    assert_eq!(mover.status, MoveStatus::Moving);
    // The rest is within one step: arrive exactly.
    tick(&mut pos, &mut facing, &mut mover, &WALKER, 5);
    assert_eq!(pos, path[2]);
    assert_eq!((mover.status, mover.goal), (MoveStatus::Arrived, MoveGoal::None));
    assert_eq!((mover.velocity, mover.locomotion), (Vec2::ZERO, Locomotion::Stationary));
    // And stays.
    tick(&mut pos, &mut facing, &mut mover, &WALKER, 6);
    assert_eq!(pos, path[2]);
}

/// A unit turns at most its TurnRate per tick, in place until it faces its way. (G) Each tick's angle is what it
/// takes of the tick's one turn budget: the aim in phase 5e has only the rest ([`Turns`]).
#[test]
fn a_slow_turn_is_made_in_place() {
    let slow = MovementAttrs::TurnRate {
        turn_rate: 0.5,
        acceleration: None,
        deceleration: None,
        max_speed: 4.0,
    };
    let path = [Vec2::ZERO, Vec2::new(0.0, 100.0)];
    let mut mover = walking(&path);
    let (mut pos, mut facing) = (path[0], 0.0);
    assert_eq!(turning_tick(&mut pos, &mut facing, &mut mover, &slow, 2), (false, 0.5));
    assert_eq!((pos, facing), (Vec2::ZERO, 0.5));
    assert_eq!(mover.locomotion, Locomotion::Stationary);
    assert_eq!(mover.orientation, Orientation::Turning(path[1]));
    assert_eq!(turning_tick(&mut pos, &mut facing, &mut mover, &slow, 3), (false, 0.5));
    assert_eq!(turning_tick(&mut pos, &mut facing, &mut mover, &slow, 4), (false, 0.5));
    assert_eq!(pos, Vec2::ZERO);
    // The last 0.07 rad complete within the tick, and the unit walks.
    let (_, last) = turning_tick(&mut pos, &mut facing, &mut mover, &slow, 5);
    assert_eq!((pos, facing), (Vec2::new(0.0, 4.0), math::PI / 2.0));
    assert_eq!(last, math::PI / 2.0 - 1.5);
    // Facing its way, it turns no more.
    assert_eq!(turning_tick(&mut pos, &mut facing, &mut mover, &slow, 6), (false, 0.0));
}

#[test]
fn without_a_path_the_unit_waits_and_its_stuck_clock_stands() {
    let mut mover = Mover::default();
    mover.go_to(MoveGoal::Point(Vec2::new(50.0, 0.0)), None, 1);
    let (mut pos, mut facing) = (Vec2::ZERO, 1.0);
    for t in 2..100 {
        assert!(!tick(&mut pos, &mut facing, &mut mover, &WALKER, t));
    }
    assert_eq!((pos, facing, mover.status), (Vec2::ZERO, 1.0, MoveStatus::Moving));
    assert_eq!(mover.last_progress, 99, "it moved on with the ticks");
}

/// `docs/movement.md > 6.5`, `> 5.6` (design change 0024): a unit that makes no progress is stuck after 24 ticks,
/// repaths three times after waits of 8, 16 and 32 ticks, then fails.
#[test]
fn a_unit_without_progress_repaths_three_times_and_fails() {
    let frozen = MovementAttrs::TurnRate {
        turn_rate: math::PI,
        acceleration: None,
        deceleration: None,
        max_speed: 0.0,
    };
    let path = [Vec2::ZERO, Vec2::new(500.0, 0.0)];
    let mut mover = walking(&path);
    let (mut pos, mut facing) = (path[0], 0.0);
    let mut repaths = Vec::new();
    let mut failed_at = None;
    for t in 2..400 {
        if matches!(mover.path_state, PathState::Wanted { .. }) {
            // Phase 5c: the same path again.
            repaths.push(t);
            mover.follow(path.to_vec(), path[1], 0);
        }
        if tick(&mut pos, &mut facing, &mut mover, &frozen, t) {
            failed_at = Some(t);
            break;
        }
    }
    // Stuck 24 ticks after the order (tick 1), in tick 25. It touches no unit, so each time it also waits the 8
    // ticks of grace: repaths in ticks 25 + 8 + 8 = 41, 41 + 16 + 8 = 65 and 65 + 32 + 8 = 105, each served a tick
    // later. It fails 24 ticks and the grace after the last.
    assert_eq!(repaths, [42, 66, 106]);
    assert_eq!(failed_at, Some(137));
    assert_eq!((mover.status, mover.goal), (MoveStatus::Failed, MoveGoal::None));
}
