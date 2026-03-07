#![allow(dead_code)]
use bevy::prelude::*;
use crate::game::units::types::movement::{
    MoveObjectTarget, Velocity,
};
use crate::game::units::types::state::behavior::{
    BaseBehaviorState, BuildingStructureBehavior, EnteringTunnelBehavior,
    LocomotionChannel, OrientationChannel,
};
use crate::game::units::types::state::commands::{UnitCommand, TurretCommandState};
use crate::game::types::{StructureInstance, TunnelState};
use crate::game::world::types::{Tile, TilePreset};
use crate::game::world::utils::{can_worker_place_structure, world_to_grid};
use crate::types::GridPosition;

/// Distance threshold for considering a waypoint reached (in grid units).
/// Must match WAYPOINT_ARRIVAL_THRESHOLD in core.rs to avoid dual-layer conflicts.
const WAYPOINT_REACHED_THRESHOLD: f32 = 0.5;

/// Path deviation threshold (gu) — recompute path if unit drifts further than this
const PATH_DEVIATION_THRESHOLD: f32 = 2.0;

/// Velocity threshold for considering a unit stopped
const STOPPED_VELOCITY_THRESHOLD: f32 = 0.01;

/// Proximity distance for MovingToObject completion (gu)
const OBJECT_PROXIMITY_DISTANCE: f32 = 1.5;

/// Distance threshold for target movement triggering path recomputation (gu)
const TARGET_MOVED_THRESHOLD: f32 = 1.0;

/// MovingToLocation behavior system.
///
/// Handles units with `UnitCommand::Move` — follows waypoints from `BaseBehaviorState`,
/// writes to `LocomotionChannel` and `OrientationChannel`. When path completes,
/// transitions to Stopping, then Idle. Glider units circle instead of stopping.
pub fn moving_to_location_system(
    mut query: Query<(
        &Transform,
        &mut BaseBehaviorState,
        &mut LocomotionChannel,
        &mut OrientationChannel,
        &UnitCommand,
        &Velocity,
    )>,
) {
    for (transform, mut behavior, mut locomotion, mut orientation, command, velocity) in query.iter_mut() {
        // Only process Move commands
        let _target = match command {
            UnitCommand::Move(target) => *target,
            _ => continue,
        };

        // Extract path data from behavior state
        let (planned_path, path_index, is_glider) = match behavior.as_ref() {
            BaseBehaviorState::TurnRate { planned_path, path_index } => {
                (planned_path.clone(), *path_index, false)
            }
            BaseBehaviorState::FixedTurnRadius { planned_path, path_index } => {
                (planned_path.clone(), *path_index, false)
            }
            BaseBehaviorState::SpeedTurnRadius { planned_path, path_index } => {
                (planned_path.clone(), *path_index, false)
            }
            BaseBehaviorState::Drag { planned_path, path_index, .. } => {
                (planned_path.clone(), *path_index, false)
            }
            BaseBehaviorState::Glider { planned_path, path_index, circling, .. } => {
                if *circling {
                    // Already circling — keep moving
                    *locomotion = LocomotionChannel::Moving(planned_path.clone());
                    *orientation = OrientationChannel::Turning(transform.translation + transform.forward() * 5.0);
                    continue;
                }
                (planned_path.clone(), *path_index, true)
            }
            BaseBehaviorState::None => continue,
        };

        if planned_path.is_empty() {
            // No path — go idle
            *locomotion = LocomotionChannel::Stationary;
            *orientation = OrientationChannel::Maintaining;
            continue;
        }

        let pos = transform.translation;
        let mut current_index = path_index;

        // Advance past reached waypoints
        while current_index < planned_path.len() {
            let wp = planned_path[current_index];
            let dist = Vec3::new(pos.x - wp.x, 0.0, pos.z - wp.z).length();
            if dist < WAYPOINT_REACHED_THRESHOLD {
                current_index += 1;
            } else {
                break;
            }
        }

        // Update path index in behavior state
        update_path_index(&mut behavior, current_index);

        if current_index >= planned_path.len() {
            // Path complete
            if is_glider {
                // Glider: enter circling mode
                set_glider_circling(&mut behavior, true);
                *locomotion = LocomotionChannel::Moving(planned_path);
                *orientation = OrientationChannel::Turning(pos + transform.forward() * 5.0);
            } else {
                // Normal unit: start stopping
                *locomotion = LocomotionChannel::Stopping;
                *orientation = OrientationChannel::Maintaining;

                // Check if already stopped
                if velocity.0.length() < STOPPED_VELOCITY_THRESHOLD {
                    *locomotion = LocomotionChannel::Stationary;
                    // Note: setting UnitCommand::Idle and BaseBehaviorState::None
                    // must be done by a separate completion system since UnitCommand
                    // is not &mut here. The Stopping->Stationary transition signals
                    // the unit is done.
                }
            }
        } else {
            // Continue following path
            let remaining: Vec<Vec3> = planned_path[current_index..].to_vec();
            let next_waypoint = planned_path[current_index];

            *locomotion = LocomotionChannel::Moving(remaining);
            *orientation = OrientationChannel::Turning(next_waypoint);
        }
    }
}

