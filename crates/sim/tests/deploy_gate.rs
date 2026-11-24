//! Beacon Deployment's gate and arrival (slice M3-F; `sc_sim::deploy`): the Deploy order goes to one Drone of the
//! player's; the gate judges its site by what the player knows (design change 0120, `Mode::Known`); the arrival judges
//! it on the truth, and fails the order for a ground unit, a structure or a patch there (0118), or for a Drone that
//! stopped short of it (0117); a Beacon under deployment takes only Cancel (0116); the sites of pending Deploys
//! reserve their footprints against their owner's further placements and block no one (0125); and the gate, fuzzed.
//! `tests/deploy.rs` is the deployment itself. Walks are judged by their outcomes, never by their ticks (the Drone's
//! DragMovement sets their pace). `TestSim` checks the movement invariants after every tick.

use sc_sim::deploy::{self, on_site};
use sc_sim::testing::{ScenarioBuilder, TestSim, dangling_references, every_command};
use sc_sim::transform::warp_in_footprint;
use sc_sim::vision::FOG_DELAY_TICKS;
use sc_sim::{
    Command, ConstructionCommand, DebugCommand, Faction, MoveStatus, Object, ObjectId, ObjectKind, OrderKind,
    PlacementCheck, PlacementProblem, PlayerCommand, PlayerId, Reject, Replay, Rng, Sim, SimEvent, SpawnAt, Target,
    TickInputs, TilePos, TileProblem, Vec2, VisibilityState,
};

use ConstructionCommand::{CancelTransform, WarpIn};
use ObjectKind::{Beacon, CrystalPatch, Drone, EgressNode, Peacekeeper, RoboPortal};
use TileProblem::{NotBuildable, OffMap, Reserved, Resource, Structure, Unit};

const P1: PlayerId = PlayerId(1);
const P2: PlayerId = PlayerId(2);

/// A site in the sight of Drone 2 of [`base`]: tiles (8, 9) to (9, 10).
const SITE: TilePos = TilePos::new(8, 9);

fn flat(w: usize, h: usize) -> String {
    vec![".".repeat(w); h].join("\n")
}

fn at(x: i32, y: i32) -> TilePos {
    TilePos::new(x, y)
}

fn tile(x: i32, y: i32) -> SpawnAt {
    SpawnAt::Tile(at(x, y))
}

/// The centre of tile `(x, y)`.
fn point(x: i32, y: i32) -> SpawnAt {
    SpawnAt::Point(at(x, y).center())
}

fn id(n: u32) -> ObjectId {
    ObjectId(n)
}

fn object(t: &TestSim, n: u32) -> &Object {
    t.sim().object(id(n)).expect("the object exists")
}

/// A Deploy to `units` (`queue` is Shift).
fn deploy(units: &[u32], origin: TilePos, queue: bool) -> Command {
    order(units, OrderKind::Deploy { origin }, queue)
}

fn order(units: &[u32], kind: OrderKind, queue: bool) -> Command {
    Command::Order {
        units: units.iter().map(|&n| id(n)).collect(),
        kind,
        queue,
    }
}

/// Cancel Deployment to `objects`.
fn cancel(objects: &[u32]) -> Command {
    Command::Construction {
        objects: objects.iter().map(|&n| id(n)).collect(),
        command: CancelTransform,
    }
}

/// What the gate says to `player`'s `command` now (`Sim::validate`).
fn gate(t: &TestSim, player: PlayerId, command: Command) -> Result<(), Reject> {
    t.sim().validate(&PlayerCommand { player, command })
}

/// The gate's refusal of a site whose first problem is `problem` on tile `(x, y)`.
fn unplaceable(x: i32, y: i32, problem: TileProblem) -> Result<(), Reject> {
    Err(Reject::Unplaceable(PlacementProblem::Tile(at(x, y), problem)))
}

/// Schedules player 1's `command` for the next tick and runs it.
fn next(t: &mut TestSim, command: Command) -> &mut TestSim {
    let tick = t.sim().tick() + 1;
    t.at(tick, P1, command).step()
}

/// Unit `n`'s current order, if any.
fn current(t: &TestSim, n: u32) -> Option<OrderKind> {
    let orders = object(t, n).orders.as_ref()?;
    orders.current.as_ref().map(|active| active.order.kind)
}

/// Whether unit `n` stands idle: no order, current or queued, and no move under way.
fn idle(t: &TestSim, n: u32) -> bool {
    let o = object(t, n);
    o.orders.as_ref().is_some_and(|orders| orders.is_idle())
        && o.mover.as_ref().is_some_and(|m| m.status != MoveStatus::Moving)
}

