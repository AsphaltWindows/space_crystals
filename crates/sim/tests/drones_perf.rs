//! Timings of whole ticks with two Colonists economies of 40 gathering Drones each and 20 Legionnaires fighting
//! between them (slice M3-K2, `artifacts/director/m3-plan.md > K2: M3 QA`, and the plan's risk 9: "Perf with many
//! Drones: path requests, crowd collision, the per-tick claims"), phase by phase (`Sim::step_observed`). Ignored by
//! default:
//!
//! ```sh
//! cargo test -p sc_sim --release --test drones_perf -- --ignored --nocapture
//! ```
//!
//! On a flat 128 × 48 map each player has a field at its end: 16 patches in two rows of 8, with the Egress Node
//! between them, 2 grid units of open ground from either row (the reference geometry of `factions/gdo/economy.md >
//! Economy Targets (GDO vs Colonists) > Reference geometry`, twice, back to back), and 40 Drones, two and a half to a
//! patch, short of the three that saturate one (`tests/gathering_soak.rs`). The Drones are dealt out to the patches
//! in tick 1, from blocks north and south of the field, and gather all along: they claim, mine, deliver, wait and
//! re-check, and are sent on from patch to patch as their patches are taken. Between the fields each player's five
//! Sabre and five Rifle Legionnaires attack-move at the other's and fight in the middle, Shields and all, out of
//! sight of the Drones.
//!
//! It prints, for the whole tick and for each phase, the mean, the 99th percentile (which a busy machine's preemption
//! inflates less than the worst) and the worst tick, flagging a worst tick over 10 ms (slice M2-J2's target for a
//! whole tick); and the whole tick in the waiting Drones' re-check ticks apart from the others, since every waiting
//! Drone looks round in the same ticks (`factions/colonists/units.md > Patch Occupancy > Waiting and re-checking`).
//! Tick 1, with the 32 Gathers of 80 Drones and the two group orders, is a section of its own. The assertions are on
//! the game, never on wall time.

use std::time::Duration;

use sc_sim::testing::{ScenarioBuilder, TestSim};
use sc_sim::{Faction, ObjectId, ObjectKind, OrderKind, Phase, PlayerId, SimEvent, SpawnAt, Tick, TilePos, Vec2};

const WIDTH: usize = 128;
const HEIGHT: usize = 48;
/// Slice M2-J2's target for a whole tick, at worst.
const BUDGET: Duration = Duration::from_millis(10);
/// The phases that take time ([`Phase::End`] only marks the end).
const PHASES: usize = Phase::ALL.len() - 1;
/// Each player's Drones, and the patches of each of its field's two rows.
const DRONES: usize = 40;
const ROW: i32 = 8;
/// The fields' north rows; the south rows are 7 tiles below, the Egress Nodes between them.
const FIELD_Y: i32 = 18;
/// Each player's Legionnaires, half Sabres and half Rifles.
const LEGIONNAIRES: usize = 10;
/// A waiting Drone looks round again in the ticks that are multiples of 16 (`orders::gather::RECHECK_TICKS`).
const RECHECK_TICKS: Tick = 16;
/// The game is timed to 3 minutes.
const TICKS: Tick = 3 * 60 * 16;

/// The wall-clock time of each phase of one tick of `t` (`Sim::step_observed`), in [`Phase`]'s order. Only this test
/// reads a clock: it measures the sim from outside, and nothing in the sim can call it (`ARCHITECTURE.md >
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

/// A player's side of the game: its field's patches by row, its Drones by the patch they are dealt to, and its
/// Legionnaires.
struct Side {
    player: PlayerId,
    deals: Vec<(ObjectId, Vec<ObjectId>)>,
    legionnaires: Vec<ObjectId>,
}

/// The spawns of the scenario, which take their ids in order.
#[derive(Default)]
struct Spawns(Vec<(ObjectKind, Option<PlayerId>, SpawnAt)>);

impl Spawns {
    fn spawn(&mut self, kind: ObjectKind, owner: Option<PlayerId>, at: SpawnAt) -> ObjectId {
        self.0.push((kind, owner, at));
        ObjectId(self.0.len() as u32)
    }

