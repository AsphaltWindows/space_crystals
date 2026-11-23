use bevy::prelude::*;
use crate::types::{Owner, ObjectEnum};
use crate::game::types::*;
use crate::game::entities::{EntityRegistry, SpawnParams};
use crate::game::entities::utils::{insert_entity_visuals, grid_to_spawn_position};
use crate::game::world::types::GdoBuildArea;
use crate::game::units::types::state::UnitCommand;
use super::power::get_power_ratio_for_owner;

// =====================================================
// DEPLOYMENT CENTER CONSTRUCTION SYSTEM
// =====================================================

/// System that ticks DC construction progress each simulation frame
pub fn dc_construction_tick_system(
    mut dc_query: Query<(&Owner, &mut DeploymentCenterState)>,
    players: Query<(&Player, &mut GdoPlayerResources)>,
    registry: Res<EntityRegistry>,
) {
    for (owner, mut dc_state) in dc_query.iter_mut() {
        if let Some(building_type) = dc_state.current_construction {
            if let Some(cost) = registry.cost(&building_type) {
                let required_frames = cost.build_frames as f32;

                // Get power ratio for this player
                let power_ratio = get_power_ratio_for_owner(owner, &players);

                if let Some(ref mut progress) = dc_state.construction_progress {
                    *progress += power_ratio;

                    if *progress >= required_frames {
                        // Construction complete — ready to place
                        dc_state.ready_to_place = Some(building_type);
                        dc_state.current_construction = None;
                        dc_state.construction_progress = None;
                        info!("DC: {:?} construction complete, ready to place", building_type);
                    }
                }
            }
        }
    }
}

// =====================================================
// EXTRACTION FACILITY CONSTRUCTION SYSTEM
// =====================================================

/// System that ticks ExtractionFacility construction progress
pub fn ef_construction_tick_system(
    mut ef_query: Query<(&Owner, &mut ExtractionFacilityState)>,
    players: Query<(&Player, &mut GdoPlayerResources)>,
    registry: Res<EntityRegistry>,
) {
    for (owner, mut ef_state) in ef_query.iter_mut() {
        if ef_state.current_construction {
            let Some(cost) = registry.cost(&ObjectEnum::ExtractionPlate) else { continue };
            let required_frames = cost.build_frames as f32;
            let power_ratio = get_power_ratio_for_owner(owner, &players);

            if let Some(ref mut progress) = ef_state.construction_progress {
                *progress += power_ratio;

                if *progress >= required_frames {
                    ef_state.ready_to_place = true;
                    ef_state.current_construction = false;
                    ef_state.construction_progress = None;
                    info!("EF: ExtractionPlate construction complete, ready to place");
                }
            }
        }
    }
}

// =====================================================
// CONSTRUCTION HP SYSTEM
// =====================================================

/// System that ticks ConstructionHP progress and scales HP accordingly.
/// Each frame, progress increments by 1.0 / build_frames.
/// HP = MaxHP * (10% + 90% * progress).
/// When progress reaches 1.0, the ConstructionHP component is removed.
pub fn construction_hp_tick_system(
    mut commands: Commands,
    mut query: Query<(Entity, &mut ObjectInstance, &mut ConstructionHP), Without<CultsConstructionState>>,
) {
    for (entity, mut obj, mut construction) in query.iter_mut() {
        let increment = 1.0 / construction.build_frames as f32;
        construction.progress = (construction.progress + increment).min(1.0);

        // Update HP based on progress
        if let Some(max_hp) = obj.max_hp {
            let new_hp = max_hp * ConstructionHP::hp_fraction(construction.progress);
            // Only set HP if not damaged below the expected value
            // (allow damage to reduce HP below the construction curve)
            if let Some(current_hp) = obj.hp {
                if current_hp < new_hp {
                    // Don't heal past damage — only raise HP if it's at or above
                    // the previous frame's expected value (i.e., not damaged)
                    let prev_progress = (construction.progress - increment).max(0.0);
                    let prev_expected = max_hp * ConstructionHP::hp_fraction(prev_progress);
                    if current_hp >= prev_expected - 0.001 {
                        obj.hp = Some(new_hp.min(max_hp));
                    }
                }
            }
        }

        // Remove component when construction completes
        if construction.is_complete() {
            commands.entity(entity).remove::<ConstructionHP>();
        }
    }
}

