//! The Tunnel's panel (`factions/syndicate/tunnels.md > Tunnel ObjectInterfaceState`, slice M4-J2). Its panel letters
//! (design change 0257, answering Q0068): "**Q** Upgrade Tunnel, **W** Expand Tunnel, **E** Eject, **X** Cancel
//! (upgrade, construction or the current operation)". W opens the ExpandMenu (`super::expand_menu`, slice M4-F), E the
//! EjectMenu (`super::eject_menu`), and X cancels the Tunnel's operation (slice M4-F); Q holds its slot, greyed, until
//! slice M4-G's upgrade replaces it ([`Planned`]). No right-click.

use super::super::{InterfaceTemplate, PanelCommand, PanelTemplate, Planned};

/// The Tunnel's panel template.
pub const TEMPLATE: PanelTemplate = PanelTemplate::of(InterfaceTemplate::Tunnel);

/// `> Tunnel ObjectInterfaceState`'s panel letters, slot by slot: Q Upgrade Tunnel, a placeholder greyed with why
/// (`Planned::why`: not in the game yet); W Expand Tunnel, which "enters ExpandMenu (StateOnlyTransition)"; E Eject,
/// which "enters EjectMenu (StateOnlyTransition)"; and X Cancel, of the Tunnel's operation, greyed while it performs
/// none.
#[rustfmt::skip]
pub const GRID: [Option<PanelCommand>; 12] = {
    use PanelCommand::{CancelOperation, EjectMenu, ExpandMenu, Planned as P};
    use Planned::UpgradeTunnel;
    [
        Some(P(UpgradeTunnel)), Some(ExpandMenu),      Some(EjectMenu), None,
        None,                   None,                  None,            None,
        None,                   Some(CancelOperation), None,            None,
    ]
};
