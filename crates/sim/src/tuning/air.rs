//! (M5-H1) Air units: a Glider's idle circle and the soft separation of air units from each other
//! (`systems/units.md > GliderMovement`, `> Air Collision`; plan decision T81; `crate::movement::glider`,
//! `crate::movement::air`). A type's own flight numbers (IdleSpeed, MaxSpeed, its accelerations, its
//! MaxCentripetalAcceleration) and its SeparationRadius are its def's.

/// `factions/cults/units/sparrow.md > Flight rules`: "Idle circle: radius 1 grid unit (64 su) at IdleSpeed
/// (placeholder; Q0120)" (design change 0354 point 5: the idle circle has a fixed radius of 1 grid unit, "IdleSpeed² /
/// 64 = 0.25 su/frame²" of centripetal acceleration at the Sparrow's IdleSpeed; the Dragonfly's and the Specter's are
/// the same 1 grid unit). A Glider circles its point on this radius while it is idle, and on arrival: grid units.
pub const IDLE_CIRCLE_GU: f32 = 1.0;

/// Plan decision T81: "Each pair of air units within the larger SeparationRadius is pushed apart, by at most 2 su a
/// tick each". Read as a cap on each unit's whole push in a tick, whatever the number of pairs it is in, so that a unit
/// never moves more than its speed and this in a tick (invariant I3, `docs/movement.md > 8`). Space units per tick.
/// Placeholder (Q0120 point 1: the design gives the radius, not the strength of the push).
pub const SEPARATION_PUSH_SU: f32 = 2.0;

/// How strongly a Glider flying onto its circle turns in toward it, per radius of distance off it
/// (`movement::glider`: the heading it wants blends the circle's tangent with the way to the circle, by this times its
/// distance off the circle over the radius). Placeholder (not in the design: how a Glider joins its circle); 1 gives a
/// 45° approach from twice the radius out, and a Glider on the circle at IdleSpeed corrects within its turn.
pub const CIRCLE_PULL: f32 = 1.0;
