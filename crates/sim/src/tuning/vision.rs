//! Vision (`core/vision.md`): the fog's delay, the reveal of an attacker that shot from outside its victim's vision,
//! and what elevation does to sight (`crate::vision`, `crate::elevation`). The SightRanges are the defs'.

use crate::units::Tick;

tunables! {
    /// The tunables of vision (module docs).
    Vision;

    /// `core/vision.md > Visible > Fog delay` (design change 0178): how many ticks a tile that leaves all of a player's
    /// sight stays Visible before it turns Explored. "0.5 seconds (8 frames), placeholder".
    fog_delay_ticks: Tick = 8,

    /// `core/vision.md > Attacks from outside vision` (design change 0177): how many ticks an attacker that shot from
    /// outside its victim's vision stays Visible to the victim's owner after each shot. "1.5 seconds (24 frames),
    /// placeholder".
    reveal_ticks: Tick = 24,

    /// `core/vision.md > ElevationModifier > Higher ground`, `> Lower ground`: one grid unit more, or less.
    elevation_modifier: f32 = 1.0,
}
