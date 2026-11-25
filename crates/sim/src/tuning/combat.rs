//! Combat (`systems/combat.md`): the damage rules every attack shares, and when a unit faces its target. The attacks'
//! own numbers are their defs' (`defs::AttackAttrs`). Used by `crate::combat`.

use crate::math::PI;

/// `systems/combat.md > MinimumDamage > MinimumDamage`: every single-target hit deals at least this much.
pub const MINIMUM_DAMAGE: f32 = 0.5;

/// `systems/combat.md > DirectionalArmor`: the half-width of the front and rear arcs, radians (±45°, 90° arcs).
pub const DIRECTIONAL_HALF_ARC: f32 = PI / 4.0;

/// `systems/combat.md > DirectionalArmor`: the multiplier of a hit from the front arc.
pub const DIRECTIONAL_FRONT: f32 = 0.75;

/// `systems/combat.md > DirectionalArmor`: from either side arc.
pub const DIRECTIONAL_SIDE: f32 = 1.0;

/// `systems/combat.md > DirectionalArmor`: from the rear arc.
pub const DIRECTIONAL_REAR: f32 = 1.25;

/// `systems/controls/behaviors.md > AttackingObject` 2: a unit attacking from its base must face its target ("unit
/// facing") before it fires. The largest angle, radians, between its facing and the direction to the target's centre
/// that counts as facing it. Placeholder, unspecified: 1°. Aiming turns the unit onto the exact direction once it is
/// within the TurnRate, so this only absorbs float rounding.
pub const FACING_TOLERANCE: f32 = PI / 180.0;