/// MovingToObject behavior system.
///
/// Tracks a moving entity target via `MoveObjectTarget`. Recomputes path when
/// target moves beyond threshold. Completes when unit is within proximity distance.
pub fn moving_to_object_system(
    mut query: Query<(
        &Transform,
        &mut BaseBehaviorState,
        &mut LocomotionChannel,
        &mut OrientationChannel,
        &MoveObjectTarget,
        &Velocity,
    )>,
    targets: Query<&Transform, Without<MoveObjectTarget>>,
) {
    for (transform, mut behavior, mut locomotion, mut orientation, obj_target, velocity) in query.iter_mut() {
        // Look up target entity's position
        let target_transform = match targets.get(obj_target.entity) {
            Ok(t) => t,
            Err(_) => {
                // Target no longer exists — stop
                *locomotion = LocomotionChannel::Stopping;
                *orientation = OrientationChannel::Maintaining;
                continue;
            }
        };

        let target_pos = target_transform.translation;
        let pos = transform.translation;
        let dist_to_target = Vec3::new(pos.x - target_pos.x, 0.0, pos.z - target_pos.z).length();

        // Check proximity — are we close enough?
        if dist_to_target < OBJECT_PROXIMITY_DISTANCE {
            *locomotion = LocomotionChannel::Stopping;
            *orientation = OrientationChannel::Maintaining;

            if velocity.0.length() < STOPPED_VELOCITY_THRESHOLD {
                *locomotion = LocomotionChannel::Stationary;
            }
            continue;
        }

        // Extract path data
        let (planned_path, path_index) = match behavior.as_ref() {
            BaseBehaviorState::TurnRate { planned_path, path_index } => {
                (planned_path.clone(), *path_index)
            }
            BaseBehaviorState::FixedTurnRadius { planned_path, path_index } => {
                (planned_path.clone(), *path_index)
            }
            BaseBehaviorState::SpeedTurnRadius { planned_path, path_index } => {
                (planned_path.clone(), *path_index)
            }
            BaseBehaviorState::Drag { planned_path, path_index, .. } => {
                (planned_path.clone(), *path_index)
            }
            BaseBehaviorState::Glider { planned_path, path_index, .. } => {
                (planned_path.clone(), *path_index)
            }
            BaseBehaviorState::None => continue,
        };

        // Check if target has moved enough to warrant path recomputation
        if let Some(last_wp) = planned_path.last() {
            let target_moved = Vec3::new(
                last_wp.x - target_pos.x, 0.0, last_wp.z - target_pos.z,
            ).length();
            if target_moved > TARGET_MOVED_THRESHOLD {
                // Target moved — mark path for recomputation by clearing it
                // (actual pathfinding requires tile queries; this signals the need)
                update_path_index(&mut behavior, 0);
                // For now, orient toward the target directly
                *locomotion = LocomotionChannel::Moving(vec![target_pos]);
                *orientation = OrientationChannel::Turning(target_pos);
                continue;
            }
        }

        if planned_path.is_empty() {
            // No path — move directly toward target
            *locomotion = LocomotionChannel::Moving(vec![target_pos]);
            *orientation = OrientationChannel::Turning(target_pos);
            continue;
        }

        // Follow path (same logic as moving_to_location)
        let mut current_index = path_index;
        while current_index < planned_path.len() {
            let wp = planned_path[current_index];
            let dist = Vec3::new(pos.x - wp.x, 0.0, pos.z - wp.z).length();
            if dist < WAYPOINT_REACHED_THRESHOLD {
                current_index += 1;
            } else {
                break;
            }
        }

        update_path_index(&mut behavior, current_index);

        if current_index >= planned_path.len() {
            // Path exhausted but not in proximity — move directly
            *locomotion = LocomotionChannel::Moving(vec![target_pos]);
            *orientation = OrientationChannel::Turning(target_pos);
        } else {
            let remaining: Vec<Vec3> = planned_path[current_index..].to_vec();
            let next_waypoint = planned_path[current_index];
            *locomotion = LocomotionChannel::Moving(remaining);
            *orientation = OrientationChannel::Turning(next_waypoint);
        }
    }
}

