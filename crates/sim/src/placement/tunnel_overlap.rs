//! The non-overlap rule of Tunnel Areas (slice M4-E), `factions/syndicate/tunnels.md > Tunnel Area Non-Overlap Rule`:
//! "The non-overlap check uses the **current tier's area**", and "**Every player's Tunnels count**" (design change
//! 0237). A Tunnel is placed at Tier 1, so its Tier 1 area must overlap no other Tunnel's area at that Tunnel's tier:
//!
//! - **Which Tunnels**: every Tunnel among the occupants, whoever owns it, a site included (design change 0289 point
//!   3: "A site counts with its whole area for every player from the moment it appears"), at its tier as the check
//!   knows it (`Occupant::tier`): its tier in a game on the truth ([`super::Mode::Arrival`]); in
//!   [`super::Mode::Known`] the caller gives what the player knows, so "**The ghost** judges enemy Tunnels by what the
//!   player knows: their remembered position and their tier as last seen", and "The real check on arrival uses the
//!   truth, so a build can fail on arrival because an enemy Tunnel was upgraded unseen". A Tunnel of a map or of a
//!   Starting Layout counts at the base tier, where it starts.
//! - **The placing player's lingering Tunnel ghosts** (design change 0289 point 2: "A pending ghost reserves its whole
//!   Tier 1 area against its owner's further Tunnel ghosts, which show red. It is the owner's only, since other
//!   players don't know about it"): each reserved site of a Tunnel reserves its Tier 1 area, as its footprint reserves
//!   its tiles (`TileProblem::Reserved`).
//!
//! It is checked for every kind with a tier table, in every mode, as the area rule of slice M4-B is
//! (`placement/tunnel_area.rs`). It is a rule of the footprint as a whole ([`WholeProblem::TunnelAreaOverlap`]): the
//! ghost turns red throughout, and never-explored tiles don't hide it, as the Tunnels it counts are known.

use super::problems::WholeProblem;
use super::site::Occupant;
use crate::defs::ObjectKind;
use crate::nav::TileRect;
use crate::tunnel::{self, BASE_TIER};

/// The rule for a placement of one kind: the Tunnel Areas it must keep clear of, gathered from the occupants and the
/// reserved sites as the check walks them ([`Overlap::see`]), then judged ([`Overlap::rule`]). It gathers nothing for
/// a kind without a tier table, so that every other placement pays only a branch per occupant.
pub(super) struct Overlap {
    /// The placed kind's Tier 1 area radius; `None` if it has no tier table, and so no Tunnel Area.
    radius: Option<u32>,
    /// The areas seen so far.
    areas: Vec<TileRect>,
}

impl Overlap {
    /// The rule for a structure of `kind`.
    pub(super) fn new(kind: ObjectKind) -> Self {
        Self {
            radius: tunnel::area::base_radius(kind),
            areas: Vec::new(),
        }
    }

    /// Takes in occupant or reserved site `o`: its Tunnel Area at its tier as the check knows it (`Occupant::tier`, the
    /// base tier when none is known), if it is a placed Tunnel whose tier the table has.
    pub(super) fn see(&mut self, o: &Occupant) {
        if self.radius.is_none() {
            return;
        }
        let Some(row) = tunnel::def_of(o.kind).and_then(|def| def.tier(o.tier.unwrap_or(BASE_TIER))) else {
            return;
        };
        if let Some(placement) = o.placement {
            let footprint = TileRect::new(placement.origin, placement.w, placement.h);
            self.areas.push(tunnel::area::area(footprint, row.area_radius));
        }
    }

    /// Design change 0237 for the structure whose footprint is `footprint`: [`WholeProblem::TunnelAreaOverlap`] if its
    /// Tier 1 Tunnel Area overlaps one of the areas seen, the other Tunnels' and the reserved ones (module docs).
    /// Nothing for a kind without a tier table.
    pub(super) fn rule(&self, footprint: TileRect) -> Option<WholeProblem> {
        let area = tunnel::area::area(footprint, self.radius?);
        (self.areas.iter().any(|&other| tunnel::area::overlaps(area, other))).then_some(WholeProblem::TunnelAreaOverlap)
    }
}
