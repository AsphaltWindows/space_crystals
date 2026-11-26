//! The exit doorways — `core/entities.md > Structure Type > Exit Doorway` (design change 0080) and the portals' energy
//! doorway (design change 0145, `factions/colonists/structures/README.md > Portals > Portal Exits (energy doorway)`):
//! where a producing structure releases its units and a building that holds units lets them out (slice M5-F1), and
//! the one case in which a unit's box may overlap a structure.
//!
//! A building's exit rule ([`exit_rule`]: `ProductionDef::exits`, or `HoldsDef::exit` for a building that holds units
//! and produces nothing) gives it one of two kinds of doorway ([`ExitDoor::of`]):
//! - **A side doorway** ([`Doorway`], `ExitRule::Side`, plan decision T19): the middle of its exit side ([`Door`]),
//!   the side lettered as the rule says (the Barracks' B) wherever it faces once the building is placed
//!   (`crate::sides::sides`). The unit "appears half inside the building": its centre on the exit side
//!   ([`Door::spot`]). A building with several exit sides (`ExitRule::Sides`, slice M5-C's; no type since design
//!   change 0342 gave the Recruitment Hall one exit side, B) has a door in the middle of each, and releases each unit
//!   through the first free one in its order ([`Doorway::order`]: the one facing the rally point, the bottom one
//!   without a rally point, then clockwise; **Placeholder (not in the design: no building has several exit sides
//!   now)**). A unit in the doorway is in the door whose side it is nearest
//!   ([`Doorway::door_at`]), and leaves through that side alone.
//! - **An energy doorway** ([`EnergyDoorway`], `ExitRule::Centre`, `energy.rs`): the unit materializes at the centre
//!   and moves straight out along a heading that leaves through an open side.
//!
//! - **The exception** ([`ExitDoor::allows`], [`ExitDoor::box_clear`]): while a released unit (`Mover::exit` names the
//!   building) still overlaps the footprint, the footprint's tiles don't block it. It may only come out: never deeper
//!   in (out of a side doorway: inside only within the exit side's span, so it leaves through that side and no other;
//!   out of an energy doorway: never closer to the centre). Every other blocked tile blocks it as ever. Once its box
//!   is clear of the footprint the exception ends for good ("once the unit is fully out, it can't re-enter").
//! - **Occupied**: a doorway is occupied while a unit the building released is still in it; the next finished unit
//!   waits inside meanwhile (`Sim::release_unit`), and so does the next unit Eject All lets out (`Sim::release_held`).
//!   With several exit sides, a door is occupied while such a unit is in it, and the building waits only while every
//!   door is (`core/entities.md > Structure Type > Exit Doorway`: "a side whose doorway is occupied counts as blocked,
//!   and the building pauses only when every eligible doorway is occupied"); at most one unit is in each door.
//!
//! **Reading of the design, to confirm** (reported to the Director): point 1 says the unit appears half inside and
//! that "nothing else can overlap a building", and the verification has a unit appear although an enemy unit, or
//! another building, stands flush against the exit side. Half inside, it would overlap that unit or building. So
//! the unit appears as far out as there is room, at most half: with something right in front of the doorway it
//! stands fully inside, against the exit side ([`Door::spot`] with a negative `out`).

mod energy;

use glam::Vec2;

use crate::defs::{ExitRule, StructureDef};
use crate::nav::NavGrid;
use crate::object::{Object, Placement};
use crate::paths::boxes_overlap;
use crate::sides::{DIRECTIONS, sides};
use crate::units::{SU_PER_GU, TilePos};

pub use energy::{EnergyDoorway, HEADING_COUNT, HEADINGS};

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
    fn of(placement: &Placement) -> Frame {
        let lo = placement.origin.corner();
        Frame {
            origin: placement.origin,
            size: (placement.w as i32, placement.h as i32),
            lo,
            hi: lo + Vec2::new(placement.w as f32, placement.h as f32) * SU_PER_GU,
        }
    }

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

/// The doorway a producing structure releases its units into, of either kind (module docs): what the release, the
/// movement step, the path serving and the invariants ask about a unit in it (`Mover::exit`).
#[derive(Copy, Clone, Debug, PartialEq)]
pub enum ExitDoor {
    /// The doorway in the middle of an exit side (`ExitRule::Side`, the Barracks', design change 0080).
    Side(Doorway),
    /// The energy doorway at the centre (`ExitRule::Centre`, the portals', design change 0145).
    Centre(EnergyDoorway),
}

