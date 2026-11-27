//! Design change 0356's Verification: "Rocketmen can shoot down a Sparrow" and "A circling Sparrow can sometimes dodge
//! the normal missile; a locked one can't" (`factions/gdo/units/rocketman.md > Target Lock`: "every missile it fires
//! at that target is **homing** [...] The projectile cannot miss"), with Sparrows (`factions/cults/units/sparrow.md`)
//! spawned directly. A Sparrow is a Glider: it "must keep moving; circles when idle", so the normal missile
//! (`systems/combat.md > DoublyDisjointed`), which flies to where its target stood at the end of Aiming, lands where
//! the Sparrow no longer is.

use sc_sim::TargetView;

use super::*;

/// What the missiles at `target` did so far: (the normal ones that hit it, those that missed, the hits of the locked
/// ones). A normal missile lands in the tick its launch named (`SimEvent::ProjectileLaunched::lands_at`), so its hit is
/// the one its shooter made on `target` then; it missed if there was none. The other hits are the locked missiles',
/// which land as they reach the target. Missiles still in the air when the target fell don't count.
fn outcomes(t: &TestSim, target: u32) -> (usize, usize, usize) {
    let end = died(t, target).unwrap_or(t.sim().tick());
    let hits = hits_on(t, target);
    let normal: Vec<Missile> = (missiles_at(t, target).into_iter())
        .filter(|m| !m.homing && m.lands_at <= end)
        .collect();
    let struck = |m: &&Missile| (hits.iter()).any(|&(tick, attacker, _)| tick == m.lands_at && attacker == m.shooter);
    let normal_hits = normal.iter().filter(struck).count();
    (normal_hits, normal.len() - normal_hits, hits.len() - normal_hits)
}

/// "Rocketmen can shoot down a Sparrow": player 1's three Rocketmen (1 to 3), on a column 60 su apart, and player 2's
/// two Sparrows (4, 5), spawned directly 280 su east of them and 180 su apart, which circle where they came into the
/// game, in sight; auto-targeting off, so that nobody fights but on orders. In tick 2 all three lock onto Sparrow 4
/// and, Shift-queued, onto Sparrow 5 (a queued order waits behind the lock, design change 0438). Sparrow 4 falls to
/// three locked missiles, (M5-S8) each a whole burst on its centre ([`BURST`]); then the locks on Sparrow 5 start,
/// LockOnDuration again, and it falls too, the other Sparrow too far off for a burst to reach. Every missile they fire
/// is a locked one, every hit a whole burst, and the Rocketmen end idle.
#[test]
fn locked_rocketmen_shoot_down_two_circling_sparrows() {
    let mut t = field(30, 20)
        .spawn(ROCKETMAN, Some(P1), point(600.0, 580.0))
        .spawn(ROCKETMAN, Some(P1), point(600.0, 640.0))
        .spawn(ROCKETMAN, Some(P1), point(600.0, 700.0))
        .spawn(SPARROW, Some(P2), point(880.0, 540.0))
        .spawn(SPARROW, Some(P2), point(880.0, 720.0))
        .start();
    t.without_auto_targeting();
    order(&mut t, 2, P1, &[1, 2, 3], OrderKind::TargetLock(id(4)), false);
    order(&mut t, 2, P1, &[1, 2, 3], OrderKind::TargetLock(id(5)), true);
    while alive(&t, 4) || alive(&t, 5) {
        assert!(t.sim().tick() < 16 * 30, "both down in 30 seconds\n{}", t.ascii(None));
        t.step();
    }
    let (first, second) = (died(&t, 4).expect("down"), died(&t, 5).expect("down"));
    assert!(first > 2 + LOCK_ON, "not before the locks: {first}");
    assert!(
        second > first + LOCK_ON,
        "the second locks start as the first target falls: {second}"
    );
    for (sparrow, locks_from) in [(4, 2), (5, first)] {
        let missiles = missiles_at(&t, sparrow);
        let locked = |m: &Missile| m.homing && m.tick >= locks_from + LOCK_ON;
        assert!(
            missiles.iter().all(locked),
            "all locked, none before the lock: {missiles:?}"
        );
        let hits = hits_on(&t, sparrow);
        assert_eq!(hits.len(), 3, "three hits bring it down: {hits:?}");
        for &(_, attacker, damage) in &hits {
            assert!((1..=3).contains(&attacker));
            assert!(whole_burst(damage), "a whole burst: {damage}");
        }
        let taken: f32 = hits.iter().map(|&(.., damage)| damage).sum();
        assert!(
            taken >= SPARROW_HP && taken - BURST < SPARROW_HP,
            "the third one: {taken}"
        );
        assert_eq!(outcomes(&t, sparrow), (0, 0, 3));
    }
    for rocketman in 1..=3 {
        assert!(alive(&t, rocketman));
        assert_eq!(current(&t, rocketman), None, "idle once both are down");
    }
}

