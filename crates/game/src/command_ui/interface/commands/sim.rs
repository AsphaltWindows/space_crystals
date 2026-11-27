//! What pressing a panel command gives the sim: an order, an ability, a production, holding, construction or Tunnel
//! command, or nothing yet ([`PanelCommand::has_sim_command`]), and the building it makes.

use sc_sim::{Ability, ConstructionCommand, HoldCommand, ObjectKind, OrderKind, ProductionCommand, TunnelCommand};

use super::PanelCommand;

impl PanelCommand {
    /// The order of a unit command that needs no target: Hold Position's, Stop's, (M5-N4b) a Disciple's Shroud,
    /// channelled where it stands, (M6-GD) a Drone Operator's Launch Drone, over itself, (M6-GH1) an aircraft's
    /// Return to Base, to its Airfield, and (M6-GA) a vehicle's Deploy or Undeploy, where it stands, into the type named.
    pub fn immediate_order(self) -> Option<OrderKind> {
        match self {
            PanelCommand::HoldPosition => Some(OrderKind::Hold),
            PanelCommand::Stop => Some(OrderKind::Stop),
            PanelCommand::Shroud => Some(OrderKind::Shroud),
            PanelCommand::Launch => Some(OrderKind::Launch),
            PanelCommand::ReturnToBase => Some(OrderKind::ReturnToBase),
            PanelCommand::VehicleDeploy(into) => Some(OrderKind::VehicleDeploy { into }),
            _ => None,
        }
    }

    /// The ability of an ability command issued at once (`sc_sim::Command::Ability`): Raise Shield's raises, Lower
    /// Shield's lowers (each type's panel has the one that turns its shields the other way, design change 0173),
    /// Convert's flips the target form, and (M6-GD) a Drone Operator's Build Drone buys one. Set Drop-off's waits for
    /// its target, the drop-off (`crate::command_ui::resolution::target_resolution`).
    pub fn ability(self) -> Option<Ability> {
        match self {
            PanelCommand::RaiseShield => Some(Ability::RiotShield { raise: true }),
            PanelCommand::LowerShield => Some(Ability::RiotShield { raise: false }),
            PanelCommand::Convert(_) => Some(Ability::Convert),
            PanelCommand::Buy(_) => Some(Ability::Buy),
            _ => None,
        }
    }

    /// The production command of a producer's command that needs no target (`sc_sim::Command::Production`): Train
    /// and Cancel Production, a pool building's KindMenu entries, a training of a kind and a Release, and an Altar's
    /// AugmentMenu\[UnitType, State\] entries, an order and a Release of the type and the state (slice M5-N1b), and a
    /// Temple's Research (slice M5-N3). Set Rally Point's waits for its target
    /// (`crate::command_ui::resolution::target_resolution`).
    pub fn production(self) -> Option<ProductionCommand> {
        match self {
            PanelCommand::Train(kind) => Some(ProductionCommand::Train(kind)),
            PanelCommand::CancelProduction => Some(ProductionCommand::CancelProduction),
            PanelCommand::TrainFrom(kind, from) => Some(ProductionCommand::TrainFrom { kind, from }),
            PanelCommand::Release(from) => Some(ProductionCommand::Release(from)),
            PanelCommand::Augment(kind, from, order) => Some(ProductionCommand::Augment { kind, from, order }),
            PanelCommand::ReleaseOf(kind, from) => Some(ProductionCommand::ReleaseOf { kind, from }),
            PanelCommand::Research(tech) => Some(ProductionCommand::Research(tech)),
            _ => None,
        }
    }

    /// The holding command of a command issued at once to buildings that hold units (`sc_sim::Command::Holding`):
    /// Eject All, and (M5-N3) a Temple's release of one Recruit of a kind.
    pub fn holding(self) -> Option<HoldCommand> {
        match self {
            PanelCommand::EjectAll => Some(HoldCommand::EjectAll),
            PanelCommand::ReleaseOne(kind) => Some(HoldCommand::ReleaseOne(kind)),
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

    /// The Tunnel command of a Tunnel's command issued at once (`sc_sim::Command::Tunnel`): an EjectMenu tile's Eject,
    /// Q's Upgrade, and X's Cancel of the Tunnel's operation. An ExpandMenu entry's Expand waits for its click in
    /// AwaitingPlacement (`crate::placement`).
    pub fn tunnel(self) -> Option<TunnelCommand> {
        match self {
            PanelCommand::Eject(kind) => Some(TunnelCommand::Eject { kind }),
            PanelCommand::CancelOperation => Some(TunnelCommand::Cancel),
            PanelCommand::UpgradeTunnel => Some(TunnelCommand::Upgrade),
            _ => None,
        }
    }

    /// Whether the sim has a command for it yet: a building or a unit that isn't in the game has none at all (Reverse
    /// has had an order since slice M5-V2, AttackGround since slice M4-O). A StateOnlyTransition needs
    /// none (a build menu, the Egress Node's F). A Temple's key in progress has its line's Research, which the gate
    /// refuses meanwhile (`PanelCommand::Researching`).
    pub fn has_sim_command(self) -> bool {
        !matches!(self, PanelCommand::Planned(_))
    }

    /// The building it constructs, places, warps in or builds: a build menu's entry, a ready building's slot, the
    /// Deployment Center's E: Place (design change 0388), an ExpandMenu entry, a worker's Build (an Agent's, a
    /// Recruit's Construct menu entry: slice M5-J1).
    pub fn building(self) -> Option<ObjectKind> {
        match self {
            PanelCommand::Construct(kind)
            | PanelCommand::PlaceReady(kind)
            | PanelCommand::Place(kind)
            | PanelCommand::WarpIn(kind)
            | PanelCommand::Expand(kind)
            | PanelCommand::Build(kind) => Some(kind),
            _ => None,
        }
    }
}
