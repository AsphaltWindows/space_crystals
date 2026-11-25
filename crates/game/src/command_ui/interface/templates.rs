//! The ObjectInterfaceState templates (`systems/controls/selection_and_hud.md > ObjectInterfaceState[ObjectEnum]`):
//! which commands a type offers, in which slots of the 4 × 3 grid (`> CommandPanel > Grid Layout`), and how its
//! clicks resolve (`super::super::resolution`). A constructor's panel also follows its state and the menu that is
//! open ([`state_grid`]): the Deployment Center's build menus ([`BuildMenu`], design change 0104) and the Extraction
//! Facility's plate (`factions/gdo/economy.md > ExtractionFacility > ObjectInterfaceState[ExtractionFacility]`). The
//! Colonists' Beacon has two Warp In menus built the same way, the Drone its Deploy and its Set Drop-off, the Egress
//! Node its F, and an object that transforms shows only its Cancel ([`colonists`], slices M3-J1 and M3-J2).

mod colonists;

use sc_sim::ObjectKind;
use sc_sim::construction::{self, Constructor};
use sc_sim::conversion;
use sc_sim::defs::AttackType;
use sc_sim::transform;

pub use colonists::{Planned, TRANSFORMING_GRID};

use super::{InterfaceState, PanelCommand};
use crate::input::CommandSlot;
use crate::selection::GroupType;

/// An ObjectInterfaceState template: which commands a type offers, in which slots, and how its clicks resolve.
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum InterfaceTemplate {
    /// `> BasicCombatUnitInterfaceState`: the Peacekeeper's (`factions/gdo/units.md > Peacekeeper`), and its
    /// shielded type's less Attack (design change 0173), and the Guard's (`factions/syndicate/units.md > Guard`).
    BasicCombatUnit,
    /// A producing structure's panel (plan decision T23), built from its `ProductionDef` ([`production_grid`]): the
    /// Barracks' (`factions/gdo/structures.md > Barracks > ObjectInterfaceState[Barracks]`) and the Colonists'
    /// portals' (`factions/colonists/structures.md > Robo Portal ObjectInterfaceState`, `> Bio Portal
    /// ObjectInterfaceState`, design change 0124: "Standard production panel, same conventions as GDO's Barracks").
    /// Its right-click sets the rally point (`> CommandPanel > Production Building Default Right-Click`).
    Production,
    /// `factions/gdo/structures.md > DeploymentCenter > ObjectInterfaceState[DeploymentCenter]`: its DefaultState
    /// opens the build menus, whose entries construct; once a building is ready, its entry enters AwaitingPlacement.
    /// It produces no units and has no right-click.
    DeploymentCenter,
    /// `factions/gdo/economy.md > ExtractionFacility > ObjectInterfaceState[ExtractionFacility]`: its DefaultState
    /// builds the Extraction Plate, then places it. No right-click.
    ExtractionFacility,
    /// `factions/colonists/economy.md > Beacon > Beacon ObjectInterfaceState` (design change 0124): its DefaultState
    /// opens the two Warp In menus, whose entries enter the Warp In's placement on the Beacon. No right-click.
    Beacon,
    /// `factions/colonists/units.md > Drone ObjectInterfaceState`: Q Move, A Deploy, S Set Drop-off, X Stop. Its
    /// right-click (`> Right-Click Resolution`: Gather, Drop Off, Move, Follow) is the sim's one rule, as every
    /// unit's (`crate::command_ui::resolution::unit_right_click`, slice M3-R).
    Drone,
    /// `factions/colonists/economy.md > Egress Node > Egress Node ObjectInterfaceState`: "F: Select Returning Drones"
    /// (slice M3-J2). No right-click.
    EgressNode,
}

/// The Deployment Center's build menus (`> ObjectInterfaceState[DeploymentCenter]`, design change 0104: "the DC's
/// buildings don't fit one 12-slot panel, so they are split"). Their idle, constructing and ready states work the
/// same ([`state_grid`]). The Colonists' Beacon has two Warp In menus "like the Deployment Center's two build menus"
/// (`factions/colonists/economy.md > Beacon > Beacon ObjectInterfaceState`): Warp In (Basic) and Warp In (Advanced),
/// whose entries are its own ([`menu_grid`]).
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

    /// The Deployment Center's entries, slot by slot (`CommandSlot::all` order): the buildings it constructs.
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

