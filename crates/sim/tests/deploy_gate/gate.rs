//! The gate of a Deploy (`Sim::check_deploy`): the order goes to one Drone of the player's, the site is judged
//! by what the player knows (design change 0120, `Mode::Known`), and a Beacon under deployment takes only
//! Cancel (0116).

use super::*;

/// Cancel Deployment to `objects`.
fn cancel(objects: &[u32]) -> Command {
    Command::Construction {
        objects: objects.iter().map(|&n| id(n)).collect(),
        command: CancelTransform,
    }
}

/// The plan's gate for a Deploy (`Sim::check_deploy`): the units named are the player's (`Reject::NotOwned`); the
/// Drones among them take it, exactly one ("a Drone becomes the Beacon, so each takes exactly one placement",
/// `systems/controls/interface.md > Shift while placing`; the client picks the one Drone of a group, design
/// change 0125): none is `Reject::NoWorkers`, several `Reject::SeveralConstructors`, and the others named keep what
/// they were doing; and the footprint is on the map, whatever the origin.
#[test]
fn a_deploy_goes_to_one_drone_of_the_players() {
    let mut t = base()
        .spawn(Drone, Some(P1), point(8, 4)) // 4
        .spawn(Peacekeeper, Some(P1), point(4, 12)) // 5
        .spawn(Drone, Some(P2), point(30, 4)) // 6
        .start();
    t.step();
    assert_eq!(gate(&t, P1, deploy(&[2], SITE, false)), Ok(()));
    assert_eq!(gate(&t, P1, deploy(&[5], SITE, false)), Err(Reject::NoWorkers));
    assert_eq!(
        gate(&t, P1, deploy(&[2, 4], SITE, true)),
        Err(Reject::SeveralConstructors)
    );
    assert_eq!(gate(&t, P1, deploy(&[2, 6], SITE, false)), Err(Reject::NotOwned(id(6))));
    assert_eq!(gate(&t, P2, deploy(&[2], SITE, false)), Err(Reject::NotOwned(id(2))));
    assert_eq!(
        gate(&t, P1, deploy(&[1], SITE, false)),
        Err(Reject::NoUnits),
        "a building"
    );
    assert_eq!(gate(&t, P1, deploy(&[], SITE, false)), Err(Reject::NoUnits));
    for (x, y) in [(39, 15), (-1, 4), (4, -2), (i32::MAX, i32::MAX), (i32::MIN, i32::MIN)] {
        let verdict = gate(&t, P1, deploy(&[2], at(x, y), false));
        assert!(
            matches!(verdict, Err(Reject::Unplaceable(PlacementProblem::Tile(_, OffMap)))),
            "({x}, {y}): {verdict:?}"
        );
    }
    // To a Peacekeeper and a Drone, and an id that is gone: the Drone takes it, the Peacekeeper stays as it was.
    next(&mut t, deploy(&[5, 2, 99], SITE, false));
    assert_eq!(current(&t, 2), Some(OrderKind::Deploy { origin: SITE }));
    assert!(idle(&t, 5));
    assert_eq!(pending_deploys(t.sim(), P1), [(id(2), deploy::site(SITE))]);
}

