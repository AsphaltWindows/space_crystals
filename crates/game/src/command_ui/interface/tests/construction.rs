//! The constructors' panels (slice M2-H2): the Deployment Center's build menus, the Extraction Facility's plate, the
//! Cancels of what they construct, AwaitingPlacement and its ghost under the cursor.

use sc_sim::{ConstructionCommand, Constructor};

use super::*;

const DC: ObjectKind = ObjectKind::DeploymentCenter;
const EF: ObjectKind = ObjectKind::ExtractionFacility;
const GENERATOR: ObjectKind = ObjectKind::Generator;
const PLATE: ObjectKind = ObjectKind::ExtractionPlate;

fn constructing(kind: ObjectKind) -> Constructor {
    Constructor::Constructing { kind, progress: 0 }
}

/// The gate's answers while a `kind` constructor is in state `constructor`, busy: every building to construct
/// refused (`Reject::ConstructorBusy`).
fn busy(kind: ObjectKind, constructor: Constructor) -> Conditions {
    sc_sim::construction::recipes(kind)
        .iter()
        .fold(Conditions::default().with_constructor(constructor), |c, r| {
            c.refusing(PanelCommand::Construct(r.kind), Reject::ConstructorBusy(id(1)))
        })
}

/// `factions/gdo/structures/deployment_center.md > ObjectInterfaceState[DeploymentCenter]` (design change 0104):
/// DefaultState opens BuildMenu[Basic] (Q) and BuildMenu[Advanced] (W, unavailable: none of its buildings is in the
/// game yet); the idle Basic menu constructs a Generator (Q), a Barracks (W) or an Extraction Facility (E), its other
/// slots hidden; Z is Back; X shows only while something is constructed or ready.
#[test]
fn the_deployment_center_opens_its_build_menus() {
    use ButtonLook::{Back, Common, Disabled, Group, Unavailable};
    use PanelCommand::{BuildMenu as Menu, Construct};
    let dc = Selection::new([(id(1), DC)]);
    let idle = Conditions::default().with_constructor(Constructor::Idle);
    let default = panel(&dc, InterfaceState::DefaultState, &idle);
    let command = PanelButton::Command;
    let (q, w, e) = (slot(0, 0), slot(0, 1), slot(0, 2));
    assert_eq!(
        shown(&default),
        [
            (q, command(Menu(BuildMenu::Basic)), Common),
            (w, command(Menu(BuildMenu::Advanced)), Unavailable),
        ]
    );
    assert_eq!([q, w].map(|s| default.get(s).unwrap().label()), ["Basic", "Adv."]);
    assert_eq!(press(&default, q), SlotPress::OpenMenu(BuildMenu::Basic));
    assert_eq!(press(&default, w), SlotPress::Nothing, "nothing to build in it yet");

    let basic = InterfaceState::BuildMenu(BuildMenu::Basic);
    let menu = panel(&dc, basic, &idle);
    assert_eq!(
        shown(&menu),
        [
            (q, command(Construct(GENERATOR)), Common),
            (w, command(Construct(BARRACKS)), Common),
            (e, command(Construct(EF)), Common),
            (CommandSlot::BACK, PanelButton::Back, Back),
        ]
    );
    assert_eq!([q, w, e].map(|s| menu.get(s).unwrap().label()), ["GN", "BR", "EF"]);
    assert_eq!(
        press(&menu, q),
        SlotPress::Construct(Construct(GENERATOR), ConstructionCommand::Construct(GENERATOR))
    );
    assert_eq!(press(&menu, CommandSlot::BACK), SlotPress::Back);
    assert_eq!(basic.back(&dc), InterfaceState::DefaultState);
    // "Only available if player has sufficient Space Crystals".
    let poor = idle.clone().refusing(Construct(BARRACKS), Reject::InsufficientCrystals);
    let menu = panel(&dc, basic, &poor);
    assert_eq!(menu.get(w).map(|s| s.look), Some(Disabled));
    assert_eq!(press(&menu, w), SlotPress::Nothing);
    assert!(still_valid(basic, &dc, true, &idle));
    assert!(!still_valid(basic, &dc, false, &idle), "not commandable");
    let advanced = InterfaceState::BuildMenu(BuildMenu::Advanced);
    assert!(!still_valid(advanced, &dc, true, &idle), "unavailable");
    // A Peacekeeper beside the Deployment Center (the Peacekeepers' group comes first: Tab to the Deployment
    // Center): its commands are group commands.
    let mut mixed = Selection::new([(id(1), DC), (id(2), PK)]);
    mixed.cycle(true);
    assert_eq!(mixed.active().map(|k| k.kind), Some(DC));
    let shown_mixed = panel(&mixed, InterfaceState::DefaultState, &idle);
    assert_eq!(shown_mixed.get(q).map(|s| s.look), Some(Group));
}

