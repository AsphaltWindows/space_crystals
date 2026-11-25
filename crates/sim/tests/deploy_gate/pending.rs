//! Pending Deploys (design change 0125, `core/entities.md > Placement Validation > Lingering build ghosts`):
//! their sites reserve their footprints against their owner's further placements, and block no one.

use super::*;

/// Design change 0125, "Lingering build ghosts" (`core/entities.md > Placement Validation`): every Deploy ordered,
/// current or Shift-queued, is pending until its deployment starts (`Sim::pending_deploys`, derived from the orders),
/// and its site is reserved against its owner's further placements: a Deploy (`TileProblem::Reserved`) and a Warp In
/// alike. A Deploy without Shift replaces the Drone's orders, so its own sites don't count against it; another
/// player's sites never count. A site is free again once its order is gone. Player 1's Beacon 5 would warp a Robo
/// Portal in on (11, 1) to (14, 4); Drone 4's site (14, 4) overlaps its corner.
#[test]
fn pending_deploys_reserve_their_sites_against_their_owners_placements() {
    let (s1, s2, s3) = (at(9, 9), at(14, 9), at(14, 4));
    let mut t = colonists(ScenarioBuilder::new(&flat(32, 16)))
        .spawn(EgressNode, Some(P1), tile(2, 2))
        .spawn(Drone, Some(P1), point(6, 8))
        .spawn(EgressNode, Some(P2), tile(28, 12))
        .spawn(Drone, Some(P1), point(8, 12)) // 4
        .spawn(Beacon, Some(P1), tile(12, 2)) // 5
        .spawn(Drone, Some(P2), point(10, 13)) // 6
        .start();
    let robo_portal = warp_in_footprint(&object(&t, 5).placement.unwrap(), RoboPortal, Default::default()).unwrap();
    assert_eq!((robo_portal.origin, robo_portal.w, robo_portal.h), (at(11, 1), 4, 4));
    let warp_in = Command::Construction {
        objects: vec![id(5)],
        command: WarpIn {
            kind: RoboPortal,
            orientation: Default::default(),
        },
    };
    t.debug_at(
        1,
        DebugCommand::SetResources {
            player: P1,
            crystals: 1000,
            supplies: 0,
            alloys: 0,
            essence: 0,
            flux: 0,
        },
    )
    .at(1, P1, deploy(&[2], s1, false))
    .at(1, P1, deploy(&[2], s2, true))
    .at(1, P1, deploy(&[4], s3, false))
    .step();
    let site = deploy::site;
    assert_eq!(
        t.sim().pending_deploys(P1),
        [(id(2), site(s1)), (id(2), site(s2)), (id(4), site(s3))]
    );
    assert_eq!(t.sim().pending_deploys(P2), []);
    // Another Drone's sites, current and queued, and a Drone's own with Shift.
    assert_eq!(
        gate(&t, P1, deploy(&[4], at(10, 10), false)),
        unplaceable(10, 10, Reserved)
    );
    assert_eq!(
        gate(&t, P1, deploy(&[4], at(15, 10), true)),
        unplaceable(15, 10, Reserved)
    );
    assert_eq!(
        gate(&t, P1, deploy(&[2], at(10, 10), true)),
        unplaceable(10, 10, Reserved)
    );
    assert_eq!(
        gate(&t, P1, deploy(&[4], at(14, 5), true)),
        unplaceable(14, 5, Reserved)
    );
    // Without Shift, a Drone's own sites are replaced.
    assert_eq!(gate(&t, P1, deploy(&[2], at(10, 10), false)), Ok(()));
    assert_eq!(gate(&t, P1, deploy(&[4], at(14, 5), false)), Ok(()));
    // Player 2 sees the site, and player 1's sites don't count for it.
    assert_eq!(t.sim().visibility(P2, at(10, 10)), VisibilityState::Visible);
    assert_eq!(gate(&t, P2, deploy(&[6], s1, false)), Ok(()));
    // A Warp In: refused, and reported as it applies, with the reserved tile.
    assert_eq!(gate(&t, P1, warp_in.clone()), unplaceable(14, 4, Reserved));
    next(&mut t, warp_in.clone());
    let refused: Vec<&PlacementCheck> = t
        .sim()
        .events()
        .iter()
        .filter_map(|e| match e {
            SimEvent::PlacementRefused { problems, .. } => Some(problems),
            _ => None,
        })
        .collect();
    assert_eq!(refused.len(), 1);
    assert_eq!(refused[0].tiles, [(at(14, 4), Reserved)]);
    assert!(!object(&t, 5).is_transforming());
    // New orders in their place: nothing is pending, and the sites are free.
    let tick = t.sim().tick() + 1;
    t.order_at(
        tick,
        P1,
        &[id(2)],
        OrderKind::Move(Target::Point(at(4, 12).center())),
        false,
    )
    .order_at(tick, P1, &[id(4)], OrderKind::Stop, false)
    .step();
    assert_eq!(t.sim().pending_deploys(P1), []);
    assert_eq!(gate(&t, P1, deploy(&[4], at(10, 10), false)), Ok(()));
    assert_eq!(gate(&t, P1, warp_in), Ok(()));
}

/// Design change 0125: a pending site "does not block units" ("Walk a unit over a ghost: not blocked"). Player 1's
/// Peacekeeper 4 crosses Drone 2's far site, which is pending all the while, and gets where it was going; a unit can
/// be put on any of its tiles. The Drone then deploys there.
#[test]
fn pending_sites_block_no_one() {
    let origin = at(30, 8);
    let site = deploy::site(origin);
    let mut t = base_on(&flat(40, 12))
        .spawn(Peacekeeper, Some(P1), point(26, 8))
        .start();
    let goal = at(36, 8).center();
    t.step();
    next(&mut t, deploy(&[2], origin, false));
    for tile in site.tiles() {
        let spawn = DebugCommand::Spawn {
            kind: Peacekeeper,
            owner: Some(P2),
            at: SpawnAt::Point(tile.center()),
            orientation: Default::default(),
        };
        assert_eq!(gate(&t, P1, Command::Debug(spawn)), Ok(()), "{tile:?}");
    }
    let tick = t.sim().tick() + 1;
    t.order_at(tick, P1, &[id(4)], OrderKind::Move(Target::Point(goal)), false);
    let mut crossed = false;
    while t.pos(4) != goal {
        assert!(t.sim().tick() < 200, "the Peacekeeper gets there");
        assert_eq!(t.sim().pending_deploys(P1), [(id(2), site)]);
        // A Peacekeeper's box is 24 across.
        crossed |= TilePos::new(30, 8).corner().x < t.pos(4).x + 12.0 && t.pos(4).x - 12.0 < 32.0 * 64.0;
        t.step();
    }
    assert!(crossed, "it crossed the site");
    outcome(&mut t, 2).expect("it deploys");
    assert_eq!(object(&t, 2).placement, Some(site));
}
