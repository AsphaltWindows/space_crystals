use bevy::prelude::*;
use crate::types::*;
use crate::simulation::{FRAMES_PER_SECOND, SPACE_UNITS_PER_GRID_UNIT};
use super::types::*;
use super::types::objects::StructureLabel;
use super::types::gdo_structure_stats::*;
use super::types::syndicate_structure_stats::*;
use super::units::types::{
    UnitType, UnitControlCost, RuggedTerrainDefenseBonus, TunnelSpaceCost,
    unit_data::{peacekeeper_type_data, peacekeeper_attack_data, frames_to_seconds,
                agent_type_data, agent_attack_data,
                guard_type_data, guard_attack_data,
                PEACEKEEPER_CONTROL_COST, PEACEKEEPER_RUGGED_BONUS,
                AGENT_CONTROL_COST, AGENT_TUNNEL_SPACE_COST, AGENT_RUGGED_BONUS,
                GUARD_CONTROL_COST, GUARD_TUNNEL_SPACE_COST, GUARD_RUGGED_BONUS},
    movement::{MovementSpeed, RotationSpeed, Velocity, TurnRateMovementParams},
    state::{UnitCommand, CommandQueue, BaseCommandState, BaseBehaviorState,
            LocomotionChannel, OrientationChannel, BaseAttackChannel, AgentCarryState},
};
use super::combat::types::{AttackCapability, AttackState, AttackType, Armor, Silhouette, SeparationRadius, MELEE_RANGE};
use super::types::structures::{
    TunnelExpansionMarker, HeadquartersState, TunnelState, TunnelTier, TunnelArea,
    SupplyTowerState, SupplyChopperState,
};
use super::units::types::movement::DragMovementParams;

/// Spawn a structure name label as a child entity.
/// `height_offset` is the Y position above the parent's origin where the label floats.
fn spawn_structure_label(
    parent: &mut ChildSpawnerCommands,
    name: &str,
    height_offset: f32,
) {
    parent.spawn((
        Text2d::new(name),
        TextFont { font_size: 36.0, ..default() },
        TextColor(Color::WHITE),
        TextLayout::new_with_justify(Justify::Center),
        Transform::from_xyz(0.0, height_offset, 0.0)
            .with_scale(Vec3::splat(0.01)), // Scale down for world-space readability
        StructureLabel,
    ));
}

/// Billboard system: rotates all StructureLabel entities to face the main camera.
/// Labels are children of structure entities, so we use GlobalTransform for world position.
pub fn billboard_label_system(
    camera_query: Query<&GlobalTransform, With<MainCamera>>,
    mut label_query: Query<(&mut Transform, &GlobalTransform), (With<StructureLabel>, Without<MainCamera>)>,
) {
    let Ok(camera_global) = camera_query.single() else { return };
    let camera_pos: Vec3 = camera_global.translation();

    for (mut label_transform, label_global) in label_query.iter_mut() {
        let label_world_pos = label_global.translation();
        let direction = camera_pos - label_world_pos;
        if direction.length_squared() > 0.001 {
            // Face towards camera — use full direction for proper billboard tilt
            let scale = label_transform.scale;
            label_transform.look_to(direction.normalize(), Vec3::Y);
            label_transform.scale = scale;
        }
    }
}

/// Get the side label characters for a given symmetry type.
/// Returns [North(+Z), East(+X), South(-Z), West(-X)] at R0 rotation.
fn side_labels(sym: SymmetryTypeEnum) -> [char; 4] {
    match sym {
        SymmetryTypeEnum::AAAA => ['A', 'A', 'A', 'A'],
        SymmetryTypeEnum::AAAB => ['A', 'A', 'A', 'B'],
        SymmetryTypeEnum::AABB => ['A', 'A', 'B', 'B'],
        SymmetryTypeEnum::ABAB => ['A', 'B', 'A', 'B'],
        SymmetryTypeEnum::AABC => ['A', 'A', 'B', 'C'],
        SymmetryTypeEnum::ABAC => ['A', 'B', 'A', 'C'],
        SymmetryTypeEnum::ABCD => ['A', 'B', 'C', 'D'],
    }
}

