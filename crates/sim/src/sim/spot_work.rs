//! The spots' step (`crate::spot_work`, plan decision T62, slice M5-E): right after the arrivals' step
//! (`Sim::step_deployments`), each unit that works at a footprint from a spot beside it gets its own spot, and walks
//! there; a unit that ended its walk out of reach looks again now and then. Before 5b, so that the paths of this tick
//! serve the walks it begins (`ARCHITECTURE.md > Tick pipeline`).

use glam::Vec2;

use super::Sim;
use super::builds::Dropped;
use crate::events::SimEvent;
use crate::ids::ObjectId;
use crate::mover::{MoveGoal, MoveStatus};
use crate::object::Placement;
use crate::orders::OrderKind;
use crate::placement::{PlacementCheck, Spots};
use crate::shape::Shape;
use crate::spot_work::{self, Claim, Work, WorkAt, suits, within_reach, work_of};
use crate::tuning::construction::{BUILD_STANDOFF_SU, RESPOT_TICKS};
use crate::units::SU_PER_GU;

impl Sim {
    /// The spots' step (module docs): for each unit from before this tick (`first_new`) whose current order works at a
    /// footprint from a spot beside it (`spot_work::work_of`), in id order:
    /// - an order on a structure that no longer suits it ends (`spot_work::suits`);
    /// - the units without a spot, nearest the footprint first (edge to edge, then by id), each get the first one of
    ///   [`Sim::free_spot`] that no other unit of its owner's working at the same footprint claims, of ring 0, or else
    ///   of every ring on the map from the second on (ring 1 left free, the way round the builders), and walk there;
    ///   with none on the whole map, the order ends ([`Sim::no_spot`]);
    /// - in the ticks that are multiples of `tuning::construction::RESPOT_TICKS`, a unit whose walk is over takes the
    ///   place where it stands as its spot if it is within reach of the footprint, so that a unit that stopped short
    ///   of its spot doesn't keep another from the spot it stands on; out of reach, it looks for a free spot of ring 0,
    ///   its own claim aside, and walks there if it finds one.
    pub(super) fn step_spots(&mut self, first_new: ObjectId) {
        let workers: Vec<(ObjectId, Work)> = (self.objects.range(..first_new))
            .filter_map(|(&id, o)| Some((id, work_of(o)?)))
            .collect();
        if workers.is_empty() {
            return;
        }
        let mut placed = Vec::with_capacity(workers.len());
        for (id, work) in workers {
            match self.work_place(id, work) {
                Some(at) => placed.push((id, work, at)),
                None => self.end_work(id),
            }
        }
        let mut claims: Vec<Claim> = (placed.iter())
            .filter_map(|&(id, work, at)| self.claim(id, at, work.spot?))
            .collect();
        // The units without a spot choose theirs nearest the footprint first, so that those in front take the spots
        // in front, and those behind walk on to the others.
        let mut spotless: Vec<(f32, ObjectId, Work, Placement)> = (placed.iter())
            .filter(|(_, work, _)| work.spot.is_none())
            .filter_map(|&(id, work, at)| {
                let gap = Shape::of(self.objects.get(&id)?).gap(&Shape::footprint(&at));
                Some((gap, id, work, at))
            })
            .collect();
        spotless.sort_by(|a, b| a.0.total_cmp(&b.0).then(a.1.cmp(&b.1)));
        for (_, id, work, at) in spotless {
            let Some(kind) = self.objects.get(&id).map(|o| o.kind) else {
                continue;
            };
            // Ring 0 within reach, or a ring from the second on to wait on: ring 1 stays free, a way round the
            // builders for those walking on to the far sides.
            let rings = std::iter::once(0).chain(2..=*self.rings_on_map(kind, &at).end());
            match self.free_spot(id, &at, rings, &claims) {
                Some(spot) => {
                    self.send_to(id, spot);
                    claims.extend(self.claim(id, at, spot));
                }
                None => self.no_spot(id, work.order),
            }
        }
        if !self.tick.is_multiple_of(RESPOT_TICKS) {
            return;
        }
        // The units whose walk is over, by whether they stand within reach.
        let mut astray = Vec::new();
        for &(id, _, at) in placed.iter().filter(|(_, work, _)| work.spot.is_some()) {
            let Some(o) = self.objects.get_mut(&id) else {
                continue;
            };
            let over = o.mover.as_ref().is_some_and(|mover| mover.status != MoveStatus::Moving);
            if !over {
                continue;
            }
            if !within_reach(&Shape::of(o), &at) {
                astray.push((id, at));
                continue;
            }
            let pos = o.pos;
            if let Some(slot) = spot_work::spot_mut(o).filter(|slot| **slot != Some(pos)) {
                *slot = Some(pos);
                if let Some(claim) = claims.iter_mut().find(|claim| claim.unit == id) {
                    claim.spot = pos;
                }
            }
        }
        for (id, at) in astray {
            if let Some(spot) = self.free_spot(id, &at, 0..=0, &claims) {
                self.send_to(id, spot);
                if let Some(claim) = claims.iter_mut().find(|claim| claim.unit == id) {
                    claim.spot = spot;
                }
            }
        }
    }

    /// The footprint unit `id` works at by `work`: a structure's while the order suits it (`spot_work::suits`).
    fn work_place(&self, id: ObjectId, work: Work) -> Option<Placement> {
        match work.at {
            WorkAt::Footprint(at) => Some(at),
            WorkAt::Object(target) => {
                let owner = self.objects.get(&id)?.owner;
                (self.objects.get(&target))
                    .filter(|target| suits(work.order, owner, target))
                    .and_then(|target| target.placement)
            }
        }
    }

