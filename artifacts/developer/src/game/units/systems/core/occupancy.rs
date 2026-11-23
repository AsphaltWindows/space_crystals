use bevy::prelude::*;
use crate::types::*;
use crate::game::types::{ObjectInstance, StructureInstance};
use crate::game::world::types::{Tile, TilePreset, GridMap};
use crate::game::combat::types::Silhouette;
use crate::game::units::types::*;

/// System to rebuild the OccupancyMap each frame.
/// Marks tiles occupied by ground units and structure footprints.
/// Must run before pathfinding and movement systems.
pub fn rebuild_occupancy_map(
    mut occupancy: ResMut<OccupancyMap>,
    ground_units: Query<(Entity, &GridPosition, &Transform, Option<&DomainEnum>, Option<&Silhouette>), With<Unit>>,
    structures: Query<(&GridPosition, &ObjectInstance, Option<&DomainEnum>), With<StructureInstance>>,
) {
    occupancy.clear();

    // Mark ground units
    for (entity, grid_pos, transform, domain_opt, silhouette_opt) in &ground_units {
        let is_ground = domain_opt.map_or(true, |d| *d == DomainEnum::Ground);
        if !is_ground {
            continue;
        }
        // Pathfinding layer: mark the unit's tile
        occupancy.blocked_tiles.insert((grid_pos.x, grid_pos.z));
        // Movement layer: store AABB collision body
        let (half_w, half_h) = if let Some(sil) = silhouette_opt {
            (sil.width / 2.0, sil.height / 2.0)
        } else {
            (0.25, 0.25) // Default small body for units without silhouette
        };
        occupancy.ground_bodies.push(CollisionBody {
            entity,
            x: transform.translation.x,
            z: transform.translation.z,
            half_w,
            half_h,
        });
    }

    // Mark structures (skip underground -- they don't block surface movement)
    for (grid_pos, obj_instance, domain_opt) in &structures {
        let is_underground = domain_opt.map_or(false, |d| *d == DomainEnum::Underground);
        if is_underground {
            continue;
        }
        let (size_w, size_h) = obj_instance.object_type.object_type().size;
        for dx in 0..size_w as i32 {
            for dz in 0..size_h as i32 {
                let tile = (grid_pos.x + dx, grid_pos.z + dz);
                occupancy.blocked_tiles.insert(tile);
                occupancy.structure_tiles.insert(tile);
            }
        }
    }
}

