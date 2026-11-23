use bevy::prelude::*;
use crate::types::{Owner, GridPosition};
use crate::game::types::*;
use crate::game::world::types::{Tile, TilePreset};
use crate::game::units::types::state::UnitCommand;
use crate::ui::types::ArmoryEjectionQueue;

/// System to process tunnel ejection queues and eject requests.
/// Each tick: process pending EjectRequest markers, decrement cooldowns,
/// eject one unit per tunnel when cooldown expires.
pub fn ejection_tick_system(
    mut commands: Commands,
    mut tunnel_query: Query<(Entity, &Transform, &StructureInstance, &Owner, &mut crate::ui::types::EjectionQueue, Option<&crate::ui::types::EjectRequest>)>,
    network_units: Query<(Entity, &ObjectInstance, &crate::game::units::types::state::behavior::InTunnelNetwork)>,
) {
    use crate::game::units::utils::tunnel_side_world_position;
    use crate::game::units::types::state::behavior::InTunnelNetwork;

    for (tunnel_entity, tunnel_tf, tunnel_si, owner, mut ejection_queue, eject_request) in tunnel_query.iter_mut() {
        // Process pending EjectRequest: find a matching unit in the network and add to queue
        if let Some(request) = eject_request {
            let owner_player = owner.player_number().unwrap_or(0);

            // Find first matching unit in the network owned by the same player
            let matching_unit = network_units.iter()
                .find(|(_, obj, in_net)| {
                    obj.object_type == request.unit_type && in_net.owner_player == owner_player
                })
                .map(|(entity, _, _)| entity);

            if let Some(unit_entity) = matching_unit {
                ejection_queue.queue.push_back(unit_entity);
                info!("Tunnel: Queued {:?} for ejection (queue len: {})", request.unit_type, ejection_queue.queue.len());
            } else {
                info!("Tunnel: No {:?} unit found in network for player {}", request.unit_type, owner_player);
            }

            // Remove the request marker
            commands.entity(tunnel_entity).remove::<crate::ui::types::EjectRequest>();
        }

        // Process ejection cooldown
        if ejection_queue.cooldown > 0 {
            ejection_queue.cooldown -= 1;
            continue;
        }

        // Eject next unit from queue
        if let Some(unit_entity) = ejection_queue.queue.pop_front() {
            // Verify entity still exists and is in network
            if let Ok((_, _, _)) = network_units.get(unit_entity) {
                // Compute Side A position
                let side_a_pos = tunnel_side_world_position(tunnel_tf, tunnel_si, 'A');

                // Teleport unit to Side A, make visible, remove InTunnelNetwork
                commands.entity(unit_entity)
                    .remove::<InTunnelNetwork>()
                    .insert(Visibility::Visible)
                    .insert(Transform::from_translation(Vec3::new(side_a_pos.x, 0.5, side_a_pos.z)));

                // Issue a move command away from the tunnel exit to clear the way
                let move_away = Vec3::new(side_a_pos.x, 0.5, side_a_pos.z - 2.0); // Move slightly away from Side A
                commands.entity(unit_entity)
                    .insert(UnitCommand::Move(move_away));

                // Reset cooldown (8 frames minimum between ejections)
                ejection_queue.cooldown = 8;

                info!("Tunnel: Ejected unit {:?} at Side A ({:.1}, {:.1})", unit_entity, side_a_pos.x, side_a_pos.z);
            } else {
                // Entity no longer valid — skip and try next
                info!("Tunnel: Ejection skipped — unit {:?} no longer in network", unit_entity);
            }
        }
    }
}

/// System to eject Recruits from Armory one at a time with a cooldown.
/// Runs in FixedUpdate.
pub fn armory_eject_tick_system(
    mut commands: Commands,
    mut armory_query: Query<(Entity, &Transform, &StructureInstance, &mut ArmoryEjectionQueue)>,
) {
    for (entity, transform, si, mut ejection_queue) in armory_query.iter_mut() {
        if ejection_queue.cooldown > 0 {
            ejection_queue.cooldown -= 1;
            continue;
        }
        if let Some(recruit_entity) = ejection_queue.queue.pop_front() {
            let exit_pos = armory_exit_side_position(transform, si);
            // Move the recruit slightly further from the exit to avoid stacking
            let eject_offset = Vec3::new(0.0, 0.0, 1.5);
            if commands.get_entity(recruit_entity).is_ok() {
                commands.entity(recruit_entity)
                    .insert(Visibility::Inherited)
                    .insert(Transform::from_translation(exit_pos))
                    .insert(UnitCommand::Move(exit_pos + eject_offset));
            }
            ejection_queue.cooldown = 8; // 8-frame spacing like tunnel ejection
        } else {
            // Queue empty — remove the component
            commands.entity(entity).remove::<ArmoryEjectionQueue>();
        }
    }
}

