//! A Tunnel's Laser Turret (`factions/syndicate/structures/laser_turret.md > Construction`: "**By a Tunnel**: from the
//! Tunnel's ExpandMenu, placed within that Tunnel's Tunnel Area like an expansion. It **uses the Tunnel's single
//! operation slot**"; design changes 0267 and 0297), the only one there is ("**only Tunnels build turrets**", design
//! change 0359), and what a turret takes of Tunnel Space (`> Pop-up turret`: "¼ Tunnel Space, subject to the rounding
//! rule").

use sc_sim::tunnel::Operation;
use sc_sim::{ConstructionCommand, PlacementProblem, TileProblem, WholeProblem};

use super::*;

/// "Build time: 320 frames (20 s)".
const FRAMES: u32 = 320;

/// [`base`] started, player 1 with `crystals` Space Crystals from tick 1, which has run.
fn started(scenario: ScenarioBuilder, crystals: u32) -> TestSim {
    let mut t = scenario.start();
    t.debug_at(1, set_crystals(P1, crystals));
    t.run_until(1);
    t
}

/// The site [`T1`] builds; panics if it builds none.
fn building(t: &TestSim) -> ObjectId {
    match object(t, T1).tunnel.as_ref().and_then(|state| state.operation) {
        Some(Operation::Building(site)) => site,
        other => panic!("the Tunnel builds nothing: {other:?}"),
    }
}

/// `factions/syndicate/tunnels.md > ExpandMenu`: "The ExpandMenu also offers the **Laser Turret** (once the player
/// owns an Optics Lab)"; "Cost: 100 Space Crystals (no Supplies)". The gate's answers short of the placement
/// (`Sim::can_expand`, which the menu greys its tenth slot by), and the placement's: inside the Tunnel's area, on
/// ground that is "**Buildable, flat and Drillable**" (design change 0358 point 3), clear of every
/// structure, a unit over it or not.
#[test]
fn the_tunnel_builds_it_behind_an_optics_lab_inside_its_area() {
    // No Optics Lab: the Tunnel alone.
    let alone = ScenarioBuilder::new(&vec![".".repeat(40); 20].join("\n"))
        .faction(P1, Faction::Syndicate)
        .faction(P2, Faction::Syndicate)
        .spawn(TUNNEL, Some(P1), tile(4, 8));
    let t = started(alone, 1000);
    assert_eq!(
        t.sim().can_expand(P1, T1, TURRET),
        Err(Reject::Requires(ObjectKind::OpticsLab))
    );
    let t = started(base(), 99);
    assert_eq!(t.sim().can_expand(P1, T1, TURRET), Err(Reject::InsufficientCrystals));
    // A slope at x = 9 (the level rises there) and a Cliff tile, Drillable but not Buildable, at (8, 13).
    let mut rows = vec![".".repeat(40); 20];
    rows[13].replace_range(8..9, "c");
    // The corner heights, one more each way than the tiles: level 0 up to x = 9, level 1 from x = 10.
    let heights = vec![format!("{}{}", "0".repeat(10), "1".repeat(31)); 21];
    let ascii = format!("{}\n\n{}", rows.join("\n"), heights.join("\n"));
    let mut t = started(
        ScenarioBuilder::new(&ascii)
            .faction(P1, Faction::Syndicate)
            .faction(P2, Faction::Syndicate)
            .spawn(TUNNEL, Some(P1), tile(4, 8))
            .spawn(ObjectKind::OpticsLab, Some(P1), tile(2, 6)),
        1000,
    );
    assert_eq!(t.sim().can_expand(P1, T1, TURRET), Ok(()));
    let refused = |t: &TestSim, x: i32, y: i32| match gate(t, P1, expand(x, y)) {
        Err(Reject::Unplaceable(problem)) => problem,
        other => panic!("({x}, {y}): {other:?}"),
    };
    assert_eq!(
        refused(&t, 10, 10),
        PlacementProblem::Whole(WholeProblem::OutsideTunnelArea)
    );
    assert_eq!(
        refused(&t, 2, 6),
        PlacementProblem::Tile(at(2, 6), TileProblem::Structure),
        "the Optics Lab"
    );
    assert_eq!(
        refused(&t, 5, 9),
        PlacementProblem::Tile(at(5, 9), TileProblem::Structure),
        "the Tunnel"
    );
    assert!(!t.sim().map().is_flat(at(9, 10)), "the slope");
    assert_eq!(
        refused(&t, 9, 10),
        PlacementProblem::Tile(at(9, 10), TileProblem::NotFlat)
    );
    assert_eq!(
        refused(&t, 8, 13),
        PlacementProblem::Tile(at(8, 13), TileProblem::NotBuildable)
    );
    // A unit over the tile doesn't block it (design change 0290): the turret starts underground.
    next_debug(&mut t, spawn(AGENT, P1, at(8, 8).center()));
    assert_eq!(gate(&t, P1, expand(8, 8)), Ok(()));
    next(&mut t, P1, expand(8, 8));
    let site = building(&t);
    assert_eq!(object(&t, site).popup, Some(PopUp::LOWERED));
    assert!(
        !t.sim().nav().grids()[0].is_blocked(at(8, 8)),
        "its site blocks nothing"
    );
}

