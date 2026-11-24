//! A Drone's Deploy from the command panel (`factions/colonists/units.md > Drone ObjectInterfaceState > A: Deploy`,
//! design changes 0120 and 0125): A enters AwaitingPlacement of a Beacon, whose ghost follows the cursor snapped to the
//! grid, green or red as far as the player knows (`Sim::deploy_check`, Reserved over a lingering ghost included); a
//! click sends one Drone of the group to deploy there, a plain one the first, a Shift-click the first with no Deploy
//! yet, placement chaining through the group while Shift is held; every Deploy ordered lingers as a ghost of its own
//! player's until the deployment starts, or the order is gone (`core/entities.md > Placement Validation > Lingering
//! build ghosts`). A Drone that deploys stays selected as its Beacon, which shows only X Cancel (design change 0116).

use sc_sim::{TileProblem, deploy};
use space_crystals::command_ui::{Builder, PanelCommand};

use super::*;
use KeyCode::{Escape, KeyA, KeyC, KeyQ, KeyX, KeyZ, ShiftLeft, Tab};

/// The Beacon's footprint.
const SIZE: (u32, u32) = (2, 2);
/// Four sites for a Beacon in the world view, a tile apart, far enough from the Drones that none of them gets to its
/// own while the tests click (over 10 tiles away, at most 8 space units a frame).
const SITES: [TilePos; 4] = [
    TilePos::new(21, 9),
    TilePos::new(24, 9),
    TilePos::new(27, 9),
    TilePos::new(30, 9),
];

/// Player 1's Drone `id` of [`drone_game`] (3 to 6), at the centre of tile (14, 17), (17, 17), (20, 17) or (23, 17):
/// where it stands, and is drawn, before it moves.
fn drone_at(id: u32) -> Vec2 {
    TilePos::new(14 + 3 * (id as i32 - 3), 17).center()
}

/// A [`colonists_game`] with player 1's four Drones 3 to 6 ([`drone_at`]) and player 2's Drone 7 in the world view
/// at the centre of tile (34, 17).
fn drone_game() -> App {
    colonists_game(0, |mut builder| {
        for id in 3..=6 {
            builder = builder.spawn(ObjectKind::Drone, Some(P1), SpawnAt::Point(drone_at(id)));
        }
        builder.spawn(
            ObjectKind::Drone,
            Some(P2),
            SpawnAt::Point(TilePos::new(34, 17).center()),
        )
    })
}

/// Selects player 1's Drones `ids` where they stand ([`drone_at`]), with a box around them.
fn select_drones(app: &mut App, ids: &[u32]) {
    let points: Vec<Vec2> = ids.iter().map(|&id| drone_at(id)).collect();
    select_around(app, &points, ids);
}

/// A Deploy into the Beacon whose top-left tile is `origin`.
fn deploy_to(origin: TilePos) -> OrderKind {
    OrderKind::Deploy { origin }
}

/// The Drones' placement, as Shift-clicks leave it while a Drone of the group has no Deploy yet.
fn queued() -> Placing {
    Placing {
        builder: Builder::Drones { queued: true },
        ..Placing::deploy()
    }
}

