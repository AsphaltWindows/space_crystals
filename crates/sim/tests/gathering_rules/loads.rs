//! Loads: a Gather while carrying (design change 0128), other orders keep the load (0129), the Drop Off of a group
//! of loaded and empty Drones (0140), and the last load of a patch (0138).

use sc_sim::Reject;

use super::*;

/// Design change 0128 (`systems/mechanics/gathering.md > Gather while carrying a load`; its Verification:
/// "Right-click a far patch with a loaded Drone: it drives to the patch, turns around without mining, delivers, then
/// mines that patch on the next trip. A Drone mining that patch is not displaced"): the loaded Drone takes no
/// occupancy, and its Gather is the new patch from then on.
#[test]
fn a_loaded_drone_sent_to_a_far_patch_turns_round_there_without_mining() {
    // Patch 1, node 2, Drone 3; patch 4, five tiles east of patch 1, and Drone 5 north of it.
    let mut t = flat(18, 10)
        .spawn(PATCH, None, tile(5, 2))
        .spawn(NODE, Some(P1), tile(5, 5))
        .spawn(DRONE, Some(P1), centre(5, 4))
        .spawn(PATCH, None, tile(10, 2))
        .spawn(DRONE, Some(P1), centre(10, 0))
        .start();
    gather(&mut t, 1, P1, &[3], 1);
    let loaded = run_to(&mut t, 100, "Drone 3 carries a load", |t| load(t, 3) == LOAD);
    gather(&mut t, loaded + 1, P1, &[3], 4);
    t.run_until(loaded + 1);
    assert_eq!(
        (order(&t, 3), phase(&t, 3)),
        (Some(OrderKind::Gather(id(4))), Some(GatherPhase::ToPatch)),
        "straight to the new patch"
    );
    // It drives to patch 4, and turns round beside it.
    run_to(&mut t, 200, "it turns round", |t| {
        phase(t, 3) == Some(GatherPhase::ToDropOff)
    });
    assert!(beside(&t, 3, 10, 2), "at patch 4: {:?}", object(&t, 3).pos);
    assert_eq!((load(&t, 3), remaining(&t, 4)), (LOAD, 1500), "it mined nothing there");
    run_to(&mut t, 200, "it delivers", |t| !deliveries(t).is_empty());
    assert_eq!(deliveries(&t), [(t.sim().tick(), 3, 2, LOAD)]);
    // The next trip is to patch 4.
    t.step();
    assert_eq!(order(&t, 3), Some(OrderKind::Gather(id(4))));
    run_to(&mut t, 200, "it mines patch 4", |t| mines(t, 3, 4));

    // Drone 5 takes patch 4 as soon as Drone 3 leaves it with its load, and Drone 3 is sent back to it at once.
    run_to(&mut t, 100, "Drone 3 leaves with a load", |t| {
        phase(t, 3) == Some(GatherPhase::ToDropOff)
    });
    let now = t.sim().tick();
    gather(&mut t, now + 1, P1, &[5], 4);
    run_to(&mut t, 20, "Drone 5 mines patch 4", |t| mines(t, 5, 4));
    let now = t.sim().tick();
    gather(&mut t, now + 1, P1, &[3], 4);
    run_to(&mut t, 30, "Drone 3 is back at patch 4", |t| {
        phase(t, 3) == Some(GatherPhase::ToPatch)
    });
    let back = run_to(&mut t, 30, "it turns round again", |t| {
        phase(t, 3) == Some(GatherPhase::ToDropOff)
    });
    assert!(mines(&t, 5, 4), "Drone 5 is not displaced (tick {back})");
    assert_eq!(load(&t, 3), LOAD);
    let before = remaining(&t, 4);
    run_to(&mut t, 40, "Drone 5's load", |t| load(t, 5) == LOAD);
    assert_eq!(remaining(&t, 4), before - LOAD, "Drone 5 mined its whole load");
}

/// Design change 0129 (`systems/mechanics/gathering.md > Other orders keep the load`; its Verification: "Move a
/// loaded Drone away, then right-click a patch: it skips mining there and delivers the old load (0128)"): an order
/// clears no load; only a delivery does.
#[test]
fn a_load_survives_stop_hold_and_move_and_the_next_gather_delivers_it() {
    // Patch 1, node 2, Drone 3.
    let mut t = flat(16, 10)
        .spawn(PATCH, None, tile(5, 2))
        .spawn(NODE, Some(P1), tile(5, 5))
        .spawn(DRONE, Some(P1), centre(5, 4))
        .start();
    gather(&mut t, 1, P1, &[3], 1);
    let loaded = run_to(&mut t, 100, "it carries a load", |t| load(t, 3) == LOAD);
    t.order_at(loaded + 1, P1, &[id(3)], OrderKind::Stop, false);
    t.run_until(loaded + 2);
    assert_eq!((order(&t, 3), load(&t, 3)), (None, LOAD), "a Stop");
    t.order_at(loaded + 3, P1, &[id(3)], OrderKind::Hold, false);
    t.run_until(loaded + 10);
    assert_eq!((order(&t, 3), load(&t, 3)), (Some(OrderKind::Hold), LOAD), "a Hold");
    let away = move_to(600.0, 300.0);
    t.order_at(loaded + 11, P1, &[id(3)], away, false);
    run_to(&mut t, 200, "it arrived", |t| order(t, 3).is_none());
    assert_eq!(load(&t, 3), LOAD, "a Move");
    assert_eq!((bank(&t, P1), deliveries(&t).len()), (BANK, 0));
    // Sent gathering again: to the patch first, where it turns round without mining, then it delivers the old load.
    let now = t.sim().tick();
    gather(&mut t, now + 1, P1, &[3], 1);
    run_to(&mut t, 200, "it turns round at the patch", |t| {
        phase(t, 3) == Some(GatherPhase::ToDropOff)
    });
    assert!(beside(&t, 3, 5, 2), "{:?}", object(&t, 3).pos);
    assert_eq!(remaining(&t, 1), 1500 - LOAD, "only the first load was mined");
    run_to(&mut t, 100, "the old load delivered", |t| !deliveries(t).is_empty());
    assert_eq!(bank(&t, P1), BANK + LOAD);
    run_to(&mut t, 100, "then it mines", |t| mines(t, 3, 1));
}

