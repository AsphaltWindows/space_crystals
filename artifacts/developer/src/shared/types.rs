#![allow(dead_code)]
use bevy::prelude::*;

/// Application state machine — controls game flow between menu and gameplay
#[derive(States, Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum AppState {
    #[default]
    Menu,
    InGame,
}

/// Component marking an entity as the main camera
#[derive(Component)]
pub struct MainCamera;

/// Resource identifying which player the local human is controlling
#[derive(Resource)]
pub struct LocalPlayer(pub u8);

/// Resource storing the faction selected by the player in the menu.
/// Inserted before `OnEnter(InGame)` systems run.
/// Automated tests can insert this directly without the menu UI.
#[derive(Resource, Debug, Clone, Copy, PartialEq, Eq)]
pub struct SelectedFaction(pub FactionEnum);

/// Component marking an entity as a unit
#[derive(Component)]
pub struct Unit;

/// Component tracking unit ownership — stores PlayerNumber or None for neutral/unowned
#[derive(Component, Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct Owner(pub Option<u8>);

impl Owner {
    /// Create an owner for a specific player
    pub fn player(id: u8) -> Self {
        Self(Some(id))
    }

    /// Create a neutral/unowned owner
    pub fn neutral() -> Self {
        Self(None)
    }

    /// Check if this entity is neutral/unowned
    pub fn is_neutral(&self) -> bool {
        self.0.is_none()
    }

    /// Get the player number, if owned
    pub fn player_number(&self) -> Option<u8> {
        self.0
    }

    /// Get visual color for this owner
    pub fn color(&self) -> Color {
        match self.0 {
            Some(0) => Color::srgb(0.2, 0.4, 0.8),  // Blue
            Some(1) => Color::srgb(0.8, 0.2, 0.2),  // Red
            Some(2) => Color::srgb(0.2, 0.8, 0.3),  // Green
            Some(3) => Color::srgb(0.8, 0.8, 0.2),  // Yellow
            _ => Color::srgb(0.6, 0.6, 0.6),        // Gray for other players/neutral
        }
    }
}

/// Component storing grid position
#[derive(Component, Debug, Clone, Copy)]
pub struct GridPosition {
    pub x: i32,
    pub z: i32,
}

/// Component marking an entity as selectable
#[derive(Component)]
pub struct Selectable;

/// Component marking an entity as currently selected
#[derive(Component)]
pub struct Selected;

/// Component defining the selection hit area for an entity (axis-aligned half-extents)
/// Used by the selection system for point-in-box hit testing instead of a fixed radius
#[derive(Component, Clone, Debug)]
pub struct SelectionBounds {
    pub half_x: f32,
    pub half_y: f32,
    pub half_z: f32,
}

impl SelectionBounds {
    /// Create selection bounds from mesh half-extents
    pub fn new(half_x: f32, half_y: f32, half_z: f32) -> Self {
        Self { half_x, half_y, half_z }
    }

    /// Create selection bounds from full mesh dimensions (width, height, depth)
    pub fn from_dimensions(width: f32, height: f32, depth: f32) -> Self {
        Self {
            half_x: width / 2.0,
            half_y: height / 2.0,
            half_z: depth / 2.0,
        }
    }

    /// Default small bounds for units and small objects
    pub fn unit() -> Self {
        Self::new(0.5, 0.5, 0.5)
    }
}

/// Resource storing control group assignments (10 groups, keys 1-9 and 0)
#[derive(Resource)]
pub struct ControlGroups {
    pub groups: [Vec<Entity>; 10],
}

impl Default for ControlGroups {
    fn default() -> Self {
        Self {
            groups: Default::default(),
        }
    }
}

/// A group of selected entities sharing the same ObjectEnum type.
/// Groupable objects of the same type share one group; ungroupable objects each get their own.
#[derive(Clone, Debug)]
pub struct SelectionGroup {
    pub object_type: ObjectEnum,
    pub entities: Vec<Entity>,
}

/// Resource tracking the current selection with type-based grouping.
/// Replaces direct iteration over `Selected` markers for UI logic.
/// The `Selected` marker component is kept in sync as a derived marker.
#[derive(Resource, Default, Clone, Debug)]
pub struct Selection {
    pub groups: Vec<SelectionGroup>,
    pub active_group_index: Option<usize>,
}