impl ExitDoor {
    /// The doorway of `building`: `None` for anything but a placed structure that releases units.
    pub fn of(building: &Object) -> Option<ExitDoor> {
        Doorway::of(building)
            .map(ExitDoor::Side)
            .or_else(|| EnergyDoorway::of(building).map(ExitDoor::Centre))
    }

    /// Whether a box at `pos` with half extents `half` overlaps the footprint: a unit the building released is still
    /// in its doorway. Touching is free, as for every box.
    pub fn holds(&self, pos: Vec2, half: Vec2) -> bool {
        match self {
            ExitDoor::Side(doorway) => doorway.holds(pos, half),
            ExitDoor::Centre(portal) => portal.holds(pos, half),
        }
    }

    /// The exception's rule for one move of a released unit in the doorway, from `from` to `to` (module docs):
    /// [`Doorway::allows`], [`EnergyDoorway::allows`].
    pub fn allows(&self, from: Vec2, to: Vec2, half: Vec2) -> bool {
        match self {
            ExitDoor::Side(doorway) => doorway.allows(from, to, half),
            ExitDoor::Centre(portal) => portal.allows(from, to, half),
        }
    }

    /// `nav::box_clear` for a released unit: whether a box with half extents `half` centred on `p` lies inside the
    /// map and overlaps no solid tile **other than the footprint's**.
    pub fn box_clear(&self, grid: &NavGrid, half: Vec2, p: Vec2) -> bool {
        match self {
            ExitDoor::Side(doorway) => doorway.box_clear(grid, half, p),
            ExitDoor::Centre(portal) => portal.box_clear(grid, half, p),
        }
    }
}

/// A producing structure's exit doorway on its exit side, as it is placed (`ExitRule::Side`): its footprint and the
/// side its units leave through, the one its SymmetryType letters so (`defs_are_consistent`); or on each of its exit
/// sides (`ExitRule::Sides`, module docs). The movement step, the path serving and the invariants ask it about a
/// released unit, which is in the door whose side it is nearest ([`Doorway::door_at`]).
#[derive(Copy, Clone, Debug, PartialEq)]
pub struct Doorway {
    /// One door per exit side, by the direction it faces, north, east, south and west (`crate::sides::DIRECTIONS`'
    /// order, clockwise on screen); at least one.
    doors: [Option<Door>; 4],
}

/// The exit side of a producing structure, as it is placed: the doorway in its middle.
#[derive(Copy, Clone, Debug, PartialEq)]
pub struct Door {
    frame: Frame,
    /// The axis the exit side faces along (0 is x, 1 is y) and which way: 1 towards +x or +y, −1 the other.
    axis: usize,
    sign: f32,
}

/// How a `structure` type releases units: by its production's exit rule (`ProductionDef::exits`), or, if it produces
/// none, by the exit of the units it holds (`HoldsDef::exit`, slice M5-F1: an Armory's Eject All, "Trained units and
/// ejected Recruits exit through the exit side, one at a time through the exit doorway",
/// `factions/cults/structures/armory.md > ExitSide - opposite short side (C)`). The two agree for a type with both
/// (`defs_are_consistent`). `None` for a type that releases none.
pub fn exit_rule(structure: &StructureDef) -> Option<ExitRule> {
    match &structure.production {
        Some(production) => Some(production.exits),
        None => structure.holds.map(|holds| holds.exit),
    }
}

impl Doorway {
    /// The doorway of `building`: `None` for anything but a placed structure that releases units through exit sides
    /// ([`exit_rule`]: a producer's, or a building's that holds units; a portal's energy doorway is [`EnergyDoorway`]).
    pub fn of(building: &Object) -> Option<Doorway> {
        let placement = building.placement.as_ref()?;
        let (ExitRule::Side(exit) | ExitRule::Sides(exit)) = exit_rule(building.def().structure.as_ref()?)? else {
            return None;
        };
        let letters = sides(building.kind, placement.orientation())?;
        let mut doors = [None; 4];
        for ((door, letter), (dx, dy)) in doors.iter_mut().zip(letters).zip(DIRECTIONS) {
            *door = (letter == exit).then(|| Door {
                frame: Frame::of(placement),
                axis: usize::from(dx == 0),
                sign: (dx + dy) as f32,
            });
        }
        doors.iter().any(Option::is_some).then_some(Doorway { doors })
    }

