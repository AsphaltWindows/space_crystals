//! Document actions: files, map properties, player slots and closing the editor.

use std::path::PathBuf;

use bevy::app::AppExit;
use bevy::math::Vec2;
use sc_sim::scenario::MAX_SLOTS;
use sc_sim::{MapDef, SlotId, Symmetry, TilePos};

use super::actions::{Editor, EditorAction};
use super::files::{self, MAX_MAP_SIDE, MIN_MAP_SIDE};
use crate::camera::slot_center;
use crate::coords::drawn_position;

/// The longest map description the editor takes (placeholder).
pub const MAX_DESCRIPTION_CHARS: usize = 400;

impl Editor<'_> {
    pub(super) fn save(&mut self) {
        self.finish_gesture();
        let map = self.doc.map();
        let path = match self.doc.path() {
            Some(path) => path.to_path_buf(),
            None => files::path_for_new_map(&self.files.dir, &map.name),
        };
        if let Err(error) = files::save(&path, map) {
            self.status.error(format!("Save failed: {error}"));
            return;
        }
        let problems = self.playtest.problems(map).len();
        let shown = files::display_path(&path);
        self.doc.mark_saved(path);
        self.files.refresh();
        if problems == 0 {
            self.status.info(format!("Saved {shown}"));
        } else {
            self.status.warn(format!(
                "Saved {shown} with {problems} problem(s): the game won't start it until they are fixed"
            ));
        }
    }

    /// Whether `action` may discard the unsaved changes: there are none, or it is asked for a second time.
    pub(super) fn confirmed(&mut self, action: EditorAction) -> bool {
        if !self.doc.is_dirty() || self.confirm.0.as_ref() == Some(&action) {
            if self.confirm.0.is_some() {
                self.confirm.0 = None;
            }
            return true;
        }
        let again = match action {
            EditorAction::Quit => "close the window again",
            _ => "do that again",
        };
        self.confirm.0 = Some(action);
        self.status.warn(format!(
            "Unsaved changes: {again} to discard them, or save first (Ctrl+S)"
        ));
        false
    }

    pub(super) fn open(&mut self, path: PathBuf) {
        if !self.confirmed(EditorAction::Open(path.clone())) {
            return;
        }
        match files::load(&path) {
            Ok(map) => {
                let problems = self.playtest.problems(&map).len();
                self.start_document(map, Some(path.clone()));
                let shown = files::display_path(&path);
                if problems == 0 {
                    self.status.info(format!("Opened {shown}"));
                } else {
                    self.status
                        .warn(format!("Opened {shown}: {problems} problem(s), see Problems"));
                }
            }
            Err(error) => self.status.error(format!("Can't open: {error}")),
        }
    }

    /// Switches to another document: no gesture, no selection, the view on the chosen slot's start.
    pub(super) fn start_document(&mut self, map: MapDef, path: Option<PathBuf>) {
        self.gesture.0 = None;
        self.selection.0 = None;
        self.doc.replace(map, path);
        let map = self.doc.map();
        if map.slot(self.tools.slot).is_none() {
            self.tools.slot = SlotId(1);
        }
        let center = slot_center(map.slot(map.source_slot(self.tools.slot)), &map.terrain);
        self.view.center = drawn_position(&map.terrain, center);
    }

    pub(super) fn set_name(&mut self, name: String) {
        let name = name.trim();
        if name.is_empty() {
            self.status.warn("A map needs a name");
            return;
        }
        if self.doc.map().name != name {
            self.doc.edit(|m| m.name = name.to_string());
        }
    }

    pub(super) fn set_description(&mut self, text: String) {
        let text = text.trim();
        if text.chars().count() > MAX_DESCRIPTION_CHARS {
            self.status
                .warn(format!("A description has at most {MAX_DESCRIPTION_CHARS} characters"));
            return;
        }
        if self.doc.map().description != text {
            self.doc.edit(|m| m.description = text.to_string());
        }
    }

    /// Resizes within the editor's limits ([`MIN_MAP_SIDE`] to [`MAX_MAP_SIDE`], placeholders); a map with 90°
    /// rotation stays square. The locked part is generated for the new size.
    pub(super) fn resize(&mut self, width: u32, height: u32) {
        let side = MIN_MAP_SIDE..=MAX_MAP_SIDE;
        if !side.contains(&width) || !side.contains(&height) {
            self.status.warn(format!(
                "Map sides go from {MIN_MAP_SIDE} to {MAX_MAP_SIDE} tiles (placeholder limits)"
            ));
            return;
        }
        let map = self.doc.map();
        if map.symmetry == Symmetry::Rotate90 && width != height {
            self.status
                .warn("This map has 90 degree rotation: it stays square (width = height)");
            return;
        }
        if (map.terrain.width(), map.terrain.height()) == (width, height) {
            return;
        }
        self.finish_gesture();
        // Review finding: resizing may remove spawns, which shifts the indexes after them.
        self.selection.0 = None;
        let report = self.doc.edit(|m| m.resize(width, height));
        if report.is_empty() {
            self.status.info(format!("Resized the map to {width}x{height}"));
            return;
        }
        let mut dropped = Vec::new();
        if !report.removed_spawns.is_empty() {
            let spawns: Vec<String> = report
                .removed_spawns
                .iter()
                .map(|(i, s)| format!("#{i} {}", s.kind.def().name))
                .collect();
            dropped.push(format!("removed {}", spawns.join(", ")));
        }
        for s in &report.moved_starts {
            dropped.push(format!("moved S{}'s start onto the map", s.0));
        }
        for s in &report.cleared_camera_starts {
            dropped.push(format!("cleared S{}'s camera start", s.0));
        }
        self.status.warn(format!(
            "Resized the map to {width}x{height}; outside it: {} (Ctrl+Z restores)",
            dropped.join("; ")
        ));
    }

    /// Adds a set of slots (one per symmetric image) at free spots of the editable part.
    pub(super) fn add_slots(&mut self) {
        self.finish_gesture();
        let images = self.doc.map().image_count();
        let Some(id) = self.doc.edit(MapDef::add_slots) else {
            self.status.warn(format!(
                "At most {MAX_SLOTS} player slots, in sets of {images} (placeholder limits)"
            ));
            return;
        };
        self.tools.slot = id;
        if images > 1 {
            self.status
                .info(format!("Added slot {} and its {} image(s)", id.0, images - 1));
        } else {
            self.status.info(format!("Added slot {}", id.0));
        }
    }

    /// Removes the set of slots `slot` belongs to; their objects become neutral, and the slots after them move
    /// down.
    pub(super) fn remove_slots(&mut self, slot: SlotId) {
        let map = self.doc.map();
        if map.slot(slot).is_none() {
            self.status.warn(format!("There is no slot {}", slot.0));
            return;
        }
        let set = map.image_count();
        if map.slots.len() <= set {
            self.status.warn("A map needs at least one set of player slots");
            return;
        }
        self.finish_gesture();
        let Some(neutralized) = self.doc.edit(|m| m.remove_slots(slot)) else {
            return;
        };
        let what = if set > 1 {
            format!("Removed slot {}'s set of {set} slots", slot.0)
        } else {
            format!("Removed slot {}", slot.0)
        };
        if neutralized == 0 {
            self.status.info(what);
        } else {
            self.status
                .warn(format!("{what}: its {neutralized} object(s) are neutral now"));
        }
        self.selection.0 = None;
    }

    /// Moves a slot's start position: an editable slot, to a tile of the editable part.
    pub(super) fn set_start(&mut self, slot: SlotId, tile: TilePos) {
        if !self.slot_editable(slot) {
            return;
        }
        let map = self.doc.map();
        if !map.terrain.in_bounds(tile) {
            self.status.warn("Start positions go on the map");
            return;
        }
        if !map.region().tile(tile) {
            self.refuse_locked("that tile");
            return;
        }
        if map.slot(slot).is_some_and(|s| s.start != tile) {
            self.doc.edit(|m| {
                if let Some(s) = m.slot_mut(slot) {
                    s.start = tile;
                }
            });
        }
    }

    /// Sets or clears a slot's camera start: an editable slot, at a point of the editable part.
    pub(super) fn set_camera_start(&mut self, slot: SlotId, pos: Option<Vec2>) {
        if !self.slot_editable(slot) {
            return;
        }
        let map = self.doc.map();
        // Also rejects non-finite points.
        if pos.is_some_and(|p| !map.terrain.contains_point(p)) {
            self.status.warn("Camera starts go on the map");
            return;
        }
        if pos.is_some_and(|p| !map.region().point(p)) {
            self.refuse_locked("that point");
            return;
        }
        if map.slot(slot).is_some_and(|s| s.camera_start != pos) {
            self.doc.edit(|m| {
                if let Some(s) = m.slot_mut(slot) {
                    s.camera_start = pos;
                }
            });
        }
    }

    /// Closes the editor (review finding: the window's close button asks a second time when there are unsaved
    /// changes, like Open and New).
    pub(super) fn quit(&mut self) {
        self.finish_gesture();
        if self.confirmed(EditorAction::Quit) {
            self.exit.write(AppExit::Success);
        }
    }
}
