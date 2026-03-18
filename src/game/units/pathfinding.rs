use bevy::prelude::*;
use crate::types::{GridPosition, UnitBaseEnum, DomainEnum};
use crate::game::world::types::{Tile, TilePreset};
use super::types::*;
use super::utils::{heuristic, get_neighbors};
use std::collections::{BinaryHeap, HashMap, HashSet};

/// Find a path for a unit, automatically choosing air vs ground pathfinding
/// based on the unit's domain. Air units skip occupancy checks.
pub fn find_path_for_domain(
    start: GridPosition,
    goal: GridPosition,
    tiles: &Query<(&GridPosition, &TilePreset), With<Tile>>,
    unit_base: &UnitBaseEnum,
    grid_width: i32,
    grid_height: i32,
    occupancy: &OccupancyMap,
    self_pos: (i32, i32),
) -> Option<Vec<GridPosition>> {
    let is_air = unit_base.data().domain == DomainEnum::Air;
    find_path_inner(start, goal, tiles, unit_base, grid_width, grid_height, occupancy, self_pos, is_air)
}

/// Pre-computed tile traversibility map for O(1) lookups during A*.
/// Built once per find_path call from the tile Query, avoiding O(n) linear scans per neighbor check.
struct TileTraversibility {
    /// Set of tiles that are traversible (exist and not blocked)
    traversible: HashSet<(i32, i32)>,
    /// Set of tiles that are rugged (traversible only by units that can handle rugged terrain)
    rugged: HashSet<(i32, i32)>,
}

impl TileTraversibility {
    /// Build from the tile query — O(n) once, then O(1) per lookup
    fn build(tiles: &Query<(&GridPosition, &TilePreset), With<Tile>>) -> Self {
        let mut traversible = HashSet::with_capacity(tiles.iter().len());
        let mut rugged = HashSet::new();
        for (pos, preset) in tiles.iter() {
            if preset.traversible {
                traversible.insert((pos.x, pos.z));
                if preset.rugged {
                    rugged.insert((pos.x, pos.z));
                }
            }
        }
        TileTraversibility { traversible, rugged }
    }

    /// O(1) traversibility check
    fn is_traversible(&self, x: i32, z: i32, unit_base: &UnitBaseEnum) -> bool {
        let key = (x, z);
        if !self.traversible.contains(&key) {
            return false;
        }
        if self.rugged.contains(&key) && !unit_base.can_traverse_rugged() {
            return false;
        }
        true
    }

    /// O(1) diagonal traversibility check (no corner cutting)
    fn is_diagonal_traversible(&self, from: &GridPosition, to: &GridPosition, unit_base: &UnitBaseEnum) -> bool {
        if !self.is_traversible(to.x, to.z, unit_base) {
            return false;
        }
        let dx = to.x - from.x;
        let dz = to.z - from.z;
        if dx.abs() == 1 && dz.abs() == 1 {
            // Both cardinal corners must be traversible to prevent corner cutting
            self.is_traversible(from.x + dx, from.z, unit_base)
                && self.is_traversible(from.x, from.z + dz, unit_base)
        } else {
            true
        }
    }
}

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
    find_path_inner(start, goal, tiles, unit_base, grid_width, grid_height, occupancy, self_pos, false)
}

/// Find a path for air units — skips occupancy checks (air units fly over obstacles).
pub fn find_path_air(
    start: GridPosition,
    goal: GridPosition,
    tiles: &Query<(&GridPosition, &TilePreset), With<Tile>>,
    unit_base: &UnitBaseEnum,
    grid_width: i32,
    grid_height: i32,
    occupancy: &OccupancyMap,
    self_pos: (i32, i32),
) -> Option<Vec<GridPosition>> {
    find_path_inner(start, goal, tiles, unit_base, grid_width, grid_height, occupancy, self_pos, true)
}

