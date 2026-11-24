//! Transformations (plan decision T15, slice M3-E): an object that turns into another kind over time, in place, and
//! stays the same object. A Beacon warping in a building is one (`factions/colonists/economy.md > Warp In`: "The
//! Beacon transforms in place into the target structure"); a Drone deploying into a Beacon (`> Beacon Deployment`,
//! slice M3-F) is the other.
//!
//! - **The state** ([`Transform`], `Object::transform`): what the object was and where it stood, which a cancel
//!   restores; how far it has come, of how many frames; and what it paid, which a cancel partly refunds. The object
//!   is the target kind from the first frame (`Sim::change_kind`; design change 0109: "the object is the target
//!   building under Warp In from the first frame, with its full footprint claimed at once"), so no unit can walk onto
//!   a building in the making, and it counts as a structure for defeat (`core/game_rules.md > Defeat Condition`,
//!   design change 0113), auto-targeting (design change 0103), Beacon Capacity (`> Beacon Capacity Retention`: "A
//!   building being warped in keeps providing 5", design change 0112; `economy::army`) and the fog's records. Its
//!   functions wait for completion (design change 0116: "Production, drop-off, research and abilities start at
//!   completion"): it holds back what it would produce, construct or mine (`Sim::step_transforms`), and what a kind
//!   does by itself checks [`Object::is_transforming`] (a drop-off at an Egress Node, `sim/workers.rs`; a building
//!   counts for tech only once complete, `core/tech_and_research.md`). Its only command is the cancel (0116: "only
//!   Cancel").
//! - **The maxima** ([`Object::max_hp`], [`Object::max_shields`]): `> Warp In > HP/Shields Interpolation`: "HP =
//!   Beacon MaxHP + (Building MaxHP - Beacon MaxHP) x progress", and the same for Shields, "where progress is frames
//!   elapsed / WarpInTime" ([`Transform::interpolate`]). The formulas set the maxima, and each frame the current HP
//!   rises by what its maximum rises ([`follow_max`], design change 0110: "damage is carried through unchanged (not
//!   scaled)", the damage taken before the transformation started included). Shields have no current value until
//!   slice M3-B (plan decision T18): their maximum interpolates here, and the current Shields are to follow it the
//!   same way, and regenerate up to it ("Shields regenerate normally up to the current maximum").
//! - **The step** (`Sim::step_transforms`, in the economy phase after construction, plan decision T5's position;
//!   `ARCHITECTURE.md > Tick pipeline`): each transformation advances one frame a tick ([`Transform::advance`] by
//!   `RATIO_ONE`: the Colonists have no power, plan decision T21), its first in the tick of its command, as a
//!   construction's (`ARCHITECTURE.md > Economy > Recipe timing`): ordered in tick `t`, a WarpInTime of `N` frames is
//!   complete at the end of tick `t + N - 1`, and the building works from tick `t + N`. Completion clears
//!   `Object::transform` and turns its functions on, with a `SimEvent::TransformCompleted`.
//! - **The commands** (plan decision T22): `ConstructionCommand::WarpIn` names one Beacon, which pays the building's
//!   cost at once (`> Constructs List`) and becomes the building on the footprint it confirmed, which the placement
//!   check allows for the Warp In method (`crate::placement::Method::WarpIn`, plan decision T16);
//!   `ConstructionCommand::CancelTransform` goes to every transforming object it names (`> Cancel Warp In`): "refunds
//!   75% of the building's resource cost (rounded down), and the structure reverts to being a normal,
//!   fully-functional Beacon" on its old tiles (design change 0109), keeping the fraction of its HP it had, at least
//!   1 ([`keep_fraction`], design change 0111), with a `SimEvent::TransformCancelled`. Both apply in the commands
//!   phase, before phase 5b brings the grids up to date with the footprint.
//! - **Destroyed** (`> Vulnerability`: "If destroyed mid-Warp-In, the Beacon is destroyed along with it"): the normal
//!   removal path, with no refund (design change 0115: "Beacon and cost are both lost"). Nothing it holds back is
//!   paid for, so the refunds of a destroyed producer or constructor (design changes 0084, 0085) find nothing to
//!   give back, a portal's included.
//!
//! **For slice M3-F**, which deploys a Drone into a Beacon with the same state: the Drone is `from`, with no
//! `from_placement`; its load goes into [`Transform`] as a field of its own (with `#[serde(default)]`).

