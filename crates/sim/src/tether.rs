//! (M6-GD) The operator's drone on its leash — plan decision T91, `factions/gdo/units/drone_operator.md > Launching and
//! piloting`: "The drone is a separate flying unit **controlled directly by the player**"; "Only **one drone can be
//! active at a time** per operator"; "The Drone Operator **cannot move** while it has a drone active".
//!
//! - **The launch** (`OrderKind::Launch`, `orders::launch`): the step right after phase 5a takes a drone out of the
//!   operator's stock (`crate::stock`) and spawns it over the operator (design change 0421: "launching doesn't change
//!   the selection"), a unit of the stock's kind, with a [`Tether`] naming its operator, whose `Object::drone` names
//!   it (`SimEvent::DroneLaunched`). It is a new object: it moves and attacks from the next tick on.
//! - **The operator holds still** while its drone is out ([`holds`]): it doesn't move (`combat::may_move`), and the
//!   gate refuses it every order that would move it (`Reject::DroneOut`, [`allows`]), with feedback.
//! - **How a drone ends** (`factions/gdo/units/drone_operator.md > Drone limits — how a drone ends`), the tether's
//!   step ([`ends`]), right after phase 5d, for the drones before the tick's new objects, in id order, the first that
//!   applies:
//!   1. **Its operator dies** ([`Fall::Operator`]): "Drone falls, no explosion". The removal path takes the drone with
//!      its operator, in the same call (`Sim::remove_objects`); the step catches an operator that has left the map
//!      some other way.
//!   2. **The leash** ([`Fall::Leash`]): its Silhouette more than its type's leash from its operator's, edge to edge
//!      like every range (design change 0421: "measured **edge to edge** like ranges (`systems/combat.md > Range
//!      Measurement`)"), as phase 5d left them: "falls and explodes". Its explosion, its attack's blast
//!      (`combat::Projectile::dropped`), lands where it fell, in this tick's phase 5e, at the leash's edge, give or
//!      take a tick's flight: on the ground units there, whoever owns them (`systems/combat.md > AoEDamage`, friendly
//!      fire), never on air units ("Explosion targets ground units only").
//!   3. **The battery** ([`Fall::Battery`]): its type's frames after the launch ("battery 20 s (320 frames)"), it falls
//!      without exploding, in the tick [`Tether::battery_until`].
//!
//!   Falling is instant (plan decision T91, D33's placeholder): the drone leaves the game in the tick its end comes,
//!   with a `SimEvent::DroneFell` saying why and a `SimEvent::Died`, before phase 5e, so it attacks nothing then.
//!   Placeholder (not in the design: which end comes first when two come in one tick): the order above.
//! - **Its explosion on enemies** is its attack, "a melee-range attack that it can use only once" (`combat`: a
//!   FullyConnected area attack bursts round its attacker, and a tethered unit's spends it, `UnitDef::tether`): the
//!   drone leaves with phase 5e's dead, with a `SimEvent::Died`.
//!
//! Every end frees the operator: the removal path clears its `Object::drone` (`Sim::forget_references`), and its
//! Build Drone may start (`crate::stock`: "only once the operator's drone has **ended**"). There is no recall: "once
//! launched, a drone is spent".

use std::collections::BTreeMap;

use serde::{Deserialize, Serialize};

use crate::defs::{ObjectKind, TetherDef};
use crate::hash::{StateHash, StateHasher};
use crate::ids::ObjectId;
use crate::object::Object;
use crate::orders::OrderKind;
use crate::shape::Shape;
use crate::units::{Tick, gu};

/// A launched drone's leash (module docs).
#[derive(Copy, Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Tether {
    /// The operator that launched it, whose `Object::drone` it is.
    pub operator: ObjectId,
    /// The tick its battery runs out in: the launch's, plus its type's battery (`TetherDef::battery`).
    pub battery_until: Tick,
}

/// Why a drone fell (module docs, `SimEvent::DroneFell`).
#[derive(Copy, Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum Fall {
    /// Its operator died, or left the map: no explosion.
    Operator,
    /// It went past its leash: it explodes where it falls.
    Leash,
    /// Its battery ran out: no explosion.
    Battery,
}

