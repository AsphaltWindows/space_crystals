//! What a unit's Augmentations do to its stats (slice M5-N1a, plan decision T73; `factions/cults/augmentation.md >
//! Increments`, `> Augmentation Effects`), read through the accessors here: MaxHP ([`Object::max_hp_with`]), top speed
//! ([`Object::movement`], [`Object::max_speed`]) and the weapon ([`augmented`], which `combat::weapon` applies). The
//! numbers are data (`defs::CultsRules::augmentation`); a unit's net increments are derived from them and its owner's
//! levels whenever a stat is read, never stored ([`Increments`]). A unit without an Augmentation reads its kind's stats
//! unchanged, at the cost of one branch. (M5-N11) The same accessors add an Inspire's top speed and faster attack
//! (`crate::buffs::max_speed_bonus`, `crate::buffs::inspired`), and (M6-SB) the sludge's slow lowers the top speeds
//! (`crate::patches::speed_factor`): a unit without an Augmentation nor a buff reads its kind's.

use super::{Augmentation, Levels, Research};
use crate::buffs;
use crate::defs::{AttackAttrs, Augmentable, MovementAttrs, ObjectKind};
use crate::object::Object;
use crate::patches;
use crate::ruleset::rules;
use crate::transform;

/// A unit's net increments of each kind (`factions/cults/augmentation.md > Increments`): Constitution and Metabolism,
/// either of which can be negative ("Increments can be negative. A negative increment applies the opposite effect").
/// Also a row of the effects table (`defs::AugmentationTable`).
#[derive(Copy, Clone, Debug, Default, PartialEq, Eq, Hash, serde::Serialize, serde::Deserialize)]
#[serde(deny_unknown_fields)]
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
        let table = &rules().factions.cults.augmentation;
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

// The loops of phases 5a, 5d and 5e read every unit's stats: a unit without an Augmentation costs them a branch, its
// kind's numbers read by reference ([`Object::movement_ref`], `combat::weapon_ref`), and an augmented unit's amounts
// are worked out out of line ([`augmented_increments`]). The hints keep the release build's split into codegen units
// from leaving the calls out of line (`ARCHITECTURE.md > Object model > Augmented stats`).
impl Object {
    /// (M5-N1a) Its net increments ([`Increments`]): its Augmentations' at its owner's current levels
    /// (`Research::levels`), none for a kind that can't be augmented.
    #[inline]
    pub fn increments(&self, research: Research) -> Increments {
        match self.augmentation.is_none() {
            true => Increments::NONE,
            false => augmented_increments(self, research),
        }
    }

    /// (M5-N1a) Its MaxHP: [`Object::max_hp`]'s with its Augmentations' ([`max_hp_of`]). The MaxHP that a unit's HP
    /// stays within and that its HP bar and panel show. `None` for an indestructible object.
    #[inline]
    pub fn max_hp_with(&self, research: Research) -> Option<f32> {
        Some(max_hp_of(self.max_hp()?, self.increments(research)))
    }

    /// (M5-N1a) Its movement numbers: its kind's with its Augmentations' top speed ([`movement_of`]). `None` for an
    /// object that isn't a unit.
    #[inline]
    pub fn movement(&self, research: Research) -> Option<MovementAttrs> {
        self.movement_ref(research, &mut None).copied()
    }

    /// (M5-N1a) Its movement numbers ([`Object::movement`]) by reference: its kind's own without an Augmentation (nor,
    /// M5-N11, a buff), else made in `slot`. What phase 5d moves it by, with nothing copied for a unit without one.
    #[inline]
    pub fn movement_ref<'a>(
        &self,
        research: Research,
        slot: &'a mut Option<MovementAttrs>,
    ) -> Option<&'a MovementAttrs> {
        let movement = &self.def().unit.as_ref()?.movement;
        match self.augmentation.is_none() && self.buffs.is_empty() {
            true => Some(movement),
            false => Some(augmented_movement(self, movement, research, slot)),
        }
    }

    /// (M5-N1a) Its top speed, space units per tick, before its mode's `Mover::speed_factor` ([`Object::movement`]'s
    /// `MovementAttrs::max_speed`); 0 for an object that isn't a unit.
    #[inline]
    pub fn max_speed(&self, research: Research) -> f32 {
        (self.movement_ref(research, &mut None)).map_or(0.0, |movement| movement.max_speed())
    }
}

/// The movement numbers of `o`, which has an Augmentation or (M5-N11) a buff, from its kind's `movement`, made in `slot`
/// ([`Object::movement_ref`]): its Augmentations' top speed ([`movement_of`]), and its Inspire's on top
/// (`crate::buffs::max_speed_bonus`: "**Stacks with augmentation** (e.g. Improved Metabolism): different sources add
/// up", `factions/cults/units/prayer_mobile.md > Abilities`), as [`faster`] raises it; then (M6-SB) the sludge's slow
/// on the whole (`crate::patches::speed_factor`: "slowed by **Y = 40%**"), as [`slower`] lowers it.
#[inline(never)]
fn augmented_movement<'a>(
    o: &Object,
    movement: &MovementAttrs,
    research: Research,
    slot: &'a mut Option<MovementAttrs>,
) -> &'a MovementAttrs {
    let augmented = movement_of(*movement, o.increments(research));
    let extra = buffs::max_speed_bonus(o);
    let movement = if extra == 0.0 {
        augmented
    } else {
        faster(augmented, extra)
    };
    let factor = patches::speed_factor(o);
    slot.insert(if factor == 1.0 {
        movement
    } else {
        slower(movement, factor)
    })
}

