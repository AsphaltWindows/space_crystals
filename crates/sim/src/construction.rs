//! Construction (slice M2-D1): the GDO constructors, the Deployment Center and the Extraction Facility, build one
//! building at a time and hold it ready until the player places it — `factions/gdo/structures/deployment_center.md`,
//! `factions/gdo/economy.md > ExtractionFacility`.
//!
//! - **The capability** ([`Constructor`], `Object::constructor`): the design's instance state
//!   (`factions/gdo/structures/deployment_center.md > DeploymentCenterInstanceState`, `factions/gdo/economy.md >
//!   ExtractionFacilityInstanceState`: CurrentConstruction, ConstructionProgress, ReadyToPlace) as one of three states:
//!   idle, constructing, or holding a building ready to place. The design's menus allow no other ("Other build options
//!   unavailable" while it constructs or holds a ready building), so a constructor never does both, and holds one ready
//!   building at most (design change 0071: "GDO builders have only one ready building at a time"). A structure type
//!   with a `Constructs` list (`StructureDef::constructs`, its [`Recipe`]s) is a constructor.
//! - **Commands** ([`ConstructionCommand`], in `Command::Construction`): Construct, Cancel Construction, Cancel
//!   Ready Building and Place (`factions/gdo/structures/deployment_center.md > ObjectInterfaceState[DeploymentCenter]`,
//!   `factions/gdo/economy.md > ObjectInterfaceState[ExtractionFacility]`), for the player's own constructors. The
//!   command gate (`Sim::validate`) checks them, and applying them goes through the same check (`sim/construction.rs`).
//!   A command goes to every constructor it names that can carry it out, in id order (no smart-casting, design change
//!   0071), except Place: one building is placed by one constructor. The Colonists' Warp In and its cancel are
//!   construction commands too (plan decision T22), for their Beacons and the buildings being warped in:
//!   `crate::transform`.
//! - **Costs and refunds** (plan decision T7): Construct pays the building's cost when it applies, from the bank as
//!   the earlier commands of the tick and the earlier constructors of the command left it, and is refused when the
//!   bank is short ("Only available if player has sufficient Space Crystals"). Cancelling refunds through
//!   [`Refund`]: in full while constructing, 75% rounded down once ready
//!   (`factions/gdo/structures/deployment_center.md > Cancellation`, `factions/gdo/economy.md > ExtractionFacility >
//!   Cancellation`). A constructor that leaves the game refunds the same way, as if cancelled (design change 0084),
//!   through the removal path (`Sim::remove_objects`). Each refund says what went back
//!   (`SimEvent::ConstructionCancelled`).
//! - **Progress** ([`step`], in the economy phase; plan decisions T5 and T6): a construction advances each tick by
//!   its constructor's speed (`PowerSummary::speed`), in [`RATIO_ONE`]s of a frame at full speed, and is ready once
//!   it reaches its recipe's frames. The Deployment Center, a power producer, builds at full speed whatever the
//!   power; the Extraction Facility, a consumer, at the power ratio (design change 0089, `factions/gdo/overview.md >
//!   GDOResources > Power`). The ratio is rounded up, so a plate at 30/51 takes exactly 400 × 51/30 = 680 ticks.
//!   The first frame is the tick of the command that starts the construction, which applies before the economy
//!   phase (`ARCHITECTURE.md > Economy > Recipe timing`): ordered in tick `t`, a Generator (240 frames) is ready at
//!   the end of tick `t + 239`, and a Place can put it on the map from tick `t + 240`.
//! - **Placing** (`factions/gdo/structures/deployment_center.md > AwaitingPlacement`, `factions/gdo/economy.md >
//!   ExtractionFacility > AwaitingPlacement`): Place puts the ready building on the map in its own tick, where the
//!   placement check in Command mode (`crate::placement`) allows it when the command applies (plan decision T4): the
//!   footprint's tiles flat, Buildable, Visible and free, at least one in the build area, a plate onto an uncovered
//!   patch. A Place the check refuses at that moment leaves the building ready, and the sim reports why
//!   (`SimEvent::PlacementRefused`). The new structure counts for its owner's power and build area at once, blocks
//!   ground units from the same tick's movement on (phase 5b), joins its owner's planning grids at the end of the tick
//!   (`paths::sync_known`), and works from the next tick on (`ARCHITECTURE.md > Tick pipeline > Spawn tick`).
//! - **Not built** (`artifacts/director/m2-plan.md` 1.2): tech prerequisites, as none of the buildings with a cost and
//!   a time has one; ConstructionHP, as GDO buildings are placed whole.

use std::collections::BTreeMap;

use serde::{Deserialize, Serialize};

