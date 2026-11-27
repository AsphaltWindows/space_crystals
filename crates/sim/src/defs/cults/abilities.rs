//! (M5-N5) The Cults' area abilities (plan decisions T75 and T76): `factions/cults/structures/shrine.md > Abilities`,
//! "Both abilities are cast on an area anywhere the player currently has vision, and spend Shrine energy". The
//! abilities ([`AreaAbility`]) and each one's numbers ([`AbilityDef`]): data only, as the technologies are
//! (`defs/cults/research.rs`). A structure type lists those it casts (`StructureDef::abilities`), a cast is a
//! `Command::Cast`, and the units it reaches hold its buff for its duration (`Object::buffs`, `crate::buffs`).
//!
//! (M5-N11) The Prayer-mobile's two are area abilities too, cast round it or ahead of it rather than at a point
//! ([`AbilityArea`]), by a unit that lists them among its spells (`SpellDef::Cast`). Their numbers are its def's
//! (`defs::cults::units::prayer_mobile`); what Suppress's status does is `crate::suppress`'s.

use serde::{Deserialize, Serialize};

use crate::defs::ObjectKind;
use crate::defs::cults::units::prayer_mobile::{INSPIRE, SUPPRESS};
use crate::hash::{StateHash, StateHasher};
use crate::ruleset::{Key, Table};
use crate::units::{Tick, seconds};

/// An area ability (`factions/cults/structures/shrine.md > Abilities`): the two of the Shrine's table, and (M5-N11) the
/// Prayer-mobile's two. Its numbers are its [`AbilityDef`] ([`AreaAbility::def`]). In files by its name, hashed by its
/// place in the list.
#[derive(Copy, Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub enum AreaAbility {
    /// "Health regeneration (name TBD)".
    HealthRegeneration,
    /// "Energy regeneration (name TBD)".
    EnergyRegeneration,
    /// (M5-N11) `factions/cults/units/prayer_mobile.md > Abilities`: "**Inspire**: buffs nearby own infantry with
    /// **increased movement speed**, and **Recruits** also with **increased attack speed**".
    Inspire,
    /// (M5-N11) `factions/cults/units/prayer_mobile.md > Abilities`: "**Suppress**: a screech in an **arc in front of
    /// it**. Affected units **temporarily can't attack**" (`crate::suppress`).
    Suppress,
}

impl AreaAbility {
    /// The four, the Shrine's in the order of its keys ("**Health regeneration** (hotkey Q)", "**Energy regeneration**
    /// (hotkey W)"), then the Prayer-mobile's in the order of its ("**Inspire** (hotkey **R**", "**Suppress** (hotkey
    /// **F**").
    pub const ALL: [AreaAbility; 4] = [
        AreaAbility::HealthRegeneration,
        AreaAbility::EnergyRegeneration,
        AreaAbility::Inspire,
        AreaAbility::Suppress,
    ];

    /// Its numbers, in the current rules (`crate::ruleset::rules`).
    #[inline]
    pub fn def(self) -> &'static AbilityDef {
        &crate::ruleset::rules().factions.cults.abilities[self]
    }

    /// Whether its buff goes to an object of `kind` (the table's "Affects", [`Affects`]), whoever's it is.
    pub fn affects(self, kind: ObjectKind) -> bool {
        self.def().affects.kinds(kind)
    }

    /// How long its buff lasts, in ticks.
    pub fn duration(self) -> Tick {
        seconds(self.def().duration)
    }
}

impl Key for AreaAbility {
    const ALL: &'static [AreaAbility] = &AreaAbility::ALL;

    #[inline]
    fn index(self) -> usize {
        self as usize
    }
}

/// Every area ability's numbers, by ability (`CultsRules::abilities`).
pub type Abilities = Table<AreaAbility, AbilityDef, 4>;

/// The built-in area abilities' numbers, in [`AreaAbility::ALL`]'s order: the Shrine's here, the Prayer-mobile's in
/// its def module.
pub const ABILITIES: Abilities = Abilities::new([HEALTH_REGENERATION, ENERGY_REGENERATION, INSPIRE, SUPPRESS]);

impl StateHash for AreaAbility {
    fn hash(&self, h: &mut StateHasher) {
        // Fieldless enum: the discriminant is the declaration index.
        (*self as u32).hash(h);
    }
}

