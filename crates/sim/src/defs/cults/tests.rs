//! What the Cults' stat blocks share, for their modules' tests (`units/`, `structures/`), and the faction's Unit
//! Control costs (plan decision T55). The rules every kind follows are in `defs_are_consistent` (`defs/tests.rs`).

use glam::Vec2;

use crate::defs::units::{AttackAttrs, MovementAttrs, UnitBase};
use crate::defs::{Armor, Category, MiningDef, ObjectKind, StructureDef, SymmetryType};
use crate::math::PI;
use crate::player::Faction;

/// A unit's attack timings, `(AimDuration, FiringDuration, CooldownDuration, ReloadDuration)`, and the frames from one
/// attack to the next, their sum.
pub(super) fn timings(attack: &AttackAttrs) -> ((u32, u32, u32, u32), u32) {
    let t = (
        attack.aim_ticks,
        attack.firing_ticks,
        attack.cooldown_ticks,
        attack.reload_ticks,
    );
    (t, t.0 + t.1 + t.2 + t.3)
}

/// The body the Cults' three infantry units share, as each of their stat blocks gives it
/// (`factions/cults/units/recruit.md`, `factions/cults/units/militant.md`, `factions/cults/units/gunner.md`): a
/// LightInfantry unit, 24×24 space units, MaxHP 40, PointArmor 0, FullArmor 0, SightRange 5, Destructible and
/// Groupable, no turret, TurnRate 180 degrees a frame with infinite acceleration and deceleration, MaxSpeed 4,
/// RuggedTerrainDefenseBonus 50%; a Unit Control cost of 1 (plan decision T55), and none of the other factions' costs
/// nor abilities. Its AttackAttributes.
pub(super) fn assert_infantry(kind: ObjectKind) -> AttackAttrs {
    let def = kind.def();
    assert_eq!(
        (def.kind, def.category, def.faction, def.silhouette()),
        (kind, Category::Unit, Some(Faction::Cults), Some(Vec2::new(24.0, 24.0))),
        "{kind:?}"
    );
    assert_eq!(
        (def.max_hp, def.max_shields, def.armor, def.sight_range),
        (Some(40.0), None, Armor::NONE, 5.0),
        "{kind:?}"
    );
    assert_eq!(
        (def.selectable, def.destructible, def.groupable, def.needs_flat_ground),
        (true, true, true, false),
        "{kind:?}"
    );
    assert_eq!(
        (def.structure.is_none(), def.default_settings),
        (true, None),
        "{kind:?}"
    );
    let unit = def.unit.as_ref().unwrap();
    let movement = MovementAttrs::TurnRate {
        turn_rate: PI,
        acceleration: None,
        deceleration: None,
        max_speed: 4.0,
    };
    assert_eq!(
        (unit.base, unit.movement),
        (UnitBase::LightInfantry, movement),
        "{kind:?}"
    );
    assert_eq!(
        (unit.rugged_terrain_defense_bonus, unit.turret.is_none()),
        (Some(0.5), true),
        "{kind:?}"
    );
    assert_eq!(
        (
            unit.unit_control_cost,
            unit.tunnel_space_cost,
            unit.beacon_capacity_cost
        ),
        (Some(1), None, None),
        "{kind:?}"
    );
    assert!(
        unit.riot_shield.is_none() && unit.conversion.is_none() && unit.deploy.is_none(),
        "{kind:?}"
    );
    unit.attack.expect("every Cults infantry unit attacks")
}

