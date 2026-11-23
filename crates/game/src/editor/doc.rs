//! The edited document: a `Scenario`, its file, and its undo history.
//!
//! Every edit is an operation on the scenario. An edit opened with [`EditorDoc::begin_edit`] may change the
//! scenario many times (a brush stroke, a drag) and becomes one undo step when it ends. Undo keeps a snapshot of the
//! whole scenario per step: scenarios are small.

use std::path::{Path, PathBuf};

use bevy::prelude::*;
use sc_sim::Scenario;

/// The most undo steps kept; older ones are forgotten.
pub const UNDO_LIMIT: usize = 200;

#[derive(Resource, Debug)]
pub struct EditorDoc {
    scenario: Scenario,
    /// Where Ctrl+S saves. A new map has none until its first save (`maps/<name>.json`).
    path: Option<PathBuf>,
    /// The scenario as last loaded or saved: the document is dirty while it differs.
    saved: Scenario,
    undo: Vec<Scenario>,
    redo: Vec<Scenario>,
    /// The scenario before the open edit, if one is open.
    open_edit: Option<Scenario>,
    /// Changes with every (possible) change of the scenario, so views know when to catch up.
    revision: u64,
}

impl EditorDoc {
    /// A document with no unsaved changes and no history.
    pub fn new(scenario: Scenario, path: Option<PathBuf>) -> Self {
        Self {
            saved: scenario.clone(),
            scenario,
            path,
            undo: Vec::new(),
            redo: Vec::new(),
            open_edit: None,
            revision: 0,
        }
    }

    /// Replaces the document, e.g. when a file is opened: the history starts over.
    pub fn replace(&mut self, scenario: Scenario, path: Option<PathBuf>) {
        let revision = self.revision + 1;
        *self = Self::new(scenario, path);
        self.revision = revision;
    }

    pub fn scenario(&self) -> &Scenario {
        &self.scenario
    }

    /// The scenario, for changing it. Outside an open edit, wrap changes in [`EditorDoc::edit`] so they can be
    /// undone.
    pub fn scenario_mut(&mut self) -> &mut Scenario {
        self.revision += 1;
        &mut self.scenario
    }

    pub fn path(&self) -> Option<&Path> {
        self.path.as_deref()
    }

    pub fn revision(&self) -> u64 {
        self.revision
    }

    /// Whether the scenario differs from the file.
    pub fn is_dirty(&self) -> bool {
        self.scenario != self.saved
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
            self.open_edit = Some(self.scenario.clone());
        }
    }

    pub fn edit_open(&self) -> bool {
        self.open_edit.is_some()
    }

    /// Closes the open edit. If it changed the scenario, it becomes an undo step and the redo steps are dropped.
    /// Returns whether it changed anything.
    pub fn end_edit(&mut self) -> bool {
        let Some(before) = self.open_edit.take() else {
            return false;
        };
        if before == self.scenario {
            return false;
        }
        self.undo.push(before);
        if self.undo.len() > UNDO_LIMIT {
            self.undo.remove(0);
        }
        self.redo.clear();
        true
    }

    /// Abandons the open edit, restoring the scenario as it was before it.
    pub fn cancel_edit(&mut self) {
        if let Some(before) = self.open_edit.take() {
            self.scenario = before;
            self.revision += 1;
        }
    }

    /// One complete edit: `change` modifies the scenario, as one undo step. Returns what `change` returns.
    pub fn edit<R>(&mut self, change: impl FnOnce(&mut Scenario) -> R) -> R {
        let nested = self.edit_open();
        self.begin_edit();
        let result = change(self.scenario_mut());
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
        let current = std::mem::replace(self.scenario_mut(), previous);
        self.redo.push(current);
        true
    }

    /// Repeats the edit last undone. Returns whether there was one.
    pub fn redo(&mut self) -> bool {
        self.end_edit();
        let Some(next) = self.redo.pop() else {
            return false;
        };
        let current = std::mem::replace(self.scenario_mut(), next);
        self.undo.push(current);
        true
    }

    /// Records that the scenario was written to `path`.
    pub fn mark_saved(&mut self, path: PathBuf) {
        self.saved = self.scenario.clone();
        self.path = Some(path);
        self.revision += 1;
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use sc_sim::scenarios;

    fn doc() -> EditorDoc {
        EditorDoc::new(scenarios::empty(8, 6), None)
    }

    #[test]
    fn edits_undo_and_redo_one_step_at_a_time() {
        let mut d = doc();
        assert!(!d.is_dirty() && !d.can_undo());
        d.edit(|s| s.seed = 5);
        d.edit(|s| s.seed = 6);
        assert!(d.is_dirty());
        assert_eq!(d.undo_depth(), 2);
        assert!(d.undo());
        assert_eq!(d.scenario().seed, 5);
        assert!(d.undo());
        assert_eq!(d.scenario().seed, 1);
        assert!(!d.is_dirty(), "back to the file's state");
        assert!(!d.undo());
        assert!(d.redo());
        assert_eq!(d.scenario().seed, 5);
        // A new edit drops the redo steps.
        d.edit(|s| s.seed = 9);
        assert!(!d.redo());
        assert_eq!(d.scenario().seed, 9);
    }

    #[test]
    fn an_open_edit_is_one_step_and_unchanged_edits_are_none() {
        let mut d = doc();
        d.begin_edit();
        d.scenario_mut().seed = 2;
        d.scenario_mut().seed = 3;
        d.scenario_mut().name = "x".into();
        assert!(d.end_edit());
        assert_eq!(d.undo_depth(), 1);
        d.edit(|s| s.seed = 3);
        assert_eq!(d.undo_depth(), 1, "an edit that changes nothing is no step");
        d.begin_edit();
        d.scenario_mut().seed = 77;
        d.cancel_edit();
        assert_eq!(d.scenario().seed, 3);
        assert!(d.undo());
        assert_eq!((d.scenario().seed, d.scenario().name.as_str()), (1, "empty"));
    }

    #[test]
    fn saving_clears_the_dirty_flag_and_history_is_capped() {
        let mut d = doc();
        for seed in 0..(UNDO_LIMIT as u64 + 10) {
            d.edit(|s| s.seed = seed + 100);
        }
        assert_eq!(d.undo_depth(), UNDO_LIMIT);
        d.mark_saved(PathBuf::from("maps/x.json"));
        assert!(!d.is_dirty());
        assert_eq!(d.path(), Some(Path::new("maps/x.json")));
        let revision = d.revision();
        d.replace(scenarios::empty(4, 4), None);
        assert!(!d.can_undo() && d.revision() > revision);
    }
}
