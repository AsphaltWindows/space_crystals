//! The Null Chamber (slice M6-SN, plan decision T98; `factions/syndicate/structures/null_chamber.md > Null Chamber
//! Exit Buff`, `factions/syndicate/structures/null_chamber.md > Placeholder stats`, design changes 0367's and 0369's
//! placeholders, 0424 answering Q0158), through the gate, one test binary:
//! - `building`: a Tier 2 Tunnel builds it behind a Hideout, for 200 Space Crystals in 480 frames, refused a tier lower
//!   and without its prerequisite, and it serves no one before it is finished;
//! - `turns`: the turns, 4 at a time for each chamber, 2 s each, first come first served (the plan's six Guards), the
//!   order the units came in by, a unit trained into the network from its production, two chambers' 8, and (design
//!   change 0463) a turn done buffing its unit with no chamber left;
//! - `buff`: what a unit whose turn is done leaves with, by Side A or (M6-SD) by the air exit door: its sight (design
//!   change 0463: its eye's alone, its AttackMove engaging where an unbuffed one's does) and its hits, its end 20 s
//!   on, its restart, and none without a chamber;
//! - `saves`: a game with units waiting, taking their turns, done and buffed goes on alike after a save and in a
//!   replay, and a held unit's turn is in the hash and the save only while it has one.
//!
//! Every test runs under the invariants of `TestSim`, the buff's F1 among them, and this binary's held-store checks
//! ([`check_held`]) after each tick it steps. The Null Chamber's numbers changed by a SetRules are `tests/rules.rs`'s,
//! as a SetRules changes the process's rules.

mod buff;
mod building;
mod saves;
mod turns;

use sc_sim::buffs::BuffKind;
use sc_sim::defs::TRANSIT_DELAY_TICKS;
use sc_sim::network::Paths;
use sc_sim::testing::{ScenarioBuilder, TestSim, dangling_references};
use sc_sim::{
    BuildingOrientation, Command, DebugCommand, Faction, Object, ObjectId, ObjectKind, OrderKind, PlayerCommand,
    PlayerId, ProductionCommand, Reject, SU_PER_GU, SimEvent, SpawnAt, Tick, TilePos, TunnelCommand, Vec2, seconds,
};

const P1: PlayerId = PlayerId(1);
const P2: PlayerId = PlayerId(2);
const GUARD: ObjectKind = ObjectKind::Guard;
const TUNNEL: ObjectKind = ObjectKind::Tunnel;
const CHAMBER: ObjectKind = ObjectKind::NullChamber;

/// `factions/syndicate/structures/null_chamber.md > Placeholder stats`: "each Null Chamber buffs **4 units at a
/// time**, each buff taking **2 s** inside before the unit can leave buffed", 32 ticks; "duration **20 s**", 320.
const SLOTS: usize = 4;
const TURN: Tick = 32;
const DURATION: Tick = 320;
/// `factions/syndicate/structures/hideout.md`: "Guard: 125 Space Crystals", "120 frames (7.5 seconds)".
const GUARD_FRAMES: Tick = 120;

/// Player 1's Tunnel in [`base`].
const T1: ObjectId = ObjectId(1);
/// Where [`base`]'s Null Chamber stands, in the Tunnel's Tier 1 area, west of Side A: (10, 8) to (11, 9).
const CHAMBER_AT: TilePos = TilePos::new(10, 8);
/// Where a Hideout stands in [`base`]'s Tier 1 area, south-east of the Tunnel: (16, 14) to (17, 15).
const HIDEOUT_AT: TilePos = TilePos::new(16, 14);
/// The Space Crystals player 1 has from tick 1.
const BANK: u32 = 10_000;

/// Where a Guard (36 × 36) stands on [`base`]'s entry path: its spot (866, 640), the box against Side A
/// (`tests/tunnel_network`'s).
const ENTRY: Vec2 = Vec2::new(866.0, 640.0 - 18.0 - 1.0 / 64.0);

fn flat(w: usize, h: usize) -> String {
    vec![".".repeat(w); h].join("\n")
}

fn tile(x: i32, y: i32) -> SpawnAt {
    SpawnAt::Tile(TilePos::new(x, y))
}

fn id(n: u32) -> ObjectId {
    ObjectId(n)
}

