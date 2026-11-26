//! (M5-H1) What a Glider's mover keeps of its flight from tick to tick (plan decision T81): the point it circles
//! ([`Mover::orbit`]) and its strafing run ([`Strafe`]). The movement step flies it (`movement::glider`); its speed is
//! its [`Mover::momentum`], as a DragMovement unit's velocity is.

use serde::{Deserialize, Serialize};

use super::Mover;

/// A Glider's strafing run at its goal: `systems/controls/command_behaviors.md > AttackingObject`, GliderMovement
/// exception: "Gliders cannot stop. Instead of stopping when in range, the Glider performs strafing runs: 1. Fly toward
/// TargetObject 2. When in range and turret arc: fire while passing through 3. Continue past the target 4. Loop around
/// for another pass 5. Repeat until target is destroyed or a new command is issued", and at a location
/// (`systems/controls/command_behaviors.md > AttackingLocation`: "Same strafing run pattern as AttackingObject,
/// targeting the ground location"); `factions/cults/units/sparrow.md > Flight rules`: "Strafing: after a pass it flies
/// on until the target is out of range, then loops back at full speed". The movement flies it (`movement::glider`): at
/// MaxSpeed through the goal, on past it, and back.
#[derive(Copy, Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct Strafe {
    /// The range a pass flies on to, space units: past its goal, the Glider holds its course until the gap from its
    /// Silhouette to the goal (edge to edge, `shape`) is more than this. The attack's Range.
    pub range: f32,
    /// Where the run is.
    pub pass: Pass,
}

/// Where a strafing run is ([`Strafe`]): a turn toward the goal, a pass at it, and the flight on past it, in a loop.
#[derive(Copy, Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum Pass {
    /// It flies at its goal, which isn't ahead of it yet: the run's start, and the loop back after each pass.
    Toward,
    /// It flies at its goal, which is ahead of it (within 90° of its heading).
    Run,
    /// The goal fell behind it: it has passed it, and holds its course until the goal is out of the run's range.
    On,
}

impl Mover {
    /// A Glider's leg that just began ([`Mover::go_to`]) is a strafing run at its goal, flying on past it to `range`
    /// space units ([`Strafe`]), from a turn toward it. The behaviours' strafing passes (`orders::engage`,
    /// `orders::attack_ground`); a new leg and a stop end it.
    pub fn strafe(&mut self, range: f32) {
        self.strafe = Some(Strafe {
            range,
            pass: Pass::Toward,
        });
    }
}
