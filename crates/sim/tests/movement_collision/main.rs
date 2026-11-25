//! Steering and hard collision (`docs/movement.md > 6`, `docs/movement.md > 8`; `systems/units.md > UnitCollision >
//! Ground Collision`): units pass, follow, go around and give up, and never overlap. Every `TestSim` checks the
//! invariants I1–I7 after each tick and fails with a picture of the map.
//!
//! The give-up numbers are the defaults of question Q0008, and what two units blocking each other for good do is
//! the default of question Q0007.
//!
//! One test binary:
//! - `passing`: two units meeting head-on in a corridor, which pass or give up, and an idle unit of the mover's
//!   own stepping aside;
//! - `crowds`: units through a choke, a group that arrives and stands still, no oscillation, and a save in a crowd;
//! - `paths`: goals out of reach, the one repath an object on the path forces, queued waypoints, and the counters
//!   of a group move.

mod crowds;
mod passing;
mod paths;

use sc_sim::mover::Locomotion;
use sc_sim::testing::{ScenarioBuilder, TestSim};
use sc_sim::tuning::movement::{GIVE_UP_TICKS, MAX_REPATHS};
use sc_sim::{
    DebugCommand, MoveGoal, MoveStatus, ObjectId, ObjectKind, OrderKind, PlayerId, SimEvent, SpawnAt, Target, TilePos,
    Vec2,
};

const P1: PlayerId = PlayerId(1);
const P2: PlayerId = PlayerId(2);
const PK: ObjectKind = ObjectKind::Peacekeeper; // 24 × 24 su, 4 su per tick
const GUARD: ObjectKind = ObjectKind::Guard; // 36 × 36 su, 5 su per tick

/// A corridor one tile (64 su) wide and 16 long, y from 64 to 128.
const CORRIDOR: &str = "
    ~~~~~~~~~~~~~~~~
    ................
    ~~~~~~~~~~~~~~~~
";

/// 16 × 12 open tiles: 1024 × 768 su.
const OPEN: &str = "
    ................
    ................
    ................
    ................
    ................
    ................
    ................
    ................
    ................
    ................
    ................
    ................
";

fn v(x: f32, y: f32) -> Vec2 {
    Vec2::new(x, y)
}

fn move_to(p: Vec2) -> OrderKind {
    OrderKind::Move(Target::Point(p))
}

fn ids(range: std::ops::RangeInclusive<u32>) -> Vec<ObjectId> {
    range.map(ObjectId).collect()
}

/// Steps until every unit of `units` has a status other than `Moving`; the tick. Panics, with the map, after
/// `max_tick`.
#[track_caller]
fn run_until_done(t: &mut TestSim, units: &[ObjectId], max_tick: u32) -> u32 {
    // The units are spawned in tick 1 and ordered in tick 2.
    t.run_until(t.sim().tick().max(2));
    while units.iter().any(|id| t.status(id.0) == MoveStatus::Moving) {
        assert!(
            t.sim().tick() < max_tick,
            "still moving at tick {max_tick}:\n{}",
            t.ascii(None)
        );
        t.step();
    }
    t.sim().tick()
}
