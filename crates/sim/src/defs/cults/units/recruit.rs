//! The Recruit — `factions/cults/units/recruit.md`.

use crate::defs::units::{
    AttackAttrs, AttackType, FullyConnectedSubtype, MELEE_CONTACT_GU, MovementAttrs, TargetDomain, TargetType,
    UnitBase, UnitDef,
};
use crate::defs::{Armor, Category, Footprint, ObjectDef, ObjectKind};
use crate::math::PI;
use crate::player::Faction;

/// `factions/cults/units/recruit.md`: "the Cults' basic worker unit, automatically produced by Recruitment Halls", with
/// "a melee attack for basic self-defense". What it does besides fight comes with each system's slice (plan §3.2): its
/// gathering (`> Gathering`) and its building (`> Building`) in its `worker`, slices M5-D and M5-E; its entering an
/// Armory (`> Armory Entry`), slice M5-F1.
pub(crate) static RECRUIT: ObjectDef = ObjectDef {
    kind: ObjectKind::Recruit,
    name: "Recruit",
    faction: Some(Faction::Cults),                         // Faction - TheCults
    category: Category::Unit,                              // Entity Type - Unit
    footprint: Footprint::Silhouette { w: 24.0, h: 24.0 }, // Silhouette - 24x24 space units (square)
    selectable: true,                                      // `core/entities.md > Object Type > Selectable - true`
    destructible: true,                                    // Destructible - true
    groupable: true,                                       // Groupable - true (user, 2026-11-24)
    sight_range: 5.0,                                      // SightRange - 5 grid units
    max_hp: Some(40.0),                                    // MaxHP - 40
    max_shields: None,
    armor: Armor {
        point: 0.0, // PointArmor - 0
        full: 0.0,  // FullArmor - 0
    },
    needs_flat_ground: false,
    unit: Some(UnitDef {
        base: UnitBase::LightInfantry, // UnitBase - LightInfantry
        // UnitBaseAttributes[LightInfantry]
        movement: MovementAttrs::TurnRate {
            turn_rate: PI,      // TurnRate: 180 degrees/frame
            acceleration: None, // Acceleration: infinite
            deceleration: None, // Deceleration: infinite
            max_speed: 4.0,     // MaxSpeed: 4 space units/frame
        },
        rugged_terrain_defense_bonus: Some(0.5), // RuggedTerrainDefenseBonus: 50%
        turret: None,                            // TurretAttributes - None
        attack: Some(AttackAttrs {
            // AttackType: FullyConnected (Melee)
            attack_type: AttackType::FullyConnected(FullyConnectedSubtype::Melee),
            target_domain: TargetDomain::Ground,   // TargetDomain: Ground
            target_type: TargetType::SingleTarget, // TargetType: SingleTarget
            damage: 3.0,                           // Damage: 3
            // Range: melee (adjacent contact): a reach of 8 su, a placeholder (design change 0149).
            range: MELEE_CONTACT_GU,
            min_range: 0.0,         // none listed: contact is as near as a target gets
            projectile_speed: None, // FullyConnected: no projectile
            aim_ticks: 2,           // AimDuration: 2 frames
            firing_ticks: 8,        // FiringDuration: 8 frames
            cooldown_ticks: 1,      // CooldownDuration: 1 frame
            reload_ticks: 1,        // ReloadDuration: 1 frame
        }),
        // Its Unit Control: the one Recruit it is (plan decision T55; `factions/cults/overview.md > CultsResources >
        // Unit Control`: "Each Cult unit has a Unit Control cost"; `factions/cults/economy.md > RecruitmentHall >
        // BaseRecruitProductionTime`: "every Cults unit (miners, builders, staff and army) counts against its
        // originating hall's capacity", and "a starting hall (6 Recruits → 20) fills in 14 × 12 s").
        unit_control_cost: Some(1),
        tunnel_space_cost: None,
        beacon_capacity_cost: None,
        riot_shield: None,
        conversion: None,
        deploy: None,
        worker: None, // `> Gathering` (slice M5-D) and `> Building` (slice M5-E)
    }),
    structure: None,
    default_settings: None,
};

#[cfg(test)]
mod tests {
    use super::*;
    use crate::defs::cults::tests::{assert_infantry, timings};

    /// The Cults' worker: LightInfantry, 24×24, MaxHP 40, no armour, SightRange 5, Groupable (user, 2026-11-24),
    /// MaxSpeed 4, with its melee attack "for basic self-defense": FullyConnected Melee, ground only, single target, 3
    /// damage at contact, one blow every 12 frames (Aim 2, Fire 8, Cooldown 1, Reload 1); Unit Control 1. Its
    /// gathering and its building come with their slices (M5-D and M5-E).
    #[test]
    fn the_recruit_follows_the_design() {
        let attack = assert_infantry(ObjectKind::Recruit);
        assert_eq!(
            attack.attack_type,
            AttackType::FullyConnected(FullyConnectedSubtype::Melee)
        );
        assert_eq!(
            (attack.target_domain, attack.target_type, attack.damage),
            (TargetDomain::Ground, TargetType::SingleTarget, 3.0)
        );
        assert_eq!(
            (attack.range, attack.min_range, attack.projectile_speed),
            (MELEE_CONTACT_GU, 0.0, None)
        );
        assert_eq!(timings(&attack), ((2, 8, 1, 1), 12), "one blow every 12 frames");
        let def = ObjectKind::Recruit.def();
        assert_eq!(def.name, "Recruit");
        assert_eq!(def.unit.as_ref().unwrap().worker, None, "slices M5-D and M5-E");
    }
}
