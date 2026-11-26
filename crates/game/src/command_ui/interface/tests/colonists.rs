//! The Colonists' panels as plain data (slice M3-J1, design change 0124): the Beacon's and its Warp In menus, the
//! Drone's and which Drone a Deploy goes to, the Cancel of whatever transforms, and why a greyed-out button is (design
//! change 0162). The gathering commands (slice M3-J2) are `gathering`'s.

use std::collections::BTreeSet;

use sc_sim::ConstructionCommand;

use super::*;
use crate::command_ui::resolution::UnitCapabilities;
use crate::editor::orientation::Turn;
use crate::selection::GroupType;

const BEACON: ObjectKind = ObjectKind::Beacon;
const DRONE: ObjectKind = ObjectKind::Drone;
const ROBO_PORTAL: ObjectKind = ObjectKind::RoboPortal;
const X: CommandSlot = CommandSlot::CANCEL_PRODUCTION;

/// `factions/colonists/economy.md > Beacon > Beacon ObjectInterfaceState`: DefaultState opens Warp In (Basic) in Q and
/// Warp In (Advanced) in W, "like the Deployment Center's two build menus"; Basic has the three buildings in the game
/// in Q, W and E and the three refineries unavailable in A, S and D; Advanced eight buildings unavailable; each menu
/// has Back in Z. A building enters its placement on the Beacon, which goes back to its menu; an unavailable one does
/// nothing and says it isn't in the game yet.
#[test]
fn the_beacon_opens_two_warp_in_menus() {
    use ButtonLook::{Back, Common, Unavailable};
    use PanelCommand::{BuildMenu as Menu, Planned as P, WarpIn};
    let beacon = Selection::new([(id(1), BEACON)]);
    let conditions = Conditions::default();
    let c = PanelButton::Command;
    let default = panel(&beacon, InterfaceState::DefaultState, &conditions);
    assert_eq!(
        shown(&default),
        [
            (slot(0, 0), c(Menu(BuildMenu::Basic)), Common),
            (slot(0, 1), c(Menu(BuildMenu::Advanced)), Common),
        ]
    );
    assert_eq!(press(&default, slot(0, 1)), SlotPress::OpenMenu(BuildMenu::Advanced));

    let basic = InterfaceState::BuildMenu(BuildMenu::Basic);
    let menu = panel(&beacon, basic, &conditions);
    assert_eq!(
        shown(&menu),
        [
            (slot(0, 0), c(WarpIn(ObjectKind::EgressNode)), Common),
            (slot(0, 1), c(WarpIn(ROBO_PORTAL)), Common),
            (slot(0, 2), c(WarpIn(ObjectKind::BioPortal)), Common),
            (slot(1, 0), c(P(Planned::AlloyFurnace)), Unavailable),
            (slot(1, 1), c(P(Planned::EssenceRefinery)), Unavailable),
            (slot(1, 2), c(P(Planned::FluxReactor)), Unavailable),
            (CommandSlot::BACK, PanelButton::Back, Back),
        ]
    );
    assert_eq!(press(&menu, slot(0, 1)), SlotPress::AwaitWarpIn(ROBO_PORTAL));
    assert_eq!(press(&menu, slot(1, 0)), SlotPress::Nothing);
    assert_eq!(menu.get(slot(1, 0)).and_then(|s| s.why), Some(WhyUnavailable::NotYet));
    assert_eq!(menu.get(slot(0, 1)).and_then(|s| s.why), None);
    assert_eq!(basic.back(&beacon), InterfaceState::DefaultState);
    assert!(still_valid(basic, &beacon, true, &conditions));

    let placing = InterfaceState::AwaitingPlacement(Placing::warp_in(id(1), ROBO_PORTAL));
    assert_eq!(placing.back(&beacon), basic, "back to its menu");
    let placed = panel(&beacon, placing, &conditions);
    assert_eq!(placed.get(slot(0, 1)).map(|s| s.look), Some(ButtonLook::Pending));
    assert_eq!(
        placed.get(slot(0, 0)).and_then(|s| s.why),
        Some(WhyUnavailable::Waiting)
    );
    assert!(still_valid(placing, &beacon, true, &conditions));

    let advanced = panel(&beacon, InterfaceState::BuildMenu(BuildMenu::Advanced), &conditions);
    let looks: Vec<ButtonLook> = shown(&advanced).into_iter().map(|(_, _, look)| look).collect();
    assert_eq!(looks, [vec![Unavailable; 8], vec![Back]].concat());
    assert!(
        CommandSlot::all()
            .take(8)
            .all(|s| press(&advanced, s) == SlotPress::Nothing)
    );
    assert_eq!(menu_of(BEACON, ROBO_PORTAL), Some(BuildMenu::Basic));
    assert_eq!(menu_of(BEACON, ObjectKind::Generator), None);
}