/// Spawn side labels (A/B/C/D) on a building's faces as children.
/// Labels are positioned at the midpoint of each face.
/// The building is centered at local (0,0,0) with given half-extents.
/// Since labels are children, they rotate with the parent automatically.
fn spawn_side_labels(
    parent: &mut ChildSpawnerCommands,
    sym: SymmetryTypeEnum,
    half_x: f32,
    half_z: f32,
    height: f32,
) {
    let labels = side_labels(sym);
    // Skip AAAA buildings (all sides identical — labels unnecessary)
    if sym == SymmetryTypeEnum::AAAA {
        return;
    }

    let label_y = height * 0.5; // Mid-height of the building
    let offset = 0.02; // Slight offset from face to avoid z-fighting

    // Side positions in local space: [North(+Z), East(+X), South(-Z), West(-X)]
    let positions = [
        Vec3::new(0.0, label_y, half_z + offset),    // North (+Z)
        Vec3::new(half_x + offset, label_y, 0.0),    // East (+X)
        Vec3::new(0.0, label_y, -(half_z + offset)),  // South (-Z)
        Vec3::new(-(half_x + offset), label_y, 0.0),  // West (-X)
    ];

    // Rotation for each label to face outward from the building
    let rotations = [
        Quat::IDENTITY,                                          // North: face +Z
        Quat::from_rotation_y(-std::f32::consts::FRAC_PI_2),     // East: face +X
        Quat::from_rotation_y(std::f32::consts::PI),             // South: face -Z
        Quat::from_rotation_y(std::f32::consts::FRAC_PI_2),     // West: face -X
    ];

    for i in 0..4 {
        let label_char = labels[i];
        let is_b_side = label_char == 'B';

        let text_color = if is_b_side {
            Color::srgb(1.0, 0.8, 0.2) // Highlighted yellow for B side (unit exit)
        } else {
            Color::srgba(0.9, 0.9, 0.9, 0.8)
        };

        let font_size = if is_b_side { 48.0 } else { 36.0 };

        parent.spawn((
            Text2d::new(label_char.to_string()),
            TextFont { font_size, ..default() },
            TextColor(text_color),
            TextLayout::new_with_justify(Justify::Center),
            Transform::from_translation(positions[i])
                .with_rotation(rotations[i])
                .with_scale(Vec3::splat(0.008)),
            StructureLabel, // Reuse for billboard system
        ));
    }
}

/// Public interface for spawning side labels on ghost preview entities.
pub fn spawn_ghost_side_labels(
    parent: &mut ChildSpawnerCommands,
    sym: SymmetryTypeEnum,
    half_x: f32,
    half_z: f32,
    height: f32,
) {
    spawn_side_labels(parent, sym, half_x, half_z, height);
}

/// Spawn a Deployment Center entity at the given grid position
pub fn spawn_deployment_center(
    commands: &mut Commands,
    meshes: &mut ResMut<Assets<Mesh>>,
    materials: &mut ResMut<Assets<StandardMaterial>>,
    grid_x: i32,
    grid_z: i32,
    owner: Owner,
) -> Entity {
    let world_x = (grid_x as f32 - 32.0) + 2.0; // Center of 4x4
    let world_z = (grid_z as f32 - 32.0) + 2.0;

    let mesh = meshes.add(Cuboid::new(4.0, 1.5, 4.0));
    let material = materials.add(StandardMaterial {
        base_color: owner.color(),
        metallic: 0.6,
        perceptual_roughness: 0.3,
        ..default()
    });

    commands.spawn((
        Mesh3d(mesh),
        MeshMaterial3d(material),
        Transform::from_xyz(world_x, 0.75, world_z),
        ObjectInstance::destructible(ObjectEnum::DeploymentCenter, DC_MAX_HP),
        StructureInstance::default(),
        owner,
        Selectable,
        SelectionBounds::from_dimensions(4.0, 1.5, 4.0),
        GridPosition { x: grid_x, z: grid_z },
        PowerValue(DC_POWER),
        BuildRadiusExtension(DC_BUILD_RADIUS),
        DeploymentCenterState::default(),
        SightRange(ObjectEnum::DeploymentCenter.object_type().sight_range),
    )).with_children(|parent| {
        spawn_structure_label(parent, "Deployment Center", 1.05);
        spawn_side_labels(parent, SymmetryTypeEnum::AAAA, 2.0, 2.0, 1.5);
    }).id()
}

/// Spawn a Power Plant entity at the given grid position
pub fn spawn_power_plant(
    commands: &mut Commands,
    meshes: &mut ResMut<Assets<Mesh>>,
    materials: &mut ResMut<Assets<StandardMaterial>>,
    grid_x: i32,
    grid_z: i32,
    owner: Owner,
    rotation: StructureRotation,
    flip_horizontal: bool,
    flip_vertical: bool,
) -> Entity {
    let (base_x, base_z) = (2u32, 2u32);
    let (rot_x, rot_z) = crate::game::world::utils::rotated_building_size(base_x, base_z, &rotation);
    let world_x = (grid_x as f32 - 32.0) + (rot_x as f32) / 2.0;
    let world_z = (grid_z as f32 - 32.0) + (rot_z as f32) / 2.0;

    let mesh = meshes.add(Cuboid::new(2.0, 1.0, 2.0));
    let is_flipped = flip_horizontal || flip_vertical;
    let material = materials.add(StandardMaterial {
        base_color: Color::srgb(0.8, 0.8, 0.2),
        metallic: 0.4,
        cull_mode: if is_flipped { None } else { Some(bevy::render::render_resource::Face::Back) },
        ..default()
    });

    let flip_scale = Vec3::new(
        if flip_horizontal { -1.0 } else { 1.0 },
        1.0,
        if flip_vertical { -1.0 } else { 1.0 },
    );

    commands.spawn((
        Mesh3d(mesh),
        MeshMaterial3d(material),
        Transform::from_xyz(world_x, 0.5, world_z)
            .with_rotation(Quat::from_rotation_y(rotation.radians()))
            .with_scale(flip_scale),
        ObjectInstance::destructible(ObjectEnum::PowerPlant, PP_MAX_HP),
        StructureInstance { rotation, flip_horizontal, flip_vertical },
        owner,
        Selectable,
        SelectionBounds::from_dimensions(rot_x as f32, 1.0, rot_z as f32),
        GridPosition { x: grid_x, z: grid_z },
        PowerValue(PP_POWER),
        BuildRadiusExtension(PP_BUILD_RADIUS),
        SightRange(ObjectEnum::PowerPlant.object_type().sight_range),
    )).with_children(|parent| {
        spawn_structure_label(parent, "Power Plant", 0.8);
        spawn_side_labels(parent, SymmetryTypeEnum::AAAA, 1.0, 1.0, 1.0);
    }).id()
}

