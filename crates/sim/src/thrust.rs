//! A unit's thrusts (`factions/syndicate/units/prism.md > Movement`): a few charges, "each giving a quick burst of
//! displacement in a chosen direction", which never come back. A unit whose type has thrusts (`UnitDef::thrust`, a
//! `ThrustDef`) has them from the start, none spent ([`Thrust`], `Object::thrust`).
//!
//! - **The order** (`OrderKind::Thrust`, [`run`]; `factions/syndicate/units/prism.md > Placeholder stats`, design change
//!   0434): a point clicked, toward which the unit thrusts "the full 3 grid units whatever the distance". It queues
//!   with Shift, and each thrust spends its charge as it starts. The gate refuses it to a unit without thrusts
//!   (`Reject::LacksAbility`) and to one with every charge spent (`Reject::NoCharges`, [`charges_left`]), and judges a
//!   queued thrust as it is given: **Placeholder (Q0198)**, one that starts with no charge left ends at once,
//!   without moving, and the orders behind it go on. A thrust that starts while the burst of another is under way
//!   waits for it to end before spending its charge, so that each charge moves the unit its full distance:
//!   **Placeholder (not in the design: an order given during a burst)**; the burst itself runs its course whatever
//!   the orders.
//! - **The burst** ([`Burst`], [`burst_step`]): the def's distance spread evenly over its frames, toward the point from
//!   where the unit stands as its thrust starts (its facing, for a point it stands on: **Placeholder (not in the
//!   design: a thrust at its own position)**). The movement step adds it after the unit's own move, apart from it: the
//!   unit's velocity and momentum stay its own move's, so that the drift after a burst is its movement's alone
//!   (`Mover::velocity`). On the map and against what blocks it as its own move is (`movement::step`).
//! - **The store**: `Object::thrust`, hashed and saved only while present (Object tag 17), so that a game without a
//!   unit that thrusts hashes as it did before them.

#[cfg(test)]
mod tests;

use glam::Vec2;
use serde::{Deserialize, Serialize};

use crate::defs::{ObjectKind, ThrustDef};
use crate::hash::{StateHash, StateHasher};
use crate::math::facing_vector;
use crate::object::Object;
use crate::units::gu;

/// A unit's thrusts (`Object::thrust`, module docs): the charges it has spent, and its burst under way.
#[derive(Copy, Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Thrust {
    /// The charges it has spent, one as each thrust starts; none ever comes back.
    #[serde(default, skip_serializing_if = "is_zero")]
    pub used: u32,
    /// Its burst under way, if one is.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub burst: Option<Burst>,
}

/// A thrust's burst (module docs): the unit's move in each of its ticks, on top of its own.
#[derive(Copy, Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Burst {
    /// Its move in each tick, space units.
    pub step: Vec2,
    /// The ticks it has left, the one under way among them.
    pub left: u32,
}

fn is_zero(n: &u32) -> bool {
    *n == 0
}

impl StateHash for Thrust {
    fn hash(&self, h: &mut StateHasher) {
        let Thrust { used, burst } = self;
        used.hash(h);
        burst.hash(h);
    }
}

impl StateHash for Burst {
    fn hash(&self, h: &mut StateHasher) {
        let Burst { step, left } = self;
        step.hash(h);
        left.hash(h);
    }
}

/// The thrusts of a new object of `kind` (module docs): none spent for a type with thrusts, `None` for every other.
pub(crate) fn fresh(kind: ObjectKind) -> Option<Thrust> {
    def_of(kind).map(|_| Thrust::default())
}

/// The thrust def of the type `kind`, if its units thrust (`UnitDef::thrust`).
pub fn def_of(kind: ObjectKind) -> Option<ThrustDef> {
    kind.def().unit.as_ref()?.thrust
}

/// Whether `o` thrusts: its type has thrusts, and so has it. The gate's [`Reject::LacksAbility`] for the others.
///
/// [`Reject::LacksAbility`]: crate::Reject::LacksAbility
pub fn thrusts(o: &Object) -> bool {
    o.thrust.is_some() && def_of(o.kind).is_some()
}

/// The charges `o` has left: its type's less those it has spent, 0 for a unit that doesn't thrust. None left is the
/// gate's [`Reject::NoCharges`], and what greys its Thrust.
///
/// [`Reject::NoCharges`]: crate::Reject::NoCharges
pub fn charges_left(o: &Object) -> u32 {
    match (def_of(o.kind), &o.thrust) {
        (Some(def), Some(thrust)) => def.charges.saturating_sub(thrust.used),
        _ => 0,
    }
}

/// A burst toward `at` from `pos` for a unit facing `facing` (module docs): `def`'s distance spread evenly over its
/// frames.
fn burst_toward(def: &ThrustDef, pos: Vec2, facing: f32, at: Vec2) -> Burst {
    let frames = def.frames.max(1);
    let toward = (at - pos).try_normalize().unwrap_or_else(|| facing_vector(facing));
    Burst {
        step: toward * (gu(def.distance) / frames as f32),
        left: frames,
    }
}

/// One tick of a Thrust toward `at` (`OrderKind::Thrust`, module docs) of a unit of type `kind` at `pos`, facing
/// `facing`, whose thrusts are `thrust`; `fired` is its behaviour's (`Behaviour::Thrust`): whether it has spent its
/// charge. Returns whether the order is complete: once its burst is over, and at once with no charge left or for a
/// unit that doesn't thrust (a save edited by hand).
pub(crate) fn run(
    thrust: Option<&mut Thrust>,
    kind: ObjectKind,
    pos: Vec2,
    facing: f32,
    at: Vec2,
    fired: &mut bool,
) -> bool {
    let (Some(thrust), Some(def)) = (thrust, def_of(kind)) else {
        return true;
    };
    if *fired {
        return thrust.burst.is_none();
    }
    // Another thrust's burst runs its course first.
    if thrust.burst.is_some() {
        return false;
    }
    // Placeholder (Q0198): no charge left, no move.
    if thrust.used >= def.charges {
        return true;
    }
    thrust.used += 1;
    thrust.burst = Some(burst_toward(&def, pos, facing, at));
    *fired = true;
    false
}

/// One tick of `thrust`'s burst, if one is under way (module docs): its move in this tick, which the movement step adds
/// to the unit's own. Its last tick ends it.
pub(crate) fn burst_step(thrust: &mut Option<Thrust>) -> Option<Vec2> {
    let thrust = thrust.as_mut()?;
    let burst = thrust.burst.as_mut()?;
    let step = burst.step;
    burst.left = burst.left.saturating_sub(1);
    if burst.left == 0 {
        thrust.burst = None;
    }
    Some(step)
}

/// How far a burst may have moved a unit of type `kind` in the tick between its thrusts `before` and `after`, if one
/// did: a burst was under way at either end, or a charge was spent in between (a burst of one tick). The movement
/// invariants' allowance for it (`testing::invariants`, I3 and I5/I7).
pub fn burst_moved(kind: ObjectKind, before: Option<&Thrust>, after: Option<&Thrust>) -> Option<f32> {
    let used = |thrust: Option<&Thrust>| thrust.map_or(0, |thrust| thrust.used);
    let bursts = [before, after].map(|thrust| thrust.and_then(|thrust| thrust.burst));
    if bursts.iter().all(Option::is_none) && used(before) == used(after) {
        return None;
    }
    let def = def_of(kind).map_or(0.0, |def| gu(def.distance) / def.frames.max(1) as f32);
    Some(
        bursts
            .iter()
            .flatten()
            .map(|burst| burst.step.length())
            .fold(def, f32::max),
    )
}
