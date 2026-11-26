//! The portals' energy doorway — design change 0145 (`factions/colonists/structures/README.md > Portals > Portal Exits
//! (energy doorway)`, which answers question Q0041 points 5 and 6): "At the centre of every portal is an energy doorway
//! from which units emerge. Portals don't use exit sides".
//!
//! 1. **Materializing**: a finished unit appears at the portal's **centre** ([`EnergyDoorway::centre`]) and moves
//!    straight out along its heading (`Mover::exit_heading`) until its box is clear of the footprint
//!    (`movement::doorway::emerge`). It passes through its own portal's footprint; nothing else can.
//! 2. **Open directions** ([`EnergyDoorway::is_open`]): a direction is open if the ray from the centre along it leaves
//!    the footprint through an **open side**, one that the exit rule letters (`ExitRule::Centre`: the A sides),
//!    wherever that side faces once the portal is placed (`crate::sides::sides`). The Robo Portal (AAAA): every
//!    direction. The Bio Portal (AABB, top and right unturned): "a half-circle from pointing at the top-left corner
//!    round to pointing at the bottom-right corner", turning and flipping with the building. Nothing here names a
//!    portal: the open arc comes from the sides, so the Heavy and Star Portals will take this code with their own
//!    letters. **The Director's reading**: a direction exactly on an end of the arc, a corner between an open and a
//!    closed side, is open ([`EnergyDoorway::arc_ends`]).
//! 3. **The heading** ([`EnergyDoorway::heading`]), chosen when the unit is released:
//!    - straight **towards the RallyPoint** (`Rally::point`: where the owner last saw an object) if that direction is
//!      open and its way out is free;
//!    - else "the nearest open direction in which it can get out" (**the Director's reading**): of the open
//!      directions whose way out is free, the one at the smallest angle from the rally's, the clockwise one first
//!      where two tie. The directions tried are the rally's itself, the ends of the arc and the [`HEADINGS`];
//!    - with **no RallyPoint** (or one right at the centre): "a random open, unblocked direction (deterministic
//!      random)": one of the open [`HEADINGS`] whose way out is free, drawn uniformly from the sim's Rng with
//!      `Rng::below(n)`, and only when there are `n` ≥ 2 of them. **The resolution** is the 32 of [`HEADINGS`],
//!      11.25° apart (the Director's: "a finite set ... is fine"): the Bio Portal has 17 open ones, its arc's ends
//!      included, and the Robo Portal all 32.
//!    - **The way out is free** ([`EnergyDoorway::way_out_free`], the Director's reading): the unit's box, swept
//!      straight from the centre along the direction to where it is clear of the footprint and one unit length (its
//!      longer side) beyond, enters no solid tile of the true grid but the portal's own (structures, terrain, the
//!      map's edge) and no unit of its layer.
//! 4. **Can't get out**: if no open direction is free, the unit still comes out, along the nearest open direction to
//!    the rally's (with none, a random open one). It walks out as far as there is room and stays there, "half out at
//!    the edge of the footprint", a normal unit (selectable, attackable, able to attack).
//! 5. **The next unit** reaches 100% and waits fully inside while a unit the portal released is still in its doorway
//!    (`Sim::release_unit`, as Exit Doorway rule 3), and is refunded if the portal is destroyed (design changes 0084
//!    and 0085, `Sim::refund_production`).
//!
//! **The exception** ([`EnergyDoorway::allows`], [`EnergyDoorway::box_clear`]): while the unit's box overlaps the
//! footprint, the portal's own tiles don't block it, but for ground under them that it can't stand on (slice M5-Z1,
//! `Frame::blocks`), and it never moves closer to the centre; every other blocked tile blocks it, and units block it
//! and it blocks them, as ever. Its walk keeps it on its heading's ray, which leaves through an open side; invariant
//! I2 checks that it does (`testing::invariants`).

use std::collections::BTreeMap;

