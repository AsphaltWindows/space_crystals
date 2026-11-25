//! Losing Tunnels (slice M4-H; `factions/syndicate/tunnels.md > Losing Tunnels`, design changes 0254 and 0258, which
//! answers question Q0069; `core/game_rules.md > On Defeat`), through the gate, one test binary:
//! - `neutral`: a Tunnel destroyed leaves its expansions neutral, visible to everyone and decaying, their work in
//!   progress lost and their queue refunded; the Syndicate defeated with its last structure;
//! - `stranded`: the network units no remaining Tunnel lets out are destroyed; a player who lives on without a Tunnel
//!   rebuilds for nothing; an Agent's site isn't the lost Tunnel's;
//! - `attack`: a neutral expansion is attacked only by an explicit Attack;
//! - `reclaim`: a Tunnel that finishes over a neutral expansion takes it back, hidden again, and never while a site;
//!   one that covers it partly leaves it neutral, blocking its cells and taking Tunnel Space.
//!
//! Every test runs under the invariants of `TestSim`, and this binary's ([`check`]) after each tick it steps.

mod attack;
mod neutral;
mod reclaim;
mod stranded;

use sc_sim::network::Place;
use sc_sim::testing::{ScenarioBuilder, TestSim, dangling_references, referenced_ids};
use sc_sim::{
    BuildingOrientation, Command, DebugCommand, Faction, Object, ObjectId, ObjectKind, OrderKind, PlayerCommand,
    PlayerId, Reject, SimEvent, SpawnAt, Tick, TilePos,
};

const P1: PlayerId = PlayerId(1);
const P2: PlayerId = PlayerId(2);
const AGENT: ObjectKind = ObjectKind::Agent;
const GUARD: ObjectKind = ObjectKind::Guard;
const HIDEOUT: ObjectKind = ObjectKind::Hideout;
const TUNNEL: ObjectKind = ObjectKind::Tunnel;

/// Player 1's Tunnel in [`base`], on tiles (4, 8) to (7, 11): its Tier 1 area is (2, 6) to (9, 13).
const T1: ObjectId = ObjectId(1);
/// Player 1's Hideout in [`base`], on tiles (8, 8) to (9, 9), inside [`T1`]'s area.
const H1: ObjectId = ObjectId(2);
/// Where [`H1`] stands.
const HIDEOUT_AT: TilePos = TilePos::new(8, 8);

/// The ticks a neutral expansion decays from full HP: 100 seconds of 16 ticks, at 1% of its MaxHP a second
/// (`sc_sim::tuning::tunnel::DECAY_PERCENT_PER_SECOND`, design change 0258's placeholder).
const DECAY_TICKS: u32 = 1600;

/// The longest walk to a site or into a Tunnel in these tests, ticks.
const WALK: u32 = 600;

fn flat(w: usize, h: usize) -> String {
    vec![".".repeat(w); h].join("\n")
}

fn at(x: i32, y: i32) -> TilePos {
    TilePos::new(x, y)
}

fn tile(x: i32, y: i32) -> SpawnAt {
    SpawnAt::Tile(at(x, y))
}

/// The centre of tile `(x, y)`.
fn point(x: i32, y: i32) -> SpawnAt {
    SpawnAt::Point(at(x, y).center())
}

fn id(n: u32) -> ObjectId {
    ObjectId(n)
}

/// Both players play the Syndicate on a flat 48 × 24 map, with player 1's Tunnel [`T1`] and its Hideout [`H1`].
fn base() -> ScenarioBuilder {
    ScenarioBuilder::new(&flat(48, 24))
        .faction(P1, Faction::Syndicate)
        .faction(P2, Faction::Syndicate)
        .spawn(TUNNEL, Some(P1), tile(4, 8))
        .spawn(HIDEOUT, Some(P1), tile(HIDEOUT_AT.x, HIDEOUT_AT.y))
}

fn order(units: &[u32], kind: OrderKind, queue: bool) -> Command {
    Command::Order {
        units: units.iter().map(|&n| id(n)).collect(),
        kind,
        queue,
    }
}

/// A Build of a Tunnel at `origin`, unturned, to `units`.
fn build(units: &[u32], origin: TilePos) -> Command {
    let orientation = BuildingOrientation::default();
    order(
        units,
        OrderKind::Build {
            kind: TUNNEL,
            origin,
            orientation,
        },
        false,
    )
}

/// A debug removal of `id`: the removal path, as a destruction (`DebugCommand::Remove`).
fn remove(id: ObjectId) -> Command {
    Command::Debug(DebugCommand::Remove { id })
}

/// Sets `player`'s Space Crystals and Supplies, their other banks to 0.
fn bank(player: PlayerId, crystals: u32, supplies: u32) -> Command {
    Command::Debug(DebugCommand::SetResources {
        player,
        crystals,
        supplies,
        alloys: 0,
        essence: 0,
        flux: 0,
    })
}

/// What the gate says to `player`'s `command` now (`Sim::validate`).
fn gate(t: &TestSim, player: PlayerId, command: Command) -> Result<(), Reject> {
    t.sim().validate(&PlayerCommand { player, command })
}

fn object(t: &TestSim, id: ObjectId) -> &Object {
    t.sim().object(id).unwrap_or_else(|| panic!("{id:?} is on the map"))
}

