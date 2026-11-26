//! A Tunnel's expansions (slice M4-F; `factions/syndicate/tunnel_menus.md > Tunnel ObjectInterfaceState > ExpandMenu`,
//! `factions/syndicate/tunnel_menus.md > AwaitingPlacement (Expansion)`, `factions/syndicate/tunnel_menus.md >
//! Expansions under construction`; design changes 0239, 0240, 0241 and 0254), through the gate, one test binary:
//! - `placing`: where an expansion may go: inside the Tunnel's area, on Drillable ground (0239), clear of every
//!   structure (0240) and resource (0241), ground units standing on it or not (design change 0290), its ghost judging
//!   by what the owner knows and the command by the truth. No lingering ghost can be in its way (0290: "No ghost check
//!   is needed"), and none is checked;
//! - `building`: the click pays and takes Tunnel Space, the frames and the HP, completion, the one operation, and Eject
//!   meanwhile;
//! - `cancelling`: the refunds (0254) by the Tunnel's X and by the site's own Cancel Construction, and the site gone
//!   otherwise;
//! - `hidden`: what the other player sees of it: nothing; and a Colonists Warp In over it, red at once (0290).
//!
//! Every test runs under the invariants of `TestSim`, the operation invariant O1 among them (a Tunnel's operation and
//! its site name each other). The test kinds of another cost or Tier Requirement are the sim's own unit tests
//! (`sim/expansions/tests.rs`), since only those can change a def.

mod building;
mod cancelling;
mod hidden;
mod placing;

use sc_sim::testing::{ScenarioBuilder, TestSim};
use sc_sim::tunnel::Operation;
use sc_sim::{
    BuildingOrientation, Command, ConstructionCommand, DebugCommand, Faction, Object, ObjectId, ObjectKind,
    PlacementCheck, PlacementProblem, PlayerCommand, PlayerId, Reject, SimEvent, SpawnAt, TilePos, TileProblem,
    TunnelCommand,
};

const P1: PlayerId = PlayerId(1);
const P2: PlayerId = PlayerId(2);
const AGENT: ObjectKind = ObjectKind::Agent;
const HIDEOUT: ObjectKind = ObjectKind::Hideout;
const TUNNEL: ObjectKind = ObjectKind::Tunnel;

/// Player 1's Tunnel in [`base`], on tiles (4, 8) to (7, 11): its Tier 1 area is (2, 6) to (9, 13).
const T1: ObjectId = ObjectId(1);
/// Player 1's Agent in [`base`], outside the area.
const AGENT_1: ObjectId = ObjectId(3);

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

/// Both players play the Syndicate on `map`.
fn syndicate(map: &str) -> ScenarioBuilder {
    ScenarioBuilder::new(map)
        .faction(P1, Faction::Syndicate)
        .faction(P2, Faction::Syndicate)
}

/// Player 1's start on a flat 40 × 20 map, as a Starting Layout has it (`factions/syndicate/tunnels.md > Tunnel >
/// Starting Conditions`): the Tier 1 Tunnel ([`T1`]), a Hideout inside its area, in its corner at (8, 12) to (9, 13)
/// (2), and an Agent outside it ([`AGENT_1`]); Tunnel Space 1/11.
fn base_on(map: &str) -> ScenarioBuilder {
    syndicate(map)
        .spawn(TUNNEL, Some(P1), tile(4, 8))
        .spawn(HIDEOUT, Some(P1), tile(8, 12))
        .spawn(AGENT, Some(P1), point(14, 10))
}

fn base() -> ScenarioBuilder {
    base_on(&flat(40, 20))
}

/// `scenario` started, player 1 with `crystals` Space Crystals from tick 1, which has run.
fn started(scenario: ScenarioBuilder, crystals: u32) -> TestSim {
    let mut t = scenario.start();
    t.debug_at(1, set_crystals(P1, crystals));
    t.run_until(1);
    t
}

