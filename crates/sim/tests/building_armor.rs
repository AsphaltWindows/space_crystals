//! Every faction's buildings against the building armor rule (slice M4-B; design change 0264; `core/entities.md >
//! Structure Type > Building armor rule`): "A building's armor should generally be **PointArmor n, FullArmor n x (its
//! size in grid squares)**, for some n chosen per building ... There may be exceptions." And the Syndicate's
//! expansions' placeholder (`factions/syndicate/structures/README.md > Tunnel Expansions > High armor underground`,
//! design changes 0262 and 0264): "**Placeholder for every expansion: n = 6, i.e. PointArmor 6, FullArmor 6 x size in
//! grid squares** (a 2x2 expansion: 6 / 24)". It reads the defs only (`ObjectKind::ALL`, `ObjectKind::def`), so a new
//! building is held to the rule as it is added; a map resource (`Category::Resource`) is no building.

use sc_sim::{Category, Footprint, ObjectKind};

/// The buildings that break the rule on purpose, each with the design's reason ("There may be exceptions"). None
/// today; a building listed here that follows the rule fails the test, so that the list never goes stale.
const EXCEPTIONS: &[(ObjectKind, &str)] = &[];

/// The PointArmor n that expansions have, the placeholder of `> High armor underground`.
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

/// 0264: every building follows the rule but the listed exceptions, and no listed exception follows it. Today all 11
/// do, the exceptions list being empty: the GDO's Deployment Center 1/16, Generator 1/4, Barracks 1/6, Extraction
/// Facility 1/9 and Extraction Plate 2/2; the Colonists' Beacon 0/0, Egress Node 0/0, Robo Portal 1/16 and Bio Portal
/// 1/9; the Syndicate's Tunnel 1/16 and Hideout 6/24.
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
        ObjectKind::Barracks,
        ObjectKind::Beacon,
        ObjectKind::BioPortal,
        ObjectKind::DeploymentCenter,
        ObjectKind::EgressNode,
        ObjectKind::ExtractionFacility,
        ObjectKind::ExtractionPlate,
        ObjectKind::Generator,
        ObjectKind::Hideout,
        ObjectKind::RoboPortal,
        ObjectKind::Tunnel,
    ];
    assert_eq!(kinds, expected, "the 11 buildings of today");
}

/// `> High armor underground`: every underground expansion has the placeholder's PointArmor 6 (and so, by the rule,
/// FullArmor 6 × its size): today the Hideout, 6 / 24 on its 2 × 2.
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
    assert_eq!(expansions, [(ObjectKind::Hideout, 6.0, 24.0)]);
}