impl Fall {
    /// Whether the drone explodes as it falls: only past its leash ("falls and explodes").
    pub fn explodes(self) -> bool {
        self == Fall::Leash
    }
}

/// The leash and battery of `kind`'s type, if it has one.
pub fn def_of(kind: ObjectKind) -> Option<TetherDef> {
    kind.def().unit.as_ref()?.tether
}

/// Whether `o` holds still for its drone (module docs): an operator with its drone out "**cannot move**".
pub fn holds(o: &Object) -> bool {
    o.drone.is_some()
}

/// Whether a unit that holds still for its drone ([`holds`]) takes an order of `kind` (module docs): only one that
/// doesn't move it, a Stop, a Hold Position, (M5-N4b) a channel or (M5-K) a Target Lock, which stand where the unit
/// is; a Launch has its own refusal while the drone is out (`Reject::DroneOut`); and (M6-GA) a vehicle deploy, which
/// stands too (no operator deploys). Every other order moves the unit, to a point, an object or within range.
pub fn allows(kind: OrderKind) -> bool {
    match kind {
        OrderKind::Stop | OrderKind::Hold | OrderKind::Shroud | OrderKind::TargetLock(_) | OrderKind::Launch => true,
        OrderKind::Move(_) | OrderKind::Attack(_) | OrderKind::AttackMove(_) | OrderKind::Patrol(_) => false,
        OrderKind::Gather(_) | OrderKind::DropOff(_) | OrderKind::Deploy { .. } | OrderKind::Enter(_) => false,
        OrderKind::AttackGround(_) | OrderKind::Build { .. } | OrderKind::Assist(_) | OrderKind::Repair(_) => false,
        OrderKind::Reverse(_) => false,
        // (M6-GH1) An aircraft's flight to its Airfield (no operator is one).
        OrderKind::Rebase(_) | OrderKind::ReturnToBase => false,
        OrderKind::VehicleDeploy { .. } => true,
        // (M6-ST1) A carrier's unload, which stands too (no operator carries).
        OrderKind::UnloadAll => true,
        // A chopper's flight to a spot (no operator is one).
        OrderKind::PickUpSupplies(_) | OrderKind::DropOffSupplies(_) | OrderKind::AttachToTower(_) => false,
    }
}

/// The tether's step (module docs): the drones before `first_new` whose end comes in `tick`, in id order, each with its
/// operator and why it falls, as phase 5d left them.
pub fn ends(objects: &BTreeMap<ObjectId, Object>, first_new: ObjectId, tick: Tick) -> Vec<(ObjectId, ObjectId, Fall)> {
    let mut ends = Vec::new();
    for (&drone, o) in objects.range(..first_new) {
        let Some(tether) = o.tether else {
            continue;
        };
        let operator = objects.get(&tether.operator).filter(|op| op.drone == Some(drone));
        let fall = match operator {
            None => Some(Fall::Operator),
            Some(operator) if past_leash(o, operator) => Some(Fall::Leash),
            Some(_) if tick >= tether.battery_until => Some(Fall::Battery),
            Some(_) => None,
        };
        ends.extend(fall.map(|fall| (drone, tether.operator, fall)));
    }
    ends
}

/// Whether drone `drone` is beyond its type's leash from `operator`, edge to edge (module docs): the gap between their
/// shapes as they lie (`Shape::gap`) longer than the leash. A drone of a type without one (a hand-edited save's) is on
/// no leash.
pub fn past_leash(drone: &Object, operator: &Object) -> bool {
    let Some(def) = def_of(drone.kind) else {
        return false;
    };
    Shape::of(drone).gap(&Shape::of(operator)) > gu(def.leash)
}

impl StateHash for Tether {
    fn hash(&self, h: &mut StateHasher) {
        let Tether {
            operator,
            battery_until,
        } = self;
        operator.hash(h);
        battery_until.hash(h);
    }
}

#[cfg(test)]
mod tests;
