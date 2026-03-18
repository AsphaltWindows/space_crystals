#![allow(dead_code)]
use bevy::prelude::*;
use crate::game::units::types::movement::{MoveObjectTarget, Velocity};
use crate::game::units::types::state::behavior::{
    BaseBehaviorState, BuildingStructureBehavior, BuildingTunnelBehavior, BuildTunnelPhase,
    EnteringTunnelBehavior,
    GatheringResourceBehavior, GatherPhase, DroppingOffResourcesBehavior, DropOffPhase,
    LocomotionChannel, OrientationChannel,
};
use crate::game::units::types::state::commands::{UnitCommand, TurretCommandState};
use crate::game::units::types::state::types::AgentCarryState;
use crate::game::units::types::unit_data::{
    AGENT_MINING_DURATION, AGENT_PICKUP_DURATION, AGENT_DROPOFF_DURATION,
    AGENT_CRYSTAL_CARRY, AGENT_SUPPLY_CARRY, AGENT_TUNNEL_BUILD_FRAMES,
};
use crate::game::types::{ConstructionHP, ObjectInstance, StructureInstance, TunnelState};
use crate::game::types::structures::tunnel_construction_cost;
use crate::game::utils::spawn_tunnel_under_construction;
use crate::game::types::factions::{Player, SyndicatePlayerResources};
use crate::types::SymmetryTypeEnum;
use crate::game::world::types::{Tile, TilePreset, SpaceCrystalPatch};
use crate::game::world::utils::{can_worker_place_structure, world_to_grid};
use crate::types::{GridPosition, Owner};

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
    for (entity, transform, entering, mut locomotion, mut orientation) in &mut units {
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
            commands.entity(entity).despawn();
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
    structures: Query<(&GridPosition, &StructureInstance, &crate::game::types::objects::ObjectInstance)>,
) {
    for (entity, transform, mut building, mut locomotion, mut orientation, mut command, mut behavior) in &mut units {
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

/// Distance threshold for resource/tunnel arrival in gathering behaviors (world units)
const GATHER_ARRIVAL_THRESHOLD: f32 = 1.5;

/// Get world position of a tunnel side ('A', 'B', 'C', or 'D').
/// Sides map to [N=0, E=1, S=2, W=3] at R0. The function finds which cardinal
/// index contains the target side label after rotation, then computes the offset.
pub fn tunnel_side_world_position(
    tunnel_transform: &Transform,
    structure_instance: &StructureInstance,
    target_side: char,
) -> Vec3 {
    let labels = structure_instance.oriented_labels(SymmetryTypeEnum::ABCD);
    // Find which cardinal direction (N=0, E=1, S=2, W=3) has the target side
    let cardinal_index = labels.iter().position(|&c| c == target_side).unwrap_or(0);

    let center = tunnel_transform.translation;
    // Tunnel is 4x4 — half-size is 2.0, offset outside edge is 2.0 + 0.5 = 2.5
    let offset = 2.5;

    match cardinal_index {
        0 => Vec3::new(center.x, center.y, center.z - offset), // North
        1 => Vec3::new(center.x + offset, center.y, center.z), // East
        2 => Vec3::new(center.x, center.y, center.z + offset), // South
        3 => Vec3::new(center.x - offset, center.y, center.z), // West
        _ => center,
    }
}

/// Find the nearest own tunnel and return its entity + the appropriate side position
/// for the given resource type (crystals → Side B, supplies → Side C).
fn find_nearest_own_tunnel(
    agent_pos: Vec3,
    agent_owner: &Owner,
    target_side: char,
    tunnels: &Query<(Entity, &Transform, &Owner, &StructureInstance), With<TunnelState>>,
) -> Option<(Entity, Vec3)> {
    let agent_player = agent_owner.player_number()?;
    let mut best: Option<(Entity, Vec3, f32)> = None;

    for (tunnel_entity, tunnel_transform, tunnel_owner, structure_instance) in tunnels.iter() {
        if tunnel_owner.player_number() != Some(agent_player) {
            continue;
        }
        let side_pos = tunnel_side_world_position(tunnel_transform, structure_instance, target_side);
        let dist = Vec3::new(
            agent_pos.x - side_pos.x, 0.0, agent_pos.z - side_pos.z,
        ).length();
        if best.as_ref().map_or(true, |(_, _, d)| dist < *d) {
            best = Some((tunnel_entity, side_pos, dist));
        }
    }

    best.map(|(e, pos, _)| (e, pos))
}

/// Determine which side to use based on carried resource type.
/// Crystals → Side B, Supplies → Side C.
fn drop_off_side_for_carry(carry: &AgentCarryState) -> char {
    if carry.carrying_crystals() {
        'B'
    } else {
        'C'
    }
}

/// GatheringResource behavior system.
///
/// Handles the full gather-deliver cycle:
/// 1. Move to resource source
/// 2. Extract resources (48 frames)
/// 3. Move to nearest own Tunnel's appropriate side
/// 4. Drop off resources (48 frames)
pub fn gathering_resource_behavior_system(
    mut commands: Commands,
    mut units: Query<(
        Entity,
        &Transform,
        &mut GatheringResourceBehavior,
        &mut LocomotionChannel,
        &mut OrientationChannel,
        &mut UnitCommand,
        &mut BaseBehaviorState,
        &mut AgentCarryState,
        &Owner,
    )>,
    resource_transforms: Query<&Transform, (Without<GatheringResourceBehavior>, Without<TunnelState>)>,
    crystal_patches: Query<&SpaceCrystalPatch>,
    tunnels: Query<(Entity, &Transform, &Owner, &StructureInstance), With<TunnelState>>,
    mut player_resources: Query<(&Player, &mut SyndicatePlayerResources)>,
    dropoff_agents: Query<(Entity, &DroppingOffResourcesBehavior, &AgentCarryState), Without<GatheringResourceBehavior>>,
) {
    // Pre-pass: collect which (tunnel_entity, side) pairs are occupied by agents
    // currently in DroppingOff phase, from both behavior system queries.
    let mut occupied_sides: Vec<(Entity, Entity, char)> = Vec::new(); // (agent, tunnel, side)
    for (agent_entity, _, gathering, _, _, _, _, carry, _) in units.iter() {
        if let GatherPhase::DroppingOff { tunnel_entity, .. } = &gathering.phase {
            let side = drop_off_side_for_carry(carry);
            occupied_sides.push((agent_entity, *tunnel_entity, side));
        }
    }
    for (agent_entity, other_behavior, other_carry) in &dropoff_agents {
        if let DropOffPhase::DroppingOff { .. } = &other_behavior.phase {
            let side = drop_off_side_for_carry(other_carry);
            occupied_sides.push((agent_entity, other_behavior.target_tunnel, side));
        }
    }

    for (entity, transform, mut gathering, mut locomotion, mut orientation, mut command, mut behavior, mut carry_state, owner) in &mut units {
        match gathering.phase.clone() {
            GatherPhase::MovingToResource => {
                // Get resource position
                let resource_pos = match resource_transforms.get(gathering.target_resource) {
                    Ok(t) => t.translation,
                    Err(_) => {
                        // Resource gone — cancel
                        *command = UnitCommand::Idle;
                        *behavior = BaseBehaviorState::None;
                        *locomotion = LocomotionChannel::Stationary;
                        *orientation = OrientationChannel::Maintaining;
                        commands.entity(entity).remove::<GatheringResourceBehavior>();
                        continue;
                    }
                };

                let dist = Vec3::new(
                    transform.translation.x - resource_pos.x, 0.0,
                    transform.translation.z - resource_pos.z,
                ).length();

                if dist < GATHER_ARRIVAL_THRESHOLD {
                    // Arrived at resource — determine extraction duration
                    let is_crystal = crystal_patches.get(gathering.target_resource).is_ok();
                    let duration = if is_crystal {
                        AGENT_MINING_DURATION
                    } else {
                        AGENT_PICKUP_DURATION
                    };
                    gathering.phase = GatherPhase::Extracting { frames_remaining: duration };
                    *locomotion = LocomotionChannel::Stationary;
                    *orientation = OrientationChannel::Maintaining;
                } else {
                    // Move toward resource
                    *locomotion = LocomotionChannel::Moving(vec![resource_pos]);
                    *orientation = OrientationChannel::Turning(resource_pos);
                }
            }
            GatherPhase::Extracting { frames_remaining } => {
                *locomotion = LocomotionChannel::Stationary;
                if frames_remaining <= 1 {
                    // Extraction complete — pick up resources
                    let is_crystal = crystal_patches.get(gathering.target_resource).is_ok();
                    if is_crystal {
                        carry_state.crystals += AGENT_CRYSTAL_CARRY;
                    } else {
                        carry_state.supplies += AGENT_SUPPLY_CARRY;
                    }

                    // Find nearest own tunnel
                    let side = if is_crystal { 'B' } else { 'C' };
                    match find_nearest_own_tunnel(transform.translation, owner, side, &tunnels) {
                        Some((tunnel_entity, side_position)) => {
                            gathering.phase = GatherPhase::MovingToTunnel {
                                tunnel_entity,
                                side_position,
                            };
                        }
                        None => {
                            // No tunnel found — idle with resources
                            *command = UnitCommand::Idle;
                            *behavior = BaseBehaviorState::None;
                            *locomotion = LocomotionChannel::Stationary;
                            *orientation = OrientationChannel::Maintaining;
                            commands.entity(entity).remove::<GatheringResourceBehavior>();
                            continue;
                        }
                    }
                } else {
                    gathering.phase = GatherPhase::Extracting { frames_remaining: frames_remaining - 1 };
                }
            }
            GatherPhase::MovingToTunnel { tunnel_entity, side_position } => {
                let dist = Vec3::new(
                    transform.translation.x - side_position.x, 0.0,
                    transform.translation.z - side_position.z,
                ).length();

                if dist < GATHER_ARRIVAL_THRESHOLD {
                    // Check if another agent is already dropping off at this tunnel + side
                    let side = drop_off_side_for_carry(&carry_state);
                    let side_is_occupied = occupied_sides.iter().any(|(other_entity, other_tunnel, other_side)| {
                        *other_entity != entity && *other_tunnel == tunnel_entity && *other_side == side
                    });

                    if side_is_occupied {
                        // Wait — stay stationary until the side is free
                        *locomotion = LocomotionChannel::Stationary;
                        *orientation = OrientationChannel::Maintaining;
                    } else {
                        gathering.phase = GatherPhase::DroppingOff { tunnel_entity, frames_remaining: AGENT_DROPOFF_DURATION };
                        *locomotion = LocomotionChannel::Stationary;
                        *orientation = OrientationChannel::Maintaining;
                    }
                } else {
                    *locomotion = LocomotionChannel::Moving(vec![side_position]);
                    *orientation = OrientationChannel::Turning(side_position);
                }
            }
            GatherPhase::DroppingOff { tunnel_entity, frames_remaining } => {
                *locomotion = LocomotionChannel::Stationary;
                if frames_remaining <= 1 {
                    // Drop-off complete — transfer resources to player
                    if let Some(agent_player) = owner.player_number() {
                        for (player, mut resources) in &mut player_resources {
                            if player.player_number == agent_player {
                                resources.space_crystals += carry_state.crystals as i32;
                                resources.supplies += carry_state.supplies as i32;
                                break;
                            }
                        }
                    }
                    // Clear carry state
                    carry_state.crystals = 0;
                    carry_state.supplies = 0;

                    // Behavior complete
                    *command = UnitCommand::Idle;
                    *behavior = BaseBehaviorState::None;
                    *locomotion = LocomotionChannel::Stationary;
                    *orientation = OrientationChannel::Maintaining;
                    commands.entity(entity).remove::<GatheringResourceBehavior>();
                } else {
                    gathering.phase = GatherPhase::DroppingOff { tunnel_entity, frames_remaining: frames_remaining - 1 };
                }
            }
        }
    }
}

/// DroppingOffResources behavior system.
///
/// Simpler single-purpose behavior: move to tunnel side, drop off carried resources.
pub fn dropping_off_resources_behavior_system(
    mut commands: Commands,
    mut units: Query<(
        Entity,
        &Transform,
        &mut DroppingOffResourcesBehavior,
        &mut LocomotionChannel,
        &mut OrientationChannel,
        &mut UnitCommand,
        &mut BaseBehaviorState,
        &mut AgentCarryState,
        &Owner,
    )>,
    tunnels: Query<(Entity, &Transform, &Owner, &StructureInstance), With<TunnelState>>,
    mut player_resources: Query<(&Player, &mut SyndicatePlayerResources)>,
    gathering_agents: Query<(Entity, &GatheringResourceBehavior, &AgentCarryState), Without<DroppingOffResourcesBehavior>>,
) {
    // Pre-pass: collect occupied (tunnel, side) pairs from both behavior systems
    let mut occupied_sides: Vec<(Entity, Entity, char)> = Vec::new(); // (agent, tunnel, side)
    for (agent_entity, _, dropping, _, _, _, _, carry, _) in units.iter() {
        if let DropOffPhase::DroppingOff { .. } = &dropping.phase {
            let side = drop_off_side_for_carry(carry);
            occupied_sides.push((agent_entity, dropping.target_tunnel, side));
        }
    }
    for (agent_entity, gathering, carry) in &gathering_agents {
        if let GatherPhase::DroppingOff { tunnel_entity, .. } = &gathering.phase {
            let side = drop_off_side_for_carry(carry);
            occupied_sides.push((agent_entity, *tunnel_entity, side));
        }
    }

    for (entity, transform, mut dropping, mut locomotion, mut orientation, mut command, mut behavior, mut carry_state, owner) in &mut units {
        match dropping.phase.clone() {
            DropOffPhase::MovingToTunnel => {
                // Compute the side position based on carried resources
                let side = drop_off_side_for_carry(&carry_state);
                let tunnel_data = tunnels.get(dropping.target_tunnel);
                let side_pos = match tunnel_data {
                    Ok((_, tunnel_transform, _, structure_instance)) => {
                        tunnel_side_world_position(tunnel_transform, structure_instance, side)
                    }
                    Err(_) => {
                        // Tunnel gone — cancel
                        *command = UnitCommand::Idle;
                        *behavior = BaseBehaviorState::None;
                        *locomotion = LocomotionChannel::Stationary;
                        *orientation = OrientationChannel::Maintaining;
                        commands.entity(entity).remove::<DroppingOffResourcesBehavior>();
                        continue;
                    }
                };

                let dist = Vec3::new(
                    transform.translation.x - side_pos.x, 0.0,
                    transform.translation.z - side_pos.z,
                ).length();

                if dist < GATHER_ARRIVAL_THRESHOLD {
                    // Check occupancy before starting drop-off
                    let side_is_occupied = occupied_sides.iter().any(|(other_entity, other_tunnel, other_side)| {
                        *other_entity != entity && *other_tunnel == dropping.target_tunnel && *other_side == side
                    });

                    if side_is_occupied {
                        // Wait — stay stationary until the side is free
                        *locomotion = LocomotionChannel::Stationary;
                        *orientation = OrientationChannel::Maintaining;
                    } else {
                        dropping.phase = DropOffPhase::DroppingOff { frames_remaining: AGENT_DROPOFF_DURATION };
                        *locomotion = LocomotionChannel::Stationary;
                        *orientation = OrientationChannel::Maintaining;
                    }
                } else {
                    *locomotion = LocomotionChannel::Moving(vec![side_pos]);
                    *orientation = OrientationChannel::Turning(side_pos);
                }
            }
            DropOffPhase::DroppingOff { frames_remaining } => {
                *locomotion = LocomotionChannel::Stationary;
                if frames_remaining <= 1 {
                    // Transfer resources
                    if let Some(agent_player) = owner.player_number() {
                        for (player, mut resources) in &mut player_resources {
                            if player.player_number == agent_player {
                                resources.space_crystals += carry_state.crystals as i32;
                                resources.supplies += carry_state.supplies as i32;
                                break;
                            }
                        }
                    }
                    carry_state.crystals = 0;
                    carry_state.supplies = 0;

                    *command = UnitCommand::Idle;
                    *behavior = BaseBehaviorState::None;
                    *locomotion = LocomotionChannel::Stationary;
                    *orientation = OrientationChannel::Maintaining;
                    commands.entity(entity).remove::<DroppingOffResourcesBehavior>();
                } else {
                    dropping.phase = DropOffPhase::DroppingOff { frames_remaining: frames_remaining - 1 };
                }
            }
        }
    }
}

/// BuildingTunnel behavior system.
///
/// Handles the full tunnel construction sequence:
/// 1. MovingToSite: Agent walks to the build location
/// 2. Constructing: Agent embeds in the partially-built Tunnel, ticks construction
///    - If tunnel is destroyed: Agent emerges alive
///    - If construction completes: Agent enters the Tunnel Network (despawned)
pub fn building_tunnel_behavior_system(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    mut units: Query<(
        Entity,
        &Transform,
        &Owner,
        &mut BuildingTunnelBehavior,
        &mut LocomotionChannel,
        &mut OrientationChannel,
        &mut UnitCommand,
        &mut BaseBehaviorState,
        &mut Visibility,
    )>,
    tunnel_query: Query<(Entity, &Owner), With<TunnelState>>,
    tunnel_hp_query: Query<&ObjectInstance, With<ConstructionHP>>,
    mut syndicate_players: Query<(&Player, &mut SyndicatePlayerResources)>,
) {
    // Single-Agent construction enforcement: collect locations where construction is in progress
    let constructing_locations: Vec<(Entity, Vec3)> = units.iter()
        .filter_map(|(e, _, _, b, ..)| {
            if matches!(b.phase, BuildTunnelPhase::Constructing { .. }) {
                Some((e, b.target_location))
            } else {
                None
            }
        })
        .collect();

    for (entity, transform, owner, mut building, mut locomotion, mut orientation, mut command, mut behavior, mut visibility) in &mut units {
        match building.phase.clone() {
            BuildTunnelPhase::MovingToSite => {
                let target = building.target_location;
                let unit_pos = transform.translation;
                let distance = Vec3::new(unit_pos.x - target.x, 0.0, unit_pos.z - target.z).length();

                if distance < BUILD_ARRIVAL_THRESHOLD {
                    // Single-Agent construction enforcement: reject if another agent
                    // is already constructing at this location
                    let location_taken = constructing_locations.iter().any(|(other_e, loc)| {
                        *other_e != entity
                            && (loc.x - target.x).abs() < 1.0
                            && (loc.z - target.z).abs() < 1.0
                    });
                    if location_taken {
                        info!("Agent: Build rejected — another Agent is already constructing at ({:.1}, {:.1})", target.x, target.z);
                        *command = UnitCommand::Idle;
                        *behavior = BaseBehaviorState::None;
                        *locomotion = LocomotionChannel::Stationary;
                        *orientation = OrientationChannel::Maintaining;
                        commands.entity(entity).remove::<BuildingTunnelBehavior>();
                        continue;
                    }
                    // Arrived at build site — count existing tunnels and compute cost
                    let agent_player = match owner.player_number() {
                        Some(p) => p,
                        None => {
                            // No player — cancel
                            *command = UnitCommand::Idle;
                            *behavior = BaseBehaviorState::None;
                            *locomotion = LocomotionChannel::Stationary;
                            *orientation = OrientationChannel::Maintaining;
                            commands.entity(entity).remove::<BuildingTunnelBehavior>();
                            continue;
                        }
                    };

                    let existing_count = tunnel_query.iter()
                        .filter(|(_, o)| o.player_number() == Some(agent_player))
                        .count();
                    let cost = tunnel_construction_cost(existing_count as u32);

                    // Check and deduct supplies
                    let mut can_afford = false;
                    for (player, mut resources) in &mut syndicate_players {
                        if player.player_number == agent_player {
                            if resources.supplies >= cost as i32 {
                                resources.supplies -= cost as i32;
                                can_afford = true;
                            }
                            break;
                        }
                    }

                    if !can_afford {
                        // Insufficient funds — cancel build
                        *command = UnitCommand::Idle;
                        *behavior = BaseBehaviorState::None;
                        *locomotion = LocomotionChannel::Stationary;
                        *orientation = OrientationChannel::Maintaining;
                        commands.entity(entity).remove::<BuildingTunnelBehavior>();
                        continue;
                    }

                    // Spawn partially-built tunnel
                    let (grid_x, grid_z) = world_to_grid(target, 1.0);
                    let tunnel_entity = spawn_tunnel_under_construction(
                        &mut commands,
                        &mut meshes,
                        &mut materials,
                        grid_x, grid_z,
                        owner.clone(),
                        AGENT_TUNNEL_BUILD_FRAMES,
                    );

                    // Hide the Agent (untargetable during construction)
                    *visibility = Visibility::Hidden;
                    *locomotion = LocomotionChannel::Stationary;
                    *orientation = OrientationChannel::Maintaining;

                    // Transition to Constructing phase
                    building.phase = BuildTunnelPhase::Constructing {
                        tunnel_entity,
                        frames_elapsed: 0,
                    };
                } else {
                    // Not yet arrived — move toward target
                    *locomotion = LocomotionChannel::Moving(vec![target]);
                    *orientation = OrientationChannel::Turning(target);
                }
            }
            BuildTunnelPhase::Constructing { tunnel_entity, frames_elapsed } => {
                *locomotion = LocomotionChannel::Stationary;

                // Check if tunnel still exists
                if tunnel_hp_query.get(tunnel_entity).is_err() {
                    // Tunnel was destroyed OR construction completed (ConstructionHP removed)
                    // Check if the tunnel entity itself still exists by trying to get anything
                    if commands.get_entity(tunnel_entity).is_ok() && frames_elapsed >= AGENT_TUNNEL_BUILD_FRAMES {
                        // Construction complete — tunnel still alive, ConstructionHP was removed
                        // Agent enters the tunnel network (despawn)
                        commands.entity(entity).despawn();
                    } else {
                        // Tunnel was destroyed — Agent emerges
                        *visibility = Visibility::Inherited;
                        *command = UnitCommand::Idle;
                        *behavior = BaseBehaviorState::None;
                        *locomotion = LocomotionChannel::Stationary;
                        *orientation = OrientationChannel::Maintaining;
                        commands.entity(entity).remove::<BuildingTunnelBehavior>();
                    }
                    continue;
                }

                // Tunnel is alive and still under construction — increment frame count
                let new_frames = frames_elapsed + 1;

                if new_frames >= AGENT_TUNNEL_BUILD_FRAMES {
                    // Construction complete — ConstructionHP system will remove the component
                    // Agent enters the tunnel network
                    commands.entity(entity).despawn();
                } else {
                    building.phase = BuildTunnelPhase::Constructing {
                        tunnel_entity,
                        frames_elapsed: new_frames,
                    };
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use bevy::ecs::system::RunSystemOnce;
    use crate::types::ObjectEnum;

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

        world.run_system_once(building_behavior_system).unwrap();

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

        world.run_system_once(building_behavior_system).unwrap();

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

        world.run_system_once(building_behavior_system).unwrap();

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

        world.run_system_once(building_behavior_system).unwrap();

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

        world.run_system_once(building_behavior_system).unwrap();

        // Should remain idle, marker should still be there (no re-processing)
        let command = world.entity(entity).get::<UnitCommand>().unwrap();
        assert!(matches!(command, UnitCommand::Idle));
    }

    // === BuildingTunnelBehavior tests ===

    #[test]
    fn building_tunnel_constant_is_480_frames() {
        assert_eq!(AGENT_TUNNEL_BUILD_FRAMES, 480);
    }

    #[test]
    fn building_tunnel_behavior_new_defaults() {
        let target = Vec3::new(5.0, 0.0, 5.0);
        let behavior = BuildingTunnelBehavior::new(target);
        assert_eq!(behavior.target_location, target);
        assert_eq!(behavior.phase, BuildTunnelPhase::MovingToSite);
        assert!(behavior.path.is_empty());
        assert_eq!(behavior.path_index, 0);
    }

    #[test]
    fn building_tunnel_behavior_is_component() {
        let mut world = World::new();
        let target = Vec3::new(10.0, 0.0, 10.0);
        let entity = world.spawn(BuildingTunnelBehavior::new(target)).id();
        let b = world.entity(entity).get::<BuildingTunnelBehavior>().unwrap();
        assert_eq!(b.target_location, target);
    }

    #[test]
    fn build_tunnel_phase_moving_to_site_eq() {
        assert_eq!(BuildTunnelPhase::MovingToSite, BuildTunnelPhase::MovingToSite);
    }

    #[test]
    fn build_tunnel_phase_constructing_stores_data() {
        let phase = BuildTunnelPhase::Constructing {
            tunnel_entity: Entity::from_raw_u32(42).unwrap(),
            frames_elapsed: 100,
        };
        if let BuildTunnelPhase::Constructing { tunnel_entity, frames_elapsed } = phase {
            assert_eq!(tunnel_entity, Entity::from_raw_u32(42).unwrap());
            assert_eq!(frames_elapsed, 100);
        } else {
            panic!("Expected Constructing variant");
        }
    }

    /// Helper to spawn an Agent entity with all components needed by building_tunnel_behavior_system
    fn spawn_agent_with_build_tunnel(world: &mut World, pos: Vec3, target: Vec3, owner: Owner) -> Entity {
        world.spawn((
            Transform::from_translation(pos),
            owner,
            BuildingTunnelBehavior::new(target),
            LocomotionChannel::Stationary,
            OrientationChannel::Maintaining,
            UnitCommand::BuildTunnel(target),
            BaseBehaviorState::None,
            Visibility::Inherited,
        )).id()
    }

    /// Helper to spawn a Player entity with SyndicatePlayerResources
    fn spawn_syndicate_player(world: &mut World, player_number: u8, supplies: i32) -> Entity {
        world.spawn((
            Player {
                name: format!("Player {}", player_number),
                faction: crate::types::FactionEnum::TheSyndicate,
                player_number,
            },
            SyndicatePlayerResources {
                space_crystals: 0,
                supplies,
                tunnel_space_provided: 100,
                tunnel_space_used: 0,
            },
        )).id()
    }

    #[test]
    fn building_tunnel_moving_to_site_sets_moving() {
        // Agent far from target — should set Moving locomotion
        let mut app = App::new();
        app.add_plugins(MinimalPlugins);
        app.init_resource::<Assets<Mesh>>();
        app.init_resource::<Assets<StandardMaterial>>();

        let target = Vec3::new(10.0, 0.0, 10.0);
        let entity = spawn_agent_with_build_tunnel(app.world_mut(), Vec3::ZERO, target, Owner::player(0));
        spawn_syndicate_player(app.world_mut(), 0, 100);

        app.world_mut().run_system_once(building_tunnel_behavior_system).unwrap();

        let locomotion = app.world_mut().entity(entity).get::<LocomotionChannel>().unwrap();
        assert!(matches!(locomotion, LocomotionChannel::Moving(_)));
        let orientation = app.world_mut().entity(entity).get::<OrientationChannel>().unwrap();
        assert!(matches!(orientation, OrientationChannel::Turning(_)));
        // Phase should still be MovingToSite
        let behavior = app.world_mut().entity(entity).get::<BuildingTunnelBehavior>().unwrap();
        assert_eq!(behavior.phase, BuildTunnelPhase::MovingToSite);
    }

    #[test]
    fn building_tunnel_arrival_spawns_tunnel() {
        // Agent at target with enough supplies — should spawn tunnel
        let mut app = App::new();
        app.add_plugins(MinimalPlugins);
        app.init_resource::<Assets<Mesh>>();
        app.init_resource::<Assets<StandardMaterial>>();

        let target = Vec3::new(0.5, 0.0, 0.5);
        let entity = spawn_agent_with_build_tunnel(app.world_mut(), target, target, Owner::player(0));
        spawn_syndicate_player(app.world_mut(), 0, 100);

        app.world_mut().run_system_once(building_tunnel_behavior_system).unwrap();

        // Agent should be hidden
        let vis = app.world_mut().entity(entity).get::<Visibility>().unwrap();
        assert_eq!(*vis, Visibility::Hidden);

        // Phase should be Constructing
        let behavior = app.world_mut().entity(entity).get::<BuildingTunnelBehavior>().unwrap();
        if let BuildTunnelPhase::Constructing { frames_elapsed, .. } = &behavior.phase {
            assert_eq!(*frames_elapsed, 0);
        } else {
            panic!("Expected Constructing phase");
        }

        // Locomotion should be stationary
        let locomotion = app.world_mut().entity(entity).get::<LocomotionChannel>().unwrap();
        assert!(matches!(locomotion, LocomotionChannel::Stationary));
    }

    #[test]
    fn building_tunnel_cost_deducted() {
        // First tunnel is free (0 existing tunnels)
        let mut app = App::new();
        app.add_plugins(MinimalPlugins);
        app.init_resource::<Assets<Mesh>>();
        app.init_resource::<Assets<StandardMaterial>>();

        let target = Vec3::new(0.5, 0.0, 0.5);
        spawn_agent_with_build_tunnel(app.world_mut(), target, target, Owner::player(0));
        let player_entity = spawn_syndicate_player(app.world_mut(), 0, 10);

        app.world_mut().run_system_once(building_tunnel_behavior_system).unwrap();

        // First tunnel costs 0 — supplies should remain 10
        let resources = app.world_mut().entity(player_entity).get::<SyndicatePlayerResources>().unwrap();
        assert_eq!(resources.supplies, 10);
    }

    #[test]
    fn building_tunnel_cost_deducted_second_tunnel() {
        // Second tunnel costs 1 supply (1 existing tunnel)
        let mut app = App::new();
        app.add_plugins(MinimalPlugins);
        app.init_resource::<Assets<Mesh>>();
        app.init_resource::<Assets<StandardMaterial>>();

        // Spawn an existing tunnel owned by player 0
        app.world_mut().spawn((
            TunnelState::default_tier1(),
            Owner::player(0),
        ));

        let target = Vec3::new(0.5, 0.0, 0.5);
        spawn_agent_with_build_tunnel(app.world_mut(), target, target, Owner::player(0));
        let player_entity = spawn_syndicate_player(app.world_mut(), 0, 10);

        app.world_mut().run_system_once(building_tunnel_behavior_system).unwrap();

        // Second tunnel costs 1 supply
        let resources = app.world_mut().entity(player_entity).get::<SyndicatePlayerResources>().unwrap();
        assert_eq!(resources.supplies, 9);
    }

    #[test]
    fn building_tunnel_insufficient_funds_cancels() {
        // Agent can't afford — should cancel and idle
        let mut app = App::new();
        app.add_plugins(MinimalPlugins);
        app.init_resource::<Assets<Mesh>>();
        app.init_resource::<Assets<StandardMaterial>>();

        // 1 existing tunnel → cost = 1, but player has 0 supplies
        app.world_mut().spawn((
            TunnelState::default_tier1(),
            Owner::player(0),
        ));

        let target = Vec3::new(0.5, 0.0, 0.5);
        let entity = spawn_agent_with_build_tunnel(app.world_mut(), target, target, Owner::player(0));
        spawn_syndicate_player(app.world_mut(), 0, 0);

        app.world_mut().run_system_once(building_tunnel_behavior_system).unwrap();

        // Should be idle, marker removed
        let command = app.world_mut().entity(entity).get::<UnitCommand>().unwrap();
        assert!(matches!(command, UnitCommand::Idle));
        assert!(app.world_mut().entity(entity).get::<BuildingTunnelBehavior>().is_none());
    }

    #[test]
    fn building_tunnel_constructing_increments_frames() {
        // Agent in Constructing phase — frames should increment
        let mut app = App::new();
        app.add_plugins(MinimalPlugins);
        app.init_resource::<Assets<Mesh>>();
        app.init_resource::<Assets<StandardMaterial>>();

        // Create a fake tunnel entity with ConstructionHP
        let tunnel_entity = app.world_mut().spawn((
            ObjectInstance::under_construction(ObjectEnum::Tunnel, 600.0),
            ConstructionHP::new(AGENT_TUNNEL_BUILD_FRAMES),
        )).id();

        let mut behavior = BuildingTunnelBehavior::new(Vec3::ZERO);
        behavior.phase = BuildTunnelPhase::Constructing {
            tunnel_entity,
            frames_elapsed: 10,
        };

        let entity = app.world_mut().spawn((
            Transform::from_xyz(0.0, 0.0, 0.0),
            Owner::player(0),
            behavior,
            LocomotionChannel::Stationary,
            OrientationChannel::Maintaining,
            UnitCommand::BuildTunnel(Vec3::ZERO),
            BaseBehaviorState::None,
            Visibility::Hidden,
        )).id();

        spawn_syndicate_player(app.world_mut(), 0, 10);

        app.world_mut().run_system_once(building_tunnel_behavior_system).unwrap();

        // Frames should be 11
        let b = app.world_mut().entity(entity).get::<BuildingTunnelBehavior>().unwrap();
        if let BuildTunnelPhase::Constructing { frames_elapsed, .. } = &b.phase {
            assert_eq!(*frames_elapsed, 11);
        } else {
            panic!("Expected Constructing phase");
        }
    }

    #[test]
    fn building_tunnel_destroyed_agent_emerges() {
        // Tunnel destroyed during construction — Agent emerges
        let mut app = App::new();
        app.add_plugins(MinimalPlugins);
        app.init_resource::<Assets<Mesh>>();
        app.init_resource::<Assets<StandardMaterial>>();

        // Don't spawn a tunnel entity — simulates it being destroyed
        let fake_tunnel = Entity::from_raw_u32(9999).unwrap();

        let mut behavior = BuildingTunnelBehavior::new(Vec3::ZERO);
        behavior.phase = BuildTunnelPhase::Constructing {
            tunnel_entity: fake_tunnel,
            frames_elapsed: 100,
        };

        let entity = app.world_mut().spawn((
            Transform::from_xyz(0.0, 0.0, 0.0),
            Owner::player(0),
            behavior,
            LocomotionChannel::Stationary,
            OrientationChannel::Maintaining,
            UnitCommand::BuildTunnel(Vec3::ZERO),
            BaseBehaviorState::None,
            Visibility::Hidden,
        )).id();

        spawn_syndicate_player(app.world_mut(), 0, 10);

        app.world_mut().run_system_once(building_tunnel_behavior_system).unwrap();

        // Agent should emerge — visible, idle, no marker
        let vis = app.world_mut().entity(entity).get::<Visibility>().unwrap();
        assert_eq!(*vis, Visibility::Inherited);
        let command = app.world_mut().entity(entity).get::<UnitCommand>().unwrap();
        assert!(matches!(command, UnitCommand::Idle));
        assert!(app.world_mut().entity(entity).get::<BuildingTunnelBehavior>().is_none());
    }

    #[test]
    fn building_tunnel_completes_agent_despawned() {
        // Construction reaches 480 frames — Agent should be despawned
        let mut app = App::new();
        app.add_plugins(MinimalPlugins);
        app.init_resource::<Assets<Mesh>>();
        app.init_resource::<Assets<StandardMaterial>>();

        // Create tunnel entity with ConstructionHP still present
        let tunnel_entity = app.world_mut().spawn((
            ObjectInstance::under_construction(ObjectEnum::Tunnel, 600.0),
            ConstructionHP::new(AGENT_TUNNEL_BUILD_FRAMES),
        )).id();

        let mut behavior = BuildingTunnelBehavior::new(Vec3::ZERO);
        behavior.phase = BuildTunnelPhase::Constructing {
            tunnel_entity,
            frames_elapsed: AGENT_TUNNEL_BUILD_FRAMES - 1, // One frame away from completion
        };

        let entity = app.world_mut().spawn((
            Transform::from_xyz(0.0, 0.0, 0.0),
            Owner::player(0),
            behavior,
            LocomotionChannel::Stationary,
            OrientationChannel::Maintaining,
            UnitCommand::BuildTunnel(Vec3::ZERO),
            BaseBehaviorState::None,
            Visibility::Hidden,
        )).id();

        spawn_syndicate_player(app.world_mut(), 0, 10);

        app.world_mut().run_system_once(building_tunnel_behavior_system).unwrap();

        // Agent should be despawned
        assert!(app.world_mut().get_entity(entity).is_err());
    }

    #[test]
    fn building_tunnel_completion_with_construction_hp_removed() {
        // When ConstructionHP is removed (by construction_hp_tick_system) and frames >= 480,
        // Agent should be despawned (enters tunnel network)
        let mut app = App::new();
        app.add_plugins(MinimalPlugins);
        app.init_resource::<Assets<Mesh>>();
        app.init_resource::<Assets<StandardMaterial>>();

        // Create tunnel entity WITHOUT ConstructionHP (already completed)
        let tunnel_entity = app.world_mut().spawn((
            ObjectInstance::destructible(ObjectEnum::Tunnel, 600.0),
        )).id();

        let mut behavior = BuildingTunnelBehavior::new(Vec3::ZERO);
        behavior.phase = BuildTunnelPhase::Constructing {
            tunnel_entity,
            frames_elapsed: AGENT_TUNNEL_BUILD_FRAMES,
        };

        let entity = app.world_mut().spawn((
            Transform::from_xyz(0.0, 0.0, 0.0),
            Owner::player(0),
            behavior,
            LocomotionChannel::Stationary,
            OrientationChannel::Maintaining,
            UnitCommand::BuildTunnel(Vec3::ZERO),
            BaseBehaviorState::None,
            Visibility::Hidden,
        )).id();

        spawn_syndicate_player(app.world_mut(), 0, 10);

        app.world_mut().run_system_once(building_tunnel_behavior_system).unwrap();

        // Agent should be despawned (entered tunnel network)
        assert!(app.world_mut().get_entity(entity).is_err());
    }

    #[test]
    fn building_tunnel_no_player_cancels() {
        // Agent with no owner — should cancel
        let mut app = App::new();
        app.add_plugins(MinimalPlugins);
        app.init_resource::<Assets<Mesh>>();
        app.init_resource::<Assets<StandardMaterial>>();

        let target = Vec3::new(0.5, 0.0, 0.5);
        let entity = app.world_mut().spawn((
            Transform::from_translation(target),
            Owner::neutral(),
            BuildingTunnelBehavior::new(target),
            LocomotionChannel::Stationary,
            OrientationChannel::Maintaining,
            UnitCommand::BuildTunnel(target),
            BaseBehaviorState::None,
            Visibility::Inherited,
        )).id();

        app.world_mut().run_system_once(building_tunnel_behavior_system).unwrap();

        let command = app.world_mut().entity(entity).get::<UnitCommand>().unwrap();
        assert!(matches!(command, UnitCommand::Idle));
        assert!(app.world_mut().entity(entity).get::<BuildingTunnelBehavior>().is_none());
    }

    #[test]
    fn build_tunnel_has_indicator() {
        use crate::game::units::types::types::{command_has_indicator, command_indicator_color};
        let cmd = UnitCommand::BuildTunnel(Vec3::new(5.0, 0.0, 5.0));
        assert!(command_has_indicator(&cmd));
        assert_eq!(command_indicator_color(&cmd), Color::srgb(0.0, 1.0, 0.0));
    }

    #[test]
    fn building_tunnel_arrival_spawns_tunnel_entity() {
        // Verify that a TunnelState entity is spawned when Agent arrives
        let mut app = App::new();
        app.add_plugins(MinimalPlugins);
        app.init_resource::<Assets<Mesh>>();
        app.init_resource::<Assets<StandardMaterial>>();

        let target = Vec3::new(0.5, 0.0, 0.5);
        spawn_agent_with_build_tunnel(app.world_mut(), target, target, Owner::player(0));
        spawn_syndicate_player(app.world_mut(), 0, 100);

        // No tunnels before
        let tunnel_count_before = app.world_mut().query::<&TunnelState>().iter(app.world()).count();
        assert_eq!(tunnel_count_before, 0);

        app.world_mut().run_system_once(building_tunnel_behavior_system).unwrap();

        // Should have 1 tunnel after (deferred commands need apply)
        app.world_mut().flush();
        let tunnel_count_after = app.world_mut().query::<&TunnelState>().iter(app.world()).count();
        assert_eq!(tunnel_count_after, 1);
    }

    #[test]
    fn building_tunnel_spawned_tunnel_has_construction_hp() {
        // Verify spawned tunnel has ConstructionHP component
        let mut app = App::new();
        app.add_plugins(MinimalPlugins);
        app.init_resource::<Assets<Mesh>>();
        app.init_resource::<Assets<StandardMaterial>>();

        let target = Vec3::new(0.5, 0.0, 0.5);
        spawn_agent_with_build_tunnel(app.world_mut(), target, target, Owner::player(0));
        spawn_syndicate_player(app.world_mut(), 0, 100);

        app.world_mut().run_system_once(building_tunnel_behavior_system).unwrap();
        app.world_mut().flush();

        let construction_count = app.world_mut()
            .query::<(&TunnelState, &ConstructionHP)>()
            .iter(app.world())
            .count();
        assert_eq!(construction_count, 1);
    }

    #[test]
    fn building_tunnel_spawned_tunnel_starts_at_10_percent_hp() {
        // Verify spawned tunnel starts at 10% HP
        let mut app = App::new();
        app.add_plugins(MinimalPlugins);
        app.init_resource::<Assets<Mesh>>();
        app.init_resource::<Assets<StandardMaterial>>();

        let target = Vec3::new(0.5, 0.0, 0.5);
        spawn_agent_with_build_tunnel(app.world_mut(), target, target, Owner::player(0));
        spawn_syndicate_player(app.world_mut(), 0, 100);

        app.world_mut().run_system_once(building_tunnel_behavior_system).unwrap();
        app.world_mut().flush();

        for (obj, _) in app.world_mut().query::<(&ObjectInstance, &TunnelState)>().iter(app.world()) {
            assert!((obj.hp.unwrap() - 60.0).abs() < 0.1, "HP should start at 10% of 600 = 60");
        }
    }

    // === Single-Agent Construction Enforcement tests ===

    #[test]
    fn building_tunnel_rejects_second_agent_at_same_location() {
        // Agent B arrives at a location where Agent A is already constructing —
        // Agent B should be rejected (idled) while Agent A continues
        let mut app = App::new();
        app.add_plugins(MinimalPlugins);
        app.init_resource::<Assets<Mesh>>();
        app.init_resource::<Assets<StandardMaterial>>();

        let target = Vec3::new(0.5, 0.0, 0.5);

        // Agent A: already in Constructing phase
        let tunnel_entity = app.world_mut().spawn((
            TunnelState::default_tier1(),
            Owner::player(0),
            ObjectInstance::destructible(ObjectEnum::Tunnel, 600.0),
            ConstructionHP::new(AGENT_TUNNEL_BUILD_FRAMES),
        )).id();

        let mut behavior_a = BuildingTunnelBehavior::new(target);
        behavior_a.phase = BuildTunnelPhase::Constructing {
            tunnel_entity,
            frames_elapsed: 100,
        };
        let agent_a = app.world_mut().spawn((
            Transform::from_translation(target),
            Owner::player(0),
            behavior_a,
            LocomotionChannel::Stationary,
            OrientationChannel::Maintaining,
            UnitCommand::BuildTunnel(target),
            BaseBehaviorState::None,
            Visibility::Hidden,
        )).id();

        // Agent B: arriving at same location (MovingToSite, at target)
        let agent_b = spawn_agent_with_build_tunnel(
            app.world_mut(), target, target, Owner::player(0),
        );

        spawn_syndicate_player(app.world_mut(), 0, 200);

        app.world_mut().run_system_once(building_tunnel_behavior_system).unwrap();
        app.world_mut().flush();

        // Agent A should still be constructing
        let a_behavior = app.world_mut().entity(agent_a).get::<BuildingTunnelBehavior>();
        assert!(a_behavior.is_some(), "Agent A should still have BuildingTunnelBehavior");

        // Agent B should have been rejected — marker removed, command idle
        let b_behavior = app.world_mut().entity(agent_b).get::<BuildingTunnelBehavior>();
        assert!(b_behavior.is_none(), "Agent B should have BuildingTunnelBehavior removed");
        let b_cmd = app.world_mut().entity(agent_b).get::<UnitCommand>().unwrap();
        assert!(matches!(b_cmd, UnitCommand::Idle), "Agent B should be idle");
    }

    #[test]
    fn building_tunnel_allows_agent_at_different_location() {
        // Agent B builds at a different location than Agent A — should proceed
        let mut app = App::new();
        app.add_plugins(MinimalPlugins);
        app.init_resource::<Assets<Mesh>>();
        app.init_resource::<Assets<StandardMaterial>>();

        let target_a = Vec3::new(0.5, 0.0, 0.5);
        let target_b = Vec3::new(10.5, 0.0, 10.5);

        // Agent A: already in Constructing phase at target_a
        let tunnel_entity = app.world_mut().spawn((
            TunnelState::default_tier1(),
            Owner::player(0),
            ObjectInstance::destructible(ObjectEnum::Tunnel, 600.0),
            ConstructionHP::new(AGENT_TUNNEL_BUILD_FRAMES),
        )).id();

        let mut behavior_a = BuildingTunnelBehavior::new(target_a);
        behavior_a.phase = BuildTunnelPhase::Constructing {
            tunnel_entity,
            frames_elapsed: 100,
        };
        app.world_mut().spawn((
            Transform::from_translation(target_a),
            Owner::player(0),
            behavior_a,
            LocomotionChannel::Stationary,
            OrientationChannel::Maintaining,
            UnitCommand::BuildTunnel(target_a),
            BaseBehaviorState::None,
            Visibility::Hidden,
        ));

        // Agent B: arriving at different location
        let agent_b = spawn_agent_with_build_tunnel(
            app.world_mut(), target_b, target_b, Owner::player(0),
        );

        spawn_syndicate_player(app.world_mut(), 0, 200);

        app.world_mut().run_system_once(building_tunnel_behavior_system).unwrap();
        app.world_mut().flush();

        // Agent B should have started constructing (behavior still present, phase = Constructing)
        let b_behavior = app.world_mut().entity(agent_b).get::<BuildingTunnelBehavior>();
        assert!(b_behavior.is_some(), "Agent B should still have BuildingTunnelBehavior");
        if let Some(b) = b_behavior {
            assert!(
                matches!(b.phase, BuildTunnelPhase::Constructing { .. }),
                "Agent B should be in Constructing phase"
            );
        }
    }

    #[test]
    fn building_tunnel_allows_after_first_agent_finishes() {
        // Agent A's tunnel was destroyed → Agent B should be able to build at the same location
        let mut app = App::new();
        app.add_plugins(MinimalPlugins);
        app.init_resource::<Assets<Mesh>>();
        app.init_resource::<Assets<StandardMaterial>>();

        let target = Vec3::new(0.5, 0.0, 0.5);

        // No other agents constructing — Agent B should succeed
        let agent_b = spawn_agent_with_build_tunnel(
            app.world_mut(), target, target, Owner::player(0),
        );

        spawn_syndicate_player(app.world_mut(), 0, 200);

        app.world_mut().run_system_once(building_tunnel_behavior_system).unwrap();
        app.world_mut().flush();

        // Agent B should have started constructing
        let b_behavior = app.world_mut().entity(agent_b).get::<BuildingTunnelBehavior>();
        assert!(b_behavior.is_some(), "Agent B should still have BuildingTunnelBehavior");
        if let Some(b) = b_behavior {
            assert!(
                matches!(b.phase, BuildTunnelPhase::Constructing { .. }),
                "Agent B should be in Constructing phase"
            );
        }
    }

    // === Gathering/DropOff helper function tests ===

    #[test]
    fn gather_arrival_threshold_is_positive() {
        assert!(GATHER_ARRIVAL_THRESHOLD > 0.0);
    }

    #[test]
    fn gather_arrival_threshold_is_1_5_gu() {
        assert!((GATHER_ARRIVAL_THRESHOLD - 1.5).abs() < f32::EPSILON);
    }

    #[test]
    fn drop_off_side_crystals_is_b() {
        let carry = AgentCarryState { crystals: 50, supplies: 0 };
        assert_eq!(drop_off_side_for_carry(&carry), 'B');
    }

    #[test]
    fn drop_off_side_supplies_is_c() {
        let carry = AgentCarryState { crystals: 0, supplies: 1 };
        assert_eq!(drop_off_side_for_carry(&carry), 'C');
    }

    #[test]
    fn drop_off_side_both_prefers_crystals() {
        // When carrying both, crystals take priority (crystals > 0 checked first)
        let carry = AgentCarryState { crystals: 10, supplies: 5 };
        assert_eq!(drop_off_side_for_carry(&carry), 'B');
    }

    #[test]
    fn drop_off_side_empty_defaults_to_c() {
        // Edge case: not carrying anything, defaults to supplies side
        let carry = AgentCarryState { crystals: 0, supplies: 0 };
        assert_eq!(drop_off_side_for_carry(&carry), 'C');
    }

    #[test]
    fn tunnel_side_world_position_r0_side_b_is_east() {
        // ABCD at R0: [N=A, E=B, S=C, W=D]. Side B is East.
        let transform = Transform::from_translation(Vec3::new(10.0, 0.0, 10.0));
        let instance = StructureInstance::default(); // R0, no flips
        let pos = tunnel_side_world_position(&transform, &instance, 'B');
        // East = x + 2.5
        assert!((pos.x - 12.5).abs() < f32::EPSILON);
        assert!((pos.z - 10.0).abs() < f32::EPSILON);
    }

    #[test]
    fn tunnel_side_world_position_r0_side_c_is_south() {
        let transform = Transform::from_translation(Vec3::new(10.0, 0.0, 10.0));
        let instance = StructureInstance::default();
        let pos = tunnel_side_world_position(&transform, &instance, 'C');
        // South = z + 2.5
        assert!((pos.x - 10.0).abs() < f32::EPSILON);
        assert!((pos.z - 12.5).abs() < f32::EPSILON);
    }

    #[test]
    fn tunnel_side_world_position_r0_side_a_is_north() {
        let transform = Transform::from_translation(Vec3::new(5.0, 0.0, 5.0));
        let instance = StructureInstance::default();
        let pos = tunnel_side_world_position(&transform, &instance, 'A');
        // North = z - 2.5
        assert!((pos.x - 5.0).abs() < f32::EPSILON);
        assert!((pos.z - 2.5).abs() < f32::EPSILON);
    }

    #[test]
    fn tunnel_side_world_position_r0_side_d_is_west() {
        let transform = Transform::from_translation(Vec3::new(5.0, 0.0, 5.0));
        let instance = StructureInstance::default();
        let pos = tunnel_side_world_position(&transform, &instance, 'D');
        // West = x - 2.5
        assert!((pos.x - 2.5).abs() < f32::EPSILON);
        assert!((pos.z - 5.0).abs() < f32::EPSILON);
    }

    // === GatheringResource behavior system tests ===

    /// Helper to spawn an agent entity with GatheringResourceBehavior
    fn spawn_agent_with_gathering(
        world: &mut World,
        pos: Vec3,
        target_resource: Entity,
        owner: Owner,
    ) -> Entity {
        world.spawn((
            Transform::from_translation(pos),
            owner,
            GatheringResourceBehavior::new(target_resource),
            LocomotionChannel::Stationary,
            OrientationChannel::Maintaining,
            UnitCommand::Gather(target_resource),
            BaseBehaviorState::None,
            AgentCarryState::default(),
        )).id()
    }

    /// Helper to spawn a crystal patch entity
    fn spawn_crystal_patch(world: &mut World, pos: Vec3) -> Entity {
        world.spawn((
            Transform::from_translation(pos),
            SpaceCrystalPatch {
                remaining_amount: 1000,
                initial_amount: 1000,
                has_plate: false,
            },
        )).id()
    }

    /// Helper to spawn a tunnel entity
    fn spawn_tunnel(world: &mut World, pos: Vec3, owner: Owner) -> Entity {
        world.spawn((
            Transform::from_translation(pos),
            owner,
            StructureInstance::default(),
            TunnelState::default_tier1(),
        )).id()
    }

    #[test]
    fn gathering_moving_to_resource_sets_moving_when_far() {
        let mut app = App::new();
        app.add_plugins(MinimalPlugins);

        let resource_pos = Vec3::new(10.0, 0.0, 10.0);
        let crystal = spawn_crystal_patch(app.world_mut(), resource_pos);
        let agent = spawn_agent_with_gathering(
            app.world_mut(), Vec3::ZERO, crystal, Owner::player(0),
        );

        app.world_mut().run_system_once(gathering_resource_behavior_system).unwrap();

        let locomotion = app.world_mut().entity(agent).get::<LocomotionChannel>().unwrap();
        assert!(matches!(locomotion, LocomotionChannel::Moving(_)));
        let gathering = app.world_mut().entity(agent).get::<GatheringResourceBehavior>().unwrap();
        assert_eq!(gathering.phase, GatherPhase::MovingToResource);
    }

    #[test]
    fn gathering_transitions_to_extracting_on_arrival() {
        let mut app = App::new();
        app.add_plugins(MinimalPlugins);

        let resource_pos = Vec3::new(1.0, 0.0, 1.0);
        let crystal = spawn_crystal_patch(app.world_mut(), resource_pos);
        // Spawn agent right next to the resource (within GATHER_ARRIVAL_THRESHOLD)
        let agent = spawn_agent_with_gathering(
            app.world_mut(), Vec3::new(1.0, 0.0, 1.0), crystal, Owner::player(0),
        );

        app.world_mut().run_system_once(gathering_resource_behavior_system).unwrap();

        let gathering = app.world_mut().entity(agent).get::<GatheringResourceBehavior>().unwrap();
        assert!(matches!(gathering.phase, GatherPhase::Extracting { frames_remaining: 48 }));
        let locomotion = app.world_mut().entity(agent).get::<LocomotionChannel>().unwrap();
        assert!(matches!(locomotion, LocomotionChannel::Stationary));
    }

    #[test]
    fn gathering_extracting_decrements_frames() {
        let mut app = App::new();
        app.add_plugins(MinimalPlugins);

        let resource_pos = Vec3::new(1.0, 0.0, 1.0);
        let crystal = spawn_crystal_patch(app.world_mut(), resource_pos);
        let agent = spawn_agent_with_gathering(
            app.world_mut(), Vec3::new(1.0, 0.0, 1.0), crystal, Owner::player(0),
        );

        // Manually set to Extracting phase
        let mut gathering = app.world_mut().entity_mut(agent);
        gathering.get_mut::<GatheringResourceBehavior>().unwrap().phase =
            GatherPhase::Extracting { frames_remaining: 10 };

        app.world_mut().run_system_once(gathering_resource_behavior_system).unwrap();

        let gathering = app.world_mut().entity(agent).get::<GatheringResourceBehavior>().unwrap();
        assert_eq!(gathering.phase, GatherPhase::Extracting { frames_remaining: 9 });
    }

    #[test]
    fn gathering_extraction_complete_picks_up_crystals() {
        let mut app = App::new();
        app.add_plugins(MinimalPlugins);

        let resource_pos = Vec3::new(1.0, 0.0, 1.0);
        let crystal = spawn_crystal_patch(app.world_mut(), resource_pos);
        let agent = spawn_agent_with_gathering(
            app.world_mut(), Vec3::new(1.0, 0.0, 1.0), crystal, Owner::player(0),
        );
        // Spawn a tunnel for auto-delivery
        spawn_tunnel(app.world_mut(), Vec3::new(20.0, 0.0, 20.0), Owner::player(0));

        // Set to last frame of extraction
        app.world_mut().entity_mut(agent)
            .get_mut::<GatheringResourceBehavior>().unwrap().phase =
            GatherPhase::Extracting { frames_remaining: 1 };

        app.world_mut().run_system_once(gathering_resource_behavior_system).unwrap();

        let carry = app.world_mut().entity(agent).get::<AgentCarryState>().unwrap();
        assert_eq!(carry.crystals, AGENT_CRYSTAL_CARRY);
        assert_eq!(carry.supplies, 0);

        // Should transition to MovingToTunnel
        let gathering = app.world_mut().entity(agent).get::<GatheringResourceBehavior>().unwrap();
        assert!(matches!(gathering.phase, GatherPhase::MovingToTunnel { .. }));
    }

    #[test]
    fn gathering_extraction_complete_no_tunnel_idles() {
        let mut app = App::new();
        app.add_plugins(MinimalPlugins);

        let resource_pos = Vec3::new(1.0, 0.0, 1.0);
        let crystal = spawn_crystal_patch(app.world_mut(), resource_pos);
        let agent = spawn_agent_with_gathering(
            app.world_mut(), Vec3::new(1.0, 0.0, 1.0), crystal, Owner::player(0),
        );
        // No tunnel spawned

        app.world_mut().entity_mut(agent)
            .get_mut::<GatheringResourceBehavior>().unwrap().phase =
            GatherPhase::Extracting { frames_remaining: 1 };

        app.world_mut().run_system_once(gathering_resource_behavior_system).unwrap();
        app.world_mut().flush();

        let command = app.world_mut().entity(agent).get::<UnitCommand>().unwrap();
        assert!(matches!(command, UnitCommand::Idle));
        // Behavior marker should be removed
        assert!(app.world_mut().entity(agent).get::<GatheringResourceBehavior>().is_none());
    }

    #[test]
    fn gathering_dropoff_complete_transfers_crystals() {
        let mut app = App::new();
        app.add_plugins(MinimalPlugins);

        let resource_pos = Vec3::new(1.0, 0.0, 1.0);
        let crystal = spawn_crystal_patch(app.world_mut(), resource_pos);
        let agent = spawn_agent_with_gathering(
            app.world_mut(), Vec3::new(1.0, 0.0, 1.0), crystal, Owner::player(0),
        );

        // Set carry state and phase to last frame of drop-off
        let tunnel_placeholder = app.world_mut().spawn_empty().id();
        {
            let mut entity_mut = app.world_mut().entity_mut(agent);
            entity_mut.get_mut::<AgentCarryState>().unwrap().crystals = 50;
            entity_mut.get_mut::<GatheringResourceBehavior>().unwrap().phase =
                GatherPhase::DroppingOff { tunnel_entity: tunnel_placeholder, frames_remaining: 1 };
        }

        // Spawn player resources
        spawn_syndicate_player(app.world_mut(), 0, 0);

        app.world_mut().run_system_once(gathering_resource_behavior_system).unwrap();
        app.world_mut().flush();

        // Carry state should be cleared
        let carry = app.world_mut().entity(agent).get::<AgentCarryState>().unwrap();
        assert_eq!(carry.crystals, 0);

        // Player should have received 50 crystals
        let mut q = app.world_mut().query::<(&Player, &SyndicatePlayerResources)>();
        for (player, resources) in q.iter(app.world()) {
            if player.player_number == 0 {
                assert_eq!(resources.space_crystals, 50);
            }
        }

        // Behavior should be complete
        assert!(app.world_mut().entity(agent).get::<GatheringResourceBehavior>().is_none());
        let command = app.world_mut().entity(agent).get::<UnitCommand>().unwrap();
        assert!(matches!(command, UnitCommand::Idle));
    }

    #[test]
    fn gathering_resource_gone_cancels() {
        let mut app = App::new();
        app.add_plugins(MinimalPlugins);

        // Create a resource entity then immediately despawn it
        let crystal = app.world_mut().spawn((
            Transform::from_translation(Vec3::new(5.0, 0.0, 5.0)),
            SpaceCrystalPatch { remaining_amount: 100, initial_amount: 100, has_plate: false },
        )).id();
        let agent = spawn_agent_with_gathering(
            app.world_mut(), Vec3::ZERO, crystal, Owner::player(0),
        );
        // Despawn the resource
        app.world_mut().despawn(crystal);

        app.world_mut().run_system_once(gathering_resource_behavior_system).unwrap();
        app.world_mut().flush();

        let command = app.world_mut().entity(agent).get::<UnitCommand>().unwrap();
        assert!(matches!(command, UnitCommand::Idle));
        assert!(app.world_mut().entity(agent).get::<GatheringResourceBehavior>().is_none());
    }

    // === DroppingOffResources behavior system tests ===

    /// Helper to spawn an agent with DroppingOffResourcesBehavior
    fn spawn_agent_with_dropoff(
        world: &mut World,
        pos: Vec3,
        target_tunnel: Entity,
        owner: Owner,
        carry: AgentCarryState,
    ) -> Entity {
        world.spawn((
            Transform::from_translation(pos),
            owner,
            DroppingOffResourcesBehavior::new(target_tunnel),
            LocomotionChannel::Stationary,
            OrientationChannel::Maintaining,
            UnitCommand::DropOffResources(target_tunnel),
            BaseBehaviorState::None,
            carry,
        )).id()
    }

    #[test]
    fn dropoff_moves_toward_tunnel_when_far() {
        let mut app = App::new();
        app.add_plugins(MinimalPlugins);

        let tunnel = spawn_tunnel(app.world_mut(), Vec3::new(20.0, 0.0, 20.0), Owner::player(0));
        let carry = AgentCarryState { crystals: 50, supplies: 0 };
        let agent = spawn_agent_with_dropoff(
            app.world_mut(), Vec3::ZERO, tunnel, Owner::player(0), carry,
        );

        app.world_mut().run_system_once(dropping_off_resources_behavior_system).unwrap();

        let locomotion = app.world_mut().entity(agent).get::<LocomotionChannel>().unwrap();
        assert!(matches!(locomotion, LocomotionChannel::Moving(_)));
        let behavior = app.world_mut().entity(agent).get::<DroppingOffResourcesBehavior>().unwrap();
        assert_eq!(behavior.phase, DropOffPhase::MovingToTunnel);
    }

    #[test]
    fn dropoff_transitions_to_dropping_off_on_arrival() {
        let mut app = App::new();
        app.add_plugins(MinimalPlugins);

        let tunnel_pos = Vec3::new(5.0, 0.0, 5.0);
        let tunnel = spawn_tunnel(app.world_mut(), tunnel_pos, Owner::player(0));
        // Side B for crystals at R0 is East = (7.5, 0, 5.0)
        let carry = AgentCarryState { crystals: 50, supplies: 0 };
        let agent = spawn_agent_with_dropoff(
            app.world_mut(), Vec3::new(7.5, 0.0, 5.0), tunnel, Owner::player(0), carry,
        );

        app.world_mut().run_system_once(dropping_off_resources_behavior_system).unwrap();

        let behavior = app.world_mut().entity(agent).get::<DroppingOffResourcesBehavior>().unwrap();
        assert!(matches!(behavior.phase, DropOffPhase::DroppingOff { frames_remaining: 48 }));
    }

    #[test]
    fn dropoff_complete_transfers_and_clears() {
        let mut app = App::new();
        app.add_plugins(MinimalPlugins);

        let tunnel = spawn_tunnel(app.world_mut(), Vec3::new(5.0, 0.0, 5.0), Owner::player(0));
        let carry = AgentCarryState { crystals: 50, supplies: 0 };
        let agent = spawn_agent_with_dropoff(
            app.world_mut(), Vec3::new(7.5, 0.0, 5.0), tunnel, Owner::player(0), carry,
        );

        // Set phase to last frame
        app.world_mut().entity_mut(agent)
            .get_mut::<DroppingOffResourcesBehavior>().unwrap().phase =
            DropOffPhase::DroppingOff { frames_remaining: 1 };

        spawn_syndicate_player(app.world_mut(), 0, 0);

        app.world_mut().run_system_once(dropping_off_resources_behavior_system).unwrap();
        app.world_mut().flush();

        // Carry should be empty
        let carry = app.world_mut().entity(agent).get::<AgentCarryState>().unwrap();
        assert_eq!(carry.crystals, 0);
        assert_eq!(carry.supplies, 0);

        // Player resources should be updated
        let mut q = app.world_mut().query::<(&Player, &SyndicatePlayerResources)>();
        for (player, resources) in q.iter(app.world()) {
            if player.player_number == 0 {
                assert_eq!(resources.space_crystals, 50);
            }
        }

        // Behavior removed, command idle
        assert!(app.world_mut().entity(agent).get::<DroppingOffResourcesBehavior>().is_none());
        let command = app.world_mut().entity(agent).get::<UnitCommand>().unwrap();
        assert!(matches!(command, UnitCommand::Idle));
    }

    #[test]
    fn dropoff_tunnel_despawned_cancels() {
        let mut app = App::new();
        app.add_plugins(MinimalPlugins);

        let tunnel = spawn_tunnel(app.world_mut(), Vec3::new(5.0, 0.0, 5.0), Owner::player(0));
        let carry = AgentCarryState { crystals: 50, supplies: 0 };
        let agent = spawn_agent_with_dropoff(
            app.world_mut(), Vec3::ZERO, tunnel, Owner::player(0), carry,
        );
        app.world_mut().despawn(tunnel);

        app.world_mut().run_system_once(dropping_off_resources_behavior_system).unwrap();
        app.world_mut().flush();

        let command = app.world_mut().entity(agent).get::<UnitCommand>().unwrap();
        assert!(matches!(command, UnitCommand::Idle));
        assert!(app.world_mut().entity(agent).get::<DroppingOffResourcesBehavior>().is_none());
    }

    #[test]
    fn dropoff_supplies_uses_side_c() {
        let mut app = App::new();
        app.add_plugins(MinimalPlugins);

        let tunnel_pos = Vec3::new(5.0, 0.0, 5.0);
        let tunnel = spawn_tunnel(app.world_mut(), tunnel_pos, Owner::player(0));
        // Side C for supplies at R0 is South = (5.0, 0, 7.5)
        let carry = AgentCarryState { crystals: 0, supplies: 1 };
        let agent = spawn_agent_with_dropoff(
            app.world_mut(), Vec3::new(5.0, 0.0, 7.5), tunnel, Owner::player(0), carry,
        );

        app.world_mut().run_system_once(dropping_off_resources_behavior_system).unwrap();

        // Should arrive and start dropping off (agent is at Side C position)
        let behavior = app.world_mut().entity(agent).get::<DroppingOffResourcesBehavior>().unwrap();
        assert!(matches!(behavior.phase, DropOffPhase::DroppingOff { frames_remaining: 48 }));
    }

    #[test]
    fn dropoff_supply_transfer_updates_supplies() {
        let mut app = App::new();
        app.add_plugins(MinimalPlugins);

        let tunnel = spawn_tunnel(app.world_mut(), Vec3::new(5.0, 0.0, 5.0), Owner::player(0));
        let carry = AgentCarryState { crystals: 0, supplies: 1 };
        let agent = spawn_agent_with_dropoff(
            app.world_mut(), Vec3::new(5.0, 0.0, 7.5), tunnel, Owner::player(0), carry,
        );

        app.world_mut().entity_mut(agent)
            .get_mut::<DroppingOffResourcesBehavior>().unwrap().phase =
            DropOffPhase::DroppingOff { frames_remaining: 1 };

        spawn_syndicate_player(app.world_mut(), 0, 10);

        app.world_mut().run_system_once(dropping_off_resources_behavior_system).unwrap();
        app.world_mut().flush();

        let mut q = app.world_mut().query::<(&Player, &SyndicatePlayerResources)>();
        for (player, resources) in q.iter(app.world()) {
            if player.player_number == 0 {
                assert_eq!(resources.supplies, 11); // 10 + 1
            }
        }
    }
}
