//! What the Syndicate's stat blocks share, for their modules' tests (`units/`, `structures/`). The rules every kind
//! follows are in `defs_are_consistent` (`defs/tests.rs`).

use std::collections::BTreeSet;

use super::EJECT_MENU_ORDER;
use crate::defs::{AttackAttrs, ObjectKind};
use crate::player::Faction;

/// A unit's attack timings, `(AimDuration, FiringDuration, CooldownDuration, ReloadDuration)`, and the frames from one
/// shot to the next, their sum.
pub(super) fn timings(attack: &AttackAttrs) -> ((u32, u32, u32, u32), u32) {
    let t = (
        attack.aim_ticks,
        attack.firing_ticks,
        attack.cooldown_ticks,
        attack.reload_ticks,
    );
    (t, t.0 + t.1 + t.2 + t.3)
}

/// Design change 0257: the EjectMenu lists every Syndicate unit type, each once and nothing else, the Agent in Q and
/// the Guard in W ("Agent, Guard, Grenadier, …").
#[test]
fn the_eject_menu_lists_every_syndicate_unit_once() {
    let units: BTreeSet<ObjectKind> = (ObjectKind::ALL.iter().copied())
        .filter(|kind| kind.def().faction == Some(Faction::Syndicate) && kind.def().unit.is_some())
        .collect();
    let listed: BTreeSet<ObjectKind> = EJECT_MENU_ORDER.iter().copied().collect();
    assert_eq!(listed.len(), EJECT_MENU_ORDER.len(), "each once: {EJECT_MENU_ORDER:?}");
    assert_eq!(listed, units);
    assert_eq!(EJECT_MENU_ORDER[..2], [ObjectKind::Agent, ObjectKind::Guard]);
}
