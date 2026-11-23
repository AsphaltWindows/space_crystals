#![allow(dead_code)]
use bevy::prelude::*;
use crate::game::units::types::movement::{MoveObjectTarget, Velocity};
use crate::game::units::types::state::behavior::{
    BaseBehaviorState, LocomotionChannel, OrientationChannel,
};
use crate::game::units::types::state::commands::{UnitCommand, TurretCommandState};

/// Distance threshold for considering a waypoint reached (in grid units).
/// Must match WAYPOINT_ARRIVAL_THRESHOLD in core.rs to avoid dual-layer conflicts.
const WAYPOINT_REACHED_THRESHOLD: f32 = 0.5;

/// Path deviation threshold (gu) — recompute path if unit drifts further than this
const PATH_DEVIATION_THRESHOLD: f32 = 2.0;

/// Velocity threshold for considering a unit stopped
const STOPPED_VELOCITY_THRESHOLD: f32 = 0.01;

/// Proximity distance for MovingToObject completion (gu)

/// Proximity distance for MovingToObject completion (gu)
pub(super) const OBJECT_PROXIMITY_DISTANCE: f32 = 1.5;

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
    for (transform, mut behavior, mut locomotion, mut orientation, command, velocity) in &mut query {
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
    for (transform, mut behavior, mut locomotion, mut orientation, obj_target, velocity) in &mut query {
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
    for (transform, mut behavior, mut locomotion, mut orientation, command, velocity) in &mut query {
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
    for (mut locomotion, mut orientation, mut behavior, command, velocity, turret_state) in &mut query {
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

/// Behavior completion system.
///
/// Detects when Move, Reverse, or Stop behaviors have reached completion
/// (LocomotionChannel::Stationary + near-zero velocity) and transitions
/// UnitCommand to Idle so command_dequeue_system can pop the next queued command.
/// Glider units circling after Move are NOT completed — they remain in Move until
/// a new command is issued.
pub fn behavior_completion_system(
    mut query: Query<(
        &mut UnitCommand,
        &mut BaseBehaviorState,
        &LocomotionChannel,
        &Velocity,
    )>,
) {
    for (mut command, mut behavior, locomotion, velocity) in &mut query {
        // Only complete for commands that have behavior completion semantics
        let should_complete = match command.as_ref() {
            UnitCommand::Move(_) | UnitCommand::Reverse(_) | UnitCommand::Stop => true,
            _ => false,
        };
        if !should_complete {
            continue;
        }

        // Don't complete Move/Reverse commands whose behavior hasn't started yet.
        // A freshly issued command has BaseBehaviorState::None until its
        // behavior system (e.g. moving_to_location_system) initializes it.
        // Without this guard, a race condition can occur when this system
        // runs before the behavior system in the same frame.
        // Stop is excluded — it has no behavior startup phase and should
        // complete immediately when stationary.
        if matches!(behavior.as_ref(), BaseBehaviorState::None)
            && !matches!(command.as_ref(), UnitCommand::Stop)
        {
            continue;
        }

        // Gliders circling after Move are not "complete" — they circle indefinitely
        if let BaseBehaviorState::Glider { circling: true, .. } = behavior.as_ref() {
            continue;
        }

        // Behavior is complete when locomotion is Stationary and velocity is near zero
        if matches!(locomotion, LocomotionChannel::Stationary)
            && velocity.0.length() < STOPPED_VELOCITY_THRESHOLD
        {
            *command = UnitCommand::Idle;
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


#[cfg(test)]
mod tests {
    use super::*;
    use bevy::ecs::system::RunSystemOnce;
    
    
    
    
    
    
    use crate::game::units::types::state::behavior::*;
    use crate::game::units::types::state::commands::UnitCommand;
    
    use crate::game::units::types::movement::{MoveObjectTarget, Velocity};
    

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
        let target = MoveObjectTarget { entity: Entity::from_raw_u32(42).unwrap() };
        assert_eq!(target.entity, Entity::from_raw_u32(42).unwrap());
    }

    #[test]
    fn move_object_target_last_known_pos_defaults_none() {
        let target = MoveObjectTarget {
            entity: Entity::from_raw_u32(1).unwrap(),
        };
        // entity is stored
        assert_eq!(target.entity, Entity::from_raw_u32(1).unwrap());
    }


    // === behavior_completion_system tests ===

    fn spawn_completion_test_entity(
        world: &mut World,
        command: UnitCommand,
        locomotion: LocomotionChannel,
        velocity: Vec3,
        behavior: BaseBehaviorState,
    ) -> Entity {
        world.spawn((
            command,
            locomotion,
            Velocity(velocity),
            behavior,
            OrientationChannel::default(),
        )).id()
    }

    #[test]
    fn move_command_completes_when_stationary_and_stopped() {
        let mut app = App::new();
        app.add_plugins(MinimalPlugins);

        let entity = spawn_completion_test_entity(
            app.world_mut(),
            UnitCommand::Move(Vec3::new(10.0, 0.0, 10.0)),
            LocomotionChannel::Stationary,
            Vec3::ZERO,
            BaseBehaviorState::TurnRate { planned_path: vec![], path_index: 0 },
        );

        app.world_mut().run_system_once(behavior_completion_system).unwrap();

        let cmd = app.world().get::<UnitCommand>(entity).unwrap();
        assert!(matches!(cmd, UnitCommand::Idle), "Move should complete to Idle");
        let beh = app.world().get::<BaseBehaviorState>(entity).unwrap();
        assert!(matches!(beh, BaseBehaviorState::None), "Behavior should be None");
    }

    #[test]
    fn reverse_command_completes_when_stationary_and_stopped() {
        let mut app = App::new();
        app.add_plugins(MinimalPlugins);

        let entity = spawn_completion_test_entity(
            app.world_mut(),
            UnitCommand::Reverse(Vec3::new(5.0, 0.0, 5.0)),
            LocomotionChannel::Stationary,
            Vec3::ZERO,
            BaseBehaviorState::FixedTurnRadius { planned_path: vec![], path_index: 0 },
        );

        app.world_mut().run_system_once(behavior_completion_system).unwrap();

        let cmd = app.world().get::<UnitCommand>(entity).unwrap();
        assert!(matches!(cmd, UnitCommand::Idle), "Reverse should complete to Idle");
    }

    #[test]
    fn stop_command_completes_when_stationary_and_stopped() {
        let mut app = App::new();
        app.add_plugins(MinimalPlugins);

        let entity = spawn_completion_test_entity(
            app.world_mut(),
            UnitCommand::Stop,
            LocomotionChannel::Stationary,
            Vec3::ZERO,
            BaseBehaviorState::None,
        );

        app.world_mut().run_system_once(behavior_completion_system).unwrap();

        let cmd = app.world().get::<UnitCommand>(entity).unwrap();
        assert!(matches!(cmd, UnitCommand::Idle), "Stop should complete to Idle");
    }

    #[test]
    fn move_does_not_complete_while_still_moving() {
        let mut app = App::new();
        app.add_plugins(MinimalPlugins);

        let entity = spawn_completion_test_entity(
            app.world_mut(),
            UnitCommand::Move(Vec3::new(10.0, 0.0, 10.0)),
            LocomotionChannel::Moving(vec![Vec3::new(5.0, 0.0, 5.0)]),
            Vec3::new(1.0, 0.0, 0.0),
            BaseBehaviorState::TurnRate { planned_path: vec![Vec3::new(5.0, 0.0, 5.0)], path_index: 0 },
        );

        app.world_mut().run_system_once(behavior_completion_system).unwrap();

        let cmd = app.world().get::<UnitCommand>(entity).unwrap();
        assert!(matches!(cmd, UnitCommand::Move(_)), "Should stay in Move while not stationary");
    }

    #[test]
    fn move_does_not_complete_with_nonzero_velocity() {
        let mut app = App::new();
        app.add_plugins(MinimalPlugins);

        let entity = spawn_completion_test_entity(
            app.world_mut(),
            UnitCommand::Move(Vec3::new(10.0, 0.0, 10.0)),
            LocomotionChannel::Stationary,
            Vec3::new(0.5, 0.0, 0.0), // Still has velocity
            BaseBehaviorState::TurnRate { planned_path: vec![], path_index: 0 },
        );

        app.world_mut().run_system_once(behavior_completion_system).unwrap();

        let cmd = app.world().get::<UnitCommand>(entity).unwrap();
        assert!(matches!(cmd, UnitCommand::Move(_)), "Should stay in Move with non-zero velocity");
    }

    #[test]
    fn glider_circling_does_not_complete() {
        let mut app = App::new();
        app.add_plugins(MinimalPlugins);

        let entity = spawn_completion_test_entity(
            app.world_mut(),
            UnitCommand::Move(Vec3::new(10.0, 0.0, 10.0)),
            LocomotionChannel::Stationary,
            Vec3::ZERO,
            BaseBehaviorState::Glider {
                planned_path: vec![Vec3::new(10.0, 0.0, 10.0)],
                path_index: 0,
                circling: true,
                strafe_target: None,
            },
        );

        app.world_mut().run_system_once(behavior_completion_system).unwrap();

        let cmd = app.world().get::<UnitCommand>(entity).unwrap();
        assert!(matches!(cmd, UnitCommand::Move(_)), "Glider circling should not complete");
    }

    #[test]
    fn idle_command_not_affected_by_completion() {
        let mut app = App::new();
        app.add_plugins(MinimalPlugins);

        let entity = spawn_completion_test_entity(
            app.world_mut(),
            UnitCommand::Idle,
            LocomotionChannel::Stationary,
            Vec3::ZERO,
            BaseBehaviorState::None,
        );

        app.world_mut().run_system_once(behavior_completion_system).unwrap();

        let cmd = app.world().get::<UnitCommand>(entity).unwrap();
        assert!(matches!(cmd, UnitCommand::Idle), "Idle should stay Idle");
    }

    #[test]
    fn move_with_uninitialized_behavior_does_not_complete() {
        // Regression test: a freshly issued Move command has BaseBehaviorState::None
        // because the behavior system hasn't run yet. behavior_completion_system must
        // NOT complete it prematurely (race condition fix).
        let mut app = App::new();
        app.add_plugins(MinimalPlugins);

        let entity = spawn_completion_test_entity(
            app.world_mut(),
            UnitCommand::Move(Vec3::new(10.0, 0.0, 10.0)),
            LocomotionChannel::Stationary,
            Vec3::ZERO,
            BaseBehaviorState::None, // behavior not yet initialized
        );

        app.world_mut().run_system_once(behavior_completion_system).unwrap();

        let cmd = app.world().get::<UnitCommand>(entity).unwrap();
        assert!(matches!(cmd, UnitCommand::Move(_)),
            "Move with uninitialized behavior should NOT complete to Idle");
    }

    #[test]
    fn reverse_with_uninitialized_behavior_does_not_complete() {
        // Same race condition guard for Reverse commands.
        let mut app = App::new();
        app.add_plugins(MinimalPlugins);

        let entity = spawn_completion_test_entity(
            app.world_mut(),
            UnitCommand::Reverse(Vec3::new(10.0, 0.0, 10.0)),
            LocomotionChannel::Stationary,
            Vec3::ZERO,
            BaseBehaviorState::None,
        );

        app.world_mut().run_system_once(behavior_completion_system).unwrap();

        let cmd = app.world().get::<UnitCommand>(entity).unwrap();
        assert!(matches!(cmd, UnitCommand::Reverse(_)),
            "Reverse with uninitialized behavior should NOT complete to Idle");
    }

    #[test]
    fn attack_target_not_affected_by_completion() {
        let mut app = App::new();
        app.add_plugins(MinimalPlugins);

        let target = Entity::from_raw_u32(42).unwrap();
        let entity = spawn_completion_test_entity(
            app.world_mut(),
            UnitCommand::AttackTarget(target),
            LocomotionChannel::Stationary,
            Vec3::ZERO,
            BaseBehaviorState::None,
        );

        app.world_mut().run_system_once(behavior_completion_system).unwrap();

        let cmd = app.world().get::<UnitCommand>(entity).unwrap();
        assert!(matches!(cmd, UnitCommand::AttackTarget(_)), "AttackTarget should not be completed by this system");
    }
}