/// How many units a producing structure's panel trains: one in each of Q, W, E, R, A, S, D and F; the bottom row is
/// `> Standard Slot Assignments`' (Z Back, X Cancel Production, C Set Rally Point).
const TRAIN_SLOTS: usize = 8;

/// A producing structure's DefaultState commands (plan decision T23): Train each of the units of its `ProductionDef`,
/// in the design's order, from Q on (Q, W, E, R, then A, S, D, F); X Cancel Production and C Set Rally Point
/// (`> Standard Slot Assignments`); Z is Back while a target is awaited.
///
/// - The Barracks: Q Build Peacekeeper, X, C (`factions/gdo/structures.md > Barracks >
///   ObjectInterfaceState[Barracks]`). "W: Build Rocketman (hotkey tentative)" stays empty: the Rocketman isn't in
///   the game yet (its cost and time are TBD, and so is its Arsenal; M2 plan § 1.2), so it isn't among the recipes
///   and its button is hidden.
/// - The Robo Portal: Q Drone, W Scout, X, C; the Bio Portal: Q Sabre Legionnaire, W Lancer, E psionic unit, X, C
///   (`factions/colonists/structures.md > Robo Portal ObjectInterfaceState`, `> Bio Portal ObjectInterfaceState`,
///   design change 0124). The Scout, the Lancer and the psionic unit aren't in the game (their stats, cost and time
///   are TBD), so they aren't among the recipes: 0124 has them show in their slot, unavailable
///   ([`PanelCommand::Planned`], `colonists::planned_units`; slice M3-J2).
fn production_grid(kind: ObjectKind) -> [Option<PanelCommand>; 12] {
    let mut grid = [None; 12];
    let production = kind.def().structure.as_ref().and_then(|s| s.production.as_ref());
    let recipes = production.map_or(&[][..], |production| production.recipes);
    debug_assert!(
        recipes.len() <= TRAIN_SLOTS,
        "{kind:?} trains more units than its panel has slots"
    );
    for (slot, recipe) in grid.iter_mut().zip(recipes.iter().take(TRAIN_SLOTS)) {
        *slot = Some(PanelCommand::Train(recipe.kind));
    }
    for &(slot, planned) in colonists::planned_units(kind) {
        grid[slot] = grid[slot].or(Some(PanelCommand::Planned(planned)));
    }
    grid[slot_index(CommandSlot::CANCEL_PRODUCTION)] = Some(PanelCommand::CancelProduction);
    grid[slot_index(CommandSlot::RALLY_POINT)] = Some(PanelCommand::SetRallyPoint);
    grid
}

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

/// `factions/gdo/units.md > Peacekeeper > Riot Shield > Hotkey`: "**Hotkey: R** (Raise Shield on Peacekeeper, Lower
/// Shield on Peacekeeper (Shielded)), placeholder" (design change 0174): slot R, the first slot the BasicCombatUnit
/// grid leaves free, where the Colonists' Legionnaires have their own command ([`CONVERT_SLOT`]).
const RIOT_SHIELD_SLOT: usize = 3;

/// `> BasicCombatUnitInterfaceState > DefaultState commands`: Attack's slot, A, which the Peacekeeper (Shielded)'s
/// panel leaves empty ([`hides_attack`]).
const ATTACK_SLOT: usize = 4;

/// `factions/colonists/units.md > Sabre Legionnaire > ObjectInterfaceState`: "BasicCombatUnitInterfaceState, plus:
/// R: Convert to Rifle Legionnaire", and the Rifle Legionnaire's "R: Convert to Sabre Legionnaire".
const CONVERT_SLOT: usize = 3;

impl InterfaceTemplate {
    /// The template of `kind`'s ObjectInterfaceState; none for types without commands: the map resources, and the
    /// structures that have none (the Generator, the Extraction Plate). The Legionnaires' is
    /// "BasicCombatUnitInterfaceState, plus" their R (`factions/colonists/units.md > Sabre Legionnaire >
    /// ObjectInterfaceState`: Convert, [`grid`]). Every type that produces units has the Production
    /// template; the Egress Node has its own, its F (slice M3-J2). Whatever its type, an object that transforms shows
    /// only its Cancel ([`TRANSFORMING_GRID`], `super::panel`) and resolves no right-click
    /// (`super::super::resolution::UnitCapabilities::of`).
    pub fn of(kind: ObjectKind) -> Option<Self> {
        match kind {
            ObjectKind::Peacekeeper
            | ObjectKind::PeacekeeperShielded
            | ObjectKind::Guard
            | ObjectKind::SabreLegionnaire
            | ObjectKind::RifleLegionnaire => Some(InterfaceTemplate::BasicCombatUnit),
            ObjectKind::Barracks | ObjectKind::RoboPortal | ObjectKind::BioPortal => {
                Some(InterfaceTemplate::Production)
            }
            ObjectKind::DeploymentCenter => Some(InterfaceTemplate::DeploymentCenter),
            ObjectKind::ExtractionFacility => Some(InterfaceTemplate::ExtractionFacility),
            ObjectKind::Beacon => Some(InterfaceTemplate::Beacon),
            ObjectKind::Drone => Some(InterfaceTemplate::Drone),
            ObjectKind::EgressNode => Some(InterfaceTemplate::EgressNode),
            ObjectKind::CrystalPatch
            | ObjectKind::SupplyDeliveryStation
            | ObjectKind::Generator
            | ObjectKind::ExtractionPlate => None,
        }
    }

