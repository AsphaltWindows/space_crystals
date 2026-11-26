//! (M5-N5) The Cults' area abilities (plan decisions T75 and T76): `factions/cults/structures/shrine.md > Abilities`,
//! "Both abilities are cast on an area anywhere the player currently has vision, and spend Shrine energy". The
//! abilities ([`AreaAbility`]) and each one's numbers ([`AbilityDef`]): data only, as the technologies are
//! (`defs/cults/research.rs`). A structure type lists those it casts (`StructureDef::abilities`), a cast is a
//! `Command::Cast`, and the units it reaches hold its buff for its duration (`Object::buffs`, `crate::buffs`).

use serde::{Deserialize, Serialize};

use crate::defs::ObjectKind;
use crate::hash::{StateHash, StateHasher};
use crate::units::{Tick, seconds};

/// An area ability (`factions/cults/structures/shrine.md > Abilities`): the two of the Shrine's table. Its numbers are
/// its [`AbilityDef`] ([`AreaAbility::def`]). In files by its name, hashed by its place in the list.
#[derive(Copy, Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub enum AreaAbility {
    /// "Health regeneration (name TBD)".
    HealthRegeneration,
    /// "Energy regeneration (name TBD)".
    EnergyRegeneration,
}

impl AreaAbility {
    /// The two, in the order of the Shrine's keys: "**Health regeneration** (hotkey Q)", "**Energy regeneration**
    /// (hotkey W)".
    pub const ALL: [AreaAbility; 2] = [AreaAbility::HealthRegeneration, AreaAbility::EnergyRegeneration];

    /// Its numbers.
    pub fn def(self) -> &'static AbilityDef {
        match self {
            AreaAbility::HealthRegeneration => &HEALTH_REGENERATION,
            AreaAbility::EnergyRegeneration => &ENERGY_REGENERATION,
        }
    }

    /// Whether its buff goes to a unit of `kind` (the table's "Affects"), its caster's owner's.
    pub fn affects(self, kind: ObjectKind) -> bool {
        self.def().affects.contains(&kind)
    }

    /// How long its buff lasts, in ticks.
    pub fn duration(self) -> Tick {
        seconds(self.def().duration)
    }
}

impl StateHash for AreaAbility {
    fn hash(&self, h: &mut StateHasher) {
        // Fieldless enum: the discriminant is the declaration index.
        (*self as u32).hash(h);
    }
}

/// What an ability's buff does every tick it lasts (`crate::buffs`).
#[derive(Copy, Clone, Debug, PartialEq)]
pub enum BuffEffect {
    /// So many HP a second, up to the unit's MaxHP with its Augmentations (`Object::max_hp_with`).
    Health { per_second: f32 },
    /// So many energy points a second, beside the unit's own regeneration, up to its MaxEnergy (`crate::energy`).
    Energy { per_second: f32 },
}

/// An area ability's numbers (plan decision T75): `factions/cults/structures/shrine.md > Abilities`, a row of its
/// table, and `> Placeholder stats`. "Cooldowns: none."
#[derive(Copy, Clone, Debug, PartialEq)]
pub struct AbilityDef {
    /// Its name, as the table has it ("name TBD").
    pub name: &'static str,
    /// What it spends of its caster's energy, points: "Energy cost".
    pub cost: f32,
    /// The radius of its circle, grid units: "Area". A unit is in it when its centre is ("**In the circle** (user,
    /// 2026-11-26; Q0118): a unit is affected when its **centre** is inside").
    pub radius: f32,
    /// The types its buff goes to (the table's "Affects"): "Only the caster's own units are affected".
    pub affects: &'static [ObjectKind],
    /// What its buff does.
    pub effect: BuffEffect,
    /// How long its buff lasts, seconds: "Duration".
    pub duration: f32,
}

