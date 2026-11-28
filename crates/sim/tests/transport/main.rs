//! The Subterranean Transport above ground (slice M6-ST1, plan decision T94;
//! `factions/syndicate/units/subterranean_transport.md`: design change 0367's placeholders, 0419, 0432 answering
//! Q0166), through the gate, one test binary:
//! - `production`: the Vehicle Assembly trains it behind a Forge, on its R, into the Tunnel Network;
//! - `boarding`: its owner's infantry board it by an Enter, four Guards and not a fifth, as it drives too; no unit of
//!   another player's, no vehicle, and none while it is burrowed;
//! - `unloading`: Unload All puts its passengers round it, on a ring, none overlapping, once it is at rest, and Shift
//!   queues it after a move; refused with no one aboard;
//! - `death`: its passengers die with it, on the map or held in the network, and their Tunnel Space is free;
//! - `network`: it drives into a Tier 2 Tunnel loaded, the EjectMenu lists it alone, Tunnel Space counts its passengers
//!   throughout, and it comes out loaded; it regenerates and takes its Null Chamber turn, and its passengers don't
//!   (design change 0432 part 2);
//! - `combat`: a ground unit: no shots from its passengers (0432 part 1), a Peacekeeper's target and not a Twister's
//!   (`Reject::CannotHit`), colliding with ground units, crushing enemy Recruits and stopped by its own infantry
//!   (0419);
//! - `saves`: a game with passengers aboard, on the map and in the network, goes on alike after a save and in a replay.
//!
//! Every test runs under the invariants of `TestSim`, the carrier invariant U1 among them.

mod boarding;
mod combat;
mod death;
mod network;
mod production;
mod saves;
mod unloading;

use sc_sim::buffs::BuffKind;
use sc_sim::testing::{ScenarioBuilder, TestSim};
use sc_sim::{
    Command, DebugCommand, Faction, Object, ObjectId, ObjectKind, OrderKind, PlayerCommand, PlayerId, Reject, SimEvent,
    SpawnAt, Target, Tick, TilePos, TunnelCommand, Vec2,
};

const P1: PlayerId = PlayerId(1);
const P2: PlayerId = PlayerId(2);
const TRANSPORT: ObjectKind = ObjectKind::SubterraneanTransport;
const BURROWED: ObjectKind = ObjectKind::SubterraneanTransportBurrowed;
const GUARD: ObjectKind = ObjectKind::Guard;
const TUNNEL: ObjectKind = ObjectKind::Tunnel;

/// `factions/syndicate/units/subterranean_transport.md > Placeholder stats`: "**Capacity 4** infantry"; "Silhouette
/// 72x48 space units", half of it along its facing and across it; "MaxHP 300".
const CAPACITY: usize = 4;
const HALF: Vec2 = Vec2::new(36.0, 24.0);
const MAX_HP: f32 = 300.0;
/// A Guard's MaxHP (`factions/syndicate/units/guard.md`: "MaxHP - 80"), and half its 36 × 36 su Silhouette.
const GUARD_HP: f32 = 80.0;
const GUARD_HALF: f32 = 18.0;
/// The Tunnel Space each takes (`> Placeholder stats`: "TunnelSpaceCost 3"; `factions/syndicate/units/guard.md`:
/// "TunnelSpaceCost - 1").
const TRANSPORT_SPACE: u32 = 3;
const GUARD_SPACE: u32 = 1;

