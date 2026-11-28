//! The supply run's clicks (`factions/gdo/units/supply_chopper.md > ObjectInterfaceState[SupplyChopper]`,
//! `factions/gdo/economy.md > SupplyTower > AwaitingTarget[ScheduleDeliveries] resolution`): a Supply Chopper's
//! right-click, the sim's rule (`sc_sim::orders::resolution::Rule::SupplyRun`), and its R, S and D's targets; a Supply
//! Tower's S's, a station under the GatherResources pointer.

use super::*;

/// A Supply Delivery Station (7), nobody's, as the sim aims it (`Aimed::station`).
fn station() -> MapTarget {
    let mut target = on(CursorTarget::NeutralObject(id(7)), false, Domain::Ground);
    target.aimed = target.aimed.map(|aimed| Aimed { station: true, ..aimed });
    target
}

/// An own Supply Tower in operation (4) with chopper `attached` attached to it, if any, as the sim aims it
/// (`Aimed::supply_tower`).
fn tower(attached: Option<u32>) -> MapTarget {
    let mut target = on(CursorTarget::FriendlyObject(id(4)), true, Domain::Ground);
    target.aimed = target.aimed.map(|aimed| Aimed {
        supply_tower: Some(attached.map(id)),
        ..aimed
    });
    target
}

/// A Supply Chopper (1) that carries Supplies or not, attached to Tower `tower` if any, as the client reads it
/// (`Modes::chopper`).
fn chopper(carrying: bool, tower: Option<u32>) -> UnitCapabilities {
    let modes = Modes::default().with_choppers([id(1)], carrying, tower.map(id));
    UnitCapabilities::of(id(1), ObjectKind::SupplyChopper, &modes)
}

/// The chopper's right-click (`factions/gdo/units/supply_chopper.md > ObjectInterfaceState[SupplyChopper]`): on a
/// station a Pick Up Supplies; on an own Supply Tower a Drop Off Supplies while it carries Supplies, else an Attach to
/// Tower, but a Move to a Tower another chopper is attached to, and to its own Tower its Drop Off; the ground a Move.
/// Their pointers: GatherResources, ReturnResources and Enter (placeholders, `PointerDisplay::of_order`). A Peacekeeper
/// moves to a station, so beside it the preview is Mixed.
#[test]
fn a_choppers_right_click_picks_up_drops_off_or_attaches() {
    use PointerDisplay as D;
    let order = |kind| Some(Resolved::Order(kind));
    let preview = |units: &[UnitCapabilities], target: MapTarget| right_click_display(units, &target);
    let (empty, loaded) = (chopper(false, None), chopper(true, None));
    for unit in [empty, loaded] {
        assert_eq!(
            object_right_click(unit, &station()),
            order(OrderKind::PickUpSupplies(id(7)))
        );
    }
    assert_eq!(preview(&[empty], station()), D::GatherResources);
    assert_eq!(
        object_right_click(loaded, &tower(None)),
        order(OrderKind::DropOffSupplies(id(4)))
    );
    assert_eq!(preview(&[loaded], tower(None)), D::ReturnResources);
    assert_eq!(
        object_right_click(empty, &tower(None)),
        order(OrderKind::AttachToTower(id(4)))
    );
    assert_eq!(preview(&[empty], tower(None)), D::Enter);
    let taken = tower(Some(9));
    assert_eq!(
        object_right_click(empty, &taken),
        order(OrderKind::Move(Target::Object(id(4))))
    );
    assert_eq!(preview(&[loaded], taken), D::Move);
    let home = chopper(true, Some(4));
    assert_eq!(
        object_right_click(home, &tower(Some(1))),
        order(OrderKind::DropOffSupplies(id(4)))
    );
    assert_eq!(
        object_right_click(empty, &MapTarget::ground(P)),
        order(OrderKind::Move(Target::Point(P)))
    );
    let peacekeeper = UnitCapabilities::of(id(2), ObjectKind::Peacekeeper, &Modes::default());
    assert_eq!(
        object_right_click(peacekeeper, &station()),
        order(OrderKind::Move(Target::Object(id(7))))
    );
    assert_eq!(preview(&[empty, peacekeeper], station()), D::Mixed);
}

/// The chopper's R takes a station (`factions/gdo/units/supply_chopper.md > AwaitingTarget[PickUpSupplies]
/// resolution`), its S and D an own Supply Tower (`factions/gdo/units/supply_chopper.md >
/// AwaitingTarget[DropOffSupplies] resolution`, `factions/gdo/units/supply_chopper.md > AwaitingTarget[AttachToTower]
/// resolution`): anything else is an invalid target. Over a Tower another chopper is attached to, S and D show
/// Inactive, the gate refusing the click (`Reject::TowerTaken`); over the chopper's own Tower, its Drop Off. The
/// Tower's S takes a station, its pointer GatherResources (`systems/controls/hud.md > Resolution Rules >
/// AwaitingTarget[CommandType]`), and Inactive elsewhere.
#[test]
fn the_supply_runs_commands_take_a_station_or_an_own_tower() {
    use PanelCommand::{AttachToTower, DropOffSupplies, PickUpSupplies, ScheduleDeliveries};
    use PointerDisplay as D;
    let ground = MapTarget::ground(P);
    assert_eq!(
        target_order(PickUpSupplies, &station()),
        Some(OrderKind::PickUpSupplies(id(7)))
    );
    for target in [tower(None), own(), ground] {
        assert_eq!(target_order(PickUpSupplies, &target), None, "{target:?}");
    }
    for (command, kind) in [
        (DropOffSupplies, OrderKind::DropOffSupplies(id(4))),
        (AttachToTower, OrderKind::AttachToTower(id(4))),
    ] {
        assert_eq!(target_order(command, &tower(None)), Some(kind));
        assert_eq!(
            target_order(command, &tower(Some(9))),
            Some(kind),
            "the gate's to refuse"
        );
        for target in [station(), own(), ground] {
            assert_eq!(target_order(command, &target), None, "{command:?} on {target:?}");
        }
        let awaiting = InterfaceState::awaiting(command);
        let shown = |units: &[UnitCapabilities], target: MapTarget| pointer_display(awaiting, units, &target);
        let empty = chopper(false, None);
        assert_ne!(shown(&[empty], tower(None)), D::Inactive, "{command:?}");
        assert_eq!(shown(&[empty], tower(Some(9))), D::Inactive, "{command:?}: taken");
        assert_ne!(
            shown(&[chopper(true, Some(4))], tower(Some(1))),
            D::Inactive,
            "{command:?}: its own"
        );
        assert_eq!(shown(&[empty], station()), D::Inactive, "{command:?}");
    }

    let schedule = Some(Resolved::Schedule(id(7)));
    assert_eq!(target_resolution(ScheduleDeliveries, &station()), schedule);
    for target in [tower(None), own(), ground] {
        assert_eq!(target_resolution(ScheduleDeliveries, &target), None, "{target:?}");
    }
    let towers = [UnitCapabilities::of(id(4), ObjectKind::SupplyTower, &Modes::default())];
    let awaiting = InterfaceState::awaiting(ScheduleDeliveries);
    assert_eq!(pointer_display(awaiting, &towers, &station()), D::GatherResources);
    assert_eq!(pointer_display(awaiting, &towers, &ground), D::Inactive);
}
