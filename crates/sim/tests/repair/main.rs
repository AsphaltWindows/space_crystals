//! Repair (slice M5-N8, plan decision T83; `factions/cults/units/recruit.md > Repair`,
//! `factions/syndicate/units/agent.md > Repair`; design changes 0321, 0325, 0328, 0329 and 0360) and the Tunnel
//! expansions' regeneration (`factions/syndicate/structures/README.md > Tunnel Expansions`), through the gate. No
//! command deals damage on demand: a test damages its target in a state dump of the game ([`damaged`]) and plays on
//! from it, a tick at a time ([`step`]). One test binary:
//! - `rates`: one to four Recruits repair an Armory from 150 to 300 HP in 120, 60, 40 and 30 frames, a crystal for
//!   every 3 HP, 50 in all, and stop at full HP (design change 0321);
//! - `bank`: the bank running out ends the Repairs, those of the Recruits on their way too, the bank at 0 and the HP
//!   where its last crystal took it, and the queued orders go on; a Repair given with no crystals ends at work,
//!   restoring nothing (design change 0329);
//! - `agents`: an Agent repairs its Tunnel for free at its construction's rate, at Tier 1 and Tier 2 (design change
//!   0328), and a Laser Turret from 150 to 300 HP in 80 frames for 25 crystals, lowered or raised, the repair never
//!   raising it; a turret whose Tunnel is lost, neutral, no more (design change 0360);
//! - `refusals`: the gate's: a building at full HP, a site, an enemy's building, infantry, a unit that repairs nothing,
//!   an expansion; of a group, each unit that repairs the target takes the order, the others keep theirs;
//! - `regeneration`: a damaged Hideout back to full at 1% of its MaxHP a second, under fire too, and not once its
//!   Tunnel is gone; the Laser Turret, which is repaired, regenerates nothing.
//!
//! - `vehicles` (slice M5-N6): one Recruit repairs a Seeker from 60 to 120 HP in 80 frames and two in 40, 19 Space
//!   Crystals either way, by the Chop Shop's recipe.
//!
//! The repairers following a vehicle as it drives off, and a glider's refusal by its movement, are the crate's own
//! tests (`sim/repairs/tests.rs`).

mod agents;
mod bank;
mod rates;
mod refusals;
mod regeneration;
mod vehicles;

use sc_sim::testing::{ScenarioBuilder, TestSim};
use sc_sim::{
    Command, DebugCommand, Faction, ObjectId, ObjectKind, OrderKind, PlayerCommand, PlayerId, Reject, Sim, SpawnAt,
    Target, TickInputs, TilePos, Vec2,
};

const P1: PlayerId = PlayerId(1);
const P2: PlayerId = PlayerId(2);
/// The longest walk to a target in these tests, ticks.
const WALK: u32 = 400;
/// The longest repair or regeneration in these tests, ticks.
const LONGEST: u32 = 2000;

// The objects of [`cults`], by id.
/// Player 1's Storage on tiles (2, 2) to (4, 3), and their Armory on (10, 10) to (12, 11): x 640..832, y 640..768.
const STORAGE: u32 = 1;
const ARMORY: u32 = 2;
/// Player 1's four Recruits, south of the Armory, a fifth far off to the south-west, and a Militant.
const RECRUITS: [u32; 4] = [3, 4, 5, 6];
const FAR_RECRUIT: u32 = 7;
const MILITANT: u32 = 8;
/// Player 2's Armory, on (16, 12) to (18, 13): in the Recruits' sight.
const ENEMY_ARMORY: u32 = 9;
/// Player 1's bank in [`cults`], Space Crystals.
const BANK: u32 = 1000;
/// What a Recruit at work on an Armory restores in a tick: its 300 HP in half its 480 frames (design change 0321:
/// "two Recruits repair an Armory from 150 to 300 HP in 60 frames").
const ARMORY_RATE: f32 = 1.25;

