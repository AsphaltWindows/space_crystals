//! Unfinished buildings form their own SelectionGroup (`systems/controls/selection_and_hud.md > SelectionGroup >
//! Unfinished buildings form their own group`, design change 0185): a Bio Portal being warped in is a group apart
//! from the finished Bio Portals selected with it, "as if it were another type", with its own panel, X Cancel alone
//! (`factions/colonists/economy.md > Beacon Deployment > While transforming: only Cancel`); the finished group trains
//! as ever. When the Warp In completes, the portal joins the finished group. Control groups, portrait clicks and
//! double-clicks keep working with the group key, (type, finished?).

use space_crystals::command_ui::PanelCommand;
use space_crystals::hud::layout::PortraitGrid;
use space_crystals::hud::selection_panel::{PortraitCell, SelectionPanelView};

use super::*;
use KeyCode::{ControlLeft, Digit1, KeyC, KeyE, KeyQ, KeyX, ShiftLeft, Tab};
use ObjectKind::{Beacon, BioPortal, Drone, SabreLegionnaire};
use PanelCommand::{CancelTransform, SetRallyPoint, Train};

// The objects of [`portals_game`], by id, and where they are drawn (the map is flat).
/// Player 1's finished Bio Portals, on tiles (14, 13) to (16, 15) and (19, 13) to (21, 15).
const PORTAL_A: u32 = 3;
const PORTAL_B: u32 = 4;
/// Player 1's Beacons, on tiles (25, 13) to (26, 14) and (31, 13) to (32, 14), each warping in a Bio Portal in the
/// tests.
const BEACON_1: u32 = 5;
const BEACON_2: u32 = 6;
const BEACON_1_AT: Vec2 = Vec2::new(1664.0, 896.0);
const BEACON_2_AT: Vec2 = Vec2::new(2048.0, 896.0);
/// Player 1's Drones, under the portals.
const DRONES: [u32; 2] = [7, 8];
const DRONES_AT: [Vec2; 2] = [Vec2::new(1200.0, 1180.0), Vec2::new(1400.0, 1180.0)];
/// Bare ground in the world view, away from everything.
const GROUND: Vec2 = Vec2::new(2300.0, 1150.0);
const X: CommandSlot = CommandSlot::CANCEL_PRODUCTION;

/// A [`colonists_game`] with player 1's Bio Portals [`PORTAL_A`] and [`PORTAL_B`], its Beacons [`BEACON_1`] and
/// [`BEACON_2`] and its [`DRONES`], and 2000 Space Crystals.
fn portals_game() -> App {
    colonists_game(2000, |builder| {
        let tile = |x, y| SpawnAt::Tile(TilePos::new(x, y));
        builder
            .spawn(BioPortal, Some(P1), tile(14, 13))
            .spawn(BioPortal, Some(P1), tile(19, 13))
            .spawn(Beacon, Some(P1), tile(25, 13))
            .spawn(Beacon, Some(P1), tile(31, 13))
            .spawn(Drone, Some(P1), SpawnAt::Point(DRONES_AT[0]))
            .spawn(Drone, Some(P1), SpawnAt::Point(DRONES_AT[1]))
    })
}

/// Warps a Bio Portal in on Beacon `beacon`, drawn at `at`, through its panel (`factions/colonists/economy.md > Beacon
/// ObjectInterfaceState`): a click on it, Q (Warp In (Basic)), E (the Bio Portal) and a click anywhere, which places
/// it.
fn warp_in_bio_portal(app: &mut App, beacon: u32, at: Vec2) {
    click(app, at, &[]);
    tap(app, &[], KeyQ);
    tap(app, &[], KeyE);
    click(app, GROUND, &[]);
    assert_eq!(object(app, beacon).kind, BioPortal);
    assert!(object(app, beacon).is_transforming());
}

/// Selects the portals `ids`: a click on the first, and a Shift + box around each of the others, which adds it.
fn select_portals(app: &mut App, ids: &[u32]) {
    click(app, object(app, ids[0]).pos, &[]);
    for &id in &ids[1..] {
        add_box(app, object(app, id).pos);
    }
    assert_eq!(selected(app), ids);
}

