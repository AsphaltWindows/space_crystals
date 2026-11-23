//! The edited document: a map (`MapDef`: what a map file holds), its file, and its undo history.
//!
//! Every edit is an operation on the map. An edit opened with [`EditorDoc::begin_edit`] may change the map many
//! times (a brush stroke, a drag) and becomes one undo step when it ends. Undo keeps a snapshot of the whole map per
//! step: maps are small.
//!
//! On a symmetric map only the editable part is edited (`core/map_editor.md > Symmetry > Editing a symmetric
//! map`). Every change through [`EditorDoc::map_mut`] marks the locked part stale, and [`EditorDoc::regenerate`]
//! generates it again from the editable part, which the editor does after every action. Opening a file, undo and
//! redo don't: they bring back a map as it was.

use std::path::{Path, PathBuf};
use std::sync::OnceLock;

use bevy::prelude::*;
use sc_sim::MapDef;

/// The most undo steps kept; older ones are forgotten.
pub const UNDO_LIMIT: usize = 200;

#[derive(Resource, Debug)]
pub struct EditorDoc {
    map: MapDef,
    /// Where Ctrl+S saves. A new map has none until its first save (`maps/<name>.json`).
    path: Option<PathBuf>,
    /// The map as last loaded or saved: the document is dirty while it differs.
    saved: MapDef,
    undo: Vec<MapDef>,
    redo: Vec<MapDef>,
    /// The map before the open edit, if one is open.
    open_edit: Option<MapDef>,
    /// Changes with every (possible) change of the map, so views know when to catch up.
    revision: u64,
    /// The map changed since the locked part was last generated.
    stale: bool,
    /// [`EditorDoc::is_dirty`], worked out when first asked after a change (review finding: the UI compared whole
    /// maps every frame).
    dirty: OnceLock<bool>,
}

impl EditorDoc {
    /// A document with no unsaved changes and no history.
    pub fn new(map: MapDef, path: Option<PathBuf>) -> Self {
        Self {
            saved: map.clone(),
            map,
            path,
            undo: Vec::new(),
            redo: Vec::new(),
            open_edit: None,
            revision: 0,
            stale: false,
            dirty: OnceLock::new(),
        }
    }

    /// Replaces the document, e.g. when a file is opened: the history starts over.
    pub fn replace(&mut self, map: MapDef, path: Option<PathBuf>) {
        let revision = self.revision + 1;
        *self = Self::new(map, path);
        self.revision = revision;
    }

    pub fn map(&self) -> &MapDef {
        &self.map
    }

    /// The map, for changing it. Outside an open edit, wrap changes in [`EditorDoc::edit`] so they can be undone.
    pub fn map_mut(&mut self) -> &mut MapDef {
        self.changed();
        self.stale = true;
        &mut self.map
    }

    fn changed(&mut self) {
        self.revision += 1;
        self.dirty = OnceLock::new();
    }

    pub fn path(&self) -> Option<&Path> {
        self.path.as_deref()
    }

    pub fn revision(&self) -> u64 {
        self.revision
    }

    /// Whether the map differs from the file.
    pub fn is_dirty(&self) -> bool {
        *self.dirty.get_or_init(|| self.map != self.saved)
    }

    /// Whether the map changed since its locked part was last generated.
    pub fn is_stale(&self) -> bool {
        self.stale
    }

    /// Generates the locked part of a symmetric map from its editable part ([`MapDef::regenerate`]) if the map
    /// changed since the last time. Returns whether it did.
    pub fn regenerate(&mut self) -> bool {
        if !std::mem::take(&mut self.stale) || self.map.image_count() < 2 {
            return false;
        }
        self.map.regenerate();
        self.changed();
        true
    }

    pub fn can_undo(&self) -> bool {
        !self.undo.is_empty()
    }

    pub fn can_redo(&self) -> bool {
        !self.redo.is_empty()
    }

    /// Undo steps available.
    pub fn undo_depth(&self) -> usize {
        self.undo.len()
    }

    /// Opens an edit: every change until [`EditorDoc::end_edit`] becomes one undo step. An edit that is already
    /// open stays open.
    pub fn begin_edit(&mut self) {
        if self.open_edit.is_none() {
            self.open_edit = Some(self.map.clone());
        }
    }

    pub fn edit_open(&self) -> bool {
        self.open_edit.is_some()
    }

    /// Closes the open edit. If it changed the map, it becomes an undo step and the redo steps are dropped. Returns
    /// whether it changed anything.
    pub fn end_edit(&mut self) -> bool {
        let Some(before) = self.open_edit.take() else {
            return false;
        };
        if before == self.map {
            return false;
        }
        self.undo.push(before);
        if self.undo.len() > UNDO_LIMIT {
            self.undo.remove(0);
        }
        self.redo.clear();
        true
    }

    /// Abandons the open edit, restoring the map as it was before it.
    pub fn cancel_edit(&mut self) {
        if let Some(before) = self.open_edit.take() {
            self.map = before;
            self.stale = false;
            self.changed();
        }
    }