    /// `player`'s side: its field west of tile column `x` and the 8 columns from it, its Legionnaires in a block
    /// with its top-left tile at `army`.
    fn side(&mut self, player: PlayerId, x: i32, army: TilePos) -> Side {
        let rows = [FIELD_Y, FIELD_Y + 7].map(|y| {
            (0..ROW)
                .map(|i| self.spawn(ObjectKind::CrystalPatch, None, SpawnAt::Tile(TilePos::new(x + i, y))))
                .collect::<Vec<_>>()
        });
        let node = TilePos::new(x + ROW / 2 - 1, FIELD_Y + 3);
        self.spawn(ObjectKind::EgressNode, Some(player), SpawnAt::Tile(node));
        // Half the Drones north of the north row, which they are dealt to, and half south of the south row: blocks
        // 5 wide, 48 space units apart, 1 tile clear of the rows.
        let mut deals: Vec<(ObjectId, Vec<ObjectId>)> = rows.iter().flatten().map(|&p| (p, Vec::new())).collect();
        for i in 0..DRONES {
            let (row, place) = (i % 2, i / 2);
            let (column, line) = (place % 5, place / 5);
            let corner = TilePos::new(x, if row == 0 { FIELD_Y - 1 } else { FIELD_Y + 9 }).corner();
            let down = if row == 0 { -1.0 } else { 1.0 };
            let at = corner + Vec2::new(64.0 + 48.0 * column as f32, down * (24.0 + 48.0 * line as f32));
            let drone = self.spawn(ObjectKind::Drone, Some(player), SpawnAt::Point(at));
            deals[row * ROW as usize + place % ROW as usize].1.push(drone);
        }
        let legionnaires = (0..LEGIONNAIRES)
            .map(|i| {
                let kind = if i % 2 == 0 {
                    ObjectKind::SabreLegionnaire
                } else {
                    ObjectKind::RifleLegionnaire
                };
                let at = army.corner() + Vec2::new(32.0 + 64.0 * (i % 2) as f32, 32.0 + 64.0 * (i / 2) as f32);
                self.spawn(kind, Some(player), SpawnAt::Point(at))
            })
            .collect();
        Side {
            player,
            deals,
            legionnaires,
        }
    }
}

/// The two sides on the map, and the game: player 1's field in the west and player 2's in the east, 96 tiles apart;
/// the Legionnaires in front of them, 22 tiles apart in the middle, each block attack-moving to the other's.
fn game() -> (TestSim, [Side; 2]) {
    let mut b = Spawns::default();
    let armies = [TilePos::new(52, 19), TilePos::new(74, 19)];
    let sides = [
        b.side(PlayerId(1), 8, armies[0]),
        b.side(PlayerId(2), WIDTH as i32 - 8 - ROW, armies[1]),
    ];
    let map = vec![".".repeat(WIDTH); HEIGHT].join("\n");
    let builder = (b.0.into_iter()).fold(
        ScenarioBuilder::new(&map)
            .faction(PlayerId(1), Faction::Colonists)
            .faction(PlayerId(2), Faction::Colonists),
        |builder, (kind, owner, at)| builder.spawn(kind, owner, at),
    );
    let scenario = builder.build();
    assert_eq!(scenario.validate(), vec![]);
    let mut t = TestSim::new(scenario);
    t.check_invariants(false);
    for (side, enemy) in sides.iter().zip(armies.iter().rev()) {
        for (patch, drones) in &side.deals {
            t.order_at(1, side.player, drones, OrderKind::Gather(*patch), false);
        }
        let to = TilePos::new(enemy.x + 1, enemy.y + 2).center();
        t.order_at(1, side.player, &side.legionnaires, OrderKind::AttackMove(to), false);
    }
    (t, sides)
}

/// Each tick's phases.
#[derive(Default)]
struct Timings {
    ticks: Vec<(Tick, [Duration; PHASES])>,
}

