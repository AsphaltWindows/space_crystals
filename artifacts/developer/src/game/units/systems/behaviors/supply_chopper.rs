#![allow(dead_code)]
use bevy::prelude::*;
use crate::game::units::types::state::behavior::{
    BaseBehaviorState, PickingUpSuppliesBehavior, PickUpPhase,
    AttachingToTowerBehavior, DroppingOffSuppliesBehavior,
    LocomotionChannel, OrientationChannel,
};
use crate::game::units::types::state::commands::UnitCommand;
use crate::game::types::ObjectInstance;
use crate::game::types::structures::{SupplyChopperState, SupplyTowerState};
use crate::game::types::factions::{Player, GdoPlayerResources};
use crate::game::world::types::SupplyDeliveryStation;
use crate::types::Owner;
use super::movement::OBJECT_PROXIMITY_DISTANCE;

// === Supply Chopper Behavior Systems ===

/// Duration in frames for transferring supplies from SDS to chopper (~0.5 seconds at 60fps)
const CHOPPER_PICKUP_DURATION: u32 = 30;

/// HP restored per frame while chopper is landed/attached at tower
const CHOPPER_REPAIR_RATE: f32 = 1.0;

/// PickUpSupplies behavior system.
///
/// Handles the supply pickup cycle:
/// 1. Move to the target Supply Delivery Station
/// 2. Transfer supplies (30 frames)
/// 3. If attached to a tower, auto-move back to tower. Otherwise idle.
pub fn supply_chopper_pickup_system(
    mut commands: Commands,
    mut choppers: Query<(
        Entity,
        &Transform,
        &mut PickingUpSuppliesBehavior,
        &mut LocomotionChannel,
        &mut OrientationChannel,
        &mut UnitCommand,
        &mut BaseBehaviorState,
        &mut SupplyChopperState,
    )>,
    mut sds_query: Query<&mut SupplyDeliveryStation, Without<PickingUpSuppliesBehavior>>,
    target_transforms: Query<&Transform, (Without<PickingUpSuppliesBehavior>, Without<SupplyTowerState>)>,
    tower_transforms: Query<&Transform, (With<SupplyTowerState>, Without<PickingUpSuppliesBehavior>)>,
) {
    for (entity, transform, mut behavior, mut locomotion, mut orientation, mut command, mut base_behavior, mut chopper_state) in &mut choppers {
        match behavior.phase.clone() {
            PickUpPhase::MovingToSDS => {
                let sds_pos = match target_transforms.get(behavior.target_sds) {
                    Ok(t) => t.translation,
                    Err(_) => {
                        // SDS gone — cancel
                        *command = UnitCommand::Idle;
                        *base_behavior = BaseBehaviorState::None;
                        *locomotion = LocomotionChannel::Stationary;
                        *orientation = OrientationChannel::Maintaining;
                        commands.entity(entity).remove::<PickingUpSuppliesBehavior>();
                        continue;
                    }
                };

                let dist = Vec3::new(
                    transform.translation.x - sds_pos.x, 0.0,
                    transform.translation.z - sds_pos.z,
                ).length();

                if dist < OBJECT_PROXIMITY_DISTANCE {
                    // Arrived — start transferring
                    behavior.phase = PickUpPhase::Transferring { frames_remaining: CHOPPER_PICKUP_DURATION };
                    *locomotion = LocomotionChannel::Stationary;
                    *orientation = OrientationChannel::Maintaining;
                } else {
                    // Move toward SDS
                    *locomotion = LocomotionChannel::Moving(vec![sds_pos]);
                    *orientation = OrientationChannel::Turning(sds_pos);
                }
            }
            PickUpPhase::Transferring { frames_remaining } => {
                if frames_remaining <= 1 {
                    // Transfer complete — take supplies from SDS
                    if let Ok(mut sds) = sds_query.get_mut(behavior.target_sds) {
                        chopper_state.carried_supplies += sds.current_supplies;
                        sds.current_supplies = 0;
                    }

                    // If attached to a tower, auto-return
                    if let Some(tower_entity) = chopper_state.attached_tower {
                        if let Ok(tower_t) = tower_transforms.get(tower_entity) {
                            *command = UnitCommand::DropOffSupplies(tower_entity);
                            *locomotion = LocomotionChannel::Moving(vec![tower_t.translation]);
                            *orientation = OrientationChannel::Turning(tower_t.translation);
                            commands.entity(entity).remove::<PickingUpSuppliesBehavior>();
                            commands.entity(entity).insert(DroppingOffSuppliesBehavior::new(tower_entity));
                            continue;
                        }
                    }

                    // No attached tower — idle
                    *command = UnitCommand::Idle;
                    *base_behavior = BaseBehaviorState::None;
                    *locomotion = LocomotionChannel::Stationary;
                    *orientation = OrientationChannel::Maintaining;
                    commands.entity(entity).remove::<PickingUpSuppliesBehavior>();
                } else {
                    behavior.phase = PickUpPhase::Transferring { frames_remaining: frames_remaining - 1 };
                }
            }
        }
    }
}

