//! The camera's tunables: the game's view and its panning (`core/camera.md`), and the map editor's zoom, which the
//! design leaves open (`core/map_editor.md`).

/// `core/camera.md > Zoom`: the camera displays exactly 28 GridUnits horizontally (`crate::camera`).
pub const VIEW_WIDTH_GU: f32 = 28.0;

/// How fast the arrow keys and the screen edge pan the view, GridUnits per second (`crate::camera`). Placeholder
/// (not in the design) until Q0002 settles the pan inputs and speed.
pub const PAN_SPEED_GU_PER_SECOND: f32 = 24.0;

/// Screen-edge panning (`core/camera.md > Panning > Arrow keys and screen edge`): a cursor this close to a window
/// edge (logical pixels) pans that way (`crate::input`). Placeholder (not in the design): the design gives no margin.
pub const EDGE_PAN_MARGIN_PX: f32 = 8.0;

/// The map editor's closest zoom: half the game's view (`crate::editor::view`). Placeholder (not in the design).
pub const MIN_ZOOM: f32 = 0.5;

/// Room around the map when the editor's view is zoomed to fit it: 4% (`crate::editor::view`). Placeholder (not in
/// the design).
pub const FIT_MARGIN: f32 = 1.04;

/// How much further out than "fit" the editor's view may zoom: half as far again (`crate::editor::view`). Placeholder
/// (not in the design).
pub const MAX_ZOOM_PAST_FIT: f32 = 1.5;

/// The editor's zoom per wheel line: one line up shows this fraction of the map shown before (`crate::editor`).
/// Placeholder (not in the design).
pub const ZOOM_PER_LINE: f32 = 0.85;
