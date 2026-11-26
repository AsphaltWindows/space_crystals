//! The rings of a Tunnel ghost (slice M4-J3; `factions/syndicate/tunnel_area_and_space.md > Tunnel Area display`,
//! design changes 0299 and 0300, answering Q0086 point 4): "**Placing a Tunnel** shows the ghost's Tier 1 area **and
//! the would-be Tier 2 and Tier 3 areas**. Cells that would stop a later upgrade are highlighted **red**: non-Drillable
//! cells, off-map cells, and cells overlapping the areas of **your own** other Tunnels [...], since all of these block
//! upgrades (Non-Overlap Rule)". The Tier 1 area is the ghost's (`super::super::ghost_area`, green or red by the check
//! its click faces); these are the rings around it.
//!
//! - **A ring** ([`Ring`], [`rings`]): the cells the area of a tier above the base adds to the area of the tier below,
//!   for each such tier of the kind's table (`sc_sim::tunnel::def_of`): Tier 2's 36 cells around Tier 1's 8 × 8, Tier
//!   3's 44 around Tier 2's 10 × 10 (`factions/syndicate/tunnels.md > Tunnel > Tiers`).
//! - **Its red cells** ([`Ring::blocked`]): those that would keep the Tunnel from upgrading to the ring's tier, by the
//!   rules of `factions/syndicate/tunnel_area_and_space.md > Tunnel Area Non-Overlap Rule`:
//!   - off the map, or not Drillable (design change 0242: "An **upgrade** is likewise blocked if the larger area would
//!     include a non-Drillable or off-map cell"; `sc_sim::tunnel::area::breaches`), as far as the player knows, as the
//!     ghost judges its own area: "never-explored tiles pass on the ghost" (`factions/syndicate/tunnels.md > Tunnel
//!     Area Non-Overlap Rule > Drillable area`; `sc_sim::placement::Mode::Known`: the ground of a tile the player has
//!     never explored is unknown, the map's edge always known);
//!   - in the area of one of the player's own other Tunnels ([`own_areas`]). A reading: the areas the ghost's own check
//!     counts against their Tunnels (`sc_sim::Sim::build_check`), each of which would block the upgrade once built: a
//!     Tunnel's at its tier, an upgrading one's at the target tier it reserves (slice M4-G), a site's (design change
//!     0289 point 3), and each pending Tunnel ghost's Tier 1 area (0289 point 2), but those of the Agent placing, which
//!     a click without Shift replaces, as the check leaves them out;
//!   - never another player's: "Enemy areas stay hidden, so an unseen enemy Tunnel can still block an upgrade".
//!
//! **The look**, placeholders until the art (Q0001): each ring's cells in its own tint ([`RING_COLORS`]), a line along
//! its outer edge, its red cells red ([`BLOCKED_FILL`]); not on the tiles the player hasn't explored, which stay black,
//! nor off the map, where the view never goes (the camera keeps to the map: `crate::camera::clamp_center`). Where a
//! ring runs off the map, a red line along the map's edge shows it instead ([`edge_marks`]), on the edge's explored
//! tiles.

use std::collections::{BTreeMap, BTreeSet, HashSet};

use bevy::prelude::*;
use sc_sim::nav::TileRect;
use sc_sim::tunnel::area::Breach;
use sc_sim::tunnel::{self, BASE_TIER};
use sc_sim::{Map, ObjectId, ObjectKind, PlayerId, Sim, TilePos};

use super::super::overlay::tint_area;
use crate::map_render::{SURFACE_LIFT_Z, surface_vertex, tile_surface};
use crate::shapes::ColorMesh;

/// The tint and the edge of each ring, Tier 2's then Tier 3's; a further tier would take the last (placeholder
/// colours).
pub const RING_COLORS: [(Color, Color); 2] = [
    (Color::srgba(1.0, 0.85, 0.30, 0.14), Color::srgba(1.0, 0.88, 0.45, 0.80)),
    (Color::srgba(0.35, 0.80, 1.0, 0.12), Color::srgba(0.50, 0.88, 1.0, 0.75)),
];
/// A cell that would block its ring's upgrade, and its edge and the map edge's line (placeholder colours).
pub const BLOCKED_FILL: Color = Color::srgba(1.0, 0.15, 0.12, 0.42);
pub const BLOCKED_EDGE: Color = Color::srgba(1.0, 0.30, 0.25, 0.90);
/// How wide the line along the map's edge is, space units, all of it inside the map.
const MARK_SU: f32 = 6.0;

/// The ring a Tunnel ghost's area would add at a later tier (module docs).
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Ring {
    /// The tier whose area it completes: 2 or 3.
    pub tier: u8,
    /// The area at that tier (`sc_sim::tunnel::area::area`), which may reach past the map's edge.
    pub area: TileRect,
    /// The area at the tier below, which it surrounds.
    pub inner: TileRect,
    /// Its cells that would block the upgrade to its tier (module docs), row by row from the north-west.
    pub blocked: Vec<TilePos>,
}

impl Ring {
    /// Whether `cell` is one of its cells.
    pub fn holds(&self, cell: TilePos) -> bool {
        tunnel::area::contains(self.area, cell) && !tunnel::area::contains(self.inner, cell)
    }

    /// Its cells, row by row from the north-west.
    pub fn cells(&self) -> impl Iterator<Item = TilePos> + '_ {
        tunnel::area::cells(self.area).filter(|&cell| self.holds(cell))
    }
}

