//! (M5-X2) The data files as plain data: the merge patch, the layout and the export, the round trip, each refusal,
//! and the poll on a driven clock. Each test has a directory of its own under the system's temporary directory.

use std::fs;
use std::path::{Path, PathBuf};

use sc_sim::{ObjectKind, Ruleset};
use serde_json::{Value, json};

use super::layout::{Part, snake_case};
use super::load::builtin;
use super::ordered::Ordered;
use super::patch::merge;
use super::poll::{POLL_SECONDS, Poll};
use super::*;
use crate::tuning::ClientTuning;

/// A new, empty directory for the test `tag`.
fn dir(tag: &str) -> PathBuf {
    let dir = std::env::temp_dir().join(format!("sc_data_{tag}_{}", std::process::id()));
    let _ = fs::remove_dir_all(&dir);
    fs::create_dir_all(&dir).expect("a temporary directory");
    dir
}

/// Writes `text` to the file at `path` in `dir`, its folders made.
fn write(dir: &Path, path: &str, text: &str) {
    let file = dir.join(path);
    fs::create_dir_all(file.parent().expect("a folder")).expect("the folders");
    fs::write(file, text).expect("the file");
}

/// The refusal of a directory holding the file `path` alone, with `text`.
fn refusal(tag: &str, path: &str, text: &str) -> Refusal {
    let dir = dir(tag);
    write(&dir, path, text);
    let refused = load(&dir).expect_err("refused");
    let _ = fs::remove_dir_all(&dir);
    refused
}

/// The data of a directory holding the file `path` alone, with `text`.
fn data(tag: &str, path: &str, text: &str) -> Data {
    let dir = dir(tag);
    write(&dir, path, text);
    let data = load(&dir).expect("passes");
    let _ = fs::remove_dir_all(&dir);
    data
}

/// RFC 7386: objects merge member by member; anything else replaces the target's whole, a list too; `null` sets a
/// member to `null` (module docs of `super::patch`: every member is always there).
#[test]
fn a_merge_patch_merges_objects_and_replaces_the_rest() {
    let mut target = json!({"a": 1, "b": {"c": 2, "d": [1, 2], "e": "x"}, "f": null});
    merge(
        &mut target,
        &json!({"b": {"c": 3.5, "d": [5]}, "f": {"g": true}, "a": null}),
    );
    assert_eq!(
        target,
        json!({"a": null, "b": {"c": 3.5, "d": [5], "e": "x"}, "f": {"g": true}})
    );
    let mut same = json!({"a": [1, {"b": 2}]});
    merge(&mut same, &json!({}));
    assert_eq!(same, json!({"a": [1, {"b": 2}]}), "an empty patch changes nothing");
}

