//! The Agent — `factions/syndicate/units/agent.md`.

use crate::defs::units::{
    AttackAttrs, AttackType, DropOffChoice, FullyConnectedSubtype, LoadDef, MovementAttrs, TargetDomain, TargetType,
    UnitBase, UnitDef, WorkerDef,
};
use crate::defs::{Armor, Category, Footprint, ObjectDef, ObjectKind};
use crate::math::PI;
use crate::player::Faction;

/// `factions/syndicate/units/agent.md`: the Syndicate's worker, with its pistol (slice M4-A) and its gathering of
/// Space Crystals and Supplies (`> Gathering`: its `worker`, slices M4-D and M4-I), and its Tunnels (`> Building`:
/// its worker's `builds`, slice M4-E).
pub(crate) static AGENT: ObjectDef = ObjectDef {
    kind: ObjectKind::Agent,
    name: "Agent",
    faction: Some(Faction::Syndicate),                     // Faction - TheSyndicate
    category: Category::Unit,                              // Entity Type - Unit
    footprint: Footprint::Silhouette { w: 36.0, h: 36.0 }, // Silhouette - 36x36
    selectable: true,                                      // `core/entities.md > Object Type > Selectable - true`
    // Not listed for the Agent; a type with MaxHP is destructible (`core/entities.md > Object Instance > HP`).
    destructible: true,
    groupable: false,   // Groupable - false
    sight_range: 5.0,   // SightRange - 5
    max_hp: Some(75.0), // MaxHP - 75
    max_shields: None,
    armor: Armor {
        point: 1.0, // PointArmor - 1
        full: 1.0,  // FullArmor - 1
    },
    needs_flat_ground: false,
    unit: Some(UnitDef {
        base: UnitBase::HeavyInfantry, // UnitBase - HeavyInfantry
        // UnitBaseAttributes[HeavyInfantry]
        movement: MovementAttrs::TurnRate {
            turn_rate: PI,      // TurnRate: 180 degrees/frame
            acceleration: None, // Acceleration: infinite
            deceleration: None, // Deceleration: infinite
            max_speed: 6.0,     // MaxSpeed: 6 space units/frame
        },
        rugged_terrain_defense_bonus: None, // only LightInfantry has it
        turret: None,                       // TurretAttributes - None
        // The pistol (user, 2026-11-24, replacing the melee attack; "Numbers are placeholders"): "One shot every 20
        // frames (1.25 s) = 4 damage per second, a third of a Guard's".
        attack: Some(AttackAttrs {
            attack_type: AttackType::FullyConnected(FullyConnectedSubtype::Ranged), // FullyConnected (Ranged; instant hit)
            target_domain: TargetDomain::Ground,                                    // TargetDomain: Ground
            target_type: TargetType::SingleTarget,                                  // TargetType: SingleTarget
            damage: 5.0,                                                            // Damage: 5
            range: 2.0,                                                             // Range: 2 grid units
            min_range: 0.0,                                                         // MinRange: 0
            projectile_speed: None,                                                 // FullyConnected: no projectile
            aim_ticks: 3,                                                           // AimDuration: 3 frames
            firing_ticks: 1,                                                        // FiringDuration: 1 frame
            cooldown_ticks: 2,                                                      // CooldownDuration: 2 frames
            reload_ticks: 14,                                                       // ReloadDuration: 14 frames
        }),
        unit_control_cost: None,
        tunnel_space_cost: Some(1), // TunnelSpaceCost - 1
        beacon_capacity_cost: None,
        riot_shield: None,
        conversion: None,
        deploy: None,
        // `> Gathering` (design change 0255, answering Q0066): "The Drone's job rules apply to Agents [...] with two
        // exceptions: Agents share patches (no occupancy rule), and there is no Set Drop-off command."
        worker: Some(WorkerDef {
            // `> Gathering > Space Crystals`
            crystals: LoadDef {
                carry_capacity: 50,  // CarryCapacity: 50 Space Crystals per load
                pick_up_ticks: 64,   // MiningDuration: 64 frames (4 seconds), design change 0294 (was 80)
                drop_off_ticks: 64,  // DropOffDuration: 64 frames (4 seconds)
                pick_up_reach: 8.0,  // MiningReach: 8 space units (placeholder)
                drop_off_reach: 8.0, // DropOffReach: 8 space units, to the crystal hatch (placeholder)
            },
            // `> Gathering > Supplies` (slice M4-I; design change 0256 for the rules at a station)
            supplies: Some(LoadDef {
                carry_capacity: 1,  // CarryCapacity: 1 Supply per trip
                pick_up_ticks: 48,  // PickUpDuration: 48 frames (3 seconds)
                drop_off_ticks: 48, // DropOffDuration: 48 frames (3 seconds)
                // The reaches (design change 0292 point 1: "8 su reaches for pick-up at a station and for drop-off at
                // the Supplies hatch", placeholders as the crystals' are).
                pick_up_reach: 8.0, // PickUpReach: 8 space units from the station's edge (placeholder)
                drop_off_reach: 8.0, // DropOffReach: 8 space units from the Supplies hatch on Side D (placeholder)
            }),
            shares_patches: true, // Patch sharing: any number of Agents may mine the same patch
            // `> Gathering rules`: the nearest Tunnel, picked again by each Gather
            drop_off: DropOffChoice::FirstReturn,
            // `> Gathering rules > Search radius` (design change 0295): "within 6 grid units of itself, the Drone's
            // value (placeholder)".
            search_radius: 6.0,
            // `factions/colonists/units/drone.md > Gathering Drones pass through each other`: "Drones only"; `> Gathering
            // > Space Crystals > Close patches and traffic` (design change 0294): "Agents keep normal collision while
            // gathering (no pass-through)".
            pass_through: false,
            // `> Building`: "Agents construct Tunnels and surface defensive structures" (slice M4-E), the Laser
            // Turret (slice M4-S1; `factions/syndicate/structures/laser_turret.md > Constructed by`: "Agent
            // (anywhere)").
            builds: &[ObjectKind::Tunnel, ObjectKind::LaserTurret],
        }),
    }),
    structure: None,
    default_settings: None,
};

