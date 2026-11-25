//! The Colonists' gathering clicks and pointers (slice M3-J2): `systems/controls/selection_and_hud.md >
//! PointerDisplayType > Resolution Rules > DefaultState`'s GatherResources and ReturnResources, resolved per unit
//! (design change 0063), and the Drones' Set Drop-off (`factions/colonists/units/drone.md > Drone
//! ObjectInterfaceState > S: Set Drop-off Location`).

use sc_sim::Ability;

use super::*;

/// "Cursor over resource (resource gatherer selected): GatherResources"; "Cursor over drop-off point (resource
/// gatherer carrying resources): ReturnResources". An empty Drone moves to its node (design change 0140): Move. Each
/// unit resolves its own right-click, so Drones with a combat unit, or loaded with empty ones over the node, would do
/// different things: Mixed (`systems/controls/selection_and_hud.md > PointerDisplayType > Mixed Right-Click Preview`).
#[test]
fn a_gatherer_previews_gathering_and_returning() {
    use PointerDisplay as D;
    let preview = |units: &[UnitCapabilities], target: MapTarget| right_click_display(units, &target);
    assert_eq!(preview(&[DRONE, LOADED_DRONE], patch()), D::GatherResources);
    assert_eq!(preview(&[LOADED_DRONE], own_node()), D::ReturnResources);
    assert_eq!(preview(&[DRONE], own_node()), D::Move, "nothing to return");
    assert_eq!(preview(&[LOADED_DRONE, DRONE], own_node()), D::Mixed);
    assert_eq!(
        preview(&[DRONE, ARMED], patch()),
        D::Mixed,
        "the Drones gather, the others move"
    );
    assert_eq!(preview(&[ARMED], patch()), D::Move);
    assert_eq!(preview(&[DRONE], enemy()), D::Move, "it follows");
    assert_eq!(preview(&[DRONE, ARMED], enemy()), D::Mixed, "the others attack");
    assert_eq!(PointerDisplay::of_order(OrderKind::Gather(id(8))), D::GatherResources);
    assert_eq!(PointerDisplay::of_order(OrderKind::DropOff(id(3))), D::ReturnResources);
}

/// `> S: Set Drop-off Location`: "Left-click an Egress Node or own refinery sets it as the selected Drone(s)'
/// AssignedDropOff": a click on the player's own node gives the ability, anything else nothing (an invalid target,
/// design change 0065). The pointer shows ReturnResources over the node (placeholder: no rule for
/// AwaitingTarget\[SetDropOffLocation\]), Inactive elsewhere. A node under Warp In is no drop-off yet (`Aimed::drop_off`
/// is false, as the gate has it).
#[test]
fn set_drop_off_takes_an_own_node() {
    use PointerDisplay as D;
    let awaiting = InterfaceState::awaiting(PanelCommand::SetDropOff);
    let set = Resolved::Ability(Ability::SetDropOff(id(3)));
    assert_eq!(target_resolution(PanelCommand::SetDropOff, &own_node()), Some(set));
    assert_eq!(drop_off_target(&own_node()), Some(id(3)));
    assert_eq!(pointer_display(awaiting, &[DRONE], &own_node()), D::ReturnResources);
    let ground = MapTarget::ground(P);
    for invalid in [patch(), enemy(), enemy_structure(), own(), ground] {
        assert_eq!(
            target_resolution(PanelCommand::SetDropOff, &invalid),
            None,
            "{invalid:?}"
        );
        assert_eq!(
            pointer_display(awaiting, &[DRONE], &invalid),
            D::Inactive,
            "{invalid:?}"
        );
    }
    assert_eq!(PointerDisplay::of(set), D::ReturnResources);
}

/// The Egress Node has no right-click (`factions/colonists/economy.md > Egress Node > Egress Node
/// ObjectInterfaceState`: only F): selected with Drones, it leaves them to theirs.
#[test]
fn the_egress_node_resolves_nothing() {
    let node = UnitCapabilities::of(id(3), ObjectKind::EgressNode, &Modes::default());
    assert_eq!(node.template, Some(InterfaceTemplate::EgressNode));
    assert_eq!(object_right_click(node, &patch()), None);
    assert_eq!(object_right_click(node, &MapTarget::ground(P)), None);
    assert_eq!(
        right_click_display(&[node, DRONE], &patch()),
        PointerDisplay::GatherResources
    );
}
