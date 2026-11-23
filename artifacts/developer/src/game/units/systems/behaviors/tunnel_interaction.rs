#![allow(dead_code)]
use bevy::prelude::*;
use crate::game::units::types::movement::Velocity;
use crate::game::units::types::state::behavior::{
    EnteringTunnelBehavior, InTunnelNetwork, EnteringArmoryBehavior,
    LocomotionChannel, OrientationChannel,
};
use crate::game::units::types::state::commands::UnitCommand;
use crate::game::types::{ObjectInstance, TunnelState};
use crate::game::types::structures::ArmoryState;
use crate::game::types::cults_structure_stats::ARMORY_INTERNAL_RECRUIT_CAPACITY;
use crate::game::units::utils::can_enter_tunnel;
use crate::game::entities::types::UnitDefinition;
use crate::game::entities::factions::syndicate::units::agent::SyndicateAgentDef;
use crate::game::entities::factions::syndicate::units::guard::SyndicateGuardDef;
use crate::types::{Owner, Unit, UnitBaseEnum, ObjectEnum};


/// Distance threshold for considering arrival at Side A position (in world units)
const TUNNEL_ARRIVAL_THRESHOLD: f32 = 0.5;

/// Dispatch system for UnitCommand::Enter.
///
/// Detects units with `UnitCommand::Enter(tunnel_entity)` that don't yet have
/// an `EnteringTunnelBehavior` marker. Validates the Enter command using
/// `can_enter_tunnel()` (faction, owner, tier checks). If valid, inserts
/// `EnteringTunnelBehavior` on the unit. If invalid, resets the command to Idle.
///
/// Schedule: Update, Phase 2 (before entering_tunnel_behavior_system)
pub fn enter_command_dispatch_system(
    mut commands: Commands,
    mut units: Query<
        (Entity, &mut UnitCommand, &ObjectInstance, &Owner),
        (With<Unit>, Without<EnteringTunnelBehavior>, Without<InTunnelNetwork>),
    >,
    tunnels: Query<(&TunnelState, &Owner), With<TunnelState>>,
) {
    for (entity, mut command, obj, unit_owner) in &mut units {
        let tunnel_entity = match &*command {
            UnitCommand::Enter(te) => *te,
            _ => continue,
        };

        // Look up tunnel — cancel if it no longer exists
        let (tunnel_state, tunnel_owner) = match tunnels.get(tunnel_entity) {
            Ok(result) => result,
            Err(_) => {
                warn!("Enter command target entity does not exist or has no TunnelState");
                *command = UnitCommand::Idle;
                continue;
            }
        };

        // Determine if unit is Syndicate and get its UnitBaseEnum
        let (is_syndicate, unit_base) = match obj.object_type {
            ObjectEnum::SyndicateAgent => (true, SyndicateAgentDef::unit_stats().unit_base),
            ObjectEnum::SyndicateGuard => (true, SyndicateGuardDef::unit_stats().unit_base),
            _ => (false, UnitBaseEnum::LightInfantry), // placeholder; will fail validation
        };

        // Validate
        match can_enter_tunnel(
            is_syndicate,
            unit_owner.player_number(),
            tunnel_owner.player_number(),
            &unit_base,
            &tunnel_state.tier,
        ) {
            Ok(()) => {
                commands.entity(entity).insert(EnteringTunnelBehavior::new(tunnel_entity));
            }
            Err(reason) => {
                warn!("Enter command rejected: {}", reason);
                *command = UnitCommand::Idle;
            }
        }
    }
}

/// Process EnteringTunnel behavior.
///
/// Units with the `EnteringTunnelBehavior` marker move toward the target Tunnel's
/// position (Side A approximated as the tunnel's transform position).
/// On arrival, the unit is placed into the tunnel network: `InTunnelNetwork` is inserted,
/// visibility is set to Hidden, and movement state is cleared. The entity is NOT despawned,
/// so it remains queryable for ejection by the production/ejection system.
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
        &Owner,
        &mut Visibility,
    )>,
    tunnels: Query<&Transform, With<TunnelState>>,
) {
    for (entity, transform, entering, mut locomotion, mut orientation, owner, mut visibility) in &mut units {
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
            let owner_player = owner.player_number().unwrap_or(0);
            commands.entity(entity).insert(InTunnelNetwork { owner_player });
            *visibility = Visibility::Hidden;
            // Clear movement state
            *locomotion = LocomotionChannel::Stationary;
            *orientation = OrientationChannel::Maintaining;
            commands.entity(entity).insert(Velocity(Vec3::ZERO));
            // Remove behavior marker
            commands.entity(entity).remove::<EnteringTunnelBehavior>();
            continue;
        }

        // Write movement channels to move toward Side A
        *locomotion = LocomotionChannel::Moving(vec![side_a_pos]);
        *orientation = OrientationChannel::Turning(side_a_pos);
    }
}

