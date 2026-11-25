//! The Enter order and the units inside the network (`systems/controls/commands.md > Enter`;
//! `factions/syndicate/tunnels.md > Tunnel Network`, design changes 0245 to 0249).

use sc_sim::defs::TRANSIT_DELAY_TICKS;
use sc_sim::network::Place;
use sc_sim::{ArmyResource, MoveStatus, PlayerCommand, Reject, Target};

use super::*;

fn validate(t: &TestSim, player: PlayerId, command: Command) -> Result<(), Reject> {
    t.sim().validate(&PlayerCommand { player, command })
}

/// The plan's acceptance for Enter, and `factions/syndicate/tunnels.md > Walking in`: "A unit leaves the map once it
/// has walked into the Tunnel." A Guard ordered to Enter walks to the entry path, the half of Side A nearer Side D
/// (design change 0248), and leaves the map into its owner's network, with its id.
#[test]
fn a_guard_walks_into_the_entry_path_and_leaves_the_map() {
    let mut t = base().start();
    t.without_auto_targeting()
        .spawn_unit_at(1, GUARD, P1, Vec2::new(700.0, 400.0)) // 2
        .at(2, P1, enter(&[id(2)], T1));
    run_checked(&mut t, 2);
    let mut last = t.pos(2);
    while t.sim().object(id(2)).is_some() {
        assert!(t.sim().tick() < 120, "it gets there");
        last = t.pos(2);
        step_checked(&mut t);
    }
    let tick = t.sim().tick();
    assert_eq!(entries(&t), [(tick, id(2))]);
    // It went in from the entry path: within its reach of the spot where a Guard stands against the wall, on the
    // half towards Side D (west of the side's middle, x = 896), and nowhere near the exit path.
    assert!(last.distance(ENTRY) < 30.0, "{last}");
    assert!(last.x < 896.0 && last.distance(EXIT) > 36.0, "{last}");
    assert!(paths(&t, T1).reached_entry(last, Vec2::splat(18.0)));
    // It is the same unit, off the map: in the network, idle, with its HP.
    let held = t.sim().held_unit(id(2)).expect("in the network");
    assert_eq!((held.place, held.since, held.entered), (Place::Network, tick, true));
    assert_eq!(
        (held.object.kind, held.object.owner, held.object.hp),
        (GUARD, Some(P1), Some(80.0))
    );
    assert_eq!(inside(&t), [id(2)]);
    assert!(t.sim().network(P2).next().is_none());
    // No Removed, no death: it left the map, not the game.
    let left_game = |e: &SimEvent| matches!(e, SimEvent::Removed(_) | SimEvent::Died { .. });
    assert!(!t.events().iter().any(|(_, e)| left_game(e)));
}

/// The plan's acceptance ("An enemy unit that was attacking it stops, as for a death") and plan decision T30
/// ("orders, attacks and follows on it end"): when a unit goes in, an enemy's Attack on it and a friend's Follow end.
#[test]
fn entering_ends_the_attacks_and_follows_on_the_unit() {
    let mut t = base().start();
    t.without_auto_targeting()
        .spawn_unit_at(1, GUARD, P1, Vec2::new(866.0, 500.0)) // 2
        .spawn_unit_at(1, GUARD, P2, Vec2::new(1100.0, 400.0)) // 3: the enemy
        .spawn_unit_at(1, AGENT, P1, Vec2::new(700.0, 500.0)) // 4: a friend
        .at(2, P1, enter(&[id(2)], T1))
        .order_at(2, P2, &[id(3)], OrderKind::Attack(id(2)), false)
        .order_at(2, P1, &[id(4)], OrderKind::Move(Target::Object(id(2))), false);
    run_checked(&mut t, 4);
    let attacker = |t: &TestSim| t.sim().object(id(3)).unwrap().clone();
    assert_eq!(attacker(&t).attack.unwrap().target, Some(id(2)), "it engages the Guard");
    while t.sim().object(id(2)).is_some() {
        assert!(t.sim().tick() < 100);
        step_checked(&mut t);
    }
    let enemy = attacker(&t);
    assert!(enemy.orders.as_ref().unwrap().is_idle(), "the Attack ended");
    assert_eq!(enemy.attack.unwrap().target, None);
    let friend = t.sim().object(id(4)).unwrap();
    assert!(friend.orders.as_ref().unwrap().is_idle(), "the Follow ended");
    // Nothing shoots at, or walks to, a unit that isn't there.
    let tick = t.sim().tick();
    run_checked(&mut t, tick + 60);
    let hits_after = (t.events().iter()).any(|(at, e)| *at > tick && matches!(e, SimEvent::Attacked { .. }));
    assert!(!hits_after);
    assert_eq!(t.status(4), MoveStatus::TargetLost);
}

