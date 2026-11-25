//! The Drone — `factions/colonists/units/drone.md`.

use crate::defs::units::{DropOffChoice, LoadDef, MovementAttrs, UnitBase, UnitDef, WorkerDef};
use crate::defs::{Armor, Category, Footprint, ObjectDef, ObjectKind, Recipe};
use crate::math::PI;
use crate::player::Faction;

/// `factions/colonists/economy.md > Beacon Deployment` (`factions/colonists/units/drone.md > Deploying`): the Drone
/// deploys into a Beacon, in place, as a transformation's recipe (`crate::deploy`, `crate::Transform::new`), for no
/// Space Crystals (the design names no cost: the Drone itself is used up).
pub const BEACON_DEPLOYMENT: Recipe = Recipe {
    kind: ObjectKind::Beacon,
    cost: 0,
    frames: 48, // `factions/colonists/economy.md > Deployment Duration`: "48 frames (3 seconds)"
    requires: &[],
};

/// `factions/colonists/units/drone.md`: the worker, unarmed. It moves by DragMovement (`movement::drag`, slice M3-G,
/// plan decision T25): it thrusts up to the equilibrium of 8 space units/frame (`MovementAttrs::max_speed`), 4 on
/// Rugged terrain, and brakes with its omni-directional thrust. Design change 0205 raised its thrust and drag ×1.5
/// (from 0.6 / 0.2 / 0.1), the same top speed reached sooner: about 65% of it after 7 frames, close to all of it
/// after 20 (slice M3-K3).
pub(crate) static DRONE: ObjectDef = ObjectDef {
    kind: ObjectKind::Drone,
    name: "Drone",
    faction: Some(Faction::Colonists),                     // Faction - Colonists
    category: Category::Unit,                              // Entity Type - Unit
    footprint: Footprint::Silhouette { w: 36.0, h: 36.0 }, // Silhouette - 36x36 space units (square)
    selectable: true,                                      // `core/entities.md > Object Type > Selectable - true`
    destructible: true,                                    // Destructible - true
    groupable: true,         // Groupable - true (design change 0125: "Drone Groupable: false → true")
    sight_range: 5.0,        // SightRange - 5 grid units
    max_hp: Some(20.0),      // MaxHP - 20
    max_shields: Some(10.0), // MaxShields - 10
    armor: Armor {
        point: 0.0, // PointArmor - 0
        full: 0.0,  // FullArmor - 0
    },
    needs_flat_ground: false,
    unit: Some(UnitDef {
        base: UnitBase::HoverVehicle, // UnitBase - HoverVehicle
        // UnitBaseAttributes[HoverVehicle], the three DragMovement numbers since design change 0205
        movement: MovementAttrs::Drag {
            forward_acceleration: 0.9,          // ForwardAcceleration: 0.9 space units/frame squared (0205)
            omni_directional_acceleration: 0.3, // OmniDirectionalAcceleration: 0.3 space units/frame squared (0205)
            drag_ratio: 0.15,                   // DragRatio: 0.15 per frame (0205)
            turn_rate: PI / 4.0,                // TurnRate: 45 degrees/frame
        },
        rugged_terrain_defense_bonus: None, // only LightInfantry has it
        // TurretAttributes - None: the HoverVehicle base can carry a turret (CanHaveTurret), and the Drone has none
        // (design change 0154): it is unarmed.
        turret: None,
        attack: None, // AttackAttributes - None (unarmed)
        unit_control_cost: None,
        tunnel_space_cost: None,
        beacon_capacity_cost: Some(1), // BeaconCapacityCost - 1
        riot_shield: None,
        conversion: None,
        deploy: Some(BEACON_DEPLOYMENT), // > Deploying
        // `> Gathering`
        worker: Some(WorkerDef {
            crystals: LoadDef {
                carry_capacity: 8,   // CarryCapacity: 8 per load
                pick_up_ticks: 32,   // MiningDuration: 32 frames
                drop_off_ticks: 2,   // DropOffDuration: 2 frames
                pick_up_reach: 8.0,  // MiningReach: 8 space units (the design's placeholder)
                drop_off_reach: 8.0, // DropOffReach: 8 space units (the design's placeholder)
            },
            supplies: None,
            shares_patches: false,             // > Patch Occupancy (one Drone per patch)
            drop_off: DropOffChoice::Assigned, // > Assigned Drop-off
            search_radius: 6.0, // > Patch Occupancy > Search radius: 6 grid units (placeholder, design change 0123)
            pass_through: true, // > Gathering Drones pass through each other (design change 0106)
            // It deploys instead (`deploy`, plan decision T51).
            builds: &[],
        }),
    }),
    structure: None,
    default_settings: None,
};

#[cfg(test)]
mod tests {
    use glam::Vec2;

    use super::*;
    use crate::defs::MovementModel;

    /// Unarmed, on the HoverVehicle base, with DragMovement attributes whose equilibrium is 8 space units per frame
    /// (`movement::drag`, slice M3-G).
    #[test]
    fn the_drone_follows_the_design() {
        let def = ObjectKind::Drone.def();
        assert_eq!((def.category, def.faction), (Category::Unit, Some(Faction::Colonists)));
        // Silhouette - 36x36 space units (square)
        assert_eq!(def.silhouette(), Some(Vec2::new(36.0, 36.0)));
        // MaxHP - 20, MaxShields - 10, PointArmor - 0, FullArmor - 0, SightRange - 5 grid units
        assert_eq!(
            (def.max_hp, def.max_shields, def.armor, def.sight_range),
            (Some(20.0), Some(10.0), Armor { point: 0.0, full: 0.0 }, 5.0)
        );
        // Destructible - true, Groupable - true (design change 0125)
        assert_eq!((def.destructible, def.groupable, def.selectable), (true, true, true));
        let unit = def.unit.as_ref().unwrap();
        // UnitBase - HoverVehicle, BeaconCapacityCost - 1
        assert_eq!(
            (unit.base, unit.beacon_capacity_cost),
            (UnitBase::HoverVehicle, Some(1))
        );
        // UnitBaseAttributes[HoverVehicle]: TurnRate 45 degrees/frame, and since design change 0205
        // ForwardAcceleration 0.9 space units/frame², OmniDirectionalAcceleration 0.3 space units/frame², DragRatio
        // 0.15 per frame (were 0.6 / 0.2 / 0.1).
        assert_eq!(
            unit.movement,
            MovementAttrs::Drag {
                forward_acceleration: 0.9,
                omni_directional_acceleration: 0.3,
                drag_ratio: 0.15,
                turn_rate: PI / 4.0,
            }
        );
        assert_eq!(unit.movement.model(), MovementModel::Drag);
        // "MaxSpeed equilibrium = (0.9 + 0.3) / 0.15 = 8 space units/frame": exactly, in f32 too.
        assert_eq!(unit.movement.max_speed(), 8.0);
        assert_eq!(unit.movement.turn_rate(), Some(PI / 4.0));
        // TurretAttributes - None, AttackAttributes - None (unarmed)
        assert_eq!((unit.turret, unit.attack), (None, None));
        assert!(def.structure.is_none());
    }
}
