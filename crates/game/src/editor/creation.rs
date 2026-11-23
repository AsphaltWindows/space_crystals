//! The creation dialog for new maps (`core/map_editor.md > Map Creation`): the map type, size, slot count,
//! symmetry, name and description, within the placeholder limits of `sc_sim::scenario::create` (sides from
//! `MIN_MAP_SIDE` to `MAX_MAP_SIDE`, at most `MAX_SLOTS` slots, in whole sets of symmetric images). A Skirmish
//! Default map needs a symmetry. The type and the symmetry are fixed once the map exists.

use bevy::prelude::*;
use sc_sim::{MapSettings, MapType, Symmetry};
use serde::{Deserialize, Serialize};

use super::actions::{Editor, EditorAction};
use super::tools::symmetry_label;

/// The creation dialog: whether it is open, and its settings, which it keeps for the next new map.
#[derive(Resource, Debug, Default)]
pub struct NewMapDialog {
    pub open: bool,
    pub settings: MapSettings,
}

/// One setting of the creation dialog.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub enum NewMapField {
    Type(MapType),
    Width(u32),
    Height(u32),
    Slots(usize),
    Symmetry(Symmetry),
    Name(String),
    Description(String),
}

/// Keeps the slot count one that the symmetry allows: the nearest allowed count at or above it, else the largest.
fn fit_slots(settings: &mut MapSettings) {
    let allowed = MapSettings::slot_counts(settings.symmetry);
    if !allowed.contains(&settings.slots) {
        settings.slots = allowed
            .iter()
            .copied()
            .find(|&n| n >= settings.slots)
            .or_else(|| allowed.last().copied())
            .unwrap_or(settings.slots);
    }
}

impl Editor<'_> {
    /// Opens the creation dialog. With unsaved changes, only when asked for twice: the dialog's map replaces the
    /// document.
    pub(super) fn new_map(&mut self) {
        if self.dialog.open || !self.confirmed(EditorAction::NewMap) {
            return;
        }
        self.finish_gesture();
        self.dialog.open = true;
        self.status
            .info("New map: choose its type, size, symmetry and slots, then Create");
    }

    pub(super) fn edit_new_map(&mut self, field: NewMapField) {
        if !self.dialog.open {
            self.status.warn("The new map dialog is closed: New (Ctrl+N) opens it");
            return;
        }
        let settings = &mut self.dialog.settings;
        match field {
            NewMapField::Type(map_type) => {
                settings.map_type = map_type;
                // `core/map_editor.md > Map Types > Skirmish Default`: symmetric by construction.
                if map_type == MapType::SkirmishDefault && settings.symmetry == Symmetry::None {
                    settings.symmetry = Symmetry::Rotate180;
                    fit_slots(settings);
                }
            }
            // A map with 90 degree rotation is square: either side sets both.
            NewMapField::Width(width) => {
                settings.width = width;
                if settings.symmetry == Symmetry::Rotate90 {
                    settings.height = width;
                }
            }
            NewMapField::Height(height) => {
                settings.height = height;
                if settings.symmetry == Symmetry::Rotate90 {
                    settings.width = height;
                }
            }
            NewMapField::Slots(slots) => settings.slots = slots,
            NewMapField::Symmetry(symmetry) => {
                settings.symmetry = symmetry;
                if symmetry == Symmetry::Rotate90 {
                    settings.height = settings.width;
                }
                fit_slots(settings);
            }
            NewMapField::Name(name) => settings.name = name,
            NewMapField::Description(text) => settings.description = text,
        }
    }

    /// Creates the dialog's map and edits it, or says why the settings make none.
    pub(super) fn create_map(&mut self) {
        if !self.dialog.open {
            self.status.warn("The new map dialog is closed: New (Ctrl+N) opens it");
            return;
        }
        let settings = self.dialog.settings.clone();
        match settings.create() {
            Ok(map) => {
                self.dialog.open = false;
                self.start_document(map, None);
                self.status.info(format!(
                    "New {} map, {}x{}, {} slots, symmetry {}: Ctrl+S saves it in maps/",
                    settings.map_type.label(),
                    settings.width,
                    settings.height,
                    settings.slots,
                    symmetry_label(settings.symmetry)
                ));
            }
            Err(problem) => self.status.warn(format!("Can't create the map: {problem}")),
        }
    }

    pub(super) fn cancel_new_map(&mut self) {
        if self.dialog.open {
            self.dialog.open = false;
            self.status.info("No new map");
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn slot_counts_follow_the_symmetry() {
        let mut settings = MapSettings {
            slots: 2,
            symmetry: Symmetry::MirrorBothAxes,
            ..MapSettings::default()
        };
        fit_slots(&mut settings);
        assert_eq!(settings.slots, 4);
        settings.slots = 6;
        settings.symmetry = Symmetry::Rotate90;
        fit_slots(&mut settings);
        assert_eq!(settings.slots, 8);
        settings.symmetry = Symmetry::Rotate180;
        settings.slots = 7;
        fit_slots(&mut settings);
        assert_eq!(settings.slots, 8);
        settings.slots = 99;
        fit_slots(&mut settings);
        assert_eq!(settings.slots, 8, "the largest allowed");
    }
}
