//! (M5-H1) Air units: a Glider's idle circle and the soft separation of air units from each other
//! (`systems/units.md > GliderMovement`, `> Air Collision`; plan decision T81; `crate::movement::glider`,
//! `crate::movement::air`); (M5-N7) where an aircraft out of a runway circles without a rally point
//! (`crate::production::exit::Runway`); (M6-GH2) how near its slot an aircraft lands; (M6-GL) where a Glider's landing
//! approach slows down. A type's own flight numbers (IdleSpeed, MaxSpeed, its accelerations, its
//! MaxCentripetalAcceleration) and its SeparationRadius are its def's.

tunables! {
    /// The tunables of air (module docs).
    Air;

    /// `factions/cults/units/sparrow.md > Placeholder stats`: "Idle circle: radius 1 grid unit (64 su) at IdleSpeed
    /// (placeholder; Q0120: at its centripetal limit an idle circle would be only 32 su, nearly a spin, so idle
    /// circling uses a gentler turn)"; design change 0354: "IdleSpeed² / 64 = 0.25 su/frame²" of centripetal
    /// acceleration at the Sparrow's IdleSpeed. The Dragonfly's and the Specter's are the same ("idle circle radius 1
    /// grid unit", `factions/gdo/units/dragonfly.md`, `factions/syndicate/units/specter.md`). A Glider circles its
    /// point on this radius while it is idle, and from its arrival: grid units.
    idle_circle_gu: f32 = 1.0,

    /// Plan decision T81: "Each pair of air units within the larger SeparationRadius is pushed apart, by at most 2 su a
    /// tick each". Read as a cap on each unit's whole push in a tick, whatever the number of pairs it is in, so that a
    /// unit never moves more than its speed and this in a tick (invariant I3, `docs/movement.md > 8`). Space units per
    /// tick. Placeholder (Q0120 point 1: the design gives the radius, not the strength of the push). (M5-H2) It is the
    /// full push, a HoverCraft's: each air base takes its share of it (`UnitBaseDef::separation_strength`), a Glider a
    /// third (design change 0412: "**a third** of the HoverCraft strength [...]; was a quarter", design change 0405's),
    /// two thirds of a space unit a tick.
    separation_push_su: f32 = 2.0,

    /// (M5-N7) `factions/cults/units/sparrow.md > Placeholder stats`: "**No rally point**: it circles 2 grid units past
    /// the runway's C end" (design change 0354 point 4): how far past the middle of a runway's `toward` end, along the
    /// runway, the point lies that an aircraft out of it circles when its building has no rally point
    /// (`ExitRule::Runway`), grid units.
    runway_circle_past_gu: f32 = 2.0,

    /// How strongly a Glider flying onto its circle turns in toward it, per radius of distance off it
    /// (`movement::glider`: the heading it wants blends the circle's tangent with the way to the circle, by this times
    /// its distance off the circle over the radius). Placeholder (not in the design: how a Glider joins its circle). At
    /// 0.5 a Glider three radii out comes in at 45°, and one on the circle at IdleSpeed corrects within its turn. A
    /// pull of 1 took back much of each push that parts Gliders circling one point (`movement::air`): six Sparrows sent
    /// to one point were 30.8 su apart at closest 10 s later, against 31.8 at 0.5, for a SeparationRadius of 32.
    circle_pull: f32 = 0.5,

    /// (M6-GH2) How near its landing slot an aircraft flying home must have come to rest to land on it
    /// (`crate::landing`; plan decision T88: "a HoverCraft brakes to a stop over its pad"), centre to slot, space
    /// units. **Placeholder (not in the design: how near over its slot an aircraft lands)**: a HoverCraft's flight ends
    /// on its goal (`movement::drag`), and this leaves room for the air units' separation to have nudged it since it
    /// came to rest (`movement::air`, at most `separation_push_su` a tick, until it lands); farther off, it flies back
    /// over the slot first. On its slot it stays within it (invariant L1).
    landing_reach_su: f32 = 8.0,

    /// (M6-GL) How far from the point of its landing approach a Glider slows to its IdleSpeed, for the last of the way
    /// (`crate::movement::approach`; `factions/gdo/structures/airfield.md > Returning and rearming`: "it approaches its
    /// slot at IdleSpeed, stops over it while it lands"), grid units. **Placeholder (Q0192 point 2)**, the designer's
    /// (`systems/mechanics/aircraft_landing.md > Vertical`: "it slows from MaxSpeed to IdleSpeed 4 GU before the
    /// spot"): the Dragonfly, braking 0.6 su/frame² from its MaxSpeed of 14, is at its IdleSpeed of 6 some 133 su on,
    /// so that it flies the last 120 su or so, about 20 frames, straight at its slot at IdleSpeed. Farther off it flies
    /// at MaxSpeed, as to any goal.
    approach_gu: f32 = 4.0,
}
