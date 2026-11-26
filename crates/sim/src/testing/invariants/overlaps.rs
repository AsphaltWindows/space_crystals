//! I1, no two units of a layer with hard collision overlap, with its exceptions ([`may_overlap`]).

use std::collections::BTreeMap;

use glam::Vec2;

use super::{Seen, Violation};
use crate::defs::Domain;
use crate::events::SimEvent;
use crate::ids::ObjectId;
use crate::object::Object;
use crate::orders::gather;
use crate::paths::boxes_overlap;
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
        let (Some(a_half), Some(b_half)) = (a.def().half_extents(), b.def().half_extents()) else {
            return false;
        };
        boxes_overlap(was_a.pos, a_half, was_b.pos, b_half)
    };
    let evacuated = arrivals.evacuated;
    (gathered(a) && gathered(b))
        || overlapped()
        || evacuated.contains(&a.id)
        || evacuated.contains(&b.id)
        || arrivals.together(a.id, b.id)
}

/// The units whose box overlapped another unit's of their layer a tick earlier, by what the checker saw then: those
/// that may be moving off it now, if they are idle (design change 0126).
pub(super) fn overlapped_before(sim: &Sim, before: &BTreeMap<ObjectId, Seen>) -> Vec<ObjectId> {
    let mut found = Vec::new();
    for domain in [Domain::Ground, Domain::Underground] {
        let mut boxes: Vec<(f32, Vec2, Vec2, ObjectId)> = sim
            .objects()
            .filter_map(|o| {
                let def = o.def();
                let half = def.half_extents()?;
                let pos = before.get(&o.id)?.pos;
                (def.unit.as_ref()?.base_def().domain == domain).then_some((pos.x - half.x, pos, half, o.id))
            })
            .collect();
        boxes.sort_by(|a, b| a.0.total_cmp(&b.0).then(a.3.cmp(&b.3)));
        for (i, &(_, a, a_half, a_id)) in boxes.iter().enumerate() {
            for &(low, b, b_half, b_id) in &boxes[i + 1..] {
                if low >= a.x + a_half.x {
                    break;
                }
                if boxes_overlap(a, a_half, b, b_half) {
                    found.extend([a_id, b_id]);
                }
            }
        }
    }
    found.sort();
    found.dedup();
    found
}

/// I1: no two units of a layer overlap, but for the exceptions of [`may_overlap`].
pub(super) fn no_overlaps(sim: &Sim, before: &BTreeMap<ObjectId, Seen>, arrivals: &Arrivals) -> Result<(), Violation> {
    for domain in [Domain::Ground, Domain::Underground] {
        // (low x, position, half extents, the unit), by low x: a unit only meets those starting before its high x.
        let mut boxes: Vec<(f32, Vec2, Vec2, &Object)> = sim
            .objects()
            .filter_map(|o| {
                let def = o.def();
                let half = def.half_extents()?;
                (def.unit.as_ref()?.base_def().domain == domain).then_some((o.pos.x - half.x, o.pos, half, o))
            })
            .collect();
        boxes.sort_by(|a, b| a.0.total_cmp(&b.0).then(a.3.id.cmp(&b.3.id)));
        for (i, &(_, a, a_half, a_unit)) in boxes.iter().enumerate() {
            for &(low, b, b_half, b_unit) in &boxes[i + 1..] {
                if low >= a.x + a_half.x {
                    break;
                }
                if boxes_overlap(a, a_half, b, b_half) && !may_overlap(a_unit, b_unit, before, arrivals) {
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
