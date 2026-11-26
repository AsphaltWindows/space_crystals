//! The Peacekeeper — `factions/gdo/units/peacekeeper.md` — and its two unit types, shield down and shield up (design
//! change 0173).

use crate::defs::units::{
    AttackAttrs, AttackType, FullyConnectedSubtype, MovementAttrs, RiotShieldDef, TargetDomain, TargetType, UnitBase,
    UnitDef,
};
use crate::defs::{Armor, Category, Footprint, ObjectDef, ObjectKind};
use crate::math::PI;
use crate::player::Faction;

/// The Peacekeeper's toggle time, ticks, as its Riot Shield gives it (`RiotShieldDef::toggle_ticks`): its name for
/// the tests that count the frames of a toggle.
pub const RIOT_SHIELD_TOGGLE_TICKS: u32 = PEACEKEEPER_RIOT_SHIELD.toggle_ticks;

/// `factions/gdo/units/peacekeeper.md > Riot Shield`: the Peacekeeper's, on both of its types (design change 0173).
const PEACEKEEPER_RIOT_SHIELD: RiotShieldDef = RiotShieldDef {
    lowered: ObjectKind::Peacekeeper,
    raised: ObjectKind::PeacekeeperShielded,
    damage_factor: 0.5, // Incoming damage is halved, a 50% pre-armor reduction
    // `> Toggle time`: "raising and lowering the shield takes a short time, so it cannot be flicked on and off
    // between enemy shots. **6 frames (0.375 s), placeholder**" (design change 0174; "should stay under half a
    // second").
    toggle_ticks: 6,
};

/// `factions/gdo/units/peacekeeper.md`, shield down.
pub(crate) static PEACEKEEPER: ObjectDef = PEACEKEEPER_DEF;

/// `factions/gdo/units/peacekeeper.md > Riot Shield > Two unit types` (design change 0173): "Peacekeeper (Shielded)",
/// the Peacekeeper with its shield up or on its way up, like the Sabre and Rifle Legionnaire ("They share everything
/// except what the shield changes"). Its MaxSpeed is halved; that it can't attack is the shield's
/// (`crate::riot_shield::allows_attack`), so its weapon is the Peacekeeper's, whose reload goes on through a toggle
/// (`Sim::change_kind`), as do its cost and its Unit Control ("Cost and Unit Control are the same for both types").
/// The Barracks produces the unshielded type only.
pub(crate) static PEACEKEEPER_SHIELDED: ObjectDef = ObjectDef {
    kind: ObjectKind::PeacekeeperShielded,
    name: "Peacekeeper (Shielded)",
    unit: Some(UnitDef {
        movement: MovementAttrs::TurnRate {
            turn_rate: PI,
            acceleration: None,
            deceleration: None,
            max_speed: 2.0, // MaxSpeed is halved (4 → 2 space units/frame)
        },
        ..PEACEKEEPER_UNIT
    }),
    ..PEACEKEEPER_DEF
};

/// `factions/gdo/units/peacekeeper.md`: what both of its types are (design change 0173).
const PEACEKEEPER_DEF: ObjectDef = ObjectDef {
    kind: ObjectKind::Peacekeeper,
    name: "Peacekeeper",
    faction: Some(Faction::Gdo), // Faction - GlobalDefenseOrganization
    category: Category::Unit,
    footprint: Footprint::Silhouette { w: 24.0, h: 24.0 }, // Silhouette - 24x24 space units (square)
    selectable: true,                                      // `core/entities.md > Object Type > Selectable - true`
    destructible: true,                                    // Destructible - true
    groupable: true,                                       // Groupable - true
    sight_range: 5.0,                                      // SightRange - 5 grid units
    max_hp: Some(50.0),                                    // MaxHP - 50
    max_shields: None,
    armor: Armor {
        point: 1.0, // PointArmor - 1
        full: 1.0,  // FullArmor - 1
    },
    needs_flat_ground: false,
    unit: Some(PEACEKEEPER_UNIT),
    structure: None,
    default_settings: None,
    energy: None,
};

