#![allow(dead_code)]
use bevy::prelude::*;
use crate::types::{Owner, FullyConnectedSubtype, TargetDomainEnum, TargetTypeEnum};

/// Visual style for projectiles
#[derive(Clone, Copy, Debug)]
pub enum ProjectileVisual {
    Sphere { radius: f32 },
    Cylinder { radius: f32, length: f32 },
}

/// Placeholder melee range in grid units (adjacent contact).
/// Concrete value TBD by design — use small fixed value for now.
pub const MELEE_RANGE: f32 = 0.75;

/// Types of attacks (from design doc)
#[derive(Clone, Debug)]
pub enum AttackType {
    FullyConnected { subtype: FullyConnectedSubtype },
    TailDisjointed {
        projectile_speed: f32,
        projectile_visual: ProjectileVisual,
    },
    HeadDisjointed {
        effect_radius: f32,
    },
    DoublyDisjointed {
        projectile_speed: f32,
        projectile_visual: ProjectileVisual,
        effect_radius: f32,
    },
}

/// Component defining a unit's attack capabilities
#[derive(Component, Clone)]
pub struct AttackCapability {
    pub damage: f32,
    pub range: f32,
    pub min_range: f32,
    pub aim_time: f32,
    pub fire_time: f32,
    pub cooldown_time: f32,
    pub reload_time: f32,
    pub attack_type: AttackType,
    pub target_domain: TargetDomainEnum,
    pub target_type: TargetTypeEnum,
    pub aoe_radius: Option<f32>,
}

impl Default for AttackCapability {
    fn default() -> Self {
        Self {
            damage: 10.0,
            range: 5.0,
            min_range: 0.0,
            aim_time: 0.3,
            fire_time: 0.1,
            cooldown_time: 0.1,
            reload_time: 1.0,
            attack_type: AttackType::FullyConnected { subtype: FullyConnectedSubtype::Ranged },
            target_domain: TargetDomainEnum::Ground,
            target_type: TargetTypeEnum::SingleTarget,
            aoe_radius: None,
        }
    }
}

impl AttackCapability {
    /// Whether this attack is a FullyConnected Melee attack
    pub fn is_melee(&self) -> bool {
        matches!(self.attack_type, AttackType::FullyConnected { subtype: FullyConnectedSubtype::Melee })
    }
}

/// Whether the attack originates from the unit base or an independent turret
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum AttackSourceEnum {
    /// Entire unit turns to aim; movement blocked during Aim/Fire/Cooldown
    UnitBase,
    /// Turret rotates independently; movement always allowed
    Turret,
}

/// Target of an attack — either a specific entity or a map location
#[derive(Clone, Copy, Debug)]
pub enum AttackTarget {
    /// Track and attack a specific entity
    UnitTarget(Entity),
    /// Attack a fixed map location
    LocationTarget(Vec3),
}

/// Attack phase states
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum AttackPhase {
    None,
    Aiming,
    Firing,
    Cooldown,
    Reloading,
}

/// Actions that can be performed by the unit base during each attack phase
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct PhaseActionConstraints {
    pub base_can_move: bool,
    pub base_can_turn: bool,
}

impl AttackPhase {
    /// Whether this phase can be interrupted by new commands.
    /// Aiming and Reloading are interruptible; Firing and Cooldown are not.
    /// None is always interruptible (not attacking).
    pub fn is_interruptible(&self) -> bool {
        matches!(self, AttackPhase::None | AttackPhase::Aiming | AttackPhase::Reloading)
    }

    /// Returns action constraints for the unit base given the attack source type.
    /// `is_turret_source` = true when the attacking weapon is a turret (unit base is free to move).
    pub fn base_action_constraints(&self, is_turret_source: bool) -> PhaseActionConstraints {
        if is_turret_source {
            // Turret source: base can always move and turn
            PhaseActionConstraints { base_can_move: true, base_can_turn: true }
        } else {
            // UnitBase source: restricted during attack phases
            match self {
                AttackPhase::None => PhaseActionConstraints { base_can_move: true, base_can_turn: true },
                AttackPhase::Aiming => PhaseActionConstraints { base_can_move: false, base_can_turn: true },
                AttackPhase::Firing | AttackPhase::Cooldown => PhaseActionConstraints { base_can_move: false, base_can_turn: false },
                AttackPhase::Reloading => PhaseActionConstraints { base_can_move: true, base_can_turn: true },
            }
        }
    }
}

