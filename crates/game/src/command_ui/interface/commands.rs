//! The command panel's commands (`systems/controls/selection_and_hud.md > CommandPanel`), for units, producing
//! structures and constructors alike (plan decision T10: one command flow, not a structure UI beside the unit UI):
//! [`PanelCommand`], and what pressing each one gives the sim.

use sc_sim::{Ability, ConstructionCommand, ObjectKind, OrderKind, ProductionCommand};

use super::{BuildMenu, Planned};

/// A command of the command panel: the unit commands of `systems/controls/commands.md`, as
/// `> BasicCombatUnitInterfaceState` lists them, those of the producing structures
/// (`factions/gdo/structures.md > Barracks > ObjectInterfaceState[Barracks]`, `> Standard Slot Assignments`), and
/// those of the constructors (`factions/gdo/structures.md > DeploymentCenter > ObjectInterfaceState[DeploymentCenter]`,
/// `factions/gdo/economy.md > ExtractionFacility > ObjectInterfaceState[ExtractionFacility]`), and the Colonists'
/// Beacon, Drone, Egress Node and transforming objects (`factions/colonists/economy.md > Beacon > Beacon
/// ObjectInterfaceState`, `factions/colonists/units.md > Drone ObjectInterfaceState`, `factions/colonists/economy.md >
/// Egress Node > Egress Node ObjectInterfaceState`, `> Beacon Deployment > While transforming: only Cancel`; slices
/// M3-J1 and M3-J2, design change 0124).
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
    /// `factions/colonists/units.md > Sabre Legionnaire > ObjectInterfaceState`: "R: Convert to Rifle Legionnaire",
    /// and the Rifle Legionnaire's "R: Convert to Sabre Legionnaire", one command (`> Weapon Conversion > Convert
    /// Command`: "units may be mixed in selection; each converts to its respective other form"), at once
    /// (`sc_sim::Ability::Convert`).
    Convert,
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
    /// A Beacon's Warp In menu entry for a building of its Constructs List (`factions/colonists/economy.md > Beacon >
    /// Beacon ObjectInterfaceState`: "Choosing a building enters placement"; `> Warp In > Warp In Command`): enters
    /// AwaitingPlacement of the building on the Beacon (a StateOnlyTransition), where a click warps it in
    /// (`sc_sim::ConstructionCommand::WarpIn`).
    WarpIn(ObjectKind),
    /// A Warp In menu's building, or a portal's unit, that isn't in the game yet (design change 0124: "Buildings and
    /// units whose costs are TBD show in their slot, unavailable"): it holds its slot and does nothing.
    Planned(Planned),
    /// `factions/colonists/units.md > Drone ObjectInterfaceState > A: Deploy`: "enters AwaitingPlacement for a Beacon
    /// (StateOnlyTransition)", where a click sends one Drone of the group to deploy there (design change 0125,
    /// `sc_sim::OrderKind::Deploy`).
    Deploy,
    /// `factions/colonists/economy.md > Beacon Deployment > While transforming: only Cancel`: "X" Cancel Deployment
    /// and Cancel Warp In (design change 0116), the one command of an object that transforms, at once
    /// (`sc_sim::ConstructionCommand::CancelTransform`; `> Cancel Deployment`, `> Warp In > Cancel Warp In`).
    CancelTransform,
    /// `factions/colonists/units.md > Drone ObjectInterfaceState > S: Set Drop-off Location`: "enters
    /// AwaitingTarget\[SetDropOffLocation\] (StateOnlyTransition)", where a left-click on the player's own Egress
    /// Node (or refinery, none in the game yet) "sets it as the selected Drone(s)' AssignedDropOff
    /// (CommandIssuingTransition, returns to DefaultState)" (`sc_sim::Ability::SetDropOff`, slice M3-J2): every
    /// selected Drone's (`> Right-Click Resolution`: "Right-click commands and Set Drop-off go to every selected
    /// Drone"; question Q0040 point 9). It "does not interrupt current mining/movement": their orders go on.
    SetDropOff,
    /// `factions/colonists/economy.md > Egress Node > Egress Node ObjectInterfaceState > F: Select Returning Drones`:
    /// "Immediate command (StateOnlyTransition). Replaces the current Selection with all Drones whose AssignedDropOff
    /// is this Egress Node and which are currently executing a Gather command" (`sc_sim::Sim::returning_drones`,
    /// slice M3-J2). "Only available if at least one such Drone exists": it shows only then
    /// (`super::Conditions::shows`).
    SelectReturningDrones,
}

