//! FixedTurnRadiusMovement in a game (slice M5-V2, plan decision T79; `systems/units.md > FixedTurnRadiusMovement`,
//! `systems/units.md > WheeledVehicle`, `systems/units.md > UnitCollision > Collision Response (Un-sticking)` and
//! `systems/units.md > UnitCollision > Ground Collision`; design changes 0035, 0323 and 0353), played through the
//! gate with the Seeker (`factions/cults/units/seeker.md`: 56 × 40 su, its
//! Silhouette turning with its facing; MinimumTurnRadius 64 su, ForwardMaxSpeed 8, ForwardAcceleration 1,
//! ReverseMaxSpeed 3, ReverseAcceleration 0.5, Deceleration 1). The model's numbers alone are
//! `movement/wheeled/tests.rs`. `TestSim` checks the invariants after every tick, the wheeled invariant W1 among them
//! (`testing/invariants/momentum.rs`): a vehicle turns at most by the speed it has, never in place.
//!
//! One test binary:
//! - `speed`: from rest to full speed in its 8th tick, a Stop's 8 ticks and 28 su, braking onto its goal;
//! - `turning`: a 90° turn at full speed on its 64 su circle; the forward loop to a point just behind it (0353);
//! - `reverse`: the Reverse order (W): backing up at 3 su a tick at most, turning at |v| / r; who takes it;
//! - `walls`: pressed against a wall at an angle, and nose first (0035's Verification), it scrapes free; Rugged
//!   terrain is no ground for it;
//! - `crowds`: eight Seekers through a field of idle own infantry, which steps aside, and an idle Seeker that stays
//!   put when infantry bumps it (0353).

mod crowds;
mod reverse;
mod speed;
mod turning;
mod walls;

use sc_sim::testing::{ScenarioBuilder, TestSim};
use sc_sim::{MoveStatus, ObjectId, ObjectKind, OrderKind, PlayerId, SpawnAt, Target, TilePos, Vec2, math};

const P1: PlayerId = PlayerId(1);
const P2: PlayerId = PlayerId(2);
const SEEKER: ObjectKind = ObjectKind::Seeker;
/// The Seeker's MinimumTurnRadius, ForwardMaxSpeed and ReverseMaxSpeed.
const RADIUS: f32 = 64.0;
const TOP: f32 = 8.0;
const BACK: f32 = 3.0;

/// An open map `w` × `h` tiles.
fn flat(w: usize, h: usize) -> ScenarioBuilder {
    ScenarioBuilder::new(&vec![".".repeat(w); h].join("\n"))
}

fn centre(x: i32, y: i32) -> Vec2 {
    TilePos::new(x, y).center()
}

/// Player 1's units `ids` sent to `to` in `tick`.
fn move_to(t: &mut TestSim, tick: u32, ids: &[u32], to: Vec2) {
    let units: Vec<ObjectId> = ids.iter().map(|&id| ObjectId(id)).collect();
    t.order_at(tick, P1, &units, OrderKind::Move(Target::Point(to)), false);
}

/// Unit `id`'s facing, radians.
fn facing(t: &TestSim, id: u32) -> f32 {
    t.sim().object(ObjectId(id)).expect("the unit").facing
}

/// The angle from `from` to `to`, radians in `(-π, π]`, the shorter way.
fn turn_between(from: f32, to: f32) -> f32 {
    math::wrap_angle(to - from)
}

/// One tick of unit `id`: its displacement, and the angle it turned (signed, clockwise on screen positive).
fn step(t: &mut TestSim, id: u32) -> (Vec2, f32) {
    let (pos, was) = (t.pos(id), facing(t, id));
    t.step();
    (t.pos(id) - pos, turn_between(was, facing(t, id)))
}

/// Unit `id`'s facing as a unit vector.
fn ahead(t: &TestSim, id: u32) -> Vec2 {
    math::facing_vector(facing(t, id))
}

/// Whether unit `id` has no move under way, once the first tick (which gives the tests' orders) has run.
fn done(t: &TestSim, id: u32) -> bool {
    t.sim().tick() > 0 && !matches!(t.status(id), MoveStatus::Moving)
}