#[cfg(test)]
mod tests {
    use glam::Vec2;

    use super::*;
    use crate::combat::domain_allows;
    use crate::defs::Domain;
    use crate::defs::syndicate::tests::timings;

    /// A 36×36 HeavyInfantry unit, MaxHP 75, PointArmor 1, FullArmor 1, SightRange 5, TunnelSpaceCost 1, Groupable
    /// false, MaxSpeed 6, no turret, with its pistol (user, 2026-11-24; "Numbers are placeholders"): FullyConnected
    /// Ranged, ground only, single target, 5 damage at up to 2 grid units, "One shot every 20 frames (1.25 s)". Its
    /// gathering comes with slice M4-D.
    #[test]
    fn the_agent_follows_the_design() {
        let agent = ObjectKind::Agent.def();
        assert_eq!(
            (agent.category, agent.faction, agent.silhouette()),
            (Category::Unit, Some(Faction::Syndicate), Some(Vec2::new(36.0, 36.0)))
        );
        let one = Armor { point: 1.0, full: 1.0 };
        assert_eq!(
            (agent.max_hp, agent.max_shields, agent.armor, agent.sight_range),
            (Some(75.0), None, one, 5.0)
        );
        assert_eq!(
            (
                agent.selectable,
                agent.destructible,
                agent.groupable,
                agent.needs_flat_ground
            ),
            (true, true, false, false)
        );
        let unit = agent.unit.as_ref().unwrap();
        assert_eq!(unit.base, UnitBase::HeavyInfantry);
        let movement = MovementAttrs::TurnRate {
            turn_rate: PI,
            acceleration: None,
            deceleration: None,
            max_speed: 6.0,
        };
        assert_eq!(unit.movement, movement);
        assert_eq!(
            (unit.turret.is_none(), unit.tunnel_space_cost, unit.worker.is_some()),
            (true, Some(1), true),
            "its gathering: `economy::gathering`'s tests have the numbers"
        );
        let attack = unit.attack.unwrap();
        assert_eq!(
            attack.attack_type,
            AttackType::FullyConnected(FullyConnectedSubtype::Ranged)
        );
        assert_eq!(
            (attack.target_domain, attack.target_type, attack.damage),
            (TargetDomain::Ground, TargetType::SingleTarget, 5.0)
        );
        assert!(domain_allows(attack.target_domain, Domain::Ground));
        assert!(!domain_allows(attack.target_domain, Domain::Air), "ground only");
        assert_eq!(
            (attack.range, attack.min_range, attack.projectile_speed),
            (2.0, 0.0, None)
        );
        assert_eq!(timings(&attack), ((3, 1, 2, 14), 20), "one shot every 20 frames");
    }
}
