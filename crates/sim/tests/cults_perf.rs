//! Timings of whole ticks with the Cults' whole economy at work (slice M5-Mb, `artifacts/director/m5-plan.md > Perf`:
//! "`cults_perf` (M): three Halls, 80 Recruits, four sites, two Armories training, one evacuation"), phase by phase
//! (`Sim::step_observed`). Ignored by default:
//!
//! ```sh
//! cargo test -p sc_sim --release --test perf cults_perf -- --ignored --nocapture --test-threads=1
//! ```
//!
//! On a flat 64 × 48 map player 1 has `recruits_perf`'s field (22 patches in two rows of 11, four Storages between
//! the rows), three Recruitment Halls whose areas are all on the map and apart, two Armories, and 80 Recruits,
//! ordered in tick 1:
//! - 40 gather, dealt to the patches from blocks north and south of the field;
//! - 20 build four sites in groups of five (a Recruitment Hall, a Chop Shop, an Altar and a Garrison), each group
//!   from a row north of its site (`factions/cults/economy.md > Cults Building Mechanics`: the group's Build, plan
//!   decision T59);
//! - 20 enter the two Armories, ten each, from blocks east of their entrances, and each Armory trains Militants from
//!   them once all ten are in: the first ten, the second five;
//! - at 1:30 the second Armory queues [`MORE_TRAINED`] more, and [`IN_TRAINING`] ticks later it is destroyed (the
//!   removal path of a destruction, `DebugCommand::Remove`): the five Recruits still inside come out alive on its
//!   footprint, the one in training as a Recruit, and the three entries are refunded (design change 0349,
//!   `factions/cults/economy.md > Cults Building Mechanics > Evacuation on Destruction`).
//!
//! Meanwhile the Halls produce Recruits, as slice M5-C makes them, and the fourth Hall too once it is built. It
//! prints, for the whole tick and for each phase, the mean, the 99th percentile and the worst tick, flagging a worst
//! tick over 10 ms (slice M2-J2's target for a whole tick); tick 1, with its orders, and the destruction's tick are
//! sections of their own. The assertions are on the game, never on wall time. New in M5-Mb, so its figures are
//! absolute ones: compare them by A/B runs.

use std::time::Duration;

use sc_sim::testing::{ScenarioBuilder, TestSim};
use sc_sim::{
    Command, DebugCommand, Faction, ObjectId, ObjectKind, OrderKind, Phase, PlayerId, ProductionCommand, RecruitKind,
    SimEvent, SpawnAt, Tick, TilePos, Vec2,
};

const WIDTH: usize = 64;
const HEIGHT: usize = 48;
const P1: PlayerId = PlayerId(1);
/// Slice M2-J2's target for a whole tick, at worst.
const BUDGET: Duration = Duration::from_millis(10);
/// The phases that take time ([`Phase::End`] only marks the end).
const PHASES: usize = Phase::ALL.len() - 1;
/// The game is timed to 3 minutes.
const TICKS: Tick = 3 * 60 * 16;
/// Player 1's Space Crystals, set in tick 1: the four sites' 800 and the 18 Militants' 360, with room to spare.
const BANK: u32 = 2000;

