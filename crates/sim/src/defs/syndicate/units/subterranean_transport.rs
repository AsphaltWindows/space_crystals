//! The Subterranean Transport — `factions/syndicate/units/subterranean_transport.md` (design change 0367's
//! placeholders; slice M6-SK) — and its two unit types, above ground and burrowed (plan decision T84's
//! `SubterraneanTransportBurrowed`, as the Laser Battery's two and the Peacekeeper's are: `> Burrowing and surfacing
//! work like vehicle deploy`, "the unit changes mode only when the process completes").

use crate::defs::units::{MovementAttrs, UnitBase, UnitDef};
use crate::defs::{Armor, Category, Footprint, ObjectDef, ObjectKind};
use crate::player::Faction;

/// `factions/syndicate/units/subterranean_transport.md`, above ground: the "Syndicate **infantry transport** that
/// travels **underground**", which "**drives out of the Tunnel in its above-ground mode**, behaving as a tracked
/// vehicle". Its numbers only (slice M6-SK, plan decision T84, by T69's rule): nothing makes one yet. The Vehicle
/// Assembly trains it behind a Forge with slice M6-ST1, which brings its passengers and the surfaced type's own domain,
/// Ground (plan decision T94: until then both types have their DrillUnit base's, Underground), and with it its
/// crushing above ground (design change 0419: its base `crushes` nothing until then); slice M6-ST2 brings Burrow and
/// Surface. Its numbers are the design's "Placeholder stats".
pub(crate) const SUBTERRANEAN_TRANSPORT: ObjectDef = SUBTERRANEAN_TRANSPORT_DEF;

/// `> Burrow (ability)`: the Transport "underground", where "it can travel **under buildings and obstacles such as
/// cliffs**: across **all drillable terrain**" (its base's nav class, `NavClass::Underground`), the above-ground type's
/// in all but its speed: "underground MaxSpeed 3.5". Its other handling underground isn't given: **Placeholder (not in
/// the design: its Acceleration, Deceleration, TrackWidth and ReverseMaxSpeed underground)**, its above-ground type's.
/// Nothing makes one yet: burrowing, 48 frames each way, comes with slice M6-ST2.
pub(crate) const SUBTERRANEAN_TRANSPORT_BURROWED: ObjectDef = ObjectDef {
    kind: ObjectKind::SubterraneanTransportBurrowed,
    name: "Subterranean Transport (Burrowed)",
    unit: Some(UnitDef {
        movement: MovementAttrs::SpeedTurnRadius {
            track_width: 36.0,
            acceleration: Some(0.4),
            deceleration: Some(1.0),
            max_speed: 3.5, // underground MaxSpeed 3.5
            reverse_max_speed: 2.25,
        },
        ..SUBTERRANEAN_TRANSPORT_UNIT
    }),
    ..SUBTERRANEAN_TRANSPORT_DEF
};

/// `factions/syndicate/units/subterranean_transport.md`: what both of its types are.
const SUBTERRANEAN_TRANSPORT_DEF: ObjectDef = ObjectDef {
    kind: ObjectKind::SubterraneanTransport,
    name: "Subterranean Transport",
    faction: Some(Faction::Syndicate), // Faction - TheSyndicate
    category: Category::Unit,          // Entity Type - Unit
    // "Silhouette 72x48 space units": its length along its facing, turned with the facing, as every vehicle's
    // (`ObjectDef::silhouette_turns`).
    footprint: Footprint::Silhouette { w: 72.0, h: 48.0 },
    selectable: true, // `core/entities.md > Object Type > Selectable - true`
    // Not listed; a type with MaxHP is destructible (`core/entities.md > Object Instance > HP`).
    destructible: true,
    // Not listed: true, as for every unit whose commands are used all together, not one unit at a time
    // (`systems/controls/selection.md > SelectionGroup > No smart-casting`: "Groupable = true for units whose abilities
    // are normally used **all together**").
    groupable: true,
    sight_range: 5.0,    // SightRange 5 grid units
    max_hp: Some(300.0), // MaxHP 300
    armor: Armor {
        point: 3.0, // PointArmor 3
        full: 3.0,  // FullArmor 3
    },
    needs_flat_ground: false,
    unit: Some(SUBTERRANEAN_TRANSPORT_UNIT),
    ..ObjectDef::NONE
};

