//! The Drone Operator — `factions/gdo/units/drone_operator.md` (design change 0366, slice M6-GK): "GDO specialist
//! infantry, trained at the Barracks once the player owns a Career Campus. Carries explosive suicide drones, launches
//! them one at a time and pilots them into enemies." Its drone is a kind of its own ([`OPERATOR_DRONE`]: "The drone is
//! a separate flying unit **controlled directly by the player**"), named "Drone" as the design names it, its kind
//! `OperatorDrone` beside the Colonists' `Drone` (plan decision T84). Their numbers are the design's "Placeholder
//! stats" (design change 0421). (M6-GD) The operator's drone stock, which Build Drone fills and Launch Drone spends
//! (`UnitDef::stock`, `crate::stock`), the drone's explosion (its attack) and its leash and battery (`UnitDef::tether`,
//! `crate::tether`), and the operator's Barracks entry ([`RECIPE`], in the Barracks' `production` since slice M6-GD).

use crate::defs::structures::Recipe;
use crate::defs::units::{
    AttackAttrs, AttackType, BlastDomain, FullyConnectedSubtype, MELEE_CONTACT_GU, MovementAttrs, StockDef,
    TargetDomain, TargetType, TetherDef, UnitBase, UnitDef,
};
use crate::defs::{Armor, Category, Footprint, ObjectDef, ObjectKind};
use crate::math::PI;
use crate::player::Faction;
use crate::units::SU_PER_GU;

/// `factions/gdo/units/drone_operator.md`: the operator's placeholder stats (design change 0366).
pub(crate) const DRONE_OPERATOR: ObjectDef = ObjectDef {
    kind: ObjectKind::DroneOperator,
    name: "Drone Operator",
    faction: Some(Faction::Gdo), // Faction - GlobalDefenseOrganization
    category: Category::Unit,    // Entity Type - Unit
    footprint: Footprint::Silhouette { w: 24.0, h: 24.0 }, // "LightInfantry 24x24"
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
        base: UnitBase::LightInfantry, // "LightInfantry"
        // "MaxSpeed 4 su/frame": UnitBaseAttributes[LightInfantry] (TurnRate 180°/frame, infinite acceleration and
        // deceleration, RuggedTerrainDefenseBonus 50%), as the Peacekeeper's.
        movement: MovementAttrs::TurnRate {
            turn_rate: PI,
            acceleration: None,
            deceleration: None,
            max_speed: 4.0,
        },
        rugged_terrain_defense_bonus: Some(0.5),
        attack: None, // "The Drone Operator has **no weapon** of its own; its only offense is its drones."
        unit_control_cost: Some(1), // UnitControlCost 1
        // `> Drone Stock` (design change 0421).
        stock: Some(StockDef {
            item: ObjectKind::OperatorDrone, // "launches a drone from its stock"
            max: 1,                          // "holds a **maximum of 1** (changed from 3)"
            start: 1,                        // "Starts with **1** drone when trained"
            price: 50,                       // "**Build Drone** (hotkey **F**): 50 Space Crystals"
            frames: 160,                     // "160 frames (10 s)"
        }),
        ..UnitDef::NONE
    }),
    ..ObjectDef::NONE
};