/// Design change 0249 ("entering clears the unit's orders, queued ones included") and the EjectMenu's "With no rally
/// point, a unit walks about 1 grid unit straight out of Side A and idles": a Guard with a Move queued behind its
/// Enter comes out without it.
#[test]
fn entering_clears_the_orders_queued_ones_included() {
    let mut t = base().start();
    t.without_auto_targeting()
        .spawn_unit_at(1, GUARD, P1, Vec2::new(866.0, 560.0)) // 2
        .at(2, P1, enter(&[id(2)], T1))
        .order_at(
            3,
            P1,
            &[id(2)],
            OrderKind::Move(Target::Point(Vec2::new(200.0, 200.0))),
            true,
        );
    run_checked(&mut t, 3);
    assert_eq!(t.sim().object(id(2)).unwrap().orders.as_ref().unwrap().queue.len(), 1);
    run_checked(&mut t, 30);
    let entered = entries(&t)[0].0;
    let held = t.sim().held_unit(id(2)).expect("inside");
    assert!(held.object.orders.as_ref().unwrap().is_idle());
    t.at(entered + TRANSIT_DELAY_TICKS, P1, eject(T1, GUARD));
    run_checked(&mut t, entered + TRANSIT_DELAY_TICKS + 40);
    // It walked 1 grid unit straight out of Side A from the exit spot, and idles there.
    assert_eq!(t.pos(2), EXIT + Vec2::new(0.0, -64.0));
    assert!(t.sim().object(id(2)).unwrap().orders.as_ref().unwrap().is_idle());
}

