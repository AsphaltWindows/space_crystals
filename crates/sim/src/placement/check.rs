//! The placement check (`core/entities.md > Placement Validation`): every rule a footprint breaks, tile by tile and
//! as a whole. The rules are listed in the `placement` module docs.

use std::fmt;

use glam::Vec2;
use serde::{Deserialize, Serialize};

use super::build_area::{BuildArea, needs_build_area, overlap};
use super::onto::Onto;
use crate::defs::{Category, Domain, ObjectKind};
use crate::ids::PlayerId;
use crate::map::Map;
use crate::nav::TileRect;
use crate::object::{Object, Placement};
use crate::paths::box_overlaps_rect;
use crate::scenario::BuildingOrientation;
use crate::units::TilePos;
use crate::vision::{VisibilityState, Vision};

/// Something on the map that a footprint must keep clear of, or is placed onto: an object of a game, or one of a map
/// or a Starting Layout that hasn't started.
#[derive(Copy, Clone, Debug, PartialEq)]
pub struct Occupant {
    pub kind: ObjectKind,
    /// Its centre, space units; a unit's box is around it.
    pub pos: Vec2,
    /// A tile-based object's footprint.
    pub placement: Option<Placement>,
}

impl Occupant {
    /// What object `o` is to a placement: its kind, centre and footprint.
    pub fn of(o: &Object) -> Self {
        Self {
            kind: o.kind,
            pos: o.pos,
            placement: o.placement,
        }
    }
}

/// How a placement is checked: in a game, or for a Starting Layout.
#[derive(Copy, Clone, Debug)]
pub enum Mode<'a> {
    /// A player's Direct Placement in a game (`core/entities.md > Placement Validation > Direct Placement`): every
    /// footprint tile must be Visible to `player` in `vision`, as the latest tick left it; an Extraction Plate's too
    /// (design change 0095).
    Command { vision: &'a Vision, player: PlayerId },
    /// An object of a Starting Layout (`core/map_editor.md > Starting Layouts > Layout validation`, design change
    /// 0097): the faction's "normal in-game placement rules, minus visibility", as nothing is "visible" in the editor.
    /// The build area to reach is the layout's own ([`BuildArea::of_layout`]).
    Layout,
}

/// How the structure comes to stand (plan decision T16: one check, whose rules follow the method;
/// `core/entities.md > Placement Validation`).
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum Method {
    /// `> Direct Placement`, the GDO's: placed at once from a menu, onto tiles the player sees (in a game), within
    /// the build area for a GDO building.
    Direct,
    /// `> Warp In` (`factions/colonists/economy.md > Warp In`): the Beacon standing at `beacon` transforms in place
    /// into the structure. Every footprint tile must be Buildable, flat and unoccupied "except for the Beacon's own
    /// footprint, which will be consumed by the transformation": the tiles under `beacon` break no rule. No tile
    /// needs to be Visible (placeholder: question Q0040 point 3) and there is no build area. The footprint is
    /// centred on the Beacon ([`WholeProblem::NotOnBeacon`]).
    WarpIn { beacon: Placement },
}

/// What a placement is checked against besides the objects on the map: the ground, the build area the footprint
/// must reach, the mode and the method.
#[derive(Copy, Clone, Debug)]
pub struct Site<'a> {
    pub map: &'a Map,
    /// In a game the placing player's build area ([`crate::Sim::build_area`]), for a Starting Layout the layout's.
    pub area: &'a BuildArea,
    pub mode: Mode<'a>,
    pub method: Method,
}

/// A rule one footprint tile breaks (`core/entities.md > Placement Validation > Direct Placement`), in the order they
/// are checked. The placement ghost shows them tile by tile (design change 0093).
#[derive(Copy, Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub enum TileProblem {
    /// The tile lies off the map; nothing else is checked on it.
    OffMap,
    /// The tile's preset isn't Buildable (`core/map.md > TilePreset`; of the defaults only Plane is,
    /// `> DefaultTilePresets`).
    NotBuildable,
    /// The tile is sloped (`core/map.md > Elevation > Structures only on flat terrain`).
    NotFlat,
    /// The tile isn't Visible to the placing player (in a game, for a Direct Placement only). It is then the tile's
    /// only problem: what stands on a tile the player doesn't see, and the terrain of one they never saw, is hidden
    /// from them (`core/vision.md > VisibilityStateEnum`), and the tile is invalid anyway.
    NotVisible,
    /// Another structure's footprint covers the tile ("no existing structure overlap"), whoever owns it: a plate on
    /// the patch, for another plate.
    Structure,
    /// A map resource covers the tile, and the footprint isn't a plate going onto a patch (`core/entities.md >
    /// Placement Validation > All placement methods: no buildings on resources`, design change 0096).
    Resource,
    /// A ground unit's box overlaps the tile, whoever owns it: units are never pushed aside, and air and underground
    /// units don't count (design change 0094).
    Unit,
}

impl TileProblem {
    /// Every problem, in the order they are listed.
    pub const ALL: [TileProblem; 7] = [
        TileProblem::OffMap,
        TileProblem::NotBuildable,
        TileProblem::NotFlat,
        TileProblem::NotVisible,
        TileProblem::Structure,
        TileProblem::Resource,
        TileProblem::Unit,
    ];

