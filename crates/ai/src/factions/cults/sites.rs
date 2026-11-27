//! Where the Cults' buildings go (`super` module docs, 2; `core/game_rules.md > Computer players (playtest tool)`: it
//! "issues the same commands a player can, under the same rules", so every site is one the check of a Build takes now,
//! `View::build_check`, `core/entities.md > Placement Validation > Worker-Built Structures`). `crate::build` places
//! the GDO's buildings in its build area; a Cults building has none and needs these.
//!
//! - **A Storage** ([`storage`]): of the sites a tile from the field's patches, unturned or turned a quarter, the one
//!   that adds the most seats (`field.rs`), at least [`STORAGE_SEATS`]; ties to the site nearest the anchor, then the
//!   northernmost, the westernmost. No patch touches it, so that every patch beside it keeps its lane
//!   (`factions/cults/units/recruit.md > Gathering`: "buildings snap to whole tiles, so a patch can't touch a Storage
//!   with room for Recruits to work"); it may stand against another Storage, as the reference field's do
//!   (`factions/cults/economy.md > Economy check`: "Storages stand along both long sides"), but a tile clear of any
//!   other building, on open ground.
//! - **Any other building** ([`near`]): the site nearest an anchor, unturned, with a ring of a tile of open ground
//!   around it that no footprint covers, and no tile within [`FIELD_ZONE`] tiles of the field's patches, so that the
//!   Storages' sites and the Recruits' ways stay free.
//! - **A Recruitment Hall** ([`hall`]): [`near`] the start, within `HALL_SEARCH_GU` of it (the ground an enemy's waves
//!   search, `crate::army::search_points`), where it would claim at least [`HALL_EFFECTIVENESS`] percent of its area
//!   as far as the player knows (`factions/cults/structures/recruitment_hall.md > Effectiveness`):
//!   the tiles of its area on the map, Recruitable, and outside the area of every Hall the player knows of, its own
//!   and the others' it saw (`factions/cults/structures/recruitment_hall.md > Recruitment Area display`: "Enemy Halls
//!   you have never seen are not counted").
//!
//! Never within [`STAGING_CLEAR_GU`] of the staging point, which the army fills. Each search makes at most
//! [`SPOT_CHECKS`] checks of the gate's, and runs only when the building is wanted and the bank covers it.

use std::collections::BTreeSet;

use sc_sim::nav::TileRect;
use sc_sim::placement::footprint_size;
use sc_sim::recruitment::{area, def_of, recruitable};
use sc_sim::{BuildingOrientation, Category, Map, ObjectId, ObjectKind, Placement, Rotation, TilePos, Vec2, gu};

use super::centre;
use super::field::{Field, rect, seats};
use crate::build::open;
use crate::tuning::army::STAGING_CLEAR_GU;
use crate::tuning::build::SPOT_CHECKS;
use crate::tuning::cults::{FIELD_ZONE, HALL_EFFECTIVENESS, STORAGE_SEATS};
use crate::{Turn, View};

/// A building's site: its footprint's top-left tile, how it is turned, and the footprint.
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub(super) struct Site {
    pub origin: TilePos,
    pub orientation: BuildingOrientation,
    pub rect: TileRect,
}

impl Site {
    /// The centre of its footprint, space units.
    pub(super) fn centre(&self) -> Vec2 {
        centre(self.origin, (self.rect.w, self.rect.h))
    }
}

/// What covers a tile, as far as the player knows.
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
enum Cover {
    Open,
    Storage,
    Patch,
    Other,
}

/// What the player knows to cover each tile of the map: its own footprints, those it knows of the others' (seen or
/// remembered) and of the map's resources; and which tiles lie within [`FIELD_ZONE`] of the field.
struct Ground<'v> {
    map: &'v Map,
    cover: Vec<Cover>,
    zone: Vec<bool>,
}

impl<'v> Ground<'v> {
    fn of(view: &View<'v>, field: &Field) -> Self {
        let map = view.map();
        let cells = (map.width() * map.height()) as usize;
        let mut ground = Self {
            map,
            cover: vec![Cover::Open; cells],
            zone: vec![false; cells],
        };
        let own = (view.own()).filter_map(|o| Some((o.kind, o.placement.as_ref()?)));
        let known = (view.known().iter()).filter_map(|k| Some((k.kind, k.placement.as_ref()?)));
        for (kind, placement) in own.chain(known) {
            let cover = match kind {
                ObjectKind::Storage => Cover::Storage,
                _ if kind.def().category == Category::Resource => Cover::Patch,
                _ => Cover::Other,
            };
            for tile in tiles(rect(placement), 0) {
                if let Some(i) = ground.index(tile) {
                    ground.cover[i] = cover;
                }
            }
        }
        for &(_, patch) in &field.patches {
            for tile in tiles(patch, FIELD_ZONE) {
                if let Some(i) = ground.index(tile) {
                    ground.zone[i] = true;
                }
            }
        }
        ground
    }

