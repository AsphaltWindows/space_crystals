//! Soak of the behaviours: 120 units of two players fight on random maps under seeded random orders of every kind
//! (Move, Attack, AttackMove, Patrol, Hold, Stop; on points, on enemies, on their own units and on map resources) for
//! 1 500 ticks. The movement invariants hold after every tick (`TestSim` checks them), nothing panics, no state
//! refers to an object that died, no behaviour asks for paths tick after tick, and the run is deterministic.

use sc_sim::testing::{self, ScenarioBuilder, TestSim};
use sc_sim::{
    Command, DebugCommand, MoveStatus, ObjectId, ObjectKind, OrderKind, PlayerCommand, PlayerId, Rng, SU_PER_GU,
    SimEvent, SpawnAt, Target, TilePos, Vec2,
};

const SIDE: i32 = 40;
const UNITS: u32 = 120;
const TICKS: u32 = 1500;

/// A 40 × 40 map with about 6 % scattered water and cliff tiles and two walls with gaps.
fn random_map(rng: &mut Rng) -> String {
    let mut tiles = vec![vec!['.'; SIDE as usize]; SIDE as usize];
    for row in &mut tiles {
        for tile in row.iter_mut() {
            *tile = match rng.below(100) {
                0..=3 => '~',
                4..=5 => 'c',
                _ => '.',
            };
        }
    }
    for at in [13, 26] {
        let gap = rng.below(SIDE as u32 - 4) as usize;
        for (i, row) in tiles.iter_mut().enumerate() {
            row[at] = if (gap..gap + 3).contains(&i) { '.' } else { 'M' };
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

/// What one seed's run came to.
struct Outcome {
    hash: u64,
    deaths: usize,
    shots: usize,
    /// Orders the gate took, of those tried.
    taken: (u32, u32),
    /// A\* searches over the whole run.
    searches: u64,
    t: TestSim,
}

/// One seed: the units are spawned in tick 1, player 1's in the west and player 2's in the east, with a few map
/// resources between them; then every few ticks one player orders a run of its units, by what the sim holds then.
/// An order the gate refuses (a target out of sight, say) is part of the test.
fn soak(seed: u64) -> Outcome {
    let mut rng = Rng::from_seed(seed);
    let mut t = ScenarioBuilder::new(&random_map(&mut rng)).seed(seed).start();
    let mut spawned = 0;
    for cell in 0..20 * 20 {
        let kind = if rng.below(3) == 0 {
            ObjectKind::Guard
        } else {
            ObjectKind::Peacekeeper
        };
        let owner = PlayerId(1 + (cell % 2) as u8);
        // Player 1 on the western third, player 2 on the eastern.
        let column = (cell / 2 % 10) as f32 * 70.0 + if owner.0 == 1 { 80.0 } else { 1760.0 };
        let at = Vec2::new(column, (cell / 20) as f32 * 110.0 + 150.0);
        let jitter = Vec2::new(rng.below(30) as f32, rng.below(30) as f32);
        let spawn = PlayerCommand {
            player: PlayerId(1),
            command: Command::Debug(DebugCommand::Spawn {
                kind,
                owner: Some(owner),
                at: SpawnAt::Point(at + jitter),
            }),
        };
        if spawned < UNITS && t.sim().validate(&spawn).is_ok() {
            t.at(1, spawn.player, spawn.command);
            spawned += 1;
        }
    }
    for _ in 0..6 {
        let kind = [ObjectKind::CrystalPatch, ObjectKind::SupplyDeliveryStation][rng.below(2) as usize];
        let tile = TilePos::new(15 + rng.below(10) as i32, 2 + rng.below(36) as i32);
        t.debug_at(
            1,
            DebugCommand::Spawn {
                kind,
                owner: None,
                at: SpawnAt::Tile(tile),
            },
        );
    }
    t.step();
    assert_eq!(t.sim().objects().filter(|o| o.mover.is_some()).count(), UNITS as usize);

    let (mut next_order, mut taken, mut tried, mut searches) = (2, 0, 0, 0_u64);
    while t.sim().tick() < TICKS {
        let tick = t.sim().tick() + 1;
        if tick >= next_order {
            next_order = tick + 1 + rng.below(8);
            let player = PlayerId(1 + rng.below(2) as u8);
            let own: Vec<ObjectId> = t
                .sim()
                .objects()
                .filter(|o| o.owner == Some(player) && o.orders.is_some())
                .map(|o| o.id)
                .collect();
            let all: Vec<ObjectId> = t.sim().objects().map(|o| o.id).collect();
            if !own.is_empty() {
                let count = [1, 1, 2, 6, 15, own.len()][rng.below(6) as usize].min(own.len());
                let first = rng.below((own.len() - count + 1) as u32) as usize;
                let units = own[first..first + count].to_vec();
                // An object target: mostly something the player sees, sometimes anything at all.
                let seen: Vec<ObjectId> = t
                    .sim()
                    .objects()
                    .filter(|o| t.sim().is_visible(player, o))
                    .map(|o| o.id)
                    .collect();
                let pool = if rng.below(5) == 0 || seen.is_empty() {
                    &all
                } else {
                    &seen
                };
                let object = pool[rng.below(pool.len() as u32) as usize];
                let point = random_point(&mut rng);
                let kind = match rng.below(12) {
                    0 => OrderKind::Stop,
                    1 => OrderKind::Hold,
                    2 | 3 => OrderKind::Attack(object),
                    4 => OrderKind::Move(Target::Object(object)),
                    5 | 6 => OrderKind::Patrol(point),
                    7..=9 => OrderKind::AttackMove(point),
                    _ => OrderKind::Move(Target::Point(point)),
                };
                let command = Command::Order {
                    units,
                    kind,
                    queue: rng.below(4) == 0,
                };
                tried += 1;
                let checked = PlayerCommand {
                    player,
                    command: command.clone(),
                };
                taken += u32::from(t.sim().validate(&checked).is_ok());
                t.at(tick, player, command);
            }
        }
        t.step();
        searches += u64::from(t.sim().nav_counters().searches);
        assert_eq!(
            testing::dangling_references(t.sim()),
            [],
            "seed {seed}, tick {}",
            t.sim().tick()
        );
        // A unit without an order has a goal only while it fights or walks back on its own.
        for o in t.sim().objects() {
            let (Some(orders), Some(mover)) = (&o.orders, &o.mover) else {
                continue;
            };
            if orders.is_idle() && mover.status == MoveStatus::Moving {
                assert_ne!(orders.auto, sc_sim::orders::Auto::default(), "seed {seed}: {o:?}");
            }
        }
    }
    let count = |wanted: fn(&SimEvent) -> bool| t.events().iter().filter(|(_, e)| wanted(e)).count();
    Outcome {
        hash: t.sim().state_hash(),
        deaths: count(|e| matches!(e, SimEvent::Died { .. })),
        shots: count(|e| matches!(e, SimEvent::Attacked { .. })),
        taken: (taken, tried),
        searches,
        t,
    }
}

#[test]
fn units_fighting_under_random_orders_keep_the_invariants() {
    for seed in [1, 2, 3] {
        let run = soak(seed);
        let alive = run.t.sim().objects().filter(|o| o.mover.is_some()).count();
        println!(
            "seed {seed}: {} shots, {} deaths, {alive} units left; {} of {} orders taken; {} path searches",
            run.shots, run.deaths, run.taken.0, run.taken.1, run.searches
        );
        // It was a fight, and the orders mostly went through.
        assert!(run.shots > 300 && run.deaths > 10, "seed {seed}");
        assert!(run.taken.0 * 2 > run.taken.1, "seed {seed}: {:?}", run.taken);
        // No behaviour plans afresh every tick: on average a unit asks for a path far less than once in 10 ticks.
        assert!(
            run.searches < u64::from(UNITS * TICKS / 10),
            "seed {seed}: {} searches",
            run.searches
        );
        if seed == 1 {
            assert_eq!(soak(seed).hash, run.hash, "the same seed again");
            assert_eq!(run.t.replay().run().state_hash(), run.hash, "and its replay");
        }
    }
}
