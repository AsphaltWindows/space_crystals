//! The Shadebringer's Sludge and the Vault (slice M6-SB, plan decisions T93 and T98;
//! `factions/syndicate/units/shadebringer.md`, `factions/syndicate/structures/vault.md`: design change 0367's
//! placeholders, 0369, 0424 answering Q0158 and 0425 answering Q0159), through the gate, one test binary:
//! - `building`: a Tier 1 Tunnel builds the Vault behind a Hideout, which serves no one before it is finished, and the
//!   Vehicle Assembly trains the Shadebringer behind a Vault, on its W;
//! - `toggle`: R switches the Sludge on with any charge left, drains its tank 10 a second, parked or not, and switches
//!   it off once the tank is empty; refused with none; the unit's orders kept;
//! - `trail`: the patches behind it, 48 su wide, each lasting 10 s, at its front while it backs up;
//! - `slow`: the units whose centres are in it, slowed by 40% while in it and for 2 s after, a Shadebringer backing up
//!   over its own trail and one braking across another's; hover units and aircraft not, nor a unit whose box reaches
//!   into it but whose centre stays out;
//! - `vault`: into the network the toggles go off; the Vault refills the tanks 5 a second, 2 units at a time, first
//!   come first served, while the Null Chamber serves them too (0424), and the EjectMenu takes the unit inside longest;
//! - `saves`: a game with a trail, slowed units and tanks filling goes on alike after a save and in a replay, and the
//!   patches and the tanks are in the hash and the save only while there are any.
//!
//! Every test runs under the invariants of `TestSim`, the tank invariant N1 (slice M6-SB) and the buff invariant F1
//! among them.

mod building;
mod saves;
mod slow;
mod toggle;
mod trail;
mod vault;

use sc_sim::buffs::BuffKind;
use sc_sim::command::Ability;
use sc_sim::defs::PatchKind;
use sc_sim::patches::{Patch, Tank};
use sc_sim::testing::{ScenarioBuilder, TestSim};
use sc_sim::{
    BuildingOrientation, Command, DebugCommand, Faction, Object, ObjectId, ObjectKind, OrderKind, PlayerCommand,
    PlayerId, ProductionCommand, Reject, SimEvent, SpawnAt, Target, Tick, TilePos, TunnelCommand, Vec2,
};

const P1: PlayerId = PlayerId(1);
const P2: PlayerId = PlayerId(2);
const SHADEBRINGER: ObjectKind = ObjectKind::Shadebringer;
const TUNNEL: ObjectKind = ObjectKind::Tunnel;
const VAULT: ObjectKind = ObjectKind::Vault;

/// `factions/syndicate/units/shadebringer.md > Placeholder stats`, in fixed point (`sc_sim::patches`, 65 536 to a unit
/// of charge): "**Tanks**: 100 each; drain **10 per second** while on (10 s of use from full); refill **5 per
/// second** inside the network with a Vault", a tick's of each.
const CAPACITY: u32 = 100 << 16;
const DRAIN: u32 = (10 << 16) / 16;
const REFILL: u32 = (5 << 16) / 16;
/// The ticks a full tank lasts on, and fills from empty: 10 s and 20 s.
const FULL_USE: Tick = 160;
const FULL_FILL: Tick = 320;
/// "each patch lasts **X = 10 s**", ticks; "trail width 48 su", the radius of its discs; "slowed by **Y = 40%** for
/// **Z = 2 s** after leaving it", the share of speed left and the ticks after.
const LIFE: Tick = 160;
const RADIUS: f32 = 24.0;
const SLOWED: f32 = 0.6;
const AFTER: Tick = 32;
/// "Silhouette 56x40 space units", along its facing: its rear edge, where it lays its trail, half its length behind its
/// centre.
const HALF_LENGTH: f32 = 28.0;
/// "ForwardMaxSpeed 8 su/frame", "ReverseMaxSpeed 3", "Deceleration 1".
const TOP: f32 = 8.0;
const REVERSE_TOP: f32 = 3.0;

fn flat(w: usize, h: usize) -> String {
    vec![".".repeat(w); h].join("\n")
}

fn tile(x: i32, y: i32) -> SpawnAt {
    SpawnAt::Tile(TilePos::new(x, y))
}

fn point(x: f32, y: f32) -> SpawnAt {
    SpawnAt::Point(Vec2::new(x, y))
}

fn id(n: u32) -> ObjectId {
    ObjectId(n)
}

/// Player 1 plays the Syndicate and player 2 the GDO, on a flat 64 × 24 map (x 0..4096, y 0..1536), with no
/// structure: units on open ground, facing east (+x) as they come.
fn field() -> ScenarioBuilder {
    field_with(Faction::Gdo)
}

