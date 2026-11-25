//! The Tunnel Network's ways in and out (`factions/syndicate/tunnels.md > Tunnel Network`, `> EjectMenu`;
//! `crate::network`): where Side A's paths lie, how near a unit goes in, how far it walks out, and the air doors. The
//! transit delay and the regeneration inside are the Tunnel's def's (`defs::TunnelDef`). And the decay of the
//! expansions a lost Tunnel leaves neutral (`factions/syndicate/tunnels.md > Losing Tunnels`, `sim/tunnels.rs`).

use crate::units::SU_PER_GU;

/// How far Side A's paths lie from the middle of the side, space units: the exit path towards Side B, the entry path
/// towards Side D (design change 0248). The middles of the two paths the client draws (`object_render::tunnel`, which
/// reads this), 60 su apart, so that a unit up to 60 su wide on one never touches one on the other ("entering and
/// exiting units never block each other"). **Placeholder**: the design gives the halves, not where in them (question
/// Q0063 point 2 proposed "centred on its half"; the drawing of slice M4-A came first).
pub const PATH_OFFSET_SU: f32 = 30.0;

/// Question Q0063 point 2, as proposed (design change 0247: "As proposed"): "A unit goes in when its silhouette is
/// within 8 su of the entry spot (DropOffReach's value, as a placeholder)".
pub const ENTRY_REACH_SU: f32 = 8.0;

/// `factions/syndicate/tunnels.md > EjectMenu` (design change 0250): "With no rally point, a unit walks about 1 grid
/// unit straight out of Side A and idles".
pub const WALK_OUT_SU: f32 = SU_PER_GU;

/// Whether air units pass through a Tunnel: **not in M4** (the Director's decision for slice M4-C). No Syndicate air
/// unit exists yet (the Phantom is deferred, the Specter a concept), so Enter and Eject refuse every air unit at every
/// tier, Tier 3 included.
///
/// **The rule to come with the first Syndicate air unit** (design changes 0250 and 0251): "Air units leave through
/// the Tier 3 air exit door, in parallel with ground units; since aircraft don't collide, the door has its own exit
/// rate: one aircraft per 8 frames (0.5 s), placeholder (the same rate for the air entry door)"; "the air doors are on
/// the Tunnel's roof: the entry door over the entry path's half of the Tunnel, the exit door over the exit path's
/// half". They never use Side A's paths, and never wait for the exit spot.
pub const AIR_DOORS: bool = false;

/// `factions/syndicate/tunnels.md > Losing Tunnels` (design change 0258 point 1, answering Q0069): a neutral expansion
/// "slowly loses HP until destroyed. Decay is a percentage of MaxHP per second ...: 1% per second, placeholder (a
/// Hideout lasts 100 s); the number is to be tuned". Whole percents of its MaxHP per second (`sim/tunnels.rs`, which
/// counts it out exactly per tick). **Placeholder** (design change 0258: "1% per second is a placeholder").
pub const DECAY_PERCENT_PER_SECOND: u32 = 1;
