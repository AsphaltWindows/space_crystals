//! Timings of whole ticks with ten full Garrisons firing (slice M5-N2, `sc_sim::combat::occupants`: "a perf run with
//! ten full Garrisons"), phase by phase (`Sim::step_observed`). Ignored by default:
//!
//! ```sh
//! cargo test -p sc_sim --release --test perf garrisons_perf -- --ignored --nocapture --test-threads=1
//! ```
//!
//! On a flat 64 × 40 map player 1 has ten Garrisons in two rows of five, 4 tiles apart, a lane of 8 tiles between the
//! rows, and three Militants and a Gunner by each one's door, which go in at tick 1: forty units inside. From tick
//! [`WAVES_FROM`] on, a wave of [`WAVE`] Peacekeepers of player 2's appears at the lane's east end every [`EVERY`]
//! ticks and walks west down it (a Move: they don't shoot back), in range of both rows; those that get through leave
//! the game at its west end. The units inside pick their targets as a structure does and shoot from the footprints.
//!
//! It prints, for the whole tick and for the combat phase (5e), the mean, the 99th percentile and the worst tick,
//! flagging a worst tick over 10 ms (slice M2-J2's target for a whole tick): while the forty go in, while they wait
//! with nothing in range, and under the waves. The assertions are on the game, never on wall time. New in M5-N2, so
//! its figures are absolute ones: compare them by A/B runs.

use std::time::Duration;

use sc_sim::testing::{ScenarioBuilder, TestSim};
use sc_sim::{
    DebugCommand, Faction, ObjectId, ObjectKind, OrderKind, Phase, PlayerId, SU_PER_GU, SimEvent, SpawnAt, Target,
    Tick, TilePos, Vec2,
};

const WIDTH: usize = 64;
const HEIGHT: usize = 40;
const P1: PlayerId = PlayerId(1);
const P2: PlayerId = PlayerId(2);
/// Slice M2-J2's target for a whole tick, at worst.
const BUDGET: Duration = Duration::from_millis(10);
/// The phases that take time ([`Phase::End`] only marks the end).
const PHASES: usize = Phase::ALL.len() - 1;
/// The Garrisons' rows' tile rows, and their columns: 2 × 2 footprints 4 tiles apart.
const ROWS: [i32; 2] = [14, 24];
const COLUMNS: [i32; 5] = [17, 23, 29, 35, 41];
/// Who goes into each: three Militants and a Gunner.
const INSIDE: [ObjectKind; 4] = [
    ObjectKind::Militant,
    ObjectKind::Militant,
    ObjectKind::Militant,
    ObjectKind::Gunner,
];
/// The waves: [`WAVE`] Peacekeepers every [`EVERY`] ticks from tick [`WAVES_FROM`], [`WAVES`] of them, and the tick
/// the run ends, once the last has gone down the lane.
const WAVES_FROM: Tick = 400;
const EVERY: Tick = 160;
const WAVE: usize = 30;
const WAVES: u32 = 10;
const END: Tick = WAVES_FROM + WAVES * EVERY + 600;
/// A Peacekeeper of player 2's this far west, in grid units, has gone through the lane and leaves the game.
const WEST_END: f32 = 8.0;

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

/// The game, with each Garrison's units ordered in at tick 1. Ids: the Garrisons 1 to 10, the Storage that keeps
/// player 1 in the game 11, the units from 12.
fn game() -> TestSim {
    let map = vec![".".repeat(WIDTH); HEIGHT].join("\n");
    let mut builder = (ScenarioBuilder::new(&map))
        .faction(P1, Faction::Cults)
        .faction(P2, Faction::Gdo);
    let mut origins = Vec::new();
    for y in ROWS {
        for x in COLUMNS {
            let origin = TilePos::new(x, y);
            builder = builder.spawn(ObjectKind::Garrison, Some(P1), SpawnAt::Tile(origin));
            origins.push(origin);
        }
    }
    builder = builder.spawn(ObjectKind::Storage, Some(P1), SpawnAt::Tile(TilePos::new(2, 2)));
    let mut next = origins.len() as u32 + 2;
    let mut fills = Vec::new();
    for (i, origin) in origins.iter().enumerate() {
        // A column of four, 32 space units apart, 1.5 grid units west of the door, the middle of side B.
        let door = origin.corner() + Vec2::new(0.0, SU_PER_GU);
        let mut units = Vec::new();
        for (k, kind) in INSIDE.into_iter().enumerate() {
            let at = door + Vec2::new(-1.5 * SU_PER_GU, 32.0 * k as f32 - 48.0);
            builder = builder.spawn(kind, Some(P1), SpawnAt::Point(at));
            units.push(ObjectId(next));
            next += 1;
        }
        fills.push((ObjectId(i as u32 + 1), units));
    }
    let scenario = builder.build();
    assert_eq!(scenario.validate(), vec![]);
    let mut t = TestSim::new(scenario);
    t.check_invariants(false);
    for (garrison, units) in &fills {
        t.order_at(1, P1, units, OrderKind::Enter(*garrison), false);
    }
    t
}

