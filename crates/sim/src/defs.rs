//! Static object type definitions (`core/entities.md > Object Type`).
//!
//! One `ObjectKind` per object type in the design; its numbers live in a `static`
//! [`ObjectDef`] whose comments cite the design section they come from. Faction
//! types get their own modules (`defs/gdo.rs`, `defs/syndicate.rs`, `defs/colonists.rs`, …); unit bases and the
//! attributes of unit types are in `defs/units.rs`, those of structure types in `defs/structures.rs`.

mod colonists;
mod gdo;
mod structures;
mod syndicate;
#[cfg(test)]
pub(crate) mod test_defs;
mod units;

use glam::Vec2;
use serde::{Deserialize, Serialize};

use crate::player::Faction;

pub use gdo::RIOT_SHIELD_TOGGLE_TICKS;
pub use structures::{
    ExitRule, MiningDef, ProductionDef, Recipe, StructureDef, SymmetryType, TierDef, Transit, TunnelDef,
};
pub use units::{
    AttackAttrs, AttackType, Domain, FullyConnectedSubtype, MELEE_CONTACT_GU, MovementAttrs, MovementModel,
    RiotShieldDef, TargetDomain, TargetType, TurretAttrs, UnitBase, UnitBaseDef, UnitDef, WorkerDef,
};

#[derive(Copy, Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub enum ObjectKind {
    /// `core/map.md > SpaceCrystalsPatch`
    CrystalPatch,
    /// `core/map.md > SupplyDeliveryStation`
    SupplyDeliveryStation,
    /// `factions/gdo/units.md > Peacekeeper`
    Peacekeeper,
    /// `factions/syndicate/units.md > Guard`
    Guard,
    // New kinds go last: a kind's place in this list is part of the state hash (`hash.rs`).
    /// `factions/gdo/structures.md > DeploymentCenter`
    DeploymentCenter,
    /// `factions/gdo/economy.md > Generator`
    Generator,
    /// `factions/gdo/structures.md > Barracks`
    Barracks,
    /// `factions/gdo/economy.md > ExtractionFacility`
    ExtractionFacility,
    /// `factions/gdo/economy.md > ExtractionPlate`
    ExtractionPlate,
    /// `factions/colonists/units.md > Drone`
    Drone,
    /// `factions/colonists/economy.md > Beacon`
    Beacon,
    /// `factions/colonists/economy.md > Egress Node`
    EgressNode,
    /// `factions/colonists/structures.md > Robo Portal`
    RoboPortal,
    /// `factions/colonists/structures.md > Bio Portal`
    BioPortal,
    /// `factions/colonists/units.md > Sabre Legionnaire`
    SabreLegionnaire,
    /// `factions/colonists/units.md > Rifle Legionnaire`
    RifleLegionnaire,
    /// `factions/gdo/units.md > Peacekeeper > Riot Shield > Two unit types` (design change 0173): the Peacekeeper
    /// with its shield up
    PeacekeeperShielded,
    /// `factions/syndicate/units.md > Agent`
    Agent,
    /// `factions/syndicate/tunnels.md > Tunnel`, at any of its tiers (plan decision T39)
    Tunnel,
    /// `factions/syndicate/structures.md > Hideout`
    Hideout,
}

#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum Category {
    Unit,
    Structure,
    /// `core/map.md > Resource`: indestructible, unowned, no vision.
    Resource,
}

#[derive(Copy, Clone, Debug, PartialEq)]
pub enum Footprint {
    /// Structures and resources: whole tiles, `w` wide and `h` tall in the default orientation (`Size - Height x
    /// Width`, given for the default orientation: `core/entities.md > Structure Type > Side lettering convention`).
    Tiles { w: u32, h: u32 },
    /// Units: silhouette rectangle in space units (`systems/units.md > Unit > Silhouette`).
    Silhouette { w: f32, h: f32 },
}

