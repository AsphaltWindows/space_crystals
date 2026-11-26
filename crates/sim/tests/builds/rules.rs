//! Where a Tunnel may stand (`core/entities.md > Placement Validation > Worker-Built Structures`): its ghost judged by
//! what the player knows (`Sim::build_check`, `Mode::Known`), its arrival by the truth; the overlap rule (design change
//! 0237), the Drillable area (0242), the hidden expansions (0240), a unit on the site, and the lingering ghosts
//! (0125; design change 0289 point 2).

use super::*;
use sc_sim::tuning::vision::FOG_DELAY_TICKS;
use sc_sim::{PlacementProblem, TileProblem, WholeProblem};

/// What a ghost shows whose only problem is its area's overlap with another Tunnel's (design change 0237).
fn overlap() -> PlacementCheck {
    PlacementCheck {
        tiles: Vec::new(),
        whole: vec![WholeProblem::TunnelAreaOverlap],
    }
}

/// The gate's refusal of a Build for its whole problem `problem`.
fn refused(problem: WholeProblem) -> Result<(), Reject> {
    Err(Reject::Unplaceable(PlacementProblem::Whole(problem)))
}

/// The tiles of a footprint of `w` × `h` at `origin`.
fn tiles(origin: TilePos, w: i32, h: i32) -> Vec<TilePos> {
    (0..h)
        .flat_map(|dy| (0..w).map(move |dx| at(origin.x + dx, origin.y + dy)))
        .collect()
}

/// Design change 0237 (`factions/syndicate/tunnel_area_and_space.md > Tunnel Area Non-Overlap Rule`): a Tunnel's Tier 1
/// area may not overlap the area of a Tunnel of its owner's, at its current tier ("The non-overlap check uses the
/// current tier's area"); it may touch it. The ghost shows it as a whole problem (`WholeProblem::TunnelAreaOverlap`),
/// and the gate refuses the order with it. On arrival the check is the truth: Tunnel 1, upgraded since the order (a
/// debug tier here, as slice M4-G will), fails it, refunded. A site's area counts as any Tunnel's (design change 0289
/// point 3: "A site counts with its whole area for every player from the moment it appears").
#[test]
fn an_area_may_not_overlap_an_own_tunnels() {
    let mut t = base()
        .spawn(AGENT, Some(P1), point(12, 16)) // 2
        .spawn(AGENT, Some(P1), point(13, 17)) // 3
        .start();
    t.debug_at(1, supplies_of(P1, 10));
    step(&mut t);
    // (10, 8): the footprint clear of Tunnel 1's, (4..8, 8..12), its area (8..16) over Tunnel 1's (2..10).
    assert_eq!(ghost(&t, 2, at(10, 8)), overlap());
    assert_eq!(
        gate(&t, P1, build(&[2], at(10, 8), false)),
        refused(WholeProblem::TunnelAreaOverlap)
    );
    // (12, 8): its area (10..18) touches Tunnel 1's.
    assert!(ghost(&t, 3, at(12, 8)).is_valid());
    next(&mut t, P1, build(&[3], at(12, 8), false));
    assert_eq!(supplies(&t, P1), 9);
    // Tunnel 1 at Tier 2: its area is (1..11, 5..15).
    next_debug(&mut t, DebugCommand::SetTier { id: T1, tier: 2 });
    let problems = until_failed(&mut t, 3);
    assert_eq!(problems.whole, [WholeProblem::TunnelAreaOverlap]);
    assert_eq!(
        (supplies(&t, P1), ghosts(&t, P1), refunds(&t)),
        (10, vec![], vec![(id(3), 1)])
    );
    assert!(idle(&t, 3));
    assert_eq!(ghost(&t, 3, at(12, 8)), overlap());

    // Agent 2's site: its area (14..22, 6..14) counts as a Tunnel's.
    next(&mut t, P1, build(&[2], SITE, false));
    until_started(&mut t, 2);
    assert_eq!(ghost(&t, 3, at(20, 8)), overlap());
    assert!(ghost(&t, 3, at(24, 8)).is_valid());
}

