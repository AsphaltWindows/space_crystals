//! View actions: the editor's zoom (`core/camera.md > Zoom` has none in the game; the editor adds its own) and
//! centring the view on things.

use bevy::prelude::*;

use super::actions::Editor;
use super::objects;
use super::state::Selection;
use crate::camera::{viewport_rect, visible_extent_su};
use crate::coords::{drawn_bounds, drawn_position};

/// The closest editor zoom: half the game's view.
pub const MIN_ZOOM: f32 = 0.5;
/// Room around the map when the view is zoomed to fit it.
const FIT_MARGIN: f32 = 1.04;
/// How much further out than "fit" the view may zoom.
const MAX_ZOOM_PAST_FIT: f32 = 1.5;

impl Editor<'_> {
    /// The zoom that shows the whole map, as drawn, in the editor's view.
    pub fn fit_zoom(&self) -> f32 {
        let (_, viewport) = viewport_rect(*self.size, *self.insets);
        let visible = visible_extent_su(viewport, self.size.physical.x, 1.0);
        let map = drawn_bounds(&self.doc.scenario().map).size();
        (map.x / visible.x).max(map.y / visible.y) * FIT_MARGIN
    }

    /// The zoom range: from half the game's view out to a bit past showing the whole map.
    pub fn zoom_range(&self) -> (f32, f32) {
        (MIN_ZOOM, self.fit_zoom().max(1.0) * MAX_ZOOM_PAST_FIT)
    }

    pub(super) fn zoom_by(&mut self, factor: f32, at: Option<Vec2>) {
        if !factor.is_finite() || factor <= 0.0 {
            return;
        }
        let old = self.zoom.0;
        let (min, max) = self.zoom_range();
        let new = (old * factor).clamp(min, max);
        if let Some(ground) = at.filter(|p| p.is_finite()) {
            // The point under the cursor stays under the cursor: where its ground is drawn.
            let p = drawn_position(&self.doc.scenario().map, ground);
            self.view.center = p + (self.view.center - p) * (new / old);
        }
        self.zoom.0 = new;
    }

    pub(super) fn fit_map(&mut self) {
        self.zoom.0 = self.fit_zoom().max(MIN_ZOOM);
        self.view.center = drawn_bounds(&self.doc.scenario().map).center();
    }

    pub(super) fn focus_problem(&mut self, index: usize) {
        let Some(problem) = self.problems.0.get(index).cloned() else {
            return;
        };
        let (center, spawn) = objects::problem_focus(self.doc.scenario(), &problem);
        if let Some(center) = center {
            self.center_on(center);
        }
        if let Some(spawn) = spawn {
            self.selection.0 = Some(Selection::Spawn(spawn));
        }
        self.status.warn(problem.to_string());
    }
}
