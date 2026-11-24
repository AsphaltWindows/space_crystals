//! Combat — `systems/combat.md`. Phase 5e (`docs/movement.md > 4`): attack phases, the list of hits, damage and
//! deaths, shooting from the positions phase 5d committed.
//!
//! - **State**: a unit that can attack (`UnitDef::attack`) has an [`AttackState`] (`Object::attack`): the target
//!   its behaviour engages, and the phase of its attack sequence with the ticks spent in it (the design's
//!   BaseAttackChannel, `systems/controls/behaviors.md`). It is hashed and serialized like all state.
//! - **Phases** (`systems/combat.md > AttackPhase`; `phases`): Aiming, Firing, Cooldown and Reloading, each
//!   lasting exactly its duration in ticks. A sequence starts when no sequence runs and the engaged target is valid
//!   and in range (`targeting`); Aiming also lasts until the unit faces the target ([`FACING_TOLERANCE`]). It turns
//!   the unit by what phase 5d left of its TurnRate: one turn budget per tick (`movement::Turns`).
//! - **Hits** (`damage`): an attack's effect lands at the end of its Firing phase, as a [`Hit`] on this tick's
//!   list. The list is applied in one place once every unit has acted, so two hits in one tick both count, whatever
//!   the order of their attackers (`ARCHITECTURE.md > Tick pipeline`).
//! - **Deaths**: an object at 0 HP or less dies (`SimEvent::Died`). `step` returns the dead, and `Sim::step`
//!   removes them through the one removal path (`Sim::remove_objects`), which clears every reference to them.
//! - **Queries**: [`may_move`] and [`may_turn`] say whether a unit's attack sequence lets it move or turn this tick.
//!   The movement step holds a unit that may not (phase 5d), and the behaviours start a chase only when it may
//!   (`orders::engage`).
//! - **Vision** (`systems/combat.md > ValidTarget` 2): a sequence starts, and Aiming goes on, only while the unit's
//!   owner sees the target, by the vision phase 5f left at the end of the last tick (`vision`). What a unit does
//!   about a target out of range or out of sight is its behaviour's (`orders::engage`: it closes in, or goes to
//!   where the target was last seen).
//! - **Range** (`> Range Measurement`, design change 0031): edge to edge, the gap between the attacker's shape and
//!   the target's (`shape`); [`reaches`] is the one check.
//!
//! - **Riot Shield** (`factions/gdo/units.md > Peacekeeper > Riot Shield`, `riot_shield`): a unit whose shield is
//!   up can't attack. Its behaviours engage nothing then; whatever is still engaged is dropped here, which cancels an
//!   aim, while Firing and Cooldown run on. Its raised shield halves the hits against it before armour
//!   ([`pre_armor_multiplier`]).
//!
//! Not built yet: turrets, projectiles and location targets (the disjointed attack types), AoE damage, and Shields.

mod damage;
mod phases;
mod targeting;

use std::collections::BTreeMap;

use serde::{Deserialize, Serialize};

use crate::defs::ObjectKind;
use crate::events::SimEvent;
use crate::hash::{StateHash, StateHasher};
use crate::ids::ObjectId;
use crate::map::Map;
use crate::movement;
use crate::object::Object;
use crate::riot_shield::RiotShield;
use crate::vision::Vision;

pub use damage::{
    DIRECTIONAL_FRONT, DIRECTIONAL_HALF_ARC, DIRECTIONAL_REAR, DIRECTIONAL_SIDE, Hit, MINIMUM_DAMAGE,
    directional_multiplier, hit_damage, pre_armor_multiplier,
};
pub use phases::FACING_TOLERANCE;
pub use targeting::{
    can_attack_now, domain_allows, max_range, object_box, object_domain, reaches, reaches_point, within_range,
};

/// `systems/combat.md > AttackPhase`, one value of the design's BaseAttackChannel (`systems/controls/behaviors.md >
/// BaseAttackChannel`). The phases run in this order; after Reloading the sequence is over.
#[derive(Copy, Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum AttackPhase {
    /// `> Aiming` (`> BaseAiming`): the unit turns to face `target`, at most its TurnRate per tick. Interruptible:
    /// a different command, or a target that is no longer valid (gone, or out of range), cancels the sequence before
    /// anything is fired.
    Aiming { target: ObjectId },
    /// `> Firing` (`> BaseFiring`): not interruptible. The attack's effect lands on `target` at the end of the phase.
    /// `None`: the target was destroyed meanwhile, and the attack hits nothing (a placeholder reading: the design
    /// doesn't say).
    Firing { target: Option<ObjectId> },
    /// `> Cooldown`: not interruptible; the unit is still unresponsive.
    Cooldown,
    /// `> Reloading`: interruptible: the unit is free to do other things, but its next attack waits for the reload
    /// to end, which keeps counting (a placeholder reading: interrupting it doesn't reset it).
    Reloading,
}

