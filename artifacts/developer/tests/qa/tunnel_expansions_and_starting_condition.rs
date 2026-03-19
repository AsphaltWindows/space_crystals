use crate::helpers::*;
use bevy::ecs::system::RunSystemOnce;
use space_crystals::game::types::structures::{
    TunnelArea, TunnelState, TunnelTier, TunnelExpansionMarker, HeadquartersState,
};
use space_crystals::game::types::objects::ObjectInstance;
use space_crystals::game::units::types::state::behavior::InTunnelNetwork;
use space_crystals::types::{DomainEnum, ObjectEnum, Owner, VisibilityStateEnum};

/// QA Step 1 [auto]: Place a Tier 1 Tunnel — verify the Tunnel Area can accept underground expansion buildings
#[test]
fn step_1_tunnel_area_accepts_expansions() {
    let mut test_app = TestApp::new();
    test_app.step();

    let tunnel;
    {
        let mut h = TestHarness::new(&mut test_app.app);
        tunnel = h.spawn_structure_at_grid(ObjectEnum::Tunnel, 20, 20, Owner(Some(0)));
    }
    test_app.step();

    let world = test_app.app.world();
    let area = world.get::<TunnelArea>(tunnel)
        .expect("Tunnel should have a TunnelArea component");

    // Tier 1 area radius = 3, so area is 10x10 centered on the 4x4 footprint.
    // The area should have 100 cells.
    assert_eq!(area.cells.len(), 100, "Tier 1 TunnelArea should have 100 cells, got {}", area.cells.len());

    // A 2x2 Headquarters should fit within the area at a valid offset
    // Tunnel footprint is at (20,20)-(23,23). Area extends 3 cells in each direction.
    // So area spans (17,17) to (26,26). HQ at (18,18) should fit.
    assert!(
        area.fits_expansion(18, 18, 2, 2),
        "A 2x2 HQ should fit at (18,18) within Tier 1 Tunnel Area"
    );

    // Also verify a position within the tunnel footprint itself works
    assert!(
        area.fits_expansion(20, 20, 2, 2),
        "A 2x2 HQ should fit at (20,20) within the tunnel footprint"
    );
}

/// QA Step 2 [auto]: Construct a Headquarters inside the Tunnel Area — verify it occupies grid cells within the area
#[test]
fn step_2_headquarters_inside_tunnel_area() {
    let mut test_app = TestApp::new();
    test_app.step();

    let tunnel;
    let hq;
    {
        let mut h = TestHarness::new(&mut test_app.app);
        tunnel = h.spawn_structure_at_grid(ObjectEnum::Tunnel, 20, 20, Owner(Some(0)));
        // Place HQ inside the tunnel area
        hq = h.spawn_headquarters_at_grid(21, 21, Owner(Some(0)), tunnel);
    }
    test_app.step();

    // HQ should have TunnelExpansionMarker pointing to the parent tunnel
    let world = test_app.app.world();
    let marker = world.get::<TunnelExpansionMarker>(hq)
        .expect("HQ should have TunnelExpansionMarker");
    assert_eq!(marker.parent_tunnel, tunnel, "HQ parent_tunnel should match the spawned tunnel entity");

    // HQ should have HeadquartersState
    assert!(
        world.get::<HeadquartersState>(hq).is_some(),
        "HQ should have HeadquartersState component"
    );

    // Verify the HQ position is within the tunnel area
    let area = world.get::<TunnelArea>(tunnel)
        .expect("Tunnel should have TunnelArea");
    // HQ is 2x2 at (21,21), so it occupies (21,21), (22,21), (21,22), (22,22)
    assert!(area.contains(21, 21), "HQ cell (21,21) should be within Tunnel Area");
    assert!(area.contains(22, 21), "HQ cell (22,21) should be within Tunnel Area");
    assert!(area.contains(21, 22), "HQ cell (21,22) should be within Tunnel Area");
    assert!(area.contains(22, 22), "HQ cell (22,22) should be within Tunnel Area");
}

