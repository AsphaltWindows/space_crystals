//! Static object type definitions (`core/entities.md > Object Type`).
//!
//! One `ObjectKind` per object type in the design; its numbers live in a `static`
//! [`ObjectDef`] whose comments cite the design section they come from. Faction
//! types get their own modules (`defs/gdo.rs`, `defs/syndicate.rs`, …); unit bases and the
//! attributes of unit types are in `defs/units.rs`, those of structure types in `defs/structures.rs`.

mod gdo;
mod structures;
mod syndicate;
mod units;

use glam::Vec2;
use serde::{Deserialize, Serialize};

use crate::player::Faction;

pub use gdo::RIOT_SHIELD_TOGGLE_TICKS;
pub use structures::{MiningDef, Recipe, StructureDef, SymmetryType};
pub use units::{
    AttackAttrs, AttackType, Domain, FullyConnectedSubtype, MovementAttrs, MovementModel, RiotShieldDef, TargetDomain,
    TargetType, TurretAttrs, UnitBase, UnitBaseDef, UnitDef,
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
    ];

    pub fn def(self) -> &'static ObjectDef {
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
        }
    }

    /// Whether the footprints of objects of `self` and `other` may cover the same tiles. Only an Extraction Plate
    /// and the Space Crystals Patch it stands on may (`factions/gdo/economy.md > ExtractionPlate`: "placed onto a
    /// Space Crystal Patch"); every other pair of tile-based objects is kept apart (`core/entities.md > Placement
    /// Validation > Direct Placement`: no structure overlap), structures and map resources included (`> All
    /// placement methods: no buildings on resources`, design change 0096). As two plates may not overlap, nor two
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
    armor: Armor::NONE,
    needs_flat_ground: true, // `core/map.md > Elevation > Map resources only on flat terrain`
    unit: None,
    structure: None,
};

#[cfg(test)]
mod tests {
    use super::*;
    use serde::Deserialize;
    use serde::de::IntoDeserializer;
    use serde::de::value::{Error as ValueError, U32Deserializer};

    /// Every variant, found through serde's derived `Deserialize`, which accepts a unit
    /// variant by its declaration index and rejects the first index past the last variant.
    fn declared_kinds() -> Vec<ObjectKind> {
        (0u32..)
            .map_while(|i| {
                let index: U32Deserializer<ValueError> = i.into_deserializer();
                ObjectKind::deserialize(index).ok()
            })
            .collect()
    }

    #[test]
    fn all_lists_every_kind_in_declaration_order() {
        assert!(!ObjectKind::ALL.is_empty());
        assert_eq!(ObjectKind::ALL, declared_kinds().as_slice());
    }

