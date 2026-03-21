#![allow(dead_code)]
use std::collections::HashSet;
use bevy::prelude::*;
use crate::types::{ObjectEnum, UnitBaseEnum};

/// Marker component for structures using the ConstructionHP rule.
/// When present, the structure's HP scales with construction progress.
/// HP formula: MaxHP × (10% + 90% × progress)
/// Remove this component when construction completes (progress >= 1.0).
#[derive(Component, Clone, Debug)]
pub struct ConstructionHP {
    /// Progress from 0.0 (just started) to 1.0 (complete)
    pub progress: f32,
    /// Total build duration in frames
    pub build_frames: u32,
}

impl ConstructionHP {
    /// Create a new ConstructionHP component with the given build duration.
    /// Progress starts at 0.0.
    pub fn new(build_frames: u32) -> Self {
        Self {
            progress: 0.0,
            build_frames,
        }
    }

    /// Calculate the HP fraction for a given progress value.
    /// Returns a value from 0.10 (at progress 0.0) to 1.0 (at progress 1.0).
    pub fn hp_fraction(progress: f32) -> f32 {
        0.10 + 0.90 * progress.clamp(0.0, 1.0)
    }

    /// Check if construction is complete.
    pub fn is_complete(&self) -> bool {
        self.progress >= 1.0
    }
}

/// Component tracking the power value of a structure
/// Positive = generator, negative = consumer
#[derive(Component, Clone, Debug)]
pub struct PowerValue(pub i32);

/// Component tracking the build radius extension of a structure (in grid units)
#[derive(Component, Clone, Debug)]
pub struct BuildRadiusExtension(pub u32);

/// Cost to construct a structure or produce a unit
#[derive(Clone, Debug)]
pub struct StructureCost {
    pub space_crystals: u32,
    pub build_frames: u32,
}

/// Rally target for unit production buildings
#[derive(Clone, Debug)]
pub enum RallyTarget {
    Location(Vec3),
    Object(Entity),
}

/// Visual marker component for rally point indicators on production structures.
/// Spawned as a mesh entity at the rally point location.
#[derive(Component, Clone, Debug)]
pub struct RallyPointMarker {
    /// The production structure entity this rally point belongs to
    pub owner_structure: Entity,
}

// === Deployment Center ===

/// Instance state for the Deployment Center
#[derive(Component, Clone, Debug, Default)]
pub struct DeploymentCenterState {
    /// Currently constructing this structure type
    pub current_construction: Option<ObjectEnum>,
    /// Frames of progress on current construction
    pub construction_progress: Option<f32>,
    /// Structure ready to be placed by the player
    pub ready_to_place: Option<ObjectEnum>,
}

impl DeploymentCenterState {
    /// Get the cost data for structures the Deployment Center can build
    pub fn construction_cost(object: &ObjectEnum) -> Option<StructureCost> {
        match object {
            ObjectEnum::PowerPlant => Some(StructureCost {
                space_crystals: 150,
                build_frames: 160,
            }),
            ObjectEnum::Barracks => Some(StructureCost {
                space_crystals: 200,
                build_frames: 160,
            }),
            ObjectEnum::SupplyTower => Some(StructureCost {
                space_crystals: 200,
                build_frames: 240,
            }),
            _ => None,
        }
    }

    /// Get the refund amount for cancellation
    /// Full refund during construction, 75% (rounded down) when ready to place
    pub fn cancellation_refund(&self, object: &ObjectEnum) -> Option<u32> {
        if let Some(cost) = Self::construction_cost(object) {
            if self.ready_to_place.is_some() {
                // 75% refund when ready to place
                Some((cost.space_crystals * 3) / 4)
            } else if self.current_construction.is_some() {
                // Full refund during construction
                Some(cost.space_crystals)
            } else {
                None
            }
        } else {
            None
        }
    }
}

// === Power Plant ===
// PowerPlant has no instance state beyond ObjectInstance and structure components.

// === Barracks ===

/// Instance state for the Barracks
#[derive(Component, Clone, Debug, Default)]
pub struct BarracksState {
    /// Rally point for produced units
    pub rally_point: Option<RallyTarget>,
    /// Build queue (max 5 entries)
    pub build_queue: Vec<ObjectEnum>,
    /// Currently building this unit type
    pub current_build: Option<ObjectEnum>,
    /// Frames of progress on current build
    pub current_build_progress: Option<f32>,
}

impl BarracksState {
    /// Maximum entries in the build queue
    pub const MAX_QUEUE_SIZE: usize = 5;

    /// Get the production cost for units the Barracks can produce
    pub fn production_cost(object: &ObjectEnum) -> Option<StructureCost> {
        match object {
            ObjectEnum::Peacekeeper => Some(StructureCost {
                space_crystals: 50,
                build_frames: 80,
            }),
            _ => None,
        }
    }

    /// Try to add to build queue (returns false if full)
    pub fn try_queue(&mut self, object: ObjectEnum) -> bool {
        if self.build_queue.len() >= Self::MAX_QUEUE_SIZE {
            return false;
        }
        self.build_queue.push(object);
        true
    }

    /// Cancel the last queued item, or the currently building unit if queue is empty.
    /// Returns the cancelled object type for refund calculation.
    pub fn cancel_last(&mut self) -> Option<ObjectEnum> {
        if let Some(cancelled) = self.build_queue.pop() {
            Some(cancelled)
        } else if let Some(cancelled) = self.current_build.take() {
            self.current_build_progress = None;
            Some(cancelled)
        } else {
            None
        }
    }

    /// Returns true if there is any active production or queued items to cancel.
    pub fn has_cancellable(&self) -> bool {
        !self.build_queue.is_empty() || self.current_build.is_some()
    }
}

// === Extraction Facility ===

/// Instance state for the Extraction Facility
#[derive(Component, Clone, Debug, Default)]
pub struct ExtractionFacilityState {
    /// Whether currently constructing an Extraction Plate
    pub current_construction: bool,
    /// Frames of progress on current construction (None if not building)
    pub construction_progress: Option<f32>,
    /// Extraction Plate ready to be placed by the player
    pub ready_to_place: bool,
}

impl ExtractionFacilityState {
    /// Get the cost data for ExtractionPlate construction
    pub fn construction_cost() -> StructureCost {
        StructureCost {
            space_crystals: 75,
            build_frames: 96,
        }
    }

    /// Get the refund amount for cancellation
    /// Full refund during construction, 75% (rounded down) when ready to place
    pub fn cancellation_refund(&self) -> Option<u32> {
        let cost = Self::construction_cost();
        if self.ready_to_place {
            // 75% refund when ready to place: 75 * 3 / 4 = 56
            Some((cost.space_crystals * 3) / 4)
        } else if self.current_construction {
            // Full refund during construction
            Some(cost.space_crystals)
        } else {
            None
        }
    }
}

// === Extraction Plate ===

/// Instance state for the Extraction Plate
#[derive(Component, Clone, Debug)]
pub struct ExtractionPlateState {
    /// The SpaceCrystalsPatch entity this plate is mining
    pub attached_patch: Entity,
    /// Frames since last mining tick
    pub mining_timer: u32,
}

/// Mining constants for Extraction Plates
pub const EXTRACTION_PLATE_MINING_RATE: u32 = 10;
pub const EXTRACTION_PLATE_RESIDUAL_RATE: u32 = 1;
pub const EXTRACTION_PLATE_MINING_INTERVAL: u32 = 48;

// === Tunnel Expansions ===

/// Marker component for underground tunnel expansions.
/// Links an expansion entity to its parent Tunnel.
#[derive(Component, Clone, Debug)]
pub struct TunnelExpansionMarker {
    /// The parent Tunnel entity that owns this expansion
    pub parent_tunnel: Entity,
}

// === Headquarters ===

/// Instance state for the Headquarters (underground unit production building)
#[derive(Component, Clone, Debug, Default)]
pub struct HeadquartersState {
    /// Rally point for produced units
    pub rally_point: Option<RallyTarget>,
    /// Build queue (max 5 entries)
    pub build_queue: Vec<ObjectEnum>,
    /// Currently building this unit type
    pub current_build: Option<ObjectEnum>,
    /// Frames of progress on current build
    pub current_build_progress: Option<f32>,
}

impl HeadquartersState {
    /// Maximum entries in the build queue
    pub const MAX_QUEUE_SIZE: usize = 5;

    /// Get the production cost for units the Headquarters can produce
    pub fn production_cost(object: &ObjectEnum) -> Option<StructureCost> {
        match object {
            ObjectEnum::SyndicateAgent => Some(StructureCost {
                space_crystals: 100,
                build_frames: 160,
            }),
            ObjectEnum::SyndicateGuard => Some(StructureCost {
                space_crystals: 125,
                build_frames: 120,
            }),
            _ => None,
        }
    }

    /// Try to add to build queue (returns false if full)
    pub fn try_queue(&mut self, object: ObjectEnum) -> bool {
        if self.build_queue.len() >= Self::MAX_QUEUE_SIZE {
            return false;
        }
        self.build_queue.push(object);
        true
    }

    /// Cancel the last queued item, or the currently building unit if queue is empty.
    /// Returns the cancelled object type for refund calculation.
    pub fn cancel_last(&mut self) -> Option<ObjectEnum> {
        if let Some(cancelled) = self.build_queue.pop() {
            Some(cancelled)
        } else if let Some(cancelled) = self.current_build.take() {
            self.current_build_progress = None;
            Some(cancelled)
        } else {
            None
        }
    }

    /// Returns true if there is any active production or queued items to cancel.
    pub fn has_cancellable(&self) -> bool {
        !self.build_queue.is_empty() || self.current_build.is_some()
    }
}

// === Supply Tower ===