/// Design change 0237: "Every player's Tunnels count"; "The ghost judges enemy Tunnels by what the player knows: their
/// remembered position and their tier as last seen ... The real check on arrival uses the truth, so a build can fail on
/// arrival because an enemy Tunnel was upgraded unseen". Player 2's Tunnel 2, on tiles (24, 8) to (27, 11), was seen
/// by player 1's scout, Agent 4, removed in tick 1: remembered at Tier 1, its area (22..30, 6..14). Then it goes to
/// Tier 2 unseen: its area is (21..31, 5..15). Agent 3's walk to [`SITE`] keeps out of its sight.
#[test]
fn an_enemy_tunnel_counts_as_last_seen_and_as_it_is_on_arrival() {
    let mut t = base()
        .spawn(TUNNEL, Some(P2), tile(24, 8)) // 2
        .spawn(AGENT, Some(P1), point(12, 16)) // 3
        .spawn(AGENT, Some(P1), point(26, 15)) // 4
        .start();
    t.without_auto_targeting()
        .debug_at(1, supplies_of(P1, 10))
        .debug_at(1, DebugCommand::Remove { id: id(4) });
    step(&mut t);
    run_until(&mut t, 2 + FOG_DELAY_TICKS);
    let sight = t.sim().vision().player(P1).expect("player 1's sight");
    let record = sight.remembered_record(id(2)).expect("player 1 remembers Tunnel 2");
    assert_eq!(record.tunnel.as_ref().map(|tunnel| tunnel.tier), Some(1));
    next_debug(&mut t, DebugCommand::SetTier { id: id(2), tier: 2 });

    // (17, 8): its area (15..23) over the area remembered: red, and refused.
    assert_eq!(ghost(&t, 3, at(17, 8)), overlap());
    assert_eq!(
        gate(&t, P1, build(&[3], at(17, 8), false)),
        refused(WholeProblem::TunnelAreaOverlap)
    );
    // [`SITE`]: its area (14..22) touches the area remembered, green, but overlaps Tier 2's.
    assert!(ghost(&t, 3, SITE).is_valid());
    next(&mut t, P1, build(&[3], SITE, false));
    assert_eq!(supplies(&t, P1), 9);
    let problems = until_failed(&mut t, 3);
    assert_eq!(problems.whole, [WholeProblem::TunnelAreaOverlap]);
    assert_eq!((supplies(&t, P1), ghosts(&t, P1), sites(&t)), (10, vec![], vec![]));
    assert!(idle(&t, 3));
}

/// Design change 0242's verification (`factions/syndicate/tunnel_area_and_space.md > Tunnel Area Non-Overlap Rule >
/// Drillable area`): "A Tunnel ghost whose 8x8 area touches Water is red"; a Tunnel 1 tile from the map's edge is red,
/// 2 tiles away valid (design change 0283: "a footprint 2 tiles from the edge is valid, and 1 tile is red"); and
/// "never-explored tiles pass on the ghost, and the real check happens on arrival". Water at (29, 9), in Agent 2's
/// sight, and at (35, 16), out of it.
#[test]
fn a_tunnels_area_lies_on_the_map_on_drillable_ground() {
    let mut rows: Vec<String> = (0..20).map(|_| ".".repeat(40)).collect();
    rows[9].replace_range(29..30, "~");
    rows[16].replace_range(35..36, "~");
    let mut t = syndicate(&rows.join("\n"))
        .spawn(TUNNEL, Some(P1), tile(4, 8)) // 1
        .spawn(AGENT, Some(P1), point(27, 12)) // 2
        .start();
    t.debug_at(1, supplies_of(P1, 10));
    step(&mut t);
    let only = |problem| PlacementCheck {
        tiles: Vec::new(),
        whole: vec![problem],
    };
    // (24, 8): its area (22..30, 6..14) takes in the Water at (29, 9).
    assert_eq!(ghost(&t, 2, at(24, 8)), only(WholeProblem::TunnelAreaNotDrillable));
    assert_eq!(
        gate(&t, P1, build(&[2], at(24, 8), false)),
        refused(WholeProblem::TunnelAreaNotDrillable)
    );
    // 1 tile from the north edge, and from the east edge: the area runs off the map. 2 tiles: valid.
    assert_eq!(ghost(&t, 2, at(20, 1)), only(WholeProblem::TunnelAreaOffMap));
    assert_eq!(ghost(&t, 2, at(35, 2)), only(WholeProblem::TunnelAreaOffMap));
    assert!(ghost(&t, 2, at(20, 2)).is_valid());
    assert!(ghost(&t, 2, at(34, 2)).is_valid());
    // (30, 12): its area (28..36, 10..18) takes in the Water at (35, 16), never explored: green, and refused on arrival.
    assert_eq!(t.sim().visibility(P1, at(35, 16)), sc_sim::VisibilityState::Unexplored);
    assert!(ghost(&t, 2, at(30, 12)).is_valid());
    next(&mut t, P1, build(&[2], at(30, 12), false));
    assert_eq!(supplies(&t, P1), 9);
    let problems = until_failed(&mut t, 2);
    assert_eq!(problems.whole, [WholeProblem::TunnelAreaNotDrillable]);
    assert_eq!((supplies(&t, P1), ghosts(&t, P1)), (10, vec![]));
}