use crate::defs::{ObjectKind, Recipe};
use crate::economy::{PowerSummary, RATIO_ONE, Refund};
use crate::events::SimEvent;
use crate::hash::{StateHash, StateHasher};
use crate::ids::{ObjectId, PlayerId};
use crate::object::Object;
use crate::scenario::BuildingOrientation;
use crate::sim::Reject;
use crate::units::TilePos;

/// A constructor's state (module docs): `Object::constructor` of a Deployment Center or an Extraction Facility.
#[derive(Copy, Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub enum Constructor {
    /// No CurrentConstruction and no ReadyToPlace: its BuildMenu offers its buildings.
    #[default]
    Idle,
    /// CurrentConstruction: `kind`. `progress` is its ConstructionProgress in fixed point (plan decision T6):
    /// [`RATIO_ONE`] for each frame done at full speed; it is ready at its recipe's `frames × RATIO_ONE`.
    Constructing { kind: ObjectKind, progress: u32 },
    /// ReadyToPlace: `kind`, built and waiting to be placed.
    Ready { kind: ObjectKind },
}

impl Constructor {
    /// CurrentConstruction: the building it is constructing.
    pub fn current(&self) -> Option<ObjectKind> {
        match *self {
            Constructor::Constructing { kind, .. } => Some(kind),
            Constructor::Idle | Constructor::Ready { .. } => None,
        }
    }

    /// ReadyToPlace: the building it holds ready to place.
    pub fn ready(&self) -> Option<ObjectKind> {
        match *self {
            Constructor::Ready { kind } => Some(kind),
            Constructor::Idle | Constructor::Constructing { .. } => None,
        }
    }

    /// Whether it constructs nothing and holds nothing ready: only then can it start a construction.
    pub fn is_idle(&self) -> bool {
        *self == Constructor::Idle
    }

    /// ConstructionProgress as the design counts it, "frames elapsed": the whole frames of its CurrentConstruction
    /// done so far at full speed, out of its recipe's `frames`. `None` while it constructs nothing.
    pub fn frames_done(&self) -> Option<u32> {
        match *self {
            Constructor::Constructing { progress, .. } => Some(progress / RATIO_ONE),
            Constructor::Idle | Constructor::Ready { .. } => None,
        }
    }
}

impl StateHash for Constructor {
    fn hash(&self, h: &mut StateHasher) {
        match self {
            Constructor::Idle => h.tag(0),
            Constructor::Constructing { kind, progress } => {
                h.tag(1);
                kind.hash(h);
                progress.hash(h);
            }
            Constructor::Ready { kind } => {
                h.tag(2);
                kind.hash(h);
            }
        }
    }
}

/// What a player's constructors are told to do (`Command::Construction`): the commands of
/// `factions/gdo/structures/deployment_center.md > ObjectInterfaceState[DeploymentCenter]` and
/// `factions/gdo/economy.md > ObjectInterfaceState[ExtractionFacility]` that reach the game. The menus themselves
/// (BuildMenu, AwaitingPlacement) are the client's.
#[derive(Copy, Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum ConstructionCommand {
    /// A BuildMenu entry of an idle constructor (the Extraction Facility's `Q: Build Extraction Plate`): "deducts"
    /// the building's cost and "sets CurrentConstruction". Only for a building on the constructor's `Constructs`
    /// list, by an idle constructor, while the bank covers the cost.
    Construct(ObjectKind),
    /// `X: Cancel Construction`: "refunds full cost, clears CurrentConstruction".
    CancelConstruction,
    /// `X: Cancel Ready Building` (the Extraction Facility's `X: Cancel Ready Plate`): "refunds 75% cost (rounded
    /// down), clears ReadyToPlace".
    CancelReady,
    /// `AwaitingPlacement > Left-click`: "places building at the ghost's position and rotation, clears
    /// ReadyToPlace": the ready building, with the top-left tile of its footprint at `origin`, turned by
    /// `orientation` (`core/entities.md > Structure Type`; optional in files, unturned by default). Only where the
    /// placement check allows it when the command applies; one constructor places one building.
    Place {
        origin: TilePos,
        #[serde(default, skip_serializing_if = "BuildingOrientation::is_default")]
        orientation: BuildingOrientation,
    },
    /// `factions/colonists/economy.md > Warp In > Warp In Command` (plan decision T22, `crate::transform`): "Left-click
    /// confirms the footprint and begins the transformation. The Beacon is consumed and begins transforming in place
    /// into the selected building": a `kind` from the Beacon's Constructs List, turned by `orientation` (optional in
    /// files, unturned by default). No position: the footprint follows from the Beacon and the orientation
    /// (`crate::transform::warp_in_footprint`, design change 0121). One Beacon, not transforming, while the bank
    /// covers the cost, which it pays at once, and where the placement check allows the Warp In when the command
    /// applies (`crate::placement::Method::WarpIn`: every tile Visible, every tile but the Beacon's own clear).
    WarpIn {
        kind: ObjectKind,
        #[serde(default, skip_serializing_if = "BuildingOrientation::is_default")]
        orientation: BuildingOrientation,
    },
    /// `factions/colonists/economy.md > Warp In > Cancel Warp In` (and slice M3-F's Cancel Deployment, both on X; plan
    /// decision T22): every transforming object named turns back into what it was, and its owner gets back 75% of what
    /// it paid, rounded down.
    CancelTransform,
}