/// The gate (`systems/controls/commands.md > Enter`: "any Syndicate unit, when the target Tunnel's tier is sufficient
/// for the unit's base category"), the plan's tiers ("A test unit with a vehicle base (a Drone owned by the Syndicate
/// test player) is refused Enter [...] at a Tier 1 Tunnel, and allowed at Tier 2"), and the default command, which is
/// Enter exactly where the gate takes it.
#[test]
fn the_gate_takes_an_enter_by_the_tunnels_owner_and_tier() {
    let mut t = base().spawn(HIDEOUT, Some(P1), tile(16, 11)).start(); // 2
    t.without_auto_targeting()
        .spawn_unit_at(1, GUARD, P1, Vec2::new(700.0, 400.0)) // 3
        .spawn_unit_at(1, ObjectKind::Drone, P1, Vec2::new(600.0, 400.0)) // 4: a vehicle
        .spawn_unit_at(1, AGENT, P1, Vec2::new(500.0, 400.0)) // 5
        .spawn_unit_at(1, GUARD, P2, Vec2::new(640.0, 600.0)) // 6, in sight of the Tunnel
        .debug_at(1, spawn(TUNNEL, P2, tile(2, 14))); // 7
    run_checked(&mut t, 2);
    let (guard, drone, agent, tunnel, hideout, theirs) = (id(3), id(4), id(5), T1, id(2), id(7));
    assert_eq!(validate(&t, P1, enter(&[guard], tunnel)), Ok(()));
    assert_eq!(validate(&t, P1, enter(&[agent], tunnel)), Ok(()));
    // A vehicle at Tier 1: refused alone, ignored in a group.
    let no_transit = Reject::NoTransit { tunnel, kind: None };
    assert_eq!(validate(&t, P1, enter(&[drone], tunnel)), Err(no_transit));
    assert_eq!(validate(&t, P1, enter(&[drone, guard], tunnel)), Ok(()));
    // Only an own Tunnel.
    assert_eq!(validate(&t, P1, enter(&[guard], theirs)), Err(Reject::NotOwned(theirs)));
    assert_eq!(
        validate(&t, P1, enter(&[guard], hideout)),
        Err(Reject::NotATunnel(hideout))
    );
    assert_eq!(validate(&t, P1, enter(&[guard], guard)), Err(Reject::NotATunnel(guard)));
    assert_eq!(
        validate(&t, P1, enter(&[guard], id(99))),
        Err(Reject::UnknownObject(id(99)))
    );
    assert_eq!(validate(&t, P2, enter(&[id(6)], tunnel)), Err(Reject::NotOwned(tunnel)));
    // The default command (`factions/syndicate/units/agent.md > Right-Click Resolution`: "Own Tunnel (not carrying
    // resources) → Enter").
    let default = |t: &TestSim, unit, target| t.sim().default_order(unit, Target::Object(target));
    assert_eq!(default(&t, guard, tunnel), Some(OrderKind::Enter(tunnel)));
    assert_eq!(default(&t, agent, tunnel), Some(OrderKind::Enter(tunnel)));
    assert_eq!(
        default(&t, drone, tunnel),
        Some(OrderKind::Move(Target::Object(tunnel)))
    );
    assert_eq!(
        default(&t, guard, hideout),
        Some(OrderKind::Move(Target::Object(hideout)))
    );
    assert_eq!(
        default(&t, id(6), tunnel),
        Some(OrderKind::Attack(tunnel)),
        "an enemy's Tunnel"
    );
    // The group's order reaches the Guard alone.
    t.at(3, P1, enter(&[drone, guard], tunnel));
    run_checked(&mut t, 3);
    let order_of = |t: &TestSim, unit| {
        let orders = t.sim().object(unit).unwrap().orders.clone().unwrap();
        orders.current.map(|active| active.order.kind)
    };
    assert_eq!(order_of(&t, guard), Some(OrderKind::Enter(tunnel)));
    assert_eq!(order_of(&t, drone), None);
    // At Tier 2 the vehicle is let in.
    t.debug_at(4, DebugCommand::SetTier { id: tunnel, tier: 2 })
        .at(5, P1, enter(&[drone], tunnel));
    run_checked(&mut t, 4);
    assert_eq!(default(&t, drone, tunnel), Some(OrderKind::Enter(tunnel)));
    run_checked(&mut t, 200);
    assert_eq!(inside(&t), [guard, drone]);
}

/// `factions/syndicate/tunnels.md > Walking in > Target Tunnel gone or unsuitable`: "if the Tunnel a unit is heading
/// for is destroyed, or its tier doesn't suit the unit, the Enter order ends and the unit idles".
#[test]
fn a_tunnel_gone_or_unsuitable_ends_the_order() {
    let mut t = base().spawn(TUNNEL, Some(P1), tile(22, 2)).start(); // 2, to keep player 1 in the game
    t.without_auto_targeting()
        .debug_at(1, DebugCommand::SetTier { id: T1, tier: 2 })
        .spawn_unit_at(1, GUARD, P1, Vec2::new(300.0, 300.0)) // 3
        .spawn_unit_at(1, ObjectKind::Drone, P1, Vec2::new(300.0, 500.0)) // 4
        .at(2, P1, enter(&[id(3), id(4)], T1))
        // The tier falls under the vehicle on its way.
        .debug_at(20, DebugCommand::SetTier { id: T1, tier: 1 });
    run_checked(&mut t, 19);
    let idle = |t: &TestSim, unit: u32| t.sim().object(id(unit)).unwrap().orders.as_ref().unwrap().is_idle();
    assert!(!idle(&t, 3) && !idle(&t, 4));
    run_checked(&mut t, 20);
    assert!(
        idle(&t, 4) && !idle(&t, 3),
        "the Drone's order ended, the Guard's goes on"
    );
    assert_eq!(gave_up(&t), [id(4)]);
    // The Tunnel is destroyed under the Guard on its way.
    t.debug_at(30, DebugCommand::Remove { id: T1 });
    run_checked(&mut t, 31);
    assert!(idle(&t, 3));
    // Both idle where they stopped (the Drone once it has braked).
    run_checked(&mut t, 50);
    let stood = (t.pos(3), t.pos(4));
    run_checked(&mut t, 80);
    assert_eq!((t.pos(3), t.pos(4)), stood);
    assert!(stood.0.distance(ENTRY) > 200.0 && stood.1.distance(ENTRY) > 200.0);
    assert!(inside(&t).is_empty());
}