/// Player 2's Syndicate base for [`hidden_expansions_0240`], player 1 playing `faction`, on a flat 24 × 20 map:
/// Tunnel 1 on tiles (12, 4) to (15, 7), and its Hideout 2 on (10, 6) to (11, 7), in the Tunnel's area (10..18,
/// 2..10), which only player 2 sees (plan decision T32).
fn hidden_base(faction: Faction) -> ScenarioBuilder {
    ScenarioBuilder::new(&flat(24, 20))
        .faction(P1, faction)
        .faction(P2, Faction::Syndicate)
        .spawn(TUNNEL, Some(P2), tile(12, 4)) // 1
        .spawn(ObjectKind::Hideout, Some(P2), tile(10, 6)) // 2
}

/// Design change 0240's verification (`factions/syndicate/structures/README.md > Tunnel Expansions`: "Surface
/// buildings over a hidden expansion: an enemy's placement is refused"): "A GDO player's ghost over a hidden Syndicate
/// Hideout is red" (a direct placement judges by the truth: `TileProblem::Structure`); "A Colonists Drone ordered to
/// deploy there fails on arrival" (its ghost green, by what the player knows); and an Agent's Tunnel ghost over it is
/// green, and fails on arrival (player 2's Tunnel, never seen, overlaps it too).
#[test]
fn hidden_expansions_0240() {
    let hideout = tiles(at(10, 6), 2, 2);
    let blocked = |check: &PlacementCheck| {
        (hideout.iter()).all(|&tile| check.problems_at(tile).any(|problem| problem == TileProblem::Structure))
    };

    let mut gdo = hidden_base(Faction::Gdo)
        .spawn(ObjectKind::DeploymentCenter, Some(P1), tile(4, 4)) // 3
        .start();
    gdo.step();
    assert!(
        !gdo.sim()
            .vision()
            .sees(P1, gdo.sim().object(id(2)).expect("the Hideout"))
    );
    let generator = gdo
        .sim()
        .placement_check(P1, ObjectKind::Generator, at(10, 6), Default::default());
    assert!(blocked(&generator), "{generator:?}");
    assert!(
        gdo.sim()
            .placement_check(P1, ObjectKind::Generator, at(6, 10), Default::default())
            .is_valid()
    );

    let mut colonists = hidden_base(Faction::Colonists)
        .spawn(ObjectKind::Drone, Some(P1), point(6, 12)) // 3
        .start();
    colonists.step();
    assert_eq!(
        colonists
            .sim()
            .deploy_check(id(3), at(10, 6), false)
            .map(|c| c.is_valid()),
        Some(true)
    );
    next(
        &mut colonists,
        P1,
        order(&[3], OrderKind::Deploy { origin: at(10, 6) }, false),
    );
    step_until(&mut colonists, WALK, "the Deploy fails", |t| {
        t.events()
            .iter()
            .any(|(_, e)| matches!(e, SimEvent::DeployFailed { .. }))
    });
    let deploy_failed = colonists.events().iter().find_map(|(_, e)| match e {
        SimEvent::DeployFailed { problems, .. } => Some(problems.clone()),
        _ => None,
    });
    assert!(deploy_failed.as_ref().is_some_and(blocked), "{deploy_failed:?}");

    let mut syndicate = hidden_base(Faction::Syndicate)
        .spawn(AGENT, Some(P1), point(4, 16)) // 3
        .start();
    syndicate.step();
    // (8, 6): the footprint (8..12, 6..10) over the Hideout, clear of Tunnel 1's; its area overlaps Tunnel 1's.
    assert!(ghost(&syndicate, 3, at(8, 6)).is_valid());
    next(&mut syndicate, P1, build(&[3], at(8, 6), false));
    let problems = until_failed(&mut syndicate, 3);
    assert!(blocked(&problems), "{problems:?}");
    assert_eq!(problems.whole, [WholeProblem::TunnelAreaOverlap]);
    assert_eq!(sites(&syndicate), []);
}

