//! A lost drop-off (design change 0135, `factions/colonists/units/drone.md > Assigned Drop-off > Drop-off gone`), and
//! one lost while the Drones are on their way back to it (design change 0170, `factions/colonists/units/drone.md >
//! Destroyed mid-trip`).

use sc_sim::{Ability, BuildingOrientation, ConstructionCommand, DebugCommand, Faction, MoveGoal};

use super::*;

/// The point unit `n` is moving to.
fn goal(t: &TestSim, n: u32) -> Vec2 {
    match t.mover(n).goal {
        MoveGoal::Point(point) => point,
        other => panic!("unit {n} isn't moving to a point: {other:?}"),
    }
}

/// Design change 0170 (`factions/colonists/units/drone.md > Assigned Drop-off > Destroyed mid-trip`; its Verification:
/// "Destroy the node while Drones are returning: they turn toward the nearest other node immediately"): in the tick
/// their node goes, the Drones on their way back to it take the nearest of the nodes left, from where each is, and head
/// there. A Drone mining meanwhile has none until its trip back starts, and then takes the nearest from there (0135).
#[test]
fn drones_on_their_way_back_to_a_node_that_goes_turn_to_the_nearest_other_at_once() {
    // Patches 1 (west) and 2 (east), node 3 south of them, nodes 4 (far west) and 5 (far east); Drones 6 and 7
    // gather patches 1 and 2, and Drone 8 patch 2 too: it waits there, and mines as Drone 7 leaves.
    let mut t = flat(24, 14)
        .faction(P1, Faction::Colonists)
        .spawn(PATCH, None, tile(8, 2))
        .spawn(PATCH, None, tile(14, 2))
        .spawn(NODE, Some(P1), tile(10, 9))
        .spawn(NODE, Some(P1), tile(1, 5))
        .spawn(NODE, Some(P1), tile(21, 5))
        .spawn(DRONE, Some(P1), centre(8, 4))
        .spawn(DRONE, Some(P1), centre(14, 4))
        .spawn(DRONE, Some(P1), centre(13, 5))
        .start();
    gather(&mut t, 1, P1, &[6], 1);
    gather(&mut t, 1, P1, &[7, 8], 2);
    run_to(&mut t, 200, "Drones 6 and 7 on their way back, Drone 8 mining", |t| {
        let back = |n| phase(t, n) == Some(GatherPhase::ToDropOff) && load(t, n) == LOAD;
        back(6) && back(7) && is_mining(t, 8)
    });
    // Partway back.
    t.run(10);
    assert!([6, 7].iter().all(|&n| phase(&t, n) == Some(GatherPhase::ToDropOff)));
    assert!([6, 7, 8].iter().all(|&n| drop_off(&t, n) == Some(id(3))));
    let now = t.sim().tick();
    t.debug_at(now + 1, DebugCommand::Remove { id: id(3) });
    t.run_until(now + 1);
    assert_eq!(
        (drop_off(&t, 6), drop_off(&t, 7)),
        (Some(id(4)), Some(id(5))),
        "the nearest of the nodes left, at once"
    );
    assert_eq!(
        drop_off(&t, 8),
        None,
        "Drone 8 is mining: none until its trip back starts"
    );
    for (n, node) in [(6, 4), (7, 5)] {
        let heading = goal(&t, n).distance(object(&t, node).pos);
        assert!(heading < 128.0, "Drone {n} heads for node {node}: {heading}");
    }
    run_to(&mut t, 300, "all three deliver", |t| {
        [6, 7, 8].iter().all(|&n| !deliveries_of(t, n).is_empty())
    });
    assert_eq!([6, 7, 8].map(|n| deliveries_of(&t, n)[0].2), [4, 5, 5]);
    assert_eq!(
        drop_off(&t, 8),
        Some(id(5)),
        "the nearest from patch 2, as its trip back started"
    );
}

/// Design change 0170 with the node destroyed in combat, the way a game loses one: the Drone on its way back to it is
/// reassigned in the tick the node dies, by the removal path of phase 5e (`Sim::reassign_drop_offs`), and turns for
/// the new node in the next tick's behaviours, the tick it turned in when its return leg looked every tick.
#[test]
fn a_node_destroyed_in_combat_reassigns_the_drone_returning_to_it_in_the_tick_it_dies() {
    // Patch 1; node 2 far south of it, which Drone 4 is set to deliver to, and node 3 west; player 2's Peacekeepers
    // 5 to 8 south of node 2, and its Generator 9, which keeps it in the game.
    let mut builder = flat(24, 24)
        .faction(P1, Faction::Colonists)
        .spawn(PATCH, None, tile(10, 2))
        .spawn(NODE, Some(P1), tile(10, 19))
        .spawn(NODE, Some(P1), tile(2, 4))
        .spawn(DRONE, Some(P1), centre(10, 4));
    for x in [9, 10, 11, 12] {
        builder = builder.spawn(ObjectKind::Peacekeeper, Some(P2), centre(x, 22));
    }
    let mut t = builder.spawn(ObjectKind::Generator, Some(P2), tile(20, 20)).start();
    t.without_auto_targeting()
        .ability_at(1, P1, &ids(&[4]), Ability::SetDropOff(id(2)));
    gather(&mut t, 2, P1, &[4], 1);
    run_to(&mut t, 200, "on its way back", |t| {
        phase(t, 4) == Some(GatherPhase::ToDropOff)
    });
    assert_eq!(drop_off(&t, 4), Some(id(2)));
    let now = t.sim().tick();
    t.order_at(now + 1, P2, &ids(&[5, 6, 7, 8]), OrderKind::Attack(id(2)), false);
    let (lost, left) = (object(&t, 2).pos, object(&t, 3).pos);
    let died = run_to(&mut t, 300, "node 2 destroyed", |t| !exists(t, 2));
    let killed = (t.events().iter())
        .any(|(tick, e)| *tick == died && matches!(e, SimEvent::Died { id: dead, .. } if *dead == id(2)));
    assert!(killed, "in combat, phase 5e");
    assert_eq!(
        (phase(&t, 4), load(&t, 4)),
        (Some(GatherPhase::ToDropOff), LOAD),
        "still on its way back"
    );
    assert_eq!(drop_off(&t, 4), Some(id(3)), "reassigned in the tick node 2 died");
    let heading = goal(&t, 4).distance(lost);
    assert!(
        heading < 128.0,
        "the tick's behaviours ran before the combat: still for node 2, {heading}"
    );
    t.step();
    let heading = goal(&t, 4).distance(left);
    assert!(
        heading < 128.0,
        "for node 3 from the next tick's behaviours on: {heading}"
    );
    run_to(&mut t, 300, "delivered", |t| !deliveries(t).is_empty());
    assert_eq!(deliveries(&t)[0].2, 3, "at node 3");
}

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