/// ReversingToLocation behavior system.
///
/// Like MovingToLocation but writes `LocomotionChannel::Reversing` instead of `Moving`.
/// Only valid for units whose UnitBase has `can_reverse = true` (enforced at command level).
pub fn reversing_to_location_system(
    mut query: Query<(
        &Transform,
        &mut BaseBehaviorState,
        &mut LocomotionChannel,
        &mut OrientationChannel,
        &UnitCommand,
        &Velocity,
    )>,
) {
    for (transform, mut behavior, mut locomotion, mut orientation, command, velocity) in query.iter_mut() {
        // Only process Reverse commands
        let _target = match command {
            UnitCommand::Reverse(target) => *target,
            _ => continue,
        };

        // Extract path data — reversing only valid for FixedTurnRadius and SpeedTurnRadius
        let (planned_path, path_index) = match behavior.as_ref() {
            BaseBehaviorState::FixedTurnRadius { planned_path, path_index } => {
                (planned_path.clone(), *path_index)
            }
            BaseBehaviorState::SpeedTurnRadius { planned_path, path_index } => {
                (planned_path.clone(), *path_index)
            }
            // Other movement models don't support reversing — skip
            _ => continue,
        };

        if planned_path.is_empty() {
            *locomotion = LocomotionChannel::Stationary;
            *orientation = OrientationChannel::Maintaining;
            continue;
        }

        let pos = transform.translation;
        let mut current_index = path_index;

        // Advance past reached waypoints
        while current_index < planned_path.len() {
            let wp = planned_path[current_index];
            let dist = Vec3::new(pos.x - wp.x, 0.0, pos.z - wp.z).length();
            if dist < WAYPOINT_REACHED_THRESHOLD {
                current_index += 1;
            } else {
                break;
            }
        }

        update_path_index(&mut behavior, current_index);

        if current_index >= planned_path.len() {
            // Path complete — stop
            *locomotion = LocomotionChannel::Stopping;
            *orientation = OrientationChannel::Maintaining;

            if velocity.0.length() < STOPPED_VELOCITY_THRESHOLD {
                *locomotion = LocomotionChannel::Stationary;
            }
        } else {
            // Continue reversing along path
            let remaining: Vec<Vec3> = planned_path[current_index..].to_vec();
            // Orientation maintains current facing while reversing
            *locomotion = LocomotionChannel::Reversing(remaining);
            *orientation = OrientationChannel::Maintaining;
        }
    }
}

/// StoppingBehavior system.
///
/// Handles `UnitCommand::Stop` — sets channels to decelerate, clears turret lock,
/// and completes when velocity reaches ~0.
pub fn stopping_behavior_system(
    mut query: Query<(
        &mut LocomotionChannel,
        &mut OrientationChannel,
        &mut BaseBehaviorState,
        &UnitCommand,
        &Velocity,
        Option<&mut TurretCommandState>,
    )>,
) {
    for (mut locomotion, mut orientation, mut behavior, command, velocity, turret_state) in query.iter_mut() {
        if !matches!(command, UnitCommand::Stop) {
            continue;
        }

        *locomotion = LocomotionChannel::Stopping;
        *orientation = OrientationChannel::Maintaining;

        // Clear turret locked target if present
        if let Some(mut turret) = turret_state {
            turret.locked_target = None;
        }

        // Check if velocity is near zero — behavior complete
        if velocity.0.length() < STOPPED_VELOCITY_THRESHOLD {
            *locomotion = LocomotionChannel::Stationary;
            *behavior = BaseBehaviorState::None;
        }
    }
}

// === Helper functions ===

/// Update the path_index in the current BaseBehaviorState variant.
fn update_path_index(behavior: &mut BaseBehaviorState, new_index: usize) {
    match behavior {
        BaseBehaviorState::TurnRate { path_index, .. } => *path_index = new_index,
        BaseBehaviorState::FixedTurnRadius { path_index, .. } => *path_index = new_index,
        BaseBehaviorState::SpeedTurnRadius { path_index, .. } => *path_index = new_index,
        BaseBehaviorState::Drag { path_index, .. } => *path_index = new_index,
        BaseBehaviorState::Glider { path_index, .. } => *path_index = new_index,
        BaseBehaviorState::None => {}
    }
}

