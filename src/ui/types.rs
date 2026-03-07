#![allow(dead_code)]
use std::collections::VecDeque;
use bevy::prelude::*;
use crate::game::units::types::commands::CommandType;

/// Height of the top resource bar in logical pixels
pub const HUD_TOP_BAR_HEIGHT: f32 = 32.0;

/// Height of the bottom HUD panel in logical pixels
pub const HUD_BOTTOM_PANEL_HEIGHT: f32 = 220.0;

/// Marker component for the dedicated UI camera (renders HUD at full window size)
#[derive(Component)]
pub struct UiCamera;

/// Resource holding the UI camera entity (used for TargetCamera on root UI nodes)
#[derive(Resource)]
pub struct UiCameraEntity(pub Entity);

/// Resource tracking whether the cursor is currently over a UI element.
/// Updated each frame by `update_cursor_over_ui` system.
/// World-click systems should early-return when this is true.
#[derive(Resource, Default)]
pub struct CursorOverUi(pub bool);

/// Marker component for the main HUD panel
#[derive(Component)]
pub struct HudPanel;

/// Marker component for the minimap section
#[derive(Component)]
pub struct MinimapSection;

/// Marker component for the selected units grid section
#[derive(Component)]
pub struct UnitsGridSection;

/// Marker component for minimap tiles
#[derive(Component)]
pub struct MinimapTile {
    pub grid_x: i32,
    pub grid_z: i32,
}

/// Marker component for minimap unit indicators
#[derive(Component)]
pub struct MinimapUnit {
    pub unit_entity: Entity,
}

/// Container for minimap tiles
#[derive(Component)]
pub struct MinimapContainer;

/// Marker for unit icon in selected units grid
#[derive(Component)]
pub struct UnitIcon {
    pub unit_entity: Entity,
}

/// Marker for unit health bar
#[derive(Component)]
pub struct UnitHealthBar {
    pub unit_entity: Entity,
}

/// Marker component for the resource bar at the top of the screen
#[derive(Component)]
pub struct ResourceBar;

/// Identifies which resource field a text element in the resource bar represents.
/// Used to query and update the correct text element for the local player's faction.
#[derive(Component, Clone, Copy, Debug, PartialEq, Eq)]
pub enum ResourceBarField {
    /// Space Crystals — all factions
    Crystals,
    /// Supplies — GDO, Syndicate
    Supplies,
    /// Power (current/total) — GDO only
    Power,
    /// Unit Control (used/cap) — GDO, Cults
    UnitControl,
    /// Tunnel Space (used/available) — Syndicate only
    TunnelSpace,
    /// Alloys — Colonists only
    Alloys,
    /// Essence — Colonists only
    Essence,
    /// Conduits — Colonists only
    Conduits,
    /// Beacon Capacity (used/available) — Colonists only
    BeaconCapacity,
}

/// Marker for structure icon in selected structures display
#[derive(Component)]
pub struct StructureIcon {
    pub structure_entity: Entity,
}

/// Marker for structure health bar
#[derive(Component)]
pub struct StructureHealthBar {
    pub structure_entity: Entity,
}

/// Marker component for the command panel section (right side of HUD)
#[derive(Component)]
pub struct CommandPanelSection;

/// What the cursor is hovering over, updated each frame by `update_cursor_target`
#[derive(Resource, Default, Debug, Clone)]
pub struct CursorTarget {
    pub kind: CursorTargetEnum,
    pub location: Option<Vec3>,
    pub entity: Option<Entity>,
}

/// Classification of what's under the cursor
#[derive(Default, Debug, Clone, PartialEq)]
pub enum CursorTargetEnum {
    #[default]
    None,
    Ground,
    EnemyObject,
    FriendlyObject,
    NeutralObject,
}

/// Transition types for the interface state machine
#[derive(Debug, Clone)]
pub enum InterfaceTransition {
    /// Only changes UI state, no command issued
    StateOnly,
    /// Changes UI state AND issues a command to selected objects
    CommandIssuing,
}