/// `factions/gdo/structures/deployment_center.md > BuildMenu (instance constructing)`: X cancels with a full refund and
/// the other options are unavailable; `factions/gdo/structures/deployment_center.md > BuildMenu (building ready)`: the
/// ready building's slot becomes "Select completed building" (design change 0104), which enters AwaitingPlacement, and
/// X cancels the ready building; the DefaultState shows X too. AwaitingPlacement marks the building, the rest waits,
/// and Back returns to the build menu.
#[test]
fn a_constructing_or_ready_deployment_center_cancels_in_x_and_places_its_building() {
    use ButtonLook::{Back, Common, Disabled, Group, Pending};
    use PanelCommand::{CancelConstruction, CancelReady, Construct, PlaceReady};
    let dc = Selection::new([(id(1), DC)]);
    let (q, w, e, x) = (slot(0, 0), slot(0, 1), slot(0, 2), CommandSlot::CANCEL_PRODUCTION);
    let command = PanelButton::Command;
    let basic = InterfaceState::BuildMenu(BuildMenu::Basic);
    let building = busy(DC, constructing(GENERATOR));
    let menu = panel(&dc, basic, &building);
    let looks = |panel: &Panel| [q, w, e, x].map(|s| panel.get(s).map(|s| (s.button, s.look)));
    assert_eq!(
        looks(&menu),
        [
            Some((command(Construct(GENERATOR)), Disabled)),
            Some((command(Construct(BARRACKS)), Disabled)),
            Some((command(Construct(EF)), Disabled)),
            Some((command(CancelConstruction), Common)),
        ]
    );
    assert_eq!(
        press(&menu, x),
        SlotPress::Construct(CancelConstruction, ConstructionCommand::CancelConstruction)
    );
    assert_eq!(press(&menu, q), SlotPress::Nothing, "unavailable");
    let default = panel(&dc, InterfaceState::DefaultState, &building);
    assert_eq!(default.get(x).map(|s| s.label()), Some("Cancel"));

    let ready = busy(DC, Constructor::Ready { kind: GENERATOR });
    let menu = panel(&dc, basic, &ready);
    assert_eq!(
        looks(&menu),
        [
            Some((command(PlaceReady(GENERATOR)), Common)),
            Some((command(Construct(BARRACKS)), Disabled)),
            Some((command(Construct(EF)), Disabled)),
            Some((command(CancelReady), Common)),
        ]
    );
    assert_eq!(menu.get(q).map(|s| s.label()), Some("Place"));
    assert_eq!(press(&menu, q), SlotPress::AwaitPlacement(GENERATOR));
    assert_eq!(
        press(&menu, x),
        SlotPress::Construct(CancelReady, ConstructionCommand::CancelReady)
    );
    assert!(
        still_valid(basic, &dc, true, &ready),
        "the menu stays open as the state changes"
    );

    let placing = InterfaceState::AwaitingPlacement(Placing::new(id(1), GENERATOR));
    let awaiting = panel(&dc, placing, &ready);
    assert_eq!(
        looks(&awaiting),
        [
            Some((command(PlaceReady(GENERATOR)), Pending)),
            Some((command(Construct(BARRACKS)), Disabled)),
            Some((command(Construct(EF)), Disabled)),
            Some((command(CancelReady), Disabled)),
        ]
    );
    assert_eq!(awaiting.get(CommandSlot::BACK).map(|s| s.look), Some(Back));
    assert_eq!(press(&awaiting, x), SlotPress::Nothing, "waits");
    assert_eq!(press(&awaiting, CommandSlot::BACK), SlotPress::Back);
    assert_eq!(placing.back(&dc), basic, "Escape/right-click: returns to BuildMenu");
    assert!(still_valid(placing, &dc, true, &ready));
    assert!(!still_valid(placing, &dc, false, &ready), "not commandable");
    let placed = Conditions::default().with_constructor(Constructor::Idle);
    assert!(!still_valid(placing, &dc, true, &placed), "nothing ready any more");
    let other = busy(DC, Constructor::Ready { kind: BARRACKS });
    assert!(!still_valid(placing, &dc, true, &other), "another building");
    let elsewhere = Selection::new([(id(2), DC)]);
    assert!(
        !still_valid(placing, &elsewhere, true, &ready),
        "another Deployment Center"
    );
    // A Peacekeeper beside it: the Deployment Center's commands are group commands.
    let mut mixed = Selection::new([(id(1), DC), (id(2), PK)]);
    mixed.cycle(true);
    assert_eq!(panel(&mixed, basic, &ready).get(q).map(|s| s.look), Some(Group));
    assert_eq!(recipients(&mixed, PlaceReady(GENERATOR)), [id(1)]);
}