/// [`field`] with player 2 playing `faction`.
fn field_with(faction: Faction) -> ScenarioBuilder {
    ScenarioBuilder::new(&flat(64, 24))
        .faction(P1, Faction::Syndicate)
        .faction(P2, faction)
}

/// `scenario` started with auto-targeting off.
fn started(scenario: ScenarioBuilder) -> TestSim {
    let mut t = scenario.start();
    t.without_auto_targeting();
    t
}

/// The Sludge's toggle switched on or off (R).
fn sludge(on: bool) -> Ability {
    Ability::Sludge { on }
}

fn press(units: &[ObjectId], ability: Ability) -> Command {
    Command::Ability {
        units: units.to_vec(),
        ability,
    }
}

fn order(units: &[ObjectId], kind: OrderKind) -> Command {
    Command::Order {
        units: units.to_vec(),
        kind,
        queue: false,
    }
}

fn move_to(x: f32, y: f32) -> OrderKind {
    OrderKind::Move(Target::Point(Vec2::new(x, y)))
}

fn eject(tunnel: ObjectId, kind: ObjectKind) -> Command {
    Command::Tunnel {
        tunnel,
        command: TunnelCommand::Eject { kind },
    }
}

fn spawn(kind: ObjectKind, owner: PlayerId, at: SpawnAt) -> DebugCommand {
    DebugCommand::Spawn {
        kind,
        owner: Some(owner),
        at,
        orientation: BuildingOrientation::default(),
    }
}

/// What the gate says to `player`'s `command` now (`Sim::validate`).
fn gate(t: &TestSim, player: PlayerId, command: Command) -> Result<(), Reject> {
    t.sim().validate(&PlayerCommand { player, command })
}

fn object(t: &TestSim, id: ObjectId) -> &Object {
    t.sim().object(id).unwrap_or_else(|| panic!("{id:?} is on the map"))
}

/// Unit `id`, on the map or held.
fn unit(t: &TestSim, id: ObjectId) -> &Object {
    let sim = t.sim();
    (sim.object(id))
        .or_else(|| sim.held_unit(id).map(|held| &held.object))
        .unwrap_or_else(|| panic!("{id:?} is in the game"))
}

/// Unit `id`'s Sludge's tank, on the map or held.
fn tank(t: &TestSim, id: ObjectId) -> Tank {
    unit(t, id).tanks.expect("a Shadebringer's tanks").sludge
}

/// How far unit `id` moved in the last tick, su.
fn speed(t: &TestSim, id: ObjectId) -> f32 {
    let o = object(t, id);
    o.pos.distance(o.prev_pos)
}

/// The tick unit `id`'s slow ends by (`Buff::until`, `BuffKind::Sludge`), if it holds it.
fn slow_until(t: &TestSim, id: ObjectId) -> Option<Tick> {
    (unit(t, id).buffs.iter())
        .find(|buff| buff.ability == BuffKind::Sludge)
        .map(|buff| buff.until)
}

/// Whether `at` is in a sludge patch now.
fn in_sludge(t: &TestSim, at: Vec2) -> bool {
    t.sim().patches().covers(PatchKind::Sludge, at)
}

/// The patches on the ground that unit `id` laid.
fn laid_by(t: &TestSim, id: ObjectId) -> Vec<Patch> {
    t.sim().patches().iter().filter(|p| p.by == id).copied().collect()
}

/// The ids of player 1's units inside the network, in id order.
fn inside(t: &TestSim) -> Vec<ObjectId> {
    t.sim().network(P1).map(|held| held.object.id).collect()
}

/// The tick in which unit `id` last went into the network.
fn entered(t: &TestSim, id: ObjectId) -> Tick {
    (t.events().iter().rev())
        .find_map(|(tick, e)| matches!(*e, SimEvent::EnteredNetwork { unit, .. } if unit == id).then_some(*tick))
        .unwrap_or_else(|| panic!("{id:?} went in"))
}

/// The tick in which unit `id` last came out of the network, if it did.
fn left(t: &TestSim, id: ObjectId) -> Option<Tick> {
    (t.events().iter().rev())
        .find_map(|(tick, e)| matches!(*e, SimEvent::LeftNetwork { unit, .. } if unit == id).then_some(*tick))
}

