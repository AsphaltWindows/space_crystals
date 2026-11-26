//! What refuses an upgrade (`factions/syndicate/tunnel_area_and_space.md > Tunnel Area Non-Overlap Rule`; design
//! changes 0237, 0242 and 0283), by the truth, and the gate's order: the top tier, an operation under way, the
//! Supplies, then what blocks the larger area, never said what. And what doesn't: the owner's lingering ghost, whose
//! build then fails on arrival.

use super::*;

/// The plan's Acceptance, "Blocked by an own neighbour within the Tier 2 area"
/// (`factions/syndicate/tunnel_area_and_space.md > Tunnel Area Non-Overlap Rule`: "upgrading a Tunnel is blocked if the
/// radius increase would cause its Tunnel Area to overlap with another Tunnel's current area"): player 1's Tunnel 2 on
/// tiles (12, 8) to (15, 11), its Tier 1 area (10, 6) to (17, 13) touching [`T1`]'s, blocks [`T1`]'s upgrade, and
/// [`T1`] its. A tile further east it leaves room for either, not both: the first to upgrade reserves its Tier 2 area
/// against the other's ("including other Tunnels' reserved upgrade areas"), until it is cancelled.
#[test]
fn an_own_neighbour_within_the_tier_2_area_blocks_it() {
    let t = started(base().spawn(TUNNEL, Some(P1), tile(12, 8)), 20);
    assert_eq!(gate(&t, P1, upgrade(T1)), Err(Reject::UpgradeBlocked(T1)));
    assert_eq!(gate(&t, P1, upgrade(id(2))), Err(Reject::UpgradeBlocked(id(2))));
    assert_eq!(t.sim().can_upgrade(P1, T1), Err(Reject::UpgradeBlocked(T1)));

    let mut t = started(base().spawn(TUNNEL, Some(P1), tile(13, 8)), 20);
    assert_eq!(gate(&t, P1, upgrade(T1)), Ok(()));
    assert_eq!(gate(&t, P1, upgrade(id(2))), Ok(()));
    next(&mut t, P1, upgrade(id(2)));
    assert_eq!(
        gate(&t, P1, upgrade(T1)),
        Err(Reject::UpgradeBlocked(T1)),
        "Tunnel 2's reserved Tier 2 area"
    );
    next(&mut t, P1, cancel(id(2)));
    assert_eq!(gate(&t, P1, upgrade(T1)), Ok(()));
}

/// Design change 0237's verification: "In a two-Syndicate game, place a Tier 1 Tunnel 1 tile from where an enemy Tier
/// 1 Tunnel's Tier 2 area would reach. The enemy can't upgrade that Tunnel." Player 2's Tunnel (1) on tiles (10, 8) to
/// (13, 11) has its Tier 1 area to x = 15, and would have its Tier 2 area to x = 16. Player 1's Agent builds a Tunnel
/// whose Tier 1 area starts there, at x = 16, beside player 2's Tier 1 area: as a site and once complete, it blocks
/// player 2's upgrade ("**Upgrades** are blocked by any Tunnel's area, own or enemy (the upgrade check uses the truth;
/// the upgrading Tunnel's owner learns that something blocks it)"; a site holds its area, design change 0289 point 3).
/// The refusal names only player 2's own Tunnel, and the command changes nothing. A tile further east, it doesn't.
#[test]
fn an_enemy_tunnel_blocks_it_by_the_truth() {
    let theirs = id(1);
    let blocked = Err(Reject::UpgradeBlocked(theirs));
    for (x, verdict) in [(18, blocked), (19, Ok(()))] {
        let scenario = syndicate(&flat(40, 20))
            .spawn(TUNNEL, Some(P2), tile(10, 8)) // 1
            .spawn(TUNNEL, Some(P1), tile(30, 8)) // 2
            .spawn(AGENT, Some(P1), point(20, 16)); // 3
        let mut t = started(scenario, 10);
        next(&mut t, P2, Command::Debug(supplies_of(P2, 20)));
        assert_eq!(gate(&t, P2, upgrade(theirs)), Ok(()));
        next(&mut t, P1, build(id(3), at(x, 8)));
        step_until(&mut t, 600, "the site starts", |t| {
            t.sim().objects().any(Object::is_site)
        });
        assert_eq!(gate(&t, P2, upgrade(theirs)), verdict, "a site at x = {x}");
        let site = (t.sim().objects()).find(|o| o.is_site()).map(|o| o.id).expect("a site");
        step_until(&mut t, 600, "the site completes", |t| !object(t, site).is_site());
        assert_eq!(gate(&t, P2, upgrade(theirs)), verdict, "a Tunnel at x = {x}");
        if verdict.is_err() {
            next(&mut t, P2, upgrade(theirs));
            assert_eq!((starts(&t), supplies(&t, P2)), (vec![], 20));
            assert_eq!(upgrading(&t, theirs), None);
        }
    }
    let said = Reject::UpgradeBlocked(theirs).to_string();
    assert_eq!(said, "something blocks Tunnel 1's upgrade", "never what");
}

