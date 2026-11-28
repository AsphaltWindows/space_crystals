//! The Shadebringer — `factions/syndicate/units/shadebringer.md` (design change 0367's placeholders; slice M6-SK: its
//! numbers; slice M6-SB: the Vehicle Assembly trains it, and its Sludge).

use crate::defs::units::{MovementAttrs, PatchKind, SlowDef, TankDef, UnitBase, UnitDef};
use crate::defs::{Armor, Category, Footprint, ObjectDef, ObjectKind};
use crate::player::Faction;

/// `factions/syndicate/units/shadebringer.md`: the "Syndicate **support vehicle**. Wheeled. **No attack and no
/// turret.**", with its two toggles, Sludge and Cover. It drives by its FixedTurnRadiusMovement as any wheeled unit
/// does; since slice M6-SB the Vehicle Assembly trains it behind a Vault (`SHADEBRINGER_RECIPE`), and it carries its
/// Sludge's tank ([`SLUDGE_TANK`], `crate::patches`: `> Sludge (toggle)`, `> Charges and recharging`); slice M6-SC adds
/// its Cover's (`> Cover (toggle)`). Its numbers are the design's "Placeholder stats".
pub(crate) const SHADEBRINGER: ObjectDef = ObjectDef {
    kind: ObjectKind::Shadebringer,
    name: "Shadebringer",
    faction: Some(Faction::Syndicate), // Faction - TheSyndicate
    category: Category::Unit,          // Entity Type - Unit
    // "Silhouette 56x40 space units": its length along its facing, turned with the facing, as every vehicle's
    // (`ObjectDef::silhouette_turns`).
    footprint: Footprint::Silhouette { w: 56.0, h: 40.0 },
    selectable: true, // `core/entities.md > Object Type > Selectable - true`
    // Not listed; a type with MaxHP is destructible (`core/entities.md > Object Instance > HP`).
    destructible: true,
    groupable: true,     // Groupable - true (assumed)
    sight_range: 6.0,    // SightRange 6 grid units
    max_hp: Some(160.0), // MaxHP 160
    armor: Armor {
        point: 2.0, // PointArmor 2
        full: 2.0,  // FullArmor 2
    },
    needs_flat_ground: false,
    unit: Some(UnitDef {
        base: UnitBase::WheeledVehicle, // UnitBase - WheeledVehicle (exits only through Tier 2+ Tunnels)
        // "FixedTurnRadiusMovement: MinimumTurnRadius 80 su; ForwardMaxSpeed 8 su/frame; ForwardAcceleration 1;
        // ReverseMaxSpeed 3; ReverseAcceleration 0.5; Deceleration 1."
        movement: MovementAttrs::FixedTurnRadius {
            minimum_turn_radius: 80.0,
            forward_acceleration: Some(1.0),
            forward_max_speed: 8.0,
            reverse_acceleration: Some(0.5),
            reverse_max_speed: 3.0,
            deceleration: Some(1.0),
        },
        rugged_terrain_defense_bonus: None, // only LightInfantry has it
        turret: None,                       // "No attack and no turret."
        attack: None,                       // AttackAttributes - none
        tunnel_space_cost: Some(2),         // TunnelSpaceCost 2
        // "Sludge and Cover each have their **own, separate charge**": Sludge's (slice M6-SB).
        tanks: &[SLUDGE_TANK],
        ..UnitDef::NONE
    }),
    ..ObjectDef::NONE
};

