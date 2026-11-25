//! DragMovement in a game (slice M3-G, plan decision T25; `systems/units.md > DragMovement`, `> HoverVehicle`,
//! `> UnitCollision > Collision Response (Un-sticking)`; design changes 0035 and 0039): the Drone's speed curve and
//! its equilibrium, braking and exact arrival, a Stop in full flight, Rugged terrain at half speed, a wall met at
//! speed, arrival within reach of a patch, a save in full flight, and the amended invariants I5 and I7 from their
//! wrong side. The model's numbers alone are `movement/drag/tests.rs`. `TestSim` checks the invariants after every
//! tick.
//!
//! One test binary:
//! - `flight`: the speed curve, braking and exact arrival, turning on the move, a Stop in full flight, and arrival
//!   within reach of a patch;
//! - `terrain`: Rugged terrain at half speed, a wall met at speed, and a wall the Drone only touches;
//! - `momentum`: the momentum as state: a save in full flight, the amended I5 and I7 from their wrong side, and a
//!   teleport.

mod flight;
mod momentum;
mod terrain;

use sc_sim::mover::Locomotion;
use sc_sim::orders::state::{Behaviour, GatherPhase};
use sc_sim::testing::invariants::Invariants;
use sc_sim::testing::{ScenarioBuilder, TestSim};
use sc_sim::{
    MoveGoal, MoveStatus, ObjectId, ObjectKind, OrderKind, PlayerId, Sim, SpawnAt, Target, TickInputs, TilePos, Vec2,
};

const P1: PlayerId = PlayerId(1);
const DRONE: ObjectKind = ObjectKind::Drone;
/// Half the Drone's Silhouette.
const HALF: f32 = 18.0;
/// `factions/colonists/units.md > Drone`: the equilibrium of its thrust and drag, space units per tick.
const TOP: f32 = 8.0;
/// The documented bounds of `movement::drag`: from top speed the Drone stops within this many space units and
/// ticks when it is stopped…
const STOPS_WITHIN: f32 = 42.0;
const STOPS_IN: u32 = 16;
/// … and on its way to a goal, where it plans with a reserve, from the step in which it begins to brake: that
/// step, of at most 8, and the 43 su it stops in.
const ARRIVES_WITHIN: f32 = 51.0;
const ARRIVES_IN: u32 = 17;

fn flat(w: usize, h: usize) -> ScenarioBuilder {
    ScenarioBuilder::new(&vec![".".repeat(w); h].join("\n"))
}

fn centre(x: i32, y: i32) -> Vec2 {
    TilePos::new(x, y).center()
}

fn move_to(t: &mut TestSim, tick: u32, to: Vec2) {
    t.order_at(tick, P1, &[ObjectId(1)], OrderKind::Move(Target::Point(to)), false);
}

/// Steps once and returns the Drone's displacement.
fn step(t: &mut TestSim) -> Vec2 {
    let from = t.pos(1);
    t.step().pos(1) - from
}
