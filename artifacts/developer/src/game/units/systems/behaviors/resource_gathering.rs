#![allow(dead_code)]
use bevy::prelude::*;
use crate::game::units::types::state::behavior::{
    BaseBehaviorState, GatheringResourceBehavior, GatherPhase,
    DroppingOffResourcesBehavior, DropOffPhase,
    LocomotionChannel, OrientationChannel,
};
use crate::game::units::types::state::commands::UnitCommand;
use crate::game::units::types::state::types::AgentCarryState;
use crate::game::entities::factions::syndicate::units::agent::{
    AGENT_MINING_DURATION, AGENT_PICKUP_DURATION, AGENT_DROPOFF_DURATION,
    AGENT_CRYSTAL_CARRY, AGENT_SUPPLY_CARRY,
};
use crate::game::types::{StructureInstance, TunnelState};
use crate::game::types::factions::{Player, SyndicatePlayerResources};
use crate::game::world::types::SpaceCrystalPatch;
use crate::types::{Owner, SymmetryTypeEnum};

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

#[cfg(test)]
mod tests {
    use super::*;
    use bevy::ecs::system::RunSystemOnce;
    use crate::types::Owner;
    use crate::game::types::{StructureInstance, TunnelState};
    
    use crate::game::types::factions::{Player, SyndicatePlayerResources};
    use crate::game::world::types::SpaceCrystalPatch;
    
    use crate::game::units::types::state::behavior::*;
    use crate::game::units::types::state::commands::UnitCommand;
    use crate::game::units::types::state::types::AgentCarryState;
    
    

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