impl AttackPhase {
    /// `systems/combat.md > AttackPhase > Interruptible`.
    pub fn interruptible(self) -> bool {
        match self {
            AttackPhase::Aiming { .. } | AttackPhase::Reloading => true,
            AttackPhase::Firing { .. } | AttackPhase::Cooldown => false,
        }
    }
}

/// A unit's attack capability (`systems/combat.md > AttackAttributes`: only units that can attack have one).
#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
pub struct AttackState {
    /// The object the unit's behaviour engages (`systems/combat.md > AttackTarget > UnitTarget`), set by its order
    /// (`orders::update`); `None`: it engages nothing.
    pub target: Option<ObjectId>,
    /// The phase of the running attack sequence; `None` while none runs and the weapon is ready.
    pub phase: Option<AttackPhase>,
    /// Ticks spent in `phase` so far.
    pub elapsed: u32,
}

impl AttackState {
    /// The unit's behaviour engages `target`, or nothing. `systems/combat.md > Aiming`: a different command
    /// interrupts an attack being aimed at another target; once Firing has begun, the sequence runs on (`> Firing`,
    /// `> Cooldown`), and a reload keeps counting (`> Reloading`).
    pub fn engage(&mut self, target: Option<ObjectId>) {
        if let Some(AttackPhase::Aiming { target: aimed }) = self.phase
            && Some(aimed) != target
        {
            self.cancel();
        }
        self.target = target;
    }

    /// The removal path (`Sim::remove_objects`): the objects `gone` (sorted by id) left the game. The unit no longer
    /// engages them; an attack aimed at one is cancelled, and one being fired at one hits nothing.
    pub fn forget(&mut self, gone: &[ObjectId]) {
        let is_gone = |id: ObjectId| gone.binary_search(&id).is_ok();
        if self.target.is_some_and(is_gone) {
            self.target = None;
        }
        match self.phase {
            Some(AttackPhase::Aiming { target }) if is_gone(target) => self.cancel(),
            Some(AttackPhase::Firing { target: Some(target) }) if is_gone(target) => {
                self.phase = Some(AttackPhase::Firing { target: None });
            }
            _ => {}
        }
    }

    /// The sequence ends without an attack.
    fn cancel(&mut self) {
        self.phase = None;
        self.elapsed = 0;
    }
}

/// `systems/combat.md > AttackPhase`, `> UnitBaseSource`: whether `o`'s attack sequence lets its base move this
/// tick. A unit that attacks from its base can't move while Aiming, Firing or in Cooldown; it may while Reloading
/// and when no sequence runs. A turret unit's base always may (`> TurretSource`, `TurretSourceBaseActions`), and so
/// may every object without an attack. The movement step (5d) holds a unit that may not, keeping its goal; the
/// behaviours (5a) don't start a chase then. Before 5e it reads the phase the last tick left, which is the one
/// this tick runs in; a Stop or a new target in 5a has already cancelled an aim.
pub fn may_move(o: &Object) -> bool {
    base_is_free(o.kind, o.attack.as_ref())
}

/// `systems/combat.md > AttackPhase > UnitBaseSourceActions`: whether anything but the attack may turn `o`'s base
/// this tick. Not while Aiming, when the attack turns the base to face its target (`systems/controls/behaviors.md >
/// BaseAiming`: the orientation is overridden), nor in Firing and Cooldown, when the orientation is locked
/// (`> BaseFiring`, `> BaseCooldown`). Turret units' bases are always free (`> TurretSource`).
pub fn may_turn(o: &Object) -> bool {
    base_is_free(o.kind, o.attack.as_ref())
}

/// Both rules come to the same in the design's phase table: the base is either free or held by the attack. For a
/// unit of `kind` whose attack is in `state` (the behaviours hold a unit's parts, not the object).
pub(crate) fn base_is_free(kind: ObjectKind, state: Option<&AttackState>) -> bool {
    let Some(state) = state else {
        return true;
    };
    let turret = kind.def().unit.as_ref().is_some_and(|unit| unit.base_def().has_turret);
    turret || matches!(state.phase, None | Some(AttackPhase::Reloading))
}

