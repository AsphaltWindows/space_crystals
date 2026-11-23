//! Combat — `systems/combat.md`. Phase 5e (`docs/movement.md > 4`): attack phases, the list of hits, damage and
//! deaths, shooting from the positions phase 5d committed.
//!
//! Slice W fills it. The units' attack attributes are already on their definitions (`defs::AttackAttrs`).

use std::collections::BTreeMap;

use crate::ids::ObjectId;
use crate::object::Object;

/// Phase 5e. Empty until slice W. A death that removes a blocking footprint must re-sync the grids
/// (`paths::sync`), as phase 5b does.
pub(crate) fn step(_objects: &mut BTreeMap<ObjectId, Object>, _first_new: ObjectId) {}
