//! Where the loads go (`factions/colonists/units.md > Drone > Assigned Drop-off`, `> Drop Off order`): Set
//! Drop-off Location in the middle of mining and of a return, the Drop Off order, a lost Egress Node.

use super::*;

/// `> Assigned Drop-off > Set Drop-off Location command`: it "does not interrupt the Drone's current action", and
/// it redirects "a trip already in progress".
#[test]
fn set_drop_off_redirects_the_return_without_interrupting_the_mining() {
    // Patch 1, nodes 2 (near) and 3 (east), Drone 4.
    let mut t = flat(16, 10)
        .spawn(PATCH, None, tile(5, 2))
        .spawn(NODE, Some(P1), tile(5, 5))
        .spawn(NODE, Some(P1), tile(10, 5))
        .spawn(DRONE, Some(P1), centre(5, 4))
        .start();
    gather(&mut t, 1, P1, &[4], 1);
    let began = mines_from(&mut t, 4);
    assert_eq!(drop_off(&t, 4), Some(id(2)));
    // In the middle of the mining.
    t.ability_at(began + 10, P1, &[id(4)], Ability::SetDropOff(id(3)));
    t.run_until(began + 10);
    assert_eq!(drop_off(&t, 4), Some(id(3)));
    assert_eq!(
        phase(&t, 4),
        Some(GatherPhase::Mining { done: 11 }),
        "the mining goes on"
    );
    assert_eq!(t.sim().returning_drones(id(3)), [id(4)]);
    assert_eq!(t.sim().returning_drones(id(2)), []);
    run_to(&mut t, 200, "the first delivery", |t| !deliveries(t).is_empty());
    assert_eq!(deliveries(&t)[0].2, 3, "to the new drop-off");
    assert_eq!(load(&t, 4), 0);
    // On the way back with the next load: the trip turns to the other node.
    run_to(&mut t, 200, "it carries the next load", |t| {
        phase(t, 4) == Some(GatherPhase::ToDropOff) && load(t, 4) == LOAD
    });
    t.run(4);
    let now = t.sim().tick();
    t.ability_at(now + 1, P1, &[id(4)], Ability::SetDropOff(id(2)));
    run_to(&mut t, 200, "the second delivery", |t| deliveries(t).len() == 2);
    assert_eq!(deliveries(&t)[1].2, 2);
    assert_eq!(
        order(&t, 4),
        Some(OrderKind::Gather(id(1))),
        "its Gather went on throughout"
    );
    assert_eq!(bank(&t, P1), 100 + 2 * LOAD);
}

/// `> Unit Commands > Drop Off Resources`: a Drone ordered away keeps its load (design change 0129, `> Other orders
/// keep the load`); a Drop Off order is only a delivery, at the drop-off it names, after which the Drone idles there
/// (design change 0127, `> Drop Off order`).
#[test]
fn a_drop_off_order_delivers_the_load_and_the_drone_idles_at_the_node() {
    // Patch 1, nodes 2 (near) and 3 (east), Drone 4.
    let mut t = flat(16, 10)
        .spawn(PATCH, None, tile(5, 2))
        .spawn(NODE, Some(P1), tile(5, 5))
        .spawn(NODE, Some(P1), tile(10, 5))
        .spawn(DRONE, Some(P1), centre(5, 4))
        .start();
    gather(&mut t, 1, P1, &[4], 1);
    let loaded = run_to(&mut t, 100, "it carries a load", |t| load(t, 4) == LOAD);
    // Ordered away: it keeps the load, and idles with it.
    let away = OrderKind::Move(Target::Point(Vec2::new(480.0, 224.0)));
    t.order_at(loaded + 1, P1, &[id(4)], away, false);
    run_to(&mut t, 100, "it arrived", |t| order(t, 4).is_none());
    assert_eq!((load(&t, 4), bank(&t, P1)), (LOAD, 100));
    assert_eq!(t.sim().returning_drones(id(2)), [], "it isn't gathering");
    // Drop Off at the other node, which doesn't become its AssignedDropOff.
    let now = t.sim().tick();
    t.order_at(now + 1, P1, &[id(4)], OrderKind::DropOff(id(3)), false);
    run_to(&mut t, 200, "delivered", |t| !deliveries(t).is_empty());
    assert_eq!(deliveries(&t)[0].2, 3);
    assert_eq!(bank(&t, P1), 100 + LOAD);
    t.step();
    assert_eq!(order(&t, 4), None, "the order is complete");
    assert_eq!(drop_off(&t, 4), Some(id(2)));
    // It stays at the node: it doesn't go back to its patch.
    t.run(100);
    assert_eq!((order(&t, 4), deliveries(&t).len()), (None, 1));
    let (at, node) = (object(&t, 4).pos, object(&t, 3).pos);
    assert!(at.distance(node) < 96.0, "beside node 3: {at:?}, the node at {node:?}");
}

