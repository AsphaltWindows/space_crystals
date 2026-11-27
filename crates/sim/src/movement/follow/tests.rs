//! One DragMovement unit's ticks against tiles, without a game: the creep of design change 0035 and the stuck
//! clock. The scenario tests are `crates/sim/tests/movement_drag/`.

use super::*;
use crate::map::Map;
use crate::movement::MoveCounters;
use crate::movement::space::Body;
use crate::nav::{Nav, NavClass, NavRules};
use crate::tuning::movement::MAX_REPATHS;

/// `factions/colonists/units/drone.md > UnitBaseAttributes[HoverVehicle]`, since design change 0205
/// (`movement/drag/tests.rs` checks them against the Drone's def).
const DRONE: MovementAttrs = MovementAttrs::Drag {
    forward_acceleration: 0.9,
    omni_directional_acceleration: 0.3,
    drag_ratio: 0.15,
    turn_rate: core::f32::consts::FRAC_PI_4,
};

/// 0035's Implementation impact: "Make sure creeping along an obstacle still counts as progress where it actually
/// gets closer, so the stuck/give-up rule (0024) doesn't fire on a unit that is successfully scraping free."
///
/// A Drone rests against a wall to its south, and its steering target lies far east and a little south, behind
/// the wall's line: every tick its move runs into the wall, the blocked part is dropped and its velocity is 0, so
/// it creeps east at single-tick acceleration, 1.2 su a tick (0.8 before design change 0205), where a free Drone
/// would be at 8. For 400 ticks, four times the give-up time, it is never stuck, asks for no other route and
/// doesn't fail: it is getting closer all the while.
#[test]
fn a_drone_creeping_along_a_wall_makes_progress_and_never_gives_up() {
    let rows = [".".repeat(48), ".".repeat(48), "M".repeat(48), "M".repeat(48)];
    let map = Map::from_ascii(&rows.join("\n")).unwrap();
    let nav = Nav::build(&map, &[], &[(NavClass::GroundRugged, 1)], NavRules::default());
    let grid = nav.grid(NavClass::GroundRugged, 1).unwrap();
    let half = Vec2::splat(18.0);
    // Its box rests on the wall, which begins at y = 128.
    let start = Vec2::new(96.0, 128.0 - half.y);
    let target = Vec2::new(2900.0, start.y + 150.0);
    let mut mover = Mover::default();
    mover.go_to(MoveGoal::Point(target), None, 1);
    mover.follow(vec![start, target], target, 0);
    let (mut pos, mut facing) = (start, 0.0);
    let mut counters = MoveCounters::default();
    for tick in 1..=400 {
        let bodies = [Body {
            id: ObjectId(1),
            owner: None,
            pos,
            half,
            oriented: None,
            velocity: mover.velocity,
            heading: target - pos,
            stationary: mover.is_stationary(),
            progress: Some(mover.last_progress),
            arrived_with: None,
            yields: false,
            gathers: false,
            leads: true,
            moved: 0,
        }];
        let mut world = World {
            grid: Some(grid),
            layer: None,
            bodies: &bodies,
            me: 0,
            contact: crate::defs::ContactStep::Sideways,
            door: None,
            lanes: &mut Vec::new(),
            counters: &mut counters,
            mine: bodies[0],
        };
        let from = pos;
        let failed = step_unit_on(
            &mut pos,
            &mut facing,
            &mut 0.0,
            &mut mover,
            &DRONE,
            false,
            &mut world,
            &mut Vec::new(),
            &mut Vec::new(),
            None,
            &[],
            tick,
        );
        let step = pos - from;
        assert!(!failed && mover.status == MoveStatus::Moving, "tick {tick}: {mover:?}");
        assert_eq!(pos.y, start.y, "tick {tick}: along the wall, never into it");
        assert!(
            step.x > 1.05 && step.x <= 1.2 + 1e-4,
            "tick {tick}: single-tick acceleration: {step}"
        );
        assert_eq!(mover.momentum, Vec2::ZERO, "tick {tick}: it met the wall");
        assert!(!mover.is_stuck(tick), "tick {tick}: creeping closer is progress");
        assert_eq!(mover.repaths_left, MAX_REPATHS, "tick {tick}");
        assert_eq!(mover.path_state, PathState::Ready, "tick {tick}");
        assert!(
            tick - mover.last_progress <= 8,
            "tick {tick}: last progress {}",
            mover.last_progress
        );
    }
    assert!(pos.x > start.x + 300.0);
}
