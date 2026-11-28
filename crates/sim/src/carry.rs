//! Carriers — plan decision T94, slice M6-ST1: one capability for every unit that carries units, the Syndicate's
//! Subterranean Transport first (`factions/syndicate/units/subterranean_transport.md`). A type has it by its def
//! (`UnitDef::carries`, [`CarrierDef`]): its places, and the bases it takes with the places a unit of each takes.
//!
//! - **Aboard** (plan decision T30, `crate::network`): a passenger is held off the map at `network::Place::Carrier`,
//!   as a unit inside a building is: no object of the map, with its id, its HP and its load, its orders cleared. It
//!   goes wherever its carrier goes, on the map or into its owner's Tunnel Network
//!   (`factions/syndicate/units/subterranean_transport.md > Entering the Tunnel Network loaded`: "they travel along
//!   inside it and stay aboard"), and nothing else lets it out: "they only come out when the transport unloads (on the
//!   surface), and can't be ejected from a Tunnel on their own" (`Sim::network` leaves them out, so the EjectMenu lists
//!   the carrier alone). "**They always keep counting against the Tunnel Space cap**, aboard or not, inside the network
//!   or on the surface", as every held unit does (`Sim::army_cap`).
//! - **Not in the network themselves** (design change 0432 point 2, answering Q0166: "only the Transport regenerates
//!   and takes a Null Chamber turn; its passengers don't regenerate and come out **unbuffed** when it unloads"): a
//!   passenger regenerates nothing (`Held::regenerate`) and takes no Null Chamber turn (`network::services`, which
//!   serves the network's units), while its carrier, in the network, does both.
//! - **Loading** ([`loads`]; `factions/syndicate/units/subterranean_transport.md > Above-ground mode`: "In this mode,
//!   **infantry can be loaded** into it, and it can **unload** them"): a carrier takes units in, and lets them out,
//!   while it is a ground object (`combat::object_domain`): the Transport's above-ground type, a Ground unit by its own
//!   Domain (`UnitDef::domain`), not its burrowed one.
//! - **Boarding** ([`admits`]; `factions/syndicate/units/subterranean_transport.md > Placeholder stats`: "**Loading**:
//!   right-click the transport with infantry selected (Enter); it must be above ground"): the Enter order on a carrier
//!   (`orders::enter`), by the one rule of every way in (`network::admits`). Its owner's units of a base it takes
//!   ([`places`]: the Transport "carries **infantry only**") walk up to it, after it as it drives, and board as their
//!   shape comes within Proximity of its, edge to edge (`Orders::proximity_su`), from any side, if it has room for them
//!   (`Sim::room`: its places less those its passengers take; "**Capacity 4** infantry"). With no room, the Enter ends
//!   and the unit idles beside it, as at a full building (design change 0347 point 2): so a fifth Guard stays out. Only
//!   the owner's units board: there are no allies yet (the plan's D45). **Placeholder (not in the design: from which
//!   side a unit boards, and how near it must be)**: any side, within Proximity.
//! - **Unload All** ([`ring_spots`]; `factions/syndicate/units/subterranean_transport.md > Placeholder stats`:
//!   "**Unloading**: **F: Unload All** (above ground only; they come out around it)"): an order, untargeted and
//!   queueable, as the Drone Operator's Launch is (plan decision T91): the carrier stops, and once it is at rest a step
//!   of the economy phase puts every passenger back on the map round it, longest aboard first, and ends the order
//!   (`Sim::step_unloads`). Each comes out at the first free spot of the rings round the carrier's shape as it lies
//!   ([`ring_spots`]), its walk round the nearest ring starting from its share of it, so that they spread round the
//!   carrier: a spot is free where the unit, facing as its carrier does, fits by the rule of every unit put on the map
//!   (`Sim::check_unit_fits`: clear of blocked tiles, of footprints and of the units of its layer, those that came out
//!   before it included), so none overlaps another. Idle, as an evacuated unit is
//!   (`factions/cults/economy.md > Cults Building Mechanics > Evacuation on Destruction`: "Afterwards evacuated units
//!   **idle**"), with a `SimEvent::Unloaded`. `systems/mechanics/carrying.md > Unloading`: "**On the move, it stops
//!   first**"; "Passengers come out on the **nearest free spots** around the carrier"; "**Crowded**: passengers with no
//!   free spot **stay aboard**": the rings `Holding::unload_gap_su` apart, `Holding::unload_rings` of them, the
//!   nearest first, and a passenger for whom none of them has a spot stays aboard, for the next Unload All.
//! - **Death** (`factions/syndicate/units/subterranean_transport.md > Destroyed with passengers`: "If it is destroyed,
//!   **all infantry inside die** with it (above ground or underground)"): the removal path takes a carrier's passengers
//!   along, those of a passenger that carries too, each with a `SimEvent::Died` where the carrier died
//!   (`Sim::remove_objects`), or with the carrier held in the network as it leaves the game (`Sim::remove_held`:
//!   stranded, or its owner defeated); their Tunnel Space is free.
//! - **No firing out** (`factions/syndicate/units/subterranean_transport.md > AttackAttributes`, design change 0432
//!   point 1: "**Passengers don't fire out**: they are carried, not fighting"): only a building's units fire from
//!   inside it (`combat::occupants`).
//!
//! **One carrier for every transport** (the plan's §B: M6-GS2's Supply Chopper, design change 0442, and M6-CP's
//! Pilgrim, design change 0462, reuse it). A base's places are its own ([`CarrierDef::places`]): the Chopper's "4
//! infantry (one place each, Light or Heavy) or 1 vehicle of any kind [...]; one kind at a time" is 4 places, an
//! infantry unit taking 1 and a vehicle 4; the Pilgrim's 16 infantry are 16 places. A carrier loads while it is a
//! ground object, by [`loads`]: an aircraft landed on the ground is one (`combat::object_domain`'s landed arm), so the
//! Pilgrim's "Loading: only while landed" needs no rule of its own, and nor does a flying carrier's hold, which goes
//! wherever its carrier is.

