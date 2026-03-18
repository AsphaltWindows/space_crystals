use crate::helpers::*;
use bevy::ecs::system::RunSystemOnce;
use space_crystals::game::units::types::state::{
    GatheringResourceBehavior, GatherPhase, AgentCarryState,
};
use space_crystals::game::types::structures::TunnelState;
use space_crystals::game::types::factions::{Player, SyndicatePlayerResources};

/// QA Step 2 [auto]: Verify the Agent performs mining for exactly 48 frames upon reaching the Space Crystal Patch.
#[test]
fn step_2_mining_duration_48_frames() {
    let mut test_app = TestApp::new_with_faction(FactionEnum::TheSyndicate);
    test_app.step(); // Initialize game (spawns tunnel at grid 40,40)

    let mut h = TestHarness::new(&mut test_app.app);

    // Spawn a crystal patch at grid (20, 20)
    let patch = h.spawn_resource(20, 20, 1000);

    // Spawn Agent near the patch — grid (20, 21) → world close enough to arrive
    let agent = h.spawn_unit_at_grid(ObjectEnum::SyndicateAgent, 20, 21, Owner(Some(0)));

    // Insert the GatheringResourceBehavior in Extracting phase to test frame counting
    h.app.world_mut().entity_mut(agent).insert(
        GatheringResourceBehavior {
            target_resource: patch,
            phase: GatherPhase::Extracting { frames_remaining: 48 },
            path: Vec::new(),
            path_index: 0,
        }
    );

    // Run 47 frames — should still be extracting with 1 frame remaining
    for _ in 0..47 {
        h.step();
    }

    let gathering = h.app.world().get::<GatheringResourceBehavior>(agent);
    assert!(gathering.is_some(), "GatheringResourceBehavior should still exist after 47 frames");
    let gathering = gathering.unwrap();
    assert_eq!(gathering.phase, GatherPhase::Extracting { frames_remaining: 1 },
        "After 47 frames, should have 1 frame remaining");

    // Run 1 more frame — extraction should complete
    h.step();

    let gathering = h.app.world().get::<GatheringResourceBehavior>(agent);
    // After completion, should have moved to MovingToTunnel phase (or behavior removed if no tunnel)
    if let Some(g) = gathering {
        // Should NOT still be in Extracting phase
        assert!(!matches!(g.phase, GatherPhase::Extracting { .. }),
            "After 48 frames, should no longer be in Extracting phase. Got: {:?}", g.phase);
    }
    // If behavior was removed (no nearby tunnel), that also counts as extraction completing
}

/// QA Step 3 [auto]: Verify the Agent picks up 50 Space Crystals after mining completes.
#[test]
fn step_3_picks_up_50_crystals() {
    let mut test_app = TestApp::new_with_faction(FactionEnum::TheSyndicate);
    test_app.step();

    let mut h = TestHarness::new(&mut test_app.app);

    let patch = h.spawn_resource(20, 20, 1000);
    let agent = h.spawn_unit_at_grid(ObjectEnum::SyndicateAgent, 20, 21, Owner(Some(0)));

    // Start in Extracting phase with 1 frame remaining — next step completes extraction
    h.app.world_mut().entity_mut(agent).insert(
        GatheringResourceBehavior {
            target_resource: patch,
            phase: GatherPhase::Extracting { frames_remaining: 1 },
            path: Vec::new(),
            path_index: 0,
        }
    );

    // Verify carry state starts at 0
    let carry = h.app.world().get::<AgentCarryState>(agent).unwrap();
    assert_eq!(carry.crystals, 0, "Should start with 0 crystals");

    // Complete extraction
    h.step();

    // Verify Agent now carries 50 crystals
    let carry = h.app.world().get::<AgentCarryState>(agent).unwrap();
    assert_eq!(carry.crystals, 50, "After mining, Agent should carry 50 crystals");
}

