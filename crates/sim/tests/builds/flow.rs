//! The order, the walk and the site (`factions/syndicate/units/agent.md > Building > Tunnel Construction Flow`, design
//! change 0234): the price paid at once, the site's HP, its completion with the Agent in the network, what the site
//! does and what is seen of it, and how it goes.

use super::*;
use sc_sim::SpawnSettings;

/// [`base`] with player 1's Agent 2, south-west of [`SITE`], player 2's Guard 3 east of it, out of the footprint but in
/// range, and player 1's idle Agent 4; auto-targeting off, and player 1 with 5 Supplies from tick 1.
fn field() -> TestSim {
    let mut t = base()
        .spawn(AGENT, Some(P1), point(12, 16)) // 2
        .spawn(GUARD, Some(P2), point(22, 10)) // 3
        .spawn(AGENT, Some(P1), point(9, 16)) // 4
        .start();
    t.without_auto_targeting().debug_at(1, supplies_of(P1, 5));
    step(&mut t);
    t
}

/// Agent 2 of [`field`] ordered to build at [`SITE`], until its site starts: the tick and the site.
fn with_site(t: &mut TestSim) -> (Tick, ObjectId) {
    next(t, P1, build(&[2], SITE, false));
    until_started(t, 2)
}

/// `player`'s Tunnel Space.
fn tunnel_space(t: &TestSim, player: PlayerId) -> u32 {
    t.sim().army_cap(player).expect("a Syndicate player").available
}

/// What worker `n` carries, on the map or held.
fn load(t: &TestSim, n: u32) -> u32 {
    let held = t.sim().held_unit(id(n)).map(|held| &held.object);
    let o = held.or_else(|| t.sim().object(id(n))).expect("in the game");
    o.worker.expect("a worker").load
}

