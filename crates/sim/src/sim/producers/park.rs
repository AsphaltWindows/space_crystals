//! (M6-GF) Parking round a taken rally point: `core/entities.md > Entrance and exit > Exit Doorway` point 7 (design
//! change 0451, answering Q0177): "a newly released vehicle whose rally point is already taken by parked vehicles
//! **parks at the nearest free spot around the point** (the free spot nearest the point where it fits), not in a line
//! behind the last one back toward the door. Applies to the rally move of a newly released vehicle, at any rally
//! distance (the default or one the player set). Infantry yield and are unaffected."
//!
//! - **Which move** ([`Sim::park`]): the rally move of a wheeled or tracked vehicle that a building releases through
//!   the doorway of an exit side (`production::ExitDoor::Side`), newly made, when its rally point gives it a Move to a
//!   point (`Sim::rally_order`): today the Factory's Raptor and Twister (`Sim::release_unit`) and the Chop Shop's
//!   Seeker and Hand of Heaven, which its pool trains out of the Recruits it holds (`Sim::let_out`, [`Sim::park_out_of`]).
//!   Not infantry, which yields; not a unit ejected or released unchanged; not a unit out of a portal's energy doorway,
//!   nor one released into a Tunnel Network or onto a runway ("Aircraft and units released underground (e.g.
//!   Syndicate Tunnels) are not covered by this rule").
//! - **Its end**: the move stays the Move to the rally point that every released unit gets, its own destination
//!   (`orders::Order::slot`, as the magic box gives each unit of a group its own) the spot it parks on. It travels
//!   and arrives as any Move does (`systems/controls/command_behaviors.md > MovingToLocation`), and a player's Move,
//!   which nothing here touches, keeps its own arrival. While the rally point is free, the move is what it always was.
//! - **Taken** ([`Parking::takes`]): within the vehicle's reach of the point, the half diagonal of its Silhouette (the
//!   circle its box lies in whichever way it faces, since it comes to rest facing the way its drive there leaves it:
//!   **placeholder reading**), stands a unit of its layer that doesn't yield, a wheeled or tracked vehicle of any owner
//!   (`UnitBaseDef::yields`: "idle wheeled and tracked vehicles don't step aside", design change 0440), or lies a spot
//!   another of its owner's vehicles is driving to, the destination of its Move to a point, within the two reaches:
//!   that vehicle's claim, so that two vehicles never head for one spot, whatever order sent the other there
//!   (**placeholder (not in the design: two vehicles on their way to one spot)**, as a builder's claim on its spot,
//!   `sim/spot_work.rs`). Own infantry, which steps aside for a vehicle (point 4, design change 0353), takes nothing.
//! - **Free**: a spot nothing takes, where the vehicle fits by the rule of every unit put on the map
//!   (`Sim::check_unit_fits_facing`: on the map, its box clear of blocked tiles, of blocking footprints and of the
//!   units of its layer), in four facings 45° apart, and off the lane out of the doorway it left by, the box it has
//!   just outside the exit side, touching it: **placeholder (not in the design: a spot in the doorway's way out)**,
//!   so that a cluster round a point near the building never stalls its doorway. And one it can get to: the straight
//!   way there from where it came out, as wide as the vehicle and `Production::parking_lane_margin_su` more on either
//!   side, meets no unit that takes a spot and no claimed spot (**placeholder (not in the design: a spot behind the
//!   cluster)**). A spot beyond the vehicles already parked, which it would drive into, is no spot for it: it would
//!   meet them within 3 grid units of its spot, MovingToLocation's "Occupied target location", and stop there, in
//!   front of the cluster, as in the line toward the door that the rule does away with.
//! - **The search** ([`Parking::spot`]): the rings round the point, `Production::parking_step_su` apart, out to
//!   `Production::parking_radius_gu`; on each ring its points that far apart, from the one straight beyond the point
//!   (on along the way from the door to it) round both ways in turn, the positive one (clockwise on screen) first, the
//!   one back toward the door last. The first free one is the spot. With none within the radius the vehicle drives to
//!   the point, as before. **Placeholder (not in the design: how finely and how far it looks, and which of two spots
//!   as near it takes)**.

use glam::Vec2;

use super::release::exit_half;
use crate::defs::{Domain, ObjectKind, UnitBase};
use crate::ids::{ObjectId, PlayerId};
use crate::math::{self, PI};
use crate::orders::{OrderKind, Target};
use crate::production::{Door, ExitDoor};
use crate::ruleset::rules;
use crate::shape::Shape;
use crate::sim::Sim;
use crate::units::SU_PER_GU;