impl Selection {
    /// Get the currently active selection group, if any
    pub fn active_group(&self) -> Option<&SelectionGroup> {
        self.active_group_index.and_then(|idx| self.groups.get(idx))
    }

    /// Get the ObjectEnum type of the active group, if any
    pub fn active_type(&self) -> Option<ObjectEnum> {
        self.active_group().map(|g| g.object_type)
    }

    /// Total number of selected entities across all groups
    pub fn total_entity_count(&self) -> usize {
        self.groups.iter().map(|g| g.entities.len()).sum()
    }

    /// Check if a specific entity is in any group
    pub fn contains_entity(&self, entity: Entity) -> bool {
        self.groups.iter().any(|g| g.entities.contains(&entity))
    }

    /// Clear all groups and reset active index
    pub fn clear(&mut self) {
        self.groups.clear();
        self.active_group_index = None;
    }

    /// Build selection groups from a list of (entity, object_type, groupable) tuples.
    /// Groupable entities with the same ObjectEnum are combined into one group.
    /// Ungroupable entities each get their own group.
    pub fn build_from_entities(&mut self, entities: &[(Entity, ObjectEnum, bool)]) {
        // Remember the active group type so we can try to preserve it after rebuild
        let old_active_type = self.active_group().map(|g| g.object_type);
        let old_active_index = self.active_group_index;

        self.groups.clear();

        for &(entity, object_type, groupable) in entities {
            if groupable {
                // Find existing group for this type
                if let Some(group) = self.groups.iter_mut().find(|g| g.object_type == object_type) {
                    if !group.entities.contains(&entity) {
                        group.entities.push(entity);
                    }
                } else {
                    self.groups.push(SelectionGroup {
                        object_type,
                        entities: vec![entity],
                    });
                }
            } else {
                // Ungroupable: always gets its own group
                self.groups.push(SelectionGroup {
                    object_type,
                    entities: vec![entity],
                });
            }
        }

        if self.groups.is_empty() {
            self.active_group_index = None;
        } else if let Some(old_type) = old_active_type {
            // Try to preserve active group by matching the old active group's type
            if let Some(pos) = self.groups.iter().position(|g| g.object_type == old_type) {
                self.active_group_index = Some(pos);
            } else if let Some(old_idx) = old_active_index {
                // Old type no longer exists — clamp index to valid range
                self.active_group_index = Some(old_idx.min(self.groups.len() - 1));
            } else {
                self.active_group_index = Some(0);
            }
        } else {
            self.active_group_index = Some(0);
        }
    }

    /// Cycle the active group to the next one (Tab key behavior)
    pub fn cycle_active_group(&mut self) {
        if let Some(idx) = self.active_group_index {
            if !self.groups.is_empty() {
                self.active_group_index = Some((idx + 1) % self.groups.len());
            }
        }
    }

    /// Cycle the active group to the previous one (Shift-Tab key behavior)
    pub fn cycle_active_group_backward(&mut self) {
        if let Some(idx) = self.active_group_index {
            if !self.groups.is_empty() {
                self.active_group_index = Some((idx + self.groups.len() - 1) % self.groups.len());
            }
        }
    }

    /// Remove a specific entity from all groups. Cleans up empty groups.
    /// Returns true if the entity was found and removed.
    pub fn remove_entity(&mut self, entity: Entity) -> bool {
        let mut found = false;
        for group in &mut self.groups {
            if let Some(pos) = group.entities.iter().position(|&e| e == entity) {
                group.entities.remove(pos);
                found = true;
            }
        }
        // Remove empty groups
        self.groups.retain(|g| !g.entities.is_empty());
        // Fix active_group_index
        if self.groups.is_empty() {
            self.active_group_index = None;
        } else if let Some(idx) = self.active_group_index {
            if idx >= self.groups.len() {
                self.active_group_index = Some(self.groups.len() - 1);
            }
        }
        found
    }
}

// === Entity Hierarchy Markers ===

/// Marker component for entities with on-screen visual representation (units, structures, tiles, resources)
#[derive(Component, Clone, Copy, Debug)]
pub struct VisibleEntity;

/// Marker component for abstract/non-visual entities (factions, players)
#[derive(Component, Clone, Copy, Debug)]
pub struct InvisibleEntity;

// === Core Identity Enums ===