/// Instance state for the Supply Tower
#[derive(Component, Clone, Debug, Default)]
pub struct SupplyTowerState {
    /// Entity reference to the attached chopper (auto-dispatched for deliveries)
    pub attached_chopper: Option<Entity>,
    /// Entity reference to a chopper currently landed on the tower (non-attached)
    pub landed_chopper: Option<Entity>,
    /// Entity reference to the scheduled Supply Delivery Station
    pub scheduled_sds: Option<Entity>,
    /// Build queue (max 5 entries)
    pub build_queue: Vec<ObjectEnum>,
    /// Currently building this unit type
    pub current_build: Option<ObjectEnum>,
    /// Frames of progress on current build
    pub current_build_progress: Option<f32>,
    /// Rally point for produced choppers
    pub rally_point: Option<RallyTarget>,
}

impl SupplyTowerState {
    /// Maximum entries in the build queue
    pub const MAX_QUEUE_SIZE: usize = 5;

    /// Get the production cost for units the Supply Tower can produce
    pub fn production_cost(object: &ObjectEnum) -> Option<StructureCost> {
        match object {
            ObjectEnum::SupplyChopper => Some(StructureCost {
                space_crystals: 100,
                build_frames: 160,
            }),
            _ => None,
        }
    }

    /// Try to add to build queue (returns false if full)
    pub fn try_queue(&mut self, object: ObjectEnum) -> bool {
        if self.build_queue.len() >= Self::MAX_QUEUE_SIZE {
            return false;
        }
        self.build_queue.push(object);
        true
    }

    /// Cancel the last queued item, or the currently building unit if queue is empty.
    /// Returns the cancelled object type for refund calculation.
    pub fn cancel_last(&mut self) -> Option<ObjectEnum> {
        if let Some(cancelled) = self.build_queue.pop() {
            Some(cancelled)
        } else if let Some(cancelled) = self.current_build.take() {
            self.current_build_progress = None;
            Some(cancelled)
        } else {
            None
        }
    }

    /// Returns true if there is any active production or queued items to cancel.
    pub fn has_cancellable(&self) -> bool {
        !self.build_queue.is_empty() || self.current_build.is_some()
    }
}

// === Supply Chopper ===

/// Instance state for the Supply Chopper unit
#[derive(Component, Clone, Debug, Default)]
pub struct SupplyChopperState {
    /// Number of supplies currently carried
    pub carried_supplies: u32,
    /// Entity reference to the tower this chopper is attached to (auto-delivery mode)
    pub attached_tower: Option<Entity>,
}

/// Structure stat constants for GDO structures
pub mod gdo_structure_stats {
    // Deployment Center
    pub const DC_MAX_HP: f32 = 1000.0;
    pub const DC_POINT_ARMOR: u32 = 1;
    pub const DC_FULL_ARMOR: u32 = 16;
    pub const DC_BUILD_RADIUS: u32 = 12;
    pub const DC_POWER: i32 = 20;

    // Power Plant
    pub const PP_MAX_HP: f32 = 350.0;
    pub const PP_POINT_ARMOR: u32 = 1;
    pub const PP_FULL_ARMOR: u32 = 4;
    pub const PP_BUILD_RADIUS: u32 = 1;
    pub const PP_POWER: i32 = 20;

    // Barracks
    pub const BK_MAX_HP: f32 = 300.0;
    pub const BK_POINT_ARMOR: u32 = 1;
    pub const BK_FULL_ARMOR: u32 = 6;
    pub const BK_BUILD_RADIUS: u32 = 2;
    pub const BK_POWER: i32 = -30;

    // Extraction Facility
    pub const EF_MAX_HP: f32 = 500.0;
    pub const EF_POINT_ARMOR: u32 = 1;
    pub const EF_FULL_ARMOR: u32 = 9;
    pub const EF_BUILD_RADIUS: u32 = 2;
    pub const EF_POWER: i32 = -15;

    // Extraction Plate
    pub const EP_MAX_HP: f32 = 85.0;
    pub const EP_POINT_ARMOR: u32 = 2;
    pub const EP_FULL_ARMOR: u32 = 2;
    pub const EP_BUILD_RADIUS: u32 = 0;

    // Supply Tower
    pub const ST_MAX_HP: f32 = 400.0;
    pub const ST_POINT_ARMOR: u32 = 1;
    pub const ST_FULL_ARMOR: u32 = 9;
    pub const ST_BUILD_RADIUS: u32 = 1;
    pub const ST_POWER: i32 = -15;
    pub const ST_SC_COST: u32 = 200;
    pub const ST_BUILD_FRAMES: u32 = 160;

    // Supply Chopper
    pub const SC_MAX_HP: f32 = 150.0;
    pub const SC_POINT_ARMOR: u32 = 1;
    pub const SC_FULL_ARMOR: u32 = 1;
}

/// Structure stat constants for Syndicate structures
pub mod syndicate_structure_stats {
    // Tunnel
    pub const TUNNEL_T1_MAX_HP: f32 = 600.0;
    pub const TUNNEL_T2_MAX_HP: f32 = 800.0;
    pub const TUNNEL_T3_MAX_HP: f32 = 1000.0;
    pub const TUNNEL_POINT_ARMOR: u32 = 1;
    pub const TUNNEL_FULL_ARMOR: u32 = 16;
    pub const TUNNEL_T1_SPACE: u32 = 20;
    pub const TUNNEL_T2_SPACE: u32 = 30;
    pub const TUNNEL_T3_SPACE: u32 = 40;
    pub const TUNNEL_T1_AREA_RADIUS: u32 = 3;
    pub const TUNNEL_T2_AREA_RADIUS: u32 = 4;
    pub const TUNNEL_T3_AREA_RADIUS: u32 = 5;
    pub const TUNNEL_SIGHT_RANGE: u32 = 5;
    pub const TUNNEL_CONSTRUCTION_FRAMES: u32 = 480; // 30 seconds at 16 FPS
    pub const TUNNEL_UPGRADE_FRAMES: u32 = 480; // 30 seconds at 16 FPS

    // Headquarters
    pub const HQ_MAX_HP: f32 = 400.0;
    pub const HQ_POINT_ARMOR: u32 = 1;
    pub const HQ_FULL_ARMOR: u32 = 4;
    pub const HQ_SC_COST: u32 = 200;
    pub const HQ_BUILD_FRAMES: u32 = 400; // 25 seconds at 16 FPS
}

// === Transit Tier ===

/// Transit tier requirements for entering/exiting a Tunnel.
/// Higher tiers allow progressively larger unit bases.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum TransitTier {
    /// Infantry only (LightInfantry, HeavyInfantry)
    Tier1,
    /// Infantry + Vehicles (WheeledVehicle, TrackedVehicle, DrillUnit, HoverVehicle, Mech)
    Tier2,
    /// Infantry + Vehicles + Air (HoverCraft, Glider)
    Tier3,
}

impl TransitTier {
    /// Check if this transit tier allows a given unit base to enter/exit
    pub fn allows_unit_base(&self, base: &UnitBaseEnum) -> bool {
        match self {
            TransitTier::Tier1 => matches!(
                base,
                UnitBaseEnum::LightInfantry | UnitBaseEnum::HeavyInfantry
            ),
            TransitTier::Tier2 => matches!(
                base,
                UnitBaseEnum::LightInfantry
                    | UnitBaseEnum::HeavyInfantry
                    | UnitBaseEnum::WheeledVehicle
                    | UnitBaseEnum::TrackedVehicle
                    | UnitBaseEnum::DrillUnit
                    | UnitBaseEnum::HoverVehicle
                    | UnitBaseEnum::Mech
            ),
            TransitTier::Tier3 => true, // All unit bases allowed
        }
    }
}

// === Tunnel Tier ===

/// Tunnel upgrade tiers with increasing capabilities
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum TunnelTier {
    Tier1,
    Tier2,
    Tier3,
}

impl TunnelTier {
    /// Get the max HP for this tier
    pub fn max_hp(&self) -> f32 {
        use syndicate_structure_stats::*;
        match self {
            TunnelTier::Tier1 => TUNNEL_T1_MAX_HP,
            TunnelTier::Tier2 => TUNNEL_T2_MAX_HP,
            TunnelTier::Tier3 => TUNNEL_T3_MAX_HP,
        }
    }

    /// Get the tunnel space provided by this tier
    pub fn tunnel_space(&self) -> u32 {
        use syndicate_structure_stats::*;
        match self {
            TunnelTier::Tier1 => TUNNEL_T1_SPACE,
            TunnelTier::Tier2 => TUNNEL_T2_SPACE,
            TunnelTier::Tier3 => TUNNEL_T3_SPACE,
        }
    }

    /// Get the tunnel area radius for this tier
    pub fn area_radius(&self) -> u32 {
        use syndicate_structure_stats::*;
        match self {
            TunnelTier::Tier1 => TUNNEL_T1_AREA_RADIUS,
            TunnelTier::Tier2 => TUNNEL_T2_AREA_RADIUS,
            TunnelTier::Tier3 => TUNNEL_T3_AREA_RADIUS,
        }
    }

    /// Get the transit tier for this tunnel tier
    pub fn transit_tier(&self) -> TransitTier {
        match self {
            TunnelTier::Tier1 => TransitTier::Tier1,
            TunnelTier::Tier2 => TransitTier::Tier2,
            TunnelTier::Tier3 => TransitTier::Tier3,
        }
    }

    /// Check if this tunnel tier allows a given unit base to transit
    pub fn can_transit(&self, base: &UnitBaseEnum) -> bool {
        self.transit_tier().allows_unit_base(base)
    }

