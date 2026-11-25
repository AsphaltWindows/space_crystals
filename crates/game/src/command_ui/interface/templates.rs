//! The ObjectInterfaceState templates (`systems/controls/selection_and_hud.md > ObjectInterfaceState[ObjectEnum]`):
//! which commands a type offers, in which slots of the 4 × 3 grid (`systems/controls/selection_and_hud.md >
//! CommandPanel > Grid Layout`), and how its clicks resolve (`super::super::resolution`). Each type's template is in
//! its module (`crate::kinds`, [`PanelTemplate`]); a panel of its own, by faction and object: [`gdo`], [`colonists`],
//! [`syndicate`]. A constructor's panel also follows its state and the menu that is open ([`state_grid`]): the
//! Deployment Center's build menus ([`BuildMenu`], design change 0104) and the Extraction Facility's plate
//! (`factions/gdo/economy.md > ExtractionFacility > ObjectInterfaceState[ExtractionFacility]`). The Colonists' Beacon
//! has two Warp In menus built the same way, the Drone its Deploy and its Set Drop-off, the Egress Node its F, and an
//! object that transforms shows only its Cancel ([`colonists`], slices M3-J1 and M3-J2). The Syndicate's Tunnel opens
//! its EjectMenu ([`syndicate`], slice M4-J2). An entry not in the game yet holds its slot, unavailable ([`Planned`]).

pub mod colonists;
pub mod gdo;
mod offers;
mod planned;
mod production;
mod state;
pub mod syndicate;

use sc_sim::ObjectKind;
use sc_sim::conversion;

pub use colonists::TRANSFORMING_GRID;
pub use offers::{commands, offers};
pub use planned::Planned;
pub use state::{menu_grid, menu_of, state_grid};

use production::production_grid;
use state::constructs;

use super::{InterfaceState, PanelCommand};
use crate::input::CommandSlot;
use crate::kinds;

/// An ObjectInterfaceState template: which commands a type offers, in which slots, and how its clicks resolve.
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum InterfaceTemplate {
    /// `systems/controls/selection_and_hud.md > BasicCombatUnitInterfaceState`: the Peacekeeper's
    /// (`factions/gdo/units/peacekeeper.md`), and its shielded type's less Attack (design change 0173), and the Guard's
    /// (`factions/syndicate/units/guard.md`).
    BasicCombatUnit,
    /// A producing structure's panel (plan decision T23), built from its `ProductionDef` ([`production_grid`]): the
    /// Barracks' (`factions/gdo/structures/barracks.md > ObjectInterfaceState[Barracks]`) and the Colonists' portals'
    /// (`factions/colonists/structures/robo_portal.md > Robo Portal ObjectInterfaceState`,
    /// `factions/colonists/structures/bio_portal.md > Bio Portal ObjectInterfaceState`, design change 0124: "Standard
    /// production panel, same conventions as GDO's Barracks"). Its right-click sets the rally point
    /// (`systems/controls/selection_and_hud.md > CommandPanel > Production Building Default Right-Click`).
    Production,
    /// `factions/gdo/structures/deployment_center.md > ObjectInterfaceState[DeploymentCenter]`: its DefaultState
    /// opens the build menus, whose entries construct; once a building is ready, its entry enters AwaitingPlacement.
    /// It produces no units and has no right-click.
    DeploymentCenter,
    /// `factions/gdo/economy.md > ExtractionFacility > ObjectInterfaceState[ExtractionFacility]`: its DefaultState
    /// builds the Extraction Plate, then places it. No right-click.
    ExtractionFacility,
    /// `factions/colonists/economy.md > Beacon > Beacon ObjectInterfaceState` (design change 0124): its DefaultState
    /// opens the two Warp In menus, whose entries enter the Warp In's placement on the Beacon. No right-click.
    Beacon,
    /// `factions/colonists/units/drone.md > Drone ObjectInterfaceState`: Q Move, A Deploy, S Set Drop-off, X Stop. Its
    /// right-click (`factions/colonists/units/drone.md > Right-Click Resolution`: Gather, Drop Off, Move, Follow) is
    /// the sim's one rule, as every unit's (`crate::command_ui::resolution::unit_right_click`, slice M3-R).
    Drone,
    /// `factions/colonists/economy.md > Egress Node > Egress Node ObjectInterfaceState`: "F: Select Returning Drones"
    /// (slice M3-J2). No right-click.
    EgressNode,
    /// `factions/syndicate/tunnels.md > Tunnel ObjectInterfaceState` (design change 0257, slice M4-J2): Q Upgrade, W
    /// Expand, E Eject, which opens the EjectMenu, and X Cancel ([`syndicate::tunnel`]). No right-click: the Tunnel
    /// produces no units.
    Tunnel,
}

