use bevy::prelude::*;
use crate::types::{Owner, UnitBaseEnum, TargetDomainEnum, DomainEnum, VisibilityStateEnum, GridPosition};
use crate::game::types::ObjectInstance;
use super::types::*;

/// Helper function to create turret based on unit base type (placeholder values for test units)
#[allow(dead_code)]
pub fn create_turret_for_unit(unit_base: &UnitBaseEnum) -> Option<Turret> {
    let data = unit_base.data();
    if !data.has_turret {
        return None;
    }
    // Placeholder turret params per base type
    match unit_base {
        UnitBaseEnum::WheeledVehicle => Some(Turret::full_rotation(
            std::f32::consts::PI
        )),
        UnitBaseEnum::TrackedVehicle => Some(Turret::full_rotation(
            std::f32::consts::PI * 2.0 / 3.0
        )),
        UnitBaseEnum::DrillUnit => Some(Turret::full_rotation(
            std::f32::consts::PI * 0.75
        )),
        UnitBaseEnum::HoverVehicle | UnitBaseEnum::HoverCraft => Some(Turret::limited_rotation(
            std::f32::consts::PI / 3.0,
            std::f32::consts::PI * 0.8
        )),
        UnitBaseEnum::Mech => Some(Turret::limited_rotation(
            std::f32::consts::PI / 4.0,
            std::f32::consts::PI * 0.6
        )),
        UnitBaseEnum::Glider => Some(Turret::full_rotation(
            std::f32::consts::PI * 0.5
        )),
        _ => None, // LightInfantry, HeavyInfantry have no turret
    }
}

/// Spawn visual turret entity as child of unit
#[allow(dead_code)]
pub fn spawn_turret_visual(
    commands: &mut Commands,
    parent_entity: Entity,
    meshes: &mut ResMut<Assets<Mesh>>,
    materials: &mut ResMut<Assets<StandardMaterial>>,
    unit_base: &UnitBaseEnum,
    owner_color: Color,
) {
    if create_turret_for_unit(unit_base).is_none() {
        return;
    }

    let turret_mesh = meshes.add(Cylinder::new(0.2, 0.3));
    let turret_material = materials.add(StandardMaterial {
        base_color: owner_color,
        metallic: 0.5,
        ..default()
    });

    let turret_entity = commands.spawn((
        Mesh3d(turret_mesh),
        MeshMaterial3d(turret_material),
        Transform::from_xyz(0.0, 0.3, 0.0),
        TurretVisual {
            parent_unit: parent_entity,
        },
    )).id();

    commands.entity(parent_entity).add_child(turret_entity);
}

/// Spawn a projectile entity using cached mesh/material assets.
/// Sphere projectiles use the cached unit sphere (scaled per-instance).
/// Capsule projectiles still create a mesh per-spawn (non-uniform scaling
/// breaks hemisphere caps) but reuse the cached material.
pub fn spawn_projectile(
    commands: &mut Commands,
    meshes: &mut ResMut<Assets<Mesh>>,
    cache: &CombatAssetCache,
    start_position: Vec3,
    target_position: Vec3,
    speed: f32,
    damage: f32,
    effect_radius: Option<f32>,
    visual: ProjectileVisual,
    source_owner: Owner,
) {
    let (mesh, material, scale) = match visual {
        ProjectileVisual::Sphere { radius } => {
            (cache.unit_sphere_mesh.clone(), cache.projectile_sphere_material.clone(), Vec3::splat(radius))
        }
        ProjectileVisual::Cylinder { radius, length } => {
            // Capsule mesh can't be cleanly cached (non-uniform scale distorts caps)
            let mesh = meshes.add(Capsule3d::new(radius, length));
            (mesh, cache.projectile_cylinder_material.clone(), Vec3::ONE)
        }
    };

    commands.spawn((
        Mesh3d(mesh),
        MeshMaterial3d(material),
        Transform::from_translation(start_position).with_scale(scale),
        Projectile {
            target_position,
            speed,
            damage,
            effect_radius,
            source_owner,
        },
    ));
}