/// Design change 0242's verification: "Upgrading a Tier 1 Tunnel whose Tier 2 area would reach a Mountain is
/// unavailable, with feedback" (`factions/syndicate/tunnel_area_and_space.md > Tunnel Area Non-Overlap Rule`:
/// "An **upgrade** is likewise blocked if the larger area would include a non-Drillable or off-map cell, so a Tier 1
/// Tunnel by a lake or a mountain may never be upgradable"): a Mountain tile, or a Water tile, at (10, 10), in [`T1`]'s
/// Tier 2 area but outside its Tier 1 area. The gate's reason is the panel's feedback. On flat ground the upgrade is
/// taken.
#[test]
fn mountain_or_water_in_the_tier_2_area_blocks_it() {
    for symbol in ["M", "~"] {
        let mut rows: Vec<String> = (0..20).map(|_| ".".repeat(40)).collect();
        rows[10].replace_range(10..11, symbol);
        let t = started(syndicate(&rows.join("\n")).spawn(TUNNEL, Some(P1), tile(4, 8)), 20);
        assert_eq!(gate(&t, P1, upgrade(T1)), Err(Reject::UpgradeBlocked(T1)), "{symbol}");
    }
    let t = started(base(), 20);
    assert_eq!(gate(&t, P1, upgrade(T1)), Ok(()));
}

/// The gate's verdict on the upgrade of player 1's lone Tunnel at `origin` on a flat 40 × 24 map, at tier `tier`.
fn edge_verdict(origin: TilePos, tier: u8) -> Result<(), Reject> {
    let mut t = started(
        syndicate(&flat(40, 24)).spawn(TUNNEL, Some(P1), tile(origin.x, origin.y)),
        20,
    );
    if tier > 1 {
        next(&mut t, P1, set_tier(T1, tier));
    }
    gate(&t, P1, upgrade(T1))
}

/// The origin of a 4 × 4 footprint `d` tiles from the `side` edge of a 40 × 24 map ("west", "east", "north" or
/// "south"), half-way along it.
fn from_edge(side: &str, d: i32) -> TilePos {
    match side {
        "west" => at(d, 10),
        "east" => at(40 - 4 - d, 10),
        "north" => at(18, d),
        _ => at(18, 24 - 4 - d),
    }
}

/// Design change 0283 (`factions/syndicate/tunnel_area_and_space.md > Tunnel Area Non-Overlap Rule`: every cell of the
/// area must be on the map, so "a Tunnel needs as many tiles to the edge as its area's radius"): "**Upgrades:** 3 tiles
/// for Tier 2, 4 for Tier 3". By each edge of a 40 × 24 map, the 4 × 4 Tunnel's footprint 2 tiles from it can't upgrade
/// to Tier 2 and 3 tiles from it can; at Tier 2, 3 tiles from it can't upgrade to Tier 3 and 4 tiles from it can.
#[test]
fn by_every_edge_the_map_blocks_it_3_tiles_off_for_tier_2_and_4_for_tier_3() {
    let blocked = Err(Reject::UpgradeBlocked(T1));
    for side in ["west", "east", "north", "south"] {
        let origin = |d| from_edge(side, d);
        assert_eq!(
            edge_verdict(origin(2), 1),
            blocked,
            "Tier 2, 2 tiles from the {side} edge"
        );
        assert_eq!(
            edge_verdict(origin(3), 1),
            Ok(()),
            "Tier 2, 3 tiles from the {side} edge"
        );
        assert_eq!(
            edge_verdict(origin(3), 2),
            blocked,
            "Tier 3, 3 tiles from the {side} edge"
        );
        assert_eq!(
            edge_verdict(origin(4), 2),
            Ok(()),
            "Tier 3, 4 tiles from the {side} edge"
        );
    }
}