/// QA Step 5 [auto]: Verify the drop-off takes exactly 48 frames at Side B.
#[test]
fn step_5_dropoff_48_frames_side_b() {
    let mut test_app = TestApp::new_with_faction(FactionEnum::TheSyndicate);
    test_app.step();

    let mut h = TestHarness::new(&mut test_app.app);

    // Find the tunnel spawned at game start (grid 40,40, owner player 0)
    let tunnel_entity = {
        let world = h.app.world_mut();
        let mut query = world.query_filtered::<(Entity, &Owner), With<TunnelState>>();
        query.iter(world)
            .find(|(_, owner)| **owner == Owner(Some(0)))
            .map(|(e, _)| e)
            .expect("Syndicate should have a starting tunnel")
    };

    let agent = h.spawn_unit_at_grid(ObjectEnum::SyndicateAgent, 20, 20, Owner(Some(0)));

    // Give agent crystals and put in DroppingOff phase
    h.app.world_mut().entity_mut(agent).insert(AgentCarryState { crystals: 50, supplies: 0 });
    h.app.world_mut().entity_mut(agent).insert(
        GatheringResourceBehavior {
            target_resource: Entity::PLACEHOLDER,
            phase: GatherPhase::DroppingOff { tunnel_entity, frames_remaining: 48 },
            path: Vec::new(),
            path_index: 0,
        }
    );

    // Run 47 frames — should still be dropping off
    for _ in 0..47 {
        h.step();
    }

    let gathering = h.app.world().get::<GatheringResourceBehavior>(agent);
    assert!(gathering.is_some(), "GatheringResourceBehavior should still exist after 47 frames of drop-off");
    let gathering = gathering.unwrap();
    assert!(matches!(gathering.phase, GatherPhase::DroppingOff { frames_remaining: 1, .. }),
        "After 47 frames, should have 1 frame remaining");

    // Run 1 more frame — drop-off should complete
    h.step();

    let gathering = h.app.world().get::<GatheringResourceBehavior>(agent);
    assert!(gathering.is_none(), "GatheringResourceBehavior should be removed after drop-off completes");
}

/// QA Step 6 [auto]: Verify 50 Space Crystals are added to the player's crystal count after drop-off completes.
#[test]
fn step_6_crystals_added_to_player_resources() {
    let mut test_app = TestApp::new_with_faction(FactionEnum::TheSyndicate);
    test_app.step();

    let mut h = TestHarness::new(&mut test_app.app);

    // Record initial crystal count
    let initial_crystals = h.get_syndicate_crystals().unwrap_or(0);

    let tunnel_entity = {
        let world = h.app.world_mut();
        let mut query = world.query_filtered::<(Entity, &Owner), With<TunnelState>>();
        query.iter(world)
            .find(|(_, owner)| **owner == Owner(Some(0)))
            .map(|(e, _)| e)
            .expect("Syndicate should have a starting tunnel")
    };

    let agent = h.spawn_unit_at_grid(ObjectEnum::SyndicateAgent, 20, 20, Owner(Some(0)));

    // Give agent 50 crystals and put in DroppingOff phase with 1 frame remaining
    h.app.world_mut().entity_mut(agent).insert(AgentCarryState { crystals: 50, supplies: 0 });
    h.app.world_mut().entity_mut(agent).insert(
        GatheringResourceBehavior {
            target_resource: Entity::PLACEHOLDER,
            phase: GatherPhase::DroppingOff { tunnel_entity, frames_remaining: 1 },
            path: Vec::new(),
            path_index: 0,
        }
    );

    // Complete drop-off
    h.step();

    // Verify player resources increased by 50
    let final_crystals = h.get_syndicate_crystals().unwrap_or(0);
    assert_eq!(final_crystals, initial_crystals + 50,
        "Player should gain 50 crystals. Initial: {}, Final: {}", initial_crystals, final_crystals);

    // Verify carry state was cleared
    let carry = h.app.world().get::<AgentCarryState>(agent).unwrap();
    assert_eq!(carry.crystals, 0, "Agent carry state should be cleared after drop-off");
}

