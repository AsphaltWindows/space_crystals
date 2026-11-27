//! (M5-X2) Reading a data directory ([`load`]): every `.json` file in it and its folders, each a merge patch over its
//! built-in counterpart (`super::patch`, `super::layout`), into the rules and the client's tuning they give
//! ([`Data`]), or the first refusal ([`Refusal`]), which names the file and the member and says why. A refused
//! directory gives nothing: the game keeps the rules and the tuning it had until the files pass again.
//!
//! The checks, in order, the first problem refused:
//! 1. each file: a file of the layout ("unknown file"), JSON, an object, and the patch's own checks
//!    (`super::patch::check`: unknown fields, kinds of JSON, whole numbers, signs);
//! 2. the whole: the client's tuning and the rules read as their types (`deny_unknown_fields`, an enum's variants, a
//!    number's range), the rules of the built-in rules' structure (`Ruleset::structural_diff`: "structural, which
//!    needs a change to the Rust default and a rebuild"), their numbers finite (`Ruleset::non_finite`) and within what
//!    the game is made for (`Ruleset::check`). A problem of the whole is pinned to its file and member by trying
//!    each changed file alone, then each changed member of it alone ([`locate`]).
//!
//! Files whose names start with `.` or `#` or end with `~` (an editor's) are skipped, and so is every file that isn't
//! `.json`.

use std::fmt;
use std::fs;
use std::path::Path;
use std::sync::LazyLock;

use sc_sim::Ruleset;
use serde_json::Value;

use super::layout::{Layout, Part};
use super::patch::{self, Step};
use crate::tuning::ClientTuning;

/// What a data directory gives (module docs).
#[derive(Clone, Debug, PartialEq)]
pub struct Data {
    /// The rules: the built-in ones where no file changes them.
    pub rules: Ruleset,
    /// The client's feel tunables, likewise.
    pub client: ClientTuning,
    /// The files whose patch changes their counterpart, by path (`/`-separated), in order.
    pub tuned: Vec<String>,
}

/// A refused directory (module docs): the file (`/`-separated, in the directory), the path of the member in it
/// (`unit.attack.damage`; empty for the whole file) and why.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Refusal {
    pub file: String,
    pub path: String,
    pub why: String,
}

impl fmt::Display for Refusal {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self.path.as_str() {
            "" => write!(f, "{}: {}", self.file, self.why),
            path => write!(f, "{}: {path}: {}", self.file, self.why),
        }
    }
}

/// The built-in data as JSON, and their layout: read once.
pub struct Builtin {
    /// The built-in rules' JSON: read back from their text, so that its numbers are those a file's text gives
    /// (an `f32` as its shortest text, not as the `f64` it widens to).
    pub rules: Value,
    /// The client's tuning's, likewise.
    pub client: Value,
    pub layout: Layout,
}

/// The built-in data (module docs of [`Builtin`]).
pub fn builtin() -> &'static Builtin {
    static BUILTIN: LazyLock<Builtin> = LazyLock::new(|| {
        let as_text = |text: String| serde_json::from_str::<Value>(&text).expect("JSON reads back");
        let rules = as_text(serde_json::to_string(Ruleset::builtin_ref()).expect("the rules serialize"));
        let client = as_text(serde_json::to_string(&ClientTuning::BUILTIN).expect("the tuning serializes"));
        let layout = Layout::of(&rules, &client);
        Builtin { rules, client, layout }
    });
    &BUILTIN
}

/// A file of the directory, read and merged onto its counterpart.
struct File {
    path: String,
    part: Part,
    /// Its counterpart merged with its patch.
    merged: Value,
}

/// Reads the data directory `dir` (module docs).
pub fn load(dir: &Path) -> Result<Data, Refusal> {
    let builtin = builtin();
    let mut files = Vec::new();
    for path in json_files(dir)? {
        files.push(read(dir, &path, builtin)?);
    }
    let tuned = (files.iter())
        .filter(|file| file.merged != *counterpart(builtin, &file.part))
        .map(|file| file.path.clone())
        .collect();
    let (rules, client) = assemble(builtin, files.iter().map(|file| (&file.part, &file.merged)));
    let client = match serde_json::from_value::<ClientTuning>(client) {
        Ok(client) => client,
        Err(error) => return Err(locate(builtin, &files, &read_back, error.to_string())),
    };
    let rules = match serde_json::from_value::<Ruleset>(rules) {
        Ok(rules) => rules,
        Err(error) => return Err(locate(builtin, &files, &read_back, error.to_string())),
    };
    if let Some(why) = problem(&rules) {
        return Err(locate(builtin, &files, &read_back, why));
    }
    Ok(Data { rules, client, tuned })
}

/// The `.json` files under `dir`, by path in it, `/`-separated, in order (module docs).
fn json_files(dir: &Path) -> Result<Vec<String>, Refusal> {
    let mut found = Vec::new();
    let mut folders = vec![(dir.to_path_buf(), String::new())];
    while let Some((folder, prefix)) = folders.pop() {
        let entries = fs::read_dir(&folder).map_err(|error| Refusal {
            file: folder.display().to_string(),
            path: String::new(),
            why: format!("can't be read: {error}"),
        })?;
        for entry in entries.flatten() {
            let name = entry.file_name().to_string_lossy().into_owned();
            if name.starts_with('.') || name.starts_with('#') || name.ends_with('~') {
                continue;
            }
            let path = format!("{prefix}{name}");
            if entry.file_type().is_ok_and(|t| t.is_dir()) {
                folders.push((entry.path(), format!("{path}/")));
            } else if name.ends_with(".json") {
                found.push(path);
            }
        }
    }
    found.sort();
    Ok(found)
}

