use bevy::prelude::*;
use crate::types::{Owner, ObjectEnum, MainCamera, LocalPlayer, InvisibleEntity, SelectedFaction, FactionEnum};
use crate::game::types::*;
use crate::game::entities::{EntityRegistry, SpawnParams};
use crate::game::entities::utils::{insert_entity_visuals, grid_to_spawn_position};
use crate::game::world::types::{GdoBuildArea, MapStartingPositions};
use crate::game::world::utils::{expand_build_area, grid_to_world};

/// Setup initial player resources using design-aligned Player + faction-specific resource components.
/// Player 0 is always the local human; the faction assignment depends on SelectedFaction.
pub fn setup_player_resources(mut commands: Commands, selected: Res<SelectedFaction>) {
    // The local human controls player 0
    commands.insert_resource(LocalPlayer(0));

    match selected.0 {
        FactionEnum::GlobalDefenseOrdinance => {
            // GDO selected: GDO=player 0 (local), Syndicate=player 1 (opponent)
            spawn_faction_and_player(&mut commands, FactionEnum::GlobalDefenseOrdinance, 0);
            spawn_faction_and_player(&mut commands, FactionEnum::TheSyndicate, 1);
        }
        FactionEnum::TheSyndicate => {
            // Syndicate selected: Syndicate=player 0 (local), GDO=player 1 (opponent)
            spawn_faction_and_player(&mut commands, FactionEnum::TheSyndicate, 0);
            spawn_faction_and_player(&mut commands, FactionEnum::GlobalDefenseOrdinance, 1);
        }
        FactionEnum::TheCults => {
            // Cults selected: Cults=player 0 (local), GDO=player 1 (opponent)
            spawn_faction_and_player(&mut commands, FactionEnum::TheCults, 0);
            spawn_faction_and_player(&mut commands, FactionEnum::GlobalDefenseOrdinance, 1);
        }
        FactionEnum::Colonists => {
            // Colonists selected: Colonists=player 0 (local), GDO=player 1 (opponent)
            spawn_faction_and_player(&mut commands, FactionEnum::Colonists, 0);
            spawn_faction_and_player(&mut commands, FactionEnum::GlobalDefenseOrdinance, 1);
        }
    }

    info!("Initialized faction resources: Selected {:?}", selected.0);
}

/// Helper to spawn a faction entity and its corresponding player entity with resources.
fn spawn_faction_and_player(commands: &mut Commands, faction: FactionEnum, player_id: u8) {
    let player_name = if player_id == 0 { "Player 1" } else { "Player 2" };

    // Spawn the faction entity
    commands.spawn((
        InvisibleEntity,
        faction,
        DisplayHud::new(faction),
    ));

    // Spawn the player entity with faction-specific resources
    match faction {
        FactionEnum::GlobalDefenseOrdinance => {
            commands.spawn((
                InvisibleEntity,
                Player::new(player_name, FactionEnum::GlobalDefenseOrdinance, player_id),
                DisplayHudInfo::new(FactionEnum::GlobalDefenseOrdinance),
                GdoPlayerResources {
                    space_crystals: 1000,
                    supplies: 0,
                    power_generated: 0,
                    power_consumed: 0,
                    unit_control_used: 0,
                    unit_control_cap: 200,
                    has_power_plant: false,
                },
            ));
        }
        FactionEnum::TheSyndicate => {
            commands.spawn((
                InvisibleEntity,
                Player::new(player_name, FactionEnum::TheSyndicate, player_id),
                DisplayHudInfo::new(FactionEnum::TheSyndicate),
                SyndicatePlayerResources::default(),
            ));
        }
        FactionEnum::TheCults => {
            commands.spawn((
                InvisibleEntity,
                Player::new(player_name, FactionEnum::TheCults, player_id),
                DisplayHudInfo::new(FactionEnum::TheCults),
                CultsPlayerResources::default(),
            ));
        }
        FactionEnum::Colonists => {
            commands.spawn((
                InvisibleEntity,
                Player::new(player_name, FactionEnum::Colonists, player_id),
                DisplayHudInfo::new(FactionEnum::Colonists),
                ColonistsPlayerResources::default(),
            ));
        }
    }
}

