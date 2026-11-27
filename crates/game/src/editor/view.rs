//! View actions: the editor's zoom (`core/camera.md > Zoom` has none in the game; the editor adds its own) and
//! centring the view on things.

use bevy::prelude::*;

use super::actions::Editor;
use super::layouts::problems::problem_lines;
use super::objects;
use super::state::Selection;
use crate::camera::{viewport_rect, visible_extent_su};
use crate::coords::{drawn_bounds, drawn_position};

impl Editor<'_> {
    /// The zoom that shows the whole map, as drawn, in the editor's view, with `CameraTuning::fit_margin` round it.
    pub fn fit_zoom(&self) -> f32 {
        let (_, viewport) = viewport_rect(*self.size, *self.insets);
        let visible = visible_extent_su(viewport, self.size.physical.x, 1.0);
        let map = drawn_bounds(&self.doc.map().terrain).size();
        (map.x / visible.x).max(map.y / visible.y) * self.tuning.camera.fit_margin
    }

    /// The zoom range: from half the game's view out to a bit past showing the whole map (`CameraTuning::min_zoom`
    /// and `max_zoom_past_fit`).
    pub fn zoom_range(&self) -> (f32, f32) {
        let camera = &self.tuning.camera;
        (camera.min_zoom, self.fit_zoom().max(1.0) * camera.max_zoom_past_fit)
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
            let p = drawn_position(&self.doc.map().terrain, ground);
            self.view.center = p + (self.view.center - p) * (new / old);
        }
        self.zoom.0 = new;
    }

    pub(super) fn fit_map(&mut self) {
        self.zoom.0 = self.fit_zoom().max(self.tuning.camera.min_zoom);
        self.view.center = drawn_bounds(&self.doc.map().terrain).center();
    }

    pub(super) fn focus_problem(&mut self, index: usize) {
        let Some(problem) = self.problems.0.get(index).cloned() else {
            return;
        };
        let (center, spawn) = objects::problem_focus(self.doc.map(), &problem);
        if let Some(center) = center {
            self.center_on(center);
        }
        if let Some(spawn) = spawn {
            self.selection.0 = Some(Selection::Spawn(spawn));
        }
        self.focus_layout_problem(&problem);
        // The problem's line in the Problems list, which may sum up several (`super::layouts::problems`).
        let map = self.doc.map();
        let line = problem_lines(map, &self.problems.0)
            .into_iter()
            .find(|l| l.index == index)
            .or_else(|| problem_lines(map, std::slice::from_ref(&problem)).into_iter().next());
        self.status.warn(line.map_or_else(|| problem.to_string(), |l| l.text));
    }
}
