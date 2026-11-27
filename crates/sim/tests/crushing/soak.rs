//! The oriented soak of slice M5-V1 (`tests/oriented/soak.rs`) with crushers: Hands of Heaven (64 × 40 su, tracked,
//! their Silhouettes turning with their facing) among Peacekeepers and Recruits (Light Infantry) and Guards (Heavy
//! Infantry), each kind half player 1's and half player 2's, on random maps of walls with gaps, under seeded random
//! orders. `TestSim` checks the invariants after every tick, the no-overlap invariant I1 among them, with no exception:
//! a crusher's move passes through its crushable enemies and overlaps them only within the tick of contact, after which
//! they are dead and gone (the plan's "one exception: a moving crusher with an enemy crushable, in the tick of contact,
//! after which it is dead" is checked here, crush by crush, instead).

use std::collections::BTreeMap;

use sc_sim::mover::Locomotion;
use sc_sim::testing::{ScenarioBuilder, TestSim};
const GIVE_UP_TICKS: sc_sim::Tick = sc_sim::tuning::movement::BUILTIN.give_up_ticks();
const STUCK_GRACE_TICKS: sc_sim::Tick = sc_sim::tuning::movement::BUILTIN.stuck_grace_ticks;
use sc_sim::{
    Command, DebugCommand, MoveStatus, ObjectId, ObjectKind, OrderKind, PlayerCommand, PlayerId, Rng, SU_PER_GU,
    SimEvent, SpawnAt, Target, Vec2,
};

use super::{GUARD, HAND, P1, P2, PEACEKEEPER, RECRUIT};

const SIDE: i32 = 32;
const UNITS: u32 = 96;
const TICKS: u32 = 1500;

/// A 32 × 32 map: scattered cliff and water tiles, three walls with gaps of two tiles, a patch of Rugged Terrain
/// (which the Hand of Heaven, tracked, goes round), as the oriented soak's.
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

