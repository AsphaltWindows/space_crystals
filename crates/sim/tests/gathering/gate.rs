//! The gate's worker commands (`sim/workers.rs`): every invalid one refused, and changing nothing.

use super::*;

fn command(player: PlayerId, command: Command) -> PlayerCommand {
    PlayerCommand { player, command }
}

fn order_of(player: PlayerId, units: &[u32], kind: OrderKind) -> PlayerCommand {
    let units = units.iter().map(|&n| id(n)).collect();
    command(
        player,
        Command::Order {
            units,
            kind,
            queue: false,
        },
    )
}

fn set_drop_off(player: PlayerId, units: &[u32], node: u32) -> PlayerCommand {
    let units = units.iter().map(|&n| id(n)).collect();
    let ability = Ability::SetDropOff(id(node));
    command(player, Command::Ability { units, ability })
}

/// The gate (`ARCHITECTURE.md > Commands, events and replays`): every invalid Gather, Drop Off and Set Drop-off is
/// rejected with its reason, and changes nothing.
#[test]
fn the_gate_rejects_invalid_worker_commands_and_they_change_nothing() {
    // Patch 1; patch 2 under player 1's plate 9, powered by its Generator 10; patch 3 far in the fog, known from the
    // start (design change 0159); player 1's node 4, Drone 5 and Peacekeeper 6; player 2's node 7 and Drone 8.
    let scenario = flat(24, 10)
        .spawn(PATCH, None, tile(5, 2))
        .spawn(PATCH, None, tile(7, 2))
        .spawn(PATCH, None, tile(22, 8))
        .spawn(NODE, Some(P1), tile(5, 5))
        .spawn(DRONE, Some(P1), centre(5, 4))
        .spawn(ObjectKind::Peacekeeper, Some(P1), centre(3, 4))
        .spawn(NODE, Some(P2), tile(9, 5))
        .spawn(DRONE, Some(P2), centre(9, 4))
        .spawn(ObjectKind::ExtractionPlate, Some(P1), tile(7, 2))
        .spawn(ObjectKind::Generator, Some(P1), tile(14, 6))
        .build();
    let mut t = TestSim::new(scenario.clone());
    t.without_auto_targeting();
    let gather_at = |units: &[u32], patch: u32| order_of(P1, units, OrderKind::Gather(id(patch)));
    let drop_at = |units: &[u32], node: u32| order_of(P1, units, OrderKind::DropOff(id(node)));
    let invalid = [
        (gather_at(&[6], 1), Reject::NoWorkers),
        (gather_at(&[5], 4), Reject::NotGatherable(id(4))),
        (gather_at(&[5], 2), Reject::NotGatherable(id(2))),
        (gather_at(&[5], 99), Reject::UnknownObject(id(99))),
        (gather_at(&[8], 1), Reject::NotOwned(id(8))),
        (gather_at(&[], 1), Reject::NoUnits),
        (drop_at(&[6], 4), Reject::NoWorkers),
        (drop_at(&[5], 7), Reject::NotDropOff(id(7))),
        (drop_at(&[5], 1), Reject::NotDropOff(id(1))),
        (drop_at(&[5], 99), Reject::UnknownObject(id(99))),
        (drop_at(&[8], 4), Reject::NotOwned(id(8))),
        // Design change 0140: no Drone of them carries anything.
        (drop_at(&[5, 6], 4), Reject::NothingToDeliver),
        (set_drop_off(P1, &[6], 4), Reject::LacksAbility),
        (set_drop_off(P1, &[5], 7), Reject::NotDropOff(id(7))),
        (set_drop_off(P1, &[5], 5), Reject::NotDropOff(id(5))),
        (set_drop_off(P1, &[5], 99), Reject::UnknownObject(id(99))),
        (set_drop_off(P1, &[8], 4), Reject::NotOwned(id(8))),
        (set_drop_off(P1, &[99], 4), Reject::NoUnits),
    ];
    for (cmd, reject) in &invalid {
        assert_eq!(t.sim().validate(cmd), Err(*reject), "{cmd:?}");
        t.at(2, cmd.player, cmd.command.clone());
    }
    let mut twin = TestSim::new(scenario);
    twin.without_auto_targeting();
    t.run(3);
    twin.run(3);
    assert_eq!(t.sim().state_hash(), twin.sim().state_hash(), "they changed nothing");

    // The valid forms; the units that aren't workers ignore them (a valid Drop Off is `gathering_rules`, for 0140).
    let valid = [gather_at(&[5, 6], 1), set_drop_off(P1, &[5, 6], 4), gather_at(&[5], 3)];
    for cmd in &valid {
        assert_eq!(t.sim().validate(cmd), Ok(()), "{cmd:?}");
    }
    t.at(4, P1, valid[0].command.clone());
    t.run(1);
    assert_eq!(order(&t, 5), Some(OrderKind::Gather(id(1))));
    assert_eq!(order(&t, 6), None, "the Peacekeeper ignored it");
    // The plate mines its patch whatever the Drones do, and the Drone never turns to the plated patch: with the
    // enemy on patch 1 it waits.
    t.at(5, P2, order_of(P2, &[8], OrderKind::Gather(id(1))).command);
    t.run(120);
    assert!(remaining(&t, 2) < 1500, "the plate mined");
    assert_eq!(order(&t, 5), Some(OrderKind::Gather(id(1))));
    assert_eq!(order(&t, 8), Some(OrderKind::Gather(id(1))));
}