/// AttachToTower behavior system.
///
/// Moves chopper to the target Supply Tower and establishes attachment on arrival.
pub fn supply_chopper_attach_system(
    mut commands: Commands,
    mut choppers: Query<(
        Entity,
        &Transform,
        &AttachingToTowerBehavior,
        &mut LocomotionChannel,
        &mut OrientationChannel,
        &mut UnitCommand,
        &mut BaseBehaviorState,
        &mut SupplyChopperState,
    )>,
    mut towers: Query<(&Transform, &mut SupplyTowerState), Without<AttachingToTowerBehavior>>,
) {
    for (entity, transform, behavior, mut locomotion, mut orientation, mut command, mut base_behavior, mut chopper_state) in &mut choppers {
        let tower_data = match towers.get_mut(behavior.target_tower) {
            Ok(data) => data,
            Err(_) => {
                // Tower gone — cancel
                *command = UnitCommand::Idle;
                *base_behavior = BaseBehaviorState::None;
                *locomotion = LocomotionChannel::Stationary;
                *orientation = OrientationChannel::Maintaining;
                commands.entity(entity).remove::<AttachingToTowerBehavior>();
                continue;
            }
        };

        let (tower_transform, mut tower_state) = tower_data;
        let tower_pos = tower_transform.translation;

        let dist = Vec3::new(
            transform.translation.x - tower_pos.x, 0.0,
            transform.translation.z - tower_pos.z,
        ).length();

        if dist < OBJECT_PROXIMITY_DISTANCE {
            // Arrived — establish attachment
            // If previously attached to a different tower, clear old attachment
            if let Some(old_tower) = chopper_state.attached_tower {
                if old_tower != behavior.target_tower {
                    // Need to use other_towers query to avoid aliased access
                    // Since we already have tower_state borrowed for behavior.target_tower,
                    // the old tower access must be done after we drop it. Use commands instead.
                    commands.queue(move |world: &mut World| {
                        if let Ok(mut old_st) = world.get_entity_mut(old_tower) {
                            if let Some(mut old_state) = old_st.get_mut::<SupplyTowerState>() {
                                if old_state.attached_chopper == Some(entity) {
                                    old_state.attached_chopper = None;
                                }
                            }
                        }
                    });
                }
            }

            chopper_state.attached_tower = Some(behavior.target_tower);
            tower_state.attached_chopper = Some(entity);

            *command = UnitCommand::Idle;
            *base_behavior = BaseBehaviorState::None;
            *locomotion = LocomotionChannel::Stationary;
            *orientation = OrientationChannel::Maintaining;
            commands.entity(entity).remove::<AttachingToTowerBehavior>();
        } else {
            // Move toward tower
            *locomotion = LocomotionChannel::Moving(vec![tower_pos]);
            *orientation = OrientationChannel::Turning(tower_pos);
        }
    }
}

