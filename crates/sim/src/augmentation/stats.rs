//! What a unit's Augmentations do to its stats (slice M5-N1a, plan decision T73; `factions/cults/augmentation.md >
//! Increments`, `> Augmentation Effects`), read through the accessors here: MaxHP ([`Object::max_hp_with`]), top
//! speed ([`Object::movement`], [`Object::max_speed`]) and the weapon ([`augmented`], which `combat::weapon` applies).
//! The numbers are data (`defs::AUGMENTATION`); a unit's net increments are derived from them and its owner's levels
//! whenever a stat is read, never stored ([`Increments`]). A unit without an Augmentation reads its kind's stats
//! unchanged, at the cost of one branch.

use super::{Augmentation, Levels, Research};
use crate::defs::{AUGMENTATION, AttackAttrs, Augmentable, MovementAttrs, ObjectKind};
use crate::object::Object;
use crate::transform;

/// A unit's net increments of each kind (`factions/cults/augmentation.md > Increments`): Constitution and Metabolism,
/// either of which can be negative ("Increments can be negative. A negative increment applies the opposite effect").
/// Also a row of the effects table (`defs::AugmentationTable`).
#[derive(Copy, Clone, Debug, Default, PartialEq, Eq, Hash)]
pub struct Increments {
    pub constitution: i32,
    pub metabolism: i32,
}

impl Increments {
    /// None of either: a unit without an Augmentation.
    pub const NONE: Increments = Increments::new(0, 0);

    pub const fn new(constitution: i32, metabolism: i32) -> Self {
        Self {
            constitution,
            metabolism,
        }
    }
}

impl Augmentation {
    /// Its net increments at `levels` (`factions/cults/augmentation.md > Augmentation Effects`): each Augmentation's
    /// row at its own line's level, and "A unit with both sums their effects" (both at no research: "+1 Constitution /
    /// +1 Metabolism").
    pub fn increments(self, levels: Levels) -> Increments {
        let table = &AUGMENTATION;
        let rows = [
            (
                self.constitution,
                table.enhanced_constitution[levels.constitution.index()],
            ),
            (self.metabolism, table.improved_metabolism[levels.metabolism.index()]),
        ];
        let had = rows.into_iter().filter(|&(has, _)| has).map(|(_, row)| row);
        had.fold(Increments::NONE, |sum, row| {
            Increments::new(sum.constitution + row.constitution, sum.metabolism + row.metabolism)
        })
    }
}

/// How units of `kind` take Augmentations (`UnitDef::augmentable`); `None` for a kind that can't be augmented.
fn augmentable(kind: ObjectKind) -> Option<Augmentable> {
    kind.def().unit.as_ref()?.augmentable
}

/// Whether units of `kind` can be augmented (`factions/cults/augmentation.md > Eligibility`: the four Cults infantry
/// types): a unit keeps its Augmentations through a change to such a kind, and loses them through a change to any
/// other (plan decision T73, `Sim::change_kind`).
pub fn is_augmentable(kind: ObjectKind) -> bool {
    augmentable(kind).is_some()
}

impl Object {
    /// (M5-N1a) Its net increments ([`Increments`]): its Augmentations' at its owner's current levels
    /// (`Research::levels`), none for a kind that can't be augmented.
    pub fn increments(&self, research: Research) -> Increments {
        if self.augmentation.is_none() || !is_augmentable(self.kind) {
            return Increments::NONE;
        }
        self.augmentation.increments(research.levels(self.owner))
    }

    /// (M5-N1a) Its MaxHP: [`Object::max_hp`]'s with its Augmentations' ([`max_hp_of`]). The MaxHP that a unit's HP
    /// stays within and that its HP bar and panel show. `None` for an indestructible object.
    pub fn max_hp_with(&self, research: Research) -> Option<f32> {
        Some(max_hp_of(self.max_hp()?, self.increments(research)))
    }

    /// (M5-N1a) Its movement numbers: its kind's with its Augmentations' top speed ([`movement_of`]). What phase 5d
    /// moves it by. `None` for an object that isn't a unit.
    pub fn movement(&self, research: Research) -> Option<MovementAttrs> {
        let movement = self.def().unit.as_ref()?.movement;
        Some(movement_of(movement, self.increments(research)))
    }

    /// (M5-N1a) Its top speed, space units per tick, before its mode's `Mover::speed_factor` ([`Object::movement`]'s
    /// `MovementAttrs::max_speed`); 0 for an object that isn't a unit.
    pub fn max_speed(&self, research: Research) -> f32 {
        self.movement(research).map_or(0.0, |movement| movement.max_speed())
    }
}

/// The MaxHP `max` of an augmentable unit with net `increments`: "+10 MaxHP" per Constitution increment
/// (`factions/cults/augmentation.md > Increments`, armed or not). `max` itself with none.
pub(crate) fn max_hp_of(max: f32, increments: Increments) -> f32 {
    match increments.constitution {
        0 => max,
        constitution => max + constitution as f32 * AUGMENTATION.increments.max_hp,
    }
}

