//! (M6-GH1) An aircraft's right-click on an own Airfield (`systems/mechanics/aircraft_landing.md > Assigned base`:
//! "select the aircraft and **right-click the other Airfield**"; `systems/mechanics/aircraft_landing.md > Assigned
//! base`: "Right-clicking its own Airfield does the same"): a Rebase onto another, a Return to Base onto its own, under
//! the Enter pointer (a placeholder, `PointerDisplay::of_order`); a unit that isn't an aircraft moves there. Out of
//! ammo, a Hornet attacks nothing.

use super::*;

/// An own Airfield in operation at `n`, as the sim aims it (`Aimed::airfield`).
fn airfield(n: u32) -> MapTarget {
    let mut target = on(CursorTarget::FriendlyObject(id(n)), true, Domain::Ground);
    target.aimed = target.aimed.map(|aimed| Aimed {
        airfield: true,
        ..aimed
    });
    target
}

/// A Hornet (1) assigned to the Airfield `base`, if any, with `ammo` missiles left, as the client reads it.
fn hornet(base: Option<u32>, ammo: u32) -> UnitCapabilities {
    let modes = Modes::default().with_aircraft([id(1)], base.map(id), ammo);
    UnitCapabilities::of(id(1), ObjectKind::Hornet, &modes)
}

/// A Hornet assigned to Airfield 4: its right-click on Airfield 5 is a Rebase, on 4 a Return to Base, both under the
/// Enter pointer; an unassigned one's on 4 a Rebase. A Peacekeeper moves to an Airfield, so beside it the Hornet's
/// preview is Mixed. With no missile left the Hornet's right-click on an enemy is no Attack (it follows it, as a unit
/// that can't attack does: `systems/controls/interface.md > RightClickResolution`).
#[test]
fn an_aircrafts_right_click_on_an_own_airfield_rebases_or_returns() {
    use PointerDisplay as D;
    let preview = |units: &[UnitCapabilities], target: MapTarget| right_click_display(units, &target);
    let order = |kind| Some(Resolved::Order(kind));
    let based = hornet(Some(4), 5);
    assert_eq!(object_right_click(based, &airfield(5)), order(OrderKind::Rebase(id(5))));
    assert_eq!(object_right_click(based, &airfield(4)), order(OrderKind::ReturnToBase));
    assert_eq!(preview(&[based], airfield(5)), D::Enter);
    assert_eq!(preview(&[based], airfield(4)), D::Enter);
    assert_eq!(
        object_right_click(hornet(None, 5), &airfield(4)),
        order(OrderKind::Rebase(id(4)))
    );
    let peacekeeper = UnitCapabilities::of(id(2), ObjectKind::Peacekeeper, &Modes::default());
    assert_eq!(
        object_right_click(peacekeeper, &airfield(4)),
        order(OrderKind::Move(Target::Object(id(4))))
    );
    assert_eq!(preview(&[based, peacekeeper], airfield(5)), D::Mixed);
    assert_eq!(
        object_right_click(based, &own()),
        order(OrderKind::Move(Target::Object(id(2))))
    );

    assert_eq!(
        object_right_click(hornet(Some(4), 2), &enemy()),
        order(OrderKind::Attack(id(6)))
    );
    let empty = hornet(Some(4), 0);
    assert!(!empty.may_attack());
    let on_enemy = object_right_click(empty, &enemy());
    assert!(
        !matches!(on_enemy, Some(Resolved::Order(OrderKind::Attack(_)))),
        "{on_enemy:?}"
    );
}
