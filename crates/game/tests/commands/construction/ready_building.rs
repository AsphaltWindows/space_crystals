//! A ready building (`factions/gdo/structures/deployment_center.md > BuildMenu (building ready)`,
//! `factions/gdo/structures/deployment_center.md > AwaitingPlacement`): placed with a left-click where the ghost is
//! green, once with Shift (design change 0071), refused by the sim when the game no longer allows it (plan decision
//! T4), and placement left with Esc, a right-click or Back.

use super::*;

/// `factions/gdo/structures/deployment_center.md > BuildMenu (building ready)`: once the Generator is ready its slot
/// becomes "Select completed building", and X Cancel Ready Building. It enters AwaitingPlacement: the ghost follows the
/// cursor in place of the pointer, green where it may go, the build area overlay shows the area, and R and F become the
/// placement keys. A left-click there places it at the ghost's tiles and orientation (one Place to the Deployment
/// Center) and returns to DefaultState; the Generator stands there from the next tick, and the ghost and the overlay
/// are gone.
#[test]
fn a_ready_generator_is_placed_where_the_ghost_is_green() {
    use ButtonLook::{Back, Common, Disabled, Pending};
    use PanelCommand::{CancelReady, Construct, PlaceReady};
    let mut app = construction_game(1000);
    select(&mut app, DC);
    tap(&mut app, &[], KeyQ);
    tap(&mut app, &[], KeyQ);
    run_until_ready(&mut app, DC);
    assert_eq!(activity(&mut app).as_deref(), Some("Generator ready to place"));
    tap(&mut app, &[], KeyQ);
    let c = PanelButton::Command;
    assert_eq!(
        buttons(&mut app),
        [
            (q(), c(PlaceReady(GENERATOR)), Common, "Place"),
            (w(), c(Construct(BARRACKS_KIND)), Disabled, "BR"),
            (e(), c(Construct(EXTRACTION_FACILITY)), Disabled, "EF"),
            (CommandSlot::BACK, PanelButton::Back, Back, "Back"),
            (X, c(CancelReady), Common, "Cancel"),
        ]
    );
    assert!(!placement_keys(&app));
    tap(&mut app, &[], KeyQ);
    assert_eq!(placing(&app), Some(Placing::new(ObjectId(DC), GENERATOR)));
    assert_eq!(buttons(&mut app)[0], (q(), c(PlaceReady(GENERATOR)), Pending, "Place"));
    assert!(placement_keys(&app));

    hover(&mut app, middle(OPEN, (2, 2)));
    let shown = ghost(&app).expect("the ghost");
    assert_eq!((shown.kind, shown.origin, shown.size), (GENERATOR, OPEN, (2, 2)));
    assert!(shown.is_valid());
    assert!(shown.tiles().all(|t| shown.tile_is_green(t)));
    assert_eq!(ghost_labels(&mut app), labels(&["A", "A", "A", "A", "GN"]));
    assert!(ghost_is_cursor(&app));
    assert_eq!(overlay(&mut app), (Some(build_area(&app)), true));

    click(&mut app, middle(OPEN, (2, 2)), &[]);
    let construct = ConstructionCommand::Construct(GENERATOR);
    let placed = place(OPEN, BuildingOrientation::default());
    assert_eq!(constructions(&app), [(P1, vec![DC], construct), (P1, vec![DC], placed)]);
    assert_eq!(interface(&app), InterfaceState::DefaultState);
    assert!(structure_at(&app, GENERATOR, OPEN).is_some());
    assert_eq!(constructor(&app, DC), Constructor::Idle);
    assert_eq!(selected(&app), [DC], "the click selected nothing");
    assert_eq!(ghost(&app), None);
    assert_eq!(ghost_labels(&mut app), []);
    assert!(!overlay(&mut app).1);
    assert!(!placement_keys(&app) && !ghost_is_cursor(&app));
    assert_eq!(activity(&mut app), None);
}

