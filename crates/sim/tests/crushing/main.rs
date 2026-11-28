//! Crushing in a game (slice M5-V4, plan decision T86 with design changes 0418 and 0419; `systems/units.md >
//! UnitCollision > Crushing`, `docs/movement.md > 6.13`), played through the gate with the Hand of Heaven
//! (`factions/cults/units/hand_of_heaven.md`: a TrackedVehicle, 64 × 40 su, MaxSpeed 5, ReverseMaxSpeed 2.5) among
//! Light Infantry (the Peacekeeper and the Recruit, 24 × 24 su, Crushable) and the Guard (Heavy Infantry, 36 × 36 su,
//! "cannot be crushed"). Auto-targeting is off: nothing here dies of damage. `TestSim` checks the invariants after
//! every tick, the no-overlap invariant I1 among them: a crushed unit is out of the game by the end of the tick it
//! was crushed in, so no exception is needed.
//!
//! One test binary:
//! - `braking` (design change 0453): a tank sent onto a spot covered by enemy Peacekeepers crushes those it overlaps
//!   and then blocks the others; one given Stop just before enemy infantry coasts into it and crushes it; one braking to
//!   set off the other way crushes what it brakes into;
//! - `spin`: design change 0418's Verification: an enemy Light Infantry against a stopped tank's side dies as the
//!   tank turns to face it; an own Recruit there stops the turn at contact, and nobody dies; a tank that stands and
//!   doesn't turn lets no enemy infantry walk into it;
//! - `line`: driving through a line of 5 enemy Peacekeepers it crushes them all and its path stays straight; backing up
//!   through one crushes it; a Guard is never crushed, and the tank goes round it;
//! - `friendly`: own Light Infantry stop it, and it goes round them ("No friendly crushing"; there are no allies yet,
//!   every player is their own side);
//! - `soak`: Hands of Heaven among both players' Peacekeepers, Recruits and Guards under random orders.
//!
//! The surfaced Subterranean Transport's crushing (design change 0419, slice M6-ST1) is `tests/transport`'s (`combat`).

mod braking;
mod friendly;
mod line;
mod soak;
mod spin;

use sc_sim::testing::{ScenarioBuilder, TestSim};
use sc_sim::{ObjectId, ObjectKind, OrderKind, PlayerId, SimEvent, Target, Tick, TilePos, Vec2};

const P1: PlayerId = PlayerId(1);
const P2: PlayerId = PlayerId(2);
const HAND: ObjectKind = ObjectKind::HandOfHeaven;
const PEACEKEEPER: ObjectKind = ObjectKind::Peacekeeper;
const RECRUIT: ObjectKind = ObjectKind::Recruit;
const GUARD: ObjectKind = ObjectKind::Guard;
/// The Hand of Heaven's half length and half width, and a Light Infantry's half size, su.
const HAND_HALF: Vec2 = Vec2::new(32.0, 20.0);
const INFANTRY_HALF: f32 = 12.0;

/// An open map `w` × `h` tiles.
fn flat(w: usize, h: usize) -> ScenarioBuilder {
    ScenarioBuilder::new(&vec![".".repeat(w); h].join("\n"))
}

fn centre(x: i32, y: i32) -> Vec2 {
    TilePos::new(x, y).center()
}

/// `player`'s unit `id` sent to `to` in `tick`.
fn move_to(t: &mut TestSim, tick: Tick, player: PlayerId, id: u32, to: Vec2) {
    t.order_at(tick, player, &[ObjectId(id)], OrderKind::Move(Target::Point(to)), false);
}

/// Unit `id`'s facing, radians.
fn facing(t: &TestSim, id: u32) -> f32 {
    t.sim().object(ObjectId(id)).expect("the unit").facing
}

fn alive(t: &TestSim, id: u32) -> bool {
    t.sim().object(ObjectId(id)).is_some()
}

/// Every crush so far: its tick, the crushed unit and its crusher.
fn crushes(t: &TestSim) -> Vec<(Tick, ObjectId, ObjectId)> {
    (t.events().iter())
        .filter_map(|(tick, event)| match *event {
            SimEvent::Crushed { id, crusher, .. } => Some((*tick, id, crusher)),
            _ => None,
        })
        .collect()
}

/// Whether anything died of damage so far: nothing may here.
fn any_died(t: &TestSim) -> bool {
    t.events().iter().any(|(_, e)| matches!(e, SimEvent::Died { .. }))
}
