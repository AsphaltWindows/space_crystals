//! The Colonists' stat blocks against the design, one test per kind: `factions/colonists/units.md > Drone`, `> Sabre
//! Legionnaire`, `> Rifle Legionnaire`; `economy.md > Beacon`, `> Egress Node`; `structures.md > Robo Portal`, `> Bio
//! Portal`. The rules every kind follows are in `defs_are_consistent` (`defs/tests.rs`).

use glam::Vec2;

use super::*;
use crate::defs::{MiningDef, MovementModel};
use crate::units::gu;

/// A Colonists structure's stat block: size (w × h in the default orientation), SymmetryType, MaxHP, MaxShields,
/// PointArmor, FullArmor, SightRange (grid units) and Groupable. It is destructible and selectable, and has neither
/// Power nor a build area, nor mines, nor constructs like the GDO.
#[expect(clippy::too_many_arguments, reason = "one argument per line of the stat block")]
fn assert_structure(
    kind: ObjectKind,
    size: (u32, u32),
    symmetry: SymmetryType,
    hp: f32,
    shields: f32,
    armor: (f32, f32),
    sight: f32,
    groupable: bool,
) -> &'static StructureDef {
    let def = kind.def();
    let structure = def.structure.as_ref().unwrap();
    assert_eq!(
        (def.category, def.faction, def.tile_size(), structure.symmetry),
        (Category::Structure, Some(Faction::Colonists), Some(size), symmetry),
        "{kind:?}"
    );
    let (point, full) = armor;
    assert_eq!(
        (def.max_hp, def.max_shields, def.armor, def.sight_range),
        (Some(hp), Some(shields), Armor { point, full }, sight),
        "{kind:?}"
    );
    assert_eq!(
        (def.destructible, def.groupable, def.selectable),
        (true, groupable, true),
        "{kind:?}"
    );
    assert_eq!((structure.power, structure.build_radius_extension), (0, 0), "{kind:?}");
    assert_eq!(
        (structure.mining, structure.constructs),
        (None::<MiningDef>, &[][..]),
        "{kind:?}"
    );
    assert!(def.unit.is_none() && def.needs_flat_ground, "{kind:?}");
    structure
}

/// The physical profile the two Legionnaire forms share (`units.md > Weapon Conversion`: "Silhouette, MaxHP,
/// MaxShields, PointArmor, FullArmor, SightRange, UnitBase, UnitBaseAttributes, BeaconCapacityCost"), as each of
/// their stat blocks gives it. Their AttackAttributes.
fn assert_legionnaire(kind: ObjectKind) -> AttackAttrs {
    let def = kind.def();
    assert_eq!((def.category, def.faction), (Category::Unit, Some(Faction::Colonists)));
    // Silhouette - 36x36 space units (square)
    assert_eq!(def.silhouette(), Some(Vec2::new(36.0, 36.0)), "{kind:?}");
    // MaxHP - 70, MaxShields - 10, PointArmor - 0, FullArmor - 0, SightRange - 5 grid units
    assert_eq!(
        (def.max_hp, def.max_shields, def.armor, def.sight_range),
        (Some(70.0), Some(10.0), Armor { point: 0.0, full: 0.0 }, 5.0),
        "{kind:?}"
    );
    // Destructible - true, Groupable - true
    assert_eq!((def.destructible, def.groupable, def.selectable), (true, true, true));
    let unit = def.unit.as_ref().unwrap();
    // UnitBase - HeavyInfantry, BeaconCapacityCost - 2
    assert_eq!(
        (unit.base, unit.beacon_capacity_cost),
        (UnitBase::HeavyInfantry, Some(2))
    );
    // UnitBaseAttributes[HeavyInfantry]: TurnRate 180 degrees/frame, Acceleration and Deceleration infinite, MaxSpeed
    // 5 space units/frame.
    assert_eq!(
        unit.movement,
        MovementAttrs::TurnRate {
            turn_rate: PI,
            acceleration: None,
            deceleration: None,
            max_speed: 5.0,
        },
        "{kind:?}"
    );
    // TurretAttributes - None
    assert_eq!(unit.turret, None, "{kind:?}");
    assert!(def.structure.is_none());
    unit.attack.expect("a Legionnaire fights")
}

