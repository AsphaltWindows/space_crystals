//! (G) Crowds (`docs/movement.md > 6.3`, `> 6.6`, `> 8`): blocks of units that meet head-on, the layout of design
//! change 0022's Verification among them, a group that walks through an idle block, and dense blocks of any shape
//! and of Guards. Every `TestSim` checks the invariants I1–I7 after each tick, so collision stays hard throughout.

use sc_sim::testing::{ScenarioBuilder, TestSim};
use sc_sim::{MoveStatus, ObjectId, ObjectKind, OrderKind, PlayerId, Target, Vec2};

const P1: PlayerId = PlayerId(1);
const P2: PlayerId = PlayerId(2);
const PK: ObjectKind = ObjectKind::Peacekeeper; // 24 × 24 su, 4 su per tick
const GUARD: ObjectKind = ObjectKind::Guard; // 36 × 36 su, 5 su per tick

/// 16 × 12 open tiles: 1024 × 768 su.
const OPEN: &str = "
    ................
    ................
    ................
    ................
    ................
    ................
    ................
    ................
    ................
    ................
    ................
    ................
";

fn v(x: f32, y: f32) -> Vec2 {
    Vec2::new(x, y)
}

fn move_to(p: Vec2) -> OrderKind {
    OrderKind::Move(Target::Point(p))
}

fn ids(range: std::ops::RangeInclusive<u32>) -> Vec<ObjectId> {
    range.map(ObjectId).collect()
}

/// Steps until every unit of `units` has a status other than `Moving`; the tick. Panics, with the map, after
/// `max_tick`.
#[track_caller]
fn run_until_done(t: &mut TestSim, units: &[ObjectId], max_tick: u32) -> u32 {
    // The units are spawned in tick 1 and ordered in tick 2.
    t.run_until(t.sim().tick().max(2));
    while units.iter().any(|id| t.status(id.0) == MoveStatus::Moving) {
        assert!(
            t.sim().tick() < max_tick,
            "still moving at tick {max_tick}:\n{}",
            t.ascii(None)
        );
        t.step();
    }
    t.sim().tick()
}

/// A block of `cols × rows` units of `kind`, `spacing` su apart, around `centre`.
fn spawn_block(
    t: &mut TestSim,
    kind: ObjectKind,
    owner: PlayerId,
    centre: Vec2,
    spacing: f32,
    (cols, rows): (u32, u32),
) {
    for i in 0..cols * rows {
        let across = (i / rows) as f32 - (cols - 1) as f32 / 2.0;
        let down = (i % rows) as f32 - (rows - 1) as f32 / 2.0;
        t.spawn_unit_at(1, kind, owner, centre + v(spacing * across, spacing * down));
    }
}

/// The blocks of 0022's Verification: 18 Peacekeepers in three columns of six.
const EIGHTEEN: (u32, u32) = (3, 6);

/// How the moves of units `1..=n` ended: (arrived, failed, the tick the last one ended).
fn outcome(t: &mut TestSim, n: u32) -> (u32, u32, u32) {
    let done = run_until_done(t, &ids(1..=n), 1200);
    let count = |s: MoveStatus| (1..=n).filter(|&id| t.status(id) == s).count() as u32;
    (count(MoveStatus::Arrived), count(MoveStatus::Failed), done)
}

/// 0022, Verification: "Re-run the 18-vs-18 head-on": two blocks of 18 Peacekeepers, each ordered onto the other's
/// place, the second block player 1's or player 2's. Every move ends.
///
/// (G) Both blocks are moving, so nobody yields: the units seek lanes (`docs/movement.md > 6.3`). 56 su apart the
/// blocks pass through each other's gaps and every unit arrives. 40 su apart the gaps (16 su) are narrower than
/// a unit: the blocks mesh, spread from the outside in, and pass. Before lane seeking 14 of the 36 units gave up
/// there (18 with two owners); now none does, and the bound leaves room for tuning. The counts are printed for
/// the record (`--nocapture`).
#[test]
fn eighteen_meet_eighteen_head_on() {
    for spacing in [40.0, 56.0] {
        for other in [P1, P2] {
            let mut t = ScenarioBuilder::new(OPEN).start();
            t.without_auto_targeting();
            let (west, east) = (v(200.0, 384.0), v(824.0, 384.0));
            spawn_block(&mut t, PK, P1, west, spacing, EIGHTEEN);
            spawn_block(&mut t, PK, other, east, spacing, EIGHTEEN);
            t.order_at(2, P1, &ids(1..=18), move_to(east), false).order_at(
                2,
                other,
                &ids(19..=36),
                move_to(west),
                false,
            );
            let (arrived, failed, done) = outcome(&mut t, 36);
            println!(
                "18 v 18 head-on, {spacing} su apart, the second block player {}'s: {arrived} arrived, {failed} \
                 failed, done in tick {done}",
                other.0
            );
            assert_eq!(arrived + failed, 36);
            if spacing > 40.0 {
                assert_eq!(failed, 0);
            }
            assert!(failed <= 2, "{failed} of 36 gave up\n{}", t.ascii(None));
            // Nobody takes much longer than the walk itself: 624 su at 4 su per tick is 156 ticks.
            assert!(done <= 2 + 156 + 120, "done in tick {done}");
            assert_eq!(t.replay().run().state_hash(), t.sim().state_hash());
        }
    }
}

