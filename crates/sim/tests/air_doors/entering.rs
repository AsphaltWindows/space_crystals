//! Into the air entry door (plan decision T96: "An air unit's Enter flies it to the entry door, and it is taken in once
//! it is over the door (D46's placeholder)"; design change 0431: "it **descends** into the entry door [...] over **16
//! frames (1 s)** [...], and it can be hit during both **as an air unit**"; `factions/syndicate/tunnel_menus.md >
//! EjectMenu`: "(the same rate for the air entry door)").

use sc_sim::network::Place;

use super::*;

/// Player 1's Phantom, debug-spawned on the map at `at` in tick 2, which has run: its id.
fn phantom_at(t: &mut TestSim, at: Vec2) -> ObjectId {
    let phantom = id(next_id(t));
    t.debug_at(2, spawn(PHANTOM, P1, SpawnAt::Point(at)));
    run_checked(t, 2);
    assert_eq!(object(t, phantom).kind, PHANTOM);
    phantom
}

/// The tick in which `unit`'s descent began, if it has: the first in which it was a whole share up out of the door.
fn descending(t: &TestSim, unit: ObjectId) -> Option<u32> {
    let active = object(t, unit).orders.as_ref()?.current.as_ref()?;
    match &active.state {
        sc_sim::orders::Behaviour::Enter(enter) => enter.descent.map(|descent| descent.ticks_done()),
        _ => None,
    }
}

/// The plan's acceptance, "A right-click on an own Tier 3 Tunnel takes a Phantom to the entry door, and it is held":
/// player 1's Phantom, 6 grid units north-west of the Tunnel, right-clicks it (its default command there is Enter) and
/// flies to the air entry door, never by Side A's paths. Once its centre is over the door its descent begins, and it
/// stays on the map for the descent's 16 ticks, its lift going down a sixteenth a tick; player 2's Guard west of the
/// door, ordered to attack it as its descent begins, hits it meanwhile, an air target. In the 16th tick it goes into
/// the network, walked in: it can't come out for the transit delay (`Reject::InTransit`), then it can.
#[test]
fn a_right_click_on_a_tier_3_tunnel_takes_a_phantom_down_the_air_entry_door_into_the_network() {
    let mut t = hangar();
    let phantom = phantom_at(&mut t, Vec2::new(500.0, 400.0));
    let guard = id(next_id(&t));
    t.debug_at(3, spawn(GUARD, P2, SpawnAt::Point(AIR_ENTRY - Vec2::new(96.0, 0.0))));
    let right_click = t.sim().default_order(phantom, Target::Object(T1));
    assert_eq!(right_click, Some(OrderKind::Enter(T1)));
    t.at(3, P1, order(&[phantom], OrderKind::Enter(T1)));
    step_until(&mut t, 400, "the Phantom's descent begins", |t| {
        descending(t, phantom).is_some()
    });
    let began = t.sim().tick();
    let pos = object(&t, phantom).pos;
    assert!(network::over_door(AIR_ENTRY, pos), "over the air entry door: {pos}");
    assert_eq!((descending(&t, phantom), lift(&t, phantom)), (Some(0), Some(1.0)));
    t.at(began + 1, P2, order(&[guard], OrderKind::Attack(phantom)));
    for k in 1..DOOR_FRAMES {
        step_checked(&mut t);
        let share = 1.0 - k as f32 / DOOR_FRAMES as f32;
        assert_eq!(
            (descending(&t, phantom), lift(&t, phantom)),
            (Some(k), Some(share)),
            "tick {}",
            t.sim().tick()
        );
        assert!(
            network::over_door(AIR_ENTRY, object(&t, phantom).pos),
            "tick {}",
            t.sim().tick()
        );
    }
    let hits: Vec<_> = hits_on(&t, phantom);
    assert!(
        !hits.is_empty() && hits.iter().all(|&(tick, by, _)| by == guard && tick > began),
        "{hits:?}"
    );
    step_checked(&mut t);
    let went_in = began + DOOR_FRAMES;
    assert_eq!(entries(&t, T1), [(went_in, phantom)]);
    assert!(t.sim().object(phantom).is_none());
    let held = t.sim().held_unit(phantom).expect("in the network");
    assert_eq!((held.place, held.since, held.entered), (Place::Network, went_in, true));
    assert_eq!(inside(&t), [phantom]);
    assert_eq!(gate(&t, P1, eject(T1, PHANTOM)), Err(Reject::InTransit(PHANTOM)));
    run_for(&mut t, network::rules().transit_delay);
    assert_eq!(gate(&t, P1, eject(T1, PHANTOM)), Ok(()));
}

