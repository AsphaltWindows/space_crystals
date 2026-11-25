//! What the Syndicate's stat blocks share, for their modules' tests (`units/`, `structures/`). The rules every kind
//! follows are in `defs_are_consistent` (`defs/tests.rs`).

use std::collections::BTreeSet;

use super::{EJECT_MENU_ORDER, EXPAND_MENU, ExpandEntry};
use crate::defs::{AttackAttrs, ObjectKind};
use crate::player::Faction;

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

/// Design change 0257: the EjectMenu lists every Syndicate unit type, each once and nothing else, the Agent in Q, the
/// Guard in W and the Grenadier in E ("Agent, Guard, Grenadier, …"; design change 0285).
#[test]
fn the_eject_menu_lists_every_syndicate_unit_once() {
    let units: BTreeSet<ObjectKind> = (ObjectKind::ALL.iter().copied())
        .filter(|kind| kind.def().faction == Some(Faction::Syndicate) && kind.def().unit.is_some())
        .collect();
    let listed: BTreeSet<ObjectKind> = EJECT_MENU_ORDER.iter().copied().collect();
    assert_eq!(listed.len(), EJECT_MENU_ORDER.len(), "each once: {EJECT_MENU_ORDER:?}");
    assert_eq!(listed, units);
    assert_eq!(
        EJECT_MENU_ORDER[..3],
        [ObjectKind::Agent, ObjectKind::Guard, ObjectKind::Grenadier]
    );
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
    for entry in EXPAND_MENU {
        if let ExpandEntry::Planned(name) = entry {
            let in_game = ObjectKind::ALL.iter().find(|kind| kind.def().name == *name);
            assert_eq!(in_game, None, "{name} is in the game: list it by its kind");
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
