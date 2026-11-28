//! One Drone's loop (`factions/colonists/units/drone.md > Gathering`): its trip, pinned at 73 ticks (`TRIP`), and
//! mining interrupted by another order (design change 0130, `systems/mechanics/gathering.md > Interrupted mining`).

use super::*;

/// Slice H's first acceptance point: one Drone, the patch 2 grid units from the node. It mines for 32 frames, the
/// patch loses 8 as the mining ends, it unloads for 2 frames, and the bank rises by 8 at the end of each drop-off.
/// The trip time with DragMovement (slice M3-G) is pinned here and recorded in `economy/gathering.rs`: 73 ticks
/// with the Drone's numbers of design change 0205 (81 before them).
#[test]
fn one_drone_carries_loads_of_eight_from_the_patch_to_the_node() {
    let mut t = one_patch(1500).start();
    let start = bank(&t, P1);
    gather(&mut t, 1, P1, &[3], 1);
    // The first leg: from rest to within reach of the patch, 73 su, turning north as it sets off.
    let began = mines_from(&mut t, 3);
    assert_eq!(began, 17);
    assert_eq!(drop_off(&t, 3), Some(id(2)), "auto-assigned: the nearest Egress Node");
    assert_eq!(order(&t, 3), Some(OrderKind::Gather(id(1))));
    // `factions/colonists/units/drone.md > Gathering`: MiningDuration. The load is taken as the mining ends, in the
    // economy phase.
    t.run_until(began + MINING - 2);
    assert_eq!((load(&t, 3), remaining(&t, 1)), (0, 1500));
    assert_eq!(phase(&t, 3), Some(GatherPhase::Mining { done: MINING - 1 }));
    t.step();
    assert_eq!((load(&t, 3), remaining(&t, 1)), (LOAD, 1500 - LOAD));
    assert_eq!(phase(&t, 3), Some(GatherPhase::ToDropOff));
    assert_eq!(bank(&t, P1), start, "nothing enters the pool before the drop-off");
    // The way back, then DropOffDuration.
    let unloading = run_to(&mut t, 40, "it unloads", |t| {
        matches!(phase(t, 3), Some(GatherPhase::DroppingOff { .. }))
    });
    assert_eq!(
        unloading,
        began + MINING - 1 + 20,
        "20 ticks from the patch to within reach of the node: thrust, top speed never reached, braking"
    );
    assert_eq!((load(&t, 3), bank(&t, P1)), (LOAD, start));
    t.run(DROP_OFF - 1);
    assert_eq!((load(&t, 3), bank(&t, P1)), (0, start + LOAD));
    assert_eq!(deliveries(&t), [(unloading + 1, 3, 2, LOAD)]);
    // And again, for good: a delivery every 73 ticks.
    t.run_until(unloading + 1 + 5 * TRIP);
    let ticks: Vec<Tick> = deliveries(&t).iter().map(|d| d.0).collect();
    assert_eq!(ticks, [69, 142, 215, 288, 361, 434]);
    assert_eq!(bank(&t, P1), start + 6 * LOAD);
    assert_eq!(
        remaining(&t, 1) + load(&t, 3) + 6 * LOAD,
        1500,
        "nothing lost on the way"
    );
    assert_eq!(
        order(&t, 3),
        Some(OrderKind::Gather(id(1))),
        "a Gather never ends on its own"
    );
    assert_eq!(t.sim().returning_drones(id(2)), [id(3)]);
}

/// Design change 0130 (`systems/mechanics/gathering.md > Interrupted mining`; its Verification: "Order a mining Drone
/// away at frame 20: it carries nothing; another waiting Drone starts mining that patch immediately"): the Drone that
/// waited mines from that very tick.
#[test]
fn interrupted_mining_collects_nothing_and_frees_the_patch_at_once() {
    // Patch 1, node 2, Drone 3 (south) mines, Drone 4 (north) waits.
    let mut t = one_patch(1500).spawn(DRONE, Some(P1), centre(5, 0)).start();
    gather(&mut t, 1, P1, &[3, 4], 1);
    let began = mines_from(&mut t, 3);
    assert!(is_waiting(&t, 4));
    let away = OrderKind::Move(Target::Point(Vec2::new(800.0, 288.0)));
    t.order_at(began + 20, P1, &[id(3)], away, false);
    t.run_until(began + 20);
    assert_eq!((order(&t, 3), load(&t, 3), remaining(&t, 1)), (Some(away), 0, 1500));
    assert_eq!(phase(&t, 4), Some(GatherPhase::Mining { done: 1 }));
    t.run_until(began + 20 + MINING - 1);
    assert_eq!((load(&t, 4), remaining(&t, 1)), (LOAD, 1500 - LOAD));
}