use serde::{Deserialize, Serialize};

use crate::defs::{ObjectKind, Recipe};
use crate::economy::RATIO_ONE;
use crate::hash::{StateHash, StateHasher};
use crate::object::{Object, Placement};

/// An object's transformation into its kind (module docs): `Object::transform` while it runs, `None` otherwise.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Transform {
    /// The kind it was, and turns back into on a cancel: a Beacon, for a Warp In.
    pub from: ObjectKind,
    /// Where it stood as `from`, and stands again after a cancel: the Beacon's footprint (`None` for a unit, the
    /// Drone of a deployment).
    pub from_placement: Option<Placement>,
    /// How far it has come, in fixed point (plan decision T6): [`RATIO_ONE`] for each frame done; complete at
    /// `total × RATIO_ONE`.
    pub progress: u32,
    /// How many frames it takes: the building's WarpInTime (`> Constructs List`).
    pub total: u32,
    /// The Space Crystals it paid when it started, of which a cancel refunds 75% (`> Cancel Warp In`).
    pub paid: u32,
}

impl Transform {
    /// A transformation from a `from` that stood at `from_placement`, following `recipe` (`> Constructs List`: the
    /// cost, paid, and the frames): no frame done yet.
    pub fn new(from: ObjectKind, from_placement: Option<Placement>, recipe: &Recipe) -> Self {
        Self {
            from,
            from_placement,
            progress: 0,
            total: recipe.frames,
            paid: recipe.cost,
        }
    }

    /// "frames elapsed": the whole frames done, at most [`Transform::total`].
    pub fn frames_done(&self) -> u32 {
        (self.progress / RATIO_ONE).min(self.total)
    }

    /// Whether every frame is done.
    pub fn is_complete(&self) -> bool {
        self.progress >= self.total.saturating_mul(RATIO_ONE)
    }

    /// The design's progress, "frames elapsed / WarpInTime", from 0 to 1 (1 for a transformation of no frames).
    pub fn ratio(&self) -> f32 {
        match self.total {
            0 => 1.0,
            total => self.frames_done() as f32 / total as f32,
        }
    }

    /// The whole percent done, rounded down: 100 only once it is complete.
    pub fn percent(&self) -> u32 {
        match self.total {
            0 => 100,
            total => (u64::from(self.frames_done()) * 100 / u64::from(total)) as u32,
        }
    }

    /// `> HP/Shields Interpolation`: a value going from `from` (the old kind's) to `to` (the new kind's) as the
    /// transformation progresses, "Beacon MaxHP + (Building MaxHP - Beacon MaxHP) x progress", with progress "frames
    /// elapsed / WarpInTime". Multiplied before it is divided: exactly `from` at 0 and `to` at the end, and, for the
    /// design's whole-number maxima, exact whenever the step per frame is (a Robo Portal's 0.75 HP, an Egress Node's
    /// 0.25), so that a damaged object's HP stays a whole number short of its maximum ([`follow_max`]).
    pub fn interpolate(&self, from: f32, to: f32) -> f32 {
        match self.total {
            0 => to,
            total => from + (to - from) * self.frames_done() as f32 / total as f32,
        }
    }

    /// One tick of the transformation at `speed`, in [`RATIO_ONE`]s of a frame: whether it is complete. Saturating:
    /// the WarpInTimes are short enough for their progress to fit (`ObjectDef` consistency tests), a hand-edited save
    /// needn't be.
    pub(crate) fn advance(&mut self, speed: u32) -> bool {
        self.progress = self.progress.saturating_add(speed);
        self.is_complete()
    }

