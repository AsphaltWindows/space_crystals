//! (M6-GH2) Landing and rearming at an Airfield — plan decision T88; `factions/gdo/structures/airfield.md > Aircraft
//! Support`, `systems/mechanics/aircraft_landing.md > Vertical` and `systems/mechanics/aircraft_landing.md > Assigned
//! base`; design changes 0428 (answering Q0162) and 0423 (answering Q0157). An aircraft that rearms at an Airfield
//! (`crate::aircraft`) flies home on a Return to Base or a Rebase (`orders::air_base`), lands on one of its Airfield's
//! slots, rearms there, and stays landed until an order lifts it off.
//!
//! - **The slots** ([`slots`]): "An Airfield has **4 landing slots**, one per possible assignment, so all of its
//!   aircraft can be landed at the same time": as many as its capacity (`AirfieldDef::capacity`), at the centres of
//!   its footprint's quarters (`AirfieldDef::slot_offset_su`; **Placeholder (Q0189 point 4)**, the designer's:
//!   `factions/gdo/structures/airfield.md > Aircraft Support`, "a 2x2 grid, one slot near each corner of the
//!   footprint"). Who is on which is derived each tick, never stored ([`claims`]; T88: "A landing aircraft claims the
//!   lowest free pad, derived each tick as E's spot claims are"): the aircraft on the Airfield, landing or down, and
//!   those taking off over it, each keep the free slot nearest it, in id order; then those flying home to it, in id
//!   order, each take the lowest free one.
//! - **The landing** (`Sim::step_landings`, right after phase 5a): an aircraft flying home flies on to its slot (over
//!   the Airfield's centre while none is free), and once it has come to rest over it (T88: "a HoverCraft brakes to a
//!   stop over its pad"; `tuning::air::Air::landing_reach_su`) it lands there, in its Airfield's
//!   `AirfieldDef::landing_ticks` ("Landing and take-off: **16 frames (1 s)** each (placeholder)"), and is down. Its
//!   [`Landing`] is its state (`Object::landing`), from the first frame of its landing to its take-off. It holds
//!   still meanwhile, its move over, and is out of the air units' separation, neither pushed nor pushing
//!   (`movement::air`; `systems/mechanics/aircraft_landing.md > Landed aircraft (every archetype)`: "flying aircraft
//!   don't push landed ones, nor the reverse"), so that it stays on its slot while others fly in beside it or over
//!   it.
//! - **Its domain** (`combat::object_domain`): "**Landed aircraft are ground targets** [...] Air-only weapons
//!   (TargetDomain Air) can't hit it"; "**During landing and take-off** [...] it is an **air target**: anti-air can hit
//!   it until it has fully landed, and again from the moment it starts to take off" (design change 0428 point 1): a
//!   ground target only while it is down ([`down`]), for every reader of its domain (the scans, ValidTarget, the
//!   blasts, the gate's `Reject::CannotHit`, the right-click).
//! - **Rearming** ([`rearm`], in the economy phase): "Rearming an aircraft takes RearmTimePerAmmo × missing ammo"
//!   (`systems/mechanics/aircraft_landing.md > Assigned base`): a round every
//!   `AmmoDef::rearm_ticks_per_ammo` ("**RearmTimePerAmmo 32 frames** (5 missiles in 10 s)",
//!   `factions/gdo/units/hornet.md`), from the frame it comes down, at the speed of its Airfield's owner's Power
//!   (design change 0423: "Each frame, these timers advance by the speed ratio (generated ÷ consumed, capped at 100%),
//!   as production does"; `economy::PowerSummary::speed`), as Scan's cooldown does (`crate::cooldown`), whatever its
//!   order: its flight home is over once it is down (`orders::air_base`). "**After rearming it stays landed**"
//!   (`systems/mechanics/aircraft_landing.md > Assigned base`).
//! - **The take-off** ([`take_off`], as an order starts in phase 5a: `orders::world::Unit::take_off`): any order lifts
//!   it off but its flight home to the Airfield it is on (plan §GH2: "Any order lifts it off, in 16 frames"), and the
//!   order runs: its landing ends, and it rises for as many frames as it landed in (the rise of an aircraft out of an
//!   air door, `Mover::rise`, slice M6-SD's, which holds it still and lets it move in its last frame), an air target
//!   meanwhile. "**An order during rearming**: the aircraft takes off at once **with the ammo rearmed so far**" (design
//!   change 0428 point 4): the round under way is lost. So does an order queued behind its flight home, which starts
//!   in the tick after it is down (`systems/mechanics/aircraft_landing.md > Assigned base`: "**Shift-queued orders run
//!   as soon as it has landed**", "taking off with the ammo rearmed so far"). **Placeholder (not in the design: an
//!   order given while an aircraft lands)**: it rises again from as low as it has come ([`rise_from`]).
//! - **New aircraft** (`Sim::release_onto_slot`, `ExitRule::Pad`): "appear **landed on a free slot**; they take off for
//!   the Airfield's rally point, and with no rally point they stay landed" (design change 0428 point 2): the first
//!   slot that no aircraft takes ([`claims`]), down, its rally point's order lifting it off in its first tick. With
//!   none free it waits inside, finished (`systems/mechanics/aircraft_landing.md > Assigned base`: "No free spot: the
//!   finished aircraft waits inside").
//! - **A destroyed Airfield** (`Sim::remove_objects`): "**Aircraft landed on it are destroyed with it**: they don't
//!   take off by themselves. Designer's reading: an aircraft still **landing** (not yet down) counts as landed and is
//!   destroyed; one already **taking off** escapes" (design change 0428 point 3), unassigned as every other of its
//!   aircraft (`crate::aircraft`).
//! - **On a slot it holds fire** (`aircraft::may_fire`; `systems/mechanics/aircraft_landing.md > Landed aircraft
//!   (every archetype)`: "**Landed aircraft hold fire**: they fire only in the air"): landing or down, it attacks
//!   nothing and threatens no one's scans (`combat::threat`); it fights again once it lifts off.