/// DropOffSupplies behavior system.
///
/// Moves chopper to the target Supply Tower and delivers carried supplies on arrival.
pub fn supply_chopper_dropoff_system(
    mut commands: Commands,
    mut choppers: Query<(
        Entity,
        &Transform,
        &DroppingOffSuppliesBehavior,
        &mut LocomotionChannel,
        &mut OrientationChannel,
        &mut UnitCommand,
        &mut BaseBehaviorState,
        &mut SupplyChopperState,
        &Owner,
    )>,
    tower_transforms: Query<&Transform, (With<SupplyTowerState>, Without<DroppingOffSuppliesBehavior>)>,
    mut player_resources: Query<(&Player, &mut GdoPlayerResources)>,
) {
    for (entity, transform, behavior, mut locomotion, mut orientation, mut command, mut base_behavior, mut chopper_state, owner) in &mut choppers {
        let tower_pos = match tower_transforms.get(behavior.target_tower) {
            Ok(t) => t.translation,
            Err(_) => {
                // Tower gone — cancel
                *command = UnitCommand::Idle;
                *base_behavior = BaseBehaviorState::None;
                *locomotion = LocomotionChannel::Stationary;
                *orientation = OrientationChannel::Maintaining;
                commands.entity(entity).remove::<DroppingOffSuppliesBehavior>();
                continue;
            }
        };

        let dist = Vec3::new(
            transform.translation.x - tower_pos.x, 0.0,
            transform.translation.z - tower_pos.z,
        ).length();

        if dist < OBJECT_PROXIMITY_DISTANCE {
            // Arrived — transfer supplies to player resources
            if chopper_state.carried_supplies > 0 {
                if let Some(owner_player) = owner.player_number() {
                    for (player, mut resources) in &mut player_resources {
                        if player.player_number == owner_player {
                            resources.supplies += chopper_state.carried_supplies as i32;
                            break;
                        }
                    }
                }
                chopper_state.carried_supplies = 0;
            }

            *command = UnitCommand::Idle;
            *base_behavior = BaseBehaviorState::None;
            *locomotion = LocomotionChannel::Stationary;
            *orientation = OrientationChannel::Maintaining;
            commands.entity(entity).remove::<DroppingOffSuppliesBehavior>();
        } else {
            // Move toward tower
            *locomotion = LocomotionChannel::Moving(vec![tower_pos]);
            *orientation = OrientationChannel::Turning(tower_pos);
        }
    }
}

/// Supply chopper detach system.
///
/// When a chopper with an attached tower receives a non-Idle command,
/// detach it from the tower so other systems see clean state.
pub fn supply_chopper_detach_system(
    mut choppers: Query<(Entity, &UnitCommand, &mut SupplyChopperState)>,
    mut towers: Query<&mut SupplyTowerState>,
) {
    for (entity, command, mut chopper_state) in &mut choppers {
        if chopper_state.attached_tower.is_none() {
            continue;
        }

        // Detach when command is anything other than Idle
        if !matches!(command, UnitCommand::Idle) {
            if let Some(tower_entity) = chopper_state.attached_tower.take() {
                if let Ok(mut tower_state) = towers.get_mut(tower_entity) {
                    if tower_state.attached_chopper == Some(entity) {
                        tower_state.attached_chopper = None;
                    }
                }
            }
        }
    }
}

/// Supply chopper repair system.
///
/// When a chopper is idle, attached to a tower, and near the tower,
/// it slowly repairs over time.
pub fn supply_chopper_repair_system(
    mut choppers: Query<(
        &Transform,
        &SupplyChopperState,
        &UnitCommand,
        &mut ObjectInstance,
    )>,
    tower_transforms: Query<&Transform, (With<SupplyTowerState>, Without<SupplyChopperState>)>,
) {
    for (transform, chopper_state, command, mut obj) in &mut choppers {
        if !matches!(command, UnitCommand::Idle) {
            continue;
        }

        let tower_entity = match chopper_state.attached_tower {
            Some(e) => e,
            None => continue,
        };

        let tower_pos = match tower_transforms.get(tower_entity) {
            Ok(t) => t.translation,
            Err(_) => continue,
        };

        let dist = Vec3::new(
            transform.translation.x - tower_pos.x, 0.0,
            transform.translation.z - tower_pos.z,
        ).length();

        if dist >= OBJECT_PROXIMITY_DISTANCE {
            continue;
        }

        // Repair: increment HP, capped at max_hp
        if let (Some(hp), Some(max_hp)) = (obj.hp, obj.max_hp) {
            if hp < max_hp {
                obj.hp = Some((hp + CHOPPER_REPAIR_RATE).min(max_hp));
            }
        }
    }
}


