//! Timings of whole ticks with a Cults economy at work (slice M5-D, `artifacts/director/m5-plan.md > D: Recruit
//! gathering`: "60 Recruits on a field with four Storages, and three Halls producing"), phase by phase
//! (`Sim::step_observed`). Ignored by default:
//!
//! ```sh
//! cargo test -p sc_sim --release --test perf recruits_perf -- --ignored --nocapture --test-threads=1
//! ```
//!
//! On a flat 48 × 40 map player 1 has a field of 22 patches in two rows of 11, 4 tiles apart, with four Storages
//! between the rows, a tile from either (`factions/cults/structures/storage.md`: "The best layout leaves a 1-tile gap
//! between patch and Storage"), so that each Storage takes loads at two faces; 60 Recruits, dealt to the patches in
//! tick 1 from blocks north and south of the field; and three Recruitment Halls south of it, which produce what their
//! slice (M5-C) makes them produce. The Recruits keep their collision while they gather (design change 0106): 60 on
//! 22 patches are more than the faces hold (`tests/recruit_gathering/jam.rs`), so the field is a crowd all along:
//! legs that end short and begin again, Recruits waiting at patches they can't get to, and the steering of all of
//! them in lanes a Recruit wide. That is the load this test times, not an economy to measure.
//!
//! It prints, for the whole tick and for each phase, the mean, the 99th percentile and the worst tick, flagging a
//! worst tick over 10 ms (slice M2-J2's target for a whole tick). Tick 1, with its 22 Gathers, is a section of its
//! own. The assertions are on the game, never on wall time. New in M5-D, so its figures are absolute ones: compare
//! them by A/B runs.

use std::time::Duration;

use sc_sim::testing::{ScenarioBuilder, TestSim};
use sc_sim::{Faction, ObjectId, ObjectKind, OrderKind, Phase, PlayerId, SimEvent, SpawnAt, Tick, TilePos, Vec2};

const WIDTH: usize = 48;
const HEIGHT: usize = 40;
const P1: PlayerId = PlayerId(1);
/// Slice M2-J2's target for a whole tick, at worst.
const BUDGET: Duration = Duration::from_millis(10);
/// The phases that take time ([`Phase::End`] only marks the end).
const PHASES: usize = Phase::ALL.len() - 1;
const RECRUITS: u32 = 60;
/// The patches of each of the field's two rows, from tile column [`FIELD_X`]; the north row is on tile row
/// [`FIELD_Y`], the Storages on the two rows 2 below it, the south row 5 below it.
const ROW: i32 = 11;
const FIELD_X: i32 = 18;
const FIELD_Y: i32 = 12;
/// The Storages' west columns, from [`FIELD_X`].
const STORAGES: [i32; 4] = [0, 3, 6, 9];
/// The game is timed to 3 minutes.
const TICKS: Tick = 3 * 60 * 16;

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