    /// Get the next upgrade tier, if any
    pub fn next_tier(&self) -> Option<TunnelTier> {
        match self {
            TunnelTier::Tier1 => Some(TunnelTier::Tier2),
            TunnelTier::Tier2 => Some(TunnelTier::Tier3),
            TunnelTier::Tier3 => None,
        }
    }
}

// === Tunnel State ===

/// Component tracking the runtime state of a Tunnel instance
#[derive(Component, Clone, Debug)]
pub struct TunnelState {
    /// Current upgrade tier of this tunnel
    pub tier: TunnelTier,
    /// Current operation mutex (construction/upgrade in progress)
    pub current_operation: Option<TunnelOperation>,
}

impl TunnelState {
    /// Create a new TunnelState at the given tier with no active operation
    pub fn new(tier: TunnelTier) -> Self {
        Self {
            tier,
            current_operation: None,
        }
    }

    /// Create a new TunnelState at Tier 1 (default for newly placed tunnels)
    pub fn default_tier1() -> Self {
        Self::new(TunnelTier::Tier1)
    }

    /// Check if an operation is currently in progress
    pub fn is_busy(&self) -> bool {
        self.current_operation.is_some()
    }
}

/// Operations that can occupy a Tunnel's construction mutex
#[derive(Clone, Debug, PartialEq)]
pub enum TunnelOperation {
    /// Upgrading to the next tier
    Upgrading { target_tier: TunnelTier, progress: f32 },
    /// Building an expansion structure
    BuildingExpansion {
        object: ObjectEnum,
        progress: f32,
        grid_x: i32,
        grid_z: i32,
        rotation: crate::types::StructureRotation,
        flip_horizontal: bool,
        flip_vertical: bool,
    },
}

// === Tunnel Area ===

/// Per-entity underground build zone for a Tunnel.
/// Each Tunnel owns its own area as a Component, unlike the global GdoBuildArea Resource.
#[derive(Component, Clone, Debug)]
pub struct TunnelArea {
    /// Top-left grid coordinate of the 4x4 Tunnel footprint
    pub origin_x: i32,
    pub origin_z: i32,
    /// Set of underground grid cells in this Tunnel's build zone
    pub cells: HashSet<(i32, i32)>,
}

impl TunnelArea {
    /// Create a new TunnelArea centered on the 4x4 footprint at (origin_x, origin_z).
    /// The area extends `tier.area_radius()` cells outward in each direction.
    pub fn new(origin_x: i32, origin_z: i32, tier: &TunnelTier) -> Self {
        let cells = compute_tunnel_area_cells(origin_x, origin_z, tier.area_radius());
        Self {
            origin_x,
            origin_z,
            cells,
        }
    }

    /// Recalculate the area cells after a tier upgrade
    pub fn recalculate(&mut self, tier: &TunnelTier) {
        self.cells = compute_tunnel_area_cells(self.origin_x, self.origin_z, tier.area_radius());
    }

    /// Check if a grid cell is within this Tunnel's build zone
    pub fn contains(&self, x: i32, z: i32) -> bool {
        self.cells.contains(&(x, z))
    }

    /// Check if this area overlaps with another TunnelArea
    pub fn overlaps(&self, other: &TunnelArea) -> bool {
        // Iterate the smaller set for efficiency
        if self.cells.len() <= other.cells.len() {
            self.cells.iter().any(|cell| other.cells.contains(cell))
        } else {
            other.cells.iter().any(|cell| self.cells.contains(cell))
        }
    }

    /// Check if an expansion footprint fits entirely within this area
    pub fn fits_expansion(&self, pos_x: i32, pos_z: i32, size_x: u32, size_z: u32) -> bool {
        for dx in 0..size_x as i32 {
            for dz in 0..size_z as i32 {
                if !self.contains(pos_x + dx, pos_z + dz) {
                    return false;
                }
            }
        }
        true
    }
}

/// Compute the set of grid cells for a Tunnel area.
/// The 4x4 footprint occupies `origin_x..origin_x+4, origin_z..origin_z+4`.
/// The area extends `radius` cells outward: total side = `2*radius + 4`.
pub fn compute_tunnel_area_cells(origin_x: i32, origin_z: i32, radius: u32) -> HashSet<(i32, i32)> {
    let r = radius as i32;
    let mut cells = HashSet::new();
    for x in (origin_x - r)..(origin_x + 4 + r) {
        for z in (origin_z - r)..(origin_z + 4 + r) {
            cells.insert((x, z));
        }
    }
    cells
}

// === Tunnel Cost Functions ===

/// Cost (in Supplies) to build the nth Tunnel (0-indexed count of existing Tunnels).
/// 1st Tunnel = 0, 2nd = 1, 3rd = 2, etc.
pub fn tunnel_construction_cost(existing_tunnel_count: u32) -> u32 {
    existing_tunnel_count
}

/// Cost (in Supplies) to upgrade to Tier 2, given count of T2+ Tunnels already owned.
/// Formula: 2 + 2 * existing_t2_plus_count
pub fn tunnel_t2_upgrade_cost(existing_t2_plus_count: u32) -> u32 {
    2 + 2 * existing_t2_plus_count
}

/// Cost (in Supplies) to upgrade to Tier 3, given count of T3 Tunnels already owned.
/// Formula: 3 + 3 * existing_t3_count
pub fn tunnel_t3_upgrade_cost(existing_t3_count: u32) -> u32 {
    3 + 3 * existing_t3_count
}

/// Validate whether a tunnel upgrade is possible.
/// Returns Ok(cost) if valid, Err(reason) if not.
pub fn validate_tunnel_upgrade(
    tunnel_entity: Entity,
    tunnel_state: &TunnelState,
    tunnel_area: &TunnelArea,
    available_supplies: i32,
    all_tunnels: &[(Entity, &TunnelArea, &TunnelState)],
) -> Result<u32, &'static str> {
    // Check tier < 3
    let target_tier = tunnel_state.tier.next_tier()
        .ok_or("Tunnel is already at maximum tier")?;

    // Check no active operation
    if tunnel_state.is_busy() {
        return Err("Tunnel already has an operation in progress");
    }

    // Count existing tunnels at target tier or higher for cost calculation
    let cost = match target_tier {
        TunnelTier::Tier2 => {
            let t2_plus_count = all_tunnels.iter()
                .filter(|(e, _, s)| *e != tunnel_entity && matches!(s.tier, TunnelTier::Tier2 | TunnelTier::Tier3))
                .count() as u32;
            tunnel_t2_upgrade_cost(t2_plus_count)
        },
        TunnelTier::Tier3 => {
            let t3_count = all_tunnels.iter()
                .filter(|(e, _, s)| *e != tunnel_entity && s.tier == TunnelTier::Tier3)
                .count() as u32;
            tunnel_t3_upgrade_cost(t3_count)
        },
        TunnelTier::Tier1 => unreachable!("next_tier never returns Tier1"),
    };

    // Check sufficient supplies
    if (available_supplies as u32) < cost {
        return Err("Insufficient supplies for upgrade");
    }

    // Check non-overlap: compute hypothetical area for target tier
    let hypothetical_cells = compute_tunnel_area_cells(
        tunnel_area.origin_x,
        tunnel_area.origin_z,
        target_tier.area_radius(),
    );
    for (e, other_area, _) in all_tunnels {
        if *e != tunnel_entity {
            // Check if any hypothetical cell is in the other tunnel's area
            if hypothetical_cells.iter().any(|cell| other_area.cells.contains(cell)) {
                return Err("Upgrade would cause area overlap with another Tunnel");
            }
        }
    }

    Ok(cost)
}

#[cfg(test)]
mod tests {
    use super::*;
    use super::gdo_structure_stats::*;
    use super::syndicate_structure_stats::*;

    #[test]
    fn power_value_generator_is_positive() {
        let pv = PowerValue(DC_POWER);
        assert!(pv.0 > 0, "Deployment Center should generate power");
        let pv2 = PowerValue(PP_POWER);
        assert!(pv2.0 > 0, "Power Plant should generate power");
    }

    #[test]
    fn power_value_consumer_is_negative() {
        let pv = PowerValue(BK_POWER);
        assert!(pv.0 < 0, "Barracks should consume power");
        let pv2 = PowerValue(EF_POWER);
        assert!(pv2.0 < 0, "Extraction Facility should consume power");
    }

    #[test]
    fn power_constants_values() {
        assert_eq!(DC_POWER, 20);
        assert_eq!(PP_POWER, 20);
        assert_eq!(BK_POWER, -30);
        assert_eq!(EF_POWER, -15);
    }

    #[test]
    fn dc_generates_enough_for_startup() {
        // DC alone provides 20 power — enough to start building before power plants
        // A single barracks (-30) will cause deficit, incentivizing power plant first
        assert!(DC_POWER > 0);
        assert!(DC_POWER + BK_POWER < 0, "DC alone cannot power a Barracks");
    }

    #[test]
    fn power_plant_covers_barracks() {
        // DC (20) + PP (20) = 40 > BK cost (30) — player can sustain one barracks
        assert!(DC_POWER + PP_POWER + BK_POWER > 0,
            "DC + Power Plant should cover one Barracks");
    }

    #[test]
    fn extraction_plate_has_no_power_cost() {
        // Extraction Plates don't have a power constant — they don't consume power
        // Only the Extraction Facility does
        assert_eq!(EF_POWER, -15);
        // EP has no power field in gdo_structure_stats (verify by absence)
    }

    // --- ConstructionHP tests ---

    #[test]
    fn construction_hp_new_starts_at_zero_progress() {
        let chp = ConstructionHP::new(100);
        assert_eq!(chp.progress, 0.0);
        assert_eq!(chp.build_frames, 100);
    }

    #[test]
    fn construction_hp_is_not_complete_at_start() {
        let chp = ConstructionHP::new(100);
        assert!(!chp.is_complete());
    }

