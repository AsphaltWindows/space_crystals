//! The spots' step (`crate::spot_work`, plan decision T62, design change 0339, slice M5-E): right after the arrivals'
//! step (`Sim::step_deployments`), each unit that works at a footprint from a spot beside it gets its own spot along
//! the footprint's wall (`placement::Wall`), and walks there round the footprint; a unit that ended its walk out of
//! reach looks again now and then. Before 5b, so that the paths of this tick serve the walks it begins
//! (`ARCHITECTURE.md > Tick pipeline`).

use glam::Vec2;

use super::Sim;
use super::builds::Dropped;
use crate::defs::ObjectKind;
use crate::events::SimEvent;
use crate::ids::ObjectId;
use crate::mover::{MoveGoal, MoveStatus};
use crate::object::Placement;
use crate::orders::OrderKind;
use crate::placement::{self, PlacementCheck, Wall};
use crate::shape::Shape;
use crate::spot_work::{self, Claim, Work, WorkAt, suits, within_reach, work_of};
use crate::tuning::construction::{BUILD_STANDOFF_SU, HUG_STEP_SU, RESPOT_TICKS};
use crate::units::SU_PER_GU;

/// The room a builder whose box has half extents `half` keeps clear round itself (`spot_work::Claim::room`): its box
/// and `tuning::construction::BUILD_STANDOFF_SU` all round, so that it stands that far off the footprint, and twice
/// that far from the builders beside it. One row of the walls round a footprint ([`Sim::free_spot`]) is twice this.
fn room(half: Vec2) -> Vec2 {
    half + Vec2::splat(BUILD_STANDOFF_SU)
}

/// The builders' wall round the footprint `at` for a unit whose box has half extents `half` (`placement::Wall`):
/// its box `tuning::construction::BUILD_STANDOFF_SU` off the footprint, within reach of it.
fn builders_wall(at: &Placement, half: Vec2) -> Wall {
    Wall::around(at, half, BUILD_STANDOFF_SU)
}

