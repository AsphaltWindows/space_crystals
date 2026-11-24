//! The ObjectInterfaceState templates (`systems/controls/selection_and_hud.md > ObjectInterfaceState[ObjectEnum]`):
//! which commands a type offers, in which slots of the 4 × 3 grid (`> CommandPanel > Grid Layout`), and how its
//! clicks resolve (`super::super::resolution`).

use sc_sim::ObjectKind;
use sc_sim::defs::AttackType;

use super::PanelCommand;
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

/// `factions/gdo/units.md > Peacekeeper > Riot Shield`: "TBD: hotkey". Placeholder: slot R, the first slot the
/// BasicCombatUnit grid leaves free, where the Colonists' Legionnaires have their own command
/// (`factions/colonists/units.md > Sabre Legionnaire > ObjectInterfaceState`: "BasicCombatUnitInterfaceState, plus:
/// R: Convert").
const RIOT_SHIELD_SLOT: usize = 3;

impl InterfaceTemplate {
    /// The template of `kind`'s ObjectInterfaceState; none for types without commands: the map resources, and the
    /// structures that have none (the Generator, the Extraction Plate) or whose panels come with the construction
    /// UI (the Deployment Center and the Extraction Facility, slice M2-H2).
    pub fn of(kind: ObjectKind) -> Option<Self> {
        match kind {
            ObjectKind::Peacekeeper | ObjectKind::Guard => Some(InterfaceTemplate::BasicCombatUnit),
            ObjectKind::Barracks => Some(InterfaceTemplate::Barracks),
            ObjectKind::CrystalPatch
            | ObjectKind::SupplyDeliveryStation
            | ObjectKind::DeploymentCenter
            | ObjectKind::Generator
            | ObjectKind::ExtractionFacility
            | ObjectKind::ExtractionPlate => None,
        }
    }

    /// Its DefaultState's commands, slot by slot (`CommandSlot::all` order).
    pub fn grid(self) -> &'static [Option<PanelCommand>; 12] {
        match self {
            InterfaceTemplate::BasicCombatUnit => &BASIC_COMBAT_UNIT_GRID,
            InterfaceTemplate::Barracks => &BARRACKS_GRID,
        }
    }
}

/// `systems/combat.md > AttackType > CanTargetGround`, derived from the AttackType.
pub fn can_target_ground(attack_type: AttackType) -> bool {
    matches!(attack_type, AttackType::TailDisjointed | AttackType::DoublyDisjointed)
}

/// `kind`'s commands, slot by slot (`CommandSlot::all` order): its template's grid, plus its own commands in slots
/// the template leaves free (a unit that carries a Riot Shield: [`PanelCommand::RiotShield`] in R).
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

/// Whether objects of `kind` offer `command`: their grid has it ([`grid`]), and a conditional one only when the
/// type can (`> BasicCombatUnitInterfaceState`: Reverse with the UnitBase's CanReverse, AttackGround with the
/// AttackType's CanTargetGround).
pub fn offers(kind: ObjectKind, command: PanelCommand) -> bool {
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
