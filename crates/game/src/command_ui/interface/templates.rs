//! The ObjectInterfaceState templates (`systems/controls/selection_and_hud.md > ObjectInterfaceState[ObjectEnum]`):
//! which commands a type offers, in which slots of the 4 × 3 grid (`> CommandPanel > Grid Layout`), and how its
//! clicks resolve (`super::super::resolution`). A constructor's panel also follows its state and the menu that is
//! open ([`state_grid`]): the Deployment Center's build menus ([`BuildMenu`], design change 0104) and the Extraction
//! Facility's plate (`factions/gdo/economy.md > ExtractionFacility > ObjectInterfaceState[ExtractionFacility]`).

use sc_sim::ObjectKind;
use sc_sim::construction::{self, Constructor};
use sc_sim::defs::AttackType;

use super::{InterfaceState, PanelCommand};
use crate::input::CommandSlot;

/// An ObjectInterfaceState template: which commands a type offers, in which slots, and how its clicks resolve.
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum InterfaceTemplate {
    /// `> BasicCombatUnitInterfaceState`: the Peacekeeper's (`factions/gdo/units.md > Peacekeeper`) and the Guard's
    /// (`factions/syndicate/units.md > Guard`).
    BasicCombatUnit,
    /// `factions/gdo/structures.md > Barracks > ObjectInterfaceState[Barracks]`: a producing structure's panel, and
    /// its right-click sets the rally point (`> CommandPanel > Production Building Default Right-Click`).
    Barracks,
    /// `factions/gdo/structures.md > DeploymentCenter > ObjectInterfaceState[DeploymentCenter]`: its DefaultState
    /// opens the build menus, whose entries construct; once a building is ready, its entry enters AwaitingPlacement.
    /// It produces no units and has no right-click.
    DeploymentCenter,
    /// `factions/gdo/economy.md > ExtractionFacility > ObjectInterfaceState[ExtractionFacility]`: its DefaultState
    /// builds the Extraction Plate, then places it. No right-click.
    ExtractionFacility,
}

/// The Deployment Center's build menus (`> ObjectInterfaceState[DeploymentCenter]`, design change 0104: "the DC's
/// buildings don't fit one 12-slot panel, so they are split"). Their idle, constructing and ready states work the
/// same ([`state_grid`]).
#[derive(Copy, Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum BuildMenu {
    /// BuildMenu\[Basic\], "Q: Basic Construction".
    Basic,
    /// BuildMenu\[Advanced\], "W: Advanced Construction": the Factory and everything after it.
    Advanced,
}

/// `> BuildMenu[Basic]`, slot by slot: Q Generator, W Barracks, E Extraction Facility (design change 0104). Its R
/// Supply Tower, A Radar Station, S Arsenal, D Career Campus and F Defense System aren't in the game yet (their costs
/// and times are TBD: M2 plan 1.2), so their slots stay hidden: "The other slots can show greyed placeholders, or be
/// hidden until those buildings exist". Z is Back and X the constructor's Cancel ([`state_grid`]).
#[rustfmt::skip]
const BASIC_MENU: [Option<ObjectKind>; 12] = {
    use ObjectKind::{Barracks, ExtractionFacility, Generator};
    [
        Some(Generator), Some(Barracks), Some(ExtractionFacility), None,
        None,            None,           None,                     None,
        None,            None,           None,                     None,
    ]
};

/// `> BuildMenu[Advanced]`: Q Factory, W Advanced Mechanics, E Airfield, R Propulsion Labs and A Industrial
/// Powerplant, none of which is in the game yet: every slot hidden, and the menu unavailable ([`offers`]).
const ADVANCED_MENU: [Option<ObjectKind>; 12] = [None; 12];

impl BuildMenu {
    pub const ALL: [BuildMenu; 2] = [BuildMenu::Basic, BuildMenu::Advanced];