/// The patches of each of the field's two rows, from tile column [`FIELD_X`]; the north row is on tile row
/// [`FIELD_Y`], the Storages on the two rows 2 below it, the south row 5 below it (`recruits_perf`'s field).
const ROW: i32 = 11;
const FIELD_X: i32 = 4;
const FIELD_Y: i32 = 6;
/// The Storages' west columns, from [`FIELD_X`].
const STORAGES: [i32; 4] = [0, 3, 6, 9];
const GATHERERS: usize = 40;
/// The Halls' north-west tiles: each one's 10 × 10 area is on the map and apart from the others', so each claims all
/// of it, with a capacity of 20 (`crate::recruitment` of `sc_sim`).
const HALLS: [(i32, i32); 3] = [(40, 3), (40, 17), (52, 10)];
/// The Armories' north-west tiles, unturned: the entrance, B, faces east and the exit, C, west.
const ARMORIES: [(i32, i32); 2] = [(20, 30), (32, 30)];
/// The Recruits each Armory takes: its capacity.
const HELD: usize = 10;
/// The Militants each Armory queues once all its Recruits are in.
const TRAINED: [usize; 2] = [10, 5];
/// The second Armory's further Militants, queued at 1:30 ([`MORE`]), and how far into the first of them it is
/// destroyed, at [`DESTROYED`].
const MORE: Tick = 1440;
const MORE_TRAINED: usize = 3;
const IN_TRAINING: Tick = 10;
const DESTROYED: Tick = MORE + IN_TRAINING;
/// The four sites, by their north-west tiles, each built by [`BUILDERS`] Recruits from the row 3 tiles north of it.
const SITES: [(ObjectKind, i32, i32); 4] = [
    (ObjectKind::RecruitmentHall, 6, 38),
    (ObjectKind::ChopShop, 16, 38),
    (ObjectKind::Altar, 24, 38),
    (ObjectKind::Garrison, 32, 38),
];
const BUILDERS: usize = 5;
/// What the four sites cost together (`factions/cults/units/recruit.md > Constructs`), and a Militant
/// (`factions/cults/structures/armory.md > Training`).
const SITES_PRICE: u32 = 300 + 200 + 150 + 150;
const MILITANT_PRICE: u32 = 20;
const RECRUITS: usize = GATHERERS + SITES.len() * BUILDERS + ARMORIES.len() * HELD;

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

/// The game's objects, by their ids.
struct Ids {
    /// Each patch, with the Recruits dealt to it.
    deals: Vec<(ObjectId, Vec<ObjectId>)>,
    armories: Vec<ObjectId>,
}

fn tile(x: i32, y: i32) -> SpawnAt {
    SpawnAt::Tile(TilePos::new(x, y))
}

/// A unit's centre in the middle of tile (`x`, `y`).
fn on_tile(x: i32, y: i32) -> SpawnAt {
    SpawnAt::Point(TilePos::new(x, y).corner() + Vec2::splat(32.0))
}

/// "Q: Train Militant" in the Armory's KindMenu of its plain Recruits.
fn train(armory: ObjectId) -> Command {
    Command::Production {
        objects: vec![armory],
        command: ProductionCommand::TrainFrom {
            kind: ObjectKind::Militant,
            from: RecruitKind::Plain,
        },
    }
}

