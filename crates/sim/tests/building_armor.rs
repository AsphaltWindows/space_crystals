//! Every faction's buildings against the building armor rule (slice M4-B; design change 0264; `core/entities.md >
//! Structure Type > Building armor rule`): "A building's armor should generally be **PointArmor n, FullArmor n x (its
//! size in grid squares)**, for some n chosen per building ... There may be exceptions." And the Syndicate's
//! expansions' placeholder (`factions/syndicate/structures/README.md > Tunnel Expansions > High armor underground`,
//! design changes 0262 and 0264): "**Placeholder for every expansion: n = 6, i.e. PointArmor 6, FullArmor 6 x size in
//! grid squares** (a 2x2 expansion: 6 / 24)". It reads the defs only (`ObjectKind::ALL`, `ObjectKind::def`), so a new
//! building is held to the rule as it is added; a map resource (`Category::Resource`) is no building.

use sc_sim::{Category, Footprint, ObjectKind};

/// The buildings that break the rule on purpose, each with the design's reason ("There may be exceptions"); a building
/// listed here that follows the rule fails the test, so that the list never goes stale. None today: the Cults' Storage,
/// the one there was (slice M5-A, 1 / 6 on its 2x2), followed the rule at 1 / 4 from slice M5-G2, and since design
/// change 0380 is a 3x2 at the rule's 1 / 6 (`factions/cults/structures/storage.md`: "FullArmor - 6 (building armor
/// rule, n × size = 1 × 6)").
const EXCEPTIONS: &[(ObjectKind, &str)] = &[];

/// The PointArmor n that expansions have, the placeholder of `factions/syndicate/structures/README.md > High armor
/// underground`.
const EXPANSION_POINT_ARMOR: f32 = 6.0;

/// Every building: its kind, its size in grid squares, and whether it is an underground expansion.
fn buildings() -> Vec<(ObjectKind, u32, bool)> {
    let structures = ObjectKind::ALL
        .iter()
        .copied()
        .filter(|k| k.def().category == Category::Structure);
    structures
        .map(|kind| {
            let def = kind.def();
            let Footprint::Tiles { w, h } = def.footprint else {
                panic!("{kind:?}: a structure has a tile footprint");
            };
            let underground = def.structure.as_ref().is_some_and(|s| s.underground);
            (kind, w * h, underground)
        })
        .collect()
}

/// Whether `kind`'s armor is PointArmor n, FullArmor n × `squares`, for its own n.
fn follows_the_rule(kind: ObjectKind, squares: u32) -> bool {
    let armor = kind.def().armor;
    armor.full == armor.point * squares as f32
}

/// 0264: every building follows the rule but the listed exceptions, and no listed exception follows it. Today all 29
/// do: the GDO's Deployment Center 1/16, Generator 1/4, Barracks 1/6, Extraction Facility 1/9, Extraction Plate 2/2,
/// Career Campus 1/4 (slice M5-N9, `factions/gdo/structures/career_campus.md`) and Arsenal 1/4 (slice M5-K,
/// `factions/gdo/structures/arsenal.md`); the Colonists' Beacon 0/0, Egress Node
/// 0/0, Robo Portal 1/16 and Bio Portal 1/9; the Syndicate's Tunnel 1/16, Hideout 6/24, Optics Lab 6/24 and Chemical
/// Plant 6/24 (slice M4-K), and its Laser Turret, raised, 1/1 (slice M4-S1;
/// `factions/syndicate/structures/laser_turret.md`: "building armor rule, 1x1"); the Cults' thirteen
/// ([`the_cults_buildings_follow_the_rule`]).
#[test]
fn every_building_has_full_armor_n_times_its_size() {
    let all = buildings();
    let breaking: Vec<String> = (all.iter())
        .filter(|&&(kind, squares, _)| !follows_the_rule(kind, squares) && !EXCEPTIONS.iter().any(|e| e.0 == kind))
        .map(|&(kind, squares, _)| {
            let armor = kind.def().armor;
            format!(
                "{kind:?}: PointArmor {}, FullArmor {} on {squares} squares",
                armor.point, armor.full
            )
        })
        .collect();
    assert_eq!(breaking, Vec::<String>::new(), "buildings that break 0264's rule");
    for &(kind, reason) in EXCEPTIONS {
        let &(_, squares, _) = all.iter().find(|b| b.0 == kind).expect("an exception is a building");
        assert!(
            !follows_the_rule(kind, squares),
            "{kind:?} follows the rule: a stale exception ({reason})"
        );
    }
    let mut kinds: Vec<ObjectKind> = all.iter().map(|b| b.0).collect();
    kinds.sort_by_key(|k| k.def().name);
    let expected = [
        ObjectKind::Airstrip,
        ObjectKind::Altar,
        ObjectKind::Armory,
        ObjectKind::Arsenal,
        ObjectKind::Barracks,
        ObjectKind::Beacon,
        ObjectKind::BioPortal,
        ObjectKind::CareerCampus,
        ObjectKind::Cathedral,
        ObjectKind::ChemicalPlant,
        ObjectKind::ChopShop,
        ObjectKind::DeploymentCenter,
        ObjectKind::EgressNode,
        ObjectKind::ExtractionFacility,
        ObjectKind::ExtractionPlate,
        ObjectKind::Garrison,
        ObjectKind::Generator,
        ObjectKind::Gunworks,
        ObjectKind::Hideout,
        ObjectKind::LaserTurret,
        ObjectKind::MachineShop,
        ObjectKind::OpticsLab,
        ObjectKind::RecruitmentHall,
        ObjectKind::RoboPortal,
        ObjectKind::ScrapHangar,
        ObjectKind::Shrine,
        ObjectKind::Storage,
        ObjectKind::Temple,
        ObjectKind::Tunnel,
    ];
    assert_eq!(kinds, expected, "the 29 buildings of today");
}

