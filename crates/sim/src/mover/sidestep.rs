//! The one short sidestep of a yielding unit — `systems/units.md > UnitCollision > Ground Collision > Friendly
//! yielding` (design changes 0022 and 0023), `docs/movement.md > 6.6`. The movement step takes it
//! (`movement::yielding`).

use crate::ruleset::rules;
use glam::Vec2;
use serde::{Deserialize, Serialize};

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
    /// How many units the request was passed through: 0 when a moving unit asked
    /// (`tuning::movement::Movement::yield_chain_depth`).
    pub depth: u8,
    /// The order group of the moving unit that asked. A unit that arrived with that group doesn't yield to it
    /// (design change 0027), so a request passed on keeps it.
    pub for_group: Option<u32>,
}

impl Sidestep {
    /// A sidestep of `dist` space units along `dir` for a unit that moves `speed` space units per tick.
    pub fn new(dir: Vec2, dist: f32, speed: f32, depth: u8, for_group: Option<u32>) -> Self {
        let tuning = &rules().tuning.movement;
        let ticks = (dist / speed).ceil().clamp(1.0, 200.0) as u8;
        Self {
            dir,
            left: dist,
            ticks_left: ticks.saturating_add(tuning.yield_wait_ticks()),
            depth,
            for_group,
        }
    }
}