    /// Its bit in a tile's set of problems.
    const fn bit(self) -> u8 {
        1 << self as u8
    }
}

/// A rule the footprint breaks as a whole, in the order they are checked.
#[derive(Copy, Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub enum WholeProblem {
    /// Only structures are placed this way, not units or map resources; nothing else is checked.
    NotAStructure,
    /// No tile of the GDO building is in the build area (`factions/gdo/structures.md > GDOBuildArea`: "at least 1
    /// grid cell of the new building must be within the current build area"). A Deployment Center seeds the area
    /// and needs none ([`super::seeds_build_area`]).
    OutsideBuildArea,
    /// The Extraction Plate isn't exactly on a Space Crystals Patch (`factions/gdo/economy.md > ExtractionFacility >
    /// AwaitingPlacement`: "Left-click Space Crystal Patch within GDO build area that does not already have a plate").
    /// A patch that has a plate is the [`TileProblem::Structure`] of that plate.
    NotOnPatch,
    /// A Warp In's footprint isn't centred on its Beacon (`factions/colonists/economy.md > Warp In > Warp In Command`:
    /// the ghost is "centered on the Beacon's location"): its centre must lie within half a tile of the Beacon's
    /// along each axis. A 4 x 4 or a 2 x 2 building has one such footprint over a 2 x 2 Beacon, a 3 x 3 one four,
    /// those that cover it (placeholder: question Q0040 point 5; which of the four the ghost shows is the client's).
    NotOnBeacon,
}

/// Everything that keeps a placement from being valid ([`check`]): nothing, if it is.
#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct PlacementCheck {
    /// Each rule each footprint tile breaks: the tiles row by row from the north-west, each one's problems in the
    /// order of [`TileProblem`]. A tile without a problem isn't listed.
    pub tiles: Vec<(TilePos, TileProblem)>,
    /// The rules the footprint breaks as a whole, in the order of [`WholeProblem`].
    pub whole: Vec<WholeProblem>,
}

impl PlacementCheck {
    /// Whether the placement breaks no rule: the ghost is green, and the building may be placed.
    pub fn is_valid(&self) -> bool {
        self.tiles.is_empty() && self.whole.is_empty()
    }

    /// The rules `tile` breaks, in order.
    pub fn problems_at(&self, tile: TilePos) -> impl Iterator<Item = TileProblem> + '_ {
        self.tiles.iter().filter(move |(t, _)| *t == tile).map(|&(_, p)| p)
    }

    /// Whether `tile` breaks no rule of its own (the footprint as a whole still may).
    pub fn tile_is_valid(&self, tile: TilePos) -> bool {
        self.problems_at(tile).next().is_none()
    }

    /// The first rule the placement breaks, for a short answer such as the command gate's
    /// (`crate::Reject::Unplaceable`): the footprint's first as a whole, else its first tile's, in the order they are
    /// listed. `None` if it is valid.
    pub fn first_problem(&self) -> Option<PlacementProblem> {
        let whole = self.whole.first().map(|&problem| PlacementProblem::Whole(problem));
        whole.or_else(|| {
            self.tiles
                .first()
                .map(|&(tile, problem)| PlacementProblem::Tile(tile, problem))
        })
    }
}

/// One rule a placement breaks ([`PlacementCheck::first_problem`]).
#[derive(Copy, Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum PlacementProblem {
    /// A rule the footprint breaks as a whole.
    Whole(WholeProblem),
    /// A rule this footprint tile breaks.
    Tile(TilePos, TileProblem),
}

/// The size `(w, h)` in tiles of `kind`'s footprint turned by `orientation`: its size in the default orientation,
/// with `w` and `h` swapped by a quarter turn (`core/entities.md > Structure Type`). `None` for units.
pub fn footprint_size(kind: ObjectKind, orientation: BuildingOrientation) -> Option<(u32, u32)> {
    let (w, h) = kind.def().tile_size()?;
    Some(if orientation.swaps_axes() { (h, w) } else { (w, h) })
}

