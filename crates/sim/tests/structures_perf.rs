//! Timings of whole ticks with four GDO bases of 40 structures each and 400 units fighting through them (slice
//! M2-J2), phase by phase (`Sim::step_observed`), against the target of a whole tick within 10 ms at worst.
//! Ignored by default:
//!
//! ```sh
//! cargo test -p sc_sim --release --test structures_perf -- --ignored --nocapture
//! ```
//!
//! Two battles on a flat 256 × 256 map: in the north player 1's 130 Peacekeepers against player 2's 70, in the south
//! player 3's 130 against player 4's 70, each army sweeping the other's base row by row with attack-moves. The units
//! fight each other, then the structures: the true grids are rebuilt as structures fall, each player's planning grids
//! as it sees them stand or fall, and the defeat check walks the objects twice every tick. The economy runs all
//! along: each base mines with 10 plates and trains Peacekeepers in its 10 Barracks.
//!
//! It prints, for the whole tick and for each phase, the mean, the 99th percentile (which a busy machine's preemption
//! inflates less than the worst) and the worst tick, flagging a worst tick over 10 ms, and how many grids were
//! rebuilt per tick. Tick 1, in which the four players give their 20 group orders at once, is a section of its own:
//! each group order is pathed in its tick (`docs/movement.md > 5.8`), so it is the worst tick by far. The assertions
//! are on the game, never on wall time.

use std::time::Duration;

use sc_sim::testing::{ScenarioBuilder, TestSim};
use sc_sim::{
    Command, DebugCommand, GameSetup, ObjectId, ObjectKind, OrderKind, Phase, PlayerId, ProductionCommand, Scenario,
    SimEvent, Slot, SpawnAt, Tick, TilePos, Vec2,
};

const SIDE: usize = 256;
/// The target of slice M2-J2 for a whole tick, at worst.
const BUDGET: Duration = Duration::from_millis(10);
/// The phases that take time ([`Phase::End`] only marks the end).
const PHASES: usize = Phase::ALL.len() - 1;

/// A base: 8 × 5 cells of [`CELL`] tiles, each with a structure at its top-left, which leaves lanes of 2 tiles or
/// more between them: P a plate on a patch, G a Generator, B a Barracks (its doorway on the open east side), D the
/// Deployment Center, E the Extraction Facility. 40 structures, with power to spare (+45).
const BASE: [&str; 5] = ["PPPPPPPP", "GGBGGBGG", "GBGDEGBG", "GGBGGBGG", "BBPPGBBG"];
const CELL: i32 = 6;
const BASE_WIDTH: i32 = 8 * CELL;

/// The armies: the bigger one of each battle, and the smaller.
const BIG: u32 = 130;
const SMALL: u32 = 70;

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

/// A player's side of the game: its base's top-left tile, its Barracks and its army, by the row of the enemy base
/// each group sweeps.
struct Side {
    player: PlayerId,
    base: TilePos,
    barracks: Vec<ObjectId>,
    groups: [Vec<ObjectId>; 5],
}

/// The spawns of the scenario, which take their ids in order: each player's base, then each player's army.
#[derive(Default)]
struct Spawns(Vec<(ObjectKind, Option<PlayerId>, SpawnAt)>);

impl Spawns {
    fn spawn(&mut self, kind: ObjectKind, owner: Option<PlayerId>, at: SpawnAt) -> ObjectId {
        self.0.push((kind, owner, at));
        ObjectId(self.0.len() as u32)
    }

    /// [`BASE`] for `player`, its top-left tile at `base`: its Barracks.
    fn base(&mut self, player: PlayerId, base: TilePos) -> Vec<ObjectId> {
        let mut barracks = Vec::new();
        for (row, cells) in BASE.iter().enumerate() {
            for (column, cell) in cells.chars().enumerate() {
                let tile = TilePos::new(base.x + column as i32 * CELL, base.y + row as i32 * CELL);
                let at = SpawnAt::Tile(tile);
                let kind = match cell {
                    'P' => {
                        self.spawn(ObjectKind::CrystalPatch, None, at);
                        ObjectKind::ExtractionPlate
                    }
                    'G' => ObjectKind::Generator,
                    'B' => ObjectKind::Barracks,
                    'D' => ObjectKind::DeploymentCenter,
                    'E' => ObjectKind::ExtractionFacility,
                    _ => unreachable!(),
                };
                let id = self.spawn(kind, Some(player), at);
                if kind == ObjectKind::Barracks {
                    barracks.push(id);
                }
            }
        }
        barracks
    }

