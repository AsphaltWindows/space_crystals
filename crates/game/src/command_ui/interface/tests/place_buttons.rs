//! The Place buttons with their progress (design changes 0388 and 0392): the Deployment Center's DefaultState E: Place
//! (`factions/gdo/structures/deployment_center.md > ObjectInterfaceState[DeploymentCenter] > DefaultState commands`),
//! and the Extraction Facility's Q: Place Plate while its plate is constructed (`factions/gdo/economy.md >
//! ExtractionFacility > ObjectInterfaceState[ExtractionFacility]`), greyed with the construction's progress, the sim's,
//! until the building is ready.

use sc_sim::economy::RATIO_ONE;
use sc_sim::{Constructor, Progress};

use super::*;
use crate::command_ui::feedback::unavailable_text;

const DC: ObjectKind = ObjectKind::DeploymentCenter;
const EF: ObjectKind = ObjectKind::ExtractionFacility;
const GENERATOR: ObjectKind = ObjectKind::Generator;
const PLATE: ObjectKind = ObjectKind::ExtractionPlate;

/// The conditions of a constructor in state `constructor`.
fn with(constructor: Constructor) -> Conditions {
    Conditions::default().with_constructor(constructor)
}

/// A construction of `kind` with `frames` frames done at full speed, and a quarter of the next.
fn constructing(kind: ObjectKind, frames: u32) -> Constructor {
    Constructor::Constructing {
        kind,
        progress: frames * RATIO_ONE + RATIO_ONE / 4,
    }
}

/// 0388: the Deployment Center's DefaultState shows E: Place "whenever CurrentConstruction or ReadyToPlace is set, with
/// the building's icon" (its tag, "GN", the label standing for it), beside Q and W. While the Generator (240 frames) is
/// constructed E is greyed, with its progress (60 frames done: a quarter), and says why when pressed; once it is ready
/// E is enabled, without a bar, and enters AwaitingPlacement. It is hidden while the Deployment Center is idle, and the
/// build menu keeps its slot of the building: Basic's Q still places it ("The build menus' existing placement slot
/// stays").
#[test]
fn the_deployment_centers_e_shows_its_construction_then_places_it() {
    use ButtonLook::{Common, Disabled, Unavailable};
    use PanelCommand::{BuildMenu as Menu, CancelConstruction, CancelReady, Place, PlaceReady};
    let dc = Selection::new([(id(1), DC)]);
    let command = PanelButton::Command;
    let (q, w, e, x) = (slot(0, 0), slot(0, 1), slot(0, 2), CommandSlot::CANCEL_PRODUCTION);
    let idle = panel(&dc, InterfaceState::DefaultState, &with(Constructor::Idle));
    assert_eq!(idle.get(e), None, "hidden while idle");

    let building = panel(&dc, InterfaceState::DefaultState, &with(constructing(GENERATOR, 60)));
    assert_eq!(
        shown(&building),
        [
            (q, command(Menu(BuildMenu::Basic)), Common),
            (w, command(Menu(BuildMenu::Advanced)), Unavailable),
            (e, command(Place(GENERATOR)), Disabled),
            (x, command(CancelConstruction), Common),
        ]
    );
    let place = building.get(e).unwrap();
    assert_eq!(
        (place.label(), building.name(e).as_deref()),
        ("GN", Some("Place Generator"))
    );
    let progress = place.progress.expect("a progress bar");
    assert_eq!(
        progress,
        Progress {
            elapsed: 60 * RATIO_ONE + RATIO_ONE / 4,
            total: 240
        }
    );
    assert_eq!(progress.ratio(), 0.25);
    assert_eq!(press(&building, e), SlotPress::Nothing);
    assert_eq!(place.why.map(unavailable_text), Some("Still under construction"));
    assert_eq!([q, w, x].map(|s| building.get(s).unwrap().progress), [None; 3]);

    let ready = with(Constructor::Ready { kind: GENERATOR });
    let lit = panel(&dc, InterfaceState::DefaultState, &ready);
    assert_eq!(
        [e, x].map(|s| lit.get(s).map(|s| (s.button, s.look, s.progress))),
        [
            Some((command(Place(GENERATOR)), Common, None)),
            Some((command(CancelReady), Common, None)),
        ]
    );
    assert_eq!(press(&lit, e), SlotPress::AwaitShortcutPlacement(GENERATOR));
    let menu = panel(&dc, InterfaceState::BuildMenu(BuildMenu::Basic), &ready);
    assert_eq!(menu.get(q).map(|s| s.button), Some(command(PlaceReady(GENERATOR))));
    assert_eq!(press(&menu, q), SlotPress::AwaitPlacement(GENERATOR));
    assert_eq!(
        menu.get(e).map(|s| s.button),
        Some(command(PanelCommand::Construct(EF)))
    );
    assert!(offers(DC, Place(GENERATOR)) && !offers(EF, Place(PLATE)));
    assert!(commands(DC).contains(&Place(GENERATOR)) && !commands(EF).contains(&Place(PLATE)));
}

