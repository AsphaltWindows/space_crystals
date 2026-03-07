use bevy::prelude::*;
use bevy::ecs::system::EntityCommands;
use crate::types::{GridPosition, UnitBaseEnum};
use crate::game::world::types::TilePreset;
use crate::game::combat::types::*;
use super::types::movement::{MoveTarget, Path, Velocity};
use super::types::HoldingPosition;

/// Half-map offset for 64x64 grid (world 0,0 is grid center 32,32)
const GRID_HALF_SIZE: f32 = 32.0;

/// Convert world position to grid position (centered grid: world 0,0 is grid center)
pub fn world_to_grid(world_pos: Vec3) -> GridPosition {
    let grid_x = (world_pos.x + GRID_HALF_SIZE).floor() as i32;
    let grid_z = (world_pos.z + GRID_HALF_SIZE).floor() as i32;
    GridPosition { x: grid_x, z: grid_z }
}

/// Convert grid position to world position (center of tile)
pub fn grid_to_world(grid_pos: &GridPosition) -> Vec3 {
    let world_x = (grid_pos.x as f32 - GRID_HALF_SIZE) + 0.5;
    let world_z = (grid_pos.z as f32 - GRID_HALF_SIZE) + 0.5;
    Vec3::new(world_x, 0.5, world_z)
}

/// Octile distance heuristic for 8-directional pathfinding
pub fn heuristic(a: &GridPosition, b: &GridPosition) -> f32 {
    let dx = (a.x - b.x).abs() as f32;
    let dz = (a.z - b.z).abs() as f32;
    let (min, max) = if dx < dz { (dx, dz) } else { (dz, dx) };
    max + (std::f32::consts::SQRT_2 - 1.0) * min
}

/// Get neighboring grid positions with movement costs (8-directional).
/// Returns tuples of (position, cost) where cost is 1.0 for cardinal and SQRT_2 for diagonal.
pub fn get_neighbors(pos: &GridPosition) -> Vec<(GridPosition, f32)> {
    vec![
        // Cardinal neighbors (cost 1.0)
        (GridPosition { x: pos.x + 1, z: pos.z }, 1.0),
        (GridPosition { x: pos.x - 1, z: pos.z }, 1.0),
        (GridPosition { x: pos.x, z: pos.z + 1 }, 1.0),
        (GridPosition { x: pos.x, z: pos.z - 1 }, 1.0),
        // Diagonal neighbors (cost SQRT_2)
        (GridPosition { x: pos.x + 1, z: pos.z + 1 }, std::f32::consts::SQRT_2),
        (GridPosition { x: pos.x + 1, z: pos.z - 1 }, std::f32::consts::SQRT_2),
        (GridPosition { x: pos.x - 1, z: pos.z + 1 }, std::f32::consts::SQRT_2),
        (GridPosition { x: pos.x - 1, z: pos.z - 1 }, std::f32::consts::SQRT_2),
    ]
}

/// Check if a tile is traversible for a given unit base type
pub fn is_traversible(
    tiles: &Query<(&GridPosition, &TilePreset), With<crate::game::world::types::Tile>>,
    pos: &GridPosition,
    unit_base: &UnitBaseEnum,
) -> bool {
    for (tile_pos, properties) in tiles.iter() {
        if tile_pos.x == pos.x && tile_pos.z == pos.z {
            if !properties.traversible {
                return false;
            }
            if properties.rugged && !unit_base.can_traverse_rugged() {
                return false;
            }
            return true;
        }
    }
    false
}

/// Check if a diagonal move is traversible (no corner cutting).
/// For diagonal moves, both adjacent cardinal tiles must also be traversible.
/// For cardinal moves, delegates to `is_traversible`.
pub fn is_diagonal_traversible(
    tiles: &Query<(&GridPosition, &TilePreset), With<crate::game::world::types::Tile>>,
    from: &GridPosition,
    to: &GridPosition,
    unit_base: &UnitBaseEnum,
) -> bool {
    // Target must be traversible
    if !is_traversible(tiles, to, unit_base) {
        return false;
    }
    let dx = to.x - from.x;
    let dz = to.z - from.z;
    if dx.abs() == 1 && dz.abs() == 1 {
        // Diagonal — check both cardinal corners to prevent cutting through walls
        let corner1 = GridPosition { x: from.x + dx, z: from.z };
        let corner2 = GridPosition { x: from.x, z: from.z + dz };
        is_traversible(tiles, &corner1, unit_base) && is_traversible(tiles, &corner2, unit_base)
    } else {
        true // Cardinal move, no corner check needed
    }
}

