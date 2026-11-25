//! Timings of 12 pop-up Laser Turrets cycling under traffic on a 256 × 256 map (slice M4-S2): what a raised pop-up
//! costs as a dynamic obstacle. Ignored by default:
//!
//! ```sh
//! cargo test -p sc_sim --release --test popup_perf -- --ignored --nocapture --test-threads=1
//! ```
//!
//! Three Tunnels of player 1 in a row, each with four turrets inside its area, and 200 Guards of player 1 walking
//! across the row in two groups, to and fro, their paths over the turrets' tiles. Every 64 ticks an enemy Guard
//! appears north and south of each Tunnel, in the Range of its two turrets there, and is removed 12 ticks later, before
//! a pop-up's shot lands: each turret rises (8 frames), stands its 32 idle frames and retracts (8), so every turret's
//! footprint starts and stops blocking once in each 64 ticks, and each change rebuilds the true grids, then its
//! owner's known ones, and sends the walkers' staggered checks along their paths. Timings are printed, never
//! asserted.
//!
//! It uses nothing of the pop-up's own API, so that the same file runs against a build without pop-ups, where the
//! turrets stand upright all along and no grid changes: the baseline of an A/B run.

use std::time::Duration;

use sc_sim::testing::{ScenarioBuilder, TestSim};
use sc_sim::{
    Command, DebugCommand, Faction, ObjectId, ObjectKind, OrderKind, PlayerId, SpawnAt, Target, TilePos, Vec2,
};

const SIDE: usize = 256;
const P1: PlayerId = PlayerId(1);
const P2: PlayerId = PlayerId(2);
/// The Tunnels' footprints' top-left tiles: their Tier 1 areas, 8 × 8, lie 16 tiles apart.
const TUNNELS: [(i32, i32); 3] = [(100, 124), (124, 124), (148, 124)];
/// The turrets of each Tunnel, as offsets of its footprint's origin: on the north and south rows of its area.
const TURRETS: [(i32, i32); 4] = [(-1, -2), (4, -2), (-1, 5), (4, 5)];
const WALKERS: u32 = 200;
const PERIOD: u32 = 64;

/// Wall-clock time of `work`. Only perf tests read a clock: they measure the sim from outside, and nothing in the
/// sim can call it (`ARCHITECTURE.md > Determinism`).
#[expect(
    clippy::disallowed_methods,
    reason = "a perf test has to read the clock; the sim itself never does"
)]
fn timed<T>(work: impl FnOnce() -> T) -> (T, Duration) {
    let start = std::time::Instant::now();
    let result = work();
    (result, start.elapsed())
}

#[test]
#[ignore = "prints timings; run in release with --ignored --nocapture"]
fn perf_12_popups_cycling_under_traffic() {
    let map = vec![".".repeat(SIDE); SIDE].join("\n");
    let mut builder = ScenarioBuilder::new(&map)
        .faction(P1, Faction::Syndicate)
        .faction(P2, Faction::Syndicate);
    for (x, y) in TUNNELS {
        builder = builder.spawn(ObjectKind::Tunnel, Some(P1), SpawnAt::Tile(TilePos::new(x, y)));
    }
    for (x, y) in TUNNELS {
        for (dx, dy) in TURRETS {
            let at = SpawnAt::Tile(TilePos::new(x + dx, y + dy));
            builder = builder.spawn(ObjectKind::LaserTurret, Some(P1), at);
        }
    }
    let mut t = builder.start();
    t.check_invariants(false);
    // Two blocks of walkers, north and south of the row, each patrolling to the other's side: across the turrets.
    let (north, south) = (112.0 * 64.0, 142.0 * 64.0);
    for i in 0..WALKERS {
        let offset = Vec2::new((i / 2 % 50) as f32 * 64.0, (i / 100) as f32 * 48.0);
        let y = if i % 2 == 0 { north } else { south + 48.0 };
        t.spawn_unit_at(1, ObjectKind::Guard, P1, Vec2::new(98.0 * 64.0, y) + offset);
    }
    t.step();
    let walkers: Vec<ObjectId> = (t.sim().objects())
        .filter(|o| o.kind == ObjectKind::Guard)
        .map(|o| o.id)
        .collect();
    assert_eq!(walkers.len(), WALKERS as usize);
    let (from_north, from_south): (Vec<ObjectId>, Vec<ObjectId>) = walkers
        .iter()
        .partition(|id| t.sim().object(**id).unwrap().pos.y < 127.0 * 64.0);
    let middle = 124.0 * 64.0;
    let (to_south, to_north) = (Vec2::new(middle, south + 48.0), Vec2::new(middle, north));
    let walk = |to: Vec2| OrderKind::Move(Target::Point(to));

    let (mut total, mut worst, mut ticks) = (Duration::ZERO, Duration::ZERO, 0_u32);
    let (mut rebuilds, mut versions, mut searches) = (0_u64, 0_u32, 0_u64);
    let mut samples = Vec::new();
    let started = t.sim().nav_version();
    for tick in 3..3 + 20 * PERIOD {
        // The walkers cross the row, each group to the other's side and back, a leg every 320 ticks.
        if tick % 320 == 3 {
            let (a, b) = if tick / 320 % 2 == 0 {
                (to_south, to_north)
            } else {
                (to_north, to_south)
            };
            t.order_at(tick, P1, &from_north, walk(a), false)
                .order_at(tick, P1, &from_south, walk(b), false);
        }
        let enemies = |t: &TestSim| -> Vec<ObjectId> {
            (t.sim().objects())
                .filter(|o| o.owner == Some(P2))
                .map(|o| o.id)
                .collect()
        };
        match tick % PERIOD {
            // An enemy 2 tiles north and one 2 tiles south of each Tunnel's area, between its two turrets there: within
            // 5 grid units of both, and beyond the other two's.
            0 => {
                for (x, y) in TUNNELS {
                    for dy in [-5, 8] {
                        let spawn = DebugCommand::Spawn {
                            kind: ObjectKind::Guard,
                            owner: Some(P2),
                            at: SpawnAt::Point(TilePos::new(x + 1, y + dy).center()),
                            orientation: Default::default(),
                        };
                        t.debug_at(tick, spawn);
                    }
                }
            }
            1 => {
                let hold = Command::Order {
                    units: enemies(&t),
                    kind: OrderKind::Hold,
                    queue: false,
                };
                t.at(tick, P2, hold);
            }
            12 => {
                for id in enemies(&t) {
                    t.debug_at(tick, DebugCommand::Remove { id });
                }
            }
            _ => {}
        }
        let ((), took) = timed(|| {
            t.step();
        });
        total += took;
        worst = worst.max(took);
        samples.push(took);
        ticks += 1;
        rebuilds += u64::from(t.sim().nav_rebuilds());
        searches += u64::from(t.sim().nav_counters().searches);
    }
    versions += t.sim().nav_version().wrapping_sub(started);
    samples.sort_unstable();
    let p99 = samples[samples.len() * 99 / 100];
    let alive = t
        .sim()
        .objects()
        .filter(|o| o.owner == Some(P1) && o.kind == ObjectKind::Guard)
        .count();
    println!(
        "12 turrets, {WALKERS} walkers, {ticks} ticks: mean {:.3} ms, p99 {:.3} ms, worst {:.3} ms per whole sim tick; \
         {versions} changes of the true grids, {rebuilds} sets of grids rebuilt, {searches} path searches; \
         {alive} of the walkers alive",
        total.as_secs_f64() * 1e3 / f64::from(ticks),
        p99.as_secs_f64() * 1e3,
        worst.as_secs_f64() * 1e3,
    );
}
