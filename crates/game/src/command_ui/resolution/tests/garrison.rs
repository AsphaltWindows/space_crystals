//! The Garrison's clicks and pointers (slice M5-N2): the four kinds it takes in enter it on a right-click, the sim's
//! one rule, under the Enter pointer, and never its site; it has no right-click of its own. (M5-S2b) Full and damaged,
//! it is the Recruits' to repair (design change 0378).

use super::*;

/// The kinds a Garrison takes in (`factions/cults/structures/garrison.md > Garrisoning > Who can enter`: "Recruits,
/// Militants, Gunners, and Disciples").
const TAKES: [ObjectKind; 4] = [
    ObjectKind::Recruit,
    ObjectKind::Militant,
    ObjectKind::Gunner,
    ObjectKind::Disciple,
];

/// One of each, ids 1 to 4, as the client reads them from their types.
fn four() -> [UnitCapabilities; 4] {
    let mut n = 0;
    TAKES.map(|kind| {
        n += 1;
        UnitCapabilities::of(id(n), kind, &Modes::default())
    })
}

/// An own Garrison in operation at id 4 (`holding::admits`), as the sim aims it; and one under construction.
fn garrison() -> MapTarget {
    let mut garrison = on(CursorTarget::FriendlyObject(id(4)), true, Domain::Ground);
    garrison.aimed = garrison.aimed.map(|aimed| Aimed {
        accepts: &TAKES,
        ..aimed
    });
    garrison
}
fn garrison_site() -> MapTarget {
    let mut site = on(CursorTarget::FriendlyObject(id(4)), true, Domain::Ground);
    site.aimed = site.aimed.map(|aimed| Aimed {
        site: Some(ObjectKind::Garrison),
        ..aimed
    });
    site
}

/// `systems/controls/interface.md > RightClickResolution` (`factions/cults/units/recruit.md > Right-Click Resolution`:
/// "Own Garrison → Enter Garrison"; `factions/cults/structures/garrison.md > Garrisoning > Who can enter`: "right-click
/// own Garrison with eligible infantry selected"): each of the four, and the four together, enter it, under the Enter
/// pointer. Over its site the Recruit assists and the others move to it (design change 0347 point 4: "a building
/// under construction can't be entered"), which together is Mixed. A Peacekeeper enters no Garrison: beside them,
/// Mixed. The Garrison itself resolves no right-click (`factions/cults/structures/garrison.md > Placeholder stats >
/// Panel`: "Right-click resolution: none (it has no rally point)").
#[test]
fn the_four_kinds_enter_an_own_garrison_under_the_enter_pointer() {
    use PointerDisplay as D;
    let preview = |units: &[UnitCapabilities], target: MapTarget| right_click_display(units, &target);
    let enter = Resolved::Order(OrderKind::Enter(id(4)));
    for unit in four() {
        assert_eq!(object_right_click(unit, &garrison()), Some(enter), "{unit:?}");
        assert_eq!(preview(&[unit], garrison()), D::Enter, "{unit:?}");
    }
    assert_eq!(preview(&four(), garrison()), D::Enter, "all four");
    let [recruit, militant, ..] = four();
    assert_eq!(preview(&[recruit], garrison_site()), D::Assist);
    assert_eq!(preview(&[militant], garrison_site()), D::Move);
    assert_eq!(preview(&four(), garrison_site()), D::Mixed);
    let peacekeeper = UnitCapabilities::of(id(5), ObjectKind::Peacekeeper, &Modes::default());
    assert_eq!(preview(&[peacekeeper], garrison()), D::Move);
    assert_eq!(preview(&[militant, peacekeeper], garrison()), D::Mixed);
    let selected = UnitCapabilities::of(id(4), ObjectKind::Garrison, &Modes::default());
    assert_eq!(selected.template, Some(InterfaceTemplate::Garrison));
    for target in [MapTarget::ground(P), own(), enemy(), garrison()] {
        assert_eq!(object_right_click(selected, &target), None, "{target:?}");
    }
}

/// (M5-S2b) Design change 0378, answering Q0140 (`factions/cults/units/recruit.md > Right-Click Resolution`: "a damaged
/// enterable building that is **full at the moment of the click** resolves to **Repair** (the pointer shows
/// Repair)"): over an own Garrison that the sim aims full and damaged (`Aimed::full`, `Aimed::repair`), the Recruit's
/// right-click is a Repair, under the Repair pointer; the other three kinds, whose types repair nothing, enter it, so
/// the four together are Mixed. With a free slot the same right-click enters (design change 0325: "**Enter** wins"),
/// and so it does on a full Garrison with nothing to repair.
#[test]
fn a_full_damaged_garrison_is_the_recruits_to_repair() {
    use PointerDisplay as D;
    let preview = |units: &[UnitCapabilities], target: MapTarget| right_click_display(units, &target);
    let aimed = |full, repair| {
        let mut target = garrison();
        target.aimed = target.aimed.map(|aimed| Aimed { full, repair, ..aimed });
        target
    };
    let garrison_kind = Some(ObjectKind::Garrison);
    let (full, free) = (aimed(true, garrison_kind), aimed(false, garrison_kind));
    let order = |kind| Some(Resolved::Order(kind));
    let [recruit, militant, gunner, disciple] = four();
    assert_eq!(object_right_click(recruit, &full), order(OrderKind::Repair(id(4))));
    assert_eq!(preview(&[recruit], full), D::Repair);
    for unit in [militant, gunner, disciple] {
        assert_eq!(
            object_right_click(unit, &full),
            order(OrderKind::Enter(id(4))),
            "{unit:?}"
        );
        assert_eq!(preview(&[unit], full), D::Enter, "{unit:?}");
    }
    assert_eq!(preview(&four(), full), D::Mixed);
    assert_eq!(object_right_click(recruit, &free), order(OrderKind::Enter(id(4))));
    assert_eq!(preview(&[recruit], free), D::Enter, "a free slot");
    assert_eq!(preview(&four(), free), D::Enter);
    assert_eq!(preview(&[recruit], aimed(true, None)), D::Enter, "nothing to repair");
}
