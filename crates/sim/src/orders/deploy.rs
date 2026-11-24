//! A Drone's walk to the site of its Beacon — `factions/colonists/economy.md > Beacon Deployment > Deploy Command`,
//! design change 0117 (slice M3-F; the whole deployment is `crate::deploy`). "The Drone walks to the centre of the
//! Beacon's footprint and starts deploying when it gets there." The walk is a move to that point; the behaviour only
//! decides when it is over, and never completes the order itself: the step right after phase 5a does, by starting the
//! deployment or by failing the order (`Sim::step_deployments`), with the events and the changes of kind a behaviour
//! may not make.

use crate::deploy::{self, on_site};
use crate::mover::{JAMMED_TICKS, MoveGoal, MoveStatus};
use crate::units::{Tick, TilePos};

use super::state::Deploy;
use super::world::Unit;

/// A Deploy order to the site at `origin` starts (`orders::start`): `unit` heads for the footprint's centre.
pub(super) fn begin(origin: TilePos, unit: &mut Unit, tick: Tick) -> Deploy {
    unit.mover
        .go_to(MoveGoal::Point(deploy::site(origin).center()), None, tick);
    Deploy::default()
}

/// One tick of the walk to the site at `origin`. It is over once the move ended, whatever its outcome: at the centre,
/// at the reachable point nearest to it, or stopped within 3 grid units of it behind units that won't move
/// (`MoveStatus::Arrived`), or given up (`Failed`); the arrival then sees whether the Drone overlaps the footprint. It
/// is also over when the Drone, still on its way, is "stopped by a blocker on the way while already overlapping the
/// footprint": it found no way forward for [`JAMMED_TICKS`] in a row, the movement's own notion of a unit at a
/// standstill (`docs/movement.md > 6.3`). Never complete (module docs).
pub(super) fn run(origin: TilePos, state: &mut Deploy, unit: &mut Unit) -> bool {
    let ended = unit.mover.status != MoveStatus::Moving;
    let stuck_on_site =
        || unit.mover.blocked_ticks >= JAMMED_TICKS && on_site(unit.kind, unit.pos, &deploy::site(origin));
    if ended || stuck_on_site() {
        state.arrived = true;
    }
    false
}