/// The four playable factions in Space Crystals
#[derive(Component, Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum FactionEnum {
    GlobalDefenseOrdinance,
    TheSyndicate,
    TheCults,
    Colonists,
}

impl FactionEnum {
    /// Get the display name for this faction
    pub fn name(&self) -> &str {
        match self {
            FactionEnum::GlobalDefenseOrdinance => "Global Defense Ordinance",
            FactionEnum::TheSyndicate => "The Syndicate",
            FactionEnum::TheCults => "The Cults",
            FactionEnum::Colonists => "Colonists",
        }
    }

    /// Get the visual color for this faction
    pub fn color(&self) -> Color {
        match self {
            FactionEnum::GlobalDefenseOrdinance => Color::srgb(0.2, 0.4, 0.8),
            FactionEnum::TheSyndicate => Color::srgb(0.8, 0.2, 0.2),
            FactionEnum::TheCults => Color::srgb(0.5, 0.2, 0.6),
            FactionEnum::Colonists => Color::srgb(0.2, 0.8, 0.3),
        }
    }

    /// Get abbreviated name (for UI)
    pub fn abbrev(&self) -> &str {
        match self {
            FactionEnum::GlobalDefenseOrdinance => "GDO",
            FactionEnum::TheSyndicate => "SYN",
            FactionEnum::TheCults => "CULT",
            FactionEnum::Colonists => "COL",
        }
    }
}

/// All game objects by identity (units, structures, resources)
#[derive(Component, Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum ObjectEnum {
    // GDO Units
    Peacekeeper,
    SupplyChopper,
    // Syndicate Units
    SyndicateAgent,
    SyndicateGuard,
    // GDO Structures
    PowerPlant,
    Barracks,
    DeploymentCenter,
    ExtractionFacility,
    ExtractionPlate,
    SupplyTower,
    // Syndicate Structures
    Tunnel,
    Headquarters,
    // Cults Units
    CultsRecruit,
    CultsSoldier,
    CultsGunner,
    // Cults Structures
    RecruitmentCenter,
    CultsStorage,
    CultsArmory,
    // Neutral Resource Objects
    SpaceCrystalsPatch,
    SupplyDeliveryStation,
}

/// Unit base movement archetypes (identity only; attributes defined separately)
#[derive(Component, Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum UnitBaseEnum {
    LightInfantry,
    HeavyInfantry,
    WheeledVehicle,
    TrackedVehicle,
    DrillUnit,
    HoverVehicle,
    Mech,
    HoverCraft,
    Glider,
}

/// Movement model categories
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum MovementModelEnum {
    TurnRate,
    FixedTurnRadius,
    SpeedTurnRadius,
    Drag,
    Glider,
}

/// FullyConnected range behavior subtypes
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum FullyConnectedSubtype {
    /// Standard ranged attack — uses numeric range, gets elevation modifier
    Ranged,
    /// Close-quarters melee attack — fixed short range, no elevation modifier
    Melee,
}

/// Attack type categories (identity only; attributes defined separately)
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum AttackTypeEnum {
    FullyConnected,
    HeadDisjointed,
    TailDisjointed,
    DoublyDisjointed,
}

impl AttackTypeEnum {
    /// Whether this attack type can miss the target (projectile may not reach)
    pub fn can_miss(&self) -> bool {
        matches!(self, AttackTypeEnum::TailDisjointed | AttackTypeEnum::DoublyDisjointed)
    }

    /// Whether this attack type can target ground locations (not just units)
    pub fn can_target_ground(&self) -> bool {
        matches!(self, AttackTypeEnum::TailDisjointed | AttackTypeEnum::DoublyDisjointed)
    }

    /// Whether this attack type requires a projectile speed value
    pub fn requires_projectile_speed(&self) -> bool {
        matches!(self, AttackTypeEnum::HeadDisjointed | AttackTypeEnum::DoublyDisjointed)
    }

    /// Whether LocationTarget (in addition to UnitTarget) is valid for this attack type
    pub fn allows_location_target(&self) -> bool {
        matches!(self, AttackTypeEnum::TailDisjointed | AttackTypeEnum::DoublyDisjointed)
    }
}

/// Which domain(s) an attack can target
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum TargetDomainEnum {
    Ground,
    Air,
    Universal,
}

/// Whether an attack hits a single target or an area
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum TargetTypeEnum {
    SingleTarget,
    AoE,
}

