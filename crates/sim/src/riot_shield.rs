//! The Peacekeeper's Riot Shield — `factions/gdo/units.md > Peacekeeper > Riot Shield (toggled ability)`.
//!
//! A **mode**, not an order. The player raises and lowers it with an ability command (`Command::Ability`), which
//! applies at once and leaves the unit's orders alone, so the shield goes up while the unit walks, holds or stands
//! idle. A mode isn't a task (design change 0023, `systems/units.md > UnitCollision > Ground Collision > Friendly
//! yielding`): an idle unit with its shield up still yields, at its shielded speed.
//!
//! **Orders are kept** (design change 0150, `> Riot Shield > Orders are kept`): toggling never clears the command
//! queue, and an order the unit can't carry out while shielded is on hold, not dropped. An explicit Attack follows
//! its target while the shield is up and attacks it again once it is down (`orders::engage::attack`); an AttackMove
//! or a Patrol only moves meanwhile, and engages along the way again once it is down (`orders::travel::march`: the
//! scans find nothing while the unit may not attack). **Not queueable** (design change 0151): the toggle always
//! applies at once; the command has no queue flag.
//!
//! - **State**: a unit whose type has a [`RiotShieldDef`] has a [`RiotShield`] (`Object::riot_shield`): Lowered,
//!   Raising, Raised or Lowering, a transition counting its ticks down. It starts Lowered, as the unit behaves
//!   normally (the design doesn't say). Hashed and serialized like all state.
//! - **Toggling** ([`toggle`]): applied with the tick's commands. A transition takes `RiotShieldDef::toggle_ticks`,
//!   counted by [`step`] after the commands and before phase 5a, so a toggle in tick `t` leaves the shield Raised (or
//!   Lowered) at the end of tick `t + toggle_ticks − 1`, and every phase of that tick already sees it so.
//! - **Effects** (`> Riot Shield`: "While the shield is up"):
//!   - MaxSpeed × `RiotShieldDef::speed_factor` (halved): the unit's `Mover::speed_factor`, written whenever the
//!     shield's mode changes ([`RiotShield::speed_factor`]), so movement and yielding go at that speed;
//!   - **no attacking** ([`RiotShield::allows_attack`]): the behaviours engage nothing and pick no target
//!     (`orders::world::Unit::attrs`: no auto-targeting either), phase 5e drops whatever is still engaged
//!     (`combat::step`), and the gate takes no new Attack order for the unit (`Sim::validate`,
//!     `Reject::CannotAttack`: "Attack orders are not possible"; design change 0075), while one it had goes on as a
//!     follow (above);
//!   - incoming damage × `RiotShieldDef::damage_factor` (halved), a pre-armor multiplier from every direction
//!     (`combat::pre_armor_multiplier`, `systems/combat.md > DamageModifierOrder` step 1; [`damage_factor`]).
//!
//! Placeholders, where the design is silent:
//! - **The transitions**: the shield's drawbacks hold from the moment it starts going up until it is all the way
//!   down again (half speed, no attacking while Raising and Lowering too); its protection only while it is all the
//!   way up (Raised). So a toggle never pays off between two enemy shots, which is what the toggle time is for.
//! - **A reversal** half-way (lowering a shield on its way up, or raising one on its way down) turns back from where
//!   the shield is: it takes as many ticks as the shield had spent going the other way.
//! - **An attack under way**: raising the shield cancels an aim; Firing and Cooldown can't be interrupted
//!   (`systems/combat.md > AttackPhase`), so a shot already fired lands.

use std::collections::BTreeMap;

use serde::{Deserialize, Serialize};

use crate::defs::RiotShieldDef;
use crate::hash::{StateHash, StateHasher};
use crate::ids::ObjectId;
use crate::object::Object;

/// The mode of a unit's Riot Shield (module docs).
#[derive(Copy, Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub enum RiotShield {
    /// Down: the unit behaves normally (`> Riot Shield`: "While the shield is down").
    #[default]
    Lowered,
    /// On its way up, `left` ticks to go.
    Raising { left: u32 },
    /// Up.
    Raised,
    /// On its way down, `left` ticks to go.
    Lowering { left: u32 },
}

impl RiotShield {
    /// How far up it is, in ticks of a toggle time of `toggle_ticks`: 0 down, `toggle_ticks` up.
    fn height(self, toggle_ticks: u32) -> u32 {
        match self {
            RiotShield::Lowered => 0,
            RiotShield::Raising { left } => toggle_ticks.saturating_sub(left),
            RiotShield::Raised => toggle_ticks,
            RiotShield::Lowering { left } => left.min(toggle_ticks),
        }
    }