/// Player 1 plays the Syndicate and player 2 the GDO, on a flat 64 × 24 map, with player 1's Tunnel ([`T1`]),
/// unturned, on tiles (12, 10) to (15, 13), x 768..1024 and y 640..896: Side A faces north, its entry path's spot at
/// (866, 640) and its exit path's at (926, 640). Its Tier 1 area is (10, 8) to (17, 15), its Tier 2 area (9, 7) to
/// (18, 16).
fn base() -> ScenarioBuilder {
    ScenarioBuilder::new(&flat(64, 24))
        .faction(P1, Faction::Syndicate)
        .faction(P2, Faction::Gdo)
        .spawn(TUNNEL, Some(P1), tile(12, 10))
}

/// [`base`] with player 1's Null Chamber (id 2), finished, at [`CHAMBER_AT`].
fn with_chamber() -> ScenarioBuilder {
    base().spawn(CHAMBER, Some(P1), SpawnAt::Tile(CHAMBER_AT))
}

/// `scenario` started with auto-targeting off, [`BANK`] Space Crystals and 100 Supplies for player 1 and, if `tier` is
/// above 1, the Tunnel set to it (a debug command stands for its upgrades, slice M4-G's), all in tick 1, which has run.
fn started(scenario: ScenarioBuilder, tier: u8) -> TestSim {
    let mut t = scenario.start();
    t.without_auto_targeting();
    t.debug_at(
        1,
        DebugCommand::SetResources {
            player: P1,
            crystals: BANK,
            supplies: 100,
            alloys: 0,
            essence: 0,
            flux: 0,
        },
    );
    if tier > 1 {
        t.debug_at(1, DebugCommand::SetTier { id: T1, tier });
    }
    run_checked(&mut t, 1);
    t
}

/// What the gate says to `player`'s `command` now (`Sim::validate`).
fn gate(t: &TestSim, player: PlayerId, command: Command) -> Result<(), Reject> {
    t.sim().validate(&PlayerCommand { player, command })
}

/// An Expand of `kind` at `origin`, unturned, to `tunnel`.
fn expand(tunnel: ObjectId, kind: ObjectKind, origin: TilePos) -> Command {
    let command = TunnelCommand::Expand {
        kind,
        origin,
        orientation: BuildingOrientation::default(),
    };
    Command::Tunnel { tunnel, command }
}

fn eject(tunnel: ObjectId, kind: ObjectKind) -> Command {
    Command::Tunnel {
        tunnel,
        command: TunnelCommand::Eject { kind },
    }
}

fn order(units: &[ObjectId], kind: OrderKind) -> Command {
    Command::Order {
        units: units.to_vec(),
        kind,
        queue: false,
    }
}