impl Sim {
    /// The spots' step (module docs): for each unit from before this tick (`first_new`) whose current order works at a
    /// footprint from a spot beside it (`spot_work::work_of`), in id order:
    /// - an order on a structure that no longer suits it ends (`spot_work::suits`);
    /// - a unit on its way round the footprint to its spot heads on for its next leg once that changes ([`Sim::leg`],
    ///   [`Sim::head_on`]);
    /// - the units without a spot, nearest the footprint first (edge to edge, then by id), each get the first spot of
    ///   [`Sim::free_spot`]'s search that no other unit of its owner's working at the same footprint claims, on the
    ///   builders' wall (row 0), or else on every row on the map from the third on (row 1 left free, the way round the
    ///   builders), and walk there ([`Sim::send_to`]); with none on the whole map, the order ends ([`Sim::no_spot`]);
    /// - in the ticks that are multiples of `tuning::construction::RESPOT_TICKS`, a unit whose walk is over takes the
    ///   place where it stands as its spot if it is within reach of the footprint, so that a unit that stopped short
    ///   of its spot doesn't keep another from the spot it stands on; out of reach, it looks for a free spot on the
    ///   builders' wall, its own claim aside, and walks there if it finds one.
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
        self.head_on(&placed, &claims);
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
            // The builders' wall within reach, or a row from the third on to wait on: row 1 stays free, the way
            // round the builders for those walking on along the wall.
            let rows = std::iter::once(0).chain(2..=self.rows_on_map(kind, &at));
            match self.free_spot(id, &at, rows, &claims) {
                Some(spot) => {
                    self.send_to(id, &at, spot, &claims);
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
                self.send_to(id, &at, spot, &claims);
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

    /// The claim of unit `id` on `spot` beside the footprint `at`, its [`room`] round it.
    fn claim(&self, id: ObjectId, at: Placement, spot: Vec2) -> Option<Claim> {
        let o = self.objects.get(&id)?;
        Some(Claim {
            unit: id,
            owner: o.owner,
            at,
            spot,
            room: room(o.def().half_extents()?),
        })
    }

    /// The last row of walls round the footprint `at` for a unit of `kind` with a point on the map ([`Sim::free_spot`],
    /// `placement::Wall::off_map`): a search "as far out as needed", as T62's is (`Sim::rings_on_map`).
    fn rows_on_map(&self, kind: ObjectKind, at: &Placement) -> u32 {
        let half = kind.def().half_extents().unwrap_or(Vec2::ZERO);
        let size = Vec2::new(self.map.width() as f32, self.map.height() as f32) * SU_PER_GU;
        builders_wall(at, half).off_map(room(half) * 2.0, size)
    }

    /// The first spot of `rows` round the footprint `at` where unit `id` fits now, that no other unit's claim among
    /// `claims` keeps it from (`spot_work::Claim::keeps`, its [`room`] against theirs): design change 0339's search
    /// (`factions/cults/economy.md > Cults Building Mechanics > Group construction rules`: "each Recruit walks to the
    /// **part of the footprint closest to it**; if that part is taken (blocked), it **wall-hugs** along the footprint
    /// until it reaches a part where it can work"), along each row's wall in turn (`placement::Wall::hug`): row 0 the
    /// builders' wall, each next one a [`room`] twice over further out. Along it, the point nearest the unit first,
    /// then further and further from there both ways, `tuning::construction::HUG_STEP_SU` at a time, of two as far the
    /// clockwise one first (**Placeholder (not in the design: the way along the wall, and how finely it is searched)**:
    /// the nearest part either way, 0339 leaving it to the implementation, "shorter way first, with a fixed
    /// tie-break"). A spot fits by the rule of every unit put on the map (`Sim::check_unit_fits`), the unit itself
    /// aside: on the map, its box clear of blocked tiles, of blocking footprints and of the units of its layer.
    ///
    /// A unit claims its spot as it gets the order, not on arrival (**Placeholder (not in the design: when a builder
    /// claims its part of the wall)**, 0339 leaving it to the implementation), so that the units of a group, those
    /// nearest the footprint first, spread along the wall as they set off, each from its own nearest part, and none
    /// walks to a part another is walking to. The search is the one exception to `Sim::spot_around`'s fixed order of
    /// T62's spots (design change 0339: "The fixed order of spots goes").
    fn free_spot(
        &self,
        id: ObjectId,
        at: &Placement,
        rows: impl IntoIterator<Item = u32>,
        claims: &[Claim],
    ) -> Option<Vec2> {
        let o = self.objects.get(&id)?;
        let (kind, owner, pos, half) = (o.kind, o.owner, o.pos, o.def().half_extents()?);
        let (room, wall) = (room(half), builders_wall(at, half));
        rows.into_iter().find_map(|row| {
            let mut hug = wall.out(room * (2.0 * row as f32)).hug(pos, HUG_STEP_SU);
            hug.find(|&spot| {
                self.map.contains_point(spot)
                    && !claims.iter().any(|claim| claim.keeps(id, owner, at, spot, room))
                    && self.check_unit_fits(kind, spot, Some(id)).is_ok()
            })
        })
    }

    /// Unit `id` takes `spot` beside the footprint `at` for its work, and walks there, round the footprint and the
    /// others' spots among `claims` ([`Sim::leg`]). Each leg presses on (`Mover::press_on`): **Placeholder (not in the
    /// design: a builder's walk to its spot)**, as design change 0315's walk to a hatch (`factions/syndicate/units/
    /// agent.md > Gathering > Gathering rules`, "Waiting at the hatch"), since a unit short of its spot builds nothing:
    /// blocked near it by units that won't move, the builders already at theirs, it doesn't count as arrived
    /// (`systems/controls/command_behaviors.md > MovingToLocation > Occupied target location`) but steers round them,
    /// looks for other routes, and ends its walk when none is left, as any stuck unit does.
    fn send_to(&mut self, id: ObjectId, at: &Placement, spot: Vec2, claims: &[Claim]) {
        let (tick, leg) = (self.tick, self.leg(id, at, spot, claims));
        let Some(o) = self.objects.get_mut(&id) else {
            return;
        };
        if let Some(slot) = spot_work::spot_mut(o) {
            *slot = Some(spot);
        }
        if let Some(mover) = &mut o.mover {
            mover.go_to(MoveGoal::Point(leg), None, tick);
            mover.press_on();
        }
    }

    /// Where unit `id` heads for now on its way to `spot` beside the footprint `at` (design change 0339: it "wall-hugs
    /// along the footprint"), never over the footprint, since a Recruit of the order standing on it as the first
    /// arrives fails the whole order (`sim/builds/shared.rs`), nor through the builders at the wall: straight to its
    /// spot while that way keeps its box off the footprint and off the rooms of the others' spots among `claims`
    /// (`placement::crosses`); else round them, along the wall a row further out than its spot's
    /// (`placement::Wall::row_of`: row 1 for a builder's spot, which nobody waits on), by that wall's corners where the
    /// way along it turns, the shorter way round (`placement::Wall::next_corner`), to its point nearest the spot, and
    /// in from there, between the builders beside the spot, whose rooms leave a box and twice the standoff clear
    /// (**Placeholder (not in the design: the way round the footprint)**). It heads on for its next leg once within two
    /// steps of its walk of a corner or of the way in ([`Sim::head_on`]), so that it never ends its walk there, short
    /// of its spot. A unit on the footprint goes straight to its spot, the quickest way off, as does one whose way in
    /// is off the map; one whose next corner is off the map heads for the way in.
    fn leg(&self, id: ObjectId, at: &Placement, spot: Vec2, claims: &[Claim]) -> Vec2 {
        let Some(o) = self.objects.get(&id) else {
            return spot;
        };
        let Some(half) = o.def().half_extents() else {
            return spot;
        };
        let (pos, centre) = (o.pos, at.center());
        let footprint = Vec2::new(at.w as f32, at.h as f32) * (SU_PER_GU / 2.0);
        if (pos - centre).abs().cmplt(footprint + half).all() {
            return spot;
        }
        let others = (claims.iter()).filter(|claim| claim.unit != id && claim.owner == o.owner && claim.at == *at);
        let mut rooms = std::iter::once((centre, footprint)).chain(others.map(|claim| (claim.spot, claim.room)));
        if !rooms.any(|(middle, room)| placement::crosses(pos, spot, middle, room + half)) {
            return spot;
        }
        let (wall, row) = (builders_wall(at, half), room(half) * 2.0);
        let way = wall.out(row * (wall.row_of(spot, row) + 1) as f32);
        let way_in = way.point(way.along(spot));
        // Two steps of its walk: it can't get there, and end its walk, before the next tick's turn.
        let turn = 2.0 * o.def().unit.as_ref().map_or(0.0, |unit| unit.movement.max_speed());
        if pos.distance(way_in) <= turn || !self.map.contains_point(way_in) {
            return spot;
        }
        let mut from = pos;
        for _ in 0..4 {
            match way.next_corner(from, way_in) {
                Some(corner) if pos.distance(corner) <= turn => from = corner,
                Some(corner) if self.map.contains_point(corner) => return corner,
                _ => break,
            }
        }
        way_in
    }

    /// The units of `placed` with a spot, on their way to a corner of their way round or to its way in ([`Sim::leg`]),
    /// head on for their next leg as soon as it is another, by `claims`: the next corner, the way in, or their spot
    /// once the straight way there is clear.
    fn head_on(&mut self, placed: &[(ObjectId, Work, Placement)], claims: &[Claim]) {
        let tick = self.tick;
        for &(id, work, at) in placed {
            let Some(spot) = work.spot else {
                continue;
            };
            let heading = (self.objects.get(&id).and_then(|o| o.mover.as_ref())).and_then(|mover| match mover.goal {
                MoveGoal::Point(goal) if mover.status == MoveStatus::Moving && goal != spot => Some(goal),
                _ => None,
            });
            let Some(heading) = heading else {
                continue;
            };
            let leg = self.leg(id, &at, spot, claims);
            if leg != heading
                && let Some(mover) = self.objects.get_mut(&id).and_then(|o| o.mover.as_mut())
            {
                mover.go_to(MoveGoal::Point(leg), None, tick);
                mover.press_on();
            }
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
