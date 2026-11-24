//! A lost drop-off (design change 0135, `> Assigned Drop-off > Drop-off gone`).

use sc_sim::{BuildingOrientation, ConstructionCommand, DebugCommand, Faction};

use super::*;

/// Design change 0135 (its Verification: "Destroy the node a mining group delivers to: on their next return they
/// go to the nearest remaining node. Destroy every node: Drones idle with loads; warp in a node and right-click it to
/// deliver"): the nearest own Egress Node replaces a missing AssignedDropOff as the return trip starts; with none,
/// the Drone idles holding its load.
#[test]
fn drones_that_lose_their_node_take_the_nearest_then_idle_loaded_then_deliver_at_a_warped_in_node() {
    // Patches 1 and 2, nodes 3 (near) and 4 (east), Drones 5 and 6; player 1's Beacon 7 keeps it in the game and
    // warps in the new node.
    let mut t = flat(24, 14)
        .faction(P1, Faction::Colonists)
        .spawn(PATCH, None, tile(5, 2))
        .spawn(PATCH, None, tile(7, 2))
        .spawn(NODE, Some(P1), tile(5, 5))
        .spawn(NODE, Some(P1), tile(14, 5))
        .spawn(DRONE, Some(P1), centre(5, 4))
        .spawn(DRONE, Some(P1), centre(7, 4))
        .spawn(ObjectKind::Beacon, Some(P1), tile(9, 10))
        .start();
    gather(&mut t, 1, P1, &[5], 1);
    gather(&mut t, 1, P1, &[6], 2);
    run_to(&mut t, 200, "both have delivered", |t| {
        !deliveries_of(t, 5).is_empty() && !deliveries_of(t, 6).is_empty()
    });
    assert!(deliveries(&t).iter().all(|d| d.2 == 3), "at the near node");
    // The group's node is destroyed: the next return of each goes to node 4.
    let now = t.sim().tick();
    t.debug_at(now + 1, DebugCommand::Remove { id: id(3) });
    run_to(&mut t, 300, "both deliver again", |t| {
        deliveries_of(t, 5).len() >= 2 && deliveries_of(t, 6).len() >= 2
    });
    for n in [5, 6] {
        assert_eq!(deliveries_of(&t, n)[1].2, 4, "Drone {n} at the node left");
        assert_eq!(drop_off(&t, n), Some(id(4)), "its drop-off now");
    }
    // Every node is destroyed: each idles with its next load.
    let now = t.sim().tick();
    t.debug_at(now + 1, DebugCommand::Remove { id: id(4) });
    run_to(&mut t, 300, "both idle with a load", |t| {
        [5, 6].iter().all(|&n| order(t, n).is_none() && load(t, n) == LOAD)
    });
    // A node warped in from the Beacon (its cost set aside for it), and a right-click on it: a Drop Off.
    let now = t.sim().tick();
    t.debug_at(
        now + 1,
        DebugCommand::SetResources {
            player: P1,
            crystals: 75,
            supplies: 0,
            alloys: 0,
            essence: 0,
            flux: 0,
        },
    );
    let warp = Command::Construction {
        objects: vec![id(7)],
        command: ConstructionCommand::WarpIn {
            kind: NODE,
            orientation: BuildingOrientation::default(),
        },
    };
    t.at(now + 1, P1, warp);
    run_to(&mut t, 600, "the node is warped in", |t| {
        let o = object(t, 7);
        o.kind == NODE && !o.is_transforming()
    });
    let banked = bank(&t, P1);
    assert_eq!(
        (load(&t, 5), load(&t, 6)),
        (LOAD, LOAD),
        "they kept their loads meanwhile"
    );
    let drop = order_of(P1, &[5, 6], OrderKind::DropOff(id(7)));
    assert_eq!(t.sim().validate(&drop), Ok(()));
    let now = t.sim().tick();
    t.at(now + 1, P1, drop.command);
    run_to(&mut t, 300, "both delivered", |t| {
        [5, 6].iter().all(|&n| load(t, n) == 0)
    });
    assert_eq!(bank(&t, P1), banked + 2 * LOAD);
    assert!(deliveries(&t).iter().rev().take(2).all(|d| d.2 == 7), "at the new node");
    t.run(2);
    assert_eq!((order(&t, 5), order(&t, 6)), (None, None), "idle at the node (0127)");
}
