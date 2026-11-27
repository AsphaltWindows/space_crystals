//! (M6-CM) The Disciple's Mirage and its illusions (`factions/cults/units/disciple.md > Spells > Mirage`; design change
//! 0448, with design change 0450's correction and 0452's answer to Q0185; `sc_sim::mirage`), played through the gate,
//! one test binary:
//! - `cast`: F's four illusions north, east, south and west of the Disciple, 1 grid unit off, for 50 energy (0448's
//!   Verification, step 1), with its HP, MaxHP and speed, Augmentations included; a blocked spot's illusion at the
//!   nearest free one; the gate's 50 energy, and no illusion casting; a recast while channelling, which ends the
//!   channel and replaces the set (step 5);
//! - `illusions`: no Unit Control, and a Disciple's sight (step 2); double damage, a release's share too (step 3); the
//!   Altar refuses them and the Shrine heals them; an illusion dies alone, and its parent's death takes them all
//!   (step 6);
//! - `mirroring`: the channelling stance without a cloud, each illusion stopping alone or with its parent (step 4), and
//!   the parent's energy on every illusion, live (0452's Verification);
//! - `garrison`: illusions go into a Garrison and fire nothing from it (0450's Verification, step 3, which is 0448's
//!   step 7 as 0450 has it), and vanish there with their parent;
//! - `state`: a game with illusions saved, loaded and replayed alike, the link in the files and the hash of an illusion
//!   only.
//!
//! The cloud's linger (0448's step 8) is `tests/shroud`'s (`linger`); what an enemy sees of an illusion, the client's
//! (`crates/game/tests/commands/mirage.rs`) and the computer players' `View`'s (`crates/ai/src/view/tests.rs`). Every
//! game runs under the invariants of `TestSim`, the illusions' M1 among them (`testing/invariants/illusions.rs`).

mod cast;
mod garrison;
mod illusions;
mod mirroring;
mod state;

use sc_sim::testing::{ScenarioBuilder, TestSim, dangling_references};
use sc_sim::{
    Ability, Command, Faction, ObjectId, ObjectKind, OrderKind, PlayerCommand, PlayerId, Reject, SimEvent, SpawnAt,
    Tick, TilePos, Vec2,
};

const P1: PlayerId = PlayerId(1);
const P2: PlayerId = PlayerId(2);
const DISCIPLE: ObjectKind = ObjectKind::Disciple;
const MILITANT: ObjectKind = ObjectKind::Militant;
const RECRUIT: ObjectKind = ObjectKind::Recruit;
/// A grid unit, su.
const GU: f32 = 64.0;
/// Where most tests' Disciple stands: the centre of tile (10, 10).
const AT: Vec2 = Vec2::new(672.0, 672.0);
/// `factions/cults/units/disciple.md > Placeholder stats > Mirage numbers`: "Cost: **50 energy**".
const COST: f32 = 50.0;
/// The Disciple's regeneration in a tick, points: `> Placeholder stats`' "regenerates **0.5 energy per second**",
/// 2,048 units of `sc_sim::energy::ONE`.
const A_TICK: f32 = 1.0 / 32.0;

fn point(x: f32, y: f32) -> SpawnAt {
    SpawnAt::Point(Vec2::new(x, y))
}

fn tile(x: i32, y: i32) -> SpawnAt {
    SpawnAt::Tile(TilePos::new(x, y))
}

/// A flat `w` × `h` map, both players the Cults'.
fn flat(w: usize, h: usize) -> ScenarioBuilder {
    cults(&vec![".".repeat(w); h].join("\n"))
}

/// The ASCII map `map`, both players the Cults'.
fn cults(map: &str) -> ScenarioBuilder {
    ScenarioBuilder::new(map)
        .faction(P1, Faction::Cults)
        .faction(P2, Faction::Cults)
}

/// Player 1's Disciple (1) at [`AT`] on a flat 20 × 20 map, auto-targeting off.
fn lone() -> TestSim {
    quiet(flat(20, 20).spawn(DISCIPLE, Some(P1), point(AT.x, AT.y)).start())
}

/// `t` with auto-targeting off, so that only the orders fight.
fn quiet(mut t: TestSim) -> TestSim {
    t.without_auto_targeting();
    t
}

fn ids(units: &[u32]) -> Vec<ObjectId> {
    units.iter().copied().map(ObjectId).collect()
}

/// `player`'s `units` ordered `kind` in tick `tick`, not queued.
fn order(t: &mut TestSim, tick: Tick, player: PlayerId, units: &[u32], kind: OrderKind) {
    t.order_at(tick, player, &ids(units), kind, false);
}

