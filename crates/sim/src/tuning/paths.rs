//! Paths: the path server's budget and costs, what blocks a ground path, and when a chase is planned again (the
//! implementation's `docs/movement.md > 5`; `crate::nav`, `crate::paths`).

use crate::units::Tick;

/// `docs/movement.md > 5.5`: node expansions per tick within which the path server starts jobs. Placeholder (not in
/// the design): the navigation library's (slice M1-C), a bound on a tick's path work.
pub const PATH_BUDGET: u32 = 32_000;

/// `docs/movement.md > 5.5`: from this many requests of one order group on whose goals the group's anchor sees, they
/// share one Dijkstra field. Placeholder (not in the design): the navigation library's (slice M1-C).
pub const GROUP_FIELD_MIN: usize = 4;

/// `docs/movement.md > 5.3`: cost of an orthogonal step. Placeholder (not in the design): with [`DIAGONAL_COST`], 1
/// and √2 in whole numbers.
pub const STEP_COST: u32 = 10;

/// `docs/movement.md > 5.3`: cost of a diagonal step (√2 × [`STEP_COST`], in whole numbers).
pub const DIAGONAL_COST: u32 = 14;

/// Whether map resources (crystal patches, supply stations) block ground units. Placeholder: the default until
/// question Q0011 (gap G10) is answered: yes.
pub const RESOURCES_BLOCK: bool = true;

/// Whether a Rugged tile is impassable for a base with RuggedTerrain = false. Placeholder: the default until
/// question Q0011 (gap G11) is answered: yes.
pub const RUGGED_IMPASSABLE: bool = true;

/// `docs/movement.md > 5.6`, "Chase target moved": a path to an object is planned again once the object is farther
/// than this from where it was when the path was planned, grid units ... Placeholder (not in the design): slice
/// M1-E's.
pub const CHASE_REPLAN_GU: f32 = 1.0;

/// ... or than this share of the unit's distance to it, whichever is more (`docs/movement.md > 5.6`). Placeholder
/// (not in the design): slice M1-E's.
pub const CHASE_REPLAN_SHARE: f32 = 0.25;

/// `docs/movement.md > 5.6`: a chase is planned again at most this often, ticks. It doesn't count against the repath
/// cap. Placeholder (not in the design): slice M1-E's, a multiple of [`STAGGER`](super::movement::STAGGER).
pub const CHASE_REPLAN_TICKS: Tick = 8;