/// `factions/gdo/units/drone_operator.md > Placeholder stats`: "Drone (designer placeholders, 2026-11-26; Q0155):
/// UnitBase **HoverCraft**; **Unit Control 0** (it is spent like ammunition); PointArmor 0, FullArmor 0; **Groupable
/// true** (several drones can be flown as one group). Air unit 24x24; MaxHP 30; top speed 10 su/frame [...]; SightRange
/// 5". (M6-GD) Its explosion, its attack, used once ("**AoE** (user, 2026-11-27), Damage **150** total, AoERadius **48
/// su**, melee range"), and its leash and battery (`> Drone limits — how a drone ends`).
pub(crate) const OPERATOR_DRONE: ObjectDef = ObjectDef {
    kind: ObjectKind::OperatorDrone,
    name: "Drone",
    faction: Some(Faction::Gdo), // Faction - GlobalDefenseOrganization
    category: Category::Unit,    // "a separate flying unit"
    footprint: Footprint::Silhouette { w: 24.0, h: 24.0 }, // Air unit 24x24
    selectable: true,            // "the player clicks the drone (or uses its portrait) to fly it"
    // "The drone **can be shot down**".
    destructible: true,
    groupable: true,    // **Groupable true**
    sight_range: 5.0,   // SightRange 5
    max_hp: Some(30.0), // MaxHP 30
    armor: Armor::NONE, // PointArmor 0, FullArmor 0
    needs_flat_ground: false,
    unit: Some(UnitDef {
        base: UnitBase::HoverCraft, // UnitBase **HoverCraft**
        // "top speed 10 su/frame (DragMovement ForwardAcceleration 0.9, OmniDirectionalAcceleration 0.1, DragRatio
        // 0.1, TurnRate 10°/frame, as the Supply Chopper; Q0154)" (design change 0420).
        movement: MovementAttrs::Drag {
            forward_acceleration: 0.9,
            omni_directional_acceleration: 0.1,
            drag_ratio: 0.1,
            turn_rate: 10.0 * PI / 180.0,
        },
        // `> Launching and piloting`: "**Detonation:** the drone behaves like a combat unit with a **melee-range attack
        // that it can use only once** — the attack is its explosion. It takes normal Attack orders and attacks like any
        // melee unit; using its attack destroys it" (`crate::combat`: the blast is centred on the drone).
        attack: Some(AttackAttrs {
            attack_type: AttackType::FullyConnected(FullyConnectedSubtype::Melee), // "melee range"
            // "**Explosion targets ground units only** (TargetDomain: Ground) — the drone cannot attack air units."
            target_domain: TargetDomain::Ground,
            target_type: TargetType::AoE {
                radius: 48.0 / SU_PER_GU, // AoERadius **48 su** (design change 0368: "the drone's explosion is AoE")
                blast: BlastDomain::Attack,
            },
            damage: 150.0, // Damage **150** total
            // Range: melee, a reach of 8 su, a placeholder (design change 0149).
            range: MELEE_CONTACT_GU,
            min_range: 0.0,
            projectile_speed: None, // FullyConnected: no projectile
            // **Placeholder (not in the design: the explosion's timings)**: an Aim of 2 frames, the Recruit's, and its
            // blast at the end of a 1-frame Firing. The drone is gone then, so its Cooldown and Reload never run.
            aim_ticks: 2,
            firing_ticks: 1,
            cooldown_ticks: 0,
            reload_ticks: 0,
            surge: None,
        }),
        unit_control_cost: Some(0), // **Unit Control 0** (it is spent like ammunition)
        // **Placeholder (Q0181)**: SeparationRadius 32 su, its Silhouette's side and 8 su, as the Sparrow's and the
        // Dragonfly's 48 on their 40 (`systems/units.md > Air Collision`: "Must be larger than the unit's Silhouette").
        separation_radius: Some(32.0),
        // `> Drone limits — how a drone ends` and `> Placeholder stats` (design change 0421, answering Q0155).
        tether: Some(TetherDef {
            leash: 8.0,   // "**leash range 8 grid units**, measured **edge to edge** like ranges"
            battery: 320, // "**battery 20 s** (320 frames)"
        }),
        ..UnitDef::NONE
    }),
    ..ObjectDef::NONE
};

/// `factions/gdo/structures/barracks.md > Produces`, and `> Placeholder stats` of
/// `factions/gdo/units/drone_operator.md`: "trained at the Barracks on **R** (Q0155) [...] Cost: Barracks, 100 Space
/// Crystals **+ 1 Supply** (user, 2026-11-26; was 2), 160 frames" (design change 0441), behind its Unit Prerequisite,
/// "TechPrerequisite - player owns at least one Career Campus". In the Barracks' `production` since slice M6-GD, which
/// built the Supply cost (`Recipe::supplies`: "paid with the crystals when queued", `sim/producers/queuing.rs`).
pub(crate) const RECIPE: Recipe = Recipe {
    kind: ObjectKind::DroneOperator,
    cost: 100,
    frames: 160,
    requires: &[ObjectKind::CareerCampus],
    supplies: 1,
};

