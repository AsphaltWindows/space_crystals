//! The command panel's commands (`systems/controls/selection_and_hud.md > CommandPanel`), for units and producing
//! structures alike (plan decision T10: one command flow, not a structure UI beside the unit UI): [`PanelCommand`],
//! and what pressing each one gives the sim.

use sc_sim::{Ability, ObjectKind, OrderKind, ProductionCommand};

/// A command of the command panel: the unit commands of `systems/controls/commands.md`, as
/// `> BasicCombatUnitInterfaceState` lists them, and those of the producing structures
/// (`factions/gdo/structures.md > Barracks > ObjectInterfaceState[Barracks]`, `> Standard Slot Assignments`).
#[derive(Copy, Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum PanelCommand {
    Move,
    Reverse,
    HoldPosition,
    Attack,
    Patrol,
    AttackGround,
    Stop,
    /// `factions/gdo/units.md > Peacekeeper > Riot Shield (toggled ability)`: raises or lowers the shields, at once.
    RiotShield,
    /// `> ObjectInterfaceState[Barracks]`: "Q: Build Peacekeeper": pays for a unit of this kind and adds it to the
    /// BuildQueue, at once (`sc_sim::ProductionCommand::Train`).
    Train(ObjectKind),
    /// `> Standard Slot Assignments > X`, "X: Cancel Production": takes the last unit queued off the BuildQueue and
    /// refunds it, at once.
    CancelProduction,
    /// `> Standard Slot Assignments > C`, "C: Set Rally Point": awaits the rally point's target
    /// (`> AwaitingTarget[SetRallyPoint]`).
    SetRallyPoint,
}

impl PanelCommand {
    /// Whether the command awaits a target (a StateOnlyTransition to AwaitingTarget): `> BasicCombatUnitInterfaceState
    /// > DefaultState commands`' target commands, and Set Rally Point. The others are issued at once: HoldPosition
    /// and Stop, the Riot Shield, and a producer's Train and Cancel Production.
    pub fn needs_target(self) -> bool {
        match self {
            PanelCommand::Move
            | PanelCommand::Reverse
            | PanelCommand::Attack
            | PanelCommand::Patrol
            | PanelCommand::AttackGround
            | PanelCommand::SetRallyPoint => true,
            PanelCommand::HoldPosition
            | PanelCommand::Stop
            | PanelCommand::RiotShield
            | PanelCommand::Train(_)
            | PanelCommand::CancelProduction => false,
        }
    }

    /// The order of a unit command that needs no target.
    pub fn immediate_order(self) -> Option<OrderKind> {
        match self {
            PanelCommand::HoldPosition => Some(OrderKind::Hold),
            PanelCommand::Stop => Some(OrderKind::Stop),
            _ => None,
        }
    }

    /// The ability of a toggled command, pressed while it is `on` (all its recipients' shields up or going up):
    /// the one that turns it the other way.
    pub fn ability(self, on: bool) -> Option<Ability> {
        match self {
            PanelCommand::RiotShield => Some(Ability::RiotShield { raise: !on }),
            _ => None,
        }
    }

    /// The production command of a producer's command that needs no target (`sc_sim::Command::Production`): Train
    /// and Cancel Production. Set Rally Point's waits for its target (`super::super::resolution::target_rally`).
    pub fn production(self) -> Option<ProductionCommand> {
        match self {
            PanelCommand::Train(kind) => Some(ProductionCommand::Train(kind)),
            PanelCommand::CancelProduction => Some(ProductionCommand::CancelProduction),
            _ => None,
        }
    }

    /// Whether the sim has a command for it yet: Reverse and AttackGround have no order (`sc_sim::OrderKind`).
    pub fn has_sim_command(self) -> bool {
        !matches!(self, PanelCommand::Reverse | PanelCommand::AttackGround)
    }

    /// Whether Shift queues it (`systems/controls/commands.md > Unit Command`): the orders of units. A Shift-click on
    /// such a command's target also keeps the target awaited (`> AwaitingTarget > Shift while targeting`, design
    /// change 0070). Abilities and production commands are never queued. Set Rally Point is no Unit Command: a
    /// producer has one rally point, which nothing queues (`factions/gdo/structures.md > Barracks >
    /// BarracksInstanceState`), so a Shift-click sets it like a plain click and ends the targeting. **Placeholder
    /// reading**: the design gives Shift no meaning for a rally point; this follows its rule for a builder that
    /// can't carry out more queued buildings (`> Shift while placing`: "a Shift-click sends it and ends placement
    /// like a plain click").
    pub fn queues(self) -> bool {
        match self {
            PanelCommand::Move
            | PanelCommand::Reverse
            | PanelCommand::HoldPosition
            | PanelCommand::Attack
            | PanelCommand::Patrol
            | PanelCommand::AttackGround
            | PanelCommand::Stop => true,
            PanelCommand::RiotShield
            | PanelCommand::Train(_)
            | PanelCommand::CancelProduction
            | PanelCommand::SetRallyPoint => false,
        }
    }

    /// The button's label: a placeholder for its icon (Q0001). A unit to train is named by its type.
    pub fn label(self) -> &'static str {
        match self {
            PanelCommand::Move => "Move",
            PanelCommand::Reverse => "Reverse",
            PanelCommand::HoldPosition => "Hold",
            PanelCommand::Attack => "Attack",
            PanelCommand::Patrol => "Patrol",
            PanelCommand::AttackGround => "Atk Gnd",
            PanelCommand::Stop => "Stop",
            PanelCommand::RiotShield => "Raise Shield",
            PanelCommand::Train(kind) => kind.def().name,
            PanelCommand::CancelProduction => "Cancel",
            PanelCommand::SetRallyPoint => "Rally",
        }
    }
}
