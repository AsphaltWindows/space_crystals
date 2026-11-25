//! The panel of a site under construction (`sc_sim::Object::is_site`, slice M4-J1), whatever its type: a Tunnel an
//! Agent builds (`factions/syndicate/units/agent.md > While the Tunnel is under construction`: "It does nothing but
//! Cancel: [...] the only command is X Cancel Construction (as for a Colonists building being transformed)", design
//! change 0237), an expansion a Tunnel builds (slice M4-F), and a Laser Turret an Agent builds (slice M4-S1). It is
//! keyed on the site, not on its kind: the unfinished objects of a type form a group of their own (design change
//! 0185, `crate::selection::GroupType`), and a site's is this panel (`super::super::Conditions::site`).

use super::PanelCommand;

/// `factions/syndicate/units/agent.md > Cancel Construction`: "**X: Cancel Construction**: cancels the in-progress
/// initial build (CommandIssuingTransition)", and nothing else (design change 0237).
#[rustfmt::skip]
pub const SITE_GRID: [Option<PanelCommand>; 12] = [
    None, None,                                   None, None,
    None, None,                                   None, None,
    None, Some(PanelCommand::CancelConstruction), None, None,
];
