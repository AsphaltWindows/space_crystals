//! What the Syndicate's stat blocks share, for their modules' tests (`units/`, `structures/`). The rules every kind
//! follows are in `defs_are_consistent` (`defs/tests.rs`).

use std::collections::BTreeSet;

use super::{EJECT_MENU_ORDER, EXPAND_MENU, ExpandEntry};
use crate::defs::structures::SymmetryType;
use crate::defs::{Armor, AttackAttrs, Category, ObjectKind};
use crate::player::Faction;
use crate::scenario::StartingConditions;

/// A unit's attack timings, `(AimDuration, FiringDuration, CooldownDuration, ReloadDuration)`, and the frames from one
/// shot to the next, their sum.
pub(super) fn timings(attack: &AttackAttrs) -> ((u32, u32, u32, u32), u32) {
    let t = (
        attack.aim_ticks,
        attack.firing_ticks,
        attack.cooldown_ticks,
        attack.reload_ticks,
    );
    (t, t.0 + t.1 + t.2 + t.3)
}

/// A Syndicate unit's second type, which never enters the Tunnel Network and so has no EjectMenu tile
/// (`EJECT_MENU_ORDER`): a deployed Laser Battery can't move, and a burrowed Transport surfaces to drive in.
const NEVER_INSIDE: [ObjectKind; 2] = [
    ObjectKind::LaserBatteryDeployed,
    ObjectKind::SubterraneanTransportBurrowed,
];

/// Design change 0257: the EjectMenu lists every Syndicate unit type, each once and nothing else, the Agent in Q, the
/// Guard in W and the Grenadier in E ("Agent, Guard, Grenadier, …"; design change 0285), then the vehicles and the
/// aircraft in `factions/syndicate/units/README.md > Files`'s order (slice M6-SK), but the two second types that never
/// enter the network ([`NEVER_INSIDE`]).
#[test]
fn the_eject_menu_lists_every_syndicate_unit_once() {
    let units: BTreeSet<ObjectKind> = (ObjectKind::ALL.iter().copied())
        .filter(|kind| kind.def().faction == Some(Faction::Syndicate) && kind.def().unit.is_some())
        .filter(|kind| !NEVER_INSIDE.contains(kind))
        .collect();
    let listed: BTreeSet<ObjectKind> = EJECT_MENU_ORDER.iter().copied().collect();
    assert_eq!(listed.len(), EJECT_MENU_ORDER.len(), "each once: {EJECT_MENU_ORDER:?}");
    assert_eq!(listed, units);
    assert_eq!(
        EJECT_MENU_ORDER[..3],
        [ObjectKind::Agent, ObjectKind::Guard, ObjectKind::Grenadier]
    );
    let names: Vec<&str> = EJECT_MENU_ORDER.iter().map(|kind| kind.def().name).collect();
    assert_eq!(names, designed_units(), "the README's order");
    for kind in NEVER_INSIDE {
        assert_eq!(kind.def().faction, Some(Faction::Syndicate), "{kind:?}");
    }
}

/// The units of `factions/syndicate/units/README.md > Files`, in its order ("Listed in panel order"), by their names
/// in bold ("**Assault Bike**"), read from the design.
fn designed_units() -> Vec<String> {
    let path = concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/../../artifacts/designer/design/factions/syndicate/units/README.md"
    );
    let text = std::fs::read_to_string(path).unwrap_or_else(|e| panic!("{path}: {e}"));
    let files = text.split("## Files").nth(1).expect("the Files list");
    let names: Vec<String> = (files.lines())
        .filter_map(|line| line.strip_prefix("- `")?.split("**").nth(1).map(str::to_owned))
        .collect();
    assert!(names.len() > 3, "the Files list in {path}: {names:?}");
    names
}

