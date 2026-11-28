//! The Prism's thrusts (`crate::thrust`; `factions/syndicate/units/prism.md > Movement`, its placeholder stats, design
//! change 0434 answering Q0168), through the gate, one test binary:
//! - `thrust`: a thrust moves it 3 grid units toward its point, whether the point is 10 grid units away or 1, its drift
//!   after it its movement's own; three spend the three charges, and the gate then refuses one (`Reject::NoCharges`),
//!   as it refuses a unit without thrusts (`Reject::LacksAbility`); Shift-queued thrusts run one after another, and
//!   one that starts with no charge left ends without moving (Placeholder (Q0198));
//! - `launch`: a thrust given while the Prism launches out of the air exit door is held, and runs once it has unfolded;
//! - `one_way`: the Prism is never let back into the network (`Reject::OneWay`), its right-click on a Tunnel a Move
//!   to it, and ordered in with a Guard it ignores the order; trained with a Null Chamber, it takes no turn and leaves
//!   unbuffed;
//! - `saves`: a game goes on alike after a save mid-burst, and the thrusts are in the save and the hash only while a
//!   unit has them.
//!
//! Every test runs under the invariants of `TestSim`, I3 and I5/I7 with their allowance for a burst among them.

mod launch;
mod one_way;
mod saves;
mod thrust;

use sc_sim::testing::{ScenarioBuilder, TestSim};
use sc_sim::{Command, Faction, Object, ObjectId, ObjectKind, OrderKind, PlayerCommand, PlayerId, Reject, SpawnAt};
use sc_sim::{Tick, Vec2};

const P1: PlayerId = PlayerId(1);
const P2: PlayerId = PlayerId(2);
const PRISM: ObjectKind = ObjectKind::Prism;
/// `factions/syndicate/units/prism.md > Placeholder stats`: "each moves it **3 grid units** over 1 s": 192 su over 16
/// ticks, 12 a tick.
const THRUST_SU: f32 = 192.0;
const THRUST_TICKS: Tick = 16;
/// "**Thrust charges: 3**".
const CHARGES: u32 = 3;

fn flat(w: usize, h: usize) -> String {
    vec![".".repeat(w); h].join("\n")
}

fn point(x: f32, y: f32) -> SpawnAt {
    SpawnAt::Point(Vec2::new(x, y))
}

fn id(n: u32) -> ObjectId {
    ObjectId(n)
}

/// Player 1 plays the Syndicate and player 2 the GDO, on a flat 64 × 24 map (x 0..4096, y 0..1536), with no
/// structure: units in the open, facing east (+x) as they come.
fn field() -> ScenarioBuilder {
    ScenarioBuilder::new(&flat(64, 24))
        .faction(P1, Faction::Syndicate)
        .faction(P2, Faction::Gdo)
}

/// `scenario` started with auto-targeting off, run to tick 1.
fn started(scenario: ScenarioBuilder) -> TestSim {
    let mut t = scenario.start();
    t.without_auto_targeting();
    t.run_until(1);
    t
}

/// The Prism's R, a point clicked (`OrderKind::Thrust`): Shift queues it.
fn thrust(units: &[ObjectId], at: Vec2, queue: bool) -> Command {
    Command::Order {
        units: units.to_vec(),
        kind: OrderKind::Thrust(at),
        queue,
    }
}

/// What the gate says to player 1's `command` now (`Sim::validate`).
fn gate(t: &TestSim, command: Command) -> Result<(), Reject> {
    t.sim().validate(&PlayerCommand { player: P1, command })
}

fn object(t: &TestSim, id: ObjectId) -> &Object {
    t.sim().object(id).expect("on the map")
}

/// The charges `id` has left (`sc_sim::thrust::charges_left`, what greys R).
fn charges(t: &TestSim, id: ObjectId) -> u32 {
    sc_sim::thrust::charges_left(object(t, id))
}

/// Steps until `done`, at most `ticks` ticks; panics naming `what` otherwise.
fn step_until(t: &mut TestSim, ticks: Tick, what: &str, done: impl Fn(&TestSim) -> bool) {
    let end = t.sim().tick() + ticks;
    while !done(t) {
        assert!(t.sim().tick() < end, "{what}: not by tick {end}");
        t.step();
    }
}
