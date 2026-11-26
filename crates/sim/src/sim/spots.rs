//! T62's search in a game (`placement::spots`, slice M5-F1): the first spot around a footprint where a unit fits now.
//! One function for every unit put on the map beside a building: the units evacuated from a building that holds them
//! (`sim/holding.rs`) so far.

use std::ops::RangeInclusive;

use glam::Vec2;

use super::Sim;
use crate::defs::ObjectKind;
use crate::object::Placement;
use crate::placement::{Round, Spots};
use crate::units::SU_PER_GU;

impl Sim {
    /// The first spot around the footprint `placement` where a unit of `kind` fits now, with the direction out of the
    /// footprint there: ring after ring through `rings`, each from its spot in the direction nearest `towards` and on
    /// by `round` (`placement::Spots::first`). A spot fits by the rule of every unit put on the map
    /// (`Sim::check_unit_fits`): on the map, its box clear of blocked tiles, of blocking footprints and of the units of
    /// its layer, those put on the map before it included. `None` if none does, or `kind` is no unit.
    pub(super) fn spot_around(
        &self,
        kind: ObjectKind,
        placement: &Placement,
        rings: RangeInclusive<u32>,
        towards: Vec2,
        round: Round,
    ) -> Option<(Vec2, Vec2)> {
        let half = kind.def().half_extents()?;
        Spots::around(placement, half).first(rings, towards, round, |spot| {
            self.map.contains_point(spot) && self.check_unit_fits(kind, spot, None).is_ok()
        })
    }

    /// Every ring around the footprint `placement` that can have a spot on the map for a unit of `kind`
    /// (`placement::Spots::last_ring`): a search "as far out as needed" (Placeholder (Q0106 point 1)).
    pub(super) fn rings_on_map(&self, kind: ObjectKind, placement: &Placement) -> RangeInclusive<u32> {
        let half = kind.def().half_extents().unwrap_or(Vec2::ZERO);
        let size = Vec2::new(self.map.width() as f32, self.map.height() as f32) * SU_PER_GU;
        0..=Spots::around(placement, half).last_ring(size)
    }
}