/// The plan addendum's G2 acceptance: "Every Cults building follows the rule (n = 2 for the Garrison) with no exception
/// left". Each of the faction's thirteen at its design file's PointArmor and FullArmor: the Recruitment Hall 1/16 on its
/// 4x4, the Storage 1/6 on its 3x2 (design change 0380), the Armory 1/6 on its 2x3; the Gunworks 1/4, the Garrison 2/8
/// ("building armor rule, n = 2: sturdier as a defense", `factions/cults/structures/garrison.md`), the Chop Shop 1/9,
/// the Machine Shop 1/4, the Airstrip 1/8 on its runway's 2x4 and the Scrap Hangar 1/9; the Altar 1/6
/// (`factions/cults/augmentation.md > Altar`), the Temple 1/9, the Cathedral 1/6 and the Shrine 1/4.
#[test]
fn the_cults_buildings_follow_the_rule() {
    let rows = [
        (ObjectKind::RecruitmentHall, 1.0, 16.0),
        (ObjectKind::Storage, 1.0, 6.0),
        (ObjectKind::Armory, 1.0, 6.0),
        (ObjectKind::Gunworks, 1.0, 4.0),
        (ObjectKind::Garrison, 2.0, 8.0),
        (ObjectKind::ChopShop, 1.0, 9.0),
        (ObjectKind::MachineShop, 1.0, 4.0),
        (ObjectKind::Airstrip, 1.0, 8.0),
        (ObjectKind::ScrapHangar, 1.0, 9.0),
        (ObjectKind::Altar, 1.0, 6.0),
        (ObjectKind::Temple, 1.0, 9.0),
        (ObjectKind::Cathedral, 1.0, 6.0),
        (ObjectKind::Shrine, 1.0, 4.0),
    ];
    let cults: Vec<(ObjectKind, f32, f32)> = (buildings().into_iter())
        .filter(|&(kind, _, _)| kind.def().faction == Some(sc_sim::Faction::Cults))
        .map(|(kind, _, _)| (kind, kind.def().armor.point, kind.def().armor.full))
        .collect();
    assert_eq!(cults, rows, "every Cults building, in the registry's order");
    for (kind, squares, _) in buildings() {
        if kind.def().faction == Some(sc_sim::Faction::Cults) {
            assert!(follows_the_rule(kind, squares), "{kind:?}");
            assert!(!EXCEPTIONS.iter().any(|e| e.0 == kind), "{kind:?}: no exception left");
        }
    }
}

/// Plan §3.3 S2's acceptance, "B's armour test gains the turret's two forms": a structure with a pop-up form has two
/// armours, and each follows the rule on its size (`factions/syndicate/structures/laser_turret.md`: "Armor (building
/// armor rule, 1x1): raised PointArmor 1, FullArmor 1; underground (pop-up) PointArmor 6, FullArmor 6"), the
/// underground one at the expansions' n. Today the Laser Turret alone.
#[test]
fn a_popup_follows_the_rule_in_both_forms() {
    let popups: Vec<(ObjectKind, u32, sc_sim::defs::Armor)> = (buildings().into_iter())
        .filter_map(|(kind, squares, _)| Some((kind, squares, sc_sim::popup::def_of(kind)?.lowered_armor)))
        .collect();
    for &(kind, squares, lowered) in &popups {
        assert!(follows_the_rule(kind, squares), "{kind:?} raised");
        assert_eq!(lowered.full, lowered.point * squares as f32, "{kind:?} underground");
        assert_eq!(lowered.point, EXPANSION_POINT_ARMOR, "{kind:?} underground");
    }
    let kinds: Vec<ObjectKind> = popups.iter().map(|p| p.0).collect();
    assert_eq!(kinds, [ObjectKind::LaserTurret]);
    assert_eq!(ObjectKind::LaserTurret.def().armor.point, 1.0);
}

/// `factions/syndicate/structures/README.md > High armor underground`: every underground expansion has the
/// placeholder's PointArmor 6 (and so, by the rule, FullArmor 6 × its size): today the Hideout, the Optics Lab and
/// the Chemical Plant (slice M4-K), each 6 / 24 on its 2 × 2.
#[test]
fn every_expansion_has_point_armor_six() {
    let expansions: Vec<(ObjectKind, f32, f32)> = (buildings().into_iter())
        .filter(|&(_, _, underground)| underground)
        .map(|(kind, _, _)| (kind, kind.def().armor.point, kind.def().armor.full))
        .collect();
    assert!(!expansions.is_empty());
    for &(kind, point, _) in &expansions {
        assert_eq!(point, EXPANSION_POINT_ARMOR, "{kind:?}");
    }
    let today = [ObjectKind::Hideout, ObjectKind::OpticsLab, ObjectKind::ChemicalPlant];
    assert_eq!(expansions, today.map(|kind| (kind, 6.0, 24.0)));
}
