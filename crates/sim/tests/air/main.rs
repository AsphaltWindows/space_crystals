//! Gliders and air units (slice M5-H1, plan decision T81): Sparrows (`factions/cults/units/sparrow.md`) played through
//! the gate, one test binary:
//! - `flight`: the Glider's flight (`systems/units.md > GliderMovement`): it speeds up by its Acceleration to MaxSpeed
//!   on orders, its full-speed 180° turn has a radius of v² / a = 200 su, and at its goal it circles the point at
//!   IdleSpeed on the idle circle of 1 grid unit (design change 0354), for ever: its Move never completes;
//! - `strafing`: an Attack makes pass after pass over a stationary target, firing only while the target is within the
//!   turret's 45° cone and the Range of 3 grid units, flying on until out of range and looping back at full speed
//!   (`systems/controls/command_behaviors.md > AttackingObject`, GliderMovement exception; Q0120); (M5-H2) a lone
//!   Sparrow, with no spotter, until its target dies, its SightRange of 8 grid units keeping the target in sight
//!   through its loops (design change 0403);
//! - `orders`: Stop circles where the Sparrow is, Hold circles there and fires only at what comes into range, without
//!   chasing (`factions/cults/units/sparrow.md > Flight rules`);
//! - `attack_move`: an AttackMove strafes what it meets on the way and ends in idle circling over its destination
//!   (`systems/controls/command_behaviors.md > AttackMovingToLocation`, GliderMovement exception);
//! - `edge`: circling near the map's edge takes it past the edge, where it stays visible and attackable as at the
//!   nearest edge tile; a click past the edge sends it to the edge (`systems/units.md > Air Collision`);
//! - `separation`: six Sparrows sent to one point circle it without stacking (`systems/units.md > Air Collision`:
//!   SeparationRadius);
//! - `shroud`: Shroud doesn't protect air units: a Sparrow under a cloud takes an anti-air attack's hits in full
//!   (`factions/cults/units/disciple.md > Spells > Shroud`: "not air units").
//!
//! The model alone (speeds, turns, the idle circle, a strafing run's parts) and the soft separation's rules are the
//! sim's unit tests (`movement/glider/tests.rs`, `movement/air/tests.rs`); the HoverCraft cases of the behaviours,
//! `orders/tests/air.rs`. Every `TestSim` game runs under the invariants, the Glider invariant G1 among them
//! (`testing/invariants/gliders.rs`).

mod attack_move;
mod edge;
mod flight;
mod orders;
mod separation;
mod shroud;
mod strafing;

use sc_sim::testing::{ScenarioBuilder, TestSim};
use sc_sim::{Faction, ObjectId, ObjectKind, OrderKind, PlayerId, SimEvent, SpawnAt, Target, Tick, Vec2};

const P1: PlayerId = PlayerId(1);
const P2: PlayerId = PlayerId(2);
const SPARROW: ObjectKind = ObjectKind::Sparrow;
/// The Sparrow's GliderMovement (design change 0355): IdleSpeed and MaxSpeed, su a tick.
const IDLE_SPEED: f32 = 4.0;
const MAX_SPEED: f32 = 10.0;
/// The idle circle's radius: 1 grid unit (design change 0354).
const IDLE_RADIUS: f32 = 64.0;
/// The Sparrow's Range, 3 grid units, and its SeparationRadius (design change 0328).
const RANGE: f32 = 192.0;
const SEPARATION: f32 = 32.0;

fn point(x: f32, y: f32) -> SpawnAt {
    SpawnAt::Point(Vec2::new(x, y))
}

/// A flat `w` × `h` map, both players the Cults'.
fn flat(w: usize, h: usize) -> ScenarioBuilder {
    ScenarioBuilder::new(&vec![".".repeat(w); h].join("\n"))
        .faction(P1, Faction::Cults)
        .faction(P2, Faction::Cults)
}

/// `t` with auto-targeting off, so that only the orders fight.
fn quiet(mut t: TestSim) -> TestSim {
    t.without_auto_targeting();
    t
}

/// `player`'s `units` ordered `kind` in tick `tick`, not queued.
fn order(t: &mut TestSim, tick: Tick, player: PlayerId, units: &[u32], kind: OrderKind) {
    let units: Vec<ObjectId> = units.iter().copied().map(ObjectId).collect();
    t.order_at(tick, player, &units, kind, false);
}

fn move_to(x: f32, y: f32) -> OrderKind {
    OrderKind::Move(Target::Point(Vec2::new(x, y)))
}

/// Unit `id`'s speed: the length of its momentum, a Glider's speed (`Mover::momentum`).
fn speed(t: &TestSim, id: u32) -> f32 {
    t.mover(id).momentum.length()
}

/// Unit `id`'s current order.
fn current(t: &TestSim, id: u32) -> Option<OrderKind> {
    let orders = t.sim().object(ObjectId(id))?.orders.as_ref()?;
    orders.current.as_ref().map(|active| active.order.kind)
}

/// Every hit so far: its tick, its attacker, its target and the HP it took.
fn hits(t: &TestSim) -> Vec<(Tick, u32, u32, f32)> {
    (t.events().iter())
        .filter_map(|(tick, e)| match *e {
            SimEvent::Hit {
                attacker,
                target,
                damage,
                ..
            } => Some((*tick, attacker.0, target.0, damage)),
            _ => None,
        })
        .collect()
}

/// Whether `a` and `b` differ by at most `tolerance`.
fn near(a: f32, b: f32, tolerance: f32) -> bool {
    (a - b).abs() <= tolerance
}