/// `> Drone ObjectInterfaceState > DefaultState commands`: Q Move, A Deploy, X Stop (S, Set Drop-off, is slice
/// M3-J2's). A enters AwaitingPlacement for a Beacon, the placement keys on and no pointer over the map: the ghost,
/// 2 × 2, follows the cursor snapped to the grid, the footprint whose centre is nearest it, and shows nothing over the
/// HUD. A click sends the Drone a Deploy there ("Left-click on a valid location issues the Deploy order"), and back to
/// DefaultState; its site lingers as a ghost.
#[test]
fn a_drone_deploys_where_its_ghost_snaps_under_the_cursor() {
    use ButtonLook::{Back, Common, Disabled, Pending};
    use PanelCommand::{Deploy, Move, Stop};
    let mut app = drone_game();
    click(&mut app, drone_at(3), &[]);
    assert_eq!(selected(&app), [3]);
    let c = PanelButton::Command;
    let x = CommandSlot::CANCEL_PRODUCTION;
    assert_eq!(
        buttons(&mut app),
        [
            (slot(0, 0), c(Move), Common, "Move"),
            (slot(1, 0), c(Deploy), Common, "Deploy"),
            (x, c(Stop), Common, "Stop"),
        ]
    );
    tap(&mut app, &[], KeyA);
    assert_eq!(placing(&app), Some(Placing::deploy()));
    assert_eq!(
        buttons(&mut app),
        [
            (slot(0, 0), c(Move), Disabled, "Move"),
            (slot(1, 0), c(Deploy), Pending, "Deploy"),
            (CommandSlot::BACK, PanelButton::Back, Back, "Back"),
            (x, c(Stop), Disabled, "Stop"),
        ]
    );
    hover(&mut app, middle(SITES[0], SIZE));
    let shown = ghost(&app).expect("a ghost");
    assert_eq!(
        (shown.kind, shown.origin, shown.size),
        (ObjectKind::Beacon, SITES[0], SIZE)
    );
    assert!(shown.is_valid());
    assert!(placement_keys(&app) && ghost_is_cursor(&app));
    hover(&mut app, middle(SITES[0], SIZE) + Vec2::new(20.0, -25.0));
    assert_eq!(ghost_at(&app), Some((SITES[0], SIZE)), "snapped");
    hover(&mut app, middle(SITES[1], SIZE) + Vec2::new(-12.0, 30.0));
    assert_eq!(ghost_at(&app), Some((SITES[1], SIZE)), "it follows the cursor");
    hover_panel(&mut app, slot(0, 0));
    assert_eq!(ghost(&app), None, "none over the HUD");

    click(&mut app, middle(SITES[0], SIZE), &[]);
    assert_eq!(orders(&app), [given(&[3], deploy_to(SITES[0]), false)]);
    assert_eq!(interface(&app), InterfaceState::DefaultState);
    assert_eq!(current(&app, 3), Some(deploy_to(SITES[0])));
    assert_eq!(pending(&app, P1), [(3, deploy::site(SITES[0]))]);
    assert_eq!(lingering(&mut app), [deploy::site(SITES[0])]);
    assert!(!placement_keys(&app));
}

/// `> AwaitingTarget`: "Escape or right-click cancels"; and Back (Z): the Drones' placement goes back to their
/// DefaultState (A is pressed there), and nothing is ordered.
#[test]
fn esc_a_right_click_and_back_end_the_placement() {
    let mut app = drone_game();
    select_drones(&mut app, &[3, 4]);
    for cancel in 0..3 {
        tap(&mut app, &[], KeyA);
        assert_eq!(placing(&app), Some(Placing::deploy()));
        match cancel {
            0 => tap(&mut app, &[], Escape),
            1 => right_click(&mut app, middle(SITES[0], SIZE), &[]),
            _ => tap(&mut app, &[], KeyZ),
        }
        assert_eq!(interface(&app), InterfaceState::DefaultState, "cancel {cancel}");
    }
    assert_eq!(orders(&app), []);
    assert_eq!(selected(&app), [3, 4]);
}

/// Design change 0120: the ghost "is green if, as far as the player knows, nothing blocks the footprint": red over
/// the player's Egress Node; and design change 0125: red where it overlaps a lingering ghost, the site of a Deploy
/// ordered (`TileProblem::Reserved`), tile by tile. The Drone whose own site it is may take it again with a plain
/// click, which replaces its Deploy: green; with Shift held, which would queue a second one, red.
#[test]
fn the_deploy_ghost_is_red_where_the_player_knows_of_a_blocker() {
    let mut app = drone_game();
    let tiles_with = |app: &App, problem: TileProblem| -> Vec<TilePos> {
        let shown = ghost(app).expect("a ghost");
        shown
            .tiles()
            .filter(|&t| shown.check.problems_at(t).any(|p| p == problem))
            .collect()
    };
    click(&mut app, drone_at(3), &[]);
    tap(&mut app, &[], KeyA);
    hover(&mut app, middle(OWN_NODE_AT, SIZE));
    assert!(!ghost(&app).unwrap().is_valid());
    assert_eq!(tiles_with(&app, TileProblem::Structure).len(), 4, "the Egress Node");
    tap(&mut app, &[], Escape);

    // Drone 4 is ordered to deploy at the first site; it stays selected.
    click(&mut app, drone_at(4), &[]);
    tap(&mut app, &[], KeyA);
    click(&mut app, middle(SITES[0], SIZE), &[]);
    assert_eq!(orders(&app), [given(&[4], deploy_to(SITES[0]), false)]);
    tap(&mut app, &[], KeyA);
    hover(&mut app, middle(SITES[0], SIZE));
    assert!(ghost(&app).unwrap().is_valid(), "its own site, its order replaced");
    press_keys(&mut app, &[ShiftLeft]);
    app.update();
    assert_eq!(
        tiles_with(&app, TileProblem::Reserved).len(),
        4,
        "queued, its own site reserved"
    );
    release_keys(&mut app, &[ShiftLeft]);
    tap(&mut app, &[], Escape);

    // Another Drone: the lingering ghost reserves its site.
    click(&mut app, drone_at(3), &[]);
    tap(&mut app, &[], KeyA);
    hover(&mut app, middle(SITES[0], SIZE));
    assert_eq!(tiles_with(&app, TileProblem::Reserved).len(), 4);
    let east = TilePos::new(SITES[0].x + 1, SITES[0].y);
    hover(&mut app, middle(east, SIZE));
    assert_eq!(
        tiles_with(&app, TileProblem::Reserved),
        [east, TilePos::new(east.x, east.y + 1)],
        "the overlap only"
    );
    click(&mut app, middle(east, SIZE), &[]);
    assert_eq!(orders(&app).len(), 1, "an invalid click sends nothing");
    assert_eq!(feedback(&app), Some("A building is already ordered there"));
    assert_eq!(placing(&app), Some(Placing::deploy()), "placement goes on");
}