/// The game, and the Recruits by the patch each is dealt to. Ids: the north row's patches 1 to 11, the south row's 12
/// to 22, the Storages 23 to 26, the Halls 27 to 29, the Recruits from 30.
fn game() -> (TestSim, Vec<(ObjectId, Vec<ObjectId>)>) {
    let map = vec![".".repeat(WIDTH); HEIGHT].join("\n");
    let mut builder = ScenarioBuilder::new(&map).faction(P1, Faction::Cults);
    let mut deals = Vec::new();
    let mut next = 1;
    for y in [FIELD_Y, FIELD_Y + 5] {
        for x in 0..ROW {
            let at = SpawnAt::Tile(TilePos::new(FIELD_X + x, y));
            builder = builder.spawn(ObjectKind::CrystalPatch, None, at);
            deals.push((ObjectId(next), Vec::new()));
            next += 1;
        }
    }
    for x in STORAGES {
        let at = SpawnAt::Tile(TilePos::new(FIELD_X + x, FIELD_Y + 2));
        builder = builder.spawn(ObjectKind::Storage, Some(P1), at);
        next += 1;
    }
    for i in 0..3 {
        let at = SpawnAt::Tile(TilePos::new(14 + 7 * i, FIELD_Y + 16));
        builder = builder.spawn(ObjectKind::RecruitmentHall, Some(P1), at);
        next += 1;
    }
    // Half the Recruits north of the north row, which they are dealt to, and half south of the south row: blocks 10
    // wide, 40 space units apart, 2 tiles clear of the rows.
    for i in 0..RECRUITS as usize {
        let (row, place) = (i % 2, i / 2);
        let (column, line) = (place % 10, place / 10);
        let corner = TilePos::new(FIELD_X, if row == 0 { FIELD_Y - 2 } else { FIELD_Y + 8 }).corner();
        let down = if row == 0 { -1.0 } else { 1.0 };
        let at = corner + Vec2::new(60.0 + 40.0 * column as f32, down * (20.0 + 40.0 * line as f32));
        builder = builder.spawn(ObjectKind::Recruit, Some(P1), SpawnAt::Point(at));
        deals[row * ROW as usize + place % ROW as usize].1.push(ObjectId(next));
        next += 1;
    }
    let scenario = builder.build();
    assert_eq!(scenario.validate(), vec![]);
    let mut t = TestSim::new(scenario);
    t.check_invariants(false);
    for (patch, recruits) in &deals {
        t.order_at(1, P1, recruits, OrderKind::Gather(*patch), false);
    }
    (t, deals)
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
fn sixty_recruits_on_a_field_with_four_storages() {
    let (mut t, deals) = game();
    let count = |t: &TestSim, kind: ObjectKind| t.sim().objects().filter(|o| o.kind == kind).count();
    assert_eq!(count(&t, ObjectKind::Recruit), RECRUITS as usize);
    let mut ticks: Vec<(Tick, [Duration; PHASES])> = Vec::new();
    while t.sim().tick() < TICKS {
        let times = timed_step(&mut t);
        ticks.push((t.sim().tick(), times));
    }
    let of = |phases: &[Phase], keep: &dyn Fn(Tick) -> bool| -> Vec<(Tick, Duration)> {
        (ticks.iter().filter(|(tick, _)| keep(*tick)))
            .map(|(tick, times)| (*tick, phases.iter().map(|&p| times[p as usize]).sum()))
            .collect()
    };
    let whole = &Phase::ALL[..PHASES];
    println!("tick 1: the {} Gathers of {RECRUITS} Recruits", deals.len());
    println!("{}", line("whole tick", &of(whole, &|tick| tick == 1)));
    println!("ticks 2 to {TICKS}: the Recruits at work");
    println!("{}", line("whole tick", &of(whole, &|tick| tick > 1)));
    for &phase in whole {
        println!("{}", line(&format!("{phase:?}"), &of(&[phase], &|tick| tick > 1)));
    }

    // The game: every load delivered is in the bank, the Recruits all still gather, and the field works.
    let delivered: Vec<(Tick, ObjectId, u32)> = (t.events().iter())
        .filter_map(|(tick, e)| match *e {
            SimEvent::ResourcesDelivered { worker, amount, .. } => Some((*tick, worker, amount)),
            _ => None,
        })
        .collect();
    let total: u32 = delivered.iter().map(|d| d.2).sum();
    let working = (deals.iter().flat_map(|(_, recruits)| recruits))
        .filter(|id| delivered.iter().any(|d| d.1 == **id && d.0 > TICKS - 960))
        .count();
    let recruits = count(&t, ObjectKind::Recruit);
    println!(
        "{total} Space Crystals in {TICKS} ticks ({:.1} per second); {working} of the {RECRUITS} delivered in the \
         last minute; {recruits} Recruits at the end",
        total as f32 * 16.0 / TICKS as f32
    );
    assert_eq!(t.sim().player(P1).expect("in the game").crystals, total);
    assert!(total > 1000, "the field works: {total}");
    let gathering = (t.sim().objects())
        .filter(|o| deals.iter().any(|(_, recruits)| recruits.contains(&o.id)))
        .filter(|o| {
            let current = o.orders.as_ref().and_then(|orders| orders.current.as_ref());
            current.is_some_and(|active| matches!(active.order.kind, OrderKind::Gather(_)))
        })
        .count();
    assert_eq!(gathering, RECRUITS as usize, "a Gather never ends on its own");
}