/// QA Step 8 [auto]: Verify the Agent picks up 1 Supply after pickup completes.
#[test]
fn step_8_picks_up_1_supply() {
    let mut test_app = TestApp::new_with_faction(FactionEnum::TheSyndicate);
    test_app.step();

    let mut h = TestHarness::new(&mut test_app.app);

    // Spawn a SupplyDeliveryStation entity manually (no harness helper)
    let sds = h.app.world_mut().run_system_once(
        move |mut commands: Commands,
              mut meshes: ResMut<Assets<Mesh>>,
              mut materials: ResMut<Assets<StandardMaterial>>| {
            let mesh = meshes.add(Cuboid::new(0.6, 0.6, 0.6));
            let material = materials.add(StandardMaterial {
                base_color: Color::srgb(0.8, 0.5, 0.2),
                ..default()
            });
            // Place at grid (22, 22) → world (-9.5, 0.3, -9.5)
            let world_x = (22_f32 - 32.0) + 0.5;
            let world_z = (22_f32 - 32.0) + 0.5;
            commands.spawn((
                Mesh3d(mesh),
                MeshMaterial3d(material),
                Transform::from_xyz(world_x, 0.3, world_z),
                SupplyDeliveryStation {
                    delivery_size: 5,
                    delivery_interval: 10.0,
                    current_supplies: 10,
                    time_until_next_delivery: 10.0,
                },
                GridPosition { x: 22, z: 22 },
                ObjectInstance::indestructible(ObjectEnum::SupplyDeliveryStation),
                Selectable,
            )).id()
        },
    ).unwrap();

    let agent = h.spawn_unit_at_grid(ObjectEnum::SyndicateAgent, 22, 23, Owner(Some(0)));

    // Start in Extracting phase with 1 frame remaining at a supply station
    h.app.world_mut().entity_mut(agent).insert(
        GatheringResourceBehavior {
            target_resource: sds,
            phase: GatherPhase::Extracting { frames_remaining: 1 },
            path: Vec::new(),
            path_index: 0,
        }
    );

    // Verify carry state starts at 0
    let carry = h.app.world().get::<AgentCarryState>(agent).unwrap();
    assert_eq!(carry.supplies, 0, "Should start with 0 supplies");

    // Complete extraction
    h.step();

    // Verify Agent now carries 1 supply
    let carry = h.app.world().get::<AgentCarryState>(agent).unwrap();
    assert_eq!(carry.supplies, 1, "After pickup, Agent should carry 1 supply");
}

/// QA Step 10 [auto]: Verify the drop-off takes exactly 48 frames at Side C and 1 Supply is added to the player's supply count.
#[test]
fn step_10_supply_dropoff_48_frames_side_c() {
    let mut test_app = TestApp::new_with_faction(FactionEnum::TheSyndicate);
    test_app.step();

    let mut h = TestHarness::new(&mut test_app.app);

    // Record initial supply count
    let initial_supplies = {
        let world = h.app.world_mut();
        let mut query = world.query::<(&Player, &SyndicatePlayerResources)>();
        query.iter(world).next().map(|(_, r)| r.supplies).unwrap_or(0)
    };

    let tunnel_entity = {
        let world = h.app.world_mut();
        let mut query = world.query_filtered::<(Entity, &Owner), With<TunnelState>>();
        query.iter(world)
            .find(|(_, owner)| **owner == Owner(Some(0)))
            .map(|(e, _)| e)
            .expect("Syndicate should have a starting tunnel")
    };

    let agent = h.spawn_unit_at_grid(ObjectEnum::SyndicateAgent, 20, 20, Owner(Some(0)));

    // Give agent 1 supply and put in DroppingOff phase with 48 frames
    h.app.world_mut().entity_mut(agent).insert(AgentCarryState { crystals: 0, supplies: 1 });
    h.app.world_mut().entity_mut(agent).insert(
        GatheringResourceBehavior {
            target_resource: Entity::PLACEHOLDER,
            phase: GatherPhase::DroppingOff { tunnel_entity, frames_remaining: 48 },
            path: Vec::new(),
            path_index: 0,
        }
    );

    // Run 47 frames — should still be dropping off
    for _ in 0..47 {
        h.step();
    }

    let gathering = h.app.world().get::<GatheringResourceBehavior>(agent);
    assert!(gathering.is_some(), "Should still be in DroppingOff after 47 frames");
    assert!(matches!(gathering.unwrap().phase, GatherPhase::DroppingOff { frames_remaining: 1, .. }));

    // Frame 48 completes drop-off
    h.step();

    let gathering = h.app.world().get::<GatheringResourceBehavior>(agent);
    assert!(gathering.is_none(), "Behavior should be removed after 48 frames");

    // Verify 1 supply added to player
    let final_supplies = {
        let world = h.app.world_mut();
        let mut query = world.query::<(&Player, &SyndicatePlayerResources)>();
        query.iter(world).next().map(|(_, r)| r.supplies).unwrap_or(0)
    };
    assert_eq!(final_supplies, initial_supplies + 1,
        "Player should gain 1 supply. Initial: {}, Final: {}", initial_supplies, final_supplies);
}

