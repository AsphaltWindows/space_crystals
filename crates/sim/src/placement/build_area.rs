//! The GDO build area (`factions/gdo/structures.md > GDOBuildArea`): where a GDO player may place buildings.

use std::ops::Range;

use crate::defs::ObjectKind;
use crate::map::Map;
use crate::nav::TileRect;
use crate::object::Placement;
use crate::player::Faction;
use crate::units::TilePos;

/// `factions/gdo/structures.md > GDOBuildArea`: a player's buildable area. Each GDO building they own adds its
/// footprint grown by its BuildRadiusExtension on every side, diagonals included, so a square building adds a square
/// (design change 0091: the starting Deployment Center, 4 × 4 with 12, adds 28 × 28 tiles). The area is the union of
/// what the buildings standing now add: it shrinks when one is destroyed, unless another covers the same tiles, and it
/// needs no Deployment Center (design change 0092). A building with a BuildRadiusExtension of 0 (the Extraction
/// Plate) adds nothing, its own footprint included: such buildings "never extend it" (`> GDOBuildArea`, of the Field
/// Engineer's), and nothing else can be placed on that footprint anyway.
///
/// It is the area only, never what is valid on it right now: slopes, fog and units show on the placement ghost, tile
/// by tile, and make no holes in it (design change 0093). It is derived from the buildings and never stored or hashed:
/// [`crate::Sim::build_area`] works it out from the objects whenever it is asked, so it follows every placement and
/// every loss at once, also between two commands of one tick. A view that draws it keeps the last one it drew and
/// redraws when the new one differs (`PartialEq`).
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct BuildArea {
    /// What each building adds, clamped to the map, in the order of the buildings (by object id for a sim's).
    parts: Vec<TileRect>,
}

impl BuildArea {
    /// The area `buildings` give on `map`, each a kind and where it stands; whatever isn't a GDO building with a
    /// BuildRadiusExtension adds nothing.
    pub fn of<'a>(map: &Map, buildings: impl IntoIterator<Item = (ObjectKind, &'a Placement)>) -> Self {
        let parts = buildings
            .into_iter()
            .filter_map(|(kind, placement)| part(map, kind, placement))
            .collect();
        Self { parts }
    }

    /// The area of a Starting Layout, which `structures` are (`core/map_editor.md > Starting Layouts > Layout
    /// validation`, design change 0097): the area its seeds give ([`seeds_build_area`]). Its other buildings must
    /// reach it, as the design says of the Extraction Facility and the plates: "within the build area of the
    /// Deployment Center placed in the same layout". So, read literally, they don't extend it for each other, as
    /// they would in a game.
    pub fn of_layout<'a>(map: &Map, structures: impl IntoIterator<Item = (ObjectKind, &'a Placement)>) -> Self {
        Self::of(map, structures.into_iter().filter(|&(kind, _)| seeds_build_area(kind)))
    }

    /// What each building adds (the area is their union), clamped to the map, in the order of the buildings.
    pub fn parts(&self) -> &[TileRect] {
        &self.parts
    }

    /// Whether the area has no tile at all.
    pub fn is_empty(&self) -> bool {
        self.parts.is_empty()
    }

    /// Whether `tile` lies in the area.
    pub fn contains(&self, tile: TilePos) -> bool {
        self.parts.iter().any(|part| overlap(*part, TileRect::new(tile, 1, 1)))
    }

    /// Whether any tile of `rect` lies in the area: `> GDOBuildArea`, "at least 1 grid cell of the new building must
    /// be within the current build area".
    pub fn reaches(&self, rect: TileRect) -> bool {
        self.parts.iter().any(|part| overlap(*part, rect))
    }

    /// The area row by row, north to south: each row's runs of tiles, west to east, as `(y, x0..x1)`. Runs don't
    /// touch: each ends where the area does.
    pub fn runs(&self) -> Vec<(i32, Range<i32>)> {
        let mut rows: Vec<i32> = self
            .parts
            .iter()
            .flat_map(|p| p.origin.y..end(p.origin.y, p.h))
            .collect();
        rows.sort_unstable();
        rows.dedup();
        let mut runs = Vec::new();
        for y in rows {
            let mut spans: Vec<Range<i32>> = self
                .parts
                .iter()
                .filter(|p| (p.origin.y..end(p.origin.y, p.h)).contains(&y))
                .map(|p| p.origin.x..end(p.origin.x, p.w))
                .collect();
            spans.sort_unstable_by_key(|s| (s.start, s.end));
            let mut spans = spans.into_iter();
            let Some(mut run) = spans.next() else {
                continue;
            };
            for span in spans {
                if span.start <= run.end {
                    run.end = run.end.max(span.end);
                } else {
                    runs.push((y, std::mem::replace(&mut run, span)));
                }
            }
            runs.push((y, run));
        }
        runs
    }

    /// Every tile of the area once, row by row from the north-west.
    pub fn tiles(&self) -> impl Iterator<Item = TilePos> {
        self.runs()
            .into_iter()
            .flat_map(|(y, xs)| xs.map(move |x| TilePos::new(x, y)))
    }
}

