use bevy::prelude::*;
use bevy::ecs::system::RunSystemOnce;
use crate::types::*;
use crate::game::combat::types::{AttackPhase, AttackState};
use crate::game::types::{
    Player, GdoPlayerResources, SyndicatePlayerResources,
    objects::{ObjectInstance, StructureInstance},
    structures::ConstructionHP,
};
use crate::game::world::types::{
    SpaceCrystalPatch, Tile, TilePresetEnum, FogOfWarMap,
};
use crate::game::units::types::{
    movement::{Velocity, Path},
    state::{UnitCommand, BaseBehaviorState, InTunnelNetwork},
};
use crate::game::utils::*;
use crate::ui::types::{
    ObjectInterfaceState, CommandButtonAction, CommandButtonEnabled,
    CommandButtonCommon, GridSlot, UnitIcon, StructureIcon, ResourceIcon,
};
use super::types::{ResourceSnapshot, EntityFilter, StructureState, TunnelNetworkInfo, CommandSlotInfo, InfoPanelSnapshot};

/// High-level test harness wrapping a Bevy `App`.
///
/// Provides convenience methods for spawning entities, issuing commands,
/// manipulating game state, and advancing the simulation. Designed for
/// use in both unit tests and integration tests.
pub struct TestHarness<'a> {
    pub app: &'a mut App,
}

impl<'a> TestHarness<'a> {
    /// Create a new TestHarness wrapping the given App.
    pub fn new(app: &'a mut App) -> Self {
        Self { app }
    }

    // =========================================================================
    // Spawning — Units
    // =========================================================================

    /// Spawn a unit at grid coordinates. Returns the spawned entity.
    ///
    /// Supported unit types: `Peacekeeper`, `SyndicateAgent`, `SupplyChopper`.
    /// Panics if the ObjectEnum is not a supported unit type.
    pub fn spawn_unit_at_grid(
        &mut self,
        unit_type: ObjectEnum,
        grid_x: i32,
        grid_z: i32,
        owner: Owner,
    ) -> Entity {
        match unit_type {
            ObjectEnum::Peacekeeper => {
                self.app.world_mut().run_system_once(
                    move |mut commands: Commands,
                          mut meshes: ResMut<Assets<Mesh>>,
                          mut materials: ResMut<Assets<StandardMaterial>>| {
                        spawn_peacekeeper(&mut commands, &mut meshes, &mut materials, grid_x, grid_z, owner)
                    },
                ).unwrap()
            }
            ObjectEnum::SyndicateAgent => {
                self.app.world_mut().run_system_once(
                    move |mut commands: Commands,
                          mut meshes: ResMut<Assets<Mesh>>,
                          mut materials: ResMut<Assets<StandardMaterial>>| {
                        spawn_syndicate_agent(&mut commands, &mut meshes, &mut materials, grid_x, grid_z, owner)
                    },
                ).unwrap()
            }
            ObjectEnum::SupplyChopper => {
                self.app.world_mut().run_system_once(
                    move |mut commands: Commands,
                          mut meshes: ResMut<Assets<Mesh>>,
                          mut materials: ResMut<Assets<StandardMaterial>>| {
                        spawn_supply_chopper(&mut commands, &mut meshes, &mut materials, grid_x, grid_z, owner)
                    },
                ).unwrap()
            }
            _ => panic!("Unsupported unit type for spawn_unit_at_grid: {:?}", unit_type),
        }
    }

    /// Spawn a unit at a world position. Converts world coords to grid coords.
    /// Grid conversion: `grid = (world + 32.0 - 0.5) as i32` (inverse of spawn functions).
    pub fn spawn_unit(
        &mut self,
        unit_type: ObjectEnum,
        world_pos: Vec3,
        owner: Owner,
    ) -> Entity {
        let grid_x = (world_pos.x - 0.5 + 32.0) as i32;
        let grid_z = (world_pos.z - 0.5 + 32.0) as i32;
        self.spawn_unit_at_grid(unit_type, grid_x, grid_z, owner)
    }

    // =========================================================================
    // Spawning — Structures
    // =========================================================================

