//! (M5-K) The Rocketman's panel as plain data (`factions/gdo/units/rocketman.md > Placeholder stats`: "Panel:
//! BasicCombatUnitInterfaceState, plus **R: Target Lock** (targeted: left-click an enemy unit or structure)"). In a
//! game: `crates/game/tests/commands/rocketman_ui.rs`.

use super::*;

const RN: ObjectKind = ObjectKind::Rocketman;

/// The Rocketman's DefaultState is the BasicCombatUnit's, with Target Lock in R, labelled "Lock" and named "Target
/// Lock": W Reverse is unavailable, as an infantry's is, and D Attack Ground available, its DoublyDisjointed missile
/// "can also be fired at the ground". R awaits its target, and Shift queues it. The Rocketman being Groupable
/// ("Groupable true"), two are one group, and R goes to both. With Peacekeepers, R is the ActiveGroup's, a group
/// command: Target Lock to the Rocketmen while they are in focus, Raise Shield to the Peacekeepers while theirs is.
#[test]
fn the_rocketmans_r_is_target_lock() {
    use ButtonLook::{Common, Group, Unavailable};
    use PanelCommand::*;
    let two = Selection::new([(id(1), RN), (id(2), RN)]);
    let default = panel(&two, InterfaceState::DefaultState, &Conditions::default());
    let c = PanelButton::Command;
    assert_eq!(
        shown(&default),
        [
            (slot(0, 0), c(Move), Common),
            (slot(0, 1), c(Reverse), Unavailable),
            (slot(0, 2), c(HoldPosition), Common),
            (slot(0, 3), c(TargetLock), Common),
            (slot(1, 0), c(Attack), Common),
            (slot(1, 1), c(Patrol), Common),
            (slot(1, 2), c(AttackGround), Common),
            (slot(2, 1), c(Stop), Common),
        ]
    );
    assert_eq!(default.get(slot(0, 3)).map(|shown| shown.label()), Some("Lock"));
    assert_eq!(default.name(slot(0, 3)).as_deref(), Some("Target Lock"));
    assert_eq!(press(&default, slot(0, 3)), SlotPress::AwaitTarget(TargetLock));
    assert!(TargetLock.needs_target() && TargetLock.queues() && !TargetLock.instance_only());
    assert_eq!(two.groups().len(), 1, "Groupable");
    assert_eq!(recipients(&two, TargetLock), [id(1), id(2)]);

    let mut mixed = Selection::new([(id(1), RN), (id(2), RN), (id(3), PK)]);
    let r = |selection: &Selection| {
        let panel = panel(selection, InterfaceState::DefaultState, &Conditions::default());
        panel.get(slot(0, 3)).map(|shown| (shown.button, shown.look))
    };
    for _ in 0..2 {
        if mixed.active().map(|active| active.kind) != Some(RN) {
            mixed.cycle(true);
        }
    }
    assert_eq!(mixed.active().map(|active| active.kind), Some(RN));
    assert_eq!(r(&mixed), Some((c(TargetLock), Group)));
    assert_eq!(recipients(&mixed, TargetLock), [id(1), id(2)]);
    mixed.cycle(true);
    assert_eq!(mixed.active().map(|active| active.kind), Some(PK));
    assert_eq!(r(&mixed), Some((c(RaiseShield), Group)));
}
