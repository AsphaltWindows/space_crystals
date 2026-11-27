//! (M6-GK) The GDO's kinds as a whole: every one of the design's is in the registry, and the ones of design change 0366
//! that no player makes yet are made by nothing until their slices (plan decision T84). The rules every kind follows
//! are in `defs_are_consistent` (`defs/tests.rs`), and each kind's own numbers in its module's test.

use std::collections::BTreeSet;

use super::PLANNED_RECIPES;
use crate::defs::{CONSTRUCT_MENU, Category, ConstructEntry, EXPAND_MENU, ExpandEntry, ObjectKind};
use crate::player::Faction;

/// The GDO's design folder, `factions/gdo/`.
const GDO_DESIGN: &str = concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/../../artifacts/designer/design/factions/gdo"
);

/// Slice M6-GK's kinds, design change 0366's, in the registry's order (`factions/gdo/structures/*.md`,
/// `factions/gdo/economy.md > SupplyTower`, `factions/gdo/units/*.md`, `factions/gdo/units/twister.md > Mines`).
const NEW: [ObjectKind; 23] = [
    ObjectKind::Factory,
    ObjectKind::Airfield,
    ObjectKind::AdvancedMechanics,
    ObjectKind::PropulsionLabs,
    ObjectKind::IndustrialPowerplant,
    ObjectKind::RadarStation,
    ObjectKind::DefenseSystem,
    ObjectKind::MissileTurret,
    ObjectKind::Outpost,
    ObjectKind::Bunker,
    ObjectKind::FirstAidTent,
    ObjectKind::SupplyTower,
    ObjectKind::Raptor,
    ObjectKind::Twister,
    ObjectKind::Prospector,
    ObjectKind::ArtilleryRig,
    ObjectKind::ArtilleryRigDeployed,
    ObjectKind::Hornet,
    ObjectKind::Dragonfly,
    ObjectKind::DroneOperator,
    ObjectKind::OperatorDrone,
    ObjectKind::SupplyChopper,
    ObjectKind::Mine,
];

/// The titles of the design files in `factions/gdo/<folder>/` but its README ("# Raptor"), spelt as the kinds' names:
/// the two the design spells as one word ("# DeploymentCenter", "# SupplyChopper") in two.
fn titles(folder: &str) -> BTreeSet<String> {
    let folder = format!("{GDO_DESIGN}/{folder}");
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
        names.insert(spelt(title.trim()));
    }
    names
}

/// A design heading's name as its kind's name: "ExtractionFacility" is "Extraction Facility".
fn spelt(title: &str) -> String {
    let mut name = String::new();
    for (i, c) in title.char_indices() {
        let after_lower = title[..i].chars().last().is_some_and(char::is_lowercase);
        if c.is_uppercase() && after_lower {
            name.push(' ');
        }
        name.push(c);
    }
    name
}

/// The economy buildings of `factions/gdo/economy.md`, one heading each ("## ExtractionFacility"), after its
/// Starting Conditions.
fn economy_buildings() -> BTreeSet<String> {
    let path = format!("{GDO_DESIGN}/economy.md");
    let text = std::fs::read_to_string(&path).unwrap_or_else(|e| panic!("{path}: {e}"));
    let names: BTreeSet<String> = (text.lines())
        .filter_map(|line| line.strip_prefix("## "))
        .filter(|heading| *heading != "Starting Conditions")
        .map(|heading| spelt(heading.trim()))
        .collect();
    assert!(names.len() > 1, "the buildings of {path}");
    names
}

