//! The contact step a moving unit is taking — `systems/units.md > UnitCollision > Ground Collision`, "The contact
//! step" (design change 0323), `docs/movement.md > 6.3`, rule 4. The movement step begins and takes it
//! (`movement::collide::contact`).

use glam::Vec2;
use serde::{Deserialize, Serialize};

/// A contact step under way (`docs/movement.md > 6.3`, rule 4): the unit steps "at **90°** to its heading (straight
/// sideways)" by "the unit's own width", which takes it several ticks. For a box the step runs along one axis, the
/// blocker's face, to a coordinate fixed when it began; (M5-V3) for a Silhouette that turns with its facing, straight
/// along the face it met, whatever its direction, to a point fixed when it began ([`StepAside::end`]).
#[derive(Copy, Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct StepAside {
    /// The axis the unit steps along: y if set, else x.
    pub along_y: bool,
    /// The coordinate on that axis at which the step ends, space units.
    pub to: f32,
    /// (M5-V3) The point a turned Silhouette's step ends on, if it is one: the step runs straight to it, and
    /// `along_y` and `to` are unused. `None` for a box. Serialized only while set, as it is hashed.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub end: Option<Vec2>,
}

impl StepAside {
    /// The index of the step's axis in a `Vec2`.
    pub fn axis(&self) -> usize {
        usize::from(self.along_y)
    }

    /// Whether a unit that moved from `from` to `pos` has made the step: it is on its end, or past it (a unit with
    /// momentum), along the step's axis or (M5-V3) its direction.
    pub fn is_done(&self, from: Vec2, pos: Vec2) -> bool {
        match self.end {
            Some(end) => pos == end || (end - from).dot(end - pos) <= 0.0,
            None => {
                let axis = self.axis();
                (self.to - from[axis]) * (self.to - pos[axis]) <= 0.0
            }
        }
    }
}
