//! The build layer: where a computer player's buildings go (`core/game_rules.md > Computer players (playtest tool)`:
//! it "issues the same commands a player can, under the same rules", so every spot is one the placement check takes
//! now, `core/entities.md > Placement Validation > Direct Placement`, as the client's ghost shows it).
//!
//! - [`spot`]: a building's spot near an anchor, with a free ring around it so that units keep their lanes, else
//!   without the ring; never on the staging point, which the army fills.
//! - [`plate_spots`]: the patches still to plate, nearest an anchor first, and [`plate_spot`], the first of them the
//!   placement check takes.
//!
//! Placeholders (not in the design: the design has no computer players' bases), their numbers in
//! `crate::tuning::build`: the ring, the anchor's order, and a budget of placement checks per search, so that a full
//! base costs little. A search runs only when a building is ready to place.

use sc_sim::nav::TileRect;
use sc_sim::placement::{footprint_size, seeds_build_area};
use sc_sim::{BuildingOrientation, Category, Map, ObjectKind, SU_PER_GU, TilePos, Vec2, VisibilityState, gu};

use crate::tuning::army::STAGING_CLEAR_GU;
use crate::tuning::build::{MARGIN, SPOT_CHECKS};
use crate::{Known, Turn, View};

/// A building's spot: the top-left tile of its footprint, unturned, and the footprint.
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub struct Spot {
    pub origin: TilePos,
    pub rect: TileRect,
}

/// Where the player may place a `kind` now, unturned, nearest `anchor` (space units): the origin nearest it whose
/// footprint reaches the build area and passes the placement check, with a free ring of [`MARGIN`] tiles of open
/// ground around it that no other footprint the player knows covers, nor one the turn has placed (`Turn::taken`); if
/// there is none, the nearest without the ring. Never within [`STAGING_CLEAR_GU`] of the staging point. Ties go to
/// the northernmost, then the westernmost. `None` if none is found within [`SPOT_CHECKS`] placement checks.
pub fn spot(turn: &Turn<'_, '_>, kind: ObjectKind, anchor: Vec2) -> Option<Spot> {
    let view = turn.view();
    let orientation = BuildingOrientation::default();
    let (w, h) = footprint_size(kind, orientation)?;
    let size = (i32::try_from(w).ok()?, i32::try_from(h).ok()?);
    let map = view.map();
    let area = view.build_area();
    let seeds = seeds_build_area(kind);
    let (map_w, map_h) = (map.width() as i32, map.height() as i32);
    let (x0, y0, x1, y1) = if seeds {
        (0, 0, map_w - 1, map_h - 1)
    } else {
        let parts = area.parts();
        let x0 = parts.iter().map(|p| p.origin.x).min()?;
        let y0 = parts.iter().map(|p| p.origin.y).min()?;
        let x1 = parts.iter().map(|p| p.origin.x + p.w as i32 - 1).max()?;
        let y1 = parts.iter().map(|p| p.origin.y + p.h as i32 - 1).max()?;
        (x0, y0, x1, y1)
    };
    let clear = turn.staging().map(|s| (s, gu(STAGING_CLEAR_GU) * gu(STAGING_CLEAR_GU)));
    let mut candidates: Vec<(f32, TilePos)> = Vec::new();
    for y in (y0 - size.1 + 1).max(0)..=y1.min(map_h - size.1) {
        for x in (x0 - size.0 + 1).max(0)..=x1.min(map_w - size.0) {
            let origin = TilePos::new(x, y);
            let rect = TileRect::new(origin, w, h);
            if !seeds && !area.reaches(rect) {
                continue;
            }
            let centre = centre(rect);
            if clear.is_some_and(|(staging, r2)| centre.distance_squared(staging) < r2) {
                continue;
            }
            candidates.push((centre.distance_squared(anchor), origin));
        }
    }
    candidates.sort_by(|a, b| (a.0.total_cmp(&b.0)).then(a.1.y.cmp(&b.1.y)).then(a.1.x.cmp(&b.1.x)));
    let occupied = Occupied::of(turn);
    let mut checks = 0;
    for ring in [true, false] {
        for &(_, origin) in &candidates {
            let rect = TileRect::new(origin, w, h);
            // The second pass leaves out what the first one checked.
            if !occupied.free(map, rect, ring) || (!ring && occupied.free(map, rect, true)) || !visible(view, rect) {
                continue;
            }
            if checks == SPOT_CHECKS {
                return None;
            }
            checks += 1;
            if view.placement_check(kind, origin, orientation).is_valid() {
                return Some(Spot { origin, rect });
            }
        }
    }
    None
}