    /// Its entries, slot by slot (`CommandSlot::all` order): the buildings it constructs.
    pub fn entries(self) -> &'static [Option<ObjectKind>; 12] {
        match self {
            BuildMenu::Basic => &BASIC_MENU,
            BuildMenu::Advanced => &ADVANCED_MENU,
        }
    }
}

/// `> BasicCombatUnitInterfaceState > DefaultState commands`, row by row from the top left (`CommandSlot::all`).
#[rustfmt::skip]
const BASIC_COMBAT_UNIT_GRID: [Option<PanelCommand>; 12] = {
    use PanelCommand::{Attack, AttackGround, HoldPosition, Move, Patrol, Reverse, Stop};
    [
        Some(Move),   Some(Reverse), Some(HoldPosition), None,
        Some(Attack), Some(Patrol),  Some(AttackGround), None,
        None,         Some(Stop),    None,               None,
    ]
};

/// `factions/gdo/structures.md > Barracks > ObjectInterfaceState[Barracks]` with `> Standard Slot Assignments`:
/// Q Build Peacekeeper, X Cancel Production, C Set Rally Point; Z is Back while a target is awaited. "W: Build
/// Rocketman (hotkey tentative)" stays empty: the Rocketman isn't in the game yet (its cost and time are TBD, and so
/// is its Arsenal; M2 plan § 1.2), so its button is hidden.
#[rustfmt::skip]
const BARRACKS_GRID: [Option<PanelCommand>; 12] = {
    use PanelCommand::{CancelProduction, SetRallyPoint, Train};
    [
        Some(Train(ObjectKind::Peacekeeper)), None,                   None,                None,
        None,                                 None,                   None,                None,
        None,                                 Some(CancelProduction), Some(SetRallyPoint), None,
    ]
};

/// `> ObjectInterfaceState[DeploymentCenter] > DefaultState commands` (design change 0104): "Q: Basic Construction"
/// and "W: Advanced Construction", which open the build menus. Its X, Cancel Construction or Cancel Ready Building,
/// shows only while it constructs or holds a building ready ([`state_grid`]).
#[rustfmt::skip]
const DEPLOYMENT_CENTER_GRID: [Option<PanelCommand>; 12] = {
    use PanelCommand::BuildMenu as Menu;
    [
        Some(Menu(BuildMenu::Basic)), Some(Menu(BuildMenu::Advanced)), None, None,
        None,                         None,                            None, None,
        None,                         None,                            None, None,
    ]
};

/// `> ObjectInterfaceState[ExtractionFacility] > DefaultState`: "Q: Build Extraction Plate", which becomes "Q: Place
/// Plate" once the plate is ready; its X, Cancel Construction or Cancel Ready Plate, shows only while it constructs
/// or holds the plate ready ([`state_grid`]).
#[rustfmt::skip]
const EXTRACTION_FACILITY_GRID: [Option<PanelCommand>; 12] = [
    Some(PanelCommand::Construct(ObjectKind::ExtractionPlate)), None, None, None,
    None,                                                       None, None, None,
    None,                                                       None, None, None,
];

/// `factions/gdo/units.md > Peacekeeper > Riot Shield`: "TBD: hotkey". Placeholder: slot R, the first slot the
/// BasicCombatUnit grid leaves free, where the Colonists' Legionnaires have their own command
/// (`factions/colonists/units.md > Sabre Legionnaire > ObjectInterfaceState`: "BasicCombatUnitInterfaceState, plus:
/// R: Convert").
const RIOT_SHIELD_SLOT: usize = 3;

