//! Static object type definitions (`core/entities.md > Object Type`).
//!
//! One `ObjectKind` per object type in the design; its numbers live in a `static` [`ObjectDef`] whose comments cite
//! the design section they come from, in a module of its own named like the type's design file: `defs/<faction>/
//! {units,structures}/<object>.rs` (`factions/<faction>/{units,structures}/<object>.md`; the economy buildings cite
//! their faction's `factions/<faction>/economy.md`, the Tunnel `factions/syndicate/tunnels.md`) and
//! `defs/map/<object>.rs` for the map's (`core/map.md`). A faction's root module, `defs/<faction>.rs`, holds the
//! faction's own rules. Unit bases and the attributes of unit types are in `defs/units.rs`, those of structure types in
//! `defs/structures.rs`.

mod colonists;
mod cults;
mod gdo;
mod leak;
mod map;
mod spells;
mod structures;
mod syndicate;
#[cfg(test)]
pub(crate) mod test_defs;
mod units;

use glam::Vec2;
use serde::{Deserialize, Serialize};

use crate::player::Faction;
use crate::scenario::SpawnSettings;

pub use colonists::units::drone::BEACON_DEPLOYMENT;
pub use colonists::units::weapon_conversion::CONVERT_TICKS;
pub use colonists::{BEACON_CAPACITY_CAP, BEACON_CAPACITY_PER_BUILDING};
pub use cults::augmentation::{AUGMENTATION, AugmentationTable, IncrementEffects};
pub use cults::research::{ResearchLine, Tech, TechDef};
pub use cults::{CONSTRUCT_MENU, ConstructEntry, ConstructMenu, ConstructSlot};
pub use gdo::UNIT_CONTROL_CAP;
pub use gdo::units::peacekeeper::RIOT_SHIELD_TOGGLE_TICKS;
pub use map::crystal_patch::DEFAULT_STARTING_AMOUNT;
pub use map::supply_delivery_station::{DEFAULT_DELIVERY_INTERVAL, DEFAULT_DELIVERY_SIZE, DEFAULT_STARTING_SUPPLIES};
pub use spells::{ShroudDef, SpellDef};
pub use structures::{
    AugmentRecipe, BuilderGoes, DropOff, ExitRule, ExpansionDef, HoldsDef, MiningDef, OccupantFire, PopUpDef,
    ProductionDef, Recipe, RecruitmentDef, ResearchDef, SiteDef, SitePrice, StructureDef, SymmetryType, TierDef,
    Transit, TunnelDef, UpgradeDef,
};
pub use syndicate::structures::tunnel::TRANSIT_DELAY_TICKS;
pub use syndicate::{CELLS_PER_TUNNEL_SPACE, EJECT_MENU_ORDER, EXPAND_MENU, ExpandEntry, TUNNEL_SPACE_CAP};
pub use units::{
    AttackAttrs, AttackType, Augmentable, ContactStep, ConversionDef, Domain, DropOffChoice, FullyConnectedSubtype,
    LoadDef, MELEE_CONTACT_GU, MovementAttrs, MovementModel, RepairDef, RepairPayment, RepairTargets, Resource,
    RiotShieldDef, TargetDomain, TargetType, TurretAttrs, UnitBase, UnitBaseDef, UnitDef, WorkerDef,
};

