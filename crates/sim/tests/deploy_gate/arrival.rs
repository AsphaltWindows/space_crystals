//! The arrival, which judges the site on the truth: a Deploy into never explored ground, a ground unit or a
//! patch on the site (design change 0118), a Drone walled off from its site or stopped while overlapping it
//! (0117).

use super::*;

/// The ticks' `SimEvent::DeployFailed` so far: the Drone and the origin of its site.
fn failures(t: &TestSim) -> Vec<(u32, TilePos)> {
    t.events()
        .iter()
        .filter_map(|(_, e)| match e {
            SimEvent::DeployFailed { drone, origin, .. } => Some((drone.0, *origin)),
            _ => None,
        })
        .collect()
}

/// The plan's acceptance, "Deploy into unexplored fog", and design change 0120 ("the Drone walks there and validates
/// on arrival"): Drone 2's Deploy over player 2's Egress Node 4, never seen, is accepted; on arrival the order fails
/// for the structure, the Drone idles, and its site is no longer pending. Drone 5's Deploy to never-explored open
/// ground deploys there.
#[test]
fn a_deploy_into_never_explored_ground_is_checked_on_arrival() {
    let mut t = base()
        .spawn(EgressNode, Some(P2), tile(30, 7)) // 4
        .spawn(Drone, Some(P1), point(5, 12)) // 5
        .start();
    t.step();
    for (x, y) in [(29, 7), (24, 12)] {
        assert_eq!(t.sim().visibility(P1, at(x, y)), VisibilityState::Unexplored);
    }
    // Its site: tiles (29, 7) to (30, 8), the Egress Node on (30, 7) to (31, 8).
    next(&mut t, deploy(&[2], at(29, 7), false));
    t.order_at(
        t.sim().tick() + 1,
        P1,
        &[id(5)],
        OrderKind::Deploy { origin: at(24, 12) },
        false,
    );
    let problems = outcome(&mut t, 2).expect_err("the order fails");
    assert_eq!(problems.tiles, [(at(30, 7), Structure), (at(30, 8), Structure)]);
    assert!(idle(&t, 2));
    assert_eq!(object(&t, 2).kind, Drone);
    assert!(pending_deploys(t.sim(), P1).iter().all(|&(drone, _)| drone != id(2)));
    assert_eq!(failures(&t), [(2, at(29, 7))]);
    outcome(&mut t, 5).expect("it deploys");
    assert_eq!(object(&t, 5).placement, Some(deploy::site(at(24, 12))));
}

/// Design change 0118 ("Park an enemy ground unit on a Deploy site: on arrival the order fails, Drone idles"; "a
/// ground unit of any player"), with 0120 ("A hidden enemy unit standing in fog on the site: ghost still green; the
/// order fails on arrival") and the plan's acceptance, "Occupied site": player 2's Peacekeeper 4 stands on a site
/// player 1 never saw, and player 1's own Peacekeeper 5 goes to stand on Drone 6's site, holding there, after its
/// Deploy was accepted. Both orders fail on arrival for the unit; both Drones idle. Auto-targeting is off.
#[test]
fn a_ground_unit_on_the_site_at_arrival_fails_the_order() {
    let enemy_site = at(20, 7);
    let own_site = at(12, 13);
    let mut t = base()
        .spawn(Peacekeeper, Some(P2), SpawnAt::Point(deploy::site(enemy_site).center())) // 4
        .spawn(Peacekeeper, Some(P1), point(16, 14)) // 5
        .spawn(Drone, Some(P1), point(1, 14)) // 6
        .start();
    t.without_auto_targeting().step();
    assert_eq!(gate(&t, P1, deploy(&[2], enemy_site, false)), Ok(()));
    next(&mut t, deploy(&[2], enemy_site, false));
    let tick = t.sim().tick() + 1;
    let centre = deploy::site(own_site).center();
    t.order_at(tick, P1, &[id(6)], OrderKind::Deploy { origin: own_site }, false)
        .order_at(tick, P1, &[id(5)], OrderKind::Move(Target::Point(centre)), false)
        .order_at(tick, P1, &[id(5)], OrderKind::Hold, true);
    let problems = outcome(&mut t, 2).expect_err("the order fails");
    assert!(
        !problems.tiles.is_empty() && problems.tiles.iter().all(|&(_, problem)| problem == Unit),
        "{problems:?}"
    );
    let problems = outcome(&mut t, 6).expect_err("the order fails");
    assert!(problems.tiles.contains(&(at(12, 13), Unit)), "{problems:?}");
    assert!(idle(&t, 2) && idle(&t, 6));
    assert_eq!(current(&t, 5), Some(OrderKind::Hold), "it held");
}