    /// The player raises (`raise`) or lowers the shield: it sets off that way from where it is, and is there at once
    /// with a toggle time of 0. Raising a shield that is up or going up changes nothing, nor does lowering one that is
    /// down or going down.
    #[must_use]
    pub fn toggle(self, raise: bool, toggle_ticks: u32) -> Self {
        let height = self.height(toggle_ticks);
        match (raise, self) {
            (true, RiotShield::Raising { .. } | RiotShield::Raised)
            | (false, RiotShield::Lowering { .. } | RiotShield::Lowered) => self,
            (true, RiotShield::Lowered | RiotShield::Lowering { .. }) => match toggle_ticks - height {
                0 => RiotShield::Raised,
                left => RiotShield::Raising { left },
            },
            (false, RiotShield::Raised | RiotShield::Raising { .. }) => match height {
                0 => RiotShield::Lowered,
                left => RiotShield::Lowering { left },
            },
        }
    }

    /// One tick of a transition ([`step`]): the last one leaves the shield Raised or Lowered.
    #[must_use]
    pub fn advance(self) -> Self {
        match self {
            RiotShield::Raising { left } if left > 1 => RiotShield::Raising { left: left - 1 },
            RiotShield::Raising { .. } => RiotShield::Raised,
            RiotShield::Lowering { left } if left > 1 => RiotShield::Lowering { left: left - 1 },
            RiotShield::Lowering { .. } => RiotShield::Lowered,
            RiotShield::Lowered | RiotShield::Raised => self,
        }
    }

    /// Whether the unit may attack: only with its shield all the way down (`> Riot Shield`: it "cannot attack"
    /// while the shield is up; module docs for the transitions).
    pub fn allows_attack(self) -> bool {
        self == RiotShield::Lowered
    }

    /// Whether it is up or on its way up: a toggle would lower it.
    pub fn is_going_up(self) -> bool {
        matches!(self, RiotShield::Raising { .. } | RiotShield::Raised)
    }

    /// The unit's `Mover::speed_factor` in this mode: `def`'s while the shield isn't all the way down.
    pub fn speed_factor(self, def: &RiotShieldDef) -> f32 {
        if self == RiotShield::Lowered {
            1.0
        } else {
            def.speed_factor
        }
    }

    /// The pre-armor multiplier on hits against the unit in this mode: `def`'s while the shield is all the way up.
    pub fn damage_factor(self, def: &RiotShieldDef) -> f32 {
        if self == RiotShield::Raised {
            def.damage_factor
        } else {
            1.0
        }
    }

    /// How far up it is, from 0 (down) to 1 (up), with a toggle time of `toggle_ticks`: for drawing it.
    pub fn progress(self, toggle_ticks: u32) -> f32 {
        if toggle_ticks == 0 {
            return if self == RiotShield::Raised { 1.0 } else { 0.0 };
        }
        self.height(toggle_ticks) as f32 / toggle_ticks as f32
    }
}

/// The Riot Shield of `o`'s type, if it has one.
pub fn def_of(o: &Object) -> Option<&'static RiotShieldDef> {
    o.def().unit.as_ref()?.riot_shield.as_ref()
}

/// Whether `o`'s Riot Shield lets it attack now ([`RiotShield::allows_attack`]); true for objects without one.
pub fn allows_attack(o: &Object) -> bool {
    o.riot_shield.is_none_or(RiotShield::allows_attack)
}

/// The pre-armor multiplier `o`'s Riot Shield gives the hits against it ([`RiotShield::damage_factor`]).
pub fn damage_factor(o: &Object) -> f32 {
    match (o.riot_shield, def_of(o)) {
        (Some(shield), Some(def)) => shield.damage_factor(def),
        _ => 1.0,
    }
}

/// `Command::Ability` with `Ability::RiotShield`: the player raises (`raise`) or lowers `o`'s shield
/// ([`RiotShield::toggle`]). Nothing for an object without one.
pub(crate) fn toggle(o: &mut Object, raise: bool) {
    if let (Some(shield), Some(def)) = (o.riot_shield, def_of(o)) {
        set(o, shield.toggle(raise, def.toggle_ticks), def);
    }
}

/// The Riot Shields' transitions advance one tick ([`RiotShield::advance`]): the objects before `first_new`, in id
/// order (`ARCHITECTURE.md > Tick pipeline`: one created this tick starts in the next).
pub(crate) fn step(objects: &mut BTreeMap<ObjectId, Object>, first_new: ObjectId) {
    for o in objects.range_mut(..first_new).map(|(_, o)| o) {
        if let (Some(shield), Some(def)) = (o.riot_shield, def_of(o)) {
            set(o, shield.advance(), def);
        }
    }
}

/// Puts `o`'s shield in `mode`. A change of mode sets the unit's speed with it ([`RiotShield::speed_factor`]); the
/// shield is the only mode that sets it yet, and only when the mode changes.
fn set(o: &mut Object, mode: RiotShield, def: &RiotShieldDef) {
    if o.riot_shield == Some(mode) {
        return;
    }
    o.riot_shield = Some(mode);
    if let Some(mover) = &mut o.mover {
        mover.speed_factor = mode.speed_factor(def);
    }
}