// The objects of [`syndicate`], by id.
/// Player 1's Tunnel on tiles (4, 8) to (7, 11), its Tier 1 area (2, 6) to (9, 13); its Hideout on (8, 8) to
/// (9, 9) and its Laser Turret on (9, 11), lowered, both in the area.
const T1: u32 = 1;
const HIDEOUT: u32 = 2;
const TURRET: u32 = 3;
/// Player 1's second Tunnel, far east, which keeps them in the game without [`T1`].
const OTHER_TUNNEL: u32 = 4;
/// Player 1's Agent, south of the turret.
const AGENT: u32 = 5;
/// The turret's tile.
const TURRET_AT: TilePos = TilePos::new(9, 11);

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

/// Both players play the Cults on a flat 40 × 30 map, with [`STORAGE`] to [`ENEMY_ARMORY`]: player 1's bank is
/// [`BANK`] Space Crystals and nobody fights by themselves (auto-targeting off), from tick 1, which has run.
fn cults() -> TestSim {
    let mut scenario = ScenarioBuilder::new(&flat(40, 30))
        .faction(P1, Faction::Cults)
        .faction(P2, Faction::Cults)
        .spawn(ObjectKind::Storage, Some(P1), tile(2, 2))
        .spawn(ObjectKind::Armory, Some(P1), tile(10, 10));
    for x in [660.0, 710.0, 760.0, 810.0] {
        scenario = scenario.spawn(ObjectKind::Recruit, Some(P1), point(x, 800.0));
    }
    let mut t = scenario
        .spawn(ObjectKind::Recruit, Some(P1), point(300.0, 1600.0))
        .spawn(ObjectKind::Militant, Some(P1), point(900.0, 800.0))
        .spawn(ObjectKind::Armory, Some(P2), tile(16, 12))
        .start();
    t.debug_at(1, banks(BANK, 0));
    t.debug_at(1, DebugCommand::SetAutoTargeting { on: false });
    t.run(1);
    t
}

/// Both players play the Syndicate on a flat 48 × 24 map, with [`T1`] to [`AGENT`], and player 2's Tunnel far off:
/// player 1's banks are 100 Space Crystals and 5 Supplies from tick 1, which has run.
fn syndicate() -> TestSim {
    let mut t = ScenarioBuilder::new(&flat(48, 24))
        .faction(P1, Faction::Syndicate)
        .faction(P2, Faction::Syndicate)
        .spawn(ObjectKind::Tunnel, Some(P1), tile(4, 8))
        .spawn(ObjectKind::Hideout, Some(P1), tile(8, 8))
        .spawn(ObjectKind::LaserTurret, Some(P1), SpawnAt::Tile(TURRET_AT))
        .spawn(ObjectKind::Tunnel, Some(P1), tile(30, 4))
        .spawn(ObjectKind::Agent, Some(P1), point(608.0, 840.0))
        .spawn(ObjectKind::Tunnel, Some(P2), tile(40, 16))
        .start();
    t.debug_at(1, banks(100, 5));
    t.run(1);
    t
}

/// Sets player 1's Space Crystals and Supplies, their other banks to 0.
fn banks(crystals: u32, supplies: u32) -> DebugCommand {
    DebugCommand::SetResources {
        player: P1,
        crystals,
        supplies,
        alloys: 0,
        essence: 0,
        flux: 0,
    }
}

/// `sim` saved, with object `n`'s HP set to `hp`, and loaded: damage that no command deals on demand.
fn damaged(sim: &Sim, n: u32, hp: f32) -> Sim {
    let mut state = serde_json::to_value(sim).unwrap();
    let objects = state["objects"].as_array_mut().unwrap();
    let o = objects.iter_mut().find(|o| o["id"] == n).expect("the object is saved");
    o["hp"] = serde_json::json!(hp);
    serde_json::from_value(state).unwrap()
}