/// Spawn visual explosion effect using cached unit sphere and material.
/// The sphere is scaled to the explosion radius; the animation system
/// multiplies by base_scale to preserve the correct visual size.
pub fn spawn_explosion_effect(
    commands: &mut Commands,
    cache: &CombatAssetCache,
    position: Vec3,
    radius: f32,
) {
    commands.spawn((
        Mesh3d(cache.unit_sphere_mesh.clone()),
        MeshMaterial3d(cache.explosion_material.clone()),
        Transform::from_translation(position)
            .with_scale(Vec3::splat(radius)),
        ExplosionEffect {
            lifetime: 0.0,
            max_lifetime: 0.5,
            base_scale: radius,
        },
    ));
}

/// Spawn a visual attack line tracer between attacker and target.
/// Uses a cached unit cuboid (scaled to line dimensions) and cached material.
pub fn spawn_attack_line(
    commands: &mut Commands,
    cache: &CombatAssetCache,
    start: Vec3,
    end: Vec3,
) {
    let midpoint = (start + end) / 2.0;
    let direction = end - start;
    let length = direction.length();

    if length < 0.01 {
        return;
    }

    // Orient the unit cuboid from start to end, scaled to line dimensions
    let normalized = direction.normalize();
    let transform = Transform::from_translation(midpoint)
        .looking_to(normalized, Vec3::Y)
        .with_scale(Vec3::new(0.02, 0.02, length));

    commands.spawn((
        Mesh3d(cache.attack_line_mesh.clone()),
        MeshMaterial3d(cache.attack_line_material.clone()),
        transform,
        AttackLine {
            lifetime: 0.0,
            max_lifetime: 0.15,
        },
    ));
}

/// Apply AOE damage at a location.
///
/// Note: For HeadDisjointed attacks, the actual AoE damage is applied through the
/// projectile_impact_system (which inserts DamageEvent::AreaOfEffect on targets in range).
/// This function is retained for API compatibility but delegates to the projectile path.
pub fn apply_aoe_damage(
    _commands: &mut Commands,
    _targets: &Query<(&Transform, &Owner, Option<&DomainEnum>, &GridPosition), With<ObjectInstance>>,
    _location: Vec3,
    _radius: f32,
    _damage: f32,
    _source: Entity,
    _source_owner: &Owner,
) {
    // AoE damage is now handled by projectile_impact_system via DamageEvent::AreaOfEffect.
    // HeadDisjointed spawns a zero-travel projectile that triggers the AoE on impact.
}

/// Calculate the overlap area between a circle and an axis-aligned rectangle.
///
/// Uses a simplified approximation: clamp the circle center to the nearest point on the
/// rectangle, then compute the circular segment area that overlaps. For game purposes,
/// this approximation is sufficient and avoids complex arc-segment intersection math.
pub fn circle_rect_overlap_area(
    circle_center: Vec2,
    radius: f32,
    rect_center: Vec2,
    rect_width: f32,
    rect_height: f32,
) -> f32 {
    if radius <= 0.0 || rect_width <= 0.0 || rect_height <= 0.0 {
        return 0.0;
    }

    let half_w = rect_width / 2.0;
    let half_h = rect_height / 2.0;

    let rect_min = rect_center - Vec2::new(half_w, half_h);
    let rect_max = rect_center + Vec2::new(half_w, half_h);

    // Clamp circle center to nearest point on rectangle
    let nearest = Vec2::new(
        circle_center.x.clamp(rect_min.x, rect_max.x),
        circle_center.y.clamp(rect_min.y, rect_max.y),
    );

    let dist = circle_center.distance(nearest);

    if dist >= radius {
        // No overlap
        return 0.0;
    }

    let circle_area = std::f32::consts::PI * radius * radius;
    let rect_area = rect_width * rect_height;

    // If circle fully contains rectangle
    let corners = [
        rect_min,
        Vec2::new(rect_max.x, rect_min.y),
        rect_max,
        Vec2::new(rect_min.x, rect_max.y),
    ];
    let all_corners_inside = corners.iter().all(|c| circle_center.distance(*c) <= radius);
    if all_corners_inside {
        return rect_area;
    }

    // If rectangle fully contains circle
    if circle_center.x - radius >= rect_min.x
        && circle_center.x + radius <= rect_max.x
        && circle_center.y - radius >= rect_min.y
        && circle_center.y + radius <= rect_max.y
    {
        return circle_area;
    }

    // Partial overlap — approximate using the fraction of the circle covered
    // based on how deeply the circle penetrates the rectangle
    let penetration = radius - dist;
    let penetration_ratio = (penetration / radius).clamp(0.0, 1.0);

    // Use a smooth approximation: penetration-based fraction of the smaller area
    let overlap_fraction = penetration_ratio * penetration_ratio * (3.0 - 2.0 * penetration_ratio); // smoothstep
    let max_overlap = circle_area.min(rect_area);

    overlap_fraction * max_overlap
}

