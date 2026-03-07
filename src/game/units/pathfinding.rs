use bevy::prelude::*;
use crate::types::{GridPosition, UnitBaseEnum};
use crate::game::world::types::{Tile, TilePreset};
use super::types::*;
use super::utils::{heuristic, get_neighbors, is_traversible, is_diagonal_traversible};
use std::collections::{BinaryHeap, HashMap};

/// A* pathfinding algorithm.
///
/// `occupancy` provides tile-level occupancy data (ground units + structures).
/// `self_pos` is the moving unit's current grid position for self-exclusion.
/// The goal tile is always considered traversible for occupancy (units can move TO an
/// occupied tile's neighbor, the movement layer handles final collision).
pub fn find_path(
    start: GridPosition,
    goal: GridPosition,
    tiles: &Query<(&GridPosition, &TilePreset), With<Tile>>,
    unit_base: &UnitBaseEnum,
    grid_width: i32,
    grid_height: i32,
    occupancy: &OccupancyMap,
    self_pos: (i32, i32),
) -> Option<Vec<GridPosition>> {
    let mut open_set = BinaryHeap::new();
    let mut closed_set = HashMap::new();
    let mut came_from: HashMap<(i32, i32), GridPosition> = HashMap::new();

    let start_node = PathNode::new(start, 0.0, heuristic(&start, &goal), None);
    open_set.push(start_node);

    let mut iterations = 0;
    let max_iterations = 10000;

    while let Some(current) = open_set.pop() {
        iterations += 1;
        if iterations > max_iterations {
            warn!("Pathfinding exceeded max iterations");
            return None;
        }

        let current_key = (current.position.x, current.position.z);

        if current.position.x == goal.x && current.position.z == goal.z {
            let mut path = vec![current.position];
            let mut current_pos = current.position;

            while let Some(parent) = came_from.get(&(current_pos.x, current_pos.z)) {
                path.push(*parent);
                current_pos = *parent;
            }

            path.reverse();
            return Some(path);
        }

        if let Some(&best_cost) = closed_set.get(&current_key) {
            if current.g_cost > best_cost {
                continue;
            }
        }

        closed_set.insert(current_key, current.g_cost);

        for (neighbor_pos, move_cost) in get_neighbors(&current.position) {
            if neighbor_pos.x < 0 || neighbor_pos.x >= grid_width || neighbor_pos.z < 0 || neighbor_pos.z >= grid_height {
                continue;
            }

            // Use diagonal traversibility check for diagonal moves (prevents corner cutting)
            let dx = (neighbor_pos.x - current.position.x).abs();
            let dz = (neighbor_pos.z - current.position.z).abs();
            if dx == 1 && dz == 1 {
                if !is_diagonal_traversible(tiles, &current.position, &neighbor_pos, unit_base) {
                    continue;
                }
            } else {
                if !is_traversible(tiles, &neighbor_pos, unit_base) {
                    continue;
                }
            }

            // Check occupancy (skip occupied tiles, except self and goal)
            let is_goal = neighbor_pos.x == goal.x && neighbor_pos.z == goal.z;
            if !is_goal && occupancy.is_blocked(neighbor_pos.x, neighbor_pos.z, self_pos) {
                continue;
            }

            let neighbor_key = (neighbor_pos.x, neighbor_pos.z);
            let tentative_g_cost = current.g_cost + move_cost;

            if let Some(&best_cost) = closed_set.get(&neighbor_key) {
                if tentative_g_cost >= best_cost {
                    continue;
                }
            }

            let h_cost = heuristic(&neighbor_pos, &goal);
            let neighbor_node = PathNode::new(neighbor_pos, tentative_g_cost, h_cost, Some(current.position));

            came_from.insert(neighbor_key, current.position);
            open_set.push(neighbor_node);
        }
    }

    None
}

#[cfg(test)]
mod tests {
    use super::*;
    use bevy::ecs::system::RunSystemOnce;