/// The local player's SelectionGroups, in their order: type, whether it is the unfinished group, its objects, and
/// whether it is the ActiveGroup.
fn groups(app: &App) -> Vec<(ObjectKind, bool, Vec<u32>, bool)> {
    let states = app.world().resource::<ControlStates>();
    let selection = states.selection(local_player(app));
    let active = selection.active();
    selection
        .groups()
        .into_iter()
        .map(|g| {
            let ids = g.instances.iter().map(|id| id.0).collect();
            (g.kind(), g.is_unfinished(), ids, Some(g.key) == active)
        })
        .collect()
}

/// The SelectionPanel's portraits, in their order: object, whether it shows as unfinished, whether it is highlighted.
fn portraits(app: &App) -> Vec<(u32, bool, bool)> {
    let cells = &app.world().resource::<SelectionPanelView>().cells;
    cells
        .iter()
        .map(|cell| match *cell {
            PortraitCell::Object {
                id, unfinished, active, ..
            } => (id.0, unfinished, active),
            PortraitCell::More { .. } => panic!("no overflow here"),
        })
        .collect()
}

/// A click on portrait `index` of the SelectionPanel with the modifier keys `modifiers` held.
fn click_portrait(app: &mut App, index: usize, modifiers: &[KeyCode]) {
    let surface = app.world().resource::<SurfaceSize>().logical();
    let point = PortraitGrid::on(surface).cell_rect(index).center();
    click_with(app, MouseButton::Left, point, modifiers);
}

/// The 0185 Verification: "Select 2 finished Bio Portals and 1 warping in: two groups; the finished group trains
/// normally; Tab to the other shows only Cancel." The finished group, active first (its group comes first), shows the
/// portal's panel, whose Train goes to both finished portals, a group command (the one warping in doesn't offer it);
/// the other group, its portrait marked, shows X Cancel alone: Q and C do nothing, and X cancels its Warp In. A Beacon
/// again, it is a group of its own (Ungroupable), which stays active.
#[test]
fn two_finished_bio_portals_and_one_warping_in_are_two_groups() {
    let mut app = portals_game();
    warp_in_bio_portal(&mut app, BEACON_1, BEACON_1_AT);
    select_portals(&mut app, &[PORTAL_A, PORTAL_B, BEACON_1]);
    assert_eq!(
        groups(&app),
        [
            (BioPortal, false, vec![PORTAL_A, PORTAL_B], true),
            (BioPortal, true, vec![BEACON_1], false),
        ]
    );
    assert_eq!(
        portraits(&app),
        [
            (PORTAL_A, false, true),
            (PORTAL_B, false, true),
            (BEACON_1, true, false)
        ]
    );
    let c = PanelButton::Command;
    let train = (slot(0, 0), c(Train(SabreLegionnaire)), ButtonLook::Group, "SL");
    assert_eq!(buttons(&mut app).first(), Some(&train));
    tap(&mut app, &[], KeyQ);
    let trained = ProductionCommand::Train(SabreLegionnaire);
    assert_eq!(productions(&app), [(P1, vec![PORTAL_A, PORTAL_B], trained)]);
    app.update();
    for portal in [PORTAL_A, PORTAL_B] {
        let queue = &production_of(&app, portal).queue;
        assert_eq!(queue, &[sc_sim::QueueEntry::Unit(SabreLegionnaire)], "{portal}");
    }
    let warping = object(&app, BEACON_1).production.as_ref();
    assert!(warping.is_none_or(|p| p.queue.is_empty()), "{warping:?}");

    tap(&mut app, &[], Tab);
    assert_eq!(
        portraits(&app),
        [
            (PORTAL_A, false, false),
            (PORTAL_B, false, false),
            (BEACON_1, true, true)
        ]
    );
    let cancel = (X, c(CancelTransform), ButtonLook::Group, "Cancel");
    assert_eq!(buttons(&mut app), [cancel]);
    for key in [KeyQ, KeyC] {
        tap(&mut app, &[], key);
        assert_eq!(interface(&app), InterfaceState::DefaultState, "{key:?} does nothing");
    }
    assert_eq!(productions(&app).len(), 1, "no Train, no rally point");
    tap(&mut app, &[], KeyX);
    let cancelled = (P1, vec![BEACON_1], ConstructionCommand::CancelTransform);
    assert_eq!(constructions(&app).last(), Some(&cancelled));
    updates(&mut app, 2);
    assert_eq!(object(&app, BEACON_1).kind, Beacon);
    assert_eq!(
        groups(&app),
        [
            (Beacon, false, vec![BEACON_1], true),
            (BioPortal, false, vec![PORTAL_A, PORTAL_B], false),
        ]
    );
}

