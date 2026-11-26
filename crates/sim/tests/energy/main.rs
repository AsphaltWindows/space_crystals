//! Energy (plan decision T75, slice M5-N4a; `factions/cults/units/disciple.md > Energy`: "The Disciple has its own
//! energy pool, which it spends on its spells. It regenerates naturally over time"; `> Placeholder stats`: "Energy -
//! MaxEnergy 200; starts at 50; regenerates 0.5 per second"), one test binary:
//! - `regeneration`: a Disciple's energy from its start to its maximum, exactly, in integers, on the map and inside a
//!   building alike;
//! - `state`: the energy in the state hash only while set, and in saves, old ones included.
//!
//! The Disciple the Cathedral trains is `tests/cathedral/`'s; the integers and a pool that regenerates by the units
//! inside it are the sim's unit tests' (`energy/tests.rs`), which can give a type a pool.

mod regeneration;
mod state;

use sc_sim::testing::{ScenarioBuilder, TestSim};
use sc_sim::{Faction, ObjectId, ObjectKind, PlayerId, SpawnAt, TilePos, Vec2};

const P1: PlayerId = PlayerId(1);
const DISCIPLE: ObjectKind = ObjectKind::Disciple;

/// A point of energy in `Object::energy`'s units, and what 0.5 a second adds in a tick.
const ONE: u32 = sc_sim::energy::ONE;
const A_TICK: u32 = 2048;

fn point(x: f32, y: f32) -> SpawnAt {
    SpawnAt::Point(Vec2::new(x, y))
}

/// A flat 30 × 22 map for the Cults' player 1.
fn cults() -> ScenarioBuilder {
    ScenarioBuilder::new(&vec![".".repeat(30); 22].join("\n")).faction(P1, Faction::Cults)
}

/// Object `id`'s energy, in `Object::energy`'s units, on the map or held.
fn raw(t: &TestSim, id: ObjectId) -> Option<u32> {
    let o = (t.sim().object(id)).or_else(|| t.sim().held_unit(id).map(|held| &held.object));
    o.expect("in the game").energy
}

fn tile(x: i32, y: i32) -> SpawnAt {
    SpawnAt::Tile(TilePos::new(x, y))
}
