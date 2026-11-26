//! Timings of whole ticks with eight Disciples channelling Shroud over sixty units under fire (slice M5-N4b; plan
//! §C's `shroud_perf`: "8 Disciples, 60 units under fire", mean and p99), phase by phase (`Sim::step_observed`).
//! Ignored by default:
//!
//! ```sh
//! cargo test -p sc_sim --release --test perf shroud_perf -- --ignored --nocapture --test-threads=1
//! ```
//!
//! On a flat 64 × 40 map player 1 has four camps, each two Disciples 5 grid units apart, one cloud of two circles, and
//! thirteen Militants under it: sixty units under the clouds. Ten Militants of player 2's stand south of each camp,
//! outside its cloud, and shoot the nearest of the camp's units (auto-targeting off: each is given its target, and a
//! new one when it dies); player 1 doesn't shoot back. The Disciples start full (a save edited to 200 energy), so they
//! channel for the whole run, 711 ticks; every shot goes into a buffer, and each cloud releases over its units.
//!
//! It prints, for the whole tick and for the combat phase (5e), the mean, the 99th percentile and the worst tick,
//! flagging a worst tick over 10 ms (slice M2-J2's target for a whole tick), with the clouds and, for comparison, the
//! same game without them (the Disciples idle, every shot landing). The assertions are on the game, never on wall
//! time. New in M5-N4b, so its figures are absolute ones: compare them by A/B runs.

use std::time::Duration;

use sc_sim::testing::ScenarioBuilder;
use sc_sim::testing::invariants::Invariants;
use sc_sim::{
    Command, DebugCommand, Faction, ObjectId, ObjectKind, OrderKind, Phase, PlayerCommand, PlayerId, Sim, SimConfig,
    SimEvent, SpawnAt, Tick, TickInputs, Vec2,
};

const P1: PlayerId = PlayerId(1);
const P2: PlayerId = PlayerId(2);
/// Slice M2-J2's target for a whole tick, at worst.
const BUDGET: Duration = Duration::from_millis(10);
/// The phases that take time ([`Phase::End`] only marks the end).
const PHASES: usize = Phase::ALL.len() - 1;
/// The camps' centres.
const CAMPS: [(f32, f32); 4] = [(800.0, 700.0), (2400.0, 700.0), (800.0, 1700.0), (2400.0, 1700.0)];
/// A full Disciple channels this many ticks (`orders/shroud.rs`), the run's length.
const END: Tick = 711;

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

/// The game at tick 0, its Disciples full: ids 1–8 the Disciples, two by camp, 9–60 player 1's Militants, 61–100
/// player 2's.
fn game() -> Sim {
    let map = vec![".".repeat(64); 40].join("\n");
    let mut builder = (ScenarioBuilder::new(&map))
        .faction(P1, Faction::Cults)
        .faction(P2, Faction::Cults)
        .config(SimConfig::DEBUG);
    let at = |x: f32, y: f32| SpawnAt::Point(Vec2::new(x, y));
    for (cx, cy) in CAMPS {
        builder = builder.spawn(ObjectKind::Disciple, Some(P1), at(cx - 160.0, cy));
        builder = builder.spawn(ObjectKind::Disciple, Some(P1), at(cx + 160.0, cy));
    }
    for (cx, cy) in CAMPS {
        for i in 0..13 {
            let (column, row) = (i % 7, i / 7);
            let y = if row == 0 { cy - 60.0 } else { cy + 60.0 };
            builder = builder.spawn(ObjectKind::Militant, Some(P1), at(cx - 240.0 + 80.0 * column as f32, y));
        }
    }
    for (cx, cy) in CAMPS {
        for i in 0..10 {
            builder = builder.spawn(
                ObjectKind::Militant,
                Some(P2),
                at(cx - 360.0 + 80.0 * i as f32, cy + 300.0),
            );
        }
    }
    let sim = Sim::new(&builder.build());
    let mut json = serde_json::to_value(&sim).unwrap();
    for object in json["objects"].as_array_mut().unwrap() {
        if object["kind"] == "Disciple" {
            object["energy"] = serde_json::json!(200 * sc_sim::energy::ONE);
        }
    }
    serde_json::from_value(json).unwrap()
}