/// Design change 0125 (`> AwaitingTarget > Shift while placing > Exception, Drones`): "4 Drones selected, hold Shift,
/// click 4 sites → 4 lingering ghosts, one Drone each. Once every Drone in the group has a Deploy, placement ends":
/// each Shift-click queues a Deploy to the next Drone in portrait order (id order) that has none yet.
#[test]
fn shift_placement_chains_through_the_group_until_every_drone_has_a_deploy() {
    let mut app = drone_game();
    select_drones(&mut app, &[3, 4, 5, 6]);
    tap(&mut app, &[], KeyA);
    press_keys(&mut app, &[ShiftLeft]);
    for (i, site) in SITES.into_iter().enumerate() {
        click(&mut app, middle(site, SIZE), &[]);
        let expected = if i < 3 { Some(queued()) } else { None };
        assert_eq!(placing(&app), expected, "after click {i}");
    }
    release_keys(&mut app, &[ShiftLeft]);
    app.update();
    let given: Vec<Given> = (3..=6)
        .zip(SITES)
        .map(|(drone, site)| given(&[drone], deploy_to(site), true))
        .collect();
    assert_eq!(orders(&app), given);
    let sites: Vec<(u32, Placement)> = (3..=6).zip(SITES.map(deploy::site)).collect();
    assert_eq!(pending(&app, P1), sites);
    assert_eq!(lingering(&mut app), SITES.map(deploy::site));
    assert_eq!(selected(&app), [3, 4, 5, 6]);
}

/// `> Shift while placing`: "releasing Shift ends placement", after a Shift-click has sent a Deploy; and "With a
/// single Drone, a Shift-click sends it and ends placement as before".
#[test]
fn letting_go_of_shift_or_a_single_drone_ends_the_placement() {
    let mut app = drone_game();
    select_drones(&mut app, &[3, 4, 5, 6]);
    tap(&mut app, &[], KeyA);
    press_keys(&mut app, &[ShiftLeft]);
    click(&mut app, middle(SITES[0], SIZE), &[]);
    assert_eq!(placing(&app), Some(queued()));
    release_keys(&mut app, &[ShiftLeft]);
    app.update();
    assert_eq!(interface(&app), InterfaceState::DefaultState);
    assert_eq!(orders(&app), [given(&[3], deploy_to(SITES[0]), true)]);

    click(&mut app, drone_at(6), &[]);
    assert_eq!(selected(&app), [6]);
    tap(&mut app, &[], KeyA);
    press_keys(&mut app, &[ShiftLeft]);
    click(&mut app, middle(SITES[2], SIZE), &[]);
    assert_eq!(interface(&app), InterfaceState::DefaultState, "Shift still held");
    release_keys(&mut app, &[ShiftLeft]);
    assert_eq!(orders(&app)[1], given(&[6], deploy_to(SITES[2]), true));
}