/// `factions/syndicate/units/agent.md > Building > Tunnel Construction Flow` and design changes 0233 and 0234: the
/// order pays the price at once (1 Supply, for Tunnel 1) and leaves a ghost, its owner's only, which counts for the
/// price. On arrival the site appears with the Agent inside, at 60 HP, 10% of the Tunnel's 600 (`core/entities.md >
/// ConstructionHP Rule`), the ghost gone and nothing refunded. Its maximum grows a frame a tick while the damage it
/// took stays taken, and 480 ticks later (`factions/syndicate/tunnels.md > Cost Scaling > Construction Time`) it is a
/// Tunnel in operation at 600 less that damage: it gives Tunnel Space, and the Agent is in the network, free to leave
/// at once, and is ejected.
#[test]
fn the_order_pays_at_once_and_the_site_grows_into_a_tunnel() {
    let mut t = field();
    assert_eq!((supplies(&t, P1), price(&t, P1), tunnel_space(&t, P1)), (5, 1, 12));
    next(&mut t, P1, build(&[2], SITE, false));
    let ordered = OrderKind::Build {
        kind: TUNNEL,
        origin: SITE,
        orientation: BuildingOrientation::default(),
    };
    assert_eq!(current(&t, 2), Some(ordered));
    assert_eq!(supplies(&t, P1), 4);
    assert_eq!(ghosts(&t, P1), [(id(2), SITE)]);
    assert_eq!(ghosts(&t, P2), []);
    assert_eq!(price(&t, P1), 2);

    let (started, site) = until_started(&mut t, 2);
    let s = object(&t, site);
    assert_eq!((s.kind, s.owner), (TUNNEL, Some(P1)));
    assert_eq!(s.placement.map(|p| (p.origin, p.w, p.h)), Some((SITE, 4, 4)));
    assert!(s.is_site() && !s.is_operational());
    assert_eq!((s.hp, s.max_hp()), (Some(60.0), Some(60.0)));
    let held = t.sim().held_unit(id(2)).expect("the Agent is held");
    assert_eq!(held.place, Place::Site(site));
    assert_eq!(ghosts(&t, P1), []);
    assert_eq!((supplies(&t, P1), price(&t, P1), refunds(&t)), (4, 2, vec![]));

    // Player 2's Guard hits it once or twice, and stops.
    next(&mut t, P2, order(&[3], OrderKind::Attack(site), false));
    let hurt = |t: &TestSim| object(t, site).hp < object(t, site).max_hp();
    step_until(&mut t, 100, "the Guard hits the site", hurt);
    next(&mut t, P2, order(&[3], OrderKind::Stop, false));
    let damage = |t: &TestSim| {
        let s = object(t, site);
        s.max_hp().expect("a maximum") - s.hp.expect("HP")
    };
    let taken = damage(&t);
    assert!(taken > 0.0);
    while t.sim().tick() < started + 480 {
        assert!(object(&t, site).is_site(), "complete before tick {}", started + 480);
        assert_eq!(tunnel_space(&t, P1), 12, "a site gives no Tunnel Space");
        step(&mut t);
        assert!(
            (damage(&t) - taken).abs() < 0.05,
            "the damage stays taken: {} of {taken}",
            damage(&t)
        );
    }
    let s = object(&t, site);
    assert!(!s.is_site() && s.is_operational());
    assert_eq!(s.max_hp(), Some(600.0));
    assert!((s.hp.expect("HP") - (600.0 - taken)).abs() < 0.05);
    let completed = t.events().iter().find_map(|(tick, e)| match *e {
        SimEvent::SiteCompleted { site: done, worker } => Some((*tick, done, worker)),
        _ => None,
    });
    assert_eq!(completed, Some((started + 480, site, id(2))));
    assert_eq!(inside(&t, P1), [id(2)]);
    assert_eq!(tunnel_space(&t, P1), 24);
    assert_eq!((supplies(&t, P1), price(&t, P1)), (4, 2));

    assert_eq!(gate(&t, P1, eject(site)), Ok(()));
    next(&mut t, P1, eject(site));
    step_until(&mut t, 100, "the Agent comes out", |t| t.sim().object(id(2)).is_some());
    let out = t.events().iter().any(|(_, e)| {
        *e == SimEvent::LeftNetwork {
            unit: id(2),
            tunnel: site,
        }
    });
    assert!(out);
}

/// Design change 0234: a Tunnel under construction "does nothing but Cancel: no Tunnel Space, no drop-offs, no Enter,
/// no Expand, no Eject; the only command is X Cancel Construction" (`Object::is_operational`; Expand comes with slice
/// M4-F). The Agent inside isn't in the network, so no Eject brings it out of another Tunnel. Only its owner cancels
/// it. Once complete, the Tunnel takes the commands.
#[test]
fn the_site_does_nothing_but_cancel() {
    let mut t = field();
    let (started, site) = with_site(&mut t);
    let transform = Command::Construction {
        objects: vec![site],
        command: ConstructionCommand::CancelTransform,
    };
    assert_eq!(
        gate(&t, P1, order(&[4], OrderKind::Enter(site), false)),
        Err(Reject::NotATunnel(site))
    );
    assert_eq!(
        gate(&t, P1, order(&[4], OrderKind::DropOff(site), false)),
        Err(Reject::NotDropOff(site))
    );
    assert_eq!(gate(&t, P1, eject(site)), Err(Reject::NotATunnel(site)));
    assert_eq!(gate(&t, P1, eject(T1)), Err(Reject::NotInNetwork(AGENT)));
    assert_eq!(gate(&t, P1, transform), Err(Reject::NotTransforming));
    assert_eq!(gate(&t, P2, cancel(&[site])), Err(Reject::NotOwned(site)));
    assert_eq!(gate(&t, P1, cancel(&[site])), Ok(()));
    // Tunnel 1 takes them all along (empty, Agent 4 has nothing to deliver).
    assert_eq!(gate(&t, P1, order(&[4], OrderKind::Enter(T1), false)), Ok(()));
    let drop_off = order(&[4], OrderKind::DropOff(T1), false);
    assert_eq!(gate(&t, P1, drop_off), Err(Reject::NothingToDeliver));

    run_until(&mut t, started + 480);
    assert!(object(&t, site).is_operational());
    assert_eq!(gate(&t, P1, order(&[4], OrderKind::Enter(site), false)), Ok(()));
    let drop_off = order(&[4], OrderKind::DropOff(site), false);
    assert_eq!(gate(&t, P1, drop_off), Err(Reject::NothingToDeliver));
    assert_eq!(gate(&t, P1, eject(T1)), Ok(()));
    assert!(gate(&t, P1, cancel(&[site])).is_err(), "nothing to cancel any more");
}