    /// The claim of unit `id` on `spot` beside the footprint `at`.
    fn claim(&self, id: ObjectId, at: Placement, spot: Vec2) -> Option<Claim> {
        let o = self.objects.get(&id)?;
        Some(Claim {
            unit: id,
            owner: o.owner,
            at,
            spot,
            half: o.def().half_extents()?,
        })
    }

    /// The first spot of `rings` around the footprint `at` where unit `id` fits now, that no other unit's claim among
    /// `claims` keeps it from: T62's spots (`placement::Spots`) for a box `tuning::construction::BUILD_STANDOFF_SU`
    /// bigger on each side than the unit's, ring after ring, of each ring those beside the footprint's sides (none
    /// round its corners), nearest the unit first and, of two as near, the one numbered lower. A spot fits by the rule
    /// of every unit put on the map (`Sim::check_unit_fits`), the unit itself aside: on the map, its box clear of
    /// blocked tiles, of blocking footprints and of the units of its layer.
    ///
    /// **The order** is the one exception to `Sim::spot_around`'s (`Round::BothWays` from the side the unit comes
    /// from), which suits a unit put on the map but not a group walking up from one side: its two ways alternate, so
    /// that half of the group crossed in front of the footprint to the other side, through the others' spots (slice
    /// M5-E's run of eight Recruits on a Storage: one never got within reach). Nearest first, each takes a spot on its
    /// own side while there is one.
    fn free_spot(
        &self,
        id: ObjectId,
        at: &Placement,
        rings: impl IntoIterator<Item = u32>,
        claims: &[Claim],
    ) -> Option<Vec2> {
        let o = self.objects.get(&id)?;
        let (kind, owner, pos, half) = (o.kind, o.owner, o.pos, o.def().half_extents()?);
        let spots = Spots::around(at, half + Vec2::splat(BUILD_STANDOFF_SU));
        let footprint = Vec2::new(at.w as f32, at.h as f32) * (SU_PER_GU / 2.0);
        let beside = |spot: Vec2| ((spot - at.center()).abs() - footprint).min_element() <= 0.0;
        rings.into_iter().find_map(|ring| {
            let mut ring: Vec<Vec2> = (spots.ring(ring).spots.into_iter())
                .map(|(spot, _)| spot)
                .filter(|&spot| beside(spot))
                .collect();
            ring.sort_by(|a, b| a.distance_squared(pos).total_cmp(&b.distance_squared(pos)));
            ring.into_iter().find(|&spot| {
                self.map.contains_point(spot)
                    && !claims.iter().any(|claim| claim.keeps(id, owner, at, spot, half))
                    && self.check_unit_fits(kind, spot, Some(id)).is_ok()
            })
        })
    }

    /// Unit `id` takes `spot` for its work, and walks there. The walk presses on (`Mover::press_on`): **Placeholder
    /// (not in the design: a builder's walk to its spot)**, as design change 0315's walk to a hatch (`factions/
    /// syndicate/units/agent.md > Gathering > Gathering rules`, "Waiting at the hatch"), since a unit short of its spot
    /// builds nothing: blocked near it by units that won't move, the builders already at theirs, it doesn't count as
    /// arrived (`systems/controls/behaviors.md > MovingToLocation > Occupied target location`) but steers round them,
    /// looks for other routes, and ends its walk when none is left, as any stuck unit does.
    fn send_to(&mut self, id: ObjectId, spot: Vec2) {
        let tick = self.tick;
        let Some(o) = self.objects.get_mut(&id) else {
            return;
        };
        if let Some(slot) = spot_work::spot_mut(o) {
            *slot = Some(spot);
        }
        if let Some(mover) = &mut o.mover {
            mover.go_to(MoveGoal::Point(spot), None, tick);
            mover.press_on();
        }
    }

    /// Unit `id`'s current order, `order`, ends: it found no spot on the whole map. A Build fails for this unit alone,
    /// with a `SimEvent::BuildFailed` without problems, refunded if it was the last to hold it (`Sim::refund_builds`).
    fn no_spot(&mut self, id: ObjectId, order: OrderKind) {
        let dropped = self.objects.get(&id).and_then(|o| {
            let OrderKind::Build { kind, .. } = order else {
                return None;
            };
            let ticket = super::builds::Ticket::of(o, order);
            Some(Dropped {
                worker: id,
                owner: o.owner?,
                kind,
                ticket,
            })
        });
        self.end_work(id);
        if let Some(mover) = self.objects.get_mut(&id).and_then(|o| o.mover.as_mut()) {
            mover.stop();
        }
        if let (
            Some(dropped),
            OrderKind::Build {
                kind,
                origin,
                orientation,
            },
        ) = (dropped, order)
        {
            self.events.push(SimEvent::BuildFailed {
                worker: id,
                kind,
                origin,
                orientation,
                problems: PlacementCheck::default(),
            });
            self.refund_builds(&[dropped]);
        }
    }

    /// Unit `id`'s current order ends; its queued orders go on from its next turn in 5a, and a walk it leaves ends
    /// there too (`orders::engage::idle`, or its next order's start).
    fn end_work(&mut self, id: ObjectId) {
        if let Some(orders) = self.objects.get_mut(&id).and_then(|o| o.orders.as_mut()) {
            orders.current = None;
        }
    }
}

#[cfg(test)]
mod tests;