/// Spawn the initial Deployment Center and initialize the build area
pub fn setup_gdo_game_start(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    mut build_area: ResMut<GdoBuildArea>,
    selected: Res<SelectedFaction>,
    registry: Res<EntityRegistry>,
) {
    let gdo_owner = if selected.0 == FactionEnum::GlobalDefenseOrdinance {
        Owner::player(0)
    } else {
        Owner::player(1)
    };

    let dc_grid_x = 30;
    let dc_grid_z = 30;
    let spawn_pos = grid_to_spawn_position(dc_grid_x, dc_grid_z, 4.0, 4.0);
    let _dc_entity = registry.spawn(&ObjectEnum::DeploymentCenter, &mut commands, SpawnParams {
        position: spawn_pos,
        owner: gdo_owner,
        player: gdo_owner.player_number().unwrap_or(0),
        rotation: None,
        under_construction: false,
        flip_horizontal: false,
        flip_vertical: false,
    });
    insert_entity_visuals(&mut commands, _dc_entity, &ObjectEnum::DeploymentCenter, &gdo_owner, &mut meshes, &mut materials);

    expand_build_area(&mut build_area, dc_grid_x, dc_grid_z, 4, 4, 12);

    info!(
        "GDO Game Start: Deployed DC at grid ({}, {}), owner={:?}, build area: {} cells",
        dc_grid_x, dc_grid_z, gdo_owner, build_area.cells.len()
    );
}

/// Spawn the initial Tunnel and Headquarters for the Syndicate faction
pub fn setup_syndicate_game_start(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    selected: Res<SelectedFaction>,
    registry: Res<EntityRegistry>,
) {
    let syn_owner = if selected.0 == FactionEnum::TheSyndicate {
        Owner::player(0)
    } else {
        Owner::player(1)
    };

    let tunnel_grid_x = 40;
    let tunnel_grid_z = 40;

    // Spawn a Tier 1 Tunnel via registry
    let tunnel_pos = grid_to_spawn_position(tunnel_grid_x, tunnel_grid_z, 4.0, 4.0);
    let tunnel_entity = registry.spawn(&ObjectEnum::Tunnel, &mut commands, SpawnParams {
        position: tunnel_pos,
        owner: syn_owner,
        player: syn_owner.player_number().unwrap_or(0),
        rotation: None,
        under_construction: false,
        flip_horizontal: false,
        flip_vertical: false,
    });
    insert_entity_visuals(&mut commands, tunnel_entity, &ObjectEnum::Tunnel, &syn_owner, &mut meshes, &mut materials);

    // Spawn a pre-built Headquarters inside the Tunnel's area
    // Place at center of Tunnel Area (offset from tunnel origin)
    let hq_grid_x = tunnel_grid_x + 2; // Offset within the 10x10 area
    let hq_grid_z = tunnel_grid_z - 2;
    let hq_pos = grid_to_spawn_position(hq_grid_x, hq_grid_z, 2.0, 2.0);
    let _hq_entity = registry.spawn(&ObjectEnum::Headquarters, &mut commands, SpawnParams {
        position: hq_pos,
        owner: syn_owner,
        player: syn_owner.player_number().unwrap_or(0),
        rotation: None,
        under_construction: false,
        flip_horizontal: false,
        flip_vertical: false,
    });
    insert_entity_visuals(&mut commands, _hq_entity, &ObjectEnum::Headquarters, &syn_owner, &mut meshes, &mut materials);
    // HQ needs parent tunnel reference
    commands.entity(_hq_entity).insert(TunnelExpansionMarker { parent_tunnel: tunnel_entity });

    info!(
        "Syndicate Game Start: Deployed Tunnel at grid ({}, {}), HQ at ({}, {}), owner={:?}",
        tunnel_grid_x, tunnel_grid_z, hq_grid_x, hq_grid_z, syn_owner
    );
}