    #[test]
    fn construction_hp_is_complete_at_one() {
        let mut chp = ConstructionHP::new(100);
        chp.progress = 1.0;
        assert!(chp.is_complete());
    }

    #[test]
    fn construction_hp_is_complete_above_one() {
        let mut chp = ConstructionHP::new(100);
        chp.progress = 1.5;
        assert!(chp.is_complete());
    }

    #[test]
    fn hp_fraction_at_zero_progress_is_ten_percent() {
        let frac = ConstructionHP::hp_fraction(0.0);
        assert!((frac - 0.10).abs() < 0.001);
    }

    #[test]
    fn hp_fraction_at_full_progress_is_one() {
        let frac = ConstructionHP::hp_fraction(1.0);
        assert!((frac - 1.0).abs() < 0.001);
    }

    #[test]
    fn hp_fraction_at_half_progress_is_fifty_five_percent() {
        // 10% + 90% * 0.5 = 55%
        let frac = ConstructionHP::hp_fraction(0.5);
        assert!((frac - 0.55).abs() < 0.001);
    }

    #[test]
    fn hp_fraction_clamps_negative_progress() {
        let frac = ConstructionHP::hp_fraction(-0.5);
        assert!((frac - 0.10).abs() < 0.001);
    }

    #[test]
    fn hp_fraction_clamps_excess_progress() {
        let frac = ConstructionHP::hp_fraction(2.0);
        assert!((frac - 1.0).abs() < 0.001);
    }

    #[test]
    fn hp_fraction_linear_at_quarter_progress() {
        // 10% + 90% * 0.25 = 32.5%
        let frac = ConstructionHP::hp_fraction(0.25);
        assert!((frac - 0.325).abs() < 0.001);
    }

    #[test]
    fn hp_fraction_linear_at_three_quarter_progress() {
        // 10% + 90% * 0.75 = 77.5%
        let frac = ConstructionHP::hp_fraction(0.75);
        assert!((frac - 0.775).abs() < 0.001);
    }

    // === Syndicate Structure Stats Tests ===

    #[test]
    fn tunnel_hp_constants_increase_per_tier() {
        assert!(TUNNEL_T1_MAX_HP < TUNNEL_T2_MAX_HP);
        assert!(TUNNEL_T2_MAX_HP < TUNNEL_T3_MAX_HP);
    }

    #[test]
    fn tunnel_hp_constant_values() {
        assert_eq!(TUNNEL_T1_MAX_HP, 600.0);
        assert_eq!(TUNNEL_T2_MAX_HP, 800.0);
        assert_eq!(TUNNEL_T3_MAX_HP, 1000.0);
    }

    #[test]
    fn tunnel_armor_constants() {
        assert_eq!(TUNNEL_POINT_ARMOR, 1);
        assert_eq!(TUNNEL_FULL_ARMOR, 16);
    }

    #[test]
    fn tunnel_space_constants_increase_per_tier() {
        assert!(TUNNEL_T1_SPACE < TUNNEL_T2_SPACE);
        assert!(TUNNEL_T2_SPACE < TUNNEL_T3_SPACE);
    }

    #[test]
    fn tunnel_space_constant_values() {
        assert_eq!(TUNNEL_T1_SPACE, 20);
        assert_eq!(TUNNEL_T2_SPACE, 30);
        assert_eq!(TUNNEL_T3_SPACE, 40);
    }

    #[test]
    fn tunnel_area_radius_constants_increase_per_tier() {
        assert!(TUNNEL_T1_AREA_RADIUS < TUNNEL_T2_AREA_RADIUS);
        assert!(TUNNEL_T2_AREA_RADIUS < TUNNEL_T3_AREA_RADIUS);
    }

    #[test]
    fn tunnel_area_radius_constant_values() {
        assert_eq!(TUNNEL_T1_AREA_RADIUS, 3);
        assert_eq!(TUNNEL_T2_AREA_RADIUS, 4);
        assert_eq!(TUNNEL_T3_AREA_RADIUS, 5);
    }

    #[test]
    fn tunnel_sight_range_constant() {
        assert_eq!(TUNNEL_SIGHT_RANGE, 5);
    }

    // === TunnelTier Tests ===

    #[test]
    fn tunnel_tier_max_hp_matches_constants() {
        assert_eq!(TunnelTier::Tier1.max_hp(), TUNNEL_T1_MAX_HP);
        assert_eq!(TunnelTier::Tier2.max_hp(), TUNNEL_T2_MAX_HP);
        assert_eq!(TunnelTier::Tier3.max_hp(), TUNNEL_T3_MAX_HP);
    }

    #[test]
    fn tunnel_tier_space_matches_constants() {
        assert_eq!(TunnelTier::Tier1.tunnel_space(), TUNNEL_T1_SPACE);
        assert_eq!(TunnelTier::Tier2.tunnel_space(), TUNNEL_T2_SPACE);
        assert_eq!(TunnelTier::Tier3.tunnel_space(), TUNNEL_T3_SPACE);
    }

    #[test]
    fn tunnel_tier_area_radius_matches_constants() {
        assert_eq!(TunnelTier::Tier1.area_radius(), TUNNEL_T1_AREA_RADIUS);
        assert_eq!(TunnelTier::Tier2.area_radius(), TUNNEL_T2_AREA_RADIUS);
        assert_eq!(TunnelTier::Tier3.area_radius(), TUNNEL_T3_AREA_RADIUS);
    }

    #[test]
    fn tunnel_tier_transit_tier_mapping() {
        assert_eq!(TunnelTier::Tier1.transit_tier(), TransitTier::Tier1);
        assert_eq!(TunnelTier::Tier2.transit_tier(), TransitTier::Tier2);
        assert_eq!(TunnelTier::Tier3.transit_tier(), TransitTier::Tier3);
    }

    #[test]
    fn tunnel_tier_next_tier() {
        assert_eq!(TunnelTier::Tier1.next_tier(), Some(TunnelTier::Tier2));
        assert_eq!(TunnelTier::Tier2.next_tier(), Some(TunnelTier::Tier3));
        assert_eq!(TunnelTier::Tier3.next_tier(), None);
    }

    // === TransitTier Tests ===

    #[test]
    fn transit_tier1_allows_infantry() {
        assert!(TransitTier::Tier1.allows_unit_base(&UnitBaseEnum::LightInfantry));
        assert!(TransitTier::Tier1.allows_unit_base(&UnitBaseEnum::HeavyInfantry));
    }

    #[test]
    fn transit_tier1_blocks_vehicles() {
        assert!(!TransitTier::Tier1.allows_unit_base(&UnitBaseEnum::WheeledVehicle));
        assert!(!TransitTier::Tier1.allows_unit_base(&UnitBaseEnum::TrackedVehicle));
        assert!(!TransitTier::Tier1.allows_unit_base(&UnitBaseEnum::DrillUnit));
        assert!(!TransitTier::Tier1.allows_unit_base(&UnitBaseEnum::HoverVehicle));
        assert!(!TransitTier::Tier1.allows_unit_base(&UnitBaseEnum::Mech));
    }

    #[test]
    fn transit_tier1_blocks_air() {
        assert!(!TransitTier::Tier1.allows_unit_base(&UnitBaseEnum::HoverCraft));
        assert!(!TransitTier::Tier1.allows_unit_base(&UnitBaseEnum::Glider));
    }

    #[test]
    fn transit_tier2_allows_infantry_and_vehicles() {
        assert!(TransitTier::Tier2.allows_unit_base(&UnitBaseEnum::LightInfantry));
        assert!(TransitTier::Tier2.allows_unit_base(&UnitBaseEnum::HeavyInfantry));
        assert!(TransitTier::Tier2.allows_unit_base(&UnitBaseEnum::WheeledVehicle));
        assert!(TransitTier::Tier2.allows_unit_base(&UnitBaseEnum::TrackedVehicle));
        assert!(TransitTier::Tier2.allows_unit_base(&UnitBaseEnum::DrillUnit));
        assert!(TransitTier::Tier2.allows_unit_base(&UnitBaseEnum::HoverVehicle));
        assert!(TransitTier::Tier2.allows_unit_base(&UnitBaseEnum::Mech));
    }

    #[test]
    fn transit_tier2_blocks_air() {
        assert!(!TransitTier::Tier2.allows_unit_base(&UnitBaseEnum::HoverCraft));
        assert!(!TransitTier::Tier2.allows_unit_base(&UnitBaseEnum::Glider));
    }

    #[test]
    fn transit_tier3_allows_all() {
        assert!(TransitTier::Tier3.allows_unit_base(&UnitBaseEnum::LightInfantry));
        assert!(TransitTier::Tier3.allows_unit_base(&UnitBaseEnum::HeavyInfantry));
        assert!(TransitTier::Tier3.allows_unit_base(&UnitBaseEnum::WheeledVehicle));
        assert!(TransitTier::Tier3.allows_unit_base(&UnitBaseEnum::TrackedVehicle));
        assert!(TransitTier::Tier3.allows_unit_base(&UnitBaseEnum::DrillUnit));
        assert!(TransitTier::Tier3.allows_unit_base(&UnitBaseEnum::HoverVehicle));
        assert!(TransitTier::Tier3.allows_unit_base(&UnitBaseEnum::Mech));
        assert!(TransitTier::Tier3.allows_unit_base(&UnitBaseEnum::HoverCraft));
        assert!(TransitTier::Tier3.allows_unit_base(&UnitBaseEnum::Glider));
    }

    // === TunnelTier::can_transit() Tests ===

    #[test]
    fn tunnel_tier1_can_transit_infantry() {
        assert!(TunnelTier::Tier1.can_transit(&UnitBaseEnum::LightInfantry));
        assert!(TunnelTier::Tier1.can_transit(&UnitBaseEnum::HeavyInfantry));
    }

