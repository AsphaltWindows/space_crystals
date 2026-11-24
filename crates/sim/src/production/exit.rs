//! The exit doorway — `core/entities.md > Structure Type > Exit Doorway` (design change 0080): where a producing
//! structure releases its units, and the one case in which a unit's box may overlap a structure.
//!
//! - **Where**: the middle of the exit side, the side lettered `ProductionDef::exit` (the Barracks' B) wherever it
//!   faces once the building is placed (`crate::sides::sides`). The unit "appears half inside the building": its
//!   centre on the exit side ([`Doorway::spot`]).
//! - **The exception** ([`Doorway::allows`], [`Doorway::box_clear`]): while a released unit (`Mover::exit` names the
//!   building) still overlaps the footprint, the footprint's tiles don't block it. It may only come out: never
//!   deeper in, and inside only within the exit side's span, so it leaves through that side and no other. Every
//!   other blocked tile blocks it as ever. Once its box is clear of the footprint the exception ends for good
//!   ("once the unit is fully out, it can't re-enter").
//! - **Occupied**: while a released unit still overlaps the footprint, the next one waits inside
//!   (`Sim::step_production`).
//!
//! **Reading of the design, to confirm** (reported to the Director): point 1 says the unit appears half inside and
//! that "nothing else can overlap a building", and the verification has a unit appear although an enemy unit, or
//! another building, stands flush against the exit side. Half inside, it would overlap that unit or building. So
//! the unit appears as far out as there is room, at most half: with something right in front of the doorway it
//! stands fully inside, against the exit side ([`Doorway::spot`] with a negative `out`).

use glam::Vec2;

use crate::nav::NavGrid;
use crate::object::Object;
use crate::paths::boxes_overlap;
use crate::sides::{DIRECTIONS, sides};
use crate::units::{SU_PER_GU, TilePos};

/// A producing structure's exit doorway, as it is placed.
#[derive(Copy, Clone, Debug, PartialEq)]
pub struct Doorway {
    /// The footprint's top-left tile and its size in tiles.
    origin: TilePos,
    size: (i32, i32),
    /// The footprint's corners, space units.
    lo: Vec2,
    hi: Vec2,
    /// The axis the exit side faces along (0 is x, 1 is y) and which way: 1 towards +x or +y, −1 the other.
    axis: usize,
    sign: f32,
}

impl Doorway {
    /// The doorway of `building`: `None` for anything but a placed structure that releases units.
    pub fn of(building: &Object) -> Option<Doorway> {
        let placement = building.placement.as_ref()?;
        let exit = building.def().structure.as_ref()?.production.as_ref()?.exit;
        let facing = sides(building.kind, placement.orientation())?
            .iter()
            .position(|&letter| letter == exit)?;
        let (dx, dy) = DIRECTIONS[facing];
        let lo = placement.origin.corner();
        Some(Doorway {
            origin: placement.origin,
            size: (placement.w as i32, placement.h as i32),
            lo,
            hi: lo + Vec2::new(placement.w as f32, placement.h as f32) * SU_PER_GU,
            axis: usize::from(dx == 0),
            sign: (dx + dy) as f32,
        })
    }

    /// The unit vector pointing out of the exit side.
    pub fn outward(&self) -> Vec2 {
        let mut out = Vec2::ZERO;
        out[self.axis] = self.sign;
        out
    }

    /// The exit side's coordinate on the doorway's axis.
    fn edge(&self) -> f32 {
        if self.sign > 0.0 {
            self.hi[self.axis]
        } else {
            self.lo[self.axis]
        }
    }

    /// The middle of the exit side.
    pub fn middle(&self) -> Vec2 {
        let mut middle = (self.lo + self.hi) / 2.0;
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
        boxes_overlap(pos, half, (self.lo + self.hi) / 2.0, (self.hi - self.lo) / 2.0)
    }

