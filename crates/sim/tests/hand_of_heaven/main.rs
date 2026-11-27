//! The Hand of Heaven (slice M5-N10; `factions/cults/units/hand_of_heaven.md`, design changes 0361 and 0363): the
//! Cults' tracked missile platform, built at the Chop Shop of three Recruits (`factions/cults/structures/chop_shop.md >
//! Production`, design change 0444: "**W: Build Hand of Heaven**"), which aims by turning its hull, its launcher
//! barely turning (20° in all), and fires a slow DoublyDisjointed area missile out to 14 grid units. One test binary:
//! - `verification`: design change 0363's Verification: fired at a building 12 tiles away, it aims for 3 s (48 ticks),
//!   launches, and drives off at once on a Move, its Reload counting on; the missile lands 96 ticks later, about 6 s;
//!   a unit inside its MinRange of 4 tiles is never fired at: it backs off first;
//! - `hull`: stopped and facing 90° away from a building in range, it spins its hull until the building is within its
//!   launcher's arc, then aims, then fires; so it does for an Attack Ground at a point behind it;
//! - `blast`: the missile's 600 over its circle of 96 su: a 3x3 building takes it all, less its armour, a lone 24x24
//!   infantry about 12, the shooter's own units in the circle too (friendly fire), and a target that drives off in the
//!   missile's flight dodges it;
//! - `reveal`: fired from beyond the target's owner's sight, with a spotter, it is revealed to that owner when the
//!   missile lands, not at the launch;
//! - `production`: W at the Chop Shop takes three free Recruits of the kind and 150 Space Crystals, and 480 ticks later
//!   one comes out a Hand of Heaven at full HP, the other two gone into it; with two inside W is refused; X refunds.
//!
//! Every `TestSim` game runs under the invariants, the tracked invariant S1 among them. Units are spawned directly,
//! with auto-targeting off, so that nobody fights but on orders.

mod blast;
mod hull;
mod production;
mod reveal;
mod verification;

use sc_sim::combat::AttackPhase;
use sc_sim::testing::{ScenarioBuilder, TestSim};
use sc_sim::{Faction, ObjectId, ObjectKind, OrderKind, PlayerId, SU_PER_GU, SimEvent, SpawnAt, Tick, TilePos, Vec2};

const P1: PlayerId = PlayerId(1);
const P2: PlayerId = PlayerId(2);
const HAND: ObjectKind = ObjectKind::HandOfHeaven;

/// `factions/cults/units/hand_of_heaven.md > Placeholder stats`: "AimDuration **48** frames (3 s); FiringDuration 2;
/// CooldownDuration **4**; ReloadDuration **96** (6 s)", "ProjectileSpeed **8 su/frame**", "AoERadius **96 su (1.5
/// GU)**", "Damage **600** total over the circle", "Range **14 grid units**; MinRange **4 grid units**".
const AIM: Tick = 48;
const FIRING: Tick = 2;
const COOLDOWN: Tick = 4;
const RELOAD: Tick = 96;
const SPEED: f32 = 8.0;
const RADIUS: f32 = 96.0;
const DAMAGE: f32 = 600.0;
const MIN_RANGE: f32 = 4.0;

/// Twelve tiles, in space units: a shot at a point this far flies 768 / 8 = 96 ticks.
const TWELVE_TILES: f32 = 12.0 * SU_PER_GU;

/// A flat `w` × `h` map of open tiles.
fn flat(w: usize, h: usize) -> String {
    vec![".".repeat(w); h].join("\n")
}

/// The centre of tile `(x, y)`.
fn centre(x: i32, y: i32) -> Vec2 {
    TilePos::new(x, y).center()
}

fn point(at: Vec2) -> SpawnAt {
    SpawnAt::Point(at)
}

fn tile(x: i32, y: i32) -> SpawnAt {
    SpawnAt::Tile(TilePos::new(x, y))
}

fn id(n: u32) -> ObjectId {
    ObjectId(n)
}

/// Both players play the Cults, on a flat `w` × `h` map.
fn cults(w: usize, h: usize) -> ScenarioBuilder {
    ScenarioBuilder::new(&flat(w, h))
        .faction(P1, Faction::Cults)
        .faction(P2, Faction::Cults)
}

/// `player`'s `units` ordered `kind` in tick `tick`, not queued.
fn order(t: &mut TestSim, tick: Tick, player: PlayerId, units: &[u32], kind: OrderKind) {
    let units: Vec<ObjectId> = units.iter().copied().map(ObjectId).collect();
    t.order_at(tick, player, &units, kind, false);
}

/// A projectile launched so far (`SimEvent::ProjectileLaunched`).
#[derive(Copy, Clone, Debug, PartialEq)]
struct Launch {
    tick: Tick,
    shooter: ObjectId,
    target: Option<ObjectId>,
    from: Vec2,
    to: Vec2,
    lands_at: Tick,
}

fn launches(t: &TestSim) -> Vec<Launch> {
    (t.events().iter())
        .filter_map(|(tick, e)| match *e {
            SimEvent::ProjectileLaunched {
                shooter,
                target,
                from,
                to,
                lands_at,
                homing,
            } => {
                assert!(!homing, "no homing");
                Some(Launch {
                    tick: *tick,
                    shooter,
                    target,
                    from,
                    to,
                    lands_at,
                })
            }
            _ => None,
        })
        .collect()
}

/// The blasts so far (`SimEvent::Blast`): their tick, attacker and centre; each of the AoERadius, 96 su.
fn blasts(t: &TestSim) -> Vec<(Tick, Option<ObjectId>, Vec2)> {
    (t.events().iter())
        .filter_map(|(tick, e)| match *e {
            SimEvent::Blast { at, radius, attacker } => {
                assert_eq!(radius, RADIUS, "an AoERadius of 96 su");
                Some((*tick, attacker, at))
            }
            _ => None,
        })
        .collect()
}

/// The hits on `target` so far: (tick, attacker, HP taken).
fn hits_on(t: &TestSim, target: u32) -> Vec<(Tick, ObjectId, f32)> {
    (t.events().iter())
        .filter_map(|(tick, e)| match *e {
            SimEvent::Hit {
                attacker,
                target: hit,
                damage,
                ..
            } if hit == id(target) => Some((*tick, attacker, damage)),
            _ => None,
        })
        .collect()
}

/// Unit `n`'s attack phase now.
fn phase(t: &TestSim, n: u32) -> Option<AttackPhase> {
    let o = t.sim().object(id(n)).expect("on the map");
    o.attack.as_ref().and_then(|attack| attack.phase)
}

/// Unit `n`'s facing, radians.
fn facing(t: &TestSim, n: u32) -> f32 {
    t.sim().object(id(n)).expect("on the map").facing
}

/// Runs until `done` holds, within `limit` ticks from now.
fn until(t: &mut TestSim, limit: Tick, what: &str, done: impl Fn(&TestSim) -> bool) {
    let end = t.sim().tick() + limit;
    while !done(t) {
        assert!(t.sim().tick() < end, "tick {}: {what}", t.sim().tick());
        t.step();
    }
}