/// The wave of `tick`: [`WAVE`] Peacekeepers in a block 3 wide and 10 deep at the lane's east end.
fn spawn_wave(t: &mut TestSim, tick: Tick) {
    for i in 0..WAVE {
        let corner = TilePos::new(58, ROWS[0] + 2).corner();
        let at = corner + Vec2::new(40.0 * (i % 3) as f32, 48.0 * (i / 3) as f32) + Vec2::splat(32.0);
        let spawn = DebugCommand::Spawn {
            kind: ObjectKind::Peacekeeper,
            owner: Some(P2),
            at: SpawnAt::Point(at),
            orientation: Default::default(),
        };
        t.debug_at(tick, spawn);
    }
}

/// Player 2's units: those standing (a wave just in, to be sent down the lane), and those through it.
fn player_2(t: &TestSim) -> (Vec<ObjectId>, Vec<ObjectId>) {
    let units: Vec<_> = t.sim().objects().filter(|o| o.owner == Some(P2)).collect();
    let standing = (units.iter())
        .filter(|o| o.orders.as_ref().is_some_and(|orders| orders.is_idle()))
        .map(|o| o.id)
        .collect();
    let through = (units.iter())
        .filter(|o| o.pos.x < WEST_END * SU_PER_GU)
        .map(|o| o.id)
        .collect();
    (standing, through)
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
        "  {label:<38} mean {:>7.3} ms  p99 {:>7.3} ms  worst {:>7.3} ms (tick {tick}){flag}",
        mean,
        ms(p99),
        ms(worst)
    )
}

#[test]
#[ignore = "timing: run with --release --ignored --nocapture"]
fn perf_10_full_garrisons_against_waves() {
    let mut t = game();
    let west = TilePos::new(1, ROWS[0] + 6).center();
    // Each section's whole ticks and combat phases.
    let mut sections: [(Times, Times); 3] = Default::default();
    let (mut shots, mut through) = (0, 0);
    let mut full_from = None;
    while t.sim().tick() < END {
        let tick = t.sim().tick() + 1;
        if tick >= WAVES_FROM && (tick - WAVES_FROM).is_multiple_of(EVERY) && tick < WAVES_FROM + WAVES * EVERY {
            spawn_wave(&mut t, tick);
        }
        let times = timed_step(&mut t);
        let inside = t.sim().held().count();
        if full_from.is_none() && inside == 40 {
            full_from = Some(tick);
        }
        let section = match full_from {
            None => 0,
            Some(_) if tick < WAVES_FROM => 1,
            Some(_) => 2,
        };
        sections[section].0.push((tick, times.iter().sum()));
        sections[section].1.push((tick, times[Phase::Combat as usize]));
        if section == 2 {
            assert_eq!(inside, 40, "tick {tick}: everyone stays inside");
        }
        for event in t.sim().events() {
            if let SimEvent::Attacked { attacker, .. } = event
                && t.sim().held_unit(*attacker).is_some()
            {
                shots += 1;
            }
        }
        let (standing, gone) = player_2(&t);
        if !standing.is_empty() {
            t.order_at(tick + 1, P2, &standing, OrderKind::Move(Target::Point(west)), false);
        }
        for id in gone {
            t.debug_at(tick + 1, DebugCommand::Remove { id });
            through += 1;
        }
    }
    let killed = (t.events().iter())
        .filter(|(_, e)| {
            matches!(
                e,
                SimEvent::Died {
                    kind: ObjectKind::Peacekeeper,
                    ..
                }
            )
        })
        .count();
    let full_from = full_from.expect("the forty went in");
    assert!(full_from < WAVES_FROM, "all inside by tick {full_from}");
    assert!(
        killed > 0 && shots > 0,
        "the units inside fire: {shots} shots, {killed} killed"
    );
    assert!(
        t.sim()
            .objects()
            .filter(|o| o.kind == ObjectKind::Garrison)
            .all(|o| o.hp == o.max_hp()),
        "the Garrisons stand, untouched"
    );
    println!(
        "10 Garrisons, 40 inside from tick {full_from}; {} waves of {WAVE} Peacekeepers: {shots} shots from inside, \
         {killed} killed, {through} through, in {END} ticks",
        WAVES
    );
    let labels = ["the forty going in", "full, nothing in range", "full, under the waves"];
    for (label, (whole, combat)) in labels.iter().zip(&sections) {
        println!("{}", line(&format!("whole tick, {label}"), whole));
        println!("{}", line(&format!("combat (5e), {label}"), combat));
    }
}