// =====================================================
// TUNNEL CONSTRUCTION TICK SYSTEM
// =====================================================

/// System that ticks Tunnel operations (construction and upgrades) each simulation frame.
/// Construction requires an Agent to be present near the Tunnel (builder_present flag).
/// When construction completes, the expansion entity is spawned.
/// When upgrade completes, the TunnelTier is advanced and TunnelArea recalculated.
pub fn tunnel_construction_tick_system(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    mut tunnel_query: Query<(Entity, &Owner, &mut TunnelState, &mut TunnelArea)>,
    mut syndicate_players: Query<(&Player, &mut SyndicatePlayerResources)>,
    registry: Res<EntityRegistry>,
) {
    for (tunnel_entity, owner, mut tunnel_state, mut tunnel_area) in tunnel_query.iter_mut() {
        let operation = match &tunnel_state.current_operation {
            Some(op) => op.clone(),
            None => continue,
        };

        match operation {
            TunnelOperation::Upgrading { target_tier, progress } => {
                let required = syndicate_structure_stats::TUNNEL_UPGRADE_FRAMES as f32;
                let new_progress = progress + 1.0;

                if new_progress >= required {
                    // Upgrade complete
                    tunnel_state.tier = target_tier;
                    tunnel_state.current_operation = None;
                    tunnel_area.recalculate(&target_tier);

                    // Update tunnel_space_provided for the owning player
                    // Re-sum all tunnels is complex here; just add the difference
                    let old_space = match target_tier {
                        TunnelTier::Tier2 => TunnelTier::Tier1.tunnel_space(),
                        TunnelTier::Tier3 => TunnelTier::Tier2.tunnel_space(),
                        TunnelTier::Tier1 => 0, // shouldn't happen
                    };
                    let new_space = target_tier.tunnel_space();
                    let space_diff = new_space.saturating_sub(old_space);

                    for (player, mut res) in syndicate_players.iter_mut() {
                        if Some(player.player_number) == owner.player_number() {
                            res.tunnel_space_provided += space_diff;
                            break;
                        }
                    }

                    info!("Tunnel upgrade to {:?} complete", target_tier);
                } else {
                    tunnel_state.current_operation = Some(TunnelOperation::Upgrading {
                        target_tier,
                        progress: new_progress,
                    });
                }
            }
            TunnelOperation::BuildingExpansion { object, progress, grid_x, grid_z, rotation, flip_horizontal, flip_vertical } => {
                // TODO: Check Agent presence (builder_present) once enter_command task lands.
                // For now, construction always progresses.
                let required = match object {
                    ObjectEnum::Headquarters => syndicate_structure_stats::HQ_BUILD_FRAMES as f32,
                    _ => syndicate_structure_stats::TUNNEL_CONSTRUCTION_FRAMES as f32,
                };
                let new_progress = progress + 1.0;

                if new_progress >= required {
                    // Construction complete — spawn the expansion entity
                    tunnel_state.current_operation = None;
                    let owner_copy = *owner;
                    match object {
                        ObjectEnum::Headquarters => {
                            let obj_type = object.object_type();
                            let spawn_pos = grid_to_spawn_position(grid_x, grid_z, obj_type.size.0 as f32, obj_type.size.1 as f32);
                            let hq_entity = registry.spawn(&object, &mut commands, SpawnParams {
                                position: spawn_pos,
                                owner: owner_copy,
                                player: owner_copy.player_number().unwrap_or(0),
                                rotation: Some(rotation),
                                under_construction: false,
                                flip_horizontal,
                                flip_vertical,
                            });
                            insert_entity_visuals(&mut commands, hq_entity, &object, &owner_copy, &mut meshes, &mut materials);
                            // HQ needs parent tunnel reference
                            commands.entity(hq_entity).insert(TunnelExpansionMarker { parent_tunnel: tunnel_entity });
                            info!("Tunnel expansion Headquarters spawned at ({}, {})", grid_x, grid_z);
                        }
                        _ => {
                            info!("Tunnel expansion {:?} construction complete (no spawn handler)", object);
                        }
                    }
                } else {
                    tunnel_state.current_operation = Some(TunnelOperation::BuildingExpansion {
                        object,
                        progress: new_progress,
                        grid_x,
                        grid_z,
                        rotation,
                        flip_horizontal,
                        flip_vertical,
                    });
                }
            }
        }
    }
}