/// A group of 18 ordered through a standing block of 18 idle Peacekeepers as wide as itself. An enemy block never
/// moves, and much of the group gives up in front of it; the owner's own block makes way (0022), and the group
/// arrives.
///
/// (G) The idle block **parts** for the group and isn't shoved along: the movers walk between its rows, and the
/// idle units step straight across their way (`docs/movement.md > 6.6`). Before, the block was pushed ahead of the
/// group, its units 204 su on average and up to 373 su; now they end 12 su from their places on average and at
/// most 35 su. The bounds leave room for tuning.
#[test]
fn eighteen_walk_through_eighteen_idle() {
    let mut failed_by_owner = Vec::new();
    for owner in [P1, P2] {
        let mut t = ScenarioBuilder::new(OPEN).start();
        t.without_auto_targeting();
        spawn_block(&mut t, PK, P1, v(200.0, 384.0), 40.0, EIGHTEEN);
        spawn_block(&mut t, PK, owner, v(512.0, 384.0), 40.0, EIGHTEEN);
        t.order_at(2, P1, &ids(1..=18), move_to(v(824.0, 384.0)), false);
        let (arrived, failed, done) = outcome(&mut t, 18);
        println!(
            "18 through 18 idle units of player {}: {arrived} arrived, {failed} failed, done in tick {done}",
            owner.0
        );
        assert_eq!(arrived + failed, 18);
        let moved: Vec<f32> = (0..18)
            .map(|i| {
                let offset = v(40.0 * ((i / 6) as f32 - 1.0), 40.0 * ((i % 6) as f32 - 2.5));
                t.pos(19 + i).distance(v(512.0, 384.0) + offset)
            })
            .collect();
        let (farthest, mean) = (
            moved.iter().fold(0.0_f32, |a, b| a.max(*b)),
            moved.iter().sum::<f32>() / 18.0,
        );
        println!(
            "the idle units of player {} ended {mean:.0} su from their places on average, at most {farthest:.0} su",
            owner.0
        );
        if owner == P1 {
            assert!(
                farthest <= 96.0 && mean <= 40.0,
                "shoved along: {moved:?}\n{}",
                t.ascii(None)
            );
            assert!(moved.iter().any(|d| *d > 0.0), "nobody made way");
            for id in 19..=36 {
                assert_eq!((t.mover(id).sidestep, t.mover(id).velocity), (None, Vec2::ZERO));
            }
        } else {
            assert_eq!(farthest, 0.0, "enemies never yield");
        }
        failed_by_owner.push(failed);
    }
    assert!(
        failed_by_owner[0] <= 2 && failed_by_owner[0] < failed_by_owner[1],
        "{failed_by_owner:?}"
    );
}