/// Set the circling flag on a Glider behavior state.
fn set_glider_circling(behavior: &mut BaseBehaviorState, circling_val: bool) {
    if let BaseBehaviorState::Glider { circling, .. } = behavior {
        *circling = circling_val;
    }
}

/// Extract planned_path and path_index from any BaseBehaviorState variant.
/// Returns None for BaseBehaviorState::None.
pub fn extract_path_data(behavior: &BaseBehaviorState) -> Option<(&[Vec3], usize)> {
    match behavior {
        BaseBehaviorState::TurnRate { planned_path, path_index } => Some((planned_path, *path_index)),
        BaseBehaviorState::FixedTurnRadius { planned_path, path_index } => Some((planned_path, *path_index)),
        BaseBehaviorState::SpeedTurnRadius { planned_path, path_index } => Some((planned_path, *path_index)),
        BaseBehaviorState::Drag { planned_path, path_index, .. } => Some((planned_path, *path_index)),
        BaseBehaviorState::Glider { planned_path, path_index, .. } => Some((planned_path, *path_index)),
        BaseBehaviorState::None => None,
    }
}

/// Check if a unit has deviated from its expected path position by more than threshold.
pub fn has_path_deviation(pos: Vec3, planned_path: &[Vec3], path_index: usize) -> bool {
    if path_index >= planned_path.len() {
        return false;
    }
    let expected = planned_path[path_index];
    let deviation = Vec3::new(pos.x - expected.x, 0.0, pos.z - expected.z).length();
    deviation > PATH_DEVIATION_THRESHOLD
}

/// Distance threshold for considering arrival at Side A position (in world units)
const TUNNEL_ARRIVAL_THRESHOLD: f32 = 0.5;

/// Process EnteringTunnel behavior.
///
/// Units with the `EnteringTunnelBehavior` marker move toward the target Tunnel's
/// position (Side A approximated as the tunnel's transform position).
/// On arrival, writes to locomotion/orientation channels. When within threshold,
/// the unit entity is despawned (logically enters the tunnel network).
///
/// Schedule: Update (same as other behavior systems)
pub fn entering_tunnel_behavior_system(
    mut commands: Commands,
    mut units: Query<(
        Entity,
        &Transform,
        &mut EnteringTunnelBehavior,
        &mut LocomotionChannel,
        &mut OrientationChannel,
    )>,
    tunnels: Query<&Transform, With<TunnelState>>,
) {
    for (entity, transform, entering, mut locomotion, mut orientation) in units.iter_mut() {
        // Get the tunnel's position (Side A approximated as tunnel center)
        let tunnel_transform = match tunnels.get(entering.target_tunnel) {
            Ok(t) => t,
            Err(_) => {
                // Tunnel no longer exists — cancel behavior, remove marker
                *locomotion = LocomotionChannel::Stopping;
                *orientation = OrientationChannel::Maintaining;
                commands.entity(entity).remove::<EnteringTunnelBehavior>();
                continue;
            }
        };

        let side_a_pos = tunnel_transform.translation;
        let unit_pos = transform.translation;
        let distance = Vec3::new(unit_pos.x - side_a_pos.x, 0.0, unit_pos.z - side_a_pos.z).length();

        if distance < TUNNEL_ARRIVAL_THRESHOLD {
            // Unit has arrived at Side A — enter the tunnel network
            // Despawn the unit entity from the map
            commands.entity(entity).despawn_recursive();
            continue;
        }

        // Write movement channels to move toward Side A
        *locomotion = LocomotionChannel::Moving(vec![side_a_pos]);
        *orientation = OrientationChannel::Turning(side_a_pos);
    }
}

/// Distance threshold for considering arrival at build site (in world units)
const BUILD_ARRIVAL_THRESHOLD: f32 = 1.0;

