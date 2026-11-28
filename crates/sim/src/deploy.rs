//! Beacon Deployment (slice M3-F; `factions/colonists/economy.md > Beacon Deployment`, plan decisions T15, T16 and
//! T17): a Drone walks to a site and transforms in place into a Beacon, over 48 frames.
//!
//! - **The order** (`OrderKind::Deploy { origin }`, T17): the Beacon's 2 × 2 footprint with its top-left tile at
//!   `origin` ([`site`]; the Beacon's SymmetryType is AAAA, so it is never turned). It goes to one Drone: a Drone
//!   becomes the Beacon, so "each takes exactly one placement" (`systems/controls/interface.md > Shift while
//!   placing`), and with Shift it is queued like any order (`systems/controls/commands.md > Unit Command`).
//! - **The gate** (`Sim::deploy_check`; T16's `Mode::Known`, design change 0120: "The ghost judges by what the player
//!   knows"): the placement check of `core/entities.md > Placement Validation > Worker-Built Structures` on what the
//!   player knows: its own objects, the others it sees, and its records of the structures and resources out of its
//!   sight. Units hidden in the fog don't count, and never-explored tiles pass: "The order can be given even where the
//!   area isn't visible; the real check happens on arrival." The player's pending Deploys reserve their sites against
//!   its further placements, Deploy and Warp In alike (`core/entities.md > Lingering build ghosts`, design change 0125;
//!   `TileProblem::Reserved`). They are derived from the order queues (`Sim::pending_builds`), never stored, so a
//!   site is free again once its order is gone, whatever ended it: the deployment started, a cancel, a new order in
//!   its place, a failure.
//! - **The walk** (`orders::build`, phase 5a; design change 0117): the Drone heads for the footprint's centre and is
//!   there once its move ends. "If it is stopped by a blocker on the way while already overlapping the footprint, it
//!   starts deploying from where it stands" (stopped: it found no way forward for
//!   `tuning::movement::Movement::jammed_ticks` in a row, the movement's own notion of a unit at a standstill); "if it
//!   stops without overlapping the footprint, the order fails".
//! - **The arrival** (`Sim::step_deployments`, right after phase 5a, before 5b claims the footprint; in id order, so
//!   that of two Drones arriving at overlapping sites in one tick the lower id deploys and the other finds its site
//!   taken): the Drone is on its site ([`on_site`]), and the check passes again, on the truth (`Mode::Arrival`: "No
//!   visibility requirement on arrival"), the Drone left out (`Method::WorkerBuilt`; design change 0118: "the building
//!   worker itself (e.g. the deploying Drone) doesn't count"). Then the Drone becomes the Beacon on its footprint,
//!   wherever it stood ("The Beacon always appears on the footprint the ghost showed"), the same object
//!   (`Sim::change_kind`; design change 0109: "the footprint is claimed immediately"), transforming from the Drone
//!   (`crate::Transform`, with no `from_placement`) for [`DEPLOYMENT_FRAMES`], its first in this tick
//!   (`docs/architecture/economy.md > Economy > Recipe timing`). Otherwise the order fails: the Drone stands, its next queued order
//!   starts in the next tick, if it has one, and `SimEvent::DeployFailed` says why.
//! - **The transformation** is the Warp In's (`crate::transform`): its HP interpolates from the Drone's 20 to the
//!   Beacon's 120, damage staying taken (design change 0110), and it completes one way
//!   (`factions/colonists/economy.md > Completion`). Its only command is Cancel Deployment,
//!   `ConstructionCommand::CancelTransform` (design change 0116), which turns it back into the Drone at the footprint's
//!   centre (0109), idle, with the fraction of its HP it had, at least 1 (0111).
//! - **Beacon Capacity** (design change 0112): the Drone's 1 is freed as it starts, a Beacon costing none, and the
//!   Beacon's 5 comes only with completion: a Beacon under deployment provides none ([`Object::is_deploying`],
//!   `economy::army`). A cancel takes the 1 back, even over capacity, and is never refused.
//! - **Defeat** (design change 0113): a Beacon under deployment is a structure, so it keeps its owner in the game; a
//!   cancel that leaves the player no structure defeats them in that tick (`economy::defeat`).
//! - **The load** (design change 0114, `factions/colonists/economy.md > Deploying with a load`): kept by the
//!   transformation (`Transform::load`), lost at completion, carried again by the Drone after a cancel.

use glam::Vec2;