/// `factions/colonists/economy.md > Warp In > Constructs List`: a Warp In the bank can't pay for is disabled, and says
/// so (design change 0162); where the building would stand is the ghost's to show, not the button's.
#[test]
fn a_warp_in_the_bank_cannot_pay_for_says_why() {
    let beacon = Selection::new([(id(1), BEACON)]);
    let robo = PanelCommand::WarpIn(ROBO_PORTAL);
    let poor = Conditions::default().refusing(robo, Reject::InsufficientCrystals);
    let menu = panel(&beacon, InterfaceState::BuildMenu(BuildMenu::Basic), &poor);
    let w = menu.get(slot(0, 1)).unwrap();
    assert_eq!(w.look, ButtonLook::Disabled);
    assert_eq!(w.why, Some(WhyUnavailable::Refused(Reject::InsufficientCrystals)));
    assert_eq!(press(&menu, slot(0, 1)), SlotPress::Nothing);
    // A Train alike, the reason the one the gate gives.
    let portal = Selection::new([(id(2), ROBO_PORTAL)]);
    let train = PanelCommand::Train(DRONE);
    let full = Conditions::default().refusing(train, Reject::QueueFull);
    let production = panel(&portal, InterfaceState::DefaultState, &full);
    assert_eq!(
        production.get(slot(0, 0)).and_then(|s| s.why),
        Some(WhyUnavailable::Refused(Reject::QueueFull))
    );
}

/// `factions/colonists/units/drone.md > Drone ObjectInterfaceState` (design changes 0124 and 0398): Q Move, W
/// Deploy, S Set Drop-off (`gathering`), X Stop, and no A, the Drones alone (no selected unit can attack: design change
/// 0399). W enters AwaitingPlacement for a Beacon (a StateOnlyTransition), the Deploy marked and Back in Z, which
/// returns to DefaultState; the placement holds while the ActiveGroup is Drones. The Beacon doesn't turn.
#[test]
fn the_drone_deploys_with_w() {
    use ButtonLook::Common;
    use PanelCommand::{Deploy, Move, SetDropOff, Stop};
    let drones = Selection::new([(id(1), DRONE), (id(2), DRONE)]);
    let conditions = Conditions::default();
    let default = panel(&drones, InterfaceState::DefaultState, &conditions);
    let c = PanelButton::Command;
    assert_eq!(
        shown(&default),
        [
            (slot(0, 0), c(Move), Common),
            (slot(0, 1), c(Deploy), Common),
            (slot(1, 1), c(SetDropOff), Common),
            (X, c(Stop), Common)
        ]
    );
    assert_eq!(press(&default, slot(0, 1)), SlotPress::AwaitDeploy);
    assert_eq!(press(&default, slot(1, 0)), SlotPress::Nothing, "no A");
    assert_eq!(press(&default, X), SlotPress::Issue(Stop, OrderKind::Stop));

    let placing = InterfaceState::AwaitingPlacement(Placing::deploy());
    let placed = panel(&drones, placing, &conditions);
    assert_eq!(placed.get(slot(0, 1)).map(|s| s.look), Some(ButtonLook::Pending));
    assert_eq!(placed.get(CommandSlot::BACK).map(|s| s.look), Some(ButtonLook::Back));
    assert_eq!(placing.back(&drones), InterfaceState::DefaultState);
    assert!(still_valid(placing, &drones, true, &conditions));
    let beacon = Selection::new([(id(3), BEACON)]);
    assert!(!still_valid(placing, &beacon, true, &conditions), "no Drones");
    assert_eq!(Placing::deploy().turned(Turn::Clockwise), Placing::deploy());
    assert_eq!(Placing::deploy().command(), Deploy);
    assert_eq!(Placing::deploy().size(), Some((2, 2)));
}