    /// One complete edit: `change` modifies the map, as one undo step. Returns what `change` returns.
    pub fn edit<R>(&mut self, change: impl FnOnce(&mut MapDef) -> R) -> R {
        let nested = self.edit_open();
        self.begin_edit();
        let result = change(self.map_mut());
        if !nested {
            self.end_edit();
        }
        result
    }

    /// Steps back one edit (closing an open one first). Returns whether there was one.
    pub fn undo(&mut self) -> bool {
        self.end_edit();
        let Some(previous) = self.undo.pop() else {
            return false;
        };
        let current = std::mem::replace(&mut self.map, previous);
        self.redo.push(current);
        self.stale = false;
        self.changed();
        true
    }

    /// Repeats the edit last undone. Returns whether there was one.
    pub fn redo(&mut self) -> bool {
        self.end_edit();
        let Some(next) = self.redo.pop() else {
            return false;
        };
        let current = std::mem::replace(&mut self.map, next);
        self.undo.push(current);
        self.stale = false;
        self.changed();
        true
    }

    /// Records that the map was written to `path`.
    pub fn mark_saved(&mut self, path: PathBuf) {
        self.saved = self.map.clone();
        self.path = Some(path);
        self.changed();
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use sc_sim::{MapSettings, Symmetry, Tile, TilePos, TilePresetId, scenarios};

    fn doc() -> EditorDoc {
        EditorDoc::new(scenarios::empty_map(8, 6), None)
    }

    #[test]
    fn edits_undo_and_redo_one_step_at_a_time() {
        let mut d = doc();
        assert!(!d.is_dirty() && !d.can_undo());
        d.edit(|m| m.description = "5".into());
        d.edit(|m| m.description = "6".into());
        assert!(d.is_dirty());
        assert_eq!(d.undo_depth(), 2);
        assert!(d.undo());
        assert_eq!(d.map().description, "5");
        assert!(d.undo());
        assert_eq!(d.map().description, "");
        assert!(!d.is_dirty(), "back to the file's state");
        assert!(!d.undo());
        assert!(d.redo());
        assert_eq!(d.map().description, "5");
        // A new edit drops the redo steps.
        d.edit(|m| m.description = "9".into());
        assert!(!d.redo());
        assert_eq!(d.map().description, "9");
    }

    #[test]
    fn an_open_edit_is_one_step_and_unchanged_edits_are_none() {
        let mut d = doc();
        d.begin_edit();
        d.map_mut().description = "2".into();
        d.map_mut().description = "3".into();
        d.map_mut().name = "x".into();
        assert!(d.end_edit());
        assert_eq!(d.undo_depth(), 1);
        d.edit(|m| m.description = "3".into());
        assert_eq!(d.undo_depth(), 1, "an edit that changes nothing is no step");
        d.begin_edit();
        d.map_mut().description = "77".into();
        d.cancel_edit();
        assert_eq!(d.map().description, "3");
        assert!(d.undo());
        assert_eq!((d.map().description.as_str(), d.map().name.as_str()), ("", "empty"));
    }

    /// Review finding: the dirty flag is cached, and follows every change, undo, redo and save.
    #[test]
    fn saving_clears_the_dirty_flag_and_history_is_capped() {
        let mut d = doc();
        for n in 0..(UNDO_LIMIT + 10) {
            d.edit(|m| m.description = n.to_string());
            assert!(d.is_dirty());
        }
        assert_eq!(d.undo_depth(), UNDO_LIMIT);
        d.mark_saved(PathBuf::from("maps/x.json"));
        assert!(!d.is_dirty());
        assert_eq!(d.path(), Some(Path::new("maps/x.json")));
        d.undo();
        assert!(d.is_dirty());
        d.redo();
        assert!(!d.is_dirty());
        let revision = d.revision();
        d.replace(scenarios::empty_map(4, 4), None);
        assert!(!d.can_undo() && !d.is_dirty() && d.revision() > revision);
    }

    /// Changes mark the locked part stale; regenerating makes it the images of the editable part again. Undo, redo
    /// and opening bring back maps as they were.
    #[test]
    fn changes_regenerate_the_locked_part() {
        let map = MapSettings {
            symmetry: Symmetry::MirrorVerticalAxis,
            width: 16,
            height: 16,
            ..MapSettings::default()
        }
        .create()
        .unwrap();
        let mut d = EditorDoc::new(map, None);
        assert!(!d.regenerate(), "nothing changed");
        d.edit(|m| m.terrain.set_tile(TilePos::new(1, 2), Tile::new(TilePresetId::WATER)));
        assert!(d.is_stale());
        assert_eq!(d.map().first_asymmetry(), Some(sc_sim::scenario::Asymmetry::Tile(TilePos::new(14, 2))));
        assert!(d.regenerate());
        assert!(!d.is_stale());
        assert_eq!(d.map().terrain.tile(TilePos::new(14, 2)), Some(Tile::new(TilePresetId::WATER)));
        assert_eq!(d.map().first_asymmetry(), None);
        d.undo();
        assert!(!d.is_stale() && !d.regenerate());

        // Without a symmetry there is nothing to generate.
        let mut d = doc();
        d.edit(|m| m.description = "x".into());
        assert!(!d.regenerate());
    }
}