/// Attributes shared by every object type (`core/entities.md > Object Type`).
#[derive(Clone, Debug)]
pub struct ObjectDef {
    pub kind: ObjectKind,
    pub name: &'static str,
    pub faction: Option<Faction>,
    pub category: Category,
    pub footprint: Footprint,
    pub selectable: bool,
    pub destructible: bool,
    pub groupable: bool,
    /// Grid units. 0 = provides no vision.
    pub sight_range: f32,
    /// `None` for indestructible types.
    pub max_hp: Option<f32>,
    /// `systems/combat.md > Shields > MaxShields`: "Colonists units/buildings only; defined per-entity alongside
    /// MaxHP", so `Some` for exactly the Colonists types (plan decision T14). The current Shields, within it, are
    /// `Object::shields` (`crate::shields`, slice M3-B).
    pub max_shields: Option<f32>,
    /// PointArmor and FullArmor, which units and structures have alike; none for map resources.
    pub armor: Armor,
    /// Every footprint tile must be flat (`core/map.md > Elevation > Structures only on flat terrain`, `> Map
    /// resources only on flat terrain`).
    pub needs_flat_ground: bool,
    /// Units only: their base, movement, attack and abilities (`systems/units.md > Unit`).
    pub unit: Option<UnitDef>,
    /// Structures only: their symmetry, BuildRadiusExtension and Power (`core/entities.md > Structure Type`).
    pub structure: Option<StructureDef>,
}

/// An object type's armour: `systems/units.md > Unit > PointArmor`, `> FullArmor` for units, and the same two
/// attributes in each structure type's stat block.
#[derive(Copy, Clone, Debug, Default, PartialEq)]
pub struct Armor {
    /// Taken off each single-target hit (`systems/combat.md > SingleTargetDamage`).
    pub point: f32,
    /// Taken off area damage, in proportion to the area hit (`systems/combat.md > AoEDamage`; no area attack exists
    /// yet).
    pub full: f32,
}

impl Armor {
    /// No armour: map resources, which are indestructible anyway.
    pub const NONE: Armor = Armor { point: 0.0, full: 0.0 };
}

impl ObjectDef {
    /// A unit's Silhouette, space units (`systems/units.md > Unit > Silhouette`); `None` for tile-based types.
    pub fn silhouette(&self) -> Option<Vec2> {
        match self.footprint {
            Footprint::Silhouette { w, h } => Some(Vec2::new(w, h)),
            Footprint::Tiles { .. } => None,
        }
    }

    /// Half a unit's Silhouette: the half extents of its collision box, axis-aligned for now (`docs/movement.md >
    /// 3`; design change 0033 makes Silhouettes rotate with the facing, which waits for the vehicles milestone).
    /// `None` for tile-based types.
    pub fn half_extents(&self) -> Option<Vec2> {
        self.silhouette().map(|s| s / 2.0)
    }

    /// A tile-based type's size in the default orientation, `(w, h)` tiles; `None` for units.
    pub fn tile_size(&self) -> Option<(u32, u32)> {
        match self.footprint {
            Footprint::Tiles { w, h } => Some((w, h)),
            Footprint::Silhouette { .. } => None,
        }
    }
}

