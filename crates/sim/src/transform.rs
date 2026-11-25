//! Transformations (plan decision T15, slice M3-E): an object that turns into another kind over time, in place, and
//! stays the same object. A Beacon warping in a building is one (`factions/colonists/economy.md > Warp In`: "The
//! Beacon transforms in place into the target structure"); a Drone deploying into a Beacon
//! (`factions/colonists/economy.md > Beacon Deployment`, slice M3-F) is the other.
//!
//! - **The state** ([`Transform`], `Object::transform`): what the object was and where it stood, which a cancel
//!   restores; how far it has come, of how many frames; and what it paid, which a cancel partly refunds. The object
//!   is the target kind from the first frame (`Sim::change_kind`; design change 0109: "the object is the target
//!   building under Warp In from the first frame, with its full footprint claimed at once"), so no unit can walk onto
//!   a building in the making, and it counts as a structure for defeat (`core/game_rules.md > Defeat Condition`,
//!   design change 0113), auto-targeting (design change 0103), Beacon Capacity (`factions/colonists/economy.md > Beacon
//!   Capacity Retention`: "A building being warped in keeps providing 5", design change 0112; `economy::army`) and the
//!   fog's records. Its functions wait for completion (design change 0116: "Production, drop-off, research and
//!   abilities start at completion"): it holds back what it would produce, construct or mine (`Sim::step_transforms`),
//!   and what a kind does by itself checks [`Object::is_transforming`] (a drop-off at an Egress Node, `sim/workers.rs`;
//!   a building counts for tech only once complete, `core/tech_and_research.md`). Its only command is the cancel (0116:
//!   "only Cancel").
//! - **The maxima** ([`Object::max_hp`], [`Object::max_shields`]): `factions/colonists/economy.md > Warp In >
//!   HP/Shields Interpolation`: "HP = Beacon MaxHP + (Building MaxHP - Beacon MaxHP) x progress", and the same for
//!   Shields, "where progress is frames elapsed / WarpInTime" ([`Transform::interpolate`]). The formulas set the
//!   maxima, and each frame the current HP rises by what its maximum rises ([`follow_max`], design change 0110: "damage
//!   is carried through unchanged (not scaled)", the damage taken before the transformation started included). The
//!   current Shields (`crate::shields`, slice M3-B) follow their maximum the same way, and regenerate up to it
//!   ("Shields regenerate normally up to the current maximum").
//! - **The step** (`Sim::step_transforms`, in the economy phase after construction, plan decision T5's position;
//!   `ARCHITECTURE.md > Tick pipeline`): each transformation advances one frame a tick ([`Transform::advance`] by
//!   `RATIO_ONE`: the Colonists have no power, plan decision T21), its first in the tick of its command, as a
//!   construction's (`ARCHITECTURE.md > Economy > Recipe timing`): ordered in tick `t`, a WarpInTime of `N` frames is
//!   complete at the end of tick `t + N - 1`, and the building works from tick `t + N`. Completion clears
//!   `Object::transform` and turns its functions on, with a `SimEvent::TransformCompleted`.
//! - **The commands** (plan decision T22): `ConstructionCommand::WarpIn` names one Beacon, a building and its
//!   orientation, no position: the footprint is derived from the Beacon ([`warp_in_footprint`], design change 0121:
//!   centred on it, an odd-sized building's extras at the bottom-right, turned and flipped about its centre). The
//!   Beacon pays the building's cost at once (`factions/colonists/economy.md > Constructs List`) and becomes the
//!   building on that footprint, where the placement check allows the Warp In method
//!   (`crate::placement::Method::WarpIn`, plan decision T16: every tile Visible, design change 0119; the Beacon's own
//!   tiles consumed, the others clear of ground units of any player, 0118); `ConstructionCommand::CancelTransform` goes
//!   to every transforming object it names (`factions/colonists/economy.md > Cancel Warp In`): "refunds 75% of the
//!   building's resource cost (rounded down), and the structure reverts to being a normal, fully-functional Beacon" on
//!   its old tiles (design change 0109), keeping the fraction of its HP it had, at least 1 ([`keep_fraction`], design
//!   change 0111), with a `SimEvent::TransformCancelled`. Both apply in the commands phase, before phase 5b brings the
//!   grids up to date with the footprint.
//! - **Destroyed** (`factions/colonists/economy.md > Vulnerability`: "If destroyed mid-Warp-In, the Beacon is destroyed
//!   along with it"): the normal removal path, with no refund (design change 0115: "Beacon and cost are both lost").
//!   Nothing it holds back is paid for, so the refunds of a destroyed producer or constructor (design changes
//!   0084, 0085) find nothing to give back, a portal's included.
//!
//! **Beacon Deployment** (slice M3-F, `crate::deploy`) uses the same state: the Drone is `from`, with no
//! `from_placement`, and its load is kept in [`Transform::load`] for a cancel.

