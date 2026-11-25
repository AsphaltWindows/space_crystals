//! Agent-built Tunnels (slice M4-E; `factions/syndicate/units/agent.md > Building > Tunnel Construction Flow`,
//! `factions/syndicate/tunnels.md > Cost Scaling > Construction Cost`, `factions/syndicate/tunnels.md > Tunnel Area
//! Non-Overlap Rule`; design changes 0233, 0234, 0237, 0240 and 0242), through the gate, one test binary:
//! - `flow`: the order, the walk and the site: the price paid at once, the HP, the completion and the Agent in the
//!   network; what the site does (nothing but Cancel), what is seen of it, and how it goes (cancelled, destroyed,
//!   the defeat);
//! - `price`: the price and its refunds (0233), and the books, a property test over random orders;
//! - `rules`: where a Tunnel may stand: the overlap rule (0237), the Drillable area (0242), the hidden expansions
//!   (0240), a unit on the site, and the lingering ghosts' reservations (design change 0289 point 2).
//!
//! Every test runs under the invariants of `TestSim`, and the sites' ([`check_sites`]) after each tick it steps.

mod flow;
mod price;
mod rules;

use sc_sim::network::Place;
use sc_sim::testing::{ScenarioBuilder, TestSim, dangling_references, referenced_ids};
use sc_sim::{
    BuildingOrientation, Command, ConstructionCommand, DebugCommand, Faction, Object, ObjectId, ObjectKind, OrderKind,
    PlacementCheck, PlayerCommand, PlayerId, Reject, SimEvent, SpawnAt, Tick, TilePos, TunnelCommand, Vec2,
};

const P1: PlayerId = PlayerId(1);
const P2: PlayerId = PlayerId(2);
const AGENT: ObjectKind = ObjectKind::Agent;
const GUARD: ObjectKind = ObjectKind::Guard;
const TUNNEL: ObjectKind = ObjectKind::Tunnel;

/// Player 1's Tunnel in [`base`].
const T1: ObjectId = ObjectId(1);

/// A site east of [`base`]'s Tunnel: tiles (16, 8) to (19, 11), its Tier 1 area (14, 6) to (21, 13), clear of the
/// Tunnel's.
const SITE: TilePos = TilePos::new(16, 8);

/// The centre of [`SITE`]'s footprint, where its Agent comes out (design change 0234).
const SITE_CENTRE: Vec2 = Vec2::new(18.0 * 64.0, 10.0 * 64.0);

/// The longest walk to a site in these tests, ticks.
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

/// Both players play the Syndicate on `map`.
fn syndicate(map: &str) -> ScenarioBuilder {
    ScenarioBuilder::new(map)
        .faction(P1, Faction::Syndicate)
        .faction(P2, Faction::Syndicate)
}

/// Player 1's Tunnel (1) on a flat 40 × 20 map, unturned, on tiles (4, 8) to (7, 11): its Tier 1 area is (2, 6) to
/// (9, 13).
fn base() -> ScenarioBuilder {
    syndicate(&flat(40, 20)).spawn(TUNNEL, Some(P1), tile(4, 8))
}

/// A Build of a Tunnel at `origin`, unturned, to `units` (`queue` is Shift).
fn build(units: &[u32], origin: TilePos, queue: bool) -> Command {
    let orientation = BuildingOrientation::default();
    order(
        units,
        OrderKind::Build {
            kind: TUNNEL,
            origin,
            orientation,
        },
        queue,
    )
}

fn order(units: &[u32], kind: OrderKind, queue: bool) -> Command {
    Command::Order {
        units: units.iter().map(|&n| id(n)).collect(),
        kind,
        queue,
    }
}

/// Cancel Construction on `objects`.
fn cancel(objects: &[ObjectId]) -> Command {
    Command::Construction {
        objects: objects.to_vec(),
        command: ConstructionCommand::CancelConstruction,
    }
}

/// An Eject of an Agent at `tunnel`.
fn eject(tunnel: ObjectId) -> Command {
    let command = TunnelCommand::Eject { kind: AGENT };
    Command::Tunnel { tunnel, command }
}

/// The units in `player`'s Tunnel Network, in id order.
fn inside(t: &TestSim, player: PlayerId) -> Vec<ObjectId> {
    t.sim().network(player).map(|held| held.object.id).collect()
}

/// Sets `player`'s Supplies to `supplies`, their other banks to 0.
fn supplies_of(player: PlayerId, supplies: u32) -> DebugCommand {
    DebugCommand::SetResources {
        player,
        crystals: 0,
        supplies,
        alloys: 0,
        essence: 0,
        flux: 0,
    }
}

/// What the gate says to `player`'s `command` now (`Sim::validate`).
fn gate(t: &TestSim, player: PlayerId, command: Command) -> Result<(), Reject> {
    t.sim().validate(&PlayerCommand { player, command })
}

/// Player `player`'s Supplies.
fn supplies(t: &TestSim, player: PlayerId) -> u32 {
    t.sim().player(player).expect("in the game").supplies
}

/// The price of `player`'s next Tunnel (`Sim::build_price`).
fn price(t: &TestSim, player: PlayerId) -> u32 {
    t.sim().build_price(player, TUNNEL).expect("an Agent builds Tunnels")
}

/// `player`'s lingering ghosts (`Sim::pending_builds`): each worker and its site's origin.
fn ghosts(t: &TestSim, player: PlayerId) -> Vec<(ObjectId, TilePos)> {
    let pending = t.sim().pending_builds(player).into_iter();
    pending.map(|pending| (pending.worker, pending.site.origin)).collect()
}