/// Component tracking current attack state
#[derive(Component)]
pub struct AttackState {
    pub phase: AttackPhase,
    pub time_in_phase: f32,
    pub current_target: Option<AttackTarget>,
}

impl Default for AttackState {
    fn default() -> Self {
        Self {
            phase: AttackPhase::None,
            time_in_phase: 0.0,
            current_target: None,
        }
    }
}

impl AttackState {
    /// Get the target entity, if the current target is a UnitTarget
    pub fn target_entity(&self) -> Option<Entity> {
        match self.current_target {
            Some(AttackTarget::UnitTarget(e)) => Some(e),
            _ => None,
        }
    }

    /// Get the target location, if the current target is a LocationTarget
    pub fn target_location(&self) -> Option<Vec3> {
        match self.current_target {
            Some(AttackTarget::LocationTarget(v)) => Some(v),
            _ => None,
        }
    }
}

/// Directional armor front multiplier — armor is stronger when hit from the front
pub const DIRECTIONAL_ARMOR_FRONT_MULTIPLIER: f32 = 1.5;
/// Directional armor rear multiplier — armor is weaker when hit from behind
pub const DIRECTIONAL_ARMOR_REAR_MULTIPLIER: f32 = 0.5;
/// Dot product threshold above which an attack is considered frontal
pub const DIRECTIONAL_ARMOR_FRONT_THRESHOLD: f32 = 0.5;
/// Dot product threshold below which an attack is considered a rear hit
pub const DIRECTIONAL_ARMOR_REAR_THRESHOLD: f32 = -0.5;

/// Component defining a unit's armor properties
#[derive(Component, Clone, Debug)]
pub struct Armor {
    /// Armor applied against single-target (point) attacks
    pub point_armor: f32,
    /// Armor applied against area-of-effect attacks
    pub full_armor: f32,
    /// Whether armor effectiveness varies by attack direction
    pub directional_armor: bool,
}

/// Component defining a unit's silhouette dimensions for AoE overlap calculations
#[derive(Component, Clone, Debug)]
pub struct Silhouette {
    /// Width in space units
    pub width: f32,
    /// Height in space units
    pub height: f32,
}

/// Component defining the soft separation radius for air units (in grid units).
/// Air units within this radius of each other experience a gentle repulsion force
/// that prevents stacking without hard-blocking movement.
/// Must be larger than the unit's Silhouette.
#[derive(Component, Clone, Debug)]
pub struct SeparationRadius(pub f32);

/// Force scale for air unit soft separation (grid units per second).
/// Controls how quickly overlapping air units drift apart.
pub const SEPARATION_FORCE_SCALE: f32 = 2.0;

/// Component for damage events — distinguishes single-target from area-of-effect damage
#[derive(Component)]
pub enum DamageEvent {
    /// Direct hit on a specific target
    SingleTarget {
        damage: f32,
        source: Entity,
        /// Position of the attacker at time of fire (for directional armor calculation)
        source_position: Vec3,
    },
    /// Area damage centered on a location
    AreaOfEffect {
        damage: f32,
        source: Entity,
        /// Center of the AoE explosion
        center: Vec3,
        /// Radius of the AoE effect
        radius: f32,
        /// Owner of the attack source (for friendly-fire filtering)
        source_owner: Owner,
    },
}

/// Component defining turret properties
#[derive(Component)]
pub struct Turret {
    pub turn_angle: f32,
    pub turn_rate: f32,
    pub current_angle: f32,
    pub target_angle: Option<f32>,
}

impl Turret {
    pub fn full_rotation(turn_rate: f32) -> Self {
        Self {
            turn_angle: std::f32::consts::PI * 2.0,
            turn_rate,
            current_angle: 0.0,
            target_angle: None,
        }
    }

    pub fn limited_rotation(max_angle: f32, turn_rate: f32) -> Self {
        Self {
            turn_angle: max_angle,
            turn_rate,
            current_angle: 0.0,
            target_angle: None,
        }
    }

    pub fn can_reach_angle(&self, angle: f32) -> bool {
        let half_angle = self.turn_angle / 2.0;
        angle.abs() <= half_angle
    }