/// Design change 0125: "A Deploy order given to a group of Drones goes to one Drone only: the first in selection
/// order [...]: a plain click goes to the first Drone in the group (replacing its orders [...]). The other Drones are
/// untouched": six mining Drones, A and one click, and one of them goes to deploy while five keep mining.
#[test]
fn a_deploy_to_mining_drones_takes_the_first_of_them_only() {
    let drones = [3, 4, 5, 6, 7, 8];
    let mut app = colonists_game(0, |mut builder| {
        for x in 14..20 {
            let at = SpawnAt::Point(TilePos::new(x, 17).center());
            builder = builder.spawn(ObjectKind::Drone, Some(P1), at);
        }
        for x in 14..20 {
            builder = builder.spawn(ObjectKind::CrystalPatch, None, SpawnAt::Tile(TilePos::new(x, 14)));
        }
        builder
    });
    let points: Vec<Vec2> = (14..20).map(|x| TilePos::new(x, 17).center()).collect();
    select_around(&mut app, &points, &drones);
    for (drone, patch) in drones.into_iter().zip(9..) {
        let gather = OrderKind::Gather(ObjectId(patch));
        submit(
            &mut app,
            P1,
            Command::Order {
                units: vec![ObjectId(drone)],
                kind: gather,
                queue: false,
            },
        );
    }
    updates(&mut app, 3);
    for (drone, patch) in drones.into_iter().zip(9..) {
        assert_eq!(
            current(&app, drone),
            Some(OrderKind::Gather(ObjectId(patch))),
            "{drone} mines"
        );
    }
    tap(&mut app, &[], KeyA);
    click(&mut app, middle(SITES[3], SIZE), &[]);
    let deploys: Vec<Given> = orders(&app)
        .into_iter()
        .filter(|g| matches!(g.kind, OrderKind::Deploy { .. }))
        .collect();
    assert_eq!(deploys, [given(&[3], deploy_to(SITES[3]), false)]);
    assert_eq!(current(&app, 3), Some(deploy_to(SITES[3])), "it leaves");
    for (drone, patch) in drones.into_iter().zip(9..).skip(1) {
        assert_eq!(
            current(&app, drone),
            Some(OrderKind::Gather(ObjectId(patch))),
            "{drone} keeps mining"
        );
    }
    assert_eq!(selected(&app), drones);
}

/// `core/entities.md > Placement Validation > Lingering build ghosts`: "Visible to its owner. Enemies never see it":
/// player 2's Deploy shows no ghost to player 1. Player 1's shows until "construction starts (the real structure
/// replaces it)": once the Drone begins deploying, it is gone. Units walk over a ghost (`pending_sites_block_no_one`
/// in `crates/sim/tests/deploy_gate.rs`: the sim's rule, which the client only draws).
#[test]
fn a_lingering_ghost_is_its_owners_until_the_deployment_starts() {
    let mut app = drone_game();
    let theirs = TilePos::new(33, 12);
    submit(
        &mut app,
        P2,
        Command::Order {
            units: vec![ObjectId(7)],
            kind: deploy_to(theirs),
            queue: false,
        },
    );
    updates(&mut app, 2);
    assert_eq!(pending(&app, P2), [(7, deploy::site(theirs))]);
    assert_eq!(lingering(&mut app), [], "not player 1's");

    click(&mut app, drone_at(4), &[]);
    tap(&mut app, &[], KeyA);
    click(&mut app, middle(SITES[0], SIZE), &[]);
    assert_eq!(lingering(&mut app), [deploy::site(SITES[0])]);
    run_until(&mut app, 600, "Drone 4 deploys", |app| {
        object(app, 4).kind == ObjectKind::Beacon
    });
    app.update();
    assert_eq!(object(&app, 4).placement, Some(deploy::site(SITES[0])));
    assert_eq!(pending(&app, P1), []);
    assert_eq!(lingering(&mut app), [], "the Beacon replaces it");
}

/// `> Lingering build ghosts`: a ghost "Disappears [...] when the order is cancelled, replaced by a new order, or
/// fails": Stop (X) replaces Drone 3's Deploy; a unit standing on Drone 5's site when it arrives fails it
/// (`core/entities.md > Placement Validation > Worker-Built Structures`).
#[test]
fn a_lingering_ghost_goes_with_a_cancelled_or_failed_deploy() {
    let mut app = drone_game();
    click(&mut app, drone_at(3), &[]);
    tap(&mut app, &[], KeyA);
    click(&mut app, middle(SITES[0], SIZE), &[]);
    assert_eq!(lingering(&mut app), [deploy::site(SITES[0])]);
    tap(&mut app, &[], KeyX);
    updates(&mut app, 2);
    assert_eq!(current(&app, 3), None);
    assert_eq!(lingering(&mut app), [], "Stop replaced it");

    click(&mut app, drone_at(5), &[]);
    tap(&mut app, &[], KeyA);
    click(&mut app, middle(SITES[1], SIZE), &[]);
    assert_eq!(lingering(&mut app), [deploy::site(SITES[1])]);
    spawn(&mut app, ObjectKind::Peacekeeper, Some(P2), SITES[1].center());
    assert_eq!(
        lingering(&mut app),
        [deploy::site(SITES[1])],
        "a unit doesn't remove it"
    );
    run_until(&mut app, 600, "the Deploy fails", |app| current(app, 5).is_none());
    app.update();
    assert_eq!(object(&app, 5).kind, ObjectKind::Drone);
    assert_eq!(lingering(&mut app), []);
    assert_eq!(feedback(&app), Some("Blocked by a unit"));
}

