//! (M4-O) The test shooter of plan §3.3 O's acceptance (`defs::test_defs`): the sim's tests give the Guard a grenade
//! with fixed numbers, so that they test the mechanics apart from the Grenadier's tuning. The Grenadier itself
//! (`factions/syndicate/units/grenadier.md`, slice M4-U) is played in `crates/sim/tests/grenadier.rs`. (M5-K) The same
//! for a homing missile ([`MISSILE`]): the Rocketman's locked one is played in the `rocketman` tests.

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

/// (M5-K) The homing missile (`systems/combat.md > HeadDisjointed`), with the Rocketman's numbers but half its speed,
/// so that a target walks a while ahead of it: a HeadDisjointed Universal single-target attack of Damage 20, Range 6, 8
/// su per tick; Aim 6, Firing 2, Cooldown 16, Reload 8 ticks.
pub(crate) const MISSILE: AttackAttrs = AttackAttrs {
    attack_type: AttackType::HeadDisjointed,
    target_domain: TargetDomain::Universal,
    target_type: TargetType::SingleTarget,
    damage: 20.0,
    range: 6.0,
    min_range: 0.0,
    projectile_speed: Some(8.0),
    aim_ticks: 6,
    firing_ticks: 2,
    cooldown_ticks: 16,
    reload_ticks: 8,
};

/// While the guard lives, the Guard throws `grenade` in place of its own attack (or fires it: [`MISSILE`]).
pub(crate) fn grenadier(grenade: AttackAttrs) -> TestDef {
    TestDef::unit(ObjectKind::Guard, |unit| unit.attack = Some(grenade))
}