    fn index(&self, tile: TilePos) -> Option<usize> {
        let width = self.map.width() as i32;
        self.map.in_bounds(tile).then(|| (tile.y * width + tile.x) as usize)
    }

    fn cover(&self, tile: TilePos) -> Option<Cover> {
        self.index(tile).map(|i| self.cover[i])
    }

    /// Whether every tile of `rect` is on the map and covered by nothing.
    fn free(&self, rect: TileRect) -> bool {
        tiles(rect, 0).all(|tile| self.cover(tile) == Some(Cover::Open))
    }

    /// Whether every tile of the ring around `rect` is on the map, and open ground that nothing covers, or a Storage
    /// when `storage` (a Storage may stand against another).
    fn ring(&self, rect: TileRect, storage: bool) -> bool {
        tiles(rect, 1)
            .filter(|&tile| !contains(rect, tile))
            .all(|tile| match self.cover(tile) {
                Some(Cover::Open) => open(self.map, tile),
                Some(Cover::Storage) => storage,
                _ => false,
            })
    }

    /// Whether no tile of `rect` lies within [`FIELD_ZONE`] of the field.
    fn off_field(&self, rect: TileRect) -> bool {
        tiles(rect, 0).all(|tile| self.index(tile).is_some_and(|i| !self.zone[i]))
    }
}

/// The checks every site passes ([`storage`], [`near`]): out of the staging point's way, and the gate's check of a
/// Build by `worker` (at most [`SPOT_CHECKS`] of them), the first passing in `candidates`' order.
fn first_valid(
    turn: &Turn<'_, '_>,
    worker: ObjectId,
    kind: ObjectKind,
    candidates: impl Iterator<Item = Site>,
) -> Option<Site> {
    let view = turn.view();
    let clear = turn.staging().map(|s| (s, gu(STAGING_CLEAR_GU) * gu(STAGING_CLEAR_GU)));
    let mut checks = 0;
    for site in candidates {
        if clear.is_some_and(|(staging, r2)| site.centre().distance_squared(staging) < r2) {
            continue;
        }
        if checks == SPOT_CHECKS {
            return None;
        }
        checks += 1;
        let check = view.build_check(worker, kind, site.origin, site.orientation, false);
        if check.is_some_and(|check| check.is_valid()) {
            return Some(site);
        }
    }
    None
}

/// The orientations a Storage is tried in: unturned, and turned a quarter.
const STORAGE_TURNS: [Rotation; 2] = [Rotation::R0, Rotation::R90];

/// Where the next Storage goes (module docs), by `worker`'s check: `None` if no site adds [`STORAGE_SEATS`] seats.
pub(super) fn storage(turn: &Turn<'_, '_>, field: &Field, worker: ObjectId, anchor: Vec2) -> Option<Site> {
    let view = turn.view();
    let ground = Ground::of(view, field);
    let mut seen = BTreeSet::new();
    let mut candidates: Vec<(u32, f32, Site)> = Vec::new();
    for &(_, patch) in &field.patches {
        for (turn_index, rotation) in STORAGE_TURNS.into_iter().enumerate() {
            let orientation = BuildingOrientation {
                rotation,
                ..BuildingOrientation::default()
            };
            let Some((w, h)) = footprint_size(ObjectKind::Storage, orientation) else {
                continue;
            };
            let (wi, hi) = (w as i32, h as i32);
            for y in patch.origin.y - hi - 1..=patch.origin.y + patch.h as i32 + 1 {
                for x in patch.origin.x - wi - 1..=patch.origin.x + patch.w as i32 + 1 {
                    if !seen.insert((y, x, turn_index)) {
                        continue;
                    }
                    let origin = TilePos::new(x, y);
                    let rect = TileRect::new(origin, w, h);
                    if !ground.free(rect) || !ground.ring(rect, true) {
                        continue;
                    }
                    let added: u32 = field.patches.iter().map(|&(_, p)| seats(p, rect)).sum();
                    if added < STORAGE_SEATS {
                        continue;
                    }
                    let site = Site {
                        origin,
                        orientation,
                        rect,
                    };
                    candidates.push((added, site.centre().distance_squared(anchor), site));
                }
            }
        }
    }
    candidates.sort_by(|a, b| {
        (b.0.cmp(&a.0))
            .then(a.1.total_cmp(&b.1))
            .then((a.2.origin.y, a.2.origin.x).cmp(&(b.2.origin.y, b.2.origin.x)))
            .then(a.2.rect.w.cmp(&b.2.rect.w))
    });
    first_valid(
        turn,
        worker,
        ObjectKind::Storage,
        candidates.into_iter().map(|(_, _, site)| site),
    )
}