    pub fn clamp_angle(&self, angle: f32) -> f32 {
        let half_angle = self.turn_angle / 2.0;
        angle.clamp(-half_angle, half_angle)
    }
}

/// Marker component for turret visual entity
#[derive(Component)]
pub struct TurretVisual {
    pub parent_unit: Entity,
}

/// Component for projectile entities
#[derive(Component)]
pub struct Projectile {
    pub target_position: Vec3,
    pub speed: f32,
    pub damage: f32,
    pub effect_radius: Option<f32>,
    pub source_owner: Owner,
}

/// Component for visual explosion effects
#[derive(Component)]
pub struct ExplosionEffect {
    pub lifetime: f32,
    pub max_lifetime: f32,
    /// Base scale factor (matches the explosion radius). The animation
    /// multiplies this by a growth factor so a unit sphere renders at
    /// the correct visual radius.
    pub base_scale: f32,
}

/// Component for attack line tracer visual (FullyConnected attacks)
#[derive(Component)]
pub struct AttackLine {
    pub lifetime: f32,
    pub max_lifetime: f32,
}

/// Component for attack target confirmation highlight
#[derive(Component)]
pub struct TargetHighlight {
    pub lifetime: f32,
    pub max_lifetime: f32,
}

/// Cached mesh and material handles for combat visual effects.
/// Created once at game start to avoid per-spawn asset allocation.
/// All combat spawn functions use handles from this cache instead of
/// calling `meshes.add()` / `materials.add()` per entity.
#[derive(Resource)]
pub struct CombatAssetCache {
    /// Unit cuboid (half-extents 1.0) for attack lines — scaled per-instance
    pub attack_line_mesh: Handle<Mesh>,
    /// Bright yellow-white unlit material for attack lines
    pub attack_line_material: Handle<StandardMaterial>,
    /// Unit sphere (radius 1.0) for sphere projectiles and explosions — scaled per-instance
    pub unit_sphere_mesh: Handle<Mesh>,
    /// Gold emissive material for sphere projectiles
    pub projectile_sphere_material: Handle<StandardMaterial>,
    /// Metallic gray material for cylinder/capsule projectiles
    pub projectile_cylinder_material: Handle<StandardMaterial>,
    /// Orange emissive blend material for explosions
    pub explosion_material: Handle<StandardMaterial>,
}

impl CombatAssetCache {
    /// Initialize all cached combat assets.
    pub fn new(
        meshes: &mut Assets<Mesh>,
        materials: &mut Assets<StandardMaterial>,
    ) -> Self {
        Self {
            attack_line_mesh: meshes.add(Cuboid::new(1.0, 1.0, 1.0)),
            attack_line_material: materials.add(StandardMaterial {
                base_color: Color::srgb(1.0, 1.0, 0.6),
                emissive: Color::srgb(2.0, 2.0, 0.5).into(),
                unlit: true,
                ..default()
            }),
            unit_sphere_mesh: meshes.add(Sphere::new(1.0)),
            projectile_sphere_material: materials.add(StandardMaterial {
                base_color: Color::srgb(1.0, 0.8, 0.0),
                emissive: Color::srgb(1.0, 0.8, 0.0).into(),
                ..default()
            }),
            projectile_cylinder_material: materials.add(StandardMaterial {
                base_color: Color::srgb(0.8, 0.8, 0.8),
                metallic: 0.8,
                ..default()
            }),
            explosion_material: materials.add(StandardMaterial {
                base_color: Color::srgb(1.0, 0.5, 0.0),
                emissive: Color::srgb(1.0, 0.3, 0.0).into(),
                alpha_mode: AlphaMode::Blend,
                ..default()
            }),
        }
    }
}

/// Component recording a unit's position when it auto-acquired an idle target.
/// Used for leash distance checking.
#[derive(Component)]
pub struct IdleOrigin(pub Vec3);

/// Maximum distance an idle-auto-targeting unit can chase before disengaging
pub const IDLE_LEASH_DISTANCE: f32 = 4.0;

/// Maximum perpendicular distance from original path before an AttackMove unit disengages
/// and returns to its path. 6.0 GU per design spec.
pub const ATTACK_MOVE_LEASH_DISTANCE: f32 = 6.0;