/// Compute the exit (Side C) world position for an Armory.
/// The Armory is 3x2 with ABCB symmetry: N=A, E=B, S=C, W=B.
/// Side C is opposite Side A. For a 3x2 building, the half-sizes are:
/// width = 1.5 (x-axis), depth = 1.0 (z-axis). Exit offset is depth + 0.5 = 1.5.
pub(super) fn armory_exit_side_position(
    transform: &Transform,
    structure_instance: &StructureInstance,
) -> Vec3 {
    use crate::types::SymmetryTypeEnum;

    let labels = structure_instance.oriented_labels(SymmetryTypeEnum::ABCB);
    // Find which cardinal direction (N=0, E=1, S=2, W=3) has Side C
    let cardinal_index = labels.iter().position(|&c| c == 'C').unwrap_or(2);

    let center = transform.translation;
    // For a 3x2 building: half-width=1.5, half-depth=1.0
    // Side C exit offset depends on which cardinal direction it faces
    let offset = match cardinal_index {
        0 | 2 => 1.5, // North/South face (depth side: 1.0 + 0.5)
        1 | 3 => 2.0, // East/West face (width side: 1.5 + 0.5)
        _ => 1.5,
    };

    match cardinal_index {
        0 => Vec3::new(center.x, center.y, center.z - offset), // North
        1 => Vec3::new(center.x + offset, center.y, center.z), // East
        2 => Vec3::new(center.x, center.y, center.z + offset), // South
        3 => Vec3::new(center.x - offset, center.y, center.z), // West
        _ => center,
    }
}

// =====================================================
// RECRUITMENT AREA TILE CLAIMING SYSTEM
// =====================================================

/// System that claims recruitable tiles for Recruitment Centers.
/// Uses approach B: stale claim cleanup at start (self-healing).
/// Priority is by build_order (ascending = first-built has priority).
pub fn recruitment_tile_claiming_system(
    mut claim_map: ResMut<crate::game::world::types::TileClaimMap>,
    mut rc_query: Query<(Entity, &GridPosition, &mut RecruitmentCenterState, &ObjectInstance)>,
    tiles: Query<(&GridPosition, &TilePreset), With<Tile>>,
) {
    // Step 1: Remove stale claims (entities that are no longer alive)
    let stale_entities: Vec<Entity> = {
        let mut stale = Vec::new();
        for (_, &claimer) in claim_map.claims.iter() {
            if !stale.contains(&claimer) {
                let is_alive = rc_query.iter().any(|(e, _, _, obj)| e == claimer && obj.is_alive());
                if !is_alive {
                    stale.push(claimer);
                }
            }
        }
        stale
    };
    for entity in stale_entities {
        claim_map.unclaim_all_for(entity);
    }

    // Step 2: Build a recruitable tile lookup from the tile query
    let recruitable_tiles: std::collections::HashSet<(i32, i32)> = tiles
        .iter()
        .filter(|(_, preset)| preset.recruitable)
        .map(|(gp, _)| (gp.x, gp.z))
        .collect();

    // Step 3: Collect RCs sorted by build_order (ascending)
    let mut rcs: Vec<(Entity, i32, i32, u64)> = rc_query
        .iter()
        .filter(|(_, _, _, obj)| obj.is_alive())
        .map(|(e, gp, state, _)| (e, gp.x, gp.z, state.build_order))
        .collect();
    rcs.sort_by_key(|&(_, _, _, order)| order);

    // Step 4: Clear all existing claims and reclaim from scratch each frame
    claim_map.claims.clear();

    // Step 5: For each RC in build_order priority, claim tiles
    let mut claimed_per_entity: std::collections::HashMap<Entity, Vec<(i32, i32)>> =
        std::collections::HashMap::new();

    for (entity, gx, gz, _order) in &rcs {
        let mut claimed = Vec::new();
        // 10x10 area centered on 4x4 footprint: extends 3 tiles beyond each edge
        let x_min = (gx - 3).max(0);
        let x_max = (gx + 6).min(63);
        let z_min = (gz - 3).max(0);
        let z_max = (gz + 6).min(63);

        for x in x_min..=x_max {
            for z in z_min..=z_max {
                let pos = (x, z);
                if recruitable_tiles.contains(&pos) && claim_map.is_claimed(pos).is_none() {
                    claim_map.claim_tile(pos, *entity);
                    claimed.push(pos);
                }
            }
        }
        claimed_per_entity.insert(*entity, claimed);
    }

    // Step 6: Update RC state with claimed tiles, effectiveness, and local_capacity
    for (entity, _, mut rc_state, _) in rc_query.iter_mut() {
        if let Some(claimed) = claimed_per_entity.remove(&entity) {
            let effectiveness = claimed.len() as f32 / 100.0;
            let local_capacity = (20.0 * effectiveness).floor() as u32;
            rc_state.claimed_tiles = claimed;
            rc_state.effectiveness = effectiveness;
            rc_state.local_capacity = local_capacity;
        } else {
            // Dead or not found — clear
            rc_state.claimed_tiles.clear();
            rc_state.effectiveness = 0.0;
            rc_state.local_capacity = 0;
        }
    }
}