/// Where a `kind` goes near `anchor` (module docs), within `radius` grid units of it, by `worker`'s check, among the
/// sites `accept` takes: the nearest, ties to the northernmost, then the westernmost.
pub(super) fn near(
    turn: &Turn<'_, '_>,
    field: &Field,
    kind: ObjectKind,
    worker: ObjectId,
    (anchor, radius): (Vec2, f32),
    accept: impl Fn(TileRect) -> bool,
) -> Option<Site> {
    let view = turn.view();
    let ground = Ground::of(view, field);
    let orientation = BuildingOrientation::default();
    let (w, h) = footprint_size(kind, orientation)?;
    let map = view.map();
    let (map_w, map_h) = (map.width() as i32, map.height() as i32);
    let r2 = gu(radius) * gu(radius);
    let mut candidates: Vec<(f32, Site)> = Vec::new();
    for y in 0..=map_h - h as i32 {
        for x in 0..=map_w - w as i32 {
            let origin = TilePos::new(x, y);
            let rect = TileRect::new(origin, w, h);
            let site = Site {
                origin,
                orientation,
                rect,
            };
            let d2 = site.centre().distance_squared(anchor);
            if d2 <= r2 {
                candidates.push((d2, site));
            }
        }
    }
    candidates
        .sort_by(|a, b| (a.0.total_cmp(&b.0)).then((a.1.origin.y, a.1.origin.x).cmp(&(b.1.origin.y, b.1.origin.x))));
    let fits = |site: &Site| {
        ground.free(site.rect) && ground.ring(site.rect, false) && ground.off_field(site.rect) && accept(site.rect)
    };
    first_valid(
        turn,
        worker,
        kind,
        candidates.into_iter().map(|(_, site)| site).filter(fits),
    )
}

/// Where a Recruitment Hall goes (module docs): [`near`] `around`'s point, within its radius in grid units, where it
/// would claim at least [`HALL_EFFECTIVENESS`] percent of its area, as far as the player knows.
pub(super) fn hall(turn: &Turn<'_, '_>, field: &Field, worker: ObjectId, around: (Vec2, f32)) -> Option<Site> {
    let view = turn.view();
    let kind = ObjectKind::RecruitmentHall;
    let def = def_of(kind)?;
    let map = view.map();
    let width = map.width() as i32;
    let mut claimed = vec![false; (map.width() * map.height()) as usize];
    let own = (view.own())
        .filter(|o| o.kind == kind)
        .filter_map(|o| o.placement.map(|p| rect(&p)));
    let known = (view.enemy_structures())
        .filter(|k| k.kind == kind)
        .filter_map(|k| k.placement.map(|p| rect(&p)));
    for hall in own.chain(known) {
        for tile in area(&placement(hall), def) {
            if map.in_bounds(tile) {
                claimed[(tile.y * width + tile.x) as usize] = true;
            }
        }
    }
    let total = def.area * def.area;
    let effective = |rect: TileRect| {
        let free = area(&placement(rect), def)
            .filter(|&tile| {
                map.in_bounds(tile) && recruitable(map, tile) && !claimed[(tile.y * width + tile.x) as usize]
            })
            .count() as u32;
        total > 0 && free * 100 >= HALL_EFFECTIVENESS * total
    };
    near(turn, field, kind, worker, around, effective)
}

/// An unturned footprint on `rect`.
fn placement(rect: TileRect) -> Placement {
    Placement {
        origin: rect.origin,
        w: rect.w,
        h: rect.h,
        rotation: Rotation::R0,
        flip_h: false,
        flip_v: false,
    }
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