impl ObjectKind {
    /// Every object type, in declaration order, so tools such as the map editor's palette
    /// list new types automatically. A test checks that it stays complete.
    pub const ALL: &'static [ObjectKind] = &[
        ObjectKind::CrystalPatch,
        ObjectKind::SupplyDeliveryStation,
        ObjectKind::Peacekeeper,
        ObjectKind::Guard,
        ObjectKind::DeploymentCenter,
        ObjectKind::Generator,
        ObjectKind::Barracks,
        ObjectKind::ExtractionFacility,
        ObjectKind::ExtractionPlate,
        ObjectKind::Drone,
        ObjectKind::Beacon,
        ObjectKind::EgressNode,
        ObjectKind::RoboPortal,
        ObjectKind::BioPortal,
        ObjectKind::SabreLegionnaire,
        ObjectKind::RifleLegionnaire,
        ObjectKind::PeacekeeperShielded,
        ObjectKind::Agent,
        ObjectKind::Tunnel,
        ObjectKind::Hideout,
    ];

    pub fn def(self) -> &'static ObjectDef {
        // The sim's own unit tests may give a kind other numbers (`test_defs`).
        #[cfg(test)]
        if let Some(def) = test_defs::get(self) {
            return def;
        }
        match self {
            ObjectKind::CrystalPatch => &CRYSTAL_PATCH,
            ObjectKind::SupplyDeliveryStation => &SUPPLY_DELIVERY_STATION,
            ObjectKind::Peacekeeper => &gdo::PEACEKEEPER,
            ObjectKind::Guard => &syndicate::GUARD,
            ObjectKind::DeploymentCenter => &gdo::DEPLOYMENT_CENTER,
            ObjectKind::Generator => &gdo::GENERATOR,
            ObjectKind::Barracks => &gdo::BARRACKS,
            ObjectKind::ExtractionFacility => &gdo::EXTRACTION_FACILITY,
            ObjectKind::ExtractionPlate => &gdo::EXTRACTION_PLATE,
            ObjectKind::Drone => &colonists::DRONE,
            ObjectKind::Beacon => &colonists::BEACON,
            ObjectKind::EgressNode => &colonists::EGRESS_NODE,
            ObjectKind::RoboPortal => &colonists::ROBO_PORTAL,
            ObjectKind::BioPortal => &colonists::BIO_PORTAL,
            ObjectKind::SabreLegionnaire => &colonists::SABRE_LEGIONNAIRE,
            ObjectKind::RifleLegionnaire => &colonists::RIFLE_LEGIONNAIRE,
            ObjectKind::PeacekeeperShielded => &gdo::PEACEKEEPER_SHIELDED,
            ObjectKind::Agent => &syndicate::AGENT,
            ObjectKind::Tunnel => &syndicate::TUNNEL,
            ObjectKind::Hideout => &syndicate::HIDEOUT,
        }
    }

    /// Whether the footprints of objects of `self` and `other` may cover the same tiles. Only an Extraction Plate
    /// and the Space Crystals Patch it stands on may (`factions/gdo/economy.md > ExtractionPlate`: "placed onto a
    /// Space Crystal Patch"); every other pair of tile-based objects is kept apart (`core/entities.md > Placement
    /// Validation > Direct Placement`: no structure overlap), structures and map resources included (`> All
    /// placement methods: no buildings on resources`, design change 0096), and an underground structure and a surface
    /// one too (design change 0240: "a surface structure and an underground expansion (any player's, including
    /// neutral ones) can never share a tile"; the placement methods of slices M4-E and M4-F follow it in both layers,
    /// the expansions' own in theirs). As two plates may not overlap, nor two
    /// patches, a patch holds one plate at most: a plate goes only onto an uncovered patch. Units have no
    /// footprint: this is about tile-based objects only.
    pub fn may_overlap(self, other: ObjectKind) -> bool {
        use ObjectKind::{CrystalPatch, ExtractionPlate};
        matches!(
            (self, other),
            (ExtractionPlate, CrystalPatch) | (CrystalPatch, ExtractionPlate)
        )
    }
}

/// `core/map.md > SpaceCrystalsPatch`, `> Resource`.
static CRYSTAL_PATCH: ObjectDef = ObjectDef {
    kind: ObjectKind::CrystalPatch,
    name: "Space Crystals",
    faction: None,
    category: Category::Resource,
    footprint: Footprint::Tiles { w: 1, h: 1 }, // Size - 1x1
    selectable: true,
    destructible: false,
    groupable: false,
    sight_range: 0.0,
    max_hp: None,
    max_shields: None,
    armor: Armor::NONE,
    needs_flat_ground: true, // `core/map.md > Elevation > Map resources only on flat terrain`
    unit: None,
    structure: None,
};

/// `core/map.md > SupplyDeliveryStation`, `> Resource`.
static SUPPLY_DELIVERY_STATION: ObjectDef = ObjectDef {
    kind: ObjectKind::SupplyDeliveryStation,
    name: "Supply Delivery Station",
    faction: None,
    category: Category::Resource,
    footprint: Footprint::Tiles { w: 2, h: 2 }, // Size - 2x2
    selectable: true,
    destructible: false,
    groupable: false,
    sight_range: 0.0,
    max_hp: None,
    max_shields: None,
    armor: Armor::NONE,
    needs_flat_ground: true, // `core/map.md > Elevation > Map resources only on flat terrain`
    unit: None,
    structure: None,
};

#[cfg(test)]
mod tests;
