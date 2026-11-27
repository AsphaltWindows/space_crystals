//! The camera's tunables: the game's view and its panning (`core/camera.md`), and the map editor's zoom, which the
//! design leaves open (`core/map_editor.md`).

super::tunables! {
    /// The camera's tunables (module docs; `client/camera.json`).
    CameraTuning;

    /// `core/camera.md > Zoom`: the camera displays exactly 28 GridUnits horizontally (`crate::camera`).
    view_width_gu: f32 = 28.0,

    /// How fast the arrow keys and the screen edge pan the view, GridUnits per second (`crate::camera`). Placeholder
    /// (not in the design) until Q0002 settles the pan inputs and speed.
    pan_speed_gu_per_second: f32 = 24.0,

    /// Screen-edge panning (`core/camera.md > Panning > Arrow keys and screen edge`): a cursor this close to a window
    /// edge (logical pixels) pans that way (`crate::input`): "the camera pans while the cursor is within **8 pixels**
    /// of the window's edge (placeholder)" (design change 0413; the design's placeholder). The cursor stays in the
    /// window while it is focused (`crate::platform`), so the edge is always within reach.
    edge_pan_margin_px: f32 = 8.0,

    /// The map editor's closest zoom: half the game's view (`crate::editor::view`). Placeholder (not in the design).
    min_zoom: f32 = 0.5,

    /// Room around the map when the editor's view is zoomed to fit it: 4% (`crate::editor::view`). Placeholder (not in
    /// the design).
    fit_margin: f32 = 1.04,

    /// How much further out than "fit" the editor's view may zoom: half as far again (`crate::editor::view`).
    /// Placeholder (not in the design).
    max_zoom_past_fit: f32 = 1.5,

    /// The editor's zoom per wheel line: one line up shows this fraction of the map shown before (`crate::editor`).
    /// Placeholder (not in the design).
    zoom_per_line: f32 = 0.85,
}
