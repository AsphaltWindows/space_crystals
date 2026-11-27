//! (M5-K) `factions/gdo/units/rocketman.md > Target Lock` through the whole sim, with the Rocketman: the locking and
//! the homing missiles from anywhere within LockRange, MinRange ignored; the four breaks, a Shift-queued order behind
//! the lock and a locked missile whose Firing outlives its lock; a lock on a structure; the gate.

use glam::Vec2;

use super::TargetLock;
use crate::command::{Command, DebugCommand, PlayerCommand};
use crate::defs::ObjectKind;
use crate::events::SimEvent;
use crate::ids::{ObjectId, PlayerId};
use crate::orders::{OrderKind, Target};
use crate::scenario::SpawnAt;
use crate::sim::Reject;
use crate::targets::TargetView;
use crate::testing::{ScenarioBuilder, TestSim};
use crate::units::{Tick, TilePos, gu};

const P1: PlayerId = PlayerId(1);
const P2: PlayerId = PlayerId(2);
const ROCKETMAN: ObjectId = ObjectId(1);
const TARGET: ObjectId = ObjectId(2);
/// Player 1's other unit, which sees what the Rocketman doesn't, where a test has one.
const SPOTTER: ObjectId = ObjectId(3);
/// Where the Rocketman stands.
const AT: Vec2 = Vec2::new(200.0, 320.0);
/// LockOnDuration, ticks.
const LOCK_ON: Tick = 48;

/// 16 × 10 open tiles.
fn open() -> String {
    "................\n".repeat(10)
}

/// The Rocketman at [`AT`] and player 2's Peacekeeper `east` space units east of it, centre to centre (both 24 su
/// wide), and player 1's Peacekeeper at `spotter`, if given; auto-targeting off, so that nobody fights but on orders.
fn scene(east: f32, spotter: Option<Vec2>) -> TestSim {
    let mut b = ScenarioBuilder::new(&open())
        .spawn(ObjectKind::Rocketman, Some(P1), SpawnAt::Point(AT))
        .spawn(
            ObjectKind::Peacekeeper,
            Some(P2),
            SpawnAt::Point(AT + Vec2::new(east, 0.0)),
        );
    if let Some(at) = spotter {
        b = b.spawn(ObjectKind::Peacekeeper, Some(P1), SpawnAt::Point(at));
    }
    let mut t = b.start();
    t.without_auto_targeting();
    t
}

/// The Rocketman's Target Lock under way.
fn lock(t: &TestSim) -> Option<TargetLock> {
    t.sim().object(ROCKETMAN)?.attack.as_ref()?.target_lock
}

/// The Rocketman's current order, if any.
fn current(t: &TestSim) -> Option<OrderKind> {
    let orders = t.sim().object(ROCKETMAN)?.orders.as_ref()?;
    orders.current.as_ref().map(|active| active.order.kind)
}

/// The Rocketman's launches of the run so far: (tick, homing).
fn launches(t: &TestSim) -> Vec<(Tick, bool)> {
    (t.events().iter())
        .filter_map(|(tick, e)| match *e {
            SimEvent::ProjectileLaunched { shooter, homing, .. } if shooter == ROCKETMAN => Some((*tick, homing)),
            _ => None,
        })
        .collect()
}

/// The ticks `id` was hit in, so far.
fn hits_on(t: &TestSim, id: ObjectId) -> Vec<Tick> {
    (t.events().iter())
        .filter_map(|(tick, e)| match *e {
            SimEvent::Hit { target, .. } if target == id => Some(*tick),
            _ => None,
        })
        .collect()
}

/// Player 1 orders the Rocketman to lock onto `target` in `tick`, which its order starts in.
fn order_lock(t: &mut TestSim, tick: Tick, target: ObjectId) {
    t.order_at(tick, P1, &[ROCKETMAN], OrderKind::TargetLock(target), false);
}

