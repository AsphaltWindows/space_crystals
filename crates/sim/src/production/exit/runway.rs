//! (M5-N7) A runway (`ExitRule::Runway`, plan decision T71): the Airstrip's, where its aircraft take off
//! (`factions/cults/structures/airstrip.md > Production`: "A finished aircraft **takes off from the Airstrip** to its
//! RallyPoint; Cults aircraft (so far) never land back"). They use no doorway: "Aircraft and units released underground
//! ... are not covered by this rule" (`core/entities.md > Structure Type > Exit Doorway`), so nothing on the runway
//! holds the next one back, and the building never pauses (`Sim::step_holding`).
//!
//! - **The runway** ([`Runway::of`]) runs between the middles of its two ends, the sides its rule letters `from` and
//!   `toward` (the Airstrip's B and C, its short sides), wherever they face once the building is placed
//!   (`crate::sides::sides`).
//! - **The start** ([`Runway::start`]): `factions/cults/units/sparrow.md > Flight rules`: "**Takeoff**: it appears
//!   over the runway's B end facing C at IdleSpeed and speeds up along the runway" (design change 0354 point 3). It
//!   appears with its silhouette wholly over the runway, against the `from` end, centred across it: **Placeholder (not
//!   in the design: where over the end it appears)**, read from "a finished aircraft is placed **directly on the
//!   runway**" (`factions/cults/structures/airstrip.md > Placeholder stats`).
//! - **The run** ([`Runway::takeoff`], `Mover::takeoff`): it flies straight along the runway, speeding up toward its
//!   MaxSpeed, until its centre is past the `toward` end; then it goes where its order sends it, as any Glider
//!   (`movement::glider`). **Placeholder (not in the design: where the run ends)**.
//! - **No rally point** ([`Runway::circle_point`]): "it circles 2 grid units past the runway's C end" (design change
//!   0354 point 4; `tuning::air::Air::runway_circle_past_gu`), the point that far along the runway past the middle of
//!   the `toward` end, or the point of the map nearest to it (`Map::clamp_point`: a Glider's orbit is a point of the
//!   map).

use glam::Vec2;

use crate::defs::ExitRule;
use crate::mover::Takeoff;
use crate::object::{Object, Placement};
use crate::ruleset::rules;
use crate::sides::{DIRECTIONS, sides};
use crate::units::{SU_PER_GU, gu};

/// A runway as its building is placed (module docs): its two ends, by the middles of their sides.
#[derive(Copy, Clone, Debug, PartialEq)]
pub struct Runway {
    /// The middle of the end its aircraft appear over (the side lettered `from`), space units.
    from: Vec2,
    /// The middle of the end they take off toward (the side lettered `toward`).
    toward: Vec2,
    /// The unit vector along the runway, from `from` toward `toward`: the way out of the `toward` side, which faces the
    /// other way from the `from` side (`defs_are_consistent`).
    along: Vec2,
}

impl Runway {
    /// The runway of `building`: `None` for anything but a placed structure whose production takes off from one
    /// (`ExitRule::Runway`).
    pub fn of(building: &Object) -> Option<Runway> {
        let placement = building.placement.as_ref()?;
        let production = building.def().structure.as_ref()?.production.as_ref()?;
        let ExitRule::Runway { from, toward } = production.exits else {
            return None;
        };
        let letters = sides(building.kind, placement.orientation())?;
        let side = |letter| letters.iter().position(|&l| l == letter).map(|i| end(placement, i));
        let ((from, _), (toward, along)) = (side(from)?, side(toward)?);
        Some(Runway { from, toward, along })
    }

    /// Where an aircraft with half extents `half` appears (module docs): over the `from` end, its silhouette wholly
    /// over the runway and against that end, centred across it.
    pub fn start(&self, half: Vec2) -> Vec2 {
        self.from + self.along * self.along.abs().dot(half)
    }

    /// The unit vector along the runway, toward its `toward` end: the way an aircraft faces as it appears.
    pub fn along(&self) -> Vec2 {
        self.along
    }

    /// The point an aircraft out of it circles when its building has no rally point (module docs): 2 grid units past
    /// the middle of the `toward` end, along the runway. It may lie off the map: the caller takes the nearest point of
    /// the map (`Map::clamp_point`).
    pub fn circle_point(&self) -> Vec2 {
        self.toward + self.along * gu(rules().tuning.air.runway_circle_past_gu)
    }

    /// The takeoff run of an aircraft that appears on it (module docs): along the runway to the `toward` end.
    pub fn takeoff(&self) -> Takeoff {
        Takeoff {
            end: self.toward,
            along: self.along,
        }
    }
}

/// The side of `placement` facing `DIRECTIONS[i]`: the middle of it, and the unit vector out of it.
fn end(placement: &Placement, i: usize) -> (Vec2, Vec2) {
    let (dx, dy) = DIRECTIONS[i];
    let out = Vec2::new(dx as f32, dy as f32);
    let half = Vec2::new(placement.w as f32, placement.h as f32) * (SU_PER_GU / 2.0);
    (placement.center() + out * out.abs().dot(half), out)
}

#[cfg(test)]
mod tests;