/// A Train of `kind` at `producer`.
fn train(producer: ObjectId, kind: ObjectKind) -> Command {
    Command::Production {
        objects: vec![producer],
        command: ProductionCommand::Train(kind),
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

/// The paths and doors of Tunnel `tunnel`.
fn paths(t: &TestSim, tunnel: ObjectId) -> Paths {
    Paths::of_object(object(t, tunnel)).expect("a Tunnel's paths")
}

/// The ticks done of held unit `id`'s Null Chamber turn (`Held::service`, `network::Service::chamber`): `None` before
/// it began, and for a unit not held.
fn turn(t: &TestSim, id: ObjectId) -> Option<u32> {
    let held = t.sim().held_unit(id)?;
    held.service.chamber.map(|turn| turn.ticks_done())
}

/// The turns of `units` ([`turn`]).
fn turns(t: &TestSim, units: &[ObjectId]) -> Vec<Option<u32>> {
    units.iter().map(|&unit| turn(t, unit)).collect()
}

/// The tick unit `id`'s Null Chamber exit buff ends by (`Buff::until`), if it holds one, on the map or held.
fn buff_until(t: &TestSim, id: ObjectId) -> Option<Tick> {
    let o = unit(t, id);
    (o.buffs.iter())
        .find(|buff| buff.ability == BuffKind::NullChamber)
        .map(|buff| buff.until)
}

/// Which of `units` hold the Null Chamber exit buff.
fn buffed(t: &TestSim, units: &[ObjectId]) -> Vec<bool> {
    units.iter().map(|&unit| buff_until(t, unit).is_some()).collect()
}

/// The ids of player 1's units inside the network, in id order.
fn inside(t: &TestSim) -> Vec<ObjectId> {
    t.sim().network(P1).map(|held| held.object.id).collect()
}

/// The ticks in which each unit went into the network, in order.
fn entries(t: &TestSim) -> Vec<(Tick, ObjectId)> {
    (t.events().iter())
        .filter_map(|(tick, e)| match *e {
            SimEvent::EnteredNetwork { unit, .. } => Some((*tick, unit)),
            _ => None,
        })
        .collect()
}

/// The ticks in which each unit came out of the network, in order.
fn exits(t: &TestSim) -> Vec<(Tick, ObjectId)> {
    (t.events().iter())
        .filter_map(|(tick, e)| match *e {
            SimEvent::LeftNetwork { unit, .. } => Some((*tick, unit)),
            _ => None,
        })
        .collect()
}

/// The tick in which unit `id` last went into the network.
fn entered(t: &TestSim, id: ObjectId) -> Tick {
    (entries(t).into_iter().rev())
        .find(|&(_, unit)| unit == id)
        .map(|(tick, _)| tick)
        .unwrap_or_else(|| panic!("{id:?} went in"))
}

/// The tick in which unit `id` last came out of the network.
fn left(t: &TestSim, id: ObjectId) -> Tick {
    (exits(t).into_iter().rev())
        .find(|&(_, unit)| unit == id)
        .map(|(tick, _)| tick)
        .unwrap_or_else(|| panic!("{id:?} came out"))
}

/// The hits on `target` so far: their ticks, attackers and damage.
fn hits_on(t: &TestSim, target: ObjectId) -> Vec<(Tick, ObjectId, f32)> {
    (t.events().iter())
        .filter_map(|(tick, e)| match *e {
            SimEvent::Hit {
                attacker,
                target: hit,
                damage,
                ..
            } if hit == target => Some((*tick, attacker, damage)),
            _ => None,
        })
        .collect()
}

/// The units `producer` finished so far, with their ticks.
fn produced(t: &TestSim, producer: ObjectId) -> Vec<(Tick, ObjectId)> {
    (t.events().iter())
        .filter_map(|(tick, e)| match *e {
            SimEvent::UnitProduced { producer: by, unit } if by == producer => Some((*tick, unit)),
            _ => None,
        })
        .collect()
}

/// The id the next object will have, with no spawn scheduled yet: the objects and the held units have the ids below
/// it.
fn next_id(t: &TestSim) -> u32 {
    let on_map = t.sim().objects().map(|o| o.id.0);
    let held = t.sim().held().map(|held| held.object.id.0);
    on_map.chain(held).max().unwrap_or(0) + 1
}

/// `count` Guards of player 1 spawned in the next tick in a line north of [`base`]'s entry path and sent in, then run
/// until all are inside: their ids, in the order they went in (the nearest first).
fn guards_in(t: &mut TestSim, count: u32) -> Vec<ObjectId> {
    let first = next_id(t);
    let guards: Vec<ObjectId> = (0..count).map(|i| id(first + i)).collect();
    let spawn_tick = t.sim().tick() + 1;
    for i in 0..count {
        t.spawn_unit_at(spawn_tick, GUARD, P1, ENTRY - Vec2::new(0.0, 42.0 + 40.0 * i as f32));
    }
    t.at(spawn_tick + 1, P1, order(&guards, OrderKind::Enter(T1)));
    let before = entries(t).len();
    step_until(t, 100 + 20 * count, "they file in", |t| {
        guards.iter().all(|guard| t.sim().held_unit(*guard).is_some())
    });
    entries(t)[before..].iter().map(|&(_, unit)| unit).collect()
}

/// The k-th of [`rows`]' Tunnels (from 0): id k + 1, on tiles (2 + 6k, 10) to (5 + 6k, 13).
fn row_tunnel(k: u32) -> ObjectId {
    id(k + 1)
}

/// Where a Guard stands `back` su north of the k-th of [`rows`]' Tunnels' entry path, as [`ENTRY`] is [`base`]'s.
fn before_row(k: u32, back: f32) -> Vec2 {
    let west = (2 + 6 * k) as f32 * SU_PER_GU;
    Vec2::new(west + 98.0, ENTRY.y - back)
}

/// Player 1 (the Syndicate) and player 2 (the GDO) on [`base`]'s map, with player 1's `tunnels` Tunnels in a row (ids
/// 1 to `tunnels`, [`row_tunnel`]), unturned, Side A north, and its `chambers` Null Chambers, finished (the ids after),
/// the j-th (from 0) on tiles (6j, 14) to (6j + 1, 15), south-west of a Tunnel.
fn rows(tunnels: u32, chambers: u32) -> ScenarioBuilder {
    let mut builder = ScenarioBuilder::new(&flat(64, 24))
        .faction(P1, Faction::Syndicate)
        .faction(P2, Faction::Gdo);
    for k in 0..tunnels {
        builder = builder.spawn(TUNNEL, Some(P1), tile(2 + 6 * k as i32, 10));
    }
    for j in 0..chambers {
        builder = builder.spawn(CHAMBER, Some(P1), tile(6 * j as i32, 14));
    }
    builder
}

/// A Guard of player 1 spawned in the next tick before each of the first `backs.len()` of [`rows`]' Tunnels, the k-th
/// `backs[k]` su north of its entry path ([`before_row`]), each sent into its own Tunnel in the tick after, then run
/// until all are inside: their ids, in the Tunnels' order.
fn row_guards(t: &mut TestSim, backs: &[f32]) -> Vec<ObjectId> {
    let first = next_id(t);
    let spawn_tick = t.sim().tick() + 1;
    let guards: Vec<ObjectId> = (0..backs.len() as u32).map(|k| id(first + k)).collect();
    for (k, &back) in backs.iter().enumerate() {
        t.spawn_unit_at(spawn_tick, GUARD, P1, before_row(k as u32, back));
    }
    for (k, &guard) in guards.iter().enumerate() {
        t.at(
            spawn_tick + 1,
            P1,
            order(&[guard], OrderKind::Enter(row_tunnel(k as u32))),
        );
    }
    step_until(t, 200, "each goes into its Tunnel", |t| {
        guards.iter().all(|guard| t.sim().held_unit(*guard).is_some())
    });
    guards
}

/// The held store's invariants (`tests/tunnel_network`'s): a held unit is never among the objects, nothing on the map
/// names one, and every unit an eject queue reserves is held in the queue's Tunnel owner's network; and (M6-SN) a unit
/// has a Null Chamber turn only while it is held.
fn check_held(t: &TestSim) {
    let sim = t.sim();
    for held in sim.held() {
        let unit = held.object.id;
        assert!(sim.object(unit).is_none(), "held unit {unit:?} is on the map too");
        for o in sim.objects() {
            let named = sc_sim::testing::referenced_ids(o);
            assert!(!named.contains(&unit), "{:?} names held unit {unit:?}", o.id);
        }
    }
    for o in sim.objects() {
        for entry in o.tunnel.iter().flat_map(|tunnel| &tunnel.eject) {
            let held = sim.held_unit(entry.unit).expect("a queued unit is held");
            assert_eq!(held.object.owner, o.owner, "queued at its owner's Tunnel");
        }
    }
    assert_eq!(dangling_references(sim), []);
}

/// One tick of `t`, with the held store's invariants.
fn step_checked(t: &mut TestSim) {
    t.step();
    check_held(t);
}

/// Runs `t` until `tick`, checking the held store's invariants after every tick.
fn run_checked(t: &mut TestSim, tick: Tick) {
    while t.sim().tick() < tick {
        step_checked(t);
    }
}

/// Steps until `done`, at most `ticks` ticks; panics naming `what` otherwise.
fn step_until(t: &mut TestSim, ticks: Tick, what: &str, done: impl Fn(&TestSim) -> bool) {
    let end = t.sim().tick() + ticks;
    while !done(t) {
        assert!(t.sim().tick() < end, "{what}: not by tick {end}\n{}", t.ascii(None));
        step_checked(t);
    }
}

/// This binary's numbers are the Null Chamber's (`StructureDef::chamber`): 4 slots, a turn of 2 s and a buff of 20 s,
/// in ticks; and a turn is as long as the transit delay a unit that walked in waits before it may leave
/// (`factions/syndicate/tunnels.md > Tunnel Network`), so that one sent in and out at once leaves buffed when a slot was
/// free.
#[test]
fn the_numbers_are_the_chamber_s() {
    let structure = CHAMBER.def().structure.as_ref().expect("a structure");
    let chamber = structure.chamber.expect("its service");
    assert_eq!(
        (chamber.slots as usize, seconds(chamber.turn), seconds(chamber.duration)),
        (SLOTS, TURN, DURATION)
    );
    assert_eq!(TRANSIT_DELAY_TICKS, TURN);
}
