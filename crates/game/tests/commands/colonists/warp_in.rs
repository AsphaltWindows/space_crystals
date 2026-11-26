//! A Beacon's Warp In from the command panel (`factions/colonists/economy.md > Beacon > Beacon ObjectInterfaceState`,
//! `> Warp In > Warp In Command`; design changes 0119, 0121, 0122 and 0124): Q and W open Warp In (Basic) and Warp In
//! (Advanced), whose buildings not in the game yet show unavailable; a building enters AwaitingPlacement with its
//! ghost on the Beacon, centred, an odd size's extras to the bottom-right, turned about the Beacon's centre by R and
//! F and never moved by the mouse, red where blocked; a click anywhere warps it in, Z goes back, and Tab picks the
//! Beacon. Whatever transforms shows only X Cancel and sets no rally point (design change 0116), and a greyed-out
//! button pressed says why (design change 0162).

use sc_sim::{BuildingOrientation, Rotation, TileProblem, transform};
use space_crystals::command_ui::{BuildMenu, PanelCommand, Planned, PointerDisplay};

use super::*;
use KeyCode::{KeyA, KeyC, KeyE, KeyF, KeyQ, KeyR, KeyS, KeyW, KeyX, KeyZ, ShiftLeft, Tab};
use ObjectKind::{BioPortal, EgressNode, RoboPortal};
use PanelCommand::{BuildMenu as Menu, CancelTransform, WarpIn};

// The objects of [`warp_in_game`], by id, and the centres of their footprints, where they are drawn (the map is flat).
/// Player 1's Beacons, on tiles (20, 11) to (21, 12) and (30, 11) to (31, 12).
const BEACON_A: u32 = 3;
const BEACON_B: u32 = 4;
const BEACON_A_AT: Vec2 = Vec2::new(1344.0, 768.0);
const BEACON_B_AT: Vec2 = Vec2::new(1984.0, 768.0);
const BEACON_A_TILE: TilePos = TilePos::new(20, 11);
/// Player 1's Robo Portal, on tiles (13, 14) to (16, 17).
const ROBO_PORTAL: u32 = 5;
const ROBO_PORTAL_AT: Vec2 = Vec2::new(960.0, 1024.0);
/// Bare ground in the world view, away from everything.
const GROUND: Vec2 = Vec2::new(2200.0, 1100.0);

/// A [`colonists_game`] with player 1's Beacons [`BEACON_A`] and [`BEACON_B`] and Robo Portal [`ROBO_PORTAL`], and
/// `crystals` Space Crystals.
fn warp_in_game(crystals: u32) -> App {
    colonists_game(crystals, |builder| {
        let tile = |x, y| SpawnAt::Tile(TilePos::new(x, y));
        builder
            .spawn(ObjectKind::Beacon, Some(P1), tile(20, 11))
            .spawn(ObjectKind::Beacon, Some(P1), tile(30, 11))
            .spawn(RoboPortal, Some(P1), tile(13, 14))
    })
}

fn q() -> CommandSlot {
    slot(0, 0)
}

const X: CommandSlot = CommandSlot::CANCEL_PRODUCTION;

/// A Warp In of `kind`, turned by `orientation`.
fn warp(kind: ObjectKind, orientation: BuildingOrientation) -> ConstructionCommand {
    ConstructionCommand::WarpIn { kind, orientation }
}

/// Where the sim puts a `kind` warped in on Beacon A turned by `orientation` (`transform::warp_in_footprint`): its
/// top-left tile and size.
fn footprint(kind: ObjectKind, orientation: BuildingOrientation) -> (TilePos, (u32, u32)) {
    let beacon = Placement::new(BEACON_A_TILE, ObjectKind::Beacon.def().footprint);
    let p = transform::warp_in_footprint(&beacon, kind, orientation).expect("a structure");
    (p.origin, (p.w, p.h))
}