    /// Spawn a structure at grid coordinates. Returns the spawned entity.
    ///
    /// Supported structure types: `DeploymentCenter`, `PowerPlant`, `Barracks`,
    /// `ExtractionFacility`, `Tunnel`, `SupplyTower`.
    ///
    /// Uses default rotation (R0) and no flipping.
    /// For `Headquarters`, use `spawn_headquarters_at_grid` (requires parent tunnel).
    /// For `ExtractionPlate`, use `spawn_extraction_plate_at_grid` (requires patch entity).
    pub fn spawn_structure_at_grid(
        &mut self,
        structure_type: ObjectEnum,
        grid_x: i32,
        grid_z: i32,
        owner: Owner,
    ) -> Entity {
        match structure_type {
            ObjectEnum::DeploymentCenter => {
                self.app.world_mut().run_system_once(
                    move |mut commands: Commands,
                          mut meshes: ResMut<Assets<Mesh>>,
                          mut materials: ResMut<Assets<StandardMaterial>>| {
                        spawn_deployment_center(&mut commands, &mut meshes, &mut materials, grid_x, grid_z, owner)
                    },
                ).unwrap()
            }
            ObjectEnum::PowerPlant => {
                self.app.world_mut().run_system_once(
                    move |mut commands: Commands,
                          mut meshes: ResMut<Assets<Mesh>>,
                          mut materials: ResMut<Assets<StandardMaterial>>| {
                        spawn_power_plant(&mut commands, &mut meshes, &mut materials, grid_x, grid_z, owner,
                            StructureRotation::R0, false, false)
                    },
                ).unwrap()
            }
            ObjectEnum::Barracks => {
                self.app.world_mut().run_system_once(
                    move |mut commands: Commands,
                          mut meshes: ResMut<Assets<Mesh>>,
                          mut materials: ResMut<Assets<StandardMaterial>>| {
                        spawn_barracks(&mut commands, &mut meshes, &mut materials, grid_x, grid_z, owner,
                            StructureRotation::R0, false, false)
                    },
                ).unwrap()
            }
            ObjectEnum::ExtractionFacility => {
                self.app.world_mut().run_system_once(
                    move |mut commands: Commands,
                          mut meshes: ResMut<Assets<Mesh>>,
                          mut materials: ResMut<Assets<StandardMaterial>>| {
                        spawn_extraction_facility(&mut commands, &mut meshes, &mut materials, grid_x, grid_z, owner,
                            StructureRotation::R0, false, false)
                    },
                ).unwrap()
            }
            ObjectEnum::Tunnel => {
                self.app.world_mut().run_system_once(
                    move |mut commands: Commands,
                          mut meshes: ResMut<Assets<Mesh>>,
                          mut materials: ResMut<Assets<StandardMaterial>>| {
                        spawn_tunnel(&mut commands, &mut meshes, &mut materials, grid_x, grid_z, owner)
                    },
                ).unwrap()
            }
            ObjectEnum::SupplyTower => {
                self.app.world_mut().run_system_once(
                    move |mut commands: Commands,
                          mut meshes: ResMut<Assets<Mesh>>,
                          mut materials: ResMut<Assets<StandardMaterial>>| {
                        spawn_supply_tower(&mut commands, &mut meshes, &mut materials, grid_x, grid_z, owner,
                            StructureRotation::R0, false, false)
                    },
                ).unwrap()
            }
            _ => panic!("Unsupported structure type for spawn_structure_at_grid: {:?}. Use spawn_headquarters_at_grid or spawn_extraction_plate_at_grid for types requiring extra params.", structure_type),
        }
    }

    /// Spawn a Headquarters (underground expansion) at grid coordinates.
    /// Requires the parent tunnel entity.
    pub fn spawn_headquarters_at_grid(
        &mut self,
        grid_x: i32,
        grid_z: i32,
        owner: Owner,
        parent_tunnel: Entity,
    ) -> Entity {
        self.app.world_mut().run_system_once(
            move |mut commands: Commands,
                  mut meshes: ResMut<Assets<Mesh>>,
                  mut materials: ResMut<Assets<StandardMaterial>>| {
                spawn_headquarters(&mut commands, &mut meshes, &mut materials, grid_x, grid_z, owner, parent_tunnel)
            },
        ).unwrap()
    }

