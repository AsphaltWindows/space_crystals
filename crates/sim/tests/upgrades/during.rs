//! During an upgrade (`factions/syndicate/tunnels.md > Cost Scaling > Upgrade Costs`: "**The future area is
//! reserved**"; `factions/syndicate/tunnels.md > Tunnel Network`, design change 0249): the reserved area against the
//! owner's ghosts and builds, and against an enemy's, which judges by what it sees until the arrival's check by the
//! truth; the old tier's transit, exits and Tunnel Space; the Tunnel's X; the Tunnel destroyed.

use sc_sim::defs::TRANSIT_DELAY_TICKS;
use sc_sim::{PlacementProblem, WholeProblem};

use super::*;

/// The plan's Acceptance, "During an upgrade, a Tunnel ghost overlapping the Tier 2 area is red, and refused"
/// (`factions/syndicate/tunnels.md > Cost Scaling > Upgrade Costs`: "from the start of the upgrade, the larger Tunnel
/// Area of the target tier counts against other placements"). Player 1's Agent's ghost at (12, 8), whose Tier 1 area
/// (10, 6) to (17, 13) takes the column x = 10 of [`T1`]'s Tier 2 area, is green before the upgrade, red from its
/// start, its Build refused, and green again once the Tunnel's X cancelled it; a tile further east it is green
/// throughout.
#[test]
fn the_owners_ghost_over_the_reserved_area_is_red_and_its_build_refused() {
    let mut t = started(base().spawn(AGENT, Some(P1), point(14, 16)), 20); // 2
    let agent = id(2);
    let (over, beside) = (at(12, 8), at(13, 8));
    assert_eq!(ghost(&t, agent, over), PlacementCheck::default());
    next(&mut t, P1, upgrade(T1));
    let overlap = WholeProblem::TunnelAreaOverlap;
    assert_eq!(ghost(&t, agent, over).whole, [overlap]);
    let refused = Err(Reject::Unplaceable(PlacementProblem::Whole(overlap)));
    assert_eq!(gate(&t, P1, build(agent, over)), refused);
    assert_eq!(ghost(&t, agent, beside), PlacementCheck::default());
    assert_eq!(gate(&t, P1, build(agent, beside)), Ok(()));
    t.run(300);
    assert_eq!(ghost(&t, agent, over).whole, [overlap], "all along");
    next(&mut t, P1, cancel(T1));
    assert_eq!(ghost(&t, agent, over), PlacementCheck::default());
}

/// Design change 0237 ("The ghost judges enemy Tunnels by what the player knows ... The real check on arrival uses the
/// truth") with the reserved area: player 2's Agent sees [`T1`], at Tier 1, and nothing of its upgrade (an enemy sees
/// a Tunnel's tier: `sc_sim::placement::Occupant::as_seen`). Its ghost is red over [`T1`]'s Tier 1 area and green
/// over the ring its upgrade reserves. Its Build there is taken, and fails on arrival, by the truth.
#[test]
fn an_enemys_ghost_judges_by_the_tier_it_sees_and_its_arrival_by_the_truth() {
    let mut t = started(base().spawn(AGENT, Some(P2), point(9, 14)), 20); // 2, T1 within its sight
    let theirs = id(2);
    next(&mut t, P1, upgrade(T1));
    assert!(t.sim().is_visible(P2, object(&t, T1)), "player 2 sees the Tunnel");
    assert_eq!(ghost(&t, theirs, at(11, 8)).whole, [WholeProblem::TunnelAreaOverlap]);
    assert_eq!(ghost(&t, theirs, at(12, 8)), PlacementCheck::default());
    assert_eq!(gate(&t, P2, build(theirs, at(12, 8))), Ok(()));
    next(&mut t, P2, build(theirs, at(12, 8)));
    step_until(&mut t, 600, "the Build ends", |t| !failures(t).is_empty());
    assert_eq!(failures(&t), [(theirs, vec![WholeProblem::TunnelAreaOverlap])]);
    assert_eq!(upgrading(&t, T1), Some(2));
    assert!(t.sim().objects().all(|o| !o.is_site()), "no site");
}

/// An Enter of `units` into `tunnel`.
fn enter(units: &[ObjectId], tunnel: ObjectId) -> Command {
    Command::Order {
        units: units.to_vec(),
        kind: OrderKind::Enter(tunnel),
        queue: false,
    }
}

/// An Eject of a Drone at `tunnel`.
fn eject_drone(tunnel: ObjectId) -> Command {
    let command = TunnelCommand::Eject {
        kind: ObjectKind::Drone,
    };
    Command::Tunnel { tunnel, command }
}