    /// Its DefaultState's commands for a `kind` object, slot by slot (`CommandSlot::all` order): the Production
    /// template's come from the type's `ProductionDef` ([`production_grid`]).
    pub fn grid(self, kind: ObjectKind) -> [Option<PanelCommand>; 12] {
        match self {
            InterfaceTemplate::BasicCombatUnit => BASIC_COMBAT_UNIT_GRID,
            InterfaceTemplate::Production => production_grid(kind),
            InterfaceTemplate::DeploymentCenter => DEPLOYMENT_CENTER_GRID,
            InterfaceTemplate::ExtractionFacility => EXTRACTION_FACILITY_GRID,
            InterfaceTemplate::Beacon => colonists::BEACON_GRID,
            InterfaceTemplate::Drone => colonists::DRONE_GRID,
            InterfaceTemplate::EgressNode => colonists::EGRESS_NODE_GRID,
        }
    }
}

/// `systems/combat.md > AttackType > CanTargetGround`, derived from the AttackType.
pub fn can_target_ground(attack_type: AttackType) -> bool {
    matches!(attack_type, AttackType::TailDisjointed | AttackType::DoublyDisjointed)
}

/// `kind`'s DefaultState commands, slot by slot (`CommandSlot::all` order): its template's grid, plus its own
/// commands in slots the template leaves free: a unit that carries a Riot Shield the command that turns it, in R
/// (`factions/gdo/units.md > Peacekeeper > Riot Shield > Two unit types`, design change 0173: "The Peacekeeper's
/// panel shows **Raise Shield**; Peacekeeper (Shielded)'s shows **Lower Shield**": [`PanelCommand::RaiseShield`] or
/// [`PanelCommand::LowerShield`]); a unit with another form, a Legionnaire, the Convert to that form in R
/// ([`PanelCommand::Convert`]: "R: Convert to Rifle Legionnaire" on the Sabre's panel). These
/// are the commands the type offers ([`offers`]); its panel may show fewer (the Peacekeeper (Shielded)'s no Attack:
/// [`hides_attack`]), and a constructor's follows its state ([`state_grid`]).
pub fn grid(kind: ObjectKind) -> [Option<PanelCommand>; 12] {
    let Some(template) = InterfaceTemplate::of(kind) else {
        return [None; 12];
    };
    let mut grid = template.grid(kind);
    if let Some(shield) = kind.def().unit.as_ref().and_then(|unit| unit.riot_shield) {
        grid[RIOT_SHIELD_SLOT] = Some(match kind == shield.raised {
            true => PanelCommand::LowerShield,
            false => PanelCommand::RaiseShield,
        });
    }
    if let Some(other) = conversion::converts_to(kind) {
        grid[CONVERT_SLOT] = Some(PanelCommand::Convert(other));
    }
    grid
}

/// Whether the panel of a `kind` ActiveGroup leaves out its Attack: the Peacekeeper (Shielded)'s
/// (`factions/gdo/units.md > Peacekeeper > Riot Shield > Two unit types`, design change 0173: its panel "shows Lower
/// Shield and no Attack"). The type still offers Attack ([`offers`]), so that in a selection with other units,
/// Attack, a common command, goes to them too: an Attack on an object leaves them out (design change 0075,
/// `super::super::resolution::unit_target_order`), and an AttackMove takes them along, walking (0076: "shielded
/// Peacekeepers take part and simply move"), as with their shields up before the two types. A selection of them
/// alone has no Attack, as it had none to use before (its button was disabled).
pub fn hides_attack(kind: ObjectKind) -> bool {
    let shield = kind.def().unit.as_ref().and_then(|unit| unit.riot_shield);
    shield.is_some_and(|shield| kind == shield.raised)
}