fn find_path_inner(
    start: GridPosition,
    goal: GridPosition,
    tiles: &Query<(&GridPosition, &TilePreset), With<Tile>>,
    unit_base: &UnitBaseEnum,
    grid_width: i32,
    grid_height: i32,
    occupancy: &OccupancyMap,
    self_pos: (i32, i32),
    skip_occupancy: bool,
) -> Option<Vec<GridPosition>> {
    // Pre-build tile lookup map for O(1) traversibility checks.
    // Without this, each is_traversible call is O(n) over all tiles,
    // making A* on a 64x64 grid prohibitively slow for long paths.
    let tile_map = TileTraversibility::build(tiles);

    let estimated_capacity = ((grid_width * grid_height) as usize).min(4096);
    let mut open_set = BinaryHeap::with_capacity(estimated_capacity);
    let mut closed_set: HashMap<(i32, i32), f32> = HashMap::with_capacity(estimated_capacity);
    let mut came_from: HashMap<(i32, i32), GridPosition> = HashMap::with_capacity(estimated_capacity);
    // Track best g-cost ever seen for each node (open or closed) to prevent
    // came_from overwrites by equal-cost alternative paths that produce zigzag routes.
    let mut best_g: HashMap<(i32, i32), f32> = HashMap::with_capacity(estimated_capacity);

    let start_node = PathNode::new(start, 0.0, heuristic(&start, &goal), None);
    open_set.push(start_node);
    best_g.insert((start.x, start.z), 0.0);

    let mut iterations: usize = 0;
    // With the >= fix in the closed_set check, each unique grid position is processed
    // at most once. Max iterations = grid size covers the worst case. Add margin for
    // duplicate entries still in the open_set that need to be popped and skipped.
    let max_iterations: usize = ((grid_width * grid_height) as usize).max(1000) * 2;

    while let Some(current) = open_set.pop() {
        iterations += 1;
        if iterations > max_iterations {
            warn!("Pathfinding exceeded max iterations ({})", max_iterations);
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

        // Skip if we've already processed this node with an equal or better cost.
        // Using >= (not >) prevents re-processing equal-cost nodes, which would cause
        // exponential duplicate growth in the open_set from floating-point-equal paths.
        if let Some(&best_cost) = closed_set.get(&current_key) {
            if current.g_cost >= best_cost {
                continue;
            }
        }

        closed_set.insert(current_key, current.g_cost);

        for (neighbor_pos, move_cost) in get_neighbors(&current.position) {
            if neighbor_pos.x < 0 || neighbor_pos.x >= grid_width || neighbor_pos.z < 0 || neighbor_pos.z >= grid_height {
                continue;
            }

            // Air units skip tile traversibility checks — they fly over all terrain
            // Ground/underground units must check tile traversibility and diagonal corner cutting
            if !skip_occupancy {
                let dx = (neighbor_pos.x - current.position.x).abs();
                let dz = (neighbor_pos.z - current.position.z).abs();
                if dx == 1 && dz == 1 {
                    if !tile_map.is_diagonal_traversible(&current.position, &neighbor_pos, unit_base) {
                        continue;
                    }
                } else {
                    if !tile_map.is_traversible(neighbor_pos.x, neighbor_pos.z, unit_base) {
                        continue;
                    }
                }
            }

            // Check occupancy (skip occupied tiles, except self and goal)
            // Air units skip occupancy checks entirely — they fly over obstacles
            if !skip_occupancy {
                let is_goal = neighbor_pos.x == goal.x && neighbor_pos.z == goal.z;
                if !is_goal && occupancy.is_blocked(neighbor_pos.x, neighbor_pos.z, self_pos) {
                    continue;
                }
            }

            let neighbor_key = (neighbor_pos.x, neighbor_pos.z);
            let tentative_g_cost = current.g_cost + move_cost;

            // Skip if we've already found an equal or better path to this node
            // (whether in open set or closed set). This prevents duplicate entries
            // and came_from overwrites that cause zigzag paths on open terrain.
            if let Some(&best) = best_g.get(&neighbor_key) {
                if tentative_g_cost >= best {
                    continue;
                }
            }

            best_g.insert(neighbor_key, tentative_g_cost);
            came_from.insert(neighbor_key, current.position);

            let h_cost = heuristic(&neighbor_pos, &goal);
            let neighbor_node = PathNode::new(neighbor_pos, tentative_g_cost, h_cost, Some(current.position));
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
        for entity in world.query::<Entity>().iter(world).collect::<Vec<_>>() {
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

        let path = path.expect("system ran").unwrap();
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

        let path = path.expect("system ran").unwrap();
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

        let path = path.expect("system ran").unwrap();
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
        assert!(path.expect("system ran").is_none(), "Should not find path through blocked corner");
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

        let path = path.expect("system ran").unwrap();
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

        let path = path.expect("system ran").unwrap();
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
        // Path should use exactly 3 diagonal + 2 cardinal = 5 + 1 = 6 nodes total
        assert_eq!(path.len(), 6, "Optimal path from (0,0) to (5,3) should have 6 nodes (3 diagonal + 2 cardinal)");
        let diagonal_steps = (1..path.len())
            .filter(|&i| {
                let dx = (path[i].x - path[i - 1].x).abs();
                let dz = (path[i].z - path[i - 1].z).abs();
                dx == 1 && dz == 1
            })
            .count();
        assert_eq!(diagonal_steps, 3, "Path from (0,0) to (5,3) should use exactly 3 diagonal steps");
        let cardinal_steps = (1..path.len())
            .filter(|&i| {
                let dx = (path[i].x - path[i - 1].x).abs();
                let dz = (path[i].z - path[i - 1].z).abs();
                (dx == 1) ^ (dz == 1)
            })
            .count();
        assert_eq!(cardinal_steps, 2, "Path from (0,0) to (5,3) should use exactly 2 cardinal steps");
    }

    #[test]
    fn find_path_diagonal_preference_consistent() {
        // Verify that on open terrain, A* consistently produces smooth diagonal paths
        // (diagonals grouped together) rather than zigzag cardinal/diagonal alternation.
        let mut world = World::new();
        spawn_tile_grid(&mut world, 20, 20);

        // Test several mixed-distance paths
        let test_cases = vec![
            ((0, 0), (7, 4)),  // 4 diag + 3 card
            ((0, 0), (4, 7)),  // 4 diag + 3 card
            ((2, 2), (10, 5)), // 3 diag + 5 card
            ((0, 0), (6, 2)),  // 2 diag + 4 card
        ];

        for ((sx, sz), (gx, gz)) in &test_cases {
            let (sx, sz, gx, gz) = (*sx, *sz, *gx, *gz);
            let path = world.run_system_once(move |tiles: Query<(&GridPosition, &TilePreset), With<Tile>>| {
                find_path(
                    GridPosition { x: sx, z: sz },
                    GridPosition { x: gx, z: gz },
                    &tiles,
                    &UnitBaseEnum::LightInfantry,
                    20, 20,
                    &OccupancyMap::default(),
                    (sx, sz),
                )
            });

            let path = path.expect("system ran").unwrap();
            let dx_total = (gx - sx).unsigned_abs();
            let dz_total = (gz - sz).unsigned_abs();
            let expected_diag = dx_total.min(dz_total);
            let expected_card = dx_total.max(dz_total) - expected_diag;
            let expected_len = (expected_diag + expected_card + 1) as usize;

            assert_eq!(path.len(), expected_len,
                "Path from ({},{}) to ({},{}) should have {} nodes, got {}",
                sx, sz, gx, gz, expected_len, path.len());

            // Count direction changes — a smooth path has at most 1 direction change
            // (all diagonals first, then all cardinals, or vice versa)
            let mut direction_changes = 0;
            for i in 2..path.len() {
                let d1x = path[i-1].x - path[i-2].x;
                let d1z = path[i-1].z - path[i-2].z;
                let d2x = path[i].x - path[i-1].x;
                let d2z = path[i].z - path[i-1].z;
                if d1x != d2x || d1z != d2z {
                    direction_changes += 1;
                }
            }
            assert!(direction_changes <= 1,
                "Path from ({},{}) to ({},{}) has {} direction changes (expected ≤1 for smooth diagonal preference)",
                sx, sz, gx, gz, direction_changes);
        }
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

        let path = path.expect("system ran").unwrap();
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

        let path = path.expect("system ran").unwrap();
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

        let path = path.expect("system ran").unwrap();
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

        assert!(path.expect("system ran").is_none(), "Path should not exist when surrounded by occupied tiles");
    }

    #[test]
    fn find_path_long_distance_64x64_grid() {
        // Ensure pathfinding works on a full 64x64 grid without hanging or OOM
        let mut world = World::new();
        spawn_tile_grid(&mut world, 64, 64);

        let path = world.run_system_once(|tiles: Query<(&GridPosition, &TilePreset), With<Tile>>| {
            find_path(
                GridPosition { x: 0, z: 0 },
                GridPosition { x: 63, z: 63 },
                &tiles,
                &UnitBaseEnum::LightInfantry,
                64, 64,
                &OccupancyMap::default(),
                (0, 0),
            )
        });

        let path = path.expect("system ran").unwrap();
        assert_eq!(path.first().unwrap().x, 0);
        assert_eq!(path.first().unwrap().z, 0);
        assert_eq!(path.last().unwrap().x, 63);
        assert_eq!(path.last().unwrap().z, 63);
        // On open terrain, a pure diagonal path from (0,0) to (63,63) should have 64 nodes
        assert_eq!(path.len(), 64);
    }

    #[test]
    fn find_path_long_distance_cardinal_64x64() {
        // Long cardinal path across full grid width
        let mut world = World::new();
        spawn_tile_grid(&mut world, 64, 64);

        let path = world.run_system_once(|tiles: Query<(&GridPosition, &TilePreset), With<Tile>>| {
            find_path(
                GridPosition { x: 0, z: 32 },
                GridPosition { x: 63, z: 32 },
                &tiles,
                &UnitBaseEnum::LightInfantry,
                64, 64,
                &OccupancyMap::default(),
                (0, 32),
            )
        });

        let path = path.expect("system ran").unwrap();
        assert_eq!(path.first().unwrap().x, 0);
        assert_eq!(path.last().unwrap().x, 63);
        assert_eq!(path.len(), 64);
    }

    #[test]
    fn find_path_routes_around_blocked_column() {
        // Test routing around a 3-tile column: x=3, z=0..2 blocked on 6x6 grid
        let mut world = World::new();
        spawn_tile_grid(&mut world, 6, 6);
        block_tile(&mut world, 3, 0);
        block_tile(&mut world, 3, 1);
        block_tile(&mut world, 3, 2);

        let path = world.run_system_once(|tiles: Query<(&GridPosition, &TilePreset), With<Tile>>| {
            find_path(
                GridPosition { x: 0, z: 0 },
                GridPosition { x: 5, z: 0 },
                &tiles,
                &UnitBaseEnum::LightInfantry,
                6, 6,
                &OccupancyMap::default(),
                (0, 0),
            )
        });

        let path = path.expect("system ran").unwrap();
        assert_eq!(path.first().unwrap().x, 0);
        assert_eq!(path.first().unwrap().z, 0);
        assert_eq!(path.last().unwrap().x, 5);
        assert_eq!(path.last().unwrap().z, 0);
        // Path must NOT pass through the blocked column
        assert!(!path.iter().any(|p| p.x == 3 && p.z <= 2));
    }

    #[test]
    fn find_path_long_distance_with_wall() {
        // Long path around a wall on a 20x20 grid
        let mut world = World::new();
        spawn_tile_grid(&mut world, 20, 20);
        // Block column x=10, z=0..16, gap at z=17-19
        for z in 0..17 {
            block_tile(&mut world, 10, z);
        }

        let path = world.run_system_once(|tiles: Query<(&GridPosition, &TilePreset), With<Tile>>| {
            find_path(
                GridPosition { x: 0, z: 0 },
                GridPosition { x: 19, z: 0 },
                &tiles,
                &UnitBaseEnum::LightInfantry,
                20, 20,
                &OccupancyMap::default(),
                (0, 0),
            )
        });

        let path = path.expect("system ran").unwrap();
        assert_eq!(path.first().unwrap().x, 0);
        assert_eq!(path.first().unwrap().z, 0);
        assert_eq!(path.last().unwrap().x, 19);
        assert_eq!(path.last().unwrap().z, 0);
        // Path must NOT pass through the wall
        assert!(!path.iter().any(|p| p.x == 10 && p.z < 17));
    }

    #[test]
    fn find_path_air_ignores_occupied_tiles() {
        // Air units should fly over occupied tiles that block ground units
        let mut world = World::new();
        spawn_tile_grid(&mut world, 10, 10);

        // Create an occupancy map with blocked tiles
        let mut occupancy = OccupancyMap::default();
        // Block tiles at x=5 for z=0..9 (wall)
        for z in 0..10 {
            occupancy.blocked_tiles.insert((5, z));
        }

        // Ground pathfinding: should fail (wall blocks all passage)
        let ground_path = world.run_system_once(move |tiles: Query<(&GridPosition, &TilePreset), With<Tile>>| {
            find_path(
                GridPosition { x: 0, z: 0 },
                GridPosition { x: 9, z: 0 },
                &tiles,
                &UnitBaseEnum::LightInfantry,
                10, 10,
                &occupancy,
                (0, 0),
            )
        });
        assert!(ground_path.expect("system ran").is_none(), "Ground path should be blocked by occupancy wall");
    }

    #[test]
    fn find_path_air_passes_through_occupancy_wall() {
        // Air units should fly over occupied tiles
        let mut world = World::new();
        spawn_tile_grid(&mut world, 10, 10);

        let mut occupancy = OccupancyMap::default();
        for z in 0..10 {
            occupancy.blocked_tiles.insert((5, z));
        }

        // Air pathfinding: should succeed (ignores occupancy)
        let air_path = world.run_system_once(move |tiles: Query<(&GridPosition, &TilePreset), With<Tile>>| {
            find_path_air(
                GridPosition { x: 0, z: 0 },
                GridPosition { x: 9, z: 0 },
                &tiles,
                &UnitBaseEnum::HoverCraft,
                10, 10,
                &occupancy,
                (0, 0),
            )
        });
        let path = air_path.expect("system ran").expect("Air path should succeed despite occupancy wall");
        assert_eq!(path.first().unwrap().x, 0);
        assert_eq!(path.last().unwrap().x, 9);
    }

    #[test]
    fn find_path_for_domain_auto_selects_air_vs_ground() {
        // find_path_for_domain should use air pathfinding for HoverCraft
        let mut world = World::new();
        spawn_tile_grid(&mut world, 10, 10);

        let mut occupancy = OccupancyMap::default();
        for z in 0..10 {
            occupancy.blocked_tiles.insert((5, z));
        }

        // HoverCraft has domain Air — should pass through occupancy
        let air_result = world.run_system_once(move |tiles: Query<(&GridPosition, &TilePreset), With<Tile>>| {
            find_path_for_domain(
                GridPosition { x: 0, z: 0 },
                GridPosition { x: 9, z: 0 },
                &tiles,
                &UnitBaseEnum::HoverCraft,
                10, 10,
                &occupancy,
                (0, 0),
            )
        });
        assert!(air_result.expect("system ran").is_some(), "HoverCraft should fly over occupancy wall");
    }

    #[test]
    fn find_path_for_domain_ground_unit_blocked() {
        // find_path_for_domain should use ground pathfinding for LightInfantry
        let mut world = World::new();
        spawn_tile_grid(&mut world, 10, 10);

        let mut occupancy = OccupancyMap::default();
        for z in 0..10 {
            occupancy.blocked_tiles.insert((5, z));
        }

        // LightInfantry has domain Ground — should be blocked
        let ground_result = world.run_system_once(move |tiles: Query<(&GridPosition, &TilePreset), With<Tile>>| {
            find_path_for_domain(
                GridPosition { x: 0, z: 0 },
                GridPosition { x: 9, z: 0 },
                &tiles,
                &UnitBaseEnum::LightInfantry,
                10, 10,
                &occupancy,
                (0, 0),
            )
        });
        assert!(ground_result.expect("system ran").is_none(), "LightInfantry should be blocked by occupancy wall");
    }

    #[test]
    fn find_path_air_ignores_impassable_terrain() {
        // Air units skip both occupancy AND terrain traversibility — they fly over everything
        let mut world = World::new();
        spawn_tile_grid(&mut world, 10, 10);
        // Block terrain at x=5 (untraversible tiles, not occupancy)
        for z in 0..10 {
            block_tile(&mut world, 5, z);
        }

        let air_path = world.run_system_once(|tiles: Query<(&GridPosition, &TilePreset), With<Tile>>| {
            find_path_air(
                GridPosition { x: 0, z: 0 },
                GridPosition { x: 9, z: 0 },
                &tiles,
                &UnitBaseEnum::HoverCraft,
                10, 10,
                &OccupancyMap::default(),
                (0, 0),
            )
        });
        // Air units CAN fly over untraversible terrain (e.g. rocks, water)
        let path = air_path.expect("system ran").expect("Air units should fly over impassable terrain");
        assert!(path.len() >= 2, "Path should have at least start and end");
        assert_eq!(path.first().unwrap().x, 0, "Path should start at origin");
        assert_eq!(path.last().unwrap().x, 9, "Path should reach the goal");
        // Verify path crosses through the blocked column at x=5
        assert!(path.iter().any(|p| p.x == 5), "Air path should cross through the impassable column at x=5");
    }

    #[test]
    fn find_path_ground_blocked_by_impassable_terrain() {
        // Ground units still can't cross impassable terrain
        let mut world = World::new();
        spawn_tile_grid(&mut world, 10, 10);
        for z in 0..10 {
            block_tile(&mut world, 5, z);
        }

        let ground_path = world.run_system_once(|tiles: Query<(&GridPosition, &TilePreset), With<Tile>>| {
            find_path(
                GridPosition { x: 0, z: 0 },
                GridPosition { x: 9, z: 0 },
                &tiles,
                &UnitBaseEnum::LightInfantry,
                10, 10,
                &OccupancyMap::default(),
                (0, 0),
            )
        });
        assert!(ground_path.expect("system ran").is_none(), "Ground units should be blocked by impassable terrain");
    }
}
