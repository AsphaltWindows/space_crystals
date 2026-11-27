//! (M5-N4b) Spells (`crate::shroud`: the Disciple's Shroud, `factions/cults/units/disciple.md > Spells > Shroud`; (M6-CM)
//! `crate::mirage`: its Mirage, `> Spells > Mirage`). A spell's own numbers, its radius, its drain and the energy it
//! needs, Mirage's cost, illusions and offset, are its caster's def's (`defs::SpellDef`).

use crate::units::Tick;

tunables! {
    /// The tunables of spells (module docs).
    Spells;

    /// `factions/cults/units/disciple.md > Spells > Shroud`: "The buffer is released **0.5 s (8 frames), placeholder,
    /// after the first hit it absorbs** (each connected cloud keeps its own timer; user, 2026-11-26; Q0113)": the ticks
    /// from a cloud's first absorbed hit to its release.
    release_delay: Tick = 8,

    /// Plan decision T77: the spacing of the square lattice on which a release's areas are counted, space units: "areas
    /// are integer counts on a 4-su lattice (a 3-GU circle is about 7,200 points): deterministic, and within about 1%
    /// of exact" (`shroud::lattice`).
    lattice_step: f32 = 4.0,

    /// (M6-CM) `factions/cults/units/disciple.md > Spells > Shroud`: "**The cloud lingers** (user, 2026-11-27): when the
    /// channel ends, by the Disciple's death or for any other reason, the cloud **stays for 1.5 s (24 frames,
    /// placeholder)** and keeps absorbing as usual": the ticks from the tick a channel ends in to the tick its circle
    /// vanishes in (`shroud::Member::until`), which a 0 would make the tick it ends in, as before the linger.
    linger: Tick = 24,

    /// (M6-CM) Placeholder (not in the design: how finely and how far a blocked illusion's "nearest free spot" is looked
    /// for; `factions/cults/units/disciple.md > Spells > Mirage`: "A blocked spot puts that illusion at the **nearest
    /// free spot**"): the spacing of the rings round the blocked spot that are searched, and of the points on each,
    /// space units (`Sim::cast_mirage`).
    mirage_search_step_su: f32 = 8.0,

    /// (M6-CM) Placeholder (as `mirage_search_step_su`): how far from the blocked spot the search goes, grid units. An
    /// illusion with no free spot within it isn't made.
    mirage_search_radius_gu: f32 = 3.0,
}
