//! SpeedTurnRadiusMovement in a game (slice M5-V3, plan decision T85 with design change 0417; `systems/units.md >
//! SpeedTurnRadiusMovement`, `systems/units.md > TrackedVehicle`, `systems/combat.md >
//! LocomotionOrientationConstraints[SpeedTurnRadiusMovement]`, `systems/units.md > UnitCollision > Collision Response
//! (Un-sticking)` and `systems/units.md > UnitCollision > Ground Collision`), played through the gate with the Hand of
//! Heaven (`factions/cults/units/hand_of_heaven.md`: 64 × 40 su, its Silhouette turning with its facing; MaxSpeed 5,
//! Acceleration 0.5, Deceleration 1, TrackWidth 30, ReverseMaxSpeed 2.5, so SpinRate 1/3 radian a tick). The model's
//! numbers alone are `movement/tracked/tests.rs`. `TestSim` checks the invariants after every tick, the tracked
//! invariant S1 among them (`testing/invariants/momentum.rs`): a unit turns at most by the law at the speed it has.
//!
//! One test binary:
//! - `spin`: standing still it turns 90° in its 5th tick, and spins round to a point behind it without moving, then
//!   drives off; a spin a wall blocks stops at contact;
//! - `law`: at full speed, ordered to a point at its side, it slows, then turns, never at MaxSpeed, on the law's
//!   radii; the Reverse order backs it up at 2.5 su a tick at most, turning by the same law;
//! - `corners`: a right-angled bend of a corridor two tiles wide, taken without touching the walls; a gentle curve
//!   within a tenth of MaxSpeed; Rugged terrain is no ground for it;
//! - `contact`: the contact step of a Hand of Heaven turned 30°, straight sideways by its own width;
//! - `standing`: an idle Hand of Heaven stands, and own infantry goes round it (design change 0440);
//! - `group_reverse`: a Reverse given to three is placed by the magic box (design change 0439).

mod contact;
mod corners;
mod group_reverse;
mod law;
mod spin;
mod standing;

use sc_sim::shape::Shape;
use sc_sim::testing::{ScenarioBuilder, TestSim};
use sc_sim::{MoveStatus, ObjectId, ObjectKind, OrderKind, PlayerId, SU_PER_GU, Target, TilePos, Vec2, math};

const P1: PlayerId = PlayerId(1);
const HAND: ObjectKind = ObjectKind::HandOfHeaven;
/// The Hand of Heaven's MaxSpeed, ReverseMaxSpeed and SpinRate (2 × 5 ÷ 30 radians a tick).
const TOP: f32 = 5.0;
const BACK: f32 = 2.5;
const SPIN: f32 = 1.0 / 3.0;

/// The tread law: the most the Hand of Heaven turns in a tick at the speed `speed`.
fn rate(speed: f32) -> f32 {
    (SPIN * (1.0 - speed.abs() / TOP)).max(0.0)
}

/// Whether a tick's move of `moved` su is one of `speed` su a tick: positions are `f32`s of hundreds of su, so a move
/// measured between two of them is off by their rounding.
fn about(moved: f32, speed: f32) -> bool {
    (moved - speed).abs() < 1e-3
}

/// Whether a tick's turn of `turned` radians is a turn at all, not the steering's rounding: the law lets a unit a
/// hair below MaxSpeed (4.9999 su a tick, which a move measures as 5) turn by a few millionths of a radian.
fn turns(turned: f32) -> bool {
    turned.abs() > 1e-4
}

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

/// Player 1's units `ids` reversed to `to` in `tick`.
fn reverse_to(t: &mut TestSim, tick: u32, ids: &[u32], to: Vec2) {
    let units: Vec<ObjectId> = ids.iter().map(|&id| ObjectId(id)).collect();
    t.order_at(tick, P1, &units, OrderKind::Reverse(to), false);
}

/// Unit `id`'s facing, radians.
fn facing(t: &TestSim, id: u32) -> f32 {
    t.sim().object(ObjectId(id)).expect("the unit").facing
}

/// One tick of unit `id`: its displacement, and the angle it turned (signed, clockwise on screen positive).
fn step(t: &mut TestSim, id: u32) -> (Vec2, f32) {
    let (pos, was) = (t.pos(id), facing(t, id));
    t.step();
    (t.pos(id) - pos, math::wrap_angle(facing(t, id) - was))
}

/// Whether unit `id` has no move under way, once the first tick (which gives the tests' orders) has run.
fn done(t: &TestSim, id: u32) -> bool {
    t.sim().tick() > 0 && !matches!(t.status(id), MoveStatus::Moving)
}

/// The least gap between unit `id`'s Silhouette, as it lies, and the tiles of the map that `solid` names: how near it
/// is to the walls.
fn gap_to_walls(t: &TestSim, id: u32, solid: impl Fn(TilePos) -> bool) -> f32 {
    let o = t.sim().object(ObjectId(id)).expect("the unit");
    let shape = Shape::silhouette(o.kind, o.pos, o.facing);
    let tile = TilePos::containing(o.pos);
    let mut least = f32::INFINITY;
    for y in tile.y - 2..=tile.y + 2 {
        for x in tile.x - 2..=tile.x + 2 {
            let at = TilePos::new(x, y);
            if solid(at) {
                let square = Shape::rect(at.center(), Vec2::splat(SU_PER_GU / 2.0));
                least = least.min(shape.gap(&square));
            }
        }
    }
    least
}
