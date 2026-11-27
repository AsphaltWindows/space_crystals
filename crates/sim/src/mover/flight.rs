//! (M5-H1) What a Glider's mover keeps of its flight from tick to tick (plan decision T81): the point it circles
//! ([`Mover::orbit`]) and its strafing run ([`Strafe`]); (M5-N7) its takeoff run along a runway ([`Takeoff`]). The
//! movement step flies it (`movement::glider`); its speed is its [`Mover::momentum`], as a DragMovement unit's velocity
//! is. (M6-SD) And an aircraft's rise out of a Tunnel's air exit door ([`Mover::rise`], [`Mover::rises`]).

use glam::Vec2;
use serde::{Deserialize, Serialize};

use super::Mover;

/// A Glider's strafing run at its goal: `systems/controls/command_behaviors.md > AttackingObject`, GliderMovement
/// exception: "Gliders cannot stop. Instead of stopping when in range, the Glider performs strafing runs: 1. Fly toward
/// TargetObject 2. When in range and turret arc: fire while passing through 3. Continue past the target 4. Loop around
/// for another pass 5. Repeat until target is destroyed or a new command is issued", and at a location
/// (`systems/controls/command_behaviors.md > AttackingLocation`: "Same strafing run pattern as AttackingObject,
/// targeting the ground location"); `factions/cults/units/sparrow.md > Flight rules`: "Strafing: after a pass it flies
/// on until the target's **centre** is out of range [...], then loops back at full speed". The movement flies it
/// (`movement::glider`): at MaxSpeed through the goal, on past it, and back.
#[derive(Copy, Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct Strafe {
    /// The range a pass flies on to, space units: past its goal, the Glider holds its course until the distance from
    /// its Silhouette's edge to the goal's centre (`Shape::distance_to`) is more than this. The attack's Range, which
    /// its fire measures edge to edge (`systems/combat.md > Range Measurement`), as every other check of it does: design
    /// change 0408 (`systems/controls/command_behaviors.md > AttackingObject`, GliderMovement exception, step 3): "the
    /// pass ends when the **target's centre** is out of weapon range, measured from the glider's edge to the target's
    /// centre", so that "the loop is the same size whatever the target" (it was edge to edge, the loop growing with the
    /// target's size).
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
    /// (M6-SD) One tick of the unit's rise out of an air door ([`Mover::rise`]), in the movement step: whether it still
    /// rises in this tick, and so holds still. The rise's last tick ends it, and in that tick the unit moves: a rise of
    /// 16 ticks, begun as the unit came out (after the movement step of its tick), holds it in the next 15 and lets it
    /// move in the 16th.
    pub fn rises(&mut self) -> bool {
        let Some(rise) = &mut self.rise else {
            return false;
        };
        if rise.step(crate::economy::RATIO_ONE) {
            self.rise = None;
            return false;
        }
        true
    }

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

/// (M5-N7) A Glider's takeoff run along the runway it appeared on (`ExitRule::Runway`, `production::Runway`;
/// `factions/cults/units/sparrow.md > Flight rules`: "**Takeoff**: it appears over the runway's B end facing C at
/// IdleSpeed and speeds up along the runway"): it flies straight on, speeding up toward its MaxSpeed, until its centre
/// is past the runway's end ([`Takeoff::is_over`]), and only then as its goal or its orbit sends it (`movement::glider`).
/// An order given meanwhile waits for the runway's end: **Placeholder (not in the design: a takeoff run cut short)**.
#[derive(Copy, Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct Takeoff {
    /// The middle of the runway's end it takes off toward, space units.
    pub end: Vec2,
    /// The unit vector along the runway, toward `end`: the way it faces as it appears.
    pub along: Vec2,
}

impl Takeoff {
    /// Whether a Glider centred on `pos` is past the runway's end, or on it: its run is over.
    pub fn is_over(&self, pos: Vec2) -> bool {
        (pos - self.end).dot(self.along) >= 0.0
    }
}