/// The kind registry's macro: from the list of kinds below, each with its def, it makes [`ObjectKind`],
/// [`ObjectKind::ALL`] and [`ObjectKind::def`], so that a new object type is one line in the list and a def module.
macro_rules! kind_registry {
    ($($(#[$doc:meta])* $kind:ident => $def:path,)+) => {
        /// An object type (`core/entities.md > Object Type`), one per type in the design, in the registry's order.
        #[derive(Copy, Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
        pub enum ObjectKind {
            $($(#[$doc])* $kind,)+
        }

        impl ObjectKind {
            /// Every object type, in declaration order, so tools such as the map editor's palette
            /// list new types automatically. A test checks that it stays complete.
            pub const ALL: &'static [ObjectKind] = &[$(ObjectKind::$kind,)+];

            /// The type's def: all its numbers, from its def module.
            pub fn def(self) -> &'static ObjectDef {
                // The sim's own unit tests may give a kind other numbers (`test_defs`).
                #[cfg(test)]
                if let Some(def) = test_defs::get(self) {
                    return def;
                }
                match self {
                    $(ObjectKind::$kind => &$def,)+
                }
            }
        }
    };
}

// The kind registry: every object type, each with the def that holds its numbers. New kinds go last: a kind's place
// in this list is part of the state hash (`hash.rs`), and its name is how saves, replays and maps store it.
kind_registry! {
    /// `core/map.md > SpaceCrystalsPatch`
    CrystalPatch => map::crystal_patch::CRYSTAL_PATCH,
    /// `core/map.md > SupplyDeliveryStation`
    SupplyDeliveryStation => map::supply_delivery_station::SUPPLY_DELIVERY_STATION,
    /// `factions/gdo/units/peacekeeper.md`
    Peacekeeper => gdo::units::peacekeeper::PEACEKEEPER,
    /// `factions/syndicate/units/guard.md`
    Guard => syndicate::units::guard::GUARD,
    /// `factions/gdo/structures/deployment_center.md`
    DeploymentCenter => gdo::structures::deployment_center::DEPLOYMENT_CENTER,
    /// `factions/gdo/economy.md > Generator`
    Generator => gdo::structures::generator::GENERATOR,
    /// `factions/gdo/structures/barracks.md`
    Barracks => gdo::structures::barracks::BARRACKS,
    /// `factions/gdo/economy.md > ExtractionFacility`
    ExtractionFacility => gdo::structures::extraction_facility::EXTRACTION_FACILITY,
    /// `factions/gdo/economy.md > ExtractionPlate`
    ExtractionPlate => gdo::structures::extraction_plate::EXTRACTION_PLATE,
    /// `factions/colonists/units/drone.md`
    Drone => colonists::units::drone::DRONE,
    /// `factions/colonists/economy.md > Beacon`
    Beacon => colonists::structures::beacon::BEACON,
    /// `factions/colonists/economy.md > Egress Node`
    EgressNode => colonists::structures::egress_node::EGRESS_NODE,
    /// `factions/colonists/structures/robo_portal.md`
    RoboPortal => colonists::structures::robo_portal::ROBO_PORTAL,
    /// `factions/colonists/structures/bio_portal.md`
    BioPortal => colonists::structures::bio_portal::BIO_PORTAL,
    /// `factions/colonists/units/sabre_legionnaire.md`
    SabreLegionnaire => colonists::units::sabre_legionnaire::SABRE_LEGIONNAIRE,
    /// `factions/colonists/units/rifle_legionnaire.md`
    RifleLegionnaire => colonists::units::rifle_legionnaire::RIFLE_LEGIONNAIRE,
    /// `factions/gdo/units/peacekeeper.md > Riot Shield > Two unit types` (design change 0173): the Peacekeeper
    /// with its shield up
    PeacekeeperShielded => gdo::units::peacekeeper::PEACEKEEPER_SHIELDED,
    /// `factions/syndicate/units/agent.md`
    Agent => syndicate::units::agent::AGENT,
    /// `factions/syndicate/tunnels.md > Tunnel`, at any of its tiers (plan decision T39)
    Tunnel => syndicate::structures::tunnel::TUNNEL,
    /// `factions/syndicate/structures/hideout.md`
    Hideout => syndicate::structures::hideout::HIDEOUT,
    /// `factions/syndicate/structures/optics_lab.md` (design change 0229)
    OpticsLab => syndicate::structures::optics_lab::OPTICS_LAB,
    /// `factions/syndicate/structures/chemical_plant.md` (design change 0260)
    ChemicalPlant => syndicate::structures::chemical_plant::CHEMICAL_PLANT,
    /// `factions/syndicate/units/grenadier.md` (design change 0260)
    Grenadier => syndicate::units::grenadier::GRENADIER,
    /// `factions/syndicate/structures/laser_turret.md` (design change 0260)
    LaserTurret => syndicate::structures::laser_turret::LASER_TURRET,
    /// `factions/cults/units/recruit.md`
    Recruit => cults::units::recruit::RECRUIT,
    /// `factions/cults/units/militant.md` (design change 0302)
    Militant => cults::units::militant::MILITANT,
    /// `factions/cults/units/gunner.md` (design change 0304)
    Gunner => cults::units::gunner::GUNNER,
    /// `factions/cults/structures/recruitment_hall.md`
    RecruitmentHall => cults::structures::recruitment_hall::RECRUITMENT_HALL,
    /// `factions/cults/structures/storage.md`
    Storage => cults::structures::storage::STORAGE,
    /// `factions/cults/structures/armory.md`
    Armory => cults::structures::armory::ARMORY,
    /// `factions/cults/structures/gunworks.md` (slice M5-G2: the Cults' other kinds, in the tech tree's order, then
    /// design changes 0361 to 0365's two vehicles; plan decision T69)
    Gunworks => cults::structures::gunworks::GUNWORKS,
    /// `factions/cults/structures/garrison.md`
    Garrison => cults::structures::garrison::GARRISON,
    /// `factions/cults/structures/chop_shop.md`
    ChopShop => cults::structures::chop_shop::CHOP_SHOP,
    /// `factions/cults/structures/machine_shop.md`
    MachineShop => cults::structures::machine_shop::MACHINE_SHOP,
    /// `factions/cults/structures/airstrip.md`
    Airstrip => cults::structures::airstrip::AIRSTRIP,
    /// `factions/cults/structures/scrap_hangar.md`
    ScrapHangar => cults::structures::scrap_hangar::SCRAP_HANGAR,
    /// `factions/cults/augmentation.md > Altar`
    Altar => cults::structures::altar::ALTAR,
    /// `factions/cults/structures/temple.md`
    Temple => cults::structures::temple::TEMPLE,
    /// `factions/cults/structures/cathedral.md`
    Cathedral => cults::structures::cathedral::CATHEDRAL,
    /// `factions/cults/structures/shrine.md`
    Shrine => cults::structures::shrine::SHRINE,
    /// `factions/cults/units/disciple.md`
    Disciple => cults::units::disciple::DISCIPLE,
    /// `factions/cults/units/seeker.md` (design change 0364: the Jackal renamed)
    Seeker => cults::units::seeker::SEEKER,
    /// `factions/cults/units/sparrow.md`
    Sparrow => cults::units::sparrow::SPARROW,
    /// `factions/cults/units/hand_of_heaven.md` (design changes 0361 and 0363)
    HandOfHeaven => cults::units::hand_of_heaven::HAND_OF_HEAVEN,
    /// `factions/cults/units/prayer_mobile.md` (design changes 0364 and 0365)
    PrayerMobile => cults::units::prayer_mobile::PRAYER_MOBILE,
    /// `factions/gdo/structures/career_campus.md` (slice M5-N9: design change 0328's two placeholders, the Field
    /// Engineer's prerequisite first)
    CareerCampus => gdo::structures::career_campus::CAREER_CAMPUS,
    /// `factions/gdo/units/field_engineer.md`
    FieldEngineer => gdo::units::field_engineer::FIELD_ENGINEER,
}

#[derive(Copy, Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum Category {
    Unit,
    Structure,
    /// `core/map.md > Resource`: indestructible, unowned, no vision.
    Resource,
}

#[derive(Copy, Clone, Debug, PartialEq, Serialize, Deserialize)]
pub enum Footprint {
    /// Structures and resources: whole tiles, `w` wide and `h` tall in the default orientation (`Size - Height x
    /// Width`, given for the default orientation: `core/entities.md > Structure Type > Side lettering convention`).
    Tiles { w: u32, h: u32 },
    /// Units: silhouette rectangle in space units (`systems/units.md > Unit > Silhouette`).
    Silhouette { w: f32, h: f32 },
}

/// A name in the defs, `&'static str`. Spelt so that serde's derive doesn't borrow it from the text a def is read from,
/// as it does with every field it sees spelt `&str`: [`leak::str`] reads it instead.
type StaticStr = &'static str;

/// Attributes shared by every object type (`core/entities.md > Object Type`).
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct ObjectDef {
    pub kind: ObjectKind,
    #[serde(deserialize_with = "leak::str")]
    pub name: StaticStr,
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
    /// Every footprint tile must be flat (`core/map.md > Elevation > Structures only on flat terrain`, `core/map.md >
    /// Map resources only on flat terrain`).
    pub needs_flat_ground: bool,
    /// Units only: their base, movement, attack and abilities (`systems/units.md > Unit`).
    pub unit: Option<UnitDef>,
    /// Structures only: their symmetry, BuildRadiusExtension and Power (`core/entities.md > Structure Type`).
    pub structure: Option<StructureDef>,
    /// The per-object values newly placed objects of the type get (`core/map_editor.md > Per-object Resource
    /// Values`; the map editor, debug spawns: `SpawnSettings::defaults`): the map resources'; `None` for the types
    /// without such values.
    pub default_settings: Option<SpawnSettings>,
    /// (M5-N4a) Its energy pool, which its spells spend (plan decision T75): the Disciple's
    /// (`factions/cults/units/disciple.md > Energy`); `None` for the types without one. A unit's or a structure's alike,
    /// as `max_shields`. The current energy is `Object::energy` (`crate::energy`). Optional in files.
    #[serde(default)]
    pub energy: Option<EnergyDef>,
}

/// An object type's armour: `systems/units.md > Unit > PointArmor`, `systems/units.md > FullArmor` for units, and the
/// same two attributes in each structure type's stat block.
#[derive(Copy, Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
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

/// (M5-N4a) An object type's energy pool (plan decision T75, `ObjectDef::energy`), in points, as the design gives them
/// (the Disciple's "Energy - MaxEnergy 200; starts at 50; regenerates 0.5 per second": `factions/cults/units/disciple.md
/// > Placeholder stats`). The sim counts it in integers (`crate::energy`), each number here converting exactly.
#[derive(Copy, Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct EnergyDef {
    /// MaxEnergy: what it regenerates up to.
    pub max: f32,
    /// What an object of the type has as it comes into the game, or becomes of the type.
    pub start: f32,
    /// How it regenerates.
    pub regen: EnergyRegen,
}

/// (M5-N4a) How an energy pool regenerates (plan decision T75): every tick, up to its maximum (`crate::energy::step`).
#[derive(Copy, Clone, Debug, PartialEq, Serialize, Deserialize)]
pub enum EnergyRegen {
    /// So many points a second: the Disciple's 0.5 (`factions/cults/units/disciple.md > Energy`: "It regenerates
    /// naturally over time").
    PerSecond(f32),
    /// So many points a second for each unit inside the structure, none with none inside: the Shrine's 0.25 a Recruit
    /// (`factions/cults/structures/shrine.md > Energy`: "proportional to the number of Recruits inside [...] With no
    /// Recruits inside, the Shrine does not regenerate energy"), whose slice gives it its pool (M5-N5).
    PerUnitInside(f32),
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
    /// Whether the footprints of objects of `self` and `other` may cover the same tiles. Only an Extraction Plate
    /// and the Space Crystals Patch it stands on may (`factions/gdo/economy.md > ExtractionPlate`: "placed onto a
    /// Space Crystal Patch"); every other pair of tile-based objects is kept apart (`core/entities.md > Placement
    /// Validation > Direct Placement`: no structure overlap), structures and map resources included
    /// (`core/entities.md > All placement methods: no buildings on resources`, design change 0096), and an underground
    /// structure and a surface one too (design change 0240: "a surface structure and an underground expansion (any
    /// player's, including neutral ones) can never share a tile"; the placement methods of slices M4-E and M4-F follow
    /// it in both layers, the expansions' own in theirs). As two plates may not overlap, nor two
    /// patches, a patch holds one plate at most: a plate goes only onto an uncovered patch. Units have no
    /// footprint: this is about tile-based objects only. The pairs are those of `StructureDef::onto`, either way.
    pub fn may_overlap(self, other: ObjectKind) -> bool {
        let onto = |kind: ObjectKind| kind.def().structure.as_ref().and_then(|s| s.onto);
        onto(self) == Some(other) || onto(other) == Some(self)
    }
}

#[cfg(test)]
mod tests;