/// The owner of `id`, which is on the map.
fn owner(t: &TestSim, id: ObjectId) -> Option<PlayerId> {
    object(t, id).owner
}

/// The HP of `id`, which is on the map.
fn hp(t: &TestSim, id: ObjectId) -> f32 {
    object(t, id).hp.expect("destructible")
}

/// The units in `player`'s Tunnel Network, in id order.
fn inside(t: &TestSim, player: PlayerId) -> Vec<ObjectId> {
    t.sim().network(player).map(|held| held.object.id).collect()
}

/// The tick in which `id` died (`SimEvent::Died`), if it did, with the owner the event names.
fn died(t: &TestSim, id: ObjectId) -> Option<(Tick, Option<PlayerId>)> {
    t.events().iter().find_map(|(tick, e)| match *e {
        SimEvent::Died { id: dead, owner, .. } if dead == id => Some((*tick, owner)),
        _ => None,
    })
}

/// The tick in which `id` left the game (`SimEvent::Removed`), if it did.
fn removed(t: &TestSim, id: ObjectId) -> Option<Tick> {
    (t.events().iter()).find_map(|(tick, e)| (*e == SimEvent::Removed(id)).then_some(*tick))
}

/// The events of tick `tick`.
fn events_of(t: &TestSim, tick: Tick) -> Vec<SimEvent> {
    (t.events().iter())
        .filter(|(at, _)| *at == tick)
        .map(|(_, e)| e.clone())
        .collect()
}

/// Whether `event` names object `id` (its debug text does): what a player whose client gets it learns of the object.
fn names(event: &SimEvent, id: ObjectId) -> bool {
    format!("{event:?}").contains(&format!("{id:?}"))
}

/// This binary's invariants: no reference dangles and no held unit is on the map or named there (the held store's,
/// `tests/tunnel_network`); each site's builder is held in it; and only a neutral expansion decays, every one of
/// those a lost Tunnel left (and no other neutral expansion is in these tests' games).
fn check(t: &TestSim) {
    let sim = t.sim();
    assert_eq!(dangling_references(sim), []);
    for held in sim.held() {
        let unit = held.object.id;
        assert!(sim.object(unit).is_none(), "held unit {unit:?} is on the map too");
        for o in sim.objects() {
            assert!(
                !referenced_ids(o).contains(&unit),
                "{:?} names held unit {unit:?}",
                o.id
            );
        }
        if let Place::Site(site) = held.place {
            let builder = (sim.object(site))
                .and_then(|o| o.construction)
                .map(|c| c.builder.unit());
            assert_eq!(builder, Some(unit), "unit {unit:?} is held at {site:?}");
        }
    }
    for o in sim.objects() {
        let expansion = o.kind == HIDEOUT;
        assert_eq!(
            o.decay.is_some(),
            expansion && o.owner.is_none(),
            "{:?}: only a neutral expansion decays",
            o.id
        );
    }
}

/// One tick, then this binary's invariants.
fn step(t: &mut TestSim) {
    t.step();
    check(t);
}

/// `player`'s `command` in the next tick, which runs.
fn next(t: &mut TestSim, player: PlayerId, command: Command) {
    let tick = t.sim().tick() + 1;
    t.at(tick, player, command);
    step(t);
}

/// Steps until `tick`.
fn run_until(t: &mut TestSim, tick: Tick) {
    while t.sim().tick() < tick {
        step(t);
    }
}

/// Steps until `done`, at most `ticks` ticks; panics naming `what` otherwise.
fn step_until(t: &mut TestSim, ticks: u32, what: &str, done: impl Fn(&TestSim) -> bool) {
    let end = t.sim().tick() + ticks;
    while !done(t) {
        assert!(t.sim().tick() < end, "{what}: not by tick {end}");
        step(t);
    }
}

/// Orders `units`, `player`'s, into Tunnel `tunnel`, and steps until they are all in the network.
fn enter(t: &mut TestSim, player: PlayerId, units: &[u32], tunnel: ObjectId) {
    next(t, player, order(units, OrderKind::Enter(tunnel), false));
    let all_in = |t: &TestSim| units.iter().all(|&n| t.sim().held_unit(id(n)).is_some());
    step_until(t, WALK, "the units go in", all_in);
}

/// Steps until worker `worker`'s site starts, at most [`WALK`] ticks: the tick and the site.
fn until_started(t: &mut TestSim, worker: u32) -> (Tick, ObjectId) {
    let end = t.sim().tick() + WALK;
    loop {
        let started = t.events().iter().find_map(|(tick, e)| match *e {
            SimEvent::SiteStarted { worker: w, site } if w == id(worker) => Some((*tick, site)),
            _ => None,
        });
        if let Some(started) = started {
            return started;
        }
        assert!(t.sim().tick() < end, "worker {worker} started no site by tick {end}");
        step(t);
    }
}

/// The tick in which site `site` completed, if it did.
fn completed(t: &TestSim, site: ObjectId) -> Option<Tick> {
    t.events().iter().find_map(|(tick, e)| match *e {
        SimEvent::SiteCompleted { site: done, .. } if done == site => Some(*tick),
        _ => None,
    })
}
