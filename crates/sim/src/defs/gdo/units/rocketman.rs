//! The Rocketman — `factions/gdo/units/rocketman.md` (design change 0356, slice M5-K): GDO missile infantry, trained at
//! the Barracks once its owner has an Arsenal (`factions/gdo/structures/barracks.md > Produces`). "Fires missiles at
//! both ground and air targets. Normally its missiles fly to where the target was when it fired, so they can be
//! dodged": its normal attack is DoublyDisjointed (`systems/combat.md > DoublyDisjointed`,
//! `crate::combat::projectiles`), with a MinRange (design change 0203), whose rules are the behaviours'
//! (`systems/controls/command_behaviors.md > AttackingObject > Target inside MinRange`,
//! `systems/controls/behaviors.md > BaseAutoTargeting > Targets inside MinRange`). (M5-S8) Its missiles, normal and
//! locked, burst as area damage (`crate::combat::area`) in the domain of what they were fired at (design change 0445,
//! answering Q0173). Its numbers are the design's "Placeholder stats".

use crate::defs::units::{
    AttackAttrs, AttackType, BlastDomain, MovementAttrs, TargetDomain, TargetLockDef, TargetType, UnitBase, UnitDef,
};
use crate::defs::{Armor, Category, Footprint, ObjectDef, ObjectKind};
use crate::math::PI;
use crate::player::Faction;
use crate::units::SU_PER_GU;

/// `factions/gdo/units/rocketman.md`: its placeholder stats (design change 0356).
pub(crate) const ROCKETMAN: ObjectDef = ObjectDef {
    kind: ObjectKind::Rocketman,
    name: "Rocketman",
    faction: Some(Faction::Gdo), // Faction - GlobalDefenseOrganization
    category: Category::Unit,    // Entity Type - Unit
    footprint: Footprint::Silhouette { w: 24.0, h: 24.0 }, // Silhouette 24x24 space units
    selectable: true,            // `core/entities.md > Object Type > Selectable - true`
    destructible: true,
    groupable: true,    // Groupable true
    sight_range: 6.0,   // SightRange 6 grid units
    max_hp: Some(45.0), // MaxHP 45
    armor: Armor {
        point: 1.0, // PointArmor 1
        full: 1.0,  // FullArmor 1
    },
    needs_flat_ground: false,
    unit: Some(UnitDef {
        base: UnitBase::LightInfantry, // UnitBase LightInfantry
        // "Movement as the Peacekeeper, but MaxSpeed **3 space units/frame** (heavier kit)": UnitBaseAttributes
        // [LightInfantry] (TurnRate 180°/frame, infinite acceleration and deceleration, RuggedTerrainDefenseBonus 50%).
        movement: MovementAttrs::TurnRate {
            turn_rate: PI,
            acceleration: None,
            deceleration: None,
            max_speed: 3.0,
        },
        rugged_terrain_defense_bonus: Some(0.5),
        attack: Some(AttackAttrs {
            // "AttackType: DoublyDisjointed (the missile travels to where the target was at the end of Aiming; targets
            // can dodge; can also be fired at the ground)".
            attack_type: AttackType::DoublyDisjointed,
            target_domain: TargetDomain::Universal, // "TargetDomain: Universal (ground and air)"
            // "TargetType: **AoE** (user, 2026-11-26; Q0173): the missile bursts at the spot it flies to
            // (systems/combat.md > AoEDamage; friendly fire applies). **Domain of the burst** (user, 2026-11-26): it
            // keeps the domain of what it was fired at: fired at an **air** unit it hits only air units; fired at a
            // **ground** unit or the ground, only ground (and underground) objects"; "Damage **40** total over an
            // AoERadius of **32 su**" (design change 0445).
            target_type: TargetType::AoE {
                radius: 32.0 / SU_PER_GU,
                blast: BlastDomain::Target,
            },
            // "Normal attack: Damage **40** total over an AoERadius of **32 su** (designer placeholder, 2026-11-26;
            // Q0173: was 20 single-target; roughly 30 to a 64x40 vehicle hit square-on, 20 to a 40x40 Sparrow, 7 to a
            // 24x24 infantry unit, before armor); Range **6 grid units**; MinRange **2 grid units**; ProjectileSpeed
            // **16 su/frame**; AimDuration 6, FiringDuration 2, CooldownDuration 16, ReloadDuration 8 frames (one
            // missile every 32 frames, 2 s: 20 total area damage per second before armor)."
            damage: 40.0,
            range: 6.0,
            min_range: 2.0,
            projectile_speed: Some(16.0),
            aim_ticks: 6,
            firing_ticks: 2,
            cooldown_ticks: 16,
            reload_ticks: 8,
        }),
        unit_control_cost: Some(1), // UnitControlCost 1
        // "Target Lock: **LockOnDuration 48 frames (3 s)**; **LockRange 7 grid units**; locked missiles use the same
        // Damage, radius and timings, homing; they are **area damage too** (user, 2026-11-26), bursting on the target"
        // (`crate::combat::target_lock::attrs`: its weapon's, homing).
        target_lock: Some(TargetLockDef {
            lock_on_ticks: 48,
            lock_range: 7.0,
        }),
        ..UnitDef::NONE
    }),
    ..ObjectDef::NONE
};

