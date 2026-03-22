// Shared type aliases for the units types module.
// Individual types are defined in commands.rs, movement.rs, and unit_data.rs.

use bevy::prelude::*;
use std::collections::HashSet;

/// Type of command indicator visual
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum CommandIndicatorType {
    /// Marker on the ground at a target position (cylinder)
    Location,
    /// Ring around a target entity's perimeter (torus, parented to target)
    Object,
}

/// Component linking an indicator entity back to its owning unit and command.
/// Used by the sync system to diff existing indicators against current commands.
#[derive(Component, Clone, Debug)]
pub struct CommandIndicator {
    /// The unit entity that owns this indicator
    pub owner_unit: Entity,
    /// The type of indicator (Location or Object)
    pub indicator_type: CommandIndicatorType,
    /// For Object indicators, the target entity being highlighted
    pub target_entity: Option<Entity>,
    /// Patrol index: 0 for most commands, 0=start 1=end for Patrol
    pub patrol_index: u8,
}

/// Returns the indicator color for a given UnitCommand variant.
/// Green = peaceful movement, Red = hostile target, Orange = aggressive movement.
pub fn command_indicator_color(cmd: &super::state::commands::UnitCommand) -> Color {
    use super::state::commands::UnitCommand;
    match cmd {
        UnitCommand::Move(_) | UnitCommand::Reverse(_) | UnitCommand::Enter(_)
        | UnitCommand::EnterArmory(_)
        | UnitCommand::Gather(_) | UnitCommand::DropOffResources(_)
        | UnitCommand::BuildTunnel(_) => {
            Color::srgb(0.0, 1.0, 0.0) // Green
        }
        UnitCommand::AttackTarget(_) | UnitCommand::AttackLocation(_) => {
            Color::srgb(1.0, 0.2, 0.0) // Red
        }
        UnitCommand::AttackMove(_) | UnitCommand::Patrol { .. } => {
            Color::srgb(1.0, 0.6, 0.0) // Orange
        }
        // Idle, HoldPosition, Stop, PickUpSupplies, AttachToTower — no indicator
        _ => Color::srgb(0.0, 0.0, 0.0),
    }
}

/// Returns true if a UnitCommand variant should have command indicators displayed.
pub fn command_has_indicator(cmd: &super::state::commands::UnitCommand) -> bool {
    use super::state::commands::UnitCommand;
    matches!(cmd,
        UnitCommand::Move(_) |
        UnitCommand::AttackTarget(_) |
        UnitCommand::AttackLocation(_) |
        UnitCommand::AttackMove(_) |
        UnitCommand::Patrol { .. } |
        UnitCommand::Reverse(_) |
        UnitCommand::Enter(_) |
        UnitCommand::EnterArmory(_) |
        UnitCommand::Gather(_) |
        UnitCommand::DropOffResources(_) |
        UnitCommand::BuildTunnel(_)
    )
}

/// Snapshot of a ground unit's collision body for movement-layer AABB checks.
/// Populated each frame by `rebuild_occupancy_map`.
#[derive(Clone, Debug)]
pub struct CollisionBody {
    pub entity: Entity,
    pub x: f32,
    pub z: f32,
    pub half_w: f32,
    pub half_h: f32,
}

/// Resource tracking tile occupancy (pathfinding) and collision bodies (movement).
/// Rebuilt each frame by `rebuild_occupancy_map` before pathfinding and movement run.
#[derive(Resource, Default, Clone, Debug)]
pub struct OccupancyMap {
    /// Tiles blocked by ground units or structure footprints (pathfinding layer).
    /// find_path checks this to avoid occupied tiles.
    pub blocked_tiles: HashSet<(i32, i32)>,
    /// Ground unit collision bodies for movement-layer AABB checks.
    pub ground_bodies: Vec<CollisionBody>,
    /// Structure footprint tiles for movement-layer collision checks.
    pub structure_tiles: HashSet<(i32, i32)>,
}

impl OccupancyMap {
    /// Clear all occupancy data for a fresh rebuild.
    pub fn clear(&mut self) {
        self.blocked_tiles.clear();
        self.ground_bodies.clear();
        self.structure_tiles.clear();
    }

    /// Check if a tile is blocked for pathfinding, with self-exclusion.
    pub fn is_blocked(&self, x: i32, z: i32, self_pos: (i32, i32)) -> bool {
        if (x, z) == self_pos {
            return false;
        }
        self.blocked_tiles.contains(&(x, z))
    }

