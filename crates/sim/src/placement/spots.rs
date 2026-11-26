//! Spots around a footprint (plan decision T62, slice M5-F1): where a unit put on the map beside a building stands, by
//! one predictable search (`systems/controls/behaviors.md > Core principle: predictable, not optimal`, design change
//! 0223). A Laser Turret's Agent comes out "at the **first clear spot next to the turret**, searched in a fixed order
//! starting from the side it came from" (design change 0298 point 7, `sim/sites.rs`); the units inside a building that
//! is destroyed are evacuated to them (`factions/cults/economy.md > Cults Building Mechanics > Evacuation on
//! Destruction`: "evacuated units appear around the building's footprint wherever there is open space",
//! `sim/holding.rs`).
//!
//! - **The spots** ([`Spots`], [`Ring`]): rings of unit-sized spots around the footprint. Ring 0's boxes stand against
//!   it, each further ring one box further out. A ring's spots are its four corners, the middles of its four sides,
//!   and as many more along each side as fit between its middle and its corners, a box apart, as many on each half.
//!   They are numbered clockwise on screen (+y is south) from the middle of the east side. A ring with no room between
//!   (a unit at least as wide as half the footprint's side) is the eight directions out of it, east first: the Laser
//!   Turret's for its Agent, as M4-S1 had them.
//! - **The order** ([`Round`]): each ring from its spot in the one of the eight directions (a side's middle or a
//!   corner) nearest a direction given, then round both ways alternately (0298 point 7) or clockwise (Placeholder
//!   (Q0106 point 1), the evacuation's), ring after ring.
//! - **The check** is the caller's (`Sim::spot_around`): each spot in that order until one takes the unit, by
//!   `Sim::check_unit_fits`, the rule of every unit put on the map, against what stands there, the units placed before
//!   it included.

use std::ops::RangeInclusive;

use glam::Vec2;

use crate::math::{self, PI};
use crate::network::WALL_CLEARANCE;
use crate::object::Placement;
use crate::units::SU_PER_GU;

/// The four sides of a footprint, clockwise on screen from the east: the direction out of each, and the way along it
/// clockwise. The next side's direction out is this one's way along.
const SIDES: [(Vec2, Vec2); 4] = [
    (Vec2::new(1.0, 0.0), Vec2::new(0.0, 1.0)),
    (Vec2::new(0.0, 1.0), Vec2::new(-1.0, 0.0)),
    (Vec2::new(-1.0, 0.0), Vec2::new(0.0, -1.0)),
    (Vec2::new(0.0, -1.0), Vec2::new(1.0, 0.0)),
];

/// How a search goes round a ring from its first spot (module docs).
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum Round {
    /// Further and further round from the first spot, both ways alternately, of two equally far the one numbered lower
    /// first, and the opposite one last: "searched in a fixed order starting from the side it came from" (design
    /// change 0298 point 7: the Laser Turret's Agent).
    BothWays,
    /// Clockwise from the first spot: Placeholder (Q0106 point 1), "each ring clockwise from the middle of the exit
    /// side" (the evacuation's).
    Clockwise,
}

impl Round {
    /// The numbers of a ring of `n` spots, in the order this round tries them from spot `first`.
    pub fn order(self, n: usize, first: usize) -> Vec<usize> {
        if n == 0 {
            return Vec::new();
        }
        let first = first % n;
        match self {
            Round::Clockwise => (0..n).map(|i| (first + i) % n).collect(),
            Round::BothWays => {
                let mut order = Vec::with_capacity(n);
                order.push(first);
                for d in 1..=n / 2 {
                    let (a, b) = ((first + d) % n, (first + n - d) % n);
                    if a == b {
                        order.push(a);
                    } else {
                        order.extend([a.min(b), a.max(b)]);
                    }
                }
                order
            }
        }
    }
}

/// The one of the eight directions out of a footprint nearest `direction`, numbered clockwise on screen from the east
/// (east 0, south-east 1, ..., north-east 7): its angle in eighths of a turn, rounded to the nearest (half away from
/// zero, as M4-S1's).
fn eighth(direction: Vec2) -> usize {
    let eighths = math::vector_angle(direction) / (PI / 4.0);
    (eighths.round() as i32).rem_euclid(8) as usize
}