/// Distance threshold for considering arrival at Armory entrance (in world units)
const ARMORY_ARRIVAL_THRESHOLD: f32 = 0.5;

/// Dispatch system for the EnterArmory command.
/// Validates that the unit is a CultsRecruit, the armory exists, has same owner,
/// and has capacity. If valid, inserts `EnteringArmoryBehavior`. Otherwise resets to Idle.
///
/// Schedule: Update, Phase 2 (before entering_armory_behavior_system)
pub fn enter_armory_dispatch_system(
    mut commands: Commands,
    mut units: Query<
        (Entity, &mut UnitCommand, &ObjectInstance, &Owner),
        (With<Unit>, Without<EnteringArmoryBehavior>),
    >,
    armories: Query<(&ArmoryState, &Owner), With<ArmoryState>>,
) {
    for (entity, mut command, obj, unit_owner) in &mut units {
        let armory_entity = match &*command {
            UnitCommand::EnterArmory(ae) => *ae,
            _ => continue,
        };
        // Only CultsRecruit can enter
        if obj.object_type != ObjectEnum::CultsRecruit {
            *command = UnitCommand::Idle;
            continue;
        }
        // Validate armory exists, is owned by same player, and has capacity
        let (armory_state, armory_owner) = match armories.get(armory_entity) {
            Ok(r) => r,
            Err(_) => { *command = UnitCommand::Idle; continue; }
        };
        if armory_owner.player_number() != unit_owner.player_number() {
            *command = UnitCommand::Idle;
            continue;
        }
        if armory_state.stored_recruits.len() >= ARMORY_INTERNAL_RECRUIT_CAPACITY {
            *command = UnitCommand::Idle;
            continue;
        }
        commands.entity(entity).insert(EnteringArmoryBehavior::new(armory_entity));
    }
}

/// Process EnteringArmory behavior.
///
/// Units with the `EnteringArmoryBehavior` marker move toward the target Armory's
/// position (entrance approximated as the armory's transform position).
/// On arrival, the unit is stored in `ArmoryState.stored_recruits`, hidden,
/// and movement is cleared. The entity is NOT despawned.
///
/// Schedule: Update (same as other behavior systems)
pub fn entering_armory_behavior_system(
    mut commands: Commands,
    mut units: Query<(
        Entity,
        &Transform,
        &mut EnteringArmoryBehavior,
        &mut LocomotionChannel,
        &mut OrientationChannel,
        &mut Visibility,
    )>,
    mut armories: Query<(&Transform, &mut ArmoryState), With<ArmoryState>>,
) {
    for (entity, transform, entering, mut locomotion, mut orientation, mut visibility) in &mut units {
        let (armory_transform, mut armory_state) = match armories.get_mut(entering.target_armory) {
            Ok(r) => r,
            Err(_) => {
                // Armory no longer exists — cancel behavior
                *locomotion = LocomotionChannel::Stopping;
                *orientation = OrientationChannel::Maintaining;
                commands.entity(entity).remove::<EnteringArmoryBehavior>();
                continue;
            }
        };
        // Re-check capacity (may have filled while walking)
        if armory_state.stored_recruits.len() >= ARMORY_INTERNAL_RECRUIT_CAPACITY {
            *locomotion = LocomotionChannel::Stopping;
            *orientation = OrientationChannel::Maintaining;
            commands.entity(entity).remove::<EnteringArmoryBehavior>();
            commands.entity(entity).insert(UnitCommand::Idle);
            continue;
        }
        // Entrance is Side A (approximated as armory center, like tunnel)
        let entrance_pos = armory_transform.translation;
        let distance = Vec3::new(
            transform.translation.x - entrance_pos.x, 0.0,
            transform.translation.z - entrance_pos.z,
        ).length();

        if distance < ARMORY_ARRIVAL_THRESHOLD {
            // Unit has arrived — enter the armory
            armory_state.stored_recruits.push(entity);
            *visibility = Visibility::Hidden;
            *locomotion = LocomotionChannel::Stationary;
            *orientation = OrientationChannel::Maintaining;
            commands.entity(entity).insert(Velocity(Vec3::ZERO));
            commands.entity(entity).remove::<EnteringArmoryBehavior>();
        } else {
            // Move toward entrance
            *locomotion = LocomotionChannel::Moving(vec![entrance_pos]);
            *orientation = OrientationChannel::Turning(entrance_pos);
        }
    }
}