/// Player 2's idle Militants, each with the nearest of player 1's units to attack.
fn targets(sim: &Sim) -> Vec<PlayerCommand> {
    let theirs: Vec<(ObjectId, Vec2)> = (sim.objects())
        .filter(|o| o.owner == Some(P1))
        .map(|o| (o.id, o.pos))
        .collect();
    (sim.objects())
        .filter(|o| o.owner == Some(P2) && o.orders.as_ref().is_some_and(|orders| orders.is_idle()))
        .filter_map(|o| {
            let nearest = (theirs.iter()).min_by(|a, b| a.1.distance(o.pos).total_cmp(&b.1.distance(o.pos)))?;
            Some(PlayerCommand {
                player: P2,
                command: Command::Order {
                    units: vec![o.id],
                    kind: OrderKind::Attack(nearest.0),
                    queue: false,
                },
            })
        })
        .collect()
}

/// Ticks' times, each with its tick.
type Times = Vec<(Tick, Duration)>;

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

/// The run, with the clouds or without: the whole ticks' and the combat phases' times, and the releases, the hits
/// and the deaths counted.
fn run(shroud: bool) -> (Times, Times, [usize; 3]) {
    let mut sim = game();
    let mut rules = Invariants::new(&sim);
    let (mut whole, mut combat) = (Vec::new(), Vec::new());
    let mut counts = [0; 3];
    while sim.tick() < END {
        let mut commands = targets(&sim);
        if sim.tick() == 0 {
            commands.push(PlayerCommand {
                player: P1,
                command: Command::Debug(DebugCommand::SetAutoTargeting { on: false }),
            });
            if shroud {
                commands.push(PlayerCommand {
                    player: P1,
                    command: Command::Order {
                        units: (1..=8).map(ObjectId).collect(),
                        kind: OrderKind::Shroud,
                        queue: false,
                    },
                });
            }
        }
        let inputs = TickInputs { commands };
        let times = timed_step(&mut sim, &inputs);
        rules.check_rules(&sim, &inputs).unwrap();
        let tick = sim.tick();
        whole.push((tick, times.iter().sum()));
        combat.push((tick, times[Phase::Combat as usize]));
        for event in sim.events() {
            match event {
                SimEvent::ShroudReleased { .. } => counts[0] += 1,
                SimEvent::Hit { .. } => counts[1] += 1,
                SimEvent::Died { .. } => counts[2] += 1,
                _ => {}
            }
        }
        if shroud {
            assert_eq!(sim.clouds().list().len(), 4, "tick {tick}: the four camps' clouds");
        }
    }
    (whole, combat, counts)
}

#[test]
#[ignore = "timing: run with --release --ignored --nocapture"]
fn perf_8_disciples_over_60_units_under_fire() {
    let (whole, combat, [releases, hits, deaths]) = run(true);
    assert!(releases > 100 && hits > 1000, "{releases} releases, {hits} hits");
    let (open_whole, open_combat, [_, open_hits, open_deaths]) = run(false);
    assert!(
        open_deaths > deaths,
        "the clouds save units: {deaths} dead under them, {open_deaths} without"
    );
    println!(
        "8 Disciples channelling over 60 units, 40 shooting them, {END} ticks: {releases} releases, {hits} hits, \
         {deaths} dead; without the clouds {open_hits} hits, {open_deaths} dead"
    );
    println!("{}", line("whole tick, with the clouds", &whole));
    println!("{}", line("combat (5e), with the clouds", &combat));
    println!("{}", line("whole tick, without", &open_whole));
    println!("{}", line("combat (5e), without", &open_combat));
}