use core::f32::consts::{FRAC_1_SQRT_2, SQRT_2};
use glam::Vec2;

use super::Frame;
use crate::defs::{ExitRule, UnitDef};
use crate::ids::ObjectId;
use crate::nav::{NavGrid, segment_enters_box};
use crate::object::Object;
use crate::paths::boxes_overlap;
use crate::sides::sides;
use crate::units::{SU_PER_GU, TilePos};

/// How many directions [`HEADINGS`] has: 32, 11.25° apart (module docs, 3).
pub const HEADING_COUNT: usize = 32;

/// The directions a unit may leave an energy doorway in with no RallyPoint, and those tried around the rally's
/// (module docs, 3): [`HEADING_COUNT`] unit vectors 11.25° apart, from east clockwise on screen (+y is south). They
/// are built from the first eighth of a turn by the square's symmetries, so the four along the axes and the four
/// diagonals are exact and a turned or flipped portal is offered the same set, turned or flipped.
pub const HEADINGS: [Vec2; HEADING_COUNT] = {
    // (cos, sin) of 0°, 11.25°, 22.5°, 33.75° and 45°.
    const EIGHTH: [(f32, f32); 5] = [
        (1.0, 0.0),
        (0.980_785_25, 0.195_090_32),
        (0.923_879_5, 0.382_683_43),
        (0.831_469_6, 0.555_570_24),
        (FRAC_1_SQRT_2, FRAC_1_SQRT_2),
    ];
    let mut headings = [Vec2::ZERO; HEADING_COUNT];
    let mut k = 0;
    while k < HEADING_COUNT {
        // Within its quarter: up to 45° as it is, beyond mirrored across the diagonal.
        let m = k % 8;
        let (x, y) = if m <= 4 {
            EIGHTH[m]
        } else {
            let (c, s) = EIGHTH[8 - m];
            (s, c)
        };
        // A quarter turn clockwise on screen takes (x, y) to (−y, x). `0.0 − v` keeps a zero positive.
        let (x, y) = match k / 8 {
            0 => (x, y),
            1 => (0.0 - y, x),
            2 => (0.0 - x, 0.0 - y),
            _ => (y, 0.0 - x),
        };
        headings[k] = Vec2::new(x, y);
        k += 1;
    }
    headings
};

/// A unit leaving an energy doorway, as [`EnergyDoorway::release`] places it.
#[derive(Copy, Clone, Debug, PartialEq)]
pub(crate) struct Release {
    /// Where it comes into the game: the portal's centre.
    pub pos: Vec2,
    /// The unit vector it leaves along (`Mover::exit_heading`), and faces.
    pub heading: Vec2,
}

/// A portal's energy doorway, as the portal is placed (module docs): its footprint and which of its sides are open.
/// The release, the movement step, the path serving and the invariants ask it about a unit it released.
#[derive(Copy, Clone, Debug, PartialEq)]
pub struct EnergyDoorway {
    frame: Frame,
    /// Which sides are open, facing north, east, south and west as placed (`sides::DIRECTIONS`).
    open: [bool; 4],
}

impl EnergyDoorway {
    /// The energy doorway of `building`: `None` for anything but a placed structure whose exit rule is
    /// `ExitRule::Centre` (a side doorway is `super::Doorway`), or one whose rule names none of its sides.
    pub fn of(building: &Object) -> Option<EnergyDoorway> {
        let placement = building.placement.as_ref()?;
        let ExitRule::Centre(letters) = building.def().structure.as_ref()?.production.as_ref()?.exits else {
            return None;
        };
        let open = sides(building.kind, placement.orientation())?.map(|letter| letters.contains(&letter));
        open.contains(&true).then_some(EnergyDoorway {
            frame: Frame::of(placement),
            open,
        })
    }

    /// The centre of the footprint, where its units materialize.
    pub fn centre(&self) -> Vec2 {
        (self.frame.lo + self.frame.hi) / 2.0
    }