/// The net increments of `o`, which has an Augmentation ([`Object::increments`]): none for a kind that can't be
/// augmented.
#[inline(never)]
fn augmented_increments(o: &Object, research: Research) -> Increments {
    match is_augmentable(o.kind) {
        true => o.augmentation.increments(research.levels_of(o)),
        false => Increments::NONE,
    }
}

/// The MaxHP `max` of an augmentable unit with net `increments`: "+10 MaxHP" per Constitution increment
/// (`factions/cults/augmentation.md > Increments`, armed or not). `max` itself with none.
#[inline]
pub(crate) fn max_hp_of(max: f32, increments: Increments) -> f32 {
    match increments.constitution {
        0 => max,
        constitution => max + constitution as f32 * rules().factions.cults.augmentation.increments.max_hp,
    }
}

/// The movement numbers `movement` of an augmentable unit with net `increments`: "+0.5 space units/frame MaxSpeed"
/// per Metabolism increment (`factions/cults/augmentation.md > Increments`, armed or not), before its mode's
/// `Mover::speed_factor` (plan decision T73). `movement` itself with none.
#[inline]
pub(crate) fn movement_of(movement: MovementAttrs, increments: Increments) -> MovementAttrs {
    match increments.metabolism {
        0 => movement,
        metabolism => faster(
            movement,
            metabolism as f32 * rules().factions.cults.augmentation.increments.max_speed,
        ),
    }
}

/// `movement` with its top speed raised by `extra` space units per tick (lowered for a negative one). Every
/// augmentable kind moves by turn rate (`augmentation/tests.rs`). For the others, **Placeholder (not in the design: no
/// such kind can be augmented)**: the field that is their MaxSpeed, the forward one alone for a fixed turn radius, and
/// for a model with momentum the thrust that balances its drag at the new top speed.
#[inline(never)]
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

/// (M6-SB) `movement` with every top speed of its model times `factor` (`crate::patches::speed_factor`): forward and,
/// for the models that back up, in reverse; a Glider's least speed too, which stays below its top one; and for a model
/// with momentum its thrusts, whose balance with its drag is its top speed. Its accelerations and its turning are its
/// own, but where the model turns by its speed (a tracked unit's spin, from its treads' top speed).
#[inline(never)]
fn slower(movement: MovementAttrs, factor: f32) -> MovementAttrs {
    let mut movement = movement;
    match &mut movement {
        MovementAttrs::TurnRate { max_speed, .. } => *max_speed *= factor,
        MovementAttrs::FixedTurnRadius {
            forward_max_speed,
            reverse_max_speed,
            ..
        }
        | MovementAttrs::SpeedTurnRadius {
            max_speed: forward_max_speed,
            reverse_max_speed,
            ..
        } => {
            *forward_max_speed *= factor;
            *reverse_max_speed *= factor;
        }
        MovementAttrs::Drag {
            forward_acceleration,
            omni_directional_acceleration,
            ..
        } => {
            *forward_acceleration *= factor;
            *omni_directional_acceleration *= factor;
        }
        MovementAttrs::Glider {
            idle_speed, max_speed, ..
        } => {
            *idle_speed *= factor;
            *max_speed *= factor;
        }
    }
    movement
}

/// (M5-N1a) `attrs`, the weapon that `o` holds (`combat::weapon`), as its Augmentations make it
/// (`factions/cults/augmentation.md > Increments`): an unarmed unit's (the Recruit's) melee Damage "+1" per net
/// Constitution increment and its FiringDuration "−1 frame" per net Metabolism increment ("a negative increment adds 1
/// frame"), "never below 2 frames"; no other floor (design change 0331: "**No floors** on the results", so an Improved
/// Metabolism Level 2 Recruit's Damage is 0, and each hit does the minimum). An armed unit's is its kind's ("Armed
/// infantry (Militant, Gunner) receive only the MaxHP part of Constitution increments and only the movement-speed part
/// of Metabolism increments"), as is that of every unit without an Augmentation. So is a weapon borrowed from another
/// kind (`factions/cults/structures/garrison.md`: "Augmentations don't change the borrowed attack"; slice M5-N2):
/// only the unit's own attack, its kind's, takes the effects.
pub(crate) fn augmented(o: &Object, attrs: &AttackAttrs, research: Research) -> AttackAttrs {
    if o.augmentation.is_none() || !augmentable(o.kind).is_some_and(|how| how.unarmed) {
        return *attrs;
    }
    let own = o.def().unit.as_ref().and_then(|unit| unit.attack.as_ref());
    if own != Some(attrs) {
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
    let effects = rules().factions.cults.augmentation.increments;
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
