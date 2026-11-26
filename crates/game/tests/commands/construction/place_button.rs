//! The Place buttons with their progress bars (design changes 0388 and 0392): the Deployment Center's DefaultState E:
//! Place (`factions/gdo/structures/deployment_center.md > ObjectInterfaceState[DeploymentCenter] > DefaultState
//! commands`) and the Extraction Facility's Q: Place Plate (`factions/gdo/economy.md > ExtractionFacility >
//! ObjectInterfaceState[ExtractionFacility]`), greyed with a bar that fills as the sim constructs the building, lit
//! once it is ready.

use space_crystals::hud::progress_bar::ProgressFill;

use super::*;

/// The button the HUD shows in `slot`: button, look, label.
fn button(app: &mut App, slot: CommandSlot) -> Option<(PanelButton, ButtonLook, &'static str)> {
    let shown = buttons(app).into_iter().find(|(at, ..)| *at == slot)?;
    Some((shown.1, shown.2, shown.3))
}

/// The progress bar of `slot`'s button, if it has one: its fill and the share done it shows.
fn bar(app: &mut App, slot: CommandSlot) -> Option<(Entity, f32)> {
    let mut buttons = app.world_mut().query::<(Entity, &CommandButton)>();
    let mut fills = app.world_mut().query::<(Entity, &ProgressFill, &ChildOf)>();
    let mut parents = app.world_mut().query::<&ChildOf>();
    let world = app.world();
    let (button, _) = buttons.iter(world).find(|(_, b)| b.slot == slot)?;
    fills
        .iter(world)
        .find(|(_, _, track)| parents.get(world, track.parent()).is_ok_and(|on| on.parent() == button))
        .map(|(fill, shown, _)| (fill, shown.0))
}

/// The share of its 240 frames the Deployment Center's Generator has done, as the sim counts it.
fn generator_done(app: &App) -> f32 {
    constructor(app, DC).frames_done().expect("constructing") as f32 / 240.0
}

/// 0388's Verification: "Order a Generator and stay in DefaultState: E shows a greyed Generator with a filling bar.
/// When it completes, E lights up. Press E, then click to place it, without opening Basic Construction." The bar shows
/// the sim's progress, moving on in place, and E pressed early says why it is greyed. Escape and a right-click from
/// the placement return to DefaultState, the Generator still ready ("Escape/right-click from there returns to
/// DefaultState"); once it is placed E is gone.
#[test]
fn e_places_the_generator_without_opening_basic_construction() {
    use ButtonLook::{Common, Disabled, Pending};
    use PanelCommand::Place;
    let mut app = construction_game(1000);
    select(&mut app, DC);
    tap(&mut app, &[], KeyQ);
    tap(&mut app, &[], KeyQ);
    assert_eq!(interface(&app), InterfaceState::DefaultState);
    let e_place = PanelButton::Command(Place(GENERATOR));
    assert_eq!(button(&mut app, e()), Some((e_place, Disabled, "GN")));
    let (fill, first) = bar(&mut app, e()).expect("a progress bar");
    assert!((first - generator_done(&app)).abs() <= 1.0 / 240.0, "{first}");
    updates(&mut app, 60);
    let (moved, later) = bar(&mut app, e()).expect("a progress bar");
    assert_eq!(moved, fill, "the bar moves on in place");
    assert!(later > first + 0.15, "{first} then {later}");
    assert!((later - generator_done(&app)).abs() <= 1.0 / 240.0, "{later}");
    tap(&mut app, &[], KeyE);
    assert_eq!(interface(&app), InterfaceState::DefaultState);
    assert_eq!(feedback(&app), Some("Still under construction"));

    run_until_ready(&mut app, DC);
    assert_eq!(button(&mut app, e()), Some((e_place, Common, "GN")));
    assert_eq!(bar(&mut app, e()), None);
    let ready = Constructor::Ready { kind: GENERATOR };
    for leave in [Escape, KeyZ] {
        tap(&mut app, &[], KeyE);
        assert_eq!(placing(&app), Some(Placing::by_shortcut(ObjectId(DC), GENERATOR)));
        assert_eq!(button(&mut app, e()), Some((e_place, Pending, "GN")));
        tap(&mut app, &[], leave);
        assert_eq!(
            (interface(&app), constructor(&app, DC)),
            (InterfaceState::DefaultState, ready)
        );
    }
    tap(&mut app, &[], KeyE);
    right_click(&mut app, middle(OPEN, (2, 2)), &[]);
    assert_eq!(
        (interface(&app), constructor(&app, DC)),
        (InterfaceState::DefaultState, ready)
    );

    tap(&mut app, &[], KeyE);
    hover(&mut app, middle(OPEN, (2, 2)));
    click(&mut app, middle(OPEN, (2, 2)), &[]);
    let construct = ConstructionCommand::Construct(GENERATOR);
    let placed = place(OPEN, BuildingOrientation::default());
    assert_eq!(constructions(&app), [(P1, vec![DC], construct), (P1, vec![DC], placed)]);
    assert_eq!(interface(&app), InterfaceState::DefaultState);
    assert!(structure_at(&app, GENERATOR, OPEN).is_some());
    assert_eq!(constructor(&app, DC), Constructor::Idle);
    assert_eq!(button(&mut app, e()), None, "hidden while idle");
}

/// 0392's Verification: "Order a plate and stay in the panel: Q shows a filling bar, then lights up."
#[test]
fn the_extraction_facilitys_q_fills_then_lights_up() {
    use ButtonLook::{Common, Disabled};
    let mut app = construction_game(1000);
    select(&mut app, EF);
    tap(&mut app, &[], KeyQ);
    assert_eq!(constructor(&app, EF).current(), Some(PLATE_KIND));
    let place = PanelButton::Command(PanelCommand::PlaceReady(PLATE_KIND));
    assert_eq!(button(&mut app, q()), Some((place, Disabled, "Place")));
    let (_, first) = bar(&mut app, q()).expect("a progress bar");
    updates(&mut app, 100);
    let (_, later) = bar(&mut app, q()).expect("a progress bar");
    assert!(later > first + 0.2, "{first} then {later}: 100 of 400 frames");
    run_until_ready(&mut app, EF);
    assert_eq!(button(&mut app, q()), Some((place, Common, "Place")));
    assert_eq!(bar(&mut app, q()), None);
}
