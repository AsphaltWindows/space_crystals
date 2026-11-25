//! Beacon Deployment (slice M3-F; `factions/colonists/economy.md > Beacon Deployment`, `sc_sim::deploy`): a Drone walks
//! to a site and transforms in place into a Beacon over 48 frames. One test per acceptance point of the plan
//! (`artifacts/director/m3-plan.md` §3.3 F) and per Verification point of design changes 0109 to 0114 that concerns a
//! deployment: the walk and the start on arrival, the timing, the HP, the Shields and the Beacon Capacity, the cancel
//! and its fractions, the load, defeat, a destruction, the footprint claimed, Shift, saves and replays. The gate (what
//! the player knows, the reservations of design change 0125, the commands a deploying Beacon refuses) and the failures
//! on arrival are `tests/deploy_gate/`'. The walks are measured by their outcomes, never by their ticks: the Drone's
//! DragMovement (slice M3-G) sets their pace. `TestSim` checks the movement invariants after every tick.
//!
//! One test binary:
//! - `arrival`: the walk to the site and the deployment begun on arrival (the Beacon, its timing, its HP and the
//!   Beacon Capacity), and a Deploy queued behind a Move (Shift);
//! - `cancel`: a cancel, back to the Drone with the fraction of its HP and of its Shields, and its Beacon Capacity
//!   back even over the cap;
//! - `loads`: a load deployed with, lost at completion or back with the Drone on a cancel (0114);
//! - `beacon`: the Beacon under deployment, a structure from the first frame: its owner kept in the game, its
//!   footprint claimed, taken for a structure by enemy auto-targeting, the damage it takes, and destroyed midway;
//! - `saves`: a game saved in the middle of a deployment, and its replay.

mod arrival;
mod beacon;
mod cancel;
mod loads;
mod saves;

use sc_sim::deploy::{self, DEPLOYMENT_FRAMES};
use sc_sim::testing::{ScenarioBuilder, TestSim, dangling_references};
use sc_sim::{
    Command, ConstructionCommand, DebugCommand, Faction, Object, ObjectId, ObjectKind, OrderKind, PlayerCommand,
    PlayerId, Reject, Replay, SU_PER_GU, Sim, SimEvent, SpawnAt, Target, Tick, TickInputs, TilePos, Vec2,
    VisibilityState,
};

use ObjectKind::{Beacon, CrystalPatch, Drone, EgressNode, Peacekeeper};

const P1: PlayerId = PlayerId(1);
const P2: PlayerId = PlayerId(2);

/// Where most tests deploy: the Beacon on tiles (8, 9) to (9, 10), in the sight of Drone 2 where it starts.
const SITE: TilePos = TilePos::new(8, 9);

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

fn object(t: &TestSim, n: u32) -> &Object {
    t.sim().object(id(n)).expect("the object exists")
}

fn deploy_at(origin: TilePos) -> OrderKind {
    OrderKind::Deploy { origin }
}

/// Player 1's order `kind` to `units`.
fn order(units: &[u32], kind: OrderKind, queue: bool) -> Command {
    Command::Order {
        units: units.iter().map(|&n| id(n)).collect(),
        kind,
        queue,
    }
}

/// Cancel Deployment (`> Cancel Deployment`): `ConstructionCommand::CancelTransform` to `objects`.
fn cancel(objects: &[u32]) -> Command {
    Command::Construction {
        objects: objects.iter().map(|&n| id(n)).collect(),
        command: ConstructionCommand::CancelTransform,
    }
}

/// Schedules player 1's `command` for the next tick and runs it.
fn next(t: &mut TestSim, command: Command) -> &mut TestSim {
    let tick = t.sim().tick() + 1;
    t.at(tick, P1, command).step()
}

/// `player`'s Beacon Capacity: used, available.
fn capacity(t: &TestSim, player: PlayerId) -> (u32, u32) {
    let cap = t.sim().army_cap(player).expect("a Colonists player");
    (cap.used, cap.available)
}

