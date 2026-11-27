//! I1, no two units of a layer with hard collision overlap, with its exceptions ([`may_overlap`]). (M5-V1) A unit's
//! shape is its Silhouette as it lies (`shape::Shape`): one that turns with its facing is turned, and two shapes
//! overlap by the separating-axis test, strictly, as in the movement step (`docs/movement.md > 3`).

use std::collections::BTreeMap;

use super::{Seen, Violation};
use crate::defs::Domain;
use crate::events::SimEvent;
use crate::ids::ObjectId;
use crate::object::Object;
use crate::orders::gather;
use crate::shape::Shape;
use crate::sim::Sim;

/// The units that came onto the map in this tick overlapping others, as the tick's events tell (I1's exceptions 3
/// and 4, [`may_overlap`]).
pub(super) struct Arrivals<'a> {
    /// The units evacuated (`SimEvent::Evacuated`, design change 0349).
    evacuated: &'a [ObjectId],
    /// (M5-N2) The units a building let out, each with the building (`SimEvent::LeftBuilding`): those let out of one
    /// door together came out overlapping (`ExitRule::AtOnce`, the Garrison's).
    let_out: Vec<(ObjectId, ObjectId)>,
}

impl<'a> Arrivals<'a> {
    /// This tick's, from `sim`'s events and the units evacuated in it.
    pub(super) fn of(sim: &Sim, evacuated: &'a [ObjectId]) -> Self {
        let let_out = (sim.events().iter())
            .filter_map(|e| match *e {
                SimEvent::LeftBuilding { unit, building } => Some((unit, building)),
                _ => None,
            })
            .collect();
        Self { evacuated, let_out }
    }

    /// Whether `a` and `b` came out of one building together in this tick.
    fn together(&self, a: ObjectId, b: ObjectId) -> bool {
        let building =
            |unit: ObjectId| (self.let_out.iter()).find_map(|&(u, building)| (u == unit).then_some(building));
        building(a).is_some_and(|building| self.let_out.contains(&(b, building)))
    }
}

/// I1: no two units of a layer with hard collision overlap. Touching is free.
/// I1's exceptions (design changes 0106 and 0349, slice M5-N2): whether the units `a` and `b` of one layer, whose boxes
/// overlap as the tick ends, may. `before` is what the checker saw of the units a tick earlier, `arrivals` the units
/// that came onto the map in this tick.
///
/// 1. **Gathering Drones** (`factions/colonists/units/drone.md > Gathering Drones pass through each other`):
///    both are Drones on a Gather job. The movement step saw a unit so if it is on one now, or was a tick earlier
///    and has no order now: an order ends after the movement only by leaving the unit idle (a Drop Off whose
///    drop-off was destroyed in phase 5e), while an order given in this tick began before the movement and is the
///    unit's current one. (A job that ended in the behaviours of this tick, by a Stop, for want of a drop-off or
///    with no patch left, design change 0123, reads the same for that one tick: the check can't see the phases.)
/// 2. **Units that already overlap** (`systems/units.md > UnitCollision > Units that already overlap`): the two
///    overlapped a tick earlier too. They ignore each other until they have separated, so the pair may move in
///    any way meanwhile; once apart it is held to I1 again, like every pair that never overlapped.
/// 3. **Evacuated units** (slice M5-F3; `factions/cults/economy.md > Cults Building Mechanics > Evacuation on
///    Destruction`, design change 0349: "evacuated units **all appear at the same time inside its former footprint**
///    [...]. Units that overlap ignore each other until separated"): one of the two came out of a destroyed building
///    in this tick (`SimEvent::Evacuated`), overlapping another that came out with it, or a unit still in the
///    building's doorway. From the next tick on the pair is exception 2's.
/// 4. **Units let out together** (slice M5-N2; `factions/cults/structures/garrison.md > Placeholder stats >
///    Leaving`: "on **Eject All** they all come out **at once**, ignoring collision with each other until they are
///    clear of the footprint"): the two came out of one building's door in this tick (`SimEvent::LeftBuilding`, the
///    Garrison's `ExitRule::AtOnce`), at its one spot. From the next tick on the pair is exception 2's.
///
/// Everything else fails: two units that come to overlap, unless both are gathering Drones. (M3-D2) A portal's
/// energy doorway (design change 0145) takes no exception here: its unit appears at the centre only where its box is
/// clear of every unit and walks out only up to contact, so it overlaps no unit (`tests/production/portal_stalls.rs`
/// has the negative test).
fn may_overlap(a: &Object, b: &Object, before: &BTreeMap<ObjectId, Seen>, arrivals: &Arrivals) -> bool {
    let gathered = |o: &Object| {
        let idle = o.orders.as_ref().is_some_and(|orders| orders.current.is_none());
        gather::on_gather_job(o) || (idle && before.get(&o.id).is_some_and(|seen| seen.gathering))
    };
    let overlapped = || {
        let (Some(was_a), Some(was_b)) = (before.get(&a.id), before.get(&b.id)) else {
            return false;
        };
        let (Some(was_a), Some(was_b)) = (seen_shape(a, was_a), seen_shape(b, was_b)) else {
            return false;
        };
        was_a.overlaps(&was_b)
    };
    let evacuated = arrivals.evacuated;
    (gathered(a) && gathered(b))
        || overlapped()
        || evacuated.contains(&a.id)
        || evacuated.contains(&b.id)
        || arrivals.together(a.id, b.id)
}