    /// Spawn an Extraction Plate on a Space Crystal Patch at grid coordinates.
    /// Requires the attached patch entity.
    pub fn spawn_extraction_plate_at_grid(
        &mut self,
        grid_x: i32,
        grid_z: i32,
        owner: Owner,
        attached_patch: Entity,
    ) -> Entity {
        self.app.world_mut().run_system_once(
            move |mut commands: Commands,
                  mut meshes: ResMut<Assets<Mesh>>,
                  mut materials: ResMut<Assets<StandardMaterial>>| {
                spawn_extraction_plate(&mut commands, &mut meshes, &mut materials, grid_x, grid_z, owner, attached_patch)
            },
        ).unwrap()
    }

    // =========================================================================
    // Spawning — Resources
    // =========================================================================

    /// Spawn a Space Crystal Patch at grid coordinates with the given amount.
    pub fn spawn_resource(
        &mut self,
        grid_x: i32,
        grid_z: i32,
        amount: u32,
    ) -> Entity {
        self.app.world_mut().run_system_once(
            move |mut commands: Commands,
                  mut meshes: ResMut<Assets<Mesh>>,
                  mut materials: ResMut<Assets<StandardMaterial>>| {
                let world_x = (grid_x as f32 - 32.0) + 0.5;
                let world_z = (grid_z as f32 - 32.0) + 0.5;

                let mesh = meshes.add(Cuboid::new(0.6, 0.4, 0.6));
                let material = materials.add(StandardMaterial {
                    base_color: Color::srgb(0.3, 0.8, 1.0),
                    ..default()
                });

                commands.spawn((
                    Mesh3d(mesh),
                    MeshMaterial3d(material),
                    Transform::from_xyz(world_x, 0.2, world_z),
                    SpaceCrystalPatch {
                        remaining_amount: amount,
                        initial_amount: amount,
                        has_plate: false,
                    },
                    GridPosition { x: grid_x, z: grid_z },
                    ObjectInstance::indestructible(ObjectEnum::SpaceCrystalsPatch),
                    Selectable,
                )).id()
            },
        ).unwrap()
    }

    // =========================================================================
    // Selection
    // =========================================================================

    /// Set the current selection to exactly the given entities.
    /// Removes `Selected` from all other entities first.
    pub fn set_selection(&mut self, entities: &[Entity]) {
        let world = self.app.world_mut();

        // Remove Selected from all entities that currently have it
        let currently_selected: Vec<Entity> = world
            .query_filtered::<Entity, With<Selected>>()
            .iter(world)
            .collect();
        for e in currently_selected {
            world.entity_mut(e).remove::<Selected>();
        }

        // Add Selected to the specified entities
        for &e in entities {
            if world.get_entity(e).is_ok() {
                world.entity_mut(e).insert(Selected);
            }
        }
    }

    /// Clear all selection — removes `Selected` from every entity.
    pub fn clear_selection(&mut self) {
        self.set_selection(&[]);
    }

    /// Get all currently selected entities.
    pub fn get_selection(&mut self) -> Vec<Entity> {
        let world = self.app.world_mut();
        world.query_filtered::<Entity, With<Selected>>()
            .iter(world)
            .collect()
    }

    // =========================================================================
    // Unit Commands
    // =========================================================================

    /// Issue a command to a unit by inserting/replacing its `UnitCommand` component.
    pub fn issue_command(&mut self, entity: Entity, command: UnitCommand) {
        self.app.world_mut().entity_mut(entity).insert(command);
    }

    // =========================================================================
    // Game State — Faction Resources
    // =========================================================================

    /// Set the space_crystals field on the GDO player's resources.
    /// Finds the Player entity with GdoPlayerResources and updates it.
    pub fn set_gdo_crystals(&mut self, amount: i32) {
        let world = self.app.world_mut();
        let mut query = world.query::<(&Player, &mut GdoPlayerResources)>();
        for (_player, mut res) in query.iter_mut(world) {
            res.space_crystals = amount;
        }
    }