/// Every kind of the GDO's design, the M1 to M5 kinds, then slice M6-GK's ([`NEW`], plan decision T84). The list is
/// complete: every structure of `factions/gdo/structures/` and every economy building of `factions/gdo/economy.md`
/// has its kind, and so has every unit of `factions/gdo/units/`, with the second types of three of them, Peacekeeper
/// (Shielded) (design change 0173), Artillery Rig (Deployed) (`factions/gdo/units/artillery_rig.md > Modes`) and the
/// Drone Operator's drone (`factions/gdo/units/drone_operator.md > Placeholder stats`), and the Twister's mine
/// (`factions/gdo/units/twister.md > Mines`). No other GDO kind exists.
#[test]
fn the_gdo_kinds_are_every_designed_one() {
    let gdo: Vec<ObjectKind> = (ObjectKind::ALL.iter().copied())
        .filter(|kind| kind.def().faction == Some(Faction::Gdo))
        .collect();
    let before = [
        ObjectKind::Peacekeeper,
        ObjectKind::DeploymentCenter,
        ObjectKind::Generator,
        ObjectKind::Barracks,
        ObjectKind::ExtractionFacility,
        ObjectKind::ExtractionPlate,
        ObjectKind::PeacekeeperShielded,
        ObjectKind::CareerCampus,
        ObjectKind::FieldEngineer,
        ObjectKind::Arsenal,
        ObjectKind::Rocketman,
    ];
    assert_eq!(gdo, [&before[..], &NEW[..]].concat());
    // Together, after the Rocketman, the registry's last kind before them (slice M6-SK's kinds follow them).
    let first = ObjectKind::Rocketman as usize + 1;
    assert_eq!(&ObjectKind::ALL[first..first + NEW.len()], NEW);
    let named = |category: Category| -> BTreeSet<String> {
        (gdo.iter())
            .filter(|kind| kind.def().category == category)
            .map(|kind| kind.def().name.to_owned())
            .collect()
    };
    let structures: BTreeSet<String> = titles("structures").union(&economy_buildings()).cloned().collect();
    assert_eq!(named(Category::Structure), structures);
    let mut units = titles("units");
    units.extend(["Peacekeeper (Shielded)", "Artillery Rig (Deployed)", "Drone"].map(String::from));
    assert_eq!(named(Category::Unit), units);
    assert_eq!(named(Category::Mine), BTreeSet::from(["Mine".to_owned()]));
}

/// The kinds of [`NEW`] that a slice since M6-GK has made live, each with the kind whose list makes it, in the order
/// the slices landed: (M6-GD) the Barracks' Drone Operator, and its drone, which the operator's stock launches
/// (`UnitDef::stock`).
const LIVE: &[(ObjectKind, ObjectKind)] = &[
    (ObjectKind::Barracks, ObjectKind::DroneOperator),
    (ObjectKind::DroneOperator, ObjectKind::OperatorDrone),
];

/// The kinds that some live list makes, each with the kind whose list it is: a constructor's `Constructs` list
/// (`StructureDef::constructs`), a Beacon's Warp In list, a producer's `Produces` list, a builder's builds, a unit's
/// deploy, its Riot Shield's two types, a Legionnaire's two forms and (M6-GD) a unit's stock; and the menus of the
/// Cults' Recruits and of the Syndicate's Agents, whose builders are their builds' (already listed).
fn made() -> Vec<(ObjectKind, ObjectKind)> {
    let mut made = Vec::new();
    for &maker in ObjectKind::ALL {
        let def = maker.def();
        if let Some(structure) = &def.structure {
            let production = structure.production.as_ref().map_or(&[][..], |p| p.recipes);
            for recipe in structure.constructs.iter().chain(structure.warp_in).chain(production) {
                made.push((maker, recipe.kind));
            }
        }
        if let Some(unit) = &def.unit {
            let builds = unit.worker.map_or(&[][..], |worker| worker.builds);
            made.extend(builds.iter().map(|&kind| (maker, kind)));
            made.extend(unit.deploy.map(|recipe| (maker, recipe.kind)));
            made.extend(unit.riot_shield.map(|shield| (maker, shield.raised)));
            made.extend(unit.riot_shield.map(|shield| (maker, shield.lowered)));
            made.extend(
                unit.conversion
                    .into_iter()
                    .flat_map(|c| c.forms.map(|form| (maker, form))),
            );
            made.extend(unit.stock.map(|stock| (maker, stock.item)));
        }
    }
    for slot in CONSTRUCT_MENU {
        if let ConstructEntry::Kind(kind) = slot.entry {
            made.push((ObjectKind::Recruit, kind));
        }
    }
    for entry in EXPAND_MENU {
        if let ExpandEntry::Kind(kind) = entry {
            made.push((ObjectKind::Agent, *kind));
        }
    }
    made
}

