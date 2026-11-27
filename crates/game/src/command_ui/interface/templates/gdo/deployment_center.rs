//! The Deployment Center's panel (`factions/gdo/structures/deployment_center.md >
//! ObjectInterfaceState[DeploymentCenter]`): its DefaultState opens the build menus ([`BuildMenu`], design change
//! 0104), whose entries construct; once a building is ready, its entry enters AwaitingPlacement
//! (`super::super::state_grid`), and so does its DefaultState's E: Place ([`PLACE_SLOT`], design change 0388). It
//! produces no units and has no right-click.

use super::super::{BuildMenu, InterfaceTemplate, PanelCommand, PanelTemplate, Planned};

/// The Deployment Center's panel template.
pub const TEMPLATE: PanelTemplate = PanelTemplate::of(InterfaceTemplate::DeploymentCenter);

/// `> ObjectInterfaceState[DeploymentCenter] > DefaultState commands` (design change 0104): "Q: Basic Construction"
/// and "W: Advanced Construction", which open the build menus. Its E, Place ([`PLACE_SLOT`]), and its X, Cancel
/// Construction or Cancel Ready Building, show only while it constructs or holds a building ready
/// (`super::super::state_grid`).
#[rustfmt::skip]
pub const GRID: [Option<PanelCommand>; 12] = {
    use PanelCommand::BuildMenu as Menu;
    [
        Some(Menu(BuildMenu::Basic)), Some(Menu(BuildMenu::Advanced)), None, None,
        None,                         None,                            None, None,
        None,                         None,                            None, None,
    ]
};

/// `> ObjectInterfaceState[DeploymentCenter] > DefaultState commands` (design change 0388): "**E: Place** (user,
/// 2026-11-27; letter: designer placeholder) [...] Shown whenever CurrentConstruction or ReadyToPlace is set, with the
/// building's icon" (`PanelCommand::Place`), the index of its slot in [`GRID`].
pub const PLACE_SLOT: usize = 2;

/// `> BuildMenu[Basic]`, slot by slot: "[Q] Generator [W] Barracks [E] Extraction Facility [R] Supply Tower / [A]
/// Radar Station [S] Arsenal [D] Career Campus [F] Defense System". The Generator, the Barracks and the Extraction
/// Facility (design change 0104), the Arsenal (slice M5-K, design change 0356) and the Career Campus (slice M5-N9,
/// design change 0328) it constructs, the last two greyed while the player owns no Barracks (their TechPrerequisite:
/// the gate's `Reject::Requires`, "Requires a Barracks"). (M6-GK) The Supply Tower, the Radar Station and the Defense
/// System are in the game (plan decision T84), but no player constructs one yet: each holds its slot, greyed
/// ([`Planned`]), until its slice makes its entry live (M6-GS the Supply Tower's, M6-GR the other two's). Z is Back and
/// X the constructor's Cancel (`super::super::state_grid`).
#[rustfmt::skip]
const BASIC_MENU: [Option<PanelCommand>; 12] = {
    use sc_sim::ObjectKind::{Arsenal, Barracks, CareerCampus, ExtractionFacility, Generator};
    use Planned::{DefenseSystem, RadarStation, SupplyTower};
    use PanelCommand::{Construct as C, Planned as P};
    [
        Some(C(Generator)),    Some(C(Barracks)), Some(C(ExtractionFacility)), Some(P(SupplyTower)),
        Some(P(RadarStation)), Some(C(Arsenal)),  Some(C(CareerCampus)),       Some(P(DefenseSystem)),
        None,                  None,              None,                         None,
    ]
};

/// `> BuildMenu[Advanced]` ("the Factory and everything that follows it"), slot by slot: "[Q] Factory [W] Advanced
/// Mechanics [E] Airfield [R] Propulsion Labs / [A] Industrial Powerplant". (M6-GK) All five are in the game (plan
/// decision T84), but no player constructs one yet: each holds its slot, greyed ([`Planned`]), until its slice makes
/// its entry live (M6-GF the Factory's, Advanced Mechanics' and the Industrial Powerplant's, M6-GH1 the Airfield's,
/// M6-GL Propulsion Labs'). The menu opens on them (`super::super::offers`).
#[rustfmt::skip]
const ADVANCED_MENU: [Option<PanelCommand>; 12] = {
    use Planned::{AdvancedMechanics, Airfield, Factory, IndustrialPowerplant, PropulsionLabs};
    use PanelCommand::Planned as P;
    [
        Some(P(Factory)),              Some(P(AdvancedMechanics)), Some(P(Airfield)), Some(P(PropulsionLabs)),
        Some(P(IndustrialPowerplant)), None,                       None,              None,
        None,                          None,                       None,              None,
    ]
};

/// The entries of the build menu `menu`, slot by slot (`CommandSlot::all` order): the buildings it constructs, and
/// (M6-GK) those it will, greyed. None for the Field Engineer's Construct menu, which the Deployment Center hasn't.
pub fn menu(menu: BuildMenu) -> &'static [Option<PanelCommand>; 12] {
    match menu {
        BuildMenu::Basic => &BASIC_MENU,
        BuildMenu::Advanced => &ADVANCED_MENU,
        BuildMenu::Construct => &[None; 12],
    }
}