/// What an ability's buff does every tick it lasts (`crate::buffs`).
#[derive(Copy, Clone, Debug, PartialEq, Serialize, Deserialize)]
pub enum BuffEffect {
    /// So many HP a second, up to the unit's MaxHP with its Augmentations (`Object::max_hp_with`).
    Health { per_second: f32 },
    /// So many energy points a second, beside the unit's own regeneration, up to its MaxEnergy (`crate::energy`).
    Energy { per_second: f32 },
    /// (M5-N11) Inspire's: `max_speed` space units a tick on the unit's MaxSpeed, beside its Augmentations'
    /// (`Object::movement`), and for a unit of a type in `faster_attacks` its own attack's cycle times `attack_cycle`
    /// (`combat::weapon`, `crate::buffs::inspired`).
    Inspire {
        max_speed: f32,
        attack_cycle: f32,
        #[serde(deserialize_with = "crate::defs::leak::slice")]
        faster_attacks: &'static [ObjectKind],
    },
    /// (M5-N11) Suppress's: the unit starts no attack while it lasts (`crate::suppress`).
    HoldFire,
}

/// (M5-N11) Where an ability's area lies as it is cast, its radius its [`AbilityDef::radius`].
#[derive(Copy, Clone, Debug, PartialEq, Serialize, Deserialize)]
pub enum AbilityArea {
    /// A circle round the point the player casts it at: the Shrine's ("cast on an area anywhere the player currently
    /// has vision").
    AtPoint,
    /// A circle round its caster: Inspire's (`factions/cults/units/prayer_mobile.md > Placeholder stats`: "within **4
    /// grid units** of the vehicle", "instant, no target").
    AroundCaster,
    /// An arc of `arc` radians in all, centred on its caster's facing, from its caster's centre out to the radius:
    /// Suppress's ("an arc of **90° total** in front of the vehicle's current facing, **5 grid units** long").
    Ahead { arc: f32 },
}

/// (M5-N11) Whose objects an ability's buff goes to (the table's "Affects").
#[derive(Copy, Clone, Debug, PartialEq, Serialize, Deserialize)]
pub enum Affects {
    /// The caster's owner's units of these types: "Only the caster's own units are affected" (the Shrine's), "every
    /// own infantry type" (Inspire's).
    Own(#[serde(deserialize_with = "crate::defs::leak::slice")] &'static [ObjectKind]),
    /// Every unit, whoever owns it, in the air too, and no building: Suppress's
    /// (`factions/cults/units/prayer_mobile.md > Abilities`: "**Affects every unit in the arc, enemies and own alike**
    /// [...] **Not buildings** [...] **Air units in the arc are affected** too").
    EveryUnit,
}

impl Affects {
    /// Whether it takes in an object of `kind`, whoever's it is.
    pub fn kinds(self, kind: ObjectKind) -> bool {
        match self {
            Affects::Own(kinds) => kinds.contains(&kind),
            Affects::EveryUnit => kind.def().unit.is_some(),
        }
    }

    /// Whether only the caster's owner's objects are taken in.
    pub fn own_only(self) -> bool {
        matches!(self, Affects::Own(_))
    }
}

/// An area ability's numbers (plan decision T75): `factions/cults/structures/shrine.md > Abilities`, a row of its
/// table, and `> Placeholder stats`. "Cooldowns: none."
#[derive(Copy, Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct AbilityDef {
    /// Its name, as the table has it ("name TBD").
    #[serde(deserialize_with = "crate::defs::leak::str")]
    pub name: crate::defs::StaticStr,
    /// What it spends of its caster's energy, points: "Energy cost".
    pub cost: f32,
    /// (M5-N11) Where its area lies ([`AbilityArea`]): the Shrine's at a point.
    pub area: AbilityArea,
    /// The radius of its circle, grid units: "Area". A unit is in it when its centre is ("**In the circle** (user,
    /// 2026-11-26; Q0118): a unit is affected when its **centre** is inside"). (M5-N11) An arc's length.
    pub radius: f32,
    /// The types its buff goes to (the table's "Affects"): "Only the caster's own units are affected". (M5-N11)
    /// Suppress's, every unit ([`Affects`]).
    pub affects: Affects,
    /// What its buff does.
    pub effect: BuffEffect,
    /// How long its buff lasts, seconds: "Duration".
    pub duration: f32,
}