/// Design change 0125: a Deploy goes to one Drone of the ActiveGroup, "the first in selection order (the
/// SelectionGroup's portrait order in the SelectionPanel)": a plain click's to the first one, whatever it has; a
/// Shift-click's to "the first Drone that has no Deploy in its queue yet", none once they all have one. Never to a
/// Drone outside the ActiveGroup.
#[test]
fn a_deploy_goes_to_one_drone_in_portrait_order() {
    let mut selection = Selection::new([(id(7), DRONE), (id(3), DRONE), (id(5), DRONE), (id(9), BEACON)]);
    assert_eq!(selection.active().map(|k| k.kind), Some(DRONE));
    let deploying = |ids: &[u32]| -> BTreeSet<ObjectId> { ids.iter().map(|&n| id(n)).collect() };
    assert_eq!(deploy_recipient(&selection, false, &deploying(&[])), Some(id(3)));
    assert_eq!(deploy_recipient(&selection, false, &deploying(&[3, 5])), Some(id(3)));
    assert_eq!(deploy_recipient(&selection, true, &deploying(&[])), Some(id(3)));
    assert_eq!(deploy_recipient(&selection, true, &deploying(&[3])), Some(id(5)));
    assert_eq!(deploy_recipient(&selection, true, &deploying(&[3, 5])), Some(id(7)));
    assert_eq!(deploy_recipient(&selection, true, &deploying(&[3, 5, 7])), None);
    selection.cycle(true);
    assert_eq!(selection.active().map(|k| k.kind), Some(BEACON));
    assert_eq!(
        deploy_recipient(&selection, false, &deploying(&[])),
        None,
        "the Beacon's group"
    );
}

