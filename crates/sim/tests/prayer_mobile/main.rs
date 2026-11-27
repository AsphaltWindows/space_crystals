//! The Prayer-mobile (slice M5-N11; `factions/cults/units/prayer_mobile.md`, design changes 0364, 0365 and 0444): the
//! Cults' caster vehicle, trained at the Chop Shop, whose energy pays for two abilities cast at once, Inspire round it
//! and Suppress ahead of it (`Command::Cast`, `sc_sim::buffs`, `sc_sim::suppress`), played through the gate, one test
//! binary:
//! - `energy`: it starts at 50 of its 200 and regenerates 0.5 a second: neither ability at first, Inspire's 75 after
//!   50 s, Suppress's 100 after 100 s;
//! - `inspire`: design change 0365's Verification, "Inspire next to Recruits: they move and punch faster for 10 s"; a
//!   Militant gets the speed and not the attack, the vehicle and another player's Recruit nothing; a recast restarts
//!   the 10 s; the speed adds to Improved Metabolism's;
//! - `suppress`: design change 0365's Verification, "Suppress facing a group of Peacekeepers and your own Militants:
//!   neither side fires for 4 s"; a Sparrow in the arc is suppressed and a building isn't, nor a unit just outside the
//!   90° or the 5 grid units; a Rocketman suppressed while it locks keeps its lock and fires its first missile once the
//!   4 s are over; a suppressed Disciple goes on channelling Shroud;
//! - `production`: the Chop Shop's E is refused without a Temple; with one, two Recruits come out a Prayer-mobile 400
//!   ticks later, with its pool;
//! - `saves`: a game with both abilities under way goes on alike after a save and in a replay, the buffs saved only
//!   while a unit has one.
//!
//! Every test runs under the invariants of `TestSim`, the buff invariant F1 among them (`testing/invariants/buffs.rs`).

mod energy;
mod inspire;
mod production;
mod saves;
mod suppress;

use sc_sim::buffs::Buff;
use sc_sim::testing::{ScenarioBuilder, TestSim};
use sc_sim::{
    AreaAbility, Command, Faction, ObjectId, ObjectKind, PlayerCommand, PlayerId, Reject, SimEvent, Tick, Vec2,
};

const P1: PlayerId = PlayerId(1);
const P2: PlayerId = PlayerId(2);
const PRAYER_MOBILE: ObjectKind = ObjectKind::PrayerMobile;
const RECRUIT: ObjectKind = ObjectKind::Recruit;
const MILITANT: ObjectKind = ObjectKind::Militant;

const INSPIRE: AreaAbility = AreaAbility::Inspire;
const SUPPRESS: AreaAbility = AreaAbility::Suppress;

/// A point of energy in `Object::energy`'s units.
const ONE: u32 = sc_sim::energy::ONE;
/// `> Placeholder stats`: Inspire's 10 s and Suppress's 4 s, in ticks.
const INSPIRE_TICKS: Tick = 160;
const SUPPRESS_TICKS: Tick = 64;
/// The vehicle's 0.5 energy a second, in a tick: 1/32 of a point.
const A_TICK: u32 = ONE / 32;

fn flat(w: usize, h: usize) -> String {
    vec![".".repeat(w); h].join("\n")
}

/// Player 1 plays the Cults, player 2 `faction`.
fn versus(faction: Faction, w: usize, h: usize) -> ScenarioBuilder {
    ScenarioBuilder::new(&flat(w, h))
        .faction(P1, Faction::Cults)
        .faction(P2, faction)
}

fn id(n: u32) -> ObjectId {
    ObjectId(n)
}

/// The vehicle `caster`'s R or F (`Command::Cast`): `ability` from it, at the point it stands on, which a cast round or
/// ahead of it doesn't use (the client sends that point, `sim/casts.rs`).
fn cast(t: &TestSim, caster: ObjectId, ability: AreaAbility) -> Command {
    let at = t.sim().object(caster).expect("on the map").pos;
    Command::Cast {
        casters: vec![caster],
        ability,
        at,
    }
}

