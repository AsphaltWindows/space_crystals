//! The input's tunables: when a press is a drag, a double-click or a double tap, how far off a click may land and
//! still pick or target what it meant, and how many pixels make a wheel line.

super::tunables! {
    /// The input's tunables (module docs; `client/input.json`).
    InputTuning;

    /// A press that moves further than this from where it went down (logical pixels) is a drag, which makes
    /// `InputAction::Select` a box instead of a click (`crate::input`). Placeholder (not in the design): a few pixels,
    /// like desktop drags.
    drag_threshold_px: f32 = 4.0,

    /// Two clicks of the same action within this many seconds (from press to press) are a double-click
    /// (`systems/controls/selection.md > MapClickSelection`). Placeholder (not in the design): the design gives no
    /// time.
    double_click_seconds: f64 = 0.4,

    /// ... and when the second is at most this far from the first (logical pixels). Placeholder (not in the design).
    double_click_distance_px: f32 = 6.0,

    /// Two presses of the same chord within this many seconds are a double tap (`systems/controls/selection.md >
    /// ControlGroups > Default keys`: "press the same number twice quickly"). Placeholder (not in the design): the
    /// design gives no time; a double-click's.
    double_tap_seconds: f64 = 0.4,

    /// `systems/controls/interface.md > AwaitingTarget > Dragging while targeting` (design change 0073): while a
    /// target is awaited, a press of the Select action that moves at most this far (logical pixels) before its release
    /// is a targeting click, at the press point; one that moves further is a drag, which does nothing (0067). The
    /// normal drag threshold ([`InputTuning::drag_threshold_px`]) applies everywhere else. "16 pixels (placeholder,
    /// half a tile)": the one value to tune.
    targeting_click_allowance_px: f32 = 16.0,

    /// A click this close to an object's drawn box (space units) still picks it (`crate::selection`). Placeholder (not
    /// in the design): a few pixels, so small units stay easy to click.
    pick_slop_su: f32 = 4.0,

    /// How close (in tiles) a click must be to a camera start marker in the map editor to grab it. Placeholder (not in
    /// the design): half a tile.
    marker_grab_radius_gu: f32 = 0.5,

    /// Wheels that report pixels instead of lines: this many pixels count as one line (`crate::input`). Placeholder
    /// (not in the design).
    pixels_per_line: f32 = 40.0,
}