/// Whether buildings of `kind` seed the build area instead of having to reach it (`factions/gdo/structures.md >
/// GDOBuildArea`: "Seeded by each Deployment Center's and each Outpost's BuildRadiusExtension"), as their def says
/// (`StructureDef::seeds_build_area`): the Deployment Center. The Outpost, which doesn't exist yet, will be the other:
/// it "can be deployed outside the current build area".
pub fn seeds_build_area(kind: ObjectKind) -> bool {
    kind.def().structure.as_ref().is_some_and(|s| s.seeds_build_area)
}

/// Whether placing a building of `kind` needs a tile in its owner's build area: a GDO building that doesn't seed it
/// (`> GDOBuildArea`: "When placing a building, at least 1 grid cell of the new building must be within the current
/// build area").
pub(super) fn needs_build_area(kind: ObjectKind) -> bool {
    is_gdo_building(kind) && !seeds_build_area(kind)
}

/// A structure of the GDO's: what the GDO build area is made of, and what is placed in it.
fn is_gdo_building(kind: ObjectKind) -> bool {
    let def = kind.def();
    def.structure.is_some() && def.faction == Some(Faction::Gdo)
}

/// What a building of `kind` standing at `placement` adds to its owner's build area: its footprint grown by its
/// BuildRadiusExtension on every side, clamped to `map`. Nothing for a building without one, or for anything but a
/// GDO building.
fn part(map: &Map, kind: ObjectKind, placement: &Placement) -> Option<TileRect> {
    let extension = kind
        .def()
        .structure
        .as_ref()
        .filter(|_| is_gdo_building(kind))?
        .build_radius_extension;
    if extension == 0 {
        return None;
    }
    // 64 bits: no overflow, wherever the footprint lies and however far the extension reaches.
    let r = i64::from(extension);
    let span = |start: i32, len: u32, size: u32| {
        let clamp = |v: i64| v.clamp(0, i64::from(size));
        (
            clamp(i64::from(start) - r),
            clamp(i64::from(start) + i64::from(len) + r),
        )
    };
    let (x0, x1) = span(placement.origin.x, placement.w, map.width());
    let (y0, y1) = span(placement.origin.y, placement.h, map.height());
    // Within 0..=MAX_MAP_SIDE after the clamp, so the casts are exact.
    (x0 < x1 && y0 < y1).then(|| TileRect::new(TilePos::new(x0 as i32, y0 as i32), (x1 - x0) as u32, (y1 - y0) as u32))
}

/// The end of a run of `len` tiles from `start`. Parts lie on the map, so it fits.
fn end(start: i32, len: u32) -> i32 {
    start.saturating_add_unsigned(len)
}

/// Whether two tile rectangles share a tile, in 64 bits so that no rectangle overflows.
pub(super) fn overlap(a: TileRect, b: TileRect) -> bool {
    let span = |start: i32, len: u32| (i64::from(start), i64::from(start) + i64::from(len));
    let apart = |(a0, a1): (i64, i64), (b0, b1): (i64, i64)| a1 <= b0 || b1 <= a0;
    !(apart(span(a.origin.x, a.w), span(b.origin.x, b.w)) || apart(span(a.origin.y, a.h), span(b.origin.y, b.h)))
}

#[cfg(test)]
mod tests {
    use super::*;

    fn at(kind: ObjectKind, x: i32, y: i32) -> (ObjectKind, Placement) {
        (kind, Placement::new(TilePos::new(x, y), kind.def().footprint))
    }

    fn area(map: &Map, buildings: &[(ObjectKind, Placement)]) -> BuildArea {
        BuildArea::of(map, buildings.iter().map(|(kind, p)| (*kind, p)))
    }

