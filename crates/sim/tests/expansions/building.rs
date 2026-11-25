//! An expansion under construction (`factions/syndicate/tunnels.md > Expansions under construction`, design change
//! 0254): the click pays and takes Tunnel Space, the frames and the HP, the expansion at work once complete, and the
//! Tunnel's Eject meanwhile.

use sc_sim::ProductionCommand;

use super::*;

/// 0254: "**Paid when the ghost is placed** ... **Tunnel Space is used once construction has started.** Today placing
/// the ghost starts construction at once": from the start's 1/11 (48 tunnel-region cells less the Hideout's 4, over
/// 4; the Agent's 1), a second Hideout gives 1/10 at the click, which pays its 200 Space Crystals
/// (`factions/syndicate/structures/hideout.md`: "Cost - 200 Space Crystals"). The site is player 1's, a structure
/// under construction from the Tunnel, at 10% of its HP.
#[test]
fn the_click_pays_and_takes_tunnel_space() {
    let mut t = started(base(), 1000);
    assert_eq!(tunnel_space(&t, P1), (1, 11));
    next(&mut t, P1, expand(2, 6));
    assert_eq!((tunnel_space(&t, P1), crystals(&t, P1)), ((1, 10), 800));
    let site = building(&t);
    let o = object(&t, site);
    assert_eq!(
        (o.kind, o.owner, o.is_site(), o.is_operational()),
        (HIDEOUT, Some(P1), true, false)
    );
    assert_eq!((o.hp, o.max_hp()), (Some(40.0), Some(40.0)));
    let started = events(&t, |e| match *e {
        SimEvent::ExpansionStarted { tunnel, site } => Some((tunnel, site)),
        _ => None,
    });
    assert_eq!(started, [(T1, site)]);
    // A Hideout too dear: the bank refuses it.
    next(&mut t, P1, cancel_operation());
    next_debug(&mut t, set_crystals(P1, 199));
    assert_eq!(gate(&t, P1, expand(2, 6)), Err(Reject::InsufficientCrystals));
}

/// `factions/syndicate/structures/hideout.md` ("Build Time - 400 frames (25 seconds)", "HP - 400") and 0254: "**HP**:
/// the ConstructionHP Rule applies to every expansion (starts at 10%, grows with progress); damage taken stays". Placed
/// in tick 2, the Hideout's maximum grows a frame a tick from 40, from tick 3 on, while the damage it takes from player
/// 1's own Peacekeeper (an Attack on it: the owner sees it) stays taken, and it is complete at the end of tick 402,
/// at 400 less that damage. It takes no Train until then (its production held back), and then trains Agents into the
/// Tunnel Network (`factions/syndicate/structures/README.md > Tunnel Expansions > Rally Point Behavior`); the Tunnel
/// is free again.
#[test]
fn a_hideout_is_built_in_400_ticks_keeping_its_damage_then_trains() {
    let scenario = base().spawn(ObjectKind::Peacekeeper, Some(P1), point(2, 2)); // 4
    let mut t = started(scenario, 1000);
    next(&mut t, P1, expand(2, 6));
    assert_eq!(t.sim().tick(), 2);
    let site = building(&t);
    let train = Command::Production {
        objects: vec![site],
        command: ProductionCommand::Train(AGENT),
    };
    assert_eq!(
        gate(&t, P1, train.clone()),
        Err(Reject::NoProducers),
        "nothing but Cancel"
    );
    let attack = Command::Order {
        units: vec![ObjectId(4)],
        kind: sc_sim::OrderKind::Attack(site),
        queue: false,
    };
    next(&mut t, P1, attack);
    let damage = |t: &TestSim| {
        let s = object(t, site);
        s.max_hp().expect("a maximum") - s.hp.expect("HP")
    };
    while damage(&t) < 4.0 {
        assert!(t.sim().tick() < 60, "the Peacekeeper hits the site");
        t.step();
    }
    let stop = Command::Order {
        units: vec![ObjectId(4)],
        kind: sc_sim::OrderKind::Stop,
        queue: false,
    };
    next(&mut t, P1, stop);
    let taken = damage(&t);
    let mut last_max = object(&t, site).max_hp();
    while t.sim().tick() < 401 {
        t.step();
        let max = object(&t, site).max_hp();
        assert!(max > last_max, "tick {}: {max:?} after {last_max:?}", t.sim().tick());
        assert!(
            (damage(&t) - taken).abs() < 0.05,
            "the damage stays taken: {} of {taken}",
            damage(&t)
        );
        last_max = max;
    }
    assert_eq!(
        (object(&t, site).is_site(), operation(&t)),
        (true, Some(Operation::Building(site)))
    );
    t.step();
    let s = object(&t, site);
    assert_eq!((t.sim().tick(), s.is_site(), s.max_hp()), (402, false, Some(400.0)));
    assert!((s.hp.expect("HP") - (400.0 - taken)).abs() < 0.05);
    assert_eq!(operation(&t), None);
    let completed = events(&t, |e| match *e {
        SimEvent::ExpansionCompleted { tunnel, site } => Some((tunnel, site)),
        _ => None,
    });
    assert_eq!(completed, [(T1, site)]);
    // At work: it trains an Agent into the network in its 240 frames.
    assert_eq!(gate(&t, P1, train.clone()), Ok(()));
    next(&mut t, P1, train);
    assert_eq!(inside(&t, P1), []);
    t.run(245);
    assert_eq!(inside(&t, P1).len(), 1, "the Agent is in the Tunnel Network");
    assert_eq!(gate(&t, P1, expand(6, 12)), Ok(()), "the Tunnel is free");
}

/// 0254: "**The Tunnel meanwhile** shows the progress; Eject still works (it isn't an operation)": an Agent in the
/// network is ejected while the Tunnel builds, and the construction goes on. The Tunnel's progress is the site's
/// (`Sim::operation_progress`).
#[test]
fn eject_works_during_construction() {
    let mut t = started(base(), 1000);
    let enter = Command::Order {
        units: vec![AGENT_1],
        kind: sc_sim::OrderKind::Enter(T1),
        queue: false,
    };
    next(&mut t, P1, enter);
    while inside(&t, P1).is_empty() {
        assert!(t.sim().tick() < 300, "the Agent walks in");
        t.step();
    }
    t.run(sc_sim::defs::TRANSIT_DELAY_TICKS + 1);
    next(&mut t, P1, expand(2, 6));
    let site = building(&t);
    t.run(100);
    let (current, progress) = t.sim().operation_progress(T1).expect("an operation");
    assert_eq!(current, Operation::Building(site));
    assert!((progress - 100.0 / 400.0).abs() < 1e-3, "{progress}");
    let eject = Command::Tunnel {
        tunnel: T1,
        command: TunnelCommand::Eject { kind: AGENT },
    };
    assert_eq!(gate(&t, P1, eject.clone()), Ok(()));
    next(&mut t, P1, eject);
    let mut out = false;
    for _ in 0..60 {
        t.step();
        out = t.sim().object(AGENT_1).is_some();
        if out {
            break;
        }
    }
    assert!(out, "the Agent came out");
    assert_eq!(
        operation(&t),
        Some(Operation::Building(site)),
        "the construction goes on"
    );
}
