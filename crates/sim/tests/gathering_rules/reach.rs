//! A patch the Drone can't get to (design change 0141, `> Patch Occupancy > A patch it can't reach`): walled off by
//! units, or by terrain.

use super::*;

/// The map: a ridge along row 3 with one pass, at tile (5, 3), and a ring of mountain round tile (7, 5).
const MAP: &str = "\
................
................
................
MMMMM.MMMMMMMMMM
......MMM.......
......M.M.......
......MMM.......
................
................
................
................";

/// Patch A (1) north of the ridge, at tile (5, 1); patch C (2) inside the ring, which nothing can get to; player 2's
/// Guard 3 in the pass; player 1's node 4 and Drone 5 south of the ridge, and its Beacon 6 north of it, in whose
/// sight A is; with `b`, patch B (7) south of the ridge too, at tile (2, 7), farther from the pass than C.
/// Auto-targeting is off: the Guard stands, idle, and an enemy's idle unit never yields (`systems/units.md >
/// UnitCollision > Ground Collision > Who yields`).
fn walled(b: bool) -> TestSim {
    let mut scenario = ScenarioBuilder::new(MAP)
        .spawn(PATCH, None, tile(5, 1))
        .spawn(PATCH, None, tile(7, 5))
        .spawn(ObjectKind::Guard, Some(P2), centre(5, 3))
        .spawn(NODE, Some(P1), tile(10, 8))
        .spawn(DRONE, Some(P1), centre(9, 8))
        .spawn(ObjectKind::Beacon, Some(P1), tile(1, 0));
    if b {
        scenario = scenario.spawn(PATCH, None, tile(2, 7));
    }
    let mut t = scenario.start();
    t.without_auto_targeting();
    t
}

/// Design change 0141 (its Verification: "Wall off a patch with idle tanks: the Drone switches to another free
/// patch"): a patch the Drone can't get within MiningReach of is handled like an occupied one. The leg that ends
/// short of it is its arrival there, and the search skips the patch its owner's map shows no way to (C), nearer as
/// it is.
#[test]
fn a_drone_shut_out_of_its_patch_by_units_goes_to_another_free_one() {
    let mut t = walled(true);
    gather(&mut t, 1, P1, &[5], 1);
    let sent = run_to(&mut t, 200, "it is sent on", |t| {
        order(t, 5) != Some(OrderKind::Gather(id(1)))
    });
    assert_eq!(order(&t, 5), Some(OrderKind::Gather(id(7))), "to B, not to C");
    for n in [2, 7] {
        assert!(t.sim().is_visible(P1, object(&t, n)), "player 1 sees patch {n}");
    }
    let at = object(&t, 5).pos;
    assert!(
        off_tile(&t, 5, 5, 3) < 128.0 && at.y > 256.0,
        "it got as far as the pass: {at:?} (tick {sent})"
    );
    run_to(&mut t, 200, "it mines B", |t| mines(t, 5, 7));
    assert_eq!((remaining(&t, 1), remaining(&t, 2)), (1500, 1500));
}

/// Design change 0141 (its Verification: "if none, it waits near the wall and resumes once a tank moves"): with no
/// other free patch it can get to, the Drone waits as close as it got, tries again at the waiting interval, and
/// mines once the way is clear. C, free and in sight, is never a patch to go to: nothing gets there.
#[test]
fn with_no_other_patch_it_waits_at_the_wall_and_mines_once_the_way_clears() {
    let mut t = walled(false);
    assert!(t.sim().is_visible(P1, object(&t, 2)), "player 1 sees C");
    gather(&mut t, 1, P1, &[5], 1);
    run_to(&mut t, 200, "it waits", |t| is_waiting(t, 5));
    let mut tries = 0;
    let mut was = phase(&t, 5);
    for _ in 0..200 {
        t.step();
        let now = t.sim().tick();
        assert_eq!(order(&t, 5), Some(OrderKind::Gather(id(1))), "tick {now}");
        assert!(!is_mining(&t, 5));
        assert!(
            off_tile(&t, 5, 5, 3) < 128.0,
            "it stays by the pass: {:?} (tick {now})",
            object(&t, 5).pos
        );
        if matches!(was, Some(GatherPhase::Waiting { .. })) && phase(&t, 5) == Some(GatherPhase::ToPatch) {
            assert!(
                now.is_multiple_of(RECHECK),
                "it tries again as it re-checks: tick {now}"
            );
            tries += 1;
        }
        was = phase(&t, 5);
    }
    // Each try takes a while to end short again, and a wait of up to one interval follows.
    assert!(tries >= 4, "it keeps trying: {tries} tries");
    // The Guard goes north, out of the pass and away.
    let now = t.sim().tick();
    t.order_at(now + 1, P2, &[id(3)], move_to(900.0, 40.0), false);
    let mined = run_to(&mut t, RECHECK + 120, "it mines A", |t| mines(t, 5, 1));
    assert!(mined > now + 1);
    assert_eq!(remaining(&t, 2), 1500);
}