/// Spawn a Barracks entity at the given grid position
pub fn spawn_barracks(
    commands: &mut Commands,
    meshes: &mut ResMut<Assets<Mesh>>,
    materials: &mut ResMut<Assets<StandardMaterial>>,
    grid_x: i32,
    grid_z: i32,
    owner: Owner,
    rotation: StructureRotation,
    flip_horizontal: bool,
    flip_vertical: bool,
) -> Entity {
    let (base_x, base_z) = (3u32, 2u32);
    let (rot_x, rot_z) = crate::game::world::utils::rotated_building_size(base_x, base_z, &rotation);
    let world_x = (grid_x as f32 - 32.0) + (rot_x as f32) / 2.0;
    let world_z = (grid_z as f32 - 32.0) + (rot_z as f32) / 2.0;

    let mesh = meshes.add(Cuboid::new(3.0, 0.8, 2.0));
    let is_flipped = flip_horizontal || flip_vertical;
    let material = materials.add(StandardMaterial {
        base_color: Color::srgb(0.3, 0.6, 0.3),
        metallic: 0.3,
        cull_mode: if is_flipped { None } else { Some(bevy::render::render_resource::Face::Back) },
        ..default()
    });

    let flip_scale = Vec3::new(
        if flip_horizontal { -1.0 } else { 1.0 },
        1.0,
        if flip_vertical { -1.0 } else { 1.0 },
    );

    commands.spawn((
        Mesh3d(mesh),
        MeshMaterial3d(material),
        Transform::from_xyz(world_x, 0.4, world_z)
            .with_rotation(Quat::from_rotation_y(rotation.radians()))
            .with_scale(flip_scale),
        ObjectInstance::destructible(ObjectEnum::Barracks, BK_MAX_HP),
        StructureInstance { rotation, flip_horizontal, flip_vertical },
        owner,
        Selectable,
        SelectionBounds::from_dimensions(rot_x as f32, 0.8, rot_z as f32),
        GridPosition { x: grid_x, z: grid_z },
        PowerValue(BK_POWER),
        BuildRadiusExtension(BK_BUILD_RADIUS),
        BarracksState::default(),
        SightRange(ObjectEnum::Barracks.object_type().sight_range),
    )).with_children(|parent| {
        spawn_structure_label(parent, "Barracks", 0.7);
        spawn_side_labels(parent, SymmetryTypeEnum::ABAC, 1.5, 1.0, 0.8);
    }).id()
}

/// Spawn an Extraction Facility entity at the given grid position
pub fn spawn_extraction_facility(
    commands: &mut Commands,
    meshes: &mut ResMut<Assets<Mesh>>,
    materials: &mut ResMut<Assets<StandardMaterial>>,
    grid_x: i32,
    grid_z: i32,
    owner: Owner,
    rotation: StructureRotation,
    flip_horizontal: bool,
    flip_vertical: bool,
) -> Entity {
    let (base_x, base_z) = (3u32, 3u32);
    let (rot_x, rot_z) = crate::game::world::utils::rotated_building_size(base_x, base_z, &rotation);
    let world_x = (grid_x as f32 - 32.0) + (rot_x as f32) / 2.0;
    let world_z = (grid_z as f32 - 32.0) + (rot_z as f32) / 2.0;

    let mesh = meshes.add(Cuboid::new(3.0, 1.2, 3.0));
    let is_flipped = flip_horizontal || flip_vertical;
    let material = materials.add(StandardMaterial {
        base_color: Color::srgb(0.6, 0.4, 0.2),
        metallic: 0.5,
        perceptual_roughness: 0.4,
        cull_mode: if is_flipped { None } else { Some(bevy::render::render_resource::Face::Back) },
        ..default()
    });

    let flip_scale = Vec3::new(
        if flip_horizontal { -1.0 } else { 1.0 },
        1.0,
        if flip_vertical { -1.0 } else { 1.0 },
    );

    commands.spawn((
        Mesh3d(mesh),
        MeshMaterial3d(material),
        Transform::from_xyz(world_x, 0.6, world_z)
            .with_rotation(Quat::from_rotation_y(rotation.radians()))
            .with_scale(flip_scale),
        ObjectInstance::destructible(ObjectEnum::ExtractionFacility, EF_MAX_HP),
        StructureInstance { rotation, flip_horizontal, flip_vertical },
        owner,
        Selectable,
        SelectionBounds::from_dimensions(rot_x as f32, 1.2, rot_z as f32),
        GridPosition { x: grid_x, z: grid_z },
        PowerValue(EF_POWER),
        BuildRadiusExtension(EF_BUILD_RADIUS),
        ExtractionFacilityState::default(),
        SightRange(ObjectEnum::ExtractionFacility.object_type().sight_range),
    )).with_children(|parent| {
        spawn_structure_label(parent, "Extraction Facility", 0.9);
        spawn_side_labels(parent, SymmetryTypeEnum::AAAA, 1.5, 1.5, 1.2);
    }).id()
}

