//! The editor's tools and their settings ([`EditorTools`]).

use bevy::prelude::*;
use sc_sim::map::MAX_ELEVATION;
use sc_sim::{BuildingOrientation, Faction, ObjectKind, SlotId, Symmetry, TilePresetId};
use serde::{Deserialize, Serialize};

/// What a click on the map does.
#[derive(Copy, Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum Tool {
    /// Select objects and markers; drag to move them, Delete to remove them.
    Select,
    /// Paint the chosen tile preset with the brush. The corner heights stay.
    Paint,
    /// Set, raise or lower the corner heights under the brush (`core/map.md > Elevation`); slopes form around them.
    Elevation,
    /// Flood-fill a region of equal tiles with the chosen preset.
    Fill,
    /// Place the chosen object type.
    Place,
    /// Place the chosen slot's start position.
    Start,
    /// Place the chosen slot's camera start (`core/camera.md > Starting Position`).
    CameraStart,
    /// Edit the chosen slot's Starting Layout of the chosen faction (`core/map_editor.md > Starting Layouts`, slice
    /// M2-I: `super::layouts`): a click on one of its objects selects it and drags it, a click elsewhere places the
    /// chosen object type there.
    Layout,
}

impl Tool {
    pub const ALL: [Tool; 8] = [
        Tool::Select,
        Tool::Paint,
        Tool::Elevation,
        Tool::Fill,
        Tool::Place,
        Tool::Start,
        Tool::CameraStart,
        Tool::Layout,
    ];

    pub fn label(self) -> &'static str {
        match self {
            Tool::Select => "Select",
            Tool::Paint => "Paint",
            Tool::Elevation => "Height",
            Tool::Fill => "Fill",
            Tool::Place => "Place",
            Tool::Start => "Start",
            Tool::CameraStart => "Camera",
            Tool::Layout => "Layout",
        }
    }

    /// Whether the tool works with the square brush, whose footprint follows the cursor.
    pub fn uses_brush(self) -> bool {
        matches!(self, Tool::Paint | Tool::Elevation)
    }
}

/// What the height tool does to each corner under the brush.
#[derive(Copy, Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum ElevationMode {
    /// Set it to the chosen level.
    Set,
    /// One level up, at most once per stroke.
    Raise,
    /// One level down, at most once per stroke.
    Lower,
}

impl ElevationMode {
    pub const ALL: [ElevationMode; 3] = [ElevationMode::Set, ElevationMode::Raise, ElevationMode::Lower];

    pub fn label(self) -> &'static str {
        match self {
            ElevationMode::Set => "Set",
            ElevationMode::Raise => "Raise",
            ElevationMode::Lower => "Lower",
        }
    }

    pub fn next(self) -> Self {
        match self {
            ElevationMode::Set => ElevationMode::Raise,
            ElevationMode::Raise => ElevationMode::Lower,
            ElevationMode::Lower => ElevationMode::Set,
        }
    }
}

/// The label of a symmetry mode (`core/map_editor.md > Symmetry`).
pub fn symmetry_label(symmetry: Symmetry) -> &'static str {
    match symmetry {
        Symmetry::None => "None",
        Symmetry::Rotate180 => "Rotate 180",
        Symmetry::Rotate90 => "Rotate 90",
        Symmetry::MirrorHorizontalAxis => "Mirror top/bottom",
        Symmetry::MirrorVerticalAxis => "Mirror left/right",
        Symmetry::MirrorBothAxes => "Mirror both",
    }
}

/// Brush sizes: the side of the square brush, in tiles.
pub const BRUSH_SIZES: [u32; 4] = [1, 3, 5, 9];

/// The highest elevation (`core/map.md > TilePlacement > Elevation - integer (0 to 16)`).
pub const TOP_ELEVATION: u8 = MAX_ELEVATION;

/// The active tool and every tool's settings.
#[derive(Resource, Clone, Debug, PartialEq)]
pub struct EditorTools {
    pub tool: Tool,
    /// Brush side in tiles, one of [`BRUSH_SIZES`].
    pub brush: u32,
    /// The preset that Paint and Fill apply.
    pub preset: TilePresetId,
    pub elevation_mode: ElevationMode,
    /// The level that [`ElevationMode::Set`] sets, `0..=16`.
    pub elevation: u8,
    /// The object type that Place places.
    pub kind: ObjectKind,
    /// The owner of placed objects: neutral or a player slot. Resources are always placed neutral, and Skirmish
    /// Default maps take neutral objects only (`core/map_editor.md > Map Types`).
    pub owner: Option<SlotId>,
    /// The slot whose markers the marker tools place and whose Starting Layouts the Layout tool edits: an editable
    /// one on a symmetric map.
    pub slot: SlotId,
    /// The faction whose Starting Layout at `slot` the Layout tool edits.
    pub faction: Faction,
    /// The object type the Layout tool places.
    pub layout_kind: ObjectKind,
    /// How the Place and Layout tools turn the structures they place (R, Shift+R, F, Shift+F): other objects are
    /// never turned.
    pub orientation: BuildingOrientation,
}

impl Default for EditorTools {
    fn default() -> Self {
        Self {
            tool: Tool::Select,
            brush: 3,
            preset: TilePresetId::PLANE,
            elevation_mode: ElevationMode::Set,
            elevation: 1,
            kind: ObjectKind::ALL[0],
            owner: None,
            slot: SlotId(1),
            faction: Faction::Gdo,
            layout_kind: ObjectKind::DeploymentCenter,
            orientation: BuildingOrientation::default(),
        }
    }
}

impl EditorTools {
    /// The next brush size up (`step > 0`) or down, staying within [`BRUSH_SIZES`].
    pub fn stepped_brush(&self, step: i32) -> u32 {
        let i = BRUSH_SIZES.iter().position(|&b| b == self.brush).unwrap_or(1) as i32;
        BRUSH_SIZES[(i + step).clamp(0, BRUSH_SIZES.len() as i32 - 1) as usize]
    }

    /// The object type the active tool places, if it places objects: the Place tool's or the Layout tool's.
    pub fn placed_kind(&self) -> Option<ObjectKind> {
        match self.tool {
            Tool::Place => Some(self.kind),
            Tool::Layout => Some(self.layout_kind),
            _ => None,
        }
    }

    /// How the active tool turns what it places: [`EditorTools::orientation`] for a structure, unturned otherwise.
    pub fn placed_orientation(&self) -> BuildingOrientation {
        match self.placed_kind() {
            Some(kind) if super::orientation::turns(kind) => self.orientation,
            _ => BuildingOrientation::default(),
        }
    }

    /// Whether the active tool places a structure, which R, Shift+R, F and Shift+F turn (`crate::input::Placing`).
    pub fn places_structure(&self) -> bool {
        self.placed_kind().is_some_and(super::orientation::turns)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn labels_and_cycles() {
        for s in Symmetry::ALL {
            assert!(!symmetry_label(s).is_empty());
        }
        let mut m = ElevationMode::Set;
        for expected in ElevationMode::ALL.iter().cycle().skip(1).take(3) {
            m = m.next();
            assert_eq!(m, *expected);
        }
    }

    #[test]
    fn brush_steps_stay_within_the_sizes() {
        let mut tools = EditorTools::default();
        assert_eq!(tools.stepped_brush(1), 5);
        assert_eq!(tools.stepped_brush(-1), 1);
        tools.brush = 9;
        assert_eq!(tools.stepped_brush(1), 9);
        tools.brush = 1;
        assert_eq!(tools.stepped_brush(-1), 1);
    }
}
