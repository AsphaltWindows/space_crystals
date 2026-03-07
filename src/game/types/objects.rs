#![allow(dead_code)]
use bevy::prelude::*;
use crate::types::{ObjectEnum, Owner, SymmetryTypeEnum, StructureRotation};

/// Trait for displaying object information in the info panel.
/// Concrete implementations will be added per object type.
pub trait InfoPanel {
    /// Render display info for this object, given its owner and the player viewing it.
    fn display_info(&self, owner: &Owner, selector_player: u8) -> String;
}

/// Static data definition for any game object (unit, structure, or resource)
#[derive(Clone, Debug)]
pub struct ObjectType {
    pub name: String,
    pub size: (u32, u32),
    pub destructible: bool,
    pub sight_range: u32,
    pub groupable: bool,
}

/// Static data extending ObjectType for structure objects
#[derive(Clone, Debug)]
pub struct StructureType {
    pub object_type: ObjectType,
    pub symmetry_type: SymmetryTypeEnum,
}

impl StructureType {
    /// Validate that the structure's size is consistent with its symmetry type.
    /// Symmetry types AAAA, AAAB, AABB, and AABC require equal height and width.
    /// Symmetry types ABAB, ABAC, and ABCD allow different height and width.
    pub fn validate_size(&self) -> Result<(), String> {
        let (h, w) = self.object_type.size;
        let requires_square = matches!(
            self.symmetry_type,
            SymmetryTypeEnum::AAAA
                | SymmetryTypeEnum::AAAB
                | SymmetryTypeEnum::AABB
                | SymmetryTypeEnum::AABC
        );
        if requires_square && h != w {
            Err(format!(
                "Symmetry type {:?} requires equal height and width, got {}x{}",
                self.symmetry_type, h, w
            ))
        } else {
            Ok(())
        }
    }
}

/// Component representing a game object instance (unit, structure, or resource)
#[derive(Component, Clone, Debug)]
pub struct ObjectInstance {
    pub object_type: ObjectEnum,
    pub hp: Option<f32>,
    pub max_hp: Option<f32>,
}

impl ObjectInstance {
    /// Create a new destructible object instance at full health
    pub fn destructible(object_type: ObjectEnum, max_hp: f32) -> Self {
        Self {
            object_type,
            hp: Some(max_hp),
            max_hp: Some(max_hp),
        }
    }

    /// Create a destructible instance at 10% HP (for ConstructionHP rule).
    /// Used for structures built on-site that gain HP as construction progresses.
    pub fn under_construction(object_type: ObjectEnum, max_hp: f32) -> Self {
        Self {
            object_type,
            hp: Some(max_hp * 0.10),
            max_hp: Some(max_hp),
        }
    }

    /// Create a new indestructible object instance
    pub fn indestructible(object_type: ObjectEnum) -> Self {
        Self {
            object_type,
            hp: None,
            max_hp: None,
        }
    }

    /// Check if this instance is destructible (has hp tracking)
    pub fn is_destructible(&self) -> bool {
        self.hp.is_some()
    }

    /// Check if this instance is alive (has hp > 0 or is indestructible)
    pub fn is_alive(&self) -> bool {
        match self.hp {
            Some(hp) => hp > 0.0,
            None => true,
        }
    }

    /// Apply damage to this instance. Returns true if the instance was destroyed.
    pub fn apply_damage(&mut self, amount: f32) -> bool {
        if let Some(ref mut hp) = self.hp {
            *hp = (*hp - amount).max(0.0);
            *hp <= 0.0
        } else {
            false // indestructible
        }
    }

    /// Get health as a fraction (0.0 to 1.0), or 1.0 for indestructible
    pub fn health_fraction(&self) -> f32 {
        match (self.hp, self.max_hp) {
            (Some(hp), Some(max_hp)) if max_hp > 0.0 => hp / max_hp,
            _ => 1.0,
        }
    }
}

/// Marker component for structure name label entities (billboard text above structures)
#[derive(Component)]
pub struct StructureLabel;

/// Component for structure instances, added alongside ObjectInstance
#[derive(Component, Clone, Debug)]
pub struct StructureInstance {
    pub rotation: StructureRotation,
    pub flip_horizontal: bool,
    pub flip_vertical: bool,
}

impl Default for StructureInstance {
    fn default() -> Self {
        Self {
            rotation: StructureRotation::R0,
            flip_horizontal: false,
            flip_vertical: false,
        }
    }
}

impl StructureInstance {
    /// Create a new StructureInstance with the given rotation and flip settings
    pub fn new(rotation: StructureRotation, flip_horizontal: bool, flip_vertical: bool) -> Self {
        Self { rotation, flip_horizontal, flip_vertical }
    }