/// Design change 0185: "When it completes, it joins the finished group." The finished group is the ActiveGroup,
/// awaiting the target of its rally point (C), when the Warp In completes: it stays active, now with the new portal,
/// and so does its ObjectInterfaceState (`systems/controls/selection_and_hud.md > ObjectInterfaceState`: a change the
/// player didn't make keeps it "unless the ActiveGroup becomes empty"), so the rally point the click sets goes to all
/// three.
#[test]
fn a_portal_whose_warp_in_completes_joins_the_finished_group() {
    let mut app = portals_game();
    warp_in_bio_portal(&mut app, BEACON_1, BEACON_1_AT);
    select_portals(&mut app, &[PORTAL_A, PORTAL_B, BEACON_1]);
    tap(&mut app, &[], KeyC);
    let awaiting = InterfaceState::awaiting(SetRallyPoint);
    assert_eq!(interface(&app), awaiting);
    run_until(&mut app, 600, "the Warp In completes", |app| {
        !object(app, BEACON_1).is_transforming()
    });
    app.update();
    let all = vec![PORTAL_A, PORTAL_B, BEACON_1];
    assert_eq!(groups(&app), [(BioPortal, false, all.clone(), true)]);
    assert_eq!(interface(&app), awaiting);
    click(&mut app, GROUND, &[]);
    let rallies: Vec<Vec<u32>> = productions(&app).into_iter().map(|(_, ids, _)| ids).collect();
    assert_eq!(rallies, [all]);
}

/// Design change 0185, with the unfinished group active: two portals warp in, Tabbed to. The first to complete joins
/// the finished group, and the unfinished group, which still has the other, stays active (keeping the ActiveGroup
/// while it exists, `crate::selection::Selection::set_type`), with only Cancel. When the last completes, its group is
/// gone, and the group it joined, now all four portals, is the ActiveGroup, with the portal's panel.
#[test]
fn the_unfinished_group_stays_active_until_its_last_portal_completes() {
    let mut app = portals_game();
    warp_in_bio_portal(&mut app, BEACON_1, BEACON_1_AT);
    warp_in_bio_portal(&mut app, BEACON_2, BEACON_2_AT);
    select_portals(&mut app, &[PORTAL_A, PORTAL_B, BEACON_1, BEACON_2]);
    tap(&mut app, &[], Tab);
    assert_eq!(
        groups(&app),
        [
            (BioPortal, false, vec![PORTAL_A, PORTAL_B], false),
            (BioPortal, true, vec![BEACON_1, BEACON_2], true),
        ]
    );
    let c = PanelButton::Command;
    let cancel = (X, c(CancelTransform), ButtonLook::Group, "Cancel");
    run_until(&mut app, 600, "the first Warp In completes", |app| {
        !object(app, BEACON_1).is_transforming()
    });
    app.update();
    assert!(object(&app, BEACON_2).is_transforming(), "started later");
    assert_eq!(
        groups(&app),
        [
            (BioPortal, false, vec![PORTAL_A, PORTAL_B, BEACON_1], false),
            (BioPortal, true, vec![BEACON_2], true),
        ]
    );
    assert_eq!(buttons(&mut app), [cancel]);
    run_until(&mut app, 600, "the second Warp In completes", |app| {
        !object(app, BEACON_2).is_transforming()
    });
    app.update();
    let all = vec![PORTAL_A, PORTAL_B, BEACON_1, BEACON_2];
    assert_eq!(groups(&app), [(BioPortal, false, all, true)]);
    assert_eq!(interface(&app), InterfaceState::DefaultState);
    let train = (slot(0, 0), c(Train(SabreLegionnaire)), ButtonLook::Common, "SL");
    assert_eq!(buttons(&mut app).first(), Some(&train));
}