/// Design change 0116: "A Beacon under deployment and a building under Warp In show only the Cancel command in their
/// CommandPanel (no rally point, no other commands)", "Cancel's hotkey: X", issued at once: the panel of the group of
/// a type's unfinished buildings (design change 0185). Their menus, placements and targets no longer hold, and a
/// right-click resolves nothing for them. (M5-S6) But a portal under Warp In, which will produce units, adds C: Set
/// Rally Point next to Cancel (design change 0396: "Its unfinished panel adds **C: Set Rally Point**, next to Cancel.
/// Right-clicking ground or an object sets it, as on the finished building"): C awaits its target, which still holds
/// while it warps in, and its right-click sets its rally point. The Beacon deploying and the Egress Node, which
/// produce nothing, show Cancel alone.
#[test]
fn whatever_transforms_shows_cancel_and_a_portal_its_rally_point() {
    let transforming = Conditions::default();
    let set_rally = PanelCommand::SetRallyPoint;
    for kind in [BEACON, ROBO_PORTAL, ObjectKind::BioPortal, ObjectKind::EgressNode] {
        let portal = matches!(kind, ROBO_PORTAL | ObjectKind::BioPortal);
        let selection = Selection::new([(id(1), GroupType::unfinished(kind))]);
        let shown_panel = panel(&selection, InterfaceState::DefaultState, &transforming);
        let cancel = PanelButton::Command(PanelCommand::CancelTransform);
        let mut expected = vec![(X, cancel, ButtonLook::Common)];
        if portal {
            expected.push((
                CommandSlot::RALLY_POINT,
                PanelButton::Command(set_rally),
                ButtonLook::Common,
            ));
        }
        assert_eq!(shown(&shown_panel), expected, "{kind:?}");
        assert_eq!(shown_panel.get(X).unwrap().label(), "Cancel");
        assert_eq!(
            press(&shown_panel, X),
            SlotPress::Construct(PanelCommand::CancelTransform, ConstructionCommand::CancelTransform)
        );
        let c = press(&shown_panel, CommandSlot::RALLY_POINT);
        assert_eq!(
            c,
            if portal {
                SlotPress::AwaitTarget(set_rally)
            } else {
                SlotPress::Nothing
            },
            "{kind:?}"
        );
    }
    let beacon = Selection::new([(id(1), BEACON)]);
    let deploying = Selection::new([(id(1), GroupType::unfinished(BEACON))]);
    let menu = InterfaceState::BuildMenu(BuildMenu::Basic);
    assert!(still_valid(menu, &beacon, true, &transforming));
    assert!(!still_valid(menu, &deploying, true, &transforming));
    let placing = InterfaceState::AwaitingPlacement(Placing::warp_in(id(1), ROBO_PORTAL));
    assert!(still_valid(placing, &beacon, true, &transforming));
    assert!(!still_valid(placing, &deploying, true, &transforming));
    let portal = Selection::new([(id(2), ROBO_PORTAL)]);
    let warping = Selection::new([(id(2), GroupType::unfinished(ROBO_PORTAL))]);
    let rally = InterfaceState::awaiting(set_rally);
    assert!(still_valid(rally, &portal, true, &transforming));
    assert!(
        still_valid(rally, &warping, true, &transforming),
        "a rally point while it warps in"
    );
    assert!(
        !still_valid(rally, &deploying, true, &transforming),
        "a Beacon has none"
    );

    let modes = Modes::default().with_transforming([id(1), id(2)]);
    let warping_in = UnitCapabilities::of(id(2), ROBO_PORTAL, &modes).template;
    assert_eq!(
        warping_in,
        Some(InterfaceTemplate::Production),
        "its right-click rallies"
    );
    let working = UnitCapabilities::of(id(3), ROBO_PORTAL, &modes).template;
    assert_eq!(working, Some(InterfaceTemplate::Production));
    assert_eq!(
        UnitCapabilities::of(id(1), ObjectKind::EgressNode, &modes).template,
        None
    );
}

/// Design change 0185: 2 finished Bio Portals and 1 warping in are two groups, each with its panel. The finished
/// group's commands go to the finished portals only, group commands (the unfinished one doesn't offer them), beside
/// the units not in the game yet (design change 0124, slice M3-J2); Tab to the other group shows X Cancel, which goes
/// to the portal warping in. (M5-S6) C, which every portal offers now (design change 0396), is a common command in
/// either group: it goes to all three.
#[test]
fn finished_and_unfinished_portals_have_their_own_panels() {
    use PanelCommand::{CancelProduction, CancelTransform, Planned as P, SetRallyPoint, Train};
    let bio = ObjectKind::BioPortal;
    let sabre = ObjectKind::SabreLegionnaire;
    let mut portals = Selection::new([
        (id(1), bio.into()),
        (id(2), bio.into()),
        (id(3), GroupType::unfinished(bio)),
    ]);
    let conditions = Conditions::default();
    let finished = panel(&portals, InterfaceState::DefaultState, &conditions);
    let c = PanelButton::Command;
    assert_eq!(
        shown(&finished),
        [
            (slot(0, 0), c(Train(sabre)), ButtonLook::Group),
            (slot(0, 1), c(P(Planned::Lancer)), ButtonLook::Unavailable),
            (slot(0, 2), c(P(Planned::PsionicUnit)), ButtonLook::Unavailable),
            (X, c(CancelProduction), ButtonLook::Group),
            (CommandSlot::RALLY_POINT, c(SetRallyPoint), ButtonLook::Common),
        ]
    );
    for command in [Train(sabre), CancelProduction] {
        assert_eq!(recipients(&portals, command), [id(1), id(2)], "{command:?}");
    }
    assert!(portals.cycle(true));
    let unfinished = panel(&portals, InterfaceState::DefaultState, &conditions);
    assert_eq!(
        shown(&unfinished),
        [
            (X, c(CancelTransform), ButtonLook::Group),
            (CommandSlot::RALLY_POINT, c(SetRallyPoint), ButtonLook::Common),
        ]
    );
    assert_eq!(recipients(&portals, CancelTransform), [id(3)]);
    assert_eq!(recipients(&portals, SetRallyPoint), [id(1), id(2), id(3)]);
    assert_eq!(press(&unfinished, slot(0, 0)), SlotPress::Nothing);
}