#[cfg(test)]
mod tests {
    use super::*;
    use bevy::ecs::system::RunSystemOnce;
    use crate::types::{Owner, Unit};
    use crate::game::types::{ObjectInstance, StructureInstance, TunnelState};
    use crate::game::types::structures::ArmoryState;
    
    
    
    use crate::game::units::types::state::behavior::*;
    use crate::game::units::types::state::commands::UnitCommand;
    
    use crate::game::units::types::movement::Velocity;
    

    // === EnteringTunnel behavior + dispatch tests ===

    /// Helper to spawn an agent entity for tunnel entering tests
    fn spawn_agent_for_tunnel_test(world: &mut World, pos: Vec3, tunnel_entity: Entity) -> Entity {
        world.spawn((
            Transform::from_translation(pos),
            Unit,
            ObjectInstance {
                object_type: ObjectEnum::SyndicateAgent,
                hp: Some(75.0),
                max_hp: Some(75.0),
            },
            Owner(Some(0)),
            UnitCommand::Enter(tunnel_entity),
            LocomotionChannel::Stationary,
            OrientationChannel::Maintaining,
            Visibility::Visible,
        )).id()
    }

    /// Helper to spawn a tunnel entity for tests
    fn spawn_tunnel_for_test(world: &mut World, pos: Vec3, owner: u8) -> Entity {
        world.spawn((
            Transform::from_translation(pos),
            TunnelState::new(crate::game::types::TunnelTier::Tier1),
            Owner(Some(owner)),
            StructureInstance::default(),
        )).id()
    }

    #[test]
    fn enter_dispatch_valid_agent_inserts_behavior_marker() {
        let mut world = World::new();
        let tunnel = spawn_tunnel_for_test(&mut world, Vec3::new(10.0, 0.5, 10.0), 0);
        let agent = spawn_agent_for_tunnel_test(&mut world, Vec3::ZERO, tunnel);

        world.run_system_once(enter_command_dispatch_system).unwrap();

        assert!(world.entity(agent).get::<EnteringTunnelBehavior>().is_some());
        let behavior = world.entity(agent).get::<EnteringTunnelBehavior>().unwrap();
        assert_eq!(behavior.target_tunnel, tunnel);
    }

    #[test]
    fn enter_dispatch_non_syndicate_rejects_command() {
        let mut world = World::new();
        let tunnel = spawn_tunnel_for_test(&mut world, Vec3::new(10.0, 0.5, 10.0), 0);
        // Spawn a non-syndicate unit (Peacekeeper) with Enter command
        let unit = world.spawn((
            Transform::from_xyz(0.0, 0.0, 0.0),
            Unit,
            ObjectInstance {
                object_type: ObjectEnum::Peacekeeper,
                hp: Some(100.0),
                max_hp: Some(100.0),
            },
            Owner(Some(0)),
            UnitCommand::Enter(tunnel),
            LocomotionChannel::Stationary,
            OrientationChannel::Maintaining,
            Visibility::Visible,
        )).id();

        world.run_system_once(enter_command_dispatch_system).unwrap();

        // Should NOT have behavior marker
        assert!(world.entity(unit).get::<EnteringTunnelBehavior>().is_none());
        // Command should be reset to Idle
        let cmd = world.entity(unit).get::<UnitCommand>().unwrap();
        assert!(matches!(cmd, UnitCommand::Idle));
    }

    #[test]
    fn enter_dispatch_wrong_owner_rejects_command() {
        let mut world = World::new();
        // Tunnel owned by player 1, agent owned by player 0
        let tunnel = spawn_tunnel_for_test(&mut world, Vec3::new(10.0, 0.5, 10.0), 1);
        let agent = spawn_agent_for_tunnel_test(&mut world, Vec3::ZERO, tunnel);

        world.run_system_once(enter_command_dispatch_system).unwrap();

        assert!(world.entity(agent).get::<EnteringTunnelBehavior>().is_none());
        let cmd = world.entity(agent).get::<UnitCommand>().unwrap();
        assert!(matches!(cmd, UnitCommand::Idle));
    }

