//! Movement and collision: how units steer, pass and make way for each other, and when a unit that gets no closer is
//! stuck, looks for another route or gives up (`systems/units.md > UnitCollision`,
//! `systems/controls/command_behaviors.md > MovingToLocation`; the implementation's `docs/movement.md`, 5.6 to 6.6).
//! Used by `crate::mover` and `crate::movement`.

use crate::units::Tick;

tunables! {
    /// The tunables of movement (module docs).
    Movement;

    // Steering (`crate::mover`, `crate::movement`).

    /// `docs/movement.md > 5.6`: a unit runs its staggered checks in the ticks where `(tick + id) % stagger == 0`.
    /// Placeholder (not in the design): the movement implementation's (slice M1-D); a quarter of the units check in
    /// each tick, and each unit four times a second.
    stagger: Tick = 4,

    /// `docs/movement.md > 6.3`: a unit that found no way forward for this many ticks in a row counts as stationary
    /// for the others. Placeholder (not in the design): the movement implementation's (slice M1-D), three quarters of a
    /// second.
    jammed_ticks: u16 = 12,

    /// `docs/movement.md > 6.3`, rule 4: a switch of `avoid_side` commits for this many ticks. Placeholder (not in the
    /// design): the movement implementation's (slice M1-D), a second, so that a unit doesn't swap sides at every
    /// contact.
    avoid_commit_ticks: Tick = 16,

    /// `systems/controls/command_behaviors.md > MovingToLocation > Occupied target location` (design changes 0026 and
    /// 0169; a placeholder to tune): a unit stuck within 3 grid units of its destination, blocked by units that aren't
    /// moving, counts as arrived (`docs/movement.md > 6.5`, rule 1). The one radius of both behaviours:
    /// `systems/controls/command_behaviors.md > MovingToObject`'s "Blocked near the target" is the same rule (design
    /// changes 0033 and 0169). It was 2 until design change 0169.
    occupied_arrival_gu: f32 = 3.0,

    /// `docs/movement.md > 6.3`, rule 4: from this share of its move along the blocker's face on, a unit goes around on
    /// the side it leans to; below it the contact is head-on and `Mover::avoid_side` decides. Placeholder (not in the
    /// design): the movement implementation's (slice M1-D).
    lean: f32 = 0.3,

    /// The detour round a row of its owner's firing units passes the row's box at this distance, space units, so that
    /// the leg after it clears the row's corner (`movement::round`). Placeholder (not in the design): slice M3-G2's.
    row_clearance_su: f32 = 2.0,

    /// How fast an idle unit moves off a unit it overlaps, space units per tick (the design leaves it to the
    /// implementation: "minimal, just until clear"). **Placeholder**, unspecified in the design.
    separate_speed_su: f32 = 2.0,

    // Lane seeking (`docs/movement.md > 6.3`, `movement::collide::lanes`; slice M1-G1). The distances in the unit's own
    // size because a Guard among Guards needs more room than a Peacekeeper among Peacekeepers.

    /// (G) Lane seeking (`docs/movement.md > 6.3`): how far ahead a unit looks for units to pass, in its own lengths.
    /// Placeholder (not in the design): 96 su for a Peacekeeper: at 4 su per tick each way, two of them have 12 ticks
    /// to step out of each other's way.
    lane_look: f32 = 4.0,

    /// (G) Lane seeking: a unit steps towards a clear lane at most this far to either side, in its own widths, and
    /// looks no farther for units in its way; with none that near it settles for the best gap between them.
    /// Placeholder (not in the design): slice M1-G1's.
    lane_reach: f32 = 3.0,

    /// (G) Lane seeking: a unit walks around a group of idle units that would make way for it only if a clear lane
    /// is this near, in its own widths; else it walks through the group, between its units, and they step aside.
    /// Placeholder (not in the design): slice M1-G1's.
    lane_around_idle: f32 = 4.0 / 3.0,

    /// (G) Lane seeking: the clearance a unit keeps from the units it passes. It is what spreads two dense groups that
    /// meet: a unit squeezed between two oncoming lanes takes the middle, and the units on the outside, which have
    /// room, move out until this much is free. Placeholder (not in the design): slice M1-G1's.
    lane_margin_su: f32 = 2.0,

    /// (G) Lane seeking: when the clear lanes to the left and to the right are this close to equally far, the unit
    /// takes its `Mover::avoid_side`: Right at first, so head-on units pass like traffic. Placeholder (not in the
    /// design): slice M1-G1's.
    lane_tie_su: f32 = 4.0,

    /// (G) Lane seeking: of two gaps between lanes, the unit takes the farther one only if it is wider by this share
    /// of the extra way, so it doesn't hop from gap to gap as the units around it shift. Placeholder (not in the
    /// design): slice M1-G1's.
    lane_near: f32 = 0.5,

    /// (G) Lane seeking: a unit counts as oncoming when its heading is against the mover's by more than this cosine
    /// (within 60° of straight at it). Placeholder (not in the design): slice M1-G1's.
    lane_oncoming_cos: f32 = -0.5,

    // Friendly yielding (`systems/units.md > UnitCollision > Ground Collision > Friendly yielding`, design change 0022;
    // `docs/movement.md > 6.6`, `mover::sidestep`, `movement::yielding`; slice M1-Y).

    /// Friendly yielding (0022): an idle unit whose centre is within this many space units of the line a mover walks
    /// along stands "in the middle" of its way, and steps to the side the mover isn't going round on; farther out it
    /// steps to the side it is already on. Placeholder (not in the design): slice M1-Y's.
    yield_centred_su: f32 = 4.0,

    /// `systems/units.md > UnitCollision > Ground Collision > Friendly yielding` (design change 0022): the longest
    /// "one short sidestep" of a yielding unit, in widths of its own silhouette. A placeholder, TBD in the design
    /// (0022, 0023: "sidestep distance"). The unit steps only as far as the mover's lane needs, up to this.
    yield_sidestep_widths: f32 = 1.0,

    /// Friendly yielding (0022): the shortest sidestep, space units. Placeholder (not in the design): one, so that a
    /// request always moves the unit a little.
    yield_min_sidestep_su: f32 = 1.0,

    /// Friendly yielding (0022) in a dense idle group: a sidestep blocked by another idle unit of the owner passes the
    /// request on, at most this many units deep; beyond that the first unit stays an ordinary blocker
    /// (`docs/movement.md > 6.6`). Placeholder (not in the design): slice M1-Y's.
    yield_chain_depth: u8 = 4,

// Stuck units and their repaths (`systems/controls/command_behaviors.md > MovingToLocation > Failure`, design change
// 0024; `docs/movement.md > 5.6`, `6.5`; `mover::schedule`).

    /// `systems/controls/command_behaviors.md > MovingToLocation > Failure` (design change 0024; a placeholder to
    /// tune): a stuck unit looks for another route at most 3 times per order leg (`docs/movement.md > 5.6`).
    max_repaths: u8 = 3,

    /// `systems/controls/command_behaviors.md > MovingToLocation > Failure` (design change 0024; a placeholder to
    /// tune): a unit is stuck after 1.5 s (24 frames) without getting any closer to its target (`docs/movement.md >
    /// 6.5`).
    stuck_ticks: Tick = 24,

    /// `systems/controls/command_behaviors.md > MovingToLocation > Failure` (design change 0024; placeholders to tune):
    /// a stuck unit waits 0.5 s, 1 s and 2 s before its first, second and third attempt at another route
    /// (`docs/movement.md > 5.6`). An attempt after the third, if [`Movement::max_repaths`] allows more, waits as long
    /// as the third.
    repath_waits: [Tick; 3] = [8, 16, 32],

    /// `docs/movement.md > 6.5`: a stuck unit that isn't in contact with a unit this very tick waits this many ticks
    /// more to meet its blocker again before it recovers without knowing it. Placeholder (not in the design): the
    /// movement implementation's (slice M1-D), half a second.
    stuck_grace_ticks: Tick = 8,

    // (M5-V1) Oriented Silhouettes (plan decision T78; `systems/units.md > UnitCollision > Collision Response
    // (Un-sticking)`: "Overlap caused by the rotation itself (a corner sweeping into a wall) follows the same
    // principle: the blocked part is discarded and the unit never ends up overlapping. The exact geometric method is a
    // technical choice"; `movement::collide::turn`).

    /// (M5-V1) T78: "If blocked, it is halved up to five times, the free part is kept": a blocked turn of a Silhouette
    /// that turns with its facing, or a move up to an oriented shape that the sweep's contact leaves inside it by a
    /// rounding, is bisected this many times, so the unit keeps all but at most a 32nd of the free part. Placeholder
    /// (not in the design: its "exact geometric method is a technical choice").
    step_halvings: u32 = 5,

    /// (M5-V1) How far the corners of a Silhouette that turns with its facing move at most between two tries of its
    /// turn: 4 su, a 16th of a tile. A turn is tried in parts no longer, so between two tries a corner can cut through
    /// at most the tip of another shape's corner, about half this deep, and it never ends inside one (each try is the
    /// predicates of invariants I1 and I2). A spin of the Hand of Heaven (64x40, its corners 37.7 su from its centre)
    /// at its SpinRate, 1/3 radian a tick, takes 4 tries. Placeholder (not in the design: a technical bound).
    turn_try_su: f32 = 4.0,

    // (M5-V3) Tracked movement (`movement::tracked`; `systems/units.md > SpeedTurnRadiusMovement`, design change 0417).

    /// (M5-V3) How many speeds the corner choice of a SpeedTurnRadiusMovement unit compares (`movement::tracked`): from
    /// a stop up to the leg's top speed, evenly spaced, both ends among them. It takes each turn at the one that makes
    /// it soonest, of those whose arc fits. Placeholder (not in the design, which leaves the method open: "the
    /// implementation may use pre-computed values"): 32 steps, 0.16 su a tick apart for the Hand of Heaven.
    corner_speeds: u32 = 32,

    /// (M5-V3) The most a SpeedTurnRadiusMovement unit's arc turns between two of the poses at which the corner choice
    /// tests it against the nav grid: 22.5°. Placeholder (not in the design: a technical bound): between two tests the
    /// corners of the Hand of Heaven's Silhouette (37.7 su from its centre) bulge out of both by less than 1 su.
    arc_test_rad: f32 = core::f32::consts::FRAC_PI_8,

    /// (M5-V3) How far a SpeedTurnRadiusMovement unit's heading may miss its steering waypoint, space units, and be
    /// held as it is: at full speed the tread law turns it not at all, so it doesn't slow down to turn for less. It
    /// still passes within a step of the waypoint. And how near to where it passes a waypoint it counts as there,
    /// braking for the corner at it (`Tracked::ahead`). Placeholder (not in the design: a technical bound).
    tracked_miss_su: f32 = 1.0,
}