    /// The footprint's half extents.
    fn extent(&self) -> Vec2 {
        (self.frame.hi - self.frame.lo) / 2.0
    }

    /// Which sides are open, facing north, east, south and west as placed (`sides::DIRECTIONS`).
    pub fn open_sides(&self) -> [bool; 4] {
        self.open
    }

    /// How surely the ray from the centre along `d` leaves the footprint through an open side: how far along it the
    /// ray reaches the nearest closed side's line, less how far it reaches the nearest open side's, both in units of
    /// the footprint's half extents (each side's reach is `d · outward / half extent`; the ray leaves through the side
    /// it reaches farthest). At least 0 for an open direction, 0 exactly on an end of the arc and for no direction
    /// at all; infinite for a footprint open all round.
    pub fn margin(&self, d: Vec2) -> f32 {
        let extent = self.extent();
        // North, east, south, west, as `open`.
        let reach = [-d.y / extent.y, d.x / extent.x, d.y / extent.y, -d.x / extent.x];
        let (mut open, mut closed) = (f32::NEG_INFINITY, f32::NEG_INFINITY);
        for (side, reach) in reach.into_iter().enumerate() {
            if self.open[side] {
                open = open.max(reach);
            } else {
                closed = closed.max(reach);
            }
        }
        open - closed
    }

    /// Whether the direction `d` from the centre is open (module docs, 2): the ray leaves through an open side, or
    /// through a corner one of whose sides is open ([`EnergyDoorway::margin`] at least 0).
    pub fn is_open(&self, d: Vec2) -> bool {
        self.margin(d) >= 0.0
    }

