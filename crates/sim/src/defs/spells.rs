//! (M5-N4b) The spells a unit casts on its energy (`UnitDef::spells`, `crate::energy`): the Disciple's Shroud
//! (`factions/cults/units/disciple.md > Spells`), and to come the Prayer-mobile's Inspire and Suppress
//! (`factions/cults/units/prayer_mobile.md`), each a [`SpellDef`] with its numbers. Every spell needs some energy to
//! start ([`SpellDef::needs`]), which one check asks (`energy::has`; the gate's `Reject::NotEnoughEnergy`).

use serde::{Deserialize, Serialize};

/// A spell of a unit's type, with its numbers (module docs).
#[derive(Copy, Clone, Debug, PartialEq, Serialize, Deserialize)]
pub enum SpellDef {
    /// `factions/cults/units/disciple.md > Spells > Shroud`: "A **channeling** spell: while the Disciple channels it, its
    /// energy **drains gradually**, and it creates a **cloud**" (`crate::shroud`; its order, `OrderKind::Shroud`).
    Shroud(ShroudDef),
}

impl SpellDef {
    /// The energy it needs to start, points: Shroud's "needs at least **10 energy** to start".
    pub fn needs(&self) -> f32 {
        match self {
            SpellDef::Shroud(shroud) => shroud.needs,
        }
    }

    /// Its numbers, if it is Shroud.
    pub fn shroud(&self) -> Option<&ShroudDef> {
        match self {
            SpellDef::Shroud(shroud) => Some(shroud),
        }
    }
}

/// `factions/cults/units/disciple.md > Placeholder stats > Shroud numbers`.
#[derive(Copy, Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct ShroudDef {
    /// "cloud radius **3 grid units** around the Disciple", grid units.
    pub radius: f32,
    /// "drains **5 energy per second**", points a second, which the Disciple's "**own regeneration continues while
    /// channelling**" offsets (`energy::step`).
    pub drain: f32,
    /// "needs at least **10 energy** to start", points.
    pub needs: f32,
}
