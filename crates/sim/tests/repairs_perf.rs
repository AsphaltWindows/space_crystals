//! Timings of whole ticks with 40 Recruits repairing 10 buildings (slice M5-N8: "a perf run with 40 repairers on 10
//! targets"), phase by phase (`Sim::step_observed`). Ignored by default:
//!
//! ```sh
//! cargo test -p sc_sim --release --test perf repairs_perf -- --ignored --nocapture --test-threads=1
//! ```
//!
//! On a flat 64 × 40 map player 1 has ten Armories in two rows of five, 3 tiles apart, and 40 Recruits in two blocks,
//! north and south of the rows, four dealt to each Armory: those of the north row from the north block, the others
//! from the south. Every Armory starts at 1 HP; each time all ten are back to full, the game is saved with them at 1
//! HP again and loaded (damage, which no command deals on demand), and the Recruits are ordered to repair them again:
//! [`ROUNDS`] rounds, the first with the walk there. The Recruits pay as they go (`sc_sim::repair`), from a bank that
//! never runs out. The ticks are timed, never the saves.
//!
//! It prints, for the whole tick and for each phase, the mean, the 99th percentile and the worst tick, flagging a
//! worst tick over 10 ms (slice M2-J2's target for a whole tick). The assertions are on the game, never on wall time.
//! New in M5-N8, so its figures are absolute ones: compare them by A/B runs.

use std::time::Duration;

use sc_sim::testing::{ScenarioBuilder, TestSim};
use sc_sim::{
    Command, DebugCommand, Faction, ObjectId, ObjectKind, OrderKind, Phase, PlayerCommand, PlayerId, Sim, SpawnAt,
    Tick, TickInputs, TilePos, Vec2,
};

const WIDTH: usize = 64;
const HEIGHT: usize = 40;
const P1: PlayerId = PlayerId(1);
/// Slice M2-J2's target for a whole tick, at worst.
const BUDGET: Duration = Duration::from_millis(10);
/// The phases that take time ([`Phase::End`] only marks the end).
const PHASES: usize = Phase::ALL.len() - 1;
const TARGETS: usize = 10;
const PER_TARGET: usize = 4;
const ROUNDS: u32 = 12;
/// The Armories' rows' tile rows, and their columns: 3 × 2 footprints 3 tiles apart.
const ROWS: [i32; 2] = [16, 21];
const COLUMNS: [i32; 5] = [17, 23, 29, 35, 41];
/// The longest round, ticks: the first, with its walk, takes a few hundred.
const LONGEST_ROUND: u32 = 1000;

/// The wall-clock time of each phase of one tick of `sim` with player 1's `commands` (`Sim::step_observed`), in
/// [`Phase`]'s order. Only perf tests read a clock: they measure the sim from outside, and nothing in the sim can call
/// it (`ARCHITECTURE.md > Determinism`).
#[expect(
    clippy::disallowed_methods,
    reason = "a perf test has to read the clock; the sim itself never does"
)]
fn timed_step(sim: &mut Sim, commands: Vec<Command>) -> [Duration; PHASES] {
    let commands = commands
        .into_iter()
        .map(|command| PlayerCommand { player: P1, command })
        .collect();
    let mut marks = Vec::with_capacity(Phase::ALL.len());
    sim.step_observed(&TickInputs { commands }, |phase| {
        marks.push((phase, std::time::Instant::now()))
    });
    let mut times = [Duration::ZERO; PHASES];
    for pair in marks.windows(2) {
        let (phase, start) = pair[0];
        times[phase as usize] += pair[1].1 - start;
    }
    times
}

/// The game, its tick 1 run (the bank filled), the Armories' ids and the Recruits dealt to each.
fn game() -> (Sim, Vec<(ObjectId, Vec<ObjectId>)>) {
    let map = vec![".".repeat(WIDTH); HEIGHT].join("\n");
    let mut builder = ScenarioBuilder::new(&map).faction(P1, Faction::Cults);
    let mut deals = Vec::new();
    let mut next = 1;
    for y in ROWS {
        for x in COLUMNS {
            let at = SpawnAt::Tile(TilePos::new(x, y));
            builder = builder.spawn(ObjectKind::Armory, Some(P1), at);
            deals.push((ObjectId(next), Vec::new()));
            next += 1;
        }
    }
    // Two blocks of 20, 10 wide and 40 space units apart, 3 tiles north of the north row and south of the south one.
    for i in 0..TARGETS * PER_TARGET {
        let (block, place) = (i % 2, i / 2);
        let (column, line) = (place % 10, place / 10);
        let corner = TilePos::new(COLUMNS[0], if block == 0 { ROWS[0] - 3 } else { ROWS[1] + 5 }).corner();
        let away = if block == 0 { -1.0 } else { 1.0 };
        let at = corner + Vec2::new(40.0 + 40.0 * column as f32, away * 40.0 * line as f32);
        builder = builder.spawn(ObjectKind::Recruit, Some(P1), SpawnAt::Point(at));
        deals[block * COLUMNS.len() + place % COLUMNS.len()]
            .1
            .push(ObjectId(next));
        next += 1;
    }
    let scenario = builder.build();
    assert_eq!(scenario.validate(), vec![]);
    let mut t = TestSim::new(scenario);
    t.check_invariants(false);
    let bank = DebugCommand::SetResources {
        player: P1,
        crystals: 1_000_000,
        supplies: 0,
        alloys: 0,
        essence: 0,
        flux: 0,
    };
    t.debug_at(1, bank);
    t.run(1);
    assert!(deals.iter().all(|(_, recruits)| recruits.len() == PER_TARGET));
    (t.sim().clone(), deals)
}

