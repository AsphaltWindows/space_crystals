//! The Prayer-mobile — `factions/cults/units/prayer_mobile.md` (design changes 0364 and 0365): its numbers, and
//! (M5-N11) those of its two abilities, Inspire ([`INSPIRE`]) and Suppress ([`SUPPRESS`]), which `AreaAbility::def`
//! reads.

use crate::defs::cults::abilities::{AbilityArea, AbilityDef, Affects, BuffEffect};
use crate::defs::units::{MovementAttrs, UnitBase, UnitDef};
use crate::defs::{AreaAbility, Armor, Category, EnergyDef, EnergyRegen, Footprint, ObjectDef, ObjectKind, SpellDef};
use crate::math::PI;
use crate::player::Faction;

/// `factions/cults/units/prayer_mobile.md`: "A Cults **energy-based caster vehicle**: a wheeled vehicle carrying a
/// **megaphone**, with **no weapon** (a pure caster)". Its numbers (slice M5-G2, plan decision T69), its movement as
/// data, which slice M5-V2's wheeled movement drives it by; since slice M5-N11 its energy (`> Energy - has an energy
/// pool for its abilities`, `crate::energy`), its two abilities (`> Abilities`: Inspire and Suppress, each cast at once
/// by a `Command::Cast`, `crate::buffs`) and its production at the Chop Shop, where its TechPrerequisite is its
/// recipe's (`> TechPrerequisite - player owns at least one **Temple**`, `Recipe::requires`). Its numbers are the
/// design's "Placeholder stats".
pub(crate) const PRAYER_MOBILE: ObjectDef = ObjectDef {
    kind: ObjectKind::PrayerMobile,
    name: "Prayer-mobile",
    faction: Some(Faction::Cults), // Faction - TheCults
    category: Category::Unit,      // Entity Type - Unit
    // "Silhouette 56x40 space units", the Seeker's, its length along its facing (`systems/units.md > Silhouette
    // Rotation`), turned with the facing (slice M5-V1, `ObjectDef::silhouette_turns`).
    footprint: Footprint::Silhouette { w: 56.0, h: 40.0 },
    selectable: true, // `core/entities.md > Object Type > Selectable - true`
    // Not listed; a type with MaxHP is destructible (`core/entities.md > Object Instance > HP`).
    destructible: true,
    // **Groupable false** (one cast spends one vehicle's energy, as for the Disciple)
    groupable: false,
    sight_range: 7.0,    // SightRange 7 grid units
    max_hp: Some(150.0), // MaxHP 150
    max_shields: None,
    armor: Armor {
        point: 1.0, // PointArmor 1
        full: 1.0,  // FullArmor 1
    },
    needs_flat_ground: false,
    unit: Some(UnitDef {
        base: UnitBase::WheeledVehicle, // UnitBase - WheeledVehicle
        // "FixedTurnRadiusMovement as the Seeker, but ForwardMaxSpeed **6 su/frame**"
        // (`factions/cults/units/seeker.md`: "MinimumTurnRadius 64 su (1 grid unit); ForwardMaxSpeed 8 su/frame;
        // ForwardAcceleration 1 su/frame²; ReverseMaxSpeed 3 su/frame; ReverseAcceleration 0.5 su/frame²; Deceleration
        // 1 su/frame²").
        movement: MovementAttrs::FixedTurnRadius {
            minimum_turn_radius: 64.0,
            forward_acceleration: Some(1.0),
            forward_max_speed: 6.0,
            reverse_acceleration: Some(0.5),
            reverse_max_speed: 3.0,
            deceleration: Some(1.0),
        },
        rugged_terrain_defense_bonus: None, // only LightInfantry has it
        turret: None,                       // "with **no weapon** (a pure caster)": nothing to turn
        attack: None,                       // AttackAttributes - None (pure caster)
        // `> Produced at - Chop Shop (consumes **2 Recruits**; Unit Control cost **2**)` (plan decision T55).
        unit_control_cost: Some(2),
        tunnel_space_cost: None,
        beacon_capacity_cost: None,
        riot_shield: None,
        conversion: None,
        deploy: None,
        worker: None,
        augmentable: None,
        repairs: None,
        // `> Placeholder stats`: "**Inspire** (hotkey **R**, instant, no target)", "**Suppress** (hotkey **F**,
        // instant, no target)": each cast at once, from where the vehicle stands (`Command::Cast`).
        spells: &[
            SpellDef::Cast(AreaAbility::Inspire),
            SpellDef::Cast(AreaAbility::Suppress),
        ],
        separation_radius: None,
        target_lock: None,
    }),
    structure: None,
    default_settings: None,
    // "Energy: MaxEnergy 200; starts at 50; regenerates 0.5 per second."
    energy: Some(EnergyDef {
        max: 200.0,
        start: 50.0,
        regen: EnergyRegen::PerSecond(0.5),
    }),
};

/// (M5-N11) `> Placeholder stats`: "**Inspire** (hotkey **R**, instant, no target): 75 energy; every own infantry unit
/// within **4 grid units** of the vehicle gets **+1.5 su/frame MaxSpeed** for **10 s**; Recruits also attack **25%
/// faster** (their attack cycle × 0.75) for the same time"; `> Abilities`: "**Who** (user, 2026-11-27): every own
/// infantry type (Recruit, Militant, Gunner, Disciple) gets the movement speed; Recruits also get the attack speed".
/// "**Stacks with augmentation** (e.g. Improved Metabolism): different sources add up. **Recasting** on units that
/// are already inspired **restarts the duration**; Inspire doesn't stack with itself" (`crate::buffs`).
pub(crate) const INSPIRE: AbilityDef = AbilityDef {
    name: "Inspire",
    cost: 75.0,
    area: AbilityArea::AroundCaster,
    radius: 4.0,
    affects: Affects::Own(&[
        ObjectKind::Recruit,
        ObjectKind::Militant,
        ObjectKind::Gunner,
        ObjectKind::Disciple,
    ]),
    effect: BuffEffect::Inspire {
        max_speed: 1.5,
        attack_cycle: 0.75,
        faster_attacks: &[ObjectKind::Recruit],
    },
    duration: 10.0,
};