/// "the Rocketman aims at the target continuously for LockOnDuration": its order starts the lock, which counts a tick
/// each phase 5e and is locked in the 48th, the Rocketman standing all the while and firing nothing; then "every
/// missile it fires at that target is **homing**": its first leaves at the end of the first sequence that starts then
/// (Aim 6, Firing 2), and hits; the next one a cycle (32 ticks) later.
#[test]
fn a_lock_takes_lock_on_duration_then_fires_homing_missiles() {
    let mut t = scene(320.0, None);
    order_lock(&mut t, 2, TARGET);
    t.run_until(2);
    assert_eq!(current(&t), Some(OrderKind::TargetLock(TARGET)));
    assert_eq!(
        lock(&t),
        Some(TargetLock {
            target: TARGET,
            elapsed: 1,
            locked: false
        })
    );
    t.run_until(2 + LOCK_ON - 2);
    assert!(!lock(&t).unwrap().locked, "still locking");
    t.step();
    assert_eq!(
        lock(&t),
        Some(TargetLock {
            target: TARGET,
            elapsed: 48,
            locked: true
        })
    );
    let first = 2 + LOCK_ON - 1 + 6 + 2 - 1;
    t.run_until(first - 1);
    assert_eq!(launches(&t), [], "nothing while locking and aiming");
    assert_eq!(t.pos(1), AT, "it stands");
    t.step();
    assert_eq!(launches(&t), [(first, true)]);
    t.run_until(first + 32);
    assert_eq!(launches(&t), [(first, true), (first + 32, true)]);
    assert!(!hits_on(&t, TARGET).is_empty(), "the first one hit");
    assert_eq!(t.pos(1), AT, "it stands");
    assert_eq!(
        current(&t),
        Some(OrderKind::TargetLock(TARGET)),
        "until the lock breaks"
    );
}

/// "**No minimum range when locked** (user, 2026-11-25): locked homing missiles can be fired at **any distance** up to
/// LockRange, even inside the normal attack's MinRange": a target 1 GU from the Rocketman's edge, inside its MinRange
/// of 2, is fired at once locked; and (no walk into range) "Locked missiles can be fired from anywhere within
/// LockRange": one 6.5 GU away, beyond its Range of 6, that player 1 sees, too.
#[test]
fn locked_missiles_ignore_min_range_and_reach_lock_range() {
    for (east, spotter) in [(24.0 + gu(1.0), None), (24.0 + gu(6.5), Some(Vec2::new(700.0, 448.0)))] {
        let mut t = scene(east, spotter);
        order_lock(&mut t, 2, TARGET);
        t.run_until(2 + LOCK_ON + 8);
        assert_eq!(launches(&t), [(2 + LOCK_ON + 6, true)], "{east}");
        t.run_until(2 + LOCK_ON + 60);
        assert!(!hits_on(&t, TARGET).is_empty(), "{east}");
        assert_eq!(t.pos(1), AT, "it stands: {east}");
    }
}

/// "the target leaves LockRange": the target walks east, seen all the way by player 1's other unit; the lock holds
/// while its gap to the Rocketman is at most LockRange, 7 GU edge to edge, and breaks in the first phase 5e it is
/// beyond; the order completes in the next tick's phase 5a and the Rocketman stands idle.
#[test]
fn the_lock_breaks_when_the_target_leaves_lock_range() {
    let mut t = scene(320.0, Some(Vec2::new(800.0, 512.0)));
    order_lock(&mut t, 2, TARGET);
    let away = OrderKind::Move(Target::Point(Vec2::new(900.0, 320.0)));
    t.order_at(2 + LOCK_ON + 10, P2, &[TARGET], away, false);
    let gap = |t: &TestSim| (t.pos(2).x - 12.0) - (AT.x + 12.0);
    t.run_until(2);
    while lock(&t).is_some() {
        assert!(gap(&t) <= gu(7.0), "held beyond LockRange: {}", gap(&t));
        assert!(t.sim().tick() < 400, "the lock never broke\n{}", t.ascii(None));
        t.step();
    }
    assert!(gap(&t) > gu(7.0), "broke within LockRange: {}", gap(&t));
    assert_eq!(
        current(&t),
        Some(OrderKind::TargetLock(TARGET)),
        "until the next phase 5a"
    );
    t.step();
    assert_eq!(current(&t), None);
    assert!(!launches(&t).is_empty() && launches(&t).iter().all(|&(_, homing)| homing));
    assert_eq!(t.pos(1), AT);
}