/// Spawn an Extraction Plate entity on a Space Crystal Patch
pub fn spawn_extraction_plate(
    commands: &mut Commands,
    meshes: &mut ResMut<Assets<Mesh>>,
    materials: &mut ResMut<Assets<StandardMaterial>>,
    grid_x: i32,
    grid_z: i32,
    owner: Owner,
    attached_patch: Entity,
) -> Entity {
    let world_x = (grid_x as f32 - 32.0) + 0.5; // Center of 1x1
    let world_z = (grid_z as f32 - 32.0) + 0.5;

    let mesh = meshes.add(Cuboid::new(0.8, 0.15, 0.8));
    let material = materials.add(StandardMaterial {
        base_color: Color::srgb(0.4, 0.5, 0.7),
        metallic: 0.7,
        perceptual_roughness: 0.2,
        ..default()
    });

    commands.spawn((
        Mesh3d(mesh),
        MeshMaterial3d(material),
        Transform::from_xyz(world_x, 0.1, world_z),
        ObjectInstance::destructible(ObjectEnum::ExtractionPlate, EP_MAX_HP),
        StructureInstance::default(),
        owner,
        Selectable,
        SelectionBounds::from_dimensions(0.8, 0.15, 0.8),
        GridPosition { x: grid_x, z: grid_z },
        BuildRadiusExtension(EP_BUILD_RADIUS),
        ExtractionPlateState {
            attached_patch,
            mining_timer: 0,
        },
    )).with_children(|parent| {
        spawn_structure_label(parent, "Extraction Plate", 0.4);
    }).id()
}

/// Spawn a Peacekeeper unit at the given grid position
pub fn spawn_peacekeeper(
    commands: &mut Commands,
    meshes: &mut ResMut<Assets<Mesh>>,
    materials: &mut ResMut<Assets<StandardMaterial>>,
    grid_x: i32,
    grid_z: i32,
    owner: Owner,
) -> Entity {
    let world_x = (grid_x as f32 - 32.0) + 0.5;
    let world_z = (grid_z as f32 - 32.0) + 0.5;

    let type_data = peacekeeper_type_data();
    let attack_data = peacekeeper_attack_data();

    let mesh = meshes.add(Capsule3d::new(0.2, 0.6));
    let material = materials.add(StandardMaterial {
        base_color: owner.color(),
        ..default()
    });

    // Convert design-spec frame-based attack timings to seconds for existing combat system
    let attack_capability = AttackCapability {
        damage: attack_data.damage as f32,
        range: attack_data.range as f32,
        min_range: attack_data.min_range as f32,
        aim_time: frames_to_seconds(attack_data.aim_duration),
        fire_time: frames_to_seconds(attack_data.firing_duration),
        cooldown_time: frames_to_seconds(attack_data.cooldown_duration),
        reload_time: frames_to_seconds(attack_data.reload_duration),
        attack_type: AttackType::FullyConnected {
            subtype: attack_data.fc_subtype.unwrap_or(crate::types::FullyConnectedSubtype::Ranged),
        },
        target_domain: attack_data.target_domain,
        target_type: attack_data.target_type,
        aoe_radius: attack_data.aoe_radius.map(|r| r as f32),
    };

    // LightInfantry: MaxSpeed 4 SU/frame * 16 FPS / 64 SU/GU = 1.0 GU/sec
    // TurnRate 180 deg/frame = effectively instant → high rotation speed
    let move_speed = 4.0 * (FRAMES_PER_SECOND as f32) / (SPACE_UNITS_PER_GRID_UNIT as f32);
    let rot_speed = 10.0; // Very high for instant-turn infantry

    // TurnRate movement params for Peacekeeper (LightInfantry):
    // TurnRate: 180 deg/frame * 16 FPS = 2880 deg/sec (effectively instant)
    // Acceleration/Deceleration: infinite (instant speed changes)
    let turn_rate_params = TurnRateMovementParams {
        turn_rate: 180.0_f32.to_radians() * (FRAMES_PER_SECOND as f32), // 2880 deg/sec in radians
        acceleration: f32::MAX,
        deceleration: f32::MAX,
        max_speed: move_speed,
    };

    commands.spawn((
        Mesh3d(mesh),
        MeshMaterial3d(material),
        Transform::from_xyz(world_x, 0.5, world_z),
        Unit,
        ObjectInstance::destructible(ObjectEnum::Peacekeeper, type_data.max_hp as f32),
        owner,
        UnitType { name: "Peacekeeper".to_string() },
        Selectable,
        SelectionBounds::unit(),
        GridPosition { x: grid_x, z: grid_z },
        type_data.unit_base,
        MovementSpeed(move_speed),
        RotationSpeed(rot_speed),
        Velocity(Vec3::ZERO),
    )).insert((
        attack_capability,
        AttackState::default(),
        UnitCommand::Idle,
        turn_rate_params,
        type_data.unit_base.data().domain,
        UnitControlCost(PEACEKEEPER_CONTROL_COST),
        RuggedTerrainDefenseBonus(PEACEKEEPER_RUGGED_BONUS),
        CommandQueue::new(),
        BaseCommandState::default(),
        BaseBehaviorState::default(),
        LocomotionChannel::default(),
        OrientationChannel::default(),
        BaseAttackChannel::default(),
        // Note: LightInfantry has_turret=false, so no turret channels
    )).insert((
        Armor {
            point_armor: type_data.point_armor as f32,
            full_armor: type_data.full_armor as f32,
            directional_armor: type_data.unit_base.data().directional_armor,
        },
        Silhouette {
            width: type_data.silhouette_width as f32 / SPACE_UNITS_PER_GRID_UNIT as f32,
            height: type_data.silhouette_height as f32 / SPACE_UNITS_PER_GRID_UNIT as f32,
        },
        SightRange(ObjectEnum::Peacekeeper.object_type().sight_range),
    )).id()
}