    #[test]
    fn tunnel_tier1_cannot_transit_vehicles() {
        assert!(!TunnelTier::Tier1.can_transit(&UnitBaseEnum::WheeledVehicle));
        assert!(!TunnelTier::Tier1.can_transit(&UnitBaseEnum::Mech));
    }

    #[test]
    fn tunnel_tier2_can_transit_vehicles() {
        assert!(TunnelTier::Tier2.can_transit(&UnitBaseEnum::WheeledVehicle));
        assert!(TunnelTier::Tier2.can_transit(&UnitBaseEnum::TrackedVehicle));
        assert!(TunnelTier::Tier2.can_transit(&UnitBaseEnum::Mech));
    }

    #[test]
    fn tunnel_tier2_cannot_transit_air() {
        assert!(!TunnelTier::Tier2.can_transit(&UnitBaseEnum::Glider));
        assert!(!TunnelTier::Tier2.can_transit(&UnitBaseEnum::HoverCraft));
    }

    #[test]
    fn tunnel_tier3_can_transit_air() {
        assert!(TunnelTier::Tier3.can_transit(&UnitBaseEnum::Glider));
        assert!(TunnelTier::Tier3.can_transit(&UnitBaseEnum::HoverCraft));
    }

    // === TunnelState Tests ===

    #[test]
    fn tunnel_state_default_tier1() {
        let state = TunnelState::default_tier1();
        assert_eq!(state.tier, TunnelTier::Tier1);
        assert!(state.current_operation.is_none());
        assert!(!state.is_busy());
    }

    #[test]
    fn tunnel_state_new_with_tier() {
        let state = TunnelState::new(TunnelTier::Tier2);
        assert_eq!(state.tier, TunnelTier::Tier2);
        assert!(!state.is_busy());
    }

    #[test]
    fn tunnel_state_busy_when_upgrading() {
        let mut state = TunnelState::default_tier1();
        state.current_operation = Some(TunnelOperation::Upgrading {
            target_tier: TunnelTier::Tier2,
            progress: 0.5,
        });
        assert!(state.is_busy());
    }

    #[test]
    fn tunnel_state_busy_when_building_expansion() {
        let mut state = TunnelState::default_tier1();
        state.current_operation = Some(TunnelOperation::BuildingExpansion {
            object: ObjectEnum::Tunnel,
            progress: 0.0,
            grid_x: 0,
            grid_z: 0,
            rotation: crate::types::StructureRotation::R0,
            flip_horizontal: false,
            flip_vertical: false,
        });
        assert!(state.is_busy());
    }

    #[test]
    fn tunnel_operation_upgrading_equality() {
        let op1 = TunnelOperation::Upgrading { target_tier: TunnelTier::Tier2, progress: 0.5 };
        let op2 = TunnelOperation::Upgrading { target_tier: TunnelTier::Tier2, progress: 0.5 };
        assert_eq!(op1, op2);
    }

    // === Power Plant Stat Tests ===

    #[test]
    fn pp_max_hp_value() {
        assert_eq!(PP_MAX_HP, 350.0);
    }

    #[test]
    fn pp_armor_values() {
        assert_eq!(PP_POINT_ARMOR, 1);
        assert_eq!(PP_FULL_ARMOR, 4);
    }

    #[test]
    fn pp_build_radius_value() {
        assert_eq!(PP_BUILD_RADIUS, 1);
    }

    #[test]
    fn pp_power_is_positive() {
        assert_eq!(PP_POWER, 20);
        assert!(PP_POWER > 0, "Power Plant should generate power");
    }

    // === Barracks Stat Tests ===

    #[test]
    fn bk_max_hp_value() {
        assert_eq!(BK_MAX_HP, 300.0);
    }

    #[test]
    fn bk_armor_values() {
        assert_eq!(BK_POINT_ARMOR, 1);
        assert_eq!(BK_FULL_ARMOR, 6);
    }

    #[test]
    fn bk_build_radius_value() {
        assert_eq!(BK_BUILD_RADIUS, 2);
    }

    #[test]
    fn bk_power_is_negative() {
        assert_eq!(BK_POWER, -30);
        assert!(BK_POWER < 0, "Barracks should consume power");
    }

    // === DC Construction Cost Tests ===

    #[test]
    fn dc_construction_cost_power_plant() {
        let cost = DeploymentCenterState::construction_cost(&ObjectEnum::PowerPlant).unwrap();
        assert_eq!(cost.space_crystals, 150);
        assert_eq!(cost.build_frames, 160);
    }

    #[test]
    fn dc_construction_cost_barracks() {
        let cost = DeploymentCenterState::construction_cost(&ObjectEnum::Barracks).unwrap();
        assert_eq!(cost.space_crystals, 200);
        assert_eq!(cost.build_frames, 160);
    }

    #[test]
    fn dc_construction_cost_invalid_returns_none() {
        assert!(DeploymentCenterState::construction_cost(&ObjectEnum::Peacekeeper).is_none());
        assert!(DeploymentCenterState::construction_cost(&ObjectEnum::SpaceCrystalsPatch).is_none());
        assert!(DeploymentCenterState::construction_cost(&ObjectEnum::ExtractionFacility).is_none());
    }

    // === DC Cancellation Refund Tests ===

    #[test]
    fn dc_full_refund_during_construction() {
        let dc = DeploymentCenterState {
            current_construction: Some(ObjectEnum::PowerPlant),
            construction_progress: Some(50.0),
            ready_to_place: None,
        };
        let refund = dc.cancellation_refund(&ObjectEnum::PowerPlant).unwrap();
        assert_eq!(refund, 150); // Full refund
    }

    #[test]
    fn dc_partial_refund_when_ready_to_place() {
        let dc = DeploymentCenterState {
            current_construction: None,
            construction_progress: None,
            ready_to_place: Some(ObjectEnum::PowerPlant),
        };
        let refund = dc.cancellation_refund(&ObjectEnum::PowerPlant).unwrap();
        assert_eq!(refund, 112); // 75% of 150 = 112
    }

    // === Barracks Production Cost Tests ===

    #[test]
    fn bk_production_cost_peacekeeper() {
        let cost = BarracksState::production_cost(&ObjectEnum::Peacekeeper).unwrap();
        assert_eq!(cost.space_crystals, 50);
        assert_eq!(cost.build_frames, 80);
    }

    #[test]
    fn bk_production_cost_invalid_returns_none() {
        assert!(BarracksState::production_cost(&ObjectEnum::Barracks).is_none());
        assert!(BarracksState::production_cost(&ObjectEnum::PowerPlant).is_none());
    }

    // === Barracks Queue Tests ===

    #[test]
    fn bk_try_queue_adds_to_queue() {
        let mut bk = BarracksState::default();
        assert!(bk.try_queue(ObjectEnum::Peacekeeper));
        assert_eq!(bk.build_queue.len(), 1);
        assert_eq!(bk.build_queue[0], ObjectEnum::Peacekeeper);
    }

    #[test]
    fn bk_try_queue_respects_max_size() {
        let mut bk = BarracksState::default();
        for _ in 0..BarracksState::MAX_QUEUE_SIZE {
            assert!(bk.try_queue(ObjectEnum::Peacekeeper));
        }
        assert!(!bk.try_queue(ObjectEnum::Peacekeeper));
        assert_eq!(bk.build_queue.len(), BarracksState::MAX_QUEUE_SIZE);
    }

    #[test]
    fn bk_max_queue_size_is_five() {
        assert_eq!(BarracksState::MAX_QUEUE_SIZE, 5);
    }

    #[test]
    fn bk_cancel_last_returns_lifo() {
        let mut bk = BarracksState::default();
        bk.try_queue(ObjectEnum::Peacekeeper);
        bk.try_queue(ObjectEnum::Peacekeeper);
        let cancelled = bk.cancel_last().unwrap();
        assert_eq!(cancelled, ObjectEnum::Peacekeeper);
        assert_eq!(bk.build_queue.len(), 1);
    }

    #[test]
    fn bk_cancel_last_empty_returns_none() {
        let mut bk = BarracksState::default();
        assert!(bk.cancel_last().is_none());
    }

    #[test]
    fn bk_cancel_last_cancels_active_build_when_queue_empty() {
        let mut bk = BarracksState::default();
        bk.current_build = Some(ObjectEnum::Peacekeeper);
        bk.current_build_progress = Some(0.5);
        let cancelled = bk.cancel_last().unwrap();
        assert_eq!(cancelled, ObjectEnum::Peacekeeper);
        assert!(bk.current_build.is_none(), "current_build should be cleared");
        assert!(bk.current_build_progress.is_none(), "progress should be cleared");
    }

    #[test]
    fn bk_cancel_last_prefers_queue_over_active_build() {
        let mut bk = BarracksState::default();
        bk.current_build = Some(ObjectEnum::Peacekeeper);
        bk.current_build_progress = Some(0.5);
        bk.try_queue(ObjectEnum::Peacekeeper);
        let cancelled = bk.cancel_last().unwrap();
        assert_eq!(cancelled, ObjectEnum::Peacekeeper);
        assert!(bk.current_build.is_some(), "active build should NOT be cancelled when queue has items");
        assert!(bk.build_queue.is_empty(), "queue item should be removed");
    }

    #[test]
    fn bk_has_cancellable_with_queue() {
        let mut bk = BarracksState::default();
        assert!(!bk.has_cancellable());
        bk.try_queue(ObjectEnum::Peacekeeper);
        assert!(bk.has_cancellable());
    }

    #[test]
    fn bk_has_cancellable_with_active_build() {
        let mut bk = BarracksState::default();
        bk.current_build = Some(ObjectEnum::Peacekeeper);
        assert!(bk.has_cancellable());
    }