/// `factions/syndicate/units/subterranean_transport.md`: its unit attributes, above ground.
const SUBTERRANEAN_TRANSPORT_UNIT: UnitDef = UnitDef {
    // UnitBase - DrillUnit (confirmed, Q0071; exits only through Tier 2+ Tunnels like other vehicles)
    base: UnitBase::DrillUnit,
    // "Above ground as a TrackedVehicle: MaxSpeed 4.5 su/frame; Acceleration 0.4; Deceleration 1; TrackWidth 36 su;
    // ReverseMaxSpeed 2.25 (designer placeholders, 2026-11-26; Q0153: the Laser Battery's handling; spins about 14.3° a
    // frame)".
    movement: MovementAttrs::SpeedTurnRadius {
        track_width: 36.0,
        acceleration: Some(0.4),
        deceleration: Some(1.0),
        max_speed: 4.5,
        reverse_max_speed: 2.25,
    },
    rugged_terrain_defense_bonus: None, // only LightInfantry has it
    turret: None,                       // Not listed: it has no attack
    attack: None,                       // AttackAttributes - none (confirmed, Q0071)
    tunnel_space_cost: Some(3),         // TunnelSpaceCost 3
    ..UnitDef::NONE
};

#[cfg(test)]
mod tests {
    use glam::Vec2;

    use super::*;
    use crate::defs::Domain;

    /// A 72×48 DrillUnit, MaxHP 300, PointArmor 3, FullArmor 3, SightRange 5, Groupable (not listed: a unit used all
    /// together), TunnelSpaceCost 3, with no turret and no attack; above ground the Laser Battery's tracked handling on
    /// a 36 su track, spinning about 14.3° a frame (Q0153).
    #[test]
    fn the_subterranean_transport_follows_the_design() {
        let def = ObjectKind::SubterraneanTransport.def();
        assert_eq!(
            (def.name, def.category, def.faction),
            ("Subterranean Transport", Category::Unit, Some(Faction::Syndicate))
        );
        assert_eq!(def.silhouette(), Some(Vec2::new(72.0, 48.0)));
        let three = Armor { point: 3.0, full: 3.0 };
        assert_eq!(
            (def.max_hp, def.max_shields, def.armor, def.sight_range),
            (Some(300.0), None, three, 5.0)
        );
        assert_eq!((def.selectable, def.destructible, def.groupable), (true, true, true));
        let unit = def.unit.as_ref().unwrap();
        assert_eq!(unit.base, UnitBase::DrillUnit);
        // The base's domain until slice M6-ST1 gives the surfaced type its own (plan decision T94).
        assert_eq!(unit.base_def().domain, Domain::Underground);
        let movement = MovementAttrs::SpeedTurnRadius {
            track_width: 36.0,
            acceleration: Some(0.4),
            deceleration: Some(1.0),
            max_speed: 4.5,
            reverse_max_speed: 2.25,
        };
        assert_eq!(unit.movement, movement);
        let spin = unit.movement.spin_rate().expect("a tracked spin").to_degrees();
        assert!((spin - 14.3).abs() < 0.05, "spins about 14.3° a frame: {spin}");
        assert_eq!((unit.turret, unit.attack), (None, None), "AttackAttributes - none");
        assert_eq!(
            (
                unit.tunnel_space_cost,
                unit.unit_control_cost,
                unit.beacon_capacity_cost
            ),
            (Some(3), None, None)
        );
        assert_eq!((unit.worker, unit.deploy, unit.separation_radius), (None, None, None));
    }

    /// `> Placeholder stats`: "underground MaxSpeed 3.5", the above-ground type's in all else.
    #[test]
    fn burrowed_it_drives_at_three_and_a_half() {
        let (surfaced, burrowed) = (
            ObjectKind::SubterraneanTransport.def(),
            ObjectKind::SubterraneanTransportBurrowed.def(),
        );
        assert_eq!(burrowed.name, "Subterranean Transport (Burrowed)");
        assert_eq!(
            ObjectDef {
                kind: surfaced.kind,
                name: surfaced.name,
                unit: surfaced.unit.clone(),
                ..burrowed.clone()
            },
            *surfaced
        );
        let (surfaced, burrowed) = (surfaced.unit.as_ref().unwrap(), burrowed.unit.as_ref().unwrap());
        let underground = MovementAttrs::SpeedTurnRadius {
            track_width: 36.0,
            acceleration: Some(0.4),
            deceleration: Some(1.0),
            max_speed: 3.5,
            reverse_max_speed: 2.25,
        };
        assert_eq!(burrowed.movement, underground);
        assert_eq!(burrowed.base_def().domain, Domain::Underground);
        assert_eq!(
            UnitDef {
                movement: surfaced.movement,
                ..burrowed.clone()
            },
            *surfaced
        );
    }
}