/// Spawn a Syndicate Agent unit at the given grid position
pub fn spawn_syndicate_agent(
    commands: &mut Commands,
    meshes: &mut ResMut<Assets<Mesh>>,
    materials: &mut ResMut<Assets<StandardMaterial>>,
    grid_x: i32,
    grid_z: i32,
    owner: Owner,
) -> Entity {
    let world_x = (grid_x as f32 - 32.0) + 0.5;
    let world_z = (grid_z as f32 - 32.0) + 0.5;

    let type_data = agent_type_data();
    let attack_data = agent_attack_data();

    // Slightly larger capsule than Peacekeeper to reflect 36x36 vs 24x24 silhouette
    let mesh = meshes.add(Capsule3d::new(0.28, 0.8));
    let material = materials.add(StandardMaterial {
        base_color: owner.color(),
        ..default()
    });

    // Convert design-spec frame-based attack timings to seconds
    let attack_capability = AttackCapability {
        damage: attack_data.damage as f32,
        range: MELEE_RANGE, // Melee subtype uses fixed melee range constant
        min_range: attack_data.min_range as f32,
        aim_time: frames_to_seconds(attack_data.aim_duration),
        fire_time: frames_to_seconds(attack_data.firing_duration),
        cooldown_time: frames_to_seconds(attack_data.cooldown_duration),
        reload_time: frames_to_seconds(attack_data.reload_duration),
        attack_type: AttackType::FullyConnected {
            subtype: attack_data.fc_subtype.unwrap_or(FullyConnectedSubtype::Melee),
        },
        target_domain: attack_data.target_domain,
        target_type: attack_data.target_type,
        aoe_radius: attack_data.aoe_radius.map(|r| r as f32),
    };

    // HeavyInfantry: MaxSpeed 6 SU/frame * 16 FPS / 64 SU/GU = 1.5 GU/sec
    // TurnRate 180 deg/frame * 16 FPS = 2880 deg/sec (effectively instant)
    let move_speed = 6.0 * (FRAMES_PER_SECOND as f32) / (SPACE_UNITS_PER_GRID_UNIT as f32);
    let rot_speed = 10.0; // Very high for instant-turn infantry

    let turn_rate_params = TurnRateMovementParams {
        turn_rate: 180.0_f32.to_radians() * (FRAMES_PER_SECOND as f32), // 2880 deg/sec in radians
        acceleration: f32::MAX,
        deceleration: f32::MAX,
        max_speed: move_speed,
    };

    commands.spawn((
        Mesh3d(mesh),
        MeshMaterial3d(material),
        Transform::from_xyz(world_x, 0.5, world_z),
        Unit,
        ObjectInstance::destructible(ObjectEnum::SyndicateAgent, type_data.max_hp as f32),
        owner,
        UnitType { name: "Agent".to_string() },
        Selectable,
        SelectionBounds::unit(),
        GridPosition { x: grid_x, z: grid_z },
        type_data.unit_base,
        MovementSpeed(move_speed),
        RotationSpeed(rot_speed),
        Velocity(Vec3::ZERO),
    )).insert((
        attack_capability,
        AttackState::default(),
        UnitCommand::Idle,
        turn_rate_params,
        type_data.unit_base.data().domain,
        UnitControlCost(AGENT_CONTROL_COST),
        RuggedTerrainDefenseBonus(AGENT_RUGGED_BONUS),
        TunnelSpaceCost(AGENT_TUNNEL_SPACE_COST),
        CommandQueue::new(),
        BaseCommandState::default(),
        BaseBehaviorState::default(),
        LocomotionChannel::default(),
        OrientationChannel::default(),
        BaseAttackChannel::default(),
        AgentCarryState::default(),
        // Note: HeavyInfantry has_turret=false, so no turret channels
    )).insert((
        Armor {
            point_armor: type_data.point_armor as f32,
            full_armor: type_data.full_armor as f32,
            directional_armor: type_data.unit_base.data().directional_armor,
        },
        Silhouette {
            width: type_data.silhouette_width as f32 / SPACE_UNITS_PER_GRID_UNIT as f32,
            height: type_data.silhouette_height as f32 / SPACE_UNITS_PER_GRID_UNIT as f32,
        },
        SightRange(ObjectEnum::SyndicateAgent.object_type().sight_range),
    )).id()
}