/// `factions/gdo/structures/deployment_center.md > AwaitingPlacement`: "Escape/right-click: returns to BuildMenu
/// (StateOnlyTransition), building remains ready to place", and so does Back (Z); the ghost, the overlay and the
/// placement keys go with the placement. From the build menu, X Cancel Ready Building refunds 75% of the cost, rounded
/// down (`factions/gdo/structures/deployment_center.md > BuildMenu (building ready)`), and returns to DefaultState.
/// Then R is a command slot again: the Peacekeeper's Riot Shield.
#[test]
fn escape_right_click_and_back_leave_placement_for_the_build_menu() {
    let mut app = construction_game(1000);
    select(&mut app, DC);
    construct_and_place(&mut app, KeyQ, GENERATOR);
    let menu = InterfaceState::BuildMenu(BuildMenu::Basic);
    let ready = Constructor::Ready { kind: GENERATOR };
    hover(&mut app, middle(OPEN, (2, 2)));
    assert!(ghost(&app).is_some() && overlay(&mut app).1 && placement_keys(&app) && ghost_is_cursor(&app));
    tap(&mut app, &[], Escape);
    assert_eq!((interface(&app), constructor(&app, DC)), (menu, ready));
    assert_eq!(ghost(&app), None);
    assert!(!overlay(&mut app).1 && !placement_keys(&app) && !ghost_is_cursor(&app));

    tap(&mut app, &[], KeyQ);
    assert!(placing(&app).is_some());
    right_click(&mut app, middle(OPEN, (2, 2)), &[]);
    assert_eq!((interface(&app), constructor(&app, DC)), (menu, ready));
    assert_eq!(orders(&app), [], "the right-click ordered nothing");
    tap(&mut app, &[], KeyQ);
    tap(&mut app, &[], KeyZ);
    assert_eq!((interface(&app), constructor(&app, DC)), (menu, ready));

    tap(&mut app, &[], KeyX);
    let cancel = ConstructionCommand::CancelReady;
    assert_eq!(constructions(&app).last(), Some(&(P1, vec![DC], cancel)));
    assert_eq!(interface(&app), InterfaceState::DefaultState);
    assert_eq!((bank(&app), constructor(&app, DC)), (900 + 75, Constructor::Idle));

    select(&mut app, PK);
    tap(&mut app, &[], KeyR);
    assert_eq!(
        abilities(&app),
        [(P1, vec![PK], sc_sim::Ability::RiotShield { raise: true })]
    );
}

/// `systems/controls/interface.md > AwaitingTarget > Shift while placing` (design change 0071): a GDO
/// constructor has one building ready at most, so a Shift-click places it once and ends placement like a plain click;
/// the next Shift-click places nothing.
#[test]
fn a_shift_click_places_once_and_ends_placement() {
    let mut app = construction_game(1000);
    select(&mut app, DC);
    construct_and_place(&mut app, KeyQ, GENERATOR);
    hover(&mut app, middle(OPEN, (2, 2)));
    click(&mut app, middle(OPEN, (2, 2)), &[ShiftLeft]);
    let placed = place(OPEN, BuildingOrientation::default());
    assert_eq!(constructions(&app).last(), Some(&(P1, vec![DC], placed)));
    assert_eq!(interface(&app), InterfaceState::DefaultState);
    assert!(structure_at(&app, GENERATOR, OPEN).is_some());
    assert!(!placement_keys(&app));
    click(&mut app, middle(TilePos::new(21, 18), (2, 2)), &[ShiftLeft]);
    assert_eq!(constructions(&app).len(), 2, "one Construct, one Place");
}

/// Plan decision T4: the sim checks a Place again as it applies it. When the game changed after the click (here a
/// unit arrives on the footprint in the same tick, before the Place), the sim refuses it, the Generator stays ready,
/// and the player hears why (`SimEvent::PlacementRefused`); they are in DefaultState, where the valid click left
/// them (`factions/gdo/structures/deployment_center.md > AwaitingPlacement`), and the ready building's slot starts
/// placement again.
#[test]
fn a_place_the_game_no_longer_allows_leaves_the_building_ready() {
    let mut app = construction_game(1000);
    select(&mut app, DC);
    construct_and_place(&mut app, KeyQ, GENERATOR);
    hover(&mut app, middle(OPEN, (2, 2)));
    input(&mut app).mouse.press(MouseButton::Left);
    app.update();
    input(&mut app).mouse.release(MouseButton::Left);
    app.update();
    let placed = place(OPEN, BuildingOrientation::default());
    assert_eq!(constructions(&app).last(), Some(&(P1, vec![DC], placed)));
    assert_eq!(interface(&app), InterfaceState::DefaultState);

    let arrives = Command::Debug(DebugCommand::Spawn {
        kind: ObjectKind::Peacekeeper,
        owner: Some(P1),
        at: SpawnAt::Point(OPEN.center()),
        orientation: BuildingOrientation::default(),
    });
    let pending = &mut app.world_mut().resource_mut::<PendingCommands>().0;
    pending.insert(
        0,
        PlayerCommand {
            player: P1,
            command: arrives,
        },
    );
    let events = run_collecting(&mut app, 1);
    let refused = events.iter().any(|event| {
        matches!(event, SimEvent::PlacementRefused { constructor, kind, origin, .. }
            if *constructor == ObjectId(DC) && *kind == GENERATOR && *origin == OPEN)
    });
    assert!(refused, "{events:?}");
    assert_eq!(constructor(&app, DC), Constructor::Ready { kind: GENERATOR });
    assert_eq!(structure_at(&app, GENERATOR, OPEN), None);
    assert_eq!(feedback(&app), Some("Blocked by a unit"));
    assert_eq!(interface(&app), InterfaceState::DefaultState);
    tap(&mut app, &[], KeyQ);
    tap(&mut app, &[], KeyQ);
    assert_eq!(placing(&app), Some(Placing::new(ObjectId(DC), GENERATOR)));
}
