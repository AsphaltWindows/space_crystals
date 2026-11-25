//! The ElevationModifier — `core/vision.md > ElevationModifier`: relative elevation adds one grid unit to, or takes
//! one from, sight range and attack range.
//!
//! Shared by combat (attack range, `combat`) and vision (sight range, slice V). An object's elevation is its
//! elevation level, the terrain height at its position rounded down (`core/map.md > Elevation > Elevation level`):
//! units and structures alike count, as sources and as targets, while air units are exempt both ways. Ranges are in
//! grid units, as the design gives them; callers convert with `units::gu`.

use std::cmp::Ordering;

use glam::Vec2;

use crate::defs::{Domain, ObjectKind};
use crate::map::Map;
use crate::object::Object;

/// `core/vision.md > ElevationModifier > Higher ground`, `> Lower ground`: one grid unit more, or less.
pub const ELEVATION_MODIFIER: f32 = 1.0;

/// Where an object stands for the ElevationModifier.
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum Elevation {
    /// A ground or underground object, unit or structure, at this elevation level. Underground units use the
    /// terrain above them (`> Underground units`), which is the terrain at their position.
    Level(u8),
    /// An air unit (`> Air exempt`): it neither gains nor suffers the modifier, as source or as target.
    Exempt,
}

impl Elevation {
    /// An object of `kind` at `pos`: exempt if it is an air unit, else at the elevation level there. For a surface
    /// structure that is the level of its flat footprint (`> Elevation used`); for an underground one, which may
    /// stand on slopes (design change 0005), the terrain under its centre (plan decision T32).
    pub fn of(map: &Map, kind: ObjectKind, pos: Vec2) -> Elevation {
        let air = kind
            .def()
            .unit
            .as_ref()
            .is_some_and(|unit| unit.base_def().domain == Domain::Air);
        if air {
            Elevation::Exempt
        } else {
            Elevation::ground(map, pos)
        }
    }

    /// [`Elevation::of`] an object where it stands.
    pub fn of_object(map: &Map, o: &Object) -> Elevation {
        Elevation::of(map, o.kind, o.pos)
    }

    /// The ground at `pos`: its elevation level. For vision, whose targets are places rather than objects.
    pub fn ground(map: &Map, pos: Vec2) -> Elevation {
        Elevation::Level(map.elevation_level(pos))
    }
}

/// `core/vision.md > ElevationModifier`: the grid units added to a range from `source` against `target`: +1 from
/// higher ground, −1 from lower ground, 0 at equal elevation or when either is exempt. `> Binary`: any difference
/// in level counts the same, whatever its size.
pub fn modifier(source: Elevation, target: Elevation) -> f32 {
    match (source, target) {
        (Elevation::Level(s), Elevation::Level(t)) => match s.cmp(&t) {
            Ordering::Greater => ELEVATION_MODIFIER,
            Ordering::Less => -ELEVATION_MODIFIER,
            Ordering::Equal => 0.0,
        },
        (Elevation::Exempt, _) | (_, Elevation::Exempt) => 0.0,
    }
}

/// An attack's `range` (grid units) from `source` against `target`, with the modifier; never below 0. Melee attacks
/// don't take the modifier (`systems/combat.md > FullyConnected > Melee`): combat leaves them out.
pub fn attack_range(range: f32, source: Elevation, target: Elevation) -> f32 {
    (range + modifier(source, target)).max(0.0)
}

/// A `sight` range (grid units) from `source` towards `target`, with the modifier; never below 0. Something that
/// provides no vision (`sight` 0) gains none from high ground.
pub fn sight_range(sight: f32, source: Elevation, target: Elevation) -> f32 {
    if sight <= 0.0 {
        return 0.0;
    }
    (sight + modifier(source, target)).max(0.0)
}

#[cfg(test)]
mod tests {
    use super::*;

    const LOW: Elevation = Elevation::Level(0);
    const HIGH: Elevation = Elevation::Level(1);
    const PEAK: Elevation = Elevation::Level(16);

    #[test]
    fn higher_ground_gains_lower_ground_loses_and_the_difference_is_binary() {
        assert_eq!(modifier(HIGH, LOW), 1.0);
        assert_eq!(modifier(LOW, HIGH), -1.0);
        assert_eq!(modifier(HIGH, HIGH), 0.0);
        // Sixteen levels count as one.
        assert_eq!(modifier(PEAK, LOW), 1.0);
        assert_eq!(modifier(LOW, PEAK), -1.0);
        assert_eq!(attack_range(4.0, PEAK, LOW), 5.0);
        assert_eq!(attack_range(3.0, LOW, HIGH), 2.0);
        assert_eq!(sight_range(5.0, HIGH, LOW), 6.0);
        assert_eq!(sight_range(5.0, LOW, PEAK), 4.0);
    }

    #[test]
    fn air_units_are_exempt_both_ways() {
        for other in [LOW, HIGH, PEAK, Elevation::Exempt] {
            assert_eq!(modifier(Elevation::Exempt, other), 0.0);
            assert_eq!(modifier(other, Elevation::Exempt), 0.0);
        }
    }

    #[test]
    fn ranges_never_go_below_zero() {
        assert_eq!(attack_range(0.5, LOW, HIGH), 0.0);
        assert_eq!(sight_range(0.0, HIGH, LOW), 0.0, "no vision stays none");
        assert_eq!(sight_range(0.5, LOW, HIGH), 0.0);
    }

    /// `core/map.md > Elevation > Elevation level`: the level is the height rounded down, so a unit walking up a
    /// ramp changes level where the height crosses the integer; air units are exempt wherever they are.
    #[test]
    fn objects_take_the_level_of_the_ground_they_stand_on() {
        // A ramp from 0 (west) to 1 (east) across the first tile, then flat at 1.
        let map = Map::from_ascii("..\n\n011\n011").unwrap();
        let at = |x: f32| Elevation::of(&map, ObjectKind::Peacekeeper, Vec2::new(x, 32.0));
        assert_eq!(at(10.0), LOW);
        assert_eq!(at(63.9), LOW);
        assert_eq!(at(64.0), HIGH, "exactly on the integer");
        assert_eq!(at(100.0), HIGH);
        assert_eq!(Elevation::ground(&map, Vec2::new(127.0, 1.0)), HIGH);
        // Structures and resources count like ground units.
        assert_eq!(
            Elevation::of(&map, ObjectKind::SupplyDeliveryStation, Vec2::new(64.0, 64.0)),
            HIGH
        );
    }
}