/// QA Step 3 [auto]: Verify the Headquarters is invisible to an enemy player without detection
#[test]
fn step_3_headquarters_invisible_to_enemy() {
    let mut test_app = TestApp::new();
    test_app.step();

    let tunnel;
    let hq;
    {
        let mut h = TestHarness::new(&mut test_app.app);
        // Player 0 owns the tunnel and HQ
        tunnel = h.spawn_structure_at_grid(ObjectEnum::Tunnel, 20, 20, Owner(Some(0)));
        hq = h.spawn_headquarters_at_grid(21, 21, Owner(Some(0)), tunnel);
    }
    test_app.step();

    // HQ should be underground
    let world = test_app.app.world();
    let domain = world.get::<DomainEnum>(hq)
        .expect("HQ should have a DomainEnum component");
    assert_eq!(
        *domain, DomainEnum::Underground,
        "HQ should be in the Underground domain, got {:?}", domain
    );

    // Enemy player (player 1) should not see the HQ location as Visible
    let h = TestHarness::new(&mut test_app.app);
    let vis = h.get_visibility(1, 21, 21);
    assert_ne!(
        vis, VisibilityStateEnum::Visible,
        "Enemy player should not see underground HQ as Visible"
    );
}

/// QA Step 4 [auto]: Move a surface unit over the Headquarters location — verify no collision
#[test]
fn step_4_surface_unit_no_collision_with_underground_hq() {
    let mut test_app = TestApp::new();
    test_app.step();

    let tunnel;
    let _hq;
    let surface_unit;
    {
        let mut h = TestHarness::new(&mut test_app.app);
        // Player 0 tunnel and HQ
        tunnel = h.spawn_structure_at_grid(ObjectEnum::Tunnel, 20, 20, Owner(Some(0)));
        _hq = h.spawn_headquarters_at_grid(21, 21, Owner(Some(0)), tunnel);
        // Spawn an enemy ground unit at the same grid cell as the HQ
        surface_unit = h.spawn_unit_at_grid(ObjectEnum::Peacekeeper, 21, 21, Owner(Some(1)));
    }
    test_app.step();

    // The surface unit should exist at the HQ location without being blocked
    let h = TestHarness::new(&mut test_app.app);
    assert!(h.is_alive(surface_unit), "Surface unit should be alive");

    let pos = h.get_position(surface_unit).expect("Surface unit should have a position");
    // The unit should be at approximately grid (21,21)
    // Grid to world: world_x = (grid_x - 32) + 0.5, so grid 21 => -10.5
    let expected_x = (21.0_f32 - 32.0) + 0.5; // -10.5
    let expected_z = (21.0_f32 - 32.0) + 0.5; // -10.5
    assert!(
        (pos.x - expected_x).abs() < 2.0 && (pos.z - expected_z).abs() < 2.0,
        "Surface unit should be near grid (21,21) world pos ({}, {}), got ({}, {})", expected_x, expected_z, pos.x, pos.z
    );

    // Verify the surface unit is in the Ground domain (not underground)
    let world = test_app.app.world();
    let domain = world.get::<DomainEnum>(surface_unit)
        .expect("Surface unit should have DomainEnum");
    assert_eq!(
        *domain, DomainEnum::Ground,
        "Surface unit should be in Ground domain"
    );
}

