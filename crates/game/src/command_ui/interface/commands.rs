//! The command panel's commands (`systems/controls/selection_and_hud.md > CommandPanel`), for units, producing
//! structures and constructors alike (plan decision T10: one command flow, not a structure UI beside the unit UI):
//! [`PanelCommand`], and what pressing each one gives the sim.

use sc_sim::{Ability, ConstructionCommand, ObjectKind, OrderKind, ProductionCommand};

use super::{BuildMenu, Planned};
use crate::kinds::{self, short_label};

/// A command of the command panel: the unit commands of `systems/controls/commands.md`, as `>
/// BasicCombatUnitInterfaceState` lists them, those of the producing structures (`factions/gdo/structures/barracks.md >
/// ObjectInterfaceState[Barracks]`, `> Standard Slot Assignments`), and those of the constructors
/// (`factions/gdo/structures/deployment_center.md > ObjectInterfaceState[DeploymentCenter]`, `factions/gdo/economy.md >
/// ExtractionFacility > ObjectInterfaceState[ExtractionFacility]`), and the Colonists' Beacon, Drone, Egress Node and
/// transforming objects (`factions/colonists/economy.md > Beacon > Beacon ObjectInterfaceState`,
/// `factions/colonists/units/drone.md > Drone ObjectInterfaceState`, `factions/colonists/economy.md > Egress Node >
/// Egress Node ObjectInterfaceState`, `factions/colonists/economy.md > Beacon Deployment > While transforming: only
/// Cancel`; slices M3-J1 and M3-J2, design change 0124).
#[derive(Copy, Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum PanelCommand {
    Move,
    Reverse,
    HoldPosition,
    Attack,
    Patrol,
    AttackGround,
    Stop,
    /// `factions/gdo/units/peacekeeper.md > Riot Shield > Two unit types` (design change 0173): "The Peacekeeper's
    /// panel shows **Raise Shield**": each Peacekeeper becomes a Peacekeeper (Shielded), its shield going up from
    /// where it is, at once (`sc_sim::Ability::RiotShield`; one Firing or in Cooldown once its shot is over, design
    /// change 0172).
    RaiseShield,
    /// The same's "Peacekeeper (Shielded)'s shows **Lower Shield**": each Peacekeeper (Shielded) becomes a
    /// Peacekeeper, its shield going down from where it is, at once.
    LowerShield,
    /// `factions/colonists/units/sabre_legionnaire.md > ObjectInterfaceState`: "R: Convert to Rifle Legionnaire", and
    /// the Rifle Legionnaire's "R: Convert to Sabre Legionnaire": each form's own command, named for the form it
    /// converts to, so that in a selection of both it goes to the ActiveGroup alone
    /// (`factions/colonists/units/weapon_conversion.md > Convert Command`, design change 0198: "It goes only to the
    /// ActiveGroup"), as Raise and Lower Shield do. At once (`sc_sim::Ability::Convert`): it flips each unit's target
    /// form, and so its type, at the press (`factions/colonists/units/weapon_conversion.md > A toggle sets a target
    /// form`).
    Convert(ObjectKind),
    /// `factions/gdo/structures/barracks.md > ObjectInterfaceState[Barracks]`: "Q: Build Peacekeeper": pays for a unit
    /// of this kind and adds it to the BuildQueue, at once (`sc_sim::ProductionCommand::Train`).
    Train(ObjectKind),
    /// `> Standard Slot Assignments > X`, "X: Cancel Production": takes the last unit queued off the BuildQueue and
    /// refunds it, at once.
    CancelProduction,
    /// `> Standard Slot Assignments > C`, "C: Set Rally Point": awaits the rally point's target
    /// (`> AwaitingTarget[SetRallyPoint]`).
    SetRallyPoint,
    /// `factions/gdo/structures/deployment_center.md > ObjectInterfaceState[DeploymentCenter] > DefaultState`: "Q:
    /// Basic Construction" and "W: Advanced Construction" (design change 0104): open that build menu (a
    /// StateOnlyTransition).
    BuildMenu(BuildMenu),
    /// A build menu's entry of an idle constructor (`factions/gdo/structures/deployment_center.md > BuildMenu[Basic]`:
    /// "Generator: deducts 100 Space Crystals, sets CurrentConstruction = Generator"), or the Extraction Facility's "Q:
    /// Build Extraction Plate": pays for the building and starts constructing it, at once
    /// (`sc_sim::ConstructionCommand::Construct`).
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
    /// Beacon ObjectInterfaceState`: "Choosing a building enters placement"; `factions/colonists/economy.md > Warp In >
    /// Warp In Command`): enters AwaitingPlacement of the building on the Beacon (a StateOnlyTransition), where a click
    /// warps it in (`sc_sim::ConstructionCommand::WarpIn`).
    WarpIn(ObjectKind),
    /// A Warp In menu's building, or a portal's unit, that isn't in the game yet (design change 0124: "Buildings and
    /// units whose costs are TBD show in their slot, unavailable"): it holds its slot and does nothing.
    Planned(Planned),
    /// `factions/colonists/units/drone.md > Drone ObjectInterfaceState > A: Deploy`: "enters AwaitingPlacement for a
    /// Beacon (StateOnlyTransition)", where a click sends one Drone of the group to deploy there (design change 0125,
    /// `sc_sim::OrderKind::Deploy`).
    Deploy,
    /// `factions/colonists/economy.md > Beacon Deployment > While transforming: only Cancel`: "X" Cancel Deployment and
    /// Cancel Warp In (design change 0116), the one command of an object that transforms, at once
    /// (`sc_sim::ConstructionCommand::CancelTransform`; `factions/colonists/economy.md > Beacon Deployment > Cancel
    /// Deployment`, `factions/colonists/economy.md > Warp In > Cancel Warp In`).
    CancelTransform,
    /// `factions/colonists/units/drone.md > Drone ObjectInterfaceState > S: Set Drop-off Location`: "enters
    /// AwaitingTarget\[SetDropOffLocation\] (StateOnlyTransition)", where a left-click on the player's own Egress Node
    /// (or refinery, none in the game yet) "sets it as the selected Drone(s)' AssignedDropOff
    /// (CommandIssuingTransition, returns to DefaultState)" (`sc_sim::Ability::SetDropOff`, slice M3-J2): every
    /// selected Drone's (`factions/colonists/units/drone.md > Right-Click Resolution`: "Right-click commands and Set
    /// Drop-off go to every selected Drone"; question Q0040 point 9). It "does not interrupt current mining/movement":
    /// their orders go on.
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
    /// others are issued at once: HoldPosition and Stop, Raise and Lower Shield, Convert, a producer's Train and Cancel
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
            | PanelCommand::RaiseShield
            | PanelCommand::LowerShield
            | PanelCommand::Convert(_)
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

    /// The ability of an ability command issued at once (`sc_sim::Command::Ability`): Raise Shield's raises, Lower
    /// Shield's lowers (each type's panel has the one that turns its shields the other way, design change 0173), and
    /// Convert's flips the target form. Set Drop-off's waits for its target, the drop-off
    /// (`super::super::resolution::target_resolution`).
    pub fn ability(self) -> Option<Ability> {
        match self {
            PanelCommand::RaiseShield => Some(Ability::RiotShield { raise: true }),
            PanelCommand::LowerShield => Some(Ability::RiotShield { raise: false }),
            PanelCommand::Convert(_) => Some(Ability::Convert),
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
    /// 0070). Abilities, production and construction commands are never queued (Convert:
    /// `factions/colonists/units/weapon_conversion.md > Orders are kept`, design change 0151: "holding Shift changes
    /// nothing"; the Riot Shield's alike, `factions/gdo/units/peacekeeper.md > Riot Shield > Not queueable`). Set Rally
    /// Point is no Unit Command: a producer has one rally point, which nothing queues
    /// (`factions/gdo/structures/barracks.md > BarracksInstanceState`), so a Shift-click sets it like a plain click and
    /// ends the targeting. **Placeholder reading**: the design gives Shift no meaning for a rally point; this follows
    /// its rule for a builder that can't carry out more queued buildings (`> Shift while placing`: "a Shift-click sends
    /// it and ends placement like a plain click"). A Deploy is a unit's order too, which Shift queues as it is placed
    /// (`> Shift while placing > Exception, Drones`, design change 0125). Set Drop-off is an ability, which "only
    /// updates AssignedDropOff": a Shift-click sets it and ends the targeting like a plain click, as a rally point's
    /// (the same placeholder reading).
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
            PanelCommand::RaiseShield
            | PanelCommand::LowerShield
            | PanelCommand::Convert(_)
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

    /// The button's label: a placeholder for its icon (Q0001), short enough to keep the label size on its button
    /// at 720p (slice M3-K2a: six characters at most, `crate::hud::command_panel`), its full name being
    /// [`Self::name_on`]'s, in the InfoPanel's hint for the button under the pointer (`crate::hud::info_panel::hint`).
    /// What a button makes, a unit to train or a building to construct or warp in (one that isn't in the game yet
    /// too), it shows as an icon would: by its type's tag, as the map and the portraits show it ([`short_label`]).
    /// A ready building's slot is "Place", and every Cancel "Cancel". The others are the command's name or its first
    /// word ("Hold", "Rally", "Raise", "Lower", "Drop" for Set Drop-off Location, "Drones" for Select Returning
    /// Drones), or a shortening of it ("Rev.", "AtkGnd", "Conv.", "Adv.").
    pub fn label(self) -> &'static str {
        match self {
            PanelCommand::Move => "Move",
            PanelCommand::Reverse => "Rev.",
            PanelCommand::HoldPosition => "Hold",
            PanelCommand::Attack => "Attack",
            PanelCommand::Patrol => "Patrol",
            PanelCommand::AttackGround => "AtkGnd",
            PanelCommand::Stop => "Stop",
            PanelCommand::RaiseShield => "Raise",
            PanelCommand::LowerShield => "Lower",
            PanelCommand::Convert(_) => "Conv.",
            PanelCommand::Train(kind) | PanelCommand::Construct(kind) | PanelCommand::WarpIn(kind) => short_label(kind),
            PanelCommand::Planned(planned) => planned.label(),
            PanelCommand::CancelProduction
            | PanelCommand::CancelConstruction
            | PanelCommand::CancelReady
            | PanelCommand::CancelTransform => "Cancel",
            PanelCommand::SetRallyPoint => "Rally",
            PanelCommand::BuildMenu(BuildMenu::Basic) => "Basic",
            PanelCommand::BuildMenu(BuildMenu::Advanced) => "Adv.",
            PanelCommand::PlaceReady(_) => "Place",
            PanelCommand::Deploy => "Deploy",
            PanelCommand::SetDropOff => "Drop",
            PanelCommand::SelectReturningDrones => "Drones",
        }
    }

    /// The command's full name, which its button's label shortens ([`Self::label`]), as the design names it. Where
    /// the design names a command after the object whose panel shows it, the panel names it so ([`Self::name_on`]):
    /// this is the name of the others, and the generic one of those.
    pub fn name(self) -> String {
        let named = |verb: &str, kind: ObjectKind| format!("{verb} {}", kind.def().name);
        match self {
            PanelCommand::Reverse => "Reverse".into(),
            PanelCommand::HoldPosition => "Hold Position".into(),
            PanelCommand::AttackGround => "Attack Ground".into(),
            PanelCommand::RaiseShield => "Raise Shield".into(),
            PanelCommand::LowerShield => "Lower Shield".into(),
            PanelCommand::Convert(kind) => named("Convert to", kind),
            PanelCommand::Train(kind) => named("Train", kind),
            PanelCommand::CancelProduction => "Cancel Production".into(),
            PanelCommand::SetRallyPoint => "Set Rally Point".into(),
            PanelCommand::BuildMenu(BuildMenu::Basic) => "Basic Construction".into(),
            PanelCommand::BuildMenu(BuildMenu::Advanced) => "Advanced Construction".into(),
            PanelCommand::Construct(kind) => named("Build", kind),
            PanelCommand::CancelConstruction => "Cancel Construction".into(),
            PanelCommand::CancelReady => "Cancel Ready Building".into(),
            PanelCommand::PlaceReady(kind) => named("Place", kind),
            PanelCommand::WarpIn(kind) => named("Warp In", kind),
            PanelCommand::Planned(planned) => planned.command_name(),
            PanelCommand::CancelTransform => "Cancel".into(),
            PanelCommand::SetDropOff => "Set Drop-off Location".into(),
            PanelCommand::SelectReturningDrones => "Select Returning Drones".into(),
            PanelCommand::Move
            | PanelCommand::Attack
            | PanelCommand::Patrol
            | PanelCommand::Stop
            | PanelCommand::Deploy => self.label().into(),
        }
    }

    /// The command's full name on the panel of a `kind` ActiveGroup, which the InfoPanel's hint shows
    /// (`super::Panel::name`): [`Self::name`], but for the commands the design names after the object whose panel
    /// shows them, from its panel template (`super::templates::PanelTemplate::names`):
    /// - a Beacon's menus: "Q: Warp In (Basic)", "W: Warp In (Advanced)" (`factions/colonists/economy.md > Beacon >
    ///   Beacon ObjectInterfaceState`), the Deployment Center's being "Basic Construction" and "Advanced
    ///   Construction";
    /// - the Cancel of an object that transforms (`factions/colonists/economy.md > Beacon Deployment > While
    ///   transforming: only Cancel`: "Cancel Deployment / Cancel Warp In"): a Beacon's under deployment, any other
    ///   building's under Warp In (the object is the building from the first frame, `sc_sim::transform`);
    /// - the Extraction Facility's "X: Cancel Ready Plate" (`factions/gdo/economy.md > ExtractionFacility >
    ///   ObjectInterfaceState[ExtractionFacility]`).
    pub fn name_on(self, kind: ObjectKind) -> String {
        let names = kinds::of(kind).template.names;
        match names.iter().find(|&&(command, _)| command == self) {
            Some(&(_, name)) => name.into(),
            None if self == PanelCommand::CancelTransform => "Cancel Warp In".into(),
            None => self.name(),
        }
    }
}