    /// Design change 0091's verification: on an empty map, a Deployment Center's area is exactly 28 × 28 tiles
    /// centred on it.
    #[test]
    fn a_deployment_center_adds_a_square_of_28_tiles() {
        let map = Map::new(64, 64);
        let dc = [at(ObjectKind::DeploymentCenter, 20, 30)];
        let area = area(&map, &dc);
        assert_eq!(area.parts(), [TileRect::new(TilePos::new(8, 18), 28, 28)]);
        let tiles: Vec<TilePos> = area.tiles().collect();
        assert_eq!(tiles.len(), 28 * 28);
        assert_eq!(
            (tiles[0], tiles[tiles.len() - 1]),
            (TilePos::new(8, 18), TilePos::new(35, 45))
        );
        // Centred on the DC: 12 tiles on every side of its 4 × 4, the corners included.
        for (tile, inside) in [
            ((8, 18), true),
            ((7, 18), false),
            ((35, 45), true),
            ((36, 45), false),
            ((22, 46), false),
        ] {
            assert_eq!(area.contains(TilePos::new(tile.0, tile.1)), inside, "{tile:?}");
        }
    }

    #[test]
    fn the_area_stops_at_the_edge_of_the_map() {
        let map = Map::new(20, 16);
        let area = area(&map, &[at(ObjectKind::DeploymentCenter, 2, 10)]);
        assert_eq!(area.parts(), [TileRect::new(TilePos::new(0, 0), 18, 16)]);
        assert_eq!(area.tiles().count(), 18 * 16);
    }

    /// Each kind adds its own BuildRadiusExtension; a plate (0) adds nothing, not even its tile, and objects that
    /// aren't GDO buildings add nothing.
    #[test]
    fn each_building_adds_its_extension() {
        use ObjectKind::*;
        let map = Map::new(64, 64);
        for (kind, x, y, part) in [
            (Generator, 10, 10, Some(TileRect::new(TilePos::new(9, 9), 4, 4))),
            (Barracks, 10, 10, Some(TileRect::new(TilePos::new(8, 8), 7, 6))),
            (
                ExtractionFacility,
                10,
                10,
                Some(TileRect::new(TilePos::new(8, 8), 7, 7)),
            ),
            (ExtractionPlate, 10, 10, None),
            (CrystalPatch, 10, 10, None),
            (SupplyDeliveryStation, 10, 10, None),
        ] {
            assert_eq!(area(&map, &[at(kind, x, y)]).parts().first().copied(), part, "{kind:?}");
        }
    }

    /// The union counts every tile once; separate parts stay separate runs.
    #[test]
    fn overlapping_parts_merge_into_runs() {
        use ObjectKind::Generator;
        let map = Map::new(32, 32);
        // Two Generators side by side (parts 9..13 and 12..16 in x) and one apart (20..24).
        let area = area(
            &map,
            &[at(Generator, 10, 10), at(Generator, 13, 10), at(Generator, 21, 10)],
        );
        let runs = area.runs();
        assert_eq!(runs.len(), 2 * 4, "two runs in each of rows 9 to 12");
        assert_eq!(runs[0], (9, 9..16));
        assert_eq!(runs[1], (9, 20..24));
        assert_eq!(area.tiles().count(), 4 * (7 + 4));
        assert!(
            area.reaches(TileRect::new(TilePos::new(15, 12), 3, 3)),
            "the corner (15, 12)"
        );
        assert!(!area.reaches(TileRect::new(TilePos::new(16, 13), 3, 3)));
        assert!(!area.reaches(TileRect::new(TilePos::new(i32::MAX, i32::MIN), 16, 16)));
    }

    #[test]
    fn a_layout_s_area_is_its_deployment_centers() {
        use ObjectKind::*;
        let map = Map::new(64, 64);
        let layout = [
            at(ExtractionFacility, 30, 30),
            at(DeploymentCenter, 20, 20),
            at(Generator, 40, 40),
        ];
        let area = BuildArea::of_layout(&map, layout.iter().map(|(kind, p)| (*kind, p)));
        assert_eq!(area.parts(), [TileRect::new(TilePos::new(8, 8), 28, 28)]);
        assert!(BuildArea::of_layout(&map, std::iter::empty()).is_empty());
        assert!(seeds_build_area(DeploymentCenter) && !needs_build_area(DeploymentCenter));
        assert!([Generator, Barracks, ExtractionFacility, ExtractionPlate].map(needs_build_area) == [true; 4]);
        assert!(!needs_build_area(Peacekeeper) && !needs_build_area(CrystalPatch));
    }
}
