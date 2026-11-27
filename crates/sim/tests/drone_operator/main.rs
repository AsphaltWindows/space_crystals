//! The Drone Operator and its drone (slice M6-GD; `factions/gdo/units/drone_operator.md`, design changes 0366, 0421,
//! 0435 and 0441; plan decisions T89 and T91), through the gate. One test binary:
//! - `training`: the Barracks' R, behind a Career Campus, for 100 Space Crystals and 1 Supply, both paid as it is
//!   queued and refunded with a cancel and with the Barracks; refused without the Supply, a Peacekeeper still needing
//!   none; the operator comes out with its one drone;
//! - `stock`: Build Drone (F) refused while the stock is full, while the drone is out and while a purchase runs; once
//!   the drone has ended it buys one for 50 Space Crystals in 160 frames, and the stock never passes 1;
//! - `launch`: Launch Drone (R) puts the drone over the operator, which refuses every order that would move it
//!   (`Reject::DroneOut`) and holds still until the drone is gone; a queued Launch waits its turn;
//! - `explosion`: the drone's attack on a Peacekeeper group, 150 over 48 su by the area rule, own units included, and
//!   then the drone is gone and the operator free; the domains: the drone attacks no air unit, and Rocketmen shoot it
//!   down where Peacekeepers can't;
//! - `ends`: ordered 9 grid units out, it falls and explodes at the leash's edge; its battery runs out 320 ticks after
//!   the launch, and its operator's death drops it, both without an explosion.
//!
//! Auto-targeting is off in the drone's tests (`TestSim::without_auto_targeting`), so that only orders fight.

mod ends;
mod explosion;
mod launch;
mod stock;
mod training;

use sc_sim::stock::Stock;
use sc_sim::testing::{ScenarioBuilder, TestSim};
use sc_sim::tether::Fall;
use sc_sim::{
    Ability, Command, DebugCommand, ObjectId, ObjectKind, OrderKind, PlayerCommand, PlayerId, Reject, SU_PER_GU, Sim,
    SimEvent, SpawnAt, Target, Tick, Vec2,
};

const P1: PlayerId = PlayerId(1);
const P2: PlayerId = PlayerId(2);
const OPERATOR: ObjectKind = ObjectKind::DroneOperator;
const DRONE: ObjectKind = ObjectKind::OperatorDrone;
const PK: ObjectKind = ObjectKind::Peacekeeper;

/// `factions/gdo/units/drone_operator.md > Placeholder stats`: "**leash range 8 grid units**, measured **edge to
/// edge** like ranges", space units.
const LEASH: f32 = 8.0 * SU_PER_GU;
/// "**battery 20 s** (320 frames)".
const BATTERY: Tick = 320;
/// "**Build Drone** (hotkey **F**): 50 Space Crystals, 160 frames (10 s)".
const DRONE_PRICE: u32 = 50;
const DRONE_FRAMES: Tick = 160;
/// "Explosion: **AoE** (user, 2026-11-27), Damage **150** total, AoERadius **48 su**".
const EXPLOSION: f32 = 150.0;
const RADIUS: f32 = 48.0;

/// Where [`field`]'s operator stands: the centre of tile (10, 15).
const HOME: Vec2 = Vec2::new(672.0, 992.0);

fn id(n: u32) -> ObjectId {
    ObjectId(n)
}

fn flat(w: usize, h: usize) -> String {
    vec![".".repeat(w); h].join("\n")
}

/// `HOME` and `dx`, `dy` space units.
fn from_home(dx: f32, dy: f32) -> Vec2 {
    HOME + Vec2::new(dx, dy)
}

