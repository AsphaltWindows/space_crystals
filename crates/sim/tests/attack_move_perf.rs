//! Timings of AttackMove's reach rule (`orders::reach`, design change 0230: "Performance is a priority here (large
//! armies attack-moving past many unreachable targets)"): 100 Guards attack-move along a river 3 tiles wide with
//! enemies standing on the far bank, in their sight and out of their Range: 100 unarmed Drones, or 30 Generators
//! (slice M3-L5: design change 0243 has each rejection judged again for every grid unit the unit walks with it in its
//! sight, "to be revisited if too expensive"). Ignored by default:
//!
//! ```sh
//! cargo test -p sc_sim --release --test perf attack_move_perf -- --ignored --nocapture --test-threads=1
//! ```
//!
//! Two maps: the river from edge to edge (the far bank is another component: every judged path ends on the near
//! bank), and the river with a crossing at the map's north edge, some 200 tiles away (every judged path is a long
//! search round it: the worst case). Timings and counts are printed, never asserted.

use std::time::Duration;

use sc_sim::testing::{ScenarioBuilder, TestSim};
use sc_sim::{DebugCommand, ObjectId, ObjectKind, OrderKind, Phase, PlayerId, SimEvent, SpawnAt, TilePos, Vec2};

const WIDTH: usize = 64;
const HEIGHT: usize = 256;
/// The river's tiles: columns 20 to 22.
const RIVER: std::ops::Range<usize> = 20..23;
const GUARDS: u32 = 100;
const DRONES: u32 = 100;
const STRUCTURES: u32 = 30;

/// What stands on the far bank.
#[derive(Clone, Copy)]
enum FarBank {
    /// [`DRONES`] Drones in 2 columns at the water's edge, along the 5000 su north of the Guards.
    Drones,
    /// [`STRUCTURES`] Generators (2 × 2 tiles) in a column a tile from the water, one every 3 tiles along the 5800 su
    /// of the Guards' walk: in the sight of the Guards walking nearest the river (from 256 su), out of their Range.
    Structures,
}

/// Wall-clock time of `work`. Only perf tests read a clock: they measure the sim from outside, and nothing in the
/// sim can call it (`ARCHITECTURE.md > Determinism`).
#[expect(
    clippy::disallowed_methods,
    reason = "a perf test has to read the clock; the sim itself never does"
)]
fn now() -> std::time::Instant {
    std::time::Instant::now()
}

/// The Guards in 5 columns on the west bank, at the map's south end (objects 1 to [`GUARDS`]), and the enemies of
/// `far` on the east bank, north of them. `crossing`: the river leaves the two northmost rows open.
fn army(crossing: bool, far: FarBank) -> (TestSim, Vec<ObjectId>) {
    let rows: Vec<String> = (0..HEIGHT)
        .map(|y| {
            (0..WIDTH)
                .map(|x| {
                    if RIVER.contains(&x) && !(crossing && y < 2) {
                        '~'
                    } else {
                        '.'
                    }
                })
                .collect()
        })
        .collect();
    let mut t = ScenarioBuilder::new(&rows.join("\n")).start();
    t.check_invariants(false);
    let (west, east) = (RIVER.start as f32 * 64.0, RIVER.end as f32 * 64.0);
    for i in 0..GUARDS {
        let at = Vec2::new(west - 20.0 - 48.0 * (i % 5) as f32, 15000.0 + 48.0 * (i / 5) as f32);
        t.spawn_unit_at(1, ObjectKind::Guard, PlayerId(1), at);
    }
    let enemies = match far {
        FarBank::Drones => {
            for i in 0..DRONES {
                let at = Vec2::new(east + 20.0 + 48.0 * (i % 2) as f32, 14900.0 - 100.0 * (i / 2) as f32);
                t.spawn_unit_at(1, ObjectKind::Drone, PlayerId(2), at);
            }
            DRONES
        }
        FarBank::Structures => {
            for i in 0..STRUCTURES {
                let at = SpawnAt::Tile(TilePos::new(RIVER.end as i32 + 1, 141 + 3 * i as i32));
                let (kind, owner, orientation) = (ObjectKind::Generator, Some(PlayerId(2)), Default::default());
                t.debug_at(
                    1,
                    DebugCommand::Spawn {
                        kind,
                        owner,
                        at,
                        orientation,
                    },
                );
            }
            STRUCTURES
        }
    };
    t.step();
    assert_eq!(t.sim().objects().count(), (GUARDS + enemies) as usize);
    (t, (1..=GUARDS).map(ObjectId).collect())
}

