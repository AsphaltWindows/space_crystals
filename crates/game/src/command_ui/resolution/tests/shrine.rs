//! (M5-N5) The Shrine's clicks and pointers: a cast awaits a point the player sees now, where a click casts it, under
//! the Cast pointer; a point out of vision is an invalid target, under the Inactive pointer. Recruits enter the Shrine
//! on a right-click, the sim's one rule, and it has no right-click of its own.

use sc_sim::AreaAbility;

use super::*;

const HEAL: PanelCommand = PanelCommand::Cast(AreaAbility::HealthRegeneration);
const BOOST: PanelCommand = PanelCommand::Cast(AreaAbility::EnergyRegeneration);

/// An own Shrine in operation at id 4 (`holding::admits`), as the sim aims it: it takes Recruits alone.
fn shrine() -> MapTarget {
    let mut shrine = on(CursorTarget::FriendlyObject(id(4)), true, Domain::Ground);
    shrine.aimed = shrine.aimed.map(|aimed| Aimed {
        accepts: &[ObjectKind::Recruit],
        ..aimed
    });
    shrine
}

/// `factions/cults/structures/shrine.md > Placeholder stats`: "Targeting: AwaitingTarget, left-click a point in
/// vision"; `> Abilities`: "cast on an area anywhere the player currently has vision". A click on a point the player
/// sees casts there: on the ground, and on the ground under an object, own or enemy, which the cast doesn't aim at
/// (**Placeholder**: "a point"); the pointer shows Cast. On a point the player doesn't see now, nothing (an invalid
/// target: plan §N5's "a click outside vision gives "invalid target" feedback"), under the Inactive pointer. Each of the
/// two casts its own ability.
#[test]
fn a_cast_takes_a_point_in_vision() {
    use PointerDisplay as D;
    let shrine_itself = [UnitCapabilities::of(id(4), ObjectKind::Shrine, &Modes::default())];
    let awaiting = |pending| InterfaceState::AwaitingTarget { pending, queued: false };
    for (pending, ability) in [
        (HEAL, AreaAbility::HealthRegeneration),
        (BOOST, AreaAbility::EnergyRegeneration),
    ] {
        for seen in [MapTarget::ground(P), own(), enemy(), shrine()] {
            assert_eq!(
                target_resolution(pending, &seen),
                Some(Resolved::Cast(ability, P)),
                "{seen:?}"
            );
            assert_eq!(pointer_display(awaiting(pending), &shrine_itself, &seen), D::Cast);
            let unseen = MapTarget { seen: false, ..seen };
            assert_eq!(target_resolution(pending, &unseen), None, "{unseen:?}");
            assert_eq!(pointer_display(awaiting(pending), &shrine_itself, &unseen), D::Inactive);
        }
    }
    assert_eq!(
        PointerDisplay::of(Resolved::Cast(AreaAbility::HealthRegeneration, P)),
        D::Cast
    );
}

/// `factions/cults/structures/shrine.md > Ritual Participation`: "Recruits enter the Shrine to take part in the ritual
/// (right-click own Shrine with Recruits selected → Enter Shrine). Only Recruits can participate." A Recruit's
/// right-click on an own Shrine enters it, under the Enter pointer; a Militant's moves to it, and beside a Recruit shows
/// Mixed. The Shrine itself resolves no right-click: it has no rally point (its panel has no C; **Placeholder (not in
/// the design)**).
#[test]
fn recruits_enter_an_own_shrine_which_has_no_right_click() {
    use PointerDisplay as D;
    let recruit = UnitCapabilities::of(id(1), ObjectKind::Recruit, &Modes::default());
    let militant = UnitCapabilities::of(id(2), ObjectKind::Militant, &Modes::default());
    let enter = Resolved::Order(OrderKind::Enter(id(4)));
    assert_eq!(object_right_click(recruit, &shrine()), Some(enter));
    assert_eq!(right_click_display(&[recruit], &shrine()), D::Enter);
    assert_eq!(right_click_display(&[militant], &shrine()), D::Move);
    assert_eq!(right_click_display(&[recruit, militant], &shrine()), D::Mixed);
    let selected = UnitCapabilities::of(id(4), ObjectKind::Shrine, &Modes::default());
    assert_eq!(selected.template, Some(InterfaceTemplate::Shrine));
    for target in [MapTarget::ground(P), own(), enemy(), shrine()] {
        assert_eq!(object_right_click(selected, &target), None, "{target:?}");
    }
}