/// A Cults structure's stat block: size (w × h in the default orientation), SymmetryType, MaxHP, PointArmor and
/// FullArmor, SightRange (grid units) and Groupable. It is a destructible, selectable building on flat ground, with
/// neither Power nor a build area (the Cults build with Recruits: `factions/cults/economy.md > Cults Building
/// Mechanics`), and none of the other factions' abilities: it mines nothing, constructs and warps in nothing, is no
/// Tunnel nor expansion, and neither attacks nor pops up (plan §3.3 A: "`attack` and `popup` ... are `None` on every
/// Cults structure for good"). The fields that the Cults' systems fill (`production`, `drop_off`, `site`) are each
/// kind's own test's.
pub(super) fn assert_structure(
    kind: ObjectKind,
    size: (u32, u32),
    symmetry: SymmetryType,
    hp: f32,
    armor: (f32, f32),
    sight: f32,
    groupable: bool,
) -> &'static StructureDef {
    let def = kind.def();
    let structure = def.structure.as_ref().unwrap();
    assert_eq!(
        (def.kind, def.category, def.faction, def.tile_size(), structure.symmetry),
        (kind, Category::Structure, Some(Faction::Cults), Some(size), symmetry),
        "{kind:?}"
    );
    let (point, full) = armor;
    assert_eq!(
        (def.max_hp, def.max_shields, def.armor, def.sight_range),
        (Some(hp), None, Armor { point, full }, sight),
        "{kind:?}"
    );
    assert_eq!(
        (def.destructible, def.groupable, def.selectable, def.needs_flat_ground),
        (true, groupable, true, true),
        "{kind:?}"
    );
    assert!(def.unit.is_none() && def.default_settings.is_none(), "{kind:?}");
    assert_eq!(
        (
            structure.power,
            structure.build_radius_extension,
            structure.seeds_build_area
        ),
        (0, 0, false),
        "{kind:?}"
    );
    assert_eq!(
        (
            structure.mining,
            structure.onto,
            structure.constructs,
            structure.warp_in
        ),
        (None::<MiningDef>, None, &[][..], &[][..]),
        "{kind:?}"
    );
    assert_eq!(
        (structure.underground, structure.tunnel, structure.expansion),
        (false, None, None),
        "{kind:?}"
    );
    assert_eq!((structure.attack, structure.popup), (None, None), "{kind:?}");
    structure
}

/// Plan decision T55 (`factions/cults/structures/recruitment_hall.md > Unit Control Tracking`: "A trained unit's Unit
/// Control cost equals the number of Recruits consumed to produce it"; `factions/cults/overview.md > TheCults`: "one
/// per Recruit consumed"): every Cults unit's Unit Control cost is the Recruits it takes. A Recruit is one; the
/// Militant and the Gunner are each trained from one (`factions/cults/structures/armory.md > Training`: an order
/// "reserves one Recruit", and "its reserved Recruit is consumed"). The table lists every Cults unit, so a new one
/// fails here until it is given its Recruits; slice M5-F2 checks it against the Armory's recipes once they exist.
#[test]
fn every_cults_unit_costs_the_recruits_it_takes() {
    let recruits = [
        (ObjectKind::Recruit, 1),
        (ObjectKind::Militant, 1),
        (ObjectKind::Gunner, 1),
    ];
    let units: Vec<ObjectKind> = (ObjectKind::ALL.iter().copied())
        .filter(|kind| kind.def().faction == Some(Faction::Cults) && kind.def().unit.is_some())
        .collect();
    assert_eq!(
        units,
        recruits.map(|(kind, _)| kind),
        "every Cults unit, in the registry's order"
    );
    for (kind, taken) in recruits {
        let cost = kind.def().unit.as_ref().and_then(|unit| unit.unit_control_cost);
        assert_eq!(cost, Some(taken), "{kind:?}");
    }
}

/// The faction's six designed kinds, after the M4 kinds, in the order plan §3.3 A gives them: Recruit, Militant, Gunner,
/// Recruitment Hall, Storage, Armory (each a stored name: `every_kind_keeps_its_place_and_name`).
#[test]
fn the_cults_kinds_are_the_six_designed_ones() {
    let cults: Vec<(ObjectKind, &str)> = (ObjectKind::ALL.iter())
        .filter(|kind| kind.def().faction == Some(Faction::Cults))
        .map(|&kind| (kind, kind.def().name))
        .collect();
    assert_eq!(
        cults,
        [
            (ObjectKind::Recruit, "Recruit"),
            (ObjectKind::Militant, "Militant"),
            (ObjectKind::Gunner, "Gunner"),
            (ObjectKind::RecruitmentHall, "Recruitment Hall"),
            (ObjectKind::Storage, "Storage"),
            (ObjectKind::Armory, "Armory"),
        ]
    );
}