/// "the Rocketman loses vision of the target": a target 6.5 GU away, within LockRange but beyond the Rocketman's own
/// sight, which player 1's other unit sees, locked onto; the other unit is taken out of the game, and the lock breaks
/// in the first tick that player 1 doesn't see the target as it begins (the vision the tick before left, after the fog
/// delay: `core/vision.md > Visible > Fog delay`), the target still where it stood, within LockRange.
#[test]
fn the_lock_breaks_when_its_owner_loses_sight_of_the_target() {
    let east = 24.0 + gu(6.5);
    let mut t = scene(east, Some(Vec2::new(700.0, 448.0)));
    order_lock(&mut t, 2, TARGET);
    t.run_until(2 + LOCK_ON + 20);
    assert!(lock(&t).is_some_and(|lock| lock.locked));
    let blinded = 2 + LOCK_ON + 21;
    t.debug_at(blinded, DebugCommand::Remove { id: SPOTTER });
    let seen = |t: &TestSim| t.sim().target_view(P1, TARGET) == TargetView::Visible;
    loop {
        let was_seen = seen(&t);
        t.step();
        if lock(&t).is_none() {
            assert!(!was_seen, "broke while seen");
            break;
        }
        assert!(was_seen, "held unseen");
        assert!(t.sim().tick() < blinded + 30, "the lock never broke");
    }
    assert!(t.sim().tick() > blinded + crate::tuning::vision::FOG_DELAY_TICKS);
    assert_eq!(t.pos(2), AT + Vec2::new(east, 0.0), "where it stood, within LockRange");
    t.step();
    assert_eq!(current(&t), None);
}

/// "the Rocketman is given **any** order": a Move without Shift, while it locks, ends the lock in that tick's phase
/// 5a, and the Rocketman walks; a lock onto another target replaces the one under way, which starts over.
#[test]
fn any_order_breaks_the_lock() {
    let mut t = scene(320.0, None);
    order_lock(&mut t, 2, TARGET);
    let walk = OrderKind::Move(Target::Point(Vec2::new(200.0, 500.0)));
    t.order_at(20, P1, &[ROCKETMAN], walk, false).run_until(19);
    assert!(lock(&t).is_some());
    t.step();
    assert_eq!(lock(&t), None);
    assert_eq!(current(&t), Some(walk));
    t.run_until(30);
    assert!(t.pos(1).y > AT.y, "it walks");
    assert_eq!(launches(&t), []);

    let mut t = scene(320.0, None);
    order_lock(&mut t, 2, TARGET);
    t.run_until(30);
    order_lock(&mut t, 31, TARGET);
    t.run_until(31);
    assert_eq!(
        lock(&t),
        Some(TargetLock {
            target: TARGET,
            elapsed: 1,
            locked: false
        }),
        "a new lock starts over"
    );
}

/// Design change 0438 (answering Q0171; plan decision T76): "a Shift-queued order waits behind the lock and runs when
/// it ends": here as "the target dies" (taken out of the game through the removal path, the one of every death), which
/// ends the lock and its order at once.
#[test]
fn a_queued_order_waits_behind_the_lock_until_its_target_dies() {
    let mut t = scene(320.0, None);
    order_lock(&mut t, 2, TARGET);
    let walk = OrderKind::Move(Target::Point(Vec2::new(200.0, 500.0)));
    t.order_at(10, P1, &[ROCKETMAN], walk, true);
    t.run_until(2 + LOCK_ON + 20);
    assert!(lock(&t).is_some_and(|lock| lock.locked), "the queued order waits");
    assert_eq!(t.pos(1), AT);
    let dies = 2 + LOCK_ON + 21;
    t.debug_at(dies, DebugCommand::Remove { id: TARGET }).run_until(dies);
    assert_eq!(lock(&t), None, "the target died");
    t.run_until(dies + 10);
    assert_eq!(current(&t), Some(walk));
    assert!(t.pos(1).y > AT.y, "it walks");
}