/// The domain a unit operates in
#[derive(Component, Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum DomainEnum {
    Ground,
    Air,
    Underground,
}

/// Fog-of-war visibility state
#[derive(Component, Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum VisibilityStateEnum {
    Unexplored,
    Explored,
    Visible,
}

/// Component marking an entity as a vision source with a range in grid units.
/// Entities with SightRange provide fog-of-war visibility for their owner.
/// Only added to entities with non-zero sight_range (units, structures with vision).
#[derive(Component, Clone, Copy, Debug)]
pub struct SightRange(pub u32);

/// Structure symmetry types — describes which sides of a structure are identical
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum SymmetryTypeEnum {
    /// All 4 sides identical
    AAAA,
    /// 3 identical sides + 1 different
    AAAB,
    /// 2 pairs of identical sides (adjacent matching)
    AABB,
    /// 2 pairs of identical sides (opposite matching)
    ABAB,
    /// 3 different types with one pair
    AABC,
    /// 3 different types with opposite pair
    ABAC,
    /// 2 unique sides + 1 opposite pair (entrance A, matching long sides B, exit C)
    ABCB,
    /// All 4 sides different
    ABCD,
}

/// Structure rotation (0, 90, 180, 270 degrees)
#[derive(Component, Clone, Copy, Debug, PartialEq, Eq, Hash, Default)]
pub enum StructureRotation {
    #[default]
    R0,
    R90,
    R180,
    R270,
}

impl StructureRotation {
    /// Get the rotation angle in degrees
    pub fn degrees(&self) -> f32 {
        match self {
            StructureRotation::R0 => 0.0,
            StructureRotation::R90 => 90.0,
            StructureRotation::R180 => 180.0,
            StructureRotation::R270 => 270.0,
        }
    }

    /// Get the rotation angle in radians
    pub fn radians(&self) -> f32 {
        self.degrees().to_radians()
    }

    /// Rotate 90 degrees clockwise
    pub fn rotate_cw(&self) -> Self {
        match self {
            StructureRotation::R0 => StructureRotation::R90,
            StructureRotation::R90 => StructureRotation::R180,
            StructureRotation::R180 => StructureRotation::R270,
            StructureRotation::R270 => StructureRotation::R0,
        }
    }