/// Design change 0234: "an Agent ordered to build while carrying crystals or Supplies keeps its load, in the site and in
/// the network"; "The Agent is untargetable for the duration of construction" (`factions/syndicate/units/agent.md >
/// Building > Tunnel Construction Flow`): it is off the map, and player 2's Guard can't be ordered to attack it, only
/// the site; and "What enemies see: the site, like any surface structure, but not the Agent inside it". Agent 2
/// mines a load at patch 5 and is ordered to build before it delivers.
#[test]
fn the_agent_inside_keeps_its_load_and_is_out_of_reach() {
    let mut t = base()
        .spawn(AGENT, Some(P1), point(12, 16)) // 2
        .spawn(GUARD, Some(P2), point(22, 10)) // 3
        .spawn(AGENT, Some(P1), point(9, 16)) // 4
        .spawn_with(SpawnSettings::CrystalPatch { starting_amount: 1000 }, tile(13, 17)) // 5
        .start();
    t.without_auto_targeting().debug_at(1, supplies_of(P1, 5));
    step(&mut t);
    next(&mut t, P1, order(&[2], OrderKind::Gather(id(5)), false));
    step_until(&mut t, 300, "Agent 2 mines a load", |t| load(t, 2) > 0);
    let carried = load(&t, 2);
    next(&mut t, P1, build(&[2], SITE, false));
    let (started, site) = until_started(&mut t, 2);
    assert_eq!(load(&t, 2), carried);
    assert!(t.sim().vision().sees(P2, object(&t, site)), "player 2 sees the site");
    assert_eq!(gate(&t, P2, order(&[3], OrderKind::Attack(site), false)), Ok(()));
    assert_eq!(
        gate(&t, P2, order(&[3], OrderKind::Attack(id(2)), false)),
        Err(Reject::UnknownObject(id(2)))
    );

    run_until(&mut t, started + 480);
    assert_eq!(inside(&t, P1), [id(2)]);
    assert_eq!(load(&t, 2), carried);
    next(&mut t, P1, eject(site));
    step_until(&mut t, 100, "the Agent comes out", |t| t.sim().object(id(2)).is_some());
    assert_eq!(load(&t, 2), carried);
}

/// Design changes 0233, 0234 and 0284: Cancel Construction refunds "the current construction price, not counting
/// itself" ("the current top price"), 1 Supply for Tunnel 1, and the Agent "comes out at the centre of the
/// footprint, which is now free; it then idles".
#[test]
fn a_cancelled_site_refunds_the_top_price_and_lets_the_agent_out() {
    let mut t = field();
    let (_, site) = with_site(&mut t);
    assert_eq!(object(&t, site).pos, SITE_CENTRE);
    next(&mut t, P1, cancel(&[site]));
    let tick = t.sim().tick();
    let cancelled = SimEvent::SiteCancelled {
        site,
        kind: TUNNEL,
        refund: 1,
    };
    assert!(t.events().contains(&(tick, cancelled)));
    assert!(t.events().contains(&(tick, SimEvent::LeftSite { worker: id(2), site })));
    assert_eq!((supplies(&t, P1), price(&t, P1), sites(&t)), (5, 1, vec![]));
    assert!(t.sim().object(site).is_none());
    assert_eq!(t.pos(2), SITE_CENTRE);
    assert!(idle(&t, 2));
    assert!(t.sim().held_unit(id(2)).is_none());
}