/// Player 1's Tunnel in [`network`], on tiles (12, 10) to (15, 13), unturned, at Tier 2, which lets vehicles through:
/// Side A faces north, its entry path's spot at (866, 640), its exit path's at (926, 640).
const T1: ObjectId = ObjectId(1);
/// Where [`network`]'s Vault stands, in the Tunnel's Tier 1 area, east of Side A: (16, 8) to (17, 9).
const VAULT_AT: TilePos = TilePos::new(16, 8);
/// Where [`network`]'s Null Chamber stands, in the Tunnel's Tier 1 area, west of Side A: (10, 8) to (11, 9).
const CHAMBER_AT: TilePos = TilePos::new(10, 8);

/// The Space Crystals player 1 has from tick 1 in [`network`].
const BANK: u32 = 10_000;

/// [`field`] with player 1's Tunnel ([`T1`]), its `structures`, finished, from id 2 in their order, and `count`
/// Shadebringers north of the Tunnel, at (500 + 300 k, 200) for the k-th (from 0), started with [`BANK`] Space
/// Crystals and the Tunnel set to Tier 2 in tick 1 (a debug command stands for its upgrades), which has run: the
/// Shadebringers' ids.
fn network(structures: &[(ObjectKind, TilePos)], count: u32) -> (TestSim, Vec<ObjectId>) {
    network_at(2, structures, count)
}

/// [`network`] with the Tunnel at `tier`.
fn network_at(tier: u8, structures: &[(ObjectKind, TilePos)], count: u32) -> (TestSim, Vec<ObjectId>) {
    let mut scenario = field().spawn(TUNNEL, Some(P1), tile(12, 10));
    for &(kind, at) in structures {
        scenario = scenario.spawn(kind, Some(P1), SpawnAt::Tile(at));
    }
    for k in 0..count {
        scenario = scenario.spawn(SHADEBRINGER, Some(P1), point(500.0 + 300.0 * k as f32, 200.0));
    }
    let mut t = started(scenario);
    let bank = DebugCommand::SetResources {
        player: P1,
        crystals: BANK,
        supplies: 100,
        alloys: 0,
        essence: 0,
        flux: 0,
    };
    t.debug_at(1, bank);
    if tier > 1 {
        t.debug_at(1, DebugCommand::SetTier { id: T1, tier });
    }
    run_until(&mut t, 1);
    let next = 2 + structures.len() as u32;
    (t, (next..next + count).map(id).collect())
}

/// `units`' Sludge switched on in the next tick, run until their tanks are empty and their toggles off by themselves,
/// 10 s on, where they stand.
fn emptied(t: &mut TestSim, units: &[ObjectId]) {
    let tick = t.sim().tick() + 1;
    t.at(tick, P1, press(units, sludge(true)));
    run_until(t, tick + FULL_USE - 1);
    for &unit in units {
        assert_eq!(
            (tank(t, unit).used, tank(t, unit).on),
            (CAPACITY, false),
            "{unit:?} empty"
        );
    }
}

/// `unit` ordered into [`T1`] in the next tick, and run until it is in: the tick it went in.
fn drive_in(t: &mut TestSim, unit: ObjectId) -> Tick {
    let tick = t.sim().tick() + 1;
    t.at(tick, P1, order(&[unit], OrderKind::Enter(T1)));
    step_until(t, 400, "it drives in", |t| t.sim().held_unit(unit).is_some());
    entered(t, unit)
}

/// `unit` ejected at [`T1`] in the next tick, run until it is out, and sent on east of the exit path so that it
/// leaves the way free (the stop point of design change 0459 is slice M6-S6b's): the tick it came out.
fn eject_out(t: &mut TestSim) -> Tick {
    let tick = t.sim().tick() + 1;
    let before = t.sim().objects().count();
    t.at(tick, P1, eject(T1, SHADEBRINGER));
    step_until(t, 60, "a Shadebringer comes out", |t| {
        t.sim().objects().count() > before
    });
    let out = t.sim().tick();
    let unit = (t.events().iter().rev())
        .find_map(|(_, e)| match *e {
            SimEvent::LeftNetwork { unit, .. } => Some(unit),
            _ => None,
        })
        .expect("out");
    t.at(out + 1, P1, order(&[unit], move_to(1500.0, 300.0)));
    out
}

/// Runs `t` until `tick`.
fn run_until(t: &mut TestSim, tick: Tick) {
    t.run_until(tick);
}

/// Steps until `done`, at most `ticks` ticks; panics naming `what` otherwise.
fn step_until(t: &mut TestSim, ticks: Tick, what: &str, done: impl Fn(&TestSim) -> bool) {
    let end = t.sim().tick() + ticks;
    while !done(t) {
        assert!(t.sim().tick() < end, "{what}: not by tick {end}\n{}", t.ascii(None));
        t.step();
    }
}