/// The areas of `player`'s own Tunnels that a Tunnel they place must keep its later tiers' areas clear of (module docs:
/// what `Sim::build_check` counts): each of their Tunnels' and sites' area as it holds it (`sc_sim::tunnel::area::
/// reserved`: an upgrade's target tier's), then each pending Tunnel ghost's Tier 1 area (`Sim::pending_builds`), but
/// those of worker `replaced`, whose orders the click replaces.
pub fn own_areas(sim: &Sim, player: PlayerId, replaced: Option<ObjectId>) -> Vec<TileRect> {
    let standing = (sim.objects())
        .filter(|o| o.owner == Some(player))
        .filter_map(tunnel::area::reserved);
    let pending = sim.pending_builds(player).into_iter().filter_map(|pending| {
        let radius = tunnel::area::base_radius(pending.kind).filter(|_| Some(pending.worker) != replaced)?;
        let site = pending.site;
        Some(tunnel::area::area(TileRect::new(site.origin, site.w, site.h), radius))
    });
    standing.chain(pending).collect()
}

/// The rings of a ghost of `kind` whose footprint is `footprint` (module docs), Tier 2's first; none for a kind
/// without a tier table. `known` says whether the player knows the ground of a tile on the map (has explored it), and
/// `own` holds the areas of their own other Tunnels ([`own_areas`]).
pub fn rings(
    map: &Map,
    kind: ObjectKind,
    footprint: TileRect,
    known: impl Fn(TilePos) -> bool,
    own: &[TileRect],
) -> Vec<Ring> {
    let Some(def) = tunnel::def_of(kind) else {
        return Vec::new();
    };
    let areas: Vec<(u8, TileRect)> = (BASE_TIER..)
        .zip(def.tiers)
        .map(|(tier, row)| (tier, tunnel::area::area(footprint, row.area_radius)))
        .collect();
    let ring = |pair: &[(u8, TileRect)]| {
        let (inner, (tier, area)) = (pair[0].1, pair[1]);
        let breaches: BTreeMap<TilePos, Breach> = tunnel::area::breaches(map, area).collect();
        let blocks = |cell: TilePos| {
            let ground = match breaches.get(&cell) {
                Some(Breach::OffMap) => true,
                Some(Breach::NotDrillable) => known(cell),
                None => false,
            };
            ground || own.iter().any(|&other| tunnel::area::contains(other, cell))
        };
        let mut ring = Ring {
            tier,
            area,
            inner,
            blocked: Vec::new(),
        };
        ring.blocked = ring.cells().filter(|&cell| blocks(cell)).collect();
        ring
    };
    areas.windows(2).map(ring).collect()
}

/// A side of a tile.
#[derive(Copy, Clone, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub enum Side {
    North,
    East,
    South,
    West,
}

/// Where the red cells of `rings` off the map meet its edge (module docs): the side of each tile along the edge that
/// faces one of them, straight across the edge, a corner tile's two sides for a cell beyond the corner. In tile order.
pub fn edge_marks(map: &Map, rings: &[Ring]) -> BTreeSet<(TilePos, Side)> {
    let (w, h) = (i64::from(map.width()), i64::from(map.height()));
    let mut marks = BTreeSet::new();
    let off = (rings.iter()).flat_map(|ring| ring.blocked.iter().copied().filter(|&cell| !map.in_bounds(cell)));
    for cell in off {
        let (x, y) = (i64::from(cell.x), i64::from(cell.y));
        let at = |v: i64, len: i64| i32::try_from(v.clamp(0, len - 1)).unwrap_or(0);
        let tile = TilePos::new(at(x, w), at(y, h));
        let sides = [
            (y < 0, Side::North),
            (x >= w, Side::East),
            (y >= h, Side::South),
            (x < 0, Side::West),
        ];
        for (_, side) in sides.into_iter().filter(|&(beyond, _)| beyond) {
            marks.insert((tile, side));
        }
    }
    marks
}

/// Adds `rings` to `mesh` (module docs): each ring's cells that are `drawn` in its tint, its red cells red
/// ([`tint_area`], each with its edge along the ring's outer edge only), then the red line along the map's edge where a
/// ring runs off it ([`edge_marks`]), on the edge's tiles that are `drawn`. A cell off the map is never drawn.
pub fn draw(mesh: &mut ColorMesh, map: &Map, rings: &[Ring], drawn: impl Fn(TilePos) -> bool) {
    let shown = |t: TilePos| map.in_bounds(t) && drawn(t);
    for (i, ring) in rings.iter().enumerate() {
        let colors = RING_COLORS[i.min(RING_COLORS.len() - 1)];
        let blocked: HashSet<TilePos> = ring.blocked.iter().copied().collect();
        let cells = || tunnel::area::cells(ring.area);
        let open = |t: TilePos| ring.holds(t) && !blocked.contains(&t) && shown(t);
        tint_area(mesh, map, cells(), open, colors);
        let red = |t: TilePos| blocked.contains(&t) && shown(t);
        tint_area(mesh, map, cells(), red, (BLOCKED_FILL, BLOCKED_EDGE));
    }
    for (tile, side) in edge_marks(map, rings) {
        if shown(tile) {
            mark(mesh, map, tile, side);
        }
    }
}

/// A red line [`MARK_SU`] wide along `side` of tile `t`, inside the tile, on the terrain's surface.
fn mark(mesh: &mut ColorMesh, map: &Map, t: TilePos, side: Side) {
    let [nw, ne, se, sw, center] =
        tile_surface(map, t).map(|(ground, height)| surface_vertex(ground, height, SURFACE_LIFT_Z));
    let (a, b) = match side {
        Side::North => (nw, ne),
        Side::East => (ne, se),
        Side::South => (se, sw),
        Side::West => (sw, nw),
    };
    let inward = (center - (a + b) / 2.0).truncate().normalize_or_zero().extend(0.0) * MARK_SU;
    mesh.quad([a, b, b + inward, a + inward], BLOCKED_EDGE);
}