    /// Check if moving to (new_x, new_z) with given half-extents would collide
    /// with any other ground unit or structure footprint.
    /// `self_entity` is excluded from unit-unit checks.
    pub fn check_movement_collision(
        &self,
        self_entity: Entity,
        new_x: f32,
        new_z: f32,
        half_w: f32,
        half_h: f32,
    ) -> bool {
        // Check against other ground unit AABBs
        for body in &self.ground_bodies {
            if body.entity == self_entity {
                continue;
            }
            if (new_x - body.x).abs() < half_w + body.half_w
                && (new_z - body.z).abs() < half_h + body.half_h
            {
                return true;
            }
        }
        // Check unit center against structure footprint tiles
        // Convert world position to grid position inline (avoids circular dependency)
        let grid_x = (new_x + 32.0).floor() as i32;
        let grid_z = (new_z + 32.0).floor() as i32;
        if self.structure_tiles.contains(&(grid_x, grid_z)) {
            return true;
        }
        false
    }
}

/// Marker component for units that need path recomputation after collision.
/// Added by movement systems when a unit is blocked. Removed by collision_repath_system
/// after successfully computing a new path.
#[derive(Component)]
pub struct NeedsRepath;

/// Tracks how many consecutive repath attempts have failed for a unit.
/// After MAX_REPATH_ATTEMPTS, the unit stops trying and clears its move target.
#[derive(Component)]
pub struct RepathAttempts(pub u8);

/// Maximum number of consecutive failed repath attempts before giving up.
pub const MAX_REPATH_ATTEMPTS: u8 = 5;

#[cfg(test)]
mod tests {
    use super::*;
    use super::super::state::commands::UnitCommand;

    #[test]
    fn move_command_has_indicator() {
        assert!(command_has_indicator(&UnitCommand::Move(Vec3::ZERO)));
    }

    #[test]
    fn idle_has_no_indicator() {
        assert!(!command_has_indicator(&UnitCommand::Idle));
    }

    #[test]
    fn hold_position_has_no_indicator() {
        assert!(!command_has_indicator(&UnitCommand::HoldPosition));
    }

    #[test]
    fn stop_has_no_indicator() {
        assert!(!command_has_indicator(&UnitCommand::Stop));
    }

    #[test]
    fn attack_target_has_indicator() {
        assert!(command_has_indicator(&UnitCommand::AttackTarget(Entity::from_raw_u32(1).unwrap())));
    }

    #[test]
    fn attack_location_has_indicator() {
        assert!(command_has_indicator(&UnitCommand::AttackLocation(Vec3::ZERO)));
    }

    #[test]
    fn attack_move_has_indicator() {
        assert!(command_has_indicator(&UnitCommand::AttackMove(Vec3::ZERO)));
    }

    #[test]
    fn patrol_has_indicator() {
        assert!(command_has_indicator(&UnitCommand::Patrol {
            start: Vec3::ZERO,
            end: Vec3::ONE,
            going_to_end: true,
        }));
    }

    #[test]
    fn reverse_has_indicator() {
        assert!(command_has_indicator(&UnitCommand::Reverse(Vec3::ZERO)));
    }

    #[test]
    fn enter_has_indicator() {
        assert!(command_has_indicator(&UnitCommand::Enter(Entity::from_raw_u32(1).unwrap())));
    }

    #[test]
    fn pick_up_supplies_has_no_indicator() {
        assert!(!command_has_indicator(&UnitCommand::PickUpSupplies(Entity::from_raw_u32(1).unwrap())));
    }

    #[test]
    fn attach_to_tower_has_no_indicator() {
        assert!(!command_has_indicator(&UnitCommand::AttachToTower(Entity::from_raw_u32(1).unwrap())));
    }

    #[test]
    fn move_color_is_green() {
        let color = command_indicator_color(&UnitCommand::Move(Vec3::ZERO));
        assert_eq!(color, Color::srgb(0.0, 1.0, 0.0));
    }

    #[test]
    fn reverse_color_is_green() {
        let color = command_indicator_color(&UnitCommand::Reverse(Vec3::ZERO));
        assert_eq!(color, Color::srgb(0.0, 1.0, 0.0));
    }

    #[test]
    fn enter_color_is_green() {
        let color = command_indicator_color(&UnitCommand::Enter(Entity::from_raw_u32(1).unwrap()));
        assert_eq!(color, Color::srgb(0.0, 1.0, 0.0));
    }

    #[test]
    fn attack_target_color_is_red() {
        let color = command_indicator_color(&UnitCommand::AttackTarget(Entity::from_raw_u32(1).unwrap()));
        assert_eq!(color, Color::srgb(1.0, 0.2, 0.0));
    }

