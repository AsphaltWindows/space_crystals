//! (M5-K) The Rocketman's R, Target Lock: it awaits an enemy unit or structure the player sees now, where a click gives
//! a Target Lock order on it, under the Attack pointer (placeholder); anything else is an invalid target, under the
//! Inactive pointer.

use super::*;

/// `factions/gdo/units/rocketman.md > Placeholder stats`: "Panel: BasicCombatUnitInterfaceState, plus **R: Target
/// Lock** (targeted: left-click an enemy unit or structure)". A click on an enemy unit, an enemy aircraft or an enemy
/// structure that the player sees gives a Target Lock on it, under the Attack pointer (placeholder: the design's
/// PointerDisplayTypes have none of its own). The ground, an own or a neutral object, an enemy object that can't be
/// destroyed, and an enemy structure the player only remembers (**reading**: its lock would break at once, as the
/// Rocketman doesn't see it) are invalid targets, under the Inactive pointer.
#[test]
fn target_lock_takes_an_enemy_unit_or_structure_the_player_sees() {
    use PointerDisplay as D;
    let rocketmen = [UnitCapabilities::of(id(1), ObjectKind::Rocketman, &Modes::default())];
    let awaiting = InterfaceState::AwaitingTarget {
        pending: PanelCommand::TargetLock,
        queued: false,
    };
    for target in [enemy(), aircraft(), enemy_structure()] {
        let locked = target.target.object().map(OrderKind::TargetLock);
        assert_eq!(target_order(PanelCommand::TargetLock, &target), locked, "{target:?}");
        assert_eq!(pointer_display(awaiting, &rocketmen, &target), D::Attack, "{target:?}");
    }
    let indestructible = on(CursorTarget::EnemyObject(id(6)), false, Domain::Ground);
    let remembered = MapTarget {
        visible: false,
        ..enemy_structure()
    };
    for target in [
        MapTarget::ground(P),
        own(),
        neutral_unit(),
        patch(),
        indestructible,
        remembered,
    ] {
        assert_eq!(target_order(PanelCommand::TargetLock, &target), None, "{target:?}");
        assert_eq!(
            pointer_display(awaiting, &rocketmen, &target),
            D::Inactive,
            "{target:?}"
        );
    }
}