/// Player 1's Drone Operator (1) at [`HOME`] on a flat map of 40 × 30 tiles, and `more` after it, with 1000 Space
/// Crystals; auto-targeting off. Tick 1 has run.
fn field(more: impl FnOnce(ScenarioBuilder) -> ScenarioBuilder) -> TestSim {
    let builder = ScenarioBuilder::new(&flat(40, 30)).spawn(OPERATOR, Some(P1), SpawnAt::Point(HOME));
    let mut t = more(builder).start();
    t.without_auto_targeting();
    bank(&mut t, 1, 1000, 0);
    t.run_until(1);
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
fn gate(sim: &Sim, player: PlayerId, command: Command) -> Result<(), Reject> {
    sim.validate(&PlayerCommand { player, command })
}

/// `player`'s `command` in the next tick, which runs.
fn next(t: &mut TestSim, player: PlayerId, command: Command) {
    let tick = t.sim().tick() + 1;
    t.at(tick, player, command);
    t.step();
}

fn order(units: &[u32], kind: OrderKind) -> Command {
    Command::Order {
        units: units.iter().copied().map(id).collect(),
        kind,
        queue: false,
    }
}

/// Launch Drone (R).
fn launch(units: &[u32]) -> Command {
    order(units, OrderKind::Launch)
}

/// Build Drone (F).
fn buy(units: &[u32]) -> Command {
    Command::Ability {
        units: units.iter().copied().map(id).collect(),
        ability: Ability::Buy,
    }
}

fn move_to(units: &[u32], to: Vec2) -> Command {
    order(units, OrderKind::Move(Target::Point(to)))
}

fn object(sim: &Sim, n: u32) -> &sc_sim::Object {
    sim.object(id(n)).unwrap_or_else(|| panic!("#{n} is on the map"))
}

fn stock(sim: &Sim, n: u32) -> Stock {
    object(sim, n).stock.expect("a stock")
}

fn crystals(sim: &Sim) -> u32 {
    sim.player(P1).expect("in the game").crystals
}

fn supplies(sim: &Sim) -> u32 {
    sim.player(P1).expect("in the game").supplies
}

/// The launches so far: (tick, operator, drone).
fn launches(t: &TestSim) -> Vec<(Tick, ObjectId, ObjectId)> {
    let launch = |(tick, e): &(Tick, SimEvent)| match *e {
        SimEvent::DroneLaunched { operator, drone } => Some((*tick, operator, drone)),
        _ => None,
    };
    t.events().iter().filter_map(launch).collect()
}

/// The drones that fell so far: (tick, drone, why).
fn falls(t: &TestSim) -> Vec<(Tick, ObjectId, Fall)> {
    let fall = |(tick, e): &(Tick, SimEvent)| match *e {
        SimEvent::DroneFell { drone, why, .. } => Some((*tick, drone, why)),
        _ => None,
    };
    t.events().iter().filter_map(fall).collect()
}

/// The blasts so far: (tick, centre, radius, attacker).
fn blasts(t: &TestSim) -> Vec<(Tick, Vec2, f32, Option<ObjectId>)> {
    let blast = |(tick, e): &(Tick, SimEvent)| match *e {
        SimEvent::Blast { at, radius, attacker } => Some((*tick, at, radius, attacker)),
        _ => None,
    };
    t.events().iter().filter_map(blast).collect()
}

/// The hits on `n` so far: (tick, attacker, HP damage).
fn hits_on(t: &TestSim, n: u32) -> Vec<(Tick, ObjectId, f32)> {
    let hit = |(tick, e): &(Tick, SimEvent)| match *e {
        SimEvent::Hit {
            attacker,
            target,
            damage,
            ..
        } if target == id(n) => Some((*tick, attacker, damage)),
        _ => None,
    };
    t.events().iter().filter_map(hit).collect()
}

/// The tick `n` died in, if it did.
fn died(t: &TestSim, n: u32) -> Option<Tick> {
    let death =
        |(tick, e): &(Tick, SimEvent)| matches!(*e, SimEvent::Died { id: dead, .. } if dead == id(n)).then_some(*tick);
    t.events().iter().find_map(death)
}

/// What the area rule (`systems/combat.md > AoEDamage`; `sc_sim::combat::overlap`) takes from a 24 × 24 Peacekeeper
/// (FullArmor 1, LightInfantry: no DirectionalArmor) at `pos` for the drone's explosion at `centre`: Damage times the
/// part of the circle its Silhouette covers, less its share of its FullArmor (`systems/combat.md >
/// DamageModifierOrder`), at least its share of the minimum damage; 0 outside the circle.
fn explosion_share(centre: Vec2, pos: Vec2) -> f32 {
    let unit = PK.def().unit.as_ref().expect("a unit");
    assert!(!unit.base_def().directional_armor, "infantry");
    let half = Vec2::splat(12.0);
    let inside = sc_sim::combat::overlap(centre, RADIUS, pos - half, pos + half);
    if inside <= 0.0 {
        return 0.0;
    }
    let fraction = inside / (24.0 * 24.0);
    let share = EXPLOSION * inside / (std::f32::consts::PI * RADIUS * RADIUS);
    (share - fraction).max(0.5 * fraction)
}
