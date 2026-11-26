//! The Repair's clicks and pointers (slice M5-N8): the right-click's Repair, which an Enter beats (design change 0325;
//! `factions/cults/units/recruit.md > Right-Click Resolution`, `factions/syndicate/units/agent.md > Right-Click
//! Resolution`), the sim's one rule, previewed by the pointer; and D Repair's target, whose pointer judges it for the
//! selected units' types.

use super::*;

fn recruit() -> UnitCapabilities {
    UnitCapabilities::of(id(1), ObjectKind::Recruit, &Modes::default())
}
fn militant() -> UnitCapabilities {
    UnitCapabilities::of(id(2), ObjectKind::Militant, &Modes::default())
}
fn agent() -> UnitCapabilities {
    UnitCapabilities::of(id(3), ObjectKind::Agent, &Modes::default())
}

/// One of the player's own objects at id 4, a `kind` below its MaxHP (`Aimed::repair`), as `aimed` makes it.
fn damaged(kind: ObjectKind, aimed: impl FnOnce(Aimed) -> Aimed) -> MapTarget {
    let mut target = on(CursorTarget::FriendlyObject(id(4)), true, Domain::Ground);
    target.aimed = target.aimed.map(|a| {
        aimed(Aimed {
            repair: Some(kind),
            ..a
        })
    });
    target
}
/// A damaged Storage, which takes the Recruits' loads; a damaged Armory, which takes them in; a damaged Laser Turret;
/// and a damaged Tier 1 Tunnel, which takes infantry in and the Agents' loads.
fn storage() -> MapTarget {
    damaged(ObjectKind::Storage, |a| Aimed { drop_off: true, ..a })
}
fn armory() -> MapTarget {
    damaged(ObjectKind::Armory, |a| Aimed {
        accepts: &[ObjectKind::Recruit],
        ..a
    })
}
fn turret() -> MapTarget {
    damaged(ObjectKind::LaserTurret, |a| a)
}
fn tunnel() -> MapTarget {
    damaged(ObjectKind::Tunnel, |a| Aimed {
        transit: &[Transit::Infantry],
        drop_off: true,
        ..a
    })
}

/// "Damaged own building or vehicle (completed) → Repair, **unless the building can be entered** by the selection:
/// then **Enter** wins" (the Recruit's); "Own Tunnel (not carrying resources) → Enter (also when damaged: Enter wins
/// over Repair)", "Damaged own Laser Turret → Repair" (the Agent's). A Recruit with a load drops it off at a damaged
/// Storage, the rule before. Each unit resolves its own: an Agent repairs no Storage and moves to it, a Militant
/// repairs nothing, so either beside a Recruit is Mixed.
#[test]
fn the_right_click_repairs_a_damaged_own_object_unless_it_enters() {
    use PointerDisplay as D;
    let order = |kind| Some(Resolved::Order(kind));
    let (repair, enter) = (order(OrderKind::Repair(id(4))), order(OrderKind::Enter(id(4))));
    let loaded = UnitCapabilities::of(id(1), ObjectKind::Recruit, &Modes::default().with_loaded([id(1)]));
    assert_eq!(object_right_click(recruit(), &storage()), repair);
    assert_eq!(object_right_click(loaded, &storage()), order(OrderKind::DropOff(id(4))));
    assert_eq!(object_right_click(recruit(), &armory()), enter, "Enter wins");
    assert_eq!(object_right_click(agent(), &turret()), repair);
    assert_eq!(object_right_click(agent(), &tunnel()), enter, "Enter wins");
    let preview = |units: &[UnitCapabilities], target: MapTarget| right_click_display(units, &target);
    assert_eq!(preview(&[recruit()], storage()), D::Repair);
    assert_eq!(preview(&[recruit()], armory()), D::Enter);
    assert_eq!(preview(&[agent()], turret()), D::Repair);
    assert_eq!(preview(&[agent()], tunnel()), D::Enter);
    assert_eq!(preview(&[agent()], storage()), D::Move);
    assert_eq!(preview(&[militant()], storage()), D::Move);
    assert_eq!(preview(&[recruit(), militant()], storage()), D::Mixed);
    assert_eq!(preview(&[recruit(), agent()], storage()), D::Mixed);
    assert_eq!(PointerDisplay::of_order(OrderKind::Repair(id(4))), D::Repair);
}

/// `factions/cults/units/recruit.md > Recruit ObjectInterfaceState > DefaultState commands`: "**D: Repair** — [...]
/// Left-click a damaged own building or vehicle to repair it"; the Agent's "a damaged own Tunnel or Laser Turret". A
/// click on a damaged own object gives a Repair, one the selection may enter too
/// (`factions/cults/units/recruit.md > Right-Click Resolution`: "Repair those with **D: Repair**"), under the Repair pointer while one of the selected units repairs its kind
/// (`sc_sim::repair::repairs`), Inactive while none does: the gate would refuse it (`Reject::CantRepair`). Anything
/// else is an invalid target (design change 0065): an own object at full HP or under construction, an enemy, a patch,
/// the ground.
#[test]
fn d_repair_takes_a_damaged_own_object_that_its_units_repair() {
    use PointerDisplay as D;
    let awaiting = InterfaceState::awaiting(PanelCommand::Repair);
    let repair = Some(Resolved::Order(OrderKind::Repair(id(4))));
    for target in [storage(), armory(), turret(), tunnel()] {
        assert_eq!(repair_target(&target), Some(id(4)), "{target:?}");
        assert_eq!(target_resolution(PanelCommand::Repair, &target), repair, "{target:?}");
    }
    let shown = |units: &[UnitCapabilities], target: MapTarget| pointer_display(awaiting, units, &target);
    assert_eq!(shown(&[recruit()], storage()), D::Repair);
    assert_eq!(shown(&[recruit()], armory()), D::Repair, "no Enter while D awaits");
    assert_eq!(
        shown(&[militant(), recruit()], storage()),
        D::Repair,
        "the Recruit repairs it"
    );
    assert_eq!(shown(&[agent()], tunnel()), D::Repair);
    assert_eq!(shown(&[agent()], turret()), D::Repair);
    assert_eq!(shown(&[agent()], storage()), D::Inactive, "no Agent repairs a Storage");
    assert_eq!(shown(&[militant()], storage()), D::Inactive);
    let full = on(CursorTarget::FriendlyObject(id(4)), true, Domain::Ground);
    let mut site = full;
    site.aimed = site.aimed.map(|a| Aimed {
        site: Some(ObjectKind::Storage),
        ..a
    });
    for invalid in [full, site, enemy(), enemy_structure(), patch(), MapTarget::ground(P)] {
        assert_eq!(repair_target(&invalid), None, "{invalid:?}");
        assert_eq!(target_resolution(PanelCommand::Repair, &invalid), None, "{invalid:?}");
        assert_eq!(shown(&[recruit()], invalid), D::Inactive, "{invalid:?}");
    }
}