    /// An interpolated maximum, from the old kind's `from` to the new kind's `to`: the new kind's if the old one has
    /// none, none if the new one has none.
    fn maximum(&self, from: Option<f32>, to: Option<f32>) -> Option<f32> {
        let to = to?;
        Some(from.map_or(to, |from| self.interpolate(from, to)))
    }
}

impl StateHash for Transform {
    fn hash(&self, h: &mut StateHasher) {
        let Transform {
            from,
            from_placement,
            progress,
            total,
            paid,
        } = self;
        from.hash(h);
        from_placement.hash(h);
        progress.hash(h);
        total.hash(h);
        paid.hash(h);
    }
}

impl Object {
    /// Whether it is transforming into its kind: a building being warped in, which doesn't work yet (module docs).
    pub fn is_transforming(&self) -> bool {
        self.transform.is_some()
    }

    /// Its MaxHP now: its kind's, or while it transforms the one interpolated from the kind it was (`> HP/Shields
    /// Interpolation`: a Robo Portal warped in halfway from a Beacon has 120 + (600 - 120) × 0.5 = 360). `None` for
    /// an indestructible object.
    pub fn max_hp(&self) -> Option<f32> {
        let to = self.def().max_hp;
        match &self.transform {
            Some(t) => t.maximum(t.from.def().max_hp, to),
            None => to,
        }
    }

    /// Its MaxShields now, as [`Object::max_hp`]: a Robo Portal warped in halfway from a Beacon has 10 + (20 - 10) ×
    /// 0.5 = 15. `None` for an object without Shields. Nothing has current Shields until slice M3-B (plan decision
    /// T18), which keeps them within this maximum.
    pub fn max_shields(&self) -> Option<f32> {
        let to = self.def().max_shields;
        match &self.transform {
            Some(t) => t.maximum(t.from.def().max_shields, to),
            None => to,
        }
    }
}

/// The current value of a maximum going from `before` to `after` in one frame (design change 0110: "current +=
/// (newMax − oldMax) each frame; never a fraction-of-max rescale"): damage taken stays taken, whenever it was taken,
/// and an undamaged object stays exactly at its maximum. It never ends above the maximum. A maximum that falls (no
/// transformation of the design's has one) takes it down by as much, but never kills: what would reach 0 stays as it
/// was, within the maximum.
pub fn follow_max(current: f32, before: f32, after: f32) -> f32 {
    let followed = (current + (after - before)).min(after);
    if followed > 0.0 { followed } else { current.min(after) }
}

/// The current value that a cancel leaves of `current` out of `max`, on the restored maximum `restored` (design
/// change 0111: "the same fraction of HP and Shields the transforming object had at cancel time", "original_max ×
/// current_before ÷ max_before"), at most `restored`: an undamaged object is exactly at its old maximum again, a
/// damaged one keeps the damage in proportion. HP keeps at least 1 more ([`MIN_HP_AFTER_CANCEL`]); the Shields of
/// slice M3-B take the fraction as it is ("no minimum needed").
pub fn keep_fraction(current: f32, max: f32, restored: f32) -> f32 {
    if max > 0.0 {
        (restored * current / max).min(restored)
    } else {
        restored
    }
}

/// Design change 0111: after a cancel the HP is "max(1, original_max × current_before ÷ max_before)", so that "a
/// nearly dead transforming object" survives.
pub const MIN_HP_AFTER_CANCEL: f32 = 1.0;

/// The Beacon's `Constructs List` (`> Warp In > Constructs List`, `StructureDef::warp_in`): what an object of `kind`
/// warps in, with costs and WarpInTimes; empty for every kind but the Beacon.
pub fn warp_in_recipes(kind: ObjectKind) -> &'static [Recipe] {
    kind.def().structure.as_ref().map_or(&[], |s| s.warp_in)
}

/// The entry for `kind` on the Warp In list of an object of kind `beacon`, if `kind` is on it.
pub fn warp_in_recipe(beacon: ObjectKind, kind: ObjectKind) -> Option<&'static Recipe> {
    warp_in_recipes(beacon).iter().find(|r| r.kind == kind)
}

#[cfg(test)]
mod tests;
