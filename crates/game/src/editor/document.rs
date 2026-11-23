//! Document actions: files, map properties and player slots.

use std::path::PathBuf;

use sc_sim::{Faction, PlayerId, PlayerSetup, Scenario, TilePos};

use super::actions::{Editor, EditorAction};
use super::files::{self, MAX_MAP_SIDE, NEW_MAP_NAME, NEW_MAP_SIZE};
use crate::camera::start_view;

/// The most player slots a map gets in the editor (placeholder until Q0004 settles slot counts).
pub const MAX_SLOTS: usize = 8;

impl Editor<'_> {
    pub(super) fn save(&mut self) {
        self.finish_gesture();
        let scenario = self.doc.scenario();
        let path = match self.doc.path() {
            Some(path) => path.to_path_buf(),
            None => files::path_for_new_map(&self.files.dir, &scenario.name),
        };
        if let Err(error) = files::save(&path, scenario) {
            self.status.error(format!("Save failed: {error}"));
            return;
        }
        let problems = scenario.validate().len();
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
    fn confirmed(&mut self, action: EditorAction) -> bool {
        if !self.doc.is_dirty() || self.confirm.0.as_ref() == Some(&action) {
            self.confirm.0 = None;
            return true;
        }
        self.confirm.0 = Some(action);
        self.status
            .warn("Unsaved changes: do that again to discard them, or save first (Ctrl+S)");
        false
    }

    pub(super) fn open(&mut self, path: PathBuf) {
        if !self.confirmed(EditorAction::Open(path.clone())) {
            return;
        }
        match files::load(&path) {
            Ok(scenario) => {
                let problems = scenario.validate().len();
                self.start_document(scenario, Some(path.clone()));
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

    pub(super) fn new_map(&mut self) {
        if !self.confirmed(EditorAction::NewMap) {
            return;
        }
        let (width, height) = NEW_MAP_SIZE;
        self.start_document(files::new_scenario(NEW_MAP_NAME, width, height), None);
        self.status
            .info(format!("New {width}x{height} map: Ctrl+S saves it in maps/"));
    }

    /// Switches to another document: no gesture, no selection, the view on the chosen slot's start.
    fn start_document(&mut self, scenario: Scenario, path: Option<PathBuf>) {
        self.gesture.0 = None;
        self.selection.0 = None;
        self.doc.replace(scenario, path);
        let scenario = self.doc.scenario();
        if !scenario.players.iter().any(|p| p.id == self.tools.slot) {
            self.tools.slot = scenario.players.first().map_or(PlayerId(1), |p| p.id);
        }
        self.view.center = start_view(scenario, self.tools.slot, &scenario.map);
    }

    pub(super) fn set_name(&mut self, name: String) {
        let name = name.trim();
        if name.is_empty() {
            self.status.warn("A map needs a name");
            return;
        }
        self.doc.edit(|s| s.name = name.to_string());
    }

    pub(super) fn resize(&mut self, width: u32, height: u32) {
        if !(1..=MAX_MAP_SIDE).contains(&width) || !(1..=MAX_MAP_SIDE).contains(&height) {
            self.status.warn(format!("Map sides go from 1 to {MAX_MAP_SIDE} tiles"));
            return;
        }
        let map = &self.doc.scenario().map;
        if (map.width(), map.height()) == (width, height) {
            return;
        }
        self.finish_gesture();
        // Review finding: resizing may remove spawns, which shifts the indexes after them.
        self.selection.0 = None;
        let report = self.doc.edit(|s| s.resize_map(width, height));
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
        for (what, slots) in [
            ("start location", &report.cleared_start_locations),
            ("camera start", &report.cleared_camera_starts),
        ] {
            for p in slots {
                dropped.push(format!("cleared P{}'s {what}", p.0));
            }
        }
        self.status.warn(format!(
            "Resized the map to {width}x{height}; outside it: {} (Ctrl+Z restores)",
            dropped.join("; ")
        ));
    }

    pub(super) fn add_slot(&mut self) {
        if self.doc.scenario().players.len() >= MAX_SLOTS {
            self.status.warn(format!("At most {MAX_SLOTS} player slots"));
            return;
        }
        let n = self.doc.scenario().players.len() + 1;
        if let Some(id) = self.doc.edit(|s| s.add_player(format!("Player {n}"), Faction::Gdo)) {
            self.tools.slot = id;
            self.status.info(format!("Added player slot {}", id.0));
        }
    }

    pub(super) fn remove_slot(&mut self, slot: PlayerId) {
        if !self.doc.scenario().players.iter().any(|p| p.id == slot) {
            self.status.warn(format!("There is no player {}", slot.0));
            return;
        }
        if self.doc.scenario().players.len() <= 1 {
            self.status.warn("A map needs at least one player slot");
            return;
        }
        self.finish_gesture();
        match self.doc.edit(|s| s.remove_player(slot)) {
            Some(0) | None => self.status.info(format!("Removed player slot {}", slot.0)),
            Some(n) => self.status.warn(format!(
                "Removed player slot {}: its {n} object(s) are neutral now",
                slot.0
            )),
        }
        // The slots after it moved down one number.
        if self.tools.slot.0 > slot.0 {
            self.tools.slot = PlayerId(self.tools.slot.0 - 1);
        }
        if let Some(owner) = self.tools.owner.filter(|o| o.0 >= slot.0) {
            self.tools.owner = (owner != slot).then(|| PlayerId(owner.0 - 1));
        }
        self.selection.0 = None;
    }

    pub(super) fn edit_slot(&mut self, slot: PlayerId, change: impl FnOnce(&mut PlayerSetup)) {
        if !self.doc.scenario().players.iter().any(|p| p.id == slot) {
            self.status.warn(format!("There is no player {}", slot.0));
            return;
        }
        self.doc.edit(|s| {
            if let Some(p) = s.players.iter_mut().find(|p| p.id == slot) {
                change(p);
            }
        });
    }

    pub(super) fn set_slot_name(&mut self, slot: PlayerId, name: String) {
        let name = name.trim().to_string();
        if name.is_empty() {
            self.status.warn("A player slot needs a name");
            return;
        }
        self.edit_slot(slot, |p| p.name = name);
    }

    pub(super) fn set_start_location(&mut self, slot: PlayerId, tile: Option<TilePos>) {
        if tile.is_some_and(|t| !self.doc.scenario().map.in_bounds(t)) {
            self.status.warn("Start locations go on the map");
            return;
        }
        self.edit_slot(slot, |p| p.start_location = tile);
    }

    pub(super) fn set_camera_start(&mut self, slot: PlayerId, pos: Option<bevy::math::Vec2>) {
        // Also rejects non-finite points.
        if pos.is_some_and(|p| !self.doc.scenario().map.contains_point(p)) {
            self.status.warn("Camera starts go on the map");
            return;
        }
        self.edit_slot(slot, |p| p.camera_start = pos);
    }
}
