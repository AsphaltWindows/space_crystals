//! The Laser Battery — `factions/syndicate/units/laser_battery.md` (design change 0367's placeholders; slice M6-SK) —
//! and its two unit types, mobile and deployed (`> Deploy (ability)`: "**Two unit types**: **Laser Battery** and
//! **Laser Battery (Deployed)**, with separate SelectionGroups").

use crate::defs::units::{
    AttackAttrs, AttackType, FullyConnectedSubtype, MovementAttrs, TargetDomain, TargetType, TurretAttrs, UnitBase,
    UnitDef,
};
use crate::defs::{Armor, Category, Footprint, ObjectDef, ObjectKind};
use crate::math::PI;
use crate::player::Faction;

/// `factions/syndicate/units/laser_battery.md`: the "Syndicate laser vehicle running on a **battery**. Treaded. Fires a
/// **red laser** (FullyConnected: instant, cannot miss) at **fairly long range**, **ground only** while mobile. Its
/// turret has **relatively little mobility**." Its numbers only (slice M6-SK, plan decision T84, by T69's rule): it
/// drives by its SpeedTurnRadiusMovement and fires from its turret as any tracked unit does, and nothing makes one yet:
/// the Vehicle Assembly trains it behind an Optics Lab with slice M6-SL, which brings its battery, its surges' damage
/// by the charge they use, Amplify and Deploy (plan decision T97), and slice M6-SR its Relay. Its numbers are the
/// design's "Placeholder stats".
pub(crate) const LASER_BATTERY: ObjectDef = LASER_BATTERY_DEF;

/// `> Deploy (ability)`: "**Deployed**: **cannot move**. The turret gets **360° rotation**. It can **only target air
/// units** (no ground targets while deployed)", the mobile type's in all else. Immobile by its def (plan decision T87):
/// its MaxSpeed and ReverseMaxSpeed are 0, so it can't move (`Mover::can_move`), turns nothing (its SpinRate, 2 ×
/// MaxSpeed ÷ TrackWidth, is 0) and never yields. Nothing makes one yet: deploying, 48 frames each way, comes with
/// slice M6-SL (plan decision T87, GA's vehicle deploy model), and its Relay with slice M6-SR.
pub(crate) const LASER_BATTERY_DEPLOYED: ObjectDef = ObjectDef {
    kind: ObjectKind::LaserBatteryDeployed,
    name: "Laser Battery (Deployed)",
    unit: Some(UnitDef {
        // "**Deployed**: **cannot move**": the mobile type's handling at a MaxSpeed of 0.
        movement: MovementAttrs::SpeedTurnRadius {
            track_width: 33.0,
            acceleration: Some(0.4),
            deceleration: Some(1.0),
            max_speed: 0.0,
            reverse_max_speed: 0.0,
        },
        // "Turret: [...] deployed 360°; TurnRate 6°/frame."
        turret: Some(TurretAttrs {
            turn_angle: 2.0 * PI,
            turn_rate: 6.0 * PI / 180.0,
        }),
        // "TargetDomain: Ground (mobile); Air (deployed, below)"
        attack: Some(AttackAttrs {
            target_domain: TargetDomain::Air,
            ..LASER
        }),
        ..LASER_BATTERY_UNIT
    }),
    ..LASER_BATTERY_DEF
};

/// The damage of a normal surge: "**Damage per surge = 4 × charge^(2/3)**: a normal surge deals about 18.6", "a normal
/// surge uses **10**", so 4 × 10^(2/3), until slice M6-SL computes each surge's from the charge it uses (plan decision
/// T97).
const SURGE_DAMAGE: f32 = 18.566_355;

/// `factions/syndicate/units/laser_battery.md`: its laser, mobile, at the ground ("TargetDomain: Ground (mobile)").
const LASER: AttackAttrs = AttackAttrs {
    // AttackType: FullyConnected (cannot miss); its Range of 8 grid units makes it the Ranged subtype
    // (`systems/combat.md > FullyConnected > Melee` is adjacent contact).
    attack_type: AttackType::FullyConnected(FullyConnectedSubtype::Ranged),
    target_domain: TargetDomain::Ground,   // TargetDomain: Ground (mobile)
    target_type: TargetType::SingleTarget, // TargetType: SingleTarget (assumed)
    damage: SURGE_DAMAGE,
    range: 8.0,             // Range **8 grid units**
    min_range: 0.0,         // Not listed: none
    projectile_speed: None, // FullyConnected: no projectile
    aim_ticks: 8,           // AimDuration 8 frames
    // "surge interval **16 frames** (1 s)": plan decision T97's split of it, **Placeholder (not in the design: the
    // split)**: Firing 1, Cooldown 1 and Reload 14, the surges' cadence once slice M6-SL aims only once while engaged
    // (until then each surge aims again, as every attack does).
    firing_ticks: 1,
    cooldown_ticks: 1,
    reload_ticks: 14,
};

