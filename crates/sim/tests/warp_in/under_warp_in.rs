//! A building under Warp In: no drop-off until it completes (design change 0116), its Beacon Capacity kept
//! (0112), its owner kept in the game (0113); and destroyed mid-Warp-In, it refunds nothing (0115) and, the last
//! structure of its owner, defeats them.

use super::*;

/// Design change 0116 ("Production, drop-off, research and abilities start at completion"; "A Drone can't deliver to
/// an Egress Node under Warp In"): an Egress Node being warped in takes no drop-off. Player 1's Drone 6 can't be sent to it, nor given it as its AssignedDropOff
/// ([`Reject::NotDropOff`]); sent to gather patch 7 next to it, it mines a load and, with no drop-off to go back to,
/// idles with it (question Q0037 point 8). Once the node is complete, it delivers there.
#[test]
fn an_egress_node_being_warped_in_takes_no_drop_off() {
    let mut t = base()
        .spawn(Drone, Some(P1), SpawnAt::Point(at(6, 9).center()))
        .spawn(CrystalPatch, None, tile(5, 10))
        .start();
    t.debug_at(1, fund(P1, 75))
        .at(1, P1, construction(&[1], warp(EgressNode)));
    t.step();
    let drone = [id(6)];
    let drop_off = PlayerCommand {
        player: P1,
        command: Command::Order {
            units: drone.to_vec(),
            kind: OrderKind::DropOff(id(1)),
            queue: false,
        },
    };
    let assign = PlayerCommand {
        player: P1,
        command: Command::Ability {
            units: drone.to_vec(),
            ability: Ability::SetDropOff(id(1)),
        },
    };
    assert_eq!(t.sim().validate(&drop_off), Err(Reject::NotDropOff(id(1))));
    assert_eq!(t.sim().validate(&assign), Err(Reject::NotDropOff(id(1))));
    t.order_at(2, P1, &drone, OrderKind::Gather(id(7)), false);
    t.run_until(200);
    let o = object(&t, 6);
    let worker = o.worker.unwrap();
    assert_eq!(
        (worker.load, worker.drop_off),
        (8, None),
        "a load, and nowhere to take it"
    );
    assert!(o.orders.as_ref().unwrap().current.is_none(), "idle");
    t.run_until(320);
    assert!(!object(&t, 1).is_transforming());
    assert_eq!(t.sim().validate(&assign), Ok(()));
    next(&mut t, drop_off.command);
    t.run(200);
    let delivered = t
        .events()
        .iter()
        .any(|(_, e)| matches!(e, SimEvent::ResourcesDelivered { worker, drop_off, amount: 8, .. } if *worker == id(6) && *drop_off == id(1)));
    assert!(delivered, "delivered at the complete Egress Node");
}

/// Design change 0112's Verification: "Warp In a building: available capacity doesn't dip during the Warp In"
/// (`factions/colonists/economy.md > Beacon Capacity Retention`: "A building being warped in keeps providing 5";
/// `Sim::army_cap`). Player 1's two Beacons provide 10, all the way through a Robo Portal's Warp In on Beacon 1 and
/// after it. And the plan's acceptance, "Destroyed mid-Warp-In": a Bio Portal being warped in on Beacon 1 that is
/// removed takes its 5 with it.
#[test]
fn a_building_being_warped_in_keeps_providing_beacon_capacity() {
    let available = |t: &TestSim| t.sim().army_cap(P1).map(|cap| cap.available);
    let mut t = base().start();
    t.debug_at(1, fund(P1, 1000))
        .at(2, P1, construction(&[1], warp(RoboPortal)));
    t.step();
    assert_eq!(available(&t), Some(10), "two Beacons");
    while t.sim().tick() < 700 {
        t.step();
        assert_eq!(available(&t), Some(10), "tick {}", t.sim().tick());
    }
    assert_eq!(completions(&t), [(641, 1, RoboPortal)]);
    let mut t = base().start();
    t.debug_at(1, fund(P1, 1000))
        .at(1, P1, construction(&[1], warp(BioPortal)))
        .debug_at(100, DebugCommand::Remove { id: id(1) });
    t.run_until(99);
    assert_eq!(available(&t), Some(10));
    t.step();
    assert_eq!(available(&t), Some(5), "destroyed mid-Warp-In: 5 fewer");
}

/// Design change 0113 ("A Beacon under deployment and a building under Warp In are structures and keep their owner
/// undefeated", `core/game_rules.md > Defeat Condition`): player 1's only structure, a Beacon warping in an Egress
/// Node, keeps them in the game all the way through the Warp In and after it.
#[test]
fn a_building_being_warped_in_keeps_its_owner_undefeated() {
    let mut t = colonists(ScenarioBuilder::new(&flat(24, 16)))
        .spawn(Beacon, Some(P1), tile(8, 6))
        .spawn(Generator, Some(P2), tile(20, 12))
        .start();
    t.debug_at(1, fund(P1, 75))
        .at(1, P1, construction(&[1], warp(EgressNode)));
    for _ in 0..400 {
        t.step();
        assert!(!t.sim().player(P1).unwrap().is_defeated(), "tick {}", t.sim().tick());
    }
    assert_eq!(completions(&t), [(320, 1, EgressNode)]);
    assert_eq!(t.sim().outcome(), None);
}