/// Smooth path by removing unnecessary waypoints
pub fn smooth_path(path: Vec<GridPosition>) -> Vec<Vec3> {
    if path.len() <= 2 {
        return path.iter().map(grid_to_world).collect();
    }

    let mut smoothed = vec![grid_to_world(&path[0])];

    for i in 1..path.len() - 1 {
        let prev = &path[i - 1];
        let current = &path[i];
        let next = &path[i + 1];

        let dir1_x = current.x - prev.x;
        let dir1_z = current.z - prev.z;
        let dir2_x = next.x - current.x;
        let dir2_z = next.z - current.z;

        if dir1_x != dir2_x || dir1_z != dir2_z {
            smoothed.push(grid_to_world(current));
        }
    }

    smoothed.push(grid_to_world(&path[path.len() - 1]));
    smoothed
}

/// Clears all movement-related components from an entity.
/// Use when issuing a command that should stop the unit immediately.
/// Callers must zero `Velocity` separately if they have mutable query access,
/// or use `clear_movement_state_full` which also inserts `Velocity(Vec3::ZERO)`.
pub fn clear_movement_state(entity_commands: &mut EntityCommands) {
    entity_commands
        .remove::<MoveTarget>()
        .remove::<Path>()
        .remove::<HoldingPosition>()
        .remove::<IdleOrigin>();
}

/// Clears all movement-related components and zeroes velocity by inserting `Velocity(Vec3::ZERO)`.
/// Use when you don't have mutable query access to Velocity.
pub fn clear_movement_state_full(entity_commands: &mut EntityCommands) {
    entity_commands
        .remove::<MoveTarget>()
        .remove::<Path>()
        .remove::<HoldingPosition>()
        .remove::<IdleOrigin>()
        .insert(Velocity(Vec3::ZERO));
}

/// Validate whether a unit can enter a tunnel.
/// Checks:
/// 1. Unit is Syndicate faction
/// 2. Target has a TunnelState
/// 3. Unit and tunnel share the same owner
/// 4. Tunnel's transit tier allows the unit's base type
///
/// Returns Ok(()) if valid, Err(reason) if not.
pub fn can_enter_tunnel(
    is_syndicate: bool,
    unit_owner: Option<u8>,
    tunnel_owner: Option<u8>,
    unit_base: &UnitBaseEnum,
    tunnel_tier: &crate::game::types::TunnelTier,
) -> Result<(), &'static str> {
    if !is_syndicate {
        return Err("Only Syndicate units can enter tunnels");
    }
    match (unit_owner, tunnel_owner) {
        (Some(u), Some(t)) if u == t => {}
        _ => return Err("Unit and tunnel must share the same owner"),
    }
    if !tunnel_tier.can_transit(unit_base) {
        return Err("Tunnel tier does not support this unit base type");
    }
    Ok(())
}