/// Plan §3.3 S2's acceptance: "A turret from the ExpandMenu inside the area: 100 SC at placement, 320 ticks", and
/// design change 0267's verification: "While a Tunnel builds a Laser Turret, its Expand and Upgrade are unavailable."
/// The site is the Tunnel's one operation, underground and hidden from its first frame (`> Construction`: "The **site
/// is hidden** like an expansion under construction"), at the ConstructionHP rule's 10%; it completes at the end of
/// its 320th tick, a lowered turret, active in its Tunnel's area, and the slot is free again.
#[test]
fn it_takes_the_operation_slot_100_crystals_and_320_frames() {
    let mut t = started(base().spawn(GUARD, Some(P2), SpawnAt::Point(at(11, 8).center())), 1000);
    t.at(2, P2, order(ObjectId(3), OrderKind::Hold));
    t.run_until(10);
    next(&mut t, P1, expand(8, 8));
    let (placed, site) = (t.sim().tick(), building(&t));
    assert_eq!(crystals(&t, P1), 900, "paid when placed, like an expansion");
    let o = object(&t, site);
    assert_eq!((o.kind, o.owner, o.is_site()), (TURRET, Some(P1), true));
    assert_eq!((o.popup, o.hp), (Some(PopUp::LOWERED), Some(30.0)));
    // 0267: the slot is taken.
    assert_eq!(gate(&t, P1, expand(8, 9)), Err(Reject::OperationUnderWay(T1)));
    assert_eq!(
        t.sim().can_expand(P1, T1, ObjectKind::Hideout),
        Err(Reject::OperationUnderWay(T1))
    );
    let upgrade = Command::Tunnel {
        tunnel: T1,
        command: TunnelCommand::Upgrade,
    };
    assert_eq!(gate(&t, P1, upgrade.clone()), Err(Reject::OperationUnderWay(T1)));
    // Hidden from player 2, whose Guard sees its tile, to the end; it never fires and nothing fires at it.
    for tick in placed..placed + FRAMES {
        assert_eq!(t.sim().tick(), tick);
        let o = object(&t, site);
        assert!(o.is_site(), "tick {tick}");
        assert!(t.sim().hidden_from(o, P2) && !t.sim().is_visible(P2, o));
        t.step();
    }
    let o = object(&t, site);
    assert!(!o.is_site(), "complete at the end of tick {}", placed + FRAMES);
    assert_eq!((o.popup, o.hp), (Some(PopUp::LOWERED), Some(300.0)));
    assert!(t.sim().is_active_turret(o));
    assert_eq!(object(&t, T1).tunnel.as_ref().and_then(|state| state.operation), None);
    assert_eq!(gate(&t, P1, expand(8, 9)), Ok(()));
    // Supplies aside, the Upgrade is no longer refused for the slot.
    assert_ne!(gate(&t, P1, upgrade), Err(Reject::OperationUnderWay(T1)));
    // Complete, it rises at the Guard, 3 tiles away.
    t.run(9);
    assert!(matches!(popup(&t, site), Some(PopUp::Raised { .. })));
    assert_eq!(crystals(&t, P1), 900);
}