impl Movement {
    /// Friendly yielding (0022): a sidestep ends this many ticks after the time its distance takes, however far the
    /// unit got: time for the units it asked in turn to make room, one tick per link of the chain (so not a tunable of
    /// its own: [`Movement::yield_chain_depth`]'s).
    pub const fn yield_wait_ticks(&self) -> u8 {
        self.yield_chain_depth
    }

    /// The wait before a leg's attempt at another route, `used` attempts having been made before it
    /// ([`Movement::repath_waits`]; one beyond the third waits as long as the third).
    pub const fn repath_wait(&self, used: usize) -> Tick {
        let last = self.repath_waits.len() - 1;
        self.repath_waits[if used < last { used } else { last }]
    }

    /// 0024's "about 7 s worst case": a unit that never gets on is stuck after [`Movement::stuck_ticks`], waits out
    /// the [`Movement::repath_waits`] of its [`Movement::max_repaths`] attempts, and gives up once it is still stuck a
    /// whole `stuck_ticks` after its last attempt: 104 ticks, 6.5 s, after its last progress (`docs/movement.md >
    /// 5.6`). Not a tunable of its own: it follows from those.
    pub const fn give_up_ticks(&self) -> Tick {
        let mut ticks = 2 * self.stuck_ticks;
        let mut used = 0;
        while used < self.max_repaths as usize {
            ticks += self.repath_wait(used);
            used += 1;
        }
        ticks
    }

    /// From this many ticks without progress on, a unit on its way counts as held up (`Mover::is_held_up`, for
    /// overlays): half of [`Movement::stuck_ticks`], so not a tunable of its own.
    pub const fn half_stuck_ticks(&self) -> Tick {
        self.stuck_ticks / 2
    }
}

// The built-in worst case is the design's (0024: "about 7 s").
const _: () = assert!(BUILTIN.give_up_ticks() == 104);