    // === Rally Target Tests ===

    #[test]
    fn rally_target_location_construction() {
        let pos = Vec3::new(10.0, 0.0, 20.0);
        let rally = RallyTarget::Location(pos);
        match rally {
            RallyTarget::Location(p) => assert_eq!(p, pos),
            _ => panic!("Expected Location variant"),
        }
    }

    #[test]
    fn rally_target_object_construction() {
        // Use a fake entity to test the enum variant
        let entity = Entity::from_raw_u32(42).unwrap();
        let rally = RallyTarget::Object(entity);
        match rally {
            RallyTarget::Object(e) => assert_eq!(e, entity),
            _ => panic!("Expected Object variant"),
        }
    }

    #[test]
    fn barracks_state_default_has_no_rally_point() {
        let bk = BarracksState::default();
        assert!(bk.rally_point.is_none());
    }

    #[test]
    fn barracks_state_default_has_empty_queue() {
        let bk = BarracksState::default();
        assert!(bk.build_queue.is_empty());
        assert!(bk.current_build.is_none());
        assert!(bk.current_build_progress.is_none());
    }

    // === TunnelArea Tests ===

    #[test]
    fn tunnel_area_new_tier1_cell_count() {
        // T1 radius=3, side = 2*3+4 = 10, total cells = 10*10 = 100
        let area = TunnelArea::new(10, 10, &TunnelTier::Tier1);
        assert_eq!(area.cells.len(), 100);
    }

    #[test]
    fn tunnel_area_new_tier2_cell_count() {
        // T2 radius=4, side = 2*4+4 = 12, total cells = 12*12 = 144
        let area = TunnelArea::new(10, 10, &TunnelTier::Tier2);
        assert_eq!(area.cells.len(), 144);
    }

    #[test]
    fn tunnel_area_new_tier3_cell_count() {
        // T3 radius=5, side = 2*5+4 = 14, total cells = 14*14 = 196
        let area = TunnelArea::new(10, 10, &TunnelTier::Tier3);
        assert_eq!(area.cells.len(), 196);
    }

    #[test]
    fn tunnel_area_contains_origin() {
        let area = TunnelArea::new(10, 10, &TunnelTier::Tier1);
        assert!(area.contains(10, 10));
    }

    #[test]
    fn tunnel_area_contains_footprint_cells() {
        let area = TunnelArea::new(10, 10, &TunnelTier::Tier1);
        // The 4x4 footprint (10..14, 10..14) should all be within the area
        for x in 10..14 {
            for z in 10..14 {
                assert!(area.contains(x, z), "Footprint cell ({},{}) should be in area", x, z);
            }
        }
    }

    #[test]
    fn tunnel_area_contains_boundary_cells_tier1() {
        let area = TunnelArea::new(10, 10, &TunnelTier::Tier1);
        // T1 radius=3: area goes from (10-3)..(10+4+3) = 7..17 on both axes
        assert!(area.contains(7, 7));    // top-left corner of area
        assert!(area.contains(16, 16));  // bottom-right corner (17 exclusive → 16 max)
        assert!(!area.contains(6, 7));   // outside left
        assert!(!area.contains(7, 17));  // outside bottom
    }

    #[test]
    fn tunnel_area_recalculate_expands_on_upgrade() {
        let mut area = TunnelArea::new(10, 10, &TunnelTier::Tier1);
        assert_eq!(area.cells.len(), 100);
        area.recalculate(&TunnelTier::Tier2);
        assert_eq!(area.cells.len(), 144);
    }

    #[test]
    fn tunnel_area_overlaps_detects_overlap() {
        let area1 = TunnelArea::new(10, 10, &TunnelTier::Tier1);
        // Place second tunnel close enough to overlap
        let area2 = TunnelArea::new(15, 10, &TunnelTier::Tier1);
        assert!(area1.overlaps(&area2));
    }

    #[test]
    fn tunnel_area_overlaps_no_overlap_when_far_apart() {
        let area1 = TunnelArea::new(0, 0, &TunnelTier::Tier1);
        // T1 goes (0-3)..(0+4+3)= -3..7. Second tunnel at 20: (20-3)..(20+4+3)= 17..27
        let area2 = TunnelArea::new(20, 20, &TunnelTier::Tier1);
        assert!(!area1.overlaps(&area2));
    }

    #[test]
    fn tunnel_area_fits_expansion_inside() {
        let area = TunnelArea::new(10, 10, &TunnelTier::Tier1);
        // 2x2 expansion inside the 10x10 area
        assert!(area.fits_expansion(10, 10, 2, 2));
    }

    #[test]
    fn tunnel_area_fits_expansion_partially_outside() {
        let area = TunnelArea::new(10, 10, &TunnelTier::Tier1);
        // T1 area: 7..17 on both axes. 2x2 at (16, 16) → needs (16,16),(17,16),(16,17),(17,17)
        // 17 is outside (range is 7..17 exclusive, so max = 16)
        assert!(!area.fits_expansion(16, 16, 2, 2));
    }

    #[test]
    fn tunnel_area_fits_expansion_completely_outside() {
        let area = TunnelArea::new(10, 10, &TunnelTier::Tier1);
        assert!(!area.fits_expansion(50, 50, 1, 1));
    }

    // === Tunnel Construction Constants ===

    #[test]
    fn tunnel_construction_frames_value() {
        assert_eq!(TUNNEL_CONSTRUCTION_FRAMES, 480);
    }

    #[test]
    fn tunnel_upgrade_frames_value() {
        assert_eq!(TUNNEL_UPGRADE_FRAMES, 480);
    }

    // === Tunnel Cost Function Tests ===

    #[test]
    fn tunnel_construction_cost_first_is_free() {
        assert_eq!(tunnel_construction_cost(0), 0);
    }

    #[test]
    fn tunnel_construction_cost_second_is_one() {
        assert_eq!(tunnel_construction_cost(1), 1);
    }

    #[test]
    fn tunnel_construction_cost_third_is_two() {
        assert_eq!(tunnel_construction_cost(2), 2);
    }

    #[test]
    fn tunnel_t2_upgrade_cost_first() {
        // 2 + 2*0 = 2
        assert_eq!(tunnel_t2_upgrade_cost(0), 2);
    }

    #[test]
    fn tunnel_t2_upgrade_cost_second() {
        // 2 + 2*1 = 4
        assert_eq!(tunnel_t2_upgrade_cost(1), 4);
    }

    #[test]
    fn tunnel_t2_upgrade_cost_third() {
        // 2 + 2*2 = 6
        assert_eq!(tunnel_t2_upgrade_cost(2), 6);
    }

    #[test]
    fn tunnel_t3_upgrade_cost_first() {
        // 3 + 3*0 = 3
        assert_eq!(tunnel_t3_upgrade_cost(0), 3);
    }

    #[test]
    fn tunnel_t3_upgrade_cost_second() {
        // 3 + 3*1 = 6
        assert_eq!(tunnel_t3_upgrade_cost(1), 6);
    }

    #[test]
    fn tunnel_t3_upgrade_cost_third() {
        // 3 + 3*2 = 9
        assert_eq!(tunnel_t3_upgrade_cost(2), 9);
    }

    // === compute_tunnel_area_cells Tests ===

    #[test]
    fn compute_cells_radius_zero() {
        // radius=0: area is exactly the 4x4 footprint = 16 cells
        let cells = compute_tunnel_area_cells(10, 10, 0);
        assert_eq!(cells.len(), 16);
        assert!(cells.contains(&(10, 10)));
        assert!(cells.contains(&(13, 13)));
        assert!(!cells.contains(&(9, 10)));
        assert!(!cells.contains(&(14, 10)));
    }

    #[test]
    fn compute_cells_radius_3_matches_tier1() {
        let cells = compute_tunnel_area_cells(10, 10, 3);
        assert_eq!(cells.len(), 100); // (2*3+4)^2 = 10^2
    }

    // === validate_tunnel_upgrade Tests ===