fn flat(w: usize, h: usize) -> String {
    vec![".".repeat(w); h].join("\n")
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

/// Player 1 plays the Syndicate and player 2 the GDO on a flat 40 × 24 map.
fn field() -> ScenarioBuilder {
    ScenarioBuilder::new(&flat(40, 24))
        .faction(P1, Faction::Syndicate)
        .faction(P2, Faction::Gdo)
}

/// `scenario` started with auto-targeting off, tick 1 run.
fn started(scenario: ScenarioBuilder) -> TestSim {
    let mut t = scenario.start();
    t.without_auto_targeting();
    t.run_until(1);
    t
}

/// [`field`] with player 1's Transport (1) at (600, 600), facing east, and `guards` of player 1's Guards in a column
/// west of it, the k-th (from 0) at (400, 520 + 60 k), ids 2 on: started ([`started`]).
fn transport_and_guards(guards: u32) -> TestSim {
    let mut scenario = field().spawn(TRANSPORT, Some(P1), point(600.0, 600.0));
    for k in 0..guards {
        scenario = scenario.spawn(GUARD, Some(P1), point(400.0, 520.0 + 60.0 * k as f32));
    }
    started(scenario)
}

fn order(units: &[ObjectId], kind: OrderKind) -> Command {
    Command::Order {
        units: units.to_vec(),
        kind,
        queue: false,
    }
}

fn queued(units: &[ObjectId], kind: OrderKind) -> Command {
    Command::Order {
        units: units.to_vec(),
        kind,
        queue: true,
    }
}

fn move_to(x: f32, y: f32) -> OrderKind {
    OrderKind::Move(Target::Point(Vec2::new(x, y)))
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

/// The passengers aboard `carrier`, longest aboard first (`Sim::passengers`).
fn aboard(t: &TestSim, carrier: ObjectId) -> Vec<ObjectId> {
    t.sim().passengers(carrier).iter().map(|held| held.object.id).collect()
}

/// Player `player`'s Tunnel Space in use (`Sim::army_cap`).
fn space_used(t: &TestSim, player: PlayerId) -> u32 {
    t.sim().army_cap(player).expect("a player").used
}

/// The units that boarded `carrier` so far, in order, with their ticks.
fn boarded(t: &TestSim, carrier: ObjectId) -> Vec<(Tick, ObjectId)> {
    (t.events().iter())
        .filter_map(|(tick, e)| match *e {
            SimEvent::Boarded { unit, carrier: on } if on == carrier => Some((*tick, unit)),
            _ => None,
        })
        .collect()
}

/// The units that came out of `carrier` so far, in order, with their ticks.
fn unloaded(t: &TestSim, carrier: ObjectId) -> Vec<(Tick, ObjectId)> {
    (t.events().iter())
        .filter_map(|(tick, e)| match *e {
            SimEvent::Unloaded { unit, carrier: from } if from == carrier => Some((*tick, unit)),
            _ => None,
        })
        .collect()
}

/// The units that died so far, with their ticks and where.
fn deaths(t: &TestSim) -> Vec<(Tick, ObjectId, Vec2)> {
    (t.events().iter())
        .filter_map(|(tick, e)| match *e {
            SimEvent::Died { id, pos, .. } => Some((*tick, id, pos)),
            _ => None,
        })
        .collect()
}

/// `units` sent onto `carrier` (an Enter) in the next tick, then run until all are aboard: the tick the last boarded.
fn board(t: &mut TestSim, units: &[ObjectId], carrier: ObjectId) -> Tick {
    let tick = t.sim().tick() + 1;
    t.at(tick, P1, order(units, OrderKind::Enter(carrier)));
    step_until(t, 300, "they board", |t| {
        units.iter().all(|&unit| aboard(t, carrier).contains(&unit))
    });
    t.sim().tick()
}

/// Whether unit `id`, on the map, has no order.
fn is_idle(t: &TestSim, id: ObjectId) -> bool {
    object(t, id).orders.as_ref().is_some_and(|orders| orders.is_idle())
}

/// Steps until `done`, at most `ticks` ticks; panics naming `what` otherwise.
fn step_until(t: &mut TestSim, ticks: Tick, what: &str, done: impl Fn(&TestSim) -> bool) {
    let end = t.sim().tick() + ticks;
    while !done(t) {
        assert!(t.sim().tick() < end, "{what}: not by tick {end}\n{}", t.ascii(None));
        t.step();
    }
}

fn spawn(kind: ObjectKind, owner: PlayerId, at: SpawnAt) -> DebugCommand {
    DebugCommand::Spawn {
        kind,
        owner: Some(owner),
        at,
        orientation: Default::default(),
    }
}

/// The id the next object will have, with no spawn scheduled yet, if the unit with the highest id so far is still in
/// the game (`tests/tunnel_network`'s).
fn next_id(t: &TestSim) -> u32 {
    let on_map = t.sim().objects().map(|o| o.id.0);
    let held = t.sim().held().map(|held| held.object.id.0);
    on_map.chain(held).max().unwrap_or(0) + 1
}

/// Player 1's Tunnel in [`network`].
const T1: ObjectId = ObjectId(1);
/// Where [`network`]'s Null Chamber stands when it has one, in the Tunnel's Tier 1 area, west of Side A: (10, 8) to
/// (11, 9).
const CHAMBER_AT: TilePos = TilePos::new(10, 8);
/// The Space Crystals player 1 has from tick 1 in [`network`].
const BANK: u32 = 10_000;

/// [`field`] with player 1's Tunnel ([`T1`]), unturned, on tiles (12, 10) to (15, 13), x 768..1024 and y 640..896
/// (Side A faces north, its entry path's spot at (866, 640) and its exit path's at (926, 640)), its `structures`,
/// finished, from id 2 in their order, then player 1's Transport at (600, 300), facing east, and `guards` of player
/// 1's Guards in a column west of it, the k-th (from 0) at (400, 220 + 60 k). Started with auto-targeting off,
/// [`BANK`] Space Crystals for player 1 and the Tunnel set to `tier` in tick 1 (a debug command stands for its
/// upgrades, slice M4-G's), which has run: the Transport's id and the Guards'.
fn network(structures: &[(ObjectKind, TilePos)], guards: u32, tier: u8) -> (TestSim, ObjectId, Vec<ObjectId>) {
    let mut scenario = field().spawn(TUNNEL, Some(P1), tile(12, 10));
    for &(kind, at) in structures {
        scenario = scenario.spawn(kind, Some(P1), SpawnAt::Tile(at));
    }
    scenario = scenario.spawn(TRANSPORT, Some(P1), point(600.0, 300.0));
    for k in 0..guards {
        scenario = scenario.spawn(GUARD, Some(P1), point(400.0, 220.0 + 60.0 * k as f32));
    }
    let mut t = scenario.start();
    t.without_auto_targeting();
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
    t.run_until(1);
    let transport = id(2 + structures.len() as u32);
    let guards = (transport.0 + 1..transport.0 + 1 + guards).map(id).collect();
    (t, transport, guards)
}

fn eject(tunnel: ObjectId, kind: ObjectKind) -> Command {
    Command::Tunnel {
        tunnel,
        command: TunnelCommand::Eject { kind },
    }
}

/// The ids of player 1's units inside the network, in id order: the EjectMenu's (`Sim::network`).
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

/// `carrier` sent into Tunnel `tunnel` (an Enter) in the next tick, then run until it is inside: the tick it went in.
fn drive_in(t: &mut TestSim, carrier: ObjectId, tunnel: ObjectId) -> Tick {
    let tick = t.sim().tick() + 1;
    t.at(tick, P1, order(&[carrier], OrderKind::Enter(tunnel)));
    step_until(t, 400, "it drives in", |t| inside(t).contains(&carrier));
    entered(t, carrier)
}

/// `kind` ejected from Tunnel `tunnel` in the next tick, then run until `unit` is out: the tick it came out.
fn eject_out(t: &mut TestSim, tunnel: ObjectId, kind: ObjectKind, unit: ObjectId) -> Tick {
    let tick = t.sim().tick() + 1;
    t.at(tick, P1, eject(tunnel, kind));
    step_until(t, 120, "it comes out", |t| left(t, unit).is_some());
    left(t, unit).unwrap()
}

/// Whether unit `id`, on the map or held, holds the Null Chamber's exit buff.
fn buffed(t: &TestSim, id: ObjectId) -> bool {
    (unit(t, id).buffs.iter()).any(|buff| buff.ability == BuffKind::NullChamber)
}

/// Unit `id`'s HP, on the map or held.
fn hp(t: &TestSim, id: ObjectId) -> f32 {
    unit(t, id).hp.expect("a unit's HP")
}

/// Unit `target` wounded by a Peacekeeper of player 2's: one debug-spawned 150 su west of it and given an Attack on it
/// in the next tick, removed in the tick after the target's HP first falls below its MaxHP. Its HP then.
fn wound(t: &mut TestSim, target: ObjectId) -> f32 {
    let max = hp(t, target);
    let at = object(t, target).pos - Vec2::new(150.0, 0.0);
    let tick = t.sim().tick() + 1;
    t.debug_at(tick, spawn(ObjectKind::Peacekeeper, P2, SpawnAt::Point(at)));
    t.run_until(tick);
    let pk = (t.sim().objects())
        .find(|o| o.kind == ObjectKind::Peacekeeper && o.pos == at)
        .expect("the Peacekeeper spawned")
        .id;
    t.at(tick + 1, P2, order(&[pk], OrderKind::Attack(target)));
    step_until(t, 120, "it is wounded", |t| hp(t, target) < max);
    let tick = t.sim().tick() + 1;
    t.debug_at(tick, DebugCommand::Remove { id: pk });
    t.run_until(tick);
    assert!(t.sim().object(pk).is_none());
    hp(t, target)
}