/// "Health regeneration (name TBD) | Own Cults infantry | Grants health regeneration (+2 HP/s, placeholder) for the
/// duration — Cults infantry have no natural regeneration | 75 | radius 4 grid units | 15 s"; `> Placeholder stats`:
/// "**Health regeneration** (hotkey Q): 75 energy; circle of radius 4 grid units; **+2 HP per second for 15
/// seconds**".
const HEALTH_REGENERATION: AbilityDef = AbilityDef {
    name: "Health regeneration",
    cost: 75.0,
    area: AbilityArea::AtPoint,
    radius: 4.0,
    // "Own Cults infantry": `factions/cults/structures/README.md > Cults Infantry`, "The Cults have four infantry unit
    // types: Recruit, Militant, Gunner, and Disciple".
    affects: Affects::Own(&[
        ObjectKind::Recruit,
        ObjectKind::Militant,
        ObjectKind::Gunner,
        ObjectKind::Disciple,
    ]),
    effect: BuffEffect::Health { per_second: 2.0 },
    duration: 15.0,
};

/// "Energy regeneration (name TBD) | Own Disciples | Increases Disciple energy regeneration (+1/s, placeholder) for the
/// duration | 50 | radius 4 grid units | 20 s"; `> Placeholder stats`: "**Energy regeneration** (hotkey W): 50 energy;
/// circle of radius 4 grid units; affected Disciples' energy regeneration is **+1 per second for 20 seconds**".
const ENERGY_REGENERATION: AbilityDef = AbilityDef {
    name: "Energy regeneration",
    cost: 50.0,
    area: AbilityArea::AtPoint,
    radius: 4.0,
    affects: Affects::Own(&[ObjectKind::Disciple]),
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
    /// for 20 s, 320 ticks of 4096 units, 20 points; both in a circle of radius 4 grid units round a point. (M5-N11)
    /// The Prayer-mobile's after them (`defs::cults::units::prayer_mobile`'s tests): Inspire round it, Suppress ahead.
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
                ("Inspire", 75.0, 4.0, 10.0, 160),
                ("Suppress", 100.0, 5.0, 4.0, 64),
            ]
        );
        let health = AreaAbility::HealthRegeneration;
        const INFANTRY: [ObjectKind; 4] = [
            ObjectKind::Recruit,
            ObjectKind::Militant,
            ObjectKind::Gunner,
            ObjectKind::Disciple,
        ];
        assert_eq!(health.def().affects, Affects::Own(&INFANTRY));
        assert_eq!(health.def().effect, BuffEffect::Health { per_second: 2.0 });
        assert_eq!(2.0 / 16.0 * 240.0, 30.0, "0.125 a tick, exact in an f32");
        let energy_regeneration = AreaAbility::EnergyRegeneration;
        assert_eq!(energy_regeneration.def().affects, Affects::Own(&[ObjectKind::Disciple]));
        assert!(!energy_regeneration.affects(ObjectKind::Recruit));
        assert_eq!(energy_regeneration.def().effect, BuffEffect::Energy { per_second: 1.0 });
        assert_eq!(energy::per_tick(1.0), 4096);
        assert_eq!(energy::per_tick(1.0) * 320, energy::units(20.0), "20 points, exactly");
        for shrine in [health, energy_regeneration] {
            assert_eq!(shrine.def().area, AbilityArea::AtPoint, "{shrine:?}");
            assert!(shrine.def().affects.own_only());
        }
        // The costs compare exactly with the pool (plan decision T75: "The thresholds (10, 50, 75) compare exactly").
        assert_eq!(energy::units(75.0), 75 * energy::ONE);
        assert_eq!(energy::units(50.0), 50 * energy::ONE);
        assert_eq!(energy::units(100.0), 100 * energy::ONE);
    }

    /// (M5-N11) Every unit, of any faction, in the air too, is a kind Suppress affects, and no structure nor map
    /// resource is; the Shrine's abilities and Inspire take only their own types.
    #[test]
    fn suppress_affects_every_unit_and_nothing_else() {
        for &kind in ObjectKind::ALL {
            let unit = kind.def().unit.is_some();
            assert_eq!(AreaAbility::Suppress.affects(kind), unit, "{kind:?}");
        }
        assert!(AreaAbility::Suppress.affects(ObjectKind::Sparrow), "in the air");
        assert!(!AreaAbility::Suppress.affects(ObjectKind::Garrison), "a building");
        assert!(!AreaAbility::Suppress.def().affects.own_only(), "enemies and own alike");
        assert!(AreaAbility::Inspire.def().affects.own_only());
    }
}