/// The movement numbers `movement` of an augmentable unit with net `increments`: "+0.5 space units/frame MaxSpeed"
/// per Metabolism increment (`factions/cults/augmentation.md > Increments`, armed or not), before its mode's
/// `Mover::speed_factor` (plan decision T73). `movement` itself with none.
pub(crate) fn movement_of(movement: MovementAttrs, increments: Increments) -> MovementAttrs {
    match increments.metabolism {
        0 => movement,
        metabolism => faster(movement, metabolism as f32 * AUGMENTATION.increments.max_speed),
    }
}

/// `movement` with its top speed raised by `extra` space units per tick (lowered for a negative one). Every
/// augmentable kind moves by turn rate (`augmentation/tests.rs`). For the others, **Placeholder (not in the design: no
/// such kind can be augmented)**: the field that is their MaxSpeed, the forward one alone for a fixed turn radius, and
/// for a model with momentum the thrust that balances its drag at the new top speed.
fn faster(movement: MovementAttrs, extra: f32) -> MovementAttrs {
    let mut movement = movement;
    match &mut movement {
        MovementAttrs::TurnRate { max_speed, .. }
        | MovementAttrs::SpeedTurnRadius { max_speed, .. }
        | MovementAttrs::Glider { max_speed, .. } => *max_speed += extra,
        MovementAttrs::FixedTurnRadius { forward_max_speed, .. } => *forward_max_speed += extra,
        MovementAttrs::Drag {
            forward_acceleration,
            drag_ratio,
            ..
        } => *forward_acceleration += extra * *drag_ratio,
    }
    movement
}

/// (M5-N1a) `attrs`, the weapon that `o` holds (`combat::weapon`), as its Augmentations make it
/// (`factions/cults/augmentation.md > Increments`): an unarmed unit's (the Recruit's) melee Damage "+1" per net
/// Constitution increment and its FiringDuration "−1 frame" per net Metabolism increment ("a negative increment adds 1
/// frame"), "never below 2 frames"; no other floor (design change 0331: "**No floors** on the results", so an Improved
/// Metabolism Level 2 Recruit's Damage is 0, and each hit does the minimum). An armed unit's is its kind's ("Armed
/// infantry (Militant, Gunner) receive only the MaxHP part of Constitution increments and only the movement-speed part
/// of Metabolism increments"), as is that of every unit without an Augmentation.
pub(crate) fn augmented(o: &Object, attrs: &AttackAttrs, research: Research) -> AttackAttrs {
    if o.augmentation.is_none() || !augmentable(o.kind).is_some_and(|how| how.unarmed) {
        return *attrs;
    }
    unarmed_weapon(attrs, o.increments(research))
}

/// The weapon `attrs` of an unarmed unit with net `increments` ([`augmented`]).
pub(super) fn unarmed_weapon(attrs: &AttackAttrs, increments: Increments) -> AttackAttrs {
    let Increments {
        constitution,
        metabolism,
    } = increments;
    let effects = AUGMENTATION.increments;
    let firing = i64::from(attrs.firing_ticks) - i64::from(metabolism) * i64::from(effects.firing_ticks);
    AttackAttrs {
        damage: attrs.damage + constitution as f32 * effects.melee_damage,
        firing_ticks: firing.max(i64::from(effects.min_firing_ticks)) as u32,
        ..*attrs
    }
}

/// (M5-N1a) A unit's HP `hp` when its Augmentations take its MaxHP from `before` to `after`
/// (`factions/cults/augmentation.md > Increments`, "HP when MaxHP changes", design changes 0330 and 0331, which replace
/// plan decision T73's floor at 1 HP). Every change of a unit's Augmentations or of its owner's levels takes its HP
/// this way: `DebugCommand::SetAugmentation` now ([`set`]), the Altar and the Temple's research later.
/// - "**MaxHP goes up**: HP goes up **by the same amount**, so the damage taken stays the same (a full unit stays
///   full; 25/40 → 45/60)": `transform`'s rule for a rise (`transform::follow_max`, design change 0110: "current +=
///   (newMax − oldMax)").
/// - "**MaxHP goes down**: HP is **unchanged**, unless it is now above the new MaxHP, in which case it becomes the new
///   MaxHP (40/40 → 10/10; 8/40 → 8/10)". Here the two rules part: a transformation's maximum takes the HP down with
///   it, the damage staying (`transform::follow_max`).
///
/// "**No floors** on the results", and "An augmentation therefore never kills": a unit's MaxHP stays above 0
/// (`augmentation/tests.rs`).
pub fn follow_max(hp: f32, before: f32, after: f32) -> f32 {
    if after >= before {
        transform::follow_max(hp, before, after)
    } else {
        hp.min(after)
    }
}

/// (M5-N1a) `o`'s Augmentations become `augmentation` (`Object::augmentation`), its HP following its MaxHP by the
/// rule ([`follow_max`]); none for a kind that can't be augmented, which keeps none. The debug command can take one
/// away, which the game never does: **Placeholder (not in the design: "A unit cannot receive an Augmentation it already
/// has", and none is lost but with the unit or into a vehicle)**, its MaxHP then going up or down by the same rule.
pub(crate) fn set(o: &mut Object, augmentation: Augmentation, research: Research) {
    if !is_augmentable(o.kind) {
        return;
    }
    let before = o.max_hp_with(research);
    o.augmentation = augmentation;
    if let (Some(hp), Some(before), Some(after)) = (o.hp, before, o.max_hp_with(research)) {
        o.hp = Some(follow_max(hp, before, after));
    }
}