/// Plan decision T15 (slice M3-E): a selected Drone that deploys stays selected, as its Beacon, in a group of its own
/// (the Beacon is Ungroupable); the Drones' group stays the ActiveGroup while a Drone is left in it, and their
/// placement goes on. The Beacon, Tabbed to, shows only X Cancel (design change 0116), and C does nothing; X cancels
/// the deployment, and the Drone it is again joins the Drones.
#[test]
fn a_drone_that_deploys_stays_selected_as_its_beacon() {
    use space_crystals::selection::ControlStates;
    let mut app = drone_game();
    select_drones(&mut app, &[3, 4]);
    tap(&mut app, &[], KeyA);
    // Drone 3's own spot: the footprint under it.
    let here = TilePos::new(14, 17);
    click(&mut app, middle(here, SIZE), &[]);
    assert_eq!(orders(&app), [given(&[3], deploy_to(here), false)]);
    tap(&mut app, &[], KeyA);
    run_until(&mut app, 200, "Drone 3 deploys", |app| {
        object(app, 3).kind == ObjectKind::Beacon
    });
    app.update();
    assert!(object(&app, 3).is_transforming());
    assert_eq!(selected(&app), [3, 4]);
    let groups = |app: &App| -> Vec<(ObjectKind, Vec<u32>)> {
        let states = app.world().resource::<ControlStates>();
        let selection = states.selection(local_player(app));
        let groups = selection.groups().into_iter();
        groups
            .map(|g| (g.kind(), g.instances.iter().map(|id| id.0).collect()))
            .collect()
    };
    let mut shown = groups(&app);
    shown.sort();
    assert_eq!(shown, [(ObjectKind::Drone, vec![4]), (ObjectKind::Beacon, vec![3])]);
    assert_eq!(placing(&app), Some(Placing::deploy()), "Drone 4 is left");

    tap(&mut app, &[], Tab);
    assert_eq!(interface(&app), InterfaceState::DefaultState);
    let c = PanelButton::Command;
    let x = CommandSlot::CANCEL_PRODUCTION;
    let cancel = (x, c(PanelCommand::CancelTransform), ButtonLook::Group, "Cancel");
    assert_eq!(buttons(&mut app), [cancel]);
    for key in [KeyQ, KeyA, KeyC] {
        tap(&mut app, &[], key);
        assert_eq!(interface(&app), InterfaceState::DefaultState, "{key:?} does nothing");
    }
    tap(&mut app, &[], KeyX);
    assert_eq!(
        constructions(&app),
        [(P1, vec![3], ConstructionCommand::CancelTransform)]
    );
    updates(&mut app, 2);
    assert_eq!(object(&app, 3).kind, ObjectKind::Drone);
    assert_eq!(groups(&app), [(ObjectKind::Drone, vec![3, 4])]);
    assert_eq!(orders(&app).len(), 1, "nothing else was ordered");
}

/// `> ObjectInterfaceState`: a placement goes back to DefaultState once it no longer holds: the one Drone placing
/// becomes a Beacon, which is then the ActiveGroup (`ControlState::set_type`), and its panel is X Cancel.
#[test]
fn the_placement_ends_when_its_last_drone_becomes_a_beacon() {
    let mut app = drone_game();
    click(&mut app, drone_at(3), &[]);
    tap(&mut app, &[], KeyA);
    click(&mut app, middle(SITES[0], SIZE), &[]);
    tap(&mut app, &[], KeyA);
    assert_eq!(placing(&app), Some(Placing::deploy()));
    run_until(&mut app, 600, "Drone 3 deploys", |app| {
        object(app, 3).kind == ObjectKind::Beacon
    });
    app.update();
    assert_eq!(interface(&app), InterfaceState::DefaultState);
    assert_eq!(selected(&app), [3]);
    let cancel = PanelButton::Command(PanelCommand::CancelTransform);
    let shown: Vec<PanelButton> = buttons(&mut app).into_iter().map(|b| b.1).collect();
    assert_eq!(shown, [cancel]);
    assert!(!placement_keys(&app));
}
