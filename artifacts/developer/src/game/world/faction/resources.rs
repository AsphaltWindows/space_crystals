use bevy::prelude::*;
use crate::types::Owner;
use crate::game::types::*;
use crate::game::world::types::SpaceCrystalPatch;
use super::power::get_power_ratio_for_owner;

// =====================================================
// EXTRACTION PLATE MINING SYSTEM
// =====================================================

/// System that ticks Extraction Plate mining each simulation frame
pub fn extraction_plate_mining_system(
    mut plates: Query<(&Owner, &mut ExtractionPlateState)>,
    mut patches: Query<&mut SpaceCrystalPatch>,
    mut players: Query<(&Player, &mut GdoPlayerResources)>,
) {
    for (owner, mut plate_state) in plates.iter_mut() {
        let power_ratio = get_power_ratio_for_owner(owner, &players);
        plate_state.mining_timer += power_ratio;

        if plate_state.mining_timer >= EXTRACTION_PLATE_MINING_INTERVAL {
            plate_state.mining_timer = 0.0;

            // Check patch state and mine
            let amount = if let Ok(mut patch) = patches.get_mut(plate_state.attached_patch) {
                if patch.remaining_amount > 0 {
                    let mine_amount = EXTRACTION_PLATE_MINING_RATE.min(patch.remaining_amount);
                    patch.remaining_amount -= mine_amount;
                    mine_amount
                } else {
                    // Depleted patch still exists — residual mining
                    EXTRACTION_PLATE_RESIDUAL_RATE
                }
            } else {
                // Patch entity no longer exists (despawned) — residual mining
                EXTRACTION_PLATE_RESIDUAL_RATE
            };

            // Add to player's resources
            for (player, mut res) in players.iter_mut() {
                if Some(player.player_number) == owner.player_number() {
                    res.space_crystals += amount as i32;
                    break;
                }
            }
        }
    }
}

// =====================================================
// DEPLETED PATCH DESPAWN SYSTEM
// =====================================================

/// Despawns SpaceCrystalPatch entities when their remaining_amount reaches 0.
/// The attached ExtractionPlate (if any) persists and continues mining at the residual rate.
/// Per the design spec: "When a patch is fully depleted, it disappears from the map."
pub fn depleted_patch_despawn_system(
    mut commands: Commands,
    patches: Query<(Entity, &SpaceCrystalPatch)>,
) {
    for (patch_entity, patch) in patches.iter() {
        if patch.remaining_amount == 0 {
            info!("Despawning depleted SpaceCrystalPatch {:?}", patch_entity);
            commands.entity(patch_entity).despawn();
        }
    }
}

// =====================================================
// DISPLAY SYSTEM
// =====================================================

