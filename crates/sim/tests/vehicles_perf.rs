//! Timings of whole ticks with thirty vehicles whose Silhouettes turn with their facing (slice M5-V1, plan decision
//! T78: "a new `vehicles_perf` (30 oriented units in a crowd)"), phase by phase (`Sim::step_observed`). Ignored by
//! default:
//!
//! ```sh
//! cargo test -p sc_sim --release --test perf vehicles_perf -- --ignored --nocapture --test-threads=1
//! ```
//!
//! On a flat 48 × 32 map player 1's fifteen Seekers (56 × 40 su) and player 2's fifteen Hands of Heaven (64 × 40 su),
//! each group in a block, are sent through each other: each group to the other's side, one group order each, so that
//! the two crowds meet head-on in the middle and every vehicle turns, and is turned back, as it goes round the others
//! (`movement::collide::turn`, the separating-axis tests of `shape`). Auto-targeting is off. The run is 600 ticks,
//! 37.5 s; every invariant is checked after each tick, outside the timing.
//!
//! It prints, for the whole tick and for the movement phase (5d), the mean, the 99th percentile and the worst tick,
//! flagging a worst tick over 10 ms (slice M2-J2's target for a whole tick). The assertions are on the game, never on
//! wall time. New in M5-V1, so its figures are absolute ones: compare them by A/B runs.

use std::time::Duration;

use sc_sim::testing::ScenarioBuilder;
use sc_sim::testing::invariants::Invariants;
use sc_sim::{
    Command, DebugCommand, Faction, MoveStatus, ObjectId, ObjectKind, OrderKind, Phase, PlayerCommand, PlayerId, Sim,
    SimConfig, SpawnAt, Target, Tick, TickInputs, Vec2,
};

const P1: PlayerId = PlayerId(1);
const P2: PlayerId = PlayerId(2);
/// Slice M2-J2's target for a whole tick, at worst.
const BUDGET: Duration = Duration::from_millis(10);
/// The phases that take time ([`Phase::End`] only marks the end).
const PHASES: usize = Phase::ALL.len() - 1;
/// Where each group starts (the middle of its block) and is sent: the other's side.
const WEST: Vec2 = Vec2::new(700.0, 1024.0);
const EAST: Vec2 = Vec2::new(2372.0, 1024.0);
const END: Tick = 600;

/// The wall-clock time of each phase of one tick of `sim` (`Sim::step_observed`), in [`Phase`]'s order. Only perf
/// tests read a clock: they measure the sim from outside, and nothing in the sim can call it (`ARCHITECTURE.md >
/// Determinism`).
#[expect(
    clippy::disallowed_methods,
    reason = "a perf test has to read the clock; the sim itself never does"
)]
fn timed_step(sim: &mut Sim, inputs: &TickInputs) -> [Duration; PHASES] {
    let mut marks = Vec::with_capacity(Phase::ALL.len());
    sim.step_observed(inputs, |phase| marks.push((phase, std::time::Instant::now())));
    let mut times = [Duration::ZERO; PHASES];
    for pair in marks.windows(2) {
        let (phase, start) = pair[0];
        times[phase as usize] += pair[1].1 - start;
    }
    times
}

/// The game at tick 0: ids 1–15 player 1's Seekers in a block of 5 × 3 around [`WEST`], 16–30 player 2's Hands of
/// Heaven around [`EAST`], 90 su apart.
fn game() -> Sim {
    let map = vec![".".repeat(48); 32].join("\n");
    let mut builder = (ScenarioBuilder::new(&map))
        .faction(P1, Faction::Cults)
        .faction(P2, Faction::Cults)
        .config(SimConfig::DEBUG);
    let block = |centre: Vec2, i: usize| {
        let offset = Vec2::new((i % 3) as f32 - 1.0, (i / 3) as f32 - 2.0) * 90.0;
        SpawnAt::Point(centre + offset)
    };
    for i in 0..15 {
        builder = builder.spawn(ObjectKind::Seeker, Some(P1), block(WEST, i));
    }
    for i in 0..15 {
        builder = builder.spawn(ObjectKind::HandOfHeaven, Some(P2), block(EAST, i));
    }
    Sim::new(&builder.build())
}

/// The orders of tick 1: auto-targeting off, each group to the other's side.
fn orders() -> Vec<PlayerCommand> {
    let group = |from: u32| (from..from + 15).map(ObjectId).collect::<Vec<_>>();
    vec![
        PlayerCommand {
            player: P1,
            command: Command::Debug(DebugCommand::SetAutoTargeting { on: false }),
        },
        PlayerCommand {
            player: P1,
            command: Command::Order {
                units: group(1),
                kind: OrderKind::Move(Target::Point(EAST)),
                queue: false,
            },
        },
        PlayerCommand {
            player: P2,
            command: Command::Order {
                units: group(16),
                kind: OrderKind::Move(Target::Point(WEST)),
                queue: false,
            },
        },
    ]
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
        "  {label:<26} mean {:>7.3} ms  p99 {:>7.3} ms  worst {:>7.3} ms (tick {tick}){flag}",
        mean,
        ms(p99),
        ms(worst)
    )
}

#[test]
#[ignore = "timing: run with --release --ignored --nocapture"]
fn perf_30_vehicles_in_a_crowd() {
    let mut sim = game();
    let mut invariants = Invariants::new(&sim);
    let (mut whole, mut movement) = (Vec::new(), Vec::new());
    let mut turns = 0;
    let mut facings: Vec<f32> = (1..=30).map(|id| sim.object(ObjectId(id)).unwrap().facing).collect();
    while sim.tick() < END {
        let commands = if sim.tick() == 0 { orders() } else { Vec::new() };
        let inputs = TickInputs { commands };
        let times = timed_step(&mut sim, &inputs);
        if let Err(violation) = invariants.check(&sim, &inputs) {
            panic!("tick {}: {violation:?}", sim.tick());
        }
        let tick = sim.tick();
        whole.push((tick, times.iter().sum()));
        movement.push((tick, times[Phase::Movement as usize]));
        for (id, facing) in (1..=30).zip(&mut facings) {
            let now = sim.object(ObjectId(id)).unwrap().facing;
            turns += usize::from(now != *facing);
            *facing = now;
        }
    }
    let crossed = (1..=30).filter(|&id| {
        let o = sim.object(ObjectId(id)).unwrap();
        let goal = if id <= 15 { EAST } else { WEST };
        o.pos.distance(goal) < o.pos.distance(if id <= 15 { WEST } else { EAST })
    });
    let crossed = crossed.count();
    let statuses = (1..=30).map(|id| sim.object(ObjectId(id)).unwrap().mover.as_ref().unwrap().status);
    let arrived = statuses.filter(|&s| s == MoveStatus::Arrived).count();
    assert!(crossed >= 20, "only {crossed} of the 30 vehicles got past the middle");
    assert!(turns > 300, "the vehicles turned {turns} times");
    println!(
        "30 vehicles, {END} ticks: two groups of 15 through each other, {turns} turns; {crossed} past the middle, \
         {arrived} arrived"
    );
    println!("{}", line("whole tick", &whole));
    println!("{}", line("movement (5d)", &movement));
}