/// Spawn the initial Recruitment Center for the Cults faction
pub fn setup_cults_game_start(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    mut rc_counter: ResMut<RecruitmentCenterCounter>,
    selected: Res<SelectedFaction>,
    registry: Res<EntityRegistry>,
) {
    let cults_owner = if selected.0 == FactionEnum::TheCults {
        Owner::player(0)
    } else {
        Owner::player(1)
    };

    let rc_grid_x = 50;
    let rc_grid_z = 50;
    let rc_pos = grid_to_spawn_position(rc_grid_x, rc_grid_z, 4.0, 4.0);
    let _rc_entity = registry.spawn(&ObjectEnum::RecruitmentCenter, &mut commands, SpawnParams {
        position: rc_pos,
        owner: cults_owner,
        player: cults_owner.player_number().unwrap_or(0),
        rotation: None,
        under_construction: false,
        flip_horizontal: false,
        flip_vertical: false,
    });
    insert_entity_visuals(&mut commands, _rc_entity, &ObjectEnum::RecruitmentCenter, &cults_owner, &mut meshes, &mut materials);
    // RC needs build_order for build priority
    commands.entity(_rc_entity).insert(RecruitmentCenterState { build_order: rc_counter.next(), ..Default::default() });

    info!(
        "Cults Game Start: Deployed Recruitment Center at grid ({}, {}), owner={:?}",
        rc_grid_x, rc_grid_z, cults_owner
    );
}

/// Stub game start for the Colonists faction. No structures are designed yet.
pub fn setup_colonists_game_start(
    selected: Res<SelectedFaction>,
) {
    let colonists_owner = if selected.0 == FactionEnum::Colonists {
        Owner::player(0)
    } else {
        Owner::player(1)
    };

    info!(
        "Colonists Game Start: No structures to spawn yet, owner={:?}",
        colonists_owner
    );
}

/// Center the camera on the local player's primary structure at game start.
/// If `MapStartingPositions` has an entry for the local player's slot,
/// the camera centers on that grid position instead.
pub fn center_camera_on_start(
    local_player: Res<LocalPlayer>,
    selected_faction: Res<SelectedFaction>,
    map_positions: Res<MapStartingPositions>,
    structures: Query<(&ObjectInstance, &Owner, &Transform), Without<MainCamera>>,
    mut camera_query: Query<&mut Transform, With<MainCamera>>,
) {
    // Step 1: Check for map-defined starting position
    if let Some(&(grid_x, grid_z)) = map_positions.positions.get(&local_player.0) {
        if let Ok(mut cam_transform) = camera_query.single_mut() {
            let world_pos = grid_to_world(grid_x, grid_z, 1.0);
            let z_offset = cam_transform.translation.y * 25.0 / 40.0;
            cam_transform.translation.x = world_pos.x;
            cam_transform.translation.z = world_pos.z + z_offset;
        }
        return;
    }

    // Step 2: Fall back to primary structure
    let primary_type = match selected_faction.0 {
        FactionEnum::GlobalDefenseOrdinance => ObjectEnum::DeploymentCenter,
        FactionEnum::TheSyndicate => ObjectEnum::Tunnel,
        FactionEnum::TheCults => ObjectEnum::RecruitmentCenter,
        _ => return,
    };
    let local_owner = Owner::player(local_player.0);

    for (obj, owner, transform) in structures.iter() {
        if obj.object_type == primary_type && *owner == local_owner {
            if let Ok(mut cam_transform) = camera_query.single_mut() {
                let z_offset = cam_transform.translation.y * 25.0 / 40.0;
                cam_transform.translation.x = transform.translation.x;
                cam_transform.translation.z = transform.translation.z + z_offset;
            }
            return;
        }
    }
}