/// What a constructor of kind `constructor` can build: its `Constructs` list, empty for every other kind.
pub fn recipes(constructor: ObjectKind) -> &'static [Recipe] {
    constructor.def().structure.as_ref().map_or(&[], |s| s.constructs)
}

/// The entry for `kind` on the `Constructs` list of a constructor of kind `constructor`, if `kind` is on it.
pub fn recipe(constructor: ObjectKind, kind: ObjectKind) -> Option<&'static Recipe> {
    recipes(constructor).iter().find(|r| r.kind == kind)
}

/// The state a new object of `kind` starts with: idle for a constructor, `None` for everything else.
pub(crate) fn constructor_for(kind: ObjectKind) -> Option<Constructor> {
    (!recipes(kind).is_empty()).then_some(Constructor::Idle)
}

/// Why `o` can't start constructing `kind` while its owner's bank holds `crystals`, if it can't; else the recipe it
/// would follow. In this order: `kind` isn't on its `Constructs` list ([`Reject::CannotConstruct`]), it constructs
/// or holds a building ready ([`Reject::ConstructorBusy`]: "Other build options unavailable"), the bank doesn't
/// cover the cost ([`Reject::InsufficientCrystals`]).
pub(crate) fn can_start(o: &Object, kind: ObjectKind, crystals: u32) -> Result<&'static Recipe, Reject> {
    let recipe = recipe(o.kind, kind).ok_or(Reject::CannotConstruct(kind))?;
    if !o.constructor.is_some_and(|c| c.is_idle()) {
        return Err(Reject::ConstructorBusy(o.id));
    }
    if recipe.cost > crystals {
        return Err(Reject::InsufficientCrystals);
    }
    Ok(recipe)
}

/// What cancelling `o`'s construction or its ready building gives back (`factions/gdo/structures/deployment_center.md >
/// Cancellation`, `factions/gdo/economy.md > ExtractionFacility > Cancellation`): the full cost while constructing, 75%
/// rounded down once ready ([`Refund`]); nothing for an idle constructor, or for a building that isn't on its list
/// (only a save edited by hand holds one).
pub(crate) fn refund(o: &Object) -> u32 {
    let (kind, refund) = match o.constructor {
        Some(Constructor::Constructing { kind, .. }) => (kind, Refund::Full),
        Some(Constructor::Ready { kind }) => (kind, Refund::Ready),
        Some(Constructor::Idle) | None => return 0,
    };
    recipe(o.kind, kind).map_or(0, |r| refund.of(r.cost))
}

/// The constructors' part of the economy phase (module docs; `crate::economy::step`), for the objects from before
/// this tick (`first_new`) in id order: each construction, one that a command of this tick started included (its
/// first frame), advances by its constructor's speed for its owner's `power`, and one that reaches its recipe's
/// frames is ready to place (`SimEvent::ConstructionCompleted`). A constructor without an owner builds nothing.
pub(crate) fn step(
    objects: &mut BTreeMap<ObjectId, Object>,
    power: &BTreeMap<PlayerId, PowerSummary>,
    first_new: ObjectId,
    events: &mut Vec<SimEvent>,
) {
    for o in objects.range_mut(..first_new).map(|(_, o)| o) {
        let (Some(Constructor::Constructing { kind, progress }), Some(owner)) = (&mut o.constructor, o.owner) else {
            continue;
        };
        let kind = *kind;
        let Some(recipe) = recipe(o.kind, kind) else {
            continue;
        };
        // Design change 0089: the Deployment Center at full speed, the Extraction Facility at the power ratio.
        let speed = power.get(&owner).copied().unwrap_or_default().speed(o.kind);
        if advance(progress, speed, recipe) {
            o.constructor = Some(Constructor::Ready { kind });
            events.push(SimEvent::ConstructionCompleted {
                constructor: o.id,
                kind,
            });
        }
    }
}

/// One tick of a construction of `recipe` at `speed` (in [`RATIO_ONE`]s): whether it is done. Saturating: the
/// recipes are short enough for their progress to fit (`ObjectDef` consistency tests), a hand-edited save needn't be.
fn advance(progress: &mut u32, speed: u32, recipe: &Recipe) -> bool {
    *progress = progress.saturating_add(speed);
    *progress >= recipe.frames.saturating_mul(RATIO_ONE)
}

#[cfg(test)]
mod tests;