impl Timings {
    /// The whole tick's times, or a group of phases', in the ticks `keep` keeps.
    fn of(&self, phases: &[Phase], keep: impl Fn(Tick) -> bool) -> Vec<(Tick, Duration)> {
        let sum = |times: &[Duration; PHASES]| phases.iter().map(|&p| times[p as usize]).sum();
        (self.ticks.iter().filter(|(tick, _)| keep(*tick)))
            .map(|(tick, times)| (*tick, sum(times)))
            .collect()
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
}

/// Runs `t` until `until` holds after a tick, or to tick `last`, timing each tick; prints the whole tick's times.
fn measure(label: &str, t: &mut TestSim, last: Tick, until: impl Fn(&TestSim) -> bool, all: &mut Timings) {
    let mut timings = Timings::default();
    while t.sim().tick() < last {
        let times = timed_step(t);
        timings.ticks.push((t.sim().tick(), times));
        if until(t) {
            break;
        }
    }
    let from = timings.ticks.first().map_or(0, |(tick, _)| *tick);
    println!("{label}: {} ticks from tick {from}", timings.ticks.len());
    println!(
        "{}",
        Timings::line("whole tick", &timings.of(&Phase::ALL[..PHASES], |_| true))
    );
    all.ticks.extend(timings.ticks);
}

/// `player`'s Legionnaires still in the game.
fn legionnaires(t: &TestSim, player: u8) -> usize {
    (t.sim().objects())
        .filter(|o| o.owner == Some(PlayerId(player)))
        .filter(|o| matches!(o.kind, ObjectKind::SabreLegionnaire | ObjectKind::RifleLegionnaire))
        .count()
}

#[test]
#[ignore = "timing: run with --release --ignored --nocapture"]
fn two_fields_of_40_drones_and_20_legionnaires() {
    let (mut t, sides) = game();
    let drones = |t: &TestSim| t.sim().objects().filter(|o| o.kind == ObjectKind::Drone).count();
    assert_eq!(drones(&t), 2 * DRONES);
    let mut all = Timings::default();
    measure(
        "tick 1: the 32 Gathers and the two AttackMoves",
        &mut t,
        1,
        |_| true,
        &mut all,
    );
    measure(
        "the Drones setting off and the Legionnaires closing in, up to the first shot",
        &mut t,
        TICKS,
        |t| t.sim().events().iter().any(|e| matches!(e, SimEvent::Attacked { .. })),
        &mut all,
    );
    measure(
        "the fight, until one side's Legionnaires are all dead",
        &mut t,
        TICKS,
        |t| legionnaires(t, 1) == 0 || legionnaires(t, 2) == 0,
        &mut all,
    );
    measure("the Drones gathering on", &mut t, TICKS, |_| false, &mut all);

    let delivered = |player: PlayerId| -> u32 {
        (t.events().iter())
            .filter_map(|(_, e)| match *e {
                SimEvent::ResourcesDelivered { worker, amount, .. } => {
                    let owner = sides.iter().find(|s| s.deals.iter().any(|(_, d)| d.contains(&worker)));
                    (owner.map(|s| s.player) == Some(player)).then_some(amount)
                }
                _ => None,
            })
            .sum()
    };
    let died = t
        .events()
        .iter()
        .filter(|(_, e)| matches!(e, SimEvent::Died { .. }))
        .count();
    println!(
        "at tick {}: delivered {:?}, Legionnaires left {:?}, {died} died, {} objects",
        t.sim().tick(),
        sides.each_ref().map(|s| delivered(s.player)),
        [1, 2].map(|p| legionnaires(&t, p)),
        t.sim().objects().count()
    );

    println!("every tick, by phase:");
    let lines: [(&str, &[Phase]); 14] = [
        ("whole tick", &Phase::ALL[..PHASES]),
        ("defeat check, walk 1 (owners)", &[Phase::DefeatOwners]),
        ("commands", &[Phase::Commands]),
        ("behaviours (5a)", &[Phase::Behaviours]),
        ("true grids (5b)", &[Phase::NavSync]),
        ("path serving (5c)", &[Phase::PathServing]),
        ("movement (5d)", &[Phase::Movement]),
        ("combat (5e)", &[Phase::Combat]),
        ("deaths and true grids (5e)", &[Phase::Deaths]),
        ("shield regen", &[Phase::Shields]),
        ("economy", &[Phase::Economy]),
        ("defeat check, walk 2", &[Phase::Defeat]),
        ("vision (5f) and records", &[Phase::Vision]),
        ("path knowledge and its grids", &[Phase::PathKnowledge]),
    ];
    for (label, phases) in lines {
        println!("{}", Timings::line(label, &all.of(phases, |_| true)));
    }
    let recheck = |tick: Tick| tick.is_multiple_of(RECHECK_TICKS);
    let after_tick_1 = |tick: Tick| tick > 1;
    let whole = &Phase::ALL[..PHASES];
    println!(
        "{}",
        Timings::line(
            "whole tick, re-check ticks",
            &all.of(whole, |tick| after_tick_1(tick) && recheck(tick))
        )
    );
    println!(
        "{}",
        Timings::line(
            "whole tick, other ticks after tick 1",
            &all.of(whole, |tick| after_tick_1(tick) && !recheck(tick))
        )
    );
    println!(
        "{}",
        Timings::line(
            "behaviours (5a), re-check ticks",
            &all.of(&[Phase::Behaviours], |tick| after_tick_1(tick) && recheck(tick))
        )
    );

    // The game, not the clock: both fields gathered all along, the Legionnaires fought, and nobody was defeated.
    for side in &sides {
        assert!(
            delivered(side.player) > 0,
            "player {}'s Drones delivered",
            side.player.0
        );
    }
    assert!(
        legionnaires(&t, 1) < LEGIONNAIRES && legionnaires(&t, 2) < LEGIONNAIRES,
        "Legionnaires died on either side"
    );
    assert_eq!(drones(&t), 2 * DRONES, "no Drone was lost");
    assert!(t.sim().outcome().is_none());
}