/// [`started`] with auto-targeting off (`TestSim::without_auto_targeting`), for player 2's units standing by player
/// 1's without a fight.
fn started_peaceful(scenario: ScenarioBuilder, crystals: u32) -> TestSim {
    let mut t = scenario.start();
    t.without_auto_targeting();
    t.debug_at(1, set_crystals(P1, crystals));
    t.run_until(1);
    t
}

/// Sets `player`'s Space Crystals to `crystals`, their other banks to 0.
fn set_crystals(player: PlayerId, crystals: u32) -> DebugCommand {
    DebugCommand::SetResources {
        player,
        crystals,
        supplies: 0,
        alloys: 0,
        essence: 0,
        flux: 0,
    }
}

/// An Expand of a Hideout at `(x, y)`, unturned, to [`T1`].
fn expand(x: i32, y: i32) -> Command {
    let command = TunnelCommand::Expand {
        kind: HIDEOUT,
        origin: at(x, y),
        orientation: BuildingOrientation::default(),
    };
    Command::Tunnel { tunnel: T1, command }
}

/// [`T1`]'s X: Cancel its operation.
fn cancel_operation() -> Command {
    Command::Tunnel {
        tunnel: T1,
        command: TunnelCommand::Cancel,
    }
}

/// Cancel Construction on `site`.
fn cancel_site(site: ObjectId) -> Command {
    Command::Construction {
        objects: vec![site],
        command: ConstructionCommand::CancelConstruction,
    }
}

/// What the gate says to `player`'s `command` now (`Sim::validate`).
fn gate(t: &TestSim, player: PlayerId, command: Command) -> Result<(), Reject> {
    t.sim().validate(&PlayerCommand { player, command })
}

/// The refusal of a footprint whose first problem is `problem` on tile `(x, y)`.
fn tile_problem(x: i32, y: i32, problem: TileProblem) -> Result<(), Reject> {
    Err(Reject::Unplaceable(PlacementProblem::Tile(at(x, y), problem)))
}

/// `player`'s `command` in the next tick, which runs.
fn next(t: &mut TestSim, player: PlayerId, command: Command) {
    let tick = t.sim().tick() + 1;
    t.at(tick, player, command);
    t.step();
}

/// A debug command in the next tick, which runs.
fn next_debug(t: &mut TestSim, command: DebugCommand) {
    next(t, P1, Command::Debug(command));
}

fn object(t: &TestSim, id: ObjectId) -> &Object {
    t.sim().object(id).unwrap_or_else(|| panic!("{id:?} is on the map"))
}

fn crystals(t: &TestSim, player: PlayerId) -> u32 {
    t.sim().player(player).expect("in the game").crystals
}

/// [`T1`]'s operation, if any.
fn operation(t: &TestSim) -> Option<Operation> {
    t.sim().object(T1)?.tunnel.as_ref()?.operation
}

/// The site [`T1`] builds; panics if it builds none.
fn building(t: &TestSim) -> ObjectId {
    match operation(t) {
        Some(Operation::Building(site)) => site,
        other => panic!("Tunnel {T1:?} builds nothing: {other:?}"),
    }
}

/// `player`'s Tunnel Space, used and available.
fn tunnel_space(t: &TestSim, player: PlayerId) -> (u32, u32) {
    let cap = t.sim().army_cap(player).expect("a Syndicate player");
    (cap.used, cap.available)
}

/// What [`T1`]'s ghost of a Hideout at `(x, y)`, unturned, shows (`Sim::expansion_check`).
fn ghost(t: &TestSim, x: i32, y: i32) -> PlacementCheck {
    let orientation = BuildingOrientation::default();
    (t.sim().expansion_check(T1, HIDEOUT, at(x, y), orientation)).expect("a Tunnel of a player's")
}

/// The units in `player`'s Tunnel Network, in id order.
fn inside(t: &TestSim, player: PlayerId) -> Vec<ObjectId> {
    t.sim().network(player).map(|held| held.object.id).collect()
}

/// The events so far that `keep` maps.
fn events<T>(t: &TestSim, keep: impl Fn(&SimEvent) -> Option<T>) -> Vec<T> {
    t.events().iter().filter_map(|(_, e)| keep(e)).collect()
}