    #[test]
    fn defs_are_consistent() {
        for &kind in ObjectKind::ALL {
            let def = kind.def();
            let name = def.name;
            assert_eq!(def.kind, kind, "{name}: def of the wrong kind");
            assert!(!name.is_empty(), "{kind:?}: empty name");
            match def.footprint {
                Footprint::Tiles { w, h } => {
                    assert!((1..=16).contains(&w) && (1..=16).contains(&h), "{name}: {w}×{h} tiles");
                    assert_ne!(def.category, Category::Unit, "{name}: units have silhouettes");
                }
                Footprint::Silhouette { w, h } => {
                    assert!(
                        w.is_finite() && w > 0.0 && h.is_finite() && h > 0.0,
                        "{name}: {w}×{h} su"
                    );
                    assert_eq!(def.category, Category::Unit, "{name}: only units have silhouettes");
                }
            }
            assert!(
                def.sight_range.is_finite() && def.sight_range >= 0.0,
                "{name}: sight range"
            );
            match def.max_hp {
                Some(hp) => assert!(def.destructible && hp.is_finite() && hp > 0.0, "{name}: max HP {hp}"),
                None => assert!(!def.destructible, "{name}: destructible without max HP"),
            }
            let Armor { point, full } = def.armor;
            assert!(
                point.is_finite() && point >= 0.0 && full.is_finite() && full >= 0.0,
                "{name}: armour {point} / {full}"
            );
            if def.category == Category::Resource {
                // `core/map.md > Resource`: indestructible, unowned (no faction), no vision.
                assert!(
                    !def.destructible && def.max_hp.is_none(),
                    "{name}: resources are indestructible"
                );
                assert_eq!(def.faction, None, "{name}: resources belong to no faction");
                assert_eq!(def.sight_range, 0.0, "{name}: resources provide no vision");
                assert_eq!(def.armor, Armor::NONE, "{name}: resources have no armour");
            }
            // Exactly the structures have structure attributes. Every structure so far is a destructible building
            // of a faction, on whole tiles of flat ground (`core/map.md > Elevation > Structures only on flat
            // terrain`), whose footprint its symmetry type allows (`core/entities.md > Structure Type`: only ABAB,
            // ABAC and ABCD may be oblong).
            assert_eq!(
                def.structure.is_some(),
                def.category == Category::Structure,
                "{name}: structure attributes"
            );
            if let Some(structure) = &def.structure {
                let (w, h) = def.tile_size().expect("structures are tile-based");
                assert!(
                    w == h || structure.symmetry.allows_oblong(),
                    "{name}: {w}×{h} with {:?}",
                    structure.symmetry
                );
                assert!(def.destructible && def.max_hp.is_some(), "{name}: destructible");
                assert!(def.faction.is_some(), "{name}: structures belong to a faction");
                assert!(def.needs_flat_ground, "{name}: structures stand on flat ground");
                assert!(def.selectable, "{name}: selectable");
                // `Constructs`: buildings of the constructor's own faction, each listed once, each taking time, and
                // short enough for their progress in fixed point (`crate::construction`).
                for (i, recipe) in structure.constructs.iter().enumerate() {
                    let built = recipe.kind.def();
                    assert!(
                        built.structure.is_some() && built.faction == def.faction,
                        "{name} constructs {:?}",
                        recipe.kind
                    );
                    let longest = u32::MAX / crate::economy::RATIO_ONE;
                    assert!((1..=longest).contains(&recipe.ticks), "{name}: {recipe:?}");
                    assert!(
                        structure.constructs[..i].iter().all(|r| r.kind != recipe.kind),
                        "{name} lists {:?} twice",
                        recipe.kind
                    );
                }
            }
            // Exactly the units have unit attributes, and their movement attributes are their base's model.
            assert_eq!(
                def.unit.is_some(),
                def.category == Category::Unit,
                "{name}: unit attributes"
            );
            if let Some(unit) = &def.unit {
                let base = unit.base_def();
                assert_eq!(unit.movement.model(), base.movement_model, "{name}: movement model");
                let speed = unit.movement.max_speed();
                assert!(speed.is_finite() && speed > 0.0, "{name}: max speed {speed}");
                assert_eq!(unit.turret.is_some(), base.has_turret, "{name}: turret");
                assert_eq!(
                    unit.rugged_terrain_defense_bonus.is_some(),
                    unit.base == UnitBase::LightInfantry,
                    "{name}: RuggedTerrainDefenseBonus belongs to LightInfantry"
                );
                assert!(def.faction.is_some(), "{name}: units belong to a faction");
                assert!(def.half_extents().is_some(), "{name}: units have a collision box");
            }
        }
    }

    /// The stat blocks of the GDO structures: `factions/gdo/structures.md > DeploymentCenter`, `> Barracks`, and
    /// `factions/gdo/economy.md > Generator`, `> ExtractionFacility`, `> ExtractionPlate`.
    #[test]
    fn the_gdo_structures_follow_the_design() {
        use ObjectKind as K;
        use SymmetryType as S;
        // Kind, size (w × h in the default orientation), symmetry, MaxHP, PointArmor, FullArmor, SightRange,
        // Groupable, BuildRadiusExtension, Power.
        let expected = [
            (
                K::DeploymentCenter,
                (4, 4),
                S::AAAA,
                1000.0,
                1.0,
                16.0,
                6.0,
                false,
                12,
                30,
            ),
            (K::Generator, (2, 2), S::AAAA, 350.0, 1.0, 4.0, 3.0, true, 1, 20),
            // `> Barracks > Size - 2x3 grid units (2 tall, 3 wide in the default orientation)` (design change 0079).
            (K::Barracks, (3, 2), S::ABAC, 300.0, 1.0, 6.0, 4.0, true, 2, -30),
            (
                K::ExtractionFacility,
                (3, 3),
                S::AAAA,
                500.0,
                1.0,
                9.0,
                3.0,
                false,
                2,
                -15,
            ),
            (K::ExtractionPlate, (1, 1), S::AAAA, 85.0, 2.0, 2.0, 0.0, true, 0, -3),
        ];
        for (kind, size, symmetry, hp, point, full, sight, groupable, extension, power) in expected {
            let def = kind.def();
            let structure = def.structure.as_ref().unwrap();
            assert_eq!(
                (def.category, def.faction, def.tile_size(), structure.symmetry),
                (Category::Structure, Some(Faction::Gdo), Some(size), symmetry),
                "{kind:?}"
            );
            assert_eq!(
                (def.max_hp, def.armor, def.sight_range),
                (Some(hp), Armor { point, full }, sight),
                "{kind:?}"
            );
            assert_eq!(
                (def.destructible, def.groupable, def.selectable),
                (true, groupable, true),
                "{kind:?}"
            );
            assert_eq!(
                (structure.build_radius_extension, structure.power),
                (extension, power),
                "{kind:?}"
            );
            assert!(def.unit.is_none() && def.needs_flat_ground, "{kind:?}");
        }
        // `> ExtractionPlate > MiningRate - 10 Space Crystals per 48 frames`, `> ResidualMiningRate - 1 Space
        // Crystal per 48 frames`: the plate is the one structure that mines.
        for &kind in ObjectKind::ALL {
            let mining = kind.def().structure.as_ref().and_then(|s| s.mining);
            let expected = (kind == K::ExtractionPlate).then_some(MiningDef {
                load: 10,
                residual: 1,
                cycle_ticks: 48,
            });
            assert_eq!(mining, expected, "{kind:?}");
        }
        // `factions/gdo/structures.md > DeploymentCenter > Constructs` and `factions/gdo/economy.md >
        // ExtractionFacility > Constructs`: the two constructors, with each building's cost and time.
        let recipe = |kind, crystals, ticks| Recipe { kind, crystals, ticks };
        for &kind in ObjectKind::ALL {
            let constructs = kind.def().structure.as_ref().map_or(&[][..], |s| s.constructs);
            let expected = match kind {
                K::DeploymentCenter => vec![
                    recipe(K::Generator, 100, 240),
                    recipe(K::Barracks, 200, 160),
                    recipe(K::ExtractionFacility, 200, 320),
                ],
                K::ExtractionFacility => vec![recipe(K::ExtractionPlate, 75, 400)],
                _ => vec![],
            };
            assert_eq!(constructs, expected.as_slice(), "{kind:?}");
        }
    }

