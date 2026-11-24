//! The one short sidestep of a yielding unit — `systems/units.md > UnitCollision > Ground Collision > Friendly
//! yielding` (design changes 0022 and 0023), `docs/movement.md > 6.6`. The movement step takes it
//! (`movement::yielding`).

use glam::Vec2;
use serde::{Deserialize, Serialize};

/// `systems/units.md > UnitCollision > Ground Collision > Friendly yielding` (design change 0022): the longest
/// "one short sidestep" of a yielding unit, in widths of its own silhouette. A placeholder, TBD in the design
/// (0022, 0023: "sidestep distance"). The unit steps only as far as the mover's lane needs, up to this.
pub const YIELD_SIDESTEP_WIDTHS: f32 = 1.0;

/// Friendly yielding (0022): the shortest sidestep, space units, so that a request always moves the unit a little
/// (tuning const).
pub const YIELD_MIN_SIDESTEP_SU: f32 = 1.0;

/// Friendly yielding (0022) in a dense idle group: a sidestep blocked by another idle unit of the owner passes the
/// request on, at most this many units deep; beyond that the first unit stays an ordinary blocker (tuning const,
/// `docs/movement.md > 6.6`).
pub const YIELD_CHAIN_DEPTH: u8 = 4;

/// Friendly yielding (0022): a sidestep ends this many ticks after the time its distance takes, however far the
/// unit got: time for the units it asked in turn to make room (one tick per link of the chain; tuning const).
pub const YIELD_WAIT_TICKS: u8 = YIELD_CHAIN_DEPTH;

/// The one short sidestep of a yielding unit (`systems/units.md > UnitCollision > Ground Collision > Friendly
/// yielding`, design changes 0022 and 0023; `docs/movement.md > 6.6`): a real move through the collision step, by
/// an idle unit that a blocked unit of its owner asked to make way.
#[derive(Copy, Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct Sidestep {
    /// The way it steps, a unit vector.
    pub dir: Vec2,
    /// How far it still has to go along `dir`, space units.
    pub left: f32,
    /// It ends after this many more ticks, wherever the unit got to.
    pub ticks_left: u8,
    /// How many units the request was passed through: 0 when a moving unit asked ([`YIELD_CHAIN_DEPTH`]).
    pub depth: u8,
    /// The order group of the moving unit that asked. A unit that arrived with that group doesn't yield to it
    /// (design change 0027), so a request passed on keeps it.
    pub for_group: Option<u32>,
}

impl Sidestep {
    /// A sidestep of `dist` space units along `dir` for a unit that moves `speed` space units per tick.
    pub fn new(dir: Vec2, dist: f32, speed: f32, depth: u8, for_group: Option<u32>) -> Self {
        let ticks = (dist / speed).ceil().clamp(1.0, 200.0) as u8;
        Self {
            dir,
            left: dist,
            ticks_left: ticks.saturating_add(YIELD_WAIT_TICKS),
            depth,
            for_group,
        }
    }
}
