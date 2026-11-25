//! What the Colonists' stat blocks share, for their modules' tests (`units/`, `structures/`), and what the two
//! Legionnaire forms share. The rules every kind follows are in `defs_are_consistent` (`defs/tests.rs`).

use glam::Vec2;

use crate::defs::units::{AttackAttrs, MovementAttrs, UnitBase, UnitDef};
use crate::defs::{Armor, Category, MiningDef, ObjectKind, StructureDef, SymmetryType};
use crate::math::PI;
use crate::player::Faction;

/// A Colonists structure's stat block: size (w × h in the default orientation), SymmetryType, MaxHP, MaxShields,
/// PointArmor, FullArmor, SightRange (grid units) and Groupable. It is destructible and selectable, and has neither
/// Power nor a build area, nor mines, nor constructs like the GDO.
#[expect(clippy::too_many_arguments, reason = "one argument per line of the stat block")]
pub(super) fn assert_structure(
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

/// The physical profile the two Legionnaire forms share (`factions/colonists/units/weapon_conversion.md`:
/// "Silhouette, MaxHP, MaxShields, PointArmor, FullArmor, SightRange, UnitBase, UnitBaseAttributes,
/// BeaconCapacityCost"), as each of their stat blocks gives it. Their AttackAttributes.
pub(super) fn assert_legionnaire(kind: ObjectKind) -> AttackAttrs {
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

/// `factions/colonists/units/weapon_conversion.md`: "two forms of the same underlying unit, sharing an identical
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