    #[test]
    fn enter_dispatch_tunnel_not_found_rejects_command() {
        let mut world = World::new();
        // Use a fake entity that doesn't exist as a tunnel
        let fake_tunnel = Entity::from_raw_u32(9999).unwrap();
        let agent = spawn_agent_for_tunnel_test(&mut world, Vec3::ZERO, fake_tunnel);

        world.run_system_once(enter_command_dispatch_system).unwrap();

        assert!(world.entity(agent).get::<EnteringTunnelBehavior>().is_none());
        let cmd = world.entity(agent).get::<UnitCommand>().unwrap();
        assert!(matches!(cmd, UnitCommand::Idle));
    }

    #[test]
    fn entering_tunnel_far_away_sets_moving() {
        let mut world = World::new();
        let tunnel = spawn_tunnel_for_test(&mut world, Vec3::new(10.0, 0.5, 10.0), 0);
        let agent = world.spawn((
            Transform::from_xyz(0.0, 0.5, 0.0), // Far from tunnel
            EnteringTunnelBehavior::new(tunnel),
            LocomotionChannel::Stationary,
            OrientationChannel::Maintaining,
            Owner(Some(0)),
            Visibility::Visible,
        )).id();

        world.run_system_once(entering_tunnel_behavior_system).unwrap();

        let locomotion = world.entity(agent).get::<LocomotionChannel>().unwrap();
        assert!(matches!(locomotion, LocomotionChannel::Moving(_)));
        let orientation = world.entity(agent).get::<OrientationChannel>().unwrap();
        assert!(matches!(orientation, OrientationChannel::Turning(_)));
        // Entity should still exist and NOT be in tunnel network
        assert!(world.entity(agent).get::<InTunnelNetwork>().is_none());
        // Behavior marker should still be present
        assert!(world.entity(agent).get::<EnteringTunnelBehavior>().is_some());
    }

    #[test]
    fn entering_tunnel_at_arrival_enters_network() {
        let mut world = World::new();
        let tunnel_pos = Vec3::new(5.0, 0.5, 5.0);
        let tunnel = spawn_tunnel_for_test(&mut world, tunnel_pos, 0);
        // Spawn agent very close to tunnel (within TUNNEL_ARRIVAL_THRESHOLD)
        let agent = world.spawn((
            Transform::from_translation(tunnel_pos + Vec3::new(0.1, 0.0, 0.1)),
            EnteringTunnelBehavior::new(tunnel),
            LocomotionChannel::Stationary,
            OrientationChannel::Maintaining,
            Owner(Some(0)),
            Visibility::Visible,
        )).id();

        world.run_system_once(entering_tunnel_behavior_system).unwrap();

        // Entity should still exist (not despawned)
        assert!(world.get_entity(agent).is_ok());
        // Should have InTunnelNetwork
        let network = world.entity(agent).get::<InTunnelNetwork>().unwrap();
        assert_eq!(network.owner_player, 0);
        // Should be hidden
        let vis = world.entity(agent).get::<Visibility>().unwrap();
        assert_eq!(*vis, Visibility::Hidden);
        // Channels should be stationary/maintaining
        let locomotion = world.entity(agent).get::<LocomotionChannel>().unwrap();
        assert!(matches!(locomotion, LocomotionChannel::Stationary));
        let orientation = world.entity(agent).get::<OrientationChannel>().unwrap();
        assert!(matches!(orientation, OrientationChannel::Maintaining));
        // Behavior marker should be removed
        assert!(world.entity(agent).get::<EnteringTunnelBehavior>().is_none());
    }

    #[test]
    fn entering_tunnel_missing_tunnel_cancels() {
        let mut world = World::new();
        let fake_tunnel = Entity::from_raw_u32(9999).unwrap();
        let agent = world.spawn((
            Transform::from_xyz(5.0, 0.5, 5.0),
            EnteringTunnelBehavior::new(fake_tunnel),
            LocomotionChannel::Moving(vec![Vec3::ZERO]),
            OrientationChannel::Turning(Vec3::ZERO),
            Owner(Some(0)),
            Visibility::Visible,
        )).id();

        world.run_system_once(entering_tunnel_behavior_system).unwrap();

        // Behavior marker should be removed
        assert!(world.entity(agent).get::<EnteringTunnelBehavior>().is_none());
        // Channels should be stopping/maintaining
        let locomotion = world.entity(agent).get::<LocomotionChannel>().unwrap();
        assert!(matches!(locomotion, LocomotionChannel::Stopping));
        let orientation = world.entity(agent).get::<OrientationChannel>().unwrap();
        assert!(matches!(orientation, OrientationChannel::Maintaining));
        // Should NOT be in tunnel network
        assert!(world.entity(agent).get::<InTunnelNetwork>().is_none());
    }

