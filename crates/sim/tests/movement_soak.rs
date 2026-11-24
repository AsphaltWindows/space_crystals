//! Soak (`docs/movement.md > 8`): 200 units on random maps under seeded random orders for 2 000 ticks. The
//! invariants I1–I7 hold after every tick (`TestSim` checks them), nothing panics, every move ends within a bounded
//! time of its last progress, most moves arrive, and the run is deterministic.

use sc_sim::mover::{GIVE_UP_TICKS, STUCK_GRACE_TICKS};
use sc_sim::testing::{ScenarioBuilder, TestSim};
use sc_sim::{
    Command, DebugCommand, MoveStatus, ObjectId, ObjectKind, OrderKind, PlayerCommand, PlayerId, Rng, SU_PER_GU,
    SpawnAt, Target, TilePos, Vec2,
};

const SIDE: i32 = 48;
const UNITS: u32 = 200;
const TICKS: u32 = 2000;
/// Of the moves that end in one seed's run, at most this share ends `Failed`.
const MAX_FAILED_SHARE: f32 = 0.6;
/// The same over all seeds.
const MAX_FAILED_SHARE_OVERALL: f32 = 0.35;

/// A 48 × 48 map: about 8 % scattered water and cliff tiles, a few walls with gaps, a patch of Rugged Terrain.
fn random_map(rng: &mut Rng) -> String {
    let mut tiles = vec![vec!['.'; SIDE as usize]; SIDE as usize];
    for row in &mut tiles {
        for tile in row.iter_mut() {
            *tile = match rng.below(100) {
                0..=4 => '~',
                5..=7 => 'c',
                8..=10 => 'r',
                _ => '.',
            };
        }
    }
    for _ in 0..4 {
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

/// The whole schedule of one seed: spawns in tick 1 (those the gate refuses are dropped), then an order every few
/// ticks: moves of groups and of single units, queued moves, Stop and Hold, a teleport and a new building now and
/// then.
fn soak(seed: u64) -> TestSim {
    let mut rng = Rng::from_seed(seed);
    let mut t = ScenarioBuilder::new(&random_map(&mut rng)).seed(seed).start();
    t.without_auto_targeting();
    // Whole-number positions on a loose lattice; the first 200 that stand on open ground.
    let mut spawned = 0;
    for cell in 0..24 * 12 {
        let kind = if rng.below(4) == 0 {
            ObjectKind::Guard
        } else {
            ObjectKind::Peacekeeper
        };
        let at = Vec2::new((cell % 24) as f32, (cell / 24) as f32) * 110.0 + Vec2::splat(150.0);
        let jitter = Vec2::new(rng.below(40) as f32, rng.below(40) as f32);
        let spawn = PlayerCommand {
            player: PlayerId(1),
            command: Command::Debug(DebugCommand::Spawn {
                kind,
                owner: Some(PlayerId(1 + (spawned % 2) as u8)),
                at: SpawnAt::Point(at + jitter),
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
    let mut tick = 2;
    while tick < TICKS - 200 {
        let player = PlayerId(1 + rng.below(2) as u8);
        let own: Vec<ObjectId> = units.iter().filter(|u| u.1 == player).map(|u| u.0).collect();
        // A run of consecutive own units: neighbours on the map, as a selection box would take.
        let count = [1, 1, 3, 12, 40, own.len()][rng.below(6) as usize].min(own.len());
        let first = rng.below((own.len() - count + 1) as u32) as usize;
        let group = &own[first..first + count];
        let queue = rng.below(4) == 0;
        match rng.below(20) {
            0 => t.order_at(tick, player, group, OrderKind::Stop, queue),
            1 => t.order_at(tick, player, group, OrderKind::Hold, queue),
            2 => t.debug_at(
                tick,
                DebugCommand::SetPosition {
                    id: group[0],
                    pos: random_point(&mut rng),
                },
            ),
            3 => t.debug_at(
                tick,
                DebugCommand::Spawn {
                    kind: ObjectKind::SupplyDeliveryStation,
                    owner: None,
                    at: SpawnAt::Tile(TilePos::containing(random_point(&mut rng))),
                },
            ),
            _ => {
                let target = Target::Point(random_point(&mut rng));
                t.order_at(tick, player, group, OrderKind::Move(target), queue)
            }
        };
        tick += 1 + rng.below(12);
    }
    // Everybody stops at the end, so no move is cut off by the end of the run.
    for player in [PlayerId(1), PlayerId(2)] {
        let own: Vec<ObjectId> = units.iter().filter(|u| u.1 == player).map(|u| u.0).collect();
        t.order_at(TICKS - 10, player, &own, OrderKind::Stop, false);
    }
    t
}

#[test]
fn two_hundred_units_under_random_orders_keep_the_invariants() {
    // A stuck unit ends its move within this many ticks of its last progress (`docs/movement.md > 6.5`, design
    // change 0024).
    let give_up = GIVE_UP_TICKS + STUCK_GRACE_TICKS;
    let (mut all_arrived, mut all_failed) = (0, 0);
    for seed in [1, 2, 3] {
        let mut t = soak(seed);
        // Per unit: the tick and the values of its latest progress, and its status a tick ago.
        let mut seen: Vec<(u32, f32, u32, MoveStatus)> = Vec::new();
        let (mut arrived, mut failed) = (0, 0);
        while t.sim().tick() < TICKS {
            t.step();
            let tick = t.sim().tick();
            let movers = t.sim().objects().filter_map(|o| o.mover.as_ref());
            for (i, mover) in movers.enumerate() {
                if seen.len() <= i {
                    seen.push((tick, f32::MAX, 0, MoveStatus::Idle));
                }
                let (since, best, progress, status) = &mut seen[i];
                // No unit moves for ever without getting anywhere: it gets nearer, or waits behind one that does
                // (`last_progress`), or its move ends.
                if mover.status != MoveStatus::Moving
                    || mover.best_remaining != *best
                    || mover.last_progress != *progress
                {
                    (*since, *best, *progress) = (tick, mover.best_remaining, mover.last_progress);
                }
                assert!(
                    tick - *since <= give_up,
                    "seed {seed}: a unit made no progress since tick {since}: {mover:?}\n{}",
                    t.ascii(None)
                );
                if mover.status != *status {
                    arrived += u32::from(mover.status == MoveStatus::Arrived);
                    failed += u32::from(mover.status == MoveStatus::Failed);
                    *status = mover.status;
                }
            }
        }
        let still_moving = t.sim().objects().filter_map(|o| o.mover.as_ref());
        assert_eq!(still_moving.filter(|m| m.status == MoveStatus::Moving).count(), 0);
        println!("seed {seed}: {arrived} moves arrived, {failed} failed");
        // The orders pile 200 units into a few tiles again and again. Moving units never make way for each other,
        // and a unit shut out more than 2 GU from its place gives up (design changes 0022 and 0026), so some of
        // those moves fail. How many depends on the seed and on every tuning constant, so the bounds are loose:
        // of the moves that ended, at most `MAX_FAILED_SHARE` failed in any one seed, and at most
        // `MAX_FAILED_SHARE_OVERALL` over all seeds. (With friendly yielding the shares are 0.38, 0.07 and 0.13,
        // and 0.16 overall; without it they were 0.66, 0.19 and 0.21, and 0.32 overall.)
        let ended = arrived + failed;
        assert!(ended >= 100, "seed {seed}: only {ended} moves ended");
        assert!(
            failed as f32 <= MAX_FAILED_SHARE * ended as f32,
            "seed {seed}: {arrived} arrived, {failed} failed"
        );
        all_arrived += arrived;
        all_failed += failed;
        // The same seed again gives the same state, and so does the replay.
        if seed == 1 {
            let mut again = soak(seed);
            again.check_invariants(false).run_until(TICKS);
            assert_eq!(again.sim().state_hash(), t.sim().state_hash());
            assert_eq!(t.replay().run().state_hash(), t.sim().state_hash());
        }
    }
    assert!(
        all_failed as f32 <= MAX_FAILED_SHARE_OVERALL * (all_arrived + all_failed) as f32,
        "{all_arrived} arrived, {all_failed} failed"
    );
}