/// `core/entities.md > Placement Validation`: everything that keeps a structure of `kind` from being placed at `site`
/// with the top-left tile of its footprint at `origin`, turned by `orientation`, among `occupants`: everything on
/// the map, the object being placed excepted (a Warp In's Beacon may be among them: its tiles break no rule). The
/// answer depends on its inputs only, whatever the occupants' order. Never panics, whatever the origin.
pub fn check(
    site: &Site<'_>,
    occupants: impl IntoIterator<Item = Occupant>,
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
    let mut found = vec![0u8; footprint.len()];
    let map = site.map;
    // `> Direct Placement`: "all footprint tiles must be in the Visible state"; no other method has the rule.
    let needs_sight = site.method == Method::Direct;
    for (i, tile) in footprint.tiles() {
        if !map.in_bounds(tile) {
            found[i] |= TileProblem::OffMap.bit();
            continue;
        }
        if !map.tile_preset(tile).is_some_and(|preset| preset.buildable) {
            found[i] |= TileProblem::NotBuildable.bit();
        }
        if def.needs_flat_ground && !map.is_flat(tile) {
            found[i] |= TileProblem::NotFlat.bit();
        }
        if let Mode::Command { vision, player } = site.mode
            && needs_sight
            && vision.state(player, tile) != VisibilityState::Visible
        {
            found[i] |= TileProblem::NotVisible.bit();
        }
    }
    for o in occupants {
        onto.see(&o);
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
                let Some(half) = ground_half_extents(o.kind) else {
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
    if let Method::WarpIn { beacon } = site.method {
        // "Except for the Beacon's own footprint, which will be consumed by the transformation."
        let consumed = TileRect::new(beacon.origin, beacon.w, beacon.h);
        footprint.clear(&mut found, |tile| overlap(consumed, TileRect::new(tile, 1, 1)));
    }
    let hidden = TileProblem::NotVisible.bit();
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
    if let Method::WarpIn { beacon } = site.method
        && !footprint.is_centred_on(&beacon)
    {
        whole.push(WholeProblem::NotOnBeacon);
    }
    PlacementCheck { tiles, whole }
}

/// [`Mode::Layout`] for object `index` of a Starting Layout (`core/map_editor.md > Starting Layouts > Layout
/// validation`, design change 0097). `layout` is the layout's objects where they stand, `others` the map's
/// pre-placed objects. The object is checked, turned as its placement is, against the map, `others` and the rest of
/// `layout`, with the area of the layout's own Deployment Center ([`BuildArea::of_layout`]). `None` for an object
/// without a footprint, a unit (the starting units' rule is the unit-start one, `paths::misplaced`), and for an
/// index past the end.
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
        method: Method::Direct,
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

/// The tiles a footprint covers, which may lie anywhere: off the map, even past the last tile coordinate.
#[derive(Copy, Clone)]
struct Footprint {
    origin: TilePos,
    w: u32,
    h: u32,
}

impl Footprint {
    /// How many tiles it covers (at most 16 × 16: `defs::tests::defs_are_consistent`).
    fn len(self) -> usize {
        self.w as usize * self.h as usize
    }

    fn rect(self) -> TileRect {
        TileRect::new(self.origin, self.w, self.h)
    }

    /// Its tiles row by row from the north-west, each with its index. Those whose coordinates would overflow are
    /// left out: the origin tile is then far off the map, so the check fails on it anyway.
    fn tiles(self) -> impl Iterator<Item = (usize, TilePos)> {
        (0..self.h)
            .flat_map(move |dy| (0..self.w).map(move |dx| (dx, dy)))
            .enumerate()
            .filter_map(move |(i, (dx, dy))| {
                let x = self.origin.x.checked_add_unsigned(dx)?;
                let y = self.origin.y.checked_add_unsigned(dy)?;
                Some((i, TilePos::new(x, y)))
            })
    }

    /// Adds `problem` to the tiles that `covered` picks.
    fn mark(self, found: &mut [u8], problem: TileProblem, covered: impl Fn(TilePos) -> bool) {
        for (i, tile) in self.tiles() {
            if covered(tile) {
                found[i] |= problem.bit();
            }
        }
    }

    /// Takes every problem off the tiles that `exempt` picks.
    fn clear(self, found: &mut [u8], exempt: impl Fn(TilePos) -> bool) {
        for (i, tile) in self.tiles() {
            if exempt(tile) {
                found[i] = 0;
            }
        }
    }

    /// Whether its centre lies within half a tile of `other`'s along each axis ([`WholeProblem::NotOnBeacon`]): in
    /// half tiles, in 64 bits, so that no footprint overflows.
    fn is_centred_on(self, other: &Placement) -> bool {
        let centre = |start: i32, len: u32| 2 * i64::from(start) + i64::from(len);
        (centre(self.origin.x, self.w) - centre(other.origin.x, other.w)).abs() <= 1
            && (centre(self.origin.y, self.h) - centre(other.origin.y, other.h)).abs() <= 1
    }
}

impl fmt::Display for TileProblem {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(match self {
            TileProblem::OffMap => "off the map",
            TileProblem::NotBuildable => "not buildable ground",
            TileProblem::NotFlat => "sloped",
            TileProblem::NotVisible => "not in sight",
            TileProblem::Structure => "under a structure",
            TileProblem::Resource => "under a resource",
            TileProblem::Unit => "under a ground unit",
        })
    }
}

impl fmt::Display for WholeProblem {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(match self {
            WholeProblem::NotAStructure => "only structures are placed",
            WholeProblem::OutsideBuildArea => "no tile is in the build area",
            WholeProblem::NotOnPatch => "an Extraction Plate goes onto a Space Crystals patch",
            WholeProblem::NotOnBeacon => "a building warps in centred on its Beacon",
        })
    }
}

impl fmt::Display for PlacementProblem {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            PlacementProblem::Whole(problem) => write!(f, "{problem}"),
            PlacementProblem::Tile(tile, problem) => write!(f, "tile ({}, {}) is {problem}", tile.x, tile.y),
        }
    }
}