/// `core/entities.md > Placement Validation > Worker-Built Structures`, as for the Drone (design change 0118): the
/// check on arrival is the truth, and a ground unit standing on the footprint fails the order; the ghost goes, the
/// Agent idles, and the price comes back. Player 2's Guard 3 stands on [`SITE`], out of player 1's sight when the
/// order is given (auto-targeting off).
#[test]
fn a_unit_on_the_site_fails_the_order() {
    let mut t = base()
        .spawn(AGENT, Some(P1), point(12, 16)) // 2
        .spawn(GUARD, Some(P2), point(17, 9)) // 3
        .start();
    t.without_auto_targeting().debug_at(1, supplies_of(P1, 5));
    step(&mut t);
    assert!(ghost(&t, 2, SITE).is_valid());
    next(&mut t, P1, build(&[2], SITE, false));
    assert_eq!(supplies(&t, P1), 4);
    let problems = until_failed(&mut t, 2);
    assert!(
        problems.tiles.iter().any(|&(_, problem)| problem == TileProblem::Unit),
        "{problems:?}"
    );
    assert_eq!((supplies(&t, P1), ghosts(&t, P1), sites(&t)), (5, vec![], vec![]));
    assert_eq!(refunds(&t), [(id(2), 1)]);
    assert!(idle(&t, 2));
}

/// `core/entities.md > Placement Validation > Lingering build ghosts` (design change 0125): Agent 2's pending Build
/// reserves its site against its owner's further placements (`TileProblem::Reserved`) and, a Tunnel's, its whole
/// Tier 1 area against their further Tunnels (design change 0289 point 2, `WholeProblem::TunnelAreaOverlap`); it may
/// be touched. It is its owner's alone (`Sim::pending_builds`: "Visible to its owner. Enemies never see it"), and
/// reserves nothing against another player. A Build without Shift to Agent 2 itself replaces it, so it doesn't count
/// against that.
#[test]
fn a_lingering_ghost_reserves_its_site_and_area_for_its_owner_only() {
    let mut t = base()
        .spawn(AGENT, Some(P1), point(12, 16)) // 2
        .spawn(AGENT, Some(P1), point(13, 17)) // 3
        .spawn(AGENT, Some(P2), point(30, 16)) // 4
        .start();
    t.without_auto_targeting().debug_at(1, supplies_of(P1, 10));
    step(&mut t);
    next(&mut t, P1, build(&[2], SITE, false));
    assert_eq!(ghosts(&t, P1), [(id(2), SITE)]);
    assert_eq!(ghosts(&t, P2), []);

    let on_it = ghost(&t, 3, SITE);
    assert_eq!(on_it.whole, [WholeProblem::TunnelAreaOverlap]);
    for tile in tiles(SITE, 4, 4) {
        assert!(
            on_it.problems_at(tile).any(|problem| problem == TileProblem::Reserved),
            "{tile:?}"
        );
    }
    // (20, 8): the footprint clear of the site, the area (18..26) over its (14..22). (24, 8): touching.
    assert_eq!(ghost(&t, 3, at(20, 8)), overlap());
    assert!(ghost(&t, 3, at(24, 8)).is_valid());
    assert!(ghost(&t, 2, at(20, 8)).is_valid());
    assert!(ghost(&t, 4, SITE).is_valid());
}
