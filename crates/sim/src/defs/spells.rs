//! (M5-N4b) The spells a unit casts on its energy (`UnitDef::spells`, `crate::energy`): the Disciple's Shroud
//! (`factions/cults/units/disciple.md > Spells`), and (M5-N11) the Prayer-mobile's Inspire and Suppress
//! (`factions/cults/units/prayer_mobile.md > Abilities`), each a [`SpellDef`] with its numbers. Every spell needs some
//! energy to start ([`SpellDef::needs`]), which one check asks (`energy::has`; the gate's `Reject::NotEnoughEnergy`).
//!
//! - **A channel** ([`SpellDef::Shroud`]) is an order, which the unit carries out over time and which drains its energy
//!   as it goes (`OrderKind::Shroud`, `crate::shroud`).
//! - **A cast** ([`SpellDef::Cast`]) is a command, applied at once in the commands phase, which spends its cost then
//!   and gives its buff to the units its area takes in (`Command::Cast`, `crate::buffs`): the Prayer-mobile's
//!   Inspire and Suppress, area abilities as the Shrine's are (`AreaAbility`), round the vehicle and ahead of it.

use serde::{Deserialize, Serialize};

use crate::defs::AreaAbility;

/// A spell of a unit's type, with its numbers (module docs).
#[derive(Copy, Clone, Debug, PartialEq, Serialize, Deserialize)]
pub enum SpellDef {
    /// `factions/cults/units/disciple.md > Spells > Shroud`: "A **channeling** spell: while the Disciple channels it, its
    /// energy **drains gradually**, and it creates a **cloud**" (`crate::shroud`; its order, `OrderKind::Shroud`).
    Shroud(ShroudDef),
    /// (M5-N11) An area ability the unit casts at once, with no target (`factions/cults/units/prayer_mobile.md >
    /// Placeholder stats`: "**Inspire** (hotkey **R**, instant, no target)", "**Suppress** (hotkey **F**, instant, no
    /// target)"), whose numbers are the ability's (`AreaAbility::def`): a `Command::Cast` (module docs).
    Cast(AreaAbility),
}

impl SpellDef {
    /// The energy it needs to start, points: Shroud's "needs at least **10 energy** to start"; (M5-N11) a cast's, its
    /// cost (Inspire's "75 energy", Suppress's "100 energy"), all of which it spends.
    pub fn needs(&self) -> f32 {
        match self {
            SpellDef::Shroud(shroud) => shroud.needs,
            SpellDef::Cast(ability) => ability.def().cost,
        }
    }

    /// Its numbers, if it is Shroud.
    pub fn shroud(&self) -> Option<&ShroudDef> {
        match self {
            SpellDef::Shroud(shroud) => Some(shroud),
            SpellDef::Cast(_) => None,
        }
    }

    /// (M5-N11) The ability it casts, if it is a cast.
    pub fn cast(&self) -> Option<AreaAbility> {
        match self {
            SpellDef::Cast(ability) => Some(*ability),
            SpellDef::Shroud(_) => None,
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