/// Spawn enemy test units for the non-selected faction
pub fn setup_enemy_test_units(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    selected: Res<SelectedFaction>,
    registry: Res<EntityRegistry>,
) {
    // Enemy is always player 1 (the non-local player)
    let enemy_owner = Owner::player(1);
    // Choose enemy positions away from the selected faction's start
    let positions = match selected.0 {
        FactionEnum::TheCults => [
            // Cults starts at (50,50) — place enemies near GDO area (30,30)
            (30, 33),
            (31, 32),
            (29, 34),
            (32, 33),
            (30, 35),
        ],
        _ => [
            // GDO/Syndicate start near (30,30) — place enemies near (50,50)
            (50, 50),
            (51, 49),
            (49, 51),
            (52, 50),
            (50, 52),
        ],
    };

    for (gx, gz) in positions {
        let spawn_pos = grid_to_spawn_position(gx, gz, 1.0, 1.0);
        let entity = registry.spawn(&ObjectEnum::Peacekeeper, &mut commands, SpawnParams {
            position: spawn_pos,
            owner: enemy_owner,
            player: 1,
            rotation: None,
            under_construction: false,
            flip_horizontal: false,
            flip_vertical: false,
        });
        insert_entity_visuals(&mut commands, entity, &ObjectEnum::Peacekeeper, &enemy_owner, &mut meshes, &mut materials);
    }

    let area = if matches!(selected.0, FactionEnum::TheCults) { "(30, 30)" } else { "(50, 50)" };
    info!("Spawned {} enemy Peacekeepers around grid {}", positions.len(), area);
}

#[cfg(test)]
mod tests {
    use super::*;
    use bevy::ecs::system::RunSystemOnce;

    #[test]
    fn setup_player_resources_gdo_selected_spawns_gdo_and_syndicate() {
        let mut app = App::new();
        app.insert_resource(SelectedFaction(FactionEnum::GlobalDefenseOrdinance));
        app.world_mut().run_system_once(setup_player_resources).unwrap();
        app.world_mut().flush();

        // Verify LocalPlayer is set
        let local = app.world().resource::<LocalPlayer>();
        assert_eq!(local.0, 0);

        // Verify GDO player entity exists with player_id 0
        let mut found_gdo = false;
        let mut found_syn = false;
        let mut query = app.world_mut().query::<(Entity, &Player)>();
        for (entity, player) in query.iter(app.world()) {
            if player.faction == FactionEnum::GlobalDefenseOrdinance {
                assert_eq!(player.player_number, 0);
                assert!(app.world().get::<GdoPlayerResources>(entity).is_some());
                found_gdo = true;
            }
            if player.faction == FactionEnum::TheSyndicate {
                assert_eq!(player.player_number, 1);
                assert!(app.world().get::<SyndicatePlayerResources>(entity).is_some());
                found_syn = true;
            }
        }
        assert!(found_gdo, "GDO player entity should exist");
        assert!(found_syn, "Syndicate player entity should exist");
    }

    #[test]
    fn setup_player_resources_cults_selected_spawns_cults_and_gdo() {
        let mut app = App::new();
        app.insert_resource(SelectedFaction(FactionEnum::TheCults));
        app.world_mut().run_system_once(setup_player_resources).unwrap();
        app.world_mut().flush();

        let mut found_cults = false;
        let mut found_gdo = false;
        let mut query = app.world_mut().query::<(Entity, &Player)>();
        for (entity, player) in query.iter(app.world()) {
            if player.faction == FactionEnum::TheCults {
                assert_eq!(player.player_number, 0);
                assert!(app.world().get::<CultsPlayerResources>(entity).is_some());
                found_cults = true;
            }
            if player.faction == FactionEnum::GlobalDefenseOrdinance {
                assert_eq!(player.player_number, 1);
                assert!(app.world().get::<GdoPlayerResources>(entity).is_some());
                found_gdo = true;
            }
        }
        assert!(found_cults, "Cults player entity should exist");
        assert!(found_gdo, "GDO opponent entity should exist");
    }

