//! One Drone per patch (`factions/colonists/units/drone.md > Patch Occupancy (one Drone per patch)`): two
//! Drones arriving together, Drones waiting with no patch free, an enemy Drone on the patch.

use super::*;

/// `> Patch Occupancy (one Drone per patch)`: two Drones reach a patch in the same tick. The lower id mines; the
/// other "goes to the nearest unoccupied patch to itself instead": the nearer of the two free ones.
#[test]
fn of_two_drones_arriving_together_the_lower_id_mines_and_the_other_goes_to_the_nearest_free_patch() {
    // Patches 1 to 3, node 4, Drone 5 south of patch 1 and Drone 6 north of it, as far away.
    let mut t = flat(16, 10)
        .spawn(PATCH, None, tile(5, 2))
        .spawn(PATCH, None, tile(8, 2))
        .spawn(PATCH, None, tile(2, 2))
        .spawn(NODE, Some(P1), tile(5, 5))
        .spawn(DRONE, Some(P1), centre(5, 4))
        .spawn(DRONE, Some(P1), SpawnAt::Point(Vec2::new(360.0, 32.0)))
        .start();
    gather(&mut t, 1, P1, &[5, 6], 1);
    let arrived = mines_from(&mut t, 5);
    assert_eq!(order(&t, 5), Some(OrderKind::Gather(id(1))));
    // Drone 6 stands a little east of the patch's middle: patch 2, to the east, is the nearer one.
    assert_eq!(
        order(&t, 6),
        Some(OrderKind::Gather(id(2))),
        "redirected in the tick it arrived"
    );
    assert_eq!(phase(&t, 6), Some(GatherPhase::ToPatch));
    let other = mines_from(&mut t, 6);
    // (By then Drone 5 may be on its way to the node: Drone 6 has to turn its momentum round first.)
    assert!(other > arrived, "both mine, each its own patch");
    assert_eq!(order(&t, 5), Some(OrderKind::Gather(id(1))));
    assert_eq!(remaining(&t, 3), 1500, "nobody went to the farther one");
}

/// `> Patch Occupancy`: "If no patch is unoccupied, it waits at the current patch and mines as soon as it frees
/// up". Design change 0133 with the Director's choice: of the Drones waiting there, the lowest id mines next,
/// whoever came first. Design change 0131: a patch its owner has never seen is no patch to go to, re-check after
/// re-check (0132).
#[test]
fn with_no_patch_free_drones_wait_and_the_lowest_id_mines_next() {
    // Patch 1; patch 2, 6 grid units east of it, out of everyone's sight; node 3. Drone 4 comes from the west and
    // arrives last; Drones 5 (south) and 6 (north) arrive together.
    let mut t = flat(16, 10)
        .spawn(PATCH, None, tile(5, 2))
        .spawn(PATCH, None, tile(11, 2))
        .spawn(NODE, Some(P1), tile(5, 5))
        .spawn(DRONE, Some(P1), centre(1, 2))
        .spawn(DRONE, Some(P1), centre(5, 4))
        .spawn(DRONE, Some(P1), centre(5, 0))
        .start();
    assert!(!t.sim().is_visible(P1, object(&t, 2)), "the far patch is in the fog");
    gather(&mut t, 1, P1, &[4, 5, 6], 1);
    let first = mines_from(&mut t, 5);
    assert_eq!(phase(&t, 6), Some(GatherPhase::Waiting { since: first }));
    assert_eq!(
        order(&t, 6),
        Some(OrderKind::Gather(id(1))),
        "it stays: it knows no other patch"
    );
    let third = run_to(&mut t, 30, "Drone 4 waits too", |t| is_waiting(t, 4));
    assert!(third > first);
    // The patch frees as the first mining ends, and the lowest id of the Drones waiting claims it in the next tick.
    assert_eq!(mines_from(&mut t, 4), first + MINING);
    assert!(is_waiting(&t, 6), "it came first, and waits on");
    assert_eq!(load(&t, 5), LOAD);
    assert_eq!(mines_from(&mut t, 6), first + 2 * MINING);
    // The patch gives 8 every 32 frames, its ceiling (`> Patch Occupancy > Consequence`).
    t.run_until(first + 3 * MINING);
    assert_eq!(remaining(&t, 1), 1500 - 3 * LOAD);
    assert_eq!(remaining(&t, 2), 1500);
}

/// `> Patch Occupancy`: "from any player — an enemy Drone mining a patch blocks your Drones too". Design change
/// 0135 (`> Assigned Drop-off > Drop-off gone`): a Drone whose owner has no Egress Node idles holding its load.
#[test]
fn an_enemy_drone_mining_a_patch_blocks_it() {
    // Patch 1, player 1's node 2 and Drone 3 (south, far), player 2's Drone 4 (north, near).
    let mut t = flat(16, 12)
        .spawn(PATCH, None, tile(5, 2))
        .spawn(NODE, Some(P1), tile(5, 7))
        .spawn(DRONE, Some(P1), centre(5, 6))
        .spawn(DRONE, Some(P2), centre(5, 0))
        .start();
    gather(&mut t, 1, P1, &[3], 1);
    gather(&mut t, 1, P2, &[4], 1);
    let enemy = mines_from(&mut t, 4);
    run_to(&mut t, 40, "player 1's Drone waits", |t| is_waiting(t, 3));
    assert!(is_mining(&t, 4));
    assert_eq!(
        mines_from(&mut t, 3),
        enemy + MINING,
        "it mines as soon as the patch frees"
    );
    // Player 2 has no Egress Node: its Drone keeps its load and has nothing more to do.
    assert_eq!((order(&t, 4), load(&t, 4), drop_off(&t, 4)), (None, LOAD, None));
    assert_eq!(bank(&t, P2), 100, "its starting bank");
}