/// Design change 0249's verification: "A Tunnel upgrading from Tier 1 to Tier 2 won't let out a vehicle until the
/// upgrade completes" (`factions/syndicate/tunnels.md > Tunnel Network`: "until the upgrade has fully finished, a
/// Tunnel only accepts, and only lets out, units supported by the tier it had **before** the upgrade began"), with the
/// plan's test vehicle, a Drone of player 1's. One is in the network, through player 1's Tier 2 Tunnel 2; another
/// stands by [`T1`]. Through [`T1`]'s upgrade neither comes out of it nor goes into it, and its Tunnel Space is its
/// Tier 1 region's; at completion it lets the vehicles in and out, and its Tunnel Space is 9 more.
#[test]
fn transit_exits_and_tunnel_space_keep_the_old_tier_until_complete() {
    let scenario = base()
        .spawn(TUNNEL, Some(P1), tile(24, 8)) // 2
        .spawn(ObjectKind::Drone, Some(P1), point(10, 16)) // 3, by T1
        .spawn(ObjectKind::Drone, Some(P1), point(26, 4)); // 4, north of Tunnel 2
    let mut t = started(scenario, 20);
    let (other, outside, inside) = (id(2), id(3), id(4));
    next(&mut t, P1, set_tier(other, 2));
    next(&mut t, P1, enter(&[inside], other));
    step_until(&mut t, 300, "the Drone walks in", |t| t.sim().object(inside).is_none());
    t.run(TRANSIT_DELAY_TICKS);
    let before = tunnel_space(&t, P1);

    next(&mut t, P1, upgrade(T1));
    let ordered = t.sim().tick();
    let no_entry = Err(Reject::NoTransit { tunnel: T1, kind: None });
    let no_exit = Err(Reject::NoTransit {
        tunnel: T1,
        kind: Some(ObjectKind::Drone),
    });
    for tick in [ordered + 100, ordered + FRAMES - 2] {
        t.run_until(tick);
        assert_eq!(upgrading(&t, T1), Some(2));
        assert_eq!(gate(&t, P1, enter(&[outside], T1)), no_entry, "tick {tick}");
        assert_eq!(gate(&t, P1, eject_drone(T1)), no_exit, "tick {tick}");
        assert_eq!(gate(&t, P1, eject_drone(other)), Ok(()), "Tunnel 2 lets it out");
        assert_eq!(tunnel_space(&t, P1), before, "tick {tick}");
    }
    // Refused in the upgrade's last tick: the Drone stays inside.
    next(&mut t, P1, eject_drone(T1));
    assert_eq!(t.sim().tick(), ordered + FRAMES - 1);
    assert_eq!(tier(&t, T1), 2, "complete at the end of the tick");
    assert!(t.sim().object(inside).is_none(), "still inside");

    assert_eq!(tunnel_space(&t, P1), (before.0, before.1 + 9));
    assert_eq!(gate(&t, P1, enter(&[outside], T1)), Ok(()));
    next(&mut t, P1, eject_drone(T1));
    assert!(t.sim().object(inside).is_some(), "out of the Tier 2 Tunnel");
}

/// The Tunnel's X ("**X** Cancel (upgrade, construction or the current operation)"): the upgrade ends, the refund comes
/// back, and the operation slot is free, for an expansion as for another upgrade.
#[test]
fn the_tunnels_x_frees_the_slot_with_the_refund() {
    let mut t = started(base(), 20);
    let banks = DebugCommand::SetResources {
        player: P1,
        crystals: 1000,
        supplies: 20,
        alloys: 0,
        essence: 0,
        flux: 0,
    };
    next(&mut t, P1, Command::Debug(banks));
    let expand = TunnelCommand::Expand {
        kind: ObjectKind::Hideout,
        origin: at(2, 6),
        orientation: BuildingOrientation::default(),
    };
    let expand = Command::Tunnel {
        tunnel: T1,
        command: expand,
    };
    next(&mut t, P1, upgrade(T1));
    assert_eq!(supplies(&t, P1), 18);
    assert_eq!(gate(&t, P1, expand.clone()), Err(Reject::OperationUnderWay(T1)));
    t.run(200);
    next(&mut t, P1, cancel(T1));
    assert_eq!(cancels(&t), [(T1, 2, 2)]);
    assert_eq!((supplies(&t, P1), tier(&t, T1), upgrading(&t, T1)), (20, 1, None));
    assert_eq!(gate(&t, P1, expand), Ok(()));
    assert_eq!(gate(&t, P1, upgrade(T1)), Ok(()));
}

/// `factions/syndicate/tunnels.md > Cost Scaling > Upgrade Costs`: "**Destroyed mid-upgrade** ...: the Supplies paid
/// for the upgrade are lost". [`T1`] and Tunnel 2 upgrade together, for 2 and 4; [`T1`] is destroyed: nothing comes
/// back, no cancel is reported, and it no longer counts in Tunnel 2's price, whose X would now refund 2.
#[test]
fn destroyed_mid_upgrade_the_supplies_are_lost() {
    let mut t = started(base().spawn(TUNNEL, Some(P1), tile(24, 8)), 20);
    let other = id(2);
    next_all(&mut t, vec![(P1, upgrade(T1)), (P1, upgrade(other))]);
    assert_eq!(supplies(&t, P1), 14);
    t.run(100);
    assert_eq!(price(&t, other), Some(4));
    next(&mut t, P1, remove(T1));
    assert!(t.sim().object(T1).is_none());
    assert_eq!((supplies(&t, P1), cancels(&t)), (14, vec![]));
    assert_eq!(price(&t, other), Some(2));
    t.run(FRAMES);
    assert_eq!((tier(&t, other), supplies(&t, P1)), (2, 14));
}