/// The transformations completed so far: tick, object, kind.
fn completions(t: &TestSim) -> Vec<(Tick, u32, ObjectKind)> {
    t.events()
        .iter()
        .filter_map(|(tick, e)| match e {
            SimEvent::TransformCompleted { id, kind } => Some((*tick, id.0, *kind)),
            _ => None,
        })
        .collect()
}

/// The ticks of the events `pick` picks so far.
fn ticks_of(t: &TestSim, pick: impl Fn(&SimEvent) -> bool) -> Vec<Tick> {
    t.events()
        .iter()
        .filter(|(_, e)| pick(e))
        .map(|&(tick, _)| tick)
        .collect()
}

/// Steps until Drone `n` is no longer one: it began deploying. Returns where it stood at the end of the last tick it
/// was a Drone. Panics if its Deploy failed, or if it takes more than 600 ticks.
fn walk(t: &mut TestSim, n: u32) -> Vec2 {
    let mut last = t.pos(n);
    while object(t, n).kind == Drone {
        let failed = t
            .sim()
            .events()
            .iter()
            .any(|e| matches!(e, SimEvent::DeployFailed { drone, .. } if *drone == id(n)));
        assert!(!failed, "Drone {n}'s Deploy failed in tick {}", t.sim().tick());
        assert!(t.sim().tick() < 600, "Drone {n} still walks at {}", t.pos(n));
        last = t.pos(n);
        t.step();
    }
    last
}

/// `builder` with both players playing the Colonists (plan decision T27), whose army cap is Beacon Capacity.
fn colonists(builder: ScenarioBuilder) -> ScenarioBuilder {
    builder.faction(P1, Faction::Colonists).faction(P2, Faction::Colonists)
}

/// A 32 × 20 flat map for two Colonists players: player 1's Egress Node 1 on (2, 2) to (3, 3) and its Drone 2 at the
/// centre of tile (6, 10); player 2's Egress Node 3 on (28, 16) to (29, 17), far out of player 1's sight.
fn base() -> ScenarioBuilder {
    colonists(ScenarioBuilder::new(&flat(32, 20)))
        .spawn(EgressNode, Some(P1), tile(2, 2))
        .spawn(Drone, Some(P1), point(6, 10))
        .spawn(EgressNode, Some(P2), tile(28, 16))
}

/// [`base`] at its Beacon Capacity, 10/10: player 1's second Egress Node 4 on (2, 6) to (3, 7), and its nine Drones
/// 5 to 13 in a row from (14, 18) to (22, 18).
fn at_capacity() -> ScenarioBuilder {
    (14..23).fold(base().spawn(EgressNode, Some(P1), tile(2, 6)), |builder, x| {
        builder.spawn(Drone, Some(P1), point(x, 18))
    })
}

/// [`base`] with Drone 2 ordered in tick 2, the site in its sight, to deploy at [`SITE`] (checked by the gate first),
/// run until it began. Returns the tick it began.
fn deployed(t: &mut TestSim) -> Tick {
    t.step();
    let tiles = deploy::site(SITE);
    assert!(
        tiles
            .tiles()
            .all(|tile| t.sim().visibility(P1, tile) == VisibilityState::Visible),
        "the site is in sight"
    );
    let command = order(&[2], deploy_at(SITE), false);
    let cmd = PlayerCommand {
        player: P1,
        command: command.clone(),
    };
    assert_eq!(t.sim().validate(&cmd), Ok(()));
    next(t, command);
    walk(t, 2);
    t.sim().tick()
}

/// `sim` saved, with object `n`'s `field` set to `value`, and loaded: a state no command makes on demand.
fn edited(sim: &Sim, n: u32, field: &str, value: serde_json::Value) -> Sim {
    let mut state = serde_json::to_value(sim).unwrap();
    let objects = state["objects"].as_array_mut().unwrap();
    let o = objects.iter_mut().find(|o| o["id"] == n).expect("the object is saved");
    o[field] = value;
    serde_json::from_value(state).unwrap()
}
