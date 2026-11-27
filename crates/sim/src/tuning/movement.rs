//! Movement and collision: how units steer, pass and make way for each other, and when a unit that gets no closer is
//! stuck, looks for another route or gives up (`systems/units.md > UnitCollision`,
//! `systems/controls/command_behaviors.md > MovingToLocation`; the implementation's `docs/movement.md`, 5.6 to 6.6).
//! Used by `crate::mover` and `crate::movement`.

use crate::units::Tick;

// Steering (`crate::mover`, `crate::movement`).

/// `docs/movement.md > 5.6`: a unit runs its staggered checks in the ticks where `(tick + id) % STAGGER == 0`.
/// Placeholder (not in the design): the movement implementation's (slice M1-D); a quarter of the units check in each
/// tick, and each unit four times a second.
pub const STAGGER: Tick = 4;

/// `docs/movement.md > 6.3`: a unit that found no way forward for this many ticks in a row counts as stationary
/// for the others. Placeholder (not in the design): the movement implementation's (slice M1-D), three quarters of a
/// second.
pub const JAMMED_TICKS: u16 = 12;

/// `docs/movement.md > 6.3`, rule 4: a switch of `avoid_side` commits for this many ticks. Placeholder (not in the
/// design): the movement implementation's (slice M1-D), a second, so that a unit doesn't swap sides at every contact.
pub const AVOID_COMMIT_TICKS: Tick = 16;

/// `systems/controls/command_behaviors.md > MovingToLocation > Occupied target location` (design changes 0026 and 0169;
/// a placeholder to tune): a unit stuck within 3 grid units of its destination, blocked by units that aren't moving,
/// counts as arrived (`docs/movement.md > 6.5`, rule 1). The one radius of both behaviours:
/// `systems/controls/command_behaviors.md > MovingToObject`'s "Blocked near the target" is the same rule (design
/// changes 0033 and 0169). It was 2 until design change 0169.
pub const OCCUPIED_ARRIVAL_GU: f32 = 3.0;

/// `docs/movement.md > 6.3`, rule 4: from this share of its move along the blocker's face on, a unit goes around on
/// the side it leans to; below it the contact is head-on and `Mover::avoid_side` decides. Placeholder (not in the
/// design): the movement implementation's (slice M1-D).
pub const LEAN: f32 = 0.3;

/// The detour round a row of its owner's firing units passes the row's box at this distance, space units, so that the
/// leg after it clears the row's corner (`movement::round`). Placeholder (not in the design): slice M3-G2's.
pub const ROW_CLEARANCE_SU: f32 = 2.0;

/// How fast an idle unit moves off a unit it overlaps, space units per tick (the design leaves it to the
/// implementation: "minimal, just until clear"). **Placeholder**, unspecified in the design.
pub const SEPARATE_SPEED_SU: f32 = 2.0;

// Lane seeking (`docs/movement.md > 6.3`, `movement::collide::lanes`; slice M1-G1). The distances in the unit's own
// size because a Guard among Guards needs more room than a Peacekeeper among Peacekeepers.

/// (G) Lane seeking (`docs/movement.md > 6.3`): how far ahead a unit looks for units to pass, in its own lengths.
/// Placeholder (not in the design): 96 su for a Peacekeeper: at 4 su per tick each way, two of them have 12 ticks to
/// step out of each other's way.
pub const LANE_LOOK: f32 = 4.0;

/// (G) Lane seeking: a unit steps towards a clear lane at most this far to either side, in its own widths, and
/// looks no farther for units in its way; with none that near it settles for the best gap between them.
/// Placeholder (not in the design): slice M1-G1's.
pub const LANE_REACH: f32 = 3.0;

/// (G) Lane seeking: a unit walks around a group of idle units that would make way for it only if a clear lane
/// is this near, in its own widths; else it walks through the group, between its units, and they step aside.
/// Placeholder (not in the design): slice M1-G1's.
pub const LANE_AROUND_IDLE: f32 = 4.0 / 3.0;

/// (G) Lane seeking: the clearance a unit keeps from the units it passes. It is what spreads two dense groups that
/// meet: a unit squeezed between two oncoming lanes takes the middle, and the units on the outside, which have room,
/// move out until this much is free. Placeholder (not in the design): slice M1-G1's.
pub const LANE_MARGIN_SU: f32 = 2.0;

/// (G) Lane seeking: when the clear lanes to the left and to the right are this close to equally far, the unit
/// takes its `Mover::avoid_side`: Right at first, so head-on units pass like traffic. Placeholder (not in the
/// design): slice M1-G1's.
pub const LANE_TIE_SU: f32 = 4.0;

/// (G) Lane seeking: of two gaps between lanes, the unit takes the farther one only if it is wider by this share
/// of the extra way, so it doesn't hop from gap to gap as the units around it shift. Placeholder (not in the
/// design): slice M1-G1's.
pub const LANE_NEAR: f32 = 0.5;

/// (G) Lane seeking: a unit counts as oncoming when its heading is against the mover's by more than this cosine
/// (within 60° of straight at it). Placeholder (not in the design): slice M1-G1's.
pub const LANE_ONCOMING_COS: f32 = -0.5;

// Friendly yielding (`systems/units.md > UnitCollision > Ground Collision > Friendly yielding`, design change 0022;
// `docs/movement.md > 6.6`, `mover::sidestep`, `movement::yielding`; slice M1-Y).

