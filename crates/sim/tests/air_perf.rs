//! Timings of whole ticks with forty Sparrows in the air (slice M5-H1; plan §C's `air_perf`: "a new `air_perf` with 40
//! gliders, recorded"), phase by phase (`Sim::step_observed`). Ignored by default:
//!
//! ```sh
//! cargo test -p sc_sim --release --test perf air_perf -- --ignored --nocapture --test-threads=1
//! ```
//!
//! On a flat 64 × 40 map player 1's twenty Sparrows strafe player 2's four Hands of Heaven, five to each (Attack
//! orders: each a strafing run, `movement::glider`), the targets kept in sight by a Militant of player 1's beside
//! each; player 2's twenty Sparrows, each sent on its own to one point, crowd it and circle it, so that the soft
//! separation (`movement::air`) has pairs to part every tick. Auto-targeting is off: only the orders fight, and the
//! Hands of Heaven (MaxHP 200, PointArmor 2: a hit of the machine gun's 3 takes 1, times the side it lands on) outlast
//! the run, 640 ticks, 40 s (the first falls at about 51 s).
//!
//! It prints, for the whole tick and for the movement phase (5d), the mean, the 99th percentile and the worst tick,
//! flagging a worst tick over 10 ms (slice M2-J2's target for a whole tick). The assertions are on the game, never on
//! wall time. New in M5-H1, so its figures are absolute ones: compare them by A/B runs.

use std::time::Duration;

use sc_sim::testing::ScenarioBuilder;
use sc_sim::testing::invariants::Invariants;
use sc_sim::{
    Command, DebugCommand, Faction, ObjectId, ObjectKind, OrderKind, Phase, PlayerCommand, PlayerId, Sim, SimConfig,
    SimEvent, SpawnAt, Target, Tick, TickInputs, Vec2,
};

const P1: PlayerId = PlayerId(1);
const P2: PlayerId = PlayerId(2);
/// Slice M2-J2's target for a whole tick, at worst.
const BUDGET: Duration = Duration::from_millis(10);
/// The phases that take time ([`Phase::End`] only marks the end).
const PHASES: usize = Phase::ALL.len() - 1;
/// Player 2's Hands of Heaven, the targets.
const TARGETS: [(f32, f32); 4] = [(1400.0, 700.0), (1400.0, 1900.0), (2200.0, 700.0), (2200.0, 1900.0)];
/// Where player 2's Sparrows crowd.
const CROWD: Vec2 = Vec2::new(3400.0, 1280.0);
const END: Tick = 640;

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

/// The game at tick 0: ids 1–20 player 1's Sparrows, 21–40 player 2's, 41–44 the Hands of Heaven, 45–48 player 1's
/// Militants.
fn game() -> Sim {
    let map = vec![".".repeat(64); 40].join("\n");
    let mut builder = (ScenarioBuilder::new(&map))
        .faction(P1, Faction::Cults)
        .faction(P2, Faction::Cults)
        .config(SimConfig::DEBUG);
    let at = |x: f32, y: f32| SpawnAt::Point(Vec2::new(x, y));
    for i in 0..20 {
        builder = builder.spawn(
            ObjectKind::Sparrow,
            Some(P1),
            at(200.0 + 60.0 * (i % 4) as f32, 300.0 + 100.0 * (i / 4) as f32),
        );
    }
    for i in 0..20 {
        builder = builder.spawn(
            ObjectKind::Sparrow,
            Some(P2),
            at(3000.0 + 60.0 * (i % 5) as f32, 900.0 + 60.0 * (i / 5) as f32),
        );
    }
    for (x, y) in TARGETS {
        builder = builder.spawn(ObjectKind::HandOfHeaven, Some(P2), at(x, y));
    }
    for (x, y) in TARGETS {
        builder = builder.spawn(ObjectKind::Militant, Some(P1), at(x, y + 150.0));
    }
    Sim::new(&builder.build())
}

/// The orders of tick 1: auto-targeting off, player 1's Sparrows five to each target, player 2's each to the crowd's
/// point on its own.
fn orders() -> Vec<PlayerCommand> {
    let mut commands = vec![PlayerCommand {
        player: P1,
        command: Command::Debug(DebugCommand::SetAutoTargeting { on: false }),
    }];
    for i in 0..20 {
        commands.push(PlayerCommand {
            player: P1,
            command: Command::Order {
                units: vec![ObjectId(1 + i)],
                kind: OrderKind::Attack(ObjectId(41 + i % 4)),
                queue: false,
            },
        });
        commands.push(PlayerCommand {
            player: P2,
            command: Command::Order {
                units: vec![ObjectId(21 + i)],
                kind: OrderKind::Move(Target::Point(CROWD)),
                queue: false,
            },
        });
    }
    commands
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
fn perf_40_sparrows() {
    let mut sim = game();
    let mut rules = Invariants::new(&sim);
    let (mut whole, mut movement) = (Vec::new(), Vec::new());
    let mut hits = 0;
    while sim.tick() < END {
        let commands = if sim.tick() == 0 { orders() } else { Vec::new() };
        let inputs = TickInputs { commands };
        let times = timed_step(&mut sim, &inputs);
        rules.check_rules(&sim, &inputs).unwrap();
        let tick = sim.tick();
        whole.push((tick, times.iter().sum()));
        movement.push((tick, times[Phase::Movement as usize]));
        hits += (sim.events().iter())
            .filter(|e| matches!(e, SimEvent::Hit { .. }))
            .count();
    }
    let hp = (41..=44).map(|id| {
        sim.object(ObjectId(id))
            .and_then(|o| o.hp)
            .expect("every target stands")
    });
    let weakest = hp.fold(f32::INFINITY, f32::min);
    let sparrows = (sim.objects()).filter(|o| o.kind == ObjectKind::Sparrow).count();
    assert_eq!(sparrows, 40, "every Sparrow flies on");
    let attacking = (1..=20).filter(|&id| {
        let orders = sim.object(ObjectId(id)).and_then(|o| o.orders.as_ref());
        orders
            .and_then(|orders| orders.current.as_ref())
            .is_some_and(|c| matches!(c.order.kind, OrderKind::Attack(_)))
    });
    assert_eq!(attacking.count(), 20, "player 1's Attacks go on");
    assert!(hits > 400, "{hits} hits");
    let crowd = (21..=40).map(|id| sim.object(ObjectId(id)).unwrap().pos.distance(CROWD));
    let farthest = crowd.fold(0.0_f32, f32::max);
    println!(
        "40 Sparrows, {END} ticks: 20 strafing 4 targets ({hits} hits, the weakest left at {weakest} HP), 20 crowding \
         one point (all within {farthest:.0} su of it at the end)"
    );
    println!("{}", line("whole tick", &whole));
    println!("{}", line("movement (5d)", &movement));
}