    /// How far out ([`Doorway::spot`]'s `out`) a box with half extents `half` can stand in the doorway before it
    /// meets the box at `other`: `None` if that box isn't in front of the doorway at all, or leaves room for the
    /// unit to stand half inside.
    pub fn room_before(&self, half: Vec2, other: Vec2, other_half: Vec2) -> Option<f32> {
        let across = 1 - self.axis;
        if (other[across] - self.middle()[across]).abs() >= half[across] + other_half[across] {
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
        let (lo, hi) = (self.lo[across] + half[across], self.hi[across] - half[across]);
        let mut ways = vec![self.outside(pos, half)];
        let tiles = if self.axis == 0 { self.size.1 } else { self.size.0 };
        for tile in 0..tiles {
            let mut way = ways[0];
            let centre = self.lo[across] + (tile as f32 + 0.5) * SU_PER_GU;
            way[across] = if lo <= hi { centre.clamp(lo, hi) } else { centre };
            ways.push(way);
        }
        ways
    }

    /// Whether tile `t` is one of the footprint's.
    fn covers(&self, t: TilePos) -> bool {
        (self.origin.x..self.origin.x + self.size.0).contains(&t.x)
            && (self.origin.y..self.origin.y + self.size.1).contains(&t.y)
    }

    /// The exception's rule for one move of a released unit, from `from` to `to`: out of the footprint is always
    /// allowed, as far as the footprint goes. Still overlapping it, the unit must not be deeper in than it was, and
    /// must stay within the exit side's span (or not move across it at all), so the only way it leaves is through
    /// the exit side.
    pub fn allows(&self, from: Vec2, to: Vec2, half: Vec2) -> bool {
        if !self.holds(to, half) {
            return true;
        }
        let across = 1 - self.axis;
        let within = to[across] - half[across] >= self.lo[across] && to[across] + half[across] <= self.hi[across];
        (within || to[across] == from[across]) && self.out_of(to) >= self.out_of(from)
    }

    /// `nav::box_clear` for a released unit: whether a box with half extents `half` centred on `p` lies inside the
    /// map and overlaps no solid tile **other than the footprint's**.
    pub fn box_clear(&self, grid: &NavGrid, half: Vec2, p: Vec2) -> bool {
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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::defs::ObjectKind;
    use crate::object::Rotation;
    use crate::scenario::{BuildingOrientation, SpawnAt};
    use crate::sim::Sim;
    use crate::testing::ScenarioBuilder;

    const HALF: Vec2 = Vec2::splat(12.0);

    fn barracks(rotation: Rotation, flip_h: bool) -> Object {
        let mut scenario = ScenarioBuilder::new("........\n........\n........\n........\n........\n........").build();
        let mut spawn = crate::scenario::Spawn::new(ObjectKind::Barracks, None, SpawnAt::Tile(TilePos::new(2, 1)));
        spawn.orientation = BuildingOrientation {
            rotation,
            flip_h,
            flip_v: false,
        };
        scenario.map.spawns.push(spawn);
        Sim::new(&scenario).objects().next().unwrap().clone()
    }

    /// Design change 0079 and `> Exit Doorway`: the doorway is in the middle of side B, wherever that side faces.
    #[test]
    fn the_doorway_is_the_middle_of_side_b_in_all_eight_orientations() {
        // Unturned: 3 wide and 2 tall at tile (2, 1), so x 128..320 and y 64..192; B faces east.
        let cases = [
            (Rotation::R0, false, Vec2::new(320.0, 128.0), Vec2::X),
            (Rotation::R180, false, Vec2::new(128.0, 128.0), -Vec2::X),
            (Rotation::R0, true, Vec2::new(128.0, 128.0), -Vec2::X),
            (Rotation::R180, true, Vec2::new(320.0, 128.0), Vec2::X),
            // Turned on its side: 2 wide and 3 tall, so x 128..256 and y 64..256.
            (Rotation::R90, false, Vec2::new(192.0, 256.0), Vec2::Y),
            (Rotation::R270, false, Vec2::new(192.0, 64.0), -Vec2::Y),
            (Rotation::R90, true, Vec2::new(192.0, 64.0), -Vec2::Y),
            (Rotation::R270, true, Vec2::new(192.0, 256.0), Vec2::Y),
        ];
        for (rotation, flip_h, middle, outward) in cases {
            let door = Doorway::of(&barracks(rotation, flip_h)).unwrap();
            assert_eq!(
                (door.middle(), door.outward()),
                (middle, outward),
                "{rotation:?} {flip_h}"
            );
            assert_eq!(door.spot(0.0), middle);
            assert!(door.holds(door.spot(0.0), HALF), "half inside");
            assert!(door.holds(door.spot(-12.0), HALF), "fully inside");
            assert!(!door.holds(door.spot(12.0), HALF), "touching it from outside");
            assert_eq!(door.outside(door.spot(-5.0), HALF), door.spot(12.0));
            assert_eq!((door.out_of(door.spot(-5.0)), door.out_of(door.spot(7.0))), (-5.0, 7.0));
        }
    }

    #[test]
    fn only_structures_that_produce_have_a_doorway() {
        let sim = ScenarioBuilder::new("....\n....\n....")
            .spawn(ObjectKind::Generator, None, SpawnAt::Tile(TilePos::new(0, 0)))
            .spawn(ObjectKind::Peacekeeper, None, SpawnAt::Point(Vec2::new(200.0, 160.0)))
            .start();
        assert!(sim.sim().objects().all(|o| Doorway::of(o).is_none()));
    }

    /// The exception's rule: out, never deeper in, and inside only within the exit side's span.
    #[test]
    fn a_unit_in_the_doorway_may_only_come_out() {
        let door = Doorway::of(&barracks(Rotation::R0, false)).unwrap();
        let at = door.spot(-4.0);
        assert!(door.allows(at, at + Vec2::new(3.0, 0.0), HALF), "out");
        assert!(door.allows(at, at + Vec2::new(40.0, 30.0), HALF), "clear of it");
        assert!(!door.allows(at, at + Vec2::new(-0.5, 0.0), HALF), "deeper in");
        assert!(
            door.allows(at, at + Vec2::new(0.0, 5.0), HALF),
            "along the side, within it"
        );
        assert!(door.allows(at, at + Vec2::new(2.0, -5.0), HALF));
        // The side spans y 64..192: a box 12 high may not reach past its ends while inside.
        let corner = Vec2::new(at.x, 64.0 + 12.0);
        assert!(
            !door.allows(corner, corner + Vec2::new(0.0, -1.0), HALF),
            "out through side A"
        );
        assert!(door.allows(corner, corner + Vec2::new(1.0, 0.0), HALF));
        // A box wider than the side may still go straight out.
        let wide = Vec2::new(12.0, 80.0);
        assert!(door.allows(at, at + Vec2::new(3.0, 0.0), wide) && !door.allows(at, at + Vec2::new(3.0, 1.0), wide));
    }

    /// The ways out: straight ahead first, then in front of each tile of the exit side.
    #[test]
    fn the_ways_out_touch_the_exit_side_from_outside() {
        let door = Doorway::of(&barracks(Rotation::R0, false)).unwrap();
        let ways = door.ways_out(door.spot(-12.0) + Vec2::new(0.0, 7.0), HALF);
        assert_eq!(
            ways,
            [Vec2::new(332.0, 135.0), Vec2::new(332.0, 96.0), Vec2::new(332.0, 160.0)]
        );
        assert!(ways.iter().all(|&way| !door.holds(way, HALF)));
        let south = Doorway::of(&barracks(Rotation::R90, false)).unwrap();
        assert_eq!(
            south.ways_out(south.spot(0.0), HALF),
            [
                Vec2::new(192.0, 268.0),
                Vec2::new(160.0, 268.0),
                Vec2::new(224.0, 268.0)
            ]
        );
    }
}
