//! The Deployment Center's build menu (`factions/gdo/structures/deployment_center.md >
//! ObjectInterfaceState[DeploymentCenter]`, design change 0104): Q Basic Construction and W Advanced Construction,
//! Construct paid on the spot, and Cancel Construction with a full refund.

use super::*;

/// `factions/gdo/structures/deployment_center.md > ObjectInterfaceState[DeploymentCenter]` (design change 0104): a
/// selected Deployment Center shows Q Basic Construction and W Advanced Construction. W opens BuildMenu[Advanced],
/// whose five buildings hold their slots greyed, none constructed yet (slice M6-GK): Q Factory, W Advanced Mechanics, E
/// Airfield, R Propulsion Labs and A Industrial Powerplant; pressing one sends nothing and says "Not available yet".
/// Q, or a click on its button, opens BuildMenu[Basic]: Q Generator, W Barracks, E Extraction Facility, S Arsenal
/// (slice M5-K) and D Career Campus (slice M5-N9; both greyed: "Requires a Barracks"), R Supply Tower, A Radar Station
/// and F Defense System greyed as not constructed yet (slice M6-GK), and Z Back. Back
/// and Esc return to DefaultState (`factions/gdo/structures/deployment_center.md > BuildMenu (instance idle)`:
/// "Escape: returns to DefaultState (StateOnlyTransition). Right-click does its normal right-click and the menu stays
/// open", design change 0400): a right-click on the ground does the Deployment Center's, nothing, and the menu stays
/// open. A left-click on the map selects as ever. Beside an
/// Extraction Facility, recalled together from a control group (design change 0048's verification: "Select two own
/// Ungroupable buildings (e.g. via control group): Tab switches the command panel between them"), Tab makes that the
/// active group: the menu closes, and Q builds a plate there, at the Extraction Facility only.
#[test]
fn the_deployment_center_opens_its_basic_build_menu() {
    use ButtonLook::{Back, Common, Disabled, Group, Unavailable};
    use PanelCommand::{BuildMenu as Menu, Construct};
    let mut app = construction_game(1000);
    select(&mut app, DC);
    let c = PanelButton::Command;
    assert_eq!(
        buttons(&mut app),
        [
            (q(), c(Menu(BuildMenu::Basic)), Common, "Basic"),
            (w(), c(Menu(BuildMenu::Advanced)), Common, "Adv."),
        ]
    );
    tap(&mut app, &[], KeyW);
    assert_eq!(interface(&app), InterfaceState::BuildMenu(BuildMenu::Advanced));
    let planned = |planned| c(PanelCommand::Planned(planned));
    assert_eq!(
        buttons(&mut app),
        [
            (q(), planned(Planned::Factory), Unavailable, "FY"),
            (w(), planned(Planned::AdvancedMechanics), Unavailable, "AM"),
            (e(), planned(Planned::Airfield), Unavailable, "AD"),
            (r(), planned(Planned::PropulsionLabs), Unavailable, "PL"),
            (a(), planned(Planned::IndustrialPowerplant), Unavailable, "IP"),
            (CommandSlot::BACK, PanelButton::Back, Back, "Back"),
        ]
    );
    tap(&mut app, &[], KeyQ);
    assert_eq!(constructions(&app), [], "not constructed yet");
    assert_eq!(feedback(&app), Some(NOT_YET_TEXT));
    tap(&mut app, &[], KeyZ);
    assert_eq!(interface(&app), InterfaceState::DefaultState);

    let menu = InterfaceState::BuildMenu(BuildMenu::Basic);
    tap(&mut app, &[], KeyQ);
    assert_eq!(interface(&app), menu);
    let [supply_tower, radar_station, defense_system] = not_yet_basic();
    assert_eq!(
        buttons(&mut app),
        [
            (q(), c(Construct(GENERATOR)), Common, "GN"),
            (w(), c(Construct(BARRACKS_KIND)), Common, "BR"),
            (e(), c(Construct(EXTRACTION_FACILITY)), Common, "EF"),
            supply_tower,
            radar_station,
            (s(), c(Construct(ARSENAL)), Disabled, "AL"),
            (d(), c(Construct(CAREER_CAMPUS)), Disabled, "CC"),
            defense_system,
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
    assert_eq!(interface(&app), menu, "a right-click doesn't reset the menu");
    assert_eq!(constructions(&app), []);
    assert_eq!(orders(&app), []);

    // The menu leaves the map's left-clicks to the selection: a click on the Peacekeeper selects it.
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
/// Generator and returns to DefaultState, which then shows E, Place, greyed (design change 0388: the `place_button`
/// tests), and X, Cancel Construction ("Only visible when CurrentConstruction is set"). In the build menu the entries
/// are unavailable while it is constructed (`factions/gdo/structures/deployment_center.md > BuildMenu (instance
/// constructing)`), and X cancels from there too, by hotkey or by button: a full refund
/// (`factions/gdo/structures/deployment_center.md > Cancellation`), and back to DefaultState. The InfoPanel shows the
/// construction and its progress (design change 0105).
#[test]
fn a_construction_is_cancelled_with_a_full_refund() {
    use ButtonLook::{Back, Common, Disabled};
    use PanelCommand::{BuildMenu as Menu, CancelConstruction, Construct, Place};
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
            (w(), c(Menu(BuildMenu::Advanced)), Common, "Adv."),
            (e(), c(Place(GENERATOR)), Disabled, "GN"),
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
    let [supply_tower, radar_station, defense_system] = not_yet_basic();
    assert_eq!(
        buttons(&mut app),
        [
            (q(), c(Construct(GENERATOR)), Disabled, "GN"),
            (w(), c(Construct(BARRACKS_KIND)), Disabled, "BR"),
            (e(), c(Construct(EXTRACTION_FACILITY)), Disabled, "EF"),
            supply_tower,
            radar_station,
            (s(), c(Construct(ARSENAL)), Disabled, "AL"),
            (d(), c(Construct(CAREER_CAMPUS)), Disabled, "CC"),
            defense_system,
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

/// `factions/gdo/structures/deployment_center.md > Constructs`: "Career Campus: 150 Space Crystals, 320 frames (20
/// seconds) (placeholder). Requires: player owns at least one Barracks", BuildMenu[Basic]'s D (design change 0328;
/// slice M5-N9), and (slice M5-K) "Arsenal: 150 Space Crystals, 320 frames (20 seconds) (placeholder). Requires: player
/// owns at least one Barracks", its S (design change 0356). Without a Barracks, each is greyed, says "Requires a
/// Barracks" when pressed and sends nothing; with one (a debug spawn), each constructs its building for 150 Space
/// Crystals, and the menu closes.
#[test]
fn d_and_s_construct_a_career_campus_and_an_arsenal_once_a_barracks_stands() {
    for (key, at, kind, label) in [(KeyD, d(), CAREER_CAMPUS, "CC"), (KeyS, s(), ARSENAL, "AL")] {
        constructs_once_a_barracks_stands(key, at, kind, label);
    }
}

/// [`d_and_s_construct_a_career_campus_and_an_arsenal_once_a_barracks_stands`]'s game for `kind`, in BuildMenu[Basic]'s
/// slot `at` (hotkey `key`), labelled `label`.
fn constructs_once_a_barracks_stands(key: KeyCode, at: CommandSlot, kind: ObjectKind, label: &str) {
    use ButtonLook::{Common, Disabled};
    let mut app = construction_game(1000);
    select(&mut app, DC);
    tap(&mut app, &[], KeyQ);
    let construct = PanelButton::Command(PanelCommand::Construct(kind));
    let shown = |app: &mut App| buttons(app).into_iter().find(|b| b.0 == at);
    assert_eq!(shown(&mut app), Some((at, construct, Disabled, label)));
    tap(&mut app, &[], key);
    assert_eq!(feedback(&app), Some("Requires a Barracks"));
    assert_eq!(constructions(&app), []);

    let spawn = DebugCommand::Spawn {
        kind: BARRACKS_KIND,
        owner: Some(P1),
        at: SpawnAt::Tile(TilePos::new(27, 17)),
        orientation: BuildingOrientation::default(),
    };
    submit(&mut app, P1, Command::Debug(spawn));
    updates(&mut app, 2);
    assert!(structure_at(&app, BARRACKS_KIND, TilePos::new(27, 17)).is_some());
    assert_eq!(interface(&app), InterfaceState::BuildMenu(BuildMenu::Basic));
    assert_eq!(shown(&mut app), Some((at, construct, Common, label)));
    tap(&mut app, &[], key);
    assert_eq!(
        constructions(&app),
        [(P1, vec![DC], ConstructionCommand::Construct(kind))]
    );
    assert_eq!(interface(&app), InterfaceState::DefaultState);
    updates(&mut app, 1);
    assert_eq!(bank(&app), 1000 - 150);
    assert_eq!(constructor(&app, DC).current(), Some(kind));
}