/// Design change 0247: "No special waiting rule [...] if the way is genuinely blocked, the normal Failure rule
/// applies: it gives up after about 7 s and idles." A Guard whose entry path is walled off by units on Hold gives
/// up, and stays on the map.
#[test]
fn a_blocked_entry_follows_the_failure_rule() {
    let mut t = base().start();
    t.without_auto_targeting();
    // A wall of player 2's Guards on Hold in front of Side A, flush with each other from x = 740 to x = 1064.
    let wall: Vec<ObjectId> = (0..9).map(|i| id(2 + i)).collect();
    for i in 0..9 {
        t.spawn_unit_at(1, GUARD, P2, Vec2::new(758.0 + 36.0 * i as f32, 621.0));
    }
    t.order_at(2, P2, &wall, OrderKind::Hold, false)
        .spawn_unit_at(1, GUARD, P1, Vec2::new(866.0, 300.0)) // 11
        .at(3, P1, enter(&[id(11)], T1));
    run_checked(&mut t, 3);
    while gave_up(&t).is_empty() {
        assert!(
            t.sim().tick() < 3 + 60 + 104 + 16,
            "it gives up within the Failure rule's time"
        );
        step_checked(&mut t);
    }
    assert_eq!(gave_up(&t), [id(11)]);
    let guard = t.sim().object(id(11)).expect("still on the map");
    assert!(guard.orders.as_ref().unwrap().is_idle());
    assert!(inside(&t).is_empty());
}

/// Design change 0245's Verification: "A Guard enters a Tunnel. For 2 s it can't be ejected anywhere; afterwards it
/// can be ejected from any Tunnel." The delay is flat: the far Tunnel lets it out as soon as the near one.
#[test]
fn an_entered_unit_can_exit_anywhere_after_the_transit_delay() {
    let mut t = base().spawn(TUNNEL, Some(P1), tile(22, 2)).start(); // 2: far away
    t.without_auto_targeting()
        .spawn_unit_at(1, GUARD, P1, Vec2::new(866.0, 560.0)) // 3
        .at(2, P1, enter(&[id(3)], T1));
    run_checked(&mut t, 30);
    let entered = entries(&t)[0].0;
    let far = id(2);
    // The command of tick `n` is checked against the state tick `n - 1` left, in tick `n`.
    while t.sim().tick() + 1 < entered + TRANSIT_DELAY_TICKS {
        for tunnel in [T1, far] {
            assert_eq!(validate(&t, P1, eject(tunnel, GUARD)), Err(Reject::InTransit(GUARD)));
            // Sent all the same, it does nothing.
            t.at(t.sim().tick() + 1, P1, eject(tunnel, GUARD));
        }
        step_checked(&mut t);
        assert_eq!(inside(&t), [id(3)]);
    }
    t.at(entered + TRANSIT_DELAY_TICKS, P1, eject(far, GUARD));
    run_checked(&mut t, entered + TRANSIT_DELAY_TICKS);
    assert_eq!(exits(&t, far), [(entered + TRANSIT_DELAY_TICKS, id(3))]);
    let at = paths(&t, far).exit_stand(Vec2::splat(18.0));
    assert_eq!(t.pos(3), at, "out of the far Tunnel, 32 frames after it went in");
}