/// Spawn a Syndicate Guard unit at the given grid position
pub fn spawn_syndicate_guard(
    commands: &mut Commands,
    meshes: &mut ResMut<Assets<Mesh>>,
    materials: &mut ResMut<Assets<StandardMaterial>>,
    grid_x: i32,
    grid_z: i32,
    owner: Owner,
) -> Entity {
    let world_x = (grid_x as f32 - 32.0) + 0.5;
    let world_z = (grid_z as f32 - 32.0) + 0.5;

    let type_data = guard_type_data();
    let attack_data = guard_attack_data();

    // Same capsule size as Agent (both have 36x36 silhouette)
    let mesh = meshes.add(Capsule3d::new(0.28, 0.8));
    let material = materials.add(StandardMaterial {
        base_color: owner.color(),
        ..default()
    });

    // Convert design-spec frame-based attack timings to seconds
    let attack_capability = AttackCapability {
        damage: attack_data.damage as f32,
        range: attack_data.range as f32, // Ranged: 3 GU (NOT MELEE_RANGE)
        min_range: attack_data.min_range as f32,
        aim_time: frames_to_seconds(attack_data.aim_duration),
        fire_time: frames_to_seconds(attack_data.firing_duration),
        cooldown_time: frames_to_seconds(attack_data.cooldown_duration),
        reload_time: frames_to_seconds(attack_data.reload_duration),
        attack_type: AttackType::FullyConnected {
            subtype: attack_data.fc_subtype.unwrap_or(FullyConnectedSubtype::Ranged),
        },
        target_domain: attack_data.target_domain,
        target_type: attack_data.target_type,
        aoe_radius: attack_data.aoe_radius.map(|r| r as f32),
    };

    // HeavyInfantry: MaxSpeed 5 SU/frame * 16 FPS / 64 SU/GU = 1.25 GU/sec
    // TurnRate 180 deg/frame * 16 FPS = 2880 deg/sec (effectively instant)
    let move_speed = 5.0 * (FRAMES_PER_SECOND as f32) / (SPACE_UNITS_PER_GRID_UNIT as f32);
    let rot_speed = 10.0; // Very high for instant-turn infantry

    let turn_rate_params = TurnRateMovementParams {
        turn_rate: 180.0_f32.to_radians() * (FRAMES_PER_SECOND as f32), // 2880 deg/sec in radians
        acceleration: f32::MAX,
        deceleration: f32::MAX,
        max_speed: move_speed,
    };

    commands.spawn((
        Mesh3d(mesh),
        MeshMaterial3d(material),
        Transform::from_xyz(world_x, 0.5, world_z),
        Unit,
        ObjectInstance::destructible(ObjectEnum::SyndicateGuard, type_data.max_hp as f32),
        owner,
        UnitType { name: "Guard".to_string() },
        Selectable,
        SelectionBounds::unit(),
        GridPosition { x: grid_x, z: grid_z },
        type_data.unit_base,
        MovementSpeed(move_speed),
        RotationSpeed(rot_speed),
        Velocity(Vec3::ZERO),
    )).insert((
        attack_capability,
        AttackState::default(),
        UnitCommand::Idle,
        turn_rate_params,
        type_data.unit_base.data().domain,
        UnitControlCost(GUARD_CONTROL_COST),
        RuggedTerrainDefenseBonus(GUARD_RUGGED_BONUS),
        TunnelSpaceCost(GUARD_TUNNEL_SPACE_COST),
        CommandQueue::new(),
        BaseCommandState::default(),
        BaseBehaviorState::default(),
        LocomotionChannel::default(),
        OrientationChannel::default(),
        BaseAttackChannel::default(),
        // Note: Guard is pure combat — NO AgentCarryState
        // Note: HeavyInfantry has_turret=false, so no turret channels
    )).insert((
        Armor {
            point_armor: type_data.point_armor as f32,
            full_armor: type_data.full_armor as f32,
            directional_armor: type_data.unit_base.data().directional_armor,
        },
        Silhouette {
            width: type_data.silhouette_width as f32 / SPACE_UNITS_PER_GRID_UNIT as f32,
            height: type_data.silhouette_height as f32 / SPACE_UNITS_PER_GRID_UNIT as f32,
        },
        SightRange(ObjectEnum::SyndicateGuard.object_type().sight_range),
    )).id()
}

