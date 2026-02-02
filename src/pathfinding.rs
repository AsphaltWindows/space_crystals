use bevy::prelude::*;
use crate::map::{GridPosition, Tile, TileProperties};
use crate::units::UnitBase;
use std::collections::{BinaryHeap, HashMap};
use std::cmp::Ordering;

/// Component storing a path as a sequence of grid positions
#[derive(Component)]
pub struct Path {
    pub waypoints: Vec<Vec3>, // World space waypoints
    pub current_waypoint: usize,
}

/// Node for A* pathfinding
#[derive(Clone)]
struct PathNode {
    position: GridPosition,
    g_cost: f32, // Cost from start
    h_cost: f32, // Heuristic cost to goal
    f_cost: f32, // g_cost + h_cost
    parent: Option<GridPosition>,
}

impl PathNode {
    fn new(position: GridPosition, g_cost: f32, h_cost: f32, parent: Option<GridPosition>) -> Self {
        Self {
            position,
            g_cost,
            h_cost,
            f_cost: g_cost + h_cost,
            parent,
        }
    }
}

impl Eq for PathNode {}

impl PartialEq for PathNode {
    fn eq(&self, other: &Self) -> bool {
        self.f_cost.eq(&other.f_cost)
    }
}

impl Ord for PathNode {
    fn cmp(&self, other: &Self) -> Ordering {
        // Reverse order for min-heap behavior
        other.f_cost.partial_cmp(&self.f_cost).unwrap_or(Ordering::Equal)
    }
}

impl PartialOrd for PathNode {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

/// Convert world position to grid position
pub fn world_to_grid(world_pos: Vec3) -> GridPosition {
    // Centered grid: world 0,0 is grid center (10, 10)
    let grid_x = (world_pos.x + 10.0).floor() as i32;
    let grid_z = (world_pos.z + 10.0).floor() as i32;
    GridPosition { x: grid_x, z: grid_z }
}

/// Convert grid position to world position (center of tile)
pub fn grid_to_world(grid_pos: &GridPosition) -> Vec3 {
    let world_x = (grid_pos.x as f32 - 10.0) + 0.5;
    let world_z = (grid_pos.z as f32 - 10.0) + 0.5;
    Vec3::new(world_x, 0.5, world_z)
}

/// Manhattan distance heuristic
fn heuristic(a: &GridPosition, b: &GridPosition) -> f32 {
    ((a.x - b.x).abs() + (a.z - b.z).abs()) as f32
}

/// Get neighboring grid positions (4-directional)
fn get_neighbors(pos: &GridPosition) -> Vec<GridPosition> {
    vec![
        GridPosition { x: pos.x + 1, z: pos.z },
        GridPosition { x: pos.x - 1, z: pos.z },
        GridPosition { x: pos.x, z: pos.z + 1 },
        GridPosition { x: pos.x, z: pos.z - 1 },
    ]
}

/// Check if a tile is traversible for a given unit base type
fn is_traversible(
    tiles: &Query<(&GridPosition, &TileProperties), With<Tile>>,
    pos: &GridPosition,
    unit_base: &UnitBase,
) -> bool {
    for (tile_pos, properties) in tiles.iter() {
        if tile_pos.x == pos.x && tile_pos.z == pos.z {
            // Check basic traversibility
            if !properties.traversible {
                return false;
            }

            // If tile is rugged, check if unit can traverse rugged terrain
            if properties.rugged && !unit_base.can_traverse_rugged() {
                return false;
            }

            return true;
        }
    }
    false
}

/// A* pathfinding algorithm
pub fn find_path(
    start: GridPosition,
    goal: GridPosition,
    tiles: &Query<(&GridPosition, &TileProperties), With<Tile>>,
    unit_base: &UnitBase,
) -> Option<Vec<GridPosition>> {
    let mut open_set = BinaryHeap::new();
    let mut closed_set = HashMap::new();
    let mut came_from: HashMap<(i32, i32), GridPosition> = HashMap::new();

    // Add start node
    let start_node = PathNode::new(start, 0.0, heuristic(&start, &goal), None);
    open_set.push(start_node);

    let mut iterations = 0;
    let max_iterations = 1000; // Prevent infinite loops

    while let Some(current) = open_set.pop() {
        iterations += 1;
        if iterations > max_iterations {
            warn!("Pathfinding exceeded max iterations");
            return None;
        }

        let current_key = (current.position.x, current.position.z);

        // Check if we reached the goal
        if current.position.x == goal.x && current.position.z == goal.z {
            // Reconstruct path
            let mut path = vec![current.position];
            let mut current_pos = current.position;

            while let Some(parent) = came_from.get(&(current_pos.x, current_pos.z)) {
                path.push(*parent);
                current_pos = *parent;
            }

            path.reverse();
            return Some(path);
        }

        // Skip if we've already processed this node with a better cost
        if let Some(&best_cost) = closed_set.get(&current_key) {
            if current.g_cost > best_cost {
                continue;
            }
        }

        closed_set.insert(current_key, current.g_cost);

        // Explore neighbors
        for neighbor_pos in get_neighbors(&current.position) {
            // Check bounds (20x20 grid, 0-19)
            if neighbor_pos.x < 0 || neighbor_pos.x >= 20 || neighbor_pos.z < 0 || neighbor_pos.z >= 20 {
                continue;
            }

            // Check if traversible
            if !is_traversible(tiles, &neighbor_pos, unit_base) {
                continue;
            }

            let neighbor_key = (neighbor_pos.x, neighbor_pos.z);

            // Calculate costs
            let tentative_g_cost = current.g_cost + 1.0;

            // Skip if we've seen this with a better cost
            if let Some(&best_cost) = closed_set.get(&neighbor_key) {
                if tentative_g_cost >= best_cost {
                    continue;
                }
            }

            // Add to open set
            let h_cost = heuristic(&neighbor_pos, &goal);
            let neighbor_node = PathNode::new(neighbor_pos, tentative_g_cost, h_cost, Some(current.position));

            came_from.insert(neighbor_key, current.position);
            open_set.push(neighbor_node);
        }
    }

    // No path found
    None
}

/// Smooth path by removing unnecessary waypoints
pub fn smooth_path(path: Vec<GridPosition>) -> Vec<Vec3> {
    if path.len() <= 2 {
        return path.iter().map(grid_to_world).collect();
    }

    let mut smoothed = vec![grid_to_world(&path[0])];

    // Simple smoothing: only keep waypoints where direction changes
    for i in 1..path.len() - 1 {
        let prev = &path[i - 1];
        let current = &path[i];
        let next = &path[i + 1];

        let dir1_x = current.x - prev.x;
        let dir1_z = current.z - prev.z;
        let dir2_x = next.x - current.x;
        let dir2_z = next.z - current.z;

        // If direction changes, keep this waypoint
        if dir1_x != dir2_x || dir1_z != dir2_z {
            smoothed.push(grid_to_world(current));
        }
    }

    smoothed.push(grid_to_world(&path[path.len() - 1]));
    smoothed
}