    /// `count` Peacekeepers of `player` in a block 10 wide, its top-left tile at `at`, in 5 groups dealt in turn.
    fn army(&mut self, player: PlayerId, count: u32, at: TilePos) -> [Vec<ObjectId>; 5] {
        let mut groups: [Vec<ObjectId>; 5] = Default::default();
        for i in 0..count {
            let offset = Vec2::new((i % 10) as f32, (i / 10) as f32) * 48.0 + Vec2::splat(24.0);
            let pos = at.corner() + offset;
            let id = self.spawn(ObjectKind::Peacekeeper, Some(player), SpawnAt::Point(pos));
            groups[i as usize % 5].push(id);
        }
        groups
    }
}

/// The four sides on the map, and the game: player 1's base in the north-west against player 2's in the north-east,
/// player 3's in the south-west against player 4's in the south-east, 60 tiles apart; the armies stand in front of
/// their own bases, 32 tiles apart.
fn game() -> (TestSim, Vec<Side>) {
    let mut b = Spawns::default();
    let bases = [(1, 50, 40), (2, 158, 40), (3, 50, 170), (4, 158, 170)];
    let barracks: Vec<Vec<ObjectId>> = bases
        .iter()
        .map(|&(p, x, y)| b.base(PlayerId(p), TilePos::new(x, y)))
        .collect();
    let mut sides = Vec::new();
    for (&(p, x, y), barracks) in bases.iter().zip(barracks) {
        let west = p % 2 == 1;
        let (count, army_x) = if west {
            (BIG, x + BASE_WIDTH + 2)
        } else {
            (SMALL, x - 10)
        };
        let groups = b.army(PlayerId(p), count, TilePos::new(army_x, y + 10));
        sides.push(Side {
            player: PlayerId(p),
            base: TilePos::new(x, y),
            barracks,
            groups,
        });
    }
    let map = vec![".".repeat(SIDE); SIDE].join("\n");
    let builder =
        b.0.into_iter()
            .fold(ScenarioBuilder::new(&map), |builder, (kind, owner, at)| {
                builder.spawn(kind, owner, at)
            });
    let mut scenario: Scenario = builder.build();
    scenario.map.slots = bases
        .iter()
        .map(|&(_, x, y)| Slot::new(TilePos::new(x + BASE_WIDTH / 2, y + 15)))
        .collect();
    scenario.setup = GameSetup::default_for(&scenario.map);
    assert_eq!(scenario.validate(), vec![]);
    let mut t = TestSim::new(scenario);
    t.check_invariants(false);

    // Tick 1: crystals for every Barracks to queue 5 Peacekeepers, the queues, and the sweeps: each group of an army
    // attack-moves along the lane under its row of the enemy base, to beyond it.
    for side in &sides {
        let enemy = &sides[(side.player.0 as usize - 1) ^ 1];
        let resources = DebugCommand::SetResources {
            player: side.player,
            crystals: 100_000,
            supplies: 0,
        };
        t.at(1, side.player, Command::Debug(resources));
        for _ in 0..5 {
            let train = Command::Production {
                objects: side.barracks.clone(),
                command: ProductionCommand::Train(ObjectKind::Peacekeeper),
            };
            t.at(1, side.player, train);
        }
        let beyond = if enemy.base.x > side.base.x {
            enemy.base.x + BASE_WIDTH + 4
        } else {
            enemy.base.x - 5
        };
        for (row, group) in side.groups.iter().enumerate() {
            let lane = TilePos::new(beyond, enemy.base.y + row as i32 * CELL + 5);
            t.order_at(1, side.player, group, OrderKind::AttackMove(lane.center()), false);
        }
    }
    (t, sides)
}

/// Each tick's phases and grid rebuilds.
#[derive(Default)]
struct Timings {
    ticks: Vec<(Tick, [Duration; PHASES], u32)>,
}

