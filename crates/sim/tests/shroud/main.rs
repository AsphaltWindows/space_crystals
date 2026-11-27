//! Shroud (slice M5-N4b, plan decisions T76 and T77 as design change 0334 amends them;
//! `factions/cults/units/disciple.md > Spells > Shroud`, `factions/cults/units/disciple.md > Placeholder stats > Shroud
//! numbers`), played through the gate, one test binary:
//! - `absorption`: what a cloud absorbs and what its release does: a lone Militant shot once for 4 from outside takes
//!   the area minimum, 0.5, 8 frames later; fire from under the same cloud hits in full; a grenade is judged by where it
//!   was thrown from; a release has no direction; a lowered pop-up turret is protected and takes its share;
//! - `clouds`: which circles are one cloud and when the buffers go: 5 grid units apart one buffer, 8 apart two; a
//!   Disciple channelling between two clouds makes both release at once, and then they buffer together; a Stop on one
//!   of a joined pair releases the shared buffer at once over both circles, (M6-CM) as its lingering circle vanishes;
//! - `channel`: what ends a channel and what starts one: damage doesn't, a new order does, and running dry does, 711
//!   ticks after it started from full; below 10 energy the gate refuses it, and a queued one that starts below 10 ends
//!   at once; (M5-N11) a Shift-queued Move waits behind the channel and a plain one ends it at once (design change
//!   0438's Verification);
//! - (M6-CM) `linger`: a cloud lingers 24 ticks where it was after a Stop and after its Disciple's death, absorbing a
//!   shot fired into it then, and releases its buffer as it vanishes (design change 0448's Verification, step 8);
//! - `state`: a replay with clouds splitting and merging reproduces the hash; the clouds saved and loaded, and hashed
//!   only while there are any;
//! - `reveals` (design change 0386): an absorbed shot or blast reveals its unseen attacker to the owner of the object
//!   it was aimed at for 1.5 s, the damage buffered; the release reveals nobody.
//!
//! The clouds' bookkeeping on its own and the release over one circle that holds a 3x3 building wholly are the sim's
//! unit tests (`shroud/tests.rs`, `shroud/lattice.rs`, `combat/damage/tests.rs`). Every `TestSim` game runs under the
//! invariants, the cloud invariant C1 among them (`testing/invariants/clouds.rs`).

mod absorption;
mod channel;
mod clouds;
mod linger;
mod reveals;
mod state;

use sc_sim::shroud::{Cloud, lattice::Union};
use sc_sim::testing::{ScenarioBuilder, TestSim};
use sc_sim::{Faction, ObjectId, ObjectKind, OrderKind, PlayerId, SimEvent, SpawnAt, Tick, Vec2};

const P1: PlayerId = PlayerId(1);
const P2: PlayerId = PlayerId(2);
const DISCIPLE: ObjectKind = ObjectKind::Disciple;
const MILITANT: ObjectKind = ObjectKind::Militant;
/// A cloud's radius, space units: "cloud radius **3 grid units** around the Disciple".
const RADIUS: f32 = 192.0;
/// (M6-CM) How long a channel's circle stays once it ends, ticks: "the cloud **stays for 1.5 s (24 frames,
/// placeholder)**" (design change 0448; `sc_sim::tuning::spells`' `linger`).
const LINGER: Tick = 24;

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

/// Object `id`'s HP.
fn hp(t: &TestSim, id: u32) -> f32 {
    t.sim().object(ObjectId(id)).and_then(|o| o.hp).expect("alive")
}

/// The clouds, each as its members' ids.
fn clouds(t: &TestSim) -> Vec<Vec<u32>> {
    let ids = |c: &Cloud| c.members.iter().map(|m| m.id.0).collect();
    t.sim().clouds().list().iter().map(ids).collect()
}

/// The buffer of the cloud whose first member is `first`; 0 while there is none.
fn buffer(t: &TestSim, first: u32) -> f32 {
    let cloud = (t.sim().clouds().list().iter()).find(|c| c.members[0].id == ObjectId(first));
    cloud.map_or(0.0, |c| c.buffer)
}

/// Every release so far: its tick, its circles' centres and its damage.
fn releases(t: &TestSim) -> Vec<(Tick, Vec<Vec2>, f32)> {
    (t.events().iter())
        .filter_map(|(tick, e)| match e {
            SimEvent::ShroudReleased { circles, damage } => {
                Some((*tick, circles.iter().map(|&(at, _)| at).collect(), *damage))
            }
            _ => None,
        })
        .collect()
}

/// Every hit on object `id` so far: its tick, its attacker and the HP it took.
fn hits_on(t: &TestSim, id: u32) -> Vec<(Tick, u32, f32)> {
    (t.events().iter())
        .filter_map(|(tick, e)| match *e {
            SimEvent::Hit {
                attacker,
                target,
                damage,
                ..
            } if target == ObjectId(id) => Some((*tick, attacker.0, damage)),
            _ => None,
        })
        .collect()
}

/// Steps `t` until `done` holds, at most `ticks` ticks; panics if it never does.
fn step_until(t: &mut TestSim, ticks: u32, what: &str, done: impl Fn(&TestSim) -> bool) {
    for _ in 0..ticks {
        if done(t) {
            return;
        }
        t.step();
    }
    assert!(done(t), "{what} within {ticks} ticks");
}

/// How many of the lattice's points the cloud whose first member is `first` covers, and how many of them fall in the
/// box of object `id` (`sc_sim::shroud::lattice`): a release's area and that object's part of it.
fn areas(t: &TestSim, first: u32, id: u32) -> (f32, f32) {
    let cloud = (t.sim().clouds().list().iter()).find(|c| c.members[0].id == ObjectId(first));
    let union = Union::of(&cloud.expect("a cloud").members);
    let o = t.sim().object(ObjectId(id)).expect("in the game");
    let (lo, hi) = sc_sim::Shape::of(o).bounds();
    (union.count() as f32, union.overlap(lo, hi).0 as f32)
}