/// (M5-N11) `> Placeholder stats`: "**Suppress** (hotkey **F**, instant, no target): 100 energy; an arc of **90°
/// total** in front of the vehicle's current facing, **5 grid units** long; units whose centre is inside can't attack
/// for **4 s**"; `> Abilities`: "**Affects every unit in the arc, enemies and own alike** (user, 2026-11-27), like
/// friendly fire. **Not buildings** [...] **Air units in the arc are affected** too" (`crate::suppress`).
pub(crate) const SUPPRESS: AbilityDef = AbilityDef {
    name: "Suppress",
    cost: 100.0,
    area: AbilityArea::Ahead { arc: PI / 2.0 },
    radius: 5.0,
    affects: Affects::EveryUnit,
    effect: BuffEffect::HoldFire,
    duration: 4.0,
};

#[cfg(test)]
mod tests {
    use glam::Vec2;

    use super::*;

    /// A 56×40 WheeledVehicle, MaxHP 150, PointArmor 1, FullArmor 1, SightRange 7, not Groupable; the Seeker's
    /// FixedTurnRadiusMovement but 6 su a frame forward; no weapon and no turret; Unit Control 2, the Recruits it is
    /// made of; (M5-N11) a pool of 200 energy from 50, +0.5 a second, and its two spells, Inspire and Suppress.
    #[test]
    fn the_prayer_mobile_follows_the_design() {
        let def = ObjectKind::PrayerMobile.def();
        assert_eq!(
            (def.name, def.category, def.faction),
            ("Prayer-mobile", Category::Unit, Some(Faction::Cults))
        );
        assert_eq!(def.silhouette(), Some(Vec2::new(56.0, 40.0)));
        let one = Armor { point: 1.0, full: 1.0 };
        assert_eq!(
            (def.max_hp, def.max_shields, def.armor, def.sight_range),
            (Some(150.0), None, one, 7.0)
        );
        assert_eq!((def.selectable, def.destructible, def.groupable), (true, true, false));
        let unit = def.unit.as_ref().unwrap();
        assert_eq!(unit.base, UnitBase::WheeledVehicle);
        // The Seeker's FixedTurnRadiusMovement, with a ForwardMaxSpeed of 6.
        let seekers = match ObjectKind::Seeker.def().unit.as_ref().unwrap().movement {
            MovementAttrs::FixedTurnRadius {
                minimum_turn_radius,
                forward_acceleration,
                reverse_acceleration,
                reverse_max_speed,
                deceleration,
                ..
            } => MovementAttrs::FixedTurnRadius {
                minimum_turn_radius,
                forward_acceleration,
                forward_max_speed: 6.0,
                reverse_acceleration,
                reverse_max_speed,
                deceleration,
            },
            other => panic!("the Seeker's is {other:?}"),
        };
        assert_eq!(unit.movement, seekers, "as the Seeker, but ForwardMaxSpeed 6");
        assert_eq!((unit.turret, unit.attack, unit.worker), (None, None, None));
        assert_eq!(
            (
                unit.unit_control_cost,
                unit.tunnel_space_cost,
                unit.beacon_capacity_cost
            ),
            (Some(2), None, None)
        );
        let energy = def.energy.expect("its pool");
        assert_eq!(
            (energy.max, energy.start, energy.regen),
            (200.0, 50.0, EnergyRegen::PerSecond(0.5))
        );
        assert_eq!(
            unit.spells,
            [
                SpellDef::Cast(AreaAbility::Inspire),
                SpellDef::Cast(AreaAbility::Suppress)
            ]
        );
        assert_eq!(
            unit.spells.iter().map(SpellDef::needs).collect::<Vec<_>>(),
            [75.0, 100.0]
        );
    }

    /// (M5-N11) Inspire: 75 energy, round the vehicle, 4 grid units, own infantry, +1.5 su a tick and the Recruits'
    /// attack cycle × 0.75, for 10 s; Suppress: 100 energy, 90° ahead, 5 grid units long, every unit, 4 s.
    #[test]
    fn its_abilities_follow_the_design() {
        let inspire = AreaAbility::Inspire.def();
        assert_eq!(
            (
                inspire.name,
                inspire.cost,
                inspire.area,
                inspire.radius,
                inspire.duration
            ),
            ("Inspire", 75.0, AbilityArea::AroundCaster, 4.0, 10.0)
        );
        const INFANTRY: [ObjectKind; 4] = [
            ObjectKind::Recruit,
            ObjectKind::Militant,
            ObjectKind::Gunner,
            ObjectKind::Disciple,
        ];
        assert_eq!(inspire.affects, Affects::Own(&INFANTRY));
        assert_eq!(
            inspire.effect,
            BuffEffect::Inspire {
                max_speed: 1.5,
                attack_cycle: 0.75,
                faster_attacks: &[ObjectKind::Recruit],
            }
        );
        let suppress = AreaAbility::Suppress.def();
        assert_eq!(
            (suppress.name, suppress.cost, suppress.radius, suppress.duration),
            ("Suppress", 100.0, 5.0, 4.0)
        );
        assert_eq!(suppress.area, AbilityArea::Ahead { arc: PI / 2.0 }, "90° in all");
        assert_eq!(
            (suppress.affects, suppress.effect),
            (Affects::EveryUnit, BuffEffect::HoldFire)
        );
    }
}