/// The patches the player may still plate, nearest `anchor` first (then by id): the Space Crystals patches it knows,
/// with crystals left as it last saw them, in its build area, under no building it knows of (its own or another's)
/// and not taken by a Place of the turn. Whether the placement check takes one now is [`plate_spot`]'s question.
pub fn plate_spots<'v>(turn: &Turn<'v, '_>, anchor: Vec2) -> Vec<&'v Known> {
    let view = turn.view();
    let area = view.build_area();
    let mut covered: Vec<TileRect> = (view.own())
        .filter(|o| o.kind.def().category == Category::Structure)
        .filter_map(|o| o.placement.map(|p| TileRect::new(p.origin, p.w, p.h)))
        .collect();
    covered.extend((view.enemy_structures()).filter_map(|k| k.placement.map(|p| TileRect::new(p.origin, p.w, p.h))));
    covered.extend_from_slice(turn.taken());
    let mut spots: Vec<&Known> = view
        .patches()
        .filter(|k| k.crystals_left().is_some_and(|left| left > 0))
        .filter(|k| {
            let Some(p) = k.placement else {
                return false;
            };
            area.contains(p.origin) && !covered.iter().any(|&rect| contains(rect, p.origin))
        })
        .collect();
    spots.sort_by(|a, b| {
        (a.pos
            .distance_squared(anchor)
            .total_cmp(&b.pos.distance_squared(anchor)))
        .then(a.id.cmp(&b.id))
    });
    spots
}

/// The first of [`plate_spots`] where the player may place a `plate` now (the placement check: "a plate onto an
/// uncovered patch", on Visible tiles).
pub fn plate_spot(turn: &Turn<'_, '_>, plate: ObjectKind, anchor: Vec2) -> Option<Spot> {
    let orientation = BuildingOrientation::default();
    let (w, h) = footprint_size(plate, orientation)?;
    (plate_spots(turn, anchor).into_iter())
        .filter_map(|k| k.placement.map(|p| p.origin))
        .find(|&origin| turn.view().placement_check(plate, origin, orientation).is_valid())
        .map(|origin| Spot {
            origin,
            rect: TileRect::new(origin, w, h),
        })
}

/// The tiles the footprints the player knows of cover: its own buildings', the others' it knows (seen or remembered),
/// the map's resources', and those a Place of the turn takes. [`spot`]'s, and a faction's own search for its sites
/// (the Colonists' Beacon sites, slice A2c).
pub(crate) struct Occupied {
    width: i32,
    cells: Vec<bool>,
}

impl Occupied {
    pub(crate) fn of(turn: &Turn<'_, '_>) -> Self {
        let view = turn.view();
        let map = view.map();
        let (width, height) = (map.width() as i32, map.height() as i32);
        let mut occupied = Self {
            width,
            cells: vec![false; (width.max(0) * height.max(0)) as usize],
        };
        let own = view
            .own()
            .filter_map(|o| o.placement.map(|p| TileRect::new(p.origin, p.w, p.h)));
        let known = view
            .known()
            .iter()
            .filter_map(|k| k.placement.map(|p| TileRect::new(p.origin, p.w, p.h)));
        for rect in own.chain(known).chain(turn.taken().iter().copied()) {
            for tile in tiles(rect, 0) {
                if let Some(i) = occupied.index(map, tile) {
                    occupied.cells[i] = true;
                }
            }
        }
        occupied
    }

    fn index(&self, map: &Map, tile: TilePos) -> Option<usize> {
        map.in_bounds(tile).then(|| (tile.y * self.width + tile.x) as usize)
    }

    /// Whether no footprint covers `rect`, nor, with the `ring`, the [`MARGIN`] tiles around it, which must then lie on
    /// the map and be open ground (Traversible, not Rugged).
    pub(crate) fn free(&self, map: &Map, rect: TileRect, ring: bool) -> bool {
        let margin = if ring { MARGIN } else { 0 };
        tiles(rect, margin).all(|tile| match self.index(map, tile) {
            Some(i) => !self.cells[i] && (!ring || contains(rect, tile) || open(map, tile)),
            None => false,
        })
    }
}

/// Whether the player sees every tile of `rect` now: the placement check's first condition, cheaper.
fn visible(view: &View<'_>, rect: TileRect) -> bool {
    tiles(rect, 0).all(|tile| view.visibility(tile) == VisibilityState::Visible)
}

/// Whether units walk on `tile`: Traversible and not Rugged ground.
pub(crate) fn open(map: &Map, tile: TilePos) -> bool {
    map.tile_preset(tile).is_some_and(|p| p.traversible && !p.rugged)
}

/// The tiles of `rect` grown by `margin` on every side.
fn tiles(rect: TileRect, margin: i32) -> impl Iterator<Item = TilePos> {
    let (x0, y0) = (rect.origin.x - margin, rect.origin.y - margin);
    let (x1, y1) = (
        rect.origin.x + rect.w as i32 + margin,
        rect.origin.y + rect.h as i32 + margin,
    );
    (y0..y1).flat_map(move |y| (x0..x1).map(move |x| TilePos::new(x, y)))
}

fn contains(rect: TileRect, tile: TilePos) -> bool {
    (rect.origin.x..rect.origin.x + rect.w as i32).contains(&tile.x)
        && (rect.origin.y..rect.origin.y + rect.h as i32).contains(&tile.y)
}

/// The centre of `rect`, space units.
fn centre(rect: TileRect) -> Vec2 {
    let corner = Vec2::new(rect.origin.x as f32, rect.origin.y as f32);
    (corner + Vec2::new(rect.w as f32, rect.h as f32) / 2.0) * SU_PER_GU
}