/// One tick of `sim` with player 1's `commands`.
fn step(sim: &mut Sim, commands: Vec<Command>) {
    let commands = commands
        .into_iter()
        .map(|command| PlayerCommand { player: P1, command })
        .collect();
    sim.step(&TickInputs { commands });
}

/// `ticks` ticks of `sim` with no commands.
fn run(sim: &mut Sim, ticks: u32) {
    for _ in 0..ticks {
        step(sim, Vec::new());
    }
}

fn hp(sim: &Sim, n: u32) -> f32 {
    let o = sim.object(id(n)).unwrap_or_else(|| panic!("{n} is on the map"));
    o.hp.expect("it has HP")
}

/// Player 1's Space Crystals.
fn crystals(sim: &Sim) -> u32 {
    sim.player(P1).expect("in the game").crystals
}

/// Unit `n`'s current order, if any.
fn current(sim: &Sim, n: u32) -> Option<OrderKind> {
    let orders = sim.object(id(n))?.orders.as_ref()?;
    orders.current.as_ref().map(|active| active.order.kind)
}

/// Whether unit `n`'s current order is a Repair.
fn repairing(sim: &Sim, n: u32) -> bool {
    matches!(current(sim, n), Some(OrderKind::Repair(_)))
}

fn order(units: &[u32], kind: OrderKind, queue: bool) -> Command {
    Command::Order {
        units: units.iter().map(|&n| id(n)).collect(),
        kind,
        queue,
    }
}

/// A Repair of `target` to `units`, without Shift.
fn repair(units: &[u32], target: u32) -> Command {
    order(units, OrderKind::Repair(id(target)), false)
}

/// What the gate says to player 1's `command` now (`Sim::validate`).
fn gate(sim: &Sim, command: Command) -> Result<(), Reject> {
    sim.validate(&PlayerCommand { player: P1, command })
}

/// Whether `gained` is `rate`, give or take the rounding of an HP in the hundreds.
fn about(gained: f32, rate: f32) -> bool {
    (gained - rate).abs() < 1e-3
}

/// Runs `sim` until object `n` gains `rate` HP in a tick: every one of its repairers at work on it. `each` sees every
/// tick. Panics after [`WALK`] ticks.
fn until_gaining(sim: &mut Sim, n: u32, rate: f32, mut each: impl FnMut(&Sim)) {
    for _ in 0..WALK {
        let before = hp(sim, n);
        step(sim, Vec::new());
        each(sim);
        if about(hp(sim, n) - before, rate) {
            return;
        }
    }
    panic!("{n}'s repairers never all got to work: {}", hp(sim, n));
}

/// Runs `sim` until object `n` is at `max` HP, checking that it gains `rate` HP in every tick, its last up to `max`,
/// and `each` after every tick; returns the ticks it took. Panics after [`LONGEST`] ticks.
fn restored_to(sim: &mut Sim, n: u32, max: f32, rate: f32, mut each: impl FnMut(&Sim)) -> u32 {
    for ticks in 1..=LONGEST {
        let before = hp(sim, n);
        step(sim, Vec::new());
        each(sim);
        let now = hp(sim, n);
        assert!(now <= max, "never above its MaxHP: {now}");
        if now == max {
            assert!(now - before <= rate + 1e-3, "its last tick: {before} to {now}");
            return ticks;
        }
        assert!(about(now - before, rate), "tick {ticks}: {before} to {now}");
    }
    panic!("{n} never got back to {max}: {}", hp(sim, n));
}

/// The Armory of [`cults`] nearly destroyed and the first `n` [`RECRUITS`] ordered to repair it, run until all of them
/// are at work on it; then its HP set to `hp`.
fn at_work(n: usize, hp: f32) -> Sim {
    let mut sim = damaged(cults().sim(), ARMORY, 1.0);
    step(&mut sim, vec![repair(&RECRUITS[..n], ARMORY)]);
    until_gaining(&mut sim, ARMORY, ARMORY_RATE * n as f32, |_| {});
    damaged(&sim, ARMORY, hp)
}