/// The game, with its orders of tick 1, and the second Armory's further Militants and its destruction scheduled. Ids
/// in spawn order: the patches, the Storages, the Halls, the Armories, then the Recruits (the gatherers, the builders,
/// the Armories').
fn game() -> (TestSim, Ids) {
    let map = vec![".".repeat(WIDTH); HEIGHT].join("\n");
    let mut builder = ScenarioBuilder::new(&map).faction(P1, Faction::Cults);
    let mut last = 0;
    let mut id = || {
        last += 1;
        ObjectId(last)
    };
    let mut deals = Vec::new();
    for y in [FIELD_Y, FIELD_Y + 5] {
        for x in 0..ROW {
            builder = builder.spawn(ObjectKind::CrystalPatch, None, tile(FIELD_X + x, y));
            deals.push((id(), Vec::new()));
        }
    }
    for x in STORAGES {
        builder = builder.spawn(ObjectKind::Storage, Some(P1), tile(FIELD_X + x, FIELD_Y + 2));
        id();
    }
    for (x, y) in HALLS {
        builder = builder.spawn(ObjectKind::RecruitmentHall, Some(P1), tile(x, y));
        id();
    }
    let mut armories = Vec::new();
    for (x, y) in ARMORIES {
        builder = builder.spawn(ObjectKind::Armory, Some(P1), tile(x, y));
        armories.push(id());
    }
    // The gatherers: half north of the north row, which they are dealt to, and half south of the south row, in blocks
    // 10 wide, 40 space units apart, 2 tiles clear of the rows (`recruits_perf`'s).
    for i in 0..GATHERERS {
        let (row, place) = (i % 2, i / 2);
        let (column, line) = (place % 10, place / 10);
        let corner = TilePos::new(FIELD_X, if row == 0 { FIELD_Y - 2 } else { FIELD_Y + 8 }).corner();
        let down = if row == 0 { -1.0 } else { 1.0 };
        let at = corner + Vec2::new(60.0 + 40.0 * column as f32, down * (20.0 + 40.0 * line as f32));
        builder = builder.spawn(ObjectKind::Recruit, Some(P1), SpawnAt::Point(at));
        deals[row * ROW as usize + place % ROW as usize].1.push(id());
    }
    // The builders: a row of five 3 tiles north of each site's north-west tile, eastwards.
    let mut groups = Vec::new();
    for &(_, x, y) in &SITES {
        let mut group = Vec::new();
        for k in 0..BUILDERS as i32 {
            builder = builder.spawn(ObjectKind::Recruit, Some(P1), on_tile(x + k, y - 3));
            group.push(id());
        }
        groups.push(group);
    }
    // Each Armory's: two columns of five, 2 and 3 tiles east of its entrance side.
    let mut held = Vec::new();
    for (x, y) in ARMORIES {
        let mut block = Vec::new();
        for k in 0..HELD as i32 {
            builder = builder.spawn(ObjectKind::Recruit, Some(P1), on_tile(x + 5 + k / 5, y - 2 + k % 5));
            block.push(id());
        }
        held.push(block);
    }
    let scenario = builder.build();
    assert_eq!(scenario.validate(), vec![]);
    let mut t = TestSim::new(scenario);
    t.check_invariants(false);
    t.debug_at(
        1,
        DebugCommand::SetResources {
            player: P1,
            crystals: BANK,
            supplies: 0,
            alloys: 0,
            essence: 0,
            flux: 0,
        },
    );
    for (patch, recruits) in &deals {
        t.order_at(1, P1, recruits, OrderKind::Gather(*patch), false);
    }
    for (&(kind, x, y), group) in SITES.iter().zip(&groups) {
        let build = OrderKind::Build {
            kind,
            origin: TilePos::new(x, y),
            orientation: Default::default(),
        };
        t.order_at(1, P1, group, build, false);
    }
    for (&armory, block) in armories.iter().zip(&held) {
        t.order_at(1, P1, block, OrderKind::Enter(armory), false);
    }
    for _ in 0..MORE_TRAINED {
        t.at(MORE, P1, train(armories[1]));
    }
    t.debug_at(DESTROYED, DebugCommand::Remove { id: armories[1] });
    (t, Ids { deals, armories })
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
fn three_halls_eighty_recruits_four_sites_two_armories_and_an_evacuation() {
    let (mut t, ids) = game();
    let count = |t: &TestSim, kind: ObjectKind| t.sim().objects().filter(|o| o.kind == kind).count();
    assert_eq!(count(&t, ObjectKind::Recruit), RECRUITS);
    let mut queued = [false; 2];
    let mut ticks: Vec<(Tick, [Duration; PHASES])> = Vec::new();
    while t.sim().tick() < TICKS {
        let times = timed_step(&mut t);
        let tick = t.sim().tick();
        ticks.push((tick, times));
        // Each Armory trains once all its Recruits are in.
        for (k, &armory) in ids.armories.iter().enumerate() {
            if !queued[k] && t.sim().held_in(armory).len() == HELD {
                queued[k] = true;
                for _ in 0..TRAINED[k] {
                    t.at(tick + 1, P1, train(armory));
                }
            }
        }
    }
    let of = |phases: &[Phase], keep: &dyn Fn(Tick) -> bool| -> Vec<(Tick, Duration)> {
        (ticks.iter().filter(|(tick, _)| keep(*tick)))
            .map(|(tick, times)| (*tick, phases.iter().map(|&p| times[p as usize]).sum()))
            .collect()
    };
    let whole = &Phase::ALL[..PHASES];
    println!("tick 1: the orders of the {RECRUITS} Recruits (22 Gathers, four group Builds, two Enters)");
    println!("{}", line("whole tick", &of(whole, &|tick| tick == 1)));
    println!("tick {DESTROYED}: the second Armory destroyed, its Recruits evacuated");
    println!("{}", line("whole tick", &of(whole, &|tick| tick == DESTROYED)));
    println!("ticks 2 to {TICKS} but {DESTROYED}: the economy at work");
    let rest = |tick| tick > 1 && tick != DESTROYED;
    println!("{}", line("whole tick", &of(whole, &rest)));
    for &phase in whole {
        println!("{}", line(&format!("{phase:?}"), &of(&[phase], &rest)));
    }

    // The game. Every site is built.
    let built: Vec<(Tick, ObjectKind)> = (t.events().iter())
        .filter_map(|(tick, e)| match *e {
            SimEvent::SiteBuilt { site } => t.sim().object(site).map(|o| (*tick, o.kind)),
            _ => None,
        })
        .collect();
    println!("sites built: {built:?}");
    let mut kinds: Vec<ObjectKind> = built.iter().map(|b| b.1).collect();
    kinds.sort();
    let mut planned: Vec<ObjectKind> = SITES.iter().map(|s| s.0).collect();
    planned.sort();
    assert_eq!(kinds, planned, "every site built");
    // The Militants: all of the first Armory's, and the second's first five. The Halls produce, the fourth too once
    // it is built, and nothing dies.
    let produced = |producer: ObjectId| {
        (t.events().iter())
            .filter(|(_, e)| matches!(*e, SimEvent::UnitProduced { producer: p, .. } if p == producer))
            .count()
    };
    let militants = count(&t, ObjectKind::Militant);
    let halls: Vec<usize> = (t.sim().objects())
        .filter(|o| o.kind == ObjectKind::RecruitmentHall)
        .map(|o| produced(o.id))
        .collect();
    println!(
        "Militants: {} and {} trained, {militants} on the map; the Halls produced {halls:?} Recruits",
        produced(ids.armories[0]),
        produced(ids.armories[1])
    );
    assert_eq!(produced(ids.armories[0]), TRAINED[0]);
    assert_eq!(produced(ids.armories[1]), TRAINED[1]);
    assert_eq!(militants, TRAINED[0] + TRAINED[1]);
    assert!(
        halls.len() == 4 && halls.iter().all(|&n| n > 0),
        "every Hall produces: {halls:?}"
    );
    assert!(!t.events().iter().any(|(_, e)| matches!(e, SimEvent::Died { .. })));
    // The destruction: the five still inside come out as Recruits in its tick, and the three entries are refunded.
    let evacuated: Vec<ObjectId> = (t.events().iter())
        .filter_map(|(tick, e)| match *e {
            SimEvent::Evacuated { unit, building } if *tick == DESTROYED && building == ids.armories[1] => Some(unit),
            _ => None,
        })
        .collect();
    let refunds: u32 = (t.events().iter())
        .filter_map(|(tick, e)| match *e {
            SimEvent::ProductionCancelled { producer, refund, .. }
                if *tick == DESTROYED && producer == ids.armories[1] =>
            {
                Some(refund)
            }
            _ => None,
        })
        .sum();
    println!(
        "evacuated: {} Recruits; refunded: {refunds} Space Crystals",
        evacuated.len()
    );
    assert_eq!(evacuated.len(), HELD - TRAINED[1]);
    for unit in &evacuated {
        assert_eq!(t.sim().object(*unit).map(|o| o.kind), Some(ObjectKind::Recruit));
    }
    assert_eq!(refunds, MORE_TRAINED as u32 * MILITANT_PRICE);
    // The bank: every load delivered is in it, the sites and the Militants paid, the three entries refunded.
    let delivered: Vec<(Tick, ObjectId, u32)> = (t.events().iter())
        .filter_map(|(tick, e)| match *e {
            SimEvent::ResourcesDelivered { worker, amount, .. } => Some((*tick, worker, amount)),
            _ => None,
        })
        .collect();
    let total: u32 = delivered.iter().map(|d| d.2).sum();
    let spent = SITES_PRICE + (TRAINED[0] + TRAINED[1]) as u32 * MILITANT_PRICE;
    println!(
        "{total} Space Crystals delivered in {TICKS} ticks; {} Recruits at the end",
        count(&t, ObjectKind::Recruit)
    );
    assert_eq!(t.sim().player(P1).expect("in the game").crystals, BANK + total - spent);
    assert!(total > 1000, "the field works: {total}");
    // The gatherers all gather still.
    let gathering = (t.sim().objects())
        .filter(|o| ids.deals.iter().any(|(_, recruits)| recruits.contains(&o.id)))
        .filter(|o| {
            let current = o.orders.as_ref().and_then(|orders| orders.current.as_ref());
            current.is_some_and(|active| matches!(active.order.kind, OrderKind::Gather(_)))
        })
        .count();
    assert_eq!(gathering, GATHERERS, "a Gather never ends on its own");
}
