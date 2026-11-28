//! Boarding (`factions/syndicate/units/subterranean_transport.md > Placeholder stats`: "**Loading**: right-click the
//! transport with infantry selected (Enter); it must be above ground", "**Capacity 4** infantry"; `sc_sim::carry`):
//! its owner's infantry walk up to it and board within Proximity, from any side, while it has room, the others idling
//! beside it as at a full building; no unit of another player's, no vehicle, and none while it is burrowed.

use super::*;

const BIKE: ObjectKind = ObjectKind::AssaultBike;

/// The plan's acceptance: "four Guards board; a fifth is refused". Five of player 1's Guards sent onto its Transport
/// together walk up to it: the first four to get there board it, one at a time, each held aboard with a
/// `SimEvent::Boarded` (`network::Place::Carrier`), and the fifth, getting there with no room left, loses its Enter
/// and idles beside it, on the map (`SimEvent::EnterFailed` with `full`). The Transport's room is then none, and the
/// Tunnel Space in use is the same throughout: the Transport's 3 and the Guards' 1 each, aboard or not.
#[test]
fn four_guards_board_and_a_fifth_stays_out() {
    let mut t = transport_and_guards(5);
    let transport = id(1);
    let guards: Vec<ObjectId> = (2..7).map(id).collect();
    let space = TRANSPORT_SPACE + 5 * GUARD_SPACE;
    assert_eq!(space_used(&t, P1), space);
    t.at(2, P1, order(&guards, OrderKind::Enter(transport)));
    let failed = |t: &TestSim| {
        (t.events().iter()).find_map(|(tick, e)| match *e {
            SimEvent::EnterFailed { unit, target, full } if target == transport => Some((*tick, unit, full)),
            _ => None,
        })
    };
    step_until(&mut t, 300, "four board and one is turned away", |t| {
        assert_eq!(space_used(t, P1), space, "tick {}", t.sim().tick());
        aboard(t, transport).len() == CAPACITY && failed(t).is_some()
    });
    let (refused_in, fifth, full) = failed(&t).unwrap();
    assert!(full, "turned away for want of room");
    let on: Vec<ObjectId> = boarded(&t, transport).into_iter().map(|(_, unit)| unit).collect();
    assert_eq!(on.len(), CAPACITY);
    assert_eq!(aboard(&t, transport), on, "aboard in the order they boarded");
    assert!(!on.contains(&fifth));
    assert!(boarded(&t, transport).iter().all(|&(tick, _)| tick <= refused_in));
    assert_eq!(t.sim().room(transport), 0);
    for &guard in &on {
        assert!(t.sim().object(guard).is_none(), "{guard:?} is off the map");
        assert_eq!(
            t.sim().held_unit(guard).map(|held| held.place),
            Some(sc_sim::network::Place::Carrier(transport))
        );
    }
    t.run(20);
    assert!(is_idle(&t, fifth), "the fifth idles");
    let gap = object(&t, fifth).pos.distance(object(&t, transport).pos);
    assert!(gap < 120.0, "beside it: {gap} su from its centre");
    assert_eq!(aboard(&t, transport).len(), CAPACITY);
    assert_eq!(space_used(&t, P1), space);
}

/// "Right-click the transport": a Guard sent onto a Transport that drives away walks after it as it goes and boards it
/// on its way (the Guard's MaxSpeed is 5 su a frame, the Transport's 4.5), from behind: the Transport, sent 400 su east
/// as the Guard is sent onto it, has driven a good way east by then, and gets where it was sent with the Guard aboard.
#[test]
fn a_guard_walks_after_a_transport_driving_away_and_boards_it_on_its_way() {
    let mut t = transport_and_guards(1);
    let (transport, guard) = (id(1), id(2));
    let goal = Vec2::new(1000.0, 600.0);
    t.at(2, P1, order(&[transport], move_to(goal.x, goal.y)));
    board(&mut t, &[guard], transport);
    let there = object(&t, transport).pos;
    assert!(there.x > 700.0, "it had gone east: {there}");
    step_until(&mut t, 200, "the Transport gets there", |t| {
        object(t, transport).pos.distance(goal) < 1.0
    });
    assert_eq!(aboard(&t, transport), [guard]);
}

/// Who boards (`network::admits`'s carrier arm, `carry::admits`; the plan's D45): of a Guard, a Bike and the Transport
/// itself sent onto it together, the Guard takes the Enter and boards, while the Bike, a vehicle ("carries **infantry
/// only**"), and the Transport ignore it, the Bike keeping its move. Alone, the Bike is refused
/// ([`Reject::NotAccepted`]); player 2's Peacekeeper may not board player 1's Transport ([`Reject::NotOwned`]); and a
/// burrowed Transport takes no one ([`Reject::NotATunnel`]: "it must be above ground").
#[test]
fn only_its_owner_s_infantry_board_it_and_only_above_ground() {
    let scenario = field()
        .spawn(TRANSPORT, Some(P1), point(600.0, 600.0))
        .spawn(GUARD, Some(P1), point(400.0, 600.0))
        .spawn(BIKE, Some(P1), point(600.0, 800.0))
        .spawn(ObjectKind::Peacekeeper, Some(P2), point(800.0, 600.0))
        .spawn(BURROWED, Some(P1), point(1200.0, 600.0));
    let mut t = started(scenario);
    let (transport, guard, bike, pk, burrowed) = (id(1), id(2), id(3), id(4), id(5));
    assert_eq!(
        gate(&t, P1, order(&[bike], OrderKind::Enter(transport))),
        Err(Reject::NotAccepted(transport))
    );
    assert_eq!(
        gate(&t, P2, order(&[pk], OrderKind::Enter(transport))),
        Err(Reject::NotOwned(transport))
    );
    assert_eq!(
        gate(&t, P1, order(&[guard], OrderKind::Enter(burrowed))),
        Err(Reject::NotATunnel(burrowed))
    );
    t.at(2, P1, order(&[bike], move_to(2000.0, 800.0)));
    t.at(3, P1, order(&[guard, bike, transport], OrderKind::Enter(transport)));
    t.run_until(3);
    let kind = |t: &TestSim, id| (object(t, id).orders.as_ref()?.current.as_ref()).map(|active| active.order.kind);
    assert_eq!(kind(&t, bike), Some(move_to(2000.0, 800.0)), "the Bike keeps its move");
    assert_eq!(kind(&t, transport), None, "the Transport takes no Enter of its own");
    board(&mut t, &[guard], transport);
    assert_eq!(aboard(&t, transport), [guard]);
    assert!(t.sim().object(transport).is_some());
}
