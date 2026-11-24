//! Objects that change their kind in place — plan decision T15: one object through every transformation. A Drone
//! deploying into a Beacon (`factions/colonists/economy.md > Beacon Deployment`, slice M3-F), a Beacon warping in a
//! building (`> Warp In`, M3-E) and a Legionnaire converting its weapon (`factions/colonists/units.md > Weapon
//! Conversion`, M3-B) each stay the same object, with the same `ObjectId`, so selections, control groups, rally
//! points and the fog's records keep it ("the Drone unit itself transforms into the Beacon in place").
//!
//! [`Sim::change_kind`] is the one place an object's kind changes. [`Capabilities::fresh`] is what an object of a
//! kind starts with, for a new object (`Sim::insert_object`) and a changed one alike, so the two can't drift apart.

use std::collections::BTreeMap;

use super::Sim;
use crate::combat::AttackState;
use crate::construction::{self, Constructor};
use crate::defs::{Category, ObjectDef, ObjectKind};
use crate::economy::{self, Extractor, Worker};
use crate::events::SimEvent;
use crate::ids::ObjectId;
use crate::mover::Mover;
use crate::nav;
use crate::object::{Object, Placement};
use crate::orders::Orders;
use crate::production::Production;
use crate::riot_shield::RiotShield;

/// The capabilities of an object (`Object`'s optional parts, but a map resource's state), as its kind's definition
/// gives them.
pub(super) struct Capabilities {
    pub orders: Option<Orders>,
    pub mover: Option<Mover>,
    pub attack: Option<AttackState>,
    pub riot_shield: Option<RiotShield>,
    pub extractor: Option<Extractor>,
    pub production: Option<Production>,
    pub constructor: Option<Constructor>,
    pub worker: Option<Worker>,
}

impl Capabilities {
    /// What an object of `kind` placed at `placement` starts with, among `objects`.
    pub(super) fn fresh(objects: &BTreeMap<ObjectId, Object>, kind: ObjectKind, placement: Option<&Placement>) -> Self {
        let def = kind.def();
        let unit = def.unit.as_ref();
        // Units take orders and move (`systems/units.md > Unit`); they start idle.
        let is_unit = unit.is_some();
        Self {
            orders: is_unit.then(Orders::default),
            mover: is_unit.then(Mover::default),
            // `systems/combat.md > AttackAttributes`: units that can attack; their weapon starts ready.
            attack: unit
                .and_then(|unit| unit.attack.as_ref())
                .map(|_| AttackState::default()),
            // `factions/gdo/units.md > Peacekeeper > Riot Shield`: units that carry one; it starts down on the
            // Peacekeeper, up on the Peacekeeper (Shielded) (design change 0173).
            riot_shield: unit
                .and_then(|unit| unit.riot_shield.as_ref())
                .map(|shield| RiotShield::fresh(kind, shield)),
            // `factions/gdo/economy.md > ExtractionPlate`: a plate mines the patch under it.
            extractor: economy::mining::extractor_for(objects, kind, placement),
            // `factions/gdo/structures.md > Barracks`: a structure that produces units starts with an empty queue.
            production: def
                .structure
                .as_ref()
                .and_then(|structure| structure.production.as_ref())
                .map(|_| Production::default()),
            // `factions/gdo/structures.md > DeploymentCenter`, `factions/gdo/economy.md > ExtractionFacility`: a
            // constructor starts idle.
            constructor: construction::constructor_for(kind),
            // `factions/colonists/units.md > Drone > Gathering`: a worker starts empty, with no drop-off assigned.
            worker: unit.and_then(|unit| unit.worker).map(|_| Worker::default()),
        }
    }
}

/// The state of a capability after a change of kind: the old state if `keep` and both kinds have the capability,
/// else what the new kind starts with (`None` if it has none).
fn rebuilt<T>(old: Option<T>, fresh: Option<T>, keep: bool) -> Option<T> {
    match (old, fresh) {
        (Some(old), Some(_)) if keep => Some(old),
        (_, fresh) => fresh,
    }
}

/// The grid a unit of this type plans and collides on (`docs/movement.md > 5.2`): its base's class and its size class.
fn grid_key(def: &ObjectDef) -> Option<(Option<nav::NavClass>, u8)> {
    let class = def.unit.as_ref()?.base_def().nav_class();
    Some((class, nav::size_class(def.silhouette()?)))
}