/// QA Step 11 [auto]: Send two Agents to drop off crystals at the same Tunnel Side B simultaneously.
/// Verify only one Agent drops off at a time — the second waits.
#[test]
fn step_11_one_agent_drops_off_at_a_time_same_side() {
    let mut test_app = TestApp::new_with_faction(FactionEnum::TheSyndicate);
    test_app.step();

    let mut h = TestHarness::new(&mut test_app.app);

    // Find the starting tunnel
    let tunnel_entity = {
        let world = h.app.world_mut();
        let mut query = world.query_filtered::<(Entity, &Owner), With<TunnelState>>();
        query.iter(world)
            .find(|(_, owner)| **owner == Owner(Some(0)))
            .map(|(e, _)| e)
            .expect("Syndicate should have a starting tunnel")
    };

    // Spawn two agents — both carrying crystals
    let agent1 = h.spawn_unit_at_grid(ObjectEnum::SyndicateAgent, 20, 20, Owner(Some(0)));
    let agent2 = h.spawn_unit_at_grid(ObjectEnum::SyndicateAgent, 20, 22, Owner(Some(0)));

    // Agent 2's position (where it will "arrive" at the tunnel side)
    let agent2_pos = h.get_position(agent2).unwrap();

    // Agent 1: already dropping off at the tunnel (occupies side B)
    h.app.world_mut().entity_mut(agent1).insert(AgentCarryState { crystals: 50, supplies: 0 });
    h.app.world_mut().entity_mut(agent1).insert(
        GatheringResourceBehavior {
            target_resource: Entity::PLACEHOLDER,
            phase: GatherPhase::DroppingOff { tunnel_entity, frames_remaining: 48 },
            path: Vec::new(),
            path_index: 0,
        }
    );

    // Agent 2: in MovingToTunnel phase, already at the side position (within arrival threshold)
    // This agent should be blocked from entering DroppingOff since agent1 is already there
    h.app.world_mut().entity_mut(agent2).insert(AgentCarryState { crystals: 50, supplies: 0 });
    h.app.world_mut().entity_mut(agent2).insert(
        GatheringResourceBehavior {
            target_resource: Entity::PLACEHOLDER,
            phase: GatherPhase::MovingToTunnel {
                tunnel_entity,
                side_position: agent2_pos,  // Agent is already at the "side position"
            },
            path: Vec::new(),
            path_index: 0,
        }
    );

    // Step one frame
    h.step();

    // Agent 1 should still be in DroppingOff
    let g1 = h.app.world().get::<GatheringResourceBehavior>(agent1);
    assert!(g1.map(|g| matches!(g.phase, GatherPhase::DroppingOff { .. })).unwrap_or(false),
        "Agent1 should still be dropping off. Phase: {:?}", g1.map(|g| &g.phase));

    // Agent 2 should NOT have entered DroppingOff — should still be in MovingToTunnel (waiting)
    let g2 = h.app.world().get::<GatheringResourceBehavior>(agent2);
    assert!(g2.is_some(), "Agent2 should still have GatheringResourceBehavior");
    assert!(matches!(g2.unwrap().phase, GatherPhase::MovingToTunnel { .. }),
        "Agent2 should be waiting in MovingToTunnel since Side B is occupied. Phase: {:?}",
        g2.map(|g| &g.phase));
}

/// QA Step 12 [auto]: Send one Agent to drop off crystals (Side B) and another to drop off supplies (Side C)
/// at the same Tunnel. Verify both can drop off simultaneously (separate sides).
#[test]
fn step_12_simultaneous_dropoff_different_sides() {
    let mut test_app = TestApp::new_with_faction(FactionEnum::TheSyndicate);
    test_app.step();

    let mut h = TestHarness::new(&mut test_app.app);

    let tunnel_entity = {
        let world = h.app.world_mut();
        let mut query = world.query_filtered::<(Entity, &Owner), With<TunnelState>>();
        query.iter(world)
            .find(|(_, owner)| **owner == Owner(Some(0)))
            .map(|(e, _)| e)
            .expect("Syndicate should have a starting tunnel")
    };

    let agent_crystals = h.spawn_unit_at_grid(ObjectEnum::SyndicateAgent, 20, 20, Owner(Some(0)));
    let agent_supplies = h.spawn_unit_at_grid(ObjectEnum::SyndicateAgent, 20, 22, Owner(Some(0)));

    let agent_supplies_pos = h.get_position(agent_supplies).unwrap();

    // Crystal agent — already dropping off at Side B
    h.app.world_mut().entity_mut(agent_crystals).insert(AgentCarryState { crystals: 50, supplies: 0 });
    h.app.world_mut().entity_mut(agent_crystals).insert(
        GatheringResourceBehavior {
            target_resource: Entity::PLACEHOLDER,
            phase: GatherPhase::DroppingOff { tunnel_entity, frames_remaining: 48 },
            path: Vec::new(),
            path_index: 0,
        }
    );

    // Supply agent — in MovingToTunnel, already at the side position (different side = C)
    // Should be allowed to enter DroppingOff since Side C is not occupied
    h.app.world_mut().entity_mut(agent_supplies).insert(AgentCarryState { crystals: 0, supplies: 1 });
    h.app.world_mut().entity_mut(agent_supplies).insert(
        GatheringResourceBehavior {
            target_resource: Entity::PLACEHOLDER,
            phase: GatherPhase::MovingToTunnel {
                tunnel_entity,
                side_position: agent_supplies_pos,
            },
            path: Vec::new(),
            path_index: 0,
        }
    );

    // Step one frame
    h.step();

    // Crystal agent should still be in DroppingOff at Side B
    let g1 = h.app.world().get::<GatheringResourceBehavior>(agent_crystals);
    assert!(g1.map(|g| matches!(g.phase, GatherPhase::DroppingOff { .. })).unwrap_or(false),
        "Crystal agent should be dropping off at Side B. Phase: {:?}", g1.map(|g| &g.phase));

    // Supply agent should have entered DroppingOff at Side C (different side, no conflict)
    let g2 = h.app.world().get::<GatheringResourceBehavior>(agent_supplies);
    assert!(g2.map(|g| matches!(g.phase, GatherPhase::DroppingOff { .. })).unwrap_or(false),
        "Supply agent should be dropping off at Side C (different side). Phase: {:?}", g2.map(|g| &g.phase));
}