/// `> Beacon ObjectInterfaceState` (design change 0124): DefaultState opens Warp In (Basic) in Q and Warp In
/// (Advanced) in W; Basic has Q Egress Node, W Robo Portal, E Bio Portal, and A Alloy Furnace, S Essence Refinery, D
/// Flux Reactor unavailable ("shown but unavailable until defined"), Z Back. A building enters placement: the Robo
/// Portal's 4 × 4 ghost centred on the Beacon (`> Warp In Command`: "shown as a ghost overlay centered on the Beacon's
/// location"), which the mouse never moves (design change 0121), over the HUD too; the pointer shows, the ghost isn't
/// the cursor. Z goes back to the menu, and from the menu to DefaultState.
#[test]
fn the_beacon_opens_warp_in_basic_and_its_ghost_stands_on_the_beacon() {
    use ButtonLook::{Back, Common, Disabled, Pending, Unavailable};
    let mut app = warp_in_game(1000);
    click(&mut app, BEACON_A_AT, &[]);
    assert_eq!(selected(&app), [BEACON_A]);
    let c = PanelButton::Command;
    assert_eq!(
        buttons(&mut app),
        [
            (q(), c(Menu(BuildMenu::Basic)), Common, "Basic"),
            (slot(0, 1), c(Menu(BuildMenu::Advanced)), Common, "Adv."),
        ]
    );
    tap(&mut app, &[], KeyQ);
    assert_eq!(interface(&app), InterfaceState::BuildMenu(BuildMenu::Basic));
    let p = |b| c(PanelCommand::Planned(b));
    assert_eq!(
        buttons(&mut app),
        [
            (q(), c(WarpIn(EgressNode)), Common, "EN"),
            (slot(0, 1), c(WarpIn(RoboPortal)), Common, "RP"),
            (slot(0, 2), c(WarpIn(BioPortal)), Common, "BP"),
            (slot(1, 0), p(Planned::AlloyFurnace), Unavailable, "AF"),
            (slot(1, 1), p(Planned::EssenceRefinery), Unavailable, "ER"),
            (slot(1, 2), p(Planned::FluxReactor), Unavailable, "FR"),
            (CommandSlot::BACK, PanelButton::Back, Back, "Back"),
        ]
    );
    tap(&mut app, &[], KeyZ);
    assert_eq!(interface(&app), InterfaceState::DefaultState);
    tap(&mut app, &[], KeyQ);

    tap(&mut app, &[], KeyW);
    assert_eq!(placing(&app), Some(Placing::warp_in(ObjectId(BEACON_A), RoboPortal)));
    let robo = (TilePos::new(19, 10), (4, 4));
    assert_eq!(ghost_at(&app), Some(robo));
    assert_eq!(footprint(RoboPortal, BuildingOrientation::default()), robo);
    let shown = ghost(&app).unwrap();
    assert!(shown.kind == RoboPortal && shown.is_valid());
    assert!(placement_keys(&app) && !ghost_is_cursor(&app));
    for ground in [GROUND, BEACON_A_AT + Vec2::new(-300.0, 200.0)] {
        hover(&mut app, ground);
        assert_eq!(ghost_at(&app), Some(robo), "the mouse never moves it");
    }
    hover_panel(&mut app, q());
    assert_eq!(ghost_at(&app), Some(robo), "over the HUD too");
    let marked: Vec<(CommandSlot, ButtonLook)> = buttons(&mut app).into_iter().map(|b| (b.0, b.2)).collect();
    assert_eq!(marked[1], (slot(0, 1), Pending));
    assert_eq!(marked[0], (q(), Disabled));

    tap(&mut app, &[], KeyZ);
    assert_eq!(
        interface(&app),
        InterfaceState::BuildMenu(BuildMenu::Basic),
        "back to its menu"
    );
    tap(&mut app, &[], KeyQ);
    assert_eq!(
        ghost_at(&app),
        Some((BEACON_A_TILE, (2, 2))),
        "the Egress Node on the Beacon's own tiles"
    );
    tap(&mut app, &[], KeyZ);
    tap(&mut app, &[], KeyZ);
    assert_eq!(interface(&app), InterfaceState::DefaultState);
    assert_eq!(constructions(&app), []);
}

