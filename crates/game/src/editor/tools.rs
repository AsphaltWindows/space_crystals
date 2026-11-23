//! The editor's tools and their settings ([`EditorTools`]).

use bevy::prelude::*;
use sc_sim::map::MAX_ELEVATION;
use sc_sim::{ObjectKind, PlayerId, Symmetry, TilePresetId};
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
    /// Place the chosen slot's start location (a Q0004 placeholder).
    StartLocation,
    /// Place the chosen slot's camera start (`core/camera.md > Starting Position`).
    CameraStart,
}

impl Tool {
    pub const ALL: [Tool; 7] = [
        Tool::Select,
        Tool::Paint,
        Tool::Elevation,
        Tool::Fill,
        Tool::Place,
        Tool::StartLocation,
        Tool::CameraStart,
    ];

    pub fn label(self) -> &'static str {
        match self {
            Tool::Select => "Select",
            Tool::Paint => "Paint",
            Tool::Elevation => "Height",
            Tool::Fill => "Fill",
            Tool::Place => "Place",
            Tool::StartLocation => "Start",
            Tool::CameraStart => "Camera",
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

/// The label of a symmetry mode (`core/map_editor.md > Symmetry`). Live mirroring repeats terrain strokes and
/// object placements at their images ([`sc_sim::symmetry`]); generated symmetric halves are a later task.
pub fn symmetry_label(symmetry: Symmetry) -> &'static str {
    match symmetry {
        Symmetry::None => "None",
        Symmetry::Rotate180 => "Rotate 180°",
        Symmetry::Rotate90 => "Rotate 90°",
        Symmetry::MirrorHorizontalAxis => "Mirror top/bottom",
        Symmetry::MirrorVerticalAxis => "Mirror left/right",
        Symmetry::MirrorBothAxes => "Mirror both",
    }
}

/// The symmetry after `symmetry` in [`Symmetry::ALL`], skipping those that don't fit a `width × height` map (90°
/// rotation needs a square map).
pub fn next_symmetry(symmetry: Symmetry, width: u32, height: u32) -> Symmetry {
    let all = Symmetry::ALL;
    let start = all.iter().position(|s| *s == symmetry).unwrap_or(0);
    (1..=all.len())
        .map(|i| all[(start + i) % all.len()])
        .find(|s| s.fits(width, height))
        .unwrap_or(Symmetry::None)
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
    /// The owner of placed objects: neutral or a player slot. Resources are always placed neutral.
    pub owner: Option<PlayerId>,
    pub symmetry: Symmetry,
    /// The player slot whose markers the marker tools place, and whom play-tests are played as.
    pub slot: PlayerId,
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
            symmetry: Symmetry::None,
            slot: PlayerId(1),
        }
    }
}

impl EditorTools {
    /// The next brush size up (`step > 0`) or down, staying within [`BRUSH_SIZES`].
    pub fn stepped_brush(&self, step: i32) -> u32 {
        let i = BRUSH_SIZES.iter().position(|&b| b == self.brush).unwrap_or(1) as i32;
        BRUSH_SIZES[(i + step).clamp(0, BRUSH_SIZES.len() as i32 - 1) as usize]
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn cycles_visit_every_value() {
        let mut s = Symmetry::None;
        for expected in Symmetry::ALL.iter().cycle().skip(1).take(6) {
            s = next_symmetry(s, 8, 8);
            assert_eq!(s, *expected);
        }
        // 90° rotation only on square maps.
        assert_eq!(next_symmetry(Symmetry::Rotate180, 8, 6), Symmetry::MirrorHorizontalAxis);
        assert_eq!(next_symmetry(Symmetry::Rotate180, 8, 8), Symmetry::Rotate90);
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