/// (G) `docs/movement.md > 6.3`, lane seeking: two dense blocks ordered onto each other's place in open ground
/// pass through each other, whatever their shape. The gaps between the units are narrower than a unit, so the
/// blocks have to mesh and spread; both are moving, so nobody yields (design change 0022), and collision stays
/// hard (the invariants run every tick). [`eighteen_meet_eighteen_head_on`] has the layout of 0022's Verification,
/// 18 against 18 Peacekeepers.
///
/// Before lane seeking about a fifth of the units of such layouts gave up in the jam, and up to four fifths of
/// them; now at most a few do (none to two of these). The bound is a fifth, loose enough to survive tuning.
#[test]
fn dense_blocks_of_any_shape_pass_through_each_other_head_on() {
    // Kind, the gap between two units, columns and rows, how far the second block is off the first one's line,
    // and its owner.
    let layouts = [
        (PK, 16.0, (5, 5), 0.0, P1),
        (PK, 8.0, (2, 9), 10.0, P2),
        (GUARD, 12.0, (3, 6), 20.0, P2),
        (GUARD, 16.0, (5, 5), 37.0, P1),
    ];
    for (kind, gap, shape, off_line, other) in layouts {
        let mut t = ScenarioBuilder::new(OPEN).start();
        t.without_auto_targeting();
        let spacing = kind.def().silhouette().unwrap().x + gap;
        let (west, east) = (v(250.0, 380.0), v(774.0, 380.0 + off_line));
        spawn_block(&mut t, kind, P1, west, spacing, shape);
        spawn_block(&mut t, kind, other, east, spacing, shape);
        let n = shape.0 * shape.1;
        t.order_at(2, P1, &ids(1..=n), move_to(east), false).order_at(
            2,
            other,
            &ids(n + 1..=2 * n),
            move_to(west),
            false,
        );
        let done = run_until_done(&mut t, &ids(1..=2 * n), 900);
        let failed = (1..=2 * n).filter(|&id| t.status(id) == MoveStatus::Failed).count() as u32;
        println!("{n} v {n} {kind:?} head-on, {gap} su between them: {failed} failed, done in tick {done}");
        assert!(failed * 5 <= 2 * n, "{failed} of {} gave up\n{}", 2 * n, t.ascii(None));
    }
}

/// 24 × 16 open tiles: 1536 × 1024 su, for [`crowd_sweep`].
const WIDE: &str = "
    ........................
    ........................
    ........................
    ........................
    ........................
    ........................
    ........................
    ........................
    ........................
    ........................
    ........................
    ........................
    ........................
    ........................
    ........................
    ........................
";

/// The crowd sweep of slice M1-G3, kept for design change 0169's Verification ("Rerun the M1-G3 crowd sweep:
/// failures drop as in the experiment"): two blocks ordered onto each other's place in open ground, 900 su apart,
/// over 48 layouts for each pair of kinds (Peacekeepers, Guards, Peacekeepers against Guards): gaps of 8 to 24 su
/// between the units, blocks of 3 × 6, 5 × 5 and 2 × 9, the second block 0 to 37 su off the first one's line; with
/// one owner, then two. A unit fails if its move failed or it is still moving in tick 2000. Prints the failures
/// for each pair of kinds and owner count. A measurement, ignored: run it in release, with
/// `cargo test --release -p sc_sim --test movement_crowds -- --ignored --nocapture crowd_sweep`.
///
/// Slice M3-V measured (2 → 3 grid units of `OCCUPIED_ARRIVAL_GU`): see `ARCHITECTURE.md > Movement and collision`.
#[test]
#[ignore]
fn crowd_sweep() {
    for owners in [P1, P2] {
        let (mut all, mut units) = (0, 0);
        for (west_kind, east_kind) in [(PK, PK), (GUARD, GUARD), (PK, GUARD)] {
            let size = west_kind
                .def()
                .silhouette()
                .unwrap()
                .x
                .max(east_kind.def().silhouette().unwrap().x);
            let (mut failed, mut worst) = (0, 0);
            for gap in [8.0, 12.0, 16.0, 24.0] {
                for shape in [(3, 6), (5, 5), (2, 9)] {
                    for off_line in [0.0, 10.0, 20.0, 37.0] {
                        let n = shape.0 * shape.1;
                        let mut t = ScenarioBuilder::new(WIDE).start();
                        t.without_auto_targeting();
                        let (west, east) = (v(300.0, 512.0), v(1200.0, 512.0 + off_line));
                        spawn_block(&mut t, west_kind, P1, west, size + gap, shape);
                        spawn_block(&mut t, east_kind, owners, east, size + gap, shape);
                        t.order_at(2, P1, &ids(1..=n), move_to(east), false).order_at(
                            2,
                            owners,
                            &ids(n + 1..=2 * n),
                            move_to(west),
                            false,
                        );
                        t.run_until(3);
                        while (1..=2 * n).any(|id| t.status(id) == MoveStatus::Moving) && t.sim().tick() < 2000 {
                            t.step();
                        }
                        let lost = (1..=2 * n)
                            .filter(|&id| matches!(t.status(id), MoveStatus::Failed | MoveStatus::Moving))
                            .count();
                        failed += lost;
                        worst = worst.max(lost);
                        units += 2 * n as usize;
                    }
                }
            }
            all += failed;
            println!(
                "crowd sweep, {west_kind:?} v {east_kind:?}, the second block player {}'s: {failed} failed, at worst \
                 {worst} in a layout",
                owners.0
            );
        }
        println!(
            "crowd sweep, the second block player {}'s: {all} of {units} failed",
            owners.0
        );
    }
}