/// The four facings a free spot fits the vehicle in: east, then each 45° on.
const FACINGS: [f32; 4] = [0.0, PI / 4.0, PI / 2.0, 3.0 * PI / 4.0];

/// A released vehicle's search for its spot round its rally point (module docs).
struct Parking {
    /// The vehicle, its kind and its reach: the half diagonal of its Silhouette.
    unit: ObjectId,
    kind: ObjectKind,
    reach: f32,
    /// Its rally point.
    point: Vec2,
    /// The shapes of the units near the point that take spots: those of its layer that don't yield.
    takers: Vec<Shape>,
    /// The spots its owner's other vehicles near the point are driving to, with their reaches.
    claims: Vec<(Vec2, f32)>,
    /// The lane out of the doorway it left by: its box just outside the exit side, touching it.
    lane: Shape,
    /// Where it came out, and half the width of the way it wants from there to its spot.
    from: Vec2,
    across: f32,
}

impl Parking {
    /// Whether something takes the vehicle's place at `p` (module docs, *Taken*).
    fn takes(&self, p: Vec2) -> bool {
        self.takers.iter().any(|shape| shape.distance_to(p) < self.reach)
            || (self.claims.iter()).any(|&(at, reach)| at.distance(p) < self.reach + reach)
    }

    /// The spot the vehicle parks on (module docs, *The search*): the first free point of the rings round its rally
    /// point, from the nearest out, taken in turn from `ahead` (an angle) round both ways. `None` if none is free.
    fn spot(&self, sim: &Sim, ahead: f32) -> Option<Vec2> {
        let tuning = &rules().tuning.production;
        let step = tuning.parking_step_su;
        let rings = (tuning.parking_radius_gu * SU_PER_GU / step) as u32;
        (1..=rings).find_map(|ring| {
            let radius = ring as f32 * step;
            // As many points as the ring's length takes `step` apart, and at least 8.
            let count = ((2.0 * PI * radius / step) as u32 + 1).max(8);
            (0..count).find_map(|i| {
                // 0, +1, −1, +2, −2, … of the ring's points from `ahead`.
                let turns = if i % 2 == 1 {
                    i.div_ceil(2) as f32
                } else {
                    -((i / 2) as f32)
                };
                let p = self.point + math::facing_vector(ahead + turns * 2.0 * PI / count as f32) * radius;
                self.free(sim, p).then_some(p)
            })
        })
    }

    /// Whether the vehicle may park at `p` (module docs, *Free*).
    fn free(&self, sim: &Sim, p: Vec2) -> bool {
        sim.map.contains_point(p)
            && !self.takes(p)
            && self.lane.distance_to(p) >= self.reach
            && self.reaches(p)
            && (FACINGS.iter()).all(|&facing| {
                sim.check_unit_fits_facing(self.kind, p, facing, Some(self.unit))
                    .is_ok()
            })
    }

    /// Whether the straight way from where the vehicle came out to `p`, [`Parking::across`] either side of it, meets
    /// no unit that takes a spot and no claimed spot (module docs, *Free*).
    fn reaches(&self, p: Vec2) -> bool {
        let way = p - self.from;
        let length = way.length();
        if length == 0.0 {
            return true;
        }
        let lane = Shape::oriented(
            (self.from + p) / 2.0,
            Vec2::new(length / 2.0, self.across),
            way / length,
        );
        !self.takers.iter().any(|shape| lane.overlaps(shape))
            && (self.claims.iter()).all(|&(at, reach)| lane.distance_to(at) >= reach)
    }
}

/// Whether a unit of `base` takes part in the parking (module docs): a wheeled or tracked vehicle.
fn parks(base: UnitBase) -> bool {
    matches!(base, UnitBase::WheeledVehicle | UnitBase::TrackedVehicle)
}

impl Sim {
    /// (M6-GF) Design change 0451 (module docs): `unit`, just released through `door` with the order its rally point
    /// gives it, parks at the free spot nearest its rally point if that is taken: its Move to the point gets that spot
    /// as its own destination (`orders::Order::slot`). Nothing for any other unit or order, nor with the point free or
    /// no spot free near it.
    pub(super) fn park(&mut self, unit: ObjectId, door: &Door) {
        let Some(spot) = self.parking_spot(unit, door) else {
            return;
        };
        let orders = self.objects.get_mut(&unit).and_then(|o| o.orders.as_mut());
        if let Some(order) = orders.and_then(|orders| orders.queue.back_mut()) {
            order.slot = Some(spot);
        }
    }