/// Component recording a unit's position/path origin during AttackMove engagement.
/// Used for leash distance checking during AttackMove detours.
#[derive(Component)]
pub struct AttackMoveOrigin(pub Vec3);

/// Component recording patrol waypoints for patrol scanning behavior.
/// When a patrolling unit detects an enemy, it temporarily engages,
/// then resumes patrol from current position.
#[derive(Component)]
pub struct PatrolEngaged {
    pub patrol_start: Vec3,
    pub patrol_end: Vec3,
    pub going_to_end: bool,
}

/// Facing arc threshold for non-turning infantry in HoldPosition (PI/6 radians = 30 degrees)
pub const HOLD_POSITION_FACING_ARC: f32 = std::f32::consts::FRAC_PI_6;

#[cfg(test)]
mod tests {
    use super::*;

    // === AttackPhase::is_interruptible ===

    #[test]
    fn none_phase_is_interruptible() {
        assert!(AttackPhase::None.is_interruptible());
    }

    #[test]
    fn aiming_phase_is_interruptible() {
        assert!(AttackPhase::Aiming.is_interruptible());
    }

    #[test]
    fn firing_phase_is_not_interruptible() {
        assert!(!AttackPhase::Firing.is_interruptible());
    }

    #[test]
    fn cooldown_phase_is_not_interruptible() {
        assert!(!AttackPhase::Cooldown.is_interruptible());
    }

    #[test]
    fn reloading_phase_is_interruptible() {
        assert!(AttackPhase::Reloading.is_interruptible());
    }

    // === PhaseActionConstraints — UnitBase source (no turret) ===

    #[test]
    fn unit_base_none_allows_all() {
        let c = AttackPhase::None.base_action_constraints(false);
        assert!(c.base_can_move);
        assert!(c.base_can_turn);
    }

    #[test]
    fn unit_base_aiming_blocks_move_allows_turn() {
        let c = AttackPhase::Aiming.base_action_constraints(false);
        assert!(!c.base_can_move);
        assert!(c.base_can_turn);
    }

    #[test]
    fn unit_base_firing_blocks_all() {
        let c = AttackPhase::Firing.base_action_constraints(false);
        assert!(!c.base_can_move);
        assert!(!c.base_can_turn);
    }

    #[test]
    fn unit_base_cooldown_blocks_all() {
        let c = AttackPhase::Cooldown.base_action_constraints(false);
        assert!(!c.base_can_move);
        assert!(!c.base_can_turn);
    }

    #[test]
    fn unit_base_reloading_allows_all() {
        let c = AttackPhase::Reloading.base_action_constraints(false);
        assert!(c.base_can_move);
        assert!(c.base_can_turn);
    }

    // === PhaseActionConstraints — Turret source ===

    #[test]
    fn turret_source_none_allows_all() {
        let c = AttackPhase::None.base_action_constraints(true);
        assert!(c.base_can_move);
        assert!(c.base_can_turn);
    }

    #[test]
    fn turret_source_aiming_allows_all() {
        let c = AttackPhase::Aiming.base_action_constraints(true);
        assert!(c.base_can_move);
        assert!(c.base_can_turn);
    }

    #[test]
    fn turret_source_firing_allows_all() {
        let c = AttackPhase::Firing.base_action_constraints(true);
        assert!(c.base_can_move);
        assert!(c.base_can_turn);
    }

    #[test]
    fn turret_source_cooldown_allows_all() {
        let c = AttackPhase::Cooldown.base_action_constraints(true);
        assert!(c.base_can_move);
        assert!(c.base_can_turn);
    }

    #[test]
    fn turret_source_reloading_allows_all() {
        let c = AttackPhase::Reloading.base_action_constraints(true);
        assert!(c.base_can_move);
        assert!(c.base_can_turn);
    }

    // === PhaseActionConstraints equality ===

    #[test]
    fn phase_action_constraints_equality() {
        let a = PhaseActionConstraints { base_can_move: true, base_can_turn: false };
        let b = PhaseActionConstraints { base_can_move: true, base_can_turn: false };
        let c = PhaseActionConstraints { base_can_move: false, base_can_turn: false };
        assert_eq!(a, b);
        assert_ne!(a, c);
    }

    // === Interruptibility consistency check ===