use serde::{Deserialize, Serialize};

use crate::defs::{ObjectKind, Recipe};
use crate::hash::{StateHash, StateHasher};
use crate::object::{Object, Placement};
use crate::progress::Progress;
use crate::scenario::BuildingOrientation;
use crate::symmetry::Orientation;
use crate::units::TilePos;

/// An object's transformation into its kind (module docs): `Object::transform` while it runs, `None` otherwise.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Transform {
    /// The kind it was, and turns back into on a cancel: a Beacon, for a Warp In.
    pub from: ObjectKind,
    /// Where it stood as `from`, and stands again after a cancel: the Beacon's footprint (`None` for a unit, the
    /// Drone of a deployment).
    pub from_placement: Option<Placement>,
    /// How far it has come, of how many frames: the building's WarpInTime (`factions/colonists/economy.md > Constructs
    /// List`). In files its two numbers stand among the others, `progress` and `total`, as before it was a
    /// [`Progress`].
    #[serde(flatten)]
    pub progress: Progress,
    /// The Space Crystals it paid when it started, of which a cancel refunds 75% (`factions/colonists/economy.md >
    /// Cancel Warp In`).
    pub paid: u32,
    /// The load the Drone of a deployment carried as it began (`factions/colonists/economy.md > Beacon Deployment >
    /// Deploying with a load`, design change 0114: "The load stays with the deploying object and is lost when
    /// deployment completes; on cancel the Drone returns still carrying it"; slice M3-F, `crate::deploy`); 0 for a Warp
    /// In. Optional in files.
    #[serde(default)]
    pub load: u32,
}

impl Transform {
    /// A transformation from a `from` that stood at `from_placement`, following `recipe`
    /// (`factions/colonists/economy.md > Constructs List`: the cost, paid, and the frames): no frame done yet.
    pub fn new(from: ObjectKind, from_placement: Option<Placement>, recipe: &Recipe) -> Self {
        Self {
            from,
            from_placement,
            progress: Progress::new(recipe.frames),
            paid: recipe.cost,
            load: 0,
        }
    }

    /// "frames elapsed": the whole frames done, at most its total ([`Progress::ticks_done`]).
    pub fn frames_done(&self) -> u32 {
        self.progress.ticks_done()
    }

    /// Whether every frame is done.
    pub fn is_complete(&self) -> bool {
        self.progress.done()
    }

    /// The design's progress, "frames elapsed / WarpInTime", from 0 to 1 (1 for a transformation of no frames).
    pub fn ratio(&self) -> f32 {
        self.progress.ratio()
    }

    /// The whole percent done, rounded down: 100 only once it is complete.
    pub fn percent(&self) -> u32 {
        self.progress.percent()
    }

    /// `factions/colonists/economy.md > HP/Shields Interpolation`: a value going from `from` (the old kind's) to `to`
    /// (the new kind's) as the transformation progresses, "Beacon MaxHP + (Building MaxHP - Beacon MaxHP) x progress",
    /// with progress "frames elapsed / WarpInTime". Multiplied before it is divided: exactly `from` at 0 and `to` at
    /// the end, and, for the design's whole-number maxima, exact whenever the step per frame is (a Robo Portal's 0.75
    /// HP, an Egress Node's 0.25), so that a damaged object's HP stays a whole number short of its maximum
    /// ([`follow_max`]).
    pub fn interpolate(&self, from: f32, to: f32) -> f32 {
        match self.progress.total {
            0 => to,
            total => from + (to - from) * self.frames_done() as f32 / total as f32,
        }
    }

    /// One tick of the transformation at `speed`, in `economy::RATIO_ONE`s of a frame ([`Progress::step`]): whether
    /// it is complete.
    pub(crate) fn advance(&mut self, speed: u32) -> bool {
        self.progress.step(speed)
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
            paid,
            load,
        } = self;
        from.hash(h);
        from_placement.hash(h);
        // The frames done, then the total (`Progress`'s hash): the bytes of the two numbers it kept before.
        progress.hash(h);
        paid.hash(h);
        load.hash(h);
    }
}

impl Object {
    /// Whether it is transforming into its kind: a building being warped in, which doesn't work yet (module docs).
    pub fn is_transforming(&self) -> bool {
        self.transform.is_some()
    }

