//! Timings of whole ticks with buildings that hold units at work (slice M5-F1, `crate::holding` of `sc_sim`): eight
//! Armories, ten Recruits each, going in and coming out by Eject All again and again, then all eight destroyed in one
//! tick with everyone inside. Ignored by default:
//!
//! ```sh
//! cargo test -p sc_sim --release --test holding_perf -- --ignored --nocapture --test-threads=1
//! ```
//!
//! On a flat 48 × 40 map player 1 has two rows of four Armories, ten tiles apart, a block of ten Recruits east of each
//! one's entrance, and a Storage that keeps the player in the game when the Armories go. The Recruits are ordered in
//! every 200 ticks and ejected 100 ticks after, so that the entries, the streams out of the doorways and the walks
//! round the Armories overlap. It prints, for the whole tick and for the economy phase (`Sim::step_holding`), the
//! mean, the 99th percentile and the worst tick, flagging a worst tick over 10 ms (slice M2-J2's target for a whole
//! tick), and the tick of the evacuation (the command phase: the removals). The assertions are on the game, never on
//! wall time. New in M5-F1, so its figures are absolute ones: compare them by A/B runs.

use std::time::Duration;

use sc_sim::testing::{ScenarioBuilder, TestSim};
use sc_sim::{
    Command, DebugCommand, Faction, HoldCommand, ObjectId, ObjectKind, OrderKind, Phase, PlayerId, SimEvent, SpawnAt,
    Tick, TilePos, Vec2,
};

const WIDTH: usize = 48;
const HEIGHT: usize = 40;
const P1: PlayerId = PlayerId(1);
/// Slice M2-J2's target for a whole tick, at worst.
const BUDGET: Duration = Duration::from_millis(10);
/// The phases that take time ([`Phase::End`] only marks the end).
const PHASES: usize = Phase::ALL.len() - 1;
const ARMORIES: u32 = 8;
const PER_ARMORY: u32 = 10;
/// The Recruits go in at ticks 2, 202, 402, ..., and are ejected 100 ticks after; the last Enter is at tick
/// [`LAST_ENTER`], after which they stay in until the Armories go, in tick [`GONE`].
const CYCLE: Tick = 200;
const LAST_ENTER: Tick = 2 + 6 * CYCLE;
const GONE: Tick = LAST_ENTER + 2 * CYCLE;

/// The wall-clock time of each phase of one tick of `t` (`Sim::step_observed`), in [`Phase`]'s order. Only perf
/// tests read a clock: they measure the sim from outside, and nothing in the sim can call it (`ARCHITECTURE.md >
/// Determinism`).
#[expect(
    clippy::disallowed_methods,
    reason = "a perf test has to read the clock; the sim itself never does"
)]
fn timed_step(t: &mut TestSim) -> [Duration; PHASES] {
    let mut marks = Vec::with_capacity(Phase::ALL.len());
    t.step_observed(|phase| marks.push((phase, std::time::Instant::now())));
    let mut times = [Duration::ZERO; PHASES];
    for pair in marks.windows(2) {
        let (phase, start) = pair[0];
        times[phase as usize] += pair[1].1 - start;
    }
    times
}

/// The game, and each Armory with its Recruits. Ids: the Armories 1 to 8, the Storage 9, the Recruits from 10.
fn game() -> (TestSim, Vec<(ObjectId, Vec<ObjectId>)>) {
    let map = vec![".".repeat(WIDTH); HEIGHT].join("\n");
    let mut builder = ScenarioBuilder::new(&map).faction(P1, Faction::Cults);
    let origins: Vec<TilePos> = (0..ARMORIES as i32)
        .map(|i| TilePos::new(3 + 10 * (i % 4), 8 + 16 * (i / 4)))
        .collect();
    for &origin in &origins {
        builder = builder.spawn(ObjectKind::Armory, Some(P1), SpawnAt::Tile(origin));
    }
    builder = builder.spawn(ObjectKind::Storage, Some(P1), SpawnAt::Tile(TilePos::new(2, 2)));
    let mut armories = Vec::new();
    let mut next = ARMORIES + 2;
    for (i, origin) in origins.iter().enumerate() {
        // Two columns of five, 32 su apart, from about 1 grid unit east of side B, centred on its middle.
        let entrance = origin.corner() + Vec2::new(192.0, 64.0);
        let mut recruits = Vec::new();
        for r in 0..PER_ARMORY {
            let at = entrance + Vec2::new(68.0 + 32.0 * (r / 5) as f32, 32.0 * (r % 5) as f32 - 64.0);
            builder = builder.spawn(ObjectKind::Recruit, Some(P1), SpawnAt::Point(at));
            recruits.push(ObjectId(next));
            next += 1;
        }
        armories.push((ObjectId(i as u32 + 1), recruits));
    }
    let scenario = builder.build();
    assert_eq!(scenario.validate(), vec![]);
    let mut t = TestSim::new(scenario);
    t.check_invariants(false);
    (t, armories)
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
fn perf_8_armories_80_recruits_cycling_then_evacuated() {
    let (mut t, armories) = game();
    let all: Vec<ObjectId> = armories.iter().flat_map(|(_, recruits)| recruits.clone()).collect();
    let buildings: Vec<ObjectId> = armories.iter().map(|&(armory, _)| armory).collect();
    let mut enter = 2;
    while enter <= LAST_ENTER {
        for (armory, recruits) in &armories {
            t.order_at(enter, P1, recruits, OrderKind::Enter(*armory), false);
        }
        if enter < LAST_ENTER {
            let objects = buildings.clone();
            t.at(
                enter + CYCLE / 2,
                P1,
                Command::Holding {
                    objects,
                    command: HoldCommand::EjectAll,
                },
            );
        }
        enter += CYCLE;
    }
    for &armory in &buildings {
        t.debug_at(GONE, DebugCommand::Remove { id: armory });
    }
    let (mut whole, mut economy) = (Vec::new(), Vec::new());
    while t.sim().tick() + 1 < GONE {
        let times = timed_step(&mut t);
        let tick = t.sim().tick();
        whole.push((tick, times.iter().sum()));
        economy.push((tick, times[Phase::Economy as usize]));
    }
    let inside = t.sim().held().count();
    assert_eq!(inside, all.len(), "all inside before the Armories go");
    let times = timed_step(&mut t);
    let gone = vec![(t.sim().tick(), times.iter().sum::<Duration>())];
    let commands = vec![(t.sim().tick(), times[Phase::Commands as usize])];
    let evacuated = (t.events().iter())
        .filter(|(_, e)| matches!(e, SimEvent::Evacuated { .. }))
        .count();
    assert_eq!(evacuated, all.len());
    assert!(all.iter().all(|&recruit| t.sim().object(recruit).is_some()));
    let (went_in, came_out) = t.events().iter().fold((0, 0), |(i, o), (_, e)| match e {
        SimEvent::EnteredBuilding { .. } => (i + 1, o),
        SimEvent::LeftBuilding { .. } => (i, o + 1),
        _ => (i, o),
    });
    println!(
        "8 Armories, 80 Recruits: {went_in} entries, {came_out} exits by Eject All in {} ticks, then {evacuated} \
         evacuated",
        GONE - 1
    );
    println!("{}", line("whole tick, cycling", &whole));
    println!("{}", line("economy phase, cycling", &economy));
    println!("{}", line("whole tick, evacuation", &gone));
    println!("{}", line("commands, evacuation", &commands));
}
