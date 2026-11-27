//! Combat (`systems/combat.md`): the damage rules every attack shares, and when a unit faces its target. The attacks'
//! own numbers are their defs' (`defs::AttackAttrs`). Used by `crate::combat`.

use crate::math::PI;

tunables! {
    /// The tunables of combat (module docs).
    Combat;

    /// `systems/combat.md > MinimumDamage > MinimumDamage`: every single-target hit deals at least this much.
    minimum_damage: f32 = 0.5,

    /// `systems/combat.md > DirectionalArmor`: the half-width of the front and rear arcs, radians (±45°, 90° arcs).
    directional_half_arc: f32 = PI / 4.0,

    /// `systems/combat.md > DirectionalArmor`: the multiplier of a hit from the front arc.
    directional_front: f32 = 0.75,

    /// `systems/combat.md > DirectionalArmor`: from either side arc.
    directional_side: f32 = 1.0,

    /// `systems/combat.md > DirectionalArmor`: from the rear arc.
    directional_rear: f32 = 1.25,

    /// `systems/controls/command_behaviors.md > AttackingObject` 2: a unit attacking from its base must face its target
    /// ("unit facing") before it fires. The largest angle, radians, between its facing and the direction to the
    /// target's centre that counts as facing it. Placeholder, unspecified: 1°. Aiming turns the unit onto the exact
    /// direction once it is within the TurnRate, so this only absorbs float rounding.
    facing_tolerance: f32 = PI / 180.0,
}
