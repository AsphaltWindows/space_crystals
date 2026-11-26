//! The panel of a site under construction (`sc_sim::Object::is_site`, slice M4-J1), whatever its type: a Tunnel an
//! Agent builds (`factions/syndicate/units/agent.md > While the Tunnel is under construction`: "It does nothing but
//! Cancel: [...] the only command is X Cancel Construction (as for a Colonists building being transformed)", design
//! change 0237), an expansion a Tunnel builds (slice M4-F), its Laser Turret among them (slice M4-S2), and a Cults
//! building its Recruits build (slice M5-E). It is keyed on the site, not on its kind: the unfinished objects of a
//! type form a group of their own (design change 0185, `crate::selection::GroupType`), and a site's is this panel
//! (`super::super::Conditions::site`). (M5-S6) A building that will produce or release units adds its C, Set Rally
//! Point, on a site and while it transforms alike ([`unfinished_grid`], design change 0396).

use sc_sim::ObjectKind;
use sc_sim::production::takes_rally;

use super::{PanelCommand, TRANSFORMING_GRID, slot_index};
use crate::input::CommandSlot;

/// `factions/syndicate/units/agent.md > Cancel Construction`: "**X: Cancel Construction**: cancels the in-progress
/// initial build (CommandIssuingTransition)", and nothing else (design change 0237); a building that will produce or
/// release units adds its C ([`unfinished_grid`], design change 0396).
#[rustfmt::skip]
pub const SITE_GRID: [Option<PanelCommand>; 12] = [
    None, None,                                   None, None,
    None, None,                                   None, None,
    None, Some(PanelCommand::CancelConstruction), None, None,
];

/// The panel of an unfinished building of type `kind`: a site's ([`SITE_GRID`], `site`) or a transforming building's
/// ([`TRANSFORMING_GRID`]), its Cancel in X, and (M5-S6) for a building that will produce or release units
/// (`sc_sim::production::takes_rally`, the gate's rule) its C (design change 0396: "Its unfinished panel adds **C: Set
/// Rally Point**, next to Cancel"), which enters AwaitingTarget\[SetRallyPoint\] as on the finished building.
pub fn unfinished_grid(kind: ObjectKind, site: bool) -> [Option<PanelCommand>; 12] {
    let mut grid = if site { SITE_GRID } else { TRANSFORMING_GRID };
    if takes_rally(kind) {
        grid[slot_index(CommandSlot::RALLY_POINT)] = Some(PanelCommand::SetRallyPoint);
    }
    grid
}