/// A Drone's deployment as a transformation's recipe (`crate::Transform::new`), its def's (`UnitDef::deploy`): into a
/// Beacon, in [`DEPLOYMENT_FRAMES`], for no Space Crystals. A Deploy order is the Drone's, the one unit that deploys
/// so far: its site is a Beacon's ([`site`]).
pub use crate::defs::BEACON_DEPLOYMENT;
use crate::defs::{Category, ObjectKind, Recipe};
use crate::nav::TileRect;
use crate::object::{Object, Placement};
use crate::paths::box_overlaps_rect;
use crate::units::TilePos;

/// `factions/colonists/economy.md > Beacon Deployment > Deployment Duration`, as the Drone's def gives it ("48 frames
/// (3 seconds)"): its name for the tests that count the frames of a deployment.
pub const DEPLOYMENT_FRAMES: u32 = BEACON_DEPLOYMENT.frames;

/// What a unit of `kind` deploys into, its def's (`UnitDef::deploy`): the Drone's Beacon
/// (`factions/colonists/units/drone.md > Deploying`); `None` for a kind that doesn't deploy.
pub fn deployment(kind: ObjectKind) -> Option<&'static Recipe> {
    kind.def().unit.as_ref()?.deploy.as_ref()
}

/// Whether a unit of `kind` takes a Deploy order: one that deploys ([`deployment`]), the Drone so far.
pub fn deploys(kind: ObjectKind) -> bool {
    deployment(kind).is_some()
}

/// The footprint of the Beacon that a Deploy with the top-left tile `origin` puts down
/// (`factions/colonists/economy.md > Placement`: "the Beacon's 2x2 footprint"), unturned. The ghost, the gate, the walk
/// and the arrival all take it from here.
pub fn site(origin: TilePos) -> Placement {
    Placement::new(origin, BEACON_DEPLOYMENT.kind.def().footprint)
}

/// Design change 0117: whether a unit of `kind` whose centre is at `pos` overlaps the footprint `site`: its box
/// overlaps a tile of it, strictly, as the placement check counts a unit on a footprint (`docs/movement.md > 3`). Never
/// for a kind without a box.
pub fn on_site(kind: ObjectKind, pos: Vec2, site: &Placement) -> bool {
    kind.def()
        .half_extents()
        .is_some_and(|half| box_overlaps_rect(pos, half, TileRect::new(site.origin, site.w, site.h)))
}

impl Object {
    /// Whether it is a Beacon under deployment (`factions/colonists/economy.md > A Beacon from the first frame`): a
    /// structure transforming from a unit, the Drone it was. A building being warped in transforms from a Beacon.
    pub fn is_deploying(&self) -> bool {
        self.transform
            .as_ref()
            .is_some_and(|t| t.from.def().category == Category::Unit)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::units::SU_PER_GU;

    /// `factions/colonists/economy.md > Deployment Duration` and `factions/colonists/economy.md > Beacon > Size - 2x2
    /// grid units`: the recipe and the site; only the Drone deploys.
    #[test]
    fn a_drone_deploys_into_a_two_by_two_beacon_in_48_frames() {
        assert_eq!(
            (BEACON_DEPLOYMENT.kind, BEACON_DEPLOYMENT.cost, BEACON_DEPLOYMENT.frames),
            (ObjectKind::Beacon, 0, 48)
        );
        for &kind in ObjectKind::ALL {
            assert_eq!(deploys(kind), kind == ObjectKind::Drone, "{kind:?}");
        }
        let site = site(TilePos::new(3, 5));
        assert_eq!((site.origin, site.w, site.h), (TilePos::new(3, 5), 2, 2));
        assert_eq!(site.orientation(), Default::default());
        assert_eq!(site.center(), Vec2::new(4.0, 6.0) * SU_PER_GU);
    }

    /// Design change 0117: a Drone (36 × 36 space units) overlaps the footprint while its box covers some of a tile,
    /// strictly: touching an edge isn't overlapping.
    #[test]
    fn on_site_is_a_strict_overlap_of_the_box() {
        let site = site(TilePos::new(2, 2));
        let (lo, hi) = (2.0 * SU_PER_GU, 4.0 * SU_PER_GU);
        let drone = |x: f32, y: f32| on_site(ObjectKind::Drone, Vec2::new(x, y), &site);
        assert!(drone(site.center().x, site.center().y));
        assert!(drone(lo - 17.5, site.center().y), "half a space unit in");
        assert!(!drone(lo - 18.0, site.center().y), "touching the west edge");
        assert!(!drone(hi + 18.0, hi + 18.0), "touching the south-east corner");
        assert!(drone(hi + 17.0, hi + 17.0));
        assert!(
            !on_site(ObjectKind::Beacon, site.center(), &site),
            "a structure has no box"
        );
    }
}
