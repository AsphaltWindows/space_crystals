//! The Airfield and the Hornet (slice M6-GH1; `factions/gdo/structures/airfield.md`, `factions/gdo/units/hornet.md`,
//! design changes 0366, 0420, 0435 and 0441; plan decisions T88 and D34), through the gate. One test binary:
//! - `training`: the Airfield's Q, a Hornet for 150 Space Crystals and 2 Supplies, both paid as it is queued and
//!   refunded with a cancel; it appears over the Airfield, assigned to it, with its 5 missiles, and flies off with the
//!   order its rally point gives;
//! - `assignment`: at most 4 aircraft an Airfield, its queue counting (`Reject::AirfieldFull`); a right-click on
//!   another Airfield with room reassigns (`OrderKind::Rebase`), on its own one flies home (`OrderKind::ReturnToBase`);
//!   a destroyed Airfield leaves its aircraft unassigned, with no Return to Base (`Reject::NoBase`), and none is
//!   reassigned;
//! - `ammo`: five missiles of 30, each dodgeable (the single-target placeholder rule, Q0182: it hits what covers the
//!   spot it lands on), then no attack (`Reject::OutOfAmmo`) and the Hornet flies home by itself, unless an order
//!   interrupts it; it hovers and turns in place to aim, and fires only once it faces its target;
//! - `braking`: an air HoverCraft brakes by drag alone, a ground hover unit with its thrust (`systems/units.md >
//!   DragMovement`), and the map's edge still stops a drifting Hornet.
//!
//! Auto-targeting is off (`TestSim::without_auto_targeting`), so that only orders fight.

mod ammo;
mod assignment;
mod braking;
mod training;

use sc_sim::aircraft::AircraftState;
use sc_sim::testing::{ScenarioBuilder, TestSim};
use sc_sim::{
    Command, DebugCommand, ObjectId, ObjectKind, OrderKind, PlayerCommand, PlayerId, ProductionCommand, Reject, Sim,
    SimEvent, SpawnAt, Target, Tick, TilePos, Vec2,
};

const P1: PlayerId = PlayerId(1);
const P2: PlayerId = PlayerId(2);
const HORNET: ObjectKind = ObjectKind::Hornet;

/// `factions/gdo/structures/airfield.md > Produces (so far)`: "Hornet: 150 Space Crystals, 320 frames", and its "**+ 2
/// Supplies**" (`factions/gdo/units/hornet.md > Placeholder stats`).
const COST: u32 = 150;
const SUPPLIES: u32 = 2;
const FRAMES: Tick = 320;
/// `factions/gdo/units/hornet.md > AttackAttributes`: "Ammo: **5 missiles**"; "Damage 30".
const AMMO: u32 = 5;
const DAMAGE: f32 = 30.0;

// The objects of [`field`], by id: 1 player 1's Deployment Center on tiles (2, 2) to (5, 5) (Power +30), 2 and 3 its
// Generators on tiles (8, 2) and (8, 5) (+20 each), 4 and 5 its Airfields below (−30 each), 6 player 2's Deployment
// Center, far off on tiles (54, 34) to (57, 37).
/// Player 1's Airfields: A on tiles (12, 3) to (14, 5), B on tiles (12, 12) to (14, 14).
const A: u32 = 4;
const B: u32 = 5;
/// Their centres.
const A_AT: Vec2 = Vec2::new(864.0, 288.0);
const B_AT: Vec2 = Vec2::new(864.0, 864.0);
/// Player 1's bank in [`field`]: plenty of Space Crystals and Supplies.
const BANK: u32 = 5000;
const BANK_SUPPLIES: u32 = 40;

fn id(n: u32) -> ObjectId {
    ObjectId(n)
}

fn flat(w: usize, h: usize) -> String {
    vec![".".repeat(w); h].join("\n")
}

fn tile(x: i32, y: i32) -> SpawnAt {
    SpawnAt::Tile(TilePos::new(x, y))
}

fn at(x: f32, y: f32) -> SpawnAt {
    SpawnAt::Point(Vec2::new(x, y))
}

