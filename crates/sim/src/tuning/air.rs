//! (M5-H1) Air units: a Glider's idle circle and the soft separation of air units from each other
//! (`systems/units.md > GliderMovement`, `> Air Collision`; plan decision T81; `crate::movement::glider`,
//! `crate::movement::air`). A type's own flight numbers (IdleSpeed, MaxSpeed, its accelerations, its
//! MaxCentripetalAcceleration) and its SeparationRadius are its def's.

/// `factions/cults/units/sparrow.md > Placeholder stats`: "Idle circle: radius 1 grid unit (64 su) at IdleSpeed
/// (placeholder; Q0120: at its centripetal limit an idle circle would be only 32 su, nearly a spin, so idle circling
/// uses a gentler turn)"; design change 0354: "IdleSpeed² / 64 = 0.25 su/frame²" of centripetal acceleration at the
/// Sparrow's IdleSpeed. The Dragonfly's and the Specter's are the same ("idle circle radius 1 grid unit",
/// `factions/gdo/units/dragonfly.md`, `factions/syndicate/units/specter.md`). A Glider circles its point on this radius
/// while it is idle, and from its arrival: grid units.
pub const IDLE_CIRCLE_GU: f32 = 1.0;

/// Plan decision T81: "Each pair of air units within the larger SeparationRadius is pushed apart, by at most 2 su a
/// tick each". Read as a cap on each unit's whole push in a tick, whatever the number of pairs it is in, so that a unit
/// never moves more than its speed and this in a tick (invariant I3, `docs/movement.md > 8`). Space units per tick.
/// Placeholder (Q0120 point 1: the design gives the radius, not the strength of the push).
pub const SEPARATION_PUSH_SU: f32 = 2.0;

/// How strongly a Glider flying onto its circle turns in toward it, per radius of distance off it
/// (`movement::glider`: the heading it wants blends the circle's tangent with the way to the circle, by this times its
/// distance off the circle over the radius). Placeholder (not in the design: how a Glider joins its circle). At 0.5 a
/// Glider three radii out comes in at 45°, and one on the circle at IdleSpeed corrects within its turn. A pull of 1
/// took back much of each push that parts Gliders circling one point (`movement::air`): six Sparrows sent to one
/// point were 31.2 su apart at closest 10 s later, against 31.8 at 0.5, for a SeparationRadius of 32.
pub const CIRCLE_PULL: f32 = 0.5;