#[cfg(test)]
mod tests {
    use glam::Vec2;

    use super::*;

    /// The placeholder stats: LightInfantry, 24×24, MaxHP 45, armour 1/1, SightRange 6, Groupable, Unit Control 1; the
    /// Peacekeeper's movement at 3 su a frame; a dodgeable missile at ground and air, (M5-S8) bursting over 32 su in the
    /// domain of what it was fired at, 40 damage from 2 to 6 grid units at 16 su a frame, one every 32 frames.
    #[test]
    fn the_rocketman_follows_the_design() {
        let def = ObjectKind::Rocketman.def();
        assert_eq!(
            (def.name, def.category, def.faction),
            ("Rocketman", Category::Unit, Some(Faction::Gdo))
        );
        assert_eq!(def.silhouette(), Some(Vec2::new(24.0, 24.0)));
        assert_eq!(
            (def.max_hp, def.armor, def.sight_range),
            (Some(45.0), Armor { point: 1.0, full: 1.0 }, 6.0)
        );
        assert_eq!((def.selectable, def.destructible, def.groupable), (true, true, true));
        let unit = def.unit.as_ref().unwrap();
        let peacekeeper = ObjectKind::Peacekeeper.def().unit.as_ref().unwrap();
        assert_eq!(unit.base, UnitBase::LightInfantry);
        assert_eq!(
            (unit.movement.turn_rate(), unit.rugged_terrain_defense_bonus),
            (
                peacekeeper.movement.turn_rate(),
                peacekeeper.rugged_terrain_defense_bonus
            ),
            "as the Peacekeeper"
        );
        assert_eq!(unit.movement.max_speed(), 3.0, "heavier kit");
        assert_eq!(unit.unit_control_cost, Some(1));
        let attack = unit.attack.unwrap();
        assert_eq!(
            (attack.attack_type, attack.target_domain, attack.target_type),
            (
                AttackType::DoublyDisjointed,
                TargetDomain::Universal,
                TargetType::AoE {
                    radius: 0.5,
                    blast: BlastDomain::Target
                }
            )
        );
        assert_eq!(0.5 * SU_PER_GU, 32.0, "an AoERadius of 32 su");
        assert_eq!(
            (attack.damage, attack.range, attack.min_range, attack.projectile_speed),
            (40.0, 6.0, 2.0, Some(16.0))
        );
        let cycle = attack.aim_ticks + attack.firing_ticks + attack.cooldown_ticks + attack.reload_ticks;
        assert_eq!(cycle, 32, "one missile every 32 frames");
        assert!(
            crate::combat::can_target_ground(attack.attack_type),
            "can also be fired at the ground"
        );
    }