/// Whether objects of `kind` construct buildings (`StructureDef::constructs`): the Deployment Center and the
/// Extraction Facility.
fn constructs(kind: ObjectKind) -> bool {
    !construction::recipes(kind).is_empty()
}

/// The entries of a `kind` object's build menu `menu`, slot by slot (`CommandSlot::all` order): a Beacon's Warp In
/// menu (`factions/colonists/economy.md > Beacon > Beacon ObjectInterfaceState`: every building of the design's
/// menu, those not in the game unavailable), else the Deployment Center's menu, its buildings to construct.
pub fn menu_grid(kind: ObjectKind, menu: BuildMenu) -> [Option<PanelCommand>; 12] {
    if colonists::warps_in(kind) {
        colonists::warp_in_menu(menu)
    } else {
        menu.entries().map(|entry| entry.map(PanelCommand::Construct))
    }
}

/// The build menu of a `kind` object whose `building` is placed (the Deployment Center's, holding it ready; a
/// Beacon's, warping it in), if it has one.
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
/// Ungroupable, `> SelectionGroup`, so the panel "can reliably read that instance's specific game state"):
/// - DefaultState and AwaitingTarget: its DefaultState grid ([`grid`]), but the Peacekeeper (Shielded)'s Attack
///   ([`hides_attack`]);
/// - a build menu: the menu's entries ([`menu_grid`]); AwaitingPlacement: those of the state it goes back to
///   (`InterfaceState::back`): the Deployment Center's menu of the building, a Beacon's Warp In menu of it, the
///   Extraction Facility's and the Drone's DefaultState;
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

/// Every command objects of type `ty` (a kind: a finished one) may show, in any state ([`state_grid`]): an
/// unfinished building's Cancel alone ([`TRANSFORMING_GRID`], design change 0185); a finished type's DefaultState
/// grid, and a constructor's buildings to construct and to place, and its two Cancels; a Beacon's buildings to warp
/// in. In order, each once.
pub fn commands(ty: impl Into<GroupType>) -> Vec<PanelCommand> {
    let GroupType { kind, unfinished } = ty.into();
    if unfinished {
        return vec![PanelCommand::CancelTransform];
    }
    let mut commands: Vec<PanelCommand> = grid(kind).into_iter().flatten().collect();
    commands.extend(
        transform::warp_in_recipes(kind)
            .iter()
            .map(|r| PanelCommand::WarpIn(r.kind)),
    );
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

/// Whether objects of type `ty` (a kind: a finished one) offer `command`. An unfinished building, in a group of its
/// own "as if it were another type" (design change 0185), offers its Cancel and nothing else (`factions/colonists/
/// economy.md > Beacon Deployment > While transforming: only Cancel`, [`TRANSFORMING_GRID`]). A finished one offers
/// what its grid has ([`grid`]), and a conditional command only when the type can (`> BasicCombatUnitInterfaceState`:
/// Reverse with the UnitBase's CanReverse, AttackGround with the AttackType's CanTargetGround). A constructor offers
/// its own buildings (its `Constructs` list) and its Cancels, a Beacon the buildings of its Constructs List; a
/// building or a unit that isn't in the game is offered by no one (design change 0124: "shown but unavailable"). A
/// build menu is offered while it shows something, so the Deployment Center's W, Advanced Construction, shows
/// unavailable: none of its buildings exists yet, and they are hidden (design change 0104: "greyed placeholders"); the
/// Beacon's Warp In (Advanced) opens, on its unavailable buildings.
pub fn offers(ty: impl Into<GroupType>, command: PanelCommand) -> bool {
    let GroupType { kind, unfinished } = ty.into();
    if unfinished {
        return command == PanelCommand::CancelTransform;
    }
    match command {
        PanelCommand::Construct(building) | PanelCommand::PlaceReady(building) => {
            return construction::recipe(kind, building).is_some();
        }
        PanelCommand::CancelConstruction | PanelCommand::CancelReady => return constructs(kind),
        PanelCommand::BuildMenu(menu) => {
            return grid(kind).contains(&Some(command)) && menu_grid(kind, menu).iter().any(Option::is_some);
        }
        PanelCommand::WarpIn(building) => return transform::warp_in_recipe(kind, building).is_some(),
        PanelCommand::Planned(_) | PanelCommand::CancelTransform => return false,
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
