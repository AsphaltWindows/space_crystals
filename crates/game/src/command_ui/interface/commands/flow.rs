//! How the panel commands flow (`systems/controls/interface.md > ObjectInterfaceState`): whether a command awaits its
//! target ([`PanelCommand::needs_target`]), goes to one instance alone ([`PanelCommand::instance_only`]), and is
//! queued by Shift ([`PanelCommand::queues`]).

use sc_sim::AbilityArea;

use super::PanelCommand;

impl PanelCommand {
    /// Whether the command awaits a target (a StateOnlyTransition to AwaitingTarget): the target commands of
    /// `> BasicCombatUnitInterfaceState > DefaultState commands`, Set Rally Point, the Drone's Set Drop-off, the
    /// Agent's Drop Off, the Recruits' Assist Construction and the Recruits' and the Agent's Repair (slice M5-N8), and
    /// (M5-N5) a Shrine's casts, on a point ("Targeting: AwaitingTarget, left-click a point in vision"), and (M5-K) a
    /// Rocketman's Target Lock, on an enemy unit or structure (`factions/gdo/units/rocketman.md > Placeholder stats`:
    /// "targeted: left-click an enemy unit or structure"). The others are issued at once: HoldPosition and Stop, a
    /// Disciple's Shroud, (M5-N11) a Prayer-mobile's casts, round it and ahead of it
    /// (`factions/cults/units/prayer_mobile.md > Placeholder stats`: "**Inspire** (hotkey **R**, instant, no target)",
    /// "**Suppress** (hotkey **F**, instant, no target)"), (M6-GD) a Drone Operator's Launch Drone, over itself, and
    /// Build Drone, (M6-GH1) an aircraft's Return to Base, (M6-GA) a vehicle's Deploy and Undeploy, where it stands,
    /// (M6-CM) a Disciple's Mirage ("An **instant** spell"), (M6-SB) a tank's toggle, (M6-ST1) a carrier's Unload All,
    /// round it, Raise and Lower Shield, Convert, a producer's Train and Cancel Production, a constructor's Construct
    /// and Cancels, a Tunnel's Eject and Cancel; or they enter a state of their own: a build menu, the EjectMenu, the
    /// ExpandMenu, AwaitingPlacement (an Agent's Build, a Tunnel's expansion and a Recruits' Construct menu entry too);
    /// or they change the selection (the Egress Node's F).
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
            | PanelCommand::Repair
            | PanelCommand::TargetLock => true,
            PanelCommand::Cast(ability) => matches!(ability.def().area, AbilityArea::AtPoint | AbilityArea::AnyPoint),
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
            | PanelCommand::Place(_)
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
            | PanelCommand::ReleaseOne(_)
            | PanelCommand::Shroud
            | PanelCommand::Launch
            | PanelCommand::Buy(_)
            | PanelCommand::ReturnToBase
            | PanelCommand::VehicleDeploy(_)
            | PanelCommand::Mirage
            | PanelCommand::Toggle { .. }
            | PanelCommand::UnloadAll => false,
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
    /// (`sc_sim::ProductionCommand::Research`), as a Train goes to every Barracks (design change 0083). So does a
    /// Disciple's Shroud (slice M5-N4b): "each R press starts one Disciple's channel; Tab to the next"
    /// (`factions/cults/units/disciple.md > Placeholder stats`), the Disciple being Ungroupable, so that the
    /// ActiveGroup is the one Disciple Tabbed to. So does a Shrine's cast (slice M5-N5), which spends that Shrine's
    /// energy: "Groupable **false** (user, 2026-11-26; Q0118: each press casts from one Shrine)"
    /// (`factions/cults/structures/shrine.md > Placeholder stats`). So does (M6-CM) a Disciple's Mirage, its F, which
    /// spends that Disciple's energy: one cast a press, as its Shroud.
    pub fn instance_only(self) -> bool {
        matches!(
            self,
            PanelCommand::BuildMenu(_)
                | PanelCommand::Construct(_)
                | PanelCommand::CancelConstruction
                | PanelCommand::CancelReady
                | PanelCommand::PlaceReady(_)
                | PanelCommand::Place(_)
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
                | PanelCommand::Shroud
                | PanelCommand::Mirage
                | PanelCommand::Cast(_)
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
    /// Recruits' Build and Assist (`factions/cults/units/recruit.md > Unit Commands`), and a Repair (slice M5-N8). So
    /// is a Disciple's Shroud, an order (`sc_sim::OrderKind::Shroud`), which Shift queues behind the Disciple's others:
    /// **Placeholder (not in the design: Shift's meaning for a spell)**, as for any unit order
    /// (`systems/controls/commands.md > Unit Command`). A Shrine's cast is no order but a command of a structure, which
    /// has no order queue (plan decision T76): a Shift-click casts it like a plain click and ends the targeting, as a
    /// rally point's (**Placeholder (not in the design: Shift's meaning for a cast)**). (M5-K) So is a Rocketman's
    /// Target Lock an order (`sc_sim::OrderKind::TargetLock`), which Shift queues behind the Rocketman's others, as the
    /// orders Shift queues behind it wait for it to end (`factions/gdo/units/rocketman.md > Target Lock`: "a
    /// Shift-queued order waits behind the lock and runs when it ends", design change 0438, answering Q0171; plan
    /// decision T76: "Shift+R after a Move queues a lock"). (M6-GD) So is a Drone Operator's Launch Drone, an order
    /// (`sc_sim::OrderKind::Launch`, plan decision T91: "untargeted and queueable"): Shift+R after a Move launches where
    /// the walk ends. Its Build Drone is an ability (`sc_sim::Ability::Buy`), which keeps the orders as a Riot Shield
    /// does: Shift changes nothing. (M6-GH1) An aircraft's Return to Base is an order too (`sc_sim::OrderKind::ReturnToBase`),
    /// which Shift queues. (M6-GA) So is a vehicle's Deploy and Undeploy (`sc_sim::OrderKind::VehicleDeploy`;
    /// `factions/gdo/units/artillery_rig.md > Modes`: "**Queueable**: Shift-Deploy waits until the earlier orders are
    /// done (e.g. move here, then deploy)"). (M6-CM) A Disciple's Mirage is an ability (`sc_sim::Ability::Mirage`), cast
    /// at once: Shift changes nothing (**Placeholder (not in the design: Shift's meaning for an instant spell)**). (M6-SB)
    /// A tank's toggle is an ability (`sc_sim::Ability::Sludge`): "Toggles follow the mode-switch rule (keep orders, not
    /// queueable)" (`factions/syndicate/units/shadebringer.md > Placeholder stats`).
    /// (M6-ST1) A carrier's Unload All is an order (`sc_sim::OrderKind::UnloadAll`, untargeted and queueable, as the
    /// Launch): Shift-F after a Move unloads where the move ends.
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
            | PanelCommand::Repair
            | PanelCommand::Shroud
            | PanelCommand::TargetLock
            | PanelCommand::Launch
            | PanelCommand::ReturnToBase
            | PanelCommand::VehicleDeploy(_)
            | PanelCommand::UnloadAll => true,
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
            | PanelCommand::Place(_)
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
            | PanelCommand::ReleaseOne(_)
            | PanelCommand::Cast(_)
            | PanelCommand::Buy(_)
            | PanelCommand::Mirage
            | PanelCommand::Toggle { .. } => false,
        }
    }
}
