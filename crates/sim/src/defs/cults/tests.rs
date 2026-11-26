//! What the Cults' stat blocks share, for their modules' tests (`units/`, `structures/`), the faction's Unit Control
//! costs (plan decision T55) and the Recruits' Construct menus (slice M5-J1, design change 0327). The rules every kind
//! follows are in `defs_are_consistent` (`defs/tests.rs`).

use std::collections::BTreeSet;

use glam::Vec2;

use super::{CONSTRUCT_MENU, ConstructEntry, ConstructMenu};
use crate::construction::site::builds;
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
/// "reserves one Recruit", and "its reserved Recruit is consumed"), and so are the Disciple (its design file's
/// "consumes 1 Recruit; Unit Control cost 1", `factions/cults/units/disciple.md`) and the Sparrow ("consumes **1
/// Recruit**", `factions/cults/units/sparrow.md`); a Seeker takes two (`factions/cults/units/seeker.md`), a Hand of
/// Heaven three (`factions/cults/units/hand_of_heaven.md`) and a Prayer-mobile two
/// (`factions/cults/units/prayer_mobile.md`). The table lists every Cults unit, so a new one fails here until it is
/// given its Recruits; slice M5-F2 checks it against the Armory's recipes.
#[test]
fn every_cults_unit_costs_the_recruits_it_takes() {
    let recruits = [
        (ObjectKind::Recruit, 1),
        (ObjectKind::Militant, 1),
        (ObjectKind::Gunner, 1),
        (ObjectKind::Disciple, 1),
        (ObjectKind::Seeker, 2),
        (ObjectKind::Sparrow, 1),
        (ObjectKind::HandOfHeaven, 3),
        (ObjectKind::PrayerMobile, 2),
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

/// Every kind of the faction's design, after the M4 kinds (each a stored name: `every_kind_keeps_its_place_and_name`):
/// slice M5-A's six, in the order plan §3.3 A gives them, Recruit, Militant, Gunner, Recruitment Hall, Storage, Armory;
/// then slice M5-G2's (plan decision T69), the structures in the tech tree's order, the Tech/Weaponry branch, then the
/// Mystical one (`factions/cults/structures/README.md > Tech Tree Overview`), the Disciple, the Seeker (design change
/// 0364, once the Jackal) and the Sparrow, then the two vehicles of design changes 0361 to 0365, the Hand of Heaven and
/// the Prayer-mobile. The list is complete: every structure of the Tech Tree Overview's table, the Altar
/// (`factions/cults/augmentation.md > Altar`) among them, and every unit of `factions/cults/units/` has its kind, and
/// no other Cults kind exists.
#[test]
fn the_cults_kinds_are_every_designed_one() {
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
            (ObjectKind::Gunworks, "Gunworks"),
            (ObjectKind::Garrison, "Garrison"),
            (ObjectKind::ChopShop, "Chop Shop"),
            (ObjectKind::MachineShop, "Machine Shop"),
            (ObjectKind::Airstrip, "Airstrip"),
            (ObjectKind::ScrapHangar, "Scrap Hangar"),
            (ObjectKind::Altar, "Altar"),
            (ObjectKind::Temple, "Temple"),
            (ObjectKind::Cathedral, "Cathedral"),
            (ObjectKind::Shrine, "Shrine"),
            (ObjectKind::Disciple, "Disciple"),
            (ObjectKind::Seeker, "Seeker"),
            (ObjectKind::Sparrow, "Sparrow"),
            (ObjectKind::HandOfHeaven, "Hand of Heaven"),
            (ObjectKind::PrayerMobile, "Prayer-mobile"),
        ]
    );
    let named = |category: Category| -> BTreeSet<String> {
        (ObjectKind::ALL.iter())
            .filter(|kind| kind.def().faction == Some(Faction::Cults) && kind.def().category == category)
            .map(|kind| kind.def().name.to_owned())
            .collect()
    };
    assert_eq!(
        named(Category::Structure),
        designed_structures(),
        "the Tech Tree Overview's"
    );
    assert_eq!(
        named(Category::Unit),
        designed_units(),
        "the units of factions/cults/units/"
    );
}

/// The design folder of the Cults, from this crate.
const CULTS_DESIGN: &str = concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/../../artifacts/designer/design/factions/cults"
);

/// The structures of `factions/cults/structures/README.md > Tech Tree Overview`, by their names in its table (its
/// first column, "RecruitmentHall" being the Recruitment Hall).
fn designed_structures() -> BTreeSet<String> {
    let path = format!("{CULTS_DESIGN}/structures/README.md");
    let text = std::fs::read_to_string(&path).unwrap_or_else(|e| panic!("{path}: {e}"));
    let table = (text.lines())
        .skip_while(|line| !line.starts_with("| Structure |"))
        .skip(2)
        .take_while(|line| line.starts_with('|'));
    let names: BTreeSet<String> = table
        .map(|row| match row.split('|').nth(1).expect("a first column").trim() {
            "RecruitmentHall" => "Recruitment Hall".to_owned(),
            name => name.to_owned(),
        })
        .collect();
    assert!(names.len() > 1, "the Tech Tree Overview's table in {path}");
    names
}

/// The units of `factions/cults/units/`, one design file each, by their files' titles ("# Hand of Heaven").
fn designed_units() -> BTreeSet<String> {
    let folder = format!("{CULTS_DESIGN}/units");
    let files = std::fs::read_dir(&folder).unwrap_or_else(|e| panic!("{folder}: {e}"));
    let mut names = BTreeSet::new();
    for file in files {
        let path = file.expect("a design file").path();
        if path.file_name().is_some_and(|name| name == "README.md") {
            continue;
        }
        let text = std::fs::read_to_string(&path).unwrap_or_else(|e| panic!("{}: {e}", path.display()));
        let title = (text.lines().next())
            .and_then(|line| line.strip_prefix("# "))
            .unwrap_or_else(|| panic!("{}: a title", path.display()));
        names.insert(title.trim().to_owned());
    }
    names
}