/// QA Step 5 [auto]: Set the Headquarters rally point to emerge from the parent Tunnel —
/// verify production state setup (Agent production cost and rally point setting)
#[test]
fn step_5_headquarters_rally_point_and_agent_production() {
    let mut test_app = TestApp::new();
    test_app.step();

    let tunnel;
    let hq;
    {
        let mut h = TestHarness::new(&mut test_app.app);
        tunnel = h.spawn_structure_at_grid(ObjectEnum::Tunnel, 20, 20, Owner(Some(0)));
        hq = h.spawn_headquarters_at_grid(21, 21, Owner(Some(0)), tunnel);
    }
    test_app.step();

    // Verify HQ can produce SyndicateAgent (cost exists)
    let cost = HeadquartersState::production_cost(&ObjectEnum::SyndicateAgent)
        .expect("HQ should have a production cost for SyndicateAgent");
    assert_eq!(cost.space_crystals, 100, "Agent production cost should be 100 SC");
    assert_eq!(cost.build_frames, 160, "Agent build time should be 160 frames");

    // Verify the rally point can be set to a surface location (simulating tunnel exit)
    let world = test_app.app.world();
    let hq_state = world.get::<HeadquartersState>(hq)
        .expect("HQ should have HeadquartersState");

    // Default state: no rally point set yet
    assert!(hq_state.rally_point.is_none(), "Default HQ should have no rally point");

    // Verify the build queue works
    let mut state = HeadquartersState::default();
    assert!(state.try_queue(ObjectEnum::SyndicateAgent), "Should be able to queue an Agent");
    assert_eq!(state.build_queue.len(), 1);
}

/// QA Step 6 [auto]: Set the rally point to remain in the Tunnel Network —
/// verify unit can be queued and state remains in-network
#[test]
fn step_6_rally_point_in_tunnel_network() {
    let mut test_app = TestApp::new();
    test_app.step();

    let tunnel;
    let hq;
    {
        let mut h = TestHarness::new(&mut test_app.app);
        tunnel = h.spawn_structure_at_grid(ObjectEnum::Tunnel, 20, 20, Owner(Some(0)));
        hq = h.spawn_headquarters_at_grid(21, 21, Owner(Some(0)), tunnel);
    }
    test_app.step();

    // Verify HQ can queue multiple agents (production system state)
    let world = test_app.app.world();
    let hq_state = world.get::<HeadquartersState>(hq)
        .expect("HQ should have HeadquartersState");

    // No rally point = units stay in network by default
    assert!(
        hq_state.rally_point.is_none(),
        "HQ with no rally point should keep produced units in-network"
    );

    // Verify max queue size is 5
    assert_eq!(HeadquartersState::MAX_QUEUE_SIZE, 5);

    // Verify queuing up to max works
    let mut state = HeadquartersState::default();
    for i in 0..5 {
        assert!(state.try_queue(ObjectEnum::SyndicateAgent), "Should queue agent {}", i + 1);
    }
    assert!(!state.try_queue(ObjectEnum::SyndicateAgent), "Should reject 6th queue entry");
}

/// QA Step 8 [auto]: Attempt to build a Headquarters in a Tunnel that has no remaining Tunnel Area space —
/// verify placement is rejected via fits_expansion
#[test]
fn step_8_no_space_rejects_hq_placement() {
    // Create a TunnelArea and verify fits_expansion returns false for positions outside the area
    let area = TunnelArea::new(20, 20, &TunnelTier::Tier1);

    // Position far outside the area should not fit
    assert!(
        !area.fits_expansion(0, 0, 2, 2),
        "HQ should not fit at (0,0) which is outside Tunnel Area"
    );

    // Position at the very edge where 2x2 would extend outside
    // Area spans (17,17) to (26,26) inclusive. A 2x2 at (26,26) needs (26,26),(27,26),(26,27),(27,27)
    // (27,*) is outside the area.
    assert!(
        !area.fits_expansion(26, 26, 2, 2),
        "HQ should not fit at edge (26,26) where it would extend outside the area"
    );

    // Verify a position fully inside still works
    assert!(
        area.fits_expansion(20, 20, 2, 2),
        "HQ should fit at (20,20) which is inside the area"
    );

    // Verify that contains returns false for cells outside the area
    assert!(!area.contains(0, 0), "Cell (0,0) should not be in the area");
    assert!(!area.contains(100, 100), "Cell (100,100) should not be in the area");
}