#[cfg(test)]
mod tests {
    use super::*;
    use bevy::ecs::system::RunSystemOnce;
    use crate::types::{Owner, Unit, ObjectEnum};
    use crate::game::types::ObjectInstance;
    use crate::game::types::structures::{SupplyChopperState, SupplyTowerState};
    use crate::game::types::factions::{Player, GdoPlayerResources};
    use crate::game::world::types::SupplyDeliveryStation;
    
    use crate::game::units::types::state::behavior::*;
    use crate::game::units::types::state::commands::UnitCommand;
    
    use crate::game::units::types::movement::Velocity;
    

    // === Supply Chopper constants tests ===

    #[test]
    fn chopper_pickup_duration_is_30_frames() {
        assert_eq!(CHOPPER_PICKUP_DURATION, 30);
    }

    #[test]
    fn chopper_repair_rate_is_1_hp_per_frame() {
        assert!((CHOPPER_REPAIR_RATE - 1.0).abs() < f32::EPSILON);
    }

    // === Supply Chopper Pickup System Tests ===

    fn spawn_chopper_entity(world: &mut World, pos: Vec3, owner: Owner) -> Entity {
        world.spawn((
            Transform::from_translation(pos),
            Unit,
            ObjectInstance::destructible(ObjectEnum::SupplyChopper, 150.0),
            owner,
            UnitCommand::PickUpSupplies(Entity::PLACEHOLDER),
            BaseBehaviorState::default(),
            LocomotionChannel::default(),
            OrientationChannel::default(),
            SupplyChopperState::default(),
            Velocity(Vec3::ZERO),
        )).id()
    }

    fn spawn_sds_entity(world: &mut World, pos: Vec3) -> Entity {
        world.spawn((
            Transform::from_translation(pos),
            SupplyDeliveryStation {
                delivery_size: 100,
                delivery_interval: 60.0,
                current_supplies: 50,
                time_until_next_delivery: 60.0,
            },
        )).id()
    }

    fn spawn_tower_entity(world: &mut World, pos: Vec3) -> Entity {
        world.spawn((
            Transform::from_translation(pos),
            SupplyTowerState::default(),
        )).id()
    }

    #[test]
    fn pickup_system_moves_toward_sds() {
        let mut app = App::new();
        app.add_plugins(MinimalPlugins);

        let sds = spawn_sds_entity(app.world_mut(), Vec3::new(10.0, 0.0, 10.0));
        let chopper = spawn_chopper_entity(app.world_mut(), Vec3::new(0.0, 1.5, 0.0), Owner::player(0));
        app.world_mut().entity_mut(chopper).insert((
            UnitCommand::PickUpSupplies(sds),
            PickingUpSuppliesBehavior::new(sds),
        ));

        app.world_mut().run_system_once(supply_chopper_pickup_system).unwrap();

        let loco = app.world().get::<LocomotionChannel>(chopper).unwrap();
        assert!(matches!(loco, LocomotionChannel::Moving(_)));
    }

    #[test]
    fn pickup_system_starts_transferring_on_arrival() {
        let mut app = App::new();
        app.add_plugins(MinimalPlugins);

        let sds = spawn_sds_entity(app.world_mut(), Vec3::new(1.0, 0.0, 1.0));
        let chopper = spawn_chopper_entity(app.world_mut(), Vec3::new(1.0, 1.5, 1.0), Owner::player(0));
        app.world_mut().entity_mut(chopper).insert((
            UnitCommand::PickUpSupplies(sds),
            PickingUpSuppliesBehavior::new(sds),
        ));

        app.world_mut().run_system_once(supply_chopper_pickup_system).unwrap();

        let behavior = app.world().get::<PickingUpSuppliesBehavior>(chopper).unwrap();
        assert!(matches!(behavior.phase, PickUpPhase::Transferring { .. }));
    }