/// One seed's game: spawns in tick 1, a Hand of Heaven, a Peacekeeper, a Recruit and a Guard in turn, four of player
/// 1's then four of player 2's (those the gate refuses are dropped), then an order every few ticks: moves of groups and
/// of single units, queued moves, Stop and Hold. Every unit stops near the end.
fn soak(seed: u64) -> TestSim {
    let mut rng = Rng::from_seed(seed);
    let mut t = ScenarioBuilder::new(&random_map(&mut rng)).seed(seed).start();
    t.without_auto_targeting();
    let mut spawned = 0;
    for cell in 0..18 * 18 {
        let kind = [HAND, PEACEKEEPER, RECRUIT, GUARD][spawned as usize % 4];
        let at = Vec2::new((cell % 18) as f32, (cell / 18) as f32) * 110.0 + Vec2::splat(100.0);
        let jitter = Vec2::new(rng.below(30) as f32, rng.below(30) as f32);
        let spawn = PlayerCommand {
            player: P1,
            command: Command::Debug(DebugCommand::Spawn {
                kind,
                owner: Some(PlayerId(1 + (spawned / 4 % 2) as u8)),
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
    let mut tick = 2;
    while tick < TICKS - 200 {
        let player = [P1, P2][rng.below(2) as usize];
        let own: Vec<ObjectId> = units.iter().filter(|u| u.1 == player).map(|u| u.0).collect();
        let count = [1, 1, 3, 8, own.len()][rng.below(5) as usize].min(own.len());
        let first = rng.below((own.len() - count + 1) as u32) as usize;
        let group = &own[first..first + count];
        let queue = rng.below(4) == 0;
        // The orders name units that may have been crushed since: the gate keeps those still in the game.
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
    t
}

/// The invariants hold after every tick (`TestSim`), and so do the crushes, each in the tick it comes in: the crushed
/// unit is a Light Infantry of an enemy of its crusher, a Hand of Heaven, and is out of the game; the crusher moved or
/// turned in the tick, and not as a Stopping move (`movement::crush`: "Locomotion = Moving or Reversing", or a spin in
/// place, design change 0418). Nothing dies of damage. No unit is on its way for ever without getting anywhere (as in
/// the oriented soak, design change 0024). The Hands of Heaven crush, and get where they are sent. The same seed
/// again gives the same state, and so does the replay.
///
/// Seeds 1 and 2: 44 and 36 of the 48 Light Infantry crushed; of the Hands of Heaven's moves, 3 and 10 arrived, 8
/// and 8 failed. Slower than the orders (5 su a tick, among 96 units on 32 × 32 tiles), few of their moves end before
/// the next order replaces them, as in the oriented soak.
#[test]
fn hands_of_heaven_among_both_players_infantry_crush_only_what_the_rules_say() {
    let give_up = GIVE_UP_TICKS + 4 * STUCK_GRACE_TICKS;
    for seed in [1, 2] {
        let mut t = soak(seed);
        let hands: Vec<ObjectId> = t.sim().objects().filter(|o| o.kind == HAND).map(|o| o.id).collect();
        // Per unit: the tick and the values of its latest progress.
        let mut since: BTreeMap<ObjectId, (u32, f32, u32)> = BTreeMap::new();
        let mut statuses = vec![MoveStatus::Idle; hands.len()];
        let (mut crushed, mut arrived, mut failed) = (0, 0, 0);
        while t.sim().tick() < TICKS {
            let before: BTreeMap<ObjectId, (Vec2, f32)> =
                t.sim().objects().map(|o| (o.id, (o.pos, o.facing))).collect();
            let kinds: BTreeMap<ObjectId, (ObjectKind, Option<PlayerId>)> =
                t.sim().objects().map(|o| (o.id, (o.kind, o.owner))).collect();
            t.step();
            let tick = t.sim().tick();
            for event in t.sim().events() {
                assert!(!matches!(event, SimEvent::Died { .. }), "seed {seed}: {event:?}");
                let SimEvent::Crushed {
                    id,
                    kind,
                    owner,
                    crusher,
                    ..
                } = *event
                else {
                    continue;
                };
                crushed += 1;
                assert!(t.sim().object(id).is_none(), "seed {seed}, tick {tick}: {id:?} is gone");
                assert_eq!(kinds[&id], (kind, owner));
                assert!(
                    matches!(kind, ObjectKind::Peacekeeper | ObjectKind::Recruit),
                    "{kind:?}"
                );
                let by = t.sim().object(crusher).expect("its crusher");
                assert!(
                    by.kind == HAND && by.owner.is_some() && by.owner != owner,
                    "seed {seed}: {by:?}"
                );
                let locomotion = by.mover.as_ref().expect("a unit").locomotion;
                assert_ne!(locomotion, Locomotion::Stopping, "seed {seed}, tick {tick}: {by:?}");
                let (pos, facing) = before[&crusher];
                assert!(
                    by.pos != pos || by.facing != facing,
                    "seed {seed}, tick {tick}: it moved or turned"
                );
            }
            for o in t.sim().objects() {
                let Some(mover) = o.mover.as_ref() else {
                    continue;
                };
                let (from, best, progress) = since.entry(o.id).or_insert((tick, f32::MAX, 0));
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
            for (id, was) in hands.iter().zip(&mut statuses) {
                let status = t.sim().object(*id).unwrap().mover.as_ref().unwrap().status;
                if status != *was {
                    arrived += u32::from(status == MoveStatus::Arrived);
                    failed += u32::from(status == MoveStatus::Failed);
                }
                *was = status;
            }
        }
        println!("seed {seed}: {crushed} crushed; of the Hands of Heaven's moves, {arrived} arrived, {failed} failed");
        assert!(crushed >= 5, "seed {seed}: {crushed} crushed");
        assert!(arrived >= 1, "seed {seed}: {arrived} arrived, {failed} failed");
        if seed == 1 {
            let mut again = soak(seed);
            again.check_invariants(false).run_until(TICKS);
            assert_eq!(again.sim().state_hash(), t.sim().state_hash());
            assert_eq!(t.replay().run().state_hash(), t.sim().state_hash());
        }
    }
}