/// Every Syndicate kind of the design, in the registry's order (each a stored name:
/// `every_kind_keeps_its_place_and_name`): the M4 kinds, then slice M6-SK's (plan decision T84, design change 0367),
/// the expansions of the Tech Tree table after its first three, then the units of
/// `factions/syndicate/units/README.md > Files` after the Grenadier, each unit's second type after its first. The list
/// is complete: every structure of `factions/syndicate/structures/README.md > Tech Tree Overview` and every unit of the
/// Files list has its kind, and no other Syndicate kind exists but the two second types.
#[test]
fn the_syndicate_kinds_are_every_designed_one() {
    let syndicate: Vec<(ObjectKind, &str)> = (ObjectKind::ALL.iter())
        .filter(|kind| kind.def().faction == Some(Faction::Syndicate))
        .map(|&kind| (kind, kind.def().name))
        .collect();
    assert_eq!(
        syndicate,
        [
            (ObjectKind::Guard, "Guard"),
            (ObjectKind::Agent, "Agent"),
            (ObjectKind::Tunnel, "Tunnel"),
            (ObjectKind::Hideout, "Hideout"),
            (ObjectKind::OpticsLab, "Optics Lab"),
            (ObjectKind::ChemicalPlant, "Chemical Plant"),
            (ObjectKind::Grenadier, "Grenadier"),
            (ObjectKind::LaserTurret, "Laser Turret"),
            (ObjectKind::Vault, "Vault"),
            (ObjectKind::VehicleAssembly, "Vehicle Assembly"),
            (ObjectKind::Forge, "Forge"),
            (ObjectKind::NullChamber, "Null Chamber"),
            (ObjectKind::AerospaceAssembly, "Aerospace Assembly"),
            (ObjectKind::OrbitalCommand, "Orbital Command"),
            (ObjectKind::AssaultBike, "Assault Bike"),
            (ObjectKind::Shadebringer, "Shadebringer"),
            (ObjectKind::LaserBattery, "Laser Battery"),
            (ObjectKind::LaserBatteryDeployed, "Laser Battery (Deployed)"),
            (ObjectKind::SubterraneanTransport, "Subterranean Transport"),
            (
                ObjectKind::SubterraneanTransportBurrowed,
                "Subterranean Transport (Burrowed)"
            ),
            (ObjectKind::Phantom, "Phantom"),
            (ObjectKind::Specter, "Specter"),
            (ObjectKind::Prism, "Prism"),
        ]
    );
    let named = |category: Category| -> BTreeSet<String> {
        (syndicate.iter())
            .filter(|&&(kind, _)| kind.def().category == category && !NEVER_INSIDE.contains(&kind))
            .map(|&(_, name)| name.to_owned())
            .collect()
    };
    let structures: BTreeSet<String> = tech_tree_rows().into_iter().map(|(name, _)| name).collect();
    assert_eq!(named(Category::Structure), structures, "the Tech Tree Overview's");
    let units: BTreeSet<String> = designed_units().into_iter().collect();
    assert_eq!(named(Category::Unit), units, "the units of the Files list");
}

/// Slice M6-SK's kinds (plan decision T84) still out of play, in the registry's order: its fifteen but those a slice
/// has made live, the Vehicle Assembly and the Assault Bike (slice M6-SV), the Forge, the Aerospace Assembly and the
/// Phantom (slice M6-SD), and the Null Chamber (slice M6-SN).
const SK_KINDS: [ObjectKind; 9] = [
    ObjectKind::Vault,
    ObjectKind::OrbitalCommand,
    ObjectKind::Shadebringer,
    ObjectKind::LaserBattery,
    ObjectKind::LaserBatteryDeployed,
    ObjectKind::SubterraneanTransport,
    ObjectKind::SubterraneanTransportBurrowed,
    ObjectKind::Specter,
    ObjectKind::Prism,
];