/// The generalized interface state, replacing the flat CommandPanelState + CommandMode.
/// When `Default` and Selection is empty, the panel is hidden.
/// When `Default` and Selection has units, unit commands are shown.
/// `AwaitingTarget` replaces the old CommandMode resource for target-selection modes.
/// `StructureMenu` wraps structure-specific UI states.
#[derive(Resource, Default, Debug, Clone, PartialEq)]
pub enum ObjectInterfaceState {
    #[default]
    Default,
    /// Awaiting a target click for the given command type
    AwaitingTarget(CommandType),
    /// Structure-specific states (preserve existing DC/BK/EF/ST behavior)
    StructureMenu(StructureMenuState),
    /// Agent-specific states (unique worker unit interface)
    AgentMenu(AgentMenuState),
}

impl ObjectInterfaceState {
    /// Whether the interface is in a placement mode (ghost follows mouse)
    pub fn is_placement_mode(&self) -> bool {
        matches!(self,
            ObjectInterfaceState::StructureMenu(
                StructureMenuState::DcAwaitingPlacement |
                StructureMenuState::EfAwaitingPlacement |
                StructureMenuState::TunnelAwaitingPlacement
            ) |
            ObjectInterfaceState::AgentMenu(AgentMenuState::AgentAwaitingPlacement)
        )
    }

    /// Whether the interface is in any non-default command mode (awaiting target selection)
    pub fn is_awaiting_target(&self) -> bool {
        matches!(self, ObjectInterfaceState::AwaitingTarget(_))
    }

    /// Get the awaiting command type, if any
    pub fn awaiting_command_type(&self) -> Option<CommandType> {
        match self {
            ObjectInterfaceState::AwaitingTarget(ct) => Some(*ct),
            _ => None,
        }
    }
}

/// Structure-specific interface states
#[derive(Debug, Clone, PartialEq)]
pub enum StructureMenuState {
    /// DC selected, showing main menu with "Build" button
    DcIdle,
    /// DC selected, build submenu open
    DcBuildMenu,
    /// DC constructing something
    DcConstructing,
    /// DC has structure ready to place
    DcReadyToPlace,
    /// DC awaiting placement click — ghost follows mouse
    DcAwaitingPlacement,
    /// Barracks selected
    BarracksMenu,
    /// ExtractionFacility selected
    EfIdle,
    /// EF constructing
    EfConstructing,
    /// EF has plate ready to place
    EfReadyToPlace,
    /// EF awaiting plate placement click — ghost follows mouse
    EfAwaitingPlacement,
    /// Supply Tower selected
    SupplyTowerMenu,
    /// Tunnel selected — DefaultState: Upgrade, Expand, Eject
    TunnelIdle,
    /// Tunnel selected — ExpandMenu: pick an expansion type
    TunnelExpandMenu,
    /// Tunnel selected — AwaitingPlacement: ghost preview for expansion
    TunnelAwaitingPlacement,
    /// Tunnel selected — EjectMenu: pick unit type to eject
    TunnelEjectMenu,
}


/// Agent-specific interface states
#[derive(Debug, Clone, PartialEq)]
pub enum AgentMenuState {
    /// Agent selected — DefaultState: Build Tunnel, Drop Off Resources
    AgentDefault,
    /// Agent selected — AwaitingPlacement: ghost preview for Tunnel building
    AgentAwaitingPlacement,
}