/// `> Construction`: "**Cancelling** the site refunds **75%** (like an expansion)": by the Tunnel's X, and by the
/// site's own Cancel Construction; the slot is free again.
#[test]
fn cancelling_its_site_refunds_75() {
    let mut t = started(base(), 1000);
    next(&mut t, P1, expand(8, 8));
    let cancel = Command::Tunnel {
        tunnel: T1,
        command: TunnelCommand::Cancel,
    };
    t.run(50);
    next(&mut t, P1, cancel);
    assert_eq!(crystals(&t, P1), 975);
    assert_eq!(object(&t, T1).tunnel.as_ref().and_then(|state| state.operation), None);
    next(&mut t, P1, expand(8, 8));
    let site = building(&t);
    let cancel = Command::Construction {
        objects: vec![site],
        command: ConstructionCommand::CancelConstruction,
    };
    next(&mut t, P1, cancel);
    assert_eq!(crystals(&t, P1), 950);
    assert!(t.sim().object(site).is_none());
    assert_eq!(gate(&t, P1, expand(8, 8)), Ok(()));
}

/// `factions/syndicate/structures/laser_turret.md > Pop-up turret`: a turret "Occupies 1 grid unit of that Tunnel's
/// tunnel-region, like an expansion, so it reduces Tunnel Space (¼ Tunnel Space, subject to the rounding rule in Tunnel
/// Space)"; `factions/syndicate/tunnel_area_and_space.md > Tunnel Space`: "structures smaller than 2x2 ... only lower
/// Tunnel Space when they push the unused total below the next multiple of 4". Plan §3.3 S2's acceptance: "Four
/// pop-ups take 1."
///
/// The Tunnel's 48 cells less the Optics Lab's 4 leave 44, Tunnel Space 11. A Tunnel's turret takes its cell as its
/// site starts, like an expansion (design changes 0254 and 0358 point 5: "**A turret site takes its Tunnel Space from
/// the start of construction**"): 43 cells, 10. Three more turrets: 40 cells, still 10: four took 1. A fifth: 39, 9. A
/// turret outside the area (a debug placement) is inactive and takes nothing.
#[test]
fn four_popups_take_one_tunnel_space() {
    let mut t = started(base(), 1000);
    assert_eq!(tunnel_space(&t, P1), (0, 11));
    next(&mut t, P1, expand(8, 8));
    assert_eq!(tunnel_space(&t, P1), (0, 10), "from the site's first frame");
    let turret = |x: i32, y: i32| DebugCommand::Spawn {
        kind: TURRET,
        owner: Some(P1),
        at: tile(x, y),
        orientation: Default::default(),
    };
    for (x, y) in [(9, 8), (9, 9)] {
        next_debug(&mut t, turret(x, y));
        assert_eq!(tunnel_space(&t, P1), (0, 10));
    }
    next_debug(&mut t, turret(9, 10));
    assert_eq!(tunnel_space(&t, P1), (0, 10), "four pop-ups: 40 cells");
    next_debug(&mut t, turret(9, 11));
    assert_eq!(tunnel_space(&t, P1), (0, 9), "the fifth: 39 cells");
    next_debug(&mut t, turret(12, 8));
    assert_eq!(tunnel_space(&t, P1), (0, 9), "a turret outside the area costs no Tunnel Space");
    assert!(!t.sim().is_active_turret(object(&t, newest(&t))));
}

/// Design change 0359 (`> Construction`: "**only Tunnels build turrets**"): an Agent builds Tunnels alone. Player 1's
/// Agent, with an Optics Lab and Space Crystals enough, has no turret to build: the gate refuses its Build of one, it
/// has no ghost of one, and a turret has no Build price; the Tunnel's ExpandMenu takes it.
#[test]
fn no_agent_builds_a_turret() {
    let t = started(base().spawn(AGENT, Some(P1), SpawnAt::Point(at(12, 12).center())), 1000);
    let (agent, origin, orientation) = (ObjectId(3), at(14, 10), BuildingOrientation::default());
    assert_eq!(object(&t, agent).kind, AGENT);
    let build = OrderKind::Build {
        kind: TURRET,
        origin,
        orientation,
    };
    assert_eq!(gate(&t, P1, order(agent, build)), Err(Reject::NoWorkers));
    assert!(t.sim().build_check(agent, TURRET, origin, orientation, false).is_none());
    assert_eq!(t.sim().build_price(P1, TURRET), None);
    assert_eq!(gate(&t, P1, expand(8, 8)), Ok(()));
}
