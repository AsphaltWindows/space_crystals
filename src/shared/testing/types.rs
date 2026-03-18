use bevy::prelude::Entity;
use crate::types::ObjectEnum;
use crate::ui::types::CommandButtonAction;

/// Snapshot of a faction's resource state (for test queries).
#[derive(Clone, Debug, PartialEq)]
pub struct ResourceSnapshot {
    pub space_crystals: i32,
    pub supplies: i32,
}

/// Filter for counting entities.
#[derive(Clone, Debug, Default)]
pub struct EntityFilter {
    /// Filter by owner player ID (None = any owner)
    pub owner: Option<Option<u8>>,
    /// Filter by object type (None = any type)
    pub object_type: Option<ObjectEnum>,
}

impl EntityFilter {
    /// No filter — match all entities.
    pub fn all() -> Self {
        Self::default()
    }

    /// Filter by owner player ID.
    pub fn with_owner(mut self, player_id: Option<u8>) -> Self {
        self.owner = Some(player_id);
        self
    }

    /// Filter by object type.
    pub fn with_object_type(mut self, object_type: ObjectEnum) -> Self {
        self.object_type = Some(object_type);
        self
    }
}

/// Structure state info returned by `get_structure_state`.
#[derive(Clone, Debug)]
pub struct StructureState {
    /// Construction progress (None if fully built / not under construction)
    pub construction_progress: Option<f32>,
    /// Whether the structure is operational (has StructureInstance but no ConstructionHP)
    pub operational: bool,
}

/// Tunnel network info returned by `get_tunnel_network`.
#[derive(Clone, Debug)]
pub struct TunnelNetworkInfo {
    /// Number of tunnel structures owned by this player
    pub tunnel_count: usize,
    /// Number of units currently inside the tunnel network
    pub units_inside: usize,
}

/// Info about a single command button in the command panel grid.
/// Returned by `TestHarness::get_visible_commands()`.
#[derive(Clone, Debug)]
pub struct CommandSlotInfo {
    /// Grid slot position (row, col) — maps to QWE/ASD/ZXC
    pub slot: (u8, u8),
    /// The action this button triggers
    pub action: CommandButtonAction,
    /// Whether the button is enabled (clickable)
    pub enabled: bool,
    /// Whether this is a "common" command (applies to all selected entities)
    pub is_common: bool,
}

/// Snapshot of info panel data for a selected entity.
/// Returned by `TestHarness::get_info_panel()`.
#[derive(Clone, Debug)]
pub struct InfoPanelSnapshot {
    /// The entity being displayed
    pub entity: Entity,
    /// Object type
    pub object_type: ObjectEnum,
    /// Current and max HP (None for indestructible entities)
    pub hp: Option<(f32, f32)>,
}