    #[test]
    fn setup_player_resources_colonists_selected_spawns_colonists_and_gdo() {
        let mut app = App::new();
        app.insert_resource(SelectedFaction(FactionEnum::Colonists));
        app.world_mut().run_system_once(setup_player_resources).unwrap();
        app.world_mut().flush();

        let mut found_colonists = false;
        let mut found_gdo = false;
        let mut query = app.world_mut().query::<(Entity, &Player)>();
        for (entity, player) in query.iter(app.world()) {
            if player.faction == FactionEnum::Colonists {
                assert_eq!(player.player_number, 0);
                assert!(app.world().get::<ColonistsPlayerResources>(entity).is_some());
                found_colonists = true;
            }
            if player.faction == FactionEnum::GlobalDefenseOrdinance {
                assert_eq!(player.player_number, 1);
                assert!(app.world().get::<GdoPlayerResources>(entity).is_some());
                found_gdo = true;
            }
        }
        assert!(found_colonists, "Colonists player entity should exist");
        assert!(found_gdo, "GDO opponent entity should exist");
    }

    #[test]
    fn setup_player_resources_syndicate_selected_spawns_syndicate_and_gdo() {
        let mut app = App::new();
        app.insert_resource(SelectedFaction(FactionEnum::TheSyndicate));
        app.world_mut().run_system_once(setup_player_resources).unwrap();
        app.world_mut().flush();

        let mut found_syn = false;
        let mut found_gdo = false;
        let mut query = app.world_mut().query::<(Entity, &Player)>();
        for (entity, player) in query.iter(app.world()) {
            if player.faction == FactionEnum::TheSyndicate {
                assert_eq!(player.player_number, 0);
                assert!(app.world().get::<SyndicatePlayerResources>(entity).is_some());
                found_syn = true;
            }
            if player.faction == FactionEnum::GlobalDefenseOrdinance {
                assert_eq!(player.player_number, 1);
                assert!(app.world().get::<GdoPlayerResources>(entity).is_some());
                found_gdo = true;
            }
        }
        assert!(found_syn, "Syndicate player entity should exist");
        assert!(found_gdo, "GDO opponent entity should exist");
    }

    #[test]
    fn cults_player_resources_default_values() {
        let res = CultsPlayerResources::default();
        assert_eq!(res.space_crystals, 500);
        assert_eq!(res.unit_control_used, 0);
        assert_eq!(res.unit_control_available, 0);
    }

    #[test]
    fn colonists_player_resources_default_values() {
        let res = ColonistsPlayerResources::default();
        assert_eq!(res.space_crystals, 500);
        assert_eq!(res.alloys, 50);
        assert_eq!(res.essence, 50);
        assert_eq!(res.beacon_capacity_provided, 20);
        assert_eq!(res.beacon_capacity_used, 0);
    }

    #[test]
    fn test_center_camera_on_start_gdo() {
        use crate::testing::TestApp;
        use crate::types::{MainCamera, FactionEnum, LocalPlayer};
        use crate::game::types::ObjectInstance;

        let mut test_app = TestApp::new_with_faction(FactionEnum::GlobalDefenseOrdinance);
        test_app.app.world_mut().insert_resource(LocalPlayer(0));
        test_app.app.world_mut().init_resource::<crate::game::world::types::MapStartingPositions>();

        // Spawn a deployment center for player 0 at a known position
        let dc_pos = Vec3::new(100.0, 0.0, 200.0);
        test_app.app.world_mut().spawn((
            ObjectInstance { object_type: ObjectEnum::DeploymentCenter, hp: None, max_hp: None },
            Owner::player(0),
            Transform::from_translation(dc_pos),
        ));

        // TestApp already spawns a MainCamera at (0, 40, 25)
        test_app.app.world_mut().run_system_once(center_camera_on_start).unwrap();

        let mut cam_query = test_app.app.world_mut().query_filtered::<&Transform, With<MainCamera>>();
        let cam = cam_query.single(test_app.app.world()).unwrap();
        assert_eq!(cam.translation.x, 100.0);
        let expected_z = 200.0 + 40.0 * 25.0 / 40.0;
        assert_eq!(cam.translation.z, expected_z);
    }