/// Every command a Colonists type may show, in any state (`commands`), which the panel's conditions ask the gate
/// about: the Beacon's menus and its three Warp Ins; a portal's production; the Drone's four, no Cancel (it never
/// transforms itself, its Beacon does); the Egress Node's F. An unfinished Beacon, portal or node, in a group of its
/// own (design change 0185), shows its Cancel (an unfinished type has both Cancels, the panel shows the one it is: a
/// site's, slice M4-J1), and a portal its C besides (slice M5-S6, design change 0396). They go to the ActiveGroup only
/// (`PanelCommand::instance_only`), but the Drone's.
#[test]
fn the_colonists_commands() {
    use PanelCommand::{
        BuildMenu as Menu, CancelTransform, Deploy, Move, SelectReturningDrones, SetDropOff, Stop, WarpIn,
    };
    let sorted = |mut commands: Vec<PanelCommand>| {
        commands.sort_unstable();
        commands
    };
    let beacon = vec![
        Menu(BuildMenu::Basic),
        Menu(BuildMenu::Advanced),
        WarpIn(ObjectKind::EgressNode),
        WarpIn(ROBO_PORTAL),
        WarpIn(ObjectKind::BioPortal),
    ];
    assert_eq!(commands(BEACON), sorted(beacon));
    assert!(commands(ROBO_PORTAL).contains(&PanelCommand::Train(DRONE)));
    assert!(!commands(ROBO_PORTAL).contains(&CancelTransform));
    for kind in [BEACON, ROBO_PORTAL, ObjectKind::EgressNode] {
        let cancels = [PanelCommand::CancelConstruction, CancelTransform];
        // (M5-S6) And a portal's C (design change 0396).
        let rally = (kind == ROBO_PORTAL).then_some(PanelCommand::SetRallyPoint);
        assert_eq!(
            commands(GroupType::unfinished(kind)),
            sorted(rally.into_iter().chain(cancels).collect()),
            "{kind:?}: either Cancel, a site's too"
        );
        assert!(offers(GroupType::unfinished(kind), CancelTransform) && !offers(kind, CancelTransform));
    }
    assert!(!offers(GroupType::unfinished(ROBO_PORTAL), PanelCommand::Train(DRONE)));
    assert_eq!(commands(DRONE), sorted(vec![Move, Deploy, Stop, SetDropOff]));
    assert_eq!(commands(ObjectKind::EgressNode), [SelectReturningDrones]);
    for command in [WarpIn(ROBO_PORTAL), Deploy, CancelTransform, SelectReturningDrones] {
        assert!(command.instance_only(), "{command:?}");
    }
    assert!(
        !SetDropOff.instance_only(),
        "every selected Drone's (question Q0040 point 9)"
    );
    assert!(Deploy.queues() && !WarpIn(ROBO_PORTAL).queues());
    assert!(!PanelCommand::Planned(Planned::Skylance).has_sim_command());
    assert_eq!(Planned::Skylance.name(), "Skylance");
    assert_eq!(
        [
            WarpIn(ObjectKind::EgressNode),
            WarpIn(ROBO_PORTAL),
            WarpIn(ObjectKind::BioPortal)
        ]
        .map(PanelCommand::label),
        ["EN", "RP", "BP"]
    );
}
