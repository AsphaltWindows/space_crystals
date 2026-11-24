//! The command panel's commands (`systems/controls/selection_and_hud.md > CommandPanel`), for units, producing
//! structures and constructors alike (plan decision T10: one command flow, not a structure UI beside the unit UI):
//! [`PanelCommand`], and what pressing each one gives the sim.

use sc_sim::{Ability, ConstructionCommand, ObjectKind, OrderKind, ProductionCommand};

use super::BuildMenu;

/// A command of the command panel: the unit commands of `systems/controls/commands.md`, as
/// `> BasicCombatUnitInterfaceState` lists them, those of the producing structures
/// (`factions/gdo/structures.md > Barracks > ObjectInterfaceState[Barracks]`, `> Standard Slot Assignments`), and
/// those of the constructors (`factions/gdo/structures.md > DeploymentCenter > ObjectInterfaceState[DeploymentCenter]`,
/// `factions/gdo/economy.md > ExtractionFacility > ObjectInterfaceState[ExtractionFacility]`).
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
    /// `> ObjectInterfaceState[DeploymentCenter] > DefaultState`: "Q: Basic Construction" and "W: Advanced
    /// Construction" (design change 0104): open that build menu (a StateOnlyTransition).
    BuildMenu(BuildMenu),
    /// A build menu's entry of an idle constructor (`> BuildMenu[Basic]`: "Generator: deducts 100 Space Crystals,
    /// sets CurrentConstruction = Generator"), or the Extraction Facility's "Q: Build Extraction Plate": pays for the
    /// building and starts constructing it, at once (`sc_sim::ConstructionCommand::Construct`).
    Construct(ObjectKind),
    /// "X: Cancel Construction": "refunds full cost, clears CurrentConstruction", at once.
    CancelConstruction,
    /// "X: Cancel Ready Building" (the Extraction Facility's "X: Cancel Ready Plate"): "refunds 75% cost (rounded
    /// down), clears ReadyToPlace", at once.
    CancelReady,
    /// The ready building's own slot, "Select completed building" (design change 0104), the Extraction Facility's
    /// "Q: Place Plate": enters AwaitingPlacement for it (a StateOnlyTransition), where a click places it.
    PlaceReady(ObjectKind),
}

impl PanelCommand {
    /// Whether the command awaits a target (a StateOnlyTransition to AwaitingTarget): the target commands of
    /// `> BasicCombatUnitInterfaceState > DefaultState commands`, and Set Rally Point. The others are issued at once:
    /// HoldPosition and Stop, the Riot Shield, a producer's Train and Cancel Production, a constructor's Construct
    /// and Cancels; or they enter a state of their own: a build menu, AwaitingPlacement.
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
            | PanelCommand::CancelProduction
            | PanelCommand::BuildMenu(_)
            | PanelCommand::Construct(_)
            | PanelCommand::CancelConstruction
            | PanelCommand::CancelReady
            | PanelCommand::PlaceReady(_) => false,
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
    /// and Cancel Production. Set Rally Point's waits for its target (`super::super::resolution::target_resolution`).
    pub fn production(self) -> Option<ProductionCommand> {
        match self {
            PanelCommand::Train(kind) => Some(ProductionCommand::Train(kind)),
            PanelCommand::CancelProduction => Some(ProductionCommand::CancelProduction),
            _ => None,
        }
    }

    /// The construction command of a constructor's command issued at once (`sc_sim::Command::Construction`):
    /// Construct, Cancel Construction and Cancel Ready Building. The Place of a ready building waits for its click
    /// in AwaitingPlacement (`crate::placement`).
    pub fn construction(self) -> Option<ConstructionCommand> {
        match self {
            PanelCommand::Construct(kind) => Some(ConstructionCommand::Construct(kind)),
            PanelCommand::CancelConstruction => Some(ConstructionCommand::CancelConstruction),
            PanelCommand::CancelReady => Some(ConstructionCommand::CancelReady),
            _ => None,
        }
    }

    /// Whether the sim has a command for it yet: Reverse and AttackGround have no order (`sc_sim::OrderKind`).
    pub fn has_sim_command(self) -> bool {
        !matches!(self, PanelCommand::Reverse | PanelCommand::AttackGround)
    }

    /// Whether it belongs to one instance's own state: a constructor's commands, which read and change what that one
    /// Deployment Center or Extraction Facility constructs (`> SelectionGroup`: for an Ungroupable type "the
    /// ObjectInterfaceState can reliably read that instance's specific game state"; design change 0071: "a Build
    /// order from the panel goes only to the active Recruit"). Such a command goes to the ActiveGroup only, never to
    /// the other selected instances of its type (`super::recipients`).
    pub fn instance_only(self) -> bool {
        matches!(
            self,
            PanelCommand::BuildMenu(_)
                | PanelCommand::Construct(_)
                | PanelCommand::CancelConstruction
                | PanelCommand::CancelReady
                | PanelCommand::PlaceReady(_)
        )
    }

    /// Whether Shift queues it (`systems/controls/commands.md > Unit Command`): the orders of units. A Shift-click on
    /// such a command's target also keeps the target awaited (`> AwaitingTarget > Shift while targeting`, design
    /// change 0070). Abilities, production and construction commands are never queued. Set Rally Point is no Unit
    /// Command: a producer has one rally point, which nothing queues (`factions/gdo/structures.md > Barracks >
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
            | PanelCommand::SetRallyPoint
            | PanelCommand::BuildMenu(_)
            | PanelCommand::Construct(_)
            | PanelCommand::CancelConstruction
            | PanelCommand::CancelReady
            | PanelCommand::PlaceReady(_) => false,
        }
    }

    /// The button's label: a placeholder for its icon (Q0001), short enough for the button. A unit to train and a
    /// building to construct or place are named by their type's short label, as the map and the portraits show it
    /// (`hud::selection_panel::short_label`).
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
            PanelCommand::Train(ObjectKind::Peacekeeper) => "Train PK",
            // No other unit is trained yet.
            PanelCommand::Train(kind) => kind.def().name,
            PanelCommand::CancelProduction | PanelCommand::CancelConstruction => "Cancel",
            PanelCommand::SetRallyPoint => "Rally",
            PanelCommand::BuildMenu(BuildMenu::Basic) => "Basic",
            PanelCommand::BuildMenu(BuildMenu::Advanced) => "Advanced",
            PanelCommand::Construct(kind) => match kind {
                ObjectKind::Generator => "Build GN",
                ObjectKind::Barracks => "Build BR",
                ObjectKind::ExtractionFacility => "Build EF",
                ObjectKind::ExtractionPlate => "Build EP",
                // Nothing else is constructed yet.
                _ => kind.def().name,
            },
            PanelCommand::CancelReady => "Cancel Ready",
            PanelCommand::PlaceReady(kind) => match kind {
                ObjectKind::Generator => "Place GN",
                ObjectKind::Barracks => "Place BR",
                ObjectKind::ExtractionFacility => "Place EF",
                ObjectKind::ExtractionPlate => "Place EP",
                _ => kind.def().name,
            },
        }
    }
}