use std::collections::BTreeMap;

use glam::Vec2;
use serde::{Deserialize, Serialize};

use crate::aircraft::{self, AircraftState};
use crate::defs::{AirfieldDef, LandingDef, ObjectKind};
use crate::economy::{PowerSummary, RATIO_ONE};
use crate::hash::{StateHash, StateHasher};
use crate::ids::{ObjectId, PlayerId};
use crate::object::Object;
use crate::orders::OrderKind;
use crate::progress::Progress;
use crate::shape::Shape;
use crate::supply_run;

/// The slots of the placeholder layout (module docs): an Airfield has as many as its capacity, at most these
/// (`Ruleset::check`).
pub const LANDING_SLOTS: usize = 4;

/// The quarters of the footprint whose centres are the slots, as signs along the two axes, in the order the slots are
/// claimed: north-west, north-east, south-west, south-east (`y` grows southward).
const QUARTERS: [Vec2; LANDING_SLOTS] = [
    Vec2::new(-1.0, -1.0),
    Vec2::new(1.0, -1.0),
    Vec2::new(-1.0, 1.0),
    Vec2::new(1.0, 1.0),
];

/// Where an aircraft lands ([`Landing::at`]; `systems/mechanics/aircraft_landing.md > Vertical`: it lands on a
/// **spot**).
#[derive(Copy, Clone, Debug, PartialEq, Serialize, Deserialize)]
pub enum Spot {
    /// A spot on a structure: one of an Airfield's slots (which one is derived each tick, never stored: [`claims`]),
    /// a Supply Tower's spot or a Supply Delivery Station's centre (`supply_run::spot`).
    Structure(ObjectId),
}

impl Spot {
    /// The structure the spot is on, if it is on one: the structure the aircraft on it is destroyed with
    /// (`Sim::landed_on`; `systems/mechanics/aircraft_landing.md > Landed aircraft (every archetype)`).
    pub fn structure(self) -> Option<ObjectId> {
        match self {
            Spot::Structure(id) => Some(id),
        }
    }
}