/// Phase 5e (`docs/movement.md > 4`): every unit before `first_new`, in id order, runs one tick of its attack
/// sequence against a snapshot of the targets taken before any of them acts; then the tick's hits are applied
/// together. Returns the objects destroyed, in id order, for the removal path (`Sim::remove_objects`). A death that
/// removes a blocking footprint must re-sync the grids (`paths::sync`), as phase 5b does. `vision` is the last
/// tick's: what each owner sees. `turns` holds the angle phase 5d of this tick turned each unit: the aim turns a
/// unit only by what is left of its TurnRate (one budget per tick).
pub(crate) fn step(
    objects: &mut BTreeMap<ObjectId, Object>,
    map: &Map,
    vision: &Vision,
    turns: &movement::Turns,
    first_new: ObjectId,
    events: &mut Vec<SimEvent>,
) -> Vec<ObjectId> {
    let busy = |o: &Object| {
        o.attack
            .as_ref()
            .is_some_and(|a| a.target.is_some() || a.phase.is_some())
    };
    if !objects.range(..first_new).any(|(_, o)| busy(o)) {
        return Vec::new();
    }
    let targets = targeting::Targets::snapshot(map, objects);
    let mut hits = Vec::new();
    for o in objects.range_mut(..first_new).map(|(_, o)| o) {
        let Some(state) = &mut o.attack else {
            continue;
        };
        let Some(unit) = &o.kind.def().unit else {
            continue;
        };
        let Some(attrs) = &unit.attack else {
            continue;
        };
        if !o.riot_shield.is_none_or(RiotShield::allows_attack) {
            state.engage(None);
        }
        let sight = o.owner.and_then(|owner| vision.player(owner));
        let turned = turns.angle(o.id);
        let shooter = phases::Shooter::new(map, o.id, o.kind, o.pos, o.facing, unit, attrs, sight, turned);
        phases::run(state, &mut o.facing, &shooter, &targets, &mut hits, events);
    }
    damage::apply(objects, &hits, events)
}

impl StateHash for AttackPhase {
    fn hash(&self, h: &mut StateHasher) {
        match self {
            AttackPhase::Aiming { target } => {
                h.tag(0);
                target.hash(h);
            }
            AttackPhase::Firing { target } => {
                h.tag(1);
                target.hash(h);
            }
            AttackPhase::Cooldown => h.tag(2),
            AttackPhase::Reloading => h.tag(3),
        }
    }
}

impl StateHash for AttackState {
    fn hash(&self, h: &mut StateHasher) {
        let AttackState { target, phase, elapsed } = self;
        target.hash(h);
        phase.hash(h);
        elapsed.hash(h);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn state(target: Option<u32>, phase: Option<AttackPhase>) -> AttackState {
        AttackState {
            target: target.map(ObjectId),
            phase,
            elapsed: 1,
        }
    }

    #[test]
    fn a_different_target_interrupts_aiming_only() {
        let aiming = Some(AttackPhase::Aiming { target: ObjectId(5) });
        let mut s = state(Some(5), aiming);
        s.engage(Some(ObjectId(5)));
        assert_eq!(s, state(Some(5), aiming), "the same target keeps the aim");
        s.engage(Some(ObjectId(6)));
        assert_eq!((s.target, s.phase, s.elapsed), (Some(ObjectId(6)), None, 0));
        let mut s = state(Some(5), aiming);
        s.engage(None);
        assert_eq!((s.target, s.phase), (None, None), "any other command");
        for phase in [
            AttackPhase::Firing {
                target: Some(ObjectId(5)),
            },
            AttackPhase::Cooldown,
            AttackPhase::Reloading,
        ] {
            let mut s = state(Some(5), Some(phase));
            s.engage(None);
            assert_eq!(
                (s.target, s.phase, s.elapsed),
                (None, Some(phase), 1),
                "{phase:?} runs on"
            );
        }
    }

    #[test]
    fn forgetting_the_gone_clears_every_reference_to_them() {
        let gone = [ObjectId(2), ObjectId(5)];
        let mut aiming = state(Some(5), Some(AttackPhase::Aiming { target: ObjectId(5) }));
        aiming.forget(&gone);
        assert_eq!((aiming.target, aiming.phase, aiming.elapsed), (None, None, 0));
        let mut firing = state(
            Some(5),
            Some(AttackPhase::Firing {
                target: Some(ObjectId(5)),
            }),
        );
        firing.forget(&gone);
        assert_eq!(firing, state(None, Some(AttackPhase::Firing { target: None })));
        // Others are untouched.
        let mut other = state(Some(3), Some(AttackPhase::Aiming { target: ObjectId(3) }));
        other.forget(&gone);
        assert_eq!(other, state(Some(3), Some(AttackPhase::Aiming { target: ObjectId(3) })));
    }

    #[test]
    fn aiming_and_reloading_are_interruptible() {
        assert!(AttackPhase::Aiming { target: ObjectId(1) }.interruptible());
        assert!(!AttackPhase::Firing { target: None }.interruptible());
        assert!(!AttackPhase::Cooldown.interruptible());
        assert!(AttackPhase::Reloading.interruptible());
    }
}