/// The sites in the game, in id order.
fn sites(t: &TestSim) -> Vec<ObjectId> {
    t.sim().objects().filter(|o| o.is_site()).map(|o| o.id).collect()
}

fn object(t: &TestSim, id: ObjectId) -> &Object {
    t.sim().object(id).unwrap_or_else(|| panic!("{id:?} is on the map"))
}

/// Unit `n`'s current order, if any.
fn current(t: &TestSim, n: u32) -> Option<OrderKind> {
    let orders = object(t, id(n)).orders.as_ref()?;
    orders.current.as_ref().map(|active| active.order.kind)
}

/// Whether unit `n` is on the map with no order, current or queued.
fn idle(t: &TestSim, n: u32) -> bool {
    (t.sim().object(id(n))).is_some_and(|o| o.orders.as_ref().is_some_and(|orders| orders.is_idle()))
}

/// What Agent `agent`'s ghost of a Tunnel at `origin`, unturned and without Shift, shows (`Sim::build_check`).
fn ghost(t: &TestSim, agent: u32, origin: TilePos) -> PlacementCheck {
    let orientation = BuildingOrientation::default();
    (t.sim().build_check(id(agent), TUNNEL, origin, orientation, false)).expect("an Agent on the map")
}

/// The Build orders refunded so far: each worker and what came back.
fn refunds(t: &TestSim) -> Vec<(ObjectId, u32)> {
    (t.events().iter())
        .filter_map(|(_, e)| match *e {
            SimEvent::BuildRefunded { worker, refund, .. } => Some((worker, refund)),
            _ => None,
        })
        .collect()
}

/// The Builds that failed so far: each worker, and what the check on its arrival found.
fn failures(t: &TestSim) -> Vec<(ObjectId, PlacementCheck)> {
    (t.events().iter())
        .filter_map(|(_, e)| match e {
            SimEvent::BuildFailed { worker, problems, .. } => Some((*worker, problems.clone())),
            _ => None,
        })
        .collect()
}

/// The sites started so far: the tick, the worker and the site.
fn starts(t: &TestSim) -> Vec<(Tick, ObjectId, ObjectId)> {
    (t.events().iter())
        .filter_map(|(tick, e)| match *e {
            SimEvent::SiteStarted { worker, site } => Some((*tick, worker, site)),
            _ => None,
        })
        .collect()
}

/// The sites' invariants: each site's builder is held in it, its owner's; each unit held at a site builds it; no held
/// unit is on the map, nor named by anything there; and no reference dangles.
fn check_sites(t: &TestSim) {
    let sim = t.sim();
    for o in sim.objects() {
        // A Tunnel's expansion has no worker (slice M4-F).
        let Some(worker) = o.construction.and_then(|site| site.builder.unit()) else {
            continue;
        };
        let held =
            (sim.held_unit(worker)).unwrap_or_else(|| panic!("site {:?}: its builder {worker:?} isn't held", o.id));
        assert_eq!(held.place, Place::Site(o.id), "site {:?}'s builder", o.id);
        assert_eq!(held.object.owner, o.owner, "site {:?}'s builder", o.id);
    }
    for held in sim.held() {
        let unit = held.object.id;
        assert!(sim.object(unit).is_none(), "held unit {unit:?} is on the map too");
        if let Place::Site(site) = held.place {
            let builder = (sim.object(site))
                .and_then(|o| o.construction)
                .and_then(|c| c.builder.unit());
            assert_eq!(
                builder,
                Some(unit),
                "unit {unit:?} is held at {site:?}, which it doesn't build"
            );
        }
        for o in sim.objects() {
            assert!(
                !referenced_ids(o).contains(&unit),
                "{:?} names held unit {unit:?}",
                o.id
            );
        }
    }
    assert_eq!(dangling_references(sim), []);
}

/// One tick, then the sites' invariants.
fn step(t: &mut TestSim) {
    t.step();
    check_sites(t);
}

/// `player`'s `command` in the next tick, which runs.
fn next(t: &mut TestSim, player: PlayerId, command: Command) {
    let tick = t.sim().tick() + 1;
    t.at(tick, player, command);
    step(t);
}

/// A debug command in the next tick, which runs.
fn next_debug(t: &mut TestSim, command: DebugCommand) {
    next(t, P1, Command::Debug(command));
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

/// Steps until worker `worker` starts a site, at most [`WALK`] ticks: the tick and the site.
fn until_started(t: &mut TestSim, worker: u32) -> (Tick, ObjectId) {
    let end = t.sim().tick() + WALK;
    loop {
        if let Some(&(tick, _, site)) = starts(t).iter().find(|(_, w, _)| *w == id(worker)) {
            return (tick, site);
        }
        assert!(t.sim().tick() < end, "worker {worker} started no site by tick {end}");
        step(t);
    }
}

/// Steps until worker `worker`'s Build fails, at most [`WALK`] ticks: what the check on its arrival found.
fn until_failed(t: &mut TestSim, worker: u32) -> PlacementCheck {
    let end = t.sim().tick() + WALK;
    loop {
        if let Some((_, problems)) = failures(t).into_iter().find(|(w, _)| *w == id(worker)) {
            return problems;
        }
        assert!(
            starts(t).iter().all(|(_, w, _)| *w != id(worker)),
            "worker {worker} started its site"
        );
        assert!(
            t.sim().tick() < end,
            "worker {worker}'s Build didn't fail by tick {end}"
        );
        step(t);
    }
}