    /// "Target Lock: **LockOnDuration 48 frames (3 s)**; **LockRange 7 grid units**": "a bit longer than the normal
    /// missile Range". (M5-S8) "locked missiles use the same Damage, radius and timings, homing; they are **area damage
    /// too**": the lock's attack is the normal one's, homing (`crate::combat::target_lock::attrs`).
    #[test]
    fn the_rocketmans_target_lock_follows_the_design() {
        let unit = ObjectKind::Rocketman.def().unit.as_ref().unwrap();
        let lock = unit.target_lock.expect("a Target Lock");
        assert_eq!((lock.lock_on_ticks, lock.lock_range), (48, 7.0));
        let normal = unit.attack.unwrap();
        assert!(lock.lock_range > normal.range);
        let locked = crate::combat::target_lock::attrs(&normal, &lock);
        assert_eq!(locked.attack_type, AttackType::HeadDisjointed, "homing");
        assert_eq!(
            (locked.target_type, locked.damage, locked.projectile_speed),
            (normal.target_type, normal.damage, normal.projectile_speed),
            "the same Damage and radius, a burst"
        );
        let timings = |a: &AttackAttrs| (a.aim_ticks, a.firing_ticks, a.cooldown_ticks, a.reload_ticks);
        assert_eq!(timings(&locked), timings(&normal), "the same timings");
    }

    /// Design change 0445's numbers: "Damage **40** total over an AoERadius of **32 su** [...] roughly 30 to a 64x40
    /// vehicle hit square-on, 20 to a 40x40 Sparrow, 7 to a 24x24 infantry unit, before armor", by the area code's
    /// exact overlap (`crate::combat::overlap`; `systems/combat.md > AoEDamage`: "A unit's damage share = Attack Damage
    /// × (unit overlap area / AoE area)") of a burst on the centre of each: 7.16 to the Rocketman's own 24 × 24
    /// silhouette, wholly inside; 19.9 to the Sparrow's 40 × 40, wholly inside too (its corners 28.3 su out); 29.6 to
    /// the Hand of Heaven's 64 × 40, whose ends the circle just touches, its sides cut off.
    #[test]
    fn a_burst_deals_the_designs_shares_before_armour() {
        let attack = ObjectKind::Rocketman.def().unit.as_ref().unwrap().attack.unwrap();
        let TargetType::AoE { radius, .. } = attack.target_type else {
            panic!("an area attack");
        };
        let r = radius * SU_PER_GU;
        let share = |kind: ObjectKind| {
            let half = kind.def().silhouette().expect("a unit") / 2.0;
            attack.damage * crate::combat::overlap(Vec2::ZERO, r, -half, half) / (PI * r * r)
        };
        let near = |share: f32, expected: f32| (share - expected).abs() < 0.05;
        let infantry = share(ObjectKind::Rocketman);
        assert!(near(infantry, 7.16), "about 7 to a 24x24 infantry unit: {infantry}");
        let sparrow = share(ObjectKind::Sparrow);
        assert!(near(sparrow, 19.89), "about 20 to a 40x40 Sparrow: {sparrow}");
        let vehicle = share(ObjectKind::HandOfHeaven);
        assert!(near(vehicle, 29.61), "about 30 to a 64x40 vehicle square-on: {vehicle}");
    }

    /// The Rocketman is the one type with a Target Lock, and its own missiles fire at the location locked: a Firing at
    /// an object is then always a locked missile's (`crate::combat::target_lock`).
    #[test]
    fn only_the_rocketman_locks_and_its_own_missiles_fire_at_a_location() {
        let locking: Vec<ObjectKind> = (ObjectKind::ALL.iter().copied())
            .filter(|kind| kind.def().unit.as_ref().is_some_and(|unit| unit.target_lock.is_some()))
            .collect();
        assert_eq!(locking, [ObjectKind::Rocketman]);
        let attack = ObjectKind::Rocketman.def().unit.as_ref().unwrap().attack.unwrap();
        assert!(matches!(
            attack.attack_type,
            AttackType::DoublyDisjointed | AttackType::TailDisjointed
        ));
    }
}