use glam::Vec2;

use crate::combat;
use crate::defs::{CarrierDef, Domain, ObjectKind};
use crate::math;
use crate::network::NoTransit;
use crate::object::Object;
use crate::ruleset::rules;

/// The hold of a `kind` unit's type ([`CarrierDef`]): `None` for anything but a carrier.
pub fn def_of_kind(kind: ObjectKind) -> Option<&'static CarrierDef> {
    kind.def().unit.as_ref()?.carries.as_ref()
}

/// The hold of `o`'s type ([`def_of_kind`]): `None` for anything but a carrier.
pub fn def_of(o: &Object) -> Option<&'static CarrierDef> {
    def_of_kind(o.kind)
}

/// Whether carrier `o` takes units in and lets them out now (module docs): it is a carrier, a ground object
/// (`combat::object_domain`: the Transport above ground, by its type's Domain, `UnitDef::domain`). `false` for anything
/// else.
pub fn loads(o: &Object) -> bool {
    def_of(o).is_some() && combat::object_domain(o) == Domain::Ground
}

/// The places a `kind` unit takes aboard a carrier whose hold is `def` (`CarrierDef::places_of`, by the unit's base),
/// if it boards at all: `None` for a unit of a base the hold doesn't list, and for anything but a unit.
pub fn places(def: &CarrierDef, kind: ObjectKind) -> Option<u32> {
    def.places_of(kind.def().unit.as_ref()?.base)
}

/// Whether a hold `def` takes a `kind` unit at all: of a base it lists, with no more places than it has ([`places`]).
pub fn takes(def: &CarrierDef, kind: ObjectKind) -> bool {
    places(def, kind).is_some_and(|places| places <= def.capacity)
}

/// Whether `carrier` takes a `kind` unit in (module docs; the carrier arm of `network::admits`): it is a carrier, with
/// an owner, that loads now ([`loads`]), else `NoTransit::NotATunnel`; the kind is a unit, else `NoTransit::NotAUnit`,
/// that its hold takes ([`takes`]), else `NoTransit::NotAccepted`. Whose units may board, and whether the carrier has
/// room, are the sim's (`Sim::board`).
pub fn admits(carrier: &Object, kind: ObjectKind) -> Result<(), NoTransit> {
    let def = def_of(carrier).ok_or(NoTransit::NotATunnel)?;
    if carrier.owner.is_none() || !loads(carrier) {
        return Err(NoTransit::NotATunnel);
    }
    kind.def().unit.as_ref().ok_or(NoTransit::NotAUnit)?;
    if !takes(def, kind) {
        return Err(NoTransit::NotAccepted);
    }
    Ok(())
}

/// The spots of ring `ring` (0 the nearest) round `carrier`'s shape as it lies, for passengers that each fit in a
/// circle of radius `radius` round their centre (module docs, *Unload All*): a rectangle in the carrier's frame,
/// `Holding::unload_gap_su` clear of its shape and of the ring inside it, with spots on its corners and spread evenly
/// along its sides, at least two radii and that gap apart, so that no two passengers on one ring, nor on two, overlap.
/// In their order round it, from the middle of the carrier's back (of two spots nearest it, the first) along one side
/// to its front and back along the other. Empty for anything but a unit, or a radius and a gap of nothing.
pub fn ring_spots(carrier: &Object, radius: f32, ring: u32) -> Vec<Vec2> {
    let Some(half) = carrier.def().half_extents() else {
        return Vec::new();
    };
    let gap = rules().tuning.holding.unload_gap_su.max(0.0);
    let pitch = 2.0 * radius + gap;
    if pitch <= 0.0 {
        return Vec::new();
    }
    // The rectangle's half extents in the carrier's frame: x along its facing, y across it.
    let out = radius + gap + ring as f32 * pitch;
    let (rx, ry) = (half.x + out, half.y + out);
    let corners = [
        Vec2::new(-rx, -ry),
        Vec2::new(-rx, ry),
        Vec2::new(rx, ry),
        Vec2::new(rx, -ry),
    ];
    let mut local: Vec<Vec2> = Vec::new();
    let mut back = 0;
    for (side, &from) in corners.iter().enumerate() {
        let to = corners[(side + 1) % corners.len()];
        // Along one axis of the frame. Every side is longer than the pitch (it spans at least two radii and two
        // gaps): one step at least.
        let length = (to - from).abs().max_element();
        let steps = ((length / pitch) as u32).max(1);
        if side == 0 {
            back = (steps / 2) as usize;
        }
        local.extend((0..steps).map(|step| from + (to - from) * (step as f32 / steps as f32)));
    }
    local.rotate_left(back);
    let ahead = math::facing_vector(carrier.facing);
    let across = Vec2::new(-ahead.y, ahead.x);
    local
        .into_iter()
        .map(|p| carrier.pos + ahead * p.x + across * p.y)
        .collect()
}

#[cfg(test)]
mod tests;
