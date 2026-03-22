#![allow(dead_code)]
use bevy::prelude::*;
use crate::types::{GridPosition, UnitBaseEnum, MovementModelEnum, DomainEnum};

/// Component storing movement target position
#[derive(Component)]
pub struct MoveTarget(pub Vec3);

/// Component storing unit velocity
#[derive(Component)]
pub struct Velocity(pub Vec3);

/// Component storing movement speed
#[derive(Component)]
pub struct MovementSpeed(pub f32);

/// Component storing rotation speed (radians per second)
#[derive(Component)]
pub struct RotationSpeed(pub f32);

/// Component for visual move target marker
#[derive(Component)]
pub struct MoveTargetMarker;

/// Component storing an entity target for move-to-object behavior.
/// The behavior system reads the target entity's Transform each tick
/// to track a moving target.
#[derive(Component)]
pub struct MoveObjectTarget {
    pub entity: Entity,
}

/// Component storing a path as a sequence of waypoints
#[derive(Component)]
pub struct Path {
    pub waypoints: Vec<Vec3>,
    pub current_waypoint: usize,
}

/// Node for A* pathfinding
#[derive(Clone)]
pub struct PathNode {
    pub position: GridPosition,
    pub g_cost: f32,
    pub h_cost: f32,
    pub f_cost: f32,
    pub parent: Option<GridPosition>,
}

impl PathNode {
    pub fn new(position: GridPosition, g_cost: f32, h_cost: f32, parent: Option<GridPosition>) -> Self {
        Self {
            position,
            g_cost,
            h_cost,
            f_cost: g_cost + h_cost,
            parent,
        }
    }
}

impl Eq for PathNode {}

impl PartialEq for PathNode {
    fn eq(&self, other: &Self) -> bool {
        self.f_cost.eq(&other.f_cost)
    }
}

impl Ord for PathNode {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        // Primary: lower f_cost is better (reversed for max-heap → min-heap)
        // Tie-break: lower h_cost is better (more progress made toward goal),
        // which produces straighter diagonal paths on open terrain.
        match other.f_cost.partial_cmp(&self.f_cost).unwrap_or(std::cmp::Ordering::Equal) {
            std::cmp::Ordering::Equal => {
                other.h_cost.partial_cmp(&self.h_cost).unwrap_or(std::cmp::Ordering::Equal)
            }
            ord => ord,
        }
    }
}

impl PartialOrd for PathNode {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.cmp(other))
    }
}

// === Locomotion and Orientation State Identifiers ===

/// Locomotion state for movement constraint lookup.
/// NOT an ECS component — used by constraint tables.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum Locomotion {
    Stationary,
    Moving,
    Stopping,
    Reversing,
}

/// Orientation state for movement constraint lookup.
/// NOT an ECS component — used by constraint tables.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum Orientation {
    Turning,
    Maintaining,
}

/// The turn rate constraint for a given Locomotion+Orientation combination.
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum TurnRateConstraint {
    /// Combination not allowed for this movement model
    Invalid,
    /// Combination valid, no turning (Maintaining)
    Valid,
    /// maxTurnRate is a constant value
    FixedRate(f32),
    /// maxTurnRate depends on current speed and model params
    SpeedDependent,
    /// No turn rate limit
    Unconstrained,
}

// === Movement Model Parameter Structs ===

/// Parameters for TurnRate movement model (infantry, mechs)
#[derive(Component, Clone, Debug)]
pub struct TurnRateMovementParams {
    pub turn_rate: f32,
    pub acceleration: f32,
    pub deceleration: f32,
    pub max_speed: f32,
}

/// Parameters for FixedTurnRadius movement model (wheeled vehicles)
#[derive(Component, Clone, Debug)]
pub struct FixedTurnRadiusMovementParams {
    pub minimum_turn_radius: f32,
    pub forward_acceleration: f32,
    pub forward_max_speed: f32,
    pub reverse_acceleration: f32,
    pub reverse_max_speed: f32,
    pub deceleration: f32,
}

/// Parameters for SpeedTurnRadius movement model (tracked vehicles, drill units)
#[derive(Component, Clone, Debug)]
pub struct SpeedTurnRadiusMovementParams {
    pub speed_to_turn_radius_ratio: f32,
    pub acceleration: f32,
    pub deceleration: f32,
    pub max_speed: f32,
}

/// Parameters for Drag movement model (hover vehicles, hovercraft)
#[derive(Component, Clone, Debug)]
pub struct DragMovementParams {
    pub forward_acceleration: f32,
    pub non_forward_acceleration: f32,
    pub drag_ratio: f32,
    pub turn_rate: f32,
}