/// Both players play the GDO on a flat map of 60 × 40 tiles, player 1 with two Airfields, [`A`] and [`B`], and the
/// power for both, [`BANK`] Space Crystals and [`BANK_SUPPLIES`] Supplies from tick 1, and `more` after them (from id
/// 7); auto-targeting off; tick 1 has run.
fn field(more: impl FnOnce(ScenarioBuilder) -> ScenarioBuilder) -> TestSim {
    field_with(BANK_SUPPLIES, more)
}

/// [`field`] with `supplies` Supplies.
fn field_with(supplies: u32, more: impl FnOnce(ScenarioBuilder) -> ScenarioBuilder) -> TestSim {
    let builder = ScenarioBuilder::new(&flat(60, 40))
        .spawn(ObjectKind::DeploymentCenter, Some(P1), tile(2, 2))
        .spawn(ObjectKind::Generator, Some(P1), tile(8, 2))
        .spawn(ObjectKind::Generator, Some(P1), tile(8, 5))
        .spawn(ObjectKind::Airfield, Some(P1), tile(12, 3))
        .spawn(ObjectKind::Airfield, Some(P1), tile(12, 12))
        .spawn(ObjectKind::DeploymentCenter, Some(P2), tile(54, 34));
    let mut t = more(builder).start();
    t.without_auto_targeting();
    bank(&mut t, 1, BANK, supplies);
    t.run_until(1);
    assert_eq!(object(t.sim(), A).pos, A_AT);
    assert_eq!(object(t.sim(), B).pos, B_AT);
    t
}

/// Player 1's bank from tick `tick` on: `crystals` Space Crystals and `supplies` Supplies.
fn bank(t: &mut TestSim, tick: Tick, crystals: u32, supplies: u32) {
    t.debug_at(
        tick,
        DebugCommand::SetResources {
            player: P1,
            crystals,
            supplies,
            alloys: 0,
            essence: 0,
            flux: 0,
        },
    );
}

/// What the gate says to `player`'s `command` now (`Sim::validate`).
fn gate(sim: &Sim, player: PlayerId, command: &Command) -> Result<(), Reject> {
    sim.validate(&PlayerCommand {
        player,
        command: command.clone(),
    })
}

/// `player`'s `command` in the next tick, which runs.
fn next(t: &mut TestSim, player: PlayerId, command: Command) {
    let tick = t.sim().tick() + 1;
    t.at(tick, player, command);
    t.step();
}

/// The debug `command` in the next tick, which runs.
fn debug(t: &mut TestSim, command: DebugCommand) {
    let tick = t.sim().tick() + 1;
    t.debug_at(tick, command);
    t.step();
}

fn order(units: &[u32], kind: OrderKind) -> Command {
    Command::Order {
        units: units.iter().copied().map(id).collect(),
        kind,
        queue: false,
    }
}

/// The right-click of `units` on `target` (`Sim::default_order`, the first unit's, as the client sends it).
fn right_click(sim: &Sim, units: &[u32], target: Target) -> Command {
    let kind = sim.default_order(id(units[0]), target).expect("a default order");
    order(units, kind)
}

/// Q at `airfield`: a Train of a Hornet.
fn train(airfield: u32) -> Command {
    Command::Production {
        objects: vec![id(airfield)],
        command: ProductionCommand::Train(HORNET),
    }
}

/// X at `airfield`: its Cancel Production.
fn cancel(airfield: u32) -> Command {
    Command::Production {
        objects: vec![id(airfield)],
        command: ProductionCommand::CancelProduction,
    }
}

fn object(sim: &Sim, n: u32) -> &sc_sim::Object {
    sim.object(id(n)).unwrap_or_else(|| panic!("#{n} is on the map"))
}

/// Aircraft `n`'s ammo and base.
fn aircraft(sim: &Sim, n: u32) -> AircraftState {
    object(sim, n).aircraft.expect("an aircraft")
}

/// Aircraft `n`'s base, by number.
fn base(sim: &Sim, n: u32) -> Option<u32> {
    aircraft(sim, n).base.map(|b| b.0)
}

