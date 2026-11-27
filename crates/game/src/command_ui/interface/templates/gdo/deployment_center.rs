//! The Deployment Center's panel (`factions/gdo/structures/deployment_center.md >
//! ObjectInterfaceState[DeploymentCenter]`): its DefaultState opens the build menus ([`BuildMenu`], design change
//! 0104), whose entries construct; once a building is ready, its entry enters AwaitingPlacement
//! (`super::super::state_grid`), and so does its DefaultState's E: Place ([`PLACE_SLOT`], design change 0388). It
//! produces no units and has no right-click.

use sc_sim::ObjectKind;

use super::super::{BuildMenu, InterfaceTemplate, PanelCommand, PanelTemplate};

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

/// `> BuildMenu[Basic]`, slot by slot: Q Generator, W Barracks, E Extraction Facility (design change 0104), "**[S]
/// Arsenal**" (slice M5-K, design change 0356) and "**[D] Career Campus**" (slice M5-N9, design change 0328), the
/// last two greyed while the player owns no Barracks (their TechPrerequisite: the gate's `Reject::Requires`, "Requires
/// a Barracks"). Its R Supply Tower, A Radar Station and F Defense System aren't in the game yet (their slices bring
/// them), so their slots stay hidden: "The other slots can show greyed placeholders, or be hidden until those
/// buildings exist". Z is Back and X the constructor's Cancel (`super::super::state_grid`).
#[rustfmt::skip]
const BASIC_MENU: [Option<ObjectKind>; 12] = {
    use ObjectKind::{Arsenal, Barracks, CareerCampus, ExtractionFacility, Generator};
    [
        Some(Generator), Some(Barracks), Some(ExtractionFacility), None,
        None,            Some(Arsenal),  Some(CareerCampus),       None,
        None,            None,           None,                     None,
    ]
};

/// `> BuildMenu[Advanced]`: Q Factory, W Advanced Mechanics, E Airfield, R Propulsion Labs and A Industrial
/// Powerplant, none of which is in the game yet: every slot hidden, and the menu unavailable (`super::super::offers`).
const ADVANCED_MENU: [Option<ObjectKind>; 12] = [None; 12];

/// The entries of the build menu `menu`, slot by slot (`CommandSlot::all` order): the buildings it constructs. None for
/// the Field Engineer's Construct menu, which the Deployment Center hasn't.
pub fn menu(menu: BuildMenu) -> &'static [Option<ObjectKind>; 12] {
    match menu {
        BuildMenu::Basic => &BASIC_MENU,
        BuildMenu::Advanced => &ADVANCED_MENU,
        BuildMenu::Construct => &[None; 12],
    }
}