/// Design change 0249's Verification: "A Guard at 50% HP enters a Tunnel. After 10 s inside, ejected, it shows about
/// 60%": 1% of MaxHP a second. A unit at full HP gains nothing.
#[test]
fn units_inside_regenerate_a_hundredth_of_their_max_hp_a_second() {
    let mut t = base().start();
    t.spawn_unit_at(1, GUARD, P1, Vec2::new(866.0, 560.0)) // 2
        .spawn_unit_at(1, GUARD, P1, Vec2::new(600.0, 300.0)) // 3: unhurt
        .spawn_unit_at(1, GUARD, P2, Vec2::new(866.0, 450.0)) // 4: shoots Guard 2, on Hold
        .order_at(2, P1, &[id(2)], OrderKind::Stop, false)
        .order_at(2, P2, &[id(4)], OrderKind::Attack(id(2)), false);
    t.debug_at(2, DebugCommand::SetAutoTargeting { on: false });
    run_checked(&mut t, 2);
    // Two shots of 18 less the armour take the Guard to about half.
    let hp = |t: &TestSim, unit: u32| t.sim().object(id(unit)).unwrap().hp.unwrap();
    while hp(&t, 2) > 50.0 {
        assert!(t.sim().tick() < 200);
        step_checked(&mut t);
    }
    let tick = t.sim().tick();
    t.debug_at(tick + 1, DebugCommand::Remove { id: id(4) })
        .at(tick + 2, P1, enter(&[id(2), id(3)], T1));
    run_checked(&mut t, tick + 2);
    let before = hp(&t, 2);
    assert!((30.0..=50.0).contains(&before), "{before}");
    while entries(&t).len() < 2 {
        assert!(t.sim().tick() < tick + 200);
        step_checked(&mut t);
    }
    let entered = entries(&t).iter().find(|(_, unit)| *unit == id(2)).unwrap().0;
    // 10 s inside: it went in during tick `entered`, whose regeneration it has; it comes out in tick `entered + 160`
    // after that tick's.
    t.at(entered + 160, P1, eject(T1, GUARD));
    run_checked(&mut t, entered + 160);
    assert_eq!(exits(&t, T1).len(), 1);
    let (out, gained) = (exits(&t, T1)[0].1, hp(&t, 2) - before);
    assert_eq!(out, id(2), "the one inside longest");
    assert!((gained - 8.05).abs() < 0.1, "about 10% of 80 in 10 s: {gained}");
    assert_eq!(t.sim().held_unit(id(3)).unwrap().object.hp, Some(80.0));
}

/// The plan's acceptance for Enter ("Tunnel Space is unchanged") and plan decision T30: a unit takes its army cost
/// "for as long as the unit lives", inside the network as on the map. The Syndicate's Tunnel Space is slice M4-B's;
/// the rule is the army cap's own, shown here on a GDO player's Unit Control, whose Peacekeeper walks into a Tunnel
/// that player owns (any unit of the Tunnel's owner may enter).
#[test]
fn a_unit_inside_the_network_still_takes_its_army_cost() {
    let mut t = ScenarioBuilder::new(&flat(30, 22))
        .spawn(TUNNEL, Some(P1), tile(12, 10))
        .start();
    t.without_auto_targeting()
        .spawn_unit_at(1, ObjectKind::Peacekeeper, P1, Vec2::new(866.0, 560.0)) // 2
        .spawn_unit_at(1, ObjectKind::Peacekeeper, P1, Vec2::new(600.0, 300.0)) // 3
        .at(2, P1, enter(&[id(2)], T1));
    run_checked(&mut t, 1);
    let used = |t: &TestSim| (t.sim().army_cap(P1).map(|cap| cap.used), t.sim().unit_control(P1));
    let before = used(&t);
    assert!(before.0.is_some_and(|cap| cap > 0) && before.1 > 0, "{before:?}");
    run_checked(&mut t, 40);
    assert_eq!(inside(&t), [id(2)]);
    assert_eq!(used(&t), before, "inside, it counts as before");
    // Out again, the same.
    let entered = entries(&t)[0].0;
    t.at(entered + TRANSIT_DELAY_TICKS, P1, eject(T1, ObjectKind::Peacekeeper));
    run_checked(&mut t, entered + TRANSIT_DELAY_TICKS);
    assert!(inside(&t).is_empty());
    assert_eq!(used(&t), before);
}