    #[test]
    fn test_center_camera_on_start_syndicate() {
        use crate::testing::TestApp;
        use crate::types::{MainCamera, FactionEnum, LocalPlayer};
        use crate::game::types::ObjectInstance;

        let mut test_app = TestApp::new_with_faction(FactionEnum::TheSyndicate);
        test_app.app.world_mut().insert_resource(LocalPlayer(0));
        test_app.app.world_mut().init_resource::<crate::game::world::types::MapStartingPositions>();

        // Spawn a tunnel for player 0 at a known position
        let tunnel_pos = Vec3::new(50.0, 0.0, 80.0);
        test_app.app.world_mut().spawn((
            ObjectInstance { object_type: ObjectEnum::Tunnel, hp: None, max_hp: None },
            Owner::player(0),
            Transform::from_translation(tunnel_pos),
        ));

        // TestApp already spawns a MainCamera at (0, 40, 25)
        test_app.app.world_mut().run_system_once(center_camera_on_start).unwrap();

        let mut cam_query = test_app.app.world_mut().query_filtered::<&Transform, With<MainCamera>>();
        let cam = cam_query.single(test_app.app.world()).unwrap();
        assert_eq!(cam.translation.x, 50.0);
        let expected_z = 80.0 + 40.0 * 25.0 / 40.0;
        assert_eq!(cam.translation.z, expected_z);
    }

    #[test]
    fn test_center_camera_on_start_cults() {
        use crate::testing::TestApp;
        use crate::types::{MainCamera, FactionEnum, LocalPlayer};
        use crate::game::types::ObjectInstance;

        let mut test_app = TestApp::new_with_faction(FactionEnum::TheCults);
        test_app.app.world_mut().insert_resource(LocalPlayer(0));
        test_app.app.world_mut().init_resource::<crate::game::world::types::MapStartingPositions>();

        // Spawn a recruitment center for player 0 at a known position
        let rc_pos = Vec3::new(18.5, 0.0, 18.5);
        test_app.app.world_mut().spawn((
            ObjectInstance { object_type: ObjectEnum::RecruitmentCenter, hp: None, max_hp: None },
            Owner::player(0),
            Transform::from_translation(rc_pos),
        ));

        // TestApp already spawns a MainCamera at (0, 40, 25)
        test_app.app.world_mut().run_system_once(center_camera_on_start).unwrap();

        let mut cam_query = test_app.app.world_mut().query_filtered::<&Transform, With<MainCamera>>();
        let cam = cam_query.single(test_app.app.world()).unwrap();
        assert_eq!(cam.translation.x, 18.5);
        let expected_z = 18.5 + 40.0 * 25.0 / 40.0;
        assert_eq!(cam.translation.z, expected_z);
    }

    #[test]
    fn test_center_camera_on_start_no_structure() {
        use crate::testing::TestApp;
        use crate::types::{MainCamera, FactionEnum, LocalPlayer};

        let mut test_app = TestApp::new_with_faction(FactionEnum::GlobalDefenseOrdinance);
        test_app.app.world_mut().insert_resource(LocalPlayer(0));
        test_app.app.world_mut().init_resource::<crate::game::world::types::MapStartingPositions>();

        // No DC spawned — system should be a no-op. TestApp camera at (0, 40, 25).
        test_app.app.world_mut().run_system_once(center_camera_on_start).unwrap();

        let mut cam_query = test_app.app.world_mut().query_filtered::<&Transform, With<MainCamera>>();
        let cam = cam_query.single(test_app.app.world()).unwrap();
        // Camera should remain at original position
        assert_eq!(cam.translation.x, 0.0);
    }

