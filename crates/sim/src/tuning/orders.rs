//! Orders and behaviours (`systems/controls/behaviors.md`): the leashes, Proximity, Patrol's failed legs, the
//! auto-targeting's retries and priorities, and when the reach rule judges an enemy again (`crate::orders`).

use crate::math;
use crate::units::{SU_PER_GU, Tick};

/// `systems/controls/behaviors.md > BaseAutoTargeting > Global Constant: IdleLeashDistance`: 4 grid units. An idle
/// unit farther than this from where it acquired its target disengages and returns.
pub const IDLE_LEASH_GU: f32 = 4.0;

/// `systems/controls/behaviors.md > AttackMovingToLocation > Global Constant: AttackMoveLeashDistance`: 6 grid
/// units, measured from the leg's PathReference. It limits chasing only (design change 0180): a unit won't move
/// farther than this from the PathReference to chase a target out of weapon range (`orders::travel::march`).
pub const ATTACK_MOVE_LEASH_GU: f32 = 6.0;

/// `systems/controls/behaviors.md > MovingToObject > Proximity` (design change 0032): "within proximity" is a gap
/// of at most 8 space units between the unit's Silhouette and the object's shape, edge to edge.
pub const PROXIMITY_SU: f32 = 8.0;

/// `systems/controls/behaviors.md > Patrolling > Failed legs` (design change 0037): this many failed legs in a row
/// end a Patrol.
pub const PATROL_FAILED_LEGS: u8 = 2;

/// An idle unit that acquired an enemy by itself and couldn't get to it doesn't acquire that same enemy again for this
/// many ticks, so that it doesn't set off for it again every tick (on an AttackMove or a Patrol, a chase given up is a
/// rejection by the reach rule instead: `orders::reach`, design change 0230):
/// `systems/controls/behaviors.md > BaseAutoTargeting > Unreachable auto-targets` (design change 0181, "for now":
/// approach, give up, retry after 32 ticks). 2 s.
pub const GIVE_UP_RETRY_TICKS: Tick = 32;

/// Design change 0230 ("the enemy moves a meaningful distance"): an enemy that has moved this far from where it was
/// judged is judged again, space units. A placeholder (the Director's): 1 grid unit.
pub const REJUDGE_MOVE_SU: f32 = SU_PER_GU;

/// Design change 0243 (answering question Q0074: "once the unit has moved about 1 GU (placeholder) from where it
/// judged that enemy"): a rejection by the reach rule is judged again once the unit stands this far from where it
/// judged, space units. A placeholder of the design's, "to be revisited if too expensive"
/// (`tests/attack_move_perf.rs` measures it): 1 grid unit.
pub const REJUDGE_SELF_SU: f32 = SU_PER_GU;

/// Design change 0179 (`systems/controls/behaviors.md > TurretAutonomousScanning > Target Selection Priority` 2):
/// the rotation a target needs is compared in steps of 45° (a placeholder), radians.
pub const ROTATION_STEP: f32 = math::PI / 4.0;
