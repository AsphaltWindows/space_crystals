//! How the panel commands flow (`systems/controls/interface.md > ObjectInterfaceState`): whether a command awaits its
//! target ([`PanelCommand::needs_target`]), goes to one instance alone ([`PanelCommand::instance_only`]), and is
//! queued by Shift ([`PanelCommand::queues`]).

use super::PanelCommand;

impl PanelCommand {
    /// Whether the command awaits a target (a StateOnlyTransition to AwaitingTarget): the target commands of
    /// `> BasicCombatUnitInterfaceState > DefaultState commands`, Set Rally Point, the Drone's Set Drop-off, the
    /// Agent's Drop Off, the Recruits' Assist Construction and the Recruits' and the Agent's Repair (slice M5-N8). The
    /// others are issued at once: HoldPosition and Stop, Raise and Lower Shield, Convert, a producer's Train and Cancel
    /// Production, a constructor's Construct and Cancels, a Tunnel's Eject and Cancel; or they enter a state of their
    /// own: a build menu, the EjectMenu, the ExpandMenu, AwaitingPlacement (an Agent's Build, a Tunnel's expansion and
    /// a Recruits' Construct menu entry too); or they change the selection (the Egress Node's F).
    pub fn needs_target(self) -> bool {
        match self {
            PanelCommand::Move
            | PanelCommand::Reverse
            | PanelCommand::Attack
            | PanelCommand::Patrol
            | PanelCommand::AttackGround
            | PanelCommand::SetRallyPoint
            | PanelCommand::SetDropOff
            | PanelCommand::DropOff
            | PanelCommand::Assist
            | PanelCommand::Repair => true,
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
            | PanelCommand::SelectReturningDrones
            | PanelCommand::EjectMenu
            | PanelCommand::Eject(_)
            | PanelCommand::Build(_)
            | PanelCommand::ExpandMenu
            | PanelCommand::Expand(_)
            | PanelCommand::CancelOperation
            | PanelCommand::UpgradeTunnel
            | PanelCommand::KindMenu(_)
            | PanelCommand::TrainFrom(..)
            | PanelCommand::Release(_)
            | PanelCommand::EjectAll
            | PanelCommand::AugmentMenu(_)
            | PanelCommand::AugmentState(..)
            | PanelCommand::Augment(..)
            | PanelCommand::ReleaseOf(..)
            | PanelCommand::Research(_)
            | PanelCommand::Researching(_)
            | PanelCommand::ReleaseOne(_) => false,
        }
    }

    /// Whether it belongs to one instance's own state: a constructor's commands, which read and change what that one
    /// Deployment Center or Extraction Facility constructs (`systems/controls/selection.md > SelectionGroup`: for an
    /// Ungroupable type "the ObjectInterfaceState can reliably read that instance's specific game state"), and a
    /// Beacon's Warp In (design change 0122: "Warp In applies to the Beacon the player has Tabbed to"). Such a command
    /// goes to the ActiveGroup only, never to the other selected instances of its type (`super::super::recipients`). So
    /// do a Deploy, which one Drone of the ActiveGroup takes (design change 0125), the Cancel of the objects that
    /// transform, whose panel shows it, and the Egress Node's F, which selects the Drones of "this Egress Node"
    /// (Ungroupable: the one Tabbed to). So do a Tunnel's EjectMenu and its tiles, which eject "from this Tunnel's Side
    /// A" (`factions/syndicate/tunnel_menus.md > EjectMenu`; Ungroupable too: the one Tabbed to), and its ExpandMenu,
    /// its entries, its Upgrade and its Cancel, which read and change this Tunnel's one operation
    /// (`factions/syndicate/tunnel_menus.md > ExpandMenu`: "**this** Tunnel's tier"). So does an Agent's Build: "each
    /// Agent takes exactly one placement", and "Placement never moves on to another selected worker"
    /// (`systems/controls/interface.md > AwaitingTarget > Shift while placing`: "to place with the next Agent, the
    /// player presses Tab"). A Recruits' Build goes to the ActiveGroup too, which is every selected Recruit, the
    /// Recruit being Groupable (slice M5-J1; `factions/cults/units/recruit.md > Recruit ObjectInterfaceState`:
    /// "Construct sends every selected Recruit to build the **same** building together", design change 0072), and never
    /// to the other types' units. So do a pool building's kind menus, their entries and its Eject All, which read and
    /// change its own pool (design change 0337: "the count inside (unreserved)"), and an Altar's menus and their
    /// entries alike (slice M5-N1b); and every command of a pool building goes to the one Tabbed to, its X and C too
    /// (design change 0348 point 5: "a command goes to one building"; `super::super::is_common`). So does a Temple's
    /// release of one Recruit of a kind, which reads and changes its own staff (slice M5-N3), to every Temple of the
    /// ActiveGroup, the Temple being Groupable (`factions/cults/structures/temple.md > Placeholder stats`: "Groupable
    /// true"); not its Research, which goes to every Temple of the selection that offers it, one of them taking it
    /// (`sc_sim::ProductionCommand::Research`), as a Train goes to every Barracks (design change 0083).
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
                | PanelCommand::EjectMenu
                | PanelCommand::Eject(_)
                | PanelCommand::Build(_)
                | PanelCommand::ExpandMenu
                | PanelCommand::Expand(_)
                | PanelCommand::CancelOperation
                | PanelCommand::UpgradeTunnel
                | PanelCommand::KindMenu(_)
                | PanelCommand::TrainFrom(..)
                | PanelCommand::Release(_)
                | PanelCommand::EjectAll
                | PanelCommand::AugmentMenu(_)
                | PanelCommand::AugmentState(..)
                | PanelCommand::Augment(..)
                | PanelCommand::ReleaseOf(..)
                | PanelCommand::ReleaseOne(_)
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
    /// (the same placeholder reading). The Agent's Drop Off is a unit's order (`factions/syndicate/units/agent.md >
    /// Unit Commands`), and so is its Build, which Shift queues as it is placed (`> Shift while placing`); so are the
    /// Recruits' Build and Assist (`factions/cults/units/recruit.md > Unit Commands`), and a Repair (slice M5-N8).
    pub fn queues(self) -> bool {
        match self {
            PanelCommand::Move
            | PanelCommand::Reverse
            | PanelCommand::HoldPosition
            | PanelCommand::Attack
            | PanelCommand::Patrol
            | PanelCommand::AttackGround
            | PanelCommand::Stop
            | PanelCommand::Deploy
            | PanelCommand::Build(_)
            | PanelCommand::DropOff
            | PanelCommand::Assist
            | PanelCommand::Repair => true,
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
            | PanelCommand::SelectReturningDrones
            | PanelCommand::EjectMenu
            | PanelCommand::Eject(_)
            | PanelCommand::ExpandMenu
            | PanelCommand::Expand(_)
            | PanelCommand::CancelOperation
            | PanelCommand::UpgradeTunnel
            | PanelCommand::KindMenu(_)
            | PanelCommand::TrainFrom(..)
            | PanelCommand::Release(_)
            | PanelCommand::EjectAll
            | PanelCommand::AugmentMenu(_)
            | PanelCommand::AugmentState(..)
            | PanelCommand::Augment(..)
            | PanelCommand::ReleaseOf(..)
            | PanelCommand::Research(_)
            | PanelCommand::Researching(_)
            | PanelCommand::ReleaseOne(_) => false,
        }
    }
}
