//! The placement check (`core/entities.md > Placement Validation`): every rule a footprint breaks, tile by tile and
//! as a whole. The rules are listed in the `placement` module docs.

use glam::Vec2;

use super::build_area::{BuildArea, needs_build_area, overlap};
use super::footprint::Footprint;
use super::onto::Onto;
use super::problems::{PlacementCheck, TileProblem, WholeProblem};
use super::site::{Method, Mode, Occupant, Site};
use super::tunnel_area::{area_rule, layout_method};
use super::tunnel_overlap::Overlap;
use crate::defs::{Category, Domain, ObjectKind};
use crate::map::Map;
use crate::nav::TileRect;
use crate::paths::box_overlaps_rect;
use crate::scenario::BuildingOrientation;
use crate::tunnel;
use crate::units::TilePos;
use crate::vision::VisibilityState;

/// The size `(w, h)` in tiles of `kind`'s footprint turned by `orientation`: its size in the default orientation,
/// with `w` and `h` swapped by a quarter turn (`core/entities.md > Structure Type`). `None` for units.
pub fn footprint_size(kind: ObjectKind, orientation: BuildingOrientation) -> Option<(u32, u32)> {
    let (w, h) = kind.def().tile_size()?;
    Some(if orientation.swaps_axes() { (h, w) } else { (w, h) })
}

/// `core/entities.md > Placement Validation`: everything that keeps a structure of `kind` from being placed at `site`
/// with the top-left tile of its footprint at `origin`, turned by `orientation`, among `occupants`: everything on
/// the map, the object being placed excepted (a Warp In's Beacon may be among them: its tiles break no rule). The
/// answer depends on its inputs only, whatever the occupants' order. Never panics, whatever the origin. With no
/// reservations: [`check_reserving`].
pub fn check(
    site: &Site<'_>,
    occupants: impl IntoIterator<Item = Occupant>,
    kind: ObjectKind,
    origin: TilePos,
    orientation: BuildingOrientation,
) -> PlacementCheck {
    check_reserving(site, occupants, &[], kind, origin, orientation)
}

