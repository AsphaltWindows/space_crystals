//! The Beacon's panel (`factions/colonists/economy.md > Beacon > Beacon ObjectInterfaceState`, design change 0124):
//! its DefaultState opens the two Warp In menus, whose entries enter the Warp In's placement on the Beacon. No
//! right-click.

use sc_sim::ObjectKind;

use super::super::{BuildMenu, InterfaceTemplate, PanelCommand, PanelTemplate};
use super::Planned;

/// The Beacon's panel template, with the commands the design names after it (`PanelCommand::name_on`): its menus,
/// "Q: Warp In (Basic)" and "W: Warp In (Advanced)", the Deployment Center's being "Basic Construction" and
/// "Advanced Construction"; and the Cancel of a Beacon under deployment (`factions/colonists/economy.md > Beacon
/// Deployment > While transforming: only Cancel`: "Cancel Deployment / Cancel Warp In"), which is a Beacon from the
/// first frame (`sc_sim::transform`).
pub const TEMPLATE: PanelTemplate = PanelTemplate {
    names: &[
        (PanelCommand::BuildMenu(BuildMenu::Basic), "Warp In (Basic)"),
        (PanelCommand::BuildMenu(BuildMenu::Advanced), "Warp In (Advanced)"),
        (PanelCommand::CancelTransform, "Cancel Deployment"),
    ],
    ..PanelTemplate::of(InterfaceTemplate::Beacon)
};

/// `> Beacon ObjectInterfaceState > DefaultState`: "Q: Warp In (Basic)", "W: Warp In (Advanced)", the two menus
/// "like the Deployment Center's two build menus" (`PanelCommand::BuildMenu`, opened in `InterfaceState::BuildMenu`).
#[rustfmt::skip]
pub const GRID: [Option<PanelCommand>; 12] = {
    use PanelCommand::BuildMenu as Menu;
    [
        Some(Menu(BuildMenu::Basic)), Some(Menu(BuildMenu::Advanced)), None, None,
        None,                         None,                            None, None,
        None,                         None,                            None, None,
    ]
};

/// `> Beacon ObjectInterfaceState > Warp In (Basic)`, "economy and production": Q Egress Node, W Robo Portal, E Bio
/// Portal, the buildings in the game, and A Alloy Furnace, S Essence Refinery, D Flux Reactor, which aren't yet; Z is
/// Back (`crate::command_ui::interface::panel`).
#[rustfmt::skip]
const WARP_IN_BASIC: [Option<PanelCommand>; 12] = {
    use ObjectKind::{BioPortal, EgressNode, RoboPortal};
    use PanelCommand::{Planned as P, WarpIn as W};
    use Planned::{AlloyFurnace, EssenceRefinery, FluxReactor};
    [
        Some(W(EgressNode)),   Some(W(RoboPortal)),      Some(W(BioPortal)),   None,
        Some(P(AlloyFurnace)), Some(P(EssenceRefinery)), Some(P(FluxReactor)), None,
        None,                  None,                     None,                 None,
    ]
};

/// `> Beacon ObjectInterfaceState > Warp In (Advanced)`, "tech and defense (each defense next to the building it
/// requires)": Q Exophysics Engine, W Skylance, E Fusion Core, R Plasma Turret, A Relay Array, S Navigation Spire, D
/// Psionic Amplifier, F Structural Analyzer, none of which is in the game yet; Z is Back.
#[rustfmt::skip]
const WARP_IN_ADVANCED: [Option<PanelCommand>; 12] = {
    use PanelCommand::Planned as P;
    use Planned::{
        ExophysicsEngine, FusionCore, NavigationSpire, PlasmaTurret, PsionicAmplifier, RelayArray, Skylance,
        StructuralAnalyzer,
    };
    [
        Some(P(ExophysicsEngine)), Some(P(Skylance)),        Some(P(FusionCore)),       Some(P(PlasmaTurret)),
        Some(P(RelayArray)),       Some(P(NavigationSpire)), Some(P(PsionicAmplifier)), Some(P(StructuralAnalyzer)),
        None,                      None,                     None,                      None,
    ]
};

/// A Beacon's Warp In menu `menu`, slot by slot: every building of the design's menu shows, the ones not in the game
/// unavailable (`PanelCommand::Planned`).
pub fn warp_in_menu(menu: BuildMenu) -> [Option<PanelCommand>; 12] {
    match menu {
        BuildMenu::Basic => WARP_IN_BASIC,
        BuildMenu::Advanced => WARP_IN_ADVANCED,
    }
}