// =====================================================
// CULTS UNIT CONTROL AGGREGATION SYSTEM
// =====================================================

/// Aggregates unit control capacity and usage from all Recruitment Centers
/// into the owning player's CultsPlayerResources each frame.
pub fn cults_unit_control_aggregation_system(
    rc_query: Query<(&Owner, &RecruitmentCenterState)>,
    mut players: Query<(&Player, &mut CultsPlayerResources)>,
) {
    for (player, mut res) in players.iter_mut() {
        let mut total_capacity: u32 = 0;
        let mut total_used: u32 = 0;

        for (owner, rc_state) in rc_query.iter() {
            if owner.player_number() == Some(player.player_number) {
                total_capacity += rc_state.local_capacity;
                total_used += rc_state.local_used;
            }
        }

        res.unit_control_available = total_capacity;
        res.unit_control_used = total_used;
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use bevy::ecs::system::RunSystemOnce;
    use crate::types::{ObjectEnum, FactionEnum, InvisibleEntity};
    use crate::game::world::types::TilePresetEnum;

    /// Helper: spawn a minimal tile entity at the given grid position with the given preset
    fn spawn_test_tile(world: &mut World, x: i32, z: i32, preset_enum: TilePresetEnum) -> Entity {
        let preset = preset_enum.properties();
        world.spawn((
            GridPosition { x, z },
            preset,
            Tile,
        )).id()
    }

    /// Helper: spawn a minimal Recruitment Center entity
    fn spawn_test_rc(world: &mut World, x: i32, z: i32, build_order: u64) -> Entity {
        world.spawn((
            GridPosition { x, z },
            RecruitmentCenterState {
                build_order,
                ..Default::default()
            },
            ObjectInstance::destructible(ObjectEnum::RecruitmentCenter, crate::game::types::structures::cults_structure_stats::RC_MAX_HP),
        )).id()
    }

    #[test]
    fn tile_claim_map_basic_operations() {
        use crate::game::world::types::TileClaimMap;
        let mut map = TileClaimMap::default();
        let e1 = Entity::from_raw_u32(1).unwrap();
        let e2 = Entity::from_raw_u32(2).unwrap();

        // Claim and check
        map.claim_tile((5, 5), e1);
        assert_eq!(map.is_claimed((5, 5)), Some(e1));
        assert_eq!(map.is_claimed((6, 6)), None);

        // Unclaim single
        map.unclaim_tile((5, 5));
        assert_eq!(map.is_claimed((5, 5)), None);

        // Unclaim all for entity
        map.claim_tile((1, 1), e1);
        map.claim_tile((2, 2), e1);
        map.claim_tile((3, 3), e2);
        map.unclaim_all_for(e1);
        assert_eq!(map.is_claimed((1, 1)), None);
        assert_eq!(map.is_claimed((2, 2)), None);
        assert_eq!(map.is_claimed((3, 3)), Some(e2));
    }

    #[test]
    fn rc_all_recruitable_terrain_full_effectiveness() {
        use crate::game::world::types::TileClaimMap;

        let mut app = App::new();
        app.init_resource::<TileClaimMap>();

        // Spawn a 10x10 area of recruitable tiles centered on RC at (10, 10)
        // RC footprint: [10..13] x [10..13], area: [7..16] x [7..16]
        for x in 7..=16 {
            for z in 7..=16 {
                spawn_test_tile(app.world_mut(), x, z, TilePresetEnum::Plane);
            }
        }

        // Spawn RC
        spawn_test_rc(app.world_mut(), 10, 10, 0);

        app.world_mut().run_system_once(recruitment_tile_claiming_system).unwrap();

        // Check state
        let mut query = app.world_mut().query::<&RecruitmentCenterState>();
        let state = query.single(app.world()).unwrap();
        assert_eq!(state.claimed_tiles.len(), 100);
        assert!((state.effectiveness - 1.0).abs() < f32::EPSILON);
        assert_eq!(state.local_capacity, 20);
    }

    #[test]
    fn rc_half_recruitable_terrain_half_effectiveness() {
        use crate::game::world::types::TileClaimMap;

        let mut app = App::new();
        app.init_resource::<TileClaimMap>();

        // RC at (10, 10), area [7..16] x [7..16]
        // Make half the tiles Water (non-recruitable)
        for x in 7..=16 {
            for z in 7..=16 {
                if z <= 11 {
                    // 5 rows of 10 = 50 recruitable
                    spawn_test_tile(app.world_mut(), x, z, TilePresetEnum::Plane);
                } else {
                    // 5 rows of 10 = 50 non-recruitable
                    spawn_test_tile(app.world_mut(), x, z, TilePresetEnum::Water);
                }
            }
        }

        spawn_test_rc(app.world_mut(), 10, 10, 0);

        app.world_mut().run_system_once(recruitment_tile_claiming_system).unwrap();

        let mut query = app.world_mut().query::<&RecruitmentCenterState>();
        let state = query.single(app.world()).unwrap();
        assert_eq!(state.claimed_tiles.len(), 50);
        assert!((state.effectiveness - 0.5).abs() < f32::EPSILON);
        assert_eq!(state.local_capacity, 10);
    }

    #[test]
    fn rc_overlapping_areas_first_built_has_priority() {
        use crate::game::world::types::TileClaimMap;

        let mut app = App::new();
        app.init_resource::<TileClaimMap>();

        // Two RCs with overlapping areas. Both at same z, close together in x.
        // RC1 at (10, 10), area [7..16] x [7..16], build_order=0
        // RC2 at (14, 10), area [11..20] x [7..16], build_order=1
        // Overlap: x in [11..16], z in [7..16] = 6 * 10 = 60 tiles

        // Spawn all tiles in combined area
        for x in 7..=20 {
            for z in 7..=16 {
                spawn_test_tile(app.world_mut(), x, z, TilePresetEnum::Plane);
            }
        }

        let rc1 = spawn_test_rc(app.world_mut(), 10, 10, 0);
        let _rc2 = spawn_test_rc(app.world_mut(), 14, 10, 1);

        app.world_mut().run_system_once(recruitment_tile_claiming_system).unwrap();

        // RC1 gets all 100 tiles (its full area)
        // RC2 gets: area [11..20]x[7..16] = 10*10=100 tiles, minus 60 overlap claimed by RC1 = 40
        let mut query = app.world_mut().query::<(Entity, &RecruitmentCenterState)>();
        let mut found_rc1 = false;
        let mut found_rc2 = false;
        for (entity, state) in query.iter(app.world()) {
            if entity == rc1 {
                assert_eq!(state.claimed_tiles.len(), 100);
                assert!((state.effectiveness - 1.0).abs() < f32::EPSILON);
                assert_eq!(state.local_capacity, 20);
                found_rc1 = true;
            } else {
                assert_eq!(state.claimed_tiles.len(), 40);
                assert!((state.effectiveness - 0.4).abs() < f32::EPSILON);
                assert_eq!(state.local_capacity, 8);
                found_rc2 = true;
            }
        }
        assert!(found_rc1 && found_rc2);
    }

    #[test]
    fn rc_destruction_frees_tiles_for_others() {
        use crate::game::world::types::TileClaimMap;

        let mut app = App::new();
        app.init_resource::<TileClaimMap>();

        // Same overlapping setup
        for x in 7..=20 {
            for z in 7..=16 {
                spawn_test_tile(app.world_mut(), x, z, TilePresetEnum::Plane);
            }
        }

        let rc1 = spawn_test_rc(app.world_mut(), 10, 10, 0);
        let _rc2 = spawn_test_rc(app.world_mut(), 14, 10, 1);

        // Run claiming once
        app.world_mut().run_system_once(recruitment_tile_claiming_system).unwrap();

        // Kill RC1
        let mut obj_query = app.world_mut().query::<&mut ObjectInstance>();
        let mut rc1_obj = obj_query.get_mut(app.world_mut(), rc1).unwrap();
        rc1_obj.apply_damage(crate::game::types::structures::cults_structure_stats::RC_MAX_HP + 1.0);

        // Run claiming again
        app.world_mut().run_system_once(recruitment_tile_claiming_system).unwrap();

        // RC2 should now get all 100 tiles in its area
        let mut state_query = app.world_mut().query::<(Entity, &RecruitmentCenterState)>();
        for (entity, state) in state_query.iter(app.world()) {
            if entity != rc1 {
                assert_eq!(state.claimed_tiles.len(), 100);
                assert!((state.effectiveness - 1.0).abs() < f32::EPSILON);
                assert_eq!(state.local_capacity, 20);
            }
        }

        // Claim map should have no claims for RC1
        let claim_map = app.world().resource::<TileClaimMap>();
        assert!(claim_map.claims.values().all(|&e| e != rc1));
    }

    #[test]
    fn rc_near_map_edge_clamps_area() {
        use crate::game::world::types::TileClaimMap;

        let mut app = App::new();
        app.init_resource::<TileClaimMap>();

        // RC at (0, 0) — area would be [-3..6] x [-3..6], clamped to [0..6] x [0..6] = 7*7 = 49 tiles
        for x in 0..=6 {
            for z in 0..=6 {
                spawn_test_tile(app.world_mut(), x, z, TilePresetEnum::Plane);
            }
        }

        spawn_test_rc(app.world_mut(), 0, 0, 0);

        app.world_mut().run_system_once(recruitment_tile_claiming_system).unwrap();

        let mut query = app.world_mut().query::<&RecruitmentCenterState>();
        let state = query.single(app.world()).unwrap();
        assert_eq!(state.claimed_tiles.len(), 49);
        assert!((state.effectiveness - 0.49).abs() < f32::EPSILON);
        // floor(20 * 0.49) = floor(9.8) = 9
        assert_eq!(state.local_capacity, 9);
    }

    // =====================================================
    // CULTS UNIT CONTROL AGGREGATION TESTS
    // =====================================================

    #[test]
    fn aggregation_sums_capacity_across_rcs() {
        let mut app = App::new();
        app.add_plugins(MinimalPlugins);

        // Spawn player entity with CultsPlayerResources
        app.world_mut().spawn((
            InvisibleEntity,
            Player::new("Test Player", FactionEnum::TheCults, 0),
            CultsPlayerResources::default(),
        ));

        // Spawn two RCs owned by player 0
        app.world_mut().spawn((
            Owner::player(0),
            RecruitmentCenterState {
                local_capacity: 20,
                local_used: 5,
                effectiveness: 1.0,
                ..Default::default()
            },
        ));
        app.world_mut().spawn((
            Owner::player(0),
            RecruitmentCenterState {
                local_capacity: 10,
                local_used: 3,
                effectiveness: 0.5,
                ..Default::default()
            },
        ));

        app.world_mut().run_system_once(cults_unit_control_aggregation_system).unwrap();

        let mut query = app.world_mut().query::<&CultsPlayerResources>();
        let res = query.single(app.world()).unwrap();
        assert_eq!(res.unit_control_available, 30, "Total capacity should be 20 + 10");
        assert_eq!(res.unit_control_used, 8, "Total used should be 5 + 3");
    }

    #[test]
    fn aggregation_ignores_other_players_rcs() {
        let mut app = App::new();
        app.add_plugins(MinimalPlugins);

        // Spawn player 0
        app.world_mut().spawn((
            InvisibleEntity,
            Player::new("Player 0", FactionEnum::TheCults, 0),
            CultsPlayerResources::default(),
        ));

        // Spawn RC owned by player 1 (different player)
        app.world_mut().spawn((
            Owner::player(1),
            RecruitmentCenterState {
                local_capacity: 20,
                local_used: 10,
                effectiveness: 1.0,
                ..Default::default()
            },
        ));

        app.world_mut().run_system_once(cults_unit_control_aggregation_system).unwrap();

        let mut query = app.world_mut().query::<&CultsPlayerResources>();
        let res = query.single(app.world()).unwrap();
        assert_eq!(res.unit_control_available, 0, "Should not count other player's RC");
        assert_eq!(res.unit_control_used, 0);
    }

    #[test]
    fn armory_exit_side_position_r0_is_south() {
        let transform = Transform::from_xyz(10.0, 0.0, 10.0);
        let si = StructureInstance {
            rotation: crate::types::StructureRotation::R0,
            flip_horizontal: false,
            flip_vertical: false,
        };
        let pos = armory_exit_side_position(&transform, &si);
        // ABCB symmetry at R0: N=A, E=B, S=C, W=B. Side C is South.
        assert!(pos.z > 10.0, "Exit should be south of center, got z={}", pos.z);
        assert!((pos.x - 10.0).abs() < 0.01, "Exit should be at same x");
    }

    // === Armory eject system tests ===

    #[test]
    fn armory_eject_removes_queue_when_empty() {
        let mut app = App::new();

        let armory = app.world_mut().spawn((
            Transform::from_xyz(10.0, 0.0, 10.0),
            StructureInstance {
                rotation: crate::types::StructureRotation::R0,
                flip_horizontal: false,
                flip_vertical: false,
            },
            ArmoryEjectionQueue {
                queue: std::collections::VecDeque::new(),
                cooldown: 0,
            },
        )).id();

        app.world_mut().run_system_once(armory_eject_tick_system).unwrap();

        // Empty queue should be removed
        assert!(app.world().get::<ArmoryEjectionQueue>(armory).is_none());
    }

    #[test]
    fn armory_eject_sets_cooldown() {
        let mut app = App::new();

        // Create a recruit entity to eject
        let recruit = app.world_mut().spawn((
            Transform::from_xyz(0.0, 0.0, 0.0),
            Visibility::Hidden,
            UnitCommand::Idle,
        )).id();

        let mut queue = std::collections::VecDeque::new();
        queue.push_back(recruit);

        let armory = app.world_mut().spawn((
            Transform::from_xyz(10.0, 0.0, 10.0),
            StructureInstance {
                rotation: crate::types::StructureRotation::R0,
                flip_horizontal: false,
                flip_vertical: false,
            },
            ArmoryEjectionQueue {
                queue,
                cooldown: 0,
            },
        )).id();

        app.world_mut().run_system_once(armory_eject_tick_system).unwrap();

        let ejection = app.world().get::<ArmoryEjectionQueue>(armory);
        // Should have cooldown set after ejecting
        if let Some(eq) = ejection {
            assert_eq!(eq.cooldown, 8);
        }
        // Queue should now be empty (but component exists due to cooldown)
    }

    #[test]
    fn armory_eject_respects_cooldown() {
        let mut app = App::new();

        let recruit = app.world_mut().spawn((
            Transform::from_xyz(0.0, 0.0, 0.0),
            Visibility::Hidden,
            UnitCommand::Idle,
        )).id();

        let mut queue = std::collections::VecDeque::new();
        queue.push_back(recruit);

        let armory = app.world_mut().spawn((
            Transform::from_xyz(10.0, 0.0, 10.0),
            StructureInstance {
                rotation: crate::types::StructureRotation::R0,
                flip_horizontal: false,
                flip_vertical: false,
            },
            ArmoryEjectionQueue {
                queue,
                cooldown: 5,
            },
        )).id();

        app.world_mut().run_system_once(armory_eject_tick_system).unwrap();

        // Cooldown should have decremented but recruit not ejected
        let ejection = app.world().get::<ArmoryEjectionQueue>(armory).unwrap();
        assert_eq!(ejection.cooldown, 4);
        assert_eq!(ejection.queue.len(), 1);
    }
}
