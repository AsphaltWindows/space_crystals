//! Orders and behaviours (`systems/controls/behaviors.md`): the leashes, Proximity, Patrol's failed legs, the
//! auto-targeting's retries and priorities, and when the reach rule judges an enemy again (`crate::orders`).

use crate::math;
use crate::units::{SU_PER_GU, Tick};

tunables! {
    /// The tunables of orders (module docs).
    Orders;

    /// `systems/controls/behaviors.md > BaseAutoTargeting > Global Constant: IdleLeashDistance`: 4 grid units. An idle
    /// unit farther than this from where it acquired its target disengages and returns.
    idle_leash_gu: f32 = 4.0,

    /// `systems/controls/command_behaviors.md > AttackMovingToLocation > Global Constant: AttackMoveLeashDistance`: 6
    /// grid units, measured from the leg's PathReference. It limits chasing only (design change 0180): a unit won't
    /// move farther than this from the PathReference to chase a target out of weapon range (`orders::travel::march`).
    attack_move_leash_gu: f32 = 6.0,

    /// `systems/controls/command_behaviors.md > MovingToObject > Proximity` (design change 0032): "within proximity" is
    /// a gap of at most 8 space units between the unit's Silhouette and the object's shape, edge to edge.
    proximity_su: f32 = 8.0,

    /// `systems/controls/command_behaviors.md > Patrolling > Failed legs` (design change 0037): this many failed legs
    /// in a row end a Patrol.
    patrol_failed_legs: u8 = 2,

    /// An idle unit that acquired an enemy by itself and couldn't get to it doesn't acquire that same enemy again for
    /// this many ticks, so that it doesn't set off for it again every tick (on an AttackMove or a Patrol, a chase given
    /// up is a rejection by the reach rule instead: `orders::reach`, design change 0230):
    /// `systems/controls/behaviors.md > BaseAutoTargeting > Unreachable auto-targets` (design change 0181, "for now":
    /// approach, give up, retry after 32 ticks). 2 s.
    give_up_retry_ticks: Tick = 32,

    /// Design change 0230 ("the enemy moves a meaningful distance"): an enemy that has moved this far from where it was
    /// judged is judged again, space units. A placeholder (the Director's): 1 grid unit.
    rejudge_move_su: f32 = SU_PER_GU,

    /// Design change 0243 (answering question Q0074: "once the unit has moved about 1 GU (placeholder) from where it
    /// judged that enemy"): a rejection by the reach rule is judged again once the unit stands this far from where it
    /// judged, space units. A placeholder of the design's, "to be revisited if too expensive"
    /// (`tests/attack_move_perf.rs` measures it): 1 grid unit.
    rejudge_self_su: f32 = SU_PER_GU,

    /// Design change 0179 (`systems/controls/behaviors.md > TurretAutonomousScanning > Target Selection Priority` 2):
    /// the rotation a target needs is compared in steps of 45° (a placeholder), radians.
    rotation_step: f32 = math::PI / 4.0,

    /// (M6-S6a) Design change 0454 (answering Q0179; `systems/controls/command_behaviors.md > AttackingObject > Target
    /// inside MinRange`: "to **MinRange + 1 grid unit** (never beyond Range), every unit"; it replaced Q0054's middle
    /// of the band): how far beyond its MinRange a unit backs off from a target inside it, grid units
    /// (`orders::engage::too_close`).
    back_off_gu: f32 = 1.0,

    /// (M6-S6a) Design change 0454 (`systems/controls/command_behaviors.md > AttackingObject > Can't aim from where it
    /// stands`: "How the spot is chosen is the Director's call"): a unit that can't bring its weapon to bear from where
    /// it stands looks for a spot it can on rings round it this far apart, at points this far apart along each, space
    /// units (`orders::aim`). Placeholder (not in the design: how finely the unit looks for its spot).
    aim_spot_step_su: f32 = 16.0,

    /// (M6-S6a) Design change 0454: how far from the unit it looks for that spot at most, grid units (`orders::aim`).
    /// Placeholder (not in the design: how far the unit looks).
    aim_spot_radius_gu: f32 = 4.0,

    /// (M6-S6a) Design change 0454: a unit that can't turn in place, with a narrow turret (the Assault Bike, the
    /// Raptor), can't come round onto a target that lies inside either of the circles it turns on or less than this
    /// beyond their rims, space units (`orders::aim`, whose margin adds the weapon's MinRange). Placeholder (not in the
    /// design: how far out a wheeled vehicle needs a target to point it into its arc): 1 grid unit.
    aim_turn_margin_su: f32 = SU_PER_GU,

    /// (M6-S6a) Design change 0454 ("**Explicit Attack**: it **keeps trying to get into a position from which it can
    /// attack** the target [...] It doesn't wait in place"): a unit on an order's own target that finds no spot within
    /// `aim_spot_radius_gu` looks this far from it at most, at the tiles' centres, its way there however long, grid
    /// units (`orders::aim`). Placeholder (not in the design: how far the unit looks once no spot is near).
    aim_spot_far_gu: f32 = 16.0,
}