// =====================================================
// CULTS CONSTRUCTION TICK SYSTEM
// =====================================================

/// System that ticks Cults building construction progress.
/// Progress scales linearly with the number of assigned Recruits.
/// When complete, all assigned Recruits are despawned and the building becomes operational.
pub fn cults_construction_tick_system(
    mut commands: Commands,
    mut query: Query<(Entity, &mut ObjectInstance, &mut ConstructionHP, &mut CultsConstructionState)>,
) {
    for (entity, mut obj, mut construction, mut cults_state) in query.iter_mut() {
        let recruit_count = cults_state.assigned_recruits.len() as u32;
        if recruit_count == 0 {
            continue; // Paused — no recruits assigned
        }

        cults_state.construction_progress += recruit_count;

        // Update ConstructionHP progress (0.0 to 1.0)
        let progress = (cults_state.construction_progress as f32 / cults_state.total_construction_frames as f32).min(1.0);
        construction.progress = progress;

        // Update HP using ConstructionHP::hp_fraction()
        if let Some(max_hp) = obj.max_hp {
            obj.hp = Some(max_hp * ConstructionHP::hp_fraction(progress));
        }

        // Check completion
        if cults_state.construction_progress >= cults_state.total_construction_frames {
            // Set HP to full max
            if let Some(max_hp) = obj.max_hp {
                obj.hp = Some(max_hp);
            }
            // Despawn all assigned recruits (they are consumed)
            for recruit_entity in cults_state.assigned_recruits.drain(..) {
                commands.entity(recruit_entity).despawn();
            }
            // Remove construction components — building is complete
            commands.entity(entity).remove::<ConstructionHP>();
            commands.entity(entity).remove::<CultsConstructionState>();
        }
    }
}

// =====================================================
// CULTS CONSTRUCTION CANCEL SYSTEM
// =====================================================

/// System that handles cancellation of Cults buildings under construction.
/// When a building is destroyed (HP reaches 0), all assigned Recruits are ejected,
/// restored to visible, and set to Idle near the building's position.
/// Must run BEFORE remove_dead_entities_system so recruits can be ejected first.
pub fn cults_construction_cancel_system(
    mut commands: Commands,
    mut query: Query<(Entity, &ObjectInstance, &Transform, &mut CultsConstructionState)>,
) {
    for (_entity, obj, transform, mut cults_state) in query.iter_mut() {
        if !obj.is_alive() {
            // Building was destroyed — eject all recruits
            let base_pos = transform.translation;
            for (i, recruit_entity) in cults_state.assigned_recruits.drain(..).enumerate() {
                // Offset each recruit slightly from the building center
                let offset_x = ((i % 3) as f32 - 1.0) * 1.5;
                let offset_z = ((i / 3) as f32) * 1.5 + 1.5;
                let eject_pos = Vec3::new(base_pos.x + offset_x, base_pos.y, base_pos.z + offset_z);

                commands.entity(recruit_entity)
                    .insert(Visibility::Inherited)
                    .insert(UnitCommand::Idle)
                    .insert(Transform::from_translation(eject_pos));
            }
        }
    }
}

// =====================================================
// DEBUG KEYBINDING SYSTEMS
// =====================================================

