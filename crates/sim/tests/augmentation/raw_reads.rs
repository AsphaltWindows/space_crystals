//! The guard on the accessors (slice M5-N1a; `ARCHITECTURE.md > Object model > Augmented stats`): a unit's MaxHP, its
//! speed and its weapon are read through those that apply its Augmentations (`Object::max_hp_with`,
//! `Object::movement` or `movement_ref`, `Object::max_speed`, `combat::weapon` or `weapon_ref`), never past them.
//! This test scans the sim's and the client's code for the reads past them, outside the definitions (`defs/`), the
//! accessors (`augmentation/`) and the tests:
//! - `.max_hp()`: `Object::max_hp`, the MaxHP before the Augmentations (and `LastKnown::max_hp`, a record's);
//! - `.movement` that isn't a call: a `UnitDef`'s movement numbers, before them;
//! - `weapon_of(`: a kind's weapon (`combat::weapon_of`), before them.
//!
//! Each read the list below doesn't allow, with the reason it isn't a unit's augmented stat, fails it: a new read goes
//! through the accessor, or joins the list with its reason. Comments are left out; a file's tests are left out from
//! its `#[cfg(test)] mod tests {` on.

use std::collections::BTreeMap;
use std::fs;
use std::path::Path;

const MAX_HP: &str = ".max_hp()";
const MOVEMENT: &str = ".movement";
const WEAPON_OF: &str = "weapon_of(";

/// The reads past the accessors that stay: file, pattern, how many, and why.
const ALLOWED: &[(&str, &str, usize, &str)] = &[
    (
        "crates/game/src/hud/info_panel.rs",
        MAX_HP,
        1,
        "a remembered structure's record (`LastKnown`)",
    ),
    (
        "crates/sim/src/combat.rs",
        WEAPON_OF,
        2,
        "the accessor's own: its definition, and the weapon in hand",
    ),
    (
        "crates/sim/src/combat/phases.rs",
        MOVEMENT,
        1,
        "the TurnRate, which no Augmentation changes",
    ),
    (
        "crates/sim/src/orders/world.rs",
        MOVEMENT,
        1,
        "a behaviour's unit's, with its increments (`Unit::max_speed`)",
    ),
    (
        "crates/sim/src/repair.rs",
        MOVEMENT,
        1,
        "whether a kind is a vehicle: its movement model, a glider's or not",
    ),
    (
        "crates/sim/src/riot_shield.rs",
        MOVEMENT,
        1,
        "the shield's speed ratio of the Peacekeeper's two types",
    ),
    (
        "crates/sim/src/sim/builds/shared.rs",
        MAX_HP,
        1,
        "a structure's, complete",
    ),
    (
        "crates/sim/src/sim/expansions.rs",
        MAX_HP,
        1,
        "an expansion's, a structure",
    ),
    (
        "crates/sim/src/sim/kinds.rs",
        WEAPON_OF,
        3,
        "whether a kind has a weapon, and keeps it through a change",
    ),
    ("crates/sim/src/sim/sites.rs", MAX_HP, 3, "a site's, a structure"),
    (
        "crates/sim/src/sim/transform.rs",
        MAX_HP,
        3,
        "a transformation's, between kinds that can't be augmented",
    ),
    ("crates/sim/src/sim/tunnels.rs", MAX_HP, 1, "a Tunnel's"),
    (
        "crates/sim/src/testing/invariants.rs",
        MOVEMENT,
        1,
        "the movement model, which no Augmentation changes",
    ),
];

/// What the scan leaves out: the definitions, the accessors and the tests.
fn skipped(path: &str) -> bool {
    let name = path.rsplit('/').next().unwrap_or(path);
    path.starts_with("crates/sim/src/defs/")
        || path.starts_with("crates/sim/src/augmentation")
        || path.split('/').any(|part| part == "tests")
        || matches!(name, "tests.rs" | "testing.rs")
}

