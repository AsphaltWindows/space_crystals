//! The placement rules of the Tunnel Area (slice M4-B), on the geometry of `crate::tunnel::area`:
//!
//! - **The area rule** ([`area_rule`]; `factions/syndicate/tunnels.md > Tunnel Area Non-Overlap Rule > Drillable
//!   area`, design change 0242): "a Tunnel can only be **placed** if every cell of its Tunnel Area is **on the map and
//!   Drillable** (no Water or Mountain ...). So **no Tunnel near the map edge** (closer than its area reaches)". It
//!   is checked for every kind with a tier table, at the base tier's radius (a Tunnel is placed at Tier 1), in every
//!   [`Mode`], so that the Starting Layouts, the editor and the Agent's ghost and arrival check to come (slice M4-E)
//!   share it. [`Mode::Layout`], [`Mode::Arrival`] and [`Mode::Command`] judge by the truth; [`Mode::Known`] counts an
//!   undrillable cell only on a tile the player has explored ("terrain is always known to the player as far as
//!   explored; never-explored tiles pass on the ghost, and the real check happens on arrival"), and a cell off the
//!   map always, the map's edge being known. Map spawns and debug spawns don't take the placement check, so they stay
//!   exempt, as from every placement rule but the shared tiles and [`super::Onto`]. Read as written, a Tunnel 2 tiles
//!   from the edge breaks nothing, its Tier 1 area just reaching the edge, where 0242's Verification calls it red
//!   (Placeholder: Q0078).
//! - **The expansion rule of Starting Layouts** ([`layout_method`]): a layout's underground structure lies wholly
//!   inside the Tier 1 Tunnel Area of a Tunnel of the same layout (`factions/syndicate/tunnels.md > Tunnel > Starting
//!   Conditions`: "One pre-built **Tier 1 Tunnel** containing one pre-built **Hideout**";
//!   `factions/syndicate/structures/README.md > Tunnel Expansions`: "Underground buildings constructed within a
//!   Tunnel's Tunnel Area"), the way a layout's Extraction Plates lie in the build area of its own Deployment Center.
//!   It follows the expansion method of a game ([`super::Method::Expansion`], slice M4-F) in that area: Drillable
//!   ground (design change 0239), and every structure, resource and ground unit keeps clear of it.

use super::problems::WholeProblem;
use super::site::{Method, Mode, Occupant, Site};
use crate::defs::ObjectKind;
use crate::nav::TileRect;
use crate::tunnel::{self, area::Breach};
use crate::units::TilePos;
use crate::vision::VisibilityState;

/// Design change 0242 for a structure of `kind` whose footprint is `footprint`, at `site`: [`WholeProblem::
/// TunnelAreaOffMap`] if a cell of its Tier 1 Tunnel Area lies past the map's edge, then
/// [`WholeProblem::TunnelAreaNotDrillable`] if one lies on ground that isn't Drillable and, in [`Mode::Known`], has
/// been explored (module docs). Nothing for a kind without a tier table.
pub(super) fn area_rule(site: &Site<'_>, kind: ObjectKind, footprint: TileRect) -> Vec<WholeProblem> {
    let Some(radius) = tunnel::area::base_radius(kind) else {
        return Vec::new();
    };
    let (mut off_map, mut not_drillable) = (false, false);
    for (cell, breach) in tunnel::area::breaches(site.map, tunnel::area::area(footprint, radius)) {
        match breach {
            Breach::OffMap => off_map = true,
            Breach::NotDrillable => not_drillable |= known(site.mode, cell),
        }
    }
    let mut problems = Vec::new();
    if off_map {
        problems.push(WholeProblem::TunnelAreaOffMap);
    }
    if not_drillable {
        problems.push(WholeProblem::TunnelAreaNotDrillable);
    }
    problems
}

/// Whether the ground of `cell`, on the map, is known in `mode`: always, but in [`Mode::Known`] on a tile the player
/// has never explored.
fn known(mode: Mode<'_>, cell: TilePos) -> bool {
    match mode {
        Mode::Known { vision, player } => vision.state(player, cell) != VisibilityState::Unexplored,
        Mode::Command { .. } | Mode::Arrival | Mode::Layout => true,
    }
}

/// The expansion rule of Starting Layouts (module docs): how `object`, an object of a layout whose objects are
/// `layout`, is placed. An underground structure by [`Method::Expansion`] in the Tier 1 Tunnel Area of the first Tunnel
/// of `layout` that holds it wholly; if none does, in an empty area at its origin, which holds nothing, so that it is
/// outside ([`WholeProblem::OutsideTunnelArea`](super::WholeProblem::OutsideTunnelArea)). Any other object by
/// [`Method::Direct`].
pub(super) fn layout_method(object: &Occupant, layout: &[Occupant]) -> Method {
    let underground = (object.kind.def().structure.as_ref()).is_some_and(|structure| structure.underground);
    let Some(placement) = object.placement.filter(|_| underground) else {
        return Method::Direct;
    };
    let rect = TileRect::new(placement.origin, placement.w, placement.h);
    let holding = layout.iter().find_map(|o| {
        let area = tunnel::area::base_radius(o.kind)
            .zip(o.placement)
            .map(|(radius, p)| tunnel::area::area(TileRect::new(p.origin, p.w, p.h), radius))?;
        tunnel::area::encloses(area, rect).then_some(area)
    });
    Method::Expansion {
        area: holding.unwrap_or(TileRect::new(placement.origin, 0, 0)),
    }
}