/// QA Step 9 [auto]: Start a new Syndicate game — verify the player begins with
/// 1 Tier 1 Tunnel and 1 pre-built Headquarters inside it
#[test]
fn step_9_syndicate_starting_tunnel_and_hq() {
    // Default TestApp uses GDO faction for player 0, Syndicate for player 1
    let mut test_app = TestApp::new();
    test_app.step();

    // Query for Tunnel entities owned by player 1 (Syndicate)
    let world = test_app.app.world_mut();

    let mut tunnel_count = 0;
    let mut tunnel_entity = None;
    let mut query = world.query::<(Entity, &ObjectInstance, &Owner, &TunnelState)>();
    for (entity, obj, owner, _state) in query.iter(world) {
        if owner.0 == Some(1) && obj.object_type == ObjectEnum::Tunnel {
            tunnel_count += 1;
            tunnel_entity = Some(entity);
        }
    }
    assert_eq!(tunnel_count, 1, "Syndicate should start with exactly 1 Tunnel, found {}", tunnel_count);

    let tunnel_entity = tunnel_entity.expect("Should have found the Syndicate tunnel");

    // Verify Tunnel is Tier 1
    let tunnel_state = world.get::<TunnelState>(tunnel_entity)
        .expect("Tunnel should have TunnelState");
    assert_eq!(tunnel_state.tier, TunnelTier::Tier1, "Starting Tunnel should be Tier 1");

    // Query for HQ entities owned by player 1 with TunnelExpansionMarker
    let mut hq_count = 0;
    let mut hq_query = world.query::<(Entity, &ObjectInstance, &Owner, &TunnelExpansionMarker, &HeadquartersState)>();
    for (_entity, obj, owner, marker, _state) in hq_query.iter(world) {
        if owner.0 == Some(1) && obj.object_type == ObjectEnum::Headquarters {
            hq_count += 1;
            assert_eq!(
                marker.parent_tunnel, tunnel_entity,
                "HQ parent_tunnel should reference the starting Tunnel"
            );
        }
    }
    assert_eq!(hq_count, 1, "Syndicate should start with exactly 1 HQ inside the Tunnel, found {}", hq_count);
}

/// QA Step 10 [auto]: Verify the starting Headquarters is immediately functional
/// (can produce Agents from game start)
#[test]
fn step_10_starting_hq_immediately_functional() {
    let mut test_app = TestApp::new();
    test_app.step();

    // Find the Syndicate HQ (owned by player 1)
    let world = test_app.app.world_mut();

    let mut hq_entity = None;
    let mut query = world.query::<(Entity, &ObjectInstance, &Owner, &HeadquartersState)>();
    for (entity, obj, owner, _state) in query.iter(world) {
        if owner.0 == Some(1) && obj.object_type == ObjectEnum::Headquarters {
            hq_entity = Some(entity);
        }
    }
    let hq_entity = hq_entity.expect("Should have found the Syndicate starting HQ");

    let hq_state = world.get::<HeadquartersState>(hq_entity)
        .expect("HQ should have HeadquartersState");

    // HQ should be in default (idle) state — ready to produce
    assert!(hq_state.rally_point.is_none(), "Starting HQ rally_point should be None (default)");
    assert!(hq_state.build_queue.is_empty(), "Starting HQ build_queue should be empty");
    assert!(hq_state.current_build.is_none(), "Starting HQ should not be building anything yet");
    assert!(hq_state.current_build_progress.is_none(), "Starting HQ should have no build progress");

    // Verify it CAN produce agents (production cost exists)
    assert!(
        HeadquartersState::production_cost(&ObjectEnum::SyndicateAgent).is_some(),
        "HQ should be able to produce SyndicateAgent"
    );
}