/// Design change 0115 ("A building destroyed mid-Warp-In refunds nothing; Beacon and cost are both lost"), and the
/// trap it names: a Robo Portal and a Bio Portal are producers from their first frame, yet the refunds of a destroyed
/// producer (design changes 0084, 0085) find nothing to give back. Each portal, warping in on Beacon 1, is destroyed
/// by player 2's six Peacekeepers; player 1's bank stays as the Warp In left it, and no refund event is sent.
#[test]
fn a_portal_destroyed_mid_warp_in_refunds_nothing() {
    for (kind, cost, frames) in [(RoboPortal, 250, 640), (BioPortal, 150, 480)] {
        let mut builder = colonists(ScenarioBuilder::new(&flat(24, 16)))
            .spawn(Beacon, Some(P1), tile(8, 6)) // 1
            .spawn(Generator, Some(P1), tile(1, 1)) // 2: player 1 stays in the game
            .spawn(Generator, Some(P2), tile(20, 12)); // 3
        for (x, y) in [(5, 6), (12, 7), (9, 3), (8, 10), (5, 9), (12, 4)] {
            builder = builder.spawn(Peacekeeper, Some(P2), SpawnAt::Point(at(x, y).center()));
        }
        let mut t = builder.start();
        let attackers: Vec<ObjectId> = (4..=9).map(id).collect();
        t.without_auto_targeting()
            .debug_at(1, fund(P1, 1000))
            .at(1, P1, construction(&[1], warp(kind)))
            .order_at(2, P2, &attackers, OrderKind::Attack(id(1)), false);
        while t.sim().object(id(1)).is_some() {
            t.step();
            assert!(t.sim().tick() < frames, "{kind:?} destroyed before it is complete");
            assert_eq!(bank(&t, P1), 1000 - cost, "{kind:?}: tick {}", t.sim().tick());
        }
        let refunds = t.events().iter().filter(|(_, e)| {
            matches!(
                e,
                SimEvent::ProductionCancelled { .. }
                    | SimEvent::ConstructionCancelled { .. }
                    | SimEvent::TransformCancelled { .. }
            )
        });
        assert_eq!(refunds.count(), 0, "{kind:?}");
        assert!(!t.sim().player(P1).unwrap().is_defeated(), "its Generator stands");
    }
}

/// The plan's acceptance, "Destroyed mid-Warp-In" (`factions/colonists/economy.md > Vulnerability`: "If destroyed
/// mid-Warp-In, the Beacon is destroyed along with it"): the object is removed through the normal path with no refund
/// (design change 0115), and its owner, for whom it was the last structure, stays in the game until then and is
/// defeated in the same tick (`core/game_rules.md > Defeat Condition`, design change 0113): player 2's Peacekeepers
/// destroy player 1's only structure, a Beacon warping in an Egress Node.
#[test]
fn a_destroyed_warp_in_is_lost_and_its_last_structure_defeats_its_owner() {
    let mut builder = colonists(ScenarioBuilder::new(&flat(24, 16)))
        .spawn(Beacon, Some(P1), tile(8, 6)) // 1
        .spawn(Generator, Some(P2), tile(20, 12)); // 2
    for (x, y) in [(6, 7), (11, 7), (9, 4), (8, 9)] {
        builder = builder.spawn(Peacekeeper, Some(P2), SpawnAt::Point(at(x, y).center()));
    }
    let mut t = builder.start();
    t.without_auto_targeting()
        .debug_at(1, fund(P1, 75))
        .at(1, P1, construction(&[1], warp(EgressNode)))
        .order_at(2, P2, &[id(3), id(4), id(5), id(6)], OrderKind::Attack(id(1)), false);
    let mut died = None;
    for _ in 0..300 {
        t.step();
        if t.sim().object(id(1)).is_none() {
            died = Some(t.sim().tick());
            break;
        }
        assert!(object(&t, 1).is_transforming());
        assert!(!t.sim().player(P1).unwrap().is_defeated(), "tick {}", t.sim().tick());
    }
    let died = died.expect("destroyed while warping in");
    assert!(died < 320, "before it is complete");
    let events: Vec<&SimEvent> = t
        .events()
        .iter()
        .filter(|(tick, _)| *tick == died)
        .map(|(_, e)| e)
        .collect();
    assert!(
        events
            .iter()
            .any(|e| matches!(e, SimEvent::Died { id: dead, kind: EgressNode, .. } if *dead == id(1)))
    );
    assert!(events.contains(&&SimEvent::Removed(id(1))));
    assert!(events.contains(&&SimEvent::PlayerDefeated { player: P1 }));
    assert!(events.contains(&&SimEvent::GameOver { winners: vec![P2] }));
    assert_eq!(bank(&t, P1), 0, "no refund");
    assert_eq!(cancellations(&t), []);
    assert!(t.sim().player(P1).unwrap().is_defeated());
    assert_eq!(dangling_references(t.sim()), []);
}