impl InterfaceTemplate {
    /// The template of `kind`'s ObjectInterfaceState; none for types without commands: the map resources, and the
    /// structures that have none (the Generator, the Extraction Plate). The Legionnaires' is
    /// "BasicCombatUnitInterfaceState, plus" their R (`factions/colonists/units.md > Sabre Legionnaire >
    /// ObjectInterfaceState`; Convert comes with slice M3-B). The other Colonists types have none until slices M3-D,
    /// J1 and J2 build theirs (plan decision T23).
    pub fn of(kind: ObjectKind) -> Option<Self> {
        match kind {
            ObjectKind::Peacekeeper
            | ObjectKind::Guard
            | ObjectKind::SabreLegionnaire
            | ObjectKind::RifleLegionnaire => Some(InterfaceTemplate::BasicCombatUnit),
            ObjectKind::Barracks => Some(InterfaceTemplate::Barracks),
            ObjectKind::DeploymentCenter => Some(InterfaceTemplate::DeploymentCenter),
            ObjectKind::ExtractionFacility => Some(InterfaceTemplate::ExtractionFacility),
            ObjectKind::CrystalPatch
            | ObjectKind::SupplyDeliveryStation
            | ObjectKind::Generator
            | ObjectKind::ExtractionPlate
            | ObjectKind::Drone
            | ObjectKind::Beacon
            | ObjectKind::EgressNode
            | ObjectKind::RoboPortal
            | ObjectKind::BioPortal => None,
        }
    }

    /// Its DefaultState's commands, slot by slot (`CommandSlot::all` order).
    pub fn grid(self) -> &'static [Option<PanelCommand>; 12] {
        match self {
            InterfaceTemplate::BasicCombatUnit => &BASIC_COMBAT_UNIT_GRID,
            InterfaceTemplate::Barracks => &BARRACKS_GRID,
            InterfaceTemplate::DeploymentCenter => &DEPLOYMENT_CENTER_GRID,
            InterfaceTemplate::ExtractionFacility => &EXTRACTION_FACILITY_GRID,
        }
    }
}

/// `systems/combat.md > AttackType > CanTargetGround`, derived from the AttackType.
pub fn can_target_ground(attack_type: AttackType) -> bool {
    matches!(attack_type, AttackType::TailDisjointed | AttackType::DoublyDisjointed)
}

/// `kind`'s DefaultState commands, slot by slot (`CommandSlot::all` order): its template's grid, plus its own
/// commands in slots the template leaves free (a unit that carries a Riot Shield: [`PanelCommand::RiotShield`] in
/// R). What a constructor shows also depends on its state ([`state_grid`]).
pub fn grid(kind: ObjectKind) -> [Option<PanelCommand>; 12] {
    let Some(template) = InterfaceTemplate::of(kind) else {
        return [None; 12];
    };
    let mut grid = *template.grid();
    if kind.def().unit.as_ref().is_some_and(|unit| unit.riot_shield.is_some()) {
        grid[RIOT_SHIELD_SLOT] = Some(PanelCommand::RiotShield);
    }
    grid
}

/// Whether objects of `kind` construct buildings (`StructureDef::constructs`): the Deployment Center and the
/// Extraction Facility.
fn constructs(kind: ObjectKind) -> bool {
    !construction::recipes(kind).is_empty()
}

/// The build menu of a `kind` constructor that holds `building` (the Deployment Center's), if it has one.
pub fn menu_of(kind: ObjectKind, building: ObjectKind) -> Option<BuildMenu> {
    BuildMenu::ALL.into_iter().find(|&menu| {
        grid(kind).contains(&Some(PanelCommand::BuildMenu(menu))) && menu.entries().contains(&Some(building))
    })
}

