//! The Artillery Rig — `factions/gdo/units/artillery_rig.md` (design change 0366, slice M6-GK): "GDO artillery vehicle
//! produced at the Factory; requires Advanced Mechanics. Has two modes: it cannot attack while mobile and must deploy
//! to fire long-range area artillery." `> Modes`: "**Two unit types**: **Artillery Rig** and **Artillery Rig
//! (Deployed)**, with separate SelectionGroups", a kind each ([`ARTILLERY_RIG`], [`ARTILLERY_RIG_DEPLOYED`]), as the
//! Peacekeeper's two (plan decision T84). Their numbers are the design's "Placeholder stats". The deployed kind is
//! immobile by its def (plan decision T87): the mobile kind's SpeedTurnRadiusMovement with MaxSpeed and ReverseMaxSpeed
//! 0, so it neither drives nor turns its hull, and aims with its turret. Deploying and undeploying ("**Deploy 48 frames
//! (3 s), Undeploy 48 frames.**", the vehicle deploy model) and its Factory entry ([`RECIPE`], held) are slice M6-GA's;
//! no player produces one before.

use crate::defs::structures::Recipe;
use crate::defs::units::{
    AttackAttrs, AttackType, BlastDomain, MovementAttrs, TargetDomain, TargetType, TurretAttrs, UnitBase, UnitDef,
};
use crate::defs::{Armor, Category, Footprint, ObjectDef, ObjectKind};
use crate::math::PI;
use crate::player::Faction;
use crate::units::SU_PER_GU;

/// `factions/gdo/units/artillery_rig.md`, mobile: "**Mobile:** moves normally; **has no attack**."
pub(crate) const ARTILLERY_RIG: ObjectDef = ObjectDef {
    kind: ObjectKind::ArtilleryRig,
    name: "Artillery Rig",
    faction: Some(Faction::Gdo), // Faction - GlobalDefenseOrganization
    category: Category::Unit,    // Entity Type - Unit
    // "Silhouette 64x48 space units": its length along its facing (`systems/units.md > Silhouette Rotation`), turned
    // with the facing (`ObjectDef::silhouette_turns`).
    footprint: Footprint::Silhouette { w: 64.0, h: 48.0 },
    selectable: true, // `core/entities.md > Object Type > Selectable - true`
    // Not listed; a type with MaxHP is destructible (`core/entities.md > Object Instance > HP`).
    destructible: true,
    groupable: true,     // Groupable true
    sight_range: 6.0,    // SightRange 6 grid units
    max_hp: Some(180.0), // MaxHP 180
    armor: Armor {
        point: 3.0, // PointArmor 3
        full: 3.0,  // FullArmor 3
    },
    needs_flat_ground: false,
    unit: Some(ARTILLERY_RIG_UNIT),
    ..ObjectDef::NONE
};

/// The mobile Rig's unit attributes, which the deployed one's are made from.
const ARTILLERY_RIG_UNIT: UnitDef = UnitDef {
    base: UnitBase::TrackedVehicle, // UnitBase - TrackedVehicle
    // "SpeedTurnRadiusMovement: MaxSpeed 4 su/frame; Acceleration 0.4; Deceleration 1; TrackWidth 36 su;
    // ReverseMaxSpeed 2 (Q0153; spins about 12.7° a frame)."
    movement: MovementAttrs::SpeedTurnRadius {
        track_width: 36.0,
        acceleration: Some(0.4),
        deceleration: Some(1.0),
        max_speed: 4.0,
        reverse_max_speed: 2.0,
    },
    attack: None,               // "**Mobile:** moves normally; **has no attack**."
    turret: None,               // "TurnAngle (Deployed): 360° (full rotation)": the deployed kind's
    unit_control_cost: Some(3), // UnitControlCost 3
    ..UnitDef::NONE
};