/// The kind of what unit `n` is doing: its current order's, else its first queued one's.
fn doing(sim: &Sim, n: u32) -> Option<OrderKind> {
    let orders = object(sim, n).orders.as_ref().expect("orders");
    (orders.current.as_ref().map(|active| &active.order))
        .or(orders.queue.front())
        .map(|order| order.kind)
}

fn crystals(sim: &Sim) -> u32 {
    sim.player(P1).expect("in the game").crystals
}

fn supplies(sim: &Sim) -> u32 {
    sim.player(P1).expect("in the game").supplies
}

fn queued(sim: &Sim, producer: u32) -> Vec<ObjectKind> {
    let production = object(sim, producer).production.as_ref().expect("a producer");
    production.queue.iter().filter_map(|entry| entry.unit()).collect()
}

/// The units produced so far: (tick, producer, unit).
fn produced(t: &TestSim) -> Vec<(Tick, ObjectId, ObjectId)> {
    let unit = |(tick, e): &(Tick, SimEvent)| match *e {
        SimEvent::UnitProduced { producer, unit } => Some((*tick, producer, unit)),
        _ => None,
    };
    t.events().iter().filter_map(unit).collect()
}

/// The missiles launched so far: (tick, shooter, target, from, to, lands_at).
fn launches(t: &TestSim) -> Vec<(Tick, ObjectId, Option<ObjectId>, Vec2, Tick)> {
    let launch = |(tick, e): &(Tick, SimEvent)| match *e {
        SimEvent::ProjectileLaunched {
            shooter,
            target,
            to,
            lands_at,
            ..
        } => Some((*tick, shooter, target, to, lands_at)),
        _ => None,
    };
    t.events().iter().filter_map(launch).collect()
}

/// The hits of `attacker` so far: (tick, target, HP damage).
fn hits_of(t: &TestSim, attacker: u32) -> Vec<(Tick, ObjectId, f32)> {
    let hit = |(tick, e): &(Tick, SimEvent)| match *e {
        SimEvent::Hit {
            attacker: by,
            target,
            damage,
            ..
        } if by == id(attacker) => Some((*tick, target, damage)),
        _ => None,
    };
    t.events().iter().filter_map(hit).collect()
}

/// The blasts of `attacker`'s shots so far: (tick, at, radius).
fn blasts_of(t: &TestSim, attacker: u32) -> Vec<(Tick, Vec2, f32)> {
    let blast = |(tick, e): &(Tick, SimEvent)| match *e {
        SimEvent::Blast {
            at,
            radius,
            attacker: Some(by),
        } if by == id(attacker) => Some((*tick, at, radius)),
        _ => None,
    };
    t.events().iter().filter_map(blast).collect()
}

/// Steps until `done` holds, at most `ticks` ticks.
fn step_until(t: &mut TestSim, ticks: Tick, what: &str, done: impl Fn(&Sim) -> bool) {
    let until = t.sim().tick() + ticks;
    while !done(t.sim()) {
        assert!(t.sim().tick() < until, "{what}: not by tick {until}");
        t.step();
    }
}

/// `n` Hornets of player 1 spawned in the next tick at `points`, ids from `first` on, and assigned by a right-click on
/// `airfield` in the tick after (`OrderKind::Rebase`).
fn based_hornets(t: &mut TestSim, first: u32, points: &[Vec2], airfield: u32) -> Vec<u32> {
    let ids: Vec<u32> = (first..first + points.len() as u32).collect();
    let tick = t.sim().tick() + 1;
    for &pos in points {
        t.spawn_unit_at(tick, HORNET, P1, pos);
    }
    t.step();
    for &n in &ids {
        assert_eq!(object(t.sim(), n).kind, HORNET);
        assert_eq!(
            aircraft(t.sim(), n),
            AircraftState { ammo: AMMO, base: None },
            "spawned unassigned"
        );
    }
    let click = right_click(t.sim(), &ids, Target::Object(id(airfield)));
    assert_eq!(click, order(&ids, OrderKind::Rebase(id(airfield))));
    next(t, P1, click);
    for &n in &ids {
        assert_eq!(base(t.sim(), n), Some(airfield));
    }
    ids
}
