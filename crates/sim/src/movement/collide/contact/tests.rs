//! Unit tests of the contact step (design change 0323), through one unit's whole tick (`follow::step_unit`): the
//! step is begun by the rules after the sweep and taken over several ticks.

use glam::Vec2;

use crate::defs::{ContactStep, MovementAttrs};
use crate::ids::ObjectId;
use crate::math;
use crate::movement::MoveCounters;
use crate::movement::collide::World;
use crate::movement::follow::step_unit;
use crate::movement::space::{Body, Layer};
use crate::mover::{MoveGoal, MoveStatus, Mover, Side, StepAside};
use crate::tuning::movement::AVOID_COMMIT_TICKS;
use crate::units::Tick;

/// An Agent's box and speed (`factions/syndicate/units/agent.md`): 36 su wide, 6 su a tick.
const HALF: f32 = 18.0;
const SPEED: f32 = 6.0;
const WALKER: MovementAttrs = MovementAttrs::TurnRate {
    turn_rate: math::PI,
    acceleration: None,
    deceleration: None,
    max_speed: SPEED,
};

/// A unit that stands for good and yields to nobody.
fn standing(id: u32, pos: Vec2) -> Body {
    Body {
        id: ObjectId(id),
        owner: None,
        pos,
        half: Vec2::splat(HALF),
        oriented: None,
        velocity: Vec2::ZERO,
        heading: Vec2::ZERO,
        stationary: true,
        progress: None,
        arrived_with: None,
        yields: false,
        gathers: false,
        leads: false,
        moved: 0,
        crushes: false,
        crushable: false,
    }
}

/// A walker (id 1) from `start` to `goal` among `others`, which stand: its positions after each of `ticks` ticks,
/// and its mover. `seen` is called after every tick.
fn walk(
    start: Vec2,
    goal: Vec2,
    others: &[Body],
    contact: ContactStep,
    ticks: Tick,
    mut seen: impl FnMut(Tick, Vec2, Vec2, &Mover),
) -> (Vec2, Mover) {
    let mut mover = Mover::default();
    mover.go_to(MoveGoal::Point(goal), None, 1);
    mover.follow(vec![start, goal], goal, 0);
    let (mut pos, mut facing) = (start, 0.0);
    let (mut layer, mut counters, mut lanes) = (Layer::default(), MoveCounters::default(), Vec::new());
    for tick in 1..=ticks {
        if mover.status != MoveStatus::Moving {
            break;
        }
        let me = Body {
            velocity: mover.velocity,
            heading: goal - pos,
            stationary: mover.is_stationary(),
            progress: Some(mover.last_progress),
            leads: true,
            ..standing(1, pos)
        };
        let bodies: Vec<Body> = std::iter::once(me).chain(others.iter().copied()).collect();
        layer.reset(16, 12);
        for (index, b) in bodies.iter().enumerate() {
            layer.insert(index as u32, b.pos, b.half, b.kinds());
        }
        let mut world = World {
            grid: None,
            layer: Some(&layer),
            bodies: &bodies,
            me: 0,
            contact,
            door: None,
            lanes: &mut lanes,
            counters: &mut counters,
            mine: bodies[0],
        };
        let from = pos;
        let (mut touched, mut requests) = (Vec::new(), Vec::new());
        step_unit(
            &mut pos,
            &mut facing,
            &mut 0.0,
            &mut mover,
            &WALKER,
            &mut world,
            &mut touched,
            &mut requests,
            None,
            &[],
            tick,
        );
        seen(tick, from, pos, &mover);
    }
    (pos, mover)
}