/// Spawn a Tunnel structure at the given grid position (Tier 1 by default).
/// The Tunnel is a surface structure with a TunnelState, TunnelArea, and visual mesh.
pub fn spawn_tunnel(
    commands: &mut Commands,
    meshes: &mut ResMut<Assets<Mesh>>,
    materials: &mut ResMut<Assets<StandardMaterial>>,
    grid_x: i32,
    grid_z: i32,
    owner: Owner,
) -> Entity {
    let world_x = (grid_x as f32 - 32.0) + 2.0; // Center of 4x4
    let world_z = (grid_z as f32 - 32.0) + 2.0;

    let mesh = meshes.add(Cuboid::new(4.0, 1.0, 4.0));
    let material = materials.add(StandardMaterial {
        base_color: Color::srgb(0.5, 0.2, 0.2),
        metallic: 0.5,
        perceptual_roughness: 0.4,
        ..default()
    });

    let tier = TunnelTier::Tier1;

    commands.spawn((
        Mesh3d(mesh),
        MeshMaterial3d(material),
        Transform::from_xyz(world_x, 0.5, world_z),
        ObjectInstance::destructible(ObjectEnum::Tunnel, tier.max_hp()),
        StructureInstance::default(),
        owner,
        Selectable,
        SelectionBounds::from_dimensions(4.0, 1.0, 4.0),
        GridPosition { x: grid_x, z: grid_z },
        TunnelState::default_tier1(),
        TunnelArea::new(grid_x, grid_z, &tier),
        SightRange(TUNNEL_SIGHT_RANGE),
        crate::ui::types::EjectionQueue::default(),
    )).with_children(|parent| {
        spawn_structure_label(parent, "Tunnel", 0.8);
        spawn_side_labels(parent, SymmetryTypeEnum::ABCD, 2.0, 2.0, 1.0);
    }).id()
}

/// Spawn a partially-built Tunnel (under construction) at the given grid position.
/// Same as `spawn_tunnel()` but starts at 10% HP with a `ConstructionHP` component.
/// The `construction_hp_tick_system` handles HP scaling automatically.
pub fn spawn_tunnel_under_construction(
    commands: &mut Commands,
    meshes: &mut ResMut<Assets<Mesh>>,
    materials: &mut ResMut<Assets<StandardMaterial>>,
    grid_x: i32,
    grid_z: i32,
    owner: Owner,
    build_frames: u32,
) -> Entity {
    let world_x = (grid_x as f32 - 32.0) + 2.0; // Center of 4x4
    let world_z = (grid_z as f32 - 32.0) + 2.0;

    let mesh = meshes.add(Cuboid::new(4.0, 1.0, 4.0));
    let material = materials.add(StandardMaterial {
        base_color: Color::srgb(0.5, 0.2, 0.2),
        metallic: 0.5,
        perceptual_roughness: 0.4,
        ..default()
    });

    let tier = TunnelTier::Tier1;

    commands.spawn((
        Mesh3d(mesh),
        MeshMaterial3d(material),
        Transform::from_xyz(world_x, 0.5, world_z),
        ObjectInstance::under_construction(ObjectEnum::Tunnel, tier.max_hp()),
        ConstructionHP::new(build_frames),
        StructureInstance::default(),
        owner,
        Selectable,
        SelectionBounds::from_dimensions(4.0, 1.0, 4.0),
        GridPosition { x: grid_x, z: grid_z },
        TunnelState::default_tier1(),
        TunnelArea::new(grid_x, grid_z, &tier),
        SightRange(TUNNEL_SIGHT_RANGE),
        crate::ui::types::EjectionQueue::default(),
    )).with_children(|parent| {
        spawn_structure_label(parent, "Tunnel", 0.8);
        spawn_side_labels(parent, SymmetryTypeEnum::ABCD, 2.0, 2.0, 1.0);
    }).id()
}

/// Spawn a Headquarters (underground expansion) at the given grid position.
/// Rendered as a surface marker so it is visible and selectable.
pub fn spawn_headquarters(
    commands: &mut Commands,
    meshes: &mut ResMut<Assets<Mesh>>,
    materials: &mut ResMut<Assets<StandardMaterial>>,
    grid_x: i32,
    grid_z: i32,
    owner: Owner,
    parent_tunnel: Entity,
) -> Entity {
    let world_x = (grid_x as f32 - 32.0) + 1.0; // Center of 2x2
    let world_z = (grid_z as f32 - 32.0) + 1.0;

    let mesh = meshes.add(Cuboid::new(2.0, 1.0, 2.0));
    let material = materials.add(StandardMaterial {
        base_color: Color::srgb(0.7, 0.2, 0.6),
        metallic: 0.5,
        perceptual_roughness: 0.3,
        ..default()
    });

    commands.spawn((
        Mesh3d(mesh),
        MeshMaterial3d(material),
        Transform::from_xyz(world_x, 0.5, world_z),
        ObjectInstance::destructible(ObjectEnum::Headquarters, HQ_MAX_HP),
        StructureInstance::default(),
        owner,
        Selectable,
        SelectionBounds::from_dimensions(2.0, 1.0, 2.0),
        GridPosition { x: grid_x, z: grid_z },
        DomainEnum::Underground,
        TunnelExpansionMarker { parent_tunnel },
        HeadquartersState::default(),
        SightRange(3),
    )).with_children(|parent| {
        spawn_structure_label(parent, "Headquarters", 0.8);
    }).id()
}

