//! A worker's walk to the site of what it builds (`core/entities.md > Placement Validation > Worker-Built
//! Structures`: "The worker pathfinds to the location and validates on arrival"; plan decision T51: one walk and one
//! arrival for every worker build): a Drone's Deploy (slice M3-F, `crate::deploy`; design change 0117: "The Drone walks
//! to the centre of the Beacon's footprint and starts deploying when it gets there") and a Build order (slice M4-E,
//! `OrderKind::Build`: an Agent's Tunnel, `crate::construction::site`). The walk is a move to the footprint's centre;
//! the behaviour only decides when it is over, and never completes the order itself: the step right after phase 5a
//! does (`Sim::step_deployments`), by starting the deployment or the site, or by failing the order, with the events
//! and the changes of the game that a behaviour may not make.
//!
//! The sites of these orders, current and queued, are the lingering build ghosts (`core/entities.md > Placement
//! Validation > Lingering build ghosts`; [`PendingBuild`], `Sim::pending_builds`): [`build_site`] says what an order
//! puts down, and where.

use crate::construction::site::shares_builds;
use crate::defs::ObjectKind;
use crate::deploy::{self, BEACON_DEPLOYMENT, on_site};
use crate::ids::ObjectId;
use crate::mover::{MoveGoal, MoveStatus};
use crate::object::Placement;
use crate::placement::footprint_size;
use crate::scenario::BuildingOrientation;
use crate::tuning::movement::JAMMED_TICKS;
use crate::units::{Tick, TilePos};

use super::state::SiteWalk;
use super::world::Unit;
use super::{OrderGroup, OrderKind};

/// A lingering build ghost (`core/entities.md > Placement Validation > Lingering build ghosts`): a worker's order to
/// build, current or queued, until the construction starts or the order goes. A group's Build is one ghost, however
/// many workers share it (plan decision T59, slice M5-E: "`pending_builds` gives one ghost per ticket").
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub struct PendingBuild {
    /// The worker ordered to build it: of a group's Build, the one with the lowest id that holds it.
    pub worker: ObjectId,
    /// What it will be: a Drone's Beacon, an Agent's Tunnel.
    pub kind: ObjectKind,
    /// Its footprint, turned as ordered.
    pub site: Placement,
    /// (M5-J2) Its order's group (`super::Order::group`): its command's, a new one with each command
    /// (`Sim::give_order`), so the order the ghosts were placed in (design change 0351: "in their order of placement",
    /// `crate::recruitment::preview`), until the counter wraps after 2³² commands.
    pub group: OrderGroup,
}

/// What a worker-build order puts down, and its footprint: a Deploy's Beacon ([`deploy::site`]), a Build's structure
/// turned as ordered ([`footprint`]). `None` for every other order, and for a Build of a kind without a footprint.
pub fn build_site(order: OrderKind) -> Option<(ObjectKind, Placement)> {
    match order {
        OrderKind::Deploy { origin } => Some((BEACON_DEPLOYMENT.kind, deploy::site(origin))),
        OrderKind::Build {
            kind,
            origin,
            orientation,
        } => Some((kind, footprint(kind, origin, orientation)?)),
        _ => None,
    }
}

/// The footprint of a structure of `kind` with its top-left tile at `origin`, turned by `orientation`
/// (`core/entities.md > Structure Type`): `None` for a kind without one.
pub fn footprint(kind: ObjectKind, origin: TilePos, orientation: BuildingOrientation) -> Option<Placement> {
    let (w, h) = footprint_size(kind, orientation)?;
    Some(Placement {
        origin,
        w,
        h,
        rotation: orientation.rotation,
        flip_h: orientation.flip_h,
        flip_v: orientation.flip_v,
    })
}

/// A worker-build order to `site` starts (`orders::start`): `unit` heads for the footprint's centre. A worker that
/// shares builds (`WorkerDef::shares_builds`, the Recruit; slice M5-E) stands instead, until the spots' step right
/// after this phase gives it its own spot beside the footprint and sends it there (`crate::spot_work`, plan decisions
/// T59 and T62: "each builder walks to its own spot", "so a group doesn't converge on one point"; design change 0339:
/// the part of the footprint closest to it, or the nearest free part along the wall).
pub(super) fn begin(site: &Placement, unit: &mut Unit, tick: Tick) -> SiteWalk {
    if !shares_builds(unit.kind) {
        unit.mover.go_to(MoveGoal::Point(site.center()), None, tick);
    }
    SiteWalk::default()
}

/// One tick of the walk to `site`. It is over once the move ended, whatever its outcome: at the centre, at the
/// reachable point nearest to it, or stopped within 3 grid units of it behind units that won't move
/// (`MoveStatus::Arrived`), or given up (`Failed`); the arrival then sees whether the worker overlaps the footprint
/// ([`on_site`]). It is also over when the worker, still on its way, is "stopped by a blocker on the way while already
/// overlapping the footprint" (design change 0117): it found no way forward for [`JAMMED_TICKS`] in a row, the
/// movement's own notion of a unit at a standstill (`docs/movement.md > 6.3`), or (M4-W3) it began the contact step
/// of design change 0323 round a unit in its way (`Mover::step_aside`), which would take it a whole width off the
/// footprint: until 0323 a blocked worker shuffled a step and stood. Never complete (module docs).
///
/// (M5-E) A worker that shares builds walks to its own spot, and its walk is over while it has one and its move there
/// has ended, whatever the outcome: the arrivals' step sees whether it stands within reach of the footprint's edge
/// (`sim/builds/shared.rs`), and the spots' step may send it on again (`crate::spot_work`).
pub(super) fn run(site: &Placement, state: &mut SiteWalk, unit: &mut Unit) -> bool {
    let ended = unit.mover.status != MoveStatus::Moving;
    if shares_builds(unit.kind) {
        state.arrived = state.spot.is_some() && ended;
        return false;
    }
    // (M4-W3) Or it began a contact step (design change 0323): a blocker stopped it, and the step, a whole width
    // sideways, would take it off the footprint it overlaps.
    let stopped = unit.mover.blocked_ticks >= JAMMED_TICKS || unit.mover.step_aside.is_some();
    let stuck_on_site = || stopped && on_site(unit.kind, unit.pos, site);
    if ended || stuck_on_site() {
        state.arrived = true;
    }
    false
}
