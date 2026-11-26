//! (M5-N4b) Spells (`crate::shroud`: the Disciple's Shroud, `factions/cults/units/disciple.md > Spells > Shroud`). A
//! spell's own numbers, its radius, its drain and the energy it needs, are its caster's def's (`defs::SpellDef`).

use crate::units::Tick;

/// `factions/cults/units/disciple.md > Spells > Shroud`: "The buffer is released **0.5 s (8 frames), placeholder,
/// after the first hit it absorbs** (each connected cloud keeps its own timer; user, 2026-11-26; Q0113)": the ticks
/// from a cloud's first absorbed hit to its release.
pub const RELEASE_DELAY: Tick = 8;

/// Plan decision T77: the spacing of the square lattice on which a release's areas are counted, space units: "areas
/// are integer counts on a 4-su lattice (a 3-GU circle is about 7,200 points): deterministic, and within about 1% of
/// exact" (`shroud::lattice`).
pub const LATTICE_STEP: f32 = 4.0;