/// Spawn a Supply Tower entity at the given grid position
pub fn spawn_supply_tower(
    commands: &mut Commands,
    meshes: &mut ResMut<Assets<Mesh>>,
    materials: &mut ResMut<Assets<StandardMaterial>>,
    grid_x: i32,
    grid_z: i32,
    owner: Owner,
    rotation: StructureRotation,
    flip_horizontal: bool,
    flip_vertical: bool,
) -> Entity {
    let (base_x, base_z) = (3u32, 3u32);
    let (rot_x, rot_z) = crate::game::world::utils::rotated_building_size(base_x, base_z, &rotation);
    let world_x = (grid_x as f32 - 32.0) + (rot_x as f32) / 2.0;
    let world_z = (grid_z as f32 - 32.0) + (rot_z as f32) / 2.0;

    let mesh = meshes.add(Cuboid::new(3.0, 1.2, 3.0));
    let is_flipped = flip_horizontal || flip_vertical;
    let material = materials.add(StandardMaterial {
        base_color: Color::srgb(0.5, 0.5, 0.8),
        metallic: 0.5,
        perceptual_roughness: 0.3,
        cull_mode: if is_flipped { None } else { Some(bevy::render::render_resource::Face::Back) },
        ..default()
    });

    let flip_scale = Vec3::new(
        if flip_horizontal { -1.0 } else { 1.0 },
        1.0,
        if flip_vertical { -1.0 } else { 1.0 },
    );

    commands.spawn((
        Mesh3d(mesh),
        MeshMaterial3d(material),
        Transform::from_xyz(world_x, 0.6, world_z)
            .with_rotation(Quat::from_rotation_y(rotation.radians()))
            .with_scale(flip_scale),
        ObjectInstance::destructible(ObjectEnum::SupplyTower, ST_MAX_HP),
        StructureInstance { rotation, flip_horizontal, flip_vertical },
        owner,
        Selectable,
        SelectionBounds::from_dimensions(rot_x as f32, 1.2, rot_z as f32),
        GridPosition { x: grid_x, z: grid_z },
        PowerValue(ST_POWER),
        BuildRadiusExtension(ST_BUILD_RADIUS),
        SupplyTowerState::default(),
        SightRange(ObjectEnum::SupplyTower.object_type().sight_range),
    )).with_children(|parent| {
        spawn_structure_label(parent, "Supply Tower", 0.9);
        spawn_side_labels(parent, SymmetryTypeEnum::AAAA, 1.5, 1.5, 1.2);
    }).id()
}

/// Spawn a Supply Chopper unit at the given grid position
pub fn spawn_supply_chopper(
    commands: &mut Commands,
    meshes: &mut ResMut<Assets<Mesh>>,
    materials: &mut ResMut<Assets<StandardMaterial>>,
    grid_x: i32,
    grid_z: i32,
    owner: Owner,
) -> Entity {
    let world_x = (grid_x as f32 - 32.0) + 0.5;
    let world_z = (grid_z as f32 - 32.0) + 0.5;

    // Supply Chopper uses a small box mesh (unarmed utility unit)
    let mesh = meshes.add(Cuboid::new(0.6, 0.3, 0.6));
    let material = materials.add(StandardMaterial {
        base_color: owner.color(),
        ..default()
    });

    // Drag movement params from ticket spec:
    // ForwardAccel 0.9 su/f^2, OmniAccel 0.1 su/f^2, DragRatio 0.1/f, TurnRate 10 deg/f
    let drag_params = DragMovementParams {
        forward_acceleration: 0.9 * (FRAMES_PER_SECOND as f32) * (FRAMES_PER_SECOND as f32)
            / (SPACE_UNITS_PER_GRID_UNIT as f32),
        non_forward_acceleration: 0.1 * (FRAMES_PER_SECOND as f32) * (FRAMES_PER_SECOND as f32)
            / (SPACE_UNITS_PER_GRID_UNIT as f32),
        drag_ratio: 0.1 * (FRAMES_PER_SECOND as f32),
        turn_rate: (10.0_f32).to_radians() * (FRAMES_PER_SECOND as f32),
    };

    let max_speed = drag_params.max_speed();

    commands.spawn((
        Mesh3d(mesh),
        MeshMaterial3d(material),
        Transform::from_xyz(world_x, 1.5, world_z), // Hover above ground
        Unit,
        ObjectInstance::destructible(ObjectEnum::SupplyChopper, SC_MAX_HP),
        owner,
        UnitType { name: "Supply Chopper".to_string() },
        Selectable,
        SelectionBounds::unit(),
        GridPosition { x: grid_x, z: grid_z },
        UnitBaseEnum::HoverCraft,
        MovementSpeed(max_speed),
        RotationSpeed(10.0),
        Velocity(Vec3::ZERO),
        UnitCommand::Idle,
    )).insert((
        drag_params,
        DomainEnum::Air,
        SupplyChopperState::default(),
        CommandQueue::new(),
        BaseCommandState::default(),
        BaseBehaviorState::default(),
        LocomotionChannel::default(),
        OrientationChannel::default(),
        // NO AttackCapability, AttackState, or attack channels — unarmed unit
    )).insert((
        Armor {
            point_armor: SC_POINT_ARMOR as f32,
            full_armor: SC_FULL_ARMOR as f32,
            directional_armor: false, // HoverCraft has no directional armor
        },
        Silhouette {
            width: 60.0 / SPACE_UNITS_PER_GRID_UNIT as f32,
            height: 60.0 / SPACE_UNITS_PER_GRID_UNIT as f32,
        },
        SeparationRadius(1.25),
        SightRange(ObjectEnum::SupplyChopper.object_type().sight_range),
    )).id()
}