    #[test]
    fn pickup_system_transfers_supplies_on_completion() {
        let mut app = App::new();
        app.add_plugins(MinimalPlugins);

        let sds = spawn_sds_entity(app.world_mut(), Vec3::new(1.0, 0.0, 1.0));
        let chopper = spawn_chopper_entity(app.world_mut(), Vec3::new(1.0, 1.5, 1.0), Owner::player(0));
        app.world_mut().entity_mut(chopper).insert((
            UnitCommand::PickUpSupplies(sds),
            PickingUpSuppliesBehavior {
                target_sds: sds,
                phase: PickUpPhase::Transferring { frames_remaining: 1 },
            },
        ));

        app.world_mut().run_system_once(supply_chopper_pickup_system).unwrap();
        app.world_mut().flush();

        // Chopper should have the supplies
        let cs = app.world().get::<SupplyChopperState>(chopper).unwrap();
        assert_eq!(cs.carried_supplies, 50);

        // SDS should be empty
        let sds_state = app.world().get::<SupplyDeliveryStation>(sds).unwrap();
        assert_eq!(sds_state.current_supplies, 0);

        // Behavior marker should be removed
        assert!(app.world().get::<PickingUpSuppliesBehavior>(chopper).is_none());

        // Should be idle (no attached tower)
        let cmd = app.world().get::<UnitCommand>(chopper).unwrap();
        assert!(matches!(cmd, UnitCommand::Idle));
    }

    #[test]
    fn pickup_system_cancels_on_missing_sds() {
        let mut app = App::new();
        app.add_plugins(MinimalPlugins);

        let fake_sds = Entity::from_raw_u32(999).unwrap();
        let chopper = spawn_chopper_entity(app.world_mut(), Vec3::new(0.0, 1.5, 0.0), Owner::player(0));
        app.world_mut().entity_mut(chopper).insert((
            UnitCommand::PickUpSupplies(fake_sds),
            PickingUpSuppliesBehavior::new(fake_sds),
        ));

        app.world_mut().run_system_once(supply_chopper_pickup_system).unwrap();
        app.world_mut().flush();

        let cmd = app.world().get::<UnitCommand>(chopper).unwrap();
        assert!(matches!(cmd, UnitCommand::Idle));
        assert!(app.world().get::<PickingUpSuppliesBehavior>(chopper).is_none());
    }

    #[test]
    fn pickup_system_auto_returns_to_attached_tower() {
        let mut app = App::new();
        app.add_plugins(MinimalPlugins);

        let tower = spawn_tower_entity(app.world_mut(), Vec3::new(5.0, 0.0, 5.0));
        let sds = spawn_sds_entity(app.world_mut(), Vec3::new(1.0, 0.0, 1.0));
        let chopper = spawn_chopper_entity(app.world_mut(), Vec3::new(1.0, 1.5, 1.0), Owner::player(0));
        app.world_mut().entity_mut(chopper).insert((
            UnitCommand::PickUpSupplies(sds),
            PickingUpSuppliesBehavior {
                target_sds: sds,
                phase: PickUpPhase::Transferring { frames_remaining: 1 },
            },
        ));
        app.world_mut().entity_mut(chopper).get_mut::<SupplyChopperState>().unwrap().attached_tower = Some(tower);

        app.world_mut().run_system_once(supply_chopper_pickup_system).unwrap();
        app.world_mut().flush();

        // Should auto-return to tower with DropOffSupplies
        let cmd = app.world().get::<UnitCommand>(chopper).unwrap();
        assert!(matches!(cmd, UnitCommand::DropOffSupplies(_)));

        // Should have DroppingOffSuppliesBehavior inserted
        assert!(app.world().get::<DroppingOffSuppliesBehavior>(chopper).is_some());
    }

    // === Supply Chopper Attach System Tests ===

    #[test]
    fn attach_system_moves_toward_tower() {
        let mut app = App::new();
        app.add_plugins(MinimalPlugins);

        let tower = spawn_tower_entity(app.world_mut(), Vec3::new(10.0, 0.0, 10.0));
        let chopper = spawn_chopper_entity(app.world_mut(), Vec3::new(0.0, 1.5, 0.0), Owner::player(0));
        app.world_mut().entity_mut(chopper).insert((
            UnitCommand::AttachToTower(tower),
            AttachingToTowerBehavior::new(tower),
        ));

        app.world_mut().run_system_once(supply_chopper_attach_system).unwrap();

        let loco = app.world().get::<LocomotionChannel>(chopper).unwrap();
        assert!(matches!(loco, LocomotionChannel::Moving(_)));
    }