/// BuildingStructure behavior system.
///
/// Units with the `BuildingStructureBehavior` marker move toward the target build location.
/// On arrival, validates the placement with `can_worker_place_structure()`:
/// - If valid: removes the marker and sets UnitCommand::Idle (structure spawning will be
///   handled by a separate system that detects the arrival+validation).
/// - If invalid: removes the marker and sets UnitCommand::Idle (build cancelled).
///
/// Schedule: Update (same as other behavior systems)
pub fn building_behavior_system(
    mut commands: Commands,
    mut units: Query<(
        Entity,
        &Transform,
        &mut BuildingStructureBehavior,
        &mut LocomotionChannel,
        &mut OrientationChannel,
        &mut UnitCommand,
        &mut BaseBehaviorState,
    )>,
    tiles: Query<(&GridPosition, &TilePreset), With<Tile>>,
    structures: Query<(&GridPosition, &StructureInstance)>,
) {
    for (entity, transform, mut building, mut locomotion, mut orientation, mut command, mut behavior) in units.iter_mut() {
        let unit_pos = transform.translation;
        let target = building.target_location;
        let distance = Vec3::new(unit_pos.x - target.x, 0.0, unit_pos.z - target.z).length();

        if distance < BUILD_ARRIVAL_THRESHOLD && !building.arrived {
            // Mark as arrived
            building.arrived = true;

            // Validate placement
            let obj_type = building.object_to_build.object_type();
            let (size_x, size_z) = obj_type.size;
            let (grid_x, grid_z) = world_to_grid(target, 1.0);

            let valid = can_worker_place_structure(
                grid_x, grid_z, size_x, size_z,
                &tiles, &structures,
            );

            if valid.is_ok() {
                // Validation passed — mark command as Idle.
                // The actual structure spawning requires meshes/materials which are
                // not available in this system. A higher-level system will detect
                // BuildingStructureBehavior with arrived=true and valid placement
                // to spawn the structure. For now, we signal completion.
                *command = UnitCommand::Idle;
                *locomotion = LocomotionChannel::Stationary;
                *orientation = OrientationChannel::Maintaining;
                *behavior = BaseBehaviorState::None;
                // Keep the marker with arrived=true so a spawning system can read it
                // The spawning system will remove the marker after spawning.
                // For this task, we remove it since structure spawning integration
                // is deferred to the agent_object_interface_state task.
                commands.entity(entity).remove::<BuildingStructureBehavior>();
            } else {
                // Validation failed — cancel build, idle
                *command = UnitCommand::Idle;
                *locomotion = LocomotionChannel::Stationary;
                *orientation = OrientationChannel::Maintaining;
                *behavior = BaseBehaviorState::None;
                commands.entity(entity).remove::<BuildingStructureBehavior>();
            }
        } else if !building.arrived {
            // Not yet arrived — move toward target
            *locomotion = LocomotionChannel::Moving(vec![target]);
            *orientation = OrientationChannel::Turning(target);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use bevy::ecs::system::RunSystemOnce;
    use crate::types::ObjectEnum;
    use crate::game::world::types::TilePresetEnum;

    // === Constants tests ===

    #[test]
    fn waypoint_reached_threshold_is_positive() {
        assert!(WAYPOINT_REACHED_THRESHOLD > 0.0);
        assert!(WAYPOINT_REACHED_THRESHOLD < 1.0); // Should be less than 1 grid unit
    }

    #[test]
    fn path_deviation_threshold_is_two_gu() {
        assert!((PATH_DEVIATION_THRESHOLD - 2.0).abs() < f32::EPSILON);
    }

    #[test]
    fn stopped_velocity_threshold_is_small() {
        assert!(STOPPED_VELOCITY_THRESHOLD > 0.0);
        assert!(STOPPED_VELOCITY_THRESHOLD < 0.1);
    }

    #[test]
    fn object_proximity_distance_is_1_5_gu() {
        assert!((OBJECT_PROXIMITY_DISTANCE - 1.5).abs() < f32::EPSILON);
    }

    #[test]
    fn target_moved_threshold_is_1_gu() {
        assert!((TARGET_MOVED_THRESHOLD - 1.0).abs() < f32::EPSILON);
    }

    // === update_path_index tests ===

    #[test]
    fn update_path_index_turn_rate() {
        let mut behavior = BaseBehaviorState::TurnRate {
            planned_path: vec![Vec3::ZERO, Vec3::ONE],
            path_index: 0,
        };
        update_path_index(&mut behavior, 1);
        if let BaseBehaviorState::TurnRate { path_index, .. } = &behavior {
            assert_eq!(*path_index, 1);
        } else {
            panic!("Expected TurnRate");
        }
    }

    #[test]
    fn update_path_index_fixed_turn_radius() {
        let mut behavior = BaseBehaviorState::FixedTurnRadius {
            planned_path: vec![Vec3::ZERO],
            path_index: 0,
        };
        update_path_index(&mut behavior, 5);
        if let BaseBehaviorState::FixedTurnRadius { path_index, .. } = &behavior {
            assert_eq!(*path_index, 5);
        } else {
            panic!("Expected FixedTurnRadius");
        }
    }

    #[test]
    fn update_path_index_speed_turn_radius() {
        let mut behavior = BaseBehaviorState::SpeedTurnRadius {
            planned_path: vec![],
            path_index: 0,
        };
        update_path_index(&mut behavior, 3);
        if let BaseBehaviorState::SpeedTurnRadius { path_index, .. } = &behavior {
            assert_eq!(*path_index, 3);
        } else {
            panic!("Expected SpeedTurnRadius");
        }
    }

    #[test]
    fn update_path_index_drag() {
        let mut behavior = BaseBehaviorState::Drag {
            planned_path: vec![],
            path_index: 0,
            drift_velocity: Vec3::ZERO,
        };
        update_path_index(&mut behavior, 2);
        if let BaseBehaviorState::Drag { path_index, .. } = &behavior {
            assert_eq!(*path_index, 2);
        } else {
            panic!("Expected Drag");
        }
    }

    #[test]
    fn update_path_index_glider() {
        let mut behavior = BaseBehaviorState::Glider {
            planned_path: vec![],
            path_index: 0,
            circling: false,
            strafe_target: None,
        };
        update_path_index(&mut behavior, 4);
        if let BaseBehaviorState::Glider { path_index, .. } = &behavior {
            assert_eq!(*path_index, 4);
        } else {
            panic!("Expected Glider");
        }
    }

    #[test]
    fn update_path_index_none_is_noop() {
        let mut behavior = BaseBehaviorState::None;
        update_path_index(&mut behavior, 10);
        assert!(matches!(behavior, BaseBehaviorState::None));
    }

    // === set_glider_circling tests ===

    #[test]
    fn set_glider_circling_enables() {
        let mut behavior = BaseBehaviorState::Glider {
            planned_path: vec![Vec3::ZERO],
            path_index: 0,
            circling: false,
            strafe_target: None,
        };
        set_glider_circling(&mut behavior, true);
        if let BaseBehaviorState::Glider { circling, .. } = &behavior {
            assert!(*circling);
        } else {
            panic!("Expected Glider");
        }
    }

    #[test]
    fn set_glider_circling_disables() {
        let mut behavior = BaseBehaviorState::Glider {
            planned_path: vec![],
            path_index: 0,
            circling: true,
            strafe_target: None,
        };
        set_glider_circling(&mut behavior, false);
        if let BaseBehaviorState::Glider { circling, .. } = &behavior {
            assert!(!*circling);
        } else {
            panic!("Expected Glider");
        }
    }

    #[test]
    fn set_glider_circling_noop_on_non_glider() {
        let mut behavior = BaseBehaviorState::TurnRate {
            planned_path: vec![],
            path_index: 0,
        };
        set_glider_circling(&mut behavior, true); // Should not panic
        assert!(matches!(behavior, BaseBehaviorState::TurnRate { .. }));
    }

    // === extract_path_data tests ===

    #[test]
    fn extract_path_data_from_turn_rate() {
        let path = vec![Vec3::new(1.0, 0.0, 2.0), Vec3::new(3.0, 0.0, 4.0)];
        let behavior = BaseBehaviorState::TurnRate {
            planned_path: path.clone(),
            path_index: 1,
        };
        let (data, idx) = extract_path_data(&behavior).unwrap();
        assert_eq!(data.len(), 2);
        assert_eq!(idx, 1);
        assert_eq!(data[0], path[0]);
    }

    #[test]
    fn extract_path_data_from_none_returns_none() {
        assert!(extract_path_data(&BaseBehaviorState::None).is_none());
    }

    #[test]
    fn extract_path_data_from_glider() {
        let behavior = BaseBehaviorState::Glider {
            planned_path: vec![Vec3::ZERO],
            path_index: 0,
            circling: true,
            strafe_target: Some(Vec3::ONE),
        };
        let (data, idx) = extract_path_data(&behavior).unwrap();
        assert_eq!(data.len(), 1);
        assert_eq!(idx, 0);
    }

    // === has_path_deviation tests ===

    #[test]
    fn has_path_deviation_within_threshold() {
        let path = vec![Vec3::new(5.0, 0.0, 5.0)];
        // Position only 1.0 gu away — below 2.0 threshold
        let pos = Vec3::new(4.0, 0.0, 5.0);
        assert!(!has_path_deviation(pos, &path, 0));
    }

    #[test]
    fn has_path_deviation_beyond_threshold() {
        let path = vec![Vec3::new(5.0, 0.0, 5.0)];
        // Position 3.0 gu away — above 2.0 threshold
        let pos = Vec3::new(2.0, 0.0, 5.0);
        assert!(has_path_deviation(pos, &path, 0));
    }

    #[test]
    fn has_path_deviation_at_exact_threshold() {
        let path = vec![Vec3::new(5.0, 0.0, 5.0)];
        // Position exactly 2.0 gu away — NOT above threshold (must be >)
        let pos = Vec3::new(3.0, 0.0, 5.0);
        assert!(!has_path_deviation(pos, &path, 0));
    }

    #[test]
    fn has_path_deviation_index_past_end() {
        let path = vec![Vec3::new(5.0, 0.0, 5.0)];
        // Index past end — no deviation
        assert!(!has_path_deviation(Vec3::ZERO, &path, 1));
    }

    #[test]
    fn has_path_deviation_empty_path() {
        let path: Vec<Vec3> = vec![];
        assert!(!has_path_deviation(Vec3::ZERO, &path, 0));
    }

    #[test]
    fn has_path_deviation_ignores_y_component() {
        let path = vec![Vec3::new(5.0, 10.0, 5.0)];
        // Position differs in Y only — should NOT count as deviation
        let pos = Vec3::new(5.0, 0.0, 5.0);
        assert!(!has_path_deviation(pos, &path, 0));
    }

    // === Reversing constraint tests ===

    #[test]
    fn reversing_only_valid_for_can_reverse_models() {
        // FixedTurnRadius and SpeedTurnRadius support reversing
        let ftr = BaseBehaviorState::FixedTurnRadius {
            planned_path: vec![Vec3::ZERO],
            path_index: 0,
        };
        let str_ = BaseBehaviorState::SpeedTurnRadius {
            planned_path: vec![Vec3::ZERO],
            path_index: 0,
        };

        // These should have extractable paths (system will process them)
        assert!(extract_path_data(&ftr).is_some());
        assert!(extract_path_data(&str_).is_some());

        // TurnRate, Drag, Glider do NOT support reversing
        // (reversing_to_location_system skips these variants)
        let tr = BaseBehaviorState::TurnRate {
            planned_path: vec![Vec3::ZERO],
            path_index: 0,
        };
        // TurnRate has path data but the reversing system ignores it via match
        assert!(extract_path_data(&tr).is_some());
    }

    // === MoveObjectTarget tests ===

    #[test]
    fn move_object_target_stores_entity() {
        let target = MoveObjectTarget { entity: Entity::from_raw(42) };
        assert_eq!(target.entity, Entity::from_raw(42));
    }

    #[test]
    fn move_object_target_last_known_pos_defaults_none() {
        let target = MoveObjectTarget {
            entity: Entity::from_raw(1),
        };
        // entity is stored
        assert_eq!(target.entity, Entity::from_raw(1));
    }

    // === BuildingStructureBehavior system tests ===

    #[test]
    fn build_arrival_threshold_is_positive() {
        assert!(BUILD_ARRIVAL_THRESHOLD > 0.0);
    }

    #[test]
    fn build_arrival_threshold_is_one_gu() {
        assert!((BUILD_ARRIVAL_THRESHOLD - 1.0).abs() < f32::EPSILON);
    }

    #[test]
    fn building_behavior_not_arrived_sets_moving() {
        // When far from target, system should set Moving locomotion
        let mut world = World::new();
        let target = Vec3::new(10.0, 0.0, 10.0);
        let entity = world.spawn((
            Transform::from_xyz(0.0, 0.0, 0.0), // Far from target
            BuildingStructureBehavior::new(target, ObjectEnum::Tunnel),
            LocomotionChannel::Stationary,
            OrientationChannel::Maintaining,
            UnitCommand::Build { target, object: ObjectEnum::Tunnel },
            BaseBehaviorState::None,
        )).id();

        world.run_system_once(building_behavior_system);

        let locomotion = world.entity(entity).get::<LocomotionChannel>().unwrap();
        assert!(matches!(locomotion, LocomotionChannel::Moving(_)));
        let orientation = world.entity(entity).get::<OrientationChannel>().unwrap();
        assert!(matches!(orientation, OrientationChannel::Turning(_)));
        // Should still have the marker
        assert!(world.entity(entity).get::<BuildingStructureBehavior>().is_some());
    }

    #[test]
    fn building_behavior_arrived_no_tiles_cancels() {
        // At target but no tiles — validation fails → idle
        let mut world = World::new();
        let target = Vec3::new(0.5, 0.0, 0.5); // At grid ~(32,32)
        let entity = world.spawn((
            Transform::from_xyz(0.5, 0.0, 0.5), // At target
            BuildingStructureBehavior::new(target, ObjectEnum::Tunnel),
            LocomotionChannel::Stationary,
            OrientationChannel::Maintaining,
            UnitCommand::Build { target, object: ObjectEnum::Tunnel },
            BaseBehaviorState::None,
        )).id();

        world.run_system_once(building_behavior_system);

        let command = world.entity(entity).get::<UnitCommand>().unwrap();
        assert!(matches!(command, UnitCommand::Idle));
        // Marker should be removed
        assert!(world.entity(entity).get::<BuildingStructureBehavior>().is_none());
    }

    #[test]
    fn building_behavior_arrived_valid_tiles_completes() {
        // At target with valid tiles — validation passes → idle (structure spawning is separate)
        let mut world = World::new();
        let target = Vec3::new(0.5, 0.0, 0.5); // Maps to grid (32, 32)
        let entity = world.spawn((
            Transform::from_xyz(0.5, 0.0, 0.5),
            BuildingStructureBehavior::new(target, ObjectEnum::Tunnel),
            LocomotionChannel::Stationary,
            OrientationChannel::Maintaining,
            UnitCommand::Build { target, object: ObjectEnum::Tunnel },
            BaseBehaviorState::None,
        )).id();

        // Spawn buildable tiles for 4x4 Tunnel at grid (32, 32)
        for dx in 0..4 {
            for dz in 0..4 {
                world.spawn((
                    GridPosition { x: 32 + dx, z: 32 + dz },
                    TilePreset {
                        value: crate::game::world::types::TilePresetEnum::Plane,
                        name: "Plane".to_string(),
                        texture: None,
                        buildable: true,
                        traversible: true,
                        rugged: false,
                        drillable: false,
                        recruitable: false,
                    },
                    Tile,
                ));
            }
        }

        world.run_system_once(building_behavior_system);

        let command = world.entity(entity).get::<UnitCommand>().unwrap();
        assert!(matches!(command, UnitCommand::Idle));
        let locomotion = world.entity(entity).get::<LocomotionChannel>().unwrap();
        assert!(matches!(locomotion, LocomotionChannel::Stationary));
        assert!(world.entity(entity).get::<BuildingStructureBehavior>().is_none());
    }

    #[test]
    fn building_behavior_arrived_structure_overlap_cancels() {
        // At target with valid tiles but structure overlap → cancels
        let mut world = World::new();
        let target = Vec3::new(0.5, 0.0, 0.5); // Maps to grid (32, 32)
        let entity = world.spawn((
            Transform::from_xyz(0.5, 0.0, 0.5),
            BuildingStructureBehavior::new(target, ObjectEnum::Tunnel),
            LocomotionChannel::Stationary,
            OrientationChannel::Maintaining,
            UnitCommand::Build { target, object: ObjectEnum::Tunnel },
            BaseBehaviorState::None,
        )).id();

        // Spawn buildable tiles
        for dx in 0..4 {
            for dz in 0..4 {
                world.spawn((
                    GridPosition { x: 32 + dx, z: 32 + dz },
                    TilePreset {
                        value: crate::game::world::types::TilePresetEnum::Plane,
                        name: "Plane".to_string(),
                        texture: None,
                        buildable: true,
                        traversible: true,
                        rugged: false,
                        drillable: false,
                        recruitable: false,
                    },
                    Tile,
                ));
            }
        }

        // Spawn an existing structure overlapping the footprint
        world.spawn((
            GridPosition { x: 33, z: 33 },
            StructureInstance::default(),
        ));

        world.run_system_once(building_behavior_system);

        let command = world.entity(entity).get::<UnitCommand>().unwrap();
        assert!(matches!(command, UnitCommand::Idle));
        assert!(world.entity(entity).get::<BuildingStructureBehavior>().is_none());
    }

    #[test]
    fn building_behavior_already_arrived_no_double_process() {
        // If arrived is already true, system should not re-process
        let mut world = World::new();
        let target = Vec3::new(0.5, 0.0, 0.5);
        let mut behavior = BuildingStructureBehavior::new(target, ObjectEnum::Tunnel);
        behavior.arrived = true; // Already processed

        let entity = world.spawn((
            Transform::from_xyz(0.5, 0.0, 0.5),
            behavior,
            LocomotionChannel::Stationary,
            OrientationChannel::Maintaining,
            UnitCommand::Idle,
            BaseBehaviorState::None,
        )).id();

        world.run_system_once(building_behavior_system);

        // Should remain idle, marker should still be there (no re-processing)
        let command = world.entity(entity).get::<UnitCommand>().unwrap();
        assert!(matches!(command, UnitCommand::Idle));
    }
}