/// The file at `path` in `dir`, checked and merged (module docs, point 1).
fn read(dir: &Path, path: &str, builtin: &Builtin) -> Result<File, Refusal> {
    let refused = |member: &str, why: String| Refusal {
        file: path.to_owned(),
        path: member.to_owned(),
        why,
    };
    let Some(part) = builtin.layout.part(path) else {
        return Err(refused("", "unknown file".to_owned()));
    };
    let text = fs::read_to_string(dir.join(path)).map_err(|error| refused("", format!("can't be read: {error}")))?;
    let patch: Value = serde_json::from_str(&text).map_err(|error| refused("", format!("not JSON: {error}")))?;
    if !patch.is_object() {
        return Err(refused("", "expected an object".to_owned()));
    }
    let counterpart = counterpart(builtin, part);
    if let Some(problem) = patch::check(counterpart, &patch) {
        return Err(refused(&problem.path, problem.why));
    }
    let mut merged = counterpart.clone();
    patch::merge(&mut merged, &patch);
    Ok(File {
        path: path.to_owned(),
        part: part.clone(),
        merged,
    })
}

/// The built-in counterpart of `part`.
fn counterpart<'a>(builtin: &'a Builtin, part: &Part) -> &'a Value {
    match part {
        Part::Rules(section, key) => &builtin.rules[*section][key.as_str()],
        Part::Client(module) => &builtin.client[module.as_str()],
    }
}

/// The rules' and the client's tuning's JSON with `parts` in place of their counterparts.
fn assemble<'a>(builtin: &Builtin, parts: impl Iterator<Item = (&'a Part, &'a Value)>) -> (Value, Value) {
    let (mut rules, mut client) = (builtin.rules.clone(), builtin.client.clone());
    for (part, value) in parts {
        match part {
            Part::Rules(section, key) => rules[*section][key.as_str()] = value.clone(),
            Part::Client(module) => client[module.as_str()] = value.clone(),
        }
    }
    (rules, client)
}

/// What is wrong with the whole that the rules' and the client's tuning's JSON `rules` and `client` make, if anything
/// (module docs, point 2).
fn read_back(rules: &Value, client: &Value) -> Option<String> {
    if let Err(error) = serde_json::from_value::<ClientTuning>(client.clone()) {
        return Some(error.to_string());
    }
    match serde_json::from_value::<Ruleset>(rules.clone()) {
        Ok(rules) => problem(&rules),
        Err(error) => Some(error.to_string()),
    }
}

/// What the game refuses of `rules`, if anything (module docs, point 2).
fn problem(rules: &Ruleset) -> Option<String> {
    if rules.structural_diff(Ruleset::builtin_ref()).is_some() {
        return Some("structural, which needs a change to the Rust default and a rebuild".to_owned());
    }
    if rules.non_finite().is_some() {
        return Some("must be a finite number".to_owned());
    }
    rules.check().map(|invalid| invalid.why.to_owned())
}

/// The refusal of a whole that `test` finds wrong (`why`), pinned to the first changed file that is wrong alone, and
/// to its first changed member that is wrong alone (module docs). Without either, the first changed file, or the
/// directory, takes it.
fn locate(builtin: &Builtin, files: &[File], test: &dyn Fn(&Value, &Value) -> Option<String>, why: String) -> Refusal {
    let changed: Vec<&File> = (files.iter())
        .filter(|file| file.merged != *counterpart(builtin, &file.part))
        .collect();
    for file in &changed {
        let (rules, client) = assemble(builtin, std::iter::once((&file.part, &file.merged)));
        let Some(file_why) = test(&rules, &client) else {
            continue;
        };
        let original = counterpart(builtin, &file.part);
        for (path, leaf) in patch::changes(original, &file.merged) {
            let alone = with_leaf(original, &path, &leaf);
            let (rules, client) = assemble(builtin, std::iter::once((&file.part, &alone)));
            if let Some(leaf_why) = test(&rules, &client) {
                return Refusal {
                    file: file.path.clone(),
                    path: patch::path_text(&path),
                    why: leaf_why,
                };
            }
        }
        return Refusal {
            file: file.path.clone(),
            path: String::new(),
            why: file_why,
        };
    }
    Refusal {
        file: changed.first().map_or_else(|| ".".to_owned(), |file| file.path.clone()),
        path: String::new(),
        why,
    }
}

/// `value` with its leaf at `path` set to `leaf` (`patch::changes`' leaves).
fn with_leaf(value: &Value, path: &[Step], leaf: &Value) -> Value {
    let mut value = value.clone();
    set_leaf(&mut value, path, leaf);
    value
}

fn set_leaf(value: &mut Value, path: &[Step], leaf: &Value) {
    let Some((step, rest)) = path.split_first() else {
        *value = leaf.clone();
        return;
    };
    match (step, value) {
        (Step::Member(key), Value::Object(members)) => {
            set_leaf(members.entry(key.clone()).or_insert(Value::Null), rest, leaf)
        }
        (Step::Element(i), Value::Array(items)) => {
            if let Some(item) = items.get_mut(*i) {
                set_leaf(item, rest, leaf);
            }
        }
        _ => {}
    }
}
