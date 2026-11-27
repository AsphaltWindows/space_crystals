//! The soak of `docs/movement.md > 8` with turned shapes: Seekers (56 × 40 su, their Silhouettes turning with their
//! facing) among Guards and Peacekeepers, on random maps of walls with gaps, under seeded random orders. `TestSim`
//! checks the invariants I1–I7 after every tick: I1 between turned shapes by the separating-axis test, I2 with a
//! turned shape against the tiles. Nothing panics, the Seekers turn and get where they are sent, every move ends, and
//! the run is deterministic.

use sc_sim::testing::{ScenarioBuilder, TestSim};
use sc_sim::tuning::movement::{GIVE_UP_TICKS, STUCK_GRACE_TICKS};
use sc_sim::{
    Command, DebugCommand, MoveStatus, ObjectId, ObjectKind, OrderKind, PlayerCommand, PlayerId, Rng, SU_PER_GU,
    SpawnAt, Target, Vec2,
};

use super::{P1, P2};

const SIDE: i32 = 32;
const UNITS: u32 = 72;
const TICKS: u32 = 1500;

/// A 32 × 32 map: scattered cliff and water tiles, three walls with gaps of two tiles, a patch of Rugged Terrain
/// (which the Seeker, a WheeledVehicle, goes round).
fn random_map(rng: &mut Rng) -> String {
    let mut tiles = vec![vec!['.'; SIDE as usize]; SIDE as usize];
    for row in &mut tiles {
        for tile in row.iter_mut() {
            *tile = match rng.below(100) {
                0..=2 => '~',
                3..=4 => 'c',
                5..=7 => 'r',
                _ => '.',
            };
        }
    }
    for _ in 0..3 {
        let (vertical, at) = (rng.below(2) == 0, 4 + rng.below(SIDE as u32 - 8) as usize);
        let gap = rng.below(SIDE as u32 - 3) as usize;
        for i in 0..SIDE as usize {
            let open = (gap..gap + 2).contains(&i);
            let tile = if vertical { &mut tiles[i][at] } else { &mut tiles[at][i] };
            *tile = if open { '.' } else { 'M' };
        }
    }
    tiles
        .into_iter()
        .map(|row| row.into_iter().collect::<String>())
        .collect::<Vec<_>>()
        .join("\n")
}

fn random_point(rng: &mut Rng) -> Vec2 {
    Vec2::new(rng.next_f32(), rng.next_f32()) * (SIDE as f32 * SU_PER_GU)
}

/// One seed's game: spawns in tick 1, every third unit a Seeker (those the gate refuses are dropped), then an order
/// every few ticks: moves of groups and of single units, queued moves, Stop and Hold. Every unit stops near the end.
fn soak(seed: u64) -> (TestSim, Vec<ObjectId>) {
    let mut rng = Rng::from_seed(seed);
    let mut t = ScenarioBuilder::new(&random_map(&mut rng)).seed(seed).start();
    t.without_auto_targeting();
    let mut spawned = 0;
    for cell in 0..18 * 18 {
        let kind = match spawned % 3 {
            0 => ObjectKind::Seeker,
            1 => ObjectKind::Guard,
            _ => ObjectKind::Peacekeeper,
        };
        let at = Vec2::new((cell % 18) as f32, (cell / 18) as f32) * 110.0 + Vec2::splat(100.0);
        let jitter = Vec2::new(rng.below(30) as f32, rng.below(30) as f32);
        let spawn = PlayerCommand {
            player: P1,
            command: Command::Debug(DebugCommand::Spawn {
                kind,
                owner: Some(PlayerId(1 + (spawned % 2) as u8)),
                at: SpawnAt::Point(at + jitter),
                orientation: Default::default(),
            }),
        };
        if spawned < UNITS && t.sim().validate(&spawn).is_ok() {
            t.at(1, spawn.player, spawn.command);
            spawned += 1;
        }
    }
    t.step();
    let units: Vec<(ObjectId, PlayerId)> = t.sim().objects().filter_map(|o| Some((o.id, o.owner?))).collect();
    assert_eq!(units.len(), UNITS as usize);
    let seekers = (t.sim().objects())
        .filter(|o| o.kind == ObjectKind::Seeker)
        .map(|o| o.id)
        .collect();
    let mut tick = 2;
    while tick < TICKS - 200 {
        let player = [P1, P2][rng.below(2) as usize];
        let own: Vec<ObjectId> = units.iter().filter(|u| u.1 == player).map(|u| u.0).collect();
        let count = [1, 1, 3, 8, own.len()][rng.below(5) as usize].min(own.len());
        let first = rng.below((own.len() - count + 1) as u32) as usize;
        let group = &own[first..first + count];
        let queue = rng.below(4) == 0;
        match rng.below(16) {
            0 => t.order_at(tick, player, group, OrderKind::Stop, queue),
            1 => t.order_at(tick, player, group, OrderKind::Hold, queue),
            _ => {
                let target = Target::Point(random_point(&mut rng));
                t.order_at(tick, player, group, OrderKind::Move(target), queue)
            }
        };
        tick += 1 + rng.below(10);
    }
    for player in [P1, P2] {
        let own: Vec<ObjectId> = units.iter().filter(|u| u.1 == player).map(|u| u.0).collect();
        t.order_at(TICKS - 10, player, &own, OrderKind::Stop, false);
    }
    (t, seekers)
}