/// `factions/gdo/units/peacekeeper.md`: its unit attributes, the Shielded type's but for its MaxSpeed.
const PEACEKEEPER_UNIT: UnitDef = UnitDef {
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
        // FullyConnected; its Range of 4 grid units makes it the Ranged subtype (`systems/combat.md >
        // FullyConnected > Melee` is adjacent contact).
        attack_type: AttackType::FullyConnected(FullyConnectedSubtype::Ranged),
        target_domain: TargetDomain::Ground,   // TargetDomain: Ground
        target_type: TargetType::SingleTarget, // TargetType: SingleTarget
        damage: 10.0,                          // Damage: 10
        range: 4.0,                            // Range: 4 grid units
        min_range: 0.0,                        // MinRange: 0
        projectile_speed: None,                // FullyConnected: no projectile
        aim_ticks: 2,                          // AimDuration: 2 frames
        firing_ticks: 1,                       // FiringDuration: 1 frame
        cooldown_ticks: 2,                     // CooldownDuration: 2 frames
        reload_ticks: 12,                      // ReloadDuration: 12 frames
    }),
    unit_control_cost: Some(1), // UnitControlCost - 1
    tunnel_space_cost: None,
    beacon_capacity_cost: None,
    riot_shield: Some(PEACEKEEPER_RIOT_SHIELD),
    conversion: None,
    deploy: None,
    worker: None,
    augmentable: None,
    repairs: None,
};

#[cfg(test)]
mod tests {
    use glam::Vec2;

    use super::*;

    /// `factions/gdo/units/peacekeeper.md`: the M1 unit.
    #[test]
    fn the_peacekeeper_follows_the_design() {
        let peacekeeper = ObjectKind::Peacekeeper.def();
        assert_eq!(peacekeeper.silhouette(), Some(Vec2::new(24.0, 24.0)));
        assert_eq!(peacekeeper.max_hp, Some(50.0));
        let one = Armor { point: 1.0, full: 1.0 };
        assert_eq!(peacekeeper.armor, one, "PointArmor - 1, FullArmor - 1");
        assert_eq!(peacekeeper.faction, Some(Faction::Gdo));
        let pk = peacekeeper.unit.as_ref().unwrap();
        assert_eq!(pk.base, UnitBase::LightInfantry);
        // TurnRateMovement: 180°/tick, so every turn completes within a tick.
        let MovementAttrs::TurnRate {
            turn_rate,
            acceleration,
            deceleration,
            ..
        } = pk.movement
        else {
            panic!("{:?} is not TurnRateMovement", pk.base);
        };
        assert_eq!(turn_rate, PI);
        assert_eq!((acceleration, deceleration), (None, None), "infinite");
        assert_eq!(pk.movement.max_speed(), 4.0);
        let attack = pk.attack.unwrap();
        assert_eq!((attack.damage, attack.range, attack.reload_ticks), (10.0, 4.0, 12));
        let shield = pk.riot_shield.unwrap();
        assert_eq!(shield.damage_factor, 0.5);
        assert!(shield.toggle_ticks < 8, "under half a second (8 ticks)");
        assert_eq!(pk.unit_control_cost, Some(1));
    }

    /// `factions/gdo/units/peacekeeper.md > Riot Shield > Two unit types` (design change 0173): Peacekeeper (Shielded)
    /// is the Peacekeeper but for its name and "what the shield changes": "MaxSpeed is halved (4 → 2 space
    /// units/frame)". Its weapon is the same, which its shield keeps it from using (`riot_shield::allows_attack`); its
    /// Unit Control is the same ("Cost and Unit Control are the same for both types"), and so is its shield, which
    /// names both types. The Barracks produces the Peacekeeper only.
    #[test]
    fn the_shielded_peacekeeper_is_the_peacekeeper_at_half_speed() {
        use ObjectKind as K;
        let (peacekeeper, shielded) = (K::Peacekeeper.def(), K::PeacekeeperShielded.def());
        assert_eq!(
            (shielded.kind, shielded.name),
            (K::PeacekeeperShielded, "Peacekeeper (Shielded)")
        );
        let (pk, sh) = (peacekeeper.unit.as_ref().unwrap(), shielded.unit.as_ref().unwrap());
        assert_eq!((pk.movement.max_speed(), sh.movement.max_speed()), (4.0, 2.0));
        let unit = UnitDef {
            movement: pk.movement,
            ..sh.clone()
        };
        assert_eq!(&unit, pk, "the same unit but for its MaxSpeed");
        let object = ObjectDef {
            kind: K::Peacekeeper,
            name: peacekeeper.name,
            unit: Some(unit),
            ..shielded.clone()
        };
        // `ObjectDef` has no `PartialEq`: its fields, all of them, as `Debug` shows them.
        assert_eq!(
            format!("{object:?}"),
            format!("{peacekeeper:?}"),
            "the same object but for its kind and name"
        );
        let shield = sh.riot_shield.unwrap();
        assert_eq!(
            (shield.lowered, shield.raised),
            (K::Peacekeeper, K::PeacekeeperShielded)
        );
        let barracks = K::Barracks.def().structure.as_ref().unwrap().production.unwrap();
        let trained: Vec<ObjectKind> = barracks.recipes.iter().map(|recipe| recipe.kind).collect();
        assert!(trained.contains(&K::Peacekeeper) && !trained.contains(&K::PeacekeeperShielded));
    }
}