/// Parameters for Glider movement model (gliders)
#[derive(Component, Clone, Debug)]
pub struct GliderMovementParams {
    pub idle_speed: f32,
    pub max_speed: f32,
    pub acceleration: f32,
    pub deceleration: f32,
    pub max_centripetal_acceleration: f32,
}

/// Unified movement params component — wraps all 5 movement model types
#[derive(Component, Clone, Debug)]
pub enum MovementParams {
    TurnRate(TurnRateMovementParams),
    FixedTurnRadius(FixedTurnRadiusMovementParams),
    SpeedTurnRadius(SpeedTurnRadiusMovementParams),
    Drag(DragMovementParams),
    Glider(GliderMovementParams),
}

// === Locomotion-Orientation Constraint Methods ===

impl TurnRateMovementParams {
    /// Get the turn rate constraint for a given locomotion+orientation state.
    /// TurnRate model: all 6 combos valid. All Turning → FixedRate(turn_rate).
    pub fn locomotion_orientation_constraint(&self, loco: Locomotion, orient: Orientation) -> TurnRateConstraint {
        match (loco, orient) {
            (Locomotion::Reversing, _) => TurnRateConstraint::Invalid,
            (_, Orientation::Maintaining) => TurnRateConstraint::Valid,
            (_, Orientation::Turning) => TurnRateConstraint::FixedRate(self.turn_rate),
        }
    }
}

impl FixedTurnRadiusMovementParams {
    /// Get the turn rate constraint for a given locomotion+orientation state.
    /// FixedTurnRadius: Stationary+Turning and Stopping+Turning are invalid.
    /// Moving+Turning and Reversing+Turning are speed-dependent (currentSpeed / minimum_turn_radius).
    pub fn locomotion_orientation_constraint(&self, loco: Locomotion, orient: Orientation) -> TurnRateConstraint {
        match (loco, orient) {
            (_, Orientation::Maintaining) => TurnRateConstraint::Valid,
            (Locomotion::Stationary, Orientation::Turning) => TurnRateConstraint::Invalid,
            (Locomotion::Stopping, Orientation::Turning) => TurnRateConstraint::Invalid,
            (Locomotion::Moving, Orientation::Turning) => TurnRateConstraint::SpeedDependent,
            (Locomotion::Reversing, Orientation::Turning) => TurnRateConstraint::SpeedDependent,
        }
    }

    /// Calculate max turn rate at a given speed: currentSpeed / minimum_turn_radius
    pub fn max_turn_rate_at_speed(&self, current_speed: f32) -> f32 {
        if self.minimum_turn_radius == 0.0 {
            return f32::INFINITY;
        }
        current_speed / self.minimum_turn_radius
    }
}

impl SpeedTurnRadiusMovementParams {
    /// Get the turn rate constraint for a given locomotion+orientation state.
    /// SpeedTurnRadius: Stationary+Turning and Stopping+Turning are unconstrained.
    /// Moving+Turning and Reversing+Turning are speed-dependent.
    pub fn locomotion_orientation_constraint(&self, loco: Locomotion, orient: Orientation) -> TurnRateConstraint {
        match (loco, orient) {
            (_, Orientation::Maintaining) => TurnRateConstraint::Valid,
            (Locomotion::Stationary, Orientation::Turning) => TurnRateConstraint::Unconstrained,
            (Locomotion::Stopping, Orientation::Turning) => TurnRateConstraint::Unconstrained,
            (Locomotion::Moving, Orientation::Turning) => TurnRateConstraint::SpeedDependent,
            (Locomotion::Reversing, Orientation::Turning) => TurnRateConstraint::SpeedDependent,
        }
    }

    /// Calculate max turn rate at a given speed using speed_to_turn_radius_ratio
    pub fn max_turn_rate_at_speed(&self, current_speed: f32) -> f32 {
        let turn_radius = current_speed * self.speed_to_turn_radius_ratio;
        if turn_radius == 0.0 {
            return f32::INFINITY;
        }
        current_speed / turn_radius
    }
}

impl DragMovementParams {
    /// Get the turn rate constraint for a given locomotion+orientation state.
    /// Drag model: same pattern as TurnRate — all Turning → FixedRate(turn_rate).
    pub fn locomotion_orientation_constraint(&self, loco: Locomotion, orient: Orientation) -> TurnRateConstraint {
        match (loco, orient) {
            (Locomotion::Reversing, _) => TurnRateConstraint::Invalid,
            (_, Orientation::Maintaining) => TurnRateConstraint::Valid,
            (_, Orientation::Turning) => TurnRateConstraint::FixedRate(self.turn_rate),
        }
    }