/// An aircraft's landing on a spot (module docs; `Object::landing`), from the first frame of its landing to its
/// take-off, which ends it ([`lift_off`]): landing until its descent is done, then down. Saved with its object, and
/// hashed with its aircraft's or its chopper's state, only while it is set (`hash.rs`).
#[derive(Copy, Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct Landing {
    /// Where it lands, or is down: an Airfield's slot, a tower's or a station's spot.
    pub at: Spot,
    /// The frames of its landing done (`AirfieldDef::landing_ticks` on an Airfield's slot, its own
    /// `LandingDef::landing_ticks` elsewhere): it is down once all are ([`down`]).
    pub descent: Progress,
    /// The round it rearms, while it is down and short of ammo: its frames done of its type's
    /// `AmmoDef::rearm_ticks_per_ammo`, at its Airfield's speed ([`rearm`]), from the frame it comes down. Left out of
    /// files while `None`.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub rearm: Option<Progress>,
}

/// `o`'s landing, if it is an aircraft landing on an Airfield or down on one, or a Supply Chopper landing on
/// a Supply Tower's or a Supply Delivery Station's spot or down on it (`crate::supply_run`): its landing and its
/// domain follow the same rules (`factions/gdo/units/supply_chopper.md > Unit Transport`: "as at the Airfield").
pub fn of(o: &Object) -> Option<Landing> {
    o.landing
}

/// The Airfield `o` lands on or is down on, if any, or the tower or the station a chopper lands on.
pub fn at(o: &Object) -> Option<ObjectId> {
    of(o).and_then(|landing| landing.at.structure())
}

/// Whether `o` is down on an Airfield's slot, or a chopper on its spot: its landing's frames all done. Only
/// then is it a ground target (module docs, `combat::object_domain`).
#[inline]
pub fn down(o: &Object) -> bool {
    of(o).is_some_and(|landing| landing.descent.done())
}

/// How far up `o` is while it is on a slot, from 1, in the air as its landing begins, to 0, down, by the whole frames
/// of its landing done. `None` for an aircraft that isn't on a slot, in the air as any air unit (its take-off is a
/// rise: `network::door_lift`). The client draws its height by it (`object_render::flight`).
pub fn lift(o: &Object) -> Option<f32> {
    of(o).map(|landing| 1.0 - landing.descent.ratio())
}

/// The landing of an aircraft down on Airfield `at` already, whose landing takes `ticks` frames: a new aircraft's,
/// which appears down on its slot (module docs).
pub fn down_on(at: ObjectId, ticks: u32) -> Landing {
    Landing {
        at: Spot::Structure(at),
        descent: Progress {
            elapsed: ticks.saturating_mul(RATIO_ONE),
            total: ticks,
        },
        rearm: None,
    }
}

/// The rise of an aircraft that takes off in the middle of `descent`, its landing, in a take-off of `takeoff_ticks`
/// frames (module docs): as high as it still is, its share of the take-off left as its share done of `descent`'s
/// total (rounded up), so that it goes on up from there; as many frames as it has come down, where the two times are
/// the same. A whole landing's is a whole take-off; none for one that hasn't come down a frame yet, in the air still.
pub fn rise_from(descent: Progress, takeoff_ticks: u32) -> Option<Progress> {
    let done = u64::from(descent.ticks_done());
    let left = (done * u64::from(takeoff_ticks)).div_ceil(u64::from(descent.total.max(1)));
    let left = u32::try_from(left).map_or(takeoff_ticks, |left| left.min(takeoff_ticks));
    (left > 0).then(|| Progress {
        elapsed: (takeoff_ticks - left).saturating_mul(RATIO_ONE),
        total: takeoff_ticks,
    })
}

/// The frames of the take-off of an aircraft of `kind` from `landing` ([`rise_from`]): its own take-off time
/// (`LandingDef::takeoff_ticks`), or, for one with none, which lands where its base gives the times (an Airfield's
/// slots: "Landing and take-off: 16 frames (1 s) each", `AirfieldDef::landing_ticks`), as many as its landing's.
fn takeoff_ticks(kind: ObjectKind, landing: Landing) -> u32 {
    def_of(kind).map_or(landing.descent.total, |def| def.takeoff_ticks)
}

/// The landing and take-off times of `kind`'s type, if they are its own (`UnitDef::landing`): the Supply Chopper's.
pub fn def_of(kind: ObjectKind) -> Option<LandingDef> {
    kind.def().unit.as_ref()?.landing
}

/// The slots of Airfield `airfield` (module docs), in the order they are claimed: as many as its capacity, the
/// placeholder layout's at most. None for an object that is no Airfield.
pub fn slots(airfield: &Object) -> impl Iterator<Item = Vec2> + use<> {
    layout(airfield.pos, aircraft::airfield_def(airfield))
}

