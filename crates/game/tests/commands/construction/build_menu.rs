//! The Deployment Center's build menu (`factions/gdo/structures/deployment_center.md >
//! ObjectInterfaceState[DeploymentCenter]`, design change 0104): Q Basic Construction and W Advanced Construction,
//! Construct paid on the spot, and Cancel Construction with a full refund.

use super::*;

/// `factions/gdo/structures/deployment_center.md > ObjectInterfaceState[DeploymentCenter]` (design change 0104): a
/// selected Deployment Center shows Q Basic Construction, and W Advanced Construction unavailable while none of its
/// buildings is in the game. Q, or a click on its button, opens BuildMenu[Basic]: Q Generator, W Barracks, E Extraction
/// Facility, the other slots hidden, and Z Back. Back, Esc and a right-click return to DefaultState
/// (`factions/gdo/structures/deployment_center.md > BuildMenu (instance idle)`: "Escape/right-click: returns to
/// DefaultState"), and a left-click on the map selects as ever. Beside an Extraction Facility, recalled together from a
/// control group (design change 0048's verification: "Select two own Ungroupable buildings (e.g. via control group):
/// Tab switches the command panel between them"), Tab makes that the active group: the menu closes, and Q builds a
/// plate there, at the Extraction Facility only.
#[test]
fn the_deployment_center_opens_its_basic_build_menu() {
    use ButtonLook::{Back, Common, Group, Unavailable};
    use PanelCommand::{BuildMenu as Menu, Construct};
    let mut app = construction_game(1000);
    select(&mut app, DC);
    let c = PanelButton::Command;
    assert_eq!(
        buttons(&mut app),
        [
            (q(), c(Menu(BuildMenu::Basic)), Common, "Basic"),
            (w(), c(Menu(BuildMenu::Advanced)), Unavailable, "Adv."),
        ]
    );
    tap(&mut app, &[], KeyW);
    assert_eq!(
        interface(&app),
        InterfaceState::DefaultState,
        "nothing to build in it yet"
    );

    let menu = InterfaceState::BuildMenu(BuildMenu::Basic);
    tap(&mut app, &[], KeyQ);
    assert_eq!(interface(&app), menu);
    assert_eq!(
        buttons(&mut app),
        [
            (q(), c(Construct(GENERATOR)), Common, "GN"),
            (w(), c(Construct(BARRACKS_KIND)), Common, "BR"),
            (e(), c(Construct(EXTRACTION_FACILITY)), Common, "EF"),
            (CommandSlot::BACK, PanelButton::Back, Back, "Back"),
        ]
    );
    tap(&mut app, &[], KeyZ);
    assert_eq!(interface(&app), InterfaceState::DefaultState);
    click_button(&mut app, q(), &[]);
    assert_eq!(interface(&app), menu);
    click_button(&mut app, CommandSlot::BACK, &[]);
    assert_eq!(interface(&app), InterfaceState::DefaultState);
    tap(&mut app, &[], KeyQ);
    tap(&mut app, &[], Escape);
    assert_eq!(interface(&app), InterfaceState::DefaultState);
    tap(&mut app, &[], KeyQ);
    right_click(&mut app, middle(OPEN, (2, 2)), &[]);
    assert_eq!(interface(&app), InterfaceState::DefaultState);
    assert_eq!(constructions(&app), []);
    assert_eq!(orders(&app), []);

    // The menu leaves the map's left-clicks to the selection: a click on the Peacekeeper selects it.
    tap(&mut app, &[], KeyQ);
    select(&mut app, PK);
    assert_eq!(interface(&app), InterfaceState::DefaultState);

    // Beside the Extraction Facility, in control group 1: the Deployment Center's group is active first (`ObjectKind`
    // order).
    select(&mut app, DC);
    add_box(&mut app, middle(TilePos::new(14, 11), (3, 3)));
    tap(&mut app, &[ControlLeft], Digit1);
    select(&mut app, PK);
    tap(&mut app, &[], Digit1);
    assert_eq!(selected(&app), [DC, EF]);
    tap(&mut app, &[], KeyQ);
    assert_eq!(interface(&app), menu);
    tap(&mut app, &[], Tab);
    assert_eq!(
        interface(&app),
        InterfaceState::DefaultState,
        "the Extraction Facility has no build menu"
    );
    assert_eq!(buttons(&mut app), [(q(), c(Construct(PLATE_KIND)), Group, "EP")]);
    tap(&mut app, &[], KeyQ);
    assert_eq!(
        constructions(&app),
        [(P1, vec![EF], ConstructionCommand::Construct(PLATE_KIND))]
    );
    updates(&mut app, 1);
    assert_eq!(constructor(&app, DC), Constructor::Idle);
    assert_eq!(constructor(&app, EF).current(), Some(PLATE_KIND));
}

