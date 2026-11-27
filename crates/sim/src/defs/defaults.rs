//! The defs' struct-update defaults (plan decision T84, slice M6-GK): [`ObjectDef::NONE`], [`UnitDef::NONE`] and
//! [`StructureDef::NONE`]. Every def literal ends with `..ObjectDef::NONE` (`..UnitDef::NONE`, `..StructureDef::NONE`)
//! and lists only what its type has: a new optional field is one line in its struct and in its `NONE` here, plus the
//! literals of the kinds that have it. A literal made from another def ends with that def instead (the Peacekeeper
//! (Shielded), from the Peacekeeper's).
//!
//! Each `NONE` leaves its optional fields unset: `None`, an empty list, or `false` for a flag few types have. Its other
//! fields are placeholders that every literal sets, so that no type takes them from here: an object type's identity,
//! category, footprint, flags, sight and armour; a unit type's base and movement; a structure type's symmetry,
//! BuildRadiusExtension and Power.

use super::structures::SymmetryType;
use super::units::{MovementAttrs, UnitBase};
use super::{Armor, Category, Footprint, ObjectDef, ObjectKind, StructureDef, UnitDef};

impl ObjectDef {
    /// The struct-update default of every object type's literal (module docs): no faction, MaxHP or MaxShields, no
    /// unit or structure attributes, no per-object settings and no energy pool.
    pub(crate) const NONE: ObjectDef = ObjectDef {
        kind: ObjectKind::CrystalPatch,
        name: "",
        faction: None,
        category: Category::Resource,
        footprint: Footprint::Tiles { w: 1, h: 1 },
        selectable: false,
        destructible: false,
        groupable: false,
        sight_range: 0.0,
        max_hp: None,
        max_shields: None,
        armor: Armor::NONE,
        needs_flat_ground: false,
        unit: None,
        structure: None,
        default_settings: None,
        energy: None,
    };
}

impl UnitDef {
    /// The struct-update default of every unit type's attributes (module docs): no Rugged-terrain bonus, turret or
    /// attack, no Unit Control, Tunnel Space or Beacon Capacity cost, and none of the abilities of a few types.
    pub(crate) const NONE: UnitDef = UnitDef {
        base: UnitBase::LightInfantry,
        movement: MovementAttrs::TurnRate {
            turn_rate: 0.0,
            acceleration: None,
            deceleration: None,
            max_speed: 0.0,
        },
        rugged_terrain_defense_bonus: None,
        turret: None,
        attack: None,
        unit_control_cost: None,
        tunnel_space_cost: None,
        beacon_capacity_cost: None,
        riot_shield: None,
        conversion: None,
        deploy: None,
        worker: None,
        augmentable: None,
        repairs: None,
        spells: &[],
        separation_radius: None,
        target_lock: None,
        stock: None,
        tether: None,
        air_door_ticks: 0,
        ammo: None,
    };
}

impl StructureDef {
    /// The struct-update default of every structure type's attributes (module docs): it seeds no build area,
    /// produces, mines, constructs and warps in nothing, takes no drop-off, isn't underground, requires nothing, and has
    /// none of the parts of a few types (a Tunnel's, a site, an expansion's, an attack, a pop-up, a hold, research,
    /// abilities).
    pub(crate) const NONE: StructureDef = StructureDef {
        symmetry: SymmetryType::AAAA,
        build_radius_extension: 0,
        power: 0,
        seeds_build_area: false,
        production: None,
        mining: None,
        onto: None,
        constructs: &[],
        warp_in: &[],
        drop_off: None,
        underground: false,
        tunnel: None,
        site: None,
        expansion: None,
        requires: &[],
        attack: None,
        popup: None,
        holds: None,
        occupants_fire: None,
        research: None,
        abilities: None,
        airfield: None,
    };
}