/// "A circling Sparrow can sometimes dodge the normal missile; a locked one can't": player 1's Rocketman and player
/// 2's Sparrow, spawned directly 280 su east of it, circling where it came into the game: within the Rocketman's Range
/// of 6 grid units and outside its MinRange of 2 all round its circle, and in its sight; auto-targeting off. Ordered to
/// attack it, the Rocketman fires its normal missiles for 30 seconds, and the Sparrow dodges them, flying on round its
/// circle in the 20-odd frames between the end of Aiming and the landing. Ordered instead to lock onto it, the
/// Rocketman fires locked missiles, and every one hits, the third bringing it down.
#[test]
fn a_circling_sparrow_dodges_the_normal_missile_and_never_a_locked_one() {
    let scene = || {
        let mut t = field(30, 20)
            .spawn(ROCKETMAN, Some(P1), point(600.0, 640.0))
            .spawn(SPARROW, Some(P2), point(880.0, 640.0))
            .start();
        t.without_auto_targeting();
        t
    };
    let mut t = scene();
    order(&mut t, 2, P1, &[1], OrderKind::Attack(id(2)), false);
    t.run_until(16 * 30);
    assert!(missiles_at(&t, 2).iter().all(|m| !m.homing && m.shooter == 1));
    let (hit, missed, locked) = outcomes(&t, 2);
    assert!(hit + missed >= 10, "a missile every 2 seconds: {hit} + {missed}");
    assert!(missed > 0, "the Sparrow dodges: {hit} hit, {missed} missed");
    assert_eq!(locked, 0);
    assert_eq!(
        current(&t, 1),
        Some(OrderKind::Attack(id(2))),
        "it attacks all the while"
    );

    let mut t = scene();
    order(&mut t, 2, P1, &[1], OrderKind::TargetLock(id(2)), false);
    while alive(&t, 2) {
        assert!(t.sim().tick() < 16 * 30, "down in 30 seconds");
        t.step();
    }
    let missiles = missiles_at(&t, 2);
    assert!(missiles.iter().all(|m| m.homing && m.shooter == 1), "{missiles:?}");
    assert_eq!(missiles.len(), 3, "one a cycle, each landing before the next leaves");
    assert_eq!(outcomes(&t, 2), (0, 0, 3), "every locked missile hits");
}

/// "Rocketmen can shoot down a Sparrow" that fights back: player 2's Sparrow (4), spawned directly 300 su east of
/// player 1's three Rocketmen (1 to 3), strafes them (auto-targeting on), each pass a loop that takes it out of their
/// SightRange of 6 grid units, where player 1's Generators round the field keep it in sight. Every second in which none
/// of the Rocketmen has a lock under way and player 1 sees the Sparrow, player 1 orders all three to lock onto it
/// again, as a player would: a lock breaks as the Sparrow leaves its LockRange of 7. Within 30 seconds the Sparrow
/// falls, the locked missiles' hits among those it takes, every one a whole burst on its centre ([`BURST`]); (M5-S8) a
/// normal missile's burst may graze it, and takes no more. Fired at an aircraft, no burst hits the Rocketmen under it,
/// and the three stand.
#[test]
fn a_strafing_sparrow_kept_in_sight_falls_to_locks_given_again() {
    let mut scenario = field(40, 24)
        .spawn(ROCKETMAN, Some(P1), point(800.0, 700.0))
        .spawn(ROCKETMAN, Some(P1), point(840.0, 760.0))
        .spawn(ROCKETMAN, Some(P1), point(800.0, 820.0))
        .spawn(SPARROW, Some(P2), point(1100.0, 760.0));
    for (x, y) in [(22, 8), (22, 17), (15, 20), (15, 4), (7, 12)] {
        scenario = scenario.spawn(ObjectKind::Generator, Some(P1), tile(x, y));
    }
    let mut t = scenario.start();
    let rocketmen = [1, 2, 3];
    while alive(&t, 4) {
        let now = t.sim().tick();
        assert!(now < 16 * 30, "down in 30 seconds\n{}", t.ascii(None));
        let locking =
            |n: u32| (t.sim().object(id(n))).is_some_and(|o| o.attack.as_ref().unwrap().target_lock.is_some());
        let seen = t.sim().target_view(P1, id(4)) == TargetView::Visible;
        if now.is_multiple_of(16) && seen && !rocketmen.iter().any(|&n| locking(n)) {
            order(&mut t, now + 1, P1, &rocketmen, OrderKind::TargetLock(id(4)), false);
        }
        t.step();
    }
    let (_, _, locked) = outcomes(&t, 4);
    assert!(locked > 0, "locked missiles hit it");
    let normal: Vec<(u32, Tick)> = (missiles_at(&t, 4).iter())
        .filter(|m| !m.homing)
        .map(|m| (m.shooter, m.lands_at))
        .collect();
    for (tick, attacker, damage) in hits_on(&t, 4) {
        assert!(rocketmen.contains(&attacker), "tick {tick}: {attacker}");
        match normal.contains(&(attacker, tick)) {
            true => assert!(damage < BURST + 1e-3, "a normal missile's: a graze at most: {damage}"),
            false => assert!(whole_burst(damage), "a locked missile's, on its centre: {damage}"),
        }
    }
    for n in rocketmen {
        assert!(alive(&t, n), "the Rocketmen stand");
        assert_eq!(
            hits_on(&t, n)
                .iter()
                .filter(|&&(_, attacker, _)| rocketmen.contains(&attacker))
                .count(),
            0
        );
    }
}
