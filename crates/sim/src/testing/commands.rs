//! One command of every kind ([`every_command`]), for the tests that a replay file keeps everything a player did. Moved
//! out of `testing.rs` whole (slice M5-N3, plan decision T28).

use glam::Vec2;

use crate::command::{Ability, Command, DebugCommand, HoldCommand, TunnelCommand};
use crate::construction::ConstructionCommand;
use crate::defs::{AreaAbility, ObjectKind};
use crate::ids::{ObjectId, PlayerId};
use crate::orders::{OrderKind, Target};
use crate::production::ProductionCommand;
use crate::ruleset::Ruleset;
use crate::scenario::SpawnAt;

/// One command of every kind: every variant of [`Command`], [`OrderKind`], [`Target`], [`Ability`],
/// [`ProductionCommand`], [`ConstructionCommand`], [`DebugCommand`], [`SpawnAt`] and [`AreaAbility`], queued and not,
/// for tests that a replay file keeps everything a player did (`docs/architecture/replays-and-golden-hashes.md > Replays and golden hashes`). The ids
/// needn't exist. The match below stops compiling when a variant is added, as a reminder to add it here.
pub fn every_command() -> Vec<Command> {
    let units = vec![ObjectId(1), ObjectId(2)];
    let point = Vec2::new(48.5, 80.25);
    let kinds = [
        OrderKind::Move(Target::Point(point)),
        OrderKind::Move(Target::Object(ObjectId(3))),
        OrderKind::Attack(ObjectId(3)),
        OrderKind::AttackMove(point),
        OrderKind::Patrol(point),
        OrderKind::Hold,
        OrderKind::Stop,
        OrderKind::Gather(ObjectId(4)),
        OrderKind::DropOff(ObjectId(5)),
        OrderKind::Deploy {
            origin: crate::units::TilePos::new(7, 3),
        },
        OrderKind::Enter(ObjectId(6)),
        OrderKind::AttackGround(point),
        OrderKind::Build {
            kind: ObjectKind::Tunnel,
            origin: crate::units::TilePos::new(9, 4),
            orientation: crate::scenario::BuildingOrientation {
                rotation: crate::object::Rotation::R90,
                flip_h: false,
                flip_v: true,
            },
        },
        OrderKind::Assist(ObjectId(7)),
        OrderKind::Repair(ObjectId(8)),
        OrderKind::Shroud,
        OrderKind::Reverse(point),
        OrderKind::TargetLock(ObjectId(9)),
        OrderKind::Launch,
        OrderKind::Rebase(ObjectId(10)),
        OrderKind::ReturnToBase,
        OrderKind::VehicleDeploy {
            into: ObjectKind::ArtilleryRigDeployed,
        },
        OrderKind::UnloadAll,
        OrderKind::PickUpSupplies(ObjectId(11)),
        OrderKind::DropOffSupplies(ObjectId(12)),
        OrderKind::AttachToTower(ObjectId(12)),
    ];
    let mut commands: Vec<Command> = kinds
        .into_iter()
        .flat_map(|kind| {
            [false, true].map(|queue| Command::Order {
                units: units.clone(),
                kind,
                queue,
            })
        })
        .collect();
    commands.extend([true, false].map(|raise| Command::Ability {
        units: units.clone(),
        ability: Ability::RiotShield { raise },
    }));
    commands.push(Command::Ability {
        units: units.clone(),
        ability: Ability::SetDropOff(ObjectId(5)),
    });
    commands.push(Command::Ability {
        units: units.clone(),
        ability: Ability::Convert,
    });
    commands.push(Command::Ability {
        units: units.clone(),
        ability: Ability::Buy,
    });
    commands.push(Command::Ability {
        units: units.clone(),
        ability: Ability::Mirage,
    });
    // (M6-SB) The tanks' toggles, each on and off.
    commands.extend([true, false].map(|on| Command::Ability {
        units: units.clone(),
        ability: Ability::Sludge { on },
    }));
    commands.extend([true, false].map(|on| Command::Ability {
        units: units.clone(),
        ability: Ability::Cover { on },
    }));
    commands.extend(
        [
            ConstructionCommand::Construct(ObjectKind::Generator),
            ConstructionCommand::CancelConstruction,
            ConstructionCommand::CancelReady,
            ConstructionCommand::Place {
                origin: crate::units::TilePos::new(5, 2),
                orientation: Default::default(),
            },
            ConstructionCommand::Place {
                origin: crate::units::TilePos::new(1, 6),
                orientation: crate::scenario::BuildingOrientation {
                    rotation: crate::object::Rotation::R270,
                    flip_h: false,
                    flip_v: true,
                },
            },
            ConstructionCommand::WarpIn {
                kind: ObjectKind::RoboPortal,
                orientation: Default::default(),
            },
            ConstructionCommand::WarpIn {
                kind: ObjectKind::BioPortal,
                orientation: crate::scenario::BuildingOrientation {
                    rotation: crate::object::Rotation::R90,
                    flip_h: true,
                    flip_v: false,
                },
            },
            ConstructionCommand::CancelTransform,
        ]
        .map(|command| Command::Construction {
            objects: units.clone(),
            command,
        }),
    );
    commands.extend(
        [
            DebugCommand::Spawn {
                kind: ObjectKind::Peacekeeper,
                owner: Some(PlayerId(1)),
                at: SpawnAt::Point(point),
                orientation: Default::default(),
            },
            DebugCommand::Spawn {
                kind: ObjectKind::SupplyDeliveryStation,
                owner: None,
                at: SpawnAt::Tile(crate::units::TilePos::new(3, 4)),
                orientation: Default::default(),
            },
            DebugCommand::Spawn {
                kind: ObjectKind::Barracks,
                owner: Some(PlayerId(2)),
                at: SpawnAt::Tile(crate::units::TilePos::new(6, 1)),
                orientation: crate::scenario::BuildingOrientation {
                    rotation: crate::object::Rotation::R90,
                    flip_h: true,
                    flip_v: false,
                },
            },
            DebugCommand::Remove { id: ObjectId(2) },
            DebugCommand::SetPosition {
                id: ObjectId(1),
                pos: point,
            },
            DebugCommand::SetAutoTargeting { on: false },
            DebugCommand::SetResources {
                player: PlayerId(2),
                crystals: 1234,
                supplies: 5,
                alloys: 6,
                essence: 7,
                flux: 8,
            },
            DebugCommand::SetTier {
                id: ObjectId(1),
                tier: 2,
            },
            DebugCommand::SetAugmentation {
                id: ObjectId(1),
                augmentation: crate::augmentation::Augmentation {
                    constitution: true,
                    metabolism: false,
                },
            },
            // (M5-X1b) The built-in rules: a game on them goes on as it was, and no generation is installed, which the
            // tests that play these commands mustn't do (`crate::ruleset`: one test program installs others).
            DebugCommand::SetRules(Box::new(Ruleset::builtin())),
        ]
        .map(Command::Debug),
    );
    commands.extend(
        [
            ProductionCommand::Train(ObjectKind::Peacekeeper),
            ProductionCommand::CancelProduction,
            ProductionCommand::SetRally(Target::Point(point)),
            ProductionCommand::SetRally(Target::Object(ObjectId(3))),
            ProductionCommand::TrainFrom {
                kind: ObjectKind::Militant,
                from: crate::augmentation::RecruitKind::Constitution,
            },
            ProductionCommand::Release(crate::augmentation::RecruitKind::Both),
            ProductionCommand::Augment {
                kind: ObjectKind::Gunner,
                from: crate::augmentation::RecruitKind::Metabolism,
                order: crate::augmentation::AugmentOrder::EnhancedConstitution,
            },
            ProductionCommand::ReleaseOf {
                kind: ObjectKind::Disciple,
                from: crate::augmentation::RecruitKind::Plain,
            },
            ProductionCommand::Research(crate::defs::Tech::EnhancedConstitutionL2),
        ]
        .map(|command| Command::Production {
            objects: units.clone(),
            command,
        }),
    );
    commands.push(Command::Tunnel {
        tunnel: ObjectId(6),
        command: TunnelCommand::Eject {
            kind: ObjectKind::Guard,
        },
    });
    commands.extend(
        [
            TunnelCommand::Expand {
                kind: ObjectKind::Hideout,
                origin: crate::units::TilePos::new(5, 6),
                orientation: crate::scenario::BuildingOrientation {
                    rotation: crate::object::Rotation::R180,
                    flip_h: true,
                    flip_v: false,
                },
            },
            TunnelCommand::Upgrade,
            TunnelCommand::Cancel,
        ]
        .map(|command| Command::Tunnel {
            tunnel: ObjectId(6),
            command,
        }),
    );
    commands.extend(
        [
            HoldCommand::EjectAll,
            HoldCommand::ReleaseOne(crate::augmentation::RecruitKind::Metabolism),
        ]
        .map(|command| Command::Holding {
            objects: units.clone(),
            command,
        }),
    );
    commands.extend(AreaAbility::ALL.map(|ability| Command::Cast {
        casters: units.clone(),
        ability,
        at: point,
    }));
    commands.push(Command::Schedule {
        towers: units.clone(),
        station: ObjectId(11),
    });
    for command in &commands {
        match command {
            Command::Order { kind, .. } => match kind {
                OrderKind::Move(Target::Point(_) | Target::Object(_))
                | OrderKind::Attack(_)
                | OrderKind::AttackMove(_)
                | OrderKind::Patrol(_)
                | OrderKind::Hold
                | OrderKind::Stop
                | OrderKind::Gather(_)
                | OrderKind::DropOff(_)
                | OrderKind::Deploy { .. }
                | OrderKind::Enter(_)
                | OrderKind::AttackGround(_)
                | OrderKind::Build { .. }
                | OrderKind::Assist(_)
                | OrderKind::Repair(_)
                | OrderKind::Shroud
                | OrderKind::Reverse(_)
                | OrderKind::TargetLock(_)
                | OrderKind::Launch
                | OrderKind::Rebase(_)
                | OrderKind::ReturnToBase
                | OrderKind::VehicleDeploy { .. }
                | OrderKind::UnloadAll
                | OrderKind::PickUpSupplies(_)
                | OrderKind::DropOffSupplies(_)
                | OrderKind::AttachToTower(_) => {}
            },
            Command::Ability {
                ability:
                    Ability::RiotShield { .. }
                    | Ability::SetDropOff(_)
                    | Ability::Convert
                    | Ability::Buy
                    | Ability::Mirage
                    | Ability::Sludge { .. }
                    | Ability::Cover { .. },
                ..
            } => {}
            Command::Production { command, .. } => match command {
                ProductionCommand::Train(_)
                | ProductionCommand::CancelProduction
                | ProductionCommand::SetRally(Target::Point(_) | Target::Object(_))
                | ProductionCommand::TrainFrom { .. }
                | ProductionCommand::Release(_)
                | ProductionCommand::Augment { .. }
                | ProductionCommand::ReleaseOf { .. }
                | ProductionCommand::Research(_) => {}
            },
            Command::Construction { command, .. } => match command {
                ConstructionCommand::Construct(_)
                | ConstructionCommand::CancelConstruction
                | ConstructionCommand::CancelReady
                | ConstructionCommand::Place { .. }
                | ConstructionCommand::WarpIn { .. }
                | ConstructionCommand::CancelTransform => {}
            },
            Command::Tunnel {
                command:
                    TunnelCommand::Eject { .. }
                    | TunnelCommand::Expand { .. }
                    | TunnelCommand::Upgrade
                    | TunnelCommand::Cancel,
                ..
            } => {}
            Command::Holding {
                command: HoldCommand::EjectAll | HoldCommand::ReleaseOne(_),
                ..
            } => {}
            Command::Cast {
                ability:
                    AreaAbility::HealthRegeneration
                    | AreaAbility::EnergyRegeneration
                    | AreaAbility::Inspire
                    | AreaAbility::Suppress
                    | AreaAbility::Scan,
                ..
            } => {}
            Command::Schedule { .. } => {}
            Command::Debug(
                DebugCommand::Spawn {
                    at: SpawnAt::Point(_) | SpawnAt::Tile(_),
                    ..
                }
                | DebugCommand::Remove { .. }
                | DebugCommand::SetPosition { .. }
                | DebugCommand::SetAutoTargeting { .. }
                | DebugCommand::SetResources { .. }
                | DebugCommand::SetTier { .. }
                | DebugCommand::SetAugmentation { .. }
                | DebugCommand::SetRules(_),
            ) => {}
        }
    }
    commands
}