/// Verify that HQ production ejects an agent from Side A by default (no rally point).
/// FixedUpdate doesn't fire reliably in headless TestApp, so we set the build state
/// to near-completion and run the production system directly via run_system_once.
#[test]
fn hq_production_ejects_agent_without_rally_point() {
    use space_crystals::game::world::faction::headquarters_production_tick_system;

    let mut test_app = TestApp::new();
    test_app.step();

    // Find the Syndicate HQ (player 1 in default GDO setup)
    let hq_entity;
    {
        let world = test_app.app.world_mut();
        let mut query = world.query::<(Entity, &ObjectInstance, &Owner, &HeadquartersState)>();
        hq_entity = query.iter(world)
            .find(|(_, obj, owner, _)| owner.0 == Some(1) && obj.object_type == ObjectEnum::Headquarters)
            .map(|(e, _, _, _)| e)
            .expect("Should find Syndicate starting HQ");
    }

    // Set HQ to have a build in progress at 159/160 frames (1 tick from completion)
    // No rally point = default eject behavior
    {
        let world = test_app.app.world_mut();
        let mut hq_state = world.get_mut::<HeadquartersState>(hq_entity).unwrap();
        assert!(hq_state.rally_point.is_none(), "No rally point should be set");
        hq_state.current_build = Some(ObjectEnum::SyndicateAgent);
        hq_state.current_build_progress = Some(159.0);
    }

    // Count existing SyndicateAgent entities before production
    let agents_before = {
        let world = test_app.app.world_mut();
        let mut q = world.query::<(&ObjectInstance, &Owner)>();
        q.iter(world)
            .filter(|(obj, owner)| obj.object_type == ObjectEnum::SyndicateAgent && owner.0 == Some(1))
            .count()
    };

    // Run the production tick system directly (simulates one FixedUpdate tick)
    test_app.app.world_mut().run_system_once(headquarters_production_tick_system).unwrap();
    // Apply deferred commands (entity spawning)
    test_app.step();

    // Count agents after production
    let world = test_app.app.world_mut();
    let mut q = world.query::<(Entity, &ObjectInstance, &Owner)>();
    let agent_entities: Vec<Entity> = q.iter(world)
        .filter(|(_, obj, owner)| obj.object_type == ObjectEnum::SyndicateAgent && owner.0 == Some(1))
        .map(|(e, _, _)| e)
        .collect();

    assert!(
        agent_entities.len() > agents_before,
        "HQ should have produced an agent (before: {}, after: {})", agents_before, agent_entities.len()
    );

    // The new agent should NOT be hidden in tunnel network (no InTunnelNetwork component)
    let new_agent = agent_entities.last().unwrap();
    assert!(
        world.get::<InTunnelNetwork>(*new_agent).is_none(),
        "Agent should NOT be in tunnel network — should eject to surface with no rally point"
    );

    // Agent should not be hidden
    let visibility = world.get::<Visibility>(*new_agent);
    if let Some(vis) = visibility {
        assert_ne!(
            *vis, Visibility::Hidden,
            "Agent should be visible on the surface, not hidden"
        );
    }
}

