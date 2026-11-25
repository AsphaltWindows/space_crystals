//! DragMovement in a game (slice M3-G, plan decision T25; `systems/units.md > DragMovement`, `> HoverVehicle`,
//! `> UnitCollision > Collision Response (Un-sticking)`; design changes 0035 and 0039): the Drone's speed curve and
//! its equilibrium, braking and exact arrival, a Stop in full flight, Rugged terrain at half speed, a wall met at
//! speed, arrival within reach of a patch, a save in full flight, and the amended invariants I5 and I7 from their
//! wrong side. The Drone's numbers are those of design change 0205 (slice M3-K3: thrust and drag ×1.5, the same
//! top speed reached sooner, and less drift when it stops). The model's numbers alone are
//! `movement/drag/tests.rs`. `TestSim` checks the invariants after every tick.
//!
//! One test binary:
//! - `flight`: the speed curve, its top speed within 1.25 s (0205), braking and exact arrival, turning on the
//!   move, a Stop in full flight and its drift (0205), and arrival within reach of a patch;
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
/// Its scheme from rest (`movement::drag`), since design change 0205 (thrust and drag ×1.5, 0.9 / 0.3 / 0.15):
/// what a tick's drag leaves of the velocity, `1 − DragRatio`, and its whole thrust along its facing,
/// ForwardAcceleration + OmniDirectionalAcceleration. Before 0205 they were 0.9 and 0.8.
const KEEP: f32 = 0.85;
const THRUST: f32 = 1.2;
/// The documented bounds of `movement::drag`: from top speed the Drone stops within this many space units and
/// ticks when it is stopped…
const STOPS_WITHIN: f32 = 26.0;
const STOPS_IN: u32 = 9;
/// … and on its way to a goal, where it plans with a reserve, from the step in which it begins to brake: that
/// step, of at most 8, and the 27 su it stops in.
const ARRIVES_WITHIN: f32 = 35.0;
const ARRIVES_IN: u32 = 11;
/// Before design change 0205 a Drone stopped at top speed came to rest 41.5 su on, after 15 ticks of braking
/// (`movement/drag/tests.rs` works it out with the numbers before 0205; the bounds were 42 su and 16 ticks).
const STOPPED_BEFORE_0205: (f32, u32) = (41.5, 15);

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
