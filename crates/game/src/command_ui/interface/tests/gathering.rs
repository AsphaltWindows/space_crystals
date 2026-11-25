//! The Colonists' gathering panels as plain data (slice M3-J2, design change 0124): the Drone's S, Set Drop-off, and
//! its targeting; the Egress Node's F, Select Returning Drones, shown only while there are Drones to select; and the
//! portals' units not in the game yet, in their slots and unavailable.

use super::*;

const DRONE: ObjectKind = ObjectKind::Drone;
const NODE: ObjectKind = ObjectKind::EgressNode;
const SABRE: ObjectKind = ObjectKind::SabreLegionnaire;
const S: (u8, u8) = (1, 1);
const F: (u8, u8) = (1, 3);

/// `factions/colonists/units.md > Drone ObjectInterfaceState > S: Set Drop-off Location`: "enters
/// AwaitingTarget\[SetDropOffLocation\] (StateOnlyTransition)", where S is marked, the other commands wait and Z is
/// Back. It goes to every selected Drone (question Q0040 point 9): a group command beside the Sabres, which don't
/// have it, and the targeting holds while the ActiveGroup is Drones, not once it is the Sabres.
#[test]
fn the_drones_s_awaits_a_drop_off() {
    let s = slot(S.0, S.1);
    let drones = Selection::new([(id(1), DRONE), (id(2), DRONE), (id(3), SABRE)]);
    let conditions = Conditions::default();
    let default = panel(&drones, InterfaceState::DefaultState, &conditions);
    let shown = default.get(s).unwrap();
    assert_eq!(
        (shown.button, shown.look, shown.label()),
        (
            PanelButton::Command(PanelCommand::SetDropOff),
            ButtonLook::Group,
            "Drop"
        )
    );
    assert_eq!(press(&default, s), SlotPress::AwaitTarget(PanelCommand::SetDropOff));
    assert_eq!(recipients(&drones, PanelCommand::SetDropOff), [id(1), id(2)]);

    let awaiting = InterfaceState::awaiting(PanelCommand::SetDropOff);
    let targeting = panel(&drones, awaiting, &conditions);
    let look = |slot| targeting.get(slot).map(|s| s.look);
    assert_eq!(look(s), Some(ButtonLook::Pending));
    assert_eq!(look(slot(0, 0)), Some(ButtonLook::Disabled));
    assert_eq!(look(CommandSlot::BACK), Some(ButtonLook::Back));
    assert_eq!(awaiting.back(&drones), InterfaceState::DefaultState);
    assert!(!PanelCommand::SetDropOff.queues(), "a Shift-click ends it too");
    assert!(still_valid(awaiting, &drones, true, &conditions));
    let mut sabres = drones.clone();
    sabres.cycle(true);
    assert_eq!(sabres.active().map(|k| k.kind), Some(SABRE));
    assert!(!still_valid(awaiting, &sabres, true, &conditions));
}

/// `factions/colonists/economy.md > Egress Node > Egress Node ObjectInterfaceState > F: Select Returning Drones`:
/// F, its one command, "Only available if at least one such Drone exists": hidden without, and shown, pressable, with
/// them. Pressing it is a StateOnlyTransition that selects them.
#[test]
fn the_egress_nodes_f_shows_only_with_drones_to_select() {
    let f = slot(F.0, F.1);
    let node = Selection::new([(id(4), NODE)]);
    let none = panel(&node, InterfaceState::DefaultState, &Conditions::default());
    assert!(none.is_empty(), "{none:?}");
    assert_eq!(press(&none, f), SlotPress::Nothing);
    let returning = Conditions::default().with_returning(vec![id(1), id(2)]);
    assert_eq!(returning.returning(), [id(1), id(2)]);
    let shown = panel(&node, InterfaceState::DefaultState, &returning);
    let slots: Vec<_> = shown.slots().filter_map(|(slot, s)| s.map(|s| (slot, s))).collect();
    assert_eq!(slots.len(), 1);
    let (at, button) = slots[0];
    assert_eq!(
        (at, button.button, button.look, button.label()),
        (
            f,
            PanelButton::Command(PanelCommand::SelectReturningDrones),
            ButtonLook::Common,
            "Drones"
        )
    );
    assert_eq!(press(&shown, f), SlotPress::SelectReturningDrones);
    assert_eq!(grid(NODE)[slot_index(f)], Some(PanelCommand::SelectReturningDrones));
}

/// Design change 0124: "Robo Portal: Q Drone, W Scout, X, C. Bio Portal: Q Sabre, W Lancer, E psionic unit, X, C";
/// "Buildings and units whose costs are TBD show in their slot, unavailable": pressing one does nothing and says it
/// isn't in the game yet (design change 0162).
#[test]
fn the_portals_show_their_units_not_in_the_game_yet() {
    use PanelCommand::Planned as P;
    let cases = [
        (ObjectKind::RoboPortal, vec![(slot(0, 1), Planned::Scout, "ST")]),
        (
            ObjectKind::BioPortal,
            vec![
                (slot(0, 1), Planned::Lancer, "LA"),
                (slot(0, 2), Planned::PsionicUnit, "PU"),
            ],
        ),
    ];
    for (portal, planned) in cases {
        let selection = Selection::new([(id(5), portal)]);
        let shown = panel(&selection, InterfaceState::DefaultState, &Conditions::default());
        for (at, unit, label) in planned {
            let button = shown.get(at).unwrap();
            assert_eq!(
                (button.button, button.look, button.why, button.label()),
                (
                    PanelButton::Command(P(unit)),
                    ButtonLook::Unavailable,
                    Some(WhyUnavailable::NotYet),
                    label
                ),
                "{portal:?}"
            );
            assert_eq!(press(&shown, at), SlotPress::Nothing);
            assert!(!offers(portal, P(unit)));
        }
        assert_eq!(shown.get(slot(0, 3)), None, "{portal:?}: R stays free");
    }
    assert_eq!(
        [Planned::Scout, Planned::Lancer, Planned::PsionicUnit].map(Planned::name),
        ["Scout", "Lancer", "Psionic unit"]
    );
}