    #[test]
    fn test_center_camera_on_start_map_position_override() {
        use crate::testing::TestApp;
        use crate::types::{MainCamera, FactionEnum, LocalPlayer};
        use crate::game::types::ObjectInstance;
        use crate::game::world::types::MapStartingPositions;
        use crate::game::world::utils::grid_to_world;

        let mut test_app = TestApp::new_with_faction(FactionEnum::GlobalDefenseOrdinance);
        test_app.app.world_mut().insert_resource(LocalPlayer(0));

        // Set a map starting position for player 0
        let mut map_pos = MapStartingPositions::default();
        map_pos.positions.insert(0, (40, 50));
        test_app.app.world_mut().insert_resource(map_pos);

        // Also spawn a DC — map position should take priority
        let dc_pos = Vec3::new(100.0, 0.0, 200.0);
        test_app.app.world_mut().spawn((
            ObjectInstance { object_type: ObjectEnum::DeploymentCenter, hp: None, max_hp: None },
            Owner::player(0),
            Transform::from_translation(dc_pos),
        ));

        test_app.app.world_mut().run_system_once(center_camera_on_start).unwrap();

        let mut cam_query = test_app.app.world_mut().query_filtered::<&Transform, With<MainCamera>>();
        let cam = cam_query.single(test_app.app.world()).unwrap();

        let expected_world = grid_to_world(40, 50, 1.0);
        assert_eq!(cam.translation.x, expected_world.x);
        let expected_z = expected_world.z + 40.0 * 25.0 / 40.0;
        assert_eq!(cam.translation.z, expected_z);
        // Confirm it did NOT use the DC position
        assert_ne!(cam.translation.x, 100.0);
    }

    #[test]
    fn test_center_camera_on_start_map_position_no_matching_slot() {
        use crate::testing::TestApp;
        use crate::types::{MainCamera, FactionEnum, LocalPlayer};
        use crate::game::types::ObjectInstance;
        use crate::game::world::types::MapStartingPositions;

        let mut test_app = TestApp::new_with_faction(FactionEnum::GlobalDefenseOrdinance);
        test_app.app.world_mut().insert_resource(LocalPlayer(0));

        // Set a map starting position for player 1 (not local player 0)
        let mut map_pos = MapStartingPositions::default();
        map_pos.positions.insert(1, (40, 50));
        test_app.app.world_mut().insert_resource(map_pos);

        // Spawn a DC for player 0 — should fall through to structure-based centering
        let dc_pos = Vec3::new(100.0, 0.0, 200.0);
        test_app.app.world_mut().spawn((
            ObjectInstance { object_type: ObjectEnum::DeploymentCenter, hp: None, max_hp: None },
            Owner::player(0),
            Transform::from_translation(dc_pos),
        ));

        test_app.app.world_mut().run_system_once(center_camera_on_start).unwrap();

        let mut cam_query = test_app.app.world_mut().query_filtered::<&Transform, With<MainCamera>>();
        let cam = cam_query.single(test_app.app.world()).unwrap();
        // Should use the DC position, not the map position for slot 1
        assert_eq!(cam.translation.x, 100.0);
        let expected_z = 200.0 + 40.0 * 25.0 / 40.0;
        assert_eq!(cam.translation.z, expected_z);
    }

    #[test]
    fn test_center_camera_on_start_map_position_grid_conversion() {
        use crate::testing::TestApp;
        use crate::types::{MainCamera, FactionEnum, LocalPlayer};
        use crate::game::world::types::MapStartingPositions;
        use crate::game::world::utils::grid_to_world;

        let mut test_app = TestApp::new_with_faction(FactionEnum::GlobalDefenseOrdinance);
        test_app.app.world_mut().insert_resource(LocalPlayer(0));

        // Use grid center (32, 32) which should map to world (0.5, 0.0, 0.5)
        let mut map_pos = MapStartingPositions::default();
        map_pos.positions.insert(0, (32, 32));
        test_app.app.world_mut().insert_resource(map_pos);

        test_app.app.world_mut().run_system_once(center_camera_on_start).unwrap();

        let mut cam_query = test_app.app.world_mut().query_filtered::<&Transform, With<MainCamera>>();
        let cam = cam_query.single(test_app.app.world()).unwrap();

        let expected_world = grid_to_world(32, 32, 1.0);
        assert_eq!(cam.translation.x, expected_world.x);
        assert_eq!(cam.translation.x, 0.5); // (32 - 32) * 1.0 + 0.5
        let expected_z = expected_world.z + 40.0 * 25.0 / 40.0;
        assert_eq!(cam.translation.z, expected_z);
    }
}
