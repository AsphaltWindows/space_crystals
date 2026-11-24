//! When a Gather ends (design change 0123), a Deploy queued behind a Drop Off (0127), and a patch that runs out
//! while a Drone is on its way to it (0134).

use sc_sim::Faction;
use sc_sim::deploy;

use super::*;

/// Design change 0123 (its Verification: "Shift-queue a Move behind a Gather: the Drone keeps mining until its
/// field is exhausted, delivers its last load, then moves"): the queued order waits for the Gather to end on its
/// own, which it does once no patch is left within the Drone's search radius.
#[test]
fn an_order_queued_behind_a_gather_waits_until_the_field_is_mined_out() {
    // Patch 1 holds 16, node 2, Drone 3; patch 4, two tiles east of patch 1, holds 8.
    let mut t = flat(16, 10)
        .spawn_with(patch_of(16), tile(5, 2))
        .spawn(NODE, Some(P1), tile(5, 5))
        .spawn(DRONE, Some(P1), centre(5, 4))
        .spawn_with(patch_of(8), tile(7, 2))
        .start();
    gather(&mut t, 1, P1, &[3], 1);
    let away = move_to(800.0, 288.0);
    t.order_at(5, P1, &[id(3)], away, true);
    run_to(&mut t, 100, "delivered", |t| !deliveries(t).is_empty());
    t.step();
    assert_eq!(order(&t, 3), Some(OrderKind::Gather(id(1))), "back for more");
    run_to(&mut t, 300, "patch 1 is mined out", |t| !exists(t, 1));
    t.step();
    assert_eq!(order(&t, 3), Some(OrderKind::Gather(id(4))), "on at the other patch");
    let last = run_to(&mut t, 300, "the field is mined out", |t| !exists(t, 4));
    assert_eq!(load(&t, 3), LOAD);
    run_to(&mut t, 100, "the last load delivered", |t| deliveries(t).len() == 3);
    assert!(deliveries(&t).iter().all(|d| d.3 == LOAD), "{:?}", deliveries(&t));
    assert!(t.sim().tick() > last);
    t.step();
    assert_eq!(order(&t, 3), Some(away), "the queued order started");
    run_to(&mut t, 100, "it arrived", |t| order(t, 3).is_none());
    assert_eq!(object(&t, 3).pos, Vec2::new(800.0, 288.0));
    assert_eq!(bank(&t, P1), BANK + 24);
}

/// Design change 0123 (its Verification: "A Drone waiting at an occupied patch with a queued order keeps waiting"):
/// a Drone waiting is still gathering, so the order queued behind its Gather waits, trip after trip.
#[test]
fn a_drone_waiting_at_an_occupied_patch_keeps_its_gather_and_its_queue() {
    // Patch 1, node 2; Drone 3 (south) and Drone 4 (north) reach the patch together, and the lower id mines.
    let mut t = flat(16, 10)
        .spawn(PATCH, None, tile(5, 2))
        .spawn(NODE, Some(P1), tile(5, 5))
        .spawn(DRONE, Some(P1), centre(5, 4))
        .spawn(DRONE, Some(P1), centre(5, 0))
        .start();
    gather(&mut t, 1, P1, &[3, 4], 1);
    let away = move_to(800.0, 300.0);
    t.order_at(2, P1, &[id(4)], away, true);
    run_to(&mut t, 60, "Drone 4 waits", |t| is_waiting(t, 4));
    assert!(is_mining(&t, 3));
    assert_eq!(queued(&t, 4), [away]);
    // Through its waits and its trips, it never leaves the job.
    for _ in 0..400 {
        t.step();
        let now = t.sim().tick();
        assert_eq!(order(&t, 4), Some(OrderKind::Gather(id(1))), "tick {now}");
        assert_eq!(queued(&t, 4), [away], "tick {now}");
    }
    assert!(
        deliveries_of(&t, 4).len() >= 2,
        "it went on gathering: {:?}",
        deliveries(&t)
    );
}

/// Design change 0123 (its Verification: "Deplete a field with a patch 8 GU away: Drones within 6 GU of no patch
/// deliver and go idle"): the search radius is 6 grid units around the Drone (`WorkerDef::search_radius`), and a
/// patch farther away, known as it is, is no patch to go on to.
#[test]
fn with_the_next_patch_8_grid_units_away_the_drones_deliver_and_idle() {
    // Patch 1 holds 16, node 2, Drones 3 (south) and 4 (north); patch 5, 8 grid units east of patch 1, in the sight
    // of player 1's Beacon 6.
    let mut t = flat(20, 10)
        .spawn_with(patch_of(16), tile(5, 2))
        .spawn(NODE, Some(P1), tile(5, 5))
        .spawn(DRONE, Some(P1), centre(5, 4))
        .spawn(DRONE, Some(P1), centre(5, 0))
        .spawn(PATCH, None, tile(13, 2))
        .spawn(ObjectKind::Beacon, Some(P1), tile(13, 4))
        .start();
    assert!(t.sim().is_visible(P1, object(&t, 5)), "a patch player 1 knows");
    gather(&mut t, 1, P1, &[3, 4], 1);
    // Drone 3 mines the first 8 and Drone 4 the last 8, while Drone 3 is on its way back.
    run_to(&mut t, 300, "patch 1 is mined out", |t| !exists(t, 1));
    assert_eq!(load(&t, 4), LOAD);
    run_to(&mut t, 300, "both idle", |t| {
        order(t, 3).is_none() && order(t, 4).is_none()
    });
    assert_eq!(bank(&t, P1), BANK + 16, "both loads delivered");
    assert_eq!((load(&t, 3), load(&t, 4)), (0, 0));
    t.run(100);
    assert_eq!((order(&t, 3), order(&t, 4)), (None, None));
    assert_eq!(remaining(&t, 5), 1500, "nobody went to the far patch");
}