/// How many times `pattern` occurs in `code`: for [`MOVEMENT`], only as a whole name that isn't called.
fn count(code: &str, pattern: &str) -> usize {
    let after = |i: usize| code[i + pattern.len()..].chars().next();
    let whole = |i: usize| !after(i).is_some_and(|c| c.is_alphanumeric() || c == '_' || c == '(');
    code.match_indices(pattern)
        .filter(|&(i, _)| pattern != MOVEMENT || whole(i))
        .count()
}

/// Every `.rs` file under `dir`, relative to `root`, with `/` between the parts.
fn rust_files(root: &Path, dir: &str, files: &mut Vec<String>) {
    let entries = fs::read_dir(root.join(dir)).unwrap_or_else(|e| panic!("{dir}: {e}"));
    for entry in entries {
        let entry = entry.expect("a directory entry");
        let name = entry.file_name().into_string().expect("a UTF-8 name");
        let path = format!("{dir}/{name}");
        if entry.file_type().expect("a file type").is_dir() {
            rust_files(root, &path, files);
        } else if name.ends_with(".rs") {
            files.push(path);
        }
    }
}

#[test]
fn no_code_reads_a_unit_s_stats_past_the_accessors() {
    let root = Path::new(env!("CARGO_MANIFEST_DIR")).join("../..");
    let mut files = Vec::new();
    for dir in ["crates/sim/src", "crates/game/src"] {
        rust_files(&root, dir, &mut files);
    }
    let mut found = BTreeMap::new();
    for path in files.iter().filter(|path| !skipped(path)) {
        let text = fs::read_to_string(root.join(path)).unwrap_or_else(|e| panic!("{path}: {e}"));
        let text = text.split("#[cfg(test)]\nmod tests {").next().unwrap_or("");
        for pattern in [MAX_HP, MOVEMENT, WEAPON_OF] {
            let code = text.lines().map(|line| line.split("//").next().unwrap_or(""));
            let n: usize = code.map(|line| count(line, pattern)).sum();
            if n > 0 {
                found.insert((path.clone(), pattern), n);
            }
        }
    }
    let allowed: BTreeMap<(String, &str), usize> = (ALLOWED.iter())
        .map(|&(path, pattern, n, _)| ((path.to_string(), pattern), n))
        .collect();
    let new: Vec<String> = (found.iter())
        .filter(|&(key, n)| allowed.get(key).is_none_or(|allowed| n > allowed))
        .map(|((path, pattern), n)| format!("{path}: {n} × `{pattern}`"))
        .collect();
    assert!(
        new.is_empty(),
        "reads past the accessors (`Object::max_hp_with`, `Object::movement`, `Object::max_speed`, \
         `combat::weapon`): read through them, or allow each read with its reason: {new:#?}"
    );
    let stale: Vec<&(String, &str)> = (allowed.iter())
        .filter(|&(key, n)| found.get(key) != Some(n))
        .map(|(key, _)| key)
        .collect();
    assert!(stale.is_empty(), "allowed reads gone: shorten the list: {stale:#?}");
}

/// The scan's own rules: a call of `Object::movement` isn't a read past it, nor is a longer name; comments don't count.
#[test]
fn the_scan_counts_reads_not_calls() {
    assert_eq!(count("unit.movement.max_speed()", MOVEMENT), 1);
    assert_eq!(count("(&unit.movement, x)", MOVEMENT), 1);
    assert_eq!(count("o.movement(research)", MOVEMENT), 0);
    assert_eq!(count("o.movement_of(x)", MOVEMENT), 0);
    assert_eq!(count("a.max_hp() + b.max_hp()", MAX_HP), 2);
    assert_eq!(count("o.max_hp_with(research)", MAX_HP), 0);
    assert!(skipped("crates/sim/src/defs/cults/units/recruit.rs") && skipped("crates/sim/src/orders/tests.rs"));
    assert!(!skipped("crates/sim/src/testing/invariants.rs") && !skipped("crates/sim/src/movement.rs"));
}