/// Player 1's `units` cast Mirage in tick `tick` (F).
fn cast(t: &mut TestSim, tick: Tick, units: &[u32]) {
    t.ability_at(tick, P1, &ids(units), Ability::Mirage);
}

/// The Mirage command for player 1's `units`.
fn mirage(units: &[u32]) -> Command {
    Command::Ability {
        units: ids(units),
        ability: Ability::Mirage,
    }
}

/// The Shroud order for `units` (R).
fn shroud(units: &[u32]) -> Command {
    Command::Order {
        units: ids(units),
        kind: OrderKind::Shroud,
        queue: false,
    }
}

fn validate(t: &TestSim, player: PlayerId, command: Command) -> Result<(), Reject> {
    t.sim().validate(&PlayerCommand { player, command })
}

/// The illusions of `parent` (`Object::illusion_of`) on the map, in id order.
fn illusions(t: &TestSim, parent: u32) -> Vec<u32> {
    (t.sim().objects())
        .filter(|o| o.illusion_of == Some(ObjectId(parent)))
        .map(|o| o.id.0)
        .collect()
}

/// The illusions of `parent` held in a building, in id order.
fn held_illusions(t: &TestSim, parent: u32) -> Vec<u32> {
    (t.sim().held())
        .filter(|held| held.object.illusion_of == Some(ObjectId(parent)))
        .map(|held| held.object.id.0)
        .collect()
}

/// Object `id`'s own energy, points.
fn energy(t: &TestSim, id: u32) -> f32 {
    let o = t.sim().object(ObjectId(id)).expect("on the map");
    sc_sim::energy::of(o).expect("a pool").0
}

/// The energy object `id` shows, points, and its pool's maximum (`Sim::energy_shown`).
fn shown(t: &TestSim, id: u32) -> Option<(f32, f32)> {
    t.sim().energy_shown(t.sim().object(ObjectId(id))?)
}

/// Object `id`'s HP.
fn hp(t: &TestSim, id: u32) -> f32 {
    t.sim().object(ObjectId(id)).and_then(|o| o.hp).expect("alive")
}

/// Whether unit `id` stands channelling: its current order a Shroud (`sc_sim::shroud::stands_channelling`).
fn stands(t: &TestSim, id: u32) -> bool {
    t.sim()
        .object(ObjectId(id))
        .is_some_and(sc_sim::shroud::stands_channelling)
}

/// The clouds, each as its members' ids.
fn clouds(t: &TestSim) -> Vec<Vec<u32>> {
    let members = |c: &sc_sim::shroud::Cloud| c.members.iter().map(|m| m.id.0).collect();
    t.sim().clouds().list().iter().map(members).collect()
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

/// The objects that left the game so far, as `(tick, id)`.
fn removed(t: &TestSim) -> Vec<(Tick, u32)> {
    (t.events().iter())
        .filter_map(|(tick, e)| match e {
            SimEvent::Removed(id) => Some((*tick, id.0)),
            _ => None,
        })
        .collect()
}

/// The objects destroyed so far, as `(tick, id)`.
fn died(t: &TestSim) -> Vec<(Tick, u32)> {
    (t.events().iter())
        .filter_map(|(tick, e)| match e {
            SimEvent::Died { id, .. } => Some((*tick, id.0)),
            _ => None,
        })
        .collect()
}

/// The Mirages cast so far, as `(tick, disciple, illusions)`.
fn mirages(t: &TestSim) -> Vec<(Tick, u32, Vec<u32>)> {
    (t.events().iter())
        .filter_map(|(tick, e)| match e {
            SimEvent::Mirage { disciple, illusions } => {
                Some((*tick, disciple.0, illusions.iter().map(|id| id.0).collect()))
            }
            _ => None,
        })
        .collect()
}

/// One tick, then: nothing names an object that left the game.
fn step_checked(t: &mut TestSim) {
    t.step();
    assert_eq!(
        dangling_references(t.sim()),
        [],
        "tick {}\n{}",
        t.sim().tick(),
        t.ascii(None)
    );
}

/// Steps `t` until `done` holds, at most `ticks` ticks; panics if it never does.
fn step_until(t: &mut TestSim, ticks: u32, what: &str, done: impl Fn(&TestSim) -> bool) {
    for _ in 0..ticks {
        if done(t) {
            return;
        }
        step_checked(t);
    }
    assert!(done(t), "{what} within {ticks} ticks");
}

/// Runs until Disciple `id` has `points` of energy: 32 ticks a point from its regeneration alone.
fn regenerate(t: &mut TestSim, id: u32, points: f32) {
    let ticks = ((points - energy(t, id)).max(0.0) * 32.0) as u32 + 2;
    step_until(t, ticks, "the energy back", |t| energy(t, id) >= points);
}
