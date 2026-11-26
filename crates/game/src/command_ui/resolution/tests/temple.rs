//! (M5-N3) The Temple's clicks and pointers: Recruits enter it on a right-click, the sim's one rule, under the Enter
//! pointer, and no other kind does; it has no right-click of its own.

use super::*;

/// An own Temple in operation at id 4 (`holding::admits`), as the sim aims it: it takes Recruits alone.
fn temple() -> MapTarget {
    let mut temple = on(CursorTarget::FriendlyObject(id(4)), true, Domain::Ground);
    temple.aimed = temple.aimed.map(|aimed| Aimed {
        accepts: &[ObjectKind::Recruit],
        ..aimed
    });
    temple
}

/// `factions/cults/structures/temple.md > Recruit Staffing`: "Recruits enter the Temple to staff its research
/// (right-click own Temple with Recruits selected → Enter Temple). Only Recruits can staff research." A Recruit's
/// right-click on an own Temple enters it, under the Enter pointer; a Militant's moves to it, and beside a Recruit
/// shows Mixed. The Temple itself resolves no right-click: it has no rally point (its panel has no C; **Placeholder
/// (not in the design)**).
#[test]
fn recruits_enter_an_own_temple_which_has_no_right_click() {
    use PointerDisplay as D;
    let recruit = UnitCapabilities::of(id(1), ObjectKind::Recruit, &Modes::default());
    let militant = UnitCapabilities::of(id(2), ObjectKind::Militant, &Modes::default());
    let enter = Resolved::Order(OrderKind::Enter(id(4)));
    assert_eq!(object_right_click(recruit, &temple()), Some(enter));
    assert_eq!(right_click_display(&[recruit], &temple()), D::Enter);
    assert_eq!(right_click_display(&[militant], &temple()), D::Move);
    assert_eq!(right_click_display(&[recruit, militant], &temple()), D::Mixed);
    let selected = UnitCapabilities::of(id(4), ObjectKind::Temple, &Modes::default());
    assert_eq!(selected.template, Some(InterfaceTemplate::Temple));
    for target in [MapTarget::ground(P), own(), enemy(), temple()] {
        assert_eq!(object_right_click(selected, &target), None, "{target:?}");
    }
}