/// Control groups, portrait clicks and the map's Ctrl-click and double-click with the group key (design change 0185).
/// A control group of the three portals recalls them as two groups again. A Ctrl-click on a portrait keeps the
/// selected objects of its type, and Ctrl + Shift removes them, the unfinished ones counting as another type: the
/// portal warping in alone, then the finished ones alone. A double-click or a Ctrl-click on a Drone selects both
/// Drones on screen.
#[test]
fn control_groups_and_type_selection_keep_the_unfinished_group_apart() {
    let mut app = portals_game();
    warp_in_bio_portal(&mut app, BEACON_1, BEACON_1_AT);
    let portals = [PORTAL_A, PORTAL_B, BEACON_1];
    select_portals(&mut app, &portals);
    tap(&mut app, &[ControlLeft], Digit1);
    click(&mut app, GROUND, &[]);
    assert!(selected(&app).is_empty());
    pause(&mut app);
    tap(&mut app, &[], Digit1);
    assert_eq!(selected(&app), portals);
    let two = [
        (BioPortal, false, vec![PORTAL_A, PORTAL_B], true),
        (BioPortal, true, vec![BEACON_1], false),
    ];
    assert_eq!(groups(&app), two);

    click_portrait(&mut app, 2, &[ControlLeft]);
    assert_eq!(groups(&app), [(BioPortal, true, vec![BEACON_1], true)]);
    tap(&mut app, &[], Digit1);
    pause(&mut app);
    click_portrait(&mut app, 0, &[ControlLeft]);
    assert_eq!(groups(&app), [(BioPortal, false, vec![PORTAL_A, PORTAL_B], true)]);
    tap(&mut app, &[], Digit1);
    pause(&mut app);
    click_portrait(&mut app, 2, &[ControlLeft, ShiftLeft]);
    assert_eq!(groups(&app), [(BioPortal, false, vec![PORTAL_A, PORTAL_B], true)]);
    tap(&mut app, &[], Digit1);
    pause(&mut app);
    click_portrait(&mut app, 1, &[ControlLeft, ShiftLeft]);
    assert_eq!(groups(&app), [(BioPortal, true, vec![BEACON_1], true)]);

    let screen = screen_at(&app, DRONES_AT[0]);
    input(&mut app).cursor = Some(screen);
    for _ in 0..2 {
        input(&mut app).mouse.press(MouseButton::Left);
        app.update();
        input(&mut app).mouse.release(MouseButton::Left);
        app.update();
    }
    pause(&mut app);
    assert_eq!(selected(&app), DRONES, "a double-click");
    click(&mut app, GROUND, &[]);
    click(&mut app, DRONES_AT[1], &[ControlLeft]);
    assert_eq!(selected(&app), DRONES, "a Ctrl-click");
}

/// Design change 0199's Verification (`systems/controls/selection_and_hud.md > Selection > ActiveGroup > When the
/// ActiveGroup disappears`): "Select a Barracks under construction (active) and a finished one. When the first
/// completes, the merged Barracks group is active." A GDO building is placed finished, so a Bio Portal warping in
/// stands in for the Barracks under construction, the one way a building is unfinished in the game. Tabbed to, its
/// group is active, with only Cancel; when the Warp In completes, its members "changed type and joined another group:
/// the group they joined becomes active", and its panel, the portal's, is in its DefaultState.
#[test]
fn a_building_under_construction_active_completes_and_the_merged_group_is_active() {
    let mut app = portals_game();
    warp_in_bio_portal(&mut app, BEACON_1, BEACON_1_AT);
    select_portals(&mut app, &[PORTAL_A, BEACON_1]);
    tap(&mut app, &[], Tab);
    assert_eq!(
        groups(&app),
        [
            (BioPortal, false, vec![PORTAL_A], false),
            (BioPortal, true, vec![BEACON_1], true),
        ]
    );
    run_until(&mut app, 600, "the Warp In completes", |app| {
        !object(app, BEACON_1).is_transforming()
    });
    app.update();
    assert_eq!(groups(&app), [(BioPortal, false, vec![PORTAL_A, BEACON_1], true)]);
    assert_eq!(interface(&app), InterfaceState::DefaultState);
    let train = (
        slot(0, 0),
        PanelButton::Command(Train(SabreLegionnaire)),
        ButtonLook::Common,
        "SL",
    );
    assert_eq!(buttons(&mut app).first(), Some(&train));
}
