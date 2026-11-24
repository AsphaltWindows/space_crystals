//! The exit doorways — `core/entities.md > Structure Type > Exit Doorway` (design change 0080): where a producing
//! structure releases its units, and the one case in which a unit's box may overlap a structure.
//!
//! - **Where**: the middle of an exit side ([`Door`]), a side lettered as `ProductionDef::exits` says (the Barracks'
//!   B, the portals' A sides; plan decision T19) wherever it faces once the building is placed
//!   (`crate::sides::sides`). A [`Doorway`] is all of a building's exit sides. The unit "appears half inside the
//!   building": its centre on the exit side ([`Door::spot`]).
//! - **Which door** (`choice.rs`): with one exit side, that one. With several ("Buildings with several
//!   exit-eligible sides ... choose the side by their own rules"), the Bio Portal's Exit Behavior: the side closest to
//!   the RallyPoint, else a random unblocked one, drawn from the sim's Rng.
//! - **The exception** ([`Doorway::allows`], [`Doorway::box_clear`]): while a released unit (`Mover::exit` names the
//!   building) still overlaps the footprint, the footprint's tiles don't block it. It may only come out: never
//!   deeper in, and inside only within its door's span, so it leaves through a door and no other side. Every other
//!   blocked tile blocks it as ever. Once its box is clear of the footprint the exception ends for good ("once the
//!   unit is fully out, it can't re-enter").
//! - **The door a unit is in is judged by geometry** (plan decision T19: `Mover::exit` stays the building's id):
//!   [`Doorway::door_at`], the exit side its centre lies least deep behind. A unit released at the middle of a side is
//!   in that side's door, and only ever comes out of it: the one move that could put it in another door while it
//!   still overlaps the footprint, deep in a corner between two exit sides, is refused ([`Doorway::allows`]).
//! - **Occupied**: a door is occupied while a unit the building released is in it; a unit waits inside while every
//!   door is (`Sim::step_production`).
//!
//! **Reading of the design, to confirm** (reported to the Director): point 1 says the unit appears half inside and
//! that "nothing else can overlap a building", and the verification has a unit appear although an enemy unit, or
//! another building, stands flush against the exit side. Half inside, it would overlap that unit or building. So
//! the unit appears as far out as there is room, at most half: with something right in front of the doorway it
//! stands fully inside, against the exit side ([`Door::spot`] with a negative `out`).

mod choice;

use glam::Vec2;

use crate::nav::NavGrid;
use crate::object::Object;
use crate::paths::boxes_overlap;
use crate::sides::{DIRECTIONS, sides};
use crate::units::{SU_PER_GU, TilePos};

pub(crate) use choice::{DoorState, choose};

/// The footprint of a producing structure, as it is placed.
#[derive(Copy, Clone, Debug, PartialEq)]
struct Frame {
    /// The top-left tile and the size in tiles.
    origin: TilePos,
    size: (i32, i32),
    /// The corners, space units.
    lo: Vec2,
    hi: Vec2,
}

impl Frame {
    /// Whether a box at `pos` with half extents `half` overlaps the footprint. Touching is free, as for every box.
    fn holds(&self, pos: Vec2, half: Vec2) -> bool {
        boxes_overlap(pos, half, (self.lo + self.hi) / 2.0, (self.hi - self.lo) / 2.0)
    }

    /// Whether tile `t` is one of the footprint's.
    fn covers(&self, t: TilePos) -> bool {
        (self.origin.x..self.origin.x + self.size.0).contains(&t.x)
            && (self.origin.y..self.origin.y + self.size.1).contains(&t.y)
    }

    /// `nav::box_clear` for a released unit: whether a box with half extents `half` centred on `p` lies inside the
    /// map and overlaps no solid tile **other than the footprint's**.
    fn box_clear(&self, grid: &NavGrid, half: Vec2, p: Vec2) -> bool {
        if !p.is_finite() {
            return false;
        }
        let (lo, hi) = (p - half, p + half);
        // The tiles the open box overlaps: touching a border isn't overlapping.
        let first = TilePos::containing(lo);
        let last = TilePos::new(
            ((hi.x / SU_PER_GU).ceil() as i32 - 1).max(first.x),
            ((hi.y / SU_PER_GU).ceil() as i32 - 1).max(first.y),
        );
        if !grid.in_bounds(first) || !grid.in_bounds(last) {
            return false;
        }
        (first.y..=last.y).all(|y| {
            (first.x..=last.x).all(|x| {
                let t = TilePos::new(x, y);
                self.covers(t) || !grid.is_solid(t)
            })
        })
    }
}

/// A producing structure's exit doorways, as it is placed: its footprint and the sides its units leave through. The
/// movement step, the path serving and the invariants ask it about a released unit, whose door it judges by where
/// the unit is ([`Doorway::door_at`]).
#[derive(Copy, Clone, Debug, PartialEq)]
pub struct Doorway {
    frame: Frame,
    /// Which sides are exits, facing north, east, south and west (`sides::DIRECTIONS`).
    exits: [bool; 4],
}