/// Design change 0140 (`systems/mechanics/gathering.md > Drop Off order > Empty Drones`; its Verification: "Mixed
/// group (some loaded, some empty) right-clicks the node: loaded ones deliver and idle; empty ones walk to the node and
/// stop"): the gate gives a Drop Off to the loaded Drones only, and rejects it for empty ones alone
/// (`Reject::NothingToDeliver`). The client's right-click gives the empty ones a Move to the node instead (slice
/// M3-J2), as here.
#[test]
fn a_mixed_group_s_drop_off_goes_to_its_loaded_drones_only() {
    // Patch 1, node 2, Drone 3 gathers; Drone 4 stands idle and empty, west of the node.
    let mut t = flat(16, 10)
        .spawn(PATCH, None, tile(5, 2))
        .spawn(NODE, Some(P1), tile(5, 5))
        .spawn(DRONE, Some(P1), centre(5, 4))
        .spawn(DRONE, Some(P1), centre(1, 8))
        .start();
    gather(&mut t, 1, P1, &[3], 1);
    let loaded = run_to(&mut t, 100, "Drone 3 carries a load", |t| load(t, 3) == LOAD);
    t.order_at(loaded + 1, P1, &[id(3)], move_to(600.0, 100.0), false);
    run_to(&mut t, 100, "it stands with its load", |t| order(t, 3).is_none());
    let empty = order_of(P1, &[4], OrderKind::DropOff(id(2)));
    assert_eq!(t.sim().validate(&empty), Err(Reject::NothingToDeliver));
    let mixed = order_of(P1, &[3, 4], OrderKind::DropOff(id(2)));
    assert_eq!(t.sim().validate(&mixed), Ok(()));
    let now = t.sim().tick();
    t.at(now + 1, P1, mixed.command);
    let node = TilePos::new(6, 6).center() - Vec2::splat(32.0);
    t.order_at(now + 1, P1, &[id(4)], OrderKind::Move(Target::Point(node)), false);
    t.run_until(now + 1);
    assert_eq!(
        order(&t, 3),
        Some(OrderKind::DropOff(id(2))),
        "the loaded Drone delivers"
    );
    assert!(
        matches!(order(&t, 4), Some(OrderKind::Move(_))),
        "the empty one walks: {:?}",
        order(&t, 4)
    );
    run_to(&mut t, 200, "delivered", |t| !deliveries(t).is_empty());
    assert_eq!(deliveries(&t)[0].1, 3);
    run_to(&mut t, 200, "both idle", |t| {
        order(t, 3).is_none() && order(t, 4).is_none()
    });
    t.run(60);
    assert_eq!((order(&t, 3), order(&t, 4)), (None, None), "neither goes mining");
    assert_eq!((load(&t, 3), load(&t, 4)), (0, 0));
    for n in [3, 4] {
        assert!(
            object(&t, n).pos.distance(node) < 120.0,
            "Drone {n} by the node: {:?}",
            object(&t, n).pos
        );
    }
}

/// Design change 0138 (`systems/mechanics/gathering.md > Last load of a patch`; its Verification: "A patch with 3
/// left: the Drone mines 2 s, carries 3, the patch disappears"), and `systems/mechanics/gathering.md > When a Gather
/// ends` (0123): with no patch left the Gather ends once the 3 are delivered.
#[test]
fn a_patch_with_3_left_gives_a_load_of_3_after_the_full_mining() {
    // Patch 1 holds 3, node 2, Drone 3.
    let mut t = flat(16, 10)
        .spawn_with(patch_of(3), tile(5, 2))
        .spawn(NODE, Some(P1), tile(5, 5))
        .spawn(DRONE, Some(P1), centre(5, 4))
        .start();
    gather(&mut t, 1, P1, &[3], 1);
    let began = run_to(&mut t, 400, "it mines", |t| is_mining(t, 3));
    t.run_until(began + MINING - 2);
    assert_eq!(
        (load(&t, 3), remaining(&t, 1)),
        (0, 3),
        "2 seconds, the full MiningDuration"
    );
    t.step();
    assert_eq!(load(&t, 3), 3);
    assert!(!exists(&t, 1), "the patch is gone");
    run_to(&mut t, 100, "delivered", |t| !deliveries(t).is_empty());
    assert_eq!(
        order(&t, 3),
        Some(OrderKind::Gather(id(1))),
        "still gathering as it unloads"
    );
    t.step();
    assert_eq!((order(&t, 3), load(&t, 3), bank(&t, P1)), (None, 0, BANK + 3));
}