    /// Its MaxHP now: its kind's, a Tunnel's its tier's (`crate::tunnel`, plan decision T39: 600, 800 or 1000), or
    /// while it transforms the one interpolated from the kind it was (`factions/colonists/economy.md > HP/Shields
    /// Interpolation`: a Robo Portal warped in halfway from a Beacon has 120 + (600 - 120) × 0.5 = 360), or while it
    /// is a site the ConstructionHP Rule's (`construction::site`, slice M4-E: a Tunnel site halfway has 600 × (10% + 90%
    /// × 0.5) = 330). `None` for an indestructible object.
    pub fn max_hp(&self) -> Option<f32> {
        let to = crate::tunnel::tier_def(self).map_or(self.def().max_hp, |tier| Some(tier.max_hp));
        if let (Some(site), Some(def)) = (&self.construction, crate::construction::site::site_def(self.kind)) {
            return to.map(|full| site.max_hp(def, full));
        }
        match &self.transform {
            Some(t) => t.maximum(t.from.def().max_hp, to),
            None => to,
        }
    }

    /// Its MaxShields now, as [`Object::max_hp`]: a Robo Portal warped in halfway from a Beacon has 10 + (20 - 10) ×
    /// 0.5 = 15. `None` for an object without Shields. The current Shields stay within it (`crate::shields`).
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
/// damaged one keeps the damage in proportion. HP keeps at least 1 more (`tuning::construction::MIN_HP_AFTER_CANCEL`);
/// the Shields take the fraction as it is ("no minimum needed", `crate::shields`).
pub fn keep_fraction(current: f32, max: f32, restored: f32) -> f32 {
    if max > 0.0 {
        (restored * current / max).min(restored)
    } else {
        restored
    }
}

/// The Beacon's `Constructs List` (`factions/colonists/economy.md > Warp In > Constructs List`,
/// `StructureDef::warp_in`): what an object of `kind` warps in, with costs and WarpInTimes; empty for every kind but
/// the Beacon.
pub fn warp_in_recipes(kind: ObjectKind) -> &'static [Recipe] {
    kind.def().structure.as_ref().map_or(&[], |s| s.warp_in)
}

/// The entry for `kind` on the Warp In list of an object of kind `beacon`, if `kind` is on it.
pub fn warp_in_recipe(beacon: ObjectKind, kind: ObjectKind) -> Option<&'static Recipe> {
    warp_in_recipes(beacon).iter().find(|r| r.kind == kind)
}

/// `factions/colonists/economy.md > Warp In > Warp In Command` (design change 0121): the footprint of a `kind`
/// building warped in on the Beacon standing at `beacon`, turned by `orientation`. It is derived, never chosen: the
/// command carries no position, and there is no snapping to the cursor. In the default orientation the footprint is
/// "centered on the Beacon's location"; an odd-sized building, which can't be (the 3 × 3 Bio Portal on the 2 × 2
/// Beacon), "starts with its extra row and column at the bottom-right". Turning and flipping the building turn and
/// flip that footprint about the Beacon's centre, as they turn its sides (`Orientation::of_building`,
/// `crate::sides`), so that the extras move with the sides: a horizontal flip takes the extra column to the left, a
/// vertical flip the extra row to the top, each quarter turn the extras to the next corner clockwise. An
/// even-sized building is centred exactly in every orientation: the Egress Node (2 × 2) on the Beacon's own tiles,
/// the Robo Portal (4 × 4) one tile beyond them all round. The command gate, [`crate::Sim::warp_in_check`] and the
/// Warp In ghost (slice M3-J1) all take the footprint from here. `None` for a kind without a tile footprint (a
/// unit); a footprint that would lie past the tile coordinates stops at their edge (it is off the map anyway).
pub fn warp_in_footprint(beacon: &Placement, kind: ObjectKind, orientation: BuildingOrientation) -> Option<Placement> {
    let (w, h) = kind.def().tile_size()?;
    // In half tiles: the Beacon's centre along each axis, and, unturned, the building's centre's offset from it:
    // none if it fits, else half a tile towards +x or +y (east, south).
    let centre = |start: i32, len: u32| 2 * i64::from(start) + i64::from(len);
    let (cx, cy) = (centre(beacon.origin.x, beacon.w), centre(beacon.origin.y, beacon.h));
    let offset = ((cx - i64::from(w)).rem_euclid(2), (cy - i64::from(h)).rem_euclid(2));
    let turn = Orientation::of_building(orientation.rotation, orientation.flip_h, orientation.flip_v);
    let (dx, dy) = turn.apply(offset);
    let (w, h) = if orientation.swaps_axes() { (h, w) } else { (w, h) };
    // The top-left tile of a building `len` long whose centre is `d` half tiles from `c`: exact for the Beacon's 2 × 2.
    let start = |c: i64, d: i64, len: u32| {
        let start = (c + d - i64::from(len)).div_euclid(2);
        i32::try_from(start).unwrap_or(if start < 0 { i32::MIN } else { i32::MAX })
    };
    Some(Placement {
        origin: TilePos::new(start(cx, dx, w), start(cy, dy, h)),
        w,
        h,
        rotation: orientation.rotation,
        flip_h: orientation.flip_h,
        flip_v: orientation.flip_v,
    })
}

#[cfg(test)]
mod tests;