/// Design change 0121: the Bio Portal's 3 × 3 "starts in a fixed default position: the extra tiles to the
/// bottom-right"; "a horizontal flip moves them to the left/right, a vertical flip to the top/bottom, a 90-degree
/// rotation to the next corner", about the Beacon's centre, as the sim derives it (`transform::warp_in_footprint`).
/// A click anywhere confirms it: one Warp In to the Beacon, turned as the ghost was, and the Beacon becomes the
/// building on the ghost's footprint.
#[test]
fn the_bio_portal_turns_about_the_beacon_and_warps_in_where_its_ghost_was() {
    let mut app = warp_in_game(1000);
    click(&mut app, BEACON_A_AT, &[]);
    tap(&mut app, &[], KeyQ);
    tap(&mut app, &[], KeyE);
    let at = |x, y| Some((TilePos::new(x, y), (3, 3)));
    assert_eq!(ghost_at(&app), at(20, 11), "the extras to the bottom-right");
    // Each turn, and the top-left tile it leaves the ghost on.
    let steps: [(&[KeyCode], KeyCode, (i32, i32)); 8] = [
        (&[], KeyF, (19, 11)),
        (&[], KeyF, (20, 11)),
        (&[ShiftLeft], KeyF, (20, 10)),
        (&[ShiftLeft], KeyF, (20, 11)),
        (&[], KeyR, (19, 11)),
        (&[], KeyR, (19, 10)),
        (&[], KeyR, (20, 10)),
        (&[], KeyR, (20, 11)),
    ];
    for (i, (modifiers, key, (x, y))) in steps.into_iter().enumerate() {
        let expected = at(x, y);
        tap(&mut app, modifiers, key);
        assert_eq!(ghost_at(&app), expected, "step {i}");
        let orientation = placing(&app).unwrap().orientation;
        assert_eq!(Some(footprint(BioPortal, orientation)), expected, "step {i}: the sim's");
    }
    tap(&mut app, &[], KeyF);
    let flipped = BuildingOrientation {
        rotation: Rotation::R0,
        flip_h: true,
        flip_v: false,
    };
    assert_eq!(placing(&app).unwrap().orientation, flipped);
    hover(&mut app, GROUND);
    assert_eq!(ghost_at(&app), at(19, 11));
    click(&mut app, GROUND, &[]);
    assert_eq!(constructions(&app), [(P1, vec![BEACON_A], warp(BioPortal, flipped))]);
    assert_eq!(interface(&app), InterfaceState::DefaultState);
    let beacon = object(&app, BEACON_A);
    assert_eq!(beacon.kind, BioPortal);
    assert_eq!(beacon.placement.map(|p| (p.origin, (p.w, p.h))), at(19, 11));
    assert!(beacon.is_transforming());
    assert_eq!(selected(&app), [BEACON_A], "it stays selected");
}

/// `> Warp In Command`: "If the footprint is blocked elsewhere, the ghost is tinted red and placement cannot be
/// confirmed": a unit on a tile of the Robo Portal's ring (design change 0119: units block), red, and a click sends
/// nothing but says why; once it is gone, the ghost is green and a click warps the Robo Portal in. A tile out of the
/// player's sight would be red too (design change 0119), but the Beacon's own sight always covers its footprint: the
/// sim's `a_warp_in_needs_every_tile_visible` (`crates/sim/tests/warp_in/`) and the ghost's
/// `a_warp_in_ghost_shows_a_tile_out_of_sight_red` check it.
#[test]
fn the_warp_in_ghost_is_red_where_a_unit_stands() {
    let mut app = warp_in_game(1000);
    click(&mut app, BEACON_A_AT, &[]);
    tap(&mut app, &[], KeyQ);
    tap(&mut app, &[], KeyW);
    assert!(ghost(&app).unwrap().is_valid());
    let corner = TilePos::new(19, 10);
    let drone = spawn(&mut app, ObjectKind::Drone, Some(P1), corner.center());
    app.update();
    let shown = ghost(&app).unwrap();
    assert!(!shown.is_valid());
    let red: Vec<TilePos> = shown.tiles().filter(|&t| !shown.tile_is_green(t)).collect();
    assert_eq!(red, [corner]);
    assert!(shown.check.problems_at(corner).any(|p| p == TileProblem::Unit));
    click(&mut app, GROUND, &[]);
    assert_eq!(constructions(&app), []);
    assert_eq!(feedback(&app), Some("Blocked by a unit"));
    assert!(placing(&app).is_some(), "placement goes on");

    remove(&mut app, drone);
    assert!(ghost(&app).unwrap().is_valid());
    click(&mut app, GROUND, &[]);
    let robo = warp(RoboPortal, BuildingOrientation::default());
    assert_eq!(constructions(&app), [(P1, vec![BEACON_A], robo)]);
}