    /// Compute the oriented side labels [N, E, S, W] after applying rotation and flipping.
    ///
    /// Starting from the base labels for the given symmetry type, rotation cycles the array
    /// (R90 = shift right by 1), horizontal flip swaps E↔W, and vertical flip swaps N↔S.
    pub fn oriented_labels(&self, sym: SymmetryTypeEnum) -> [char; 4] {
        let base = match sym {
            SymmetryTypeEnum::AAAA => ['A', 'A', 'A', 'A'],
            SymmetryTypeEnum::AAAB => ['A', 'A', 'A', 'B'],
            SymmetryTypeEnum::AABB => ['A', 'A', 'B', 'B'],
            SymmetryTypeEnum::ABAB => ['A', 'B', 'A', 'B'],
            SymmetryTypeEnum::AABC => ['A', 'A', 'B', 'C'],
            SymmetryTypeEnum::ABAC => ['A', 'B', 'A', 'C'],
            SymmetryTypeEnum::ABCD => ['A', 'B', 'C', 'D'],
        };

        // Apply rotation: R90 shifts right by 1 (West becomes North, etc.)
        let shift = match self.rotation {
            StructureRotation::R0 => 0,
            StructureRotation::R90 => 1,
            StructureRotation::R180 => 2,
            StructureRotation::R270 => 3,
        };
        let mut labels = [' '; 4];
        for i in 0..4 {
            labels[(i + shift) % 4] = base[i];
        }

        // Apply horizontal flip: swap E(index 1) ↔ W(index 3)
        if self.flip_horizontal {
            labels.swap(1, 3);
        }

        // Apply vertical flip: swap N(index 0) ↔ S(index 2)
        if self.flip_vertical {
            labels.swap(0, 2);
        }

        labels
    }

    /// Count the number of distinct orientations for a given symmetry type.
    /// Enumerates all 16 combinations of (4 rotations × 2 flip_h × 2 flip_v)
    /// and returns the number of unique side-label arrangements.
    pub fn distinct_orientation_count(sym: SymmetryTypeEnum) -> usize {
        use std::collections::HashSet;
        let rotations = [
            StructureRotation::R0,
            StructureRotation::R90,
            StructureRotation::R180,
            StructureRotation::R270,
        ];
        let mut seen = HashSet::new();
        for &rot in &rotations {
            for &fh in &[false, true] {
                for &fv in &[false, true] {
                    let inst = StructureInstance::new(rot, fh, fv);
                    seen.insert(inst.oriented_labels(sym));
                }
            }
        }
        seen.len()
    }
}

impl ObjectEnum {
    /// Get the static ObjectType data for this object
    pub fn object_type(&self) -> ObjectType {
        match self {
            ObjectEnum::Peacekeeper => ObjectType {
                name: "Peacekeeper".to_string(),
                size: (24, 24),
                destructible: true,
                sight_range: 5,
                groupable: true,
            },
            ObjectEnum::SyndicateAgent => ObjectType {
                name: "Agent".to_string(),
                size: (36, 36),
                destructible: true,
                sight_range: 5,
                groupable: false, // Ungroupable — each Agent is its own SelectionGroup
            },
            ObjectEnum::DeploymentCenter => ObjectType {
                name: "Deployment Center".to_string(),
                size: (4, 4),
                destructible: true,
                sight_range: 6,
                groupable: false,
            },
            ObjectEnum::PowerPlant => ObjectType {
                name: "Power Plant".to_string(),
                size: (2, 2),
                destructible: true,
                sight_range: 3,
                groupable: true,
            },
            ObjectEnum::Barracks => ObjectType {
                name: "Barracks".to_string(),
                size: (3, 2),
                destructible: true,
                sight_range: 4,
                groupable: true,
            },
            ObjectEnum::ExtractionFacility => ObjectType {
                name: "Extraction Facility".to_string(),
                size: (3, 3),
                destructible: true,
                sight_range: 3,
                groupable: false,
            },
            ObjectEnum::ExtractionPlate => ObjectType {
                name: "Extraction Plate".to_string(),
                size: (1, 1),
                destructible: true,
                sight_range: 0,
                groupable: true,
            },
            ObjectEnum::SupplyTower => ObjectType {
                name: "Supply Tower".to_string(),
                size: (3, 3),
                destructible: true,
                sight_range: 4,
                groupable: false,
            },
            ObjectEnum::SupplyChopper => ObjectType {
                name: "Supply Chopper".to_string(),
                size: (1, 1),
                destructible: true,
                sight_range: 5,
                groupable: true,
            },
            ObjectEnum::Tunnel => ObjectType {
                name: "Tunnel".to_string(),
                size: (4, 4),
                destructible: true,
                sight_range: 5,
                groupable: false,
            },
            ObjectEnum::Headquarters => ObjectType {
                name: "Headquarters".to_string(),
                size: (2, 2),
                destructible: true,
                sight_range: 0, // Underground, no surface sight
                groupable: false, // Ungroupable — each HQ is its own SelectionGroup
            },
            ObjectEnum::SpaceCrystalsPatch => ObjectType {
                name: "Space Crystals Patch".to_string(),
                size: (1, 1),
                destructible: false,
                sight_range: 0,
                groupable: false,
            },
            ObjectEnum::SupplyDeliveryStation => ObjectType {
                name: "Supply Delivery Station".to_string(),
                size: (2, 2),
                destructible: false,
                sight_range: 0,
                groupable: false,
            },
        }
    }

