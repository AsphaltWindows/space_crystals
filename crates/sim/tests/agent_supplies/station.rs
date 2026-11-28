//! Design change 0256's Verification at the station (`factions/syndicate/units/agent.md > Gathering > Supplies`):
//! - point 1, "any number pick up at once, as at patches": four Agents, one off each face, pick up together; a crowd
//!   sent from one side spreads round the station (`orders::gather::station::spot`);
//! - point 2, "A Supply leaves the station when the pick-up ends. A pick-up starts only if a Supply remains that no
//!   running pick-up has claimed": a station holding 2 keeps them until each pick-up's last frame, an interrupted
//!   pick-up takes none (design change 0130), and of four Agents only two pick up;
//! - point 3, "An empty station: the owner gets a notification, and every Agent picking up there stands next to it
//!   indefinitely, keeping its job, waiting for the next supply delivery": `SimEvent::SupplyStationEmpty`, once for
//!   each player whose Agents gather there, and the Agents resume when it refills.

use super::*;

/// Player 1's Tunnel (1) on tiles (12, 6) to (15, 9), its Supplies hatch 5 grid units east of a station (2) of
/// `supplies` on tiles (5, 7) to (6, 8), x 320..448 and y 448..576; and an Agent a tile off each of the station's
/// faces, 78 su from it, ids 3 to 6: north, east, south and west.
fn ringed(supplies: SpawnSettings) -> ScenarioBuilder {
    flat(30, 20)
        .spawn(TUNNEL, Some(P1), tile(12, 6))
        .spawn_with(supplies, tile(5, 7))
        .spawn(AGENT, Some(P1), centre(5, 5))
        .spawn(AGENT, Some(P1), centre(8, 8))
        .spawn(AGENT, Some(P1), centre(6, 10))
        .spawn(AGENT, Some(P1), centre(3, 7))
}

/// Point 1: four Agents, one off each face of a station that holds plenty, all pick up in the same ticks, and none
/// waits.
#[test]
fn four_agents_pick_up_at_one_station_at_once() {
    let mut t = ringed(station_of(100)).start();
    let all = agents(4);
    gather(&mut t, 1, &all, STATION);
    run_to(&mut t, &all, &[STATION], 60, "all four pick up at once", |t| {
        picking_up_at(t, &all, STATION).len() == 4
    });
    for _ in 0..PICK_UP - 2 {
        step(&mut t, &all, &[STATION]);
        assert!(all.iter().all(|&n| !is_waiting(&t, n)), "tick {}", t.sim().tick());
    }
    assert_eq!(picking_up_at(&t, &all, STATION).len(), 4);
    assert_eq!(supplies(&t, STATION), 100, "none has left yet");
    run_to(&mut t, &all, &[STATION], 2, "the pick-ups end", |t| {
        all.iter().all(|&n| load(t, n) == (CARRY, Resource::Supplies))
    });
    assert_eq!(supplies(&t, STATION), 96);
}

/// Point 1 with a crowd: six Agents sent together from one side, south of the station, each head for a spot of its
/// own round it (`orders::gather::station::spot`), and at least four pick up at once. None waits for good: each
/// delivers.
#[test]
fn a_crowd_from_one_side_spreads_round_the_station() {
    let mut t = field(6, 3, station_of(100)).start();
    let all = agents(6);
    gather(&mut t, 1, &all, STATION);
    let mut most = 0;
    for _ in 0..1500 {
        step(&mut t, &all, &[STATION]);
        most = most.max(picking_up_at(&t, &all, STATION).len());
    }
    assert!(most >= 4, "at most {most} picked up at once");
    let delivered = supply_deliveries(&t);
    for n in all {
        assert!(
            delivered.iter().any(|d| d.1 == n),
            "Agent {n} never delivered: {delivered:?}"
        );
    }
}

/// Point 2: a Supply leaves the station as its pick-up ends, and not before. The pick-up's PickUpDuration counts
/// the tick it began in and the one it ended in (`docs/architecture/economy.md > Economy > Recipe timing`): the station holds 2
/// through its first 47 frames, and 1 at the end of the 48th, when the Agent carries the Supply. A pick-up stopped
/// before then takes nothing (design change 0130: "interrupted work collects nothing"), and the Supply it claimed is
/// free again: the next Gather picks it up.
#[test]
fn a_supply_leaves_the_station_as_its_pick_up_ends() {
    let mut t = field(1, 3, station_of(2)).start();
    let all = [3];
    gather(&mut t, 1, &all, STATION);
    let began = run_to(&mut t, &all, &[STATION], 100, "the pick-up", |t| is_picking_up(t, 3));
    while load(&t, 3).0 == 0 {
        assert_eq!(supplies(&t, STATION), 2, "tick {}", t.sim().tick());
        assert!(is_picking_up(&t, 3));
        step(&mut t, &all, &[STATION]);
    }
    let ended = t.sim().tick();
    assert_eq!(ended - began + 1, PICK_UP);
    assert_eq!(supplies(&t, STATION), 1);
    assert_eq!(load(&t, 3), (CARRY, Resource::Supplies));

    // Stopped in its 21st frame.
    let mut t = field(1, 3, station_of(2)).start();
    gather(&mut t, 1, &all, STATION);
    let began = run_to(&mut t, &all, &[STATION], 100, "the pick-up", |t| is_picking_up(t, 3));
    t.order_at(began + 20, P1, &[id(3)], OrderKind::Stop, false);
    run_to(&mut t, &all, &[STATION], 20, "stopped", |t| phase(t, 3).is_none());
    t.run(PICK_UP);
    assert_eq!((supplies(&t, STATION), load(&t, 3).0), (2, 0), "it took nothing");
    let again = t.sim().tick() + 1;
    gather(&mut t, again, &all, STATION);
    run_to(&mut t, &all, &[STATION], PICK_UP + 5, "a pick-up from the start", |t| {
        load(t, 3) == (CARRY, Resource::Supplies)
    });
    assert_eq!(supplies(&t, STATION), 1);
}

