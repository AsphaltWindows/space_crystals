//! The pop-up Laser Turret (slices M4-S2 and M5-Y, plan decision T50; `factions/syndicate/structures/laser_turret.md >
//! Pop-up turret`, `factions/syndicate/structures/laser_turret.md > Pop-up rules`,
//! `factions/syndicate/structures/laser_turret.md > When its Tunnel is lost`,
//! `factions/syndicate/structures/laser_turret.md > Construction`; design changes 0267, 0296 and 0358 to 0360), through
//! the gate, one test binary:
//! - `cycle`: lowered from its first tick, 8 frames up at an enemy in range, firing only while raised, 32 frames after
//!   its last target 8 frames down, and a target seen while it retracts;
//! - `blocking`: design change 0296 point 3: a raised turret blocks a walker and a lowered one doesn't; a unit over it
//!   keeps it down, an enemy or an own unit with an order; its owner's idle unit is pushed off, and it rises;
//! - `hidden`: what the other player sees of it lowered, nothing, and its two armours;
//! - `building`: the Tunnel's build of it, from the ExpandMenu: the Optics Lab, the price, the operation slot
//!   (0267), the hidden site, the 75% cancel, Tunnel Space by quarters from the site's first frame; and no Agent's
//!   (design change 0359);
//! - `active`: active only inside its Tunnel's area (design change 0360): one outside its owner's areas stays down, and
//!   an area that comes to cover it makes it active; its Tunnel lost, it drops, neutral and seen by everyone, never
//!   fires and decays, until a new Tunnel takes it back, active again.
//!
//! Every test runs under the invariants of `TestSim`: I2 among them (no ground box overlaps a blocked tile), which a
//! turret rising under a unit would break in the tick it did.

mod active;
mod blocking;
mod building;
mod cycle;
mod hidden;

use glam::Vec2;
use sc_sim::popup::PopUp;
use sc_sim::progress::Progress;
use sc_sim::testing::{ScenarioBuilder, TestSim};
use sc_sim::units::Tick;
use sc_sim::{
    BuildingOrientation, Command, DebugCommand, Faction, Object, ObjectId, ObjectKind, OrderKind, PlayerCommand,
    PlayerId, Reject, SimEvent, SpawnAt, Target, TilePos, TunnelCommand,
};

const P1: PlayerId = PlayerId(1);
const P2: PlayerId = PlayerId(2);
const AGENT: ObjectKind = ObjectKind::Agent;
const GUARD: ObjectKind = ObjectKind::Guard;
/// The Colonists' worker, unarmed: an idle unit that goes for no enemy by itself.
const DRONE: ObjectKind = ObjectKind::Drone;
const TUNNEL: ObjectKind = ObjectKind::Tunnel;
const TURRET: ObjectKind = ObjectKind::LaserTurret;

/// Player 1's Tunnel in [`base`], on tiles (4, 8) to (7, 11): its Tier 1 area is (2, 6) to (9, 13).
const T1: ObjectId = ObjectId(1);
/// Player 1's turret in [`with_turret`], on [`SPOT`].
const POPUP: ObjectId = ObjectId(3);
/// The turret's tile, on the east edge of the Tunnel's area, and its centre.
const SPOT: TilePos = TilePos::new(9, 10);
const CENTRE: Vec2 = Vec2::new(9.5 * 64.0, 10.5 * 64.0);

fn at(x: i32, y: i32) -> TilePos {
    TilePos::new(x, y)
}

fn tile(x: i32, y: i32) -> SpawnAt {
    SpawnAt::Tile(at(x, y))
}

/// Both players play the Syndicate on a flat 40 × 20 map; player 1 has its Tier 1 Tunnel ([`T1`]) and an Optics Lab
/// (object 2, on (2, 6) to (3, 7)) in its area.
fn base() -> ScenarioBuilder {
    ScenarioBuilder::new(&vec![".".repeat(40); 20].join("\n"))
        .faction(P1, Faction::Syndicate)
        .faction(P2, Faction::Syndicate)
        .spawn(TUNNEL, Some(P1), tile(4, 8))
        .spawn(ObjectKind::OpticsLab, Some(P1), tile(2, 6))
}

/// [`base`] with player 1's turret ([`POPUP`]) on [`SPOT`], inside the area.
fn with_turret() -> ScenarioBuilder {
    base().spawn(TURRET, Some(P1), SpawnAt::Tile(SPOT))
}