    /// [`Sim::park`] for `unit`, which `building`'s pool just trained and let out of its exit side's doorway with its
    /// rally point's order (`Sim::let_out`): through the door it came out of.
    pub(in crate::sim) fn park_out_of(&mut self, building: ObjectId, unit: ObjectId) {
        let (Some(o), Some(pos)) = (self.objects.get(&building), self.objects.get(&unit).map(|o| o.pos)) else {
            return;
        };
        if let Some(ExitDoor::Side(doorway)) = ExitDoor::of(o) {
            let door = doorway.door_at(pos);
            self.park(unit, &door);
        }
    }

    /// The spot of [`Sim::park`], if the unit parks.
    fn parking_spot(&self, unit: ObjectId, door: &Door) -> Option<Vec2> {
        let o = self.objects.get(&unit)?;
        let def = o.def();
        let unit_def = def.unit.as_ref().filter(|u| parks(u.base))?;
        let order = o.orders.as_ref()?.queue.back()?;
        let OrderKind::Move(Target::Point(point)) = order.kind else {
            return None;
        };
        let size = def.half_extents()?;
        let reach = size.length();
        // (M6-ST1) Its type's Domain, its own over its base's (`UnitDef::domain`), as the others' below.
        let domain = unit_def.domain();
        let tuning = &rules().tuning.production;
        let near = tuning.parking_radius_gu * SU_PER_GU + reach;
        let half = exit_half(o.kind, door);
        let depth = door.outward().abs().dot(half);
        // Its half extents are half its length along its facing and half its width (`ObjectDef::half_extents`).
        let across = size.y + tuning.parking_lane_margin_su;
        // Everything between where it came out and the point's ring of spots may be in its way.
        let near_way = near + o.pos.distance(point);
        let parking = Parking {
            unit,
            kind: o.kind,
            reach,
            point,
            takers: self.parking_takers(unit, domain, point, near_way),
            claims: self.parking_claims(unit, o.owner, domain, point, near_way),
            lane: Shape::rect(door.spot(depth), half),
            from: o.pos,
            across,
        };
        if !parking.takes(point) {
            return None;
        }
        let toward = point - door.middle();
        let ahead = math::vector_angle(if toward == Vec2::ZERO { door.outward() } else { toward });
        parking.spot(self, ahead)
    }

    /// The shapes of the units of `domain` but `unit` that don't yield, within `near` and their own reach of `point`
    /// (module docs, *Taken*).
    fn parking_takers(&self, unit: ObjectId, domain: Domain, point: Vec2, near: f32) -> Vec<Shape> {
        (self.objects.values())
            .filter(|other| other.id != unit)
            .filter_map(|other| {
                let def = other.def();
                let other_unit = def.unit.as_ref()?;
                let reach = def.half_extents()?.length();
                let stands = other_unit.domain() == domain && !other_unit.yields();
                let takes = stands && other.pos.distance(point) < near + reach;
                takes.then(|| Shape::of(other))
            })
            .collect()
    }

    /// The spots within `near` and their own reach of `point` that `owner`'s vehicles of `domain` but `unit` are on
    /// their way to, the destinations of their Moves to a point, current or next, with their reaches (module docs,
    /// *Taken*).
    fn parking_claims(
        &self,
        unit: ObjectId,
        owner: Option<PlayerId>,
        domain: Domain,
        point: Vec2,
        near: f32,
    ) -> Vec<(Vec2, f32)> {
        (self.objects.values())
            .filter(|other| other.id != unit && other.owner == owner)
            .filter_map(|other| {
                let def = other.def();
                let other_unit = def.unit.as_ref()?;
                let orders = other.orders.as_ref()?;
                let order = (orders.current.as_ref().map(|active| &active.order)).or_else(|| orders.queue.front())?;
                let OrderKind::Move(Target::Point(_)) = order.kind else {
                    return None;
                };
                let (at, reach) = (order.destination()?, def.half_extents()?.length());
                let stands = other_unit.domain() == domain && !other_unit.yields();
                let claims = stands && at.distance(point) < near + reach;
                claims.then_some((at, reach))
            })
            .collect()
    }
}