    #[test]
    fn attack_location_color_is_red() {
        let color = command_indicator_color(&UnitCommand::AttackLocation(Vec3::ZERO));
        assert_eq!(color, Color::srgb(1.0, 0.2, 0.0));
    }

    #[test]
    fn attack_move_color_is_orange() {
        let color = command_indicator_color(&UnitCommand::AttackMove(Vec3::ZERO));
        assert_eq!(color, Color::srgb(1.0, 0.6, 0.0));
    }

    #[test]
    fn patrol_color_is_orange() {
        let color = command_indicator_color(&UnitCommand::Patrol {
            start: Vec3::ZERO, end: Vec3::ONE, going_to_end: true,
        });
        assert_eq!(color, Color::srgb(1.0, 0.6, 0.0));
    }

    #[test]
    fn command_indicator_type_equality() {
        assert_eq!(CommandIndicatorType::Location, CommandIndicatorType::Location);
        assert_ne!(CommandIndicatorType::Location, CommandIndicatorType::Object);
    }

    #[test]
    fn command_indicator_component_creation() {
        let indicator = CommandIndicator {
            owner_unit: Entity::from_raw_u32(1).unwrap(),
            indicator_type: CommandIndicatorType::Location,
            target_entity: None,
            patrol_index: 0,
        };
        assert_eq!(indicator.owner_unit, Entity::from_raw_u32(1).unwrap());
        assert_eq!(indicator.indicator_type, CommandIndicatorType::Location);
        assert!(indicator.target_entity.is_none());
        assert_eq!(indicator.patrol_index, 0);
    }

    #[test]
    fn command_indicator_object_type() {
        let indicator = CommandIndicator {
            owner_unit: Entity::from_raw_u32(1).unwrap(),
            indicator_type: CommandIndicatorType::Object,
            target_entity: Some(Entity::from_raw_u32(2).unwrap()),
            patrol_index: 0,
        };
        assert_eq!(indicator.indicator_type, CommandIndicatorType::Object);
        assert_eq!(indicator.target_entity, Some(Entity::from_raw_u32(2).unwrap()));
    }

    // === OccupancyMap tests ===

    #[test]
    fn occupancy_map_default_is_empty() {
        let map = OccupancyMap::default();
        assert!(map.blocked_tiles.is_empty());
        assert!(map.ground_bodies.is_empty());
        assert!(map.structure_tiles.is_empty());
    }

    #[test]
    fn occupancy_map_is_blocked_returns_true_for_occupied() {
        let mut map = OccupancyMap::default();
        map.blocked_tiles.insert((5, 5));
        assert!(map.is_blocked(5, 5, (0, 0)));
    }

    #[test]
    fn occupancy_map_is_blocked_self_exclusion() {
        let mut map = OccupancyMap::default();
        map.blocked_tiles.insert((5, 5));
        // Self-exclusion: unit's own tile is not blocked
        assert!(!map.is_blocked(5, 5, (5, 5)));
    }

    #[test]
    fn occupancy_map_is_blocked_returns_false_for_free() {
        let map = OccupancyMap::default();
        assert!(!map.is_blocked(3, 3, (0, 0)));
    }

    #[test]
    fn occupancy_map_clear_resets_all() {
        let mut map = OccupancyMap::default();
        map.blocked_tiles.insert((1, 1));
        map.structure_tiles.insert((2, 2));
        map.ground_bodies.push(CollisionBody {
            entity: Entity::from_raw_u32(1).unwrap(),
            x: 0.0, z: 0.0,
            half_w: 0.5, half_h: 0.5,
        });
        map.clear();
        assert!(map.blocked_tiles.is_empty());
        assert!(map.ground_bodies.is_empty());
        assert!(map.structure_tiles.is_empty());
    }

    #[test]
    fn occupancy_map_unit_aabb_collision_detected() {
        let mut map = OccupancyMap::default();
        let other = Entity::from_raw_u32(10).unwrap();
        map.ground_bodies.push(CollisionBody {
            entity: other,
            x: 1.0, z: 1.0,
            half_w: 0.2, half_h: 0.2,
        });
        let self_e = Entity::from_raw_u32(20).unwrap();
        // Overlapping position
        assert!(map.check_movement_collision(self_e, 1.1, 1.1, 0.2, 0.2));
    }