    /// Set the space_crystals field on the Syndicate player's resources.
    pub fn set_syndicate_crystals(&mut self, amount: i32) {
        let world = self.app.world_mut();
        let mut query = world.query::<(&Player, &mut SyndicatePlayerResources)>();
        for (_player, mut res) in query.iter_mut(world) {
            res.space_crystals = amount;
        }
    }

    /// Get the GDO player's space crystal count. Returns None if no GDO player exists.
    pub fn get_gdo_crystals(&mut self) -> Option<i32> {
        let world = self.app.world_mut();
        let mut query = world.query::<(&Player, &GdoPlayerResources)>();
        query.iter(world).next().map(|(_, res)| res.space_crystals)
    }

    /// Get the Syndicate player's space crystal count. Returns None if no Syndicate player exists.
    pub fn get_syndicate_crystals(&mut self) -> Option<i32> {
        let world = self.app.world_mut();
        let mut query = world.query::<(&Player, &SyndicatePlayerResources)>();
        query.iter(world).next().map(|(_, res)| res.space_crystals)
    }

    // =========================================================================
    // Game State — Simulation Control
    // =========================================================================

    /// Advance the simulation by N frames (calls `app.update()` N times).
    pub fn advance_frames(&mut self, n: usize) {
        for _ in 0..n {
            self.app.update();
        }
    }

    /// Advance the simulation by a single frame.
    pub fn step(&mut self) {
        self.app.update();
    }

    // =========================================================================
    // Game State — Tiles
    // =========================================================================

    /// Set the tile type at the given grid position.
    /// Finds the Tile entity at `(grid_x, grid_z)` and replaces its preset components.
    pub fn set_tile(&mut self, grid_x: i32, grid_z: i32, preset_enum: TilePresetEnum) {
        let world = self.app.world_mut();
        let preset = preset_enum.properties();

        // Find the tile entity at this grid position
        let mut query = world.query_filtered::<(Entity, &GridPosition), With<Tile>>();
        let tile_entity = query.iter(world)
            .find(|(_, gp)| gp.x == grid_x && gp.z == grid_z)
            .map(|(e, _)| e);

        if let Some(entity) = tile_entity {
            world.entity_mut(entity).insert((preset, preset_enum));
        }
    }

    // =========================================================================
    // Game State — Fog of War
    // =========================================================================

    /// Reveal the entire map for the given player_id.
    /// Sets all tiles to `VisibilityStateEnum::Visible`.
    pub fn reveal_map(&mut self, player_id: u8) {
        let world = self.app.world_mut();
        let mut fog_map = world.resource_mut::<FogOfWarMap>();
        fog_map.ensure_player(player_id);
        let width = fog_map.width;
        let height = fog_map.height;
        for z in 0..height as i32 {
            for x in 0..width as i32 {
                fog_map.set(player_id, x, z, VisibilityStateEnum::Visible);
            }
        }
    }

    // =========================================================================
    // Game State — Camera
    // =========================================================================

    /// Set the main camera position and zoom (Y height).
    pub fn set_camera(&mut self, position: Vec3, zoom: f32) {
        let world = self.app.world_mut();
        let mut query = world.query_filtered::<&mut Transform, With<MainCamera>>();
        for mut transform in query.iter_mut(world) {
            transform.translation = Vec3::new(position.x, zoom, position.z);
        }
    }

    // =========================================================================
    // Queries — Entity State
    // =========================================================================

    /// Get an entity's world position (Transform.translation).
    /// Returns None if the entity has no Transform.
    pub fn get_position(&self, entity: Entity) -> Option<Vec3> {
        self.app.world().get::<Transform>(entity).map(|t| t.translation)
    }

    /// Get an entity's current and max HP.
    /// Returns `(current_hp, max_hp)`. Returns None if entity has no ObjectInstance or no HP.
    pub fn get_health(&self, entity: Entity) -> Option<(f32, f32)> {
        self.app.world().get::<ObjectInstance>(entity).and_then(|obj| {
            match (obj.hp, obj.max_hp) {
                (Some(hp), Some(max)) => Some((hp, max)),
                _ => None,
            }
        })
    }