/// The slots of an Airfield centred on `centre` whose support is `def` ([`slots`]): the client draws a type's pads by
/// them (`object_render::pads`).
pub fn layout(centre: Vec2, def: Option<AirfieldDef>) -> impl Iterator<Item = Vec2> {
    let count = def.map_or(0, |def| (def.capacity as usize).min(LANDING_SLOTS));
    let offset = def.map_or(0.0, |def| def.slot_offset_su);
    QUARTERS
        .into_iter()
        .take(count)
        .map(move |quarter| centre + quarter * offset)
}

/// Whether `o` flies home: its current order is a Return to Base or a Rebase (`orders::air_base`), the flight to its
/// base that ends in a landing.
pub fn flies_home(o: &Object) -> bool {
    (o.orders.as_ref().and_then(|orders| orders.current.as_ref()))
        .is_some_and(|active| matches!(active.order.kind, OrderKind::Rebase(_) | OrderKind::ReturnToBase))
}

/// An aircraft that takes, or will take, a slot of an Airfield ([`users`]), as the tick has it so far.
#[derive(Copy, Clone, Debug, PartialEq)]
pub(crate) struct User {
    pub id: ObjectId,
    pub pos: Vec2,
    /// The Airfield it lands on or is down on.
    pub on: Option<ObjectId>,
    /// It takes off (`Mover::rise`): over the Airfield it takes off from, if it rises in one's footprint.
    pub rising: bool,
    /// The Airfield it flies home to ([`flies_home`]): its base.
    pub home: Option<ObjectId>,
}

/// The aircraft among `objects` that take a slot or will ([`User`]), in their order (id order for a map's range): those
/// on a slot, those taking off, and those flying home.
pub(crate) fn users<'a>(objects: impl IntoIterator<Item = &'a Object>) -> Vec<User> {
    (objects.into_iter())
        .filter_map(|o| {
            let state = o.aircraft?;
            let on = at(o);
            let rising = o.mover.as_ref().is_some_and(|mover| mover.rise.is_some());
            let home = state.base.filter(|_| flies_home(o));
            (on.is_some() || rising || home.is_some()).then_some(User {
                id: o.id,
                pos: o.pos,
                on,
                rising,
                home,
            })
        })
        .collect()
}

/// Whether `pos` lies in `airfield`'s footprint, its edges included.
fn over(airfield: &Object, pos: Vec2) -> bool {
    let (lo, hi) = Shape::of(airfield).bounds();
    pos.cmpge(lo).all() && pos.cmple(hi).all()
}

/// Who takes each slot of `airfield` (module docs), slot by slot in [`slots`]' order, of `users` in id order: first
/// the aircraft on it, landing or down, and those taking off over its footprint, each the free slot nearest it (the
/// first of two as near); then those flying home to it that hold none, each the first free one. A slot past its
/// count is never taken.
pub(crate) fn claims(airfield: &Object, users: &[User]) -> [Option<ObjectId>; LANDING_SLOTS] {
    let points: Vec<Vec2> = slots(airfield).collect();
    let mut taken = [None; LANDING_SLOTS];
    for user in users {
        let holds = user.on == Some(airfield.id) || (user.on.is_none() && user.rising && over(airfield, user.pos));
        if !holds {
            continue;
        }
        let nearest = (0..points.len()).filter(|&i| taken[i].is_none()).min_by(|&a, &b| {
            let (da, db) = (
                points[a].distance_squared(user.pos),
                points[b].distance_squared(user.pos),
            );
            da.total_cmp(&db).then(a.cmp(&b))
        });
        if let Some(i) = nearest {
            taken[i] = Some(user.id);
        }
    }
    for user in users
        .iter()
        .filter(|user| user.home == Some(airfield.id) && user.on.is_none())
    {
        if taken.contains(&Some(user.id)) {
            continue;
        }
        if let Some(i) = (0..points.len()).find(|&i| taken[i].is_none()) {
            taken[i] = Some(user.id);
        }
    }
    taken
}