/// Steps until Drone `n`'s Deploy is over, if it isn't yet: `Ok` with where the Drone stood when it began deploying
/// (at the end of its last tick as a Drone, which the arrival judges; where it is now, if that was before this call),
/// or `Err` with the problems its failed order reported. Panics after tick 900.
fn outcome(t: &mut TestSim, n: u32) -> Result<Vec2, PlacementCheck> {
    let mut stood = t.pos(n);
    loop {
        if object(t, n).kind != Drone {
            return Ok(stood);
        }
        let failed = t.events().iter().rev().find_map(|(_, e)| match e {
            SimEvent::DeployFailed { drone, problems, .. } if *drone == id(n) => Some(problems.clone()),
            _ => None,
        });
        if let Some(problems) = failed {
            return Err(problems);
        }
        assert!(t.sim().tick() < 900, "Drone {n} still on its way, at {}", t.pos(n));
        stood = t.pos(n);
        t.step();
    }
}

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

fn colonists(builder: ScenarioBuilder) -> ScenarioBuilder {
    builder.faction(P1, Faction::Colonists).faction(P2, Faction::Colonists)
}

/// `map` for two Colonists players: player 1's Egress Node 1 on (2, 2) to (3, 3) and its Drone 2 at the centre of
/// tile (5, 8), whose sight of 5 grid units covers [`SITE`]; player 2's Egress Node 3 on (36, 2) to (37, 3).
fn base_on(map: &str) -> ScenarioBuilder {
    colonists(ScenarioBuilder::new(map))
        .spawn(EgressNode, Some(P1), tile(2, 2))
        .spawn(Drone, Some(P1), point(5, 8))
        .spawn(EgressNode, Some(P2), tile(36, 2))
}

/// [`base_on`] a flat 40 × 16 map.
fn base() -> ScenarioBuilder {
    base_on(&flat(40, 16))
}

/// The plan's gate for a Deploy (`Sim::check_deploy`): the units named are the player's (`Reject::NotOwned`); the
/// Drones among them take it, exactly one ("a Drone becomes the Beacon, so each takes exactly one placement",
/// `systems/controls/selection_and_hud.md > Shift while placing`; the client picks the one Drone of a group, design
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
    assert_eq!(t.sim().pending_deploys(P1), [(id(2), deploy::site(SITE))]);
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
    assert!(t.sim().pending_deploys(P1).iter().all(|&(drone, _)| drone != id(2)));
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

/// What a fuzz run did.
#[derive(Debug, Default)]
struct Counts {
    accepted: usize,
    rejected: usize,
    /// Drones that began deploying.
    started: usize,
    /// `SimEvent::DeployFailed`.
    failed: usize,
    completed: usize,
    cancelled: usize,
}

/// The gate and the arrival, fuzzed: random Deploys, Shift or not, from any player (in the game or not) to any ids,
/// one Drone, several or none, with any origin, among moves, stops, cancels, new Drones and removals, never panic; a
/// rejected one changes nothing, events included. Each command runs alone in a tick, next to a copy of the game that
/// runs the same tick without it. A transforming object is always a Beacon deploying from a Drone on a 2 × 2 site,
/// with nothing but its HP; the pending Deploys are the players' own Drones'. Twice from the same seed, the runs are
/// identical, and the recorded commands replay to the same state.
#[test]
fn random_deploys_never_panic_and_rejections_change_nothing() {
    let (counts, hashes, replay) = deploy_fuzz(0xde9_10ce_u64);
    assert!(
        counts.accepted > 400 && counts.rejected > 1500 && counts.failed > 20,
        "{counts:?}"
    );
    assert!(
        counts.started > 60 && counts.completed > 10 && counts.cancelled > 25,
        "{counts:?}"
    );
    let (_, again, _) = deploy_fuzz(0xde9_10ce_u64);
    assert_eq!(hashes, again);
    let last = *hashes.last().unwrap();
    let file: Replay = serde_json::from_str(&serde_json::to_string(&replay).unwrap()).unwrap();
    assert_eq!(file.run().state_hash(), last);
}