/// `sim` saved, with every one of `targets` at 1 HP, and loaded.
fn damaged(sim: &Sim, targets: &[ObjectId]) -> Sim {
    let mut state = serde_json::to_value(sim).unwrap();
    let objects = state["objects"].as_array_mut().unwrap();
    for o in objects.iter_mut().filter(|o| targets.iter().any(|t| o["id"] == t.0)) {
        o["hp"] = serde_json::json!(1.0);
    }
    serde_json::from_value(state).unwrap()
}

fn full(sim: &Sim, id: ObjectId) -> bool {
    let o = sim.object(id).expect("standing");
    o.hp == o.max_hp()
}

/// One line: the mean, the 99th percentile and the worst tick of `times`, flagged over [`BUDGET`].
fn line(label: &str, times: &[(Tick, Duration)]) -> String {
    let ms = |d: Duration| d.as_secs_f64() * 1000.0;
    let mean = times.iter().map(|(_, d)| ms(*d)).sum::<f64>() / times.len().max(1) as f64;
    let mut sorted: Vec<Duration> = times.iter().map(|(_, d)| *d).collect();
    sorted.sort();
    let p99 = sorted.get(sorted.len() * 99 / 100).copied().unwrap_or_default();
    let (tick, worst) = times.iter().copied().max_by_key(|(_, d)| *d).unwrap_or_default();
    let flag = if worst > BUDGET { "  OVER 10 ms" } else { "" };
    format!(
        "  {label:<34} mean {:>7.3} ms  p99 {:>7.3} ms  worst {:>7.3} ms (tick {tick}){flag}",
        mean,
        ms(p99),
        ms(worst)
    )
}

#[test]
#[ignore = "timing: run with --release --ignored --nocapture"]
fn forty_recruits_repairing_ten_armories() {
    let (mut sim, deals) = game();
    let targets: Vec<ObjectId> = deals.iter().map(|(target, _)| *target).collect();
    let bank = sim.player(P1).expect("in the game").crystals;
    let mut ticks: Vec<(Tick, [Duration; PHASES])> = Vec::new();
    let mut rounds = Vec::new();
    for _ in 0..ROUNDS {
        sim = damaged(&sim, &targets);
        let orders: Vec<Command> = (deals.iter())
            .map(|(target, recruits)| Command::Order {
                units: recruits.clone(),
                kind: OrderKind::Repair(*target),
                queue: false,
            })
            .collect();
        let start = sim.tick();
        ticks.push((start + 1, timed_step(&mut sim, orders)));
        while !targets.iter().all(|&target| full(&sim, target)) {
            assert!(sim.tick() - start < LONGEST_ROUND, "the round ends");
            ticks.push((sim.tick() + 1, timed_step(&mut sim, Vec::new())));
        }
        rounds.push(sim.tick() - start);
    }
    let of = |phases: &[Phase]| -> Vec<(Tick, Duration)> {
        (ticks.iter())
            .map(|(tick, times)| (*tick, phases.iter().map(|&p| times[p as usize]).sum()))
            .collect()
    };
    let whole = &Phase::ALL[..PHASES];
    println!(
        "{} Recruits repairing {TARGETS} Armories from 1 HP, {ROUNDS} rounds: {} ticks ({rounds:?} each)",
        TARGETS * PER_TARGET,
        ticks.len()
    );
    println!("{}", line("whole tick", &of(whole)));
    for &phase in whole {
        println!("{}", line(&format!("{phase:?}"), &of(&[phase])));
    }

    // The game: every round restores the ten Armories' 299 HP each, at 100 crystals for 0 to full, the crystal of 1
    // HP paid before the round; and every Recruit is idle at the end.
    let paid = bank - sim.player(P1).expect("in the game").crystals;
    println!("{paid} Space Crystals paid");
    assert_eq!(paid, ROUNDS * TARGETS as u32 * 99);
    // Four at work on each, 5 HP a tick: 60 ticks, and a few to take their spots again.
    assert!(rounds[1..].iter().all(|&round| round <= 75), "{rounds:?}");
    let idle = (deals.iter().flat_map(|(_, recruits)| recruits)).all(|id| {
        sim.object(*id)
            .and_then(|o| o.orders.as_ref())
            .is_some_and(|orders| orders.current.is_none())
    });
    assert!(idle, "every Repair ended at full HP");
}
