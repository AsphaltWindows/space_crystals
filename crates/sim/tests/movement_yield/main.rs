//! Friendly yielding, occupied targets and the give-up schedule (`docs/movement.md > 5.6`, `docs/movement.md > 6.5`,
//! `docs/movement.md > 6.6`), by the Verification lists of the design changes:
//! - 0022, 0023 and 0027 (`systems/units.md > UnitCollision > Ground Collision > Friendly yielding`): an idle unit
//!   takes one short sidestep for a blocked unit of its owner; enemies, units on Hold, busy units and units that
//!   can't move never do; a unit that arrived doesn't yield to the rest of its own group order.
//! - 0024 and 0026 (`systems/controls/behaviors.md > MovingToLocation > Failure`, `systems/controls/behaviors.md >
//!   Occupied target location`): stuck after 24 ticks, three re-route attempts after waits of 8, 16 and 32 ticks, then
//!   give up; arrived when stuck within 3 grid units of the destination behind units that aren't moving (2 until design
//!   change 0169).
//!
//! Every `TestSim` checks the invariants I1–I7 after each tick (`docs/movement.md > 8`): collision stays hard while
//! units sidestep.
//!
//! One test binary: `yielding` (who yields and who doesn't, 0022 and 0023; yielding replayed and saved),
//! `group_order` (a group order's units among themselves, 0027), `give_up` (the give-up schedule, 0024) and
//! `occupied` (arrived behind units that won't move, 0026 and 0169).

mod give_up;
mod group_order;
mod occupied;
mod yielding;

use sc_sim::testing::invariants::Invariants;
use sc_sim::testing::{ScenarioBuilder, TestSim};
use sc_sim::tuning::movement::{GIVE_UP_TICKS, MAX_REPATHS, OCCUPIED_ARRIVAL_GU, REPATH_WAITS, STUCK_TICKS};
use sc_sim::{
    Ability, Command, DebugCommand, MoveStatus, ObjectId, ObjectKind, OrderKind, PlayerCommand, PlayerId, Shape, Sim,
    SimEvent, SpawnAt, Target, TickInputs, TilePos, Vec2, gu,
};

const P1: PlayerId = PlayerId(1);

const P2: PlayerId = PlayerId(2);

const PK: ObjectKind = ObjectKind::Peacekeeper; // 24 × 24 su, 4 su per tick
const GUARD: ObjectKind = ObjectKind::Guard; // 36 × 36 su, 5 su per tick

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

/// A corridor one tile (64 su) wide and 16 long, y from 64 to 128.
const CORRIDOR: &str = "
    ~~~~~~~~~~~~~~~~
    ................
    ~~~~~~~~~~~~~~~~
";

/// A passage three tiles (192 su) wide and 16 long, y from 64 to 256.
const PASSAGE: &str = "
    ~~~~~~~~~~~~~~~~
    ................
    ................
    ................
    ~~~~~~~~~~~~~~~~
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

fn failures(t: &TestSim) -> usize {
    let failed = |(_, e): &&(u32, SimEvent)| matches!(e, SimEvent::MoveFailed { .. });
    t.events().iter().filter(failed).count()
}

fn is_idle(t: &TestSim, id: u32) -> bool {
    let unit = t.sim().object(ObjectId(id)).unwrap();
    unit.orders.as_ref().unwrap().is_idle()
}

/// Steps until none of `units` is moving; the tick. Panics, with the map, after `max_tick`.
#[track_caller]
fn run_until_done(t: &mut TestSim, units: &[ObjectId], max_tick: u32) -> u32 {
    t.run_until(t.sim().tick().max(2));
    while units.iter().any(|id| t.status(id.0) == MoveStatus::Moving) {
        assert!(t.sim().tick() < max_tick, "still moving\n{}", t.ascii(None));
        t.step();
    }
    t.sim().tick()
}