    /// Get an entity's current attack phase.
    /// Returns None if the entity has no AttackState.
    pub fn get_attack_state(&self, entity: Entity) -> Option<AttackPhase> {
        self.app.world().get::<AttackState>(entity).map(|s| s.phase)
    }

    /// Get a clone of an entity's BaseBehaviorState.
    /// Returns None if the entity has no BaseBehaviorState.
    pub fn get_behavior(&self, entity: Entity) -> Option<BaseBehaviorState> {
        self.app.world().get::<BaseBehaviorState>(entity).cloned()
    }

    /// Get a clone of an entity's UnitCommand.
    /// Returns None if the entity has no UnitCommand.
    pub fn get_command(&self, entity: Entity) -> Option<UnitCommand> {
        self.app.world().get::<UnitCommand>(entity).cloned()
    }

    /// Get an entity's velocity and current path target.
    /// Returns `(velocity_vec, current_waypoint_target)`.
    /// Either or both may be None if the components are missing.
    pub fn get_movement(&self, entity: Entity) -> (Option<Vec3>, Option<Vec3>) {
        let world = self.app.world();
        let vel = world.get::<Velocity>(entity).map(|v| v.0);
        let path_target = world.get::<Path>(entity).and_then(|p| {
            p.waypoints.get(p.current_waypoint).copied()
        });
        (vel, path_target)
    }

    /// Check if an entity is alive.
    /// Returns true if the entity exists AND (has no HP field OR HP > 0).
    /// Returns false if the entity is despawned or HP <= 0.
    pub fn is_alive(&self, entity: Entity) -> bool {
        let world = self.app.world();
        match world.get_entity(entity) {
            Err(_) => false,
            Ok(entity_ref) => {
                match entity_ref.get::<ObjectInstance>() {
                    None => true, // No ObjectInstance — considered alive
                    Some(obj) => obj.hp.map_or(true, |hp| hp > 0.0),
                }
            }
        }
    }

    // =========================================================================
    // Queries — World State
    // =========================================================================

    /// Get visibility state for a player at a tile position.
    pub fn get_visibility(&self, player_id: u8, x: i32, z: i32) -> VisibilityStateEnum {
        let fog_map = self.app.world().resource::<FogOfWarMap>();
        fog_map.get(player_id, x, z)
    }

    /// Get a snapshot of a faction's resources.
    /// `player_id`: 0 for GDO, 1 for Syndicate (must match setup_player_resources).
    /// Returns None if no matching player entity is found.
    pub fn get_resources(&mut self, player_id: u8) -> Option<ResourceSnapshot> {
        let world = self.app.world_mut();

        // Try GDO resources
        let mut query_gdo = world.query::<(&Player, &GdoPlayerResources)>();
        for (player, res) in query_gdo.iter(world) {
            if player.player_number == player_id {
                return Some(ResourceSnapshot {
                    space_crystals: res.space_crystals,
                    supplies: res.supplies,
                });
            }
        }

        // Try Syndicate resources
        let mut query_syn = world.query::<(&Player, &SyndicatePlayerResources)>();
        for (player, res) in query_syn.iter(world) {
            if player.player_number == player_id {
                return Some(ResourceSnapshot {
                    space_crystals: res.space_crystals,
                    supplies: res.supplies,
                });
            }
        }

        None
    }

    /// Get all game entities (with ObjectInstance) within a radius of `center`.
    /// Returns entities with Unit or StructureInstance components.
    pub fn get_entities_in_area(&mut self, center: Vec3, radius: f32) -> Vec<Entity> {
        let world = self.app.world_mut();
        let radius_sq = radius * radius;
        let mut result = Vec::new();

        let mut query = world.query_filtered::<(Entity, &Transform), Or<(With<Unit>, With<StructureInstance>)>>();
        for (entity, transform) in query.iter(world) {
            let dist_sq = (transform.translation - center).length_squared();
            if dist_sq <= radius_sq {
                result.push(entity);
            }
        }
        result
    }