    /// Helper: spawn a grid of traversible tiles in the world
    fn spawn_tile_grid(world: &mut World, width: i32, height: i32) {
        for x in 0..width {
            for z in 0..height {
                world.spawn((
                    GridPosition { x, z },
                    TilePreset {
                        value: crate::game::world::types::TilePresetEnum::Plane,
                        name: "Open".to_string(),
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
    }

    /// Helper: make a specific tile impassable
    fn block_tile(world: &mut World, bx: i32, bz: i32) {
        for entity in world.iter_entities().map(|e| e.id()).collect::<Vec<_>>() {
            let is_target = {
                if let Some(gp) = world.entity(entity).get::<GridPosition>() {
                    gp.x == bx && gp.z == bz
                } else {
                    false
                }
            };
            if is_target {
                if let Some(mut preset) = world.entity_mut(entity).get_mut::<TilePreset>() {
                    preset.traversible = false;
                }
            }
        }
    }

    #[test]
    fn find_path_diagonal_straight_line() {
        let mut world = World::new();
        spawn_tile_grid(&mut world, 10, 10);

        let path = world.run_system_once(|tiles: Query<(&GridPosition, &TilePreset), With<Tile>>| {
            find_path(
                GridPosition { x: 0, z: 0 },
                GridPosition { x: 3, z: 3 },
                &tiles,
                &UnitBaseEnum::LightInfantry,
                10, 10,
                &OccupancyMap::default(),
                (0, 0),
            )
        });

        let path = path.expect("Path should exist");
        assert_eq!(path.first().unwrap().x, 0);
        assert_eq!(path.first().unwrap().z, 0);
        assert_eq!(path.last().unwrap().x, 3);
        assert_eq!(path.last().unwrap().z, 3);
        // A diagonal straight line should have exactly 4 nodes (0,0 -> 1,1 -> 2,2 -> 3,3)
        assert_eq!(path.len(), 4);
        // Each step should be diagonal
        for i in 1..path.len() {
            let dx = (path[i].x - path[i - 1].x).abs();
            let dz = (path[i].z - path[i - 1].z).abs();
            assert_eq!(dx, 1);
            assert_eq!(dz, 1);
        }
    }

    #[test]
    fn find_path_cardinal_straight_line() {
        let mut world = World::new();
        spawn_tile_grid(&mut world, 10, 10);

        let path = world.run_system_once(|tiles: Query<(&GridPosition, &TilePreset), With<Tile>>| {
            find_path(
                GridPosition { x: 0, z: 0 },
                GridPosition { x: 5, z: 0 },
                &tiles,
                &UnitBaseEnum::LightInfantry,
                10, 10,
                &OccupancyMap::default(),
                (0, 0),
            )
        });

        let path = path.expect("Path should exist");
        assert_eq!(path.len(), 6); // 0..5 inclusive
        // All z should be 0 (straight horizontal)
        for node in &path {
            assert_eq!(node.z, 0);
        }
    }

    #[test]
    fn find_path_no_corner_cutting() {
        // Set up a corner scenario:
        //   . . .
        //   . X .   (X at 1,1 is blocked)
        //   . . .
        // Path from (0,0) to (2,2) should NOT go through (1,1)
        let mut world = World::new();
        spawn_tile_grid(&mut world, 4, 4);
        block_tile(&mut world, 1, 1);

        let path = world.run_system_once(|tiles: Query<(&GridPosition, &TilePreset), With<Tile>>| {
            find_path(
                GridPosition { x: 0, z: 0 },
                GridPosition { x: 2, z: 2 },
                &tiles,
                &UnitBaseEnum::LightInfantry,
                4, 4,
                &OccupancyMap::default(),
                (0, 0),
            )
        });

        let path = path.expect("Path should exist");
        // Path should not contain the blocked tile
        assert!(!path.iter().any(|p| p.x == 1 && p.z == 1));
    }

    #[test]
    fn find_path_corner_cutting_blocked_by_adjacent() {
        // Block (1,0) and (0,1) — diagonal from (0,0) to (1,1) should be blocked
        // even though (1,1) is traversible, because both cardinal neighbors are blocked
        let mut world = World::new();
        spawn_tile_grid(&mut world, 4, 4);
        block_tile(&mut world, 1, 0);
        block_tile(&mut world, 0, 1);

        let path = world.run_system_once(|tiles: Query<(&GridPosition, &TilePreset), With<Tile>>| {
            find_path(
                GridPosition { x: 0, z: 0 },
                GridPosition { x: 1, z: 1 },
                &tiles,
                &UnitBaseEnum::LightInfantry,
                4, 4,
                &OccupancyMap::default(),
                (0, 0),
            )
        });

        // Path should not exist or should go around since direct diagonal is blocked
        // and both cardinal neighbors are also blocked
        assert!(path.is_none(), "Should not find path through blocked corner");
    }

    #[test]
    fn find_path_same_start_and_goal() {
        let mut world = World::new();
        spawn_tile_grid(&mut world, 5, 5);

        let path = world.run_system_once(|tiles: Query<(&GridPosition, &TilePreset), With<Tile>>| {
            find_path(
                GridPosition { x: 2, z: 2 },
                GridPosition { x: 2, z: 2 },
                &tiles,
                &UnitBaseEnum::LightInfantry,
                5, 5,
                &OccupancyMap::default(),
                (2, 2),
            )
        });

        let path = path.expect("Path should exist");
        assert_eq!(path.len(), 1);
        assert_eq!(path[0].x, 2);
        assert_eq!(path[0].z, 2);
    }

    #[test]
    fn find_path_mixed_diagonal_and_cardinal() {
        let mut world = World::new();
        spawn_tile_grid(&mut world, 10, 10);

        // From (0,0) to (5,3) — should use 3 diagonal + 2 cardinal steps
        let path = world.run_system_once(|tiles: Query<(&GridPosition, &TilePreset), With<Tile>>| {
            find_path(
                GridPosition { x: 0, z: 0 },
                GridPosition { x: 5, z: 3 },
                &tiles,
                &UnitBaseEnum::LightInfantry,
                10, 10,
                &OccupancyMap::default(),
                (0, 0),
            )
        });

        let path = path.expect("Path should exist");
        assert_eq!(path.first().unwrap().x, 0);
        assert_eq!(path.first().unwrap().z, 0);
        assert_eq!(path.last().unwrap().x, 5);
        assert_eq!(path.last().unwrap().z, 3);
        // Verify the path reaches the goal and each step is at most 1 tile
        for i in 1..path.len() {
            let dx = (path[i].x - path[i - 1].x).abs();
            let dz = (path[i].z - path[i - 1].z).abs();
            assert!(dx <= 1 && dz <= 1, "Step from ({},{}) to ({},{}) exceeds 1 tile",
                path[i-1].x, path[i-1].z, path[i].x, path[i].z);
        }
        // Path should use some diagonal steps (not all cardinal)
        let diagonal_steps = (1..path.len())
            .filter(|&i| {
                let dx = (path[i].x - path[i - 1].x).abs();
                let dz = (path[i].z - path[i - 1].z).abs();
                dx == 1 && dz == 1
            })
            .count();
        assert!(diagonal_steps > 0, "Path from (0,0) to (5,3) should use diagonal steps");
    }

    #[test]
    fn find_path_avoids_occupied_tile() {
        // Occupy tile (2,0) — path from (0,0) to (4,0) should go around it
        let mut world = World::new();
        spawn_tile_grid(&mut world, 6, 3);

        let mut occupancy = OccupancyMap::default();
        occupancy.blocked_tiles.insert((2, 0));

        let path = world.run_system_once(move |tiles: Query<(&GridPosition, &TilePreset), With<Tile>>| {
            find_path(
                GridPosition { x: 0, z: 0 },
                GridPosition { x: 4, z: 0 },
                &tiles,
                &UnitBaseEnum::LightInfantry,
                6, 3,
                &occupancy,
                (0, 0),
            )
        });

        let path = path.expect("Path should exist (going around occupied tile)");
        // Path must NOT pass through (2,0)
        assert!(!path.iter().any(|p| p.x == 2 && p.z == 0),
            "Path should avoid occupied tile (2,0)");
        assert_eq!(path.last().unwrap().x, 4);
        assert_eq!(path.last().unwrap().z, 0);
    }

    #[test]
    fn find_path_self_exclusion_allows_own_tile() {
        // Unit is on (2,0) which is also marked occupied — self-exclusion allows start
        let mut world = World::new();
        spawn_tile_grid(&mut world, 5, 3);

        let mut occupancy = OccupancyMap::default();
        occupancy.blocked_tiles.insert((2, 0));

        let path = world.run_system_once(move |tiles: Query<(&GridPosition, &TilePreset), With<Tile>>| {
            find_path(
                GridPosition { x: 2, z: 0 },
                GridPosition { x: 4, z: 0 },
                &tiles,
                &UnitBaseEnum::LightInfantry,
                5, 3,
                &occupancy,
                (2, 0),  // self is at (2,0) — should be excluded from blocking
            )
        });

        let path = path.expect("Path should exist despite self-tile being in occupancy");
        assert_eq!(path.first().unwrap().x, 2);
        assert_eq!(path.last().unwrap().x, 4);
    }

    #[test]
    fn find_path_goal_tile_allowed_even_if_occupied() {
        // Goal tile (3,0) is occupied — path should still reach it
        let mut world = World::new();
        spawn_tile_grid(&mut world, 5, 3);

        let mut occupancy = OccupancyMap::default();
        occupancy.blocked_tiles.insert((3, 0));

        let path = world.run_system_once(move |tiles: Query<(&GridPosition, &TilePreset), With<Tile>>| {
            find_path(
                GridPosition { x: 0, z: 0 },
                GridPosition { x: 3, z: 0 },
                &tiles,
                &UnitBaseEnum::LightInfantry,
                5, 3,
                &occupancy,
                (0, 0),
            )
        });

        let path = path.expect("Path to occupied goal should still exist");
        assert_eq!(path.last().unwrap().x, 3);
        assert_eq!(path.last().unwrap().z, 0);
    }

    #[test]
    fn find_path_fully_blocked_returns_none() {
        // Block all tiles around start — should return None
        let mut world = World::new();
        spawn_tile_grid(&mut world, 5, 5);

        let mut occupancy = OccupancyMap::default();
        // Block all 8 neighbors of (2,2)
        for dx in -1..=1 {
            for dz in -1..=1 {
                if dx == 0 && dz == 0 { continue; }
                occupancy.blocked_tiles.insert((2 + dx, 2 + dz));
            }
        }

        let path = world.run_system_once(move |tiles: Query<(&GridPosition, &TilePreset), With<Tile>>| {
            find_path(
                GridPosition { x: 2, z: 2 },
                GridPosition { x: 4, z: 4 },
                &tiles,
                &UnitBaseEnum::LightInfantry,
                5, 5,
                &occupancy,
                (2, 2),
            )
        });

        assert!(path.is_none(), "Path should not exist when surrounded by occupied tiles");
    }
}