/// `factions/colonists/units.md > Drone`: unarmed, on the HoverVehicle base, with DragMovement attributes whose
/// equilibrium is 8 space units per frame (`movement::drag`, slice M3-G).
#[test]
fn the_drone_follows_the_design() {
    let def = ObjectKind::Drone.def();
    assert_eq!((def.category, def.faction), (Category::Unit, Some(Faction::Colonists)));
    // Silhouette - 36x36 space units (square)
    assert_eq!(def.silhouette(), Some(Vec2::new(36.0, 36.0)));
    // MaxHP - 20, MaxShields - 10, PointArmor - 0, FullArmor - 0, SightRange - 5 grid units
    assert_eq!(
        (def.max_hp, def.max_shields, def.armor, def.sight_range),
        (Some(20.0), Some(10.0), Armor { point: 0.0, full: 0.0 }, 5.0)
    );
    // Destructible - true, Groupable - true (design change 0125)
    assert_eq!((def.destructible, def.groupable, def.selectable), (true, true, true));
    let unit = def.unit.as_ref().unwrap();
    // UnitBase - HoverVehicle, BeaconCapacityCost - 1
    assert_eq!(
        (unit.base, unit.beacon_capacity_cost),
        (UnitBase::HoverVehicle, Some(1))
    );
    // UnitBaseAttributes[HoverVehicle]: TurnRate 45 degrees/frame, ForwardAcceleration 0.6 space units/frame²,
    // OmniDirectionalAcceleration 0.2 space units/frame², DragRatio 0.1 per frame.
    assert_eq!(
        unit.movement,
        MovementAttrs::Drag {
            forward_acceleration: 0.6,
            omni_directional_acceleration: 0.2,
            drag_ratio: 0.1,
            turn_rate: PI / 4.0,
        }
    );
    assert_eq!(unit.movement.model(), MovementModel::Drag);
    // "MaxSpeed equilibrium = (0.6 + 0.2) / 0.1 = 8 space units/frame": exactly, in f32 too.
    assert_eq!(unit.movement.max_speed(), 8.0);
    assert_eq!(unit.movement.turn_rate(), Some(PI / 4.0));
    // TurretAttributes - None, AttackAttributes - None (unarmed)
    assert_eq!((unit.turret, unit.attack), (None, None));
    assert!(def.structure.is_none());
}

/// `factions/colonists/units.md > Sabre Legionnaire`: the Legionnaire's melee form, whose "Range: melee (adjacent
/// contact)" is the placeholder reach of 8 su of design change 0149.
#[test]
fn the_sabre_legionnaire_follows_the_design() {
    let attack = assert_legionnaire(ObjectKind::SabreLegionnaire);
    assert_eq!(
        attack,
        AttackAttrs {
            attack_type: AttackType::FullyConnected(FullyConnectedSubtype::Melee),
            target_domain: TargetDomain::Ground,
            target_type: TargetType::SingleTarget,
            damage: 15.0,
            range: MELEE_CONTACT_GU,
            min_range: 0.0,
            projectile_speed: None,
            aim_ticks: 3,
            firing_ticks: 3,
            cooldown_ticks: 1,
            reload_ticks: 5,
        }
    );
    assert_eq!(gu(attack.range), 8.0, "8 space units, edge to edge");
}

/// `factions/colonists/units.md > Rifle Legionnaire`: the Legionnaire's ranged form.
#[test]
fn the_rifle_legionnaire_follows_the_design() {
    let attack = assert_legionnaire(ObjectKind::RifleLegionnaire);
    assert_eq!(
        attack,
        AttackAttrs {
            attack_type: AttackType::FullyConnected(FullyConnectedSubtype::Ranged),
            target_domain: TargetDomain::Ground,
            target_type: TargetType::SingleTarget,
            damage: 10.0,
            range: 4.0,
            min_range: 0.0,
            projectile_speed: None,
            aim_ticks: 3,
            firing_ticks: 2,
            cooldown_ticks: 7,
            reload_ticks: 4,
        }
    );
}