/// `factions/syndicate/tunnel_menus.md > Tunnel ObjectInterfaceState`: "Unavailable if already Tier 3 or if currently
/// performing an operation" ("A Tunnel can only perform one operation at a time": an upgrade, or an expansion it
/// builds), and the Supplies; in the gate's order, the top tier first, then the operation, then the Supplies, then
/// what blocks the area. The Upgrade is the owner's, of a Tunnel in operation: not of a site, nor a Hideout.
#[test]
fn busy_at_the_top_tier_or_short_it_is_refused_in_that_order() {
    let mut t = started(base(), 20);
    next(&mut t, P1, upgrade(T1));
    assert_eq!(gate(&t, P1, upgrade(T1)), Err(Reject::OperationUnderWay(T1)));
    assert_eq!(gate(&t, P2, upgrade(T1)), Err(Reject::NotOwned(T1)));
    next(&mut t, P1, Command::Debug(supplies_of(P1, 0)));
    assert_eq!(
        gate(&t, P1, upgrade(T1)),
        Err(Reject::OperationUnderWay(T1)),
        "busy before short"
    );
    next(&mut t, P1, cancel(T1));
    assert_eq!(supplies(&t, P1), 2, "the refund");
    next(&mut t, P1, Command::Debug(supplies_of(P1, 1)));
    assert_eq!(gate(&t, P1, upgrade(T1)), Err(Reject::InsufficientSupplies));

    // An expansion under way is the Tunnel's operation too.
    let crystals = DebugCommand::SetResources {
        player: P1,
        crystals: 1000,
        supplies: 20,
        alloys: 0,
        essence: 0,
        flux: 0,
    };
    next(&mut t, P1, Command::Debug(crystals));
    let expand = TunnelCommand::Expand {
        kind: ObjectKind::Hideout,
        origin: at(2, 6),
        orientation: BuildingOrientation::default(),
    };
    next(
        &mut t,
        P1,
        Command::Tunnel {
            tunnel: T1,
            command: expand,
        },
    );
    let site = (t.sim().objects())
        .find(|o| o.is_site())
        .map(|o| o.id)
        .expect("the Hideout's site");
    assert_eq!(gate(&t, P1, upgrade(T1)), Err(Reject::OperationUnderWay(T1)));
    assert_eq!(gate(&t, P1, upgrade(site)), Err(Reject::NotATunnel(site)));
    next(&mut t, P1, cancel(T1));

    next(&mut t, P1, set_tier(T1, 3));
    assert_eq!(gate(&t, P1, upgrade(T1)), Err(Reject::TopTier(T1)));
    next(&mut t, P1, Command::Debug(supplies_of(P1, 0)));
    assert_eq!(
        gate(&t, P1, upgrade(T1)),
        Err(Reject::TopTier(T1)),
        "the top tier before the Supplies"
    );

    // Short and blocked (by Tunnel 2, as in `an_own_neighbour_within_the_tier_2_area_blocks_it`): the Supplies first.
    let mut t = started(base().spawn(TUNNEL, Some(P1), tile(12, 8)), 1);
    assert_eq!(gate(&t, P1, upgrade(T1)), Err(Reject::InsufficientSupplies));
    next(&mut t, P1, Command::Debug(supplies_of(P1, 2)));
    assert_eq!(gate(&t, P1, upgrade(T1)), Err(Reject::UpgradeBlocked(T1)));
}

/// The Director's reading of `factions/syndicate/tunnel_area_and_space.md > Tunnel Area Non-Overlap Rule` (note 3 of
/// slice M4-G's brief): a lingering Tunnel ghost "reserves its whole Tier 1 area against its **owner's** further Tunnel
/// placements", and "**Upgrades** are blocked by any Tunnel's area": a ghost is no Tunnel, and an upgrade no placement.
/// So player 1's Agent's pending Build, whose Tier 1 area (10, 6) to (17, 13) reaches into [`T1`]'s Tier 2 area,
/// doesn't block [`T1`]'s upgrade; the ghost is red over the reserved area from then on, and the Build fails on its
/// arrival by the truth (`Sim::arrival_check`), its price refunded.
#[test]
fn the_owners_lingering_ghost_doesnt_block_it_and_its_build_fails_on_arrival() {
    let mut t = started(base().spawn(AGENT, Some(P1), point(36, 2)), 20); // 2, far off
    let agent = id(2);
    let site = at(12, 8);
    assert_eq!(ghost(&t, agent, site), PlacementCheck::default());
    next(&mut t, P1, build(agent, site));
    assert_eq!(t.sim().pending_builds(P1).len(), 1);
    assert_eq!(supplies(&t, P1), 19, "the next Tunnel's price, 1");
    assert_eq!(gate(&t, P1, upgrade(T1)), Ok(()));
    next(&mut t, P1, upgrade(T1));
    assert_eq!(upgrading(&t, T1), Some(2));
    assert_eq!(ghost(&t, agent, site).whole, [sc_sim::WholeProblem::TunnelAreaOverlap]);
    step_until(&mut t, 600, "the Build ends", |t| !failures(t).is_empty());
    assert_eq!(failures(&t), [(agent, vec![sc_sim::WholeProblem::TunnelAreaOverlap])]);
    assert_eq!(supplies(&t, P1), 18, "the Build's price back; the upgrade's 2 paid");
    assert_eq!(upgrading(&t, T1), Some(2), "the upgrade goes on");
}