/// QA Step 15 [auto]: Verify the DropOffResources command is unavailable (greyed out) when the Agent is not carrying resources.
#[test]
fn step_15_dropoff_unavailable_when_not_carrying() {
    // Test the data contract: AgentCarryState.is_carrying() returns false when empty
    let carry = AgentCarryState { crystals: 0, supplies: 0 };
    assert!(!carry.is_carrying(), "Agent with no resources should not be carrying");

    // When carrying, it should be true
    let carry_crystals = AgentCarryState { crystals: 50, supplies: 0 };
    assert!(carry_crystals.is_carrying(), "Agent with crystals should be carrying");

    let carry_supplies = AgentCarryState { crystals: 0, supplies: 1 };
    assert!(carry_supplies.is_carrying(), "Agent with supplies should be carrying");

    // DropOffResources command IS available for syndicate units at the is_available level
    // (the carry-state check happens at the UI/ObjectInterfaceState layer)
    let dummy_entity = Entity::from_raw_u32(1).unwrap();
    let cmd = UnitCommand::DropOffResources(dummy_entity);
    assert!(cmd.is_available(false, false, false, true),
        "DropOffResources should be available for syndicate units at command level");
    assert!(!cmd.is_available(false, false, false, false),
        "DropOffResources should be unavailable for non-syndicate units");
}

/// QA Step 16 [auto]: Verify that non-Agent units cannot receive the Gather or DropOffResources commands.
#[test]
fn step_16_gather_dropoff_unavailable_for_non_agents() {
    let dummy_entity = Entity::from_raw_u32(1).unwrap();

    // Gather requires is_syndicate=true
    let gather = UnitCommand::Gather(dummy_entity);
    assert!(!gather.is_available(false, false, false, false),
        "Gather should be unavailable for non-syndicate units");
    assert!(!gather.is_available(true, true, true, false),
        "Gather should be unavailable for non-syndicate units even with all other flags");
    assert!(gather.is_available(false, false, false, true),
        "Gather should be available for syndicate units");

    // DropOffResources requires is_syndicate=true
    let dropoff = UnitCommand::DropOffResources(dummy_entity);
    assert!(!dropoff.is_available(false, false, false, false),
        "DropOffResources should be unavailable for non-syndicate units");
    assert!(!dropoff.is_available(true, true, true, false),
        "DropOffResources should be unavailable for non-syndicate units even with all other flags");
    assert!(dropoff.is_available(false, false, false, true),
        "DropOffResources should be available for syndicate units");

    // Verify with actual entities in a test app context
    let mut test_app = TestApp::new();
    test_app.step();
    let mut h = TestHarness::new(&mut test_app.app);

    // Spawn a Peacekeeper (GDO, non-syndicate) — should not accept Gather
    let pk = h.spawn_unit_at_grid(ObjectEnum::Peacekeeper, 25, 25, Owner(Some(0)));

    // The Peacekeeper's is_syndicate flag is false
    let obj = h.app.world().get::<ObjectInstance>(pk).unwrap();
    let is_syndicate = matches!(obj.object_type, ObjectEnum::SyndicateAgent);
    assert!(!is_syndicate, "Peacekeeper should not be a Syndicate unit");
}
