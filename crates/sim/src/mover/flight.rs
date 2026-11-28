//! (M5-H1) What a Glider's mover keeps of its flight from tick to tick (plan decision T81): the point it circles
//! ([`Mover::orbit`]) and its strafing run ([`Strafe`]); (M5-N7) its takeoff run along a runway ([`Takeoff`]). The
//! movement step flies it (`movement::glider`); its speed is its [`Mover::momentum`], as a DragMovement unit's velocity
//! is. (M6-SD) And an aircraft's rise out of a Tunnel's air exit door ([`Mover::rise`], [`Mover::rises`]). (M6-GL) And
//! a Glider's landing approach, which ends stopped over its point ([`Approach`], [`Mover::approach_to`]).

use glam::Vec2;
use serde::{Deserialize, Serialize};

use super::{MoveGoal, Mover};
use crate::units::Tick;

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

    /// (M6-GL) A Glider's leg to `spot` that ends stopped over it, in the air, as the Dragonfly's landing does
    /// ([`Approach`]): a new leg to the point ([`Mover::go_to`]), flown as a landing approach (`movement::approach`).
    /// Its landing at an Airfield's slot (`Sim::step_landings`).
    pub fn approach_to(&mut self, spot: Vec2, tick: Tick) {
        self.go_to(MoveGoal::Point(spot), None, tick);
        self.approach = Some(Approach::Flying);
    }

    /// (M6-GL) Whether it is a Glider stopped over the point of its landing approach ([`Approach::Over`]).
    pub fn is_over(&self) -> bool {
        self.approach == Some(Approach::Over)
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

/// (M6-GL) A Glider's landing approach to its goal's point ([`Mover::approach`], [`Mover::approach_to`]):
/// `factions/gdo/structures/airfield.md > Returning and rearming`: "**The Dragonfly** (a Glider) lands and takes off
/// **vertically**, as the Specter at an air door (user, 2026-11-26; Q0162): it approaches its slot at IdleSpeed, stops
/// over it while it lands (16 frames), and takes off straight up, leaving at IdleSpeed. Landed, it doesn't move"
/// (design change 0428 point 5). The movement step flies it (`movement::approach`); a new leg and a stop end it.
#[derive(Copy, Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum Approach {
    /// It flies at its goal's point, slowing to IdleSpeed for the last of the way, to stop over it.
    Flying,
    /// It has stopped over the point, its leg over (`MoveStatus::Arrived`), and holds still there in the air while the
    /// order that sent it there runs, for its landing to begin (`movement::approach::holds`).
    Over,
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