/// Friendly yielding (0022): an idle unit whose centre is within this many space units of the line a mover walks
/// along stands "in the middle" of its way, and steps to the side the mover isn't going round on; farther out it
/// steps to the side it is already on. Placeholder (not in the design): slice M1-Y's.
pub const YIELD_CENTRED_SU: f32 = 4.0;

/// `systems/units.md > UnitCollision > Ground Collision > Friendly yielding` (design change 0022): the longest
/// "one short sidestep" of a yielding unit, in widths of its own silhouette. A placeholder, TBD in the design
/// (0022, 0023: "sidestep distance"). The unit steps only as far as the mover's lane needs, up to this.
pub const YIELD_SIDESTEP_WIDTHS: f32 = 1.0;

/// Friendly yielding (0022): the shortest sidestep, space units. Placeholder (not in the design): one, so that a
/// request always moves the unit a little.
pub const YIELD_MIN_SIDESTEP_SU: f32 = 1.0;

/// Friendly yielding (0022) in a dense idle group: a sidestep blocked by another idle unit of the owner passes the
/// request on, at most this many units deep; beyond that the first unit stays an ordinary blocker
/// (`docs/movement.md > 6.6`). Placeholder (not in the design): slice M1-Y's.
pub const YIELD_CHAIN_DEPTH: u8 = 4;

/// Friendly yielding (0022): a sidestep ends this many ticks after the time its distance takes, however far the
/// unit got: time for the units it asked in turn to make room, one tick per link of the chain (so not a tunable of
/// its own: [`YIELD_CHAIN_DEPTH`]'s).
pub const YIELD_WAIT_TICKS: u8 = YIELD_CHAIN_DEPTH;

// Stuck units and their repaths (`systems/controls/command_behaviors.md > MovingToLocation > Failure`, design change
// 0024; `docs/movement.md > 5.6`, `6.5`; `mover::schedule`).

/// `systems/controls/command_behaviors.md > MovingToLocation > Failure` (design change 0024; a placeholder to tune): a
/// stuck unit looks for another route at most 3 times per order leg (`docs/movement.md > 5.6`).
pub const MAX_REPATHS: u8 = 3;

/// `systems/controls/command_behaviors.md > MovingToLocation > Failure` (design change 0024; a placeholder to tune): a
/// unit is stuck after 1.5 s (24 frames) without getting any closer to its target (`docs/movement.md > 6.5`).
pub const STUCK_TICKS: Tick = 24;

/// `systems/controls/command_behaviors.md > MovingToLocation > Failure` (design change 0024; placeholders to tune): a
/// stuck unit waits 0.5 s, 1 s and 2 s before its first, second and third attempt at another route
/// (`docs/movement.md > 5.6`).
pub const REPATH_WAITS: [Tick; MAX_REPATHS as usize] = [8, 16, 32];

/// 0024's "about 7 s worst case": a unit that never gets on is stuck after [`STUCK_TICKS`], waits out the three
/// [`REPATH_WAITS`], and gives up once it is still stuck a whole [`STUCK_TICKS`] after its third attempt: 104 ticks,
/// 6.5 s, after its last progress (`docs/movement.md > 5.6`). Not a tunable of its own: it follows from those.
pub const GIVE_UP_TICKS: Tick = STUCK_TICKS + REPATH_WAITS[0] + REPATH_WAITS[1] + REPATH_WAITS[2] + STUCK_TICKS;

/// `docs/movement.md > 6.5`: a stuck unit that isn't in contact with a unit this very tick waits this many ticks
/// more to meet its blocker again before it recovers without knowing it. Placeholder (not in the design): the
/// movement implementation's (slice M1-D), half a second.
pub const STUCK_GRACE_TICKS: Tick = 8;

/// From this many ticks without progress on, a unit on its way counts as held up (`Mover::is_held_up`, for
/// overlays): half of [`STUCK_TICKS`], so not a tunable of its own.
pub const HALF_STUCK_TICKS: Tick = STUCK_TICKS / 2;

// (M5-V1) Oriented Silhouettes (plan decision T78; `systems/units.md > UnitCollision > Collision Response
// (Un-sticking)`: "Overlap caused by the rotation itself (a corner sweeping into a wall) follows the same principle:
// the blocked part is discarded and the unit never ends up overlapping. The exact geometric method is a technical
// choice"; `movement::collide::turn`).

/// (M5-V1) T78: "If blocked, it is halved up to five times, the free part is kept": a blocked turn of a Silhouette
/// that turns with its facing, or a move up to an oriented shape that the sweep's contact leaves inside it by a
/// rounding, is bisected this many times, so the unit keeps all but at most a 32nd of the free part. Placeholder (not
/// in the design: its "exact geometric method is a technical choice").
pub const STEP_HALVINGS: u32 = 5;

/// (M5-V1) How far the corners of a Silhouette that turns with its facing move at most between two tries of its turn:
/// 4 su, a 16th of a tile. A turn is tried in parts no longer, so between two tries a corner can cut through at
/// most the tip of another shape's corner, about half this deep, and it never ends inside one (each try is the
/// predicates of invariants I1 and I2). The units whose turning isn't built turn without limit (`movement::turn_rate`):
/// an instant turn of 180° of the Hand of Heaven's Silhouette (64x40, its corners 37.7 su from its centre) takes 30
/// tries. Placeholder (not in the design: a technical bound).
pub const TURN_TRY_SU: f32 = 4.0;