/// The plan's acceptance, "Occupied site": "A patch refuses it at the gate if the player knows it, or on arrival if
/// not". Patch 4 at (5, 11) is in Drone 2's sight, patch 5 at (25, 8) never seen.
#[test]
fn a_patch_on_the_site_refuses_it_at_the_gate_if_known_else_on_arrival() {
    let mut t = base()
        .spawn(CrystalPatch, None, tile(5, 11)) // 4
        .spawn(CrystalPatch, None, tile(25, 8)) // 5
        .start();
    t.step();
    assert_eq!(
        gate(&t, P1, deploy(&[2], at(5, 10), false)),
        unplaceable(5, 11, Resource)
    );
    next(&mut t, deploy(&[2], at(24, 7), false));
    let problems = outcome(&mut t, 2).expect_err("the order fails");
    assert_eq!(problems.tiles, [(at(25, 8), Resource)]);
    assert!(idle(&t, 2));
}

/// Design change 0117 ("Wall the Drone off before it reaches the footprint: order fails, Drone idles"; "if it stops
/// without overlapping the footprint, the order fails"): a Mountain range at column 8 runs from top to bottom between
/// Drone 2 and its site. The Drone goes as near as it can and stops there, off the site: the order fails with no
/// problem found on the site, and the Drone idles.
#[test]
fn a_drone_walled_off_from_its_site_fails_and_idles() {
    let row = format!("{}M{}", ".".repeat(8), ".".repeat(31));
    let mut t = base_on(&vec![row; 16].join("\n")).start();
    t.step();
    let site = deploy::site(at(12, 8));
    assert_eq!(gate(&t, P1, deploy(&[2], site.origin, false)), Ok(()));
    next(&mut t, deploy(&[2], site.origin, false));
    let problems = outcome(&mut t, 2).expect_err("the order fails");
    assert_eq!(problems, PlacementCheck::default(), "it stopped short");
    assert!(t.pos(2).x < 8.0 * 64.0 && !on_site(Drone, t.pos(2), &site));
    assert!(idle(&t, 2));
    assert_eq!(failures(&t), [(2, site.origin)]);
    t.run(30);
    assert!(idle(&t, 2), "it stays idle");
}

/// Design change 0117: "If it is stopped by a blocker on the way while already overlapping the footprint, it starts
/// deploying from where it stands"; "the Beacon always appears on the footprint the ghost showed". Its Verification:
/// "Block the approach with a unit just outside the footprint while the Drone's edge overlaps it: it deploys." The
/// site is in the bottom-right corner of the map; player 2's Peacekeepers 4 to 7 line its top edge and 8 to 11 its
/// left edge, just outside it, their boxes touching it; Drone 2 stands in the corner they leave, overlapping its
/// top-left tile by 8 × 8 space units. It can't get on, and deploys from there, onto the site. Auto-targeting is off.
#[test]
fn a_drone_stopped_while_overlapping_its_site_deploys_from_there() {
    let site = deploy::site(at(14, 10));
    // The site spans x 896 to 1024 and y 640 to 768; a Peacekeeper's box is 24 across, a Drone's 36.
    let start = Vec2::new(886.0, 630.0);
    let mut builder = colonists(ScenarioBuilder::new(&flat(16, 12)))
        .spawn(EgressNode, Some(P1), tile(1, 1))
        .spawn(Drone, Some(P1), SpawnAt::Point(start))
        .spawn(EgressNode, Some(P2), tile(1, 8));
    for x in [917.0, 942.0, 967.0, 992.0] {
        builder = builder.spawn(Peacekeeper, Some(P2), SpawnAt::Point(Vec2::new(x, 628.0)));
    }
    for y in [661.0, 686.0, 711.0, 736.0] {
        builder = builder.spawn(Peacekeeper, Some(P2), SpawnAt::Point(Vec2::new(884.0, y)));
    }
    let mut t = builder.start();
    t.without_auto_targeting().step();
    assert!(on_site(Drone, start, &site));
    next(&mut t, deploy(&[2], site.origin, false));
    let stood = outcome(&mut t, 2).expect("it deploys");
    assert!(on_site(Drone, stood, &site), "it stood at {stood}");
    assert!(stood.distance(start) < 8.0, "it got no further than {stood}");
    let o = object(&t, 2);
    assert_eq!((o.kind, o.placement, o.pos), (Beacon, Some(site), site.center()));
    assert!(o.is_deploying());
}
