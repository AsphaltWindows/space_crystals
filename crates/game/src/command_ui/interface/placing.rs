//! AwaitingPlacement's building (`factions/gdo/structures.md > DeploymentCenter >
//! ObjectInterfaceState[DeploymentCenter] > AwaitingPlacement`, `factions/gdo/economy.md > ExtractionFacility >
//! ObjectInterfaceState[ExtractionFacility] > AwaitingPlacement`): which one is placed ([`Placing`]), where its ghost
//! stands under the cursor ([`Placing::origin_under`]), and how the placement keys turn it ([`Placing::turned`]). The
//! ghost and the build area overlay that show it are `crate::placement`'s.
//!
//! The placement keys are the ones the map editor turns its structures with, one definition for both: the shared
//! placement actions of the input layer (`crate::input::InputAction::is_placement`, which take R, Shift+R, F and
//! Shift+F while the game reports a placement, `crate::input::Placing`), and their turns
//! (`crate::editor::orientation::Turn`: as seen on the map).

use bevy::math::Vec2;
use sc_sim::placement::footprint_size;
use sc_sim::{BuildingOrientation, ObjectId, ObjectKind, SU_PER_GU, TilePos};

use crate::editor::orientation::Turn;

/// `> AwaitingPlacement`: the building ready to place that the local player is placing (plan decision T10).
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub struct Placing {
    /// The constructor holding it ready: the ActiveGroup's one Deployment Center or Extraction Facility. Placement
    /// never moves on to another one (`> SelectionGroup > No smart-casting`, design change 0071).
    pub constructor: ObjectId,
    /// The ready building.
    pub kind: ObjectKind,
    /// How the ghost is turned (`core/entities.md > Structure Type`: flipped first, then rotated clockwise), which a
    /// click places it with. Unturned as placement starts (placeholder: the design doesn't say whether the last
    /// orientation is kept).
    pub orientation: BuildingOrientation,
}

impl Placing {
    /// The placement of `constructor`'s ready `kind`, unturned.
    pub fn new(constructor: ObjectId, kind: ObjectKind) -> Self {
        Self {
            constructor,
            kind,
            orientation: BuildingOrientation::default(),
        }
    }

    /// The footprint's size `(w, h)` in tiles, as turned: "Non-square buildings swap footprint dimensions when
    /// rotated to 90 or 270".
    pub fn size(self) -> Option<(u32, u32)> {
        footprint_size(self.kind, self.orientation)
    }

    /// The top-left tile of the ghost's footprint with the cursor on the ground point `ground`: "A ghost preview of
    /// the building follows the cursor, snapped to the grid". The footprint whose centre is nearest the point, so the
    /// cursor stays in the middle of the ghost (on the tile under it, for an odd size). `None` for a point that isn't
    /// finite, or a kind without a footprint.
    pub fn origin_under(self, ground: Vec2) -> Option<TilePos> {
        let (w, h) = self.size()?;
        if !ground.is_finite() {
            return None;
        }
        // Float to int casts saturate: a point far off the map gives a tile far off it, never a panic.
        let snap = |at: f32, size: u32| (at / SU_PER_GU - size as f32 / 2.0).round() as i32;
        Some(TilePos::new(snap(ground.x, w), snap(ground.y, h)))
    }

    /// The same, with the ghost turned by `turn`: "R rotates the ghost 90 degrees clockwise, Shift+R rotates 90
    /// degrees counter-clockwise. [...] F flips the ghost horizontally, Shift+F flips vertically", as seen on the
    /// map, whichever way it is turned already.
    pub fn turned(self, turn: Turn) -> Self {
        Self {
            orientation: turn.apply(self.orientation),
            ..self
        }
    }
}
