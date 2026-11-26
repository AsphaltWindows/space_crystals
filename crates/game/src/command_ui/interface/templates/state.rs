//! A panel that follows its constructor's state and the build menu that is open ([`state_grid`]): the Deployment
//! Center's build menus, a Beacon's Warp In menus, and a constructor's building ready to place and its Cancels
//! (`factions/gdo/structures/deployment_center.md > ObjectInterfaceState[DeploymentCenter]`; a constructor is alone in
//! its SelectionGroup: `systems/controls/selection.md > SelectionGroup`).

use sc_sim::ObjectKind;
use sc_sim::construction::{self, Constructor};

use super::{
    ATTACK_SLOT, BuildMenu, InterfaceState, InterfaceTemplate, PanelCommand, colonists, cults, gdo, grid, hides_attack,
    slot_index, syndicate,
};
use crate::command_ui::interface::Builder;
use crate::input::CommandSlot;

/// Whether objects of `kind` construct buildings (`StructureDef::constructs`): the Deployment Center and the
/// Extraction Facility.
pub(super) fn constructs(kind: ObjectKind) -> bool {
    !construction::recipes(kind).is_empty()
}

/// The entries of a `kind` object's build menu `menu`, slot by slot (`CommandSlot::all` order): a Beacon's Warp In
/// menu (`factions/colonists/economy.md > Beacon > Beacon ObjectInterfaceState`: every building of the design's
/// menu, those not in the game unavailable), a Recruit's Construct menu (slice M5-J1, the same:
/// `cults::construct_menu`), a Field Engineer's (slice M5-N9, its buildings not in the game yet:
/// `gdo::field_engineer::menu`), else the Deployment Center's menu, its buildings to construct.
pub fn menu_grid(kind: ObjectKind, menu: BuildMenu) -> [Option<PanelCommand>; 12] {
    if colonists::warps_in(kind) {
        colonists::beacon::warp_in_menu(menu)
    } else if InterfaceTemplate::of(kind) == Some(InterfaceTemplate::Recruit) {
        cults::construct_menu::grid(menu)
    } else if InterfaceTemplate::of(kind) == Some(InterfaceTemplate::FieldEngineer) {
        gdo::field_engineer::menu(menu)
    } else {
        menu.entries().map(|entry| entry.map(PanelCommand::Construct))
    }
}

/// The build menu of a `kind` object whose `building` is placed (the Deployment Center's, holding it ready; a
/// Beacon's, warping it in; a Recruit's, building it), if it has one.
pub fn menu_of(kind: ObjectKind, building: ObjectKind) -> Option<BuildMenu> {
    BuildMenu::ALL.into_iter().find(|&menu| {
        grid(kind).contains(&Some(PanelCommand::BuildMenu(menu)))
            && menu_grid(kind, menu)
                .iter()
                .flatten()
                .any(|entry| entry.building() == Some(building))
    })
}