fn measure(label: &str, crossing: bool, far: FarBank) {
    let (mut t, guards) = army(crossing, far);
    let to = Vec2::new(RIVER.start as f32 * 64.0 - 116.0, 9000.0);
    t.order_at(2, PlayerId(1), &guards, OrderKind::AttackMove(to), false);
    let (mut total, mut worst) = (Duration::ZERO, Duration::ZERO);
    let (mut behaviours, mut worst_behaviours) = (Duration::ZERO, Duration::ZERO);
    let (mut paths, mut worst_paths, mut most_per_unit) = (0_u64, 0, 0);
    let (mut expansions, mut worst_expansions) = (0_u64, 0);
    // The slowest tick, its judgement paths and their expansions.
    let mut slowest = (0, 0, 0);
    let (mut requests, mut ticks) = (0_u64, 0_u32);
    let moving = |t: &TestSim| {
        (t.sim().objects()).any(|o| o.owner == Some(PlayerId(1)) && o.orders.as_ref().is_some_and(|o| !o.is_idle()))
    };
    while ticks < 2 || (ticks < 3000 && moving(&t)) {
        let start = now();
        let (mut began, mut took) = (None, Duration::ZERO);
        t.step_observed(|phase| match phase {
            Phase::Behaviours => began = Some(now()),
            Phase::NavSync => took = began.map_or(Duration::ZERO, |began| began.elapsed()),
            _ => {}
        });
        let tick = start.elapsed();
        let reach = t.sim().reach_counters();
        if tick > worst {
            slowest = (t.sim().tick(), reach.paths, reach.expansions);
        }
        total += tick;
        worst = worst.max(tick);
        behaviours += took;
        worst_behaviours = worst_behaviours.max(took);
        paths += u64::from(reach.paths);
        worst_paths = worst_paths.max(reach.paths);
        most_per_unit = most_per_unit.max(reach.most_per_unit);
        expansions += u64::from(reach.expansions);
        worst_expansions = worst_expansions.max(reach.expansions);
        let nav = t.sim().nav_counters();
        requests += u64::from(nav.searches + nav.fields);
        ticks += 1;
    }
    let per_tick = |d: Duration| d.as_secs_f64() * 1e3 / f64::from(ticks);
    let ms = |d: Duration| d.as_secs_f64() * 1e3;
    println!(
        "{label}: {ticks} ticks; whole tick mean {:.3} ms, worst {:.3} ms; behaviours (5a) mean {:.3} ms, worst {:.3} \
         ms; judgement paths {paths} ({:.2} per tick, worst tick {worst_paths}, most by one unit in a tick \
         {most_per_unit}), their expansions {expansions} (worst tick {worst_expansions}); path requests served in 5c \
         {requests} ({:.2} per tick); the slowest tick: {}, with {} judgement paths, {} expansions",
        per_tick(total),
        ms(worst),
        per_tick(behaviours),
        ms(worst_behaviours),
        paths as f64 / f64::from(ticks),
        requests as f64 / f64::from(ticks),
        slowest.0,
        slowest.1,
        slowest.2,
    );
    let shots = t
        .events()
        .iter()
        .filter(|(_, e)| matches!(e, SimEvent::Attacked { .. }))
        .count();
    let arrived = (t.sim().objects())
        .filter(|o| o.owner == Some(PlayerId(1)) && o.pos.y < 9000.0 + 600.0)
        .count();
    println!("  {shots} shots; {arrived} of {GUARDS} Guards at their destination");
}

#[test]
#[ignore = "prints timings; run in release with --ignored --nocapture"]
fn perf_100_guards_attack_moving_past_100_unreachable_drones() {
    measure("river from edge to edge (no way across)", false, FarBank::Drones);
    measure(
        "river with a crossing 200 tiles north (a long way round)",
        true,
        FarBank::Drones,
    );
}

/// The standing targets of design change 0243: a rejected building is judged again for each grid unit a Guard walks
/// with it in its sight (slice M3-L5).
#[test]
#[ignore = "prints timings; run in release with --ignored --nocapture"]
fn perf_100_guards_attack_moving_past_30_unreachable_structures() {
    measure("structures, no way across", false, FarBank::Structures);
    measure("structures, a crossing 200 tiles north", true, FarBank::Structures);
}