/// The DeliveryInterval of the stations of [`only_unclaimed_supplies_are_picked_up_and_an_empty_station_keeps_its_agents`]:
/// long enough for the Agents that picked up to deliver and come back.
const INTERVAL: Tick = 400;

/// Point 2: of four Agents at a station that holds 2, only two pick up, one Supply each; the two others wait beside
/// it. Point 3: the last pick-up's end empties it, and player 1 is notified once, in that tick. Every Agent stands
/// beside the empty station, keeping its Gather (the two that picked up deliver and come back), until the next
/// delivery: then they all pick up again, and the 4 Supplies it brings run out the same way.
#[test]
fn only_unclaimed_supplies_are_picked_up_and_an_empty_station_keeps_its_agents() {
    // Deliveries 400 ticks apart: the delivery of 4 comes while all four wait.
    let mut t = ringed(station_with(2, DELIVERY_SIZE, INTERVAL)).start();
    let all = agents(4);
    gather(&mut t, 1, &all, STATION);
    run_to(&mut t, &all, &[STATION], 60, "two pick-ups", |t| {
        picking_up_at(t, &all, STATION).len() == 2
    });
    let first = picking_up_at(&t, &all, STATION);
    let others: Vec<u32> = (all.iter().copied()).filter(|n| !first.contains(n)).collect();
    let emptied = run_to(&mut t, &all, &[STATION], PICK_UP + 10, "the station empties", |t| {
        for &n in &others {
            assert!(!is_picking_up(t, n), "tick {}: Agent {n} picks up", t.sim().tick());
        }
        supplies(t, STATION) == 0
    });
    assert_eq!(empties(&t), [(emptied, STATION, P1)]);
    for &n in &first {
        assert_eq!(load(&t, n), (CARRY, Resource::Supplies), "Agent {n}");
    }
    for &n in &others {
        assert!(
            is_waiting(&t, n) && gap(&t, n, STATION) <= REACH,
            "Agent {n} waits beside it"
        );
        assert_eq!(load(&t, n).0, 0);
    }
    // Until the delivery, nobody's Gather ends and nobody picks up; the two deliver and come back to wait.
    let refill = run_to(&mut t, &all, &[STATION], INTERVAL, "the delivery", |t| {
        for &n in &all {
            assert_eq!(order(t, n), Some(OrderKind::Gather(id(STATION))), "Agent {n}");
            assert!(!is_picking_up(t, n), "Agent {n}");
        }
        !refills(t).is_empty()
    });
    assert_eq!(refill - emptied, INTERVAL - 1);
    assert_eq!(supply_deliveries(&t).len(), 2);
    for &n in &all {
        assert!(
            is_waiting(&t, n) && gap(&t, n, STATION) <= REACH,
            "Agent {n} waits beside it"
        );
    }
    step(&mut t, &all, &[STATION]);
    assert_eq!(picking_up_at(&t, &all, STATION), all, "all four resume");
    let again = run_to(&mut t, &all, &[STATION], PICK_UP, "empty again", |t| {
        supplies(t, STATION) == 0
    });
    assert_eq!(empties(&t), [(emptied, STATION, P1), (again, STATION, P1)]);
}

/// Point 3: "the owner gets a notification", read as each player whose Agents gather at the station when its last
/// Supply goes, in player order (Placeholder: the notifications are not designed; 0256's "Implementation impact").
/// Player 2's Agent at another station, and player 2's own Agent sent to an empty one, get no notification: the
/// station they wait at was empty already.
#[test]
fn every_player_gathering_at_a_station_is_notified_when_it_empties() {
    let mut t = flat(30, 20)
        .faction(P2, Faction::Syndicate)
        .spawn(TUNNEL, Some(P1), tile(20, 6)) // 1
        .spawn_with(station_of(2), tile(5, 7)) // 2
        .spawn(TUNNEL, Some(P2), tile(20, 12)) // 3
        .spawn(AGENT, Some(P1), centre(5, 5)) // 4: north of station 2
        .spawn(AGENT, Some(P2), centre(6, 10)) // 5: south of it
        .spawn_with(station_of(0), tile(10, 15)) // 6: empty
        .spawn(AGENT, Some(P2), centre(13, 16)) // 7: east of station 6
        .start();
    let (all, stations) = ([4, 5, 7], [STATION, 6]);
    gather(&mut t, 1, &[4], STATION);
    t.order_at(1, P2, &[id(5)], OrderKind::Gather(id(STATION)), false);
    t.order_at(1, P2, &[id(7)], OrderKind::Gather(id(6)), false);
    let emptied = run_to(&mut t, &all, &stations, 100, "station 2 empties", |t| {
        !empties(t).is_empty()
    });
    assert_eq!(empties(&t), [(emptied, STATION, P1), (emptied, STATION, P2)]);
    assert_eq!(load(&t, 4), (CARRY, Resource::Supplies));
    assert_eq!(load(&t, 5), (CARRY, Resource::Supplies));
    t.run(400);
    assert_eq!(empties(&t).len(), 2, "nothing more");
    assert!(
        is_waiting(&t, 7) && gap(&t, 7, 6) <= REACH,
        "Agent 7 waits at the empty station"
    );
    assert_eq!(order(&t, 7), Some(OrderKind::Gather(id(6))));
}