    /// Get the StructureType data if this object is a structure, None otherwise
    pub fn structure_type(&self) -> Option<StructureType> {
        match self {
            ObjectEnum::DeploymentCenter => Some(StructureType {
                object_type: self.object_type(),
                symmetry_type: SymmetryTypeEnum::AAAA,
            }),
            ObjectEnum::PowerPlant => Some(StructureType {
                object_type: self.object_type(),
                symmetry_type: SymmetryTypeEnum::AAAA,
            }),
            ObjectEnum::Barracks => Some(StructureType {
                object_type: self.object_type(),
                symmetry_type: SymmetryTypeEnum::ABAC,
            }),
            ObjectEnum::ExtractionFacility => Some(StructureType {
                object_type: self.object_type(),
                symmetry_type: SymmetryTypeEnum::AAAA,
            }),
            ObjectEnum::ExtractionPlate => Some(StructureType {
                object_type: self.object_type(),
                symmetry_type: SymmetryTypeEnum::AAAA,
            }),
            ObjectEnum::SupplyTower => Some(StructureType {
                object_type: self.object_type(),
                symmetry_type: SymmetryTypeEnum::AAAA,
            }),
            ObjectEnum::Tunnel => Some(StructureType {
                object_type: self.object_type(),
                symmetry_type: SymmetryTypeEnum::ABCD,
            }),
            ObjectEnum::Headquarters => Some(StructureType {
                object_type: self.object_type(),
                symmetry_type: SymmetryTypeEnum::AAAA,
            }),
            _ => None,
        }
    }

    /// Check if this object is a structure
    pub fn is_structure(&self) -> bool {
        self.structure_type().is_some()
    }

    /// Check if this object is a unit
    pub fn is_unit(&self) -> bool {
        matches!(self, ObjectEnum::Peacekeeper | ObjectEnum::SupplyChopper | ObjectEnum::SyndicateAgent)
    }

    /// Check if this object is a resource
    pub fn is_resource(&self) -> bool {
        matches!(self, ObjectEnum::SpaceCrystalsPatch | ObjectEnum::SupplyDeliveryStation)
    }