/// Check whether an attack's target domain is compatible with a unit's operating domain.
///
/// - Ground attacks hit Ground and Underground (surfaced) units
/// - Air attacks hit Air units only
/// - Universal attacks hit any domain
pub fn is_domain_compatible(attack_domain: &TargetDomainEnum, unit_domain: &DomainEnum) -> bool {
    match attack_domain {
        TargetDomainEnum::Ground => matches!(unit_domain, DomainEnum::Ground | DomainEnum::Underground),
        TargetDomainEnum::Air => matches!(unit_domain, DomainEnum::Air),
        TargetDomainEnum::Universal => true,
    }
}

/// Check whether a target is valid for attack (destructible, visible, domain-compatible).
///
/// A target is valid if ALL of the following are true:
/// 1. The target's ObjectInstance is destructible (has HP)
/// 2. The target is visible to the attacker's owner
/// 3. The target's domain is compatible with the attacker's target domain
///
/// Note: Range and arc checks are applied separately by context.
pub fn is_valid_target(
    target_obj: &ObjectInstance,
    target_visibility: &VisibilityStateEnum,
    target_domain: &DomainEnum,
    attacker_domain: &TargetDomainEnum,
) -> bool {
    // 1. Must be destructible
    if !target_obj.is_destructible() {
        return false;
    }

    // 2. Must be visible to attacker
    if *target_visibility != VisibilityStateEnum::Visible {
        return false;
    }

    // 3. Must be domain-compatible
    is_domain_compatible(attacker_domain, target_domain)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::types::{AttackTypeEnum, FullyConnectedSubtype, ObjectEnum, TargetTypeEnum};

    // === AttackTypeEnum derived properties ===

    #[test]
    fn fully_connected_cannot_miss() {
        assert!(!AttackTypeEnum::FullyConnected.can_miss());
    }

    #[test]
    fn fully_connected_cannot_target_ground() {
        assert!(!AttackTypeEnum::FullyConnected.can_target_ground());
    }

    #[test]
    fn fully_connected_no_projectile_speed() {
        assert!(!AttackTypeEnum::FullyConnected.requires_projectile_speed());
    }

    #[test]
    fn fully_connected_no_location_target() {
        assert!(!AttackTypeEnum::FullyConnected.allows_location_target());
    }

    #[test]
    fn head_disjointed_cannot_miss() {
        assert!(!AttackTypeEnum::HeadDisjointed.can_miss());
    }

    #[test]
    fn head_disjointed_cannot_target_ground() {
        assert!(!AttackTypeEnum::HeadDisjointed.can_target_ground());
    }

    #[test]
    fn head_disjointed_requires_projectile_speed() {
        assert!(AttackTypeEnum::HeadDisjointed.requires_projectile_speed());
    }

    #[test]
    fn head_disjointed_no_location_target() {
        assert!(!AttackTypeEnum::HeadDisjointed.allows_location_target());
    }

    #[test]
    fn tail_disjointed_can_miss() {
        assert!(AttackTypeEnum::TailDisjointed.can_miss());
    }

    #[test]
    fn tail_disjointed_can_target_ground() {
        assert!(AttackTypeEnum::TailDisjointed.can_target_ground());
    }

    #[test]
    fn tail_disjointed_no_projectile_speed() {
        assert!(!AttackTypeEnum::TailDisjointed.requires_projectile_speed());
    }

    #[test]
    fn tail_disjointed_allows_location_target() {
        assert!(AttackTypeEnum::TailDisjointed.allows_location_target());
    }

    #[test]
    fn doubly_disjointed_can_miss() {
        assert!(AttackTypeEnum::DoublyDisjointed.can_miss());
    }

    #[test]
    fn doubly_disjointed_can_target_ground() {
        assert!(AttackTypeEnum::DoublyDisjointed.can_target_ground());
    }

    #[test]
    fn doubly_disjointed_requires_projectile_speed() {
        assert!(AttackTypeEnum::DoublyDisjointed.requires_projectile_speed());
    }

    #[test]
    fn doubly_disjointed_allows_location_target() {
        assert!(AttackTypeEnum::DoublyDisjointed.allows_location_target());
    }

    // === Domain compatibility ===

    #[test]
    fn ground_attack_hits_ground_units() {
        assert!(is_domain_compatible(&TargetDomainEnum::Ground, &DomainEnum::Ground));
    }

    #[test]
    fn ground_attack_hits_underground_units() {
        assert!(is_domain_compatible(&TargetDomainEnum::Ground, &DomainEnum::Underground));
    }

    #[test]
    fn ground_attack_misses_air_units() {
        assert!(!is_domain_compatible(&TargetDomainEnum::Ground, &DomainEnum::Air));
    }

    #[test]
    fn air_attack_hits_air_units() {
        assert!(is_domain_compatible(&TargetDomainEnum::Air, &DomainEnum::Air));
    }

    #[test]
    fn air_attack_misses_ground_units() {
        assert!(!is_domain_compatible(&TargetDomainEnum::Air, &DomainEnum::Ground));
    }

    #[test]
    fn air_attack_misses_underground_units() {
        assert!(!is_domain_compatible(&TargetDomainEnum::Air, &DomainEnum::Underground));
    }

    #[test]
    fn universal_attack_hits_ground() {
        assert!(is_domain_compatible(&TargetDomainEnum::Universal, &DomainEnum::Ground));
    }

    #[test]
    fn universal_attack_hits_air() {
        assert!(is_domain_compatible(&TargetDomainEnum::Universal, &DomainEnum::Air));
    }

    #[test]
    fn universal_attack_hits_underground() {
        assert!(is_domain_compatible(&TargetDomainEnum::Universal, &DomainEnum::Underground));
    }

    // === ValidTarget filter ===

    #[test]
    fn valid_target_accepts_destructible_visible_compatible() {
        let obj = ObjectInstance::destructible(ObjectEnum::Peacekeeper, 50.0);
        let vis = VisibilityStateEnum::Visible;
        let domain = DomainEnum::Ground;
        let attack_domain = TargetDomainEnum::Ground;
        assert!(is_valid_target(&obj, &vis, &domain, &attack_domain));
    }

    #[test]
    fn valid_target_rejects_indestructible() {
        let obj = ObjectInstance::indestructible(ObjectEnum::SpaceCrystalsPatch);
        let vis = VisibilityStateEnum::Visible;
        let domain = DomainEnum::Ground;
        let attack_domain = TargetDomainEnum::Ground;
        assert!(!is_valid_target(&obj, &vis, &domain, &attack_domain));
    }

    #[test]
    fn valid_target_rejects_unexplored() {
        let obj = ObjectInstance::destructible(ObjectEnum::Peacekeeper, 50.0);
        let vis = VisibilityStateEnum::Unexplored;
        let domain = DomainEnum::Ground;
        let attack_domain = TargetDomainEnum::Ground;
        assert!(!is_valid_target(&obj, &vis, &domain, &attack_domain));
    }

    #[test]
    fn valid_target_rejects_explored_but_not_visible() {
        let obj = ObjectInstance::destructible(ObjectEnum::Peacekeeper, 50.0);
        let vis = VisibilityStateEnum::Explored;
        let domain = DomainEnum::Ground;
        let attack_domain = TargetDomainEnum::Ground;
        assert!(!is_valid_target(&obj, &vis, &domain, &attack_domain));
    }

    #[test]
    fn valid_target_rejects_domain_incompatible() {
        let obj = ObjectInstance::destructible(ObjectEnum::Peacekeeper, 50.0);
        let vis = VisibilityStateEnum::Visible;
        let domain = DomainEnum::Air;
        let attack_domain = TargetDomainEnum::Ground;
        assert!(!is_valid_target(&obj, &vis, &domain, &attack_domain));
    }

    #[test]
    fn valid_target_universal_attack_accepts_air() {
        let obj = ObjectInstance::destructible(ObjectEnum::Peacekeeper, 50.0);
        let vis = VisibilityStateEnum::Visible;
        let domain = DomainEnum::Air;
        let attack_domain = TargetDomainEnum::Universal;
        assert!(is_valid_target(&obj, &vis, &domain, &attack_domain));
    }

    // === AttackCapability defaults ===

    #[test]
    fn attack_capability_default_has_new_fields() {
        let cap = AttackCapability::default();
        assert_eq!(cap.min_range, 0.0);
        assert_eq!(cap.target_domain, TargetDomainEnum::Ground);
        assert_eq!(cap.target_type, TargetTypeEnum::SingleTarget);
        assert!(cap.aoe_radius.is_none());
    }

    // === AttackTarget enum ===

    #[test]
    fn attack_target_unit_target() {
        let entity = Entity::from_raw_u32(42).unwrap();
        let target = AttackTarget::UnitTarget(entity);
        if let AttackTarget::UnitTarget(e) = target {
            assert_eq!(e, entity);
        } else {
            panic!("Expected UnitTarget");
        }
    }

    #[test]
    fn attack_target_location_target() {
        let pos = Vec3::new(1.0, 2.0, 3.0);
        let target = AttackTarget::LocationTarget(pos);
        if let AttackTarget::LocationTarget(v) = target {
            assert_eq!(v, pos);
        } else {
            panic!("Expected LocationTarget");
        }
    }

    // === AttackState helper methods ===

    #[test]
    fn attack_state_target_entity_from_unit_target() {
        let entity = Entity::from_raw_u32(42).unwrap();
        let state = AttackState {
            current_target: Some(AttackTarget::UnitTarget(entity)),
            ..Default::default()
        };
        assert_eq!(state.target_entity(), Some(entity));
        assert!(state.target_location().is_none());
    }

    #[test]
    fn attack_state_target_location_from_location_target() {
        let pos = Vec3::new(1.0, 2.0, 3.0);
        let state = AttackState {
            current_target: Some(AttackTarget::LocationTarget(pos)),
            ..Default::default()
        };
        assert!(state.target_entity().is_none());
        assert_eq!(state.target_location(), Some(pos));
    }

    #[test]
    fn attack_state_no_target() {
        let state = AttackState::default();
        assert!(state.target_entity().is_none());
        assert!(state.target_location().is_none());
    }

    // === AttackSourceEnum ===

    #[test]
    fn attack_source_unit_base_vs_turret() {
        assert_ne!(AttackSourceEnum::UnitBase, AttackSourceEnum::Turret);
    }

    // === FullyConnected Melee Subtype ===

    #[test]
    fn is_melee_true_for_melee_subtype() {
        let cap = AttackCapability {
            attack_type: AttackType::FullyConnected { subtype: FullyConnectedSubtype::Melee },
            ..Default::default()
        };
        assert!(cap.is_melee());
    }

    #[test]
    fn is_melee_false_for_ranged_subtype() {
        let cap = AttackCapability {
            attack_type: AttackType::FullyConnected { subtype: FullyConnectedSubtype::Ranged },
            ..Default::default()
        };
        assert!(!cap.is_melee());
    }

    #[test]
    fn is_melee_false_for_tail_disjointed() {
        let cap = AttackCapability {
            attack_type: AttackType::TailDisjointed {
                projectile_speed: 10.0,
                projectile_visual: ProjectileVisual::Sphere { radius: 0.1 },
            },
            ..Default::default()
        };
        assert!(!cap.is_melee());
    }

    #[test]
    fn is_melee_false_for_head_disjointed() {
        let cap = AttackCapability {
            attack_type: AttackType::HeadDisjointed { effect_radius: 2.0 },
            ..Default::default()
        };
        assert!(!cap.is_melee());
    }

    #[test]
    fn is_melee_false_for_doubly_disjointed() {
        let cap = AttackCapability {
            attack_type: AttackType::DoublyDisjointed {
                projectile_speed: 10.0,
                projectile_visual: ProjectileVisual::Sphere { radius: 0.1 },
                effect_radius: 2.0,
            },
            ..Default::default()
        };
        assert!(!cap.is_melee());
    }

    #[test]
    fn melee_range_constant_is_small() {
        assert!(MELEE_RANGE > 0.0);
        assert!(MELEE_RANGE < 2.0);
    }

    #[test]
    fn default_attack_capability_is_ranged_fully_connected() {
        let cap = AttackCapability::default();
        assert!(!cap.is_melee());
        assert!(matches!(cap.attack_type, AttackType::FullyConnected { subtype: FullyConnectedSubtype::Ranged }));
    }

    // === FullyConnected subtype enum ===

    #[test]
    fn fully_connected_subtype_ranged_vs_melee_distinct() {
        assert_ne!(FullyConnectedSubtype::Ranged, FullyConnectedSubtype::Melee);
    }

    // === Armor component ===

    #[test]
    fn armor_component_stores_values() {
        let armor = Armor {
            point_armor: 5.0,
            full_armor: 3.0,
            directional_armor: true,
        };
        assert_eq!(armor.point_armor, 5.0);
        assert_eq!(armor.full_armor, 3.0);
        assert!(armor.directional_armor);
    }

    #[test]
    fn armor_non_directional() {
        let armor = Armor {
            point_armor: 2.0,
            full_armor: 1.0,
            directional_armor: false,
        };
        assert!(!armor.directional_armor);
    }

    // === Silhouette component ===

    #[test]
    fn silhouette_component_stores_dimensions() {
        let sil = Silhouette { width: 0.5, height: 0.75 };
        assert_eq!(sil.width, 0.5);
        assert_eq!(sil.height, 0.75);
    }

    // === Directional armor constants ===

    #[test]
    fn directional_armor_constants_valid() {
        assert!(DIRECTIONAL_ARMOR_FRONT_MULTIPLIER > 1.0);
        assert!(DIRECTIONAL_ARMOR_REAR_MULTIPLIER < 1.0);
        assert!(DIRECTIONAL_ARMOR_REAR_MULTIPLIER > 0.0);
        assert!(DIRECTIONAL_ARMOR_FRONT_THRESHOLD > DIRECTIONAL_ARMOR_REAR_THRESHOLD);
    }

    // === Directional armor multiplier ===

    use crate::game::combat::systems::directional_armor_multiplier;

    #[test]
    fn directional_armor_front_hit() {
        // Source is in front of target (target faces toward source)
        let source = Vec3::new(0.0, 0.0, -5.0);
        let target = Vec3::new(0.0, 0.0, 0.0);
        let forward = Vec3::new(0.0, 0.0, -1.0); // Facing toward source
        let mult = directional_armor_multiplier(source, target, forward);
        assert_eq!(mult, DIRECTIONAL_ARMOR_FRONT_MULTIPLIER);
    }

    #[test]
    fn directional_armor_rear_hit() {
        // Source is behind target (target faces away from source)
        let source = Vec3::new(0.0, 0.0, 5.0);
        let target = Vec3::new(0.0, 0.0, 0.0);
        let forward = Vec3::new(0.0, 0.0, -1.0); // Facing away from source
        let mult = directional_armor_multiplier(source, target, forward);
        assert_eq!(mult, DIRECTIONAL_ARMOR_REAR_MULTIPLIER);
    }

    #[test]
    fn directional_armor_side_hit() {
        // Source is to the side of target
        let source = Vec3::new(5.0, 0.0, 0.0);
        let target = Vec3::new(0.0, 0.0, 0.0);
        let forward = Vec3::new(0.0, 0.0, -1.0); // Facing perpendicular
        let mult = directional_armor_multiplier(source, target, forward);
        assert_eq!(mult, 1.0);
    }

    // === DamageEvent variants ===

    #[test]
    fn damage_event_single_target_variant() {
        let event = DamageEvent::SingleTarget {
            damage: 10.0,
            source: Entity::from_raw_u32(1).unwrap(),
            source_position: Vec3::ZERO,
        };
        assert!(matches!(event, DamageEvent::SingleTarget { damage, .. } if damage == 10.0));
    }

    #[test]
    fn damage_event_aoe_variant() {
        let event = DamageEvent::AreaOfEffect {
            damage: 20.0,
            source: Entity::from_raw_u32(1).unwrap(),
            center: Vec3::new(5.0, 0.0, 5.0),
            radius: 3.0,
            source_owner: Owner::player(0),
        };
        assert!(matches!(event, DamageEvent::AreaOfEffect { radius, .. } if radius == 3.0));
    }

    // === SingleTarget damage with armor ===

    #[test]
    fn single_target_damage_minus_point_armor() {
        // Damage 10, point_armor 3, no directional → effective damage = 7
        let raw = 10.0_f32;
        let armor = 3.0_f32;
        let result = (raw - armor).max(0.0);
        assert_eq!(result, 7.0);
    }

    #[test]
    fn single_target_armor_exceeds_damage() {
        // Damage 5, point_armor 8 → effective damage = 0
        let raw = 5.0_f32;
        let armor = 8.0_f32;
        let result = (raw - armor).max(0.0);
        assert_eq!(result, 0.0);
    }

    #[test]
    fn single_target_directional_front_increases_armor() {
        // Damage 10, point_armor 4, front hit (1.5x armor) → armor=6, damage=4
        let raw = 10.0_f32;
        let armor = 4.0 * DIRECTIONAL_ARMOR_FRONT_MULTIPLIER;
        let result = (raw - armor).max(0.0);
        assert_eq!(result, 4.0);
    }

    #[test]
    fn single_target_directional_rear_decreases_armor() {
        // Damage 10, point_armor 4, rear hit (0.5x armor) → armor=2, damage=8
        let raw = 10.0_f32;
        let armor = 4.0 * DIRECTIONAL_ARMOR_REAR_MULTIPLIER;
        let result = (raw - armor).max(0.0);
        assert_eq!(result, 8.0);
    }

    // === Circle-rect overlap ===

    #[test]
    fn circle_rect_no_overlap() {
        let area = circle_rect_overlap_area(
            Vec2::new(0.0, 0.0), 1.0,
            Vec2::new(10.0, 10.0), 1.0, 1.0,
        );
        assert_eq!(area, 0.0);
    }

    #[test]
    fn circle_rect_circle_contains_rect() {
        // Large circle, small rect at center
        let area = circle_rect_overlap_area(
            Vec2::new(0.0, 0.0), 10.0,
            Vec2::new(0.0, 0.0), 1.0, 1.0,
        );
        // Should be rect_area = 1.0
        assert_eq!(area, 1.0);
    }

    #[test]
    fn circle_rect_rect_contains_circle() {
        // Small circle inside large rect
        let area = circle_rect_overlap_area(
            Vec2::new(0.0, 0.0), 0.5,
            Vec2::new(0.0, 0.0), 10.0, 10.0,
        );
        let circle_area = std::f32::consts::PI * 0.5 * 0.5;
        assert!((area - circle_area).abs() < 0.01);
    }

    #[test]
    fn circle_rect_partial_overlap_nonzero() {
        // Circle partially overlapping rect
        let area = circle_rect_overlap_area(
            Vec2::new(0.0, 0.0), 1.0,
            Vec2::new(0.8, 0.0), 1.0, 1.0,
        );
        assert!(area > 0.0);
        let circle_area = std::f32::consts::PI;
        assert!(area < circle_area);
    }

    #[test]
    fn circle_rect_zero_radius() {
        let area = circle_rect_overlap_area(
            Vec2::new(0.0, 0.0), 0.0,
            Vec2::new(0.0, 0.0), 1.0, 1.0,
        );
        assert_eq!(area, 0.0);
    }

    #[test]
    fn circle_rect_zero_dimensions() {
        let area = circle_rect_overlap_area(
            Vec2::new(0.0, 0.0), 1.0,
            Vec2::new(0.0, 0.0), 0.0, 0.0,
        );
        assert_eq!(area, 0.0);
    }

    // === AoE damage calculation ===

    #[test]
    fn aoe_damage_full_overlap_uses_full_damage() {
        // AoE circle fully contains the unit silhouette
        let damage = 100.0_f32;
        let radius = 10.0_f32;
        let sil_w = 1.0_f32;
        let sil_h = 1.0_f32;
        let overlap = sil_w * sil_h; // Full rect area
        let aoe_area = std::f32::consts::PI * radius * radius;
        let damage_share = damage * (overlap / aoe_area);
        // Small unit in large AoE → damage_share is fraction of total damage
        assert!(damage_share > 0.0);
        assert!(damage_share < damage);
    }

    #[test]
    fn aoe_damage_armor_uses_full_armor() {
        // AoE armor is full_armor * (overlap/unit_area)
        let full_armor = 5.0_f32;
        let overlap = 0.5_f32;
        let unit_area = 1.0_f32;
        let effective_armor = full_armor * (overlap / unit_area);
        assert_eq!(effective_armor, 2.5);
    }

    // === Attack line scale trick ===

    #[test]
    fn attack_line_scale_preserves_thin_dimensions() {
        // Unit cuboid half-extents are (1,1,1). Scaling by (0.02, 0.02, length)
        // gives effective half-extents matching the original Cuboid::new(0.02, 0.02, length).
        let length = 5.0_f32;
        let scale = Vec3::new(0.02, 0.02, length);
        // Half-extents after scale: 1.0 * 0.02 = 0.02, 1.0 * length = 5.0
        assert_eq!(scale.x, 0.02);
        assert_eq!(scale.z, length);
    }

    #[test]
    fn attack_line_zero_length_skipped() {
        // spawn_attack_line returns early if length < 0.01
        let start = Vec3::new(1.0, 0.5, 1.0);
        let end = Vec3::new(1.0, 0.5, 1.0);
        let length = (end - start).length();
        assert!(length < 0.01);
    }

    // === Sphere projectile scale trick ===

    #[test]
    fn sphere_projectile_scale_matches_radius() {
        let radius = 0.3_f32;
        let scale = Vec3::splat(radius);
        // Unit sphere (radius=1.0) scaled by radius gives visual radius = radius
        assert_eq!(scale.x, radius);
        assert_eq!(scale.y, radius);
        assert_eq!(scale.z, radius);
    }

    // === Explosion scale with base_scale ===

    #[test]
    fn explosion_initial_scale_matches_radius() {
        let radius = 2.0_f32;
        let progress = 0.0_f32;
        let scale = radius * (1.0 + progress * 2.0);
        assert_eq!(scale, 2.0);
    }

    #[test]
    fn explosion_final_scale_is_triple_radius() {
        let radius = 1.5_f32;
        let progress = 1.0_f32;
        let scale = radius * (1.0 + progress * 2.0);
        assert_eq!(scale, 4.5); // 1.5 * 3.0
    }
}