/// One ring of spots around a footprint ([`Spots::ring`]).
#[derive(Clone, Debug, PartialEq)]
pub struct Ring {
    /// Its spots, numbered clockwise from the middle of the east side, each with the direction out of the footprint
    /// there: its side's for a spot along a side, the diagonal for a corner.
    pub spots: Vec<(Vec2, Vec2)>,
    /// The number of its spot in each of the eight directions ([`eighth`]): the side middles and the corners.
    pub anchors: [usize; 8],
}

/// The spots around a footprint for a unit of one size (module docs).
#[derive(Copy, Clone, Debug, PartialEq)]
pub struct Spots {
    /// The footprint's centre.
    centre: Vec2,
    /// From the centre to ring 0's spots along each axis: half the footprint, half the unit's box, and a margin
    /// against rounding (`network::WALL_CLEARANCE`), so that its box stands against the footprint and overlaps none of
    /// it.
    reach: Vec2,
    /// From one spot to the next along a side, and from one ring to the next: the unit's box and the same margin, so
    /// that two boxes on neighbouring spots touch and never overlap.
    step: Vec2,
}

impl Spots {
    /// The spots around the footprint of `placement` for a unit whose box has half extents `half`.
    pub fn around(placement: &Placement, half: Vec2) -> Spots {
        let footprint = Vec2::new(placement.w as f32, placement.h as f32) * (SU_PER_GU / 2.0);
        Spots {
            centre: placement.center(),
            reach: footprint + half + Vec2::splat(WALL_CLEARANCE),
            step: half * 2.0 + Vec2::splat(WALL_CLEARANCE),
        }
    }

    /// Ring `ring`'s spots (module docs): ring 0 against the footprint, each next one a step further out.
    pub fn ring(&self, ring: u32) -> Ring {
        let reach = self.reach + self.step * ring as f32;
        // How many spots fit on each half of a side between its middle and its corners, a step apart: the one `j`
        // steps from the middle, if a step more still leaves the corner's box clear of it.
        let between = |reach: f32, step: f32| {
            let mut count = 0_u32;
            while step > 0.0 && (count + 2) as f32 * step <= reach {
                count += 1;
            }
            count
        };
        // Along a side facing along x the spots go along y, and the other way round.
        let along = |out: Vec2| {
            if out.x != 0.0 {
                (between(reach.y, self.step.y), self.step.y)
            } else {
                (between(reach.x, self.step.x), self.step.x)
            }
        };
        let mut spots = Vec::new();
        let mut anchors = [0; 8];
        for (side, &(out, way)) in SIDES.iter().enumerate() {
            let (count, step) = along(out);
            let middle = self.centre + out * reach;
            anchors[2 * side] = spots.len();
            spots.push((middle, out));
            spots.extend((1..=count).map(|j| (middle + way * (j as f32 * step), out)));
            let corner = out + way;
            anchors[2 * side + 1] = spots.len();
            spots.push((self.centre + corner * reach, corner.normalize()));
            // The next side's half up to its middle, from the corner.
            let (next, next_way) = SIDES[(side + 1) % 4];
            let (count, step) = along(next);
            let next_middle = self.centre + next * reach;
            spots.extend(
                (1..=count)
                    .rev()
                    .map(|j| (next_middle - next_way * (j as f32 * step), next)),
            );
        }
        Ring { spots, anchors }
    }

    /// The first spot a search tries that `fits` takes, with the direction out of the footprint there; `None` if none
    /// does: ring after ring through `rings`, each from its spot in the direction nearest `towards` ([`eighth`]: a
    /// corner or a side's middle) and on by `round`.
    pub fn first(
        &self,
        rings: RangeInclusive<u32>,
        towards: Vec2,
        round: Round,
        mut fits: impl FnMut(Vec2) -> bool,
    ) -> Option<(Vec2, Vec2)> {
        let start = eighth(towards);
        for ring in rings {
            let ring = self.ring(ring);
            let found = (round.order(ring.spots.len(), ring.anchors[start]).into_iter())
                .map(|i| ring.spots[i])
                .find(|&(spot, _)| fits(spot));
            if found.is_some() {
                return found;
            }
        }
        None
    }

    /// The last ring a search on a map `size` space units wide and high need try: the first whose reach from the
    /// centre gets to the map's farthest edge along both axes, so that its spots, and every later ring's, are off it.
    pub fn last_ring(&self, size: Vec2) -> u32 {
        let farthest = self.centre.max(size - self.centre);
        let rings = ((farthest - self.reach) / self.step).ceil().max_element();
        if rings.is_finite() { rings.max(0.0) as u32 } else { 0 }
    }
}

#[cfg(test)]
mod tests;
