//! The GDO's anti-air (slice M5-K, design change 0356): the Rocketman (`factions/gdo/units/rocketman.md`) and the
//! Arsenal (`factions/gdo/structures/arsenal.md`), through the gate. One test binary:
//! - `anti_air`: 0356's Verification, "Rocketmen can shoot down a Sparrow" and "A circling Sparrow can sometimes dodge
//!   the normal missile; a locked one can't", with Sparrows spawned directly: locked Rocketmen shoot down two circling
//!   Sparrows, one lock after the other; a circling Sparrow dodges the normal missiles and takes every locked one; and
//!   a strafing Sparrow, which player 1's buildings keep in sight, falls to locks given again whenever none is under
//!   way, as a player would;
//! - `min_range`: the normal attack never fires within its MinRange of 2 grid units, backing off first; a locked
//!   Rocketman fires from where it stands at any distance up to LockRange;
//! - `prerequisites`: the Barracks' W refused without an Arsenal, the Arsenal without a Barracks; losing the last
//!   Arsenal refuses W again, and the Rocketmen already queued still come out; the Arsenal extends the build area by 2.
//!
//! Target Lock's own rules (the locking, its four breaks, a Shift-queued order behind it, a lock on a structure, the
//! gate) are the sim's unit tests (`combat/target_lock/tests.rs`), HeadDisjointed's
//! `combat/projectiles/homing/tests.rs`. Every `TestSim` game runs under the invariants.

mod anti_air;
mod min_range;
mod prerequisites;

use sc_sim::testing::{ScenarioBuilder, TestSim};
use sc_sim::{Faction, ObjectId, ObjectKind, OrderKind, PlayerId, SimEvent, SpawnAt, Tick, TilePos, Vec2};

const P1: PlayerId = PlayerId(1);
const P2: PlayerId = PlayerId(2);
const ROCKETMAN: ObjectKind = ObjectKind::Rocketman;
const ARSENAL: ObjectKind = ObjectKind::Arsenal;
const SPARROW: ObjectKind = ObjectKind::Sparrow;

/// `factions/gdo/units/rocketman.md > Placeholder stats`: the missile's Damage 20, which the Sparrow's PointArmor 0
/// leaves whole, against its MaxHP 50 (`factions/cults/units/sparrow.md`): three hits bring it down.
const DAMAGE: f32 = 20.0;
const SPARROW_HP: f32 = 50.0;
/// `factions/gdo/units/rocketman.md > Target Lock`: LockOnDuration 48 frames.
const LOCK_ON: Tick = 48;

/// A flat `w` × `h` map of open tiles.
fn flat(w: usize, h: usize) -> String {
    vec![".".repeat(w); h].join("\n")
}

/// [`flat`] with player 1 the GDO's and player 2 the Cults', whose Sparrows they are.
fn field(w: usize, h: usize) -> ScenarioBuilder {
    ScenarioBuilder::new(&flat(w, h))
        .faction(P1, Faction::Gdo)
        .faction(P2, Faction::Cults)
}

fn point(x: f32, y: f32) -> SpawnAt {
    SpawnAt::Point(Vec2::new(x, y))
}

fn tile(x: i32, y: i32) -> SpawnAt {
    SpawnAt::Tile(TilePos::new(x, y))
}

fn id(n: u32) -> ObjectId {
    ObjectId(n)
}

/// `player`'s `units` ordered `kind` in tick `tick`, queued or not.
fn order(t: &mut TestSim, tick: Tick, player: PlayerId, units: &[u32], kind: OrderKind, queue: bool) {
    let units: Vec<ObjectId> = units.iter().copied().map(ObjectId).collect();
    t.order_at(tick, player, &units, kind, queue);
}

/// Whether object `n` is in the game.
fn alive(t: &TestSim, n: u32) -> bool {
    t.sim().object(id(n)).is_some()
}

/// Unit `n`'s current order.
fn current(t: &TestSim, n: u32) -> Option<OrderKind> {
    let orders = t.sim().object(id(n))?.orders.as_ref()?;
    orders.current.as_ref().map(|active| active.order.kind)
}

/// A missile launched so far at `target` (`SimEvent::ProjectileLaunched`): its tick, its shooter, whether it homes
/// (a locked one, `factions/gdo/units/rocketman.md > Target Lock`) and when it lands if the target stands still.
#[derive(Copy, Clone, Debug, PartialEq)]
struct Missile {
    tick: Tick,
    shooter: u32,
    homing: bool,
    lands_at: Tick,
}

fn missiles_at(t: &TestSim, target: u32) -> Vec<Missile> {
    (t.events().iter())
        .filter_map(|(tick, e)| match *e {
            SimEvent::ProjectileLaunched {
                shooter,
                target: Some(aimed),
                homing,
                lands_at,
                ..
            } if aimed == id(target) => Some(Missile {
                tick: *tick,
                shooter: shooter.0,
                homing,
                lands_at,
            }),
            _ => None,
        })
        .collect()
}

/// The hits on `target` so far: (tick, attacker, HP taken).
fn hits_on(t: &TestSim, target: u32) -> Vec<(Tick, u32, f32)> {
    (t.events().iter())
        .filter_map(|(tick, e)| match *e {
            SimEvent::Hit {
                attacker,
                target: hit,
                damage,
                ..
            } if hit == id(target) => Some((*tick, attacker.0, damage)),
            _ => None,
        })
        .collect()
}

/// The tick `n` died in, if it did.
fn died(t: &TestSim, n: u32) -> Option<Tick> {
    (t.events().iter()).find_map(|(tick, e)| match *e {
        SimEvent::Died { id: dead, .. } if dead == id(n) => Some(*tick),
        _ => None,
    })
}