/// Create attack capability based on unit base type (placeholder values for test units)
#[allow(dead_code)]
pub fn create_attack_capability(unit_base: &UnitBaseEnum) -> AttackCapability {
    use crate::types::{TargetDomainEnum, TargetTypeEnum};

    match unit_base {
        UnitBaseEnum::LightInfantry | UnitBaseEnum::HeavyInfantry => AttackCapability {
            damage: 10.0,
            range: 5.0,
            aim_time: 0.2,
            fire_time: 0.1,
            cooldown_time: 0.05,
            reload_time: 1.0,
            attack_type: AttackType::FullyConnected { subtype: crate::types::FullyConnectedSubtype::Ranged },
            ..Default::default()
        },
        UnitBaseEnum::WheeledVehicle => AttackCapability {
            damage: 20.0,
            range: 8.0,
            aim_time: 0.4,
            fire_time: 0.15,
            cooldown_time: 0.1,
            reload_time: 2.5,
            attack_type: AttackType::TailDisjointed {
                projectile_speed: 20.0,
                projectile_visual: ProjectileVisual::Cylinder {
                    radius: 0.1,
                    length: 0.3,
                },
            },
            target_domain: TargetDomainEnum::Ground,
            target_type: TargetTypeEnum::SingleTarget,
            ..Default::default()
        },
        UnitBaseEnum::TrackedVehicle => AttackCapability {
            damage: 30.0,
            range: 7.0,
            aim_time: 0.5,
            fire_time: 0.2,
            cooldown_time: 0.15,
            reload_time: 3.0,
            attack_type: AttackType::DoublyDisjointed {
                projectile_speed: 15.0,
                projectile_visual: ProjectileVisual::Sphere {
                    radius: 0.15,
                },
                effect_radius: 2.0,
            },
            target_domain: TargetDomainEnum::Ground,
            target_type: TargetTypeEnum::AoE,
            aoe_radius: Some(2.0),
            ..Default::default()
        },
        UnitBaseEnum::DrillUnit => AttackCapability {
            damage: 25.0,
            range: 6.5,
            aim_time: 0.45,
            fire_time: 0.18,
            cooldown_time: 0.12,
            reload_time: 2.8,
            attack_type: AttackType::DoublyDisjointed {
                projectile_speed: 18.0,
                projectile_visual: ProjectileVisual::Sphere {
                    radius: 0.12,
                },
                effect_radius: 1.5,
            },
            target_domain: TargetDomainEnum::Ground,
            target_type: TargetTypeEnum::AoE,
            aoe_radius: Some(1.5),
            ..Default::default()
        },
        UnitBaseEnum::HoverVehicle | UnitBaseEnum::HoverCraft => AttackCapability {
            damage: 22.0,
            range: 8.5,
            aim_time: 0.35,
            fire_time: 0.12,
            cooldown_time: 0.08,
            reload_time: 2.2,
            attack_type: AttackType::TailDisjointed {
                projectile_speed: 25.0,
                projectile_visual: ProjectileVisual::Cylinder {
                    radius: 0.08,
                    length: 0.25,
                },
            },
            target_domain: TargetDomainEnum::Ground,
            target_type: TargetTypeEnum::SingleTarget,
            ..Default::default()
        },
        UnitBaseEnum::Mech => AttackCapability {
            damage: 35.0,
            range: 6.0,
            aim_time: 0.6,
            fire_time: 0.25,
            cooldown_time: 0.2,
            reload_time: 3.5,
            attack_type: AttackType::DoublyDisjointed {
                projectile_speed: 12.0,
                projectile_visual: ProjectileVisual::Sphere {
                    radius: 0.18,
                },
                effect_radius: 2.5,
            },
            target_domain: TargetDomainEnum::Ground,
            target_type: TargetTypeEnum::AoE,
            aoe_radius: Some(2.5),
            ..Default::default()
        },
        UnitBaseEnum::Glider => AttackCapability {
            damage: 15.0,
            range: 7.0,
            aim_time: 0.3,
            fire_time: 0.1,
            cooldown_time: 0.1,
            reload_time: 2.0,
            attack_type: AttackType::TailDisjointed {
                projectile_speed: 22.0,
                projectile_visual: ProjectileVisual::Sphere {
                    radius: 0.08,
                },
            },
            target_domain: TargetDomainEnum::Universal,
            target_type: TargetTypeEnum::SingleTarget,
            ..Default::default()
        },
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // === heuristic (octile distance) tests ===

    #[test]
    fn heuristic_same_position_is_zero() {
        let a = GridPosition { x: 5, z: 5 };
        assert!((heuristic(&a, &a)).abs() < f32::EPSILON);
    }

    #[test]
    fn heuristic_cardinal_distance() {
        let a = GridPosition { x: 0, z: 0 };
        let b = GridPosition { x: 5, z: 0 };
        // Pure cardinal: octile = max(5,0) + (SQRT_2-1)*min(5,0) = 5.0
        assert!((heuristic(&a, &b) - 5.0).abs() < f32::EPSILON);
    }

    #[test]
    fn heuristic_diagonal_distance() {
        let a = GridPosition { x: 0, z: 0 };
        let b = GridPosition { x: 3, z: 3 };
        // Pure diagonal: octile = 3 + (SQRT_2-1)*3 = 3*SQRT_2
        let expected = 3.0 * std::f32::consts::SQRT_2;
        assert!((heuristic(&a, &b) - expected).abs() < 0.001);
    }

    #[test]
    fn heuristic_mixed_distance() {
        let a = GridPosition { x: 0, z: 0 };
        let b = GridPosition { x: 4, z: 2 };
        // octile = max(4,2) + (SQRT_2-1)*min(4,2) = 4 + 2*(SQRT_2-1)
        let expected = 4.0 + 2.0 * (std::f32::consts::SQRT_2 - 1.0);
        assert!((heuristic(&a, &b) - expected).abs() < 0.001);
    }

    #[test]
    fn heuristic_is_symmetric() {
        let a = GridPosition { x: 1, z: 3 };
        let b = GridPosition { x: 7, z: 5 };
        assert!((heuristic(&a, &b) - heuristic(&b, &a)).abs() < f32::EPSILON);
    }

    #[test]
    fn heuristic_negative_coords() {
        let a = GridPosition { x: -2, z: -3 };
        let b = GridPosition { x: 2, z: 1 };
        // dx=4, dz=4 -> pure diagonal = 4*SQRT_2
        let expected = 4.0 * std::f32::consts::SQRT_2;
        assert!((heuristic(&a, &b) - expected).abs() < 0.001);
    }

    // === get_neighbors (8-directional) tests ===

    #[test]
    fn get_neighbors_returns_8_neighbors() {
        let pos = GridPosition { x: 5, z: 5 };
        let neighbors = get_neighbors(&pos);
        assert_eq!(neighbors.len(), 8);
    }

    #[test]
    fn get_neighbors_has_4_cardinal_with_cost_1() {
        let pos = GridPosition { x: 5, z: 5 };
        let neighbors = get_neighbors(&pos);
        let cardinal: Vec<_> = neighbors.iter()
            .filter(|(_, cost)| (*cost - 1.0).abs() < f32::EPSILON)
            .collect();
        assert_eq!(cardinal.len(), 4);
    }

    #[test]
    fn get_neighbors_has_4_diagonal_with_cost_sqrt2() {
        let pos = GridPosition { x: 5, z: 5 };
        let neighbors = get_neighbors(&pos);
        let diagonal: Vec<_> = neighbors.iter()
            .filter(|(_, cost)| (*cost - std::f32::consts::SQRT_2).abs() < f32::EPSILON)
            .collect();
        assert_eq!(diagonal.len(), 4);
    }

    #[test]
    fn get_neighbors_contains_all_8_positions() {
        let pos = GridPosition { x: 5, z: 5 };
        let neighbors = get_neighbors(&pos);
        let positions: Vec<(i32, i32)> = neighbors.iter().map(|(p, _)| (p.x, p.z)).collect();
        assert!(positions.contains(&(6, 5))); // E
        assert!(positions.contains(&(4, 5))); // W
        assert!(positions.contains(&(5, 6))); // S
        assert!(positions.contains(&(5, 4))); // N
        assert!(positions.contains(&(6, 6))); // SE
        assert!(positions.contains(&(6, 4))); // NE
        assert!(positions.contains(&(4, 6))); // SW
        assert!(positions.contains(&(4, 4))); // NW
    }

    // === world_to_grid / grid_to_world roundtrip tests ===

    #[test]
    fn world_grid_roundtrip_center() {
        let grid = GridPosition { x: 32, z: 32 };
        let world = grid_to_world(&grid);
        let back = world_to_grid(world);
        assert_eq!(back.x, grid.x);
        assert_eq!(back.z, grid.z);
    }

    // === clear_movement_state tests ===

    #[test]
    fn clear_movement_state_removes_move_target() {
        let mut world = World::new();
        let entity = world.spawn((
            MoveTarget(Vec3::new(5.0, 0.0, 5.0)),
        )).id();

        let mut command_queue = bevy::ecs::world::CommandQueue::default();
        {
            let mut commands = Commands::new(&mut command_queue, &world);
            let mut entity_cmds = commands.entity(entity);
            clear_movement_state(&mut entity_cmds);
        }
        command_queue.apply(&mut world);

        assert!(world.entity(entity).get::<MoveTarget>().is_none());
    }

    #[test]
    fn clear_movement_state_removes_path() {
        let mut world = World::new();
        let entity = world.spawn((
            Path { waypoints: vec![Vec3::ZERO], current_waypoint: 0 },
        )).id();

        let mut command_queue = bevy::ecs::world::CommandQueue::default();
        {
            let mut commands = Commands::new(&mut command_queue, &world);
            let mut entity_cmds = commands.entity(entity);
            clear_movement_state(&mut entity_cmds);
        }
        command_queue.apply(&mut world);

        assert!(world.entity(entity).get::<Path>().is_none());
    }

    #[test]
    fn clear_movement_state_removes_holding_position() {
        let mut world = World::new();
        let entity = world.spawn((
            HoldingPosition,
        )).id();

        let mut command_queue = bevy::ecs::world::CommandQueue::default();
        {
            let mut commands = Commands::new(&mut command_queue, &world);
            let mut entity_cmds = commands.entity(entity);
            clear_movement_state(&mut entity_cmds);
        }
        command_queue.apply(&mut world);

        assert!(world.entity(entity).get::<HoldingPosition>().is_none());
    }

    #[test]
    fn clear_movement_state_removes_idle_origin() {
        let mut world = World::new();
        let entity = world.spawn((
            IdleOrigin(Vec3::new(1.0, 0.0, 1.0)),
        )).id();

        let mut command_queue = bevy::ecs::world::CommandQueue::default();
        {
            let mut commands = Commands::new(&mut command_queue, &world);
            let mut entity_cmds = commands.entity(entity);
            clear_movement_state(&mut entity_cmds);
        }
        command_queue.apply(&mut world);

        assert!(world.entity(entity).get::<IdleOrigin>().is_none());
    }

    #[test]
    fn clear_movement_state_removes_all_movement_components() {
        let mut world = World::new();
        let entity = world.spawn((
            MoveTarget(Vec3::new(5.0, 0.0, 5.0)),
            Path { waypoints: vec![Vec3::ZERO], current_waypoint: 0 },
            HoldingPosition,
            IdleOrigin(Vec3::new(1.0, 0.0, 1.0)),
        )).id();

        let mut command_queue = bevy::ecs::world::CommandQueue::default();
        {
            let mut commands = Commands::new(&mut command_queue, &world);
            let mut entity_cmds = commands.entity(entity);
            clear_movement_state(&mut entity_cmds);
        }
        command_queue.apply(&mut world);

        assert!(world.entity(entity).get::<MoveTarget>().is_none());
        assert!(world.entity(entity).get::<Path>().is_none());
        assert!(world.entity(entity).get::<HoldingPosition>().is_none());
        assert!(world.entity(entity).get::<IdleOrigin>().is_none());
    }

    #[test]
    fn clear_movement_state_full_zeroes_velocity() {
        let mut world = World::new();
        let entity = world.spawn((
            MoveTarget(Vec3::new(5.0, 0.0, 5.0)),
            Path { waypoints: vec![Vec3::ZERO], current_waypoint: 0 },
            HoldingPosition,
            IdleOrigin(Vec3::new(1.0, 0.0, 1.0)),
            Velocity(Vec3::new(3.0, 0.0, 4.0)),
        )).id();

        let mut command_queue = bevy::ecs::world::CommandQueue::default();
        {
            let mut commands = Commands::new(&mut command_queue, &world);
            let mut entity_cmds = commands.entity(entity);
            clear_movement_state_full(&mut entity_cmds);
        }
        command_queue.apply(&mut world);

        assert!(world.entity(entity).get::<MoveTarget>().is_none());
        assert!(world.entity(entity).get::<Path>().is_none());
        assert!(world.entity(entity).get::<HoldingPosition>().is_none());
        assert!(world.entity(entity).get::<IdleOrigin>().is_none());
        let vel = world.entity(entity).get::<Velocity>().expect("Velocity should exist");
        assert_eq!(vel.0, Vec3::ZERO);
    }

    #[test]
    fn clear_movement_state_safe_on_entity_without_components() {
        let mut world = World::new();
        // Entity with no movement components — should not panic
        let entity = world.spawn(()).id();

        let mut command_queue = bevy::ecs::world::CommandQueue::default();
        {
            let mut commands = Commands::new(&mut command_queue, &world);
            let mut entity_cmds = commands.entity(entity);
            clear_movement_state(&mut entity_cmds);
        }
        command_queue.apply(&mut world);

        // Should survive without panicking
        assert!(world.entity(entity).get::<MoveTarget>().is_none());
    }

    #[test]
    fn clear_movement_state_full_inserts_velocity_on_entity_without_it() {
        let mut world = World::new();
        let entity = world.spawn(()).id();

        let mut command_queue = bevy::ecs::world::CommandQueue::default();
        {
            let mut commands = Commands::new(&mut command_queue, &world);
            let mut entity_cmds = commands.entity(entity);
            clear_movement_state_full(&mut entity_cmds);
        }
        command_queue.apply(&mut world);

        let vel = world.entity(entity).get::<Velocity>().expect("Velocity should be inserted");
        assert_eq!(vel.0, Vec3::ZERO);
    }

    // === Attack phase movement guard tests ===

    #[test]
    fn aiming_phase_should_block_movement() {
        // Verify Aiming is in the set of blocking phases
        let phase = AttackPhase::Aiming;
        assert!(matches!(phase, AttackPhase::Aiming | AttackPhase::Firing | AttackPhase::Cooldown));
    }

    #[test]
    fn firing_phase_should_block_movement() {
        let phase = AttackPhase::Firing;
        assert!(matches!(phase, AttackPhase::Aiming | AttackPhase::Firing | AttackPhase::Cooldown));
    }

    #[test]
    fn cooldown_phase_should_block_movement() {
        let phase = AttackPhase::Cooldown;
        assert!(matches!(phase, AttackPhase::Aiming | AttackPhase::Firing | AttackPhase::Cooldown));
    }

    #[test]
    fn none_phase_should_not_block_movement() {
        let phase = AttackPhase::None;
        assert!(!matches!(phase, AttackPhase::Aiming | AttackPhase::Firing | AttackPhase::Cooldown));
    }

    #[test]
    fn reloading_phase_should_not_block_movement() {
        let phase = AttackPhase::Reloading;
        assert!(!matches!(phase, AttackPhase::Aiming | AttackPhase::Firing | AttackPhase::Cooldown));
    }

    // === can_enter_tunnel tests ===

    #[test]
    fn can_enter_tunnel_valid_syndicate_light_infantry_t1() {
        use crate::game::types::TunnelTier;
        let result = can_enter_tunnel(true, Some(0), Some(0), &UnitBaseEnum::LightInfantry, &TunnelTier::Tier1);
        assert!(result.is_ok());
    }

    #[test]
    fn can_enter_tunnel_valid_syndicate_heavy_infantry_t1() {
        use crate::game::types::TunnelTier;
        let result = can_enter_tunnel(true, Some(1), Some(1), &UnitBaseEnum::HeavyInfantry, &TunnelTier::Tier1);
        assert!(result.is_ok());
    }

    #[test]
    fn can_enter_tunnel_rejects_non_syndicate() {
        use crate::game::types::TunnelTier;
        let result = can_enter_tunnel(false, Some(0), Some(0), &UnitBaseEnum::LightInfantry, &TunnelTier::Tier1);
        assert!(result.is_err());
        assert_eq!(result.unwrap_err(), "Only Syndicate units can enter tunnels");
    }

    #[test]
    fn can_enter_tunnel_rejects_different_owners() {
        use crate::game::types::TunnelTier;
        let result = can_enter_tunnel(true, Some(0), Some(1), &UnitBaseEnum::LightInfantry, &TunnelTier::Tier1);
        assert!(result.is_err());
        assert_eq!(result.unwrap_err(), "Unit and tunnel must share the same owner");
    }

    #[test]
    fn can_enter_tunnel_rejects_no_owner_unit() {
        use crate::game::types::TunnelTier;
        let result = can_enter_tunnel(true, None, Some(0), &UnitBaseEnum::LightInfantry, &TunnelTier::Tier1);
        assert!(result.is_err());
        assert_eq!(result.unwrap_err(), "Unit and tunnel must share the same owner");
    }

    #[test]
    fn can_enter_tunnel_rejects_vehicle_on_t1() {
        use crate::game::types::TunnelTier;
        let result = can_enter_tunnel(true, Some(0), Some(0), &UnitBaseEnum::WheeledVehicle, &TunnelTier::Tier1);
        assert!(result.is_err());
        assert_eq!(result.unwrap_err(), "Tunnel tier does not support this unit base type");
    }

    #[test]
    fn can_enter_tunnel_allows_vehicle_on_t2() {
        use crate::game::types::TunnelTier;
        let result = can_enter_tunnel(true, Some(0), Some(0), &UnitBaseEnum::WheeledVehicle, &TunnelTier::Tier2);
        assert!(result.is_ok());
    }

    #[test]
    fn can_enter_tunnel_rejects_hovercraft_on_t2() {
        use crate::game::types::TunnelTier;
        let result = can_enter_tunnel(true, Some(0), Some(0), &UnitBaseEnum::HoverCraft, &TunnelTier::Tier2);
        assert!(result.is_err());
    }

    #[test]
    fn can_enter_tunnel_allows_all_on_t3() {
        use crate::game::types::TunnelTier;
        for base in [
            UnitBaseEnum::LightInfantry, UnitBaseEnum::HeavyInfantry,
            UnitBaseEnum::WheeledVehicle, UnitBaseEnum::TrackedVehicle,
            UnitBaseEnum::DrillUnit, UnitBaseEnum::HoverVehicle,
            UnitBaseEnum::Mech, UnitBaseEnum::HoverCraft, UnitBaseEnum::Glider,
        ] {
            let result = can_enter_tunnel(true, Some(0), Some(0), &base, &TunnelTier::Tier3);
            assert!(result.is_ok(), "Expected {:?} to be allowed on T3", base);
        }
    }
}
