//! What one sight source sees (`core/vision.md`): a disc of tiles, as far as its SightRange ([`range`]), each by the
//! one test of what an eye sees ([`sees_tile`]).

use glam::Vec2;

use super::Eye;
use crate::elevation::{Elevation, sight_range_by};
use crate::network::services;
use crate::object::Object;
use crate::ruleset::rules;
use crate::units::{SU_PER_GU, gu};

/// The point of a tile that has to lie within a source's sight for the source to see the tile: its centre, as an
/// offset from the tile's north-west corner (space units). The boundary counts: a tile whose centre lies exactly at
/// the SightRange is seen.
pub const TILE_SIGHT_POINT: Vec2 = Vec2::splat(SU_PER_GU / 2.0);

/// Added to the widest sight when choosing which tiles to test, so that rounding never leaves out a tile whose
/// centre lies exactly on the boundary; the exact test decides (space units).
const SCAN_MARGIN: f32 = 1.0;

/// (M6-SN) `o`'s SightRange as its eye has it, grid units: its type's (`ObjectDef::sight_range`), and while it holds
/// the Null Chamber exit buff what that adds (`factions/syndicate/structures/null_chamber.md > Placeholder stats`:
/// "**+2 grid units SightRange**", `network::services::sight_bonus`). A type with none sees nothing, buffed or not.
pub(super) fn range(o: &Object) -> f32 {
    let sight = o.def().sight_range;
    if sight <= 0.0 || o.buffs.is_empty() {
        return sight;
    }
    sight + services::sight_bonus(o)
}

/// The tiles `eye` sees, as row-major indexes in ascending order, into `out` (cleared first). A tile is seen when
/// its sight point ([`TILE_SIGHT_POINT`]) lies within the disc around the eye whose radius is the eye's SightRange
/// with the ElevationModifier between the eye and the ground there (`tile_levels`: the elevation level at each
/// tile's sight point, `width` × `height`, row-major), the boundary included.
pub(super) fn stamp(eye: &Eye, width: u32, height: u32, tile_levels: &[u8], out: &mut Vec<u32>) {
    let modifier = rules().tuning.vision.elevation_modifier;
    out.clear();
    // No tile is seen from further than one modifier beyond the SightRange.
    let reach = gu(eye.sight + modifier) + SCAN_MARGIN;
    let rows = scan_span(eye.at.y, reach, TILE_SIGHT_POINT.y, height);
    let columns = scan_span(eye.at.x, reach, TILE_SIGHT_POINT.x, width);
    for y in rows {
        for x in columns.clone() {
            let i = y * width + x;
            if sees_tile(
                modifier,
                eye.at,
                eye.sight,
                eye.elevation,
                (x, y),
                tile_levels[i as usize],
            ) {
                out.push(i);
            }
        }
    }
}

/// Whether an eye at `at` whose SightRange is `sight` grid units, on `elevation`, sees tile `(x, y)`, whose ground
/// lies at elevation level `level`: whether the tile's sight point ([`TILE_SIGHT_POINT`]) lies within the disc around
/// the eye whose radius is the SightRange with the ElevationModifier (of `modifier` grid units) between the eye and
/// that ground, the boundary included: the one test of what an eye sees, [`stamp`]'s.
#[inline]
pub(super) fn sees_tile(
    modifier: f32,
    at: Vec2,
    sight: f32,
    elevation: Elevation,
    tile: (u32, u32),
    level: u8,
) -> bool {
    let dx = tile.0 as f32 * SU_PER_GU + TILE_SIGHT_POINT.x - at.x;
    let dy = tile.1 as f32 * SU_PER_GU + TILE_SIGHT_POINT.y - at.y;
    let radius = gu(sight_range_by(modifier, sight, elevation, Elevation::Level(level)));
    dx * dx + dy * dy <= radius * radius
}

/// The tiles along one axis whose sight point (at `offset` into the tile) lies within `reach` of `at`, clamped to
/// the map's `0..len`.
fn scan_span(at: f32, reach: f32, offset: f32, len: u32) -> std::ops::Range<u32> {
    let first = ((at - reach - offset) / SU_PER_GU).ceil().max(0.0);
    let end = (((at + reach - offset) / SU_PER_GU).floor() + 1.0).min(len as f32);
    if end <= first { 0..0 } else { first as u32..end as u32 }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::units::TilePos;

    /// A tile whose sight point lies on the boundary is seen (`core/vision.md`: a disc of radius SightRange): from
    /// the centre of tile (0, 0), at 5 grid units, the tiles (5, 0) and (3, 4) are, and (5, 1) is not; 6 grid units
    /// over lower ground from level 1, 4 up to higher, 5 for an air unit's eye over any.
    #[test]
    fn a_tile_on_the_boundary_is_seen() {
        let at = TilePos::new(0, 0).center();
        let sees = |elevation, tile, level| sees_tile(1.0, at, 5.0, elevation, tile, level);
        let ground = Elevation::Level(1);
        assert!(sees(ground, (5, 0), 1) && sees(ground, (3, 4), 1));
        assert!(!sees(ground, (5, 1), 1));
        assert!(
            sees(ground, (6, 0), 0) && !sees(ground, (6, 1), 0),
            "6 over lower ground"
        );
        assert!(
            sees(ground, (4, 0), 2) && !sees(ground, (5, 0), 2),
            "4 up to higher ground"
        );
        assert!(sees(Elevation::Exempt, (5, 0), 0) && !sees(Elevation::Exempt, (6, 0), 0));
    }
}