    /// The ends of the open arc (module docs, 2): the unit vectors from the centre to each corner between an open
    /// side and a closed one, open by the Director's reading. None for a footprint open all round; the Bio Portal's
    /// top-left and bottom-right corners, unturned.
    pub fn arc_ends(&self) -> impl Iterator<Item = Vec2> + '_ {
        let e = self.extent();
        // The corners between north and east, east and south, south and west, and west and north.
        let corners = [
            (0, 1, Vec2::new(e.x, -e.y)),
            (1, 2, Vec2::new(e.x, e.y)),
            (2, 3, Vec2::new(-e.x, e.y)),
            (3, 0, Vec2::new(-e.x, -e.y)),
        ];
        corners
            .into_iter()
            .filter(|&(a, b, _)| self.open[a] != self.open[b])
            .map(|(_, _, corner)| corner.normalize())
    }

    /// The heading a unit at `pos` in the doorway leaves along: `heading` (`Mover::exit_heading`, chosen when it was
    /// released); for a unit without one (a state saved before the field existed), straight away from the centre if
    /// that is open, else the first open one of the [`HEADINGS`].
    pub fn heading_of(&self, pos: Vec2, heading: Option<Vec2>) -> Vec2 {
        heading
            .or_else(|| (pos - self.centre()).try_normalize().filter(|&d| self.is_open(d)))
            .or_else(|| HEADINGS.into_iter().find(|&d| self.is_open(d)))
            .unwrap_or(Vec2::X)
    }

    /// Whether a box at `pos` with half extents `half` overlaps the footprint: a unit the portal released is still in
    /// its doorway. Touching is free, as for every box.
    pub fn holds(&self, pos: Vec2, half: Vec2) -> bool {
        self.frame.holds(pos, half)
    }

    /// The exception's rule for one move of a released unit, from `from` to `to` (module docs): out of the footprint
    /// is always allowed, as far as the footprint goes; still overlapping it, the unit must not be closer to the
    /// centre than it was. A move along a ray from the centre moves each coordinate away from the centre's, so the
    /// walk out passes this exactly, whatever the rounding.
    pub fn allows(&self, from: Vec2, to: Vec2, half: Vec2) -> bool {
        let centre = self.centre();
        !self.holds(to, half) || (to - centre).length_squared() >= (from - centre).length_squared()
    }

    /// The point straight out from `pos` along the unit vector `heading` where a box with half extents `half` is just
    /// clear of the footprint: it touches the footprint across the side it clears, that coordinate assigned exactly.
    /// `pos` itself if the box is clear already.
    pub fn outside(&self, pos: Vec2, heading: Vec2, half: Vec2) -> Vec2 {
        if !self.holds(pos, half) {
            return pos;
        }
        let (centre, reach) = (self.centre(), self.extent() + half);
        // The first axis along which the box comes clear, the parameter along the heading and the coordinate there.
        let mut first: Option<(f32, usize, f32)> = None;
        for axis in 0..2 {
            let h = heading[axis];
            if h == 0.0 {
                continue;
            }
            let edge = if h > 0.0 {
                centre[axis] + reach[axis]
            } else {
                centre[axis] - reach[axis]
            };
            let t = (edge - pos[axis]) / h;
            if first.is_none_or(|(sooner, ..)| t < sooner) {
                first = Some((t, axis, edge));
            }
        }
        let Some((t, axis, edge)) = first else {
            return pos;
        };
        let mut out = pos + heading * t;
        out[axis] = edge;
        out
    }

    /// `nav::box_clear` for a released unit: whether a box with half extents `half` centred on `p` lies inside the
    /// map and overlaps no solid tile **other than the footprint's**, nor ground of the footprint that the unit can't
    /// stand on (`Frame::blocks`).
    pub fn box_clear(&self, grid: &NavGrid, half: Vec2, p: Vec2) -> bool {
        self.frame.box_clear(grid, half, p)
    }

    /// Whether the way out along the unit vector `dir` is free for a box with half extents `half` (module docs, 3):
    /// swept straight from the centre to where it is clear of the footprint ([`EnergyDoorway::outside`]) and one unit
    /// length beyond (its longer side), it enters no solid tile of `grid` but the footprint's own, nor ground of the
    /// footprint that the unit can't stand on (`Frame::blocks`), nor the map's edge, nor any of the boxes `others`
    /// (centre and half extents). Touching is free. `grid` is `None` for a unit that crosses every tile.
    pub fn way_out_free(&self, dir: Vec2, half: Vec2, grid: Option<&NavGrid>, others: &[(Vec2, Vec2)]) -> bool {
        let centre = self.centre();
        let end = self.outside(centre, dir, half) + dir * (2.0 * half.max_element());
        let d = end - centre;
        if let Some(grid) = grid {
            let (lo, hi) = (centre.min(end) - half, centre.max(end) + half);
            let first = TilePos::containing(lo);
            let last = TilePos::new(
                ((hi.x / SU_PER_GU).ceil() as i32 - 1).max(first.x),
                ((hi.y / SU_PER_GU).ceil() as i32 - 1).max(first.y),
            );
            for y in first.y..=last.y {
                for x in first.x..=last.x {
                    let t = TilePos::new(x, y);
                    let tile = (t.corner() - half, t.corner() + Vec2::splat(SU_PER_GU) + half);
                    // Off the map is solid; under the footprint only ground the unit can't stand on is (M5-Z1).
                    if self.frame.blocks(grid, t) && segment_enters_box(centre, d, tile.0, tile.1) {
                        return false;
                    }
                }
            }
        }
        others.iter().all(|&(pos, other)| {
            let reach = half + other;
            !segment_enters_box(centre, d, pos - reach, pos + reach)
        })
    }

    /// The heading the next unit leaves in (module docs, 3 and 4): towards `rally` (the RallyPoint's position, if the
    /// portal has one) if that is open and `free`, else the nearest open direction to it that is `free`, else the
    /// nearest open one; with no rally, a random open and `free` one of the [`HEADINGS`], else a random open one.
    /// `free(dir)` is whether the way out along `dir` is free; `draw(n)` draws a uniform index below `n` from the
    /// sim's Rng, and is called only with no rally and two or more candidates. `None` only for a doorway without an
    /// open direction, which [`EnergyDoorway::of`] never makes.
    pub(crate) fn heading(
        &self,
        rally: Option<Vec2>,
        free: impl Fn(Vec2) -> bool,
        draw: impl FnOnce(u32) -> u32,
    ) -> Option<Vec2> {
        if let Some(toward) = rally.and_then(|point| (point - self.centre()).try_normalize()) {
            let open = self.is_open(toward);
            if open && free(toward) {
                return Some(toward);
            }
            let mut candidates: Vec<Vec2> = self
                .arc_ends()
                .chain(HEADINGS.into_iter().filter(|&d| self.is_open(d)))
                .collect();
            // The smallest angle from the rally's direction first (the greatest cosine); of two at the same angle,
            // the clockwise one (on screen, +y south: the greater `perp_dot`).
            candidates.sort_by(|a, b| {
                let by_angle = toward.dot(*b).total_cmp(&toward.dot(*a));
                by_angle.then(toward.perp_dot(*b).total_cmp(&toward.perp_dot(*a)))
            });
            // No way out at all: the nearest open direction all the same, the rally's own if that is open.
            let nearest = if open {
                Some(toward)
            } else {
                candidates.first().copied()
            };
            return candidates.iter().copied().find(|&d| free(d)).or(nearest);
        }
        let open: Vec<Vec2> = HEADINGS.into_iter().filter(|&d| self.is_open(d)).collect();
        let unblocked: Vec<Vec2> = open.iter().copied().filter(|&d| free(d)).collect();
        let pool = if unblocked.is_empty() { open } else { unblocked };
        match pool.len() {
            0 => None,
            1 => Some(pool[0]),
            n => pool.get(draw(n as u32) as usize).copied(),
        }
    }

    /// The release of a unit with definition `unit` and half extents `half` (module docs): at the centre, with its
    /// heading ([`EnergyDoorway::heading`], whose way out is judged on the true grid `grid` of the unit's class and
    /// against the units of its layer among `objects`). `None` if its box at the centre would overlap a unit or a
    /// blocked tile but the portal's own (it then waits, finished, and is tried again in the next tick). The caller
    /// checks that no unit the portal released is still in its doorway.
    pub(crate) fn release(
        &self,
        unit: &UnitDef,
        half: Vec2,
        grid: Option<&NavGrid>,
        objects: &BTreeMap<ObjectId, Object>,
        rally: Option<Vec2>,
        draw: impl FnOnce(u32) -> u32,
    ) -> Option<Release> {
        let centre = self.centre();
        let domain = unit.base_def().domain;
        // The units of its layer that any way out could meet: a sweep reaches at most √2 times the footprint's and
        // its own half extents beyond the centre, plus the unit length.
        let reach = Vec2::splat(SQRT_2 * (self.extent() + half).max_element() + 2.0 * half.max_element()) + half;
        let others: Vec<(Vec2, Vec2)> = if domain.hard_collision() {
            objects
                .values()
                .filter_map(|o| {
                    let def = o.def();
                    let other = def.half_extents()?;
                    let layer = def.unit.as_ref().is_some_and(|u| u.base_def().domain == domain);
                    let near = (o.pos - centre).abs().cmplt(reach + other).all();
                    (layer && near).then_some((o.pos, other))
                })
                .collect()
        } else {
            Vec::new()
        };
        let tiles_clear = grid.is_none_or(|grid| self.box_clear(grid, half, centre));
        if !tiles_clear
            || others
                .iter()
                .any(|&(pos, other)| boxes_overlap(centre, half, pos, other))
        {
            return None;
        }
        let heading = self.heading(rally, |dir| self.way_out_free(dir, half, grid, &others), draw)?;
        Some(Release { pos: centre, heading })
    }
}

#[cfg(test)]
mod tests;
