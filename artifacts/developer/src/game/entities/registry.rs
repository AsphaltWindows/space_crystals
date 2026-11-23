//! Entity registry builder and Bevy plugin.

use bevy::prelude::*;
use super::types::EntityRegistry;
use super::factions::gdo::units::peacekeeper::Peacekeeper;
use super::factions::gdo::units::supply_chopper::SupplyChopper;
use super::factions::gdo::buildings::base::{DeploymentCenter, PowerPlant};
use super::factions::gdo::buildings::military::{Barracks, SupplyTowerDef};
use super::factions::gdo::buildings::extraction::{ExtractionFacility, ExtractionPlate};
use super::factions::syndicate::units::agent::SyndicateAgentDef;
use super::factions::syndicate::units::guard::SyndicateGuardDef;
use super::factions::syndicate::buildings::tunnel::TunnelDef;
use super::factions::syndicate::buildings::headquarters::HeadquartersDef;
use super::factions::cults::buildings::recruitment_center::RecruitmentCenterDef;
use super::factions::cults::buildings::storage::CultsStorageDef;
use super::factions::cults::buildings::armory::CultsArmoryDef;
use super::factions::cults::units::recruit::CultsRecruitDef;
use super::factions::cults::units::soldier::CultsSoldierDef;
use super::factions::cults::units::gunner::CultsGunnerDef;
use super::neutral::space_crystals_patch::SpaceCrystalsPatchDef;
use super::neutral::supply_delivery_station::SupplyDeliveryStationDef;

/// Build the entity registry with all registered entity definitions.
pub fn build_registry() -> EntityRegistry {
    let mut registry = EntityRegistry::new();
    registry.register_unit::<Peacekeeper>();
    registry.register_unit::<SupplyChopper>();
    registry.register_unit::<SyndicateAgentDef>();
    registry.register_unit::<SyndicateGuardDef>();
    // GDO structures
    registry.register_structure::<DeploymentCenter>();
    registry.register_structure::<PowerPlant>();
    registry.register_structure::<Barracks>();
    registry.register_structure::<ExtractionFacility>();
    registry.register_structure::<ExtractionPlate>();
    registry.register_structure::<SupplyTowerDef>();
    // Syndicate structures
    registry.register_structure::<TunnelDef>();
    registry.register_structure::<HeadquartersDef>();
    // Cults structures
    registry.register_structure::<RecruitmentCenterDef>();
    registry.register_structure::<CultsStorageDef>();
    registry.register_structure::<CultsArmoryDef>();
    // Cults units
    registry.register_unit::<CultsRecruitDef>();
    registry.register_unit::<CultsSoldierDef>();
    registry.register_unit::<CultsGunnerDef>();
    // Neutral resources
    registry.register_resource::<SpaceCrystalsPatchDef>();
    registry.register_resource::<SupplyDeliveryStationDef>();
    registry
}

/// Bevy plugin that inserts the `EntityRegistry` resource at startup.
pub struct EntitiesPlugin;

impl Plugin for EntitiesPlugin {
    fn build(&self, app: &mut App) {
        app.insert_resource(build_registry());
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_build_registry_has_peacekeeper() {
        let registry = build_registry();
        assert_eq!(registry.len(), 20);
        let entry = registry.get(&crate::types::ObjectEnum::Peacekeeper);
        assert!(entry.is_some());
        assert_eq!(entry.unwrap().object_type.name, "Peacekeeper");
        let sc_entry = registry.get(&crate::types::ObjectEnum::SupplyChopper);
        assert!(sc_entry.is_some());
        assert_eq!(sc_entry.unwrap().object_type.name, "Supply Chopper");
    }

    #[test]
    fn test_entities_plugin_inserts_registry() {
        let mut app = App::new();
        app.add_plugins(EntitiesPlugin);
        app.update();
        assert!(app.world().get_resource::<EntityRegistry>().is_some());
    }

    #[test]
    fn test_registry_cost_all_producible_entities() {
        use crate::types::ObjectEnum;
        let registry = build_registry();

        // GDO structures (built by DC)
        assert_eq!(registry.cost(&ObjectEnum::PowerPlant).unwrap().space_crystals, 150);
        assert_eq!(registry.cost(&ObjectEnum::PowerPlant).unwrap().build_frames, 160);
        assert_eq!(registry.cost(&ObjectEnum::Barracks).unwrap().space_crystals, 200);
        assert_eq!(registry.cost(&ObjectEnum::SupplyTower).unwrap().space_crystals, 200);
        assert_eq!(registry.cost(&ObjectEnum::SupplyTower).unwrap().build_frames, 240);
        assert_eq!(registry.cost(&ObjectEnum::ExtractionFacility).unwrap().space_crystals, 200);
        assert_eq!(registry.cost(&ObjectEnum::ExtractionFacility).unwrap().build_frames, 320);
        assert_eq!(registry.cost(&ObjectEnum::ExtractionPlate).unwrap().space_crystals, 75);
        assert_eq!(registry.cost(&ObjectEnum::ExtractionPlate).unwrap().build_frames, 96);

        // GDO units
        assert_eq!(registry.cost(&ObjectEnum::Peacekeeper).unwrap().space_crystals, 50);
        assert_eq!(registry.cost(&ObjectEnum::Peacekeeper).unwrap().build_frames, 80);
        assert_eq!(registry.cost(&ObjectEnum::SupplyChopper).unwrap().space_crystals, 100);
        assert_eq!(registry.cost(&ObjectEnum::SupplyChopper).unwrap().build_frames, 160);

        // Syndicate units
        assert_eq!(registry.cost(&ObjectEnum::SyndicateAgent).unwrap().space_crystals, 100);
        assert_eq!(registry.cost(&ObjectEnum::SyndicateAgent).unwrap().build_frames, 160);
        assert_eq!(registry.cost(&ObjectEnum::SyndicateGuard).unwrap().space_crystals, 125);
        assert_eq!(registry.cost(&ObjectEnum::SyndicateGuard).unwrap().build_frames, 120);

        // Cults units (Soldier/Gunner have training costs)
        assert_eq!(registry.cost(&ObjectEnum::CultsSoldier).unwrap().space_crystals, 75);
        assert_eq!(registry.cost(&ObjectEnum::CultsGunner).unwrap().space_crystals, 100);

        // Cults structures
        assert_eq!(registry.cost(&ObjectEnum::CultsArmory).unwrap().space_crystals, 150);

        // Non-producible entities (primary structures, resources, auto-produced units)
        assert!(registry.cost(&ObjectEnum::DeploymentCenter).is_none());
        assert!(registry.cost(&ObjectEnum::Tunnel).is_none());
        assert!(registry.cost(&ObjectEnum::RecruitmentCenter).is_none());
        assert!(registry.cost(&ObjectEnum::CultsRecruit).is_none());
        assert!(registry.cost(&ObjectEnum::SpaceCrystalsPatch).is_none());
        assert!(registry.cost(&ObjectEnum::SupplyDeliveryStation).is_none());
    }
}