    #[test]
    fn interruptible_phases_allow_move_for_unit_base() {
        // All interruptible phases should allow movement for unit base source
        // (except Aiming which allows turn but not move)
        for phase in [AttackPhase::None, AttackPhase::Reloading] {
            assert!(phase.is_interruptible());
            let c = phase.base_action_constraints(false);
            assert!(c.base_can_move, "Phase {:?} should allow move", phase);
        }
    }

    #[test]
    fn non_interruptible_phases_block_move_for_unit_base() {
        for phase in [AttackPhase::Firing, AttackPhase::Cooldown] {
            assert!(!phase.is_interruptible());
            let c = phase.base_action_constraints(false);
            assert!(!c.base_can_move, "Phase {:?} should block move", phase);
        }
    }

    #[test]
    fn turret_source_always_allows_move_regardless_of_phase() {
        for phase in [AttackPhase::None, AttackPhase::Aiming, AttackPhase::Firing, AttackPhase::Cooldown, AttackPhase::Reloading] {
            let c = phase.base_action_constraints(true);
            assert!(c.base_can_move, "Turret source phase {:?} should allow move", phase);
            assert!(c.base_can_turn, "Turret source phase {:?} should allow turn", phase);
        }
    }

    // === CombatAssetCache ===

    #[test]
    fn combat_asset_cache_creates_all_handles() {
        let mut meshes = Assets::<Mesh>::default();
        let mut materials = Assets::<StandardMaterial>::default();
        let cache = CombatAssetCache::new(&mut meshes, &mut materials);
        // All handles should reference valid assets
        assert!(meshes.get(&cache.attack_line_mesh).is_some());
        assert!(meshes.get(&cache.unit_sphere_mesh).is_some());
        assert!(materials.get(&cache.attack_line_material).is_some());
        assert!(materials.get(&cache.projectile_sphere_material).is_some());
        assert!(materials.get(&cache.projectile_cylinder_material).is_some());
        assert!(materials.get(&cache.explosion_material).is_some());
    }

    #[test]
    fn combat_asset_cache_attack_line_and_sphere_are_distinct() {
        let mut meshes = Assets::<Mesh>::default();
        let mut materials = Assets::<StandardMaterial>::default();
        let cache = CombatAssetCache::new(&mut meshes, &mut materials);
        // Different meshes for attack lines vs spheres
        assert_ne!(cache.attack_line_mesh, cache.unit_sphere_mesh);
        // Different materials for each combat visual type
        assert_ne!(cache.attack_line_material, cache.projectile_sphere_material);
        assert_ne!(cache.projectile_sphere_material, cache.projectile_cylinder_material);
        assert_ne!(cache.explosion_material, cache.attack_line_material);
    }

    #[test]
    fn combat_asset_cache_only_creates_two_meshes() {
        let mut meshes = Assets::<Mesh>::default();
        let mut materials = Assets::<StandardMaterial>::default();
        let _cache = CombatAssetCache::new(&mut meshes, &mut materials);
        // Exactly 2 meshes: unit cuboid + unit sphere
        assert_eq!(meshes.len(), 2);
        // Exactly 4 materials: attack_line, projectile_sphere, projectile_cylinder, explosion
        assert_eq!(materials.len(), 4);
    }

    // === ExplosionEffect base_scale ===

    #[test]
    fn explosion_effect_base_scale_stored() {
        let effect = ExplosionEffect {
            lifetime: 0.0,
            max_lifetime: 0.5,
            base_scale: 3.0,
        };
        assert_eq!(effect.base_scale, 3.0);
    }

    #[test]
    fn explosion_animation_at_start_equals_base_scale() {
        let base_scale = 2.5_f32;
        let progress = 0.0_f32;
        let scale = base_scale * (1.0 + progress * 2.0);
        assert_eq!(scale, 2.5);
    }

    #[test]
    fn explosion_animation_at_end_triples_base_scale() {
        let base_scale = 2.0_f32;
        let progress = 1.0_f32;
        let scale = base_scale * (1.0 + progress * 2.0);
        assert_eq!(scale, 6.0); // 2.0 * 3.0
    }

    #[test]
    fn explosion_animation_at_midpoint() {
        let base_scale = 1.0_f32;
        let progress = 0.5_f32;
        let scale = base_scale * (1.0 + progress * 2.0);
        assert_eq!(scale, 2.0); // 1.0 * 2.0
    }
}
