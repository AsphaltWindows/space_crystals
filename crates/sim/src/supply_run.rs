//! The GDO's supply loop — `factions/gdo/economy.md > SupplyTower` and `factions/gdo/units/supply_chopper.md`:
//! Supply Choppers fly to a Supply Delivery Station, land on it, pick up all its
//! Supplies, and drop them at a Supply Tower, which pays its owner.
//!
//! - **A chopper's state** ([`ChopperState`], `Object::chopper`): what it carries ([`Cargo`]: one kind at a time,
//!   `factions/gdo/units/supply_chopper.md > Cargo`, of which Supplies alone are in the game), the tower it is
//!   attached to (`factions/gdo/units/supply_chopper.md > SupplyChopperInstanceState`: "AttachedTower") and its landing
//!   on a spot ([`Landing`], `crate::landing`, whose `at` names the tower or the station).
//! - **The spots** ([`spot`]): a tower has one, its def's (`SupplyTowerDef::spot_offset_su`; `factions/gdo/economy.md >
//!   SupplyTower > SupplyTowerInstanceState`: "LandedChopper: ObjectInstance | None"), and so has a station, its centre
//!   (**Placeholder (not in the design: where on a Supply Delivery Station a chopper lands, and how many at once)**:
//!   one at a time, at its centre). Who lands on one is derived each tick, as an Airfield's slots are
//!   (`Sim::step_supply_runs`).
//! - **The orders** (`orders::supply_run`): Pick Up Supplies, Drop Off Supplies and Attach to Tower, each a flight to
//!   its target's spot and a landing there; the economy phase picks up, drops off and repairs (`economy::supply_run`).
//! - **Landing and taking off** take the chopper's own times (`UnitDef::landing`, `LandingDef`) each
//!   (`factions/gdo/units/supply_chopper.md > Unit Transport`), as an aircraft's do (`crate::landing`): an air target
//!   while landing and taking off, a ground target while down. Any order but its own on the spot lifts it off
//!   ([`take_off`]).
//! - **The schedule** (`Object::schedule`, `Sim::step_schedules`): a tower's Scheduled Supply Delivery Station
//!   (`factions/gdo/economy.md > SupplyTower > Scheduled Deliveries`).

use glam::Vec2;
use serde::{Deserialize, Serialize};

use crate::defs::{ObjectKind, SupplyChopperDef, SupplyTowerDef};
use crate::economy::stations::is_station;
use crate::hash::{StateHash, StateHasher};
use crate::ids::ObjectId;
use crate::landing::Landing;
use crate::object::{Object, ResourceState};
use crate::orders::OrderKind;

/// What a Supply Chopper carries (`factions/gdo/units/supply_chopper.md > Cargo`): one kind at a time, each a variant
/// of its own, of which Supplies alone are in the game.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum Cargo {
    /// Supplies, at least one: all a station held when the chopper landed on it, unclaimed
    /// (`factions/gdo/units/supply_chopper.md > Supply Pickup`: "it automatically picks up all CurrentSupplies").
    Supplies(u32),
}

/// A Supply Chopper's part in the supply loop (module docs; `factions/gdo/units/supply_chopper.md >
/// SupplyChopperInstanceState`), for a unit whose type has a `UnitDef::supply_chopper`. Saved with its object and
/// hashed with it (`Object::chopper`); each part is left out of files while it is `None`.
#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct ChopperState {
    /// What it carries ("CarriedSupplies"), nothing for none.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub cargo: Option<Cargo>,
    /// The Supply Tower it is attached to ("AttachedTower"; `factions/gdo/economy.md > SupplyTower > Landing and
    /// Attachment`): set by an Attach to Tower, and by the tower that comes with it; broken by any command its owner
    /// gives it (`Sim::apply_command`).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub attached: Option<ObjectId>,
}

impl ChopperState {
    /// The Supplies it carries.
    pub fn supplies(&self) -> u32 {
        match self.cargo {
            Some(Cargo::Supplies(amount)) => amount,
            None => 0,
        }
    }
}

/// The supply loop's numbers of objects of `kind`, if they are Supply Choppers (`UnitDef::supply_chopper`).
pub fn chopper_def(kind: ObjectKind) -> Option<SupplyChopperDef> {
    kind.def().unit.as_ref()?.supply_chopper
}

/// The state a new object of `kind` starts with: an empty, unattached chopper's in the air, for a Supply Chopper
/// (`Sim::new_object`); `None` for every other kind.
pub fn fresh(kind: ObjectKind) -> Option<ChopperState> {
    chopper_def(kind).map(|_| ChopperState::default())
}

/// The supply loop's numbers of objects of `kind`, if they are Supply Towers (`StructureDef::supply_tower`).
pub fn tower_def(kind: ObjectKind) -> Option<SupplyTowerDef> {
    kind.def().structure.as_ref()?.supply_tower
}