/// `factions/syndicate/units/laser_battery.md`: what both of its types are.
const LASER_BATTERY_DEF: ObjectDef = ObjectDef {
    kind: ObjectKind::LaserBattery,
    name: "Laser Battery",
    faction: Some(Faction::Syndicate), // Faction - TheSyndicate
    category: Category::Unit,          // Entity Type - Unit
    // "Silhouette 64x44 space units": its length along its facing, turned with the facing, as every vehicle's
    // (`ObjectDef::silhouette_turns`).
    footprint: Footprint::Silhouette { w: 64.0, h: 44.0 },
    selectable: true, // `core/entities.md > Object Type > Selectable - true`
    // Not listed; a type with MaxHP is destructible (`core/entities.md > Object Instance > HP`).
    destructible: true,
    groupable: true,     // Groupable - true (assumed)
    sight_range: 7.0,    // SightRange 7 grid units
    max_hp: Some(220.0), // MaxHP 220
    armor: Armor {
        point: 3.0, // PointArmor 3
        full: 3.0,  // FullArmor 3
    },
    needs_flat_ground: false,
    unit: Some(LASER_BATTERY_UNIT),
    ..ObjectDef::NONE
};

/// `factions/syndicate/units/laser_battery.md`: its unit attributes, mobile.
const LASER_BATTERY_UNIT: UnitDef = UnitDef {
    base: UnitBase::TrackedVehicle, // UnitBase - TrackedVehicle (exits only through Tier 2+ Tunnels)
    // "SpeedTurnRadiusMovement: MaxSpeed 4.5 su/frame; Acceleration 0.4; Deceleration 1; TrackWidth 33 su;
    // ReverseMaxSpeed 2.25 (Q0153; spins about 15.6° a frame)."
    movement: MovementAttrs::SpeedTurnRadius {
        track_width: 33.0,
        acceleration: Some(0.4),
        deceleration: Some(1.0),
        max_speed: 4.5,
        reverse_max_speed: 2.25,
    },
    rugged_terrain_defense_bonus: None, // only LightInfantry has it
    // "Turret: mobile TurnAngle **30°** total; deployed 360°; TurnRate 6°/frame": "it can only aim within a cone ahead
    // of the hull, so the tank must roughly face its target (like the Assault Bike)".
    turret: Some(TurretAttrs {
        turn_angle: 30.0 * PI / 180.0,
        turn_rate: 6.0 * PI / 180.0,
    }),
    attack: Some(LASER),
    tunnel_space_cost: Some(3), // TunnelSpaceCost 3
    deploy: None,               // Deploy is the vehicle deploy model's (slice M6-SL), not the Drone's (`deploy`)
    ..UnitDef::NONE
};

#[cfg(test)]
mod tests {
    use glam::Vec2;

    use super::*;
    use crate::defs::syndicate::tests::timings;