/// Marker for command panel buttons with their action
#[derive(Component, Clone, Debug)]
pub enum CommandButtonAction {
    /// DC: Open build submenu
    DcOpenBuildMenu,
    /// DC: Build a specific structure type
    DcBuild(crate::types::ObjectEnum),
    /// DC: Cancel construction or placement
    DcCancel,
    /// Barracks: Train a unit
    BkTrain(crate::types::ObjectEnum),
    /// Barracks: Cancel last queued item
    BkCancel,
    /// EF: Build extraction plate
    EfBuildPlate,
    /// EF: Cancel construction or placement
    EfCancel,
    /// Go back to previous menu
    Back,
    /// Enter placement mode (DC or EF)
    EnterPlacement,
    /// Unit: Set command mode to Move
    UnitMove,
    /// Unit: Set command mode to Attack
    UnitAttack,
    /// Unit: Set command mode to Attack Ground
    UnitAttackGround,
    /// Unit: Set command mode to Attack Move
    UnitAttackMove,
    /// Unit: Set command mode to Patrol
    UnitPatrol,
    /// Unit: Hold Position (immediate)
    UnitHoldPosition,
    /// Unit: Stop (immediate)
    UnitStop,
    /// Unit: Set command mode to Reverse
    UnitReverse,
    /// Supply Tower: Train a Supply Chopper
    StTrain(crate::types::ObjectEnum),
    /// Supply Tower: Cancel last queued item
    StCancel,
    /// Supply Tower: Schedule Deliveries (enter awaiting target mode)
    StScheduleDeliveries,
    /// Tunnel: Upgrade to next tier
    TunnelUpgrade,
    /// Tunnel: Open expand menu
    TunnelOpenExpandMenu,
    /// Tunnel: Open eject menu
    TunnelOpenEjectMenu,
    /// Tunnel: Select an expansion type to place (carries the ObjectEnum)
    TunnelSelectExpansion(crate::types::ObjectEnum),
    /// Tunnel: Eject a unit of the given type from Side A
    TunnelEjectUnit(crate::types::ObjectEnum),
    /// Agent: Build Tunnel (enters AwaitingPlacement)
    AgentBuildTunnel,
    /// Agent: Drop Off Resources at own Tunnel
    AgentDropOff,
}

/// Marker component for the placement ghost entity
#[derive(Component)]
pub struct PlacementGhost;

/// Marker component for the build area overlay mesh
#[derive(Component)]
pub struct BuildAreaOverlay;

/// Resource tracking placement mode state
#[derive(Resource, Default)]
pub struct PlacementState {
    /// The type of building being placed
    pub building_type: Option<crate::types::ObjectEnum>,
    /// The source structure entity (DC or EF)
    pub source_entity: Option<Entity>,
    /// Current grid position the ghost is snapped to
    pub grid_pos: Option<(i32, i32)>,
    /// Whether the current position is valid
    pub is_valid: bool,
    /// Current rotation for the building being placed
    pub rotation: crate::types::StructureRotation,
    /// Whether the building is flipped horizontally (mirror E↔W sides)
    pub flip_horizontal: bool,
    /// Whether the building is flipped vertically (mirror N↔S sides)
    pub flip_vertical: bool,
}

/// Grid slot position for command panel buttons (row, col)
/// Used to map hotkeys: Q(0,0) W(0,1) E(0,2) / A(1,0) S(1,1) D(1,2) / Z(2,0) X(2,1) C(2,2)
#[derive(Component, Clone, Copy, Debug)]
pub struct GridSlot {
    pub row: u8,
    pub col: u8,
}

/// Marker component for the 3x3 grid container within the command panel
#[derive(Component)]
pub struct CommandGridContainer;

/// Tracks which structure entity the command panel is showing commands for
#[derive(Resource, Default)]
pub struct CommandPanelTarget {
    pub entity: Option<Entity>,
}

/// Tracks the aggregate capabilities of currently selected units.
/// Updated by `update_command_panel_state` when units are selected.
/// Used to conditionally show/hide commands in the command panel grid.
#[derive(Resource, Default, Debug, Clone, PartialEq)]
pub struct SelectedUnitCapabilities {
    /// At least one selected unit has an AttackCapability component
    pub has_attack: bool,
    /// At least one selected unit's attack type can target ground locations
    pub can_target_ground: bool,
    /// At least one selected unit's base supports reversing
    pub can_reverse: bool,
    /// Whether the selected Agent is carrying resources (for drop-off button grey-out)
    pub agent_carrying: bool,
}

/// Marker component for each portrait in the multi-select panel.
/// Stores the entity this portrait represents, enabling click interactions.
#[derive(Component)]
pub struct SelectionPortrait {
    pub entity: Entity,
}

/// Queue of units waiting to eject from a Tunnel's Side A.
/// Attached as a component to Tunnel entities.
#[derive(Component, Clone, Debug, Default)]
pub struct EjectionQueue {
    /// Units queued to eject from this Tunnel's Side A
    pub queue: VecDeque<Entity>,
    /// Frames since last unit began ejecting (for 8-frame minimum spacing)
    pub cooldown: u32,
}