/// The layout mirrors the def modules (`super::layout`): every kind's file lies in its def module's folder of the
/// registry (`sc_sim::defs`'s `kind_registry!`; the GDO's Mine, neither a unit nor a building, in `gdo/`), named after
/// the kind, which is its module's own file but for the forms that share another kind's module (`PeacekeeperShielded`
/// in `peacekeeper.rs`), each beside it. Every kind, unit base, faction, tuning module and client module has its file,
/// and no two share one.
#[test]
fn the_files_mirror_the_def_modules_and_cover_every_part() {
    let layout = &builtin().layout;
    let registry = include_str!("../../../sim/src/defs.rs");
    let start = registry.find("kind_registry! {").expect("the registry");
    let body = &registry[start..start + registry[start..].find("\n}\n").expect("its end")];
    let entries: Vec<(&str, Vec<&str>)> = (body.lines())
        .filter_map(|line| {
            let (kind, path) = line.trim().strip_suffix(',')?.split_once(" => ")?;
            Some((kind, path.split("::").collect()))
        })
        .collect();
    assert_eq!(entries.len(), ObjectKind::COUNT, "every kind of the registry");
    let mut shared = Vec::new();
    for (kind, path) in &entries {
        let (module, folder) = (path[path.len() - 2], path[..path.len() - 2].join("/"));
        let file = layout.file_of(&Part::Rules("defs", (*kind).to_owned()));
        assert_eq!(
            file,
            Some(format!("defs/{folder}/{}.json", snake_case(kind)).as_str()),
            "{kind}"
        );
        if module != snake_case(kind) {
            let owner = entries.iter().find(|(other, _)| snake_case(other) == module);
            assert!(owner.is_some(), "{kind} shares the module {module} of another kind");
            shared.push(*kind);
        }
    }
    assert_eq!(
        shared,
        [
            "PeacekeeperShielded",
            "ArtilleryRigDeployed",
            "OperatorDrone",
            "LaserBatteryDeployed",
            "SubterraneanTransportBurrowed"
        ],
        "the forms beside their kind's file"
    );
    let rules = &builtin().rules;
    let count = |part: &str| rules[part].as_object().expect("a map").len();
    for (section, _) in rules["bases"].as_object().into_iter().flatten() {
        assert!(
            layout.file_of(&Part::Rules("bases", section.clone())).is_some(),
            "{section}"
        );
    }
    let files = layout.files().count();
    assert_eq!(
        files,
        ObjectKind::COUNT + count("bases") + count("factions") + count("tuning") + 4,
        "a file for every part"
    );
    assert_eq!(count("tuning"), 15, "every tuning module of `sc_sim::tuning`");
    for path in [
        "defs/gdo/units/peacekeeper.json",
        "defs/gdo/units/peacekeeper_shielded.json",
        "defs/syndicate/structures/tunnel.json",
        "defs/map/crystal_patch.json",
        "defs/bases/light_infantry.json",
        "defs/cults.json",
        "tuning/combat.json",
        "client/camera.json",
        "client/feedback.json",
        "client/input.json",
        "client/look.json",
    ] {
        assert!(layout.part(path).is_some(), "{path}");
    }
}

/// The round trip: the exported directory reads back as the built-in data, with no file tuned; so does an empty one.
/// Each exported file is pretty-printed, its members in their structs' order (as the def's own serialization lists
/// them), one per line.
#[test]
fn an_exported_directory_reads_back_as_the_built_in_data() {
    let dir = dir("round_trip");
    let count = export(&dir).expect("exported");
    assert_eq!(count, builtin().layout.files().count());
    let read = load(&dir).expect("passes");
    assert!(read.rules == *Ruleset::builtin_ref(), "the built-in rules");
    assert_eq!(read.client, ClientTuning::BUILTIN);
    assert_eq!(read.tuned, Vec::<String>::new());
    let text = fs::read_to_string(dir.join("defs/gdo/units/peacekeeper.json")).expect("the file");
    let def = serde_json::to_string(Ruleset::builtin_ref().def(ObjectKind::Peacekeeper)).expect("serializes");
    let ordered = |text: &str| serde_json::from_str::<Ordered>(text).expect("JSON");
    assert_eq!(ordered(&text), ordered(&def), "its members in the struct's order");
    assert!(
        text.starts_with("{\n  \"kind\": \"Peacekeeper\",\n  \"name\": \"Peacekeeper\",\n"),
        "{text}"
    );
    assert!(text.ends_with("}\n"));
    let _ = fs::remove_dir_all(&dir);
    let empty = self::dir("empty");
    let read = load(&empty).expect("passes");
    assert!(read.rules == *Ruleset::builtin_ref() && read.client == ClientTuning::BUILTIN && read.tuned.is_empty());
    let _ = fs::remove_dir_all(&empty);
}

