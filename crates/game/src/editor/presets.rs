//! Custom tile presets (`core/map_editor.md > Custom Tile Presets`): the map maker adds, edits and removes them. A
//! custom preset has a name, any combination of the game properties (`core/map.md > TilePreset`), and one of the
//! existing looks, which the renderer draws it with. The default presets are fixed.
//!
//! A preset that tiles still use can't be removed (placeholder rule, `sc_sim::Map::remove_preset`): the map maker
//! paints them over first, so that no terrain changes behind their back.

use sc_sim::map::DEFAULT_PRESET_SYMBOLS;
use sc_sim::{TileLook, TilePreset, TilePresetId};
use serde::{Deserialize, Serialize};

use super::actions::Editor;
use super::tools::Tool;

/// The most custom presets a map gets in the editor (placeholder: the design sets no limit; the map format's text
/// symbols allow 87).
pub const MAX_CUSTOM_PRESETS: usize = 32;
/// The longest preset name.
pub const MAX_PRESET_NAME_CHARS: usize = 24;

/// A game property of a tile preset (`core/map.md > TilePreset`).
#[derive(Copy, Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum PresetFlag {
    Buildable,
    Traversible,
    Rugged,
    Drillable,
    Recruitable,
}

impl PresetFlag {
    pub const ALL: [PresetFlag; 5] = [
        PresetFlag::Buildable,
        PresetFlag::Traversible,
        PresetFlag::Rugged,
        PresetFlag::Drillable,
        PresetFlag::Recruitable,
    ];

    pub fn label(self) -> &'static str {
        match self {
            PresetFlag::Buildable => "Buildable",
            PresetFlag::Traversible => "Traversible",
            PresetFlag::Rugged => "Rugged",
            PresetFlag::Drillable => "Drillable",
            PresetFlag::Recruitable => "Recruitable",
        }
    }

    pub fn get(self, preset: &TilePreset) -> bool {
        match self {
            PresetFlag::Buildable => preset.buildable,
            PresetFlag::Traversible => preset.traversible,
            PresetFlag::Rugged => preset.rugged,
            PresetFlag::Drillable => preset.drillable,
            PresetFlag::Recruitable => preset.recruitable,
        }
    }

    pub fn set(self, preset: &mut TilePreset, value: bool) {
        let flag = match self {
            PresetFlag::Buildable => &mut preset.buildable,
            PresetFlag::Traversible => &mut preset.traversible,
            PresetFlag::Rugged => &mut preset.rugged,
            PresetFlag::Drillable => &mut preset.drillable,
            PresetFlag::Recruitable => &mut preset.recruitable,
        };
        *flag = value;
    }
}

/// How many presets every map has: the default ones.
pub fn default_preset_count() -> usize {
    DEFAULT_PRESET_SYMBOLS.len()
}

impl Editor<'_> {
    /// Adds a custom preset with Plane's properties and look, named "Custom n", and chooses it for painting (or
    /// filling).
    pub(super) fn add_preset(&mut self) {
        let presets = self.doc.map().terrain.presets();
        let custom = presets.len().saturating_sub(default_preset_count());
        if custom >= MAX_CUSTOM_PRESETS {
            self.status.warn(format!(
                "At most {MAX_CUSTOM_PRESETS} custom presets (placeholder limit)"
            ));
            return;
        }
        let name = (custom + 1..)
            .map(|n| format!("Custom {n}"))
            .find(|name| presets.iter().all(|p| p.name != *name))
            .expect("some name is free");
        let preset = TilePreset {
            name: name.clone(),
            look: TileLook::Plane,
            ..presets[TilePresetId::PLANE.0 as usize].clone()
        };
        self.finish_gesture();
        let id = self.doc.edit(|m| m.terrain.add_preset(preset));
        self.tools.preset = id;
        if self.tools.tool != Tool::Fill {
            self.tools.tool = Tool::Paint;
        }
        self.status.info(format!(
            "Added the custom preset {name}: set its properties and look below"
        ));
    }

    /// Changes custom preset `id` with `change`; the default presets are fixed.
    pub(super) fn edit_preset(&mut self, id: TilePresetId, change: impl FnOnce(&mut TilePreset)) {
        let Some(current) = self.doc.map().terrain.presets().get(id.0 as usize) else {
            self.status.warn(format!("The map has no preset {}", id.0));
            return;
        };
        if id.is_default() {
            self.status.warn(format!(
                "{} is a default preset, which is fixed: add a custom preset instead",
                current.name
            ));
            return;
        }
        let mut preset = current.clone();
        change(&mut preset);
        if preset == *current {
            return;
        }
        self.finish_gesture();
        if let Err(error) = self.doc.edit(|m| m.terrain.set_preset(id, preset)) {
            self.status.warn(format!("Can't change the preset: {error}"));
        }
    }

    pub(super) fn set_preset_name(&mut self, id: TilePresetId, name: String) {
        let name = name.trim().to_string();
        if name.is_empty() || name.chars().count() > MAX_PRESET_NAME_CHARS {
            self.status
                .warn(format!("A preset name has 1 to {MAX_PRESET_NAME_CHARS} characters"));
            return;
        }
        self.edit_preset(id, |p| p.name = name);
    }

    pub(super) fn toggle_preset_flag(&mut self, id: TilePresetId, flag: PresetFlag) {
        self.edit_preset(id, |p| flag.set(p, !flag.get(p)));
    }

    /// Removes custom preset `id` if no tile uses it; the presets after it move down one id.
    pub(super) fn remove_preset(&mut self, id: TilePresetId) {
        let terrain = &self.doc.map().terrain;
        let Some(preset) = terrain.presets().get(id.0 as usize) else {
            self.status.warn(format!("The map has no preset {}", id.0));
            return;
        };
        let name = preset.name.clone();
        if id.is_default() {
            self.status
                .warn(format!("{name} is a default preset, which every map has"));
            return;
        }
        let used = terrain.preset_use_count(id);
        if used > 0 {
            self.status.warn(format!(
                "{name} is still used by {used} tile(s): paint them with another preset first"
            ));
            return;
        }
        self.finish_gesture();
        if let Err(error) = self.doc.edit(|m| m.terrain.remove_preset(id)) {
            self.status.warn(format!("Can't remove {name}: {error}"));
            return;
        }
        // The presets after it moved down one id.
        let chosen = self.tools.preset;
        if chosen == id {
            self.tools.preset = TilePresetId::PLANE;
        } else if chosen > id {
            self.tools.preset = TilePresetId(chosen.0 - 1);
        }
        self.status.info(format!("Removed the custom preset {name}"));
    }
}
