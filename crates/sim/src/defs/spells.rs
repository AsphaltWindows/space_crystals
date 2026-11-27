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
//! - (M6-CM) **Mirage** ([`SpellDef::Mirage`]) is an ability, applied at once in the commands phase, with no target
//!   (`Ability::Mirage`, `crate::mirage`): it spends its cost then and makes the Disciple's illusions round it.

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
    /// (M6-CM) `factions/cults/units/disciple.md > Spells > Mirage`: "An **instant** spell: the Disciple creates **4
    /// illusions** of itself" (`crate::mirage`; its command, `Ability::Mirage`).
    Mirage(MirageDef),
}

impl SpellDef {
    /// The energy it needs to start, points: Shroud's "needs at least **10 energy** to start"; (M5-N11) a cast's, its
    /// cost (Inspire's "75 energy", Suppress's "100 energy"), and (M6-CM) Mirage's ("**50 energy**"), all of which it
    /// spends.
    pub fn needs(&self) -> f32 {
        match self {
            SpellDef::Shroud(shroud) => shroud.needs,
            SpellDef::Cast(ability) => ability.def().cost,
            SpellDef::Mirage(mirage) => mirage.cost,
        }
    }

    /// Its numbers, if it is Shroud.
    pub fn shroud(&self) -> Option<&ShroudDef> {
        match self {
            SpellDef::Shroud(shroud) => Some(shroud),
            SpellDef::Cast(_) | SpellDef::Mirage(_) => None,
        }
    }

    /// (M5-N11) The ability it casts, if it is a cast.
    pub fn cast(&self) -> Option<AreaAbility> {
        match self {
            SpellDef::Cast(ability) => Some(*ability),
            SpellDef::Shroud(_) | SpellDef::Mirage(_) => None,
        }
    }

    /// (M6-CM) Its numbers, if it is Mirage.
    pub fn mirage(&self) -> Option<&MirageDef> {
        match self {
            SpellDef::Mirage(mirage) => Some(mirage),
            SpellDef::Shroud(_) | SpellDef::Cast(_) => None,
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

/// (M6-CM) `factions/cults/units/disciple.md > Placeholder stats > Mirage numbers`, and the illusions' own number
/// (`factions/cults/units/disciple.md > Spells > Mirage`; `crate::mirage`).
#[derive(Copy, Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct MirageDef {
    /// "**50 energy**", points, spent at the cast.
    pub cost: f32,
    /// "the Disciple creates **4 illusions** of itself", "one on each side of the Disciple, **north, east, south and
    /// west**": as many, evenly round it clockwise from the north (`mirage::compass`).
    pub illusions: u32,
    /// "illusions appear **1 grid unit** from the Disciple's centre (centre to centre)", grid units.
    pub offset: f32,
    /// "illusions take **double damage from all sources** (Shroud releases included)": what the HP an illusion loses
    /// to a hit is multiplied by, after every step of `systems/combat.md > DamageModifierOrder`
    /// (`combat::damage::apply`).
    pub damage_taken: f32,
}