/// 0388: "pressing it enters AwaitingPlacement for the ready building (StateOnlyTransition), exactly as the building's
/// slot in the build menu does. Escape/right-click from there returns to DefaultState": the placement's panel is the
/// DefaultState's, E lit and the rest waiting, and Back leads there, where one entered in the build menu leads back to
/// the menu. It lasts while the Deployment Center holds the building ready, and goes to it alone.
#[test]
fn a_placement_entered_with_e_goes_back_to_default_state() {
    use ButtonLook::{Back, Disabled, Pending};
    use PanelCommand::{BuildMenu as Menu, CancelReady, Place};
    let dc = Selection::new([(id(1), DC)]);
    let command = PanelButton::Command;
    let (q, e, x) = (slot(0, 0), slot(0, 2), CommandSlot::CANCEL_PRODUCTION);
    let ready = with(Constructor::Ready { kind: GENERATOR });
    let shortcut = Placing::by_shortcut(id(1), GENERATOR);
    assert_eq!(shortcut.command(), Place(GENERATOR));
    let placing = InterfaceState::AwaitingPlacement(shortcut);
    let awaiting = panel(&dc, placing, &ready);
    assert_eq!(
        [q, e, x, CommandSlot::BACK].map(|s| awaiting.get(s).map(|s| (s.button, s.look))),
        [
            Some((command(Menu(BuildMenu::Basic)), Disabled)),
            Some((command(Place(GENERATOR)), Pending)),
            Some((command(CancelReady), Disabled)),
            Some((PanelButton::Back, Back)),
        ]
    );
    assert_eq!(placing.back(&dc), InterfaceState::DefaultState);
    let from_menu = InterfaceState::AwaitingPlacement(Placing::new(id(1), GENERATOR));
    assert_eq!(from_menu.back(&dc), InterfaceState::BuildMenu(BuildMenu::Basic));
    assert!(still_valid(placing, &dc, true, &ready));
    assert!(
        !still_valid(placing, &dc, true, &with(Constructor::Idle)),
        "placed or cancelled"
    );
    let mut two = Selection::new([(id(1), DC), (id(2), DC)]);
    assert_eq!(recipients(&two, Place(GENERATOR)), [id(1)], "Ungroupable");
    two.cycle(true);
    assert!(!still_valid(placing, &two, true, &ready), "another Deployment Center");
}

/// 0392: "while a plate is under construction, **Q: Place Plate** is shown greyed out, with a running progress bar. It
/// lights up when the plate is ready": the Extraction Facility's Q shows the plate's progress (400 frames: 30 done),
/// then places it, without a bar.
#[test]
fn the_extraction_facilitys_q_shows_its_plates_progress() {
    use ButtonLook::{Common, Disabled};
    use PanelCommand::PlaceReady;
    let ef = Selection::new([(id(3), EF)]);
    let q = slot(0, 0);
    let building = panel(&ef, InterfaceState::DefaultState, &with(constructing(PLATE, 30)));
    let place = building.get(q).unwrap();
    assert_eq!(
        (place.button, place.look, place.label()),
        (PanelButton::Command(PlaceReady(PLATE)), Disabled, "Place")
    );
    assert_eq!(place.progress.map(|p| (p.ticks_done(), p.total)), Some((30, 400)));
    let ready = panel(
        &ef,
        InterfaceState::DefaultState,
        &with(Constructor::Ready { kind: PLATE }),
    );
    let lit = ready.get(q).unwrap();
    assert_eq!((lit.look, lit.progress), (Common, None));
    assert_eq!(press(&ready, q), SlotPress::AwaitPlacement(PLATE));
}

/// A panel is the same as another but for its bars' progress while only how far they have come differs: the HUD then
/// moves the bars on in place (`crate::hud::command_panel`), and rebuilds the buttons for anything else.
#[test]
fn a_panel_differs_only_in_progress_while_its_bars_move_on() {
    let dc = Selection::new([(id(1), DC)]);
    let at = |frames| {
        panel(
            &dc,
            InterfaceState::DefaultState,
            &with(constructing(GENERATOR, frames)),
        )
    };
    assert_ne!(at(10), at(11));
    assert!(at(10).same_but_progress(&at(11)));
    let ready = panel(
        &dc,
        InterfaceState::DefaultState,
        &with(Constructor::Ready { kind: GENERATOR }),
    );
    assert!(!at(10).same_but_progress(&ready), "lit, without a bar");
    let barracks = panel(&dc, InterfaceState::DefaultState, &with(constructing(BARRACKS, 10)));
    assert!(!at(10).same_but_progress(&barracks), "another building");
}