impl PanelCommand {
    /// Whether the command awaits a target (a StateOnlyTransition to AwaitingTarget): the target commands of
    /// `> BasicCombatUnitInterfaceState > DefaultState commands`, Set Rally Point and the Drone's Set Drop-off. The
    /// others are issued at once: HoldPosition and Stop, the Riot Shield, Convert, a producer's Train and Cancel
    /// Production, a constructor's Construct and Cancels; or they enter a state of their own: a build menu,
    /// AwaitingPlacement; or they change the selection (the Egress Node's F).
    pub fn needs_target(self) -> bool {
        match self {
            PanelCommand::Move
            | PanelCommand::Reverse
            | PanelCommand::Attack
            | PanelCommand::Patrol
            | PanelCommand::AttackGround
            | PanelCommand::SetRallyPoint
            | PanelCommand::SetDropOff => true,
            PanelCommand::HoldPosition
            | PanelCommand::Stop
            | PanelCommand::RiotShield
            | PanelCommand::Convert
            | PanelCommand::Train(_)
            | PanelCommand::CancelProduction
            | PanelCommand::BuildMenu(_)
            | PanelCommand::Construct(_)
            | PanelCommand::CancelConstruction
            | PanelCommand::CancelReady
            | PanelCommand::PlaceReady(_)
            | PanelCommand::WarpIn(_)
            | PanelCommand::Planned(_)
            | PanelCommand::Deploy
            | PanelCommand::CancelTransform
            | PanelCommand::SelectReturningDrones => false,
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

    /// The ability of an ability command issued at once (`sc_sim::Command::Ability`). A toggled one's
    /// ([`Self::toggles`]), pressed while it is `on` (all its recipients' shields up or going up), is the one that
    /// turns it the other way. Set Drop-off's waits for its target, the drop-off
    /// (`super::super::resolution::target_resolution`).
    pub fn ability(self, on: bool) -> Option<Ability> {
        match self {
            PanelCommand::RiotShield => Some(Ability::RiotShield { raise: !on }),
            PanelCommand::Convert => Some(Ability::Convert),
            _ => None,
        }
    }

    /// Whether it is a toggled ability, which is on or off: the Riot Shield (`factions/gdo/units.md > Peacekeeper >
    /// Riot Shield (toggled ability)`). Convert is no toggle: it starts a conversion, which can't be cancelled
    /// (`factions/colonists/units.md > Weapon Conversion > Completion`).
    pub fn toggles(self) -> bool {
        self == PanelCommand::RiotShield
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
    /// Construct, Cancel Construction and Cancel Ready Building, and a transforming object's Cancel. The Place of a
    /// ready building and a Beacon's Warp In wait for their click in AwaitingPlacement (`crate::placement`).
    pub fn construction(self) -> Option<ConstructionCommand> {
        match self {
            PanelCommand::Construct(kind) => Some(ConstructionCommand::Construct(kind)),
            PanelCommand::CancelConstruction => Some(ConstructionCommand::CancelConstruction),
            PanelCommand::CancelReady => Some(ConstructionCommand::CancelReady),
            PanelCommand::CancelTransform => Some(ConstructionCommand::CancelTransform),
            _ => None,
        }
    }

    /// Whether the sim has a command for it yet: Reverse and AttackGround have no order (`sc_sim::OrderKind`), and a
    /// building or a unit that isn't in the game has none at all. A StateOnlyTransition needs none (a build menu, the
    /// Egress Node's F).
    pub fn has_sim_command(self) -> bool {
        !matches!(
            self,
            PanelCommand::Reverse | PanelCommand::AttackGround | PanelCommand::Planned(_)
        )
    }

    /// The building it constructs, places or warps in: a build menu's entry, or a ready building's slot.
    pub fn building(self) -> Option<ObjectKind> {
        match self {
            PanelCommand::Construct(kind) | PanelCommand::PlaceReady(kind) | PanelCommand::WarpIn(kind) => Some(kind),
            _ => None,
        }
    }

    /// Whether it belongs to one instance's own state: a constructor's commands, which read and change what that one
    /// Deployment Center or Extraction Facility constructs (`> SelectionGroup`: for an Ungroupable type "the
    /// ObjectInterfaceState can reliably read that instance's specific game state"; design change 0071: "a Build
    /// order from the panel goes only to the active Recruit"), and a Beacon's Warp In (design change 0122: "Warp In
    /// applies to the Beacon the player has Tabbed to"). Such a command goes to the ActiveGroup only, never to the
    /// other selected instances of its type (`super::recipients`). So do a Deploy, which one Drone of the ActiveGroup
    /// takes (design change 0125), the Cancel of the objects that transform, whose panel shows it, and the Egress
    /// Node's F, which selects the Drones of "this Egress Node" (Ungroupable: the one Tabbed to).
    pub fn instance_only(self) -> bool {
        matches!(
            self,
            PanelCommand::BuildMenu(_)
                | PanelCommand::Construct(_)
                | PanelCommand::CancelConstruction
                | PanelCommand::CancelReady
                | PanelCommand::PlaceReady(_)
                | PanelCommand::WarpIn(_)
                | PanelCommand::Planned(_)
                | PanelCommand::Deploy
                | PanelCommand::CancelTransform
                | PanelCommand::SelectReturningDrones
        )
    }

    /// Whether Shift queues it (`systems/controls/commands.md > Unit Command`): the orders of units. A Shift-click on
    /// such a command's target also keeps the target awaited (`> AwaitingTarget > Shift while targeting`, design change
    /// 0070). Abilities, production and construction commands are never queued (Convert: `factions/colonists/units.md >
    /// Weapon Conversion > Orders are kept`, design change 0151: "holding Shift changes nothing"). Set Rally Point is
    /// no Unit Command: a producer has one rally point, which nothing queues (`factions/gdo/structures.md > Barracks >
    /// BarracksInstanceState`), so a Shift-click sets it like a plain click and ends the targeting. **Placeholder
    /// reading**: the design gives Shift no meaning for a rally point; this follows its rule for a builder that can't
    /// carry out more queued buildings (`> Shift while placing`: "a Shift-click sends it and ends placement like a
    /// plain click"). A Deploy is a unit's order too, which Shift queues as it is placed (`> Shift while placing >
    /// Exception, Drones`, design change 0125). Set Drop-off is an ability, which "only updates AssignedDropOff": a
    /// Shift-click sets it and ends the targeting like a plain click, as a rally point's (the same placeholder
    /// reading).
    pub fn queues(self) -> bool {
        match self {
            PanelCommand::Move
            | PanelCommand::Reverse
            | PanelCommand::HoldPosition
            | PanelCommand::Attack
            | PanelCommand::Patrol
            | PanelCommand::AttackGround
            | PanelCommand::Stop
            | PanelCommand::Deploy => true,
            PanelCommand::RiotShield
            | PanelCommand::Convert
            | PanelCommand::Train(_)
            | PanelCommand::CancelProduction
            | PanelCommand::SetRallyPoint
            | PanelCommand::BuildMenu(_)
            | PanelCommand::Construct(_)
            | PanelCommand::CancelConstruction
            | PanelCommand::CancelReady
            | PanelCommand::PlaceReady(_)
            | PanelCommand::WarpIn(_)
            | PanelCommand::Planned(_)
            | PanelCommand::CancelTransform
            | PanelCommand::SetDropOff
            | PanelCommand::SelectReturningDrones => false,
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
            PanelCommand::Convert => "Convert",
            PanelCommand::Train(kind) => match kind {
                ObjectKind::Peacekeeper => "Train PK",
                ObjectKind::Drone => "Train DR",
                ObjectKind::SabreLegionnaire => "Train SL",
                // No other unit is trained yet.
                _ => kind.def().name,
            },
            PanelCommand::CancelProduction | PanelCommand::CancelConstruction | PanelCommand::CancelTransform => {
                "Cancel"
            }
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
            PanelCommand::WarpIn(kind) => match kind {
                ObjectKind::EgressNode => "Warp EN",
                ObjectKind::RoboPortal => "Warp RP",
                ObjectKind::BioPortal => "Warp BP",
                // Nothing else is warped in yet.
                _ => kind.def().name,
            },
            PanelCommand::Planned(planned) => planned.label(),
            PanelCommand::Deploy => "Deploy",
            PanelCommand::SetDropOff => "Drop-off",
            PanelCommand::SelectReturningDrones => "Returning",
        }
    }
}