    #[test]
    fn attach_system_establishes_attachment_on_arrival() {
        let mut app = App::new();
        app.add_plugins(MinimalPlugins);

        let tower = spawn_tower_entity(app.world_mut(), Vec3::new(1.0, 0.0, 1.0));
        let chopper = spawn_chopper_entity(app.world_mut(), Vec3::new(1.0, 1.5, 1.0), Owner::player(0));
        app.world_mut().entity_mut(chopper).insert((
            UnitCommand::AttachToTower(tower),
            AttachingToTowerBehavior::new(tower),
        ));

        app.world_mut().run_system_once(supply_chopper_attach_system).unwrap();
        app.world_mut().flush();

        // Chopper should be attached to tower
        let cs = app.world().get::<SupplyChopperState>(chopper).unwrap();
        assert_eq!(cs.attached_tower, Some(tower));

        // Tower should reference chopper
        let ts = app.world().get::<SupplyTowerState>(tower).unwrap();
        assert_eq!(ts.attached_chopper, Some(chopper));

        // Behavior marker removed
        assert!(app.world().get::<AttachingToTowerBehavior>(chopper).is_none());

        // Should be idle
        let cmd = app.world().get::<UnitCommand>(chopper).unwrap();
        assert!(matches!(cmd, UnitCommand::Idle));
    }

    #[test]
    fn attach_system_cancels_on_missing_tower() {
        let mut app = App::new();
        app.add_plugins(MinimalPlugins);

        let fake_tower = Entity::from_raw_u32(888).unwrap();
        let chopper = spawn_chopper_entity(app.world_mut(), Vec3::new(0.0, 1.5, 0.0), Owner::player(0));
        app.world_mut().entity_mut(chopper).insert((
            UnitCommand::AttachToTower(fake_tower),
            AttachingToTowerBehavior::new(fake_tower),
        ));

        app.world_mut().run_system_once(supply_chopper_attach_system).unwrap();
        app.world_mut().flush();

        let cmd = app.world().get::<UnitCommand>(chopper).unwrap();
        assert!(matches!(cmd, UnitCommand::Idle));
        assert!(app.world().get::<AttachingToTowerBehavior>(chopper).is_none());
    }

    // === Supply Chopper Dropoff System Tests ===

    fn spawn_gdo_player(world: &mut World, player_number: u8) {
        use crate::game::types::factions::Player;
        use crate::types::FactionEnum;
        world.spawn((
            Player::new("Test", FactionEnum::GlobalDefenseOrdinance, player_number),
            GdoPlayerResources {
                space_crystals: 100,
                supplies: 50,
                power_generated: 10,
                power_consumed: 5,
                unit_control_used: 0,
                unit_control_cap: 200,
                has_power_plant: false,
            },
        ));
    }

    #[test]
    fn dropoff_system_delivers_supplies_on_arrival() {
        let mut app = App::new();
        app.add_plugins(MinimalPlugins);

        let tower = spawn_tower_entity(app.world_mut(), Vec3::new(1.0, 0.0, 1.0));
        let chopper = spawn_chopper_entity(app.world_mut(), Vec3::new(1.0, 1.5, 1.0), Owner::player(0));
        app.world_mut().entity_mut(chopper).insert((
            UnitCommand::DropOffSupplies(tower),
            DroppingOffSuppliesBehavior::new(tower),
        ));
        app.world_mut().entity_mut(chopper).get_mut::<SupplyChopperState>().unwrap().carried_supplies = 25;
        spawn_gdo_player(app.world_mut(), 0);

        app.world_mut().run_system_once(supply_chopper_dropoff_system).unwrap();
        app.world_mut().flush();

        // Carried supplies should be zero
        let cs = app.world().get::<SupplyChopperState>(chopper).unwrap();
        assert_eq!(cs.carried_supplies, 0);

        // Player resources should have increased
        let mut q = app.world_mut().query::<(&Player, &GdoPlayerResources)>();
        for (player, resources) in q.iter(app.world()) {
            if player.player_number == 0 {
                assert_eq!(resources.supplies, 75); // 50 + 25
            }
        }

        // Behavior removed
        assert!(app.world().get::<DroppingOffSuppliesBehavior>(chopper).is_none());

        // Should be idle
        let cmd = app.world().get::<UnitCommand>(chopper).unwrap();
        assert!(matches!(cmd, UnitCommand::Idle));
    }

