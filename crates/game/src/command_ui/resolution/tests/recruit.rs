//! The Recruit's clicks and pointers (slice M5-J1): `factions/cults/units/recruit.md > Right-Click Resolution`, the
//! sim's one rule, previewed by the pointer (design change 0322's Assist pointer, 0347 point 4's site that no one
//! enters), and S Assist Construction's target.

use super::*;

const RECRUIT_KIND: ObjectKind = ObjectKind::Recruit;

/// A Recruit, empty-handed, as the client reads it from its type; and one carrying Space Crystals.
fn recruit() -> UnitCapabilities {
    UnitCapabilities::of(id(1), RECRUIT_KIND, &Modes::default())
}
fn loaded_recruit() -> UnitCapabilities {
    UnitCapabilities::of(id(1), RECRUIT_KIND, &Modes::default().with_loaded([id(1)]))
}

/// One of the player's own buildings at id 4, as `aimed` makes it.
fn own_building(aimed: impl FnOnce(Aimed) -> Aimed) -> MapTarget {
    let mut building = on(CursorTarget::FriendlyObject(id(4)), true, Domain::Ground);
    building.aimed = building.aimed.map(aimed);
    building
}
/// An own Storage under construction, which Recruits build from outside.
fn storage_site() -> MapTarget {
    own_building(|aimed| Aimed {
        site: Some(ObjectKind::Storage),
        ..aimed
    })
}
/// An own Armory under construction: it takes no one in yet (`holding::admits`: in operation only).
fn armory_site() -> MapTarget {
    own_building(|aimed| Aimed {
        site: Some(ObjectKind::Armory),
        ..aimed
    })
}
/// An own Armory in operation, which takes Recruits in.
fn armory() -> MapTarget {
    own_building(|aimed| Aimed {
        accepts: &[ObjectKind::Recruit],
        ..aimed
    })
}
/// An own Storage, which takes the Recruits' loads.
fn storage() -> MapTarget {
    own_building(|aimed| Aimed {
        drop_off: true,
        ..aimed
    })
}

/// `> Right-Click Resolution`: "In-progress own building → Assist Construction", whatever it will be: over an Armory
/// under construction too, which no one enters (design change 0347 point 4: "a building under construction can't be
/// entered"); "Own Armory → Enter Armory" once it stands; "Space Crystal patch → Gather"; "Own Storage (carrying Space
/// Crystals) → Drop off resources", and a Move to it without a load; "Enemy unit/building → Attack (melee)". The
/// pointer previews each (design change 0322: "Assist, over an own building under construction"). A Militant builds
/// nothing and enters no Armory: beside a Recruit, Mixed.
#[test]
fn a_recruit_assists_a_site_enters_an_armory_and_gathers() {
    use PointerDisplay as D;
    let preview = |units: &[UnitCapabilities], target: MapTarget| right_click_display(units, &target);
    let site = Resolved::Order(OrderKind::Assist(id(4)));
    assert_eq!(recruit().template, Some(InterfaceTemplate::Recruit));
    assert_eq!(object_right_click(recruit(), &storage_site()), Some(site));
    assert_eq!(object_right_click(recruit(), &armory_site()), Some(site));
    assert_eq!(preview(&[recruit()], storage_site()), D::Assist);
    assert_eq!(preview(&[recruit()], armory_site()), D::Assist, "no Enter over a site");
    assert_eq!(preview(&[recruit()], armory()), D::Enter);
    assert_eq!(preview(&[recruit()], patch()), D::GatherResources);
    assert_eq!(preview(&[loaded_recruit()], storage()), D::ReturnResources);
    assert_eq!(preview(&[recruit()], storage()), D::Move, "nothing to drop off");
    assert_eq!(preview(&[recruit()], enemy()), D::Attack);
    assert_eq!(preview(&[recruit()], MapTarget::ground(P)), D::Move);
    let militant = UnitCapabilities::of(id(2), ObjectKind::Militant, &Modes::default());
    assert_eq!(preview(&[militant], storage_site()), D::Move);
    assert_eq!(preview(&[recruit(), militant], storage_site()), D::Mixed);
    assert_eq!(preview(&[recruit(), militant], armory()), D::Mixed);
    assert_eq!(PointerDisplay::of_order(OrderKind::Assist(id(4))), D::Assist);
}

/// `> Recruit ObjectInterfaceState > DefaultState commands`: "**S: Assist Construction** — targeted command
/// (StateOnlyTransition). Left-click an in-progress own building to join construction": a click on an own site gives
/// the recipients an Assist, under the Assist pointer; anything else is an invalid target (design change 0065), and
/// the pointer is Inactive there. Repair has its pointer's hook (design change 0322), which no order gives before
/// slice N8.
#[test]
fn assist_construction_takes_an_own_site() {
    use PointerDisplay as D;
    let awaiting = InterfaceState::awaiting(PanelCommand::Assist);
    let assist = Resolved::Order(OrderKind::Assist(id(4)));
    for target in [storage_site(), armory_site()] {
        assert_eq!(assist_target(&target), Some(id(4)));
        assert_eq!(target_resolution(PanelCommand::Assist, &target), Some(assist));
        assert_eq!(pointer_display(awaiting, &[recruit()], &target), D::Assist);
    }
    let ground = MapTarget::ground(P);
    for invalid in [armory(), storage(), patch(), enemy(), own(), ground] {
        assert_eq!(assist_target(&invalid), None, "{invalid:?}");
        assert_eq!(target_resolution(PanelCommand::Assist, &invalid), None, "{invalid:?}");
        assert_eq!(
            pointer_display(awaiting, &[recruit()], &invalid),
            D::Inactive,
            "{invalid:?}"
        );
    }
    let previews = [storage_site(), armory(), storage(), patch(), enemy()]
        .map(|target| right_click_display(&[recruit(), loaded_recruit()], &target));
    assert!(!previews.contains(&D::Repair), "{previews:?}");
}