/// Design change 0120 ("The ghost judges by what the player knows"; the plan's `Mode::Known`): tiles never explored
/// pass; explored ground, remembered structures and resources, and the units the player sees count. Player 1's
/// scout, Drone 4, saw player 2's Egress Node 5 and Peacekeeper 6 at the start, and is removed in tick 1: then, once
/// the fog delay is over, the Egress Node is remembered in the fog, and the Peacekeeper hidden there. Player 2's
/// Egress Node 3 was never seen. Drone 2 sees Peacekeeper 7, patch 8 and the Mountain at (8, 6).
#[test]
fn the_gate_judges_a_site_by_what_the_player_knows() {
    let mut rows = vec![".".repeat(40); 16];
    rows[6].replace_range(8..9, "M");
    rows[8].replace_range(36..37, "M");
    let mut t = base_on(&rows.join("\n"))
        .spawn(Drone, Some(P1), point(26, 12)) // 4
        .spawn(EgressNode, Some(P2), tile(28, 12)) // 5
        .spawn(Peacekeeper, Some(P2), point(24, 12)) // 6
        .spawn(Peacekeeper, Some(P2), point(7, 8)) // 7
        .spawn(CrystalPatch, None, tile(5, 11)) // 8
        .start();
    t.without_auto_targeting()
        .debug_at(1, DebugCommand::Remove { id: id(4) })
        .step()
        .run(FOG_DELAY_TICKS);
    let seen = |t: &TestSim, x, y| t.sim().visibility(P1, at(x, y));
    assert_eq!(
        (seen(&t, 28, 12), seen(&t, 24, 12), seen(&t, 36, 2), seen(&t, 36, 8)),
        (
            VisibilityState::Explored,
            VisibilityState::Explored,
            VisibilityState::Unexplored,
            VisibilityState::Unexplored
        )
    );
    let verdict = |t: &TestSim, x, y| gate(t, P1, deploy(&[2], at(x, y), false));
    // "Ghost over a fogged area containing a remembered enemy building: red there."
    assert_eq!(verdict(&t, 27, 11), unplaceable(28, 12, Structure));
    // "A hidden enemy unit standing in fog on the site: ghost still green."
    assert_eq!(verdict(&t, 24, 12), Ok(()));
    // "Ghost over a never-explored area: green", whatever stands there or the ground is.
    assert_eq!(verdict(&t, 36, 2), Ok(()));
    assert_eq!(verdict(&t, 35, 7), Ok(()));
    // In sight: a unit, a building of the player's own, a patch, ground that isn't buildable.
    assert_eq!(verdict(&t, 7, 8), unplaceable(7, 8, Unit));
    assert_eq!(verdict(&t, 3, 3), unplaceable(3, 3, Structure));
    assert_eq!(verdict(&t, 4, 10), unplaceable(5, 11, Resource));
    assert_eq!(verdict(&t, 8, 5), unplaceable(8, 6, NotBuildable));
    // Open ground in sight, and the ground under the Drone itself, which doesn't count (design change 0118).
    assert_eq!(verdict(&t, 5, 5), Ok(()));
    assert_eq!(verdict(&t, 5, 8), Ok(()));
    // A Shift-queued Deploy is judged alike.
    assert_eq!(gate(&t, P1, deploy(&[2], at(7, 8), true)), unplaceable(7, 8, Unit));
    assert_eq!(gate(&t, P1, deploy(&[2], at(36, 2), true)), Ok(()));
}

/// Design change 0116 (`factions/colonists/economy.md > Beacon Deployment > Cancel Deployment`): "Only cancel is
/// available" on a Beacon under deployment. Each command of `every_command` but the debug ones, given to it, is
/// refused: the orders (it is a unit no more), the abilities, production and rallies, construction and Warp In; and
/// Cancel Deployment is taken. Cancelled, it is a Drone again and takes orders.
#[test]
fn a_beacon_under_deployment_takes_only_cancel() {
    let mut t = base().start();
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
    .step();
    next(&mut t, deploy(&[2], SITE, false));
    outcome(&mut t, 2).expect("it deploys");
    let to_it = vec![id(2)];
    let mut taken = Vec::new();
    for command in every_command() {
        let command = match command {
            Command::Order { kind, queue, .. } => Command::Order {
                units: to_it.clone(),
                kind,
                queue,
            },
            Command::Ability { ability, .. } => Command::Ability {
                units: to_it.clone(),
                ability,
            },
            Command::Production { command, .. } => Command::Production {
                objects: to_it.clone(),
                command,
            },
            Command::Construction { command, .. } => Command::Construction {
                objects: to_it.clone(),
                command,
            },
            Command::Tunnel { command, .. } => Command::Tunnel { tunnel: id(2), command },
            Command::Holding { command, .. } => Command::Holding {
                objects: to_it.clone(),
                command,
            },
            Command::Cast { ability, at, .. } => Command::Cast {
                casters: to_it.clone(),
                ability,
                at,
            },
            Command::Debug(_) => continue,
        };
        if gate(&t, P1, command.clone()).is_ok() {
            taken.push(command);
        }
    }
    assert_eq!(taken, [cancel(&[2])]);
    assert!(object(&t, 2).is_deploying());
    next(&mut t, cancel(&[2]));
    assert_eq!(object(&t, 2).kind, Drone);
    assert_eq!(gate(&t, P1, order(&[2], OrderKind::Stop, false)), Ok(()));
}