    // === EnterArmory dispatch tests ===

    fn spawn_test_armory(world: &mut World, pos: Vec3, owner_player: u8, stored_count: usize) -> Entity {
        let mut stored_recruits = Vec::new();
        // Create dummy entities to fill the stored list
        for _ in 0..stored_count {
            let dummy = world.spawn_empty().id();
            stored_recruits.push(dummy);
        }
        world.spawn((
            Transform::from_translation(pos),
            ArmoryState {
                stored_recruits,
                training_queue: None,
                training_progress: 0,
                rally_point: None,
            },
            Owner::player(owner_player),
            ObjectInstance::destructible(ObjectEnum::CultsArmory, 300.0),
        )).id()
    }

    fn spawn_test_recruit_for_armory(world: &mut World, pos: Vec3, owner_player: u8) -> Entity {
        world.spawn((
            Transform::from_translation(pos),
            Unit,
            ObjectInstance::destructible(ObjectEnum::CultsRecruit, 50.0),
            Owner::player(owner_player),
            UnitCommand::Idle,
            LocomotionChannel::Stationary,
            OrientationChannel::Maintaining,
            Velocity(Vec3::ZERO),
            Visibility::Inherited,
        )).id()
    }

    #[test]
    fn enter_armory_dispatch_inserts_behavior_for_recruit() {
        let mut app = App::new();
        let armory = spawn_test_armory(app.world_mut(), Vec3::new(10.0, 0.0, 10.0), 0, 0);
        let recruit = spawn_test_recruit_for_armory(app.world_mut(), Vec3::new(5.0, 0.0, 5.0), 0);
        app.world_mut().entity_mut(recruit).insert(UnitCommand::EnterArmory(armory));

        app.world_mut().run_system_once(enter_armory_dispatch_system).unwrap();

        assert!(app.world().get::<EnteringArmoryBehavior>(recruit).is_some(),
            "Dispatch should insert EnteringArmoryBehavior");
    }

    #[test]
    fn enter_armory_dispatch_rejects_non_recruit() {
        let mut app = App::new();
        let armory = spawn_test_armory(app.world_mut(), Vec3::new(10.0, 0.0, 10.0), 0, 0);
        // Spawn a Guard instead of a Recruit
        let guard = app.world_mut().spawn((
            Transform::from_translation(Vec3::new(5.0, 0.0, 5.0)),
            Unit,
            ObjectInstance::destructible(ObjectEnum::SyndicateGuard, 100.0),
            Owner::player(0),
            UnitCommand::EnterArmory(armory),
        )).id();

        app.world_mut().run_system_once(enter_armory_dispatch_system).unwrap();

        assert!(app.world().get::<EnteringArmoryBehavior>(guard).is_none(),
            "Non-recruit should not get behavior");
        assert!(matches!(app.world().get::<UnitCommand>(guard).unwrap(), UnitCommand::Idle));
    }

    #[test]
    fn enter_armory_dispatch_rejects_full_armory() {
        let mut app = App::new();
        let armory = spawn_test_armory(app.world_mut(), Vec3::new(10.0, 0.0, 10.0), 0, ARMORY_INTERNAL_RECRUIT_CAPACITY);
        let recruit = spawn_test_recruit_for_armory(app.world_mut(), Vec3::new(5.0, 0.0, 5.0), 0);
        app.world_mut().entity_mut(recruit).insert(UnitCommand::EnterArmory(armory));

        app.world_mut().run_system_once(enter_armory_dispatch_system).unwrap();

        assert!(app.world().get::<EnteringArmoryBehavior>(recruit).is_none(),
            "Full armory should reject entry");
        assert!(matches!(app.world().get::<UnitCommand>(recruit).unwrap(), UnitCommand::Idle));
    }