/// Design change 0127 (its Verification: "Right-click the node, then Shift-place a Beacon: after delivering it goes
/// to deploy"): a Drop Off is only a delivery, and the Deploy queued behind it (slice M3-F) starts once the load is
/// in.
#[test]
fn a_deploy_queued_behind_a_drop_off_starts_once_the_load_is_delivered() {
    // Patch 1, node 2, Drone 3; the Beacon's site east of the node, in its sight.
    let site = TilePos::new(9, 5);
    let mut t = flat(16, 10)
        .faction(P1, Faction::Colonists)
        .spawn(PATCH, None, tile(5, 2))
        .spawn(NODE, Some(P1), tile(5, 5))
        .spawn(DRONE, Some(P1), centre(5, 4))
        .start();
    gather(&mut t, 1, P1, &[3], 1);
    let loaded = run_to(&mut t, 100, "it carries a load", |t| load(t, 3) == LOAD);
    let deploy = OrderKind::Deploy { origin: site };
    t.order_at(loaded + 1, P1, &[id(3)], OrderKind::DropOff(id(2)), false);
    t.order_at(loaded + 1, P1, &[id(3)], deploy, true);
    t.run_until(loaded + 1);
    assert_eq!(order(&t, 3), Some(OrderKind::DropOff(id(2))));
    assert_eq!(queued(&t, 3), [deploy], "the gate let the Deploy through");
    run_to(&mut t, 100, "delivered", |t| !deliveries(t).is_empty());
    assert_eq!(bank(&t, P1), BANK + LOAD);
    t.step();
    assert_eq!(order(&t, 3), Some(deploy), "the queued Deploy started");
    run_to(&mut t, 200, "it deploys", |t| object(t, 3).kind == ObjectKind::Beacon);
    assert_eq!(object(&t, 3).placement, Some(deploy::site(site)), "on its site");
    assert_eq!(
        (deliveries(&t).len(), remaining(&t, 1)),
        (1, 1500 - LOAD),
        "it never went back to the patch"
    );
}

/// Design change 0134 (its Verification: "Deplete a patch while a Drone is leaving the node toward it: it walks
/// all the way to the empty spot, then moves to a neighbouring patch"): its Gather goes on to where the patch was,
/// and the Drone looks round only there, "so the search radius is centred on the field".
#[test]
fn a_drone_heading_for_a_patch_that_runs_out_walks_to_its_spot_then_on_to_a_neighbour() {
    // Patch 1 holds 8 and patch 2 is two tiles east of it; node 3 far to the south; Drone 4 mines patch 1 from the
    // north, and Drone 5 sets off from beside the node.
    let mut t = flat(16, 14)
        .spawn_with(patch_of(8), tile(5, 2))
        .spawn(PATCH, None, tile(7, 2))
        .spawn(NODE, Some(P1), tile(4, 10))
        .spawn(DRONE, Some(P1), centre(5, 1))
        .spawn(DRONE, Some(P1), centre(6, 10))
        .start();
    gather(&mut t, 1, P1, &[4, 5], 1);
    let gone = run_to(&mut t, 100, "patch 1 is mined out", |t| !exists(t, 1));
    assert_eq!(
        (order(&t, 5), phase(&t, 5)),
        (Some(OrderKind::Gather(id(1))), Some(GatherPhase::ToPatch)),
        "on its way"
    );
    assert!(
        off_tile(&t, 5, 5, 2) > 128.0,
        "still far from the patch: {:?}",
        object(&t, 5).pos
    );
    // It walks on to the spot, its Gather unchanged, and looks round once it is there.
    let turned = run_to(&mut t, 200, "it goes on", |t| {
        order(t, 5) != Some(OrderKind::Gather(id(1)))
    });
    assert!(turned > gone + 10);
    assert_eq!(order(&t, 5), Some(OrderKind::Gather(id(2))), "the neighbouring patch");
    assert!(
        beside(&t, 5, 5, 2),
        "it got to the empty spot first: {:?}",
        object(&t, 5).pos
    );
    run_to(&mut t, 100, "it mines the neighbour", |t| mines(t, 5, 2));
}