/// Debug system: press keys to trigger construction/production (DEPRECATED - replaced by command panel UI)
/// Kept for reference but no longer registered in any plugin.
#[allow(dead_code)]
fn debug_construction_input(
    keyboard: Res<ButtonInput<KeyCode>>,
    _commands: Commands,
    _meshes: ResMut<Assets<Mesh>>,
    _materials: ResMut<Assets<StandardMaterial>>,
    mut dc_query: Query<(Entity, &Owner, &crate::types::GridPosition, &mut DeploymentCenterState)>,
    mut bk_query: Query<(Entity, &Owner, &mut BarracksState)>,
    mut players: Query<(&Player, &mut GdoPlayerResources)>,
    _build_area: ResMut<GdoBuildArea>,
    registry: Res<EntityRegistry>,
) {
    // DC construction commands
    let build_request = if keyboard.just_pressed(KeyCode::Digit1) {
        Some(ObjectEnum::PowerPlant)
    } else if keyboard.just_pressed(KeyCode::Digit2) {
        Some(ObjectEnum::Barracks)
    } else if keyboard.just_pressed(KeyCode::Digit3) {
        Some(ObjectEnum::ExtractionFacility)
    } else {
        None
    };

    if let Some(object_type) = build_request {
        for (_entity, owner, _grid_pos, mut dc_state) in dc_query.iter_mut() {
            if dc_state.current_construction.is_some() || dc_state.ready_to_place.is_some() {
                info!("DC: Already building or has structure ready to place");
                continue;
            }

            if let Some(cost) = registry.cost(&object_type) {
                // Check player resources
                for (player, mut res) in players.iter_mut() {
                    if Some(player.player_number) == owner.player_number() {
                        if res.space_crystals >= cost.space_crystals as i32 {
                            res.space_crystals -= cost.space_crystals as i32;
                            dc_state.current_construction = Some(object_type);
                            dc_state.construction_progress = Some(0.0);
                            info!("DC: Started building {:?} ({} SC, {} frames)",
                                object_type, cost.space_crystals, cost.build_frames);
                        } else {
                            info!("DC: Not enough SC ({} needed, {} available)",
                                cost.space_crystals, res.space_crystals);
                        }
                        break;
                    }
                }
            }
            break; // Only use first DC
        }
    }

    // F = Enter placement mode (handled via command panel state machine)
    // The old instant-place behavior is replaced by click-to-place via AwaitingPlacement

    // P = Produce Peacekeeper from first Barracks
    if keyboard.just_pressed(KeyCode::KeyP) {
        for (_entity, owner, mut bk_state) in bk_query.iter_mut() {
            if let Some(cost) = registry.cost(&ObjectEnum::Peacekeeper) {
                // Check resources and unit control
                for (player, mut res) in players.iter_mut() {
                    if Some(player.player_number) == owner.player_number() {
                        if res.space_crystals < cost.space_crystals as i32 {
                            info!("Barracks: Not enough SC ({} needed, {} available)",
                                cost.space_crystals, res.space_crystals);
                            break;
                        }
                        if !res.has_unit_control(crate::game::entities::factions::gdo::units::peacekeeper::PEACEKEEPER_CONTROL_COST) {
                            info!("Barracks: Unit control cap reached ({}/{})",
                                res.unit_control_used, res.unit_control_cap);
                            break;
                        }
                        if bk_state.try_queue(ObjectEnum::Peacekeeper) {
                            res.space_crystals -= cost.space_crystals as i32;
                            info!("Barracks: Queued Peacekeeper ({} SC, queue size: {})",
                                cost.space_crystals, bk_state.build_queue.len());
                        } else {
                            info!("Barracks: Queue full (max {})", BarracksState::MAX_QUEUE_SIZE);
                        }
                        break;
                    }
                }
            }
            break; // Only use first Barracks
        }
    }

    // C = Cancel construction/production
    if keyboard.just_pressed(KeyCode::KeyC) {
        // Cancel DC construction
        for (_entity, owner, _grid_pos, mut dc_state) in dc_query.iter_mut() {
            if let Some(building_type) = dc_state.current_construction {
                if let Some(cost) = registry.cost(&building_type) {
                    if let Some(refund) = dc_state.cancellation_refund(&cost) {
                        for (player, mut res) in players.iter_mut() {
                            if Some(player.player_number) == owner.player_number() {
                                res.space_crystals += refund as i32;
                                info!("DC: Cancelled construction, refunded {} SC", refund);
                                break;
                            }
                        }
                    }
                }
                dc_state.current_construction = None;
                dc_state.construction_progress = None;
                break;
            }
            if let Some(ready_type) = &dc_state.ready_to_place.clone() {
                if let Some(cost) = registry.cost(ready_type) {
                    if let Some(refund) = dc_state.cancellation_refund(&cost) {
                        for (player, mut res) in players.iter_mut() {
                            if Some(player.player_number) == owner.player_number() {
                                res.space_crystals += refund as i32;
                                info!("DC: Cancelled ready structure, refunded {} SC (75%)", refund);
                                break;
                            }
                        }
                    }
                }
                dc_state.ready_to_place = None;
                break;
            }
        }

        // Cancel last Barracks queue item
        for (_entity, owner, mut bk_state) in bk_query.iter_mut() {
            if let Some(cancelled) = bk_state.cancel_last() {
                if let Some(cost) = registry.cost(&cancelled) {
                    for (player, mut res) in players.iter_mut() {
                        if Some(player.player_number) == owner.player_number() {
                            res.space_crystals += cost.space_crystals as i32;
                            info!("Barracks: Cancelled {:?}, refunded {} SC", cancelled, cost.space_crystals);
                            break;
                        }
                    }
                }
            }
            break;
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use bevy::ecs::system::RunSystemOnce;

    fn setup_construction_app() -> App {
        let mut app = App::new();
        app.add_plugins(MinimalPlugins);
        app.init_resource::<Assets<Mesh>>();
        app.init_resource::<Assets<StandardMaterial>>();
        app
    }

    /// Helper to spawn a Cults building under construction with CultsConstructionState
    fn spawn_cults_building_under_construction(world: &mut World, total_frames: u32, max_hp: f32) -> Entity {
        world.spawn((
            ObjectInstance::under_construction(ObjectEnum::CultsStorage, max_hp),
            ConstructionHP::new(total_frames),
            CultsConstructionState::new(total_frames),
            Transform::from_xyz(5.0, 0.0, 5.0),
            Owner::player(0),
        )).id()
    }

    /// Helper to spawn a hidden Recruit entity (simulating one that has entered a building)
    fn spawn_hidden_recruit(world: &mut World) -> Entity {
        world.spawn((
            crate::types::Unit,
            ObjectInstance::destructible(ObjectEnum::CultsRecruit, 50.0),
            Owner::player(0),
            UnitCommand::Idle,
            Visibility::Hidden,
            Transform::from_xyz(0.0, 0.0, 0.0),
        )).id()
    }

    #[test]
    fn cults_construction_one_recruit_completes_in_total_frames() {
        let mut app = setup_construction_app();
        let total_frames = 100;
        let max_hp = 200.0;

        let recruit = spawn_hidden_recruit(app.world_mut());
        let building = spawn_cults_building_under_construction(app.world_mut(), total_frames, max_hp);

        // Assign 1 recruit to the building
        app.world_mut().get_mut::<CultsConstructionState>(building).unwrap()
            .assigned_recruits.push(recruit);

        // Run for exactly total_frames ticks
        for _ in 0..total_frames {
            app.world_mut().run_system_once(cults_construction_tick_system).unwrap();
        }
        // Flush deferred commands
        app.update();

        // Building should be complete (no ConstructionHP)
        assert!(app.world().get::<ConstructionHP>(building).is_none(),
            "ConstructionHP should be removed on completion");
        assert!(app.world().get::<CultsConstructionState>(building).is_none(),
            "CultsConstructionState should be removed on completion");

        // HP should be at max
        let obj = app.world().get::<ObjectInstance>(building).unwrap();
        assert_eq!(obj.hp, Some(max_hp));

        // Recruit should be despawned
        assert!(app.world().get_entity(recruit).is_err(),
            "Recruit should be despawned on completion");
    }

    #[test]
    fn cults_construction_two_recruits_complete_in_half_frames() {
        let mut app = setup_construction_app();
        let total_frames = 100;
        let max_hp = 200.0;

        let recruit1 = spawn_hidden_recruit(app.world_mut());
        let recruit2 = spawn_hidden_recruit(app.world_mut());
        let building = spawn_cults_building_under_construction(app.world_mut(), total_frames, max_hp);

        // Assign 2 recruits
        {
            let mut state = app.world_mut().get_mut::<CultsConstructionState>(building).unwrap();
            state.assigned_recruits.push(recruit1);
            state.assigned_recruits.push(recruit2);
        }

        // Run for half the total frames
        for _ in 0..50 {
            app.world_mut().run_system_once(cults_construction_tick_system).unwrap();
        }
        app.update();

        // Building should be complete
        assert!(app.world().get::<ConstructionHP>(building).is_none(),
            "Should complete in N/2 frames with 2 recruits");

        // Both recruits should be despawned
        assert!(app.world().get_entity(recruit1).is_err(), "Recruit 1 should be despawned");
        assert!(app.world().get_entity(recruit2).is_err(), "Recruit 2 should be despawned");
    }

    #[test]
    fn cults_construction_no_recruits_no_progress() {
        let mut app = setup_construction_app();
        let total_frames = 100;
        let max_hp = 200.0;

        let building = spawn_cults_building_under_construction(app.world_mut(), total_frames, max_hp);

        // Run many frames with no recruits assigned
        for _ in 0..200 {
            app.world_mut().run_system_once(cults_construction_tick_system).unwrap();
        }

        // Should still have ConstructionHP (not complete)
        let state = app.world().get::<CultsConstructionState>(building).unwrap();
        assert_eq!(state.construction_progress, 0, "No progress without recruits");
        assert!(app.world().get::<ConstructionHP>(building).is_some());
    }

    #[test]
    fn cults_construction_hp_scales_during_progress() {
        let mut app = setup_construction_app();
        let total_frames = 100;
        let max_hp = 200.0;

        let recruit = spawn_hidden_recruit(app.world_mut());
        let building = spawn_cults_building_under_construction(app.world_mut(), total_frames, max_hp);

        app.world_mut().get_mut::<CultsConstructionState>(building).unwrap()
            .assigned_recruits.push(recruit);

        // Run for 50 frames (50% progress)
        for _ in 0..50 {
            app.world_mut().run_system_once(cults_construction_tick_system).unwrap();
        }

        let obj = app.world().get::<ObjectInstance>(building).unwrap();
        let expected_hp = max_hp * ConstructionHP::hp_fraction(0.5); // 10% + 90% * 0.5 = 55% = 110.0
        assert!(f32::abs(obj.hp.unwrap() - expected_hp) < 0.01,
            "HP should scale with progress: expected {}, got {}", expected_hp, obj.hp.unwrap());
    }

    #[test]
    fn cults_construction_cancel_ejects_recruits_on_death() {
        let mut app = setup_construction_app();
        let total_frames = 100;

        let recruit1 = spawn_hidden_recruit(app.world_mut());
        let recruit2 = spawn_hidden_recruit(app.world_mut());
        let building = spawn_cults_building_under_construction(app.world_mut(), total_frames, 200.0);

        {
            let mut state = app.world_mut().get_mut::<CultsConstructionState>(building).unwrap();
            state.assigned_recruits.push(recruit1);
            state.assigned_recruits.push(recruit2);
        }

        // Kill the building
        app.world_mut().get_mut::<ObjectInstance>(building).unwrap().hp = Some(0.0);

        // Run cancel system
        app.world_mut().run_system_once(cults_construction_cancel_system).unwrap();
        app.update();

        // Recruits should be visible again and idle
        let vis1 = app.world().get::<Visibility>(recruit1).unwrap();
        assert_eq!(*vis1, Visibility::Inherited, "Recruit 1 should be visible after cancel");

        let vis2 = app.world().get::<Visibility>(recruit2).unwrap();
        assert_eq!(*vis2, Visibility::Inherited, "Recruit 2 should be visible after cancel");

        let cmd1 = app.world().get::<UnitCommand>(recruit1).unwrap();
        assert!(matches!(*cmd1, UnitCommand::Idle), "Recruit 1 should be idle after cancel");

        let cmd2 = app.world().get::<UnitCommand>(recruit2).unwrap();
        assert!(matches!(*cmd2, UnitCommand::Idle), "Recruit 2 should be idle after cancel");
    }

    #[test]
    fn cults_construction_state_new() {
        let state = CultsConstructionState::new(300);
        assert!(state.assigned_recruits.is_empty());
        assert_eq!(state.construction_progress, 0);
        assert_eq!(state.total_construction_frames, 300);
    }

    #[test]
    fn generic_construction_hp_tick_skips_cults_buildings() {
        let mut app = setup_construction_app();
        let total_frames = 100;
        let max_hp = 200.0;

        // Spawn a Cults building with BOTH ConstructionHP and CultsConstructionState
        let building = spawn_cults_building_under_construction(app.world_mut(), total_frames, max_hp);

        // Run the GENERIC construction_hp_tick_system
        for _ in 0..50 {
            app.world_mut().run_system_once(construction_hp_tick_system).unwrap();
        }

        // The generic system should NOT have advanced progress (due to Without<CultsConstructionState> filter)
        let construction = app.world().get::<ConstructionHP>(building).unwrap();
        assert_eq!(construction.progress, 0.0,
            "Generic construction_hp_tick_system should skip Cults buildings");
    }
}