    /// The doors, one per exit side, clockwise from the one facing north.
    pub fn doors(&self) -> impl Iterator<Item = Door> + '_ {
        self.doors.iter().flatten().copied()
    }

    /// The doors in the order a unit tries them (`Sim::release_unit`), the first free one taking it: the one facing
    /// `toward`, the direction from the building's centre to its rally point, then the next ones clockwise (slice
    /// M5-C's rule for the Hall's four sides before design change 0342; **Placeholder (not in the design: no building
    /// has several exit sides now)**). With no direction (no rally point, or one right at the centre) the first is the one
    /// facing south, the bottom side. The door facing `toward` is the one at the smallest angle from it, and of two at
    /// the same angle, a rally point exactly on a diagonal, the clockwise one. With one exit side, its door alone.
    pub fn order(&self, toward: Option<Vec2>) -> impl Iterator<Item = Door> + '_ {
        let toward = toward.filter(|t| *t != Vec2::ZERO).unwrap_or(Vec2::Y);
        // The doors by the direction they face, the first being the one facing `toward`: the greatest cosine, then the
        // greatest `perp_dot` (clockwise on screen, +y south), as the energy doorway's nearest direction.
        let first = (0..4)
            .filter(|&i| self.doors[i].is_some())
            .max_by(|&a, &b| {
                let (da, db) = (Self::facing(a), Self::facing(b));
                (toward.dot(da).total_cmp(&toward.dot(db))).then(toward.perp_dot(da).total_cmp(&toward.perp_dot(db)))
            })
            .unwrap_or(0);
        (0..4).filter_map(move |k| self.doors[(first + k) % 4])
    }

    /// The direction the side of `DIRECTIONS`' index `i` faces.
    fn facing(i: usize) -> Vec2 {
        let (dx, dy) = DIRECTIONS[i];
        Vec2::new(dx as f32, dy as f32)
    }

    /// The door a unit centred on `pos` is in: the one whose side it is least inside of, the first clockwise from north
    /// of two at the same depth. With one exit side, its door, wherever the unit stands.
    pub fn door_at(&self, pos: Vec2) -> Door {
        let mut doors = self.doors();
        let first = doors.next().expect("a doorway has a door");
        doors.fold(first, |best, door| {
            if door.out_of(pos) > best.out_of(pos) {
                door
            } else {
                best
            }
        })
    }

    /// Whether a box at `pos` with half extents `half` overlaps the footprint: a unit the building released is still
    /// in its doorway. Touching is free, as for every box.
    pub fn holds(&self, pos: Vec2, half: Vec2) -> bool {
        self.door_at(pos).holds(pos, half)
    }

    /// [`Door::allows`], for the door the unit is in at `from`.
    pub fn allows(&self, from: Vec2, to: Vec2, half: Vec2) -> bool {
        self.door_at(from).allows(from, to, half)
    }

    /// [`Door::outside`], for the door the unit is in.
    pub fn outside(&self, pos: Vec2, half: Vec2) -> Vec2 {
        self.door_at(pos).outside(pos, half)
    }

    /// [`Door::ways_out`], for the door the unit is in.
    pub fn ways_out(&self, pos: Vec2, half: Vec2) -> Vec<Vec2> {
        self.door_at(pos).ways_out(pos, half)
    }

    /// `nav::box_clear` for a released unit: whether a box with half extents `half` centred on `p` lies inside the
    /// map and overlaps no solid tile **other than the footprint's**.
    pub fn box_clear(&self, grid: &NavGrid, half: Vec2, p: Vec2) -> bool {
        self.door_at(p).box_clear(grid, half, p)
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
    /// stand half inside. A box behind the doorway, as far in as the unit standing fully inside or farther (beyond
    /// the building's other sides), is not in front of it.
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

    /// `nav::box_clear` for a released unit: whether a box with half extents `half` centred on `p` lies inside the
    /// map and overlaps no solid tile **other than the footprint's**.
    pub fn box_clear(&self, grid: &NavGrid, half: Vec2, p: Vec2) -> bool {
        self.frame.box_clear(grid, half, p)
    }
}

#[cfg(test)]
mod tests;
