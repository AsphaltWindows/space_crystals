//! The Disciple — `factions/cults/units/disciple.md`.

use crate::defs::units::{Augmentable, MovementAttrs, UnitBase, UnitDef};
use crate::defs::{Armor, Category, EnergyDef, EnergyRegen, Footprint, ObjectDef, ObjectKind, ShroudDef, SpellDef};
use crate::math::PI;
use crate::player::Faction;

/// `factions/cults/units/disciple.md`: "The Cults' spell caster, trained from a single Recruit at the Cathedral". Its
/// numbers (slice M5-G2, plan decision T69), and since slice M5-N4a its energy pool (`> Energy`: "The Disciple has its
/// own energy pool, which it spends on its spells. It regenerates naturally over time", `crate::energy`), which the
/// Cathedral trains it with (`factions/cults/structures/cathedral.md > Training`), and since slice M5-N4b its spell
/// (`> Spells > Shroud`, `crate::shroud`). Its numbers are the design's "Placeholder stats".
pub(crate) static DISCIPLE: ObjectDef = ObjectDef {
    kind: ObjectKind::Disciple,
    name: "Disciple",
    faction: Some(Faction::Cults),                         // Faction - TheCults
    category: Category::Unit,                              // Entity Type - Unit
    footprint: Footprint::Silhouette { w: 24.0, h: 24.0 }, // Silhouette - 24x24 space units (square)
    selectable: true,                                      // `core/entities.md > Object Type > Selectable - true`
    // Not listed; a type with MaxHP is destructible (`core/entities.md > Object Instance > HP`).
    destructible: true,
    // Groupable - false (user, 2026-11-26; Q0112: each R press starts one Disciple's channel; Tab to the next)
    groupable: false,
    sight_range: 6.0,   // SightRange - 6 grid units
    max_hp: Some(40.0), // MaxHP - 40
    max_shields: None,
    armor: Armor {
        point: 0.0, // PointArmor - 0
        full: 0.0,  // FullArmor - 0
    },
    needs_flat_ground: false,
    unit: Some(UnitDef {
        base: UnitBase::LightInfantry, // UnitBase - LightInfantry
        // UnitBaseAttributes[LightInfantry]: as the Militant (TurnRate 180°/frame, infinite acceleration, MaxSpeed 4
        // space units/frame, RuggedTerrainDefenseBonus 50%)
        movement: MovementAttrs::TurnRate {
            turn_rate: PI,
            acceleration: None,
            deceleration: None,
            max_speed: 4.0,
        },
        rugged_terrain_defense_bonus: Some(0.5),
        turret: None, // TurretAttributes - None
        // AttackAttributes - None (placeholder: no weapon; for Augmentation it is **armed**, i.e. it gets only the MaxHP
        // and speed parts of increments. To confirm with the user.)
        attack: None,
        // `> Trained at - Cathedral (consumes 1 Recruit; Unit Control cost 1)`: "A trained unit's Unit Control cost
        // equals the number of Recruits consumed to produce it" (`factions/cults/structures/recruitment_hall.md > Unit
        // Control Tracking`; plan decision T55).
        unit_control_cost: Some(1),
        tunnel_space_cost: None,
        beacon_capacity_cost: None,
        riot_shield: None,
        conversion: None,
        deploy: None,
        worker: None,
        // (M5-N1a) `factions/cults/units/disciple.md > AttackAttributes`: "for Augmentation it is **armed**".
        augmentable: Some(Augmentable { unarmed: false }),
        repairs: None,
        // (M5-N4b) `> Spells > Shroud`; Shroud numbers - cloud radius 3 grid units around the Disciple; drains 5 energy
        // per second; [...] needs at least 10 energy to start
        spells: &[SpellDef::Shroud(ShroudDef {
            radius: 3.0,
            drain: 5.0,
            needs: 10.0,
        })],
    }),
    structure: None,
    default_settings: None,
    // Energy - MaxEnergy 200; starts at 50; regenerates 0.5 per second
    energy: Some(EnergyDef {
        max: 200.0,
        start: 50.0,
        regen: EnergyRegen::PerSecond(0.5),
    }),
};

#[cfg(test)]
mod tests {
    use glam::Vec2;

    use super::*;

    /// The Cults' infantry body (LightInfantry, 24×24, MaxHP 40, no armour, MaxSpeed 4, RuggedTerrainDefenseBonus 50%),
    /// with SightRange 6, not Groupable (Q0112), and no weapon nor turret; Unit Control 1, the one Recruit it is
    /// trained from; (M5-N4a) its energy pool: "MaxEnergy 200; starts at 50; regenerates 0.5 per second"; (M5-N4b) its
    /// Shroud: "cloud radius **3 grid units** around the Disciple; drains **5 energy per second**; [...] needs at least
    /// **10 energy** to start".
    #[test]
    fn the_disciple_follows_the_design() {
        let def = ObjectKind::Disciple.def();
        assert_eq!(
            (def.name, def.category, def.faction),
            ("Disciple", Category::Unit, Some(Faction::Cults))
        );
        assert_eq!(def.silhouette(), Some(Vec2::new(24.0, 24.0)));
        assert_eq!(
            (def.max_hp, def.max_shields, def.armor, def.sight_range),
            (Some(40.0), None, Armor::NONE, 6.0)
        );
        assert_eq!((def.selectable, def.destructible, def.groupable), (true, true, false));
        let unit = def.unit.as_ref().unwrap();
        let militant = ObjectKind::Militant.def().unit.as_ref().unwrap();
        assert_eq!(unit.base, UnitBase::LightInfantry);
        assert_eq!(
            (unit.movement, unit.rugged_terrain_defense_bonus),
            (militant.movement, Some(0.5)),
            "as the Militant"
        );
        assert_eq!((unit.turret, unit.attack, unit.worker), (None, None, None));
        assert_eq!(
            (
                unit.unit_control_cost,
                unit.tunnel_space_cost,
                unit.beacon_capacity_cost
            ),
            (Some(1), None, None)
        );
        assert_eq!(
            def.energy,
            Some(EnergyDef {
                max: 200.0,
                start: 50.0,
                regen: EnergyRegen::PerSecond(0.5)
            })
        );
        let shroud = ShroudDef {
            radius: 3.0,
            drain: 5.0,
            needs: 10.0,
        };
        assert_eq!(
            (unit.spells, unit.shroud()),
            (&[SpellDef::Shroud(shroud)][..], Some(&shroud))
        );
        assert_eq!(unit.spells[0].needs(), 10.0);
        let casters = (ObjectKind::ALL.iter())
            .filter(|kind| (kind.def().unit.as_ref()).is_some_and(|unit| !unit.spells.is_empty()));
        assert_eq!(
            casters.collect::<Vec<_>>(),
            [&ObjectKind::Disciple],
            "the one caster so far"
        );
    }
}