/// Casts `ability` from player 1's `caster` in the next tick, which it runs: that tick.
fn cast_now(t: &mut TestSim, caster: ObjectId, ability: AreaAbility) -> Tick {
    let tick = t.sim().tick() + 1;
    let command = cast(t, caster, ability);
    assert_eq!(validate(t, command.clone()), Ok(()), "{ability:?} at {tick}");
    t.at(tick, P1, command);
    t.run_until(tick);
    tick
}

/// Whether the gate takes `command` from player 1 now.
fn validate(t: &TestSim, command: Command) -> Result<(), Reject> {
    t.sim().validate(&PlayerCommand { player: P1, command })
}

/// Object `id`'s energy, in `Object::energy`'s units.
fn energy(t: &TestSim, id: ObjectId) -> u32 {
    (t.sim().object(id))
        .and_then(|o| o.energy)
        .unwrap_or_else(|| panic!("no pool for {id:?}"))
}

/// Runs until `caster` has `points` of energy, within the 300 s from none to its MaxEnergy.
fn charge(t: &mut TestSim, caster: ObjectId, points: u32) {
    let start = t.sim().tick();
    while energy(t, caster) < points * ONE {
        assert!(t.sim().tick() < start + 4800, "charged to {}", energy(t, caster) / ONE);
        t.step();
    }
}

/// Unit `id`'s buffs.
fn buffs(t: &TestSim, id: ObjectId) -> Vec<Buff> {
    t.sim()
        .object(id)
        .unwrap_or_else(|| panic!("no unit {id:?}"))
        .buffs
        .clone()
}

/// The buff of `ability` cast in tick `cast`, alone.
fn buff(ability: AreaAbility, cast: Tick, ticks: Tick) -> Vec<Buff> {
    vec![Buff {
        ability: ability.into(),
        until: cast + ticks,
    }]
}

/// Unit `id`'s top speed now, its Augmentations' and its buffs' included (`Object::max_speed`).
fn top_speed(t: &TestSim, id: ObjectId) -> f32 {
    let o = t.sim().object(id).unwrap_or_else(|| panic!("no unit {id:?}"));
    o.max_speed(t.sim().research())
}

/// The id the next object will have, with no spawn scheduled yet and none removed.
fn next_id(t: &TestSim) -> u32 {
    let on_map = t.sim().objects().map(|o| o.id.0);
    let held = t.sim().held().map(|held| held.object.id.0);
    on_map.chain(held).max().unwrap_or(0) + 1
}

/// A unit of `owner`'s of type `kind` at `at`, spawned in the next tick, which it runs: its id.
fn spawn(t: &mut TestSim, kind: ObjectKind, owner: PlayerId, at: Vec2) -> ObjectId {
    let tick = t.sim().tick() + 1;
    let spawned = id(next_id(t));
    t.spawn_unit_at(tick, kind, owner, at);
    t.run_until(tick);
    assert_eq!(t.sim().object(spawned).map(|o| o.kind), Some(kind), "{kind:?} at {at}");
    spawned
}

/// The ticks in which `attacker`'s single-target attacks took effect so far (`SimEvent::Attacked`: the end of a
/// Firing).
fn attacks_by(t: &TestSim, attacker: ObjectId) -> Vec<Tick> {
    (t.events().iter())
        .filter_map(|(tick, e)| match *e {
            SimEvent::Attacked { attacker: by, .. } if by == attacker => Some(*tick),
            _ => None,
        })
        .collect()
}

/// The ticks of `attacker`'s hits on `target` so far.
fn hits(t: &TestSim, attacker: ObjectId, target: ObjectId) -> Vec<Tick> {
    (t.events().iter())
        .filter_map(|(tick, e)| match *e {
            SimEvent::Hit {
                attacker: by,
                target: on,
                ..
            } if (by, on) == (attacker, target) => Some(*tick),
            _ => None,
        })
        .collect()
}
