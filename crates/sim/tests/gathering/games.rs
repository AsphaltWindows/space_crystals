//! Gathering in a game: a Drone rallied on a patch, and a game replayed and saved in its middle.

use super::*;

/// Question Q0037 point 9 (its proposal): a Drone produced with the rally point on a patch gets a Gather, its
/// default right-click there.
#[test]
fn a_drone_rallied_on_a_patch_gathers_it() {
    // Patch 1, node 2, Robo Portal 3.
    let mut t = flat(16, 14)
        .spawn(PATCH, None, tile(8, 4))
        .spawn(NODE, Some(P1), tile(11, 3))
        .spawn(ObjectKind::RoboPortal, Some(P1), tile(6, 6))
        .start();
    let portal = vec![id(3)];
    let to_portal = |command| Command::Production {
        objects: portal.clone(),
        command,
    };
    t.at(1, P1, to_portal(ProductionCommand::SetRally(Target::Object(id(1)))));
    t.at(1, P1, to_portal(ProductionCommand::Train(DRONE)));
    run_to(&mut t, 200, "the Drone is out", |t| t.sim().object(id(4)).is_some());
    t.step();
    assert_eq!(order(&t, 4), Some(OrderKind::Gather(id(1))));
    run_to(&mut t, 300, "its first delivery", |t| !deliveries(t).is_empty());
    assert_eq!(deliveries(&t)[0].1, 4);
}

/// The plan's determinism acceptance: a game of gathering with a patch mined out, a drop-off set, a node lost and a
/// Drone ordered about replays to the same state, also through JSON; the state serializes and loads as it was, and a
/// state saved in the middle plays on to the same end.
#[test]
fn a_gathering_game_replays_and_round_trips() {
    let mut scenario = flat(16, 12)
        .spawn_with(patch_of(24), tile(4, 2))
        .spawn(PATCH, None, tile(5, 2))
        .spawn(PATCH, None, tile(6, 2))
        .spawn(PATCH, None, tile(7, 2))
        .spawn(NODE, Some(P1), tile(5, 5))
        .spawn(NODE, Some(P1), tile(9, 5))
        .spawn(NODE, Some(P2), tile(5, 9));
    for i in 0..6 {
        scenario = scenario.spawn(
            DRONE,
            Some(P1),
            SpawnAt::Point(Vec2::new(250.0 + 44.0 * i as f32, 290.0)),
        );
    }
    let mut t = scenario.spawn(DRONE, Some(P2), centre(1, 4)).start();
    let drones: Vec<ObjectId> = (8..14).map(ObjectId).collect();
    t.order_at(1, P1, &drones, OrderKind::Gather(id(1)), false);
    t.order_at(2, P2, &[id(14)], OrderKind::Gather(id(2)), false);
    t.ability_at(90, P1, &drones[..3], Ability::SetDropOff(id(6)));
    t.order_at(200, P1, &drones[4..], OrderKind::DropOff(id(6)), false);
    t.order_at(
        260,
        P1,
        &drones[..1],
        OrderKind::Move(Target::Point(Vec2::new(700.0, 500.0))),
        false,
    );
    t.order_at(300, P1, &drones[..1], OrderKind::Gather(id(3)), true);
    t.debug_at(420, DebugCommand::Remove { id: id(5) });
    t.run_until(500);
    let saved: Sim = serde_json::from_str(&serde_json::to_string(t.sim()).unwrap()).unwrap();
    assert_eq!(&saved, t.sim());
    t.run_until(900);
    let sim = t.sim();
    assert!(sim.object(id(1)).is_none(), "the patch of 24 was mined out");
    assert!(deliveries(&t).len() > 10, "{:?}", deliveries(&t));
    assert!(
        deliveries(&t).iter().any(|d| d.1 == 14 && d.2 == 7),
        "player 2 gathers too"
    );
    assert_eq!(dangling_references(sim), []);

    let replayed = t.replay().run();
    assert_eq!(replayed.state_hash(), sim.state_hash());
    assert_eq!(&replayed, sim);
    let json = serde_json::to_string(t.replay()).unwrap();
    let loaded: Replay = serde_json::from_str(&json).unwrap();
    assert_eq!(&loaded, t.replay());
    assert_eq!(loaded.run().state_hash(), sim.state_hash());
    // The save of tick 500 plays on like the game it was taken from: nothing was ordered after it.
    let mut saved = saved;
    for _ in 500..900 {
        saved.step(&Default::default());
    }
    assert_eq!(saved.state_hash(), sim.state_hash());
    assert_eq!(&saved, sim);
}