/// Slice M4-B: the Syndicate's own army cap, Tunnel Space, counts a unit inside the network as one on the map
/// (`factions/syndicate/tunnels.md > Tunnel Space`: the units "currently-committed", design change 0265; plan decision
/// T30: "for as long as the unit lives"), through `Sim::army_cap`'s held units. Player 1's Tier 1 Tunnel gives 12
/// (48 tunnel-region cells over 4); each Guard takes its TunnelSpaceCost of 1. The Guard that goes in keeps its 1
/// inside and out again, and gives it back when it dies.
#[test]
fn a_guard_inside_the_network_keeps_its_tunnel_space_until_it_dies() {
    let mut t = base().start();
    t.without_auto_targeting()
        .spawn_unit_at(1, GUARD, P1, Vec2::new(866.0, 560.0)) // 2: goes in
        .spawn_unit_at(1, GUARD, P1, Vec2::new(600.0, 300.0)) // 3: stays out
        .at(2, P1, enter(&[id(2)], T1));
    let tunnel_space = |t: &TestSim| {
        let cap = t.sim().army_cap(P1).expect("a Syndicate player");
        assert_eq!(cap.resource, ArmyResource::TunnelSpace);
        (cap.used, cap.available)
    };
    run_checked(&mut t, 1);
    assert_eq!(tunnel_space(&t), (2, 12));
    run_checked(&mut t, 40);
    assert_eq!(inside(&t), [id(2)]);
    assert_eq!(tunnel_space(&t), (2, 12), "inside, it counts as before");
    let entered = entries(&t)[0].0;
    t.at(entered + TRANSIT_DELAY_TICKS, P1, eject(T1, GUARD));
    run_checked(&mut t, entered + TRANSIT_DELAY_TICKS);
    assert!(inside(&t).is_empty() && t.sim().object(id(2)).is_some());
    assert_eq!(tunnel_space(&t), (2, 12), "out again");
    let tick = t.sim().tick();
    t.debug_at(tick + 1, DebugCommand::Remove { id: id(2) });
    run_checked(&mut t, tick + 1);
    assert_eq!(tunnel_space(&t), (1, 12), "dead, it takes none");
}

/// Design change 0249: "An Agent keeps its load." An Agent carrying crystals gets Drop Off by default on its own
/// Tunnel (`units/agent.md > Right-Click Resolution`: "Own Tunnel (carrying resources) → Drop off resources"), and
/// Enter without a load; ordered to Enter with its load, it has it inside the network and when it comes out.
#[test]
fn an_agent_keeps_its_load_through_the_network() {
    // A patch (2) on tile (19, 11), east of the Tunnel's Side B.
    let mut t = base().spawn(ObjectKind::CrystalPatch, None, tile(19, 11)).start();
    t.without_auto_targeting()
        .spawn_unit_at(1, AGENT, P1, Vec2::new(1180.0, 620.0)) // 3
        .order_at(2, P1, &[id(3)], OrderKind::Gather(id(2)), false);
    run_checked(&mut t, 1);
    // The Syndicate's starting banks (`factions/syndicate/tunnels.md > Tunnel > Starting Conditions`; slice M4-B).
    let crystals = |t: &TestSim| t.sim().player(P1).unwrap().crystals;
    let banked = crystals(&t);
    let default = |t: &TestSim| t.sim().default_order(id(3), Target::Object(T1));
    assert_eq!(default(&t), Some(OrderKind::Enter(T1)), "empty");
    let load = |t: &TestSim| t.sim().object(id(3)).unwrap().worker.unwrap().load;
    while load(&t) == 0 {
        assert!(t.sim().tick() < 300, "it mines");
        step_checked(&mut t);
    }
    let carried = load(&t);
    assert_eq!(carried, 50);
    assert_eq!(default(&t), Some(OrderKind::DropOff(T1)), "carrying");
    let tick = t.sim().tick();
    t.at(tick + 1, P1, enter(&[id(3)], T1));
    while inside(&t).is_empty() {
        assert!(t.sim().tick() < tick + 200, "it walks in");
        step_checked(&mut t);
    }
    let held = t.sim().held_unit(id(3)).unwrap();
    assert_eq!(held.object.worker.unwrap().load, carried, "inside");
    assert_eq!(crystals(&t), banked, "nothing was delivered");
    let entered = t.sim().tick();
    t.at(entered + TRANSIT_DELAY_TICKS, P1, eject(T1, AGENT));
    run_checked(&mut t, entered + TRANSIT_DELAY_TICKS + 5);
    assert_eq!(load(&t), carried, "out again");
}