/// `systems/units.md > UnitCollision > Ground Collision`, "The contact step" (design change 0323): "the step is at
/// **90°** to its heading (straight sideways), and its length is **the unit's own width** [...] so it clears the lane
/// it was in"; "to its right first". A unit heading east meets a standing one squarely: it steps straight south,
/// its Right, 36 su in six ticks of its speed, without a step forward, and then walks on past it to its goal.
#[test]
fn a_blocked_unit_steps_straight_sideways_by_its_own_width_and_walks_on() {
    let (start, goal) = (Vec2::new(100.0, 300.0), Vec2::new(500.0, 300.0));
    let blocker = standing(2, Vec2::new(250.0, 300.0));
    // In contact at x = 250 − 36, after 19 ticks.
    let contact_x = 250.0 - 2.0 * HALF;
    let mut aside = Vec::new();
    let (end, mover) = walk(
        start,
        goal,
        &[blocker],
        ContactStep::Sideways,
        200,
        |tick, from, pos, mover| {
            if from.x == contact_x && pos.x == contact_x {
                aside.push((tick, pos.y - from.y, mover.step_aside, mover.blocked_ticks));
            }
        },
    );
    assert_eq!((end, mover.status), (goal, MoveStatus::Arrived));
    let step = Some(StepAside {
        along_y: true,
        to: 336.0,
        end: None,
    });
    let expected: Vec<_> = (0..6u16)
        .map(|i| (20 + Tick::from(i), SPEED, step.filter(|_| i < 5), i + 1))
        .collect();
    assert_eq!(
        aside, expected,
        "(tick, the step south, the step under way, blocked ticks)"
    );
    assert_eq!(mover.avoid_side, Side::Right);
}

/// A base that keeps the step as it was built before design change 0323 (`ContactStep::Diagonal`: the WheeledVehicle's
/// until slice M5-V2 built its turn and graze, now only the air bases', which ground collision never reaches) goes
/// along the face a tick at a time for as long as it is blocked, and takes no step over several ticks.
#[test]
fn a_diagonal_base_steps_as_before() {
    let (start, goal) = (Vec2::new(100.0, 300.0), Vec2::new(500.0, 300.0));
    let blocker = standing(2, Vec2::new(250.0, 300.0));
    let mut aside = 0;
    let (end, mover) = walk(
        start,
        goal,
        &[blocker],
        ContactStep::Diagonal,
        200,
        |_, from, pos, mover| {
            assert_eq!(mover.step_aside, None);
            aside += u32::from(pos.x == from.x && pos.y != from.y);
        },
    );
    assert_eq!((end, mover.status), (goal, MoveStatus::Arrived));
    assert!(aside > 0, "it went round");
}

/// The technical decisions of `docs/movement.md > 6.3`: a contact step that meets something ends there, and the
/// sides are tried as before: Right first, the other side once `avoid_until` has passed, for `AVOID_COMMIT_TICKS`.
/// South of the walker's lane stands a second unit, 10 su off: the step south ends against it after two ticks (6
/// and 4 su); the next begins to the north and is made whole, 36 su from where it began, and so is the one after it.
#[test]
fn a_step_that_meets_a_unit_ends_there_and_the_other_side_is_taken() {
    let (start, goal) = (Vec2::new(100.0, 300.0), Vec2::new(500.0, 300.0));
    let blocker = standing(2, Vec2::new(250.0, 300.0));
    let beside = standing(3, Vec2::new(214.0, 300.0 + 2.0 * HALF + 10.0));
    let mut ys = Vec::new();
    let mut switched = None;
    let (end, mover) = walk(
        start,
        goal,
        &[blocker, beside],
        ContactStep::Sideways,
        200,
        |tick, from, pos, mover| {
            if pos.x == from.x && pos.y != from.y {
                ys.push(pos.y);
            }
            if switched.is_none() && mover.avoid_side == Side::Left {
                switched = Some((tick, mover.avoid_until, mover.step_aside));
            }
        },
    );
    assert_eq!((end, mover.status), (goal, MoveStatus::Arrived));
    assert_eq!(&ys[..2], [306.0, 310.0], "south, up to the unit beside it");
    // Back north from y = 310: 36 su, the last part exactly onto the step's end. There the unit ahead is still in
    // its way, by 10 su, and a second step of its width begins.
    let north: Vec<f32> = (1..=12).map(|i| 310.0 - SPEED * i as f32).collect();
    assert_eq!(&ys[2..], north);
    let (tick, until, step) = switched.expect("it took the other side");
    assert_eq!(until, tick + AVOID_COMMIT_TICKS);
    assert_eq!(
        step,
        Some(StepAside {
            along_y: true,
            to: 274.0,
            end: None,
        })
    );
}