/// Plan decision T84, after T69's rule: no player makes one of slice M6-SK's kinds still out of play ([`SK_KINDS`]),
/// each kept out until its slice. No Tunnel builds one of the two expansions left (neither has
/// `StructureDef::expansion`, so the gate refuses each Expand: `Reject::CannotConstruct`), and no def names one of the
/// nine where a kind is made: no producer's recipe (`ProductionDef::recipes`; the Vehicle Assembly trains the Assault
/// Bike alone since slice M6-SV, and the Aerospace Assembly the Phantom alone since slice M6-SD), no constructor's
/// Construct or Warp In list, no worker's
/// builds, no Deploy, no Conversion's or Riot Shield's form; nor does a faction's starting layout
/// (`StartingConditions`). Their numbers wait beside their defs, each for its slice: the expansions' `*_EXPANSION` and
/// the producers' `*_RECIPE`s (`syndicate::structures`).
#[test]
fn no_player_makes_one_of_the_new_kinds() {
    let made = |kind: ObjectKind| -> Vec<ObjectKind> {
        let def = kind.def();
        let mut made = Vec::new();
        if let Some(structure) = &def.structure {
            let recipes = structure
                .production
                .as_ref()
                .map_or(&[][..], |production| production.recipes);
            made.extend(recipes.iter().map(|recipe| recipe.kind));
            made.extend(structure.constructs.iter().map(|recipe| recipe.kind));
            made.extend(structure.warp_in.iter().map(|recipe| recipe.kind));
        }
        if let Some(unit) = &def.unit {
            made.extend(unit.worker.iter().flat_map(|worker| worker.builds.iter().copied()));
            made.extend(unit.deploy.iter().map(|recipe| recipe.kind));
            made.extend(unit.conversion.iter().flat_map(|conversion| conversion.forms));
            made.extend(
                unit.riot_shield
                    .iter()
                    .flat_map(|shield| [shield.lowered, shield.raised]),
            );
        }
        made
    };
    for &kind in ObjectKind::ALL {
        let named: Vec<ObjectKind> = made(kind).into_iter().filter(|made| SK_KINDS.contains(made)).collect();
        assert_eq!(named, Vec::<ObjectKind>::new(), "{kind:?} makes slice M6-SK's");
    }
    for kind in SK_KINDS
        .iter()
        .filter(|kind| kind.def().category == Category::Structure)
    {
        let structure = kind.def().structure.as_ref().expect("an expansion");
        assert_eq!(
            (structure.expansion, structure.production.as_ref()),
            (None, None),
            "{kind:?}"
        );
    }
    let expand: Vec<ObjectKind> = EXPAND_MENU.iter().filter_map(|entry| entry.kind()).collect();
    assert!(expand.iter().all(|kind| !SK_KINDS.contains(kind)), "{expand:?}");
    for &faction in Faction::ALL {
        let objects = StartingConditions::of(faction).objects;
        assert!(
            objects.iter().all(|(kind, _)| !SK_KINDS.contains(kind)),
            "{faction:?}: {objects:?}"
        );
    }
}

/// What every expansion of slice M6-SK is (`factions/syndicate/structures/README.md > Tunnel Expansions`, design
/// change 0367's "Placeholder stats"): `name`, an underground Syndicate structure of `side` × `side` tiles, MaxHP
/// `max_hp`, the expansions' armour (n = 6: PointArmor 6, FullArmor 6 × its squares), no sight and Groupable (design
/// change 0254), slopes allowed (design change 0005), SymmetryType AAAA (plan decision T29: not given), behind
/// `requires`, as the Tech Tree Overview's table has it; while it is out of play ([`SK_KINDS`]), no Tunnel builds it
/// and it produces nothing until its slice (plan decision T84), whose own test checks what it makes live.
pub(super) fn assert_expansion(kind: ObjectKind, name: &str, side: u32, max_hp: f32, requires: &[ObjectKind]) {
    let def = kind.def();
    let structure = def.structure.as_ref().expect("a structure");
    assert_eq!(
        (def.name, def.category, def.faction, def.tile_size(), structure.symmetry),
        (
            name,
            Category::Structure,
            Some(Faction::Syndicate),
            Some((side, side)),
            SymmetryType::AAAA
        )
    );
    let armor = Armor {
        point: 6.0,
        full: 6.0 * (side * side) as f32,
    };
    assert_eq!(
        (def.max_hp, def.max_shields, def.armor, def.sight_range),
        (Some(max_hp), None, armor, 0.0)
    );
    assert_eq!(
        (def.selectable, def.destructible, def.groupable, def.needs_flat_ground),
        (true, true, true, false)
    );
    assert_eq!((structure.underground, structure.requires), (true, requires));
    assert_eq!(structure.site, None, "no worker builds it");
    if SK_KINDS.contains(&kind) {
        assert_eq!(
            (structure.expansion, structure.production.as_ref()),
            (None, None),
            "not live until its slice"
        );
    }
    let row = tech_tree_rows().into_iter().find(|(row, _)| row == name);
    let required: Vec<&str> = requires.iter().map(|kind| kind.def().name).collect();
    assert_eq!(
        row,
        Some((name.to_owned(), required.join(", "))),
        "the Tech Tree Overview's"
    );
}