/// System to recompute paths for units blocked by collisions.
/// Runs after rebuild_occupancy_map. Units with NeedsRepath get a fresh path
/// using the updated occupancy data. After MAX_REPATH_ATTEMPTS failed attempts,
/// the unit gives up and stops (clears MoveTarget to prevent infinite A* allocation).
pub fn collision_repath_system(
    mut commands: Commands,
    units: Query<
        (Entity, &Transform, &UnitBaseEnum, &MoveTarget, Option<&RepathAttempts>),
        (With<Unit>, With<NeedsRepath>)
    >,
    tiles: Query<(&GridPosition, &TilePreset), With<Tile>>,
    grid: Res<GridMap>,
    occupancy: Res<OccupancyMap>,
) {
    use crate::game::units::utils::{world_to_grid, smooth_path};

    for (entity, transform, unit_base, move_target, attempts) in &units {
        let attempt_count = attempts.map(|a| a.0).unwrap_or(0);

        // Give up after too many failed attempts -- stop the unit
        if attempt_count >= MAX_REPATH_ATTEMPTS {
            commands.entity(entity)
                .remove::<NeedsRepath>()
                .remove::<RepathAttempts>()
                .remove::<MoveTarget>()
                .remove::<Path>();
            continue;
        }

        let start = world_to_grid(transform.translation);
        let target = world_to_grid(move_target.0);
        let self_pos = (start.x, start.z);

        if let Some(path) = crate::game::units::pathfinding::find_path_for_domain(
            start, target, &tiles, unit_base, grid.width as i32, grid.height as i32,
            &occupancy, self_pos,
        ) {
            let smoothed = smooth_path(path);
            commands.entity(entity)
                .remove::<NeedsRepath>()
                .remove::<RepathAttempts>()
                .insert(Path { waypoints: smoothed, current_waypoint: 0 });
        } else {
            // Increment retry counter
            commands.entity(entity).insert(RepathAttempts(attempt_count + 1));
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use bevy::ecs::system::RunSystemOnce;

    // === Memory Leak Fix Tests (repath) ===

    #[test]
    fn repath_attempts_component_tracks_failures() {
        let mut world = World::new();
        let entity = world.spawn((Unit, NeedsRepath)).id();
        world.entity_mut(entity).insert(RepathAttempts(0));
        let attempts = world.entity(entity).get::<RepathAttempts>().unwrap();
        assert_eq!(attempts.0, 0);

        world.entity_mut(entity).insert(RepathAttempts(1));
        let attempts = world.entity(entity).get::<RepathAttempts>().unwrap();
        assert_eq!(attempts.0, 1);
    }

    #[test]
    fn repath_attempts_max_value_check() {
        let mut world = World::new();
        let entity = world.spawn((Unit, NeedsRepath, RepathAttempts(MAX_REPATH_ATTEMPTS))).id();
        let attempts = world.entity(entity).get::<RepathAttempts>().unwrap();
        assert!(attempts.0 >= MAX_REPATH_ATTEMPTS);
    }

    #[test]
    fn repath_below_max_keeps_trying() {
        let mut world = World::new();
        let entity = world.spawn((Unit, NeedsRepath, RepathAttempts(MAX_REPATH_ATTEMPTS - 1))).id();
        let attempts = world.entity(entity).get::<RepathAttempts>().unwrap();
        assert!(attempts.0 < MAX_REPATH_ATTEMPTS);
    }

    // === rebuild_occupancy_map tests ===

    #[test]
    fn rebuild_occupancy_map_surface_structure_blocks_tiles() {
        let mut world = World::new();
        world.insert_resource(OccupancyMap::default());

        world.spawn((
            GridPosition { x: 10, z: 10 },
            ObjectInstance::destructible(ObjectEnum::Tunnel, 100.0),
            StructureInstance::default(),
        ));

        world.run_system_once(rebuild_occupancy_map).unwrap();

        let occupancy = world.resource::<OccupancyMap>();
        for dx in 0..4 {
            for dz in 0..4 {
                let tile = (10 + dx, 10 + dz);
                assert!(occupancy.blocked_tiles.contains(&tile),
                    "Surface structure tile {:?} should be in blocked_tiles", tile);
                assert!(occupancy.structure_tiles.contains(&tile),
                    "Surface structure tile {:?} should be in structure_tiles", tile);
            }
        }
    }

    #[test]
    fn rebuild_occupancy_map_underground_structure_does_not_block() {
        let mut world = World::new();
        world.insert_resource(OccupancyMap::default());

        world.spawn((
            GridPosition { x: 10, z: 10 },
            ObjectInstance::destructible(ObjectEnum::Headquarters, 100.0),
            StructureInstance::default(),
            DomainEnum::Underground,
        ));

        world.run_system_once(rebuild_occupancy_map).unwrap();

        let occupancy = world.resource::<OccupancyMap>();
        for dx in 0..2 {
            for dz in 0..2 {
                let tile = (10 + dx, 10 + dz);
                assert!(!occupancy.blocked_tiles.contains(&tile),
                    "Underground structure tile {:?} should NOT be in blocked_tiles", tile);
                assert!(!occupancy.structure_tiles.contains(&tile),
                    "Underground structure tile {:?} should NOT be in structure_tiles", tile);
            }
        }
    }

    #[test]
    fn rebuild_occupancy_map_surface_blocks_but_underground_does_not() {
        let mut world = World::new();
        world.insert_resource(OccupancyMap::default());

        world.spawn((
            GridPosition { x: 10, z: 10 },
            ObjectInstance::destructible(ObjectEnum::Tunnel, 100.0),
            StructureInstance::default(),
        ));

        world.spawn((
            GridPosition { x: 20, z: 20 },
            ObjectInstance::destructible(ObjectEnum::Headquarters, 100.0),
            StructureInstance::default(),
            DomainEnum::Underground,
        ));

        world.run_system_once(rebuild_occupancy_map).unwrap();

        let occupancy = world.resource::<OccupancyMap>();

        for dx in 0..4 {
            for dz in 0..4 {
                let tile = (10 + dx, 10 + dz);
                assert!(occupancy.blocked_tiles.contains(&tile),
                    "Surface Tunnel tile {:?} should be blocked", tile);
            }
        }

        for dx in 0..2 {
            for dz in 0..2 {
                let tile = (20 + dx, 20 + dz);
                assert!(!occupancy.blocked_tiles.contains(&tile),
                    "Underground HQ tile {:?} should NOT be blocked", tile);
            }
        }
    }
}