    #[test]
    fn dropoff_system_moves_toward_tower() {
        let mut app = App::new();
        app.add_plugins(MinimalPlugins);

        let tower = spawn_tower_entity(app.world_mut(), Vec3::new(10.0, 0.0, 10.0));
        let chopper = spawn_chopper_entity(app.world_mut(), Vec3::new(0.0, 1.5, 0.0), Owner::player(0));
        app.world_mut().entity_mut(chopper).insert((
            UnitCommand::DropOffSupplies(tower),
            DroppingOffSuppliesBehavior::new(tower),
        ));

        app.world_mut().run_system_once(supply_chopper_dropoff_system).unwrap();

        let loco = app.world().get::<LocomotionChannel>(chopper).unwrap();
        assert!(matches!(loco, LocomotionChannel::Moving(_)));
    }

    // === Supply Chopper Detach System Tests ===

    #[test]
    fn detach_system_detaches_on_non_idle_command() {
        let mut app = App::new();
        app.add_plugins(MinimalPlugins);

        let tower = spawn_tower_entity(app.world_mut(), Vec3::new(5.0, 0.0, 5.0));
        let chopper = spawn_chopper_entity(app.world_mut(), Vec3::new(0.0, 1.5, 0.0), Owner::player(0));
        app.world_mut().entity_mut(chopper).get_mut::<SupplyChopperState>().unwrap().attached_tower = Some(tower);
        app.world_mut().entity_mut(tower).get_mut::<SupplyTowerState>().unwrap().attached_chopper = Some(chopper);
        app.world_mut().entity_mut(chopper).insert(UnitCommand::Move(Vec3::new(10.0, 0.0, 10.0)));

        app.world_mut().run_system_once(supply_chopper_detach_system).unwrap();

        let cs = app.world().get::<SupplyChopperState>(chopper).unwrap();
        assert!(cs.attached_tower.is_none());

        let ts = app.world().get::<SupplyTowerState>(tower).unwrap();
        assert!(ts.attached_chopper.is_none());
    }

    #[test]
    fn detach_system_keeps_attachment_when_idle() {
        let mut app = App::new();
        app.add_plugins(MinimalPlugins);

        let tower = spawn_tower_entity(app.world_mut(), Vec3::new(5.0, 0.0, 5.0));
        let chopper = spawn_chopper_entity(app.world_mut(), Vec3::new(5.0, 1.5, 5.0), Owner::player(0));
        app.world_mut().entity_mut(chopper).get_mut::<SupplyChopperState>().unwrap().attached_tower = Some(tower);
        app.world_mut().entity_mut(tower).get_mut::<SupplyTowerState>().unwrap().attached_chopper = Some(chopper);
        app.world_mut().entity_mut(chopper).insert(UnitCommand::Idle);

        app.world_mut().run_system_once(supply_chopper_detach_system).unwrap();

        let cs = app.world().get::<SupplyChopperState>(chopper).unwrap();
        assert_eq!(cs.attached_tower, Some(tower));

        let ts = app.world().get::<SupplyTowerState>(tower).unwrap();
        assert_eq!(ts.attached_chopper, Some(chopper));
    }

    // === Supply Chopper Repair System Tests ===

    #[test]
    fn repair_system_heals_damaged_chopper_at_tower() {
        let mut app = App::new();
        app.add_plugins(MinimalPlugins);

        let tower = spawn_tower_entity(app.world_mut(), Vec3::new(1.0, 0.0, 1.0));
        let chopper = spawn_chopper_entity(app.world_mut(), Vec3::new(1.0, 1.5, 1.0), Owner::player(0));
        app.world_mut().entity_mut(chopper).get_mut::<SupplyChopperState>().unwrap().attached_tower = Some(tower);
        app.world_mut().entity_mut(chopper).insert(UnitCommand::Idle);
        // Damage the chopper
        app.world_mut().entity_mut(chopper).get_mut::<ObjectInstance>().unwrap().hp = Some(100.0);

        app.world_mut().run_system_once(supply_chopper_repair_system).unwrap();

        let obj = app.world().get::<ObjectInstance>(chopper).unwrap();
        assert_eq!(obj.hp, Some(101.0)); // +1 HP
    }