/// Design change 0135 (`> Assigned Drop-off > Drop-off gone`): a Drone whose Egress Node is destroyed assigns the
/// nearest one as it starts its return; with none left it idles holding its load, and delivers it when it is sent
/// gathering again (0128: to the patch first, where it turns round without mining).
#[test]
fn drones_of_a_lost_egress_node_assign_another_or_idle_with_their_load() {
    // Patch 1, nodes 2 (near) and 3 (east), Drone 4; and a Beacon, 5, so that player 1 isn't defeated with its
    // last node (`core/game_rules.md > Defeat Condition`).
    let mut t = flat(16, 10)
        .spawn(PATCH, None, tile(5, 2))
        .spawn(NODE, Some(P1), tile(5, 5))
        .spawn(NODE, Some(P1), tile(10, 5))
        .spawn(DRONE, Some(P1), centre(5, 4))
        .spawn(ObjectKind::Beacon, Some(P1), tile(14, 8))
        .start();
    gather(&mut t, 1, P1, &[4], 1);
    let began = mines_from(&mut t, 4);
    t.debug_at(began + 5, DebugCommand::Remove { id: id(2) });
    t.run_until(began + 5);
    assert_eq!(drop_off(&t, 4), None, "its drop-off is gone");
    assert!(is_mining(&t, 4), "the mining goes on");
    run_to(&mut t, 200, "delivered", |t| !deliveries(t).is_empty());
    assert_eq!(deliveries(&t)[0].2, 3, "at the node left");
    assert_eq!(drop_off(&t, 4), Some(id(3)), "which is its drop-off now");
    // The last node goes while the Drone is on its way to it with the next load.
    run_to(&mut t, 200, "it returns with a load", |t| {
        phase(t, 4) == Some(GatherPhase::ToDropOff) && load(t, 4) == LOAD
    });
    let now = t.sim().tick();
    t.debug_at(now + 3, DebugCommand::Remove { id: id(3) });
    t.run_until(now + 4);
    assert_eq!((order(&t, 4), load(&t, 4), drop_off(&t, 4)), (None, LOAD, None));
    assert_eq!(dangling_references(t.sim()), []);
    t.run(40);
    assert_eq!(object(&t, 4).pos, object(&t, 4).prev_pos, "it stands");
    // A new node, and a new Gather: the old load is delivered at the new node, the nearest.
    let now = t.sim().tick();
    t.debug_at(
        now + 1,
        DebugCommand::Spawn {
            kind: NODE,
            owner: Some(P1),
            at: tile(2, 5),
            orientation: Default::default(),
        },
    );
    gather(&mut t, now + 2, P1, &[4], 1);
    run_to(&mut t, 200, "the kept load delivered", |t| deliveries(t).len() == 2);
    let node = deliveries(&t)[1].2;
    assert_eq!(object(&t, node).kind, NODE);
    assert_eq!(drop_off(&t, 4), Some(id(node)));
    assert_eq!(bank(&t, P1), 100 + 2 * LOAD);
}