/// `factions/syndicate/units/agent.md > Building > Tunnel Construction Flow`: "If the partially-built Tunnel is
/// destroyed: The Agent survives and emerges at the location. The Tunnel is lost and any Supplies spent are lost."
/// Six Guards of player 2's, spawned around the site once it stands, outdo its growing maximum.
#[test]
fn a_destroyed_site_loses_its_supplies_and_lets_the_agent_out() {
    let mut t = field();
    let (_, site) = with_site(&mut t);
    let spots = [(21, 9), (21, 10), (21, 11), (14, 9), (14, 10), (14, 11)];
    for (x, y) in spots {
        let spawn = DebugCommand::Spawn {
            kind: GUARD,
            owner: Some(P2),
            at: point(x, y),
            orientation: Default::default(),
        };
        t.debug_at(t.sim().tick() + 1, spawn);
    }
    step(&mut t);
    let guards: Vec<u32> = (site.0 + 1..=site.0 + 6).collect();
    next(&mut t, P2, order(&guards, OrderKind::Attack(site), false));
    step_until(&mut t, 400, "the Guards destroy the site", |t| {
        t.sim().object(site).is_none()
    });
    let died = t
        .events()
        .iter()
        .any(|(_, e)| matches!(*e, SimEvent::Died { id: dead, .. } if dead == site));
    assert!(died);
    assert!(
        t.events()
            .iter()
            .any(|(_, e)| *e == SimEvent::LeftSite { worker: id(2), site })
    );
    assert_eq!(t.pos(2), SITE_CENTRE);
    assert!(idle(&t, 2));
    assert_eq!((supplies(&t, P1), price(&t, P1), refunds(&t)), (4, 1, vec![]));
    let cancelled = t
        .events()
        .iter()
        .any(|(_, e)| matches!(e, SimEvent::SiteCancelled { .. }));
    assert!(!cancelled);
}

/// Design change 0234: "Defeat: the site counts as a structure ... Cancelling it when it is the player's only structure
/// defeats them in that tick". Player 1 starts with no structure, only Agent 1, whose Tunnel costs nothing
/// (`factions/syndicate/tunnels.md > Losing Tunnels`: "with no Tunnels owned, a new Tunnel costs 0 Supplies"), and
/// gets nothing back for it; player 2, who never owned a structure, stands.
#[test]
fn cancelling_the_only_structure_defeats_its_player_in_that_tick() {
    let mut t = syndicate(&flat(40, 20))
        .spawn(AGENT, Some(P1), point(12, 16)) // 1
        .spawn(GUARD, Some(P2), point(36, 2)) // 2
        .start();
    t.without_auto_targeting().debug_at(1, supplies_of(P1, 0));
    step(&mut t);
    assert_eq!(price(&t, P1), 0);
    next(&mut t, P1, build(&[1], SITE, false));
    assert_eq!((supplies(&t, P1), price(&t, P1)), (0, 1));
    let (_, site) = until_started(&mut t, 1);
    for _ in 0..10 {
        step(&mut t);
    }
    assert_eq!(t.sim().player(P1).and_then(|p| p.defeated_at), None);
    next(&mut t, P1, cancel(&[site]));
    let tick = t.sim().tick();
    assert_eq!(t.sim().player(P1).and_then(|p| p.defeated_at), Some(tick));
    assert!(t.events().contains(&(tick, SimEvent::PlayerDefeated { player: P1 })));
    let refund = SimEvent::SiteCancelled {
        site,
        kind: TUNNEL,
        refund: 0,
    };
    assert!(t.events().contains(&(tick, refund)));
    assert_eq!(t.sim().outcome().map(|outcome| outcome.winners.clone()), Some(vec![P2]));
}