    #[test]
    fn repair_system_caps_at_max_hp() {
        let mut app = App::new();
        app.add_plugins(MinimalPlugins);

        let tower = spawn_tower_entity(app.world_mut(), Vec3::new(1.0, 0.0, 1.0));
        let chopper = spawn_chopper_entity(app.world_mut(), Vec3::new(1.0, 1.5, 1.0), Owner::player(0));
        app.world_mut().entity_mut(chopper).get_mut::<SupplyChopperState>().unwrap().attached_tower = Some(tower);
        app.world_mut().entity_mut(chopper).insert(UnitCommand::Idle);
        // HP very close to max
        app.world_mut().entity_mut(chopper).get_mut::<ObjectInstance>().unwrap().hp = Some(149.5);

        app.world_mut().run_system_once(supply_chopper_repair_system).unwrap();

        let obj = app.world().get::<ObjectInstance>(chopper).unwrap();
        assert_eq!(obj.hp, Some(150.0)); // Capped at max
    }

    #[test]
    fn repair_system_does_not_heal_at_full_hp() {
        let mut app = App::new();
        app.add_plugins(MinimalPlugins);

        let tower = spawn_tower_entity(app.world_mut(), Vec3::new(1.0, 0.0, 1.0));
        let chopper = spawn_chopper_entity(app.world_mut(), Vec3::new(1.0, 1.5, 1.0), Owner::player(0));
        app.world_mut().entity_mut(chopper).get_mut::<SupplyChopperState>().unwrap().attached_tower = Some(tower);
        app.world_mut().entity_mut(chopper).insert(UnitCommand::Idle);
        // Already at full HP
        assert_eq!(app.world().get::<ObjectInstance>(chopper).unwrap().hp, Some(150.0));

        app.world_mut().run_system_once(supply_chopper_repair_system).unwrap();

        let obj = app.world().get::<ObjectInstance>(chopper).unwrap();
        assert_eq!(obj.hp, Some(150.0)); // Unchanged
    }

    #[test]
    fn repair_system_does_not_heal_when_not_idle() {
        let mut app = App::new();
        app.add_plugins(MinimalPlugins);

        let tower = spawn_tower_entity(app.world_mut(), Vec3::new(1.0, 0.0, 1.0));
        let chopper = spawn_chopper_entity(app.world_mut(), Vec3::new(1.0, 1.5, 1.0), Owner::player(0));
        app.world_mut().entity_mut(chopper).get_mut::<SupplyChopperState>().unwrap().attached_tower = Some(tower);
        app.world_mut().entity_mut(chopper).insert(UnitCommand::Move(Vec3::new(10.0, 0.0, 10.0)));
        app.world_mut().entity_mut(chopper).get_mut::<ObjectInstance>().unwrap().hp = Some(100.0);

        app.world_mut().run_system_once(supply_chopper_repair_system).unwrap();

        let obj = app.world().get::<ObjectInstance>(chopper).unwrap();
        assert_eq!(obj.hp, Some(100.0)); // Unchanged — not idle
    }

    #[test]
    fn repair_system_does_not_heal_when_far_from_tower() {
        let mut app = App::new();
        app.add_plugins(MinimalPlugins);

        let tower = spawn_tower_entity(app.world_mut(), Vec3::new(20.0, 0.0, 20.0));
        let chopper = spawn_chopper_entity(app.world_mut(), Vec3::new(0.0, 1.5, 0.0), Owner::player(0));
        app.world_mut().entity_mut(chopper).get_mut::<SupplyChopperState>().unwrap().attached_tower = Some(tower);
        app.world_mut().entity_mut(chopper).insert(UnitCommand::Idle);
        app.world_mut().entity_mut(chopper).get_mut::<ObjectInstance>().unwrap().hp = Some(100.0);

        app.world_mut().run_system_once(supply_chopper_repair_system).unwrap();

        let obj = app.world().get::<ObjectInstance>(chopper).unwrap();
        assert_eq!(obj.hp, Some(100.0)); // Unchanged — too far
    }
}