/// `scenario` started and its first tick run: its turret ([`POPUP`]) is lowered from the start (design change 0360:
/// "always a hidden **pop-up turret**"), and stays down with no target.
fn lowered(scenario: ScenarioBuilder) -> TestSim {
    let mut t = scenario.start();
    assert_eq!(popup(&t, POPUP), Some(PopUp::LOWERED), "as the game starts");
    t.step();
    assert_eq!(popup(&t, POPUP), Some(PopUp::LOWERED));
    t
}

fn object(t: &TestSim, id: ObjectId) -> &Object {
    t.sim().object(id).unwrap_or_else(|| panic!("{id:?} is on the map"))
}

/// The pop-up state of turret `id`.
fn popup(t: &TestSim, id: ObjectId) -> Option<PopUp> {
    object(t, id).popup
}

/// Where a Guard stands `gap` grid units east of the turret's tile, edge to edge, level with it: inside the turret's
/// Range of 5 up to 5, and beyond the Guard's own Range of 3 from 3 on.
fn east(gap: f32) -> Vec2 {
    Vec2::new(10.0 * 64.0 + gap * 64.0 + 18.0, CENTRE.y)
}

/// A debug spawn of a unit of `kind` for `owner` at `pos`.
fn spawn(kind: ObjectKind, owner: PlayerId, pos: Vec2) -> DebugCommand {
    DebugCommand::Spawn {
        kind,
        owner: Some(owner),
        at: SpawnAt::Point(pos),
        orientation: Default::default(),
    }
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

/// The id of the object the latest tick spawned.
fn newest(t: &TestSim) -> ObjectId {
    t.sim().objects().last().expect("an object").id
}

/// An order of `kind` to `unit`, without Shift.
fn order(unit: ObjectId, kind: OrderKind) -> Command {
    Command::Order {
        units: vec![unit],
        kind,
        queue: false,
    }
}

/// What the gate says to `player`'s `command` now (`Sim::validate`).
fn gate(t: &TestSim, player: PlayerId, command: Command) -> Result<(), Reject> {
    t.sim().validate(&PlayerCommand { player, command })
}

/// The ticks of `attacker`'s shots so far.
fn shots(t: &TestSim, attacker: ObjectId) -> Vec<Tick> {
    (t.events().iter())
        .filter_map(|(tick, e)| match *e {
            SimEvent::Attacked { attacker: a, .. } if a == attacker => Some(*tick),
            _ => None,
        })
        .collect()
}

/// Whether the box of unit `id` overlaps the turret's tile ([`SPOT`]): strictly, as the turret's own test.
fn over_the_turret(t: &TestSim, id: ObjectId) -> bool {
    let o = object(t, id);
    let half = o.def().half_extents().expect("a unit");
    let offset = (o.pos - CENTRE).abs();
    offset.x < half.x + 32.0 && offset.y < half.y + 32.0
}

/// Whether the turret's tile ([`SPOT`]) is blocked on the true grids, which the units collide on.
fn blocked(t: &TestSim) -> bool {
    let grids = t.sim().nav().grids();
    assert!(!grids.is_empty());
    let blocked = grids[0].is_blocked(SPOT);
    assert!(grids.iter().all(|grid| grid.is_blocked(SPOT) == blocked));
    blocked
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

/// An Expand of a Laser Turret at `(x, y)` to [`T1`].
fn expand(x: i32, y: i32) -> Command {
    let command = TunnelCommand::Expand {
        kind: TURRET,
        origin: at(x, y),
        orientation: BuildingOrientation::default(),
    };
    Command::Tunnel { tunnel: T1, command }
}

/// `player`'s Tunnel Space, used and available.
fn tunnel_space(t: &TestSim, player: PlayerId) -> (u32, u32) {
    let cap = t.sim().army_cap(player).expect("a Syndicate player");
    (cap.used, cap.available)
}

fn crystals(t: &TestSim, player: PlayerId) -> u32 {
    t.sim().player(player).expect("in the game").crystals
}

/// A `Progress` of `total` frames with `done` of them done.
fn frames(done: u32, total: u32) -> Progress {
    let mut progress = Progress::new(total);
    for _ in 0..done {
        progress.step(sc_sim::economy::RATIO_ONE);
    }
    progress
}
