//! (M4-O) The test shooter of plan §3.3 O's acceptance (`defs::test_defs`): no kind of the game throws grenades yet
//! (the Grenadier, `factions/syndicate/units/grenadier.md`, is slice U's), so the sim's tests give the Guard one.

use crate::defs::test_defs::TestDef;
use crate::defs::{AttackAttrs, AttackType, ObjectKind, TargetDomain, TargetType};
use crate::units::SU_PER_GU;

/// The grenade, with the Grenadier's numbers: a DoublyDisjointed Ground attack of Damage 100 over a circle of 40 su
/// (0.625 grid units), Range 4, 8 su per tick; Aim 6, Firing 2, Cooldown 4, Reload 28 ticks.
pub(crate) const GRENADE: AttackAttrs = AttackAttrs {
    attack_type: AttackType::DoublyDisjointed,
    target_domain: TargetDomain::Ground,
    target_type: TargetType::AoE {
        radius: 40.0 / SU_PER_GU,
    },
    damage: 100.0,
    range: 4.0,
    min_range: 0.0,
    projectile_speed: Some(8.0),
    aim_ticks: 6,
    firing_ticks: 2,
    cooldown_ticks: 4,
    reload_ticks: 28,
};

/// While the guard lives, the Guard throws `grenade` in place of its own attack.
pub(crate) fn grenadier(grenade: AttackAttrs) -> TestDef {
    TestDef::unit(ObjectKind::Guard, |unit| unit.attack = Some(grenade))
}
