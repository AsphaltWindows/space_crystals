//! (M5-K) The Target Locks' reticles: one on each lock's target, at its body, for the lock's owner alone, filling as
//! LockOnDuration runs, then locked; none once the lock is gone.

use sc_sim::testing::{ScenarioBuilder, TestSim};
use sc_sim::{Faction, ObjectId, ObjectKind, OrderKind, PlayerId, SpawnAt};

use super::*;

const P1: PlayerId = PlayerId(1);
const P2: PlayerId = PlayerId(2);
const ROCKETMAN: ObjectId = ObjectId(1);
const PEACEKEEPER: ObjectId = ObjectId(2);
const ROCKETMAN_AT: Vec2 = Vec2::new(200.0, 384.0);
const PEACEKEEPER_AT: Vec2 = Vec2::new(456.0, 384.0);

/// Player 1's Rocketman and player 2's Peacekeeper 4 GU east of it, in its sight and its LockRange, on flat ground,
/// neither attacking by itself; player 1 orders a Target Lock on the Peacekeeper in tick 2.
fn locking() -> TestSim {
    let scenario = ScenarioBuilder::new(&vec![".".repeat(16); 12].join("\n"))
        .faction(P1, Faction::Gdo)
        .faction(P2, Faction::Gdo)
        .spawn(ObjectKind::Rocketman, Some(P1), SpawnAt::Point(ROCKETMAN_AT))
        .spawn(ObjectKind::Peacekeeper, Some(P2), SpawnAt::Point(PEACEKEEPER_AT))
        .build();
    let mut t = TestSim::new(scenario);
    t.without_auto_targeting();
    t.order_at(2, P1, &[ROCKETMAN], OrderKind::TargetLock(PEACEKEEPER), false);
    t
}

/// The reticles `viewer` sees now, through the fog.
fn marks(t: &TestSim, viewer: PlayerId) -> Vec<LockMark> {
    lock_marks(t.sim(), &Sight::new(t.sim(), viewer, true), 1.0)
}

/// From the tick its order starts, the lock shows the Rocketman's owner a reticle round the Peacekeeper's body, its
/// aiming line from the Rocketman: a 48th of it filled in its first tick (`sc_sim::combat::target_lock`: the lock
/// counts the order's ticks), half of it 23 ticks on, and all of it, locked, in the 48th; player 2, whose unit is the
/// target, sees none. Once a Stop ends the lock, it is gone.
#[test]
fn a_lock_shows_its_owner_a_reticle_that_fills_then_locks() {
    let mut t = locking();
    t.run_until(1);
    assert_eq!(marks(&t, P1), [], "no lock yet");
    t.run_until(2);
    let radius = ObjectKind::Peacekeeper.def().half_extents().unwrap().length() + GAP;
    let first = LockMark {
        center: PEACEKEEPER_AT,
        radius,
        from: ROCKETMAN_AT,
        progress: 1.0 / 48.0,
        locked: false,
    };
    assert_eq!(marks(&t, P1), [first]);
    assert_eq!(marks(&t, P2), [], "the lock's owner's alone");
    t.run_until(25);
    assert_eq!(marks(&t, P1)[0].progress, 0.5);
    t.run_until(48);
    assert!(!marks(&t, P1)[0].locked);
    t.run_until(49);
    let locked = LockMark {
        progress: 1.0,
        locked: true,
        ..first
    };
    assert_eq!(marks(&t, P1), [locked]);
    t.order_at(51, P1, &[ROCKETMAN], OrderKind::Stop, false);
    t.run_until(51);
    assert_eq!(marks(&t, P1), [], "the lock ended");
}