impl Timings {
    /// The whole tick's times, or a group of phases'.
    fn of(&self, phases: &[Phase]) -> Vec<(Tick, Duration)> {
        let sum = |times: &[Duration; PHASES]| phases.iter().map(|&p| times[p as usize]).sum();
        self.ticks.iter().map(|(tick, times, _)| (*tick, sum(times))).collect()
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

    /// The grids rebuilt: in all, in how many ticks, at most in one.
    fn rebuilds(&self) -> String {
        let counts: Vec<u32> = self.ticks.iter().map(|(.., n)| *n).collect();
        let ticks = counts.iter().filter(|&&n| n > 0).count();
        let most = counts.iter().max().copied().unwrap_or(0);
        let all: u32 = counts.iter().sum();
        format!("  grids rebuilt: {all} in {ticks} ticks, at most {most} in a tick")
    }
}

/// Runs `t` until `until` holds after a tick, or for `max_ticks`, timing each tick; prints the whole tick's times.
fn measure(label: &str, t: &mut TestSim, max_ticks: u32, until: impl Fn(&TestSim) -> bool, all: &mut Timings) {
    let mut timings = Timings::default();
    for _ in 0..max_ticks {
        let times = timed_step(t);
        timings.ticks.push((t.sim().tick(), times, t.sim().nav_rebuilds()));
        if until(t) {
            break;
        }
    }
    let from = timings.ticks.first().map_or(0, |(tick, ..)| *tick);
    println!("{label}: {} ticks from tick {from}", timings.ticks.len());
    println!("{}", Timings::line("whole tick", &timings.of(&Phase::ALL[..PHASES])));
    println!("{}", timings.rebuilds());
    all.ticks.extend(timings.ticks);
}

fn defeated(t: &TestSim, player: u8) -> bool {
    t.sim().player(PlayerId(player)).is_some_and(|p| p.is_defeated())
}

fn structures(t: &TestSim, player: u8) -> usize {
    t.sim()
        .objects()
        .filter(|o| o.owner == Some(PlayerId(player)) && o.placement.is_some())
        .count()
}

#[test]
#[ignore = "timing: run with --release --ignored --nocapture"]
fn four_bases_and_400_units() {
    let (mut t, sides) = game();
    assert_eq!(sides.iter().map(|s| s.barracks.len()).sum::<usize>(), 40);
    assert_eq!(
        t.sim().objects().filter(|o| o.placement.is_some()).count(),
        4 * (40 + 10)
    );
    let mut all = Timings::default();
    // Tick 1 alone: 20 group orders at once, each pathed in its tick (`docs/movement.md > 5.8`).
    measure(
        "tick 1: the 40 queues and the 20 group orders",
        &mut t,
        1,
        |_| true,
        &mut all,
    );
    measure(
        "the armies closing in, up to the first shot",
        &mut t,
        2000,
        |t| t.sim().events().iter().any(|e| matches!(e, SimEvent::Attacked { .. })),
        &mut all,
    );
    measure(
        "the battles and the sweeps, until the small armies' players are defeated",
        &mut t,
        6000,
        |t| defeated(t, 2) && defeated(t, 4),
        &mut all,
    );
    let fallen = t
        .events()
        .iter()
        .filter(|(_, e)| matches!(e, SimEvent::Died { kind, .. } if kind.def().structure.is_some()))
        .count();
    let left: Vec<usize> = (1..=4).map(|p| structures(&t, p)).collect();
    println!(
        "at tick {}: {fallen} structures destroyed, structures left per player {left:?}, {} objects",
        t.sim().tick(),
        t.sim().objects().count()
    );

    println!("every tick, by phase:");
    let lines: [(&str, &[Phase]); 13] = [
        ("whole tick", &Phase::ALL[..PHASES]),
        ("defeat check, walk 1 (owners)", &[Phase::DefeatOwners]),
        ("commands", &[Phase::Commands]),
        ("behaviours (5a)", &[Phase::Behaviours]),
        ("true grids (5b)", &[Phase::NavSync]),
        ("path serving (5c)", &[Phase::PathServing]),
        ("movement (5d)", &[Phase::Movement]),
        ("combat (5e)", &[Phase::Combat]),
        ("deaths and true grids (5e)", &[Phase::Deaths]),
        ("economy", &[Phase::Economy]),
        ("defeat check, walk 2", &[Phase::Defeat]),
        ("vision (5f) and records", &[Phase::Vision]),
        ("path knowledge and its grids", &[Phase::PathKnowledge]),
    ];
    for (label, phases) in lines {
        println!("{}", Timings::line(label, &all.of(phases)));
    }
    let grids = [Phase::NavSync, Phase::Deaths, Phase::PathKnowledge];
    println!(
        "{}",
        Timings::line("every grid sync (5b, 5e, knowledge)", &all.of(&grids))
    );
    println!(
        "{}",
        Timings::line("both defeat walks", &all.of(&[Phase::DefeatOwners, Phase::Defeat]))
    );
    println!("{}", all.rebuilds());
    let with_rebuilds: Vec<(Tick, Duration)> = all
        .of(&Phase::ALL[..PHASES])
        .into_iter()
        .zip(&all.ticks)
        .filter(|(_, (.., n))| *n > 0)
        .map(|(time, _)| time)
        .collect();
    println!("{}", Timings::line("whole tick, ticks with a rebuild", &with_rebuilds));

    assert!(defeated(&t, 2) && defeated(&t, 4), "the small armies' bases fall");
    assert!(!defeated(&t, 1) && !defeated(&t, 3) && t.sim().outcome().is_none());
    assert!(fallen >= 2 * 40, "{fallen} structures destroyed");
}
