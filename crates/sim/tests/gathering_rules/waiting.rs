//! Waiting at a crowded patch, the re-check and the return to the last mined patch (design change 0132), and which
//! waiting Drone mines next (0133).

use super::*;

/// Design change 0132 (`> Patch Occupancy > Waiting and re-checking`; its Verification: "Overcrowd one patch: extra
/// Drones wait beside it; when a neighbouring patch frees, a waiting Drone moves over within ~1 s").
#[test]
fn extra_drones_wait_beside_a_crowded_patch_and_one_moves_over_when_a_neighbour_frees() {
    // Patch 1; patch 2 two tiles east of it, mined by player 2's Drone 3, whose node 4 is far to the north-east;
    // player 1's node 5 and Drones 6 to 8 south of patch 1.
    let mut t = flat(20, 12)
        .spawn(PATCH, None, tile(5, 2))
        .spawn(PATCH, None, tile(7, 2))
        .spawn(DRONE, Some(P2), centre(7, 1))
        .spawn(NODE, Some(P2), tile(17, 0))
        .spawn(NODE, Some(P1), tile(5, 7))
        .spawn(DRONE, Some(P1), centre(5, 4))
        .spawn(DRONE, Some(P1), centre(4, 5))
        .spawn(DRONE, Some(P1), centre(6, 5))
        .start();
    gather(&mut t, 1, P2, &[3], 2);
    gather(&mut t, 1, P1, &[6, 7, 8], 1);
    run_to(&mut t, 60, "Drones 7 and 8 wait", |t| {
        is_waiting(t, 7) && is_waiting(t, 8)
    });
    assert!(mines(&t, 6, 1) && mines(&t, 3, 2), "both patches are taken");
    for n in [7, 8] {
        assert_eq!(order(&t, n), Some(OrderKind::Gather(id(1))));
        assert!(
            beside(&t, n, 5, 2),
            "Drone {n} waits beside patch 1: {:?}",
            object(&t, n).pos
        );
    }
    // Player 2's Drone leaves patch 2 with its load.
    let freed = run_to(&mut t, 60, "patch 2 frees", |t| !is_mining(t, 3));
    let moved = run_to(&mut t, RECHECK + 1, "a waiting Drone moves over", |t| {
        [7, 8].iter().any(|&n| order(t, n) == Some(OrderKind::Gather(id(2))))
    });
    assert!(moved - freed <= RECHECK, "within a second: {} ticks", moved - freed);
    let mover = if order(&t, 7) == Some(OrderKind::Gather(id(2))) {
        7
    } else {
        8
    };
    let other = 15 - mover;
    assert_eq!(order(&t, other), Some(OrderKind::Gather(id(1))), "one patch, one Drone");
    run_to(&mut t, 60, "it mines patch 2", |t| mines(t, mover, 2));
}

/// Design change 0132 (`> Returning to the last mined patch`; its Verification: "A Drone redirected from patch A
/// to patch B mines B, delivers, and then returns to B, not A"): the Gather follows the patch the Drone mines.
#[test]
fn a_drone_redirected_from_a_to_b_returns_to_b() {
    // Patch A (1) and patch B (2) two tiles east of it, node 3; Drone 4 north of A takes it first, Drone 5 comes
    // from the south.
    let mut t = flat(16, 10)
        .spawn(PATCH, None, tile(5, 2))
        .spawn(PATCH, None, tile(7, 2))
        .spawn(NODE, Some(P1), tile(5, 5))
        .spawn(DRONE, Some(P1), centre(5, 1))
        .spawn(DRONE, Some(P1), centre(5, 4))
        .start();
    gather(&mut t, 1, P1, &[4, 5], 1);
    run_to(&mut t, 40, "Drone 5 is sent on", |t| {
        order(t, 5) == Some(OrderKind::Gather(id(2)))
    });
    assert!(mines(&t, 4, 1));
    run_to(&mut t, 60, "Drone 5 mines B", |t| mines(t, 5, 2));
    // Drone 4 leaves for good once it has its load: A is free from then on.
    run_to(&mut t, 40, "Drone 4 carries a load", |t| load(t, 4) == LOAD);
    let now = t.sim().tick();
    t.order_at(now + 1, P1, &[id(4)], move_to(100.0, 560.0), false);
    run_to(&mut t, 200, "Drone 5 delivers", |t| !deliveries_of(t, 5).is_empty());
    t.step();
    assert_eq!(order(&t, 5), Some(OrderKind::Gather(id(2))), "back to B");
    run_to(&mut t, 100, "it mines B again", |t| mines(t, 5, 2));
    assert_eq!(remaining(&t, 1), 1500 - LOAD, "A was free, and left alone");
}

/// The miners of a patch, in the order they started: `(tick, Drone)`.
fn miners(t: &mut TestSim, drones: &[u32], until: Tick) -> Vec<(Tick, u32)> {
    let mut started = Vec::new();
    let mut was = vec![false; drones.len()];
    while t.sim().tick() < until {
        t.step();
        for (i, &n) in drones.iter().enumerate() {
            let now = is_mining(t, n);
            if now && !was[i] {
                started.push((t.sim().tick(), n));
            }
            was[i] = now;
        }
    }
    started
}

/// Design change 0133 with the Director's choice (its Verification: "Same setup replayed twice gives the same Drone
/// mining next"): of the Drones waiting at a patch as it frees, the lowest id mines next, whenever it came.
#[test]
fn the_same_game_twice_gives_the_same_drone_mining_next() {
    // Patch 1, node 2 far to the east; Drone 6 is nearest, then Drones 5 and 4, and Drone 3 comes from farthest.
    let game = || {
        let mut t = flat(16, 12)
            .spawn(PATCH, None, tile(5, 2))
            .spawn(NODE, Some(P1), tile(10, 6))
            .spawn(DRONE, Some(P1), centre(1, 6))
            .spawn(DRONE, Some(P1), centre(2, 4))
            .spawn(DRONE, Some(P1), centre(8, 3))
            .spawn(DRONE, Some(P1), centre(5, 4))
            .start();
        gather(&mut t, 1, P1, &[3, 4, 5, 6], 1);
        t
    };
    let (mut first, mut second) = (game(), game());
    let once = miners(&mut first, &[3, 4, 5, 6], 300);
    let twice = miners(&mut second, &[3, 4, 5, 6], 300);
    assert_eq!(once, twice);
    assert_eq!(first.sim().state_hash(), second.sim().state_hash());
    // The nearest mines first; when the patch frees, all three others wait there, and the lowest id goes next.
    assert_eq!(once[0].1, 6);
    assert_eq!(once[1], (once[0].0 + MINING, 3), "{once:?}");
    assert_eq!(once[2], (once[0].0 + 2 * MINING, 4), "{once:?}");
}
