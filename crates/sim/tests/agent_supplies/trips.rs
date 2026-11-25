//! The trips at a station: the slice's acceptance ("One Agent at a station holding 4: four trips empty it, the 960-tick
//! countdown runs, and 4 Supplies arrive again. The bank gains 4"), and two Agents at one station at once.

use super::*;

/// The slice's acceptance, with the design's station (`core/map.md > SupplyDeliveryStation`: DeliverySize 4,
/// DeliveryInterval 960, "The countdown begins only once the station is at 0"), 3 grid units from the Supplies hatch.
/// Four trips of 1 Supply each (`factions/syndicate/units/agent.md > Gathering > Supplies`: "CarryCapacity: 1 Supply
/// per trip") empty it: the bank's Supplies gain 4 and its Space Crystals nothing. The last pick-up notifies player 1
/// (design change 0256 point 3), and the Agent comes back and waits beside the station, keeping its Gather. The
/// countdown begins in the tick the station empties, its first frame (the recipes' convention, `ARCHITECTURE.md >
/// Economy > Recipe timing`); 4 Supplies arrive 960 frames on, and the Agent picks up again in the next tick.
#[test]
fn one_agent_empties_a_station_of_four_and_four_arrive_again() {
    let mut t = field(1, 3, station_of(4)).start();
    let all = [3];
    gather(&mut t, 1, &all, STATION);
    let before = banks(&t, P1);
    let emptied = run_to(&mut t, &all, &[STATION], 1500, "the station empties", |t| {
        !empties(t).is_empty()
    });
    assert_eq!(empties(&t), [(emptied, STATION, P1)]);
    assert_eq!(supply_deliveries(&t).len(), 3, "the fourth Supply is on its way");
    assert_eq!(load(&t, 3), (CARRY, Resource::Supplies));
    assert_eq!(supplies(&t, STATION), 0);
    assert_eq!(countdown(&t, STATION), Some(DELIVERY_INTERVAL - 1), "its first frame");
    run_to(&mut t, &all, &[STATION], 300, "the fourth delivery", |t| {
        supply_deliveries(t).len() == 4
    });
    let delivered = supply_deliveries(&t);
    assert!(
        delivered.iter().all(|d| (d.1, d.2, d.3) == (3, T1, CARRY)),
        "{delivered:?}"
    );
    assert_eq!(banks(&t, P1), (before.0, before.1 + 4), "the bank gains 4 Supplies");
    // 0256 point 3: "stands next to it indefinitely, keeping its job, waiting for the next supply delivery".
    run_to(&mut t, &all, &[STATION], 300, "back at the station", |t| {
        is_waiting(t, 3)
    });
    let refill = run_to(&mut t, &all, &[STATION], DELIVERY_INTERVAL, "the delivery", |t| {
        assert_eq!(order(t, 3), Some(OrderKind::Gather(id(STATION))));
        assert!(gap(t, 3, STATION) <= REACH, "beside the station");
        let refilled = !refills(t).is_empty();
        assert!(
            refilled || is_waiting(t, 3),
            "tick {}: {:?}",
            t.sim().tick(),
            phase(t, 3)
        );
        refilled
    });
    assert_eq!(refills(&t), [(refill, STATION, DELIVERY_SIZE)]);
    assert_eq!(
        refill - emptied,
        DELIVERY_INTERVAL - 1,
        "960 frames, the tick it emptied the first"
    );
    assert_eq!((supplies(&t, STATION), countdown(&t, STATION)), (4, None));
    step(&mut t, &all, &[STATION]);
    assert_eq!(picking_up_at(&t, &all, STATION), [3], "it resumes");
}

/// Design change 0256 point 1 ("any number pick up at once, as at patches"): two Agents at one station that holds
/// plenty, 3 grid units from the hatch. They pick up at once, each delivers its Supplies at the Supplies hatch, one at
/// a time there (`factions/syndicate/tunnels.md > Tunnel > SymmetryType - ABCD`: "Only one Agent may drop off resources
/// at a side at a time"), and no Supply is lost or made: what the station gave is what the Agents carry and the bank
/// holds.
#[test]
fn two_agents_pick_up_at_one_station_at_once() {
    let mut t = field(2, 3, station_of(100)).start();
    let all = agents(2);
    gather(&mut t, 1, &all, STATION);
    // The Syndicate's starting Supplies (`factions/syndicate/tunnels.md > Tunnel > Starting Conditions`; slice M4-B).
    let banked = banks(&t, P1).1;
    let mut together = false;
    for _ in 0..1200 {
        step(&mut t, &all, &[STATION]);
        together |= picking_up_at(&t, &all, STATION).len() == 2;
        let unloading: Vec<u32> = (all.iter().copied()).filter(|&n| is_unloading(&t, n)).collect();
        assert!(
            unloading.len() <= 1,
            "tick {}: {unloading:?} unload at once",
            t.sim().tick()
        );
        let carried: u32 = (all.iter())
            .map(|&n| load(&t, n))
            .filter(|l| l.1 == Resource::Supplies)
            .map(|l| l.0)
            .sum();
        let gained = banks(&t, P1).1 - banked;
        assert_eq!(supplies(&t, STATION) + carried + gained, 100, "tick {}", t.sim().tick());
    }
    assert!(together, "both picked up at once");
    let delivered = supply_deliveries(&t);
    for n in all {
        let trips = delivered.iter().filter(|d| d.1 == n && d.2 == T1).count();
        assert!(trips >= 4, "Agent {n} delivered {trips} Supplies: {delivered:?}");
    }
    for pair in delivered.windows(2) {
        assert!(
            pair[1].0 - pair[0].0 >= DROP_OFF,
            "one at a time at the hatch: {pair:?}"
        );
    }
}