    /// Derived max speed: (non_forward_acceleration + forward_acceleration) / drag_ratio
    pub fn max_speed(&self) -> f32 {
        (self.non_forward_acceleration + self.forward_acceleration) / self.drag_ratio
    }
}

impl GliderMovementParams {
    /// Get the turn rate constraint for a given locomotion+orientation state.
    /// Glider: only Moving+Turning (SpeedDependent) and Moving+Maintaining are valid.
    pub fn locomotion_orientation_constraint(&self, loco: Locomotion, orient: Orientation) -> TurnRateConstraint {
        match (loco, orient) {
            (Locomotion::Moving, Orientation::Maintaining) => TurnRateConstraint::Valid,
            (Locomotion::Moving, Orientation::Turning) => TurnRateConstraint::SpeedDependent,
            _ => TurnRateConstraint::Invalid,
        }
    }

    /// Derived turn radius at a given speed: speed^2 / max_centripetal_acceleration
    pub fn turn_radius(&self, speed: f32) -> f32 {
        if self.max_centripetal_acceleration == 0.0 {
            return f32::INFINITY;
        }
        speed * speed / self.max_centripetal_acceleration
    }
}

// === UnitBase Data ===

/// Static data for a UnitBase archetype
#[derive(Clone, Debug)]
pub struct UnitBaseData {
    pub domain: DomainEnum,
    pub has_turret: bool,
    pub directional_armor: bool,
    pub rugged_terrain: bool,
    pub crushable: bool,
    pub can_crush: bool,
    pub can_turn_in_place: bool,
    pub can_reverse: bool,
    pub movement_model: MovementModelEnum,
}

impl UnitBaseEnum {
    /// Get the static attributes for this unit base type
    pub fn data(&self) -> UnitBaseData {
        match self {
            UnitBaseEnum::LightInfantry => UnitBaseData {
                domain: DomainEnum::Ground,
                has_turret: false,
                directional_armor: false,
                rugged_terrain: true,
                crushable: true,
                can_crush: false,
                can_turn_in_place: true,
                can_reverse: false,
                movement_model: MovementModelEnum::TurnRate,
            },
            UnitBaseEnum::HeavyInfantry => UnitBaseData {
                domain: DomainEnum::Ground,
                has_turret: false,
                directional_armor: false,
                rugged_terrain: true,
                crushable: false,
                can_crush: false,
                can_turn_in_place: true,
                can_reverse: false,
                movement_model: MovementModelEnum::TurnRate,
            },
            UnitBaseEnum::WheeledVehicle => UnitBaseData {
                domain: DomainEnum::Ground,
                has_turret: true,
                directional_armor: true,
                rugged_terrain: false,
                crushable: false,
                can_crush: false,
                can_turn_in_place: false,
                can_reverse: true,
                movement_model: MovementModelEnum::FixedTurnRadius,
            },
            UnitBaseEnum::TrackedVehicle => UnitBaseData {
                domain: DomainEnum::Ground,
                has_turret: true,
                directional_armor: true,
                rugged_terrain: false,
                crushable: false,
                can_crush: true,
                can_turn_in_place: true,
                can_reverse: true,
                movement_model: MovementModelEnum::SpeedTurnRadius,
            },
            UnitBaseEnum::DrillUnit => UnitBaseData {
                domain: DomainEnum::Underground,
                has_turret: true,
                directional_armor: true,
                rugged_terrain: false,
                crushable: false,
                can_crush: false,
                can_turn_in_place: true,
                can_reverse: true,
                movement_model: MovementModelEnum::SpeedTurnRadius,
            },
            UnitBaseEnum::HoverVehicle => UnitBaseData {
                domain: DomainEnum::Ground,
                has_turret: true,
                directional_armor: true,
                rugged_terrain: false,
                crushable: false,
                can_crush: false,
                can_turn_in_place: true,
                can_reverse: false,
                movement_model: MovementModelEnum::Drag,
            },
            UnitBaseEnum::Mech => UnitBaseData {
                domain: DomainEnum::Ground,
                has_turret: true,
                directional_armor: true,
                rugged_terrain: true,
                crushable: false,
                can_crush: true,
                can_turn_in_place: true,
                can_reverse: false,
                movement_model: MovementModelEnum::TurnRate,
            },
            UnitBaseEnum::HoverCraft => UnitBaseData {
                domain: DomainEnum::Air,
                has_turret: true,
                directional_armor: false,
                rugged_terrain: false,
                crushable: false,
                can_crush: false,
                can_turn_in_place: true,
                can_reverse: false,
                movement_model: MovementModelEnum::Drag,
            },
            UnitBaseEnum::Glider => UnitBaseData {
                domain: DomainEnum::Air,
                has_turret: true,
                directional_armor: false,
                rugged_terrain: false,
                crushable: false,
                can_crush: false,
                can_turn_in_place: false,
                can_reverse: false,
                movement_model: MovementModelEnum::Glider,
            },
        }
    }