/// The Deployment Center's build menus (`factions/gdo/structures/deployment_center.md >
/// ObjectInterfaceState[DeploymentCenter]`, design change 0104: "the DC's buildings don't fit one 12-slot panel, so
/// they are split"). Their idle, constructing and ready states work the same ([`state_grid`]). The Colonists' Beacon
/// has two Warp In menus "like the Deployment Center's two build menus" (`factions/colonists/economy.md > Beacon >
/// Beacon ObjectInterfaceState`): Warp In (Basic) and Warp In (Advanced), whose entries are its own ([`menu_grid`]).
#[derive(Copy, Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum BuildMenu {
    /// BuildMenu\[Basic\], "Q: Basic Construction".
    Basic,
    /// BuildMenu\[Advanced\], "W: Advanced Construction": the Factory and everything after it.
    Advanced,
}

impl BuildMenu {
    pub const ALL: [BuildMenu; 2] = [BuildMenu::Basic, BuildMenu::Advanced];

    /// The Deployment Center's entries, slot by slot (`CommandSlot::all` order): the buildings it constructs
    /// ([`gdo::deployment_center::menu`]).
    pub fn entries(self) -> &'static [Option<ObjectKind>; 12] {
        gdo::deployment_center::menu(self)
    }
}

/// A type's panel template, from its module (`crate::kinds::ClientKind::template`): its ObjectInterfaceState
/// template, and what its own panel adds to it. A type with a panel of its own has its template in [`gdo`] or
/// [`colonists`], by object.
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub struct PanelTemplate {
    /// Its ObjectInterfaceState template ([`InterfaceTemplate::of`]); none for a type without commands.
    pub interface: Option<InterfaceTemplate>,
    /// The units of a producing structure's panel that aren't in the game yet ([`Planned`]), each with its slot
    /// (`CommandSlot::all` order), which [`production_grid`] adds to its recipes: a portal's (design change 0124).
    pub planned: &'static [(usize, Planned)],
    /// The commands the design names after the object whose panel shows them, with their names
    /// (`PanelCommand::name_on`).
    pub names: &'static [(PanelCommand, &'static str)],
}

impl PanelTemplate {
    /// No panel: a type without commands.
    pub const NONE: PanelTemplate = PanelTemplate {
        interface: None,
        planned: &[],
        names: &[],
    };

    /// The panel of `interface`, with nothing of its own.
    pub const fn of(interface: InterfaceTemplate) -> PanelTemplate {
        PanelTemplate {
            interface: Some(interface),
            ..PanelTemplate::NONE
        }
    }
}

/// `systems/controls/selection_and_hud.md > BasicCombatUnitInterfaceState > DefaultState commands`, row by row from the
/// top left (`CommandSlot::all`).
#[rustfmt::skip]
const BASIC_COMBAT_UNIT_GRID: [Option<PanelCommand>; 12] = {
    use PanelCommand::{Attack, AttackGround, HoldPosition, Move, Patrol, Reverse, Stop};
    [
        Some(Move),   Some(Reverse), Some(HoldPosition), None,
        Some(Attack), Some(Patrol),  Some(AttackGround), None,
        None,         Some(Stop),    None,               None,
    ]
};

/// `factions/gdo/units/peacekeeper.md > Riot Shield > Hotkey`: "**Hotkey: R** (Raise Shield on Peacekeeper, Lower
/// Shield on Peacekeeper (Shielded)), placeholder" (design change 0174): slot R, the first slot the BasicCombatUnit
/// grid leaves free, where the Colonists' Legionnaires have their own command ([`CONVERT_SLOT`]).
const RIOT_SHIELD_SLOT: usize = 3;

