//! The Tunnel's panel (`factions/syndicate/tunnels.md > Tunnel ObjectInterfaceState`, slice M4-J2). Its panel letters
//! (design change 0257, answering Q0068): "**Q** Upgrade Tunnel, **W** Expand Tunnel, **E** Eject, **X** Cancel
//! (upgrade, construction or the current operation)". E opens the EjectMenu (`super::eject_menu`); the others hold
//! their slots, greyed, until the slices that bring their commands replace them ([`Planned`]: Q slice M4-G's upgrade,
//! W slice M4-F's ExpandMenu, X the Cancel of their operations). No right-click.

use super::super::{InterfaceTemplate, PanelCommand, PanelTemplate, Planned};

/// The Tunnel's panel template.
pub const TEMPLATE: PanelTemplate = PanelTemplate::of(InterfaceTemplate::Tunnel);

/// `> Tunnel ObjectInterfaceState`'s panel letters, slot by slot: Q Upgrade Tunnel, W Expand Tunnel and X Cancel,
/// placeholders greyed with why (`Planned::why`: not in the game yet, and nothing to cancel), and E Eject, which
/// "enters EjectMenu (StateOnlyTransition)".
#[rustfmt::skip]
pub const GRID: [Option<PanelCommand>; 12] = {
    use PanelCommand::{EjectMenu, Planned as P};
    use Planned::{CancelOperation, ExpandTunnel, UpgradeTunnel};
    [
        Some(P(UpgradeTunnel)), Some(P(ExpandTunnel)),    Some(EjectMenu), None,
        None,                   None,                     None,            None,
        None,                   Some(P(CancelOperation)), None,            None,
    ]
};