/// System to display resource information when 'R' key is pressed
pub fn display_resources_system(
    keyboard: Res<ButtonInput<KeyCode>>,
    gdo_players: Query<(&Player, &GdoPlayerResources)>,
    syndicate_players: Query<(&Player, &SyndicatePlayerResources)>,
    cults_players: Query<(&Player, &CultsPlayerResources)>,
    colonist_players: Query<(&Player, &ColonistsPlayerResources)>,
) {
    if keyboard.just_pressed(KeyCode::KeyR) {
        for (player, res) in gdo_players.iter() {
            info!("=== {} ({}) Resources ===", player.name, player.faction.name());
            info!("Space Crystals: {}", res.space_crystals);
            info!("Supplies: {}", res.supplies);
            info!("Power: {} / {} (net: {})",
                res.power_generated, res.power_consumed, res.current_power());
            info!("Power Ratio: {:.1}%", res.power_ratio() * 100.0);
            info!("Unit Control: {} / {}", res.unit_control_used, res.unit_control_cap);
            info!("");
        }
        for (player, res) in syndicate_players.iter() {
            info!("=== {} ({}) Resources ===", player.name, player.faction.name());
            info!("Space Crystals: {}", res.space_crystals);
            info!("Supplies: {}", res.supplies);
            info!("Tunnel Space: {} / {}",
                res.tunnel_space_used, res.tunnel_space_provided);
            info!("");
        }
        for (player, res) in cults_players.iter() {
            info!("=== {} ({}) Resources ===", player.name, player.faction.name());
            info!("Space Crystals: {}", res.space_crystals);
            info!("Unit Control: {} / {}", res.unit_control_used, res.unit_control_available);
            info!("");
        }
        for (player, res) in colonist_players.iter() {
            info!("=== {} ({}) Resources ===", player.name, player.faction.name());
            info!("Space Crystals: {}", res.space_crystals);
            info!("Alloys: {}", res.alloys);
            info!("Essence: {}", res.essence);
            info!("Conduits: {}", res.conduits);
            info!("Beacon Capacity: {} / {}",
                res.beacon_capacity_used, res.beacon_capacity_provided);
            info!("");
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use bevy::ecs::system::RunSystemOnce;
    use crate::types::FactionEnum;

    // --- Depleted Patch Despawn Tests ---

    #[test]
    fn depleted_patch_is_despawned() {
        let mut world = World::new();
        // Spawn a depleted patch (remaining_amount == 0)
        let patch_entity = world.spawn(SpaceCrystalPatch {
            remaining_amount: 0,
            initial_amount: 1000,
            has_plate: false,
        }).id();

        world.run_system_once(depleted_patch_despawn_system).unwrap();
        world.flush();

        assert!(world.get_entity(patch_entity).is_err(),
            "Depleted patch should be despawned");
    }

    #[test]
    fn non_depleted_patch_is_not_despawned() {
        let mut world = World::new();
        // Spawn a patch with remaining resources
        let patch_entity = world.spawn(SpaceCrystalPatch {
            remaining_amount: 500,
            initial_amount: 1000,
            has_plate: true,
        }).id();

        world.run_system_once(depleted_patch_despawn_system).unwrap();
        world.flush();

        assert!(world.get_entity(patch_entity).is_ok(),
            "Non-depleted patch should not be despawned");
    }

    #[test]
    fn depleted_patch_does_not_despawn_attached_plate() {
        let mut world = World::new();
        // Spawn a depleted patch
        let patch_entity = world.spawn(SpaceCrystalPatch {
            remaining_amount: 0,
            initial_amount: 1000,
            has_plate: true,
        }).id();

        // Spawn an extraction plate attached to this patch
        let plate_entity = world.spawn(ExtractionPlateState {
            attached_patch: patch_entity,
            mining_timer: 0.0,
        }).id();

        world.run_system_once(depleted_patch_despawn_system).unwrap();
        world.flush();

        assert!(world.get_entity(patch_entity).is_err(),
            "Depleted patch should be despawned");
        assert!(world.get_entity(plate_entity).is_ok(),
            "Plate attached to depleted patch should persist independently");
    }

    #[test]
    fn plate_on_non_depleted_patch_is_not_despawned() {
        let mut world = World::new();
        // Spawn a patch with remaining resources
        let patch_entity = world.spawn(SpaceCrystalPatch {
            remaining_amount: 100,
            initial_amount: 1000,
            has_plate: true,
        }).id();

        // Spawn an extraction plate attached to this patch
        let plate_entity = world.spawn(ExtractionPlateState {
            attached_patch: patch_entity,
            mining_timer: 0.0,
        }).id();

        world.run_system_once(depleted_patch_despawn_system).unwrap();
        world.flush();

        assert!(world.get_entity(patch_entity).is_ok(),
            "Non-depleted patch should not be despawned");
        assert!(world.get_entity(plate_entity).is_ok(),
            "Plate on non-depleted patch should not be despawned");
    }

    #[test]
    fn extraction_plate_mining_slowed_by_power_deficit() {
        let mut world = World::new();

        // Spawn a player with power deficit: ratio = 50/100 = 0.5
        let player_entity = world.spawn((
            Player::new("Test", FactionEnum::GlobalDefenseOrdinance, 0),
            GdoPlayerResources {
                space_crystals: 0,
                supplies: 0,
                power_generated: 50,
                power_consumed: 100,
                unit_control_used: 0,
                unit_control_cap: 10,
                has_power_plant: true,
            },
        )).id();

        // Spawn a crystal patch with resources
        let patch_entity = world.spawn(SpaceCrystalPatch {
            remaining_amount: 1000,
            initial_amount: 1000,
            has_plate: true,
        }).id();

        // Spawn an extraction plate owned by the player
        let plate_entity = world.spawn((
            Owner::player(0),
            ExtractionPlateState {
                attached_patch: patch_entity,
                mining_timer: 0.0,
            },
        )).id();

        // Run 48 ticks — at full power this would complete one mining cycle,
        // but at 0.5 ratio only 24.0 effective ticks accumulate
        for _ in 0..48 {
            world.run_system_once(extraction_plate_mining_system).unwrap();
            world.flush();
        }

        // Mining should NOT have completed yet
        let res = world.get::<GdoPlayerResources>(player_entity).unwrap();
        assert_eq!(res.space_crystals, 0,
            "Mining should not complete in 48 ticks at 0.5 power ratio");

        // Run 48 more ticks (total 96 ticks at 0.5 = 48 effective)
        for _ in 0..48 {
            world.run_system_once(extraction_plate_mining_system).unwrap();
            world.flush();
        }

        // Now mining should have completed at least once
        let res = world.get::<GdoPlayerResources>(player_entity).unwrap();
        assert!(res.space_crystals > 0,
            "Mining should complete after 96 ticks at 0.5 power ratio");
        assert_eq!(res.space_crystals, EXTRACTION_PLATE_MINING_RATE as i32,
            "Should have mined exactly one cycle worth of crystals");

        // Verify the timer has been reset
        let plate = world.get::<ExtractionPlateState>(plate_entity).unwrap();
        assert!(plate.mining_timer < EXTRACTION_PLATE_MINING_INTERVAL,
            "Timer should have been reset after mining cycle");
    }

    #[test]
    fn extraction_plate_mining_full_power() {
        let mut world = World::new();

        // Spawn a player with full power (generated >= consumed)
        let player_entity = world.spawn((
            Player::new("Test", FactionEnum::GlobalDefenseOrdinance, 0),
            GdoPlayerResources {
                space_crystals: 0,
                supplies: 0,
                power_generated: 100,
                power_consumed: 50,
                unit_control_used: 0,
                unit_control_cap: 10,
                has_power_plant: true,
            },
        )).id();

        let patch_entity = world.spawn(SpaceCrystalPatch {
            remaining_amount: 1000,
            initial_amount: 1000,
            has_plate: true,
        }).id();

        world.spawn((
            Owner::player(0),
            ExtractionPlateState {
                attached_patch: patch_entity,
                mining_timer: 0.0,
            },
        ));

        // Run exactly 48 ticks at full power — should complete one cycle
        for _ in 0..48 {
            world.run_system_once(extraction_plate_mining_system).unwrap();
            world.flush();
        }

        let res = world.get::<GdoPlayerResources>(player_entity).unwrap();
        assert_eq!(res.space_crystals, EXTRACTION_PLATE_MINING_RATE as i32,
            "Mining should complete in exactly 48 ticks at full power");
    }

    #[test]
    fn extraction_plate_mines_residual_after_patch_despawned() {
        let mut world = World::new();

        // Spawn a player with full power
        let player_entity = world.spawn((
            Player::new("Test", FactionEnum::GlobalDefenseOrdinance, 0),
            GdoPlayerResources {
                space_crystals: 0,
                supplies: 0,
                power_generated: 100,
                power_consumed: 50,
                unit_control_used: 0,
                unit_control_cap: 10,
                has_power_plant: true,
            },
        )).id();

        // Spawn a patch entity, then despawn it to simulate depletion cleanup
        let patch_entity = world.spawn(SpaceCrystalPatch {
            remaining_amount: 0,
            initial_amount: 1000,
            has_plate: true,
        }).id();

        // Spawn an extraction plate pointing to the patch
        world.spawn((
            Owner::player(0),
            ExtractionPlateState {
                attached_patch: patch_entity,
                mining_timer: 0.0,
            },
        ));

        // Despawn the patch to simulate depleted_patch_despawn_system having run
        world.despawn(patch_entity);

        // Verify the patch is gone
        assert!(world.get_entity(patch_entity).is_err(),
            "Patch should be despawned before mining test");

        // Run 48 ticks at full power — should complete one mining cycle
        for _ in 0..48 {
            world.run_system_once(extraction_plate_mining_system).unwrap();
            world.flush();
        }

        // Plate should have mined at residual rate
        let res = world.get::<GdoPlayerResources>(player_entity).unwrap();
        assert_eq!(res.space_crystals, EXTRACTION_PLATE_RESIDUAL_RATE as i32,
            "Plate should mine at residual rate when patch entity is despawned");
    }

    #[test]
    fn extraction_plate_power_grid_integration() {
        use crate::testing::{TestApp, TestHarness};
        use crate::game::types::structures::gdo_structure_stats::{DC_POWER, PP_POWER, EP_POWER};
        use crate::types::ObjectEnum;
        use crate::types::Owner;
        use super::super::power::compute_power_grid;

        let mut test_app = TestApp::new_with_faction(crate::types::FactionEnum::GlobalDefenseOrdinance);
        // Step to trigger OnEnter(InGame) which spawns faction/player entities
        test_app.step();
        let owner = Owner::player(0);

        // Spawn a power plant (+20)
        {
            let mut harness = TestHarness::new(&mut test_app.app);
            harness.spawn_structure_at_grid(ObjectEnum::PowerPlant, 32, 32, owner);
        }

        // Spawn 3 extraction plates (-3 each = -9 total)
        {
            let mut harness = TestHarness::new(&mut test_app.app);
            let patch = harness.spawn_resource(30, 30, 1000);
            harness.spawn_extraction_plate_at_grid(30, 31, owner, patch);
            let patch2 = harness.spawn_resource(31, 30, 1000);
            harness.spawn_extraction_plate_at_grid(31, 31, owner, patch2);
            let patch3 = harness.spawn_resource(33, 30, 1000);
            harness.spawn_extraction_plate_at_grid(33, 31, owner, patch3);
        }

        // Run the power grid computation
        test_app.app.world_mut().run_system_once(compute_power_grid).unwrap();

        // Check power values on the GDO player resources
        let mut query = test_app.app.world_mut().query::<(&Player, &GdoPlayerResources)>();
        let found = query.iter(test_app.app.world())
            .find(|(p, _)| p.player_number == 0);
        assert!(found.is_some(), "Player 0 should exist");
        let (_, res) = found.unwrap();
        // DC (20) is spawned by TestApp initialization + our PP (20) = 40 generated
        let expected_generated = DC_POWER + PP_POWER;
        let expected_consumed = EP_POWER.abs() * 3; // 3 plates * 3 = 9
        assert_eq!(res.power_generated, expected_generated, "DC + Power Plant generate power");
        assert_eq!(res.power_consumed, expected_consumed, "3 extraction plates consume {} power each", EP_POWER.abs());
        // Net power: 40 - 9 = 31
        assert_eq!(res.current_power(), expected_generated - expected_consumed);
    }
}