    /// Check if this base type can traverse rugged terrain
    pub fn can_traverse_rugged(&self) -> bool {
        self.data().rugged_terrain
    }

    /// Check if this base type can traverse drillable terrain (underground domain only)
    pub fn can_traverse_drillable(&self) -> bool {
        self.data().domain == DomainEnum::Underground
    }

    /// Get a display name for this unit base type
    pub fn display_name(&self) -> &str {
        match self {
            UnitBaseEnum::LightInfantry => "Light Infantry",
            UnitBaseEnum::HeavyInfantry => "Heavy Infantry",
            UnitBaseEnum::WheeledVehicle => "Wheeled Vehicle",
            UnitBaseEnum::TrackedVehicle => "Tracked Vehicle",
            UnitBaseEnum::DrillUnit => "Drill Unit",
            UnitBaseEnum::HoverVehicle => "Hover Vehicle",
            UnitBaseEnum::Mech => "Mech",
            UnitBaseEnum::HoverCraft => "Hovercraft",
            UnitBaseEnum::Glider => "Glider",
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Helper: all 9 UnitBaseEnum variants
    fn all_variants() -> Vec<UnitBaseEnum> {
        vec![
            UnitBaseEnum::LightInfantry,
            UnitBaseEnum::HeavyInfantry,
            UnitBaseEnum::WheeledVehicle,
            UnitBaseEnum::TrackedVehicle,
            UnitBaseEnum::DrillUnit,
            UnitBaseEnum::HoverVehicle,
            UnitBaseEnum::Mech,
            UnitBaseEnum::HoverCraft,
            UnitBaseEnum::Glider,
        ]
    }

    #[test]
    fn unit_base_enum_has_9_variants() {
        assert_eq!(all_variants().len(), 9);
    }

    // --- LightInfantry ---
    #[test]
    fn light_infantry_data() {
        let d = UnitBaseEnum::LightInfantry.data();
        assert_eq!(d.domain, DomainEnum::Ground);
        assert!(!d.has_turret);
        assert!(!d.directional_armor);
        assert!(d.rugged_terrain);
        assert!(d.crushable);
        assert!(d.can_turn_in_place);
        assert!(!d.can_reverse);
        assert_eq!(d.movement_model, MovementModelEnum::TurnRate);
    }

    // --- HeavyInfantry ---
    #[test]
    fn heavy_infantry_data() {
        let d = UnitBaseEnum::HeavyInfantry.data();
        assert_eq!(d.domain, DomainEnum::Ground);
        assert!(!d.has_turret);
        assert!(!d.directional_armor);
        assert!(d.rugged_terrain);
        assert!(!d.crushable);
        assert!(d.can_turn_in_place);
        assert!(!d.can_reverse);
        assert_eq!(d.movement_model, MovementModelEnum::TurnRate);
    }

    // --- WheeledVehicle ---
    #[test]
    fn wheeled_vehicle_data() {
        let d = UnitBaseEnum::WheeledVehicle.data();
        assert_eq!(d.domain, DomainEnum::Ground);
        assert!(d.has_turret);
        assert!(d.directional_armor);
        assert!(!d.rugged_terrain);
        assert!(!d.crushable);
        assert!(!d.can_turn_in_place);
        assert!(d.can_reverse);
        assert_eq!(d.movement_model, MovementModelEnum::FixedTurnRadius);
    }

    // --- TrackedVehicle ---
    #[test]
    fn tracked_vehicle_data() {
        let d = UnitBaseEnum::TrackedVehicle.data();
        assert_eq!(d.domain, DomainEnum::Ground);
        assert!(d.has_turret);
        assert!(d.directional_armor);
        assert!(!d.rugged_terrain);
        assert!(!d.crushable);
        assert!(d.can_turn_in_place);
        assert!(d.can_reverse);
        assert_eq!(d.movement_model, MovementModelEnum::SpeedTurnRadius);
    }

    // --- DrillUnit ---
    #[test]
    fn drill_unit_data() {
        let d = UnitBaseEnum::DrillUnit.data();
        assert_eq!(d.domain, DomainEnum::Underground);
        assert!(d.has_turret);
        assert!(d.directional_armor);
        assert!(!d.rugged_terrain);
        assert!(!d.crushable);
        assert!(d.can_turn_in_place);
        assert!(d.can_reverse);
        assert_eq!(d.movement_model, MovementModelEnum::SpeedTurnRadius);
    }

    // --- HoverVehicle ---
    #[test]
    fn hover_vehicle_data() {
        let d = UnitBaseEnum::HoverVehicle.data();
        assert_eq!(d.domain, DomainEnum::Ground);
        assert!(d.has_turret);
        assert!(d.directional_armor);
        assert!(!d.rugged_terrain);
        assert!(!d.crushable);
        assert!(d.can_turn_in_place);
        assert!(!d.can_reverse);
        assert_eq!(d.movement_model, MovementModelEnum::Drag);
    }

    // --- Mech ---
    #[test]
    fn mech_data() {
        let d = UnitBaseEnum::Mech.data();
        assert_eq!(d.domain, DomainEnum::Ground);
        assert!(d.has_turret);
        assert!(d.directional_armor);
        assert!(d.rugged_terrain);
        assert!(!d.crushable);
        assert!(d.can_turn_in_place);
        assert!(!d.can_reverse);
        assert_eq!(d.movement_model, MovementModelEnum::TurnRate);
    }

    // --- HoverCraft ---
    #[test]
    fn hovercraft_data() {
        let d = UnitBaseEnum::HoverCraft.data();
        assert_eq!(d.domain, DomainEnum::Air);
        assert!(d.has_turret);
        assert!(!d.directional_armor);
        assert!(!d.rugged_terrain);
        assert!(!d.crushable);
        assert!(d.can_turn_in_place);
        assert!(!d.can_reverse);
        assert_eq!(d.movement_model, MovementModelEnum::Drag);
    }

    // --- Glider ---
    #[test]
    fn glider_data() {
        let d = UnitBaseEnum::Glider.data();
        assert_eq!(d.domain, DomainEnum::Air);
        assert!(d.has_turret);
        assert!(!d.directional_armor);
        assert!(!d.rugged_terrain);
        assert!(!d.crushable);
        assert!(!d.can_turn_in_place);
        assert!(!d.can_reverse);
        assert_eq!(d.movement_model, MovementModelEnum::Glider);
    }

    // --- can_traverse_rugged ---
    #[test]
    fn can_traverse_rugged_only_infantry_and_mech() {
        let rugged_bases = all_variants()
            .into_iter()
            .filter(|b| b.can_traverse_rugged())
            .collect::<Vec<_>>();
        assert_eq!(rugged_bases, vec![
            UnitBaseEnum::LightInfantry,
            UnitBaseEnum::HeavyInfantry,
            UnitBaseEnum::Mech,
        ]);
    }

    // --- can_traverse_drillable ---
    #[test]
    fn can_traverse_drillable_only_drill_unit() {
        let drillable_bases = all_variants()
            .into_iter()
            .filter(|b| b.can_traverse_drillable())
            .collect::<Vec<_>>();
        assert_eq!(drillable_bases, vec![UnitBaseEnum::DrillUnit]);
    }

    // --- display_name ---
    #[test]
    fn display_names_correct() {
        assert_eq!(UnitBaseEnum::LightInfantry.display_name(), "Light Infantry");
        assert_eq!(UnitBaseEnum::HeavyInfantry.display_name(), "Heavy Infantry");
        assert_eq!(UnitBaseEnum::WheeledVehicle.display_name(), "Wheeled Vehicle");
        assert_eq!(UnitBaseEnum::TrackedVehicle.display_name(), "Tracked Vehicle");
        assert_eq!(UnitBaseEnum::DrillUnit.display_name(), "Drill Unit");
        assert_eq!(UnitBaseEnum::HoverVehicle.display_name(), "Hover Vehicle");
        assert_eq!(UnitBaseEnum::Mech.display_name(), "Mech");
        assert_eq!(UnitBaseEnum::HoverCraft.display_name(), "Hovercraft");
        assert_eq!(UnitBaseEnum::Glider.display_name(), "Glider");
    }

    // --- Domain distribution ---
    #[test]
    fn ground_domain_bases() {
        let ground = all_variants()
            .into_iter()
            .filter(|b| b.data().domain == DomainEnum::Ground)
            .collect::<Vec<_>>();
        assert_eq!(ground.len(), 6); // LI, HI, Wheeled, Tracked, HoverVehicle, Mech
    }

    #[test]
    fn air_domain_bases() {
        let air = all_variants()
            .into_iter()
            .filter(|b| b.data().domain == DomainEnum::Air)
            .collect::<Vec<_>>();
        assert_eq!(air, vec![UnitBaseEnum::HoverCraft, UnitBaseEnum::Glider]);
    }

    #[test]
    fn underground_domain_bases() {
        let underground = all_variants()
            .into_iter()
            .filter(|b| b.data().domain == DomainEnum::Underground)
            .collect::<Vec<_>>();
        assert_eq!(underground, vec![UnitBaseEnum::DrillUnit]);
    }

    // --- Movement model distribution ---
    #[test]
    fn turn_rate_bases() {
        let tr = all_variants()
            .into_iter()
            .filter(|b| b.data().movement_model == MovementModelEnum::TurnRate)
            .collect::<Vec<_>>();
        assert_eq!(tr, vec![
            UnitBaseEnum::LightInfantry,
            UnitBaseEnum::HeavyInfantry,
            UnitBaseEnum::Mech,
        ]);
    }

    // --- Can crush: only TrackedVehicle and Mech ---
    #[test]
    fn only_tracked_and_mech_can_crush() {
        let crushers = all_variants()
            .into_iter()
            .filter(|b| b.data().can_crush)
            .collect::<Vec<_>>();
        assert_eq!(crushers, vec![UnitBaseEnum::TrackedVehicle, UnitBaseEnum::Mech]);
    }

    // --- Crushable only LightInfantry ---
    #[test]
    fn only_light_infantry_is_crushable() {
        let crushable = all_variants()
            .into_iter()
            .filter(|b| b.data().crushable)
            .collect::<Vec<_>>();
        assert_eq!(crushable, vec![UnitBaseEnum::LightInfantry]);
    }

    // --- All variants produce valid data ---
    #[test]
    fn all_variants_return_valid_data() {
        for variant in all_variants() {
            let d = variant.data();
            let name = variant.display_name();
            assert!(!name.is_empty(), "display_name should not be empty for {:?}", variant);
            // Movement model should be one of 5 known types
            match d.movement_model {
                MovementModelEnum::TurnRate
                | MovementModelEnum::FixedTurnRadius
                | MovementModelEnum::SpeedTurnRadius
                | MovementModelEnum::Drag
                | MovementModelEnum::Glider => {}
            }
        }
    }

    // ===== Movement Model Constraint Tests =====

    fn sample_turn_rate_params() -> TurnRateMovementParams {
        TurnRateMovementParams {
            turn_rate: 3.0,
            acceleration: 5.0,
            deceleration: 10.0,
            max_speed: 8.0,
        }
    }

    fn sample_fixed_turn_radius_params() -> FixedTurnRadiusMovementParams {
        FixedTurnRadiusMovementParams {
            minimum_turn_radius: 4.0,
            forward_acceleration: 5.0,
            forward_max_speed: 10.0,
            reverse_acceleration: 3.0,
            reverse_max_speed: 5.0,
            deceleration: 8.0,
        }
    }

    fn sample_speed_turn_radius_params() -> SpeedTurnRadiusMovementParams {
        SpeedTurnRadiusMovementParams {
            speed_to_turn_radius_ratio: 0.5,
            acceleration: 5.0,
            deceleration: 10.0,
            max_speed: 12.0,
        }
    }

    fn sample_drag_params() -> DragMovementParams {
        DragMovementParams {
            forward_acceleration: 6.0,
            non_forward_acceleration: 4.0,
            drag_ratio: 2.0,
            turn_rate: 2.5,
        }
    }

    fn sample_glider_params() -> GliderMovementParams {
        GliderMovementParams {
            idle_speed: 5.0,
            max_speed: 15.0,
            acceleration: 3.0,
            deceleration: 6.0,
            max_centripetal_acceleration: 10.0,
        }
    }

    // --- TurnRate constraint table ---
    #[test]
    fn turn_rate_all_maintaining_valid() {
        let p = sample_turn_rate_params();
        assert_eq!(p.locomotion_orientation_constraint(Locomotion::Stationary, Orientation::Maintaining), TurnRateConstraint::Valid);
        assert_eq!(p.locomotion_orientation_constraint(Locomotion::Moving, Orientation::Maintaining), TurnRateConstraint::Valid);
        assert_eq!(p.locomotion_orientation_constraint(Locomotion::Stopping, Orientation::Maintaining), TurnRateConstraint::Valid);
    }

    #[test]
    fn turn_rate_all_turning_fixed_rate() {
        let p = sample_turn_rate_params();
        assert_eq!(p.locomotion_orientation_constraint(Locomotion::Stationary, Orientation::Turning), TurnRateConstraint::FixedRate(3.0));
        assert_eq!(p.locomotion_orientation_constraint(Locomotion::Moving, Orientation::Turning), TurnRateConstraint::FixedRate(3.0));
        assert_eq!(p.locomotion_orientation_constraint(Locomotion::Stopping, Orientation::Turning), TurnRateConstraint::FixedRate(3.0));
    }

    #[test]
    fn turn_rate_reversing_invalid() {
        let p = sample_turn_rate_params();
        assert_eq!(p.locomotion_orientation_constraint(Locomotion::Reversing, Orientation::Turning), TurnRateConstraint::Invalid);
        assert_eq!(p.locomotion_orientation_constraint(Locomotion::Reversing, Orientation::Maintaining), TurnRateConstraint::Invalid);
    }

    // --- FixedTurnRadius constraint table ---
    #[test]
    fn fixed_turn_radius_all_maintaining_valid() {
        let p = sample_fixed_turn_radius_params();
        assert_eq!(p.locomotion_orientation_constraint(Locomotion::Stationary, Orientation::Maintaining), TurnRateConstraint::Valid);
        assert_eq!(p.locomotion_orientation_constraint(Locomotion::Moving, Orientation::Maintaining), TurnRateConstraint::Valid);
        assert_eq!(p.locomotion_orientation_constraint(Locomotion::Stopping, Orientation::Maintaining), TurnRateConstraint::Valid);
        assert_eq!(p.locomotion_orientation_constraint(Locomotion::Reversing, Orientation::Maintaining), TurnRateConstraint::Valid);
    }

    #[test]
    fn fixed_turn_radius_stationary_stopping_turning_invalid() {
        let p = sample_fixed_turn_radius_params();
        assert_eq!(p.locomotion_orientation_constraint(Locomotion::Stationary, Orientation::Turning), TurnRateConstraint::Invalid);
        assert_eq!(p.locomotion_orientation_constraint(Locomotion::Stopping, Orientation::Turning), TurnRateConstraint::Invalid);
    }

    #[test]
    fn fixed_turn_radius_moving_reversing_turning_speed_dependent() {
        let p = sample_fixed_turn_radius_params();
        assert_eq!(p.locomotion_orientation_constraint(Locomotion::Moving, Orientation::Turning), TurnRateConstraint::SpeedDependent);
        assert_eq!(p.locomotion_orientation_constraint(Locomotion::Reversing, Orientation::Turning), TurnRateConstraint::SpeedDependent);
    }

    #[test]
    fn fixed_turn_radius_max_turn_rate_at_speed() {
        let p = sample_fixed_turn_radius_params();
        // currentSpeed / minimum_turn_radius = 8.0 / 4.0 = 2.0
        assert!((p.max_turn_rate_at_speed(8.0) - 2.0).abs() < f32::EPSILON);
        assert!((p.max_turn_rate_at_speed(0.0) - 0.0).abs() < f32::EPSILON);
    }

    // --- SpeedTurnRadius constraint table ---
    #[test]
    fn speed_turn_radius_all_maintaining_valid() {
        let p = sample_speed_turn_radius_params();
        assert_eq!(p.locomotion_orientation_constraint(Locomotion::Stationary, Orientation::Maintaining), TurnRateConstraint::Valid);
        assert_eq!(p.locomotion_orientation_constraint(Locomotion::Moving, Orientation::Maintaining), TurnRateConstraint::Valid);
        assert_eq!(p.locomotion_orientation_constraint(Locomotion::Stopping, Orientation::Maintaining), TurnRateConstraint::Valid);
        assert_eq!(p.locomotion_orientation_constraint(Locomotion::Reversing, Orientation::Maintaining), TurnRateConstraint::Valid);
    }

    #[test]
    fn speed_turn_radius_stationary_stopping_turning_unconstrained() {
        let p = sample_speed_turn_radius_params();
        assert_eq!(p.locomotion_orientation_constraint(Locomotion::Stationary, Orientation::Turning), TurnRateConstraint::Unconstrained);
        assert_eq!(p.locomotion_orientation_constraint(Locomotion::Stopping, Orientation::Turning), TurnRateConstraint::Unconstrained);
    }

    #[test]
    fn speed_turn_radius_moving_reversing_turning_speed_dependent() {
        let p = sample_speed_turn_radius_params();
        assert_eq!(p.locomotion_orientation_constraint(Locomotion::Moving, Orientation::Turning), TurnRateConstraint::SpeedDependent);
        assert_eq!(p.locomotion_orientation_constraint(Locomotion::Reversing, Orientation::Turning), TurnRateConstraint::SpeedDependent);
    }

    #[test]
    fn speed_turn_radius_max_turn_rate_at_speed() {
        let p = sample_speed_turn_radius_params();
        // turn_radius = speed * ratio = 10.0 * 0.5 = 5.0
        // max_turn_rate = speed / turn_radius = 10.0 / 5.0 = 2.0
        assert!((p.max_turn_rate_at_speed(10.0) - 2.0).abs() < f32::EPSILON);
    }

    // --- Drag constraint table ---
    #[test]
    fn drag_all_maintaining_valid() {
        let p = sample_drag_params();
        assert_eq!(p.locomotion_orientation_constraint(Locomotion::Stationary, Orientation::Maintaining), TurnRateConstraint::Valid);
        assert_eq!(p.locomotion_orientation_constraint(Locomotion::Moving, Orientation::Maintaining), TurnRateConstraint::Valid);
        assert_eq!(p.locomotion_orientation_constraint(Locomotion::Stopping, Orientation::Maintaining), TurnRateConstraint::Valid);
    }

    #[test]
    fn drag_all_turning_fixed_rate() {
        let p = sample_drag_params();
        assert_eq!(p.locomotion_orientation_constraint(Locomotion::Stationary, Orientation::Turning), TurnRateConstraint::FixedRate(2.5));
        assert_eq!(p.locomotion_orientation_constraint(Locomotion::Moving, Orientation::Turning), TurnRateConstraint::FixedRate(2.5));
        assert_eq!(p.locomotion_orientation_constraint(Locomotion::Stopping, Orientation::Turning), TurnRateConstraint::FixedRate(2.5));
    }

    #[test]
    fn drag_reversing_invalid() {
        let p = sample_drag_params();
        assert_eq!(p.locomotion_orientation_constraint(Locomotion::Reversing, Orientation::Turning), TurnRateConstraint::Invalid);
        assert_eq!(p.locomotion_orientation_constraint(Locomotion::Reversing, Orientation::Maintaining), TurnRateConstraint::Invalid);
    }

    #[test]
    fn drag_max_speed_derived() {
        let p = sample_drag_params();
        // (4.0 + 6.0) / 2.0 = 5.0
        assert!((p.max_speed() - 5.0).abs() < f32::EPSILON);
    }

    // --- Glider constraint table ---
    #[test]
    fn glider_moving_maintaining_valid() {
        let p = sample_glider_params();
        assert_eq!(p.locomotion_orientation_constraint(Locomotion::Moving, Orientation::Maintaining), TurnRateConstraint::Valid);
    }

    #[test]
    fn glider_moving_turning_speed_dependent() {
        let p = sample_glider_params();
        assert_eq!(p.locomotion_orientation_constraint(Locomotion::Moving, Orientation::Turning), TurnRateConstraint::SpeedDependent);
    }

    #[test]
    fn glider_non_moving_all_invalid() {
        let p = sample_glider_params();
        assert_eq!(p.locomotion_orientation_constraint(Locomotion::Stationary, Orientation::Turning), TurnRateConstraint::Invalid);
        assert_eq!(p.locomotion_orientation_constraint(Locomotion::Stationary, Orientation::Maintaining), TurnRateConstraint::Invalid);
        assert_eq!(p.locomotion_orientation_constraint(Locomotion::Stopping, Orientation::Turning), TurnRateConstraint::Invalid);
        assert_eq!(p.locomotion_orientation_constraint(Locomotion::Stopping, Orientation::Maintaining), TurnRateConstraint::Invalid);
        assert_eq!(p.locomotion_orientation_constraint(Locomotion::Reversing, Orientation::Turning), TurnRateConstraint::Invalid);
        assert_eq!(p.locomotion_orientation_constraint(Locomotion::Reversing, Orientation::Maintaining), TurnRateConstraint::Invalid);
    }

    #[test]
    fn glider_turn_radius_derived() {
        let p = sample_glider_params();
        // speed^2 / max_centripetal_acceleration = 10.0^2 / 10.0 = 10.0
        assert!((p.turn_radius(10.0) - 10.0).abs() < f32::EPSILON);
        // speed 0 → turn_radius 0
        assert!((p.turn_radius(0.0) - 0.0).abs() < f32::EPSILON);
    }
}