/// "(the same rate for the air entry door)": two Phantoms ordered in together. Each descends in its turn once over the
/// door, the second's descent beginning 8 ticks after the first's at the soonest, and both go in, one after the other.
/// The air units' soft separation keeps the second off the door while the first descends, so it waits, flying back
/// to the door (Placeholder (not in the design: an aircraft waiting its turn at the entry door)).
#[test]
fn the_air_entry_door_lets_one_phantom_down_every_8_ticks_at_most() {
    let mut t = hangar();
    let first = phantom_at(&mut t, Vec2::new(500.0, 400.0));
    let second = id(first.0 + 1);
    t.debug_at(3, spawn(PHANTOM, P1, SpawnAt::Point(Vec2::new(500.0, 1100.0))));
    t.at(4, P1, order(&[first, second], OrderKind::Enter(T1)));
    let mut began: Vec<(ObjectId, Tick)> = Vec::new();
    step_until(&mut t, 800, "both go in", |t| entries(t, T1).len() == 2);
    // The descents, read back from the entries: each began 16 ticks before its unit went in.
    for (tick, unit) in entries(&t, T1) {
        began.push((unit, tick - DOOR_FRAMES));
    }
    assert_eq!(began.len(), 2);
    assert!(began[1].1 >= began[0].1 + DOOR_RATE, "{began:?}");
    assert_eq!(inside(&t).len(), 2);
}

/// Placeholder (not in the design: an order during the descent): a new order ends the Enter, its descent with it. A
/// Phantom descending is ordered to move away: it is no longer descending, stays on the map and flies off, and never
/// goes in.
#[test]
fn an_order_during_the_descent_ends_it() {
    let mut t = hangar();
    let phantom = phantom_at(&mut t, Vec2::new(700.0, 500.0));
    t.at(3, P1, order(&[phantom], OrderKind::Enter(T1)));
    step_until(&mut t, 400, "the descent begins", |t| descending(t, phantom).is_some());
    run_for(&mut t, 4);
    assert_eq!(descending(&t, phantom), Some(4));
    let away = Vec2::new(500.0, 300.0);
    t.at(
        t.sim().tick() + 1,
        P1,
        order(&[phantom], OrderKind::Move(Target::Point(away))),
    );
    run_for(&mut t, 1);
    assert_eq!((descending(&t, phantom), lift(&t, phantom)), (None, None));
    run_for(&mut t, 120);
    assert!(object(&t, phantom).pos.distance(away) < 1.0);
    assert!(entries(&t, T1).is_empty() && inside(&t).is_empty());
}

/// `factions/syndicate/tunnels.md > Tunnel Network > Walking in > Target Tunnel gone or unsuitable` ("if the Tunnel a
/// unit is heading for is destroyed [...] the Enter order ends and the unit idles"): a Phantom descending when its
/// Tunnel is destroyed (player 1 lives on by a second Tunnel, far east) stays on the map, idle, at the height of any
/// air unit, and never goes in.
#[test]
fn a_phantom_descending_into_a_lost_tunnel_idles_on_the_map() {
    let mut t = started(hangar_scenario().spawn(TUNNEL, Some(P1), tile(30, 10)), 3);
    let phantom = phantom_at(&mut t, Vec2::new(700.0, 500.0));
    t.at(3, P1, order(&[phantom], OrderKind::Enter(T1)));
    step_until(&mut t, 400, "the descent begins", |t| descending(t, phantom).is_some());
    let fell = t.sim().tick() + 1;
    t.debug_at(fell, DebugCommand::Remove { id: T1 });
    run_checked(&mut t, fell + DOOR_FRAMES);
    let o = object(&t, phantom);
    assert_eq!(o.owner, Some(P1), "player 1 lives on");
    assert!(o.orders.as_ref().unwrap().is_idle(), "{:?}", o.orders);
    assert_eq!(lift(&t, phantom), None);
    assert!(inside(&t).is_empty() && entries(&t, T1).is_empty());
}