/// `systems/controls/selection_and_hud.md > BasicCombatUnitInterfaceState > DefaultState commands`: Attack's slot, A,
/// which the Peacekeeper (Shielded)'s panel leaves empty ([`hides_attack`]).
const ATTACK_SLOT: usize = 4;

/// `factions/colonists/units/sabre_legionnaire.md > ObjectInterfaceState`: "BasicCombatUnitInterfaceState, plus:
/// R: Convert to Rifle Legionnaire", and the Rifle Legionnaire's "R: Convert to Sabre Legionnaire".
const CONVERT_SLOT: usize = 3;

impl InterfaceTemplate {
    /// The template of `kind`'s ObjectInterfaceState, its module's (`crate::kinds`); none for types without commands:
    /// the map resources, and the structures that have none (the Generator, the Extraction Plate). The Legionnaires'
    /// is "BasicCombatUnitInterfaceState, plus" their R (`factions/colonists/units/sabre_legionnaire.md >
    /// ObjectInterfaceState`: Convert, [`grid`]). Every type that produces units has the Production template; the
    /// Egress Node has its own, its F (slice M3-J2). Whatever its type, an object that transforms shows only its
    /// Cancel ([`TRANSFORMING_GRID`], `super::panel`) and resolves no right-click
    /// (`super::super::resolution::UnitCapabilities::of`).
    pub fn of(kind: ObjectKind) -> Option<Self> {
        kinds::of(kind).template.interface
    }

    /// Its DefaultState's commands for a `kind` object, slot by slot (`CommandSlot::all` order): the Production
    /// template's come from the type's `ProductionDef` ([`production_grid`]), an object's own from its module.
    pub fn grid(self, kind: ObjectKind) -> [Option<PanelCommand>; 12] {
        match self {
            InterfaceTemplate::BasicCombatUnit => BASIC_COMBAT_UNIT_GRID,
            InterfaceTemplate::Production => production_grid(kind),
            InterfaceTemplate::DeploymentCenter => gdo::deployment_center::GRID,
            InterfaceTemplate::ExtractionFacility => gdo::extraction_facility::GRID,
            InterfaceTemplate::Beacon => colonists::beacon::GRID,
            InterfaceTemplate::Drone => colonists::drone::GRID,
            InterfaceTemplate::EgressNode => colonists::egress_node::GRID,
            InterfaceTemplate::Tunnel => syndicate::tunnel::GRID,
        }
    }
}

/// `systems/combat.md > AttackType > CanTargetGround`, derived from the AttackType: the sim's rule, which its gate
/// takes an Attack Ground by (slice M4-O).
pub use sc_sim::combat::can_target_ground;

/// `kind`'s DefaultState commands, slot by slot (`CommandSlot::all` order): its template's grid, plus its own
/// commands in slots the template leaves free: a unit that carries a Riot Shield the command that turns it, in R
/// (`factions/gdo/units/peacekeeper.md > Riot Shield > Two unit types`, design change 0173: "The Peacekeeper's
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
/// (`factions/gdo/units/peacekeeper.md > Riot Shield > Two unit types`, design change 0173: its panel "shows Lower
/// Shield and no Attack"). The type still offers Attack ([`offers`]), so that in a selection with other units,
/// Attack, a common command, goes to them too: an Attack on an object leaves them out (design change 0075,
/// `super::super::resolution::unit_target_order`), and an AttackMove takes them along, walking (0076: "shielded
/// Peacekeepers take part and simply move"), as with their shields up before the two types. A selection of them
/// alone has no Attack, as it had none to use before (its button was disabled).
pub fn hides_attack(kind: ObjectKind) -> bool {
    let shield = kind.def().unit.as_ref().and_then(|unit| unit.riot_shield);
    shield.is_some_and(|shield| kind == shield.raised)
}

/// A slot's position in `CommandSlot::all` order.
pub fn slot_index(slot: CommandSlot) -> usize {
    usize::from(slot.row()) * usize::from(CommandSlot::COLUMNS) + usize::from(slot.column())
}