    #[test]
    fn enter_armory_dispatch_rejects_wrong_owner() {
        let mut app = App::new();
        let armory = spawn_test_armory(app.world_mut(), Vec3::new(10.0, 0.0, 10.0), 1, 0); // player 1
        let recruit = spawn_test_recruit_for_armory(app.world_mut(), Vec3::new(5.0, 0.0, 5.0), 0); // player 0
        app.world_mut().entity_mut(recruit).insert(UnitCommand::EnterArmory(armory));

        app.world_mut().run_system_once(enter_armory_dispatch_system).unwrap();

        assert!(app.world().get::<EnteringArmoryBehavior>(recruit).is_none(),
            "Wrong owner should reject entry");
        assert!(matches!(app.world().get::<UnitCommand>(recruit).unwrap(), UnitCommand::Idle));
    }

    // === EnteringArmory behavior tests ===

    #[test]
    fn entering_armory_behavior_moves_toward_armory() {
        let mut app = App::new();
        let armory = spawn_test_armory(app.world_mut(), Vec3::new(10.0, 0.0, 10.0), 0, 0);
        let recruit = spawn_test_recruit_for_armory(app.world_mut(), Vec3::new(5.0, 0.0, 5.0), 0);
        app.world_mut().entity_mut(recruit).insert(EnteringArmoryBehavior::new(armory));

        app.world_mut().run_system_once(entering_armory_behavior_system).unwrap();

        let loco = app.world().get::<LocomotionChannel>(recruit).unwrap();
        assert!(matches!(loco, LocomotionChannel::Moving(_)),
            "Should be moving toward armory");
    }

    #[test]
    fn entering_armory_behavior_enters_on_arrival() {
        let mut app = App::new();
        let armory_pos = Vec3::new(10.0, 0.0, 10.0);
        let armory = spawn_test_armory(app.world_mut(), armory_pos, 0, 0);
        // Place recruit within arrival threshold
        let recruit = spawn_test_recruit_for_armory(app.world_mut(), Vec3::new(10.0, 0.0, 10.0), 0);
        app.world_mut().entity_mut(recruit).insert(EnteringArmoryBehavior::new(armory));

        app.world_mut().run_system_once(entering_armory_behavior_system).unwrap();

        // Recruit should be stored in armory
        let armory_state = app.world().get::<ArmoryState>(armory).unwrap();
        assert_eq!(armory_state.stored_recruits.len(), 1);
        assert_eq!(armory_state.stored_recruits[0], recruit);
        // Recruit should be hidden
        assert_eq!(*app.world().get::<Visibility>(recruit).unwrap(), Visibility::Hidden);
        // Behavior marker should be removed
        assert!(app.world().get::<EnteringArmoryBehavior>(recruit).is_none());
    }

    #[test]
    fn entering_armory_behavior_cancels_when_full() {
        let mut app = App::new();
        let armory = spawn_test_armory(app.world_mut(), Vec3::new(10.0, 0.0, 10.0), 0, ARMORY_INTERNAL_RECRUIT_CAPACITY);
        let recruit = spawn_test_recruit_for_armory(app.world_mut(), Vec3::new(10.0, 0.0, 10.0), 0);
        app.world_mut().entity_mut(recruit).insert(EnteringArmoryBehavior::new(armory));

        app.world_mut().run_system_once(entering_armory_behavior_system).unwrap();

        // Behavior should be removed
        assert!(app.world().get::<EnteringArmoryBehavior>(recruit).is_none());
        // Should get Idle command
        assert!(matches!(app.world().get::<UnitCommand>(recruit).unwrap(), UnitCommand::Idle));
        // Should NOT be stored
        let armory_state = app.world().get::<ArmoryState>(armory).unwrap();
        assert_eq!(armory_state.stored_recruits.len(), ARMORY_INTERNAL_RECRUIT_CAPACITY);
    }

    #[test]
    fn entering_armory_behavior_cancels_when_armory_despawned() {
        let mut app = App::new();
        let armory = spawn_test_armory(app.world_mut(), Vec3::new(10.0, 0.0, 10.0), 0, 0);
        let recruit = spawn_test_recruit_for_armory(app.world_mut(), Vec3::new(5.0, 0.0, 5.0), 0);
        app.world_mut().entity_mut(recruit).insert(EnteringArmoryBehavior::new(armory));
        // Despawn the armory
        app.world_mut().despawn(armory);

        app.world_mut().run_system_once(entering_armory_behavior_system).unwrap();

        assert!(app.world().get::<EnteringArmoryBehavior>(recruit).is_none(),
            "Behavior should be removed when armory is gone");
    }
}