/// (M5-V1) The shape of unit `o` where the checker saw it a tick earlier: its Silhouette at the position and facing it
/// had then (`Shape::silhouette`). `None` for an object without one.
fn seen_shape(o: &Object, seen: &Seen) -> Option<Shape> {
    o.def().silhouette()?;
    Some(Shape::silhouette(o.kind, seen.pos, seen.facing))
}

/// The units of `domain`'s layer with their shapes, `shape` of each (`None` leaves it out), each with the low x of its
/// bounding box, by that: a shape only meets those whose box begins before its own ends.
fn by_low_x(sim: &Sim, domain: Domain, shape: impl Fn(&Object) -> Option<Shape>) -> Vec<(f32, Shape, &Object)> {
    let mut boxes: Vec<(f32, Shape, &Object)> = sim
        .objects()
        .filter_map(|o| {
            let in_layer = o.def().unit.as_ref()?.base_def().domain == domain;
            let shape = shape(o).filter(|_| in_layer)?;
            Some((shape.bounds().0.x, shape, o))
        })
        .collect();
    boxes.sort_by(|a, b| a.0.total_cmp(&b.0).then(a.2.id.cmp(&b.2.id)));
    boxes
}

/// The units whose box overlapped another unit's of their layer a tick earlier, by what the checker saw then: those
/// that may be moving off it now, if they are idle (design change 0126). (M5-V1) Their shapes as they lay then.
pub(super) fn overlapped_before(sim: &Sim, before: &BTreeMap<ObjectId, Seen>) -> Vec<ObjectId> {
    let mut found = Vec::new();
    for domain in [Domain::Ground, Domain::Underground] {
        let boxes = by_low_x(sim, domain, |o| seen_shape(o, before.get(&o.id)?));
        for (i, (_, a, a_unit)) in boxes.iter().enumerate() {
            let high = a.bounds().1.x;
            for (low, b, b_unit) in &boxes[i + 1..] {
                if *low >= high {
                    break;
                }
                if a.overlaps(b) {
                    found.extend([a_unit.id, b_unit.id]);
                }
            }
        }
    }
    found.sort();
    found.dedup();
    found
}

/// (M5-N6) The units that a unit evacuated in this tick (`evacuated`, design change 0349) came out on: their shapes as
/// the checker saw them a tick earlier overlap the evacuee's where it was put (its `Object::prev_pos` and
/// `Object::prev_facing`: the spot it came out at, before this tick's movement), in its layer. A unit still in the
/// doorway of the building that went is one (I1's exception 3, [`may_overlap`]): idle, it may be moving off the evacuee
/// now ("Units that overlap ignore each other until separated, and idle ones push apart"), as the evacuee may.
pub(super) fn landed_on(sim: &Sim, before: &BTreeMap<ObjectId, Seen>, evacuated: &[ObjectId]) -> Vec<ObjectId> {
    let layer = |o: &Object| Some(o.def().unit.as_ref()?.base_def().domain);
    let put: Vec<(Domain, Shape)> = (evacuated.iter())
        .filter_map(|&id| {
            let o = sim.object(id)?;
            o.def().silhouette()?;
            Some((layer(o)?, Shape::silhouette(o.kind, o.prev_pos, o.prev_facing)))
        })
        .collect();
    if put.is_empty() {
        return Vec::new();
    }
    (sim.objects())
        .filter(|o| !evacuated.contains(&o.id))
        .filter(|o| {
            let (Some(domain), Some(shape)) = (layer(o), before.get(&o.id).and_then(|seen| seen_shape(o, seen))) else {
                return false;
            };
            domain.hard_collision() && put.iter().any(|(d, e)| *d == domain && e.overlaps(&shape))
        })
        .map(|o| o.id)
        .collect()
}

/// I1: no two units of a layer overlap, but for the exceptions of [`may_overlap`]. (M5-V1) Their shapes
/// (`Shape::of`): a Silhouette that turns with its facing as it lies, by the separating-axis test (`Shape::overlaps`).
pub(super) fn no_overlaps(sim: &Sim, before: &BTreeMap<ObjectId, Seen>, arrivals: &Arrivals) -> Result<(), Violation> {
    for domain in [Domain::Ground, Domain::Underground] {
        let boxes = by_low_x(sim, domain, |o| o.def().silhouette().map(|_| Shape::of(o)));
        for (i, &(_, a, a_unit)) in boxes.iter().enumerate() {
            let high = a.bounds().1.x;
            for &(low, b, b_unit) in &boxes[i + 1..] {
                if low >= high {
                    break;
                }
                if a.overlaps(&b) && !may_overlap(a_unit, b_unit, before, arrivals) {
                    let (a, b) = (a.centre(), b.centre());
                    return Err(Violation {
                        invariant: "I1",
                        message: format!("#{} at {a} and #{} at {b} overlap", a_unit.id.0, b_unit.id.0),
                        at: (a + b) / 2.0,
                    });
                }
            }
        }
    }
    Ok(())
}
