//! The contact step a moving unit is taking — `systems/units.md > UnitCollision > Ground Collision`, "The contact
//! step" (design change 0323), `docs/movement.md > 6.3`, rule 4. The movement step begins and takes it
//! (`movement::collide::contact`).

use serde::{Deserialize, Serialize};

/// A contact step under way (`docs/movement.md > 6.3`, rule 4): the unit steps "at **90°** to its heading (straight
/// sideways)" by "the unit's own width", which takes it several ticks. Boxes are axis-aligned, so the step runs along
/// one axis, the blocker's face, to a coordinate fixed when it began.
#[derive(Copy, Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct StepAside {
    /// The axis the unit steps along: y if set, else x.
    pub along_y: bool,
    /// The coordinate on that axis at which the step ends, space units.
    pub to: f32,
}

impl StepAside {
    /// The index of the step's axis in a `Vec2`.
    pub fn axis(&self) -> usize {
        usize::from(self.along_y)
    }

    /// Whether a unit that moved from `from` to `pos` on the step's axis has made the step: it is on the
    /// coordinate, or past it (a unit with momentum).
    pub fn is_done(&self, from: f32, pos: f32) -> bool {
        (self.to - from) * (self.to - pos) <= 0.0
    }
}
