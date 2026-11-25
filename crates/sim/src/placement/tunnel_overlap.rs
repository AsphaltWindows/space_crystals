//! The non-overlap rule of Tunnel Areas (slice M4-E), `factions/syndicate/tunnels.md > Tunnel Area Non-Overlap Rule`:
//! "The non-overlap check uses the **current tier's area**", and "**Every player's Tunnels count**" (design change
//! 0237). A Tunnel is placed at Tier 1, so its Tier 1 area must overlap no other Tunnel's area at that Tunnel's tier:
//!
//! - **Which Tunnels**: every Tunnel among the occupants, whoever owns it, a site included (question Q0079 point 3:
//!   "It counts like any Tunnel's, for every player, from the moment the site appears"), at its tier as the check
//!   knows it (`Occupant::tier`): its tier in a game on the truth ([`super::Mode::Arrival`]); in
//!   [`super::Mode::Known`] the caller gives what the player knows, so "**The ghost** judges enemy Tunnels by what the
//!   player knows: their remembered position and their tier as last seen", and "The real check on arrival uses the
//!   truth, so a build can fail on arrival because an enemy Tunnel was upgraded unseen". A Tunnel of a map or of a
//!   Starting Layout counts at the base tier, where it starts.
//! - **The placing player's lingering Tunnel ghosts** (Placeholder: Q0079 point 2, "The ghost reserves its whole
//!   Tier 1 area against its owner's further Tunnel ghosts; the ghost shows red over it"): each reserved site of a
//!   Tunnel reserves its Tier 1 area, as its footprint reserves its tiles (`TileProblem::Reserved`).
//!
//! It is checked for every kind with a tier table, in every mode, as the area rule of slice M4-B is
//! (`placement/tunnel_area.rs`). It is a rule of the footprint as a whole ([`WholeProblem::TunnelAreaOverlap`]): the
//! ghost turns red throughout, and never-explored tiles don't hide it, as the Tunnels it counts are known.

use super::problems::WholeProblem;
use super::site::Occupant;
use crate::defs::ObjectKind;
use crate::nav::TileRect;
use crate::tunnel::{self, BASE_TIER};

/// The Tunnel Area of occupant `o` at its tier as the check knows it (`Occupant::tier`, the base tier when none is
/// known): `None` for an occupant that isn't a placed Tunnel, or whose tier the table hasn't.
pub(super) fn area_of(o: &Occupant) -> Option<TileRect> {
    let row = tunnel::def_of(o.kind)?.tier(o.tier.unwrap_or(BASE_TIER))?;
    let placement = o.placement?;
    let footprint = TileRect::new(placement.origin, placement.w, placement.h);
    Some(tunnel::area::area(footprint, row.area_radius))
}

/// Design change 0237 for a structure of `kind` whose footprint is `footprint`: [`WholeProblem::TunnelAreaOverlap`] if
/// its Tier 1 Tunnel Area overlaps one of `areas`, the other Tunnels' and the reserved ones (module docs). Nothing for
/// a kind without a tier table.
pub(super) fn overlap_rule(kind: ObjectKind, footprint: TileRect, areas: &[TileRect]) -> Option<WholeProblem> {
    let area = tunnel::area::area(footprint, tunnel::area::base_radius(kind)?);
    (areas.iter().any(|&other| tunnel::area::overlaps(area, other))).then_some(WholeProblem::TunnelAreaOverlap)
}