/// The commands a `kind` ActiveGroup shows in `interface`, slot by slot (`CommandSlot::all` order), while its
/// constructor, if it is one, is `constructor` (the ActiveGroup is then a single instance: constructors are
/// Ungroupable, `> SelectionGroup`, so the panel "can reliably read that instance's specific game state"):
/// - DefaultState and AwaitingTarget: its DefaultState grid ([`grid`]);
/// - a build menu: the menu's entries; AwaitingPlacement: those of the state it goes back to
///   (`InterfaceState::back`): the Deployment Center's menu of the building, the Extraction Facility's DefaultState;
/// - a constructor's building ready to place takes its entry's slot, as "Select completed building" (design change
///   0104; the Extraction Facility's "Q: Place Plate"), and its X is Cancel Construction while it constructs and
///   Cancel Ready Building while it holds a building ready, and empty while it is idle (`> DefaultState commands`:
///   "Only visible when CurrentConstruction is set", "Only visible when ReadyToPlace is set");
/// - a build menu shows its other entries while the constructor is busy, unavailable (`> BuildMenu (instance
///   constructing)`: "Other build options unavailable": [`super::Conditions`] disables them), but the Extraction
///   Facility's DefaultState lists only X while it constructs and only Q Place Plate and X while its plate is ready
///   (`factions/gdo/economy.md > ExtractionFacility > ObjectInterfaceState[ExtractionFacility]`), so a building to
///   construct in a DefaultState grid shows only while the constructor is idle.
pub fn state_grid(
    kind: ObjectKind,
    interface: InterfaceState,
    constructor: Option<Constructor>,
) -> [Option<PanelCommand>; 12] {
    // A menu `kind` doesn't have (another group just became active, until the state is validated) shows its grid.
    let menu = match interface {
        InterfaceState::BuildMenu(menu) => {
            Some(menu).filter(|&menu| grid(kind).contains(&Some(PanelCommand::BuildMenu(menu))))
        }
        InterfaceState::AwaitingPlacement(placing) => menu_of(kind, placing.kind),
        InterfaceState::DefaultState | InterfaceState::AwaitingTarget { .. } => None,
    };
    let mut commands = match menu {
        Some(menu) => menu.entries().map(|entry| entry.map(PanelCommand::Construct)),
        None => grid(kind),
    };
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

/// Every command `kind` may show, in any state ([`state_grid`]): its DefaultState grid, and a constructor's
/// buildings to construct and to place, and its two Cancels. In order, each once.
pub fn commands(kind: ObjectKind) -> Vec<PanelCommand> {
    let mut commands: Vec<PanelCommand> = grid(kind).into_iter().flatten().collect();
    if constructs(kind) {
        for recipe in construction::recipes(kind) {
            commands.extend([
                PanelCommand::Construct(recipe.kind),
                PanelCommand::PlaceReady(recipe.kind),
            ]);
        }
        commands.extend([PanelCommand::CancelConstruction, PanelCommand::CancelReady]);
    }
    commands.sort_unstable();
    commands.dedup();
    commands
}

/// Whether objects of `kind` offer `command`: their grid has it ([`grid`]), and a conditional one only when the
/// type can (`> BasicCombatUnitInterfaceState`: Reverse with the UnitBase's CanReverse, AttackGround with the
/// AttackType's CanTargetGround). A constructor offers its own buildings (its `Constructs` list) and its Cancels; a
/// build menu is offered while something in it is in the game, so the Deployment Center's W, Advanced Construction,
/// shows unavailable: none of its buildings exists yet (design change 0104: "greyed placeholders").
pub fn offers(kind: ObjectKind, command: PanelCommand) -> bool {
    match command {
        PanelCommand::Construct(building) | PanelCommand::PlaceReady(building) => {
            return construction::recipe(kind, building).is_some();
        }
        PanelCommand::CancelConstruction | PanelCommand::CancelReady => return constructs(kind),
        PanelCommand::BuildMenu(menu) => {
            return grid(kind).contains(&Some(command))
                && menu
                    .entries()
                    .iter()
                    .flatten()
                    .any(|&building| construction::recipe(kind, building).is_some());
        }
        _ => {}
    }
    if !grid(kind).contains(&Some(command)) {
        return false;
    }
    let unit = kind.def().unit.as_ref();
    match command {
        PanelCommand::Reverse => unit.is_some_and(|unit| unit.base_def().can_reverse),
        PanelCommand::AttackGround => unit
            .and_then(|unit| unit.attack)
            .is_some_and(|attack| can_target_ground(attack.attack_type)),
        _ => true,
    }
}

/// A slot's position in `CommandSlot::all` order.
pub fn slot_index(slot: CommandSlot) -> usize {
    usize::from(slot.row()) * usize::from(CommandSlot::COLUMNS) + usize::from(slot.column())
}