/// Whether `o` is a Supply Tower in operation: placed and complete (`Object::is_operational`), where choppers drop off
/// and attach.
pub fn is_tower(o: &Object) -> bool {
    tower_def(o.kind).is_some() && o.placement.is_some() && o.is_operational()
}

/// The spot of an object of `kind` centred on `centre` (module docs): a Supply Tower's, its def's offset from its
/// centre, the tower being symmetric (`SymmetryType::AAAA`), as an Airfield's slots are (`crate::landing::layout`); a
/// station's centre. `None` for any other kind.
pub fn spot_of(kind: ObjectKind, centre: Vec2) -> Option<Vec2> {
    if let Some(def) = tower_def(kind) {
        return Some(centre + Vec2::from(def.spot_offset_su));
    }
    is_station(kind).then_some(centre)
}

/// The spot of object `o` ([`spot_of`]).
pub fn spot(o: &Object) -> Option<Vec2> {
    spot_of(o.kind, o.pos)
}

/// The Supplies `o` carries, if it is a chopper.
pub fn carried(o: &Object) -> u32 {
    o.chopper.as_ref().map_or(0, ChopperState::supplies)
}

/// The tower `o` is attached to, if it is an attached chopper.
pub fn attached_to(o: &Object) -> Option<ObjectId> {
    o.chopper.as_ref()?.attached
}

/// The chopper among `objects` attached to `tower`, the first in their order if there were two (a save edited by hand).
pub fn attached_chopper<'a>(objects: impl IntoIterator<Item = &'a Object>, tower: ObjectId) -> Option<ObjectId> {
    (objects.into_iter())
        .find(|o| attached_to(o) == Some(tower))
        .map(|o| o.id)
}

/// The object whose spot an order of `kind` lands a chopper on, and keeps it down on: a Pick Up Supplies' station, a
/// Drop Off Supplies' or an Attach to Tower's tower. `None` for every other order.
pub fn lands_for(kind: OrderKind) -> Option<ObjectId> {
    match kind {
        OrderKind::PickUpSupplies(at) | OrderKind::DropOffSupplies(at) | OrderKind::AttachToTower(at) => Some(at),
        _ => None,
    }
}

/// The ticks from now until a pick-up on station `station` may take Supplies, of which `claimed` are claimed by the
/// workers picking them up there now (`orders::world::pick_up`; `factions/syndicate/units/agent.md > Supplies`): none
/// while it holds
/// Supplies no one has claimed; its interval while it holds only claimed ones, at least (it refills only once it is
/// empty, `Sim::update_supply_stations`); and while it is empty, the ticks until its next delivery is in it (the
/// countdown the last tick left counts the ticks until the delivery, at the end of the tick before). `None` if
/// `station` is no station.
pub fn due_in(station: &Object, claimed: u32) -> Option<u32> {
    let Some(ResourceState::SupplyStation {
        delivery_interval,
        current_supplies,
        next_delivery_in,
        ..
    }) = station.resource
    else {
        return None;
    };
    Some(match current_supplies {
        held if held > claimed => 0,
        0 => next_delivery_in.unwrap_or(delivery_interval),
        _ => delivery_interval,
    })
}

/// The removal path (`Sim::remove_objects`): `o` forgets the objects for which `is_gone` holds: a chopper's tower,
/// which unattaches it, and a tower's station, which ends its schedule. A chopper's landing on a tower that goes goes
/// with the chopper (`Sim::landed_on`), as an aircraft's on its Airfield.
pub(crate) fn forget(o: &mut Object, is_gone: &impl Fn(ObjectId) -> bool) {
    if let Some(chopper) = &mut o.chopper
        && chopper.attached.is_some_and(is_gone)
    {
        chopper.attached = None;
    }
    if o.schedule.is_some_and(is_gone) {
        o.schedule = None;
    }
}

/// The hash of an object's supply loop state (`Object::chopper`, `Object::schedule`), each under a tag of its
/// own, only while it is set, so that the games without one hash as they did: a chopper's under tag 27, a Supply
/// Tower's Scheduled Supply Delivery Station under tag 28. A chopper's landing (`Object::landing`) hashes in its
/// state's place, after its cargo and its attachment, set or not.
pub(crate) fn hash_object(
    chopper: &Option<ChopperState>,
    landing: Option<Landing>,
    schedule: &Option<ObjectId>,
    h: &mut StateHasher,
) {
    if let Some(chopper) = chopper {
        h.tag(27);
        chopper.hash(h);
        landing.hash(h);
    }
    if let Some(station) = schedule {
        h.tag(28);
        station.hash(h);
    }
}

impl StateHash for Cargo {
    fn hash(&self, h: &mut StateHasher) {
        match self {
            Cargo::Supplies(amount) => {
                h.tag(0);
                amount.hash(h);
            }
        }
    }
}

impl StateHash for ChopperState {
    fn hash(&self, h: &mut StateHasher) {
        let ChopperState { cargo, attached } = self;
        cargo.hash(h);
        attached.hash(h);
    }
}