    #[test]
    fn validate_upgrade_tier1_to_tier2_succeeds() {
        let e1 = Entity::from_raw_u32(1).unwrap();
        let state = TunnelState::default_tier1();
        let area = TunnelArea::new(10, 10, &TunnelTier::Tier1);
        let result = validate_tunnel_upgrade(e1, &state, &area, 100, &[]);
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), 2); // 2 + 2*0
    }

    #[test]
    fn validate_upgrade_tier2_to_tier3_succeeds() {
        let e1 = Entity::from_raw_u32(1).unwrap();
        let state = TunnelState::new(TunnelTier::Tier2);
        let area = TunnelArea::new(10, 10, &TunnelTier::Tier2);
        let result = validate_tunnel_upgrade(e1, &state, &area, 100, &[]);
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), 3); // 3 + 3*0
    }

    #[test]
    fn validate_upgrade_tier3_already_max() {
        let e1 = Entity::from_raw_u32(1).unwrap();
        let state = TunnelState::new(TunnelTier::Tier3);
        let area = TunnelArea::new(10, 10, &TunnelTier::Tier3);
        let result = validate_tunnel_upgrade(e1, &state, &area, 100, &[]);
        assert!(result.is_err());
        assert_eq!(result.unwrap_err(), "Tunnel is already at maximum tier");
    }

    #[test]
    fn validate_upgrade_busy_tunnel_blocked() {
        let e1 = Entity::from_raw_u32(1).unwrap();
        let mut state = TunnelState::default_tier1();
        state.current_operation = Some(TunnelOperation::BuildingExpansion {
            object: ObjectEnum::Tunnel,
            progress: 0.0,
            grid_x: 0,
            grid_z: 0,
            rotation: crate::types::StructureRotation::R0,
            flip_horizontal: false,
            flip_vertical: false,
        });
        let area = TunnelArea::new(10, 10, &TunnelTier::Tier1);
        let result = validate_tunnel_upgrade(e1, &state, &area, 100, &[]);
        assert!(result.is_err());
        assert_eq!(result.unwrap_err(), "Tunnel already has an operation in progress");
    }

    #[test]
    fn validate_upgrade_insufficient_supplies() {
        let e1 = Entity::from_raw_u32(1).unwrap();
        let state = TunnelState::default_tier1();
        let area = TunnelArea::new(10, 10, &TunnelTier::Tier1);
        let result = validate_tunnel_upgrade(e1, &state, &area, 1, &[]); // Need 2, have 1
        assert!(result.is_err());
        assert_eq!(result.unwrap_err(), "Insufficient supplies for upgrade");
    }

    #[test]
    fn validate_upgrade_would_overlap_blocked() {
        let e1 = Entity::from_raw_u32(1).unwrap();
        let e2 = Entity::from_raw_u32(2).unwrap();
        let state1 = TunnelState::default_tier1();
        let area1 = TunnelArea::new(10, 10, &TunnelTier::Tier1);
        // Place second tunnel close enough that T2 area would overlap
        let state2 = TunnelState::default_tier1();
        let area2 = TunnelArea::new(18, 10, &TunnelTier::Tier1);
        // T1 areas: e1=(7..17), e2=(15..25) — already overlapping at T1
        // Actually let's be more careful:
        // e1 T1: (10-3)..(10+4+3) = 7..17
        // e2 at (18,10) T1: (18-3)..(18+4+3) = 15..25 — overlap at 15,16
        // e1 T2 would be: (10-4)..(10+4+4) = 6..18, e2 stays T1 at 15..25 — overlap at 15..18
        let all = vec![(e1, &area1, &state1), (e2, &area2, &state2)];
        let result = validate_tunnel_upgrade(e1, &state1, &area1, 100, &all);
        // T2 area for e1: (10-4)..(10+4+4) = 6..18 on both axes.
        // e2 T1 area: (15..25). Overlap at 15,16,17 → blocked.
        assert!(result.is_err());
        assert_eq!(result.unwrap_err(), "Upgrade would cause area overlap with another Tunnel");
    }

    #[test]
    fn validate_upgrade_no_overlap_when_far_apart() {
        let e1 = Entity::from_raw_u32(1).unwrap();
        let e2 = Entity::from_raw_u32(2).unwrap();
        let state1 = TunnelState::default_tier1();
        let area1 = TunnelArea::new(0, 0, &TunnelTier::Tier1);
        let state2 = TunnelState::default_tier1();
        let area2 = TunnelArea::new(30, 30, &TunnelTier::Tier1);
        let all = vec![(e1, &area1, &state1), (e2, &area2, &state2)];
        let result = validate_tunnel_upgrade(e1, &state1, &area1, 100, &all);
        assert!(result.is_ok());
    }

    #[test]
    fn validate_upgrade_t2_cost_with_existing_t2() {
        let e1 = Entity::from_raw_u32(1).unwrap();
        let e2 = Entity::from_raw_u32(2).unwrap();
        let state1 = TunnelState::default_tier1();
        let area1 = TunnelArea::new(0, 0, &TunnelTier::Tier1);
        let state2 = TunnelState::new(TunnelTier::Tier2);
        let area2 = TunnelArea::new(50, 50, &TunnelTier::Tier2);
        let all = vec![(e1, &area1, &state1), (e2, &area2, &state2)];
        let result = validate_tunnel_upgrade(e1, &state1, &area1, 100, &all);
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), 4); // 2 + 2*1 (one existing T2+)
    }

    #[test]
    fn validate_upgrade_t3_cost_with_existing_t3() {
        let e1 = Entity::from_raw_u32(1).unwrap();
        let e2 = Entity::from_raw_u32(2).unwrap();
        let state1 = TunnelState::new(TunnelTier::Tier2);
        let area1 = TunnelArea::new(0, 0, &TunnelTier::Tier2);
        let state2 = TunnelState::new(TunnelTier::Tier3);
        let area2 = TunnelArea::new(50, 50, &TunnelTier::Tier3);
        let all = vec![(e1, &area1, &state1), (e2, &area2, &state2)];
        let result = validate_tunnel_upgrade(e1, &state1, &area1, 100, &all);
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), 6); // 3 + 3*1 (one existing T3)
    }

    // === TunnelExpansionMarker Tests ===

    #[test]
    fn tunnel_expansion_marker_stores_parent() {
        let parent = Entity::from_raw_u32(42).unwrap();
        let marker = TunnelExpansionMarker { parent_tunnel: parent };
        assert_eq!(marker.parent_tunnel, parent);
    }

    #[test]
    fn tunnel_expansion_marker_clone() {
        let parent = Entity::from_raw_u32(7).unwrap();
        let marker = TunnelExpansionMarker { parent_tunnel: parent };
        let cloned = marker.clone();
        assert_eq!(cloned.parent_tunnel, parent);
    }

    // === HeadquartersState Tests ===

    #[test]
    fn headquarters_state_default() {
        let state = HeadquartersState::default();
        assert!(state.rally_point.is_none());
        assert!(state.build_queue.is_empty());
        assert!(state.current_build.is_none());
        assert!(state.current_build_progress.is_none());
    }

    #[test]
    fn headquarters_max_queue_size_is_five() {
        assert_eq!(HeadquartersState::MAX_QUEUE_SIZE, 5);
    }

    #[test]
    fn headquarters_production_cost_agent() {
        let cost = HeadquartersState::production_cost(&ObjectEnum::SyndicateAgent);
        assert!(cost.is_some());
        let cost = cost.unwrap();
        assert_eq!(cost.space_crystals, 100);
        assert_eq!(cost.build_frames, 160);
    }

    #[test]
    fn headquarters_production_cost_unknown_returns_none() {
        assert!(HeadquartersState::production_cost(&ObjectEnum::Peacekeeper).is_none());
        assert!(HeadquartersState::production_cost(&ObjectEnum::Tunnel).is_none());
    }

    #[test]
    fn headquarters_try_queue_succeeds() {
        let mut state = HeadquartersState::default();
        assert!(state.try_queue(ObjectEnum::SyndicateAgent));
        assert_eq!(state.build_queue.len(), 1);
    }

    #[test]
    fn headquarters_try_queue_up_to_max() {
        let mut state = HeadquartersState::default();
        for _ in 0..5 {
            assert!(state.try_queue(ObjectEnum::SyndicateAgent));
        }
        assert_eq!(state.build_queue.len(), 5);
    }

    #[test]
    fn headquarters_try_queue_fails_when_full() {
        let mut state = HeadquartersState::default();
        for _ in 0..5 {
            state.try_queue(ObjectEnum::SyndicateAgent);
        }
        assert!(!state.try_queue(ObjectEnum::SyndicateAgent));
        assert_eq!(state.build_queue.len(), 5);
    }

    #[test]
    fn headquarters_cancel_last_returns_item() {
        let mut state = HeadquartersState::default();
        state.try_queue(ObjectEnum::SyndicateAgent);
        let cancelled = state.cancel_last();
        assert_eq!(cancelled, Some(ObjectEnum::SyndicateAgent));
        assert!(state.build_queue.is_empty());
    }

    #[test]
    fn headquarters_cancel_last_empty_returns_none() {
        let mut state = HeadquartersState::default();
        assert!(state.cancel_last().is_none());
    }

    #[test]
    fn headquarters_cancel_last_removes_most_recent() {
        let mut state = HeadquartersState::default();
        state.try_queue(ObjectEnum::SyndicateAgent);
        state.try_queue(ObjectEnum::SyndicateAgent);
        state.cancel_last();
        assert_eq!(state.build_queue.len(), 1);
    }

    #[test]
    fn headquarters_production_cost_guard() {
        let cost = HeadquartersState::production_cost(&ObjectEnum::SyndicateGuard);
        assert!(cost.is_some());
        let cost = cost.unwrap();
        assert_eq!(cost.space_crystals, 125);
        assert_eq!(cost.build_frames, 120);
    }

    #[test]
    fn headquarters_guard_can_be_queued() {
        let mut state = HeadquartersState::default();
        assert!(state.try_queue(ObjectEnum::SyndicateGuard));
        assert_eq!(state.build_queue.len(), 1);
        assert_eq!(state.build_queue[0], ObjectEnum::SyndicateGuard);
    }

    #[test]
    fn headquarters_mixed_queue_agent_and_guard() {
        let mut state = HeadquartersState::default();
        assert!(state.try_queue(ObjectEnum::SyndicateAgent));
        assert!(state.try_queue(ObjectEnum::SyndicateGuard));
        assert!(state.try_queue(ObjectEnum::SyndicateAgent));
        assert_eq!(state.build_queue.len(), 3);
        assert_eq!(state.build_queue[0], ObjectEnum::SyndicateAgent);
        assert_eq!(state.build_queue[1], ObjectEnum::SyndicateGuard);
        assert_eq!(state.build_queue[2], ObjectEnum::SyndicateAgent);
    }

    #[test]
    fn headquarters_cancel_last_guard_from_mixed_queue() {
        let mut state = HeadquartersState::default();
        state.try_queue(ObjectEnum::SyndicateAgent);
        state.try_queue(ObjectEnum::SyndicateGuard);
        let cancelled = state.cancel_last();
        assert_eq!(cancelled, Some(ObjectEnum::SyndicateGuard));
        assert_eq!(state.build_queue.len(), 1);
        assert_eq!(state.build_queue[0], ObjectEnum::SyndicateAgent);
    }

    #[test]
    fn headquarters_cancel_last_cancels_active_build_when_queue_empty() {
        let mut state = HeadquartersState::default();
        state.current_build = Some(ObjectEnum::SyndicateAgent);
        state.current_build_progress = Some(0.3);
        let cancelled = state.cancel_last().unwrap();
        assert_eq!(cancelled, ObjectEnum::SyndicateAgent);
        assert!(state.current_build.is_none());
        assert!(state.current_build_progress.is_none());
    }

    #[test]
    fn headquarters_has_cancellable() {
        let mut state = HeadquartersState::default();
        assert!(!state.has_cancellable());
        state.current_build = Some(ObjectEnum::SyndicateGuard);
        assert!(state.has_cancellable());
        state.current_build = None;
        state.try_queue(ObjectEnum::SyndicateAgent);
        assert!(state.has_cancellable());
    }

    // === Headquarters Stat Constants Tests ===

    #[test]
    fn hq_max_hp_value() {
        assert_eq!(HQ_MAX_HP, 400.0);
    }

    #[test]
    fn hq_armor_values() {
        assert_eq!(HQ_POINT_ARMOR, 1);
        assert_eq!(HQ_FULL_ARMOR, 4);
    }

    #[test]
    fn hq_cost_and_build_time() {
        assert_eq!(HQ_SC_COST, 200);
        assert_eq!(HQ_BUILD_FRAMES, 400);
    }

    // === Headquarters ObjectEnum Integration Tests ===

    #[test]
    fn headquarters_is_structure() {
        assert!(ObjectEnum::Headquarters.is_structure());
    }

    #[test]
    fn headquarters_is_not_unit() {
        assert!(!ObjectEnum::Headquarters.is_unit());
    }

    #[test]
    fn headquarters_is_not_resource() {
        assert!(!ObjectEnum::Headquarters.is_resource());
    }

    #[test]
    fn headquarters_object_type_name() {
        let ot = ObjectEnum::Headquarters.object_type();
        assert_eq!(ot.name, "Headquarters");
    }

    #[test]
    fn headquarters_object_type_size() {
        let ot = ObjectEnum::Headquarters.object_type();
        assert_eq!(ot.size, (2, 2));
    }

    #[test]
    fn headquarters_object_type_destructible() {
        let ot = ObjectEnum::Headquarters.object_type();
        assert!(ot.destructible);
    }

    #[test]
    fn headquarters_object_type_no_sight() {
        let ot = ObjectEnum::Headquarters.object_type();
        assert_eq!(ot.sight_range, 0);
    }

    #[test]
    fn headquarters_object_type_not_groupable() {
        let ot = ObjectEnum::Headquarters.object_type();
        assert!(!ot.groupable);
    }

    #[test]
    fn headquarters_structure_type_symmetry() {
        use crate::types::SymmetryTypeEnum;
        let st = ObjectEnum::Headquarters.structure_type().unwrap();
        assert_eq!(st.symmetry_type, SymmetryTypeEnum::AAAA);
    }

    #[test]
    fn headquarters_structure_passes_validation() {
        let st = ObjectEnum::Headquarters.structure_type().unwrap();
        assert!(st.validate_size().is_ok());
    }

    #[test]
    fn headquarters_destructible_instance() {
        use crate::game::types::objects::ObjectInstance;
        let obj = ObjectInstance::destructible(ObjectEnum::Headquarters, HQ_MAX_HP);
        assert_eq!(obj.hp, Some(400.0));
        assert_eq!(obj.max_hp, Some(400.0));
        assert!(obj.is_alive());
    }

    // === Supply Tower tests ===

    #[test]
    fn supply_tower_constants() {
        assert_eq!(ST_MAX_HP, 400.0);
        assert_eq!(ST_POINT_ARMOR, 1);
        assert_eq!(ST_FULL_ARMOR, 9);
        assert_eq!(ST_BUILD_RADIUS, 1);
        assert_eq!(ST_POWER, -15);
        assert_eq!(ST_SC_COST, 200);
        assert_eq!(ST_BUILD_FRAMES, 160);
    }

    #[test]
    fn supply_tower_consumes_power() {
        let pv = PowerValue(ST_POWER);
        assert!(pv.0 < 0, "Supply Tower should consume power");
    }

    #[test]
    fn supply_tower_state_default() {
        let st = SupplyTowerState::default();
        assert!(st.attached_chopper.is_none());
        assert!(st.landed_chopper.is_none());
        assert!(st.scheduled_sds.is_none());
        assert!(st.build_queue.is_empty());
        assert!(st.current_build.is_none());
        assert!(st.current_build_progress.is_none());
    }

    #[test]
    fn supply_tower_max_queue_size() {
        assert_eq!(SupplyTowerState::MAX_QUEUE_SIZE, 5);
    }

    #[test]
    fn supply_tower_production_cost_chopper() {
        let cost = SupplyTowerState::production_cost(&ObjectEnum::SupplyChopper).unwrap();
        assert_eq!(cost.space_crystals, 100);
        assert_eq!(cost.build_frames, 160);
    }

    #[test]
    fn supply_tower_production_cost_invalid() {
        assert!(SupplyTowerState::production_cost(&ObjectEnum::Peacekeeper).is_none());
    }

    #[test]
    fn supply_tower_try_queue() {
        let mut st = SupplyTowerState::default();
        assert!(st.try_queue(ObjectEnum::SupplyChopper));
        assert_eq!(st.build_queue.len(), 1);
    }

    #[test]
    fn supply_tower_try_queue_full() {
        let mut st = SupplyTowerState::default();
        for _ in 0..5 {
            assert!(st.try_queue(ObjectEnum::SupplyChopper));
        }
        assert!(!st.try_queue(ObjectEnum::SupplyChopper));
        assert_eq!(st.build_queue.len(), 5);
    }

    #[test]
    fn supply_tower_cancel_last() {
        let mut st = SupplyTowerState::default();
        st.try_queue(ObjectEnum::SupplyChopper);
        st.try_queue(ObjectEnum::SupplyChopper);
        let cancelled = st.cancel_last();
        assert!(matches!(cancelled, Some(ObjectEnum::SupplyChopper)));
        assert_eq!(st.build_queue.len(), 1);
    }

    #[test]
    fn supply_tower_cancel_empty() {
        let mut st = SupplyTowerState::default();
        assert!(st.cancel_last().is_none());
    }

    #[test]
    fn supply_tower_cancel_last_cancels_active_build_when_queue_empty() {
        let mut st = SupplyTowerState::default();
        st.current_build = Some(ObjectEnum::SupplyChopper);
        st.current_build_progress = Some(0.75);
        let cancelled = st.cancel_last().unwrap();
        assert_eq!(cancelled, ObjectEnum::SupplyChopper);
        assert!(st.current_build.is_none());
        assert!(st.current_build_progress.is_none());
    }

    #[test]
    fn supply_tower_has_cancellable() {
        let mut st = SupplyTowerState::default();
        assert!(!st.has_cancellable());
        st.current_build = Some(ObjectEnum::SupplyChopper);
        assert!(st.has_cancellable());
        st.current_build = None;
        st.try_queue(ObjectEnum::SupplyChopper);
        assert!(st.has_cancellable());
    }

    #[test]
    fn supply_tower_construction_cost() {
        let cost = DeploymentCenterState::construction_cost(&ObjectEnum::SupplyTower).unwrap();
        assert_eq!(cost.space_crystals, 200);
        assert_eq!(cost.build_frames, 240);
    }

    #[test]
    fn supply_tower_object_type() {
        let ot = ObjectEnum::SupplyTower.object_type();
        assert_eq!(ot.name, "Supply Tower");
        assert_eq!(ot.size, (3, 3));
        assert!(ot.destructible);
        assert_eq!(ot.sight_range, 4);
        assert!(!ot.groupable);
    }

    #[test]
    fn supply_tower_structure_type() {
        let st = ObjectEnum::SupplyTower.structure_type().unwrap();
        assert_eq!(st.symmetry_type, crate::types::SymmetryTypeEnum::AAAA);
    }

    // === Supply Chopper tests ===

    #[test]
    fn supply_chopper_constants() {
        assert_eq!(SC_MAX_HP, 150.0);
        assert_eq!(SC_POINT_ARMOR, 1);
        assert_eq!(SC_FULL_ARMOR, 1);
    }

    #[test]
    fn supply_chopper_state_default() {
        let sc = SupplyChopperState::default();
        assert_eq!(sc.carried_supplies, 0);
        assert!(sc.attached_tower.is_none());
    }

    #[test]
    fn supply_chopper_object_type() {
        let ot = ObjectEnum::SupplyChopper.object_type();
        assert_eq!(ot.name, "Supply Chopper");
        assert_eq!(ot.size, (1, 1));
        assert!(ot.destructible);
        assert_eq!(ot.sight_range, 5);
        assert!(ot.groupable);
    }

    #[test]
    fn supply_chopper_is_unit() {
        assert!(ObjectEnum::SupplyChopper.is_unit());
    }

    #[test]
    fn supply_chopper_is_not_structure() {
        assert!(ObjectEnum::SupplyChopper.structure_type().is_none());
    }

    #[test]
    fn supply_chopper_unit_control_cost_zero() {
        assert_eq!(ObjectEnum::SupplyChopper.unit_control_cost(), 0);
    }

    #[test]
    fn supply_tower_is_structure() {
        assert!(ObjectEnum::SupplyTower.structure_type().is_some());
    }

    #[test]
    fn supply_tower_is_not_unit() {
        assert!(!ObjectEnum::SupplyTower.is_unit());
    }

    #[test]
    fn supply_tower_state_default_has_no_rally_point() {
        let state = SupplyTowerState::default();
        assert!(state.rally_point.is_none());
    }
}