fn random_command(rng: &mut Rng, sim: &Sim) -> PlayerCommand {
    let player = PlayerId([1, 1, 1, 2, 2, 2, 0, 3][rng.below(8) as usize]);
    let own: Vec<&Object> = sim.objects().filter(|o| o.owner == Some(player)).collect();
    let drones: Vec<ObjectId> = own.iter().filter(|o| o.kind == Drone).map(|o| o.id).collect();
    let deploying: Vec<ObjectId> = own.iter().filter(|o| o.is_deploying()).map(|o| o.id).collect();
    let any = |rng: &mut Rng| ObjectId(rng.below(40) + 1);
    let pick = |rng: &mut Rng, ids: &[ObjectId]| ids[rng.below(ids.len() as u32) as usize];
    let command = match rng.below(40) {
        0..2 => {
            let at = Vec2::new(rng.below(24 * 64) as f32, rng.below(16 * 64) as f32);
            return PlayerCommand {
                player: P1,
                command: Command::Debug(DebugCommand::Spawn {
                    kind: Drone,
                    owner: Some(PlayerId(1 + rng.below(2) as u8)),
                    at: SpawnAt::Point(at),
                    orientation: Default::default(),
                }),
            };
        }
        2 => {
            // Never the Egress Nodes 1 and 2, which keep both players in the game; often a Beacon under deployment.
            let deploying: Vec<ObjectId> = sim.objects().filter(|o| o.is_deploying()).map(|o| o.id).collect();
            let id = match rng.below(2) {
                0 if !deploying.is_empty() => pick(rng, &deploying),
                _ => ObjectId(rng.below(40) + 3),
            };
            return PlayerCommand {
                player: P1,
                command: Command::Debug(DebugCommand::Remove { id }),
            };
        }
        3..6 => {
            let objects = match rng.below(4) {
                _ if deploying.is_empty() => vec![any(rng)],
                0 => vec![any(rng)],
                _ => vec![pick(rng, &deploying)],
            };
            Command::Construction {
                objects,
                command: CancelTransform,
            }
        }
        6..9 => {
            let units = if drones.is_empty() {
                vec![any(rng)]
            } else {
                vec![pick(rng, &drones)]
            };
            let kind = match rng.below(2) {
                0 => OrderKind::Stop,
                _ => OrderKind::Move(Target::Point(Vec2::new(
                    rng.below(24 * 64) as f32,
                    rng.below(16 * 64) as f32,
                ))),
            };
            Command::Order {
                units,
                kind,
                queue: rng.below(3) == 0,
            }
        }
        _ => {
            let units = match rng.below(8) {
                _ if drones.is_empty() => vec![any(rng)],
                0 => drones.clone(),
                1 => vec![any(rng)],
                2 => own.iter().map(|o| o.id).collect(),
                _ => vec![pick(rng, &drones)],
            };
            let origin = match rng.below(12) {
                0 => at(rng.next_u32() as i32, rng.next_u32() as i32),
                _ => at(rng.below(27) as i32 - 2, rng.below(19) as i32 - 2),
            };
            Command::Order {
                units,
                kind: OrderKind::Deploy { origin },
                queue: rng.below(3) == 0,
            }
        }
    };
    PlayerCommand { player, command }
}

/// 3000 ticks of one random command each, on a 24 × 16 map with four Drones a player and a patch: what happened, the
/// state hash of every tick, and the replay.
fn deploy_fuzz(seed: u64) -> (Counts, Vec<u64>, Replay) {
    let mut builder = colonists(ScenarioBuilder::new(&flat(24, 16)))
        .spawn(EgressNode, Some(P1), tile(1, 1))
        .spawn(EgressNode, Some(P2), tile(21, 13))
        .spawn(CrystalPatch, None, tile(11, 7));
    for (x, y) in [(4, 4), (6, 9), (9, 3), (3, 12)] {
        builder = builder
            .spawn(Drone, Some(P1), point(x, y))
            .spawn(Drone, Some(P2), point(23 - x, 15 - y));
    }
    let mut t = builder.start();
    let mut rng = Rng::from_seed(seed);
    let mut counts = Counts::default();
    let mut hashes = Vec::new();
    for _ in 0..3000 {
        let cmd = random_command(&mut rng, t.sim());
        let verdict = t.sim().validate(&cmd);
        let mut without = t.sim().clone();
        without.step(&TickInputs::default());
        let tick = t.sim().tick() + 1;
        t.at(tick, cmd.player, cmd.command.clone()).step();
        match verdict {
            Ok(()) => counts.accepted += 1,
            Err(reason) => {
                counts.rejected += 1;
                assert_eq!(t.sim().state_hash(), without.state_hash(), "{cmd:?}: {reason}");
                assert_eq!(t.sim().events(), without.events(), "{cmd:?}: {reason}");
            }
        }
        for e in t.sim().events() {
            match e {
                SimEvent::KindChanged {
                    from: Drone,
                    to: Beacon,
                    ..
                } => counts.started += 1,
                SimEvent::DeployFailed { .. } => counts.failed += 1,
                SimEvent::TransformCompleted { .. } => counts.completed += 1,
                SimEvent::TransformCancelled { .. } => counts.cancelled += 1,
                _ => {}
            }
        }
        let sim = t.sim();
        for o in sim.objects().filter(|o| o.is_transforming()) {
            let deploying = o.transform.as_ref().unwrap();
            let site = o.placement.expect("a structure");
            assert_eq!(
                (
                    o.kind,
                    deploying.from,
                    deploying.from_placement,
                    deploying.total,
                    deploying.paid
                ),
                (Beacon, Drone, None, 48, 0),
                "#{}",
                o.id.0
            );
            assert_eq!((site.w, site.h, o.pos), (2, 2, site.center()), "#{}", o.id.0);
            assert!(o.orders.is_none() && o.mover.is_none() && o.worker.is_none() && o.production.is_none());
        }
        for player in [P1, P2] {
            for (drone, _) in sim.pending_deploys(player) {
                let o = sim.object(drone).expect("a pending Deploy's Drone exists");
                assert_eq!((o.kind, o.owner), (Drone, Some(player)));
            }
        }
        assert_eq!(dangling_references(sim), []);
        hashes.push(sim.state_hash());
    }
    (counts, hashes, t.replay().clone())
}