/// `factions/gdo/units/artillery_rig.md > Modes`: "**Deployed:** cannot move; fires its artillery attack", the Rig's
/// type once deploying has finished (design change 0366), the mobile kind but for its movement, its turret and its
/// attack.
pub(crate) const ARTILLERY_RIG_DEPLOYED: ObjectDef = ObjectDef {
    kind: ObjectKind::ArtilleryRigDeployed,
    name: "Artillery Rig (Deployed)",
    unit: Some(UnitDef {
        // "cannot move" (plan decision T87): the mobile kind's movement at a standstill, MaxSpeed and ReverseMaxSpeed 0
        // (so its SpinRate is 0 too: its hull doesn't turn).
        movement: MovementAttrs::SpeedTurnRadius {
            track_width: 36.0,
            acceleration: Some(0.4),
            deceleration: Some(1.0),
            max_speed: 0.0,
            reverse_max_speed: 0.0,
        },
        // `> TurretAttributes`: "TurnAngle (Deployed): 360° (full rotation)"; "Turret TurnRate 3°/frame."
        turret: Some(TurretAttrs {
            turn_angle: 2.0 * PI,
            turn_rate: 3.0 * PI / 180.0,
        }),
        attack: Some(AttackAttrs {
            // "AttackType: **TailDisjointed** — the target location is locked at the end of Aiming; the blast lands
            // there at the end of Firing (no tracking projectile) [...] Can also be fired at a ground location."
            attack_type: AttackType::TailDisjointed,
            target_domain: TargetDomain::Ground, // "TargetDomain: Ground"
            // "TargetType: AoE — **friendly fire**: the blast damages friendly units too"; "AoERadius **64 su (1
            // GU)**", in its TargetDomain's domain, Ground (`systems/combat.md > AoEDamage`), as the Hand of Heaven's.
            target_type: TargetType::AoE {
                radius: 64.0 / SU_PER_GU,
                blast: BlastDomain::Attack,
            },
            // "Deployed attack: Damage **300** total over the circle (AoEDamage); AoERadius **64 su (1 GU)**; Range
            // **12 grid units**; MinRange **3 grid units**; AimDuration 16, FiringDuration **16** (the shell's flight:
            // the gap in which targets can leave the locked area), CooldownDuration 8, ReloadDuration 64 frames (one
            // shell per 104 frames, 6.5 s)."
            damage: 300.0,
            range: 12.0,
            min_range: 3.0,
            projectile_speed: None, // TailDisjointed: no projectile
            aim_ticks: 16,
            firing_ticks: 16,
            cooldown_ticks: 8,
            reload_ticks: 64,
        }),
        ..ARTILLERY_RIG_UNIT
    }),
    ..ARTILLERY_RIG
};

/// `factions/gdo/structures/factory.md > Produces`: "Artillery Rig: 250 Space Crystals, 480 frames (30 seconds)
/// (placeholder). Requires: player owns at least one Advanced Mechanics", its Unit Prerequisite, and `> Placeholder
/// stats` of `factions/gdo/units/artillery_rig.md`: "Cost: Factory, 250 Space Crystals **+ 5 Supplies** (user,
/// 2026-11-26; Q0169), 480 frames" (design change 0435). Held until slice M6-GA (`crate::defs::PLANNED_RECIPES`).
pub(crate) const RECIPE: Recipe = Recipe {
    kind: ObjectKind::ArtilleryRig,
    cost: 250,
    frames: 480,
    requires: &[ObjectKind::AdvancedMechanics],
    supplies: 5,
};

#[cfg(test)]
mod tests {
    use super::super::cycle;
    use super::*;

    /// The placeholder stats, both types: a TrackedVehicle, 64×48, MaxHP 180, armour 3/3, SightRange 6, Unit Control
    /// 3. Mobile, up to 4 su a frame with no attack and no turret. Deployed, it doesn't move (MaxSpeed, ReverseMaxSpeed
    /// and SpinRate 0), turns a full turret 3° a frame, and fires a TailDisjointed shell at the ground, 300 over 1 GU,
    /// from 3 to 12 grid units, one every 104 frames. Its Factory entry, 250 Space Crystals and 5 Supplies in 480 frames
    /// behind Advanced Mechanics, is held.
    #[test]
    fn the_artillery_rig_follows_the_design() {
        let mobile = super::super::assert_stat_block(
            ObjectKind::ArtilleryRig,
            "Artillery Rig",
            (64.0, 48.0),
            UnitBase::TrackedVehicle,
            180.0,
            (3.0, 3.0),
            6.0,
            3,
        );
        assert_eq!(mobile.movement.max_speed(), 4.0);
        assert_eq!((mobile.attack, mobile.turret), (None, None));
        let deployed = super::super::assert_stat_block(
            ObjectKind::ArtilleryRigDeployed,
            "Artillery Rig (Deployed)",
            (64.0, 48.0),
            UnitBase::TrackedVehicle,
            180.0,
            (3.0, 3.0),
            6.0,
            3,
        );
        let still = (deployed.movement.max_speed(), deployed.movement.spin_rate());
        let back = deployed.movement.reverse_speed().map(|reverse| reverse.max_speed);
        assert_eq!((still, back), ((0.0, Some(0.0)), Some(0.0)), "it cannot move");
        let turret = deployed.turret.expect("its gun");
        assert_eq!(turret.turn_angle, 2.0 * PI);
        assert!((turret.turn_rate.to_degrees() - 3.0).abs() < 1e-4);
        let attack = deployed.attack.expect("its shell");
        let area = TargetType::AoE {
            radius: 1.0,
            blast: BlastDomain::Attack,
        };
        assert_eq!(
            (attack.attack_type, attack.target_domain, attack.target_type),
            (AttackType::TailDisjointed, TargetDomain::Ground, area)
        );
        assert_eq!((attack.damage, attack.range, attack.min_range), (300.0, 12.0, 3.0));
        assert_eq!(
            (attack.projectile_speed, cycle(&attack)),
            (None, ((16, 16, 8, 64), 104))
        );
        let recipe = Recipe {
            kind: ObjectKind::ArtilleryRig,
            cost: 250,
            frames: 480,
            requires: &[ObjectKind::AdvancedMechanics],
            supplies: 5,
        };
        assert_eq!(RECIPE, recipe);
    }
}