/// `factions/colonists/units.md > Weapon Conversion`: "two forms of the same underlying unit, sharing an identical
/// physical profile … and differing only in AttackAttributes".
#[test]
fn the_legionnaires_two_forms_differ_only_in_their_weapon() {
    let profile = |kind: ObjectKind| {
        let def = kind.def();
        let unit = def.unit.clone().unwrap();
        (
            (def.faction, def.category, def.footprint, def.selectable),
            (def.destructible, def.groupable, def.sight_range),
            (def.max_hp, def.max_shields, def.armor, def.needs_flat_ground),
            UnitDef { attack: None, ..unit },
        )
    };
    assert_eq!(
        profile(ObjectKind::SabreLegionnaire),
        profile(ObjectKind::RifleLegionnaire)
    );
    let attack = |kind: ObjectKind| kind.def().unit.as_ref().unwrap().attack;
    assert_ne!(
        attack(ObjectKind::SabreLegionnaire),
        attack(ObjectKind::RifleLegionnaire)
    );
}

/// `factions/colonists/economy.md > Beacon`, and its Warp In list (`> Warp In > Constructs List`): the three
/// buildings with a cost and a WarpInTime. Ungroupable (design change 0122: "Beacon Groupable true → false").
#[test]
fn the_beacon_follows_the_design() {
    let structure = assert_structure(
        ObjectKind::Beacon,
        (2, 2),
        SymmetryType::AAAA,
        120.0,
        10.0,
        (0.0, 0.0),
        4.0,
        false,
    );
    assert_eq!(structure.production, None);
    let recipe = |kind, cost, frames| Recipe { kind, cost, frames };
    assert_eq!(
        structure.warp_in,
        [
            recipe(ObjectKind::EgressNode, 75, 320),
            recipe(ObjectKind::RoboPortal, 250, 640), // design change 0167 (was 400)
            recipe(ObjectKind::BioPortal, 150, 480),
        ]
    );
    // The Beacon is the one type with a Warp In list.
    for &kind in ObjectKind::ALL {
        let warps_in = kind.def().structure.as_ref().is_some_and(|s| !s.warp_in.is_empty());
        assert_eq!(warps_in, kind == ObjectKind::Beacon, "{kind:?}");
    }
}

/// `factions/colonists/economy.md > Egress Node`.
#[test]
fn the_egress_node_follows_the_design() {
    let structure = assert_structure(
        ObjectKind::EgressNode,
        (2, 2),
        SymmetryType::AAAA,
        200.0,
        10.0,
        (0.0, 0.0),
        3.0,
        false,
    );
    assert_eq!((structure.production, structure.warp_in), (None, &[][..]));
}

/// `factions/colonists/structures.md > Robo Portal`: it produces Drones, out of any of its four sides (all A).
#[test]
fn the_robo_portal_follows_the_design() {
    let structure = assert_structure(
        ObjectKind::RoboPortal,
        (4, 4),
        SymmetryType::AAAA,
        600.0,
        20.0,
        (1.0, 16.0),
        4.0,
        true,
    );
    assert!(structure.warp_in.is_empty());
    // Produces: "Drone: 50 Space Crystals, 160 frames (10 seconds)" (the Scout's cost and time are TBD).
    let production = structure.production.unwrap();
    assert_eq!(
        (production.exits, production.recipes),
        (
            ExitRule::Nearest(&[SideLetter::A]),
            &[Recipe {
                kind: ObjectKind::Drone,
                cost: 50,
                frames: 160,
            }][..]
        )
    );
}

/// `factions/colonists/structures.md > Bio Portal`: it produces Sabre Legionnaires through one of its A sides
/// (`> Exit Behavior`), never Rifle Legionnaires ("Not directly produced").
#[test]
fn the_bio_portal_follows_the_design() {
    let structure = assert_structure(
        ObjectKind::BioPortal,
        (3, 3),
        SymmetryType::AABB,
        250.0,
        15.0,
        (1.0, 9.0),
        3.0,
        true,
    );
    assert!(structure.warp_in.is_empty());
    // Produces: "Sabre Legionnaire: 100 Space Crystals, 160 frames (10 seconds)" (the Lancer's and the psionic
    // unit's cost and time are TBD).
    let production = structure.production.unwrap();
    assert_eq!(
        (production.exits, production.recipes),
        (
            ExitRule::Nearest(&[SideLetter::A]),
            &[Recipe {
                kind: ObjectKind::SabreLegionnaire,
                cost: 100,
                frames: 160,
            }][..]
        )
    );
    for &kind in ObjectKind::ALL {
        let produced = kind.def().structure.as_ref().and_then(|s| s.production.as_ref());
        assert!(
            produced.is_none_or(|p| p.recipe(ObjectKind::RifleLegionnaire).is_none()),
            "{kind:?} produces Rifle Legionnaires"
        );
    }
}