    #[test]
    fn occupancy_map_unit_aabb_collision_self_excluded() {
        let mut map = OccupancyMap::default();
        let self_e = Entity::from_raw_u32(10).unwrap();
        map.ground_bodies.push(CollisionBody {
            entity: self_e,
            x: 1.0, z: 1.0,
            half_w: 0.2, half_h: 0.2,
        });
        // Should NOT collide with self
        assert!(!map.check_movement_collision(self_e, 1.0, 1.0, 0.2, 0.2));
    }

    #[test]
    fn occupancy_map_unit_aabb_no_collision_when_far() {
        let mut map = OccupancyMap::default();
        let other = Entity::from_raw_u32(10).unwrap();
        map.ground_bodies.push(CollisionBody {
            entity: other,
            x: 10.0, z: 10.0,
            half_w: 0.2, half_h: 0.2,
        });
        let self_e = Entity::from_raw_u32(20).unwrap();
        // Far away — no collision
        assert!(!map.check_movement_collision(self_e, 0.0, 0.0, 0.2, 0.2));
    }

    #[test]
    fn occupancy_map_structure_tile_collision() {
        let mut map = OccupancyMap::default();
        // Structure at grid (32, 32) — world center (0.5, 0.5)
        map.structure_tiles.insert((32, 32));
        let self_e = Entity::from_raw_u32(20).unwrap();
        // World position that maps to grid (32, 32): x + 32 = 32, so x = 0.0
        assert!(map.check_movement_collision(self_e, 0.0, 0.0, 0.2, 0.2));
    }

    #[test]
    fn occupancy_map_no_structure_collision_on_free_tile() {
        let mut map = OccupancyMap::default();
        map.structure_tiles.insert((32, 32));
        let self_e = Entity::from_raw_u32(20).unwrap();
        // Position at grid (33, 33) — no structure
        assert!(!map.check_movement_collision(self_e, 1.0, 1.0, 0.2, 0.2));
    }

    // === RepathAttempts tests ===

    #[test]
    fn repath_attempts_default_value() {
        let attempts = RepathAttempts(0);
        assert_eq!(attempts.0, 0);
    }

    #[test]
    fn repath_attempts_increment() {
        let attempts = RepathAttempts(3);
        let next = RepathAttempts(attempts.0 + 1);
        assert_eq!(next.0, 4);
    }

    #[test]
    fn max_repath_attempts_is_five() {
        assert_eq!(MAX_REPATH_ATTEMPTS, 5);
    }

    #[test]
    fn repath_attempts_at_max_triggers_give_up() {
        let attempts = RepathAttempts(MAX_REPATH_ATTEMPTS);
        assert!(attempts.0 >= MAX_REPATH_ATTEMPTS);
    }

    #[test]
    fn repath_attempts_below_max_allows_retry() {
        for i in 0..MAX_REPATH_ATTEMPTS {
            let attempts = RepathAttempts(i);
            assert!(attempts.0 < MAX_REPATH_ATTEMPTS);
        }
    }

    // === Gather/DropOff command indicator tests ===

    #[test]
    fn gather_has_indicator() {
        assert!(command_has_indicator(&UnitCommand::Gather(Entity::from_raw_u32(1).unwrap())));
    }

    #[test]
    fn drop_off_resources_has_indicator() {
        assert!(command_has_indicator(&UnitCommand::DropOffResources(Entity::from_raw_u32(1).unwrap())));
    }

    #[test]
    fn gather_color_is_green() {
        let color = command_indicator_color(&UnitCommand::Gather(Entity::from_raw_u32(1).unwrap()));
        assert_eq!(color, Color::srgb(0.0, 1.0, 0.0));
    }

    #[test]
    fn drop_off_resources_color_is_green() {
        let color = command_indicator_color(&UnitCommand::DropOffResources(Entity::from_raw_u32(1).unwrap()));
        assert_eq!(color, Color::srgb(0.0, 1.0, 0.0));
    }

    #[test]
    fn build_tunnel_has_indicator() {
        assert!(command_has_indicator(&UnitCommand::BuildTunnel(Vec3::ZERO)));
    }

    #[test]
    fn build_tunnel_color_is_green() {
        let color = command_indicator_color(&UnitCommand::BuildTunnel(Vec3::ZERO));
        assert_eq!(color, Color::srgb(0.0, 1.0, 0.0));
    }

    #[test]
    fn enter_armory_has_indicator() {
        assert!(command_has_indicator(&UnitCommand::EnterArmory(Entity::from_raw_u32(1).unwrap())));
    }

    #[test]
    fn enter_armory_color_is_green() {
        let color = command_indicator_color(&UnitCommand::EnterArmory(Entity::from_raw_u32(1).unwrap()));
        assert_eq!(color, Color::srgb(0.0, 1.0, 0.0));
    }
}
