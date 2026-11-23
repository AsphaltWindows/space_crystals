//! Vision — `core/vision.md`. Phase 5f (`docs/movement.md > 4`): what each player sees, from the final positions
//! of the tick.
//!
//! Slice V fills it. The units' SightRange is already on their definitions (`ObjectDef::sight_range`).

use std::collections::BTreeMap;

use crate::ids::ObjectId;
use crate::object::Object;

/// Phase 5f. Empty until slice V.
pub(crate) fn update(_objects: &BTreeMap<ObjectId, Object>) {}