    /// Rotate 90 degrees counter-clockwise
    pub fn rotate_ccw(&self) -> Self {
        match self {
            StructureRotation::R0 => StructureRotation::R270,
            StructureRotation::R90 => StructureRotation::R0,
            StructureRotation::R180 => StructureRotation::R90,
            StructureRotation::R270 => StructureRotation::R180,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // === AppState tests ===

    #[test]
    fn app_state_default_is_menu() {
        let state = AppState::default();
        assert_eq!(state, AppState::Menu);
    }

    #[test]
    fn app_state_has_menu_and_in_game_variants() {
        let menu = AppState::Menu;
        let in_game = AppState::InGame;
        assert_ne!(menu, in_game);
    }

    #[test]
    fn app_state_is_hashable() {
        use std::collections::HashSet;
        let mut set = HashSet::new();
        set.insert(AppState::Menu);
        set.insert(AppState::InGame);
        assert_eq!(set.len(), 2);
    }

    #[test]
    fn app_state_is_copy_and_clone() {
        let state = AppState::InGame;
        let copied = state;
        let cloned = state.clone();
        assert_eq!(copied, cloned);
    }

    // === SelectedFaction tests ===

    #[test]
    fn selected_faction_stores_gdo() {
        let sf = SelectedFaction(FactionEnum::GlobalDefenseOrdinance);
        assert_eq!(sf.0, FactionEnum::GlobalDefenseOrdinance);
    }

    #[test]
    fn selected_faction_stores_syndicate() {
        let sf = SelectedFaction(FactionEnum::TheSyndicate);
        assert_eq!(sf.0, FactionEnum::TheSyndicate);
    }

    #[test]
    fn selected_faction_equality() {
        let sf1 = SelectedFaction(FactionEnum::GlobalDefenseOrdinance);
        let sf2 = SelectedFaction(FactionEnum::GlobalDefenseOrdinance);
        let sf3 = SelectedFaction(FactionEnum::TheSyndicate);
        assert_eq!(sf1, sf2);
        assert_ne!(sf1, sf3);
    }

    #[test]
    fn selected_faction_is_copy() {
        let sf = SelectedFaction(FactionEnum::TheSyndicate);
        let copied = sf;
        assert_eq!(sf, copied);
    }

    #[test]
    fn faction_enum_has_four_variants() {
        let factions = [
            FactionEnum::GlobalDefenseOrdinance,
            FactionEnum::TheSyndicate,
            FactionEnum::TheCults,
            FactionEnum::Colonists,
        ];
        assert_eq!(factions.len(), 4);
    }

    #[test]
    fn faction_enum_display_names() {
        assert_eq!(FactionEnum::GlobalDefenseOrdinance.name(), "Global Defense Ordinance");
        assert_eq!(FactionEnum::TheSyndicate.name(), "The Syndicate");
        assert_eq!(FactionEnum::TheCults.name(), "The Cults");
        assert_eq!(FactionEnum::Colonists.name(), "Colonists");
    }

    #[test]
    fn faction_enum_abbreviations() {
        assert_eq!(FactionEnum::GlobalDefenseOrdinance.abbrev(), "GDO");
        assert_eq!(FactionEnum::TheSyndicate.abbrev(), "SYN");
        assert_eq!(FactionEnum::TheCults.abbrev(), "CULT");
        assert_eq!(FactionEnum::Colonists.abbrev(), "COL");
    }

    #[test]
    fn visible_and_invisible_entity_markers_are_distinct() {
        // VisibleEntity and InvisibleEntity are distinct types
        let _visible = VisibleEntity;
        let _invisible = InvisibleEntity;
        // If this compiles, they are distinct types
    }

    #[test]
    fn owner_neutral_is_unowned() {
        let neutral = Owner::neutral();
        assert!(neutral.is_neutral());
        assert_eq!(neutral.player_number(), None);
    }

    #[test]
    fn owner_player_is_owned() {
        let owned = Owner::player(0);
        assert!(!owned.is_neutral());
        assert_eq!(owned.player_number(), Some(0));
    }

    // === Selection and SelectionGroup tests ===

    #[test]
    fn selection_default_is_empty() {
        let sel = Selection::default();
        assert!(sel.groups.is_empty());
        assert_eq!(sel.active_group_index, None);
        assert_eq!(sel.total_entity_count(), 0);
    }

    #[test]
    fn selection_active_group_returns_none_when_empty() {
        let sel = Selection::default();
        assert!(sel.active_group().is_none());
        assert!(sel.active_type().is_none());
    }

    #[test]
    fn selection_build_from_groupable_same_type() {
        let mut sel = Selection::default();
        let e1 = Entity::from_raw_u32(1).unwrap();
        let e2 = Entity::from_raw_u32(2).unwrap();
        sel.build_from_entities(&[
            (e1, ObjectEnum::Peacekeeper, true),
            (e2, ObjectEnum::Peacekeeper, true),
        ]);
        assert_eq!(sel.groups.len(), 1);
        assert_eq!(sel.groups[0].entities.len(), 2);
        assert_eq!(sel.groups[0].object_type, ObjectEnum::Peacekeeper);
        assert_eq!(sel.active_group_index, Some(0));
    }

    #[test]
    fn selection_build_from_different_groupable_types() {
        let mut sel = Selection::default();
        let e1 = Entity::from_raw_u32(1).unwrap();
        let e2 = Entity::from_raw_u32(2).unwrap();
        sel.build_from_entities(&[
            (e1, ObjectEnum::Peacekeeper, true),
            (e2, ObjectEnum::PowerPlant, true),
        ]);
        assert_eq!(sel.groups.len(), 2);
        assert_eq!(sel.total_entity_count(), 2);
    }

    #[test]
    fn selection_build_from_ungroupable_same_type() {
        let mut sel = Selection::default();
        let e1 = Entity::from_raw_u32(1).unwrap();
        let e2 = Entity::from_raw_u32(2).unwrap();
        // DeploymentCenter is ungroupable
        sel.build_from_entities(&[
            (e1, ObjectEnum::DeploymentCenter, false),
            (e2, ObjectEnum::DeploymentCenter, false),
        ]);
        // Each ungroupable entity gets its own group
        assert_eq!(sel.groups.len(), 2);
        assert_eq!(sel.groups[0].entities.len(), 1);
        assert_eq!(sel.groups[1].entities.len(), 1);
    }

    #[test]
    fn selection_build_from_mixed_groupable_and_ungroupable() {
        let mut sel = Selection::default();
        let e1 = Entity::from_raw_u32(1).unwrap();
        let e2 = Entity::from_raw_u32(2).unwrap();
        let e3 = Entity::from_raw_u32(3).unwrap();
        sel.build_from_entities(&[
            (e1, ObjectEnum::Peacekeeper, true),
            (e2, ObjectEnum::Peacekeeper, true),
            (e3, ObjectEnum::DeploymentCenter, false),
        ]);
        assert_eq!(sel.groups.len(), 2); // 1 grouped peacekeepers + 1 ungroupable DC
        assert_eq!(sel.total_entity_count(), 3);
    }

    #[test]
    fn selection_active_type_returns_first_group_type() {
        let mut sel = Selection::default();
        let e1 = Entity::from_raw_u32(1).unwrap();
        sel.build_from_entities(&[(e1, ObjectEnum::Peacekeeper, true)]);
        assert_eq!(sel.active_type(), Some(ObjectEnum::Peacekeeper));
    }

    #[test]
    fn selection_cycle_active_group() {
        let mut sel = Selection::default();
        let e1 = Entity::from_raw_u32(1).unwrap();
        let e2 = Entity::from_raw_u32(2).unwrap();
        sel.build_from_entities(&[
            (e1, ObjectEnum::Peacekeeper, true),
            (e2, ObjectEnum::PowerPlant, true),
        ]);
        assert_eq!(sel.active_group_index, Some(0));
        sel.cycle_active_group();
        assert_eq!(sel.active_group_index, Some(1));
        sel.cycle_active_group();
        assert_eq!(sel.active_group_index, Some(0)); // wraps around
    }

    #[test]
    fn selection_cycle_active_group_single_group() {
        let mut sel = Selection::default();
        let e1 = Entity::from_raw_u32(1).unwrap();
        sel.build_from_entities(&[(e1, ObjectEnum::Peacekeeper, true)]);
        sel.cycle_active_group();
        assert_eq!(sel.active_group_index, Some(0)); // stays at 0
    }

    #[test]
    fn selection_cycle_active_group_empty_selection() {
        let mut sel = Selection::default();
        sel.cycle_active_group();
        assert_eq!(sel.active_group_index, None); // stays None
    }

    #[test]
    fn selection_cycle_active_group_backward() {
        let mut sel = Selection::default();
        let e1 = Entity::from_raw_u32(1).unwrap();
        let e2 = Entity::from_raw_u32(2).unwrap();
        sel.build_from_entities(&[
            (e1, ObjectEnum::Peacekeeper, true),
            (e2, ObjectEnum::PowerPlant, true),
        ]);
        assert_eq!(sel.active_group_index, Some(0));
        sel.cycle_active_group_backward();
        assert_eq!(sel.active_group_index, Some(1)); // wraps backward from 0 to 1
        sel.cycle_active_group_backward();
        assert_eq!(sel.active_group_index, Some(0)); // wraps back to 0
    }

    #[test]
    fn selection_cycle_active_group_backward_single_group() {
        let mut sel = Selection::default();
        let e1 = Entity::from_raw_u32(1).unwrap();
        sel.build_from_entities(&[(e1, ObjectEnum::Peacekeeper, true)]);
        sel.cycle_active_group_backward();
        assert_eq!(sel.active_group_index, Some(0)); // stays at 0
    }

    #[test]
    fn selection_cycle_active_group_backward_empty_selection() {
        let mut sel = Selection::default();
        sel.cycle_active_group_backward();
        assert_eq!(sel.active_group_index, None); // stays None
    }

    #[test]
    fn selection_contains_entity() {
        let mut sel = Selection::default();
        let e1 = Entity::from_raw_u32(1).unwrap();
        let e2 = Entity::from_raw_u32(2).unwrap();
        sel.build_from_entities(&[(e1, ObjectEnum::Peacekeeper, true)]);
        assert!(sel.contains_entity(e1));
        assert!(!sel.contains_entity(e2));
    }

    #[test]
    fn selection_remove_entity() {
        let mut sel = Selection::default();
        let e1 = Entity::from_raw_u32(1).unwrap();
        let e2 = Entity::from_raw_u32(2).unwrap();
        sel.build_from_entities(&[
            (e1, ObjectEnum::Peacekeeper, true),
            (e2, ObjectEnum::Peacekeeper, true),
        ]);
        assert!(sel.remove_entity(e1));
        assert_eq!(sel.total_entity_count(), 1);
        assert!(!sel.contains_entity(e1));
        assert!(sel.contains_entity(e2));
    }

    #[test]
    fn selection_remove_entity_cleans_empty_groups() {
        let mut sel = Selection::default();
        let e1 = Entity::from_raw_u32(1).unwrap();
        sel.build_from_entities(&[(e1, ObjectEnum::Peacekeeper, true)]);
        sel.remove_entity(e1);
        assert!(sel.groups.is_empty());
        assert_eq!(sel.active_group_index, None);
    }

    #[test]
    fn selection_remove_entity_not_found() {
        let mut sel = Selection::default();
        let e1 = Entity::from_raw_u32(1).unwrap();
        let e2 = Entity::from_raw_u32(2).unwrap();
        sel.build_from_entities(&[(e1, ObjectEnum::Peacekeeper, true)]);
        assert!(!sel.remove_entity(e2));
        assert_eq!(sel.total_entity_count(), 1);
    }

    #[test]
    fn selection_remove_entity_fixes_active_index() {
        let mut sel = Selection::default();
        let e1 = Entity::from_raw_u32(1).unwrap();
        let e2 = Entity::from_raw_u32(2).unwrap();
        sel.build_from_entities(&[
            (e1, ObjectEnum::DeploymentCenter, false),
            (e2, ObjectEnum::ExtractionFacility, false),
        ]);
        // active_group_index = Some(0)
        sel.active_group_index = Some(1);
        // Remove e2 (only entity in group index 1)
        sel.remove_entity(e2);
        assert_eq!(sel.groups.len(), 1);
        // active_group_index should be clamped to 0
        assert_eq!(sel.active_group_index, Some(0));
    }

    #[test]
    fn selection_clear() {
        let mut sel = Selection::default();
        let e1 = Entity::from_raw_u32(1).unwrap();
        sel.build_from_entities(&[(e1, ObjectEnum::Peacekeeper, true)]);
        sel.clear();
        assert!(sel.groups.is_empty());
        assert_eq!(sel.active_group_index, None);
    }

    #[test]
    fn selection_build_deduplicates_entities() {
        let mut sel = Selection::default();
        let e1 = Entity::from_raw_u32(1).unwrap();
        // Same entity appears twice
        sel.build_from_entities(&[
            (e1, ObjectEnum::Peacekeeper, true),
            (e1, ObjectEnum::Peacekeeper, true),
        ]);
        assert_eq!(sel.groups.len(), 1);
        assert_eq!(sel.groups[0].entities.len(), 1); // deduped
    }

    #[test]
    fn agent_is_ungroupable() {
        let obj = ObjectEnum::SyndicateAgent.object_type();
        assert!(!obj.groupable, "Agent must be ungroupable — each Agent is its own SelectionGroup");
    }

    #[test]
    fn multi_agent_selection_creates_separate_groups() {
        let mut sel = Selection::default();
        let e1 = Entity::from_raw_u32(10).unwrap();
        let e2 = Entity::from_raw_u32(11).unwrap();
        let e3 = Entity::from_raw_u32(12).unwrap();
        // Agent is ungroupable (groupable: false)
        sel.build_from_entities(&[
            (e1, ObjectEnum::SyndicateAgent, false),
            (e2, ObjectEnum::SyndicateAgent, false),
            (e3, ObjectEnum::SyndicateAgent, false),
        ]);
        // Each Agent must be in its own SelectionGroup
        assert_eq!(sel.groups.len(), 3);
        assert_eq!(sel.groups[0].entities.len(), 1);
        assert_eq!(sel.groups[1].entities.len(), 1);
        assert_eq!(sel.groups[2].entities.len(), 1);
        // Tab should cycle through all 3
        assert_eq!(sel.active_group_index, Some(0));
        sel.cycle_active_group();
        assert_eq!(sel.active_group_index, Some(1));
        sel.cycle_active_group();
        assert_eq!(sel.active_group_index, Some(2));
        sel.cycle_active_group();
        assert_eq!(sel.active_group_index, Some(0)); // wraps
    }

    #[test]
    fn mixed_agents_and_groupable_units_selection() {
        let mut sel = Selection::default();
        let e1 = Entity::from_raw_u32(20).unwrap();
        let e2 = Entity::from_raw_u32(21).unwrap();
        let e3 = Entity::from_raw_u32(22).unwrap();
        let e4 = Entity::from_raw_u32(23).unwrap();
        sel.build_from_entities(&[
            (e1, ObjectEnum::Peacekeeper, true),
            (e2, ObjectEnum::Peacekeeper, true),
            (e3, ObjectEnum::SyndicateAgent, false),
            (e4, ObjectEnum::SyndicateAgent, false),
        ]);
        // 1 group for 2 Peacekeepers + 2 separate groups for Agents = 3 groups
        assert_eq!(sel.groups.len(), 3);
        assert_eq!(sel.total_entity_count(), 4);
        // First group has both peacekeepers
        assert_eq!(sel.groups[0].entities.len(), 2);
        assert_eq!(sel.groups[0].object_type, ObjectEnum::Peacekeeper);
        // Each Agent in own group
        assert_eq!(sel.groups[1].entities.len(), 1);
        assert_eq!(sel.groups[1].object_type, ObjectEnum::SyndicateAgent);
        assert_eq!(sel.groups[2].entities.len(), 1);
        assert_eq!(sel.groups[2].object_type, ObjectEnum::SyndicateAgent);
    }

    #[test]
    fn build_from_entities_preserves_active_group_on_rebuild() {
        let mut sel = Selection::default();
        let e1 = Entity::from_raw_u32(30).unwrap();
        let e2 = Entity::from_raw_u32(31).unwrap();
        let e3 = Entity::from_raw_u32(32).unwrap();

        // Initial build: two groups
        sel.build_from_entities(&[
            (e1, ObjectEnum::Peacekeeper, true),
            (e2, ObjectEnum::Peacekeeper, true),
            (e3, ObjectEnum::SyndicateAgent, false),
        ]);
        assert_eq!(sel.active_group_index, Some(0));

        // Simulate Tab cycling to group 1 (Agent)
        sel.cycle_active_group();
        assert_eq!(sel.active_group_index, Some(1));
        assert_eq!(sel.active_group().unwrap().object_type, ObjectEnum::SyndicateAgent);

        // Rebuild with same entities (simulates selection_group_sync_system)
        sel.build_from_entities(&[
            (e1, ObjectEnum::Peacekeeper, true),
            (e2, ObjectEnum::Peacekeeper, true),
            (e3, ObjectEnum::SyndicateAgent, false),
        ]);

        // Active group should still be SyndicateAgent at index 1
        assert_eq!(sel.active_group_index, Some(1));
        assert_eq!(sel.active_group().unwrap().object_type, ObjectEnum::SyndicateAgent);
    }

    #[test]
    fn build_from_entities_resets_when_active_type_removed() {
        let mut sel = Selection::default();
        let e1 = Entity::from_raw_u32(40).unwrap();
        let e2 = Entity::from_raw_u32(41).unwrap();

        // Initial build: two groups
        sel.build_from_entities(&[
            (e1, ObjectEnum::Peacekeeper, true),
            (e2, ObjectEnum::SyndicateAgent, false),
        ]);
        sel.cycle_active_group();
        assert_eq!(sel.active_group_index, Some(1));
        assert_eq!(sel.active_group().unwrap().object_type, ObjectEnum::SyndicateAgent);

        // Rebuild with only Peacekeepers (Agent removed from selection)
        sel.build_from_entities(&[
            (e1, ObjectEnum::Peacekeeper, true),
        ]);

        // Active group should fall back — index clamped to valid range
        assert_eq!(sel.active_group_index, Some(0));
        assert_eq!(sel.active_group().unwrap().object_type, ObjectEnum::Peacekeeper);
    }

    #[test]
    fn build_from_entities_fresh_selection_starts_at_zero() {
        let mut sel = Selection::default();
        let e1 = Entity::from_raw_u32(50).unwrap();

        // No prior active group (fresh selection)
        sel.build_from_entities(&[
            (e1, ObjectEnum::Peacekeeper, true),
        ]);
        assert_eq!(sel.active_group_index, Some(0));
    }
}
