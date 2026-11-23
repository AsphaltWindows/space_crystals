//! Static object type definitions (`core/entities.md > Object Type`).
//!
//! One `ObjectKind` per object type in the design; its numbers live in a `static`
//! [`ObjectDef`] whose comments cite the design section they come from. Faction
//! types get their own modules (`defs/gdo.rs`, `defs/syndicate.rs`, …); unit bases and the
//! attributes of unit types are in `defs/units.rs`.

mod gdo;
mod syndicate;
mod units;

use glam::Vec2;
use serde::{Deserialize, Serialize};

use crate::player::Faction;

pub use gdo::RIOT_SHIELD_TOGGLE_TICKS;
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
    /// Structures and resources: whole tiles, unrotated `w × h` (`Size - Height x Width`).
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
    /// Every footprint tile must be flat (`core/map.md > Elevation > Map resources only on flat terrain`).
    pub needs_flat_ground: bool,
    /// Units only: their base, movement, armour and attack (`systems/units.md > Unit`).
    pub unit: Option<UnitDef>,
}

impl ObjectDef {
    /// A unit's Silhouette, space units (`systems/units.md > Unit > Silhouette`); `None` for tile-based types.
    pub fn silhouette(&self) -> Option<Vec2> {
        match self.footprint {
            Footprint::Silhouette { w, h } => Some(Vec2::new(w, h)),
            Footprint::Tiles { .. } => None,
        }
    }

    /// Half a unit's Silhouette: the half extents of its collision box, axis-aligned and never rotated
    /// (`docs/movement.md > 3`; question Q0009, gap G6). `None` for tile-based types.
    pub fn half_extents(&self) -> Option<Vec2> {
        self.silhouette().map(|s| s / 2.0)
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
    ];

    pub fn def(self) -> &'static ObjectDef {
        match self {
            ObjectKind::CrystalPatch => &CRYSTAL_PATCH,
            ObjectKind::SupplyDeliveryStation => &SUPPLY_DELIVERY_STATION,
            ObjectKind::Peacekeeper => &gdo::PEACEKEEPER,
            ObjectKind::Guard => &syndicate::GUARD,
        }
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
    needs_flat_ground: true, // `core/map.md > Elevation > Map resources only on flat terrain`
    unit: None,
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
    needs_flat_ground: true, // `core/map.md > Elevation > Map resources only on flat terrain`
    unit: None,
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
            if def.category == Category::Resource {
                // `core/map.md > Resource`: indestructible, unowned (no faction), no vision.
                assert!(
                    !def.destructible && def.max_hp.is_none(),
                    "{name}: resources are indestructible"
                );
                assert_eq!(def.faction, None, "{name}: resources belong to no faction");
                assert_eq!(def.sight_range, 0.0, "{name}: resources provide no vision");
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

    /// `factions/gdo/units.md > Peacekeeper`, `factions/syndicate/units.md > Guard`: the M1 units.
    #[test]
    fn the_m1_units_follow_the_design() {
        let peacekeeper = ObjectKind::Peacekeeper.def();
        let guard = ObjectKind::Guard.def();
        assert_eq!(peacekeeper.silhouette(), Some(Vec2::new(24.0, 24.0)));
        assert_eq!(guard.half_extents(), Some(Vec2::new(18.0, 18.0)));
        assert_eq!((peacekeeper.max_hp, guard.max_hp), (Some(50.0), Some(80.0)));
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