impl StateHash for RiotShield {
    fn hash(&self, h: &mut StateHasher) {
        match self {
            RiotShield::Lowered => h.tag(0),
            RiotShield::Raising { left } => {
                h.tag(1);
                left.hash(h);
            }
            RiotShield::Raised => h.tag(2),
            RiotShield::Lowering { left } => {
                h.tag(3);
                left.hash(h);
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::defs::{ObjectKind, RIOT_SHIELD_TOGGLE_TICKS};

    const T: u32 = RIOT_SHIELD_TOGGLE_TICKS;

    /// The modes a shield goes through after a toggle at `from`, tick by tick, until it settles.
    fn course(from: RiotShield, raise: bool, toggle_ticks: u32) -> Vec<RiotShield> {
        let mut modes = vec![from.toggle(raise, toggle_ticks)];
        while let Some(&last) = modes.last().filter(|m| m.advance() != **m) {
            modes.push(last.advance());
        }
        modes
    }

    /// `> Riot Shield > Toggle time`: raising and lowering each take the toggle time, and a toggle the way the shield
    /// already goes changes nothing.
    #[test]
    fn a_toggle_takes_the_toggle_time_either_way() {
        let up = course(RiotShield::Lowered, true, 3);
        assert_eq!(
            up,
            [
                RiotShield::Raising { left: 3 },
                RiotShield::Raising { left: 2 },
                RiotShield::Raising { left: 1 },
                RiotShield::Raised,
            ]
        );
        let down = course(RiotShield::Raised, false, 3);
        assert_eq!(down.len(), 4);
        assert_eq!(down.last(), Some(&RiotShield::Lowered));
        for mode in [RiotShield::Raising { left: 2 }, RiotShield::Raised] {
            assert_eq!(mode.toggle(true, 3), mode);
        }
        for mode in [RiotShield::Lowering { left: 2 }, RiotShield::Lowered] {
            assert_eq!(mode.toggle(false, 3), mode);
        }
        // A toggle time of 0 is there at once.
        assert_eq!(RiotShield::Lowered.toggle(true, 0), RiotShield::Raised);
        assert_eq!(RiotShield::Raised.toggle(false, 0), RiotShield::Lowered);
        // The Peacekeeper's: under half a second (`> Toggle time`), more than a tick.
        const { assert!(T > 1 && T < crate::units::TICKS_PER_SECOND / 2) };
    }

    /// Placeholder (module docs): a reversal turns back from where the shield is.
    #[test]
    fn a_reversal_turns_back_from_where_the_shield_is() {
        let half_up = RiotShield::Raising { left: 4 };
        assert_eq!(half_up.toggle(false, 6), RiotShield::Lowering { left: 2 });
        assert_eq!(
            RiotShield::Lowering { left: 1 }.toggle(true, 6),
            RiotShield::Raising { left: 5 }
        );
        // Lowered in the tick it was raised: down again at once.
        assert_eq!(RiotShield::Raising { left: 6 }.toggle(false, 6), RiotShield::Lowered);
        // A save edited by hand never underflows.
        assert_eq!(RiotShield::Lowering { left: 99 }.toggle(true, 6), RiotShield::Raised);
        assert_eq!(RiotShield::Raising { left: 99 }.toggle(false, 6), RiotShield::Lowered);
        assert_eq!(RiotShield::Raising { left: 0 }.advance(), RiotShield::Raised);
    }

    /// `> Riot Shield`: the drawbacks from the start of the raise to the end of the lowering, the protection only
    /// while all the way up.
    #[test]
    fn the_effects_follow_the_mode() {
        let def = ObjectKind::Peacekeeper
            .def()
            .unit
            .as_ref()
            .unwrap()
            .riot_shield
            .unwrap();
        let modes = [
            RiotShield::Lowered,
            RiotShield::Raising { left: 1 },
            RiotShield::Raised,
            RiotShield::Lowering { left: 1 },
        ];
        let effects: Vec<(bool, f32, f32)> = modes
            .iter()
            .map(|m| (m.allows_attack(), m.speed_factor(&def), m.damage_factor(&def)))
            .collect();
        assert_eq!(
            effects,
            [
                (true, 1.0, 1.0),
                (false, 0.5, 1.0),
                (false, 0.5, 0.5),
                (false, 0.5, 1.0)
            ]
        );
        let progress: Vec<f32> = [
            RiotShield::Lowered,
            RiotShield::Raising { left: 4 },
            RiotShield::Lowering { left: 3 },
            RiotShield::Raised,
        ]
        .iter()
        .map(|m| m.progress(6))
        .collect();
        assert_eq!(progress, [0.0, 2.0 / 6.0, 0.5, 1.0]);
        assert_eq!(RiotShield::Raising { left: 2 }.progress(0), 0.0);
    }
}