/// [`check`], with the sites the placing player has `reserved` for its pending worker builds, each the structure it
/// will be where it will stand: their footprints' tiles ([`TileProblem::Reserved`], design change 0125), against a
/// Deploy, a Warp In (slice M3-F) and a Build (slice M4-E), and a Tunnel's Tier 1 area against a Tunnel's
/// ([`WholeProblem::TunnelAreaOverlap`], Placeholder: Q0079 point 2; `placement/tunnel_overlap.rs`).
pub fn check_reserving(
    site: &Site<'_>,
    occupants: impl IntoIterator<Item = Occupant>,
    reserved: &[Occupant],
    kind: ObjectKind,
    origin: TilePos,
    orientation: BuildingOrientation,
) -> PlacementCheck {
    let def = kind.def();
    let Some((w, h)) = footprint_size(kind, orientation).filter(|_| def.structure.is_some()) else {
        return PlacementCheck {
            tiles: Vec::new(),
            whole: vec![WholeProblem::NotAStructure],
        };
    };
    let footprint = Footprint { origin, w, h };
    let mut onto = Onto::new(kind, footprint.rect());
    // Each footprint tile's problems, as bits, row by row.
    let mut found = vec![0u16; footprint.len()];
    let map = site.map;
    // An underground expansion's ground (`super::Method::Expansion`, slice M4-F): Drillable in place of Buildable
    // (design change 0239), and neither flatness nor sight.
    let expansion = matches!(site.method, Method::Expansion { .. });
    // `> Direct Placement`: "all footprint tiles must be in the Visible state"; `> Warp In` too (design change 0119).
    for (i, tile) in footprint.tiles() {
        if !map.in_bounds(tile) {
            found[i] |= TileProblem::OffMap.bit();
            continue;
        }
        let preset = map.tile_preset(tile);
        if expansion {
            if !preset.is_some_and(|preset| preset.drillable) {
                found[i] |= TileProblem::NotDrillable.bit();
            }
            continue;
        }
        if !preset.is_some_and(|preset| preset.buildable) {
            found[i] |= TileProblem::NotBuildable.bit();
        }
        if def.needs_flat_ground && !map.is_flat(tile) {
            found[i] |= TileProblem::NotFlat.bit();
        }
        if let Mode::Command { vision, player } = site.mode
            && vision.state(player, tile) != VisibilityState::Visible
        {
            found[i] |= TileProblem::NotVisible.bit();
        }
    }
    // The Tunnel Areas the structure's, if it has one, must keep clear of (`super::tunnel_overlap`).
    let mut tunnel_areas = Overlap::new(kind);
    for o in occupants {
        onto.see(&o);
        tunnel_areas.see(&o);
        match &o.placement {
            Some(placement) => {
                let rect = TileRect::new(placement.origin, placement.w, placement.h);
                // The one rule of which footprints may share tiles, as for map objects and debug spawns.
                if kind.may_overlap(o.kind) {
                    continue;
                }
                let problem = match o.kind.def().category {
                    Category::Structure => TileProblem::Structure,
                    Category::Resource => TileProblem::Resource,
                    Category::Unit => continue,
                };
                if overlap(rect, footprint.rect()) {
                    footprint.mark(&mut found, problem, |tile| overlap(rect, TileRect::new(tile, 1, 1)));
                }
            }
            None => {
                // `core/entities.md > Direct Placement`'s exception (design change 0290): "ground units **don't** block
                // a Syndicate expansion's placement, since it is underground and units walk over expansions anyway".
                let Some(half) = ground_half_extents(o.kind).filter(|_| !expansion) else {
                    continue;
                };
                // Strictly, so a unit only touching the footprint's edge leaves it free (`docs/movement.md > 3`).
                if box_overlaps_rect(o.pos, half, footprint.rect()) {
                    footprint.mark(&mut found, TileProblem::Unit, |tile| {
                        box_overlaps_rect(o.pos, half, TileRect::new(tile, 1, 1))
                    });
                }
            }
        }
    }
    for reserved in reserved {
        tunnel_areas.see(reserved);
        let Some(reserved) = reserved.placement else {
            continue;
        };
        let rect = TileRect::new(reserved.origin, reserved.w, reserved.h);
        if overlap(rect, footprint.rect()) {
            footprint.mark(&mut found, TileProblem::Reserved, |tile| {
                overlap(rect, TileRect::new(tile, 1, 1))
            });
        }
    }
    if let Mode::Known { vision, player } = site.mode {
        // Design change 0120: "Never-explored tiles pass".
        footprint.keep_only(&mut found, TileProblem::Reserved.bit(), |tile| {
            map.in_bounds(tile) && vision.state(player, tile) == VisibilityState::Unexplored
        });
    }
    let hidden = TileProblem::NotVisible.bit();
    if let Method::WarpIn { beacon } = site.method {
        // "Except for the Beacon's own footprint, which will be consumed by the transformation": every rule but
        // sight, which is every tile's (design change 0119).
        let consumed = TileRect::new(beacon.origin, beacon.w, beacon.h);
        footprint.keep_only(&mut found, hidden, |tile| overlap(consumed, TileRect::new(tile, 1, 1)));
    }
    let tiles = footprint
        .tiles()
        .flat_map(|(i, tile)| {
            let bits = if found[i] & hidden != 0 { hidden } else { found[i] };
            TileProblem::ALL
                .into_iter()
                .filter(move |p| bits & p.bit() != 0)
                .map(move |p| (tile, p))
        })
        .collect();
    let mut whole = Vec::new();
    if site.method == Method::Direct && needs_build_area(kind) && !site.area.reaches(footprint.rect()) {
        whole.push(WholeProblem::OutsideBuildArea);
    }
    if !onto.is_kept() {
        whole.push(WholeProblem::NotOnPatch);
    }
    // Design change 0242: a Tunnel's whole Tier 1 area on the map and Drillable (`super::tunnel_area`).
    whole.extend(area_rule(site, kind, footprint.rect()));
    // Design change 0237: and clear of every other Tunnel's area (`super::tunnel_overlap`).
    whole.extend(tunnel_areas.rule(footprint.rect()));
    // `factions/syndicate/tunnels.md > AwaitingPlacement (Expansion)`: "The expansion must fit entirely within the
    // Tunnel Area".
    if let Method::Expansion { area } = site.method
        && !tunnel::area::encloses(area, footprint.rect())
    {
        whole.push(WholeProblem::OutsideTunnelArea);
    }
    PlacementCheck { tiles, whole }
}

/// [`Mode::Layout`] for object `index` of a Starting Layout (`core/map_editor.md > Starting Layouts > Layout
/// validation`, design change 0097). `layout` is the layout's objects where they stand, `others` the map's
/// pre-placed objects. The object is checked, turned as its placement is, against the map, `others` and the rest of
/// `layout`, with the area of the layout's own Deployment Center ([`BuildArea::of_layout`]); an underground
/// structure by the expansion method ([`Method::Expansion`], slice M4-F), in the Tier 1 Tunnel Area of a Tunnel of
/// `layout` that holds it ([`WholeProblem::OutsideTunnelArea`] if none does, slice M4-B: `super::tunnel_area`).
/// `None` for an object without a footprint, a unit (the starting units' rule is the unit-start one,
/// `paths::misplaced`), and for an index past the end.
pub fn check_layout(
    map: &Map,
    others: impl IntoIterator<Item = Occupant>,
    layout: &[Occupant],
    index: usize,
) -> Option<PlacementCheck> {
    let object = layout.get(index)?;
    let placement = object.placement?;
    let area = BuildArea::of_layout(map, layout.iter().filter_map(|o| Some((o.kind, o.placement.as_ref()?))));
    let site = Site {
        map,
        area: &area,
        mode: Mode::Layout,
        method: layout_method(object, layout),
    };
    let rest = layout.iter().enumerate().filter(|&(i, _)| i != index).map(|(_, o)| *o);
    Some(check(
        &site,
        others.into_iter().chain(rest),
        object.kind,
        placement.origin,
        placement.orientation(),
    ))
}

/// The half extents of the box of a ground unit of `kind` (`docs/movement.md > 3`); `None` for anything else, air
/// and underground units included, which never block placement (design change 0094).
fn ground_half_extents(kind: ObjectKind) -> Option<Vec2> {
    let def = kind.def();
    let unit = def.unit.as_ref()?;
    (unit.base_def().domain == Domain::Ground)
        .then(|| def.half_extents())
        .flatten()
}
