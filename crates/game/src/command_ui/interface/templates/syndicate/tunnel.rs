//! The Tunnel's panel (`factions/syndicate/tunnel_menus.md > Tunnel ObjectInterfaceState`, slice M4-J2). Its panel
//! letters (design change 0257, answering Q0068): "**Q** Upgrade Tunnel, **W** Expand Tunnel, **E** Eject, **X** Cancel
//! (upgrade, construction or the current operation)". Q upgrades the Tunnel at once (slice M4-G), W opens the
//! ExpandMenu (`super::expand_menu`, slice M4-F), E the EjectMenu (`super::eject_menu`), and X cancels the Tunnel's
//! operation, an upgrade or an expansion (slice M4-F). No right-click.

use super::super::{InterfaceTemplate, PanelCommand, PanelTemplate};

/// The Tunnel's panel template.
pub const TEMPLATE: PanelTemplate = PanelTemplate::of(InterfaceTemplate::Tunnel);

/// `> Tunnel ObjectInterfaceState`'s panel letters, slot by slot: Q Upgrade Tunnel, which "upgrades to next tier
/// (CommandIssuingTransition)", greyed with the gate's reason ("Unavailable if already Tier 3 or if currently performing
/// an operation", the Supplies, or something blocking its larger area: `sc_sim::Sim::can_upgrade`); W Expand Tunnel,
/// which "enters ExpandMenu (StateOnlyTransition)"; E Eject, which "enters EjectMenu (StateOnlyTransition)"; and X
/// Cancel, of the Tunnel's operation, greyed while it performs none.
#[rustfmt::skip]
pub const GRID: [Option<PanelCommand>; 12] = {
    use PanelCommand::{CancelOperation, EjectMenu, ExpandMenu, UpgradeTunnel};
    [
        Some(UpgradeTunnel), Some(ExpandMenu),      Some(EjectMenu), None,
        None,                None,                  None,            None,
        None,                Some(CancelOperation), None,            None,
    ]
};