/// One exit side of a producing structure, as it is placed: the doorway in its middle.
#[derive(Copy, Clone, Debug, PartialEq)]
pub struct Door {
    frame: Frame,
    /// The axis the exit side faces along (0 is x, 1 is y) and which way: 1 towards +x or +y, −1 the other.
    axis: usize,
    sign: f32,
}

impl Doorway {
    /// The doorways of `building`: `None` for anything but a placed structure that releases units.
    pub fn of(building: &Object) -> Option<Doorway> {
        let placement = building.placement.as_ref()?;
        let rule = building.def().structure.as_ref()?.production.as_ref()?.exits;
        let exits = sides(building.kind, placement.orientation())?.map(|letter| rule.includes(letter));
        let lo = placement.origin.corner();
        let frame = Frame {
            origin: placement.origin,
            size: (placement.w as i32, placement.h as i32),
            lo,
            hi: lo + Vec2::new(placement.w as f32, placement.h as f32) * SU_PER_GU,
        };
        exits.contains(&true).then_some(Doorway { frame, exits })
    }

    /// The doors, one per exit side, in the order of the sides facing north, east, south and west: "top, right,
    /// bottom, left" as placed (question Q0041 point 4's order for ties).
    pub fn doors(&self) -> impl Iterator<Item = Door> + '_ {
        (0..4).filter(|&facing| self.exits[facing]).map(|facing| {
            let (dx, dy) = DIRECTIONS[facing];
            Door {
                frame: self.frame,
                axis: usize::from(dx == 0),
                sign: (dx + dy) as f32,
            }
        })
    }

    /// The door a unit centred on `pos` is in: of the exit sides, the one its centre lies least deep behind (or
    /// farthest out of), the first in [`Doorway::doors`]' order of those that tie. A unit released in a door is in it
    /// (its centre lies on that side's middle, the other sides farther away than it is tall or wide) and stays in it:
    /// it only comes out, and within the side's span ([`Door::allows`]), so the side it came in by stays the
    /// nearest, unless it slides to the very end of the span from deeper than fully inside, where
    /// [`Doorway::allows`] stops it.
    pub fn door_at(&self, pos: Vec2) -> Door {
        let door = self.doors().min_by(|a, b| b.out_of(pos).total_cmp(&a.out_of(pos)));
        door.expect("a doorway has a door")
    }

    /// Whether a box at `pos` with half extents `half` overlaps the footprint: a unit the building released is still
    /// in its doorway. Touching is free, as for every box.
    pub fn holds(&self, pos: Vec2, half: Vec2) -> bool {
        self.frame.holds(pos, half)
    }

    /// [`Door::allows`], for the door the unit is in where it comes from, which it may not leave for another while
    /// it overlaps the footprint: a move only ever takes a unit out of its door, never into the next one (that could
    /// happen only deep in a corner between two exit sides: see [`Doorway::door_at`]). Moves that this allows one
    /// after the other, it allows as one.
    pub fn allows(&self, from: Vec2, to: Vec2, half: Vec2) -> bool {
        let door = self.door_at(from);
        door.allows(from, to, half) && (!self.holds(to, half) || self.door_at(to) == door)
    }

    /// [`Door::outside`], for the door the unit at `pos` is in.
    pub fn outside(&self, pos: Vec2, half: Vec2) -> Vec2 {
        self.door_at(pos).outside(pos, half)
    }

    /// [`Door::ways_out`], for the door the unit at `pos` is in.
    pub fn ways_out(&self, pos: Vec2, half: Vec2) -> Vec<Vec2> {
        self.door_at(pos).ways_out(pos, half)
    }

    /// `nav::box_clear` for a released unit: whether a box with half extents `half` centred on `p` lies inside the
    /// map and overlaps no solid tile **other than the footprint's**.
    pub fn box_clear(&self, grid: &NavGrid, half: Vec2, p: Vec2) -> bool {
        self.frame.box_clear(grid, half, p)
    }
}

impl Door {
    /// The unit vector pointing out of the exit side.
    pub fn outward(&self) -> Vec2 {
        let mut out = Vec2::ZERO;
        out[self.axis] = self.sign;
        out
    }

    /// The exit side's coordinate on the door's axis.
    fn edge(&self) -> f32 {
        if self.sign > 0.0 {
            self.frame.hi[self.axis]
        } else {
            self.frame.lo[self.axis]
        }
    }

    /// The middle of the exit side.
    pub fn middle(&self) -> Vec2 {
        let mut middle = (self.frame.lo + self.frame.hi) / 2.0;
        middle[self.axis] = self.edge();
        middle
    }