    /// A 64×44 TrackedVehicle, MaxHP 220, PointArmor 3, FullArmor 3, SightRange 7, Groupable ("true (assumed)"),
    /// TunnelSpaceCost 3; SpeedTurnRadiusMovement at 4.5 su a frame on a 33 su track, spinning about 15.6° a frame
    /// (Q0153); a 30° turret turning 6° a frame; a red laser that can't miss, at the ground, up to 8 grid units, aiming
    /// 8 frames, a normal surge's 4 × 10^(2/3) damage, about 18.6, every 16 frames once engaged (plan decision T97's
    /// split).
    #[test]
    fn the_laser_battery_follows_the_design() {
        let def = ObjectKind::LaserBattery.def();
        assert_eq!(
            (def.name, def.category, def.faction),
            ("Laser Battery", Category::Unit, Some(Faction::Syndicate))
        );
        assert_eq!(def.silhouette(), Some(Vec2::new(64.0, 44.0)));
        let three = Armor { point: 3.0, full: 3.0 };
        assert_eq!(
            (def.max_hp, def.max_shields, def.armor, def.sight_range),
            (Some(220.0), None, three, 7.0)
        );
        assert_eq!((def.selectable, def.destructible, def.groupable), (true, true, true));
        let unit = def.unit.as_ref().unwrap();
        assert_eq!(unit.base, UnitBase::TrackedVehicle);
        let movement = MovementAttrs::SpeedTurnRadius {
            track_width: 33.0,
            acceleration: Some(0.4),
            deceleration: Some(1.0),
            max_speed: 4.5,
            reverse_max_speed: 2.25,
        };
        assert_eq!(unit.movement, movement);
        let spin = unit.movement.spin_rate().expect("a tracked spin").to_degrees();
        assert!((spin - 15.6).abs() < 0.05, "spins about 15.6° a frame: {spin}");
        let turret = unit.turret.expect("a turret");
        assert!((turret.turn_angle.to_degrees() - 30.0).abs() < 1e-4, "a 30° cone");
        assert!((turret.turn_rate.to_degrees() - 6.0).abs() < 1e-4, "6° a frame");
        let attack = unit.attack.expect("its laser");
        assert_eq!(
            attack.attack_type,
            AttackType::FullyConnected(FullyConnectedSubtype::Ranged)
        );
        assert_eq!(
            (attack.target_domain, attack.target_type),
            (TargetDomain::Ground, TargetType::SingleTarget)
        );
        let surge = 4.0 * libm::powf(10.0, 2.0 / 3.0);
        assert!(
            (attack.damage - surge).abs() < 1e-4,
            "4 × charge^(2/3) of a normal surge's 10: {surge}"
        );
        assert!((attack.damage - 18.6).abs() < 0.05, "about 18.6");
        assert_eq!(
            (attack.range, attack.min_range, attack.projectile_speed),
            (8.0, 0.0, None)
        );
        let ((aim, firing, cooldown, reload), _) = timings(&attack);
        assert_eq!(
            (aim, firing + cooldown + reload),
            (8, 16),
            "Aim 8; a surge every 16 frames"
        );
        assert_eq!(
            (
                unit.tunnel_space_cost,
                unit.unit_control_cost,
                unit.beacon_capacity_cost
            ),
            (Some(3), None, None)
        );
        assert_eq!((unit.worker, unit.deploy, unit.separation_radius), (None, None, None));
    }

    /// `> Deploy (ability)`: the deployed type is the mobile one's, but it "**cannot move**" (a MaxSpeed and a
    /// ReverseMaxSpeed of 0, so no spin either: plan decision T87's immobile def), its turret turns all the way round,
    /// and its laser targets only air units.
    #[test]
    fn deployed_it_cannot_move_turns_its_turret_round_and_targets_air() {
        let (mobile, deployed) = (ObjectKind::LaserBattery.def(), ObjectKind::LaserBatteryDeployed.def());
        assert_eq!(deployed.name, "Laser Battery (Deployed)");
        assert_eq!(
            ObjectDef {
                kind: mobile.kind,
                name: mobile.name,
                unit: mobile.unit.clone(),
                ..deployed.clone()
            },
            *mobile,
            "the mobile type's in all but its unit attributes"
        );
        let (mobile, deployed) = (mobile.unit.as_ref().unwrap(), deployed.unit.as_ref().unwrap());
        let MovementAttrs::SpeedTurnRadius {
            max_speed,
            reverse_max_speed,
            ..
        } = deployed.movement
        else {
            panic!("{:?}", deployed.movement);
        };
        assert_eq!((max_speed, reverse_max_speed), (0.0, 0.0), "cannot move");
        assert_eq!(deployed.movement.spin_rate(), Some(0.0), "nor turn its hull");
        let turret = deployed.turret.expect("a turret");
        assert!((turret.turn_angle.to_degrees() - 360.0).abs() < 1e-3, "360° rotation");
        assert_eq!(turret.turn_rate, mobile.turret.unwrap().turn_rate);
        let attack = deployed.attack.expect("its laser");
        assert_eq!(attack.target_domain, TargetDomain::Air, "only air units");
        assert_eq!(
            AttackAttrs {
                target_domain: TargetDomain::Ground,
                ..attack
            },
            mobile.attack.unwrap()
        );
        assert_eq!(
            UnitDef {
                movement: mobile.movement,
                turret: mobile.turret,
                attack: mobile.attack,
                ..deployed.clone()
            },
            *mobile
        );
    }
}
