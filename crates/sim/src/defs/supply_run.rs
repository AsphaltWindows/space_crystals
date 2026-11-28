//! The defs of the GDO's supply loop (`crate::supply_run`): a Supply Tower's ([`SupplyTowerDef`],
//! `StructureDef::supply_tower`) and a Supply Chopper's ([`SupplyChopperDef`], `UnitDef::supply_chopper`).

use serde::{Deserialize, Serialize};

use super::ObjectKind;

/// A Supply Tower's part in the supply loop (`StructureDef::supply_tower`, `crate::supply_run`;
/// `factions/gdo/economy.md > SupplyTower`).
#[derive(Copy, Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct SupplyTowerDef {
    /// The chopper it comes with (`factions/gdo/economy.md > SupplyTower > On Placement`: "One Supply Chopper is
    /// spawned on the tower at no additional cost (included in tower's construction cost). This chopper is
    /// automatically attached to the tower"), down on its spot ([`SupplyTowerDef::spot_offset_su`]).
    pub chopper: ObjectKind,
    /// Its one landing spot, from its centre, space units (`factions/gdo/economy.md > SupplyTower >
    /// SupplyTowerInstanceState`: "LandedChopper: ObjectInstance | None", one at a time): **Placeholder (not in the
    /// design: where on the tower a chopper lands)**, its centre. Inside its footprint (`Ruleset::check`).
    pub spot_offset_su: [f32; 2],
}

/// A Supply Chopper's part in the supply loop (`UnitDef::supply_chopper`, `crate::supply_run`;
/// `factions/gdo/units/supply_chopper.md`): it carries Supplies, and lands on Supply Towers and Supply Delivery
/// Stations.
#[derive(Copy, Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct SupplyChopperDef {
    /// The frames it takes to land on a spot, and to take off from it (`factions/gdo/units/supply_chopper.md > Unit
    /// Transport`: "**Landing and taking off take 16 frames (1 s) each** (as at the Airfield): it is an **air target**
    /// while landing and taking off, and a **ground target** while landed"; `crate::landing`). Positive, and short
    /// enough for its progress to fit (`Ruleset::check`).
    pub landing_ticks: u32,
}
