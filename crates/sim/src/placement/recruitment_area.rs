//! The placement rule of the Recruitment Area (slice M5-J1), on the geometry of `crate::recruitment::area`: design
//! change 0341 point 1 (`factions/cults/economy.md > RecruitmentHall > RecruitmentArea`): "**The whole area must lie on
//! the map** (user, 2026-11-26; Q0099): a Hall whose area would leave the map can't be placed (as Tunnels, 0242)". It
//! is checked for every kind that recruits (`crate::recruitment::def_of`), on the area its footprint would have, in
//! every [`super::Mode`], as the Tunnel Area's edge rule is (`placement/tunnel_area.rs`): the map's edge is known, so
//! the Recruits' ghost ([`super::Mode::Known`]), the gate, the check on arrival and the Starting Layouts
//! ([`super::Mode::Layout`], `core/map_editor.md > Starting Layouts > Layout validation`) share it. Map spawns and
//! debug spawns don't take the placement check, so they stay exempt: such a Hall's tiles off the map count among its
//! area's and can't be claimed (`crate::recruitment`).

use super::problems::WholeProblem;
use crate::defs::ObjectKind;
use crate::map::Map;
use crate::nav::TileRect;
use crate::object::{Placement, Rotation};
use crate::recruitment;
use crate::units::TilePos;

/// Design change 0341 point 1 for a structure of `kind` whose footprint is `footprint`, on `map`:
/// [`WholeProblem::RecruitmentAreaOffMap`] if a tile of its Recruitment Area lies past the map's edge, as one does when
/// a tile of the footprint does (the area holds the footprint). Nothing for a kind that doesn't recruit. The area is
/// centred on the footprint as turned, which a Hall's square footprint leaves as it is. Never panics, whatever the
/// origin.
pub(super) fn area_rule(map: &Map, kind: ObjectKind, footprint: TileRect) -> Option<WholeProblem> {
    let def = recruitment::def_of(kind)?;
    let last = |origin: i32, len: u32| origin.saturating_add_unsigned(len.saturating_sub(1));
    let far = TilePos::new(
        last(footprint.origin.x, footprint.w),
        last(footprint.origin.y, footprint.h),
    );
    // A footprint past the edge has its area past it; one on the map keeps the area's tiles within reach of an i32.
    if !(map.in_bounds(footprint.origin) && map.in_bounds(far)) {
        return Some(WholeProblem::RecruitmentAreaOffMap);
    }
    let placement = Placement {
        origin: footprint.origin,
        w: footprint.w,
        h: footprint.h,
        rotation: Rotation::R0,
        flip_h: false,
        flip_v: false,
    };
    recruitment::area(&placement, def)
        .any(|tile| !map.in_bounds(tile))
        .then_some(WholeProblem::RecruitmentAreaOffMap)
}