/// T69's rule, for plan decision T84: no live list makes a kind of slice M6-GK's (no `constructs`, `production`,
/// `builds` or menu entry goes live with the kinds) until its slice makes it live ([`LIVE`]), and the entries that will
/// are held, each once, in [`PLANNED_RECIPES`]: a GDO maker's, for a kind of slice M6-GK's, every one of them but
/// those made live, the Outpost (deployed of a Prospector, `factions/gdo/units/prospector.md`), the deployed Rig and
/// the mine (laid by a Twister), which no recipe makes.
#[test]
fn no_new_kind_is_made_yet() {
    for (maker, kind) in made() {
        assert!(
            !NEW.contains(&kind) || LIVE.contains(&(maker, kind)),
            "{maker:?} makes {kind:?}"
        );
    }
    let live: Vec<ObjectKind> = LIVE.iter().map(|&(_, kind)| kind).collect();
    let held: Vec<ObjectKind> = PLANNED_RECIPES.iter().map(|(_, recipe)| recipe.kind).collect();
    let unique: BTreeSet<ObjectKind> = held.iter().copied().collect();
    assert_eq!(unique.len(), held.len(), "each held once");
    for &(maker, recipe) in PLANNED_RECIPES {
        assert_eq!(maker.def().faction, Some(Faction::Gdo), "{maker:?}");
        assert!(NEW.contains(&recipe.kind), "{:?}", recipe.kind);
        assert!(recipe.cost > 0 && recipe.frames > 0, "{:?}", recipe.kind);
        assert!(
            recipe
                .requires
                .iter()
                .all(|kind| kind.def().faction == Some(Faction::Gdo))
        );
    }
    assert!(held.iter().all(|kind| !live.contains(kind)), "a live kind isn't held");
    let unmade: Vec<ObjectKind> = (NEW.iter().copied())
        .filter(|kind| !held.contains(kind) && !live.contains(kind))
        .collect();
    let others = [ObjectKind::Outpost, ObjectKind::ArtilleryRigDeployed, ObjectKind::Mine];
    assert_eq!(unmade, others);
}

/// `factions/gdo/structures/deployment_center.md > Constructs`: the Deployment Center's held buildings, in the
/// design's order, each behind the buildings of its "Requires" clause, its own TechPrerequisite
/// (`StructureDef::requires`): the Supply Tower a Radar Station, the Factory a Barracks, the Radar Station a Generator,
/// Advanced Mechanics a Factory, the Airfield Advanced Mechanics, Propulsion Labs an Airfield, the Industrial
/// Powerplant Advanced Mechanics, and the Defense System a Radar Station and an Arsenal.
#[test]
fn the_deployment_centers_held_buildings_need_theirs() {
    let held: Vec<(ObjectKind, &[ObjectKind])> = (PLANNED_RECIPES.iter())
        .filter(|(maker, _)| *maker == ObjectKind::DeploymentCenter)
        .map(|(_, recipe)| (recipe.kind, recipe.kind.def().structure.as_ref().unwrap().requires))
        .collect();
    assert_eq!(
        held,
        [
            (ObjectKind::SupplyTower, &[ObjectKind::RadarStation][..]),
            (ObjectKind::Factory, &[ObjectKind::Barracks]),
            (ObjectKind::RadarStation, &[ObjectKind::Generator]),
            (ObjectKind::AdvancedMechanics, &[ObjectKind::Factory]),
            (ObjectKind::Airfield, &[ObjectKind::AdvancedMechanics]),
            (ObjectKind::PropulsionLabs, &[ObjectKind::Airfield]),
            (ObjectKind::IndustrialPowerplant, &[ObjectKind::AdvancedMechanics]),
            (
                ObjectKind::DefenseSystem,
                &[ObjectKind::RadarStation, ObjectKind::Arsenal]
            ),
        ]
    );
}