impl Sim {
    /// Plan decision T15: object `id` becomes a `kind`, placed at `placement` (`None` for a unit), and stays the same
    /// object. Returns whether it exists; nothing happens to an id that doesn't.
    ///
    /// - **Kept**: its id, owner and facing. Its position, unless it gets a placement: a tile-based object stands at
    ///   its placement's centre (`Placement::center`), and jumps there, as a teleport does (`prev_pos` too). Its HP,
    ///   at most the new kind's MaxHP, and its Shields, at most the new kind's MaxShields (`crate::shields`; full
    ///   for an object that had none, none for a kind without them): a transformation in progress sets its
    ///   interpolated values itself (`factions/colonists/economy.md > Beacon Deployment > HP/Shields Interpolation`,
    ///   design change 0110), and a Weapon Conversion's forms have the same maxima (`crate::conversion`). A
    ///   conversion or a transformation under way is its caller's to end.
    /// - **Capabilities**, rebuilt from the new kind's definition ([`Capabilities::fresh`]): one the old kind had with
    ///   the same definition keeps its state, the others start as on a new object, and those the new kind lacks go. So
    ///   a unit that stays a unit keeps its orders and its move (design change 0150: "Orders are kept"), and plans its
    ///   path again if it walks on another grid. A unit that keeps an attack keeps what it engages (its order's
    ///   target); with another weapon, its attack sequence starts again, the new weapon ready (design change 0152:
    ///   a Weapon Conversion), and with the same one it goes on as it was, a reload or a deferred ability included (the
    ///   Riot Shield's two types, design change 0173: "They share everything except what the shield changes"). A Riot
    ///   Shield that the new kind shares keeps how far up it is, whose speed its caller settles (`riot_shield`); one
    ///   that goes takes its slowdown with it.
    /// - **Paid state is never lost**: a production queue or a construction the new kind has no place for is
    ///   refunded first, as when the object leaves the game (`Sim::refund_production`, `Sim::refund_constructors`).
    /// - A [`SimEvent::KindChanged`] (after the refunds' events).
    ///
    /// The new kind counts from the change on, its footprint and its capabilities included (T15: "the object is
    /// the target kind from the first frame"); a caller whose object isn't functional yet (a building warping in)
    /// holds those back itself, and keeps its `Object::transform` (`sim/transform.rs`: a Warp In and its cancel).
    /// Phase 5b brings the grids up to date with the footprint, so a kind changes before it, in the commands phase
    /// or in a step right after 5a, or the caller syncs them (`paths::sync`). Map resources never change kind.
    pub(crate) fn change_kind(&mut self, id: ObjectId, kind: ObjectKind, placement: Option<Placement>) -> bool {
        let Some(o) = self.objects.get(&id) else {
            return false;
        };
        let from = o.kind;
        let (old, new) = (from.def(), kind.def());
        debug_assert!(
            old.category != Category::Resource && new.category != Category::Resource,
            "map resources never change kind: {from:?} to {kind:?}"
        );
        debug_assert_eq!(
            new.tile_size().is_some(),
            placement.is_some(),
            "a tile-based kind has a placement, a unit none: {kind:?}"
        );
        let production_of = |def: &ObjectDef| def.structure.as_ref().and_then(|s| s.production);
        let keeps_production = production_of(old) == production_of(new);
        let keeps_constructor = construction::recipes(from) == construction::recipes(kind);
        if !keeps_production {
            self.refund_production(id);
        }
        if !keeps_constructor {
            self.refund_constructors(&[id]);
        }
        let fresh = Capabilities::fresh(&self.objects, kind, placement.as_ref());
        let tick = self.tick;
        let Some(o) = self.objects.get_mut(&id) else {
            return false;
        };
        let Object {
            id: _,
            kind: object_kind,
            owner: _,
            pos,
            prev_pos,
            facing: _,
            prev_facing: _,
            hp,
            shields,
            placement: object_placement,
            resource,
            orders,
            mover,
            attack,
            riot_shield,
            extractor,
            production,
            constructor,
            worker,
            transform: _,
            conversion: _,
        } = o;
        *object_kind = kind;
        if let Some(placement) = &placement {
            *pos = placement.center();
            *prev_pos = *pos;
        }
        *object_placement = placement;
        *hp = new.max_hp.map(|max| hp.map_or(max, |hp| hp.min(max)));
        *shields = new
            .max_shields
            .map(|max| shields.map_or(max, |shields| shields.min(max)));
        *resource = None;
        let unit_of = |def: &'static ObjectDef| def.unit.as_ref();
        let keeps_shield = unit_of(old).map(|u| u.riot_shield) == unit_of(new).map(|u| u.riot_shield);
        *orders = rebuilt(orders.take(), fresh.orders, true);
        *mover = rebuilt(mover.take(), fresh.mover, true);
        if let Some(mover) = mover {
            if grid_key(old) != grid_key(new) {
                mover.repath(tick);
            }
            if !keeps_shield {
                mover.speed_factor = Mover::default().speed_factor;
            }
        }
        let weapon_of = |def: &'static ObjectDef| unit_of(def).and_then(|u| u.attack);
        let keeps_weapon = weapon_of(old) == weapon_of(new);
        *attack = match (attack.take(), fresh.attack) {
            (Some(old), Some(_)) if keeps_weapon => Some(old),
            (Some(old), Some(fresh)) => Some(AttackState {
                target: old.target,
                ..fresh
            }),
            (_, fresh) => fresh,
        };
        *riot_shield = rebuilt(riot_shield.take(), fresh.riot_shield, keeps_shield);
        let mining_of = |def: &ObjectDef| def.structure.as_ref().and_then(|s| s.mining);
        *extractor = rebuilt(extractor.take(), fresh.extractor, mining_of(old) == mining_of(new));
        *production = rebuilt(production.take(), fresh.production, keeps_production);
        *constructor = rebuilt(constructor.take(), fresh.constructor, keeps_constructor);
        // A worker that stays one keeps its load and its drop-off; a Drone that becomes a Beacon loses what it
        // carried with the capability.
        *worker = rebuilt(worker.take(), fresh.worker, true);
        self.events.push(SimEvent::KindChanged { id, from, to: kind });
        true
    }
}

#[cfg(test)]
mod tests;