/// (M6-SB) Its Sludge's tank (`UnitDef::tanks`, `crate::patches`): `> Placeholder stats`, "**Sludge**: each patch lasts
/// **X = 10 s**; units are slowed by **Y = 40%** for **Z = 2 s** after leaving it, and the slow **refreshes while they
/// stay in it**; trail width 48 su", and "**Tanks**: 100 each; drain **10 per second** while on (10 s of use from
/// full); refill **5 per second** inside the network with a Vault"; the bases it slows, `> Sludge (toggle)`'s.
pub(crate) const SLUDGE_TANK: TankDef = TankDef {
    lays: PatchKind::Sludge,
    capacity: 100.0, // "Tanks: 100 each"
    drain: 10.0,     // "drain 10 per second while on"
    refill: 5.0,     // "refill 5 per second inside the network with a Vault"
    life: 10.0,      // "each patch lasts X = 10 s"
    width: 48.0,     // "trail width 48 su"
    slow: Some(SlowDef {
        by: 0.4,    // "slowed by Y = 40%"
        after: 2.0, // "for Z = 2 s after leaving it"
        // "**Infantry and non-hover vehicles** that step into sludge are **slowed**"; "**Mechs** are slowed like
        // vehicles"; "**Drill Units** are slowed while on the surface" (while on the ground: `patches::slows`). "Not
        // affected: hover units (HoverVehicle, HoverCraft) and air units".
        bases: &[
            UnitBase::LightInfantry,
            UnitBase::HeavyInfantry,
            UnitBase::WheeledVehicle,
            UnitBase::TrackedVehicle,
            UnitBase::DrillUnit,
            UnitBase::Mech,
        ],
    }),
};

#[cfg(test)]
mod tests {
    use glam::Vec2;

    use super::*;

    /// A 56×40 WheeledVehicle, MaxHP 160, PointArmor 2, FullArmor 2, SightRange 6, Groupable ("true (assumed)"),
    /// TunnelSpaceCost 2; FixedTurnRadiusMovement at 8 su a frame forward and 3 in reverse, on a turning radius of 80
    /// su; no turret and no attack; (M6-SB) its Sludge's tank of 100, draining 10 a second and refilled at 5, its
    /// patches 48 su across for 10 s, slowing the ground bases but the hover ones by 40% for 2 s after.
    #[test]
    fn the_shadebringer_follows_the_design() {
        let def = ObjectKind::Shadebringer.def();
        assert_eq!(
            (def.name, def.category, def.faction),
            ("Shadebringer", Category::Unit, Some(Faction::Syndicate))
        );
        assert_eq!(def.silhouette(), Some(Vec2::new(56.0, 40.0)));
        let two = Armor { point: 2.0, full: 2.0 };
        assert_eq!(
            (def.max_hp, def.max_shields, def.armor, def.sight_range),
            (Some(160.0), None, two, 6.0)
        );
        assert_eq!((def.selectable, def.destructible, def.groupable), (true, true, true));
        let unit = def.unit.as_ref().unwrap();
        assert_eq!(unit.base, UnitBase::WheeledVehicle);
        let movement = MovementAttrs::FixedTurnRadius {
            minimum_turn_radius: 80.0,
            forward_acceleration: Some(1.0),
            forward_max_speed: 8.0,
            reverse_acceleration: Some(0.5),
            reverse_max_speed: 3.0,
            deceleration: Some(1.0),
        };
        assert_eq!(unit.movement, movement);
        assert_eq!((unit.turret, unit.attack), (None, None), "no attack and no turret");
        assert_eq!(
            (
                unit.tunnel_space_cost,
                unit.unit_control_cost,
                unit.beacon_capacity_cost
            ),
            (Some(2), None, None)
        );
        assert_eq!((unit.worker, unit.deploy, unit.separation_radius), (None, None, None));
        let sludge = unit.tank(PatchKind::Sludge).expect("its Sludge's tank");
        assert_eq!(
            (sludge.capacity, sludge.drain, sludge.refill, sludge.life, sludge.width),
            (100.0, 10.0, 5.0, 10.0, 48.0)
        );
        let slow = sludge.slow.expect("sludge slows");
        assert_eq!((slow.by, slow.after), (0.4, 2.0));
        let slowed: Vec<UnitBase> = (UnitBase::ALL.into_iter())
            .filter(|base| slow.bases.contains(base))
            .collect();
        let hover_or_air = [UnitBase::HoverVehicle, UnitBase::HoverCraft, UnitBase::Glider];
        let ground: Vec<UnitBase> = (UnitBase::ALL.into_iter())
            .filter(|base| !hover_or_air.contains(base))
            .collect();
        assert_eq!(slowed, ground, "infantry, non-hover vehicles, Mechs and Drill Units");
        assert_eq!(unit.tanks.len(), 1, "Cover's is slice M6-SC's");
        assert_eq!(unit.tank(PatchKind::Smoke), None);
    }
}