/// The invariants hold after every tick (`TestSim`). No unit is on its way for ever without getting anywhere, as in
/// `movement_soak`: a stuck one ends its move within `GIVE_UP_TICKS` of its last progress (design change 0024), and a
/// grace before each of its attempts if no unit blocks it, as a turn cut by a wall does (`STUCK_GRACE_TICKS`,
/// `docs/movement.md > 6.5`). The Seekers turn, and most of their moves that end arrive. The same seed again gives the
/// same state, and so does the replay.
///
/// A Seeker jams where a box wouldn't: its turn is cut where a corner meets a unit or a wall (design change 0035), and
/// a turned Silhouette doesn't slide past another as two boxes do. The orders cross the two players' units again and
/// again, and enemies never make way, so many more of the Seekers' moves fail than of the others'; the bound is loose.
/// (At the end of slice M5-V1, seeds 1 and 2: 30 of the Seekers' moves arrived and 16 failed, and 54 and 6.)
#[test]
fn seekers_among_other_units_under_random_orders_keep_the_invariants() {
    let give_up = GIVE_UP_TICKS + 4 * STUCK_GRACE_TICKS;
    for seed in [1, 2] {
        let (mut t, seekers) = soak(seed);
        // Per unit: the tick and the values of its latest progress.
        let mut since: Vec<(u32, f32, u32)> = Vec::new();
        // Per Seeker: its facing and its move's status a tick ago.
        let mut seen: Vec<(f32, MoveStatus)> = seekers.iter().map(|_| (0.0, MoveStatus::Idle)).collect();
        let (mut turns, mut arrived, mut failed) = (0, 0, 0);
        while t.sim().tick() < TICKS {
            t.step();
            let tick = t.sim().tick();
            for (i, mover) in t.sim().objects().filter_map(|o| o.mover.as_ref()).enumerate() {
                if since.len() <= i {
                    since.push((tick, f32::MAX, 0));
                }
                let (from, best, progress) = &mut since[i];
                if mover.status != MoveStatus::Moving
                    || mover.best_remaining != *best
                    || mover.last_progress != *progress
                {
                    (*from, *best, *progress) = (tick, mover.best_remaining, mover.last_progress);
                }
                assert!(
                    tick - *from <= give_up,
                    "seed {seed}: a unit made no progress since tick {from}: {mover:?}"
                );
            }
            for (id, was) in seekers.iter().zip(&mut seen) {
                let seeker = t.sim().object(*id).unwrap();
                let status = seeker.mover.as_ref().unwrap().status;
                turns += usize::from(seeker.facing != was.0);
                if status != was.1 {
                    arrived += u32::from(status == MoveStatus::Arrived);
                    failed += u32::from(status == MoveStatus::Failed);
                }
                *was = (seeker.facing, status);
            }
        }
        println!("seed {seed}: the Seekers turned {turns} times; {arrived} of their moves arrived, {failed} failed");
        assert!(turns > 1000, "seed {seed}: the Seekers turned {turns} times");
        assert!(
            arrived >= 20 && failed <= arrived,
            "seed {seed}: {arrived} arrived, {failed} failed"
        );
        if seed == 1 {
            let (mut again, _) = soak(seed);
            again.check_invariants(false).run_until(TICKS);
            assert_eq!(again.sim().state_hash(), t.sim().state_hash());
            assert_eq!(t.replay().run().state_hash(), t.sim().state_hash());
        }
    }
}