/// `factions/gdo/structures/deployment_center.md > BuildMenu (instance idle)`: Q pays 100 Space Crystals for a
/// Generator and returns to DefaultState, which then shows X, Cancel Construction ("Only visible when
/// CurrentConstruction is set"). In the build menu the entries are unavailable while it is constructed
/// (`factions/gdo/structures/deployment_center.md > BuildMenu (instance constructing)`), and X cancels from there too,
/// by hotkey or by button: a full refund (`factions/gdo/structures/deployment_center.md > Cancellation`), and back to
/// DefaultState. The InfoPanel shows the construction and its progress (design change 0105).
#[test]
fn a_construction_is_cancelled_with_a_full_refund() {
    use ButtonLook::{Back, Common, Disabled, Unavailable};
    use PanelCommand::{BuildMenu as Menu, CancelConstruction, Construct};
    let mut app = construction_game(1000);
    select(&mut app, DC);
    tap(&mut app, &[], KeyQ);
    tap(&mut app, &[], KeyQ);
    let construct = ConstructionCommand::Construct(GENERATOR);
    assert_eq!(constructions(&app), [(P1, vec![DC], construct)]);
    assert_eq!(interface(&app), InterfaceState::DefaultState);
    assert_eq!(bank(&app), 900);
    assert_eq!(constructor(&app, DC).current(), Some(GENERATOR));
    let c = PanelButton::Command;
    assert_eq!(
        buttons(&mut app),
        [
            (q(), c(Menu(BuildMenu::Basic)), Common, "Basic"),
            (w(), c(Menu(BuildMenu::Advanced)), Unavailable, "Adv."),
            (X, c(CancelConstruction), Common, "Cancel"),
        ]
    );
    updates(&mut app, 24);
    let progress = activity(&mut app).expect("an activity");
    assert!(progress.starts_with("Constructing Generator: "), "{progress}");
    assert_ne!(progress, "Constructing Generator: 0%");

    tap(&mut app, &[], KeyX);
    let cancel = ConstructionCommand::CancelConstruction;
    assert_eq!(constructions(&app), [(P1, vec![DC], construct), (P1, vec![DC], cancel)]);
    assert_eq!((bank(&app), constructor(&app, DC)), (1000, Constructor::Idle));
    assert_eq!(activity(&mut app), None);

    tap(&mut app, &[], KeyQ);
    tap(&mut app, &[], KeyW);
    assert_eq!(bank(&app), 800, "a Barracks");
    tap(&mut app, &[], KeyQ);
    assert_eq!(
        buttons(&mut app),
        [
            (q(), c(Construct(GENERATOR)), Disabled, "GN"),
            (w(), c(Construct(BARRACKS_KIND)), Disabled, "BR"),
            (e(), c(Construct(EXTRACTION_FACILITY)), Disabled, "EF"),
            (CommandSlot::BACK, PanelButton::Back, Back, "Back"),
            (X, c(CancelConstruction), Common, "Cancel"),
        ]
    );
    tap(&mut app, &[], KeyE);
    assert_eq!(constructions(&app).len(), 3, "unavailable");
    click_button(&mut app, X, &[]);
    assert_eq!(constructions(&app).last(), Some(&(P1, vec![DC], cancel)));
    assert_eq!(interface(&app), InterfaceState::DefaultState);
    assert_eq!((bank(&app), constructor(&app, DC)), (1000, Constructor::Idle));
}