/// "Health regeneration (name TBD) | Own Cults infantry | Grants health regeneration (+2 HP/s, placeholder) for the
/// duration — Cults infantry have no natural regeneration | 75 | radius 4 grid units | 15 s"; `> Placeholder stats`:
/// "**Health regeneration** (hotkey Q): 75 energy; circle of radius 4 grid units; **+2 HP per second for 15
/// seconds**".
static HEALTH_REGENERATION: AbilityDef = AbilityDef {
    name: "Health regeneration",
    cost: 75.0,
    radius: 4.0,
    // "Own Cults infantry": `factions/cults/structures/README.md > Cults Infantry`, "The Cults have four infantry unit
    // types: Recruit, Militant, Gunner, and Disciple".
    affects: &[
        ObjectKind::Recruit,
        ObjectKind::Militant,
        ObjectKind::Gunner,
        ObjectKind::Disciple,
    ],
    effect: BuffEffect::Health { per_second: 2.0 },
    duration: 15.0,
};

/// "Energy regeneration (name TBD) | Own Disciples | Increases Disciple energy regeneration (+1/s, placeholder) for the
/// duration | 50 | radius 4 grid units | 20 s"; `> Placeholder stats`: "**Energy regeneration** (hotkey W): 50 energy;
/// circle of radius 4 grid units; affected Disciples' energy regeneration is **+1 per second for 20 seconds**".
static ENERGY_REGENERATION: AbilityDef = AbilityDef {
    name: "Energy regeneration",
    cost: 50.0,
    radius: 4.0,
    affects: &[ObjectKind::Disciple],
    effect: BuffEffect::Energy { per_second: 1.0 },
    duration: 20.0,
};

#[cfg(test)]
mod tests {
    use super::*;
    use crate::energy;

    /// `factions/cults/structures/shrine.md > Abilities` and `> Placeholder stats`: Health regeneration for 75 energy,
    /// on own Cults infantry (the four of `factions/cults/structures/README.md > Cults Infantry`), +2 HP a second for
    /// 15 s, 240 ticks of 0.125 HP, 30 HP in all, exactly; Energy regeneration for 50, on own Disciples, +1 a second
    /// for 20 s, 320 ticks of 4096 units, 20 points; both in a circle of radius 4 grid units.
    #[test]
    fn the_abilities_follow_the_design() {
        let row = |ability: AreaAbility| {
            let def = ability.def();
            (def.name, def.cost, def.radius, def.duration, ability.duration())
        };
        assert_eq!(
            AreaAbility::ALL.map(row),
            [
                ("Health regeneration", 75.0, 4.0, 15.0, 240),
                ("Energy regeneration", 50.0, 4.0, 20.0, 320),
            ]
        );
        let health = AreaAbility::HealthRegeneration;
        let infantry = [
            ObjectKind::Recruit,
            ObjectKind::Militant,
            ObjectKind::Gunner,
            ObjectKind::Disciple,
        ];
        assert_eq!(health.def().affects, infantry);
        assert_eq!(health.def().effect, BuffEffect::Health { per_second: 2.0 });
        assert_eq!(2.0 / 16.0 * 240.0, 30.0, "0.125 a tick, exact in an f32");
        let energy_regeneration = AreaAbility::EnergyRegeneration;
        assert_eq!(energy_regeneration.def().affects, [ObjectKind::Disciple]);
        assert!(!energy_regeneration.affects(ObjectKind::Recruit));
        assert_eq!(energy_regeneration.def().effect, BuffEffect::Energy { per_second: 1.0 });
        assert_eq!(energy::per_tick(1.0), 4096);
        assert_eq!(energy::per_tick(1.0) * 320, energy::units(20.0), "20 points, exactly");
        // The costs compare exactly with the pool (plan decision T75: "The thresholds (10, 50, 75) compare exactly").
        assert_eq!(energy::units(75.0), 75 * energy::ONE);
        assert_eq!(energy::units(50.0), 50 * energy::ONE);
    }
}