/// The Tech Tree's structures, in its table's order: each row of `factions/syndicate/structures/README.md > Tech Tree
/// Overview`, its Structure and its Requires ("—" for none, as ""), read from the design.
fn tech_tree_rows() -> Vec<(String, String)> {
    let path = concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/../../artifacts/designer/design/factions/syndicate/structures/README.md"
    );
    let text = std::fs::read_to_string(path).unwrap_or_else(|e| panic!("{path}: {e}"));
    let rows: Vec<(String, String)> = (text.lines())
        .skip_while(|line| !line.starts_with("| Structure |"))
        .skip(2)
        .take_while(|line| line.starts_with('|'))
        .map(|line| {
            let cells: Vec<&str> = line.split('|').map(str::trim).collect();
            let requires = if cells[5] == "—" { "" } else { cells[5] };
            (cells[1].to_owned(), requires.to_owned())
        })
        .collect();
    assert!(rows.len() > 1, "the Tech Tree Overview's table in {path}");
    rows
}

/// The Tech Tree's expansions, in its table's order: the rows of `factions/syndicate/structures/README.md > Tech Tree
/// Overview` whose Type is "Expansion", read from the design.
fn tech_tree_expansions() -> Vec<String> {
    let path = concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/../../artifacts/designer/design/factions/syndicate/structures/README.md"
    );
    let text = std::fs::read_to_string(path).unwrap_or_else(|e| panic!("{path}: {e}"));
    let table = text
        .split("## Tech Tree Overview")
        .nth(1)
        .expect("the Tech Tree Overview");
    let table = table.split("\n## ").next().unwrap_or(table);
    table
        .lines()
        .filter_map(|line| {
            let cells: Vec<&str> = line.split('|').map(str::trim).collect();
            (cells.len() > 3 && cells[2] == "Expansion").then(|| cells[1].to_owned())
        })
        .collect()
}

/// Design changes 0257 and 0291: the ExpandMenu lists every expansion of the Tech Tree table, in its order, each once,
/// a type in the game by its kind, the Hideout first, and then the Laser Turret: 0291's verification, "The ExpandMenu
/// shows nine expansion slots and then the turret".
#[test]
fn the_expand_menu_follows_the_tech_tree() {
    let names: Vec<&str> = EXPAND_MENU.iter().map(|entry| entry.name()).collect();
    let table = tech_tree_expansions();
    assert_eq!(table.len(), 9, "the table's expansions: {table:?}");
    assert_eq!(names[..9], table);
    assert_eq!(names[9..], ["Laser Turret"], "the tenth slot");
    assert_eq!(EXPAND_MENU[0], ExpandEntry::Kind(ObjectKind::Hideout));
    // A type a Tunnel builds is listed by its kind: the Laser Turret's slot since slice M4-S2, a Tunnel's build (the
    // only one since design change 0359).
    for entry in EXPAND_MENU {
        if let ExpandEntry::Planned(name) = entry {
            let built = (ObjectKind::ALL.iter()).find(|kind| {
                let structure = kind.def().structure.as_ref();
                kind.def().name == *name && structure.is_some_and(|structure| structure.expansion.is_some())
            });
            assert_eq!(built, None, "a Tunnel builds the {name}: list it by its kind");
        }
    }
}

/// Every type a Tunnel builds (`StructureDef::expansion`) has its entry, once, and every entry's type is built by a
/// Tunnel.
#[test]
fn the_expand_menu_lists_every_expansion_once() {
    let expansions: BTreeSet<ObjectKind> = (ObjectKind::ALL.iter().copied())
        .filter(|kind| (kind.def().structure.as_ref()).is_some_and(|structure| structure.expansion.is_some()))
        .collect();
    let kinds: Vec<ObjectKind> = EXPAND_MENU.iter().filter_map(|entry| entry.kind()).collect();
    let listed: BTreeSet<ObjectKind> = kinds.iter().copied().collect();
    assert_eq!(listed.len(), kinds.len(), "each once: {kinds:?}");
    assert_eq!(listed, expansions);
}