#[cfg(test)]
mod tests {
    use super::*;
    use crate::defs::Domain;

    /// The operator's placeholder stats: LightInfantry, 24×24, MaxHP 45, armour 1/1, SightRange 6, Unit Control 1; the
    /// Peacekeeper's movement; no weapon. (M6-GD) Its stock: one drone at the start, one at most, another for 50 Space
    /// Crystals in 160 frames. Its Barracks entry, 100 Space Crystals and 1 Supply in 160 frames behind a Career
    /// Campus, is the Barracks' fourth (`super::super::super::structures::barracks`).
    #[test]
    fn the_drone_operator_follows_the_design() {
        let unit = super::super::assert_stat_block(
            ObjectKind::DroneOperator,
            "Drone Operator",
            (24.0, 24.0),
            UnitBase::LightInfantry,
            45.0,
            (1.0, 1.0),
            6.0,
            1,
        );
        let peacekeeper = ObjectKind::Peacekeeper.def().unit.as_ref().unwrap();
        assert_eq!(
            (unit.movement, unit.rugged_terrain_defense_bonus),
            (peacekeeper.movement, peacekeeper.rugged_terrain_defense_bonus),
            "as the Peacekeeper"
        );
        assert_eq!((unit.attack, unit.turret, unit.tether), (None, None, None));
        let stock = StockDef {
            item: ObjectKind::OperatorDrone,
            max: 1,
            start: 1,
            price: 50,
            frames: 160,
        };
        assert_eq!(unit.stock, Some(stock));
        let recipe = Recipe {
            kind: ObjectKind::DroneOperator,
            cost: 100,
            frames: 160,
            requires: &[ObjectKind::CareerCampus],
            supplies: 1,
        };
        assert_eq!(RECIPE, recipe);
    }

    /// The drone's (design change 0421): a HoverCraft in the air, 24×24, named "Drone", MaxHP 30, no armour,
    /// SightRange 5, Groupable, Unit Control 0; DragMovement to a top speed of 10 su a frame, turning 10° a frame, as the
    /// Supply Chopper's; a SeparationRadius of 32 su, more than its Silhouette. (M6-GD) Its explosion, a melee attack
    /// on the ground of 150 over 48 su; its leash of 8 GU and its battery of 320 frames; no stock of its own.
    #[test]
    fn the_operators_drone_follows_the_design() {
        let unit = super::super::assert_stat_block(
            ObjectKind::OperatorDrone,
            "Drone",
            (24.0, 24.0),
            UnitBase::HoverCraft,
            30.0,
            (0.0, 0.0),
            5.0,
            0,
        );
        assert_eq!(unit.base_def().domain, Domain::Air);
        assert!((unit.movement.max_speed() - 10.0).abs() < 1e-4, "top speed 10 su/frame");
        let chopper = ObjectKind::SupplyChopper.def().unit.as_ref().unwrap();
        assert_eq!(unit.movement, chopper.movement, "as the Supply Chopper");
        assert_eq!(
            (unit.turret, unit.separation_radius, unit.stock),
            (None, Some(32.0), None)
        );
        let attack = unit.attack.expect("its explosion");
        assert_eq!(
            (attack.attack_type, attack.target_domain, attack.damage),
            (
                AttackType::FullyConnected(FullyConnectedSubtype::Melee),
                TargetDomain::Ground,
                150.0
            )
        );
        let TargetType::AoE { radius, blast } = attack.target_type else {
            panic!("an area attack: {:?}", attack.target_type);
        };
        assert_eq!((radius * SU_PER_GU, blast), (48.0, BlastDomain::Attack));
        assert_eq!(
            unit.tether,
            Some(TetherDef {
                leash: 8.0,
                battery: 320
            })
        );
        assert_ne!(
            ObjectKind::OperatorDrone,
            ObjectKind::Drone,
            "the Colonists' Drone is another kind"
        );
    }
}