    /// Get the unit control cost for producing this unit type.
    /// Returns 0 for non-unit objects.
    pub fn unit_control_cost(&self) -> u32 {
        use crate::game::units::types::unit_data::{PEACEKEEPER_CONTROL_COST, AGENT_CONTROL_COST};
        match self {
            ObjectEnum::Peacekeeper => PEACEKEEPER_CONTROL_COST,
            ObjectEnum::SyndicateAgent => AGENT_CONTROL_COST,
            ObjectEnum::SupplyChopper => 0, // Choppers don't consume unit control
            _ => 0,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // --- SpaceCrystalsPatch ObjectType tests ---

    #[test]
    fn test_space_crystals_patch_object_type_size() {
        let obj = ObjectEnum::SpaceCrystalsPatch.object_type();
        assert_eq!(obj.size, (1, 1));
    }

    #[test]
    fn test_space_crystals_patch_object_type_indestructible() {
        let obj = ObjectEnum::SpaceCrystalsPatch.object_type();
        assert!(!obj.destructible);
    }

    #[test]
    fn test_space_crystals_patch_object_type_no_sight() {
        let obj = ObjectEnum::SpaceCrystalsPatch.object_type();
        assert_eq!(obj.sight_range, 0);
    }

    #[test]
    fn test_space_crystals_patch_object_type_not_groupable() {
        let obj = ObjectEnum::SpaceCrystalsPatch.object_type();
        assert!(!obj.groupable);
    }

    #[test]
    fn test_space_crystals_patch_object_type_name() {
        let obj = ObjectEnum::SpaceCrystalsPatch.object_type();
        assert_eq!(obj.name, "Space Crystals Patch");
    }

    // --- SupplyDeliveryStation ObjectType tests ---

    #[test]
    fn test_supply_delivery_station_object_type_size() {
        let obj = ObjectEnum::SupplyDeliveryStation.object_type();
        assert_eq!(obj.size, (2, 2));
    }

    #[test]
    fn test_supply_delivery_station_object_type_indestructible() {
        let obj = ObjectEnum::SupplyDeliveryStation.object_type();
        assert!(!obj.destructible);
    }

    #[test]
    fn test_supply_delivery_station_object_type_no_sight() {
        let obj = ObjectEnum::SupplyDeliveryStation.object_type();
        assert_eq!(obj.sight_range, 0);
    }

    #[test]
    fn test_supply_delivery_station_object_type_not_groupable() {
        let obj = ObjectEnum::SupplyDeliveryStation.object_type();
        assert!(!obj.groupable);
    }

    #[test]
    fn test_supply_delivery_station_object_type_name() {
        let obj = ObjectEnum::SupplyDeliveryStation.object_type();
        assert_eq!(obj.name, "Supply Delivery Station");
    }

    // --- is_resource() tests ---

    #[test]
    fn test_is_resource_true_for_space_crystals_patch() {
        assert!(ObjectEnum::SpaceCrystalsPatch.is_resource());
    }

    #[test]
    fn test_is_resource_true_for_supply_delivery_station() {
        assert!(ObjectEnum::SupplyDeliveryStation.is_resource());
    }

    #[test]
    fn test_is_resource_false_for_peacekeeper() {
        assert!(!ObjectEnum::Peacekeeper.is_resource());
    }

    #[test]
    fn test_is_resource_false_for_deployment_center() {
        assert!(!ObjectEnum::DeploymentCenter.is_resource());
    }

    #[test]
    fn test_is_resource_false_for_barracks() {
        assert!(!ObjectEnum::Barracks.is_resource());
    }

    #[test]
    fn test_is_resource_false_for_tunnel() {
        assert!(!ObjectEnum::Tunnel.is_resource());
    }

    // --- ObjectInstance tests ---

    #[test]
    fn test_indestructible_instance_has_no_hp() {
        let obj = ObjectInstance::indestructible(ObjectEnum::SpaceCrystalsPatch);
        assert!(obj.hp.is_none());
        assert!(obj.max_hp.is_none());
    }

    #[test]
    fn test_indestructible_instance_is_not_destructible() {
        let obj = ObjectInstance::indestructible(ObjectEnum::SpaceCrystalsPatch);
        assert!(!obj.is_destructible());
    }

    #[test]
    fn test_indestructible_instance_is_alive() {
        let obj = ObjectInstance::indestructible(ObjectEnum::SpaceCrystalsPatch);
        assert!(obj.is_alive());
    }

    #[test]
    fn test_indestructible_instance_cannot_be_damaged() {
        let mut obj = ObjectInstance::indestructible(ObjectEnum::SpaceCrystalsPatch);
        let destroyed = obj.apply_damage(100.0);
        assert!(!destroyed);
        assert!(obj.is_alive());
    }

    #[test]
    fn test_indestructible_instance_health_fraction_is_one() {
        let obj = ObjectInstance::indestructible(ObjectEnum::SpaceCrystalsPatch);
        assert_eq!(obj.health_fraction(), 1.0);
    }

    #[test]
    fn test_destructible_instance_is_destructible() {
        let obj = ObjectInstance::destructible(ObjectEnum::Peacekeeper, 100.0);
        assert!(obj.is_destructible());
    }

    #[test]
    fn test_destructible_instance_can_be_damaged() {
        let mut obj = ObjectInstance::destructible(ObjectEnum::Peacekeeper, 100.0);
        let destroyed = obj.apply_damage(50.0);
        assert!(!destroyed);
        assert_eq!(obj.hp, Some(50.0));
    }

    #[test]
    fn test_destructible_instance_destroyed_on_full_damage() {
        let mut obj = ObjectInstance::destructible(ObjectEnum::Peacekeeper, 100.0);
        let destroyed = obj.apply_damage(100.0);
        assert!(destroyed);
        assert!(!obj.is_alive());
    }

    // --- Resource types are not structures ---

    #[test]
    fn test_space_crystals_patch_is_not_structure() {
        assert!(!ObjectEnum::SpaceCrystalsPatch.is_structure());
    }

    #[test]
    fn test_supply_delivery_station_is_not_structure() {
        assert!(!ObjectEnum::SupplyDeliveryStation.is_structure());
    }

    // --- Resource types are not units ---

    #[test]
    fn test_space_crystals_patch_is_not_unit() {
        assert!(!ObjectEnum::SpaceCrystalsPatch.is_unit());
    }

    #[test]
    fn test_supply_delivery_station_is_not_unit() {
        assert!(!ObjectEnum::SupplyDeliveryStation.is_unit());
    }

    // --- Resources spawned with neutral owner ---

    #[test]
    fn test_neutral_owner_has_no_player_id() {
        let owner = Owner::neutral();
        assert_eq!(owner.0, None);
    }

    // --- Symmetry/size constraint tests ---

    #[test]
    fn test_aaaa_symmetry_equal_size_valid() {
        // DeploymentCenter is 4x4 with AAAA symmetry — should pass validation
        let st = ObjectEnum::DeploymentCenter.structure_type().unwrap();
        assert_eq!(st.symmetry_type, SymmetryTypeEnum::AAAA);
        assert!(st.validate_size().is_ok());
    }

    #[test]
    fn test_aaaa_symmetry_unequal_size_invalid() {
        let st = StructureType {
            object_type: ObjectType {
                name: "Test".to_string(),
                size: (3, 2),
                destructible: true,
                sight_range: 1,
                groupable: false,
            },
            symmetry_type: SymmetryTypeEnum::AAAA,
        };
        assert!(st.validate_size().is_err());
    }

    #[test]
    fn test_aaab_symmetry_requires_equal_size() {
        let st = StructureType {
            object_type: ObjectType {
                name: "Test".to_string(),
                size: (3, 2),
                destructible: true,
                sight_range: 1,
                groupable: false,
            },
            symmetry_type: SymmetryTypeEnum::AAAB,
        };
        assert!(st.validate_size().is_err());
    }

    #[test]
    fn test_aabb_symmetry_requires_equal_size() {
        let st = StructureType {
            object_type: ObjectType {
                name: "Test".to_string(),
                size: (4, 2),
                destructible: true,
                sight_range: 1,
                groupable: false,
            },
            symmetry_type: SymmetryTypeEnum::AABB,
        };
        assert!(st.validate_size().is_err());
    }

    #[test]
    fn test_aabc_symmetry_requires_equal_size() {
        let st = StructureType {
            object_type: ObjectType {
                name: "Test".to_string(),
                size: (5, 3),
                destructible: true,
                sight_range: 1,
                groupable: false,
            },
            symmetry_type: SymmetryTypeEnum::AABC,
        };
        assert!(st.validate_size().is_err());
    }

    #[test]
    fn test_abac_symmetry_allows_unequal_size() {
        // Barracks is 3x2 with ABAC symmetry — should pass validation
        let st = ObjectEnum::Barracks.structure_type().unwrap();
        assert_eq!(st.symmetry_type, SymmetryTypeEnum::ABAC);
        assert_eq!(st.object_type.size, (3, 2));
        assert!(st.validate_size().is_ok());
    }

    #[test]
    fn test_abab_symmetry_allows_unequal_size() {
        let st = StructureType {
            object_type: ObjectType {
                name: "Test".to_string(),
                size: (4, 2),
                destructible: true,
                sight_range: 1,
                groupable: false,
            },
            symmetry_type: SymmetryTypeEnum::ABAB,
        };
        assert!(st.validate_size().is_ok());
    }

    #[test]
    fn test_abcd_symmetry_allows_unequal_size() {
        let st = StructureType {
            object_type: ObjectType {
                name: "Test".to_string(),
                size: (5, 3),
                destructible: true,
                sight_range: 1,
                groupable: false,
            },
            symmetry_type: SymmetryTypeEnum::ABCD,
        };
        assert!(st.validate_size().is_ok());
    }

    // --- All existing structures pass validation ---

    #[test]
    fn test_all_existing_structures_pass_validation() {
        let structures = [
            ObjectEnum::DeploymentCenter,
            ObjectEnum::PowerPlant,
            ObjectEnum::Barracks,
            ObjectEnum::ExtractionFacility,
            ObjectEnum::ExtractionPlate,
            ObjectEnum::SupplyTower,
            ObjectEnum::Tunnel,
            ObjectEnum::Headquarters,
        ];
        for obj in structures {
            let st = obj.structure_type().expect(&format!("{:?} should be a structure", obj));
            assert!(
                st.validate_size().is_ok(),
                "{:?} failed size validation: {:?}",
                obj,
                st.validate_size()
            );
        }
    }

    // --- Destructible/indestructible HP tests ---

    #[test]
    fn test_destructible_peacekeeper_has_hp() {
        let obj = ObjectInstance::destructible(ObjectEnum::Peacekeeper, 100.0);
        assert_eq!(obj.hp, Some(100.0));
        assert_eq!(obj.max_hp, Some(100.0));
    }

    #[test]
    fn test_indestructible_resource_has_no_hp() {
        let obj = ObjectInstance::indestructible(ObjectEnum::SpaceCrystalsPatch);
        assert_eq!(obj.hp, None);
        assert_eq!(obj.max_hp, None);
    }

    #[test]
    fn test_damage_destroys_at_zero_hp() {
        let mut obj = ObjectInstance::destructible(ObjectEnum::Peacekeeper, 50.0);
        assert!(!obj.apply_damage(30.0)); // 20 hp left
        assert!(obj.is_alive());
        assert!(obj.apply_damage(20.0)); // 0 hp, destroyed
        assert!(!obj.is_alive());
    }

    // --- All ObjectEnum variants have valid object_type() ---

    #[test]
    fn test_all_object_enum_variants_have_valid_object_type() {
        let all_variants = [
            ObjectEnum::Peacekeeper,
            ObjectEnum::SupplyChopper,
            ObjectEnum::SyndicateAgent,
            ObjectEnum::PowerPlant,
            ObjectEnum::Barracks,
            ObjectEnum::DeploymentCenter,
            ObjectEnum::ExtractionFacility,
            ObjectEnum::ExtractionPlate,
            ObjectEnum::SupplyTower,
            ObjectEnum::Tunnel,
            ObjectEnum::Headquarters,
            ObjectEnum::SpaceCrystalsPatch,
            ObjectEnum::SupplyDeliveryStation,
        ];
        for variant in all_variants {
            let ot = variant.object_type();
            assert!(!ot.name.is_empty(), "{:?} has empty name", variant);
            assert!(ot.size.0 > 0 && ot.size.1 > 0, "{:?} has zero size", variant);
        }
    }

    // --- Classification tests ---

    #[test]
    fn test_structures_are_structures() {
        assert!(ObjectEnum::DeploymentCenter.is_structure());
        assert!(ObjectEnum::PowerPlant.is_structure());
        assert!(ObjectEnum::Barracks.is_structure());
        assert!(ObjectEnum::ExtractionFacility.is_structure());
        assert!(ObjectEnum::ExtractionPlate.is_structure());
        assert!(ObjectEnum::SupplyTower.is_structure());
        assert!(ObjectEnum::Tunnel.is_structure());
        assert!(ObjectEnum::Headquarters.is_structure());
    }

    #[test]
    fn test_units_are_units() {
        assert!(ObjectEnum::Peacekeeper.is_unit());
        assert!(ObjectEnum::SupplyChopper.is_unit());
    }

    #[test]
    fn test_non_structures_return_none() {
        assert!(ObjectEnum::Peacekeeper.structure_type().is_none());
        assert!(ObjectEnum::SupplyChopper.structure_type().is_none());
        assert!(ObjectEnum::SpaceCrystalsPatch.structure_type().is_none());
        assert!(ObjectEnum::SupplyDeliveryStation.structure_type().is_none());
    }

    // --- under_construction tests ---

    #[test]
    fn test_under_construction_starts_at_ten_percent_hp() {
        let obj = ObjectInstance::under_construction(ObjectEnum::DeploymentCenter, 1000.0);
        assert_eq!(obj.hp, Some(100.0));
        assert_eq!(obj.max_hp, Some(1000.0));
    }

    #[test]
    fn test_under_construction_is_destructible() {
        let obj = ObjectInstance::under_construction(ObjectEnum::DeploymentCenter, 500.0);
        assert!(obj.is_destructible());
    }

    #[test]
    fn test_under_construction_is_alive() {
        let obj = ObjectInstance::under_construction(ObjectEnum::DeploymentCenter, 500.0);
        assert!(obj.is_alive());
    }

    #[test]
    fn test_under_construction_health_fraction_is_ten_percent() {
        let obj = ObjectInstance::under_construction(ObjectEnum::DeploymentCenter, 600.0);
        assert!((obj.health_fraction() - 0.10).abs() < 0.001);
    }

    #[test]
    fn test_under_construction_can_be_damaged() {
        let mut obj = ObjectInstance::under_construction(ObjectEnum::DeploymentCenter, 1000.0);
        // HP starts at 100 (10% of 1000)
        let destroyed = obj.apply_damage(50.0);
        assert!(!destroyed);
        assert_eq!(obj.hp, Some(50.0));
    }

    #[test]
    fn test_under_construction_can_be_destroyed() {
        let mut obj = ObjectInstance::under_construction(ObjectEnum::DeploymentCenter, 1000.0);
        // HP starts at 100 (10% of 1000)
        let destroyed = obj.apply_damage(100.0);
        assert!(destroyed);
        assert!(!obj.is_alive());
    }

    #[test]
    fn test_under_construction_with_zero_max_hp() {
        let obj = ObjectInstance::under_construction(ObjectEnum::DeploymentCenter, 0.0);
        assert_eq!(obj.hp, Some(0.0));
        assert_eq!(obj.max_hp, Some(0.0));
    }

    // --- InfoPanel trait exists ---

    #[test]
    fn test_info_panel_trait_is_object_safe() {
        // Verify InfoPanel trait can be used as a trait object
        struct TestPanel;
        impl InfoPanel for TestPanel {
            fn display_info(&self, _owner: &Owner, _selector_player: u8) -> String {
                "test".to_string()
            }
        }
        let panel: Box<dyn InfoPanel> = Box::new(TestPanel);
        let owner = Owner::neutral();
        assert_eq!(panel.display_info(&owner, 0), "test");
    }

    // --- StructureInstance flip fields tests ---

    #[test]
    fn test_structure_instance_default_no_flip() {
        let si = StructureInstance::default();
        assert!(!si.flip_horizontal);
        assert!(!si.flip_vertical);
        assert_eq!(si.rotation, StructureRotation::R0);
    }

    #[test]
    fn test_structure_instance_new_with_flips() {
        let si = StructureInstance::new(StructureRotation::R0, true, false);
        assert!(si.flip_horizontal);
        assert!(!si.flip_vertical);
    }

    #[test]
    fn test_structure_instance_new_with_rotation_and_flips() {
        let si = StructureInstance::new(StructureRotation::R90, false, true);
        assert_eq!(si.rotation, StructureRotation::R90);
        assert!(!si.flip_horizontal);
        assert!(si.flip_vertical);
    }

    // --- Oriented labels tests ---

    #[test]
    fn test_oriented_labels_r0_no_flip() {
        let si = StructureInstance::new(StructureRotation::R0, false, false);
        assert_eq!(si.oriented_labels(SymmetryTypeEnum::ABCD), ['A', 'B', 'C', 'D']);
    }

    #[test]
    fn test_oriented_labels_r90_no_flip() {
        let si = StructureInstance::new(StructureRotation::R90, false, false);
        // R90 shifts right by 1: base [A,B,C,D] -> [D,A,B,C]
        assert_eq!(si.oriented_labels(SymmetryTypeEnum::ABCD), ['D', 'A', 'B', 'C']);
    }

    #[test]
    fn test_oriented_labels_r0_flip_horizontal() {
        let si = StructureInstance::new(StructureRotation::R0, true, false);
        // Horizontal flip swaps E↔W: [A,B,C,D] -> [A,D,C,B]
        assert_eq!(si.oriented_labels(SymmetryTypeEnum::ABCD), ['A', 'D', 'C', 'B']);
    }

    #[test]
    fn test_oriented_labels_r0_flip_vertical() {
        let si = StructureInstance::new(StructureRotation::R0, false, true);
        // Vertical flip swaps N↔S: [A,B,C,D] -> [C,B,A,D]
        assert_eq!(si.oriented_labels(SymmetryTypeEnum::ABCD), ['C', 'B', 'A', 'D']);
    }

    #[test]
    fn test_oriented_labels_r0_both_flips() {
        let si = StructureInstance::new(StructureRotation::R0, true, true);
        // Both flips: [A,B,C,D] -> swap E↔W -> [A,D,C,B] -> swap N↔S -> [C,D,A,B]
        assert_eq!(si.oriented_labels(SymmetryTypeEnum::ABCD), ['C', 'D', 'A', 'B']);
    }

    // --- Distinct orientation count tests ---

    #[test]
    fn test_distinct_orientations_aaaa() {
        assert_eq!(StructureInstance::distinct_orientation_count(SymmetryTypeEnum::AAAA), 1);
    }

    #[test]
    fn test_distinct_orientations_aaab() {
        assert_eq!(StructureInstance::distinct_orientation_count(SymmetryTypeEnum::AAAB), 4);
    }

    #[test]
    fn test_distinct_orientations_aabb() {
        // AABB=[A,A,B,B]: 4 rotations all produce distinct label arrays.
        // Flipping only duplicates rotation results (fh at R0 = R270, fv at R0 = R90, etc.)
        assert_eq!(StructureInstance::distinct_orientation_count(SymmetryTypeEnum::AABB), 4);
    }

    #[test]
    fn test_distinct_orientations_abab() {
        assert_eq!(StructureInstance::distinct_orientation_count(SymmetryTypeEnum::ABAB), 2);
    }

    #[test]
    fn test_distinct_orientations_aabc() {
        // AABC=[A,A,B,C]: 4 rotations + horizontal flip each produce distinct arrays.
        // No pair of (rotation, flip) combos collide, giving 8 distinct orientations.
        assert_eq!(StructureInstance::distinct_orientation_count(SymmetryTypeEnum::AABC), 8);
    }

    #[test]
    fn test_distinct_orientations_abac() {
        assert_eq!(StructureInstance::distinct_orientation_count(SymmetryTypeEnum::ABAC), 4);
    }

    #[test]
    fn test_distinct_orientations_abcd() {
        assert_eq!(StructureInstance::distinct_orientation_count(SymmetryTypeEnum::ABCD), 8);
    }

    // --- Tunnel ObjectType tests ---

    #[test]
    fn test_tunnel_object_type_name() {
        let obj = ObjectEnum::Tunnel.object_type();
        assert_eq!(obj.name, "Tunnel");
    }

    #[test]
    fn test_tunnel_object_type_size() {
        let obj = ObjectEnum::Tunnel.object_type();
        assert_eq!(obj.size, (4, 4));
    }

    #[test]
    fn test_tunnel_object_type_destructible() {
        let obj = ObjectEnum::Tunnel.object_type();
        assert!(obj.destructible);
    }

    #[test]
    fn test_tunnel_object_type_sight_range() {
        let obj = ObjectEnum::Tunnel.object_type();
        assert_eq!(obj.sight_range, 5);
    }

    #[test]
    fn test_tunnel_object_type_not_groupable() {
        let obj = ObjectEnum::Tunnel.object_type();
        assert!(!obj.groupable);
    }

    #[test]
    fn test_tunnel_is_structure() {
        assert!(ObjectEnum::Tunnel.is_structure());
    }

    #[test]
    fn test_tunnel_is_not_unit() {
        assert!(!ObjectEnum::Tunnel.is_unit());
    }

    #[test]
    fn test_tunnel_symmetry_is_abcd() {
        let st = ObjectEnum::Tunnel.structure_type().unwrap();
        assert_eq!(st.symmetry_type, SymmetryTypeEnum::ABCD);
    }

    #[test]
    fn test_tunnel_has_8_distinct_orientations() {
        // ABCD symmetry gives 8 distinct orientations
        let st = ObjectEnum::Tunnel.structure_type().unwrap();
        assert_eq!(StructureInstance::distinct_orientation_count(st.symmetry_type), 8);
    }

    #[test]
    fn test_tunnel_passes_size_validation() {
        let st = ObjectEnum::Tunnel.structure_type().unwrap();
        assert!(st.validate_size().is_ok());
    }

    // === Power Plant ObjectType Tests ===

    #[test]
    fn test_power_plant_object_type_name() {
        let obj = ObjectEnum::PowerPlant.object_type();
        assert_eq!(obj.name, "Power Plant");
    }

    #[test]
    fn test_power_plant_object_type_size() {
        let obj = ObjectEnum::PowerPlant.object_type();
        assert_eq!(obj.size, (2, 2));
    }

    #[test]
    fn test_power_plant_object_type_destructible() {
        let obj = ObjectEnum::PowerPlant.object_type();
        assert!(obj.destructible);
    }

    #[test]
    fn test_power_plant_object_type_sight_range() {
        let obj = ObjectEnum::PowerPlant.object_type();
        assert_eq!(obj.sight_range, 3);
    }

    #[test]
    fn test_power_plant_object_type_groupable() {
        let obj = ObjectEnum::PowerPlant.object_type();
        assert!(obj.groupable);
    }

    #[test]
    fn test_power_plant_symmetry_aaaa() {
        let st = ObjectEnum::PowerPlant.structure_type().unwrap();
        assert_eq!(st.symmetry_type, SymmetryTypeEnum::AAAA);
    }

    // === Barracks ObjectType Tests ===

    #[test]
    fn test_barracks_object_type_name() {
        let obj = ObjectEnum::Barracks.object_type();
        assert_eq!(obj.name, "Barracks");
    }

    #[test]
    fn test_barracks_object_type_size() {
        let obj = ObjectEnum::Barracks.object_type();
        assert_eq!(obj.size, (3, 2));
    }

    #[test]
    fn test_barracks_object_type_destructible() {
        let obj = ObjectEnum::Barracks.object_type();
        assert!(obj.destructible);
    }

    #[test]
    fn test_barracks_object_type_sight_range() {
        let obj = ObjectEnum::Barracks.object_type();
        assert_eq!(obj.sight_range, 4);
    }

    #[test]
    fn test_barracks_object_type_groupable() {
        let obj = ObjectEnum::Barracks.object_type();
        assert!(obj.groupable);
    }

    #[test]
    fn test_barracks_symmetry_abac() {
        let st = ObjectEnum::Barracks.structure_type().unwrap();
        assert_eq!(st.symmetry_type, SymmetryTypeEnum::ABAC);
    }

    // --- Agent ungroupable test ---

    #[test]
    fn test_syndicate_agent_is_ungroupable() {
        let obj = ObjectEnum::SyndicateAgent.object_type();
        assert!(!obj.groupable, "Agent must be ungroupable per design spec");
    }

    // === unit_control_cost() Tests ===

    #[test]
    fn test_peacekeeper_unit_control_cost() {
        let cost = ObjectEnum::Peacekeeper.unit_control_cost();
        assert_eq!(cost, 1);
    }

    #[test]
    fn test_syndicate_agent_unit_control_cost() {
        let cost = ObjectEnum::SyndicateAgent.unit_control_cost();
        assert_eq!(cost, 1);
    }

    #[test]
    fn test_structure_unit_control_cost_is_zero() {
        assert_eq!(ObjectEnum::DeploymentCenter.unit_control_cost(), 0);
        assert_eq!(ObjectEnum::PowerPlant.unit_control_cost(), 0);
        assert_eq!(ObjectEnum::Barracks.unit_control_cost(), 0);
    }

    #[test]
    fn test_resource_unit_control_cost_is_zero() {
        assert_eq!(ObjectEnum::SpaceCrystalsPatch.unit_control_cost(), 0);
        assert_eq!(ObjectEnum::SupplyDeliveryStation.unit_control_cost(), 0);
    }
}