/// `> Beacon ObjectInterfaceState > Warp In (Advanced)`: Q Exophysics Engine, W Skylance, E Fusion Core, R Plasma
/// Turret, A Relay Array, S Navigation Spire, D Psionic Amplifier, F Structural Analyzer, none in the game yet: eight
/// unavailable slots and Back. Design change 0162: pressing one, by hotkey or button, issues nothing and says why.
#[test]
fn the_advanced_menu_shows_its_eight_buildings_unavailable() {
    use Planned::*;
    let mut app = warp_in_game(1000);
    click(&mut app, BEACON_A_AT, &[]);
    tap(&mut app, &[], KeyW);
    assert_eq!(interface(&app), InterfaceState::BuildMenu(BuildMenu::Advanced));
    let planned = [
        (ExophysicsEngine, "EE"),
        (Skylance, "SK"),
        (FusionCore, "FC"),
        (PlasmaTurret, "PT"),
        (RelayArray, "RA"),
        (NavigationSpire, "NS"),
        (PsionicAmplifier, "PA"),
        (StructuralAnalyzer, "SA"),
    ];
    let mut expected: Vec<_> = CommandSlot::all()
        .zip(planned)
        .map(|(slot, (building, label))| {
            let button = PanelButton::Command(PanelCommand::Planned(building));
            (slot, button, ButtonLook::Unavailable, label)
        })
        .collect();
    expected.push((CommandSlot::BACK, PanelButton::Back, ButtonLook::Back, "Back"));
    expected.sort_by_key(|b| b.0);
    assert_eq!(buttons(&mut app), expected);
    assert_eq!(feedback(&app), None);
    tap(&mut app, &[], KeyS);
    assert_eq!(feedback(&app), Some("Not available yet"));
    assert_eq!(interface(&app), InterfaceState::BuildMenu(BuildMenu::Advanced));
    pause(&mut app);
    pause(&mut app);
    pause(&mut app);
    pause(&mut app);
    assert_eq!(feedback(&app), None, "it fades");
    click_button(&mut app, slot(1, 3), &[]);
    assert_eq!(feedback(&app), Some("Not available yet"), "a click on the button too");
    assert_eq!(constructions(&app), []);
}

/// Design change 0122: "Beacons are Ungroupable [...] Warp In applies to the Beacon the player has Tabbed to": with
/// both Beacons selected, the ghost stands on the active one only; Tab makes the other one active, which ends the
/// placement (a player's change of the ActiveGroup), and its Warp In goes to it alone.
#[test]
fn the_warp_in_goes_to_the_beacon_tabbed_to() {
    let mut app = warp_in_game(1000);
    click(&mut app, BEACON_A_AT, &[]);
    add_box(&mut app, BEACON_B_AT);
    assert_eq!(selected(&app), [BEACON_A, BEACON_B]);
    tap(&mut app, &[], KeyQ);
    tap(&mut app, &[], KeyW);
    assert_eq!(ghost_at(&app), Some((TilePos::new(19, 10), (4, 4))));
    tap(&mut app, &[], Tab);
    assert_eq!(interface(&app), InterfaceState::DefaultState);
    assert_eq!(ghost(&app), None);
    tap(&mut app, &[], KeyQ);
    tap(&mut app, &[], KeyW);
    assert_eq!(placing(&app), Some(Placing::warp_in(ObjectId(BEACON_B), RoboPortal)));
    assert_eq!(ghost_at(&app), Some((TilePos::new(29, 10), (4, 4))), "on Beacon B only");
    click(&mut app, GROUND, &[]);
    let robo = warp(RoboPortal, BuildingOrientation::default());
    assert_eq!(constructions(&app), [(P1, vec![BEACON_B], robo)]);
    assert_eq!(object(&app, BEACON_A).kind, ObjectKind::Beacon);
    assert_eq!(object(&app, BEACON_B).kind, RoboPortal);
}