/// The commands a `kind` ActiveGroup shows in `interface`, slot by slot (`CommandSlot::all` order), while its
/// constructor, if it is one, is `constructor` (the ActiveGroup is then a single instance: constructors are
/// Ungroupable, `systems/controls/selection.md > SelectionGroup`, so the panel "can reliably read that
/// instance's specific game state"):
/// - DefaultState and AwaitingTarget: its DefaultState grid ([`grid`]), but the Peacekeeper (Shielded)'s Attack
///   ([`hides_attack`]);
/// - a build menu: the menu's entries ([`menu_grid`]); AwaitingPlacement: those of the state it goes back to
///   (`InterfaceState::back`): the Deployment Center's menu of the building, a Beacon's Warp In menu of it, the
///   Extraction Facility's and the Drone's DefaultState;
/// - the EjectMenu: its tiles (`syndicate::eject_menu`), on a type whose DefaultState opens it, the Tunnel; the
///   ExpandMenu alike, its entries (`syndicate::expand_menu`), and in an expansion's AwaitingPlacement, which goes
///   back to it; a KindMenu its entries (`cults::armory::kind_menu`), on a pool building; an AugmentMenu its tiles or
///   its entries (`cults::altar::type_menu`, `cults::altar::state_menu`), on an Altar with a tile for its type;
/// - a constructor's building ready to place takes its entry's slot, as "Select completed building" (design change
///   0104; the Extraction Facility's "Q: Place Plate"), and its X is Cancel Construction while it constructs and Cancel
///   Ready Building while it holds a building ready, and empty while it is idle
///   (`factions/gdo/structures/deployment_center.md > DefaultState commands`: "Only visible when CurrentConstruction is
///   set", "Only visible when ReadyToPlace is set");
/// - a build menu shows its other entries while the constructor is busy, unavailable
///   (`factions/gdo/structures/deployment_center.md > BuildMenu (instance constructing)`: "Other build options
///   unavailable": [`super::Conditions`] disables them), but the Extraction Facility's DefaultState lists only X while
///   it constructs and only Q Place Plate and X while its plate is ready (`factions/gdo/economy.md >
///   ExtractionFacility > ObjectInterfaceState[ExtractionFacility]`), so a building to construct in a DefaultState grid
///   shows only while the constructor is idle.
pub fn state_grid(
    kind: ObjectKind,
    interface: InterfaceState,
    constructor: Option<Constructor>,
) -> [Option<PanelCommand>; 12] {
    // A menu `kind` doesn't have (another group just became active, until the state is validated) shows its grid.
    if interface == InterfaceState::EjectMenu && grid(kind).contains(&Some(PanelCommand::EjectMenu)) {
        return syndicate::eject_menu::grid();
    }
    let expanding = interface
        .placing()
        .is_some_and(|placing| matches!(placing.builder, Builder::Tunnel(_)));
    if (interface == InterfaceState::ExpandMenu || expanding) && grid(kind).contains(&Some(PanelCommand::ExpandMenu)) {
        return syndicate::expand_menu::grid();
    }
    if let InterfaceState::KindMenu(from) = interface
        && grid(kind).contains(&Some(PanelCommand::KindMenu(from)))
    {
        return cults::armory::kind_menu(kind, from);
    }
    let has_tile = |unit| grid(kind).contains(&Some(PanelCommand::AugmentMenu(unit)));
    match interface {
        InterfaceState::AugmentMenu(unit) if has_tile(unit) => return cults::altar::type_menu(unit),
        InterfaceState::AugmentStateMenu(unit, state) if has_tile(unit) => {
            return cults::altar::state_menu(kind, unit, state);
        }
        _ => {}
    }
    let menu = match interface {
        InterfaceState::BuildMenu(menu) => {
            Some(menu).filter(|&menu| grid(kind).contains(&Some(PanelCommand::BuildMenu(menu))))
        }
        InterfaceState::AwaitingPlacement(placing) => menu_of(kind, placing.kind),
        InterfaceState::DefaultState
        | InterfaceState::AwaitingTarget { .. }
        | InterfaceState::EjectMenu
        | InterfaceState::ExpandMenu
        | InterfaceState::KindMenu(_)
        | InterfaceState::AugmentMenu(_)
        | InterfaceState::AugmentStateMenu(..) => None,
    };
    let mut commands = match menu {
        Some(menu) => menu_grid(kind, menu),
        None => grid(kind),
    };
    if menu.is_none() && hides_attack(kind) {
        commands[ATTACK_SLOT] = None;
    }
    let Some(constructor) = constructor.filter(|_| constructs(kind)) else {
        return commands;
    };
    if let Some(ready) = constructor.ready() {
        for command in commands
            .iter_mut()
            .filter(|c| **c == Some(PanelCommand::Construct(ready)))
        {
            *command = Some(PanelCommand::PlaceReady(ready));
        }
    }
    if menu.is_none() && !constructor.is_idle() {
        for command in commands
            .iter_mut()
            .filter(|c| matches!(c, Some(PanelCommand::Construct(_))))
        {
            *command = None;
        }
    }
    commands[slot_index(CommandSlot::CANCEL_PRODUCTION)] = match constructor {
        Constructor::Constructing { .. } => Some(PanelCommand::CancelConstruction),
        Constructor::Ready { .. } => Some(PanelCommand::CancelReady),
        Constructor::Idle => None,
    };
    commands
}