    /// Count entities matching a filter.
    /// Filter by owner player ID and/or object type.
    pub fn count_entities(&mut self, filter: &EntityFilter) -> usize {
        let world = self.app.world_mut();
        let mut query = world.query::<(Entity, &ObjectInstance, &Owner)>();
        let mut count = 0;

        for (_, obj, owner) in query.iter(world) {
            // Check owner filter
            if let Some(filter_owner) = filter.owner {
                if owner.0 != filter_owner {
                    continue;
                }
            }
            // Check object type filter
            if let Some(ref filter_type) = filter.object_type {
                if obj.object_type != *filter_type {
                    continue;
                }
            }
            count += 1;
        }
        count
    }

    // =========================================================================
    // Queries — Structural
    // =========================================================================

    /// Get the tunnel network info for a given player.
    /// Counts Tunnel structures and units with InTunnelNetwork marker.
    pub fn get_tunnel_network(&mut self, player_id: u8) -> TunnelNetworkInfo {
        let world = self.app.world_mut();

        // Count tunnels owned by this player
        let mut tunnel_query = world.query::<(&ObjectInstance, &Owner)>();
        let tunnel_count = tunnel_query.iter(world)
            .filter(|(obj, owner)| {
                obj.object_type == ObjectEnum::Tunnel && owner.0 == Some(player_id)
            })
            .count();

        // Count units inside the tunnel network
        let mut unit_query = world.query::<&InTunnelNetwork>();
        let units_inside = unit_query.iter(world)
            .filter(|itn| itn.owner_player == player_id)
            .count();

        TunnelNetworkInfo {
            tunnel_count,
            units_inside,
        }
    }

    /// Get the state of a structure entity.
    /// Returns construction progress and operational status.
    /// Returns None if the entity has no StructureInstance.
    pub fn get_structure_state(&self, entity: Entity) -> Option<StructureState> {
        let world = self.app.world();
        if world.get::<StructureInstance>(entity).is_none() {
            return None;
        }

        let construction_progress = world.get::<ConstructionHP>(entity).map(|c| c.progress);
        let operational = construction_progress.is_none();

        Some(StructureState {
            construction_progress,
            operational,
        })
    }

    // =========================================================================
    // Queries — UI State
    // =========================================================================

    /// Get the current ObjectInterfaceState resource.
    pub fn get_interface_state(&self) -> ObjectInterfaceState {
        self.app.world().resource::<ObjectInterfaceState>().clone()
    }

    /// Get all visible command buttons in the command panel grid.
    /// Returns info about each button's slot, action, enabled state, and common flag.
    /// Note: Call `advance_frames(1)` before this to ensure UI systems have run.
    pub fn get_visible_commands(&mut self) -> Vec<CommandSlotInfo> {
        let world = self.app.world_mut();
        let mut query = world.query::<(&CommandButtonAction, &GridSlot, &CommandButtonEnabled, &CommandButtonCommon)>();
        let mut result: Vec<CommandSlotInfo> = query.iter(world)
            .map(|(action, slot, enabled, is_common)| {
                CommandSlotInfo {
                    slot: (slot.row, slot.col),
                    action: action.clone(),
                    enabled: enabled.0,
                    is_common: is_common.0,
                }
            })
            .collect();
        // Sort by slot position for deterministic ordering
        result.sort_by_key(|info| (info.slot.0, info.slot.1));
        result
    }

    /// Get the currently active selection group.
    /// Returns None if no active group is set.
    pub fn get_active_group(&self) -> Option<SelectionGroup> {
        let selection = self.app.world().resource::<Selection>();
        selection.active_group().cloned()
    }

    /// Get all selection groups.
    pub fn get_selection_groups(&self) -> Vec<SelectionGroup> {
        let selection = self.app.world().resource::<Selection>();
        selection.groups.clone()
    }

    /// Get info panel data for the selected entity.
    /// For single selection: returns the entity's type and HP.
    /// For multi-selection: returns info for the active group's first entity.
    /// Returns None if nothing is selected.
    pub fn get_info_panel(&self) -> Option<InfoPanelSnapshot> {
        let world = self.app.world();
        let selection = world.resource::<Selection>();

        // Get the representative entity: first entity of active group
        let entity = selection.active_group()
            .and_then(|g| g.entities.first().copied())?;

        let obj = world.get::<ObjectInstance>(entity)?;

        let hp = match (obj.hp, obj.max_hp) {
            (Some(hp), Some(max)) => Some((hp, max)),
            _ => None,
        };

        Some(InfoPanelSnapshot {
            entity,
            object_type: obj.object_type,
            hp,
        })
    }