/// A file that names one number changes that number alone; a full exported file beside it changes nothing and isn't
/// counted as tuned. The client's files tune the client's tuning alike.
#[test]
fn a_patch_changes_what_it_names_alone() {
    let dir = dir("one_number");
    export(&dir).expect("exported");
    write(
        &dir,
        "defs/gdo/units/peacekeeper.json",
        r#"{"unit": {"attack": {"damage": 100}}}"#,
    );
    write(&dir, "client/feedback.json", r#"{"refusal_ticks": 48}"#);
    let read = load(&dir).expect("passes");
    let mut expected = Ruleset::builtin();
    let attack = (expected.defs.get_mut(ObjectKind::Peacekeeper).unit.as_mut()).and_then(|unit| unit.attack.as_mut());
    attack.expect("an attack").damage = 100.0;
    assert!(read.rules == expected, "the Peacekeeper's damage alone");
    assert_eq!(read.client.feedback.refusal_ticks, 48);
    assert_eq!(read.client.camera, ClientTuning::BUILTIN.camera);
    assert_eq!(read.tuned, ["client/feedback.json", "defs/gdo/units/peacekeeper.json"]);
    let _ = fs::remove_dir_all(&dir);
}

/// Each refusal names the file and the member, and says why (`super::load`'s checks): the feedback line's text.
#[test]
fn a_refusal_names_the_file_and_the_member() {
    let said = |refused: Refusal| refused.to_string();
    assert_eq!(
        said(refusal("unknown_file", "defs/gdo/units/peacekeepr.json", "{}")),
        "defs/gdo/units/peacekeepr.json: unknown file"
    );
    assert_eq!(
        said(refusal(
            "unknown_field",
            "defs/gdo/units/peacekeeper.json",
            r#"{"unit": {"attack": {"damag": 5}}}"#
        )),
        "defs/gdo/units/peacekeeper.json: unit.attack.damag: unknown field"
    );
    assert_eq!(
        said(refusal(
            "wrong_type",
            "tuning/combat.json",
            r#"{"minimum_damage": "high"}"#
        )),
        "tuning/combat.json: minimum_damage: expected a number"
    );
    assert_eq!(
        said(refusal(
            "whole",
            "defs/gdo/units/peacekeeper.json",
            r#"{"unit": {"attack": {"aim_ticks": 2.5}}}"#
        )),
        "defs/gdo/units/peacekeeper.json: unit.attack.aim_ticks: expected a whole number"
    );
    assert_eq!(
        said(refusal("client_field", "client/camera.json", r#"{"zoom": 2}"#)),
        "client/camera.json: zoom: unknown field"
    );
    assert_eq!(
        said(refusal(
            "client_sign",
            "client/feedback.json",
            r#"{"refusal_ticks": 0}"#
        )),
        "client/feedback.json: refusal_ticks: must be positive"
    );
    let not_json = said(refusal("not_json", "tuning/combat.json", "{\"minimum_damage\": }"));
    assert!(not_json.starts_with("tuning/combat.json: not JSON: "), "{not_json}");
    assert_eq!(
        said(refusal("not_object", "tuning/combat.json", "[1]")),
        "tuning/combat.json: expected an object"
    );
}

/// The slice's acceptance: a zero speed and a negative cost are refused, the member named; so is a number past what
/// the game is made for (`Ruleset::check`: a recipe's frames), pinned to its member.
#[test]
fn a_zero_speed_and_a_negative_cost_are_refused() {
    let speed = r#"{"unit": {"movement": {"TurnRate": {"max_speed": 0}}}}"#;
    assert_eq!(
        refusal("zero_speed", "defs/gdo/units/peacekeeper.json", speed).to_string(),
        "defs/gdo/units/peacekeeper.json: unit.movement.TurnRate.max_speed: must be positive"
    );
    // The Barracks' recipes, a list that a patch gives whole, with the first one's `member` at `value`.
    let recipes = |member: &str, value: Value| {
        let mut barracks = builtin().rules["defs"]["Barracks"]["structure"]["production"]["recipes"].clone();
        barracks[0][member] = value;
        json!({"structure": {"production": {"recipes": barracks}}}).to_string()
    };
    let cost = recipes("cost", json!(-50));
    assert_eq!(
        refusal("negative_cost", "defs/gdo/structures/barracks.json", &cost).to_string(),
        "defs/gdo/structures/barracks.json: structure.production.recipes[0].cost: must be positive"
    );
    let frames = recipes("frames", json!(70000));
    assert_eq!(
        refusal("frames", "defs/gdo/structures/barracks.json", &frames).to_string(),
        "defs/gdo/structures/barracks.json: structure.production.recipes[0].frames: must be at most 65535 ticks"
    );
    let cheaper = recipes("cost", json!(40));
    let read = data("cheaper", "defs/gdo/structures/barracks.json", &cheaper);
    let production = (read.rules.def(ObjectKind::Barracks).structure.as_ref()).and_then(|s| s.production.as_ref());
    assert_eq!(
        production.expect("production").recipes[0].cost,
        40,
        "within its sign, it passes"
    );
}

/// A change of structure (`Ruleset::structural_diff`) is refused, its file and member named, as needing a change to
/// the Rust default and a rebuild: a footprint's size, an optional value given where the built-in rules have none.
#[test]
fn a_structural_change_needs_a_rebuild() {
    assert_eq!(
        refusal(
            "footprint",
            "defs/gdo/structures/barracks.json",
            r#"{"footprint": {"Tiles": {"w": 4}}}"#
        )
        .to_string(),
        "defs/gdo/structures/barracks.json: footprint.Tiles.w: structural, which needs a change to the Rust default and \
         a rebuild"
    );
    assert_eq!(
        refusal(
            "shields",
            "defs/gdo/structures/barracks.json",
            r#"{"max_shields": 20.0}"#
        )
        .to_string(),
        "defs/gdo/structures/barracks.json: max_shields: structural, which needs a change to the Rust default and a \
         rebuild"
    );
}

/// A refused directory gives nothing (all or nothing): one good file beside a bad one is refused with it. The files
/// an editor or the system keep beside the data are skipped: hidden ones, an editor's backups and locks, and every
/// file that isn't `.json`.
#[test]
fn a_directory_passes_or_is_refused_whole() {
    let dir = dir("all_or_nothing");
    write(&dir, "tuning/combat.json", r#"{"minimum_damage": 1.0}"#);
    write(&dir, "defs/gdo/units/peacekeeper.json", r#"{"max_hp": -1}"#);
    let refused = load(&dir).expect_err("refused");
    assert_eq!(refused.file, "defs/gdo/units/peacekeeper.json", "{refused:?}");
    fs::remove_file(dir.join("defs/gdo/units/peacekeeper.json")).expect("removed");
    for skipped in [
        ".hidden.json",
        "#tuning.json#",
        "tuning/combat.json~",
        "notes.txt",
        "tuning/.combat.json.swp",
    ] {
        write(&dir, skipped, "not JSON");
    }
    let read = load(&dir).expect("passes");
    assert_eq!(read.rules.tuning.combat.minimum_damage, 1.0);
    assert_eq!(read.tuned, ["tuning/combat.json"]);
    let _ = fs::remove_dir_all(&dir);
}

/// The poll on a driven clock, no sleep: nothing before [`POLL_SECONDS`] has passed; then a look, which finds a
/// file changed (here its length) or added once, and nothing more until the files change again; a rebase takes a
/// change as read.
#[test]
fn the_poll_looks_every_half_second() {
    let dir = dir("poll");
    write(&dir, "tuning/combat.json", "{}");
    let mut poll = Poll::new(&dir, 10.0);
    assert!(!poll.changed(10.0 + POLL_SECONDS / 2.0), "not yet time");
    assert!(!poll.changed(10.0 + POLL_SECONDS), "time, nothing changed");
    write(&dir, "tuning/combat.json", r#"{"minimum_damage": 1.0}"#);
    assert!(!poll.changed(10.0 + POLL_SECONDS * 1.5), "changed, but not yet time");
    assert!(poll.changed(10.0 + POLL_SECONDS * 2.0), "the next look finds it");
    assert!(!poll.changed(10.0 + POLL_SECONDS * 3.0), "found once");
    write(&dir, "client/look.json", "{}");
    assert!(poll.changed(10.0 + POLL_SECONDS * 4.0), "a file added");
    write(&dir, "client/look.json", r#"{"cloud_alpha": 50}"#);
    poll.rebase();
    assert!(!poll.changed(10.0 + POLL_SECONDS * 5.0), "a rebased change is read");
    let _ = fs::remove_dir_all(&dir);
}