/// `factions/gdo/economy.md > ExtractionFacility > ObjectInterfaceState[ExtractionFacility]`: DefaultState builds the
/// plate (Q); while it is constructed only X (Cancel Construction) shows; once it is ready Q places it and X cancels
/// it. Placement goes back to DefaultState.
#[test]
fn the_extraction_facility_builds_then_places_its_plate() {
    use ButtonLook::Common;
    use PanelCommand::{CancelConstruction, CancelReady, Construct, PlaceReady};
    let ef = Selection::new([(id(3), EF)]);
    let command = PanelButton::Command;
    let (q, x) = (slot(0, 0), CommandSlot::CANCEL_PRODUCTION);
    let idle = Conditions::default().with_constructor(Constructor::Idle);
    let default = panel(&ef, InterfaceState::DefaultState, &idle);
    assert_eq!(shown(&default), [(q, command(Construct(PLATE)), Common)]);
    assert_eq!(default.get(q).map(|s| s.label()), Some("EP"));
    assert_eq!(
        press(&default, q),
        SlotPress::Construct(Construct(PLATE), ConstructionCommand::Construct(PLATE))
    );
    let building = panel(&ef, InterfaceState::DefaultState, &busy(EF, constructing(PLATE)));
    assert_eq!(shown(&building), [(x, command(CancelConstruction), Common)]);
    let ready = busy(EF, Constructor::Ready { kind: PLATE });
    let place = panel(&ef, InterfaceState::DefaultState, &ready);
    assert_eq!(
        shown(&place),
        [
            (q, command(PlaceReady(PLATE)), Common),
            (x, command(CancelReady), Common),
        ]
    );
    assert_eq!(place.get(q).map(|s| s.label()), Some("Place"));
    assert_eq!(press(&place, q), SlotPress::AwaitPlacement(PLATE));
    let placing = InterfaceState::AwaitingPlacement(Placing::new(id(3), PLATE));
    assert_eq!(
        placing.back(&ef),
        InterfaceState::DefaultState,
        "Escape/right-click: returns to DefaultState"
    );
    let awaiting = panel(&ef, placing, &ready);
    assert_eq!(awaiting.get(q).map(|s| s.look), Some(ButtonLook::Pending));
    assert_eq!(awaiting.get(CommandSlot::BACK).map(|s| s.look), Some(ButtonLook::Back));
}

/// No smart-casting (`systems/controls/selection_and_hud.md > SelectionGroup > No smart-casting`, design change 0071):
/// Deployment Centers and Extraction Facilities are Ungroupable, each its own SelectionGroup, and a constructor's
/// command goes to the one whose state the panel shows, never to the other selected ones.
#[test]
fn a_constructors_commands_go_to_the_active_instance_only() {
    use PanelCommand::{CancelConstruction, Construct, PlaceReady};
    let mut two = Selection::new([(id(1), DC), (id(2), DC)]);
    assert_eq!(two.groups().len(), 2, "Ungroupable");
    for command in [Construct(GENERATOR), CancelConstruction, PlaceReady(GENERATOR)] {
        assert_eq!(recipients(&two, command), [id(1)], "{command:?}");
        assert!(!is_common(&two, command));
    }
    two.cycle(true);
    assert_eq!(recipients(&two, Construct(GENERATOR)), [id(2)]);
    let one = Selection::new([(id(1), DC)]);
    assert!(is_common(&one, Construct(GENERATOR)));
    let with_ef = Selection::new([(id(1), DC), (id(3), EF)]);
    assert_eq!(recipients(&with_ef, CancelConstruction), [id(1)]);
    assert!(!offers(DC, Construct(PLATE)) && offers(EF, Construct(PLATE)));
    assert_eq!(menu_of(DC, GENERATOR), Some(BuildMenu::Basic));
    assert_eq!(menu_of(EF, PLATE), None);
}

/// `factions/gdo/structures/deployment_center.md > AwaitingPlacement`: the ghost is snapped under the cursor, centred
/// on it; R turns it a quarter turn clockwise and Shift + R back, so a Barracks (3 × 2) becomes 2 × 3; F and Shift + F
/// mirror it as seen, each undoing itself.
#[test]
fn the_ghost_snaps_under_the_cursor_and_turns() {
    use bevy::math::Vec2;
    use sc_sim::TilePos;

    use crate::editor::orientation::Turn;
    let barracks = Placing::new(id(1), BARRACKS);
    assert_eq!(barracks.size(), Some((3, 2)));
    // Around the middle of tile (10, 7): the 3 × 2 footprint whose centre is nearest.
    let at = |x: f32, y: f32| Vec2::new(x * 64.0, y * 64.0);
    assert_eq!(barracks.origin_under(at(10.5, 7.4)), Some(TilePos::new(9, 6)));
    assert_eq!(barracks.origin_under(at(10.5, 7.6)), Some(TilePos::new(9, 7)));
    assert_eq!(barracks.origin_under(Vec2::NAN), None);
    let turned = barracks.turned(Turn::Clockwise);
    assert_eq!(turned.size(), Some((2, 3)));
    assert_eq!(turned.turned(Turn::CounterClockwise), barracks);
    assert_eq!(turned.origin_under(at(10.4, 7.5)), Some(TilePos::new(9, 6)));
    let flipped = barracks.turned(Turn::FlipHorizontal);
    assert_eq!(flipped.size(), Some((3, 2)));
    assert_eq!(flipped.turned(Turn::FlipHorizontal), barracks);
    assert_eq!(barracks.turned(Turn::FlipVertical).turned(Turn::FlipVertical), barracks);
}