/// The economy phase (module docs): every aircraft down on an Airfield and short of ammo, idle there or with any order
/// that leaves it down, rearms one frame on, at the speed its Airfield's owner's Power in `power` gives the Airfield
/// (`economy::power::summaries`, `PowerSummary::speed`), its first round begun in the frame it comes down; a round
/// whose frames are done is in its ammo, what the frame had left over going on to the next round, until it is full. A
/// full one rearms nothing.
pub(crate) fn rearm(objects: &mut BTreeMap<ObjectId, Object>, power: &BTreeMap<PlayerId, PowerSummary>) {
    let speed = |airfield: &Object| {
        airfield.owner.map_or(RATIO_ONE, |owner| {
            power.get(&owner).copied().unwrap_or_default().speed(airfield.kind)
        })
    };
    let rearming: Vec<(ObjectId, u32)> = (objects.values())
        .filter(|o| down(o))
        .filter_map(|o| Some((o.id, objects.get(&at(o)?).map_or(RATIO_ONE, speed))))
        .collect();
    for (id, speed) in rearming {
        let Some(o) = objects.get_mut(&id) else {
            continue;
        };
        let Some(def) = aircraft::def_of(o.kind) else {
            continue;
        };
        let (Some(AircraftState { ammo, .. }), Some(landing)) = (o.aircraft.as_mut(), o.landing.as_mut()) else {
            continue;
        };
        if *ammo >= def.max {
            landing.rearm = None;
            continue;
        }
        let round = (landing.rearm).get_or_insert_with(|| Progress::new(def.rearm_ticks_per_ammo));
        if !round.step(speed) {
            continue;
        }
        *ammo += 1;
        round.elapsed = round.elapsed.saturating_sub(round.total.saturating_mul(RATIO_ONE));
        if *ammo >= def.max {
            landing.rearm = None;
        }
    }
}

/// An order of `kind` starts for a unit of type `unit` assigned to `base`, whose landing is `landing` and whose mover's
/// rise is `rise` (`orders::world::Unit::take_off`, module docs): if it is on a spot and the order doesn't land it on
/// that spot's structure ([`lands_on`]: an aircraft's flight home to the Airfield it is on, a chopper's order for the
/// tower or the station it is on), it takes off ([`lift_off`]). Returns whether it took off.
pub(crate) fn take_off(
    kind: OrderKind,
    unit: ObjectKind,
    base: Option<ObjectId>,
    landing: &mut Option<Landing>,
    rise: &mut Option<Progress>,
) -> bool {
    let Some(on) = *landing else {
        return false;
    };
    if on.at.structure().is_some_and(|at| lands_on(kind, base) == Some(at)) {
        return false;
    }
    lift_off(unit, landing, rise);
    true
}

/// The structure an order of `kind` lands its unit on, for a unit assigned to `base`: a Rebase's Airfield, a Return
/// to Base's base (`orders::air_base`), and a Supply Chopper's Pick Up, Drop Off or Attach to Tower's object
/// (`supply_run::lands_for`). `None` for every other order.
pub fn lands_on(kind: OrderKind, base: Option<ObjectId>) -> Option<ObjectId> {
    match kind {
        OrderKind::Rebase(airfield) => Some(airfield),
        OrderKind::ReturnToBase => base,
        _ => supply_run::lands_for(kind),
    }
}

/// An aircraft of `kind` whose landing is `landing` and whose mover's rise is `rise` takes off, if it is on a spot:
/// by an order ([`take_off`]), a chopper after a pick-up or a drop-off (`economy::supply_run`) or evicted
/// (`Sim::step_supply_runs`). Its landing ends, the round it rearms with it, and it rises from as low as it has come
/// ([`rise_from`]), in its take-off's frames.
pub(crate) fn lift_off(kind: ObjectKind, landing: &mut Option<Landing>, rise: &mut Option<Progress>) {
    if let Some(on) = landing.take() {
        *rise = rise_from(on.descent, takeoff_ticks(kind, on));
    }
}

impl StateHash for Spot {
    fn hash(&self, h: &mut StateHasher) {
        // A spot on a structure hashes as the structure's id alone.
        match self {
            Spot::Structure(id) => id.hash(h),
        }
    }
}

impl StateHash for Landing {
    fn hash(&self, h: &mut StateHasher) {
        let Landing { at, descent, rearm } = self;
        at.hash(h);
        descent.hash(h);
        rearm.hash(h);
    }
}

#[cfg(test)]
mod tests;