/// A Shift+R after a Move queues the lock, which starts where the Move ends.
#[test]
fn a_queued_lock_starts_after_the_order_before_it() {
    let mut t = scene(320.0, None);
    let walk = OrderKind::Move(Target::Point(Vec2::new(200.0, 448.0)));
    t.order_at(2, P1, &[ROCKETMAN], walk, false);
    t.order_at(2, P1, &[ROCKETMAN], OrderKind::TargetLock(TARGET), true);
    t.run_until(10);
    assert_eq!(lock(&t), None, "not yet");
    while current(&t) == Some(walk) {
        assert!(t.sim().tick() < 200);
        t.step();
    }
    assert_eq!(current(&t), Some(OrderKind::TargetLock(TARGET)));
    assert!(lock(&t).is_some());
    assert!(t.pos(1).y > AT.y + 64.0, "where the Move took it");
}

/// A missile whose Firing has begun is launched as it began, homing, though an order breaks the lock meanwhile
/// (`systems/combat.md > Firing`: not interruptible): a Stop in the last tick of a locked missile's Firing.
#[test]
fn a_locked_missile_in_firing_is_launched_homing_though_the_lock_breaks() {
    let mut t = scene(320.0, None);
    order_lock(&mut t, 2, TARGET);
    let first = 2 + LOCK_ON - 1 + 6 + 2 - 1;
    t.order_at(first, P1, &[ROCKETMAN], OrderKind::Stop, false)
        .run_until(first);
    assert_eq!(lock(&t), None);
    assert_eq!(launches(&t), [(first, true)]);
}

/// "**Target Lock can target structures** (user, 2026-11-26) as well as units": player 2's Arsenal, 2 × 2 tiles, its
/// edge 300 su from the Rocketman's, is locked onto and hit.
#[test]
fn a_lock_on_a_structure_hits_it() {
    let mut t = ScenarioBuilder::new(&open())
        .spawn(ObjectKind::Rocketman, Some(P1), SpawnAt::Point(AT))
        .spawn(ObjectKind::Arsenal, Some(P2), SpawnAt::Tile(TilePos::new(8, 4)))
        .start();
    t.without_auto_targeting();
    order_lock(&mut t, 2, TARGET);
    t.run_until(2 + LOCK_ON + 60);
    assert!(lock(&t).is_some_and(|lock| lock.locked));
    assert_eq!(launches(&t).first(), Some(&(2 + LOCK_ON + 6, true)));
    assert!(!hits_on(&t, TARGET).is_empty());
}

/// The gate: "The player orders Target Lock on one enemy unit" or structure: not on the player's own, nor on an object
/// that can't be destroyed; only units whose type has a Target Lock take it, the others of a group ignore it.
#[test]
fn only_a_rocketman_locks_and_only_onto_an_enemy() {
    let t = ScenarioBuilder::new(&open())
        .spawn(ObjectKind::Rocketman, Some(P1), SpawnAt::Point(AT))
        .spawn(
            ObjectKind::Peacekeeper,
            Some(P2),
            SpawnAt::Point(AT + Vec2::new(320.0, 0.0)),
        )
        .spawn(
            ObjectKind::Peacekeeper,
            Some(P1),
            SpawnAt::Point(AT + Vec2::new(0.0, 128.0)),
        )
        .spawn(ObjectKind::CrystalPatch, None, SpawnAt::Tile(TilePos::new(5, 7)))
        .start();
    let validate = |units: &[ObjectId], target: u32| {
        let command = Command::Order {
            units: units.to_vec(),
            kind: OrderKind::TargetLock(ObjectId(target)),
            queue: false,
        };
        t.sim().validate(&PlayerCommand { player: P1, command })
    };
    let (rocketman, own) = (ROCKETMAN, SPOTTER);
    assert_eq!(validate(&[rocketman], 2), Ok(()));
    assert_eq!(validate(&[rocketman, own], 2), Ok(()), "the Peacekeeper ignores it");
    assert_eq!(validate(&[own], 2), Err(Reject::LacksAbility));
    assert_eq!(validate(&[rocketman], 3), Err(Reject::NotAnEnemy(ObjectId(3))));
    assert_eq!(validate(&[rocketman], 4), Err(Reject::NotAttackable(ObjectId(4))));
}