    /// The point in the doorway whose centre lies `out` space units outside the exit side: 0 is "half inside the
    /// building, at the middle of the exit side"; minus the unit's half extent is fully inside, against the side.
    pub fn spot(&self, out: f32) -> Vec2 {
        self.middle() + self.outward() * out
    }

    /// How far outside the exit side the centre of a box at `pos` lies (negative inside).
    pub fn out_of(&self, pos: Vec2) -> f32 {
        self.sign * (pos[self.axis] - self.edge())
    }

    /// Whether a box at `pos` with half extents `half` overlaps the footprint: the unit is still in the doorway.
    /// Touching is free, as for every box.
    pub fn holds(&self, pos: Vec2, half: Vec2) -> bool {
        self.frame.holds(pos, half)
    }

    /// How far out ([`Door::spot`]'s `out`) a box with half extents `half` can stand in the doorway before it meets
    /// the box at `other`: `None` if that box isn't in front of the doorway at all, or leaves room for the unit to
    /// stand half inside. A box behind the doorway, as far in as the unit standing fully inside or farther (in
    /// another door, or beyond the building's other sides), is not in front of it.
    pub fn room_before(&self, half: Vec2, other: Vec2, other_half: Vec2) -> Option<f32> {
        let across = 1 - self.axis;
        if (other[across] - self.middle()[across]).abs() >= half[across] + other_half[across] {
            return None;
        }
        if self.out_of(other) + other_half[self.axis] <= -2.0 * half[self.axis] {
            return None;
        }
        // From the exit side to the other box's near face, less the unit's own half.
        let room = self.out_of(other) - other_half[self.axis] - half[self.axis];
        (room < 0.0).then_some(room)
    }

    /// The point straight out of the doorway from `pos` where a box with half extents `half` is just clear of the
    /// footprint, touching the exit side.
    pub fn outside(&self, pos: Vec2, half: Vec2) -> Vec2 {
        let mut out = pos;
        out[self.axis] = self.edge() + self.sign * half[self.axis];
        out
    }

    /// Where a unit in the doorway at `pos` may start its path from, in the order to try them: straight out, then
    /// in front of each tile of the exit side, from the low end of the side to the high one (a unit can come out
    /// beside a tile that something blocks). Each is a point touching the exit side from outside.
    pub fn ways_out(&self, pos: Vec2, half: Vec2) -> Vec<Vec2> {
        let across = 1 - self.axis;
        let (lo, hi) = (
            self.frame.lo[across] + half[across],
            self.frame.hi[across] - half[across],
        );
        let mut ways = vec![self.outside(pos, half)];
        let tiles = if self.axis == 0 {
            self.frame.size.1
        } else {
            self.frame.size.0
        };
        for tile in 0..tiles {
            let mut way = ways[0];
            let centre = self.frame.lo[across] + (tile as f32 + 0.5) * SU_PER_GU;
            way[across] = if lo <= hi { centre.clamp(lo, hi) } else { centre };
            ways.push(way);
        }
        ways
    }

    /// The exception's rule for one move of a released unit in this door, from `from` to `to`: out of the
    /// footprint is always allowed, as far as the footprint goes. Still overlapping it, the unit must not be deeper
    /// in than it was, and must stay within the exit side's span (or not move across it at all), so the only way it
    /// leaves is through the exit side.
    pub fn allows(&self, from: Vec2, to: Vec2, half: Vec2) -> bool {
        if !self.holds(to, half) {
            return true;
        }
        let across = 1 - self.axis;
        let (lo, hi) = (self.frame.lo, self.frame.hi);
        let within = to[across] - half[across] >= lo[across] && to[across] + half[across] <= hi[across];
        (within || to[across] == from[across]) && self.out_of(to) >= self.out_of(from)
    }

    /// Whether the door has a way out for a box with half extents `half`: one of its [`Door::ways_out`] from the
    /// middle is clear of the solid tiles of `grid` (`nav::box_clear`). A side that a building, a cliff or the
    /// map's edge closes off along its whole length has none: the design's "obstructed, not currently traversable"
    /// (`factions/colonists/structures.md > Bio Portal > Exit Behavior`). Units don't close it: they step aside or
    /// are shot (design change 0080).
    pub fn has_way_out(&self, grid: &NavGrid, half: Vec2) -> bool {
        let ways = self.ways_out(self.spot(0.0), half);
        ways.into_iter().any(|way| crate::nav::box_clear(grid, half, way))
    }

    /// `nav::box_clear` for a released unit: whether a box with half extents `half` centred on `p` lies inside the
    /// map and overlaps no solid tile **other than the footprint's**.
    pub fn box_clear(&self, grid: &NavGrid, half: Vec2, p: Vec2) -> bool {
        self.frame.box_clear(grid, half, p)
    }
}

#[cfg(test)]
mod tests;