/// Design change 0116: "A Beacon under deployment and a building under Warp In show only the Cancel command in their
/// CommandPanel (no rally point, no other commands)": the Robo Portal warping in shows X alone; C does nothing, nor
/// does a right-click, whose pointer previews nothing; X cancels the Warp In, and it is a Beacon again. Selected with
/// a working Robo Portal, it is a group of its own (design change 0185), which stays active, still with X alone, and
/// Tab shows the working portal's panel.
#[test]
fn a_portal_warping_in_shows_only_cancel_and_sets_no_rally_point() {
    let mut app = warp_in_game(1000);
    click(&mut app, BEACON_A_AT, &[]);
    tap(&mut app, &[], KeyQ);
    tap(&mut app, &[], KeyW);
    click(&mut app, GROUND, &[]);
    assert_eq!(object(&app, BEACON_A).kind, RoboPortal);
    assert!(object(&app, BEACON_A).is_transforming());
    let c = PanelButton::Command;
    assert_eq!(
        buttons(&mut app),
        [(X, c(CancelTransform), ButtonLook::Common, "Cancel")]
    );
    tap(&mut app, &[], KeyC);
    assert_eq!(interface(&app), InterfaceState::DefaultState, "no rally to await");
    right_click(&mut app, GROUND, &[]);
    assert_eq!(pointer_over(&mut app, GROUND), Some(PointerDisplay::Inactive));
    assert_eq!(productions(&app), [], "no rally point");
    assert_eq!(orders(&app), []);

    add_box(&mut app, ROBO_PORTAL_AT);
    assert_eq!(selected(&app), [BEACON_A, ROBO_PORTAL]);
    assert_eq!(
        buttons(&mut app),
        [(X, c(CancelTransform), ButtonLook::Group, "Cancel")]
    );
    tap(&mut app, &[], Tab);
    let shown = buttons(&mut app);
    let train = c(PanelCommand::Train(ObjectKind::Drone));
    assert_eq!(
        shown.first().map(|b| (b.0, b.1)),
        Some((q(), train)),
        "the portal's panel"
    );
    let warping = object(&app, BEACON_A).pos;
    click(&mut app, warping, &[]);
    assert_eq!(selected(&app), [BEACON_A]);

    let bank_before = bank(&app);
    tap(&mut app, &[], KeyX);
    let warp_in = warp(RoboPortal, BuildingOrientation::default());
    assert_eq!(
        constructions(&app),
        [
            (P1, vec![BEACON_A], warp_in),
            (P1, vec![BEACON_A], ConstructionCommand::CancelTransform)
        ]
    );
    updates(&mut app, 2);
    assert_eq!(object(&app, BEACON_A).kind, ObjectKind::Beacon);
    assert!(bank(&app) > bank_before, "refunded");
    assert_eq!(selected(&app), [BEACON_A]);
}

/// Design change 0162 (`systems/controls/interface.md > CommandPanel > Pressing an unavailable command`): "a
/// greyed-out command (e.g. Train with too few Space Crystals [...]) can still be pressed or its hotkey used; nothing
/// is issued, and the player gets the same feedback as when an order is refused, naming the reason": the Robo
/// Portal's Train and the Beacon's Warp In with an empty bank, and a building not in the game; and a Warp In whose
/// click the sim refuses, the bank emptied since its entry was pressed, which leaves the placement on.
#[test]
fn pressing_a_greyed_out_button_says_why() {
    let mut app = warp_in_game(0);
    click(&mut app, ROBO_PORTAL_AT, &[]);
    let look = |app: &mut App, slot| buttons(app).into_iter().find(|b| b.0 == slot).map(|b| b.2);
    assert_eq!(look(&mut app, q()), Some(ButtonLook::Disabled));
    tap(&mut app, &[], KeyQ);
    assert_eq!(feedback(&app), Some("Not enough Space Crystals"));
    assert_eq!(productions(&app), []);

    click(&mut app, BEACON_A_AT, &[]);
    tap(&mut app, &[], KeyQ);
    assert_eq!(look(&mut app, slot(0, 1)), Some(ButtonLook::Disabled));
    tap(&mut app, &[], KeyA);
    assert_eq!(feedback(&app), Some("Not available yet"));
    tap(&mut app, &[], KeyW);
    assert_eq!(feedback(&app), Some("Not enough Space Crystals"));
    assert_eq!(
        interface(&app),
        InterfaceState::BuildMenu(BuildMenu::Basic),
        "no placement"
    );

    set_crystals(&mut app, 1000);
    app.update();
    assert_eq!(look(&mut app, slot(0, 1)), Some(ButtonLook::Common));
    tap(&mut app, &[], KeyW);
    assert!(placing(&app).is_some());
    set_crystals(&mut app, 0);
    updates(&mut app, 32);
    assert_eq!(feedback(&app), None);
    click(&mut app, GROUND, &[]);
    assert_eq!(feedback(&app), Some("Not enough Space Crystals"));
    assert_eq!(constructions(&app), []);
    assert_eq!(placing(&app), Some(Placing::warp_in(ObjectId(BEACON_A), RoboPortal)));
}