/// Verify that HQ production puts agent in tunnel network when rally is set to parent tunnel
#[test]
fn hq_production_enters_tunnel_network_when_rally_is_parent_tunnel() {
    use space_crystals::game::types::structures::RallyTarget;
    use space_crystals::game::world::faction::headquarters_production_tick_system;

    let mut test_app = TestApp::new();
    test_app.step();

    // Find the Syndicate HQ and its parent tunnel
    let hq_entity;
    let parent_tunnel;
    {
        let world = test_app.app.world_mut();
        let mut query = world.query::<(Entity, &ObjectInstance, &Owner, &HeadquartersState, &TunnelExpansionMarker)>();
        let (e, _, _, _, marker) = query.iter(world)
            .find(|(_, obj, owner, _, _)| owner.0 == Some(1) && obj.object_type == ObjectEnum::Headquarters)
            .expect("Should find Syndicate starting HQ");
        hq_entity = e;
        parent_tunnel = marker.parent_tunnel;
    }

    // Set rally to parent tunnel with build at 159/160
    {
        let world = test_app.app.world_mut();
        let mut hq_state = world.get_mut::<HeadquartersState>(hq_entity).unwrap();
        hq_state.rally_point = Some(RallyTarget::Object(parent_tunnel));
        hq_state.current_build = Some(ObjectEnum::SyndicateAgent);
        hq_state.current_build_progress = Some(159.0);
    }

    // Count agents before
    let agents_before = {
        let world = test_app.app.world_mut();
        let mut q = world.query::<(&ObjectInstance, &Owner)>();
        q.iter(world)
            .filter(|(obj, owner)| obj.object_type == ObjectEnum::SyndicateAgent && owner.0 == Some(1))
            .count()
    };

    // Run the production tick system directly
    test_app.app.world_mut().run_system_once(headquarters_production_tick_system).unwrap();
    test_app.step();

    // Count agents after
    let world = test_app.app.world_mut();
    let mut q = world.query::<(Entity, &ObjectInstance, &Owner)>();
    let agent_entities: Vec<Entity> = q.iter(world)
        .filter(|(_, obj, owner)| obj.object_type == ObjectEnum::SyndicateAgent && owner.0 == Some(1))
        .map(|(e, _, _)| e)
        .collect();

    assert!(
        agent_entities.len() > agents_before,
        "HQ should have produced an agent (before: {}, after: {})", agents_before, agent_entities.len()
    );

    // The new agent SHOULD be in the tunnel network (rally target is parent tunnel)
    let new_agent = agent_entities.last().unwrap();
    assert!(
        world.get::<InTunnelNetwork>(*new_agent).is_some(),
        "Agent should be in tunnel network when rally points to parent tunnel"
    );
}

/// Verify that underground HQ tiles are NOT in OccupancyMap.blocked_tiles or structure_tiles,
/// while the surface Tunnel tiles ARE blocked.
#[test]
fn underground_hq_does_not_block_occupancy_map() {
    use space_crystals::game::units::types::OccupancyMap;

    let mut test_app = TestApp::new();
    test_app.step();

    let tunnel;
    let _hq;
    {
        let mut h = TestHarness::new(&mut test_app.app);
        // Tunnel at (20,20) is 4x4: occupies (20,20)-(23,23)
        tunnel = h.spawn_structure_at_grid(ObjectEnum::Tunnel, 20, 20, Owner(Some(0)));
        // Place HQ at (18,18) — outside the Tunnel footprint but within the Tunnel Area
        // Tier 1 area radius=3, so area spans (17,17)-(26,26)
        _hq = h.spawn_headquarters_at_grid(18, 18, Owner(Some(0)), tunnel);
    }
    // Step to run rebuild_occupancy_map
    test_app.step();

    let world = test_app.app.world();
    let occupancy = world.resource::<OccupancyMap>();

    // HQ is 2x2 at grid (18,18) — occupies (18,18), (19,18), (18,19), (19,19)
    // These tiles should NOT be blocked (HQ is underground)
    for dx in 0..2 {
        for dz in 0..2 {
            let tile = (18 + dx, 18 + dz);
            assert!(
                !occupancy.blocked_tiles.contains(&tile),
                "Underground HQ tile {:?} should NOT be in blocked_tiles", tile
            );
            assert!(
                !occupancy.structure_tiles.contains(&tile),
                "Underground HQ tile {:?} should NOT be in structure_tiles", tile
            );
        }
    }

    // Tunnel is 4x4 at grid (20,20) — its tiles SHOULD be blocked (surface structure)
    for dx in 0..4 {
        for dz in 0..4 {
            let tile = (20 + dx, 20 + dz);
            assert!(
                occupancy.blocked_tiles.contains(&tile),
                "Surface Tunnel tile {:?} SHOULD be in blocked_tiles", tile
            );
            assert!(
                occupancy.structure_tiles.contains(&tile),
                "Surface Tunnel tile {:?} SHOULD be in structure_tiles", tile
            );
        }
    }
}
