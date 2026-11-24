//! The editor's view state besides the document and the tools: cursor, selection, status line, files, problems.

use std::path::PathBuf;

use bevy::prelude::*;
use sc_sim::{Faction, ScenarioProblem, SlotId};

use super::files;

/// What is selected: a spawn (by its index in `MapDef::spawns`), one of a slot's markers, or an object of a slot's
/// Starting Layout of a faction (by its index in the layout), always in the editable part of the map. An index goes
/// stale when an undo, a redo or a resize reorders the spawns, so those clear the selection; regenerating the
/// locked part keeps it on its spawn, and never touches the layouts of editable slots.
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum Selection {
    Spawn(usize),
    Start(SlotId),
    CameraStart(SlotId),
    LayoutObject {
        slot: SlotId,
        faction: Faction,
        index: usize,
    },
}

#[derive(Resource, Default, Debug)]
pub struct EditorSelection(pub Option<Selection>);

/// The ground point under the cursor (space units, `crate::coords::ground_under`), while the cursor is over the
/// map view.
#[derive(Resource, Default, Debug)]
pub struct EditorCursor(pub Option<Vec2>);

#[derive(Copy, Clone, Debug, Default, PartialEq, Eq)]
pub enum StatusLevel {
    #[default]
    Info,
    Warning,
    Error,
}

/// The status line's message: what the last action did, or why it didn't.
#[derive(Resource, Default, Debug)]
pub struct EditorStatus {
    pub message: String,
    pub level: StatusLevel,
}

impl EditorStatus {
    pub fn info(&mut self, message: impl Into<String>) {
        self.set(StatusLevel::Info, message.into());
    }

    pub fn warn(&mut self, message: impl Into<String>) {
        self.set(StatusLevel::Warning, message.into());
    }

    pub fn error(&mut self, message: impl Into<String>) {
        self.set(StatusLevel::Error, message.into());
    }

    fn set(&mut self, level: StatusLevel, message: String) {
        self.level = level;
        self.message = message;
        self.log();
    }

    /// Writes the message to the log at its level.
    pub fn log(&self) {
        let message = &self.message;
        match self.level {
            StatusLevel::Info => info!("editor: {message}"),
            StatusLevel::Warning => warn!("editor: {message}"),
            StatusLevel::Error => error!("editor: {message}"),
        }
    }
}

/// The `maps/` listing of the files panel.
#[derive(Resource, Debug)]
pub struct EditorFiles {
    pub dir: PathBuf,
    pub maps: Vec<PathBuf>,
}

impl EditorFiles {
    pub fn new(dir: PathBuf) -> Self {
        let maps = files::list_maps(&dir);
        Self { dir, maps }
    }

    /// Lists the directory again; changes the resource only when the listing changed.
    pub fn refresh(&mut self) {
        let maps = files::list_maps(&self.dir);
        if maps != self.maps {
            self.maps = maps;
        }
    }
}

/// `MapDef::validate()` of the document, as last shown.
#[derive(Resource, Default, Debug)]
pub struct EditorProblems(pub Vec<ScenarioProblem>);

/// UI toggles.
#[derive(Resource, Default, Debug)]
pub struct EditorUi {
    /// The keyboard help overlay (F1).
    pub help: bool,
    /// The overlay that tints flat, sloped and too steep tiles (K).
    pub slopes: bool,
}