    /// Get selection panel portrait entities and whether they are in the active group.
    /// Returns `(referenced_entity, is_in_active_group)` for each portrait UI element.
    /// Note: Call `advance_frames(1)` before this to ensure UI systems have run.
    pub fn get_selection_panel_portraits(&mut self) -> Vec<(Entity, bool)> {
        let world = self.app.world_mut();

        // Collect all portrait entities and their referenced game entities
        let mut portraits: Vec<Entity> = Vec::new();

        let mut unit_query = world.query::<&UnitIcon>();
        for icon in unit_query.iter(world) {
            portraits.push(icon.unit_entity);
        }

        let mut structure_query = world.query::<&StructureIcon>();
        for icon in structure_query.iter(world) {
            portraits.push(icon.structure_entity);
        }

        let mut resource_query = world.query::<&ResourceIcon>();
        for icon in resource_query.iter(world) {
            portraits.push(icon.resource_entity);
        }

        // Determine which entities are in the active group
        let selection = world.resource::<Selection>();
        let active_entities: Vec<Entity> = selection.active_group()
            .map(|g| g.entities.clone())
            .unwrap_or_default();

        portraits.into_iter()
            .map(|e| (e, active_entities.contains(&e)))
            .collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use bevy::prelude::*;

    // Unit tests for TestHarness go here.
    // Integration tests using TestApp are in tests/scenarios/.

    #[test]
    fn world_to_grid_conversion() {
        // Verify the inverse conversion formula: grid = (world - 0.5 + 32.0) as i32
        // Forward: world = (grid as f32 - 32.0) + 0.5
        // grid=32 → world=0.5, grid=0 → world=-31.5, grid=63 → world=31.5
        assert_eq!((0.5_f32 - 0.5 + 32.0) as i32, 32);
        assert_eq!((-31.5_f32 - 0.5 + 32.0) as i32, 0);
        assert_eq!((31.5_f32 - 0.5 + 32.0) as i32, 63);
    }

    fn minimal_app_with_ui_resources() -> App {
        let mut app = App::new();
        app.insert_resource(ObjectInterfaceState::Default);
        app.insert_resource(Selection::default());
        app
    }

    #[test]
    fn get_interface_state_returns_default() {
        let mut app = minimal_app_with_ui_resources();
        let harness = TestHarness::new(&mut app);
        assert_eq!(harness.get_interface_state(), ObjectInterfaceState::Default);
    }

    #[test]
    fn get_interface_state_returns_awaiting_target() {
        use crate::game::units::types::commands::CommandType;
        let mut app = minimal_app_with_ui_resources();
        app.insert_resource(ObjectInterfaceState::AwaitingTarget(CommandType::Move));
        let harness = TestHarness::new(&mut app);
        assert_eq!(
            harness.get_interface_state(),
            ObjectInterfaceState::AwaitingTarget(CommandType::Move)
        );
    }

    #[test]
    fn get_visible_commands_returns_spawned_buttons() {
        let mut app = minimal_app_with_ui_resources();
        app.world_mut().spawn((
            CommandButtonAction::UnitMove,
            GridSlot { row: 0, col: 0 },
            CommandButtonEnabled(true),
            CommandButtonCommon(true),
        ));
        app.world_mut().spawn((
            CommandButtonAction::UnitAttack,
            GridSlot { row: 0, col: 1 },
            CommandButtonEnabled(true),
            CommandButtonCommon(false),
        ));
        let mut harness = TestHarness::new(&mut app);
        let commands = harness.get_visible_commands();
        assert_eq!(commands.len(), 2);
        assert_eq!(commands[0].slot, (0, 0));
        assert!(commands[0].enabled);
        assert!(commands[0].is_common);
        assert_eq!(commands[1].slot, (0, 1));
        assert!(!commands[1].is_common);
    }

    #[test]
    fn get_visible_commands_empty_when_no_buttons() {
        let mut app = minimal_app_with_ui_resources();
        let mut harness = TestHarness::new(&mut app);
        let commands = harness.get_visible_commands();
        assert!(commands.is_empty());
    }

    #[test]
    fn get_active_group_returns_none_when_empty() {
        let mut app = minimal_app_with_ui_resources();
        let harness = TestHarness::new(&mut app);
        assert!(harness.get_active_group().is_none());
    }

    #[test]
    fn get_active_group_returns_group_when_set() {
        let mut app = minimal_app_with_ui_resources();
        let entity = app.world_mut().spawn_empty().id();
        app.insert_resource(Selection {
            groups: vec![SelectionGroup {
                object_type: ObjectEnum::Peacekeeper,
                entities: vec![entity],
            }],
            active_group_index: Some(0),
        });
        let harness = TestHarness::new(&mut app);
        let group = harness.get_active_group().unwrap();
        assert_eq!(group.object_type, ObjectEnum::Peacekeeper);
        assert_eq!(group.entities.len(), 1);
    }

    #[test]
    fn get_selection_groups_returns_all_groups() {
        let mut app = minimal_app_with_ui_resources();
        let e1 = app.world_mut().spawn_empty().id();
        let e2 = app.world_mut().spawn_empty().id();
        app.insert_resource(Selection {
            groups: vec![
                SelectionGroup { object_type: ObjectEnum::Peacekeeper, entities: vec![e1] },
                SelectionGroup { object_type: ObjectEnum::DeploymentCenter, entities: vec![e2] },
            ],
            active_group_index: Some(0),
        });
        let harness = TestHarness::new(&mut app);
        let groups = harness.get_selection_groups();
        assert_eq!(groups.len(), 2);
    }

    #[test]
    fn get_info_panel_returns_none_when_empty_selection() {
        let mut app = minimal_app_with_ui_resources();
        let harness = TestHarness::new(&mut app);
        assert!(harness.get_info_panel().is_none());
    }

    #[test]
    fn get_info_panel_returns_snapshot_for_selected_entity() {
        let mut app = minimal_app_with_ui_resources();
        let entity = app.world_mut().spawn(
            ObjectInstance::destructible(ObjectEnum::Peacekeeper, 100.0)
        ).id();
        app.insert_resource(Selection {
            groups: vec![SelectionGroup {
                object_type: ObjectEnum::Peacekeeper,
                entities: vec![entity],
            }],
            active_group_index: Some(0),
        });
        let harness = TestHarness::new(&mut app);
        let snapshot = harness.get_info_panel().unwrap();
        assert_eq!(snapshot.entity, entity);
        assert_eq!(snapshot.object_type, ObjectEnum::Peacekeeper);
        assert_eq!(snapshot.hp, Some((100.0, 100.0)));
    }

    #[test]
    fn get_selection_panel_portraits_returns_icons() {
        let mut app = minimal_app_with_ui_resources();
        let unit_entity = app.world_mut().spawn_empty().id();
        let struct_entity = app.world_mut().spawn_empty().id();
        // Spawn portrait UI entities
        app.world_mut().spawn(UnitIcon { unit_entity });
        app.world_mut().spawn(StructureIcon { structure_entity: struct_entity });
        // Set active group to contain only the unit
        app.insert_resource(Selection {
            groups: vec![SelectionGroup {
                object_type: ObjectEnum::Peacekeeper,
                entities: vec![unit_entity],
            }],
            active_group_index: Some(0),
        });
        let mut harness = TestHarness::new(&mut app);
        let portraits = harness.get_selection_panel_portraits();
        assert_eq!(portraits.len(), 2);
        // unit_entity should be in active group
        let unit_portrait = portraits.iter().find(|(e, _)| *e == unit_entity).unwrap();
        assert!(unit_portrait.1); // in active group
        // struct_entity should NOT be in active group
        let struct_portrait = portraits.iter().find(|(e, _)| *e == struct_entity).unwrap();
        assert!(!struct_portrait.1); // not in active group
    }
}
