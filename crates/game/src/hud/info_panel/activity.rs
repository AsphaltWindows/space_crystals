//! The InfoPanel's activity area (`core/entities.md > Object Type > InfoPanel > Default InfoPanel for structures`,
//! design change 0105): for one of the local player's own structures, "what it is doing: construction or production
//! with progress, queue length, a building ready to place, and the rally point", under its HP, in a text of its own
//! (`InfoText::Activity`, hidden while it has no line). Another player's structure has no activity area: "name, owner
//! and HP only (no production or queue information)".
//!
//! **The extension point** for the structures' activities: [`lines`] gives the area's lines, one capability of the
//! structure after another, in this order, and the panel shows them as they come:
//! - construction (the constructor capability of slice M2-D1; its lines come with the construction UI, slice
//!   M2-H2): the building under construction and its progress, or the building ready to place;
//! - production (the production capability of slice M2-D2; its lines come with the production UI, slice M2-H1): the
//!   unit in production and its progress, the queue length, and the rally point.
//!
//! Nothing fills it yet: at this point no structure has a construction or a production state to show (a plate's
//! mining is its own section's line, the patch's remaining crystals, which every viewer sees: `super`). Lines read
//! only the sim and the structure, so a test can check them without the UI.

use sc_sim::{Object, Sim};

/// What `structure`, one of the local player's own, is doing, one line per item, for the activity area (module docs):
/// empty while it has nothing to show.
pub fn lines(_sim: &Sim, _structure: &Object) -> Vec<String> {
    Vec::new()
}