    /// Only a plate and a patch share tiles, in either order.
    #[test]
    fn only_a_plate_and_a_patch_may_overlap() {
        for &a in ObjectKind::ALL {
            for &b in ObjectKind::ALL {
                let pair = [a, b];
                let plate_on_patch =
                    pair.contains(&ObjectKind::ExtractionPlate) && pair.contains(&ObjectKind::CrystalPatch);
                assert_eq!(a.may_overlap(b), plate_on_patch, "{a:?} and {b:?}");
            }
        }
    }

    /// `factions/gdo/units.md > Peacekeeper`, `factions/syndicate/units.md > Guard`: the M1 units.
    #[test]
    fn the_m1_units_follow_the_design() {
        let peacekeeper = ObjectKind::Peacekeeper.def();
        let guard = ObjectKind::Guard.def();
        assert_eq!(peacekeeper.silhouette(), Some(Vec2::new(24.0, 24.0)));
        assert_eq!(guard.half_extents(), Some(Vec2::new(18.0, 18.0)));
        assert_eq!((peacekeeper.max_hp, guard.max_hp), (Some(50.0), Some(80.0)));
        let one = Armor { point: 1.0, full: 1.0 };
        assert_eq!(
            (peacekeeper.armor, guard.armor),
            (one, one),
            "PointArmor - 1, FullArmor - 1"
        );
        assert_eq!(
            (peacekeeper.faction, guard.faction),
            (Some(Faction::Gdo), Some(Faction::Syndicate))
        );
        let (pk, gd) = (peacekeeper.unit.as_ref().unwrap(), guard.unit.as_ref().unwrap());
        assert_eq!((pk.base, gd.base), (UnitBase::LightInfantry, UnitBase::HeavyInfantry));
        // Only TurnRateMovement exists in M1: 180°/tick, so every turn completes within a tick.
        for unit in [pk, gd] {
            let MovementAttrs::TurnRate {
                turn_rate,
                acceleration,
                deceleration,
                ..
            } = unit.movement
            else {
                panic!("{:?} is not TurnRateMovement", unit.base);
            };
            assert_eq!(turn_rate, crate::math::PI);
            assert_eq!((acceleration, deceleration), (None, None), "infinite");
        }
        assert_eq!((pk.movement.max_speed(), gd.movement.max_speed()), (4.0, 5.0));
        let (pk_attack, gd_attack) = (pk.attack.unwrap(), gd.attack.unwrap());
        assert_eq!(
            (pk_attack.damage, pk_attack.range, pk_attack.reload_ticks),
            (10.0, 4.0, 12)
        );
        assert_eq!(
            (gd_attack.damage, gd_attack.range, gd_attack.reload_ticks),
            (6.0, 3.0, 4)
        );
        let shield = pk.riot_shield.unwrap();
        assert_eq!((shield.speed_factor, shield.damage_factor), (0.5, 0.5));
        assert!(shield.toggle_ticks < 8, "under half a second (8 ticks)");
        assert_eq!((pk.unit_control_cost, gd.tunnel_space_cost), (Some(1), Some(1)));
    }
}