/// The keys of the panel's grid, row by row (`systems/controls/interface.md > CommandPanel`).
const KEYS: [char; 12] = ['Q', 'W', 'E', 'R', 'A', 'S', 'D', 'F', 'Z', 'X', 'C', 'V'];

/// The line of a Construct menu in `factions/cults/units/recruit.md > Recruit ObjectInterfaceState > DefaultState
/// commands`, read from the design ("**BuildMenu[Basic]**: **Q** Recruitment Hall, **W** Storage; ..."): its keys and
/// their buildings' names, in the line's order.
fn designed_menu(menu: &str) -> Vec<(char, String)> {
    let path = concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/../../artifacts/designer/design/factions/cults/units/recruit.md"
    );
    let text = std::fs::read_to_string(path).unwrap_or_else(|e| panic!("{path}: {e}"));
    let marker = format!("- **BuildMenu[{menu}]**:");
    let line = (text.lines())
        .find(|line| line.trim_start().starts_with(&marker))
        .unwrap_or_else(|| panic!("{marker} in {path}"));
    // "  - ", "BuildMenu[Basic]", ": ", "Q", " Recruitment Hall, ", "W", " Storage; ", ...
    let parts: Vec<&str> = line.split("**").skip(3).collect();
    (parts.chunks(2))
        .map(|pair| {
            let key = pair[0].chars().next().expect("a key");
            let name = pair
                .get(1)
                .map_or("", |name| name.trim().trim_end_matches([',', ';', '.']));
            (key, name.to_owned())
        })
        .collect()
}

/// Design change 0327 (`factions/cults/units/recruit.md > Recruit ObjectInterfaceState > DefaultState commands`): each
/// Construct menu holds the design's buildings in the design's keys, and nothing else; no entry takes Z, which returns
/// to DefaultState, and none shares a slot.
#[test]
fn the_construct_menus_follow_the_design() {
    for (menu, name) in [(ConstructMenu::Basic, "Basic"), (ConstructMenu::Advanced, "Advanced")] {
        let listed: Vec<(char, String)> = (CONSTRUCT_MENU.iter())
            .filter(|slot| slot.menu == menu)
            .map(|slot| {
                assert!(slot.row < 3 && slot.column < 4, "{slot:?}");
                let key = KEYS[usize::from(slot.row) * 4 + usize::from(slot.column)];
                (key, slot.entry.name().to_owned())
            })
            .collect();
        assert_eq!(listed, designed_menu(name), "BuildMenu[{name}]");
        assert!(listed.iter().all(|&(key, _)| key != 'Z'), "Z returns");
    }
    let slots: BTreeSet<_> = (CONSTRUCT_MENU.iter())
        .map(|slot| (slot.menu, slot.row, slot.column))
        .collect();
    assert_eq!(slots.len(), CONSTRUCT_MENU.len(), "a slot each");
}

/// Every type the Recruits build (`construction::site::builds`) has its entry, once, and every entry's type is one they
/// build; a building they don't build yet is a `Planned` entry, by the name of its type, a Cults structure with its
/// site (every one is in the game's model since slice M5-G2, plan decision T69). Slice M5-G2 makes three of them live:
/// the Gunworks, the Machine Shop and the Scrap Hangar; slice M5-N2 the Garrison; slice M5-N1b the Altar; slice M5-N4a
/// the Cathedral.
#[test]
fn the_construct_menus_list_every_recruit_build_once() {
    let builds: BTreeSet<ObjectKind> = (ObjectKind::ALL.iter().copied())
        .filter(|&kind| builds(ObjectKind::Recruit, kind))
        .collect();
    let kinds: Vec<ObjectKind> = CONSTRUCT_MENU.iter().filter_map(|slot| slot.entry.kind()).collect();
    assert_eq!(kinds.len(), builds.len(), "each once: {kinds:?}");
    assert_eq!(kinds.into_iter().collect::<BTreeSet<_>>(), builds);
    let mut planned = Vec::new();
    for slot in CONSTRUCT_MENU {
        if let ConstructEntry::Planned(name) = slot.entry {
            let kind = (ObjectKind::ALL.iter().copied())
                .find(|kind| kind.def().name == name)
                .unwrap_or_else(|| panic!("the {name}: no type of that name"));
            let def = kind.def();
            assert_eq!(def.faction, Some(Faction::Cults), "{name}");
            assert!(
                crate::construction::site::site_def(kind).is_some(),
                "the {name} has its site"
            );
            assert!(
                !builds.contains(&kind),
                "the Recruits build the {name}: list it by its kind"
            );
            planned.push(kind);
        }
    }
    let later = [
        ObjectKind::ChopShop,
        ObjectKind::Airstrip,
        ObjectKind::Temple,
        ObjectKind::Shrine,
    ];
    assert_eq!(planned, later, "the buildings whose slices come later");
    let live = [
        ObjectKind::Gunworks,
        ObjectKind::MachineShop,
        ObjectKind::ScrapHangar,
        ObjectKind::Garrison,
        ObjectKind::Altar,
        ObjectKind::Cathedral,
    ];
    assert!(
        live.iter().all(|kind| builds.contains(kind)),
        "slice M5-G2's three, M5-N2's, M5-N1b's and M5-N4a's"
    );
}
