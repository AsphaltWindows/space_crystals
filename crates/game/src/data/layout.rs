//! (M5-X2) Which data file holds which part of the rules or of the client's tuning ([`Layout`]), as the modules hold
//! them (plan decision T66):
//!
//! - `defs/<faction>/units/<object>.json` and `defs/<faction>/structures/<object>.json`: an object type's def
//!   (`Ruleset::defs`), by its faction and whether it is a unit, its file named after the kind in snake case
//!   (`Peacekeeper`'s `defs/gdo/units/peacekeeper.json`); a type of no faction's in `defs/map/<object>.json`. So each
//!   file mirrors its def module, `crates/sim/src/defs/<faction>/{units,structures}/<object>.rs` and
//!   `crates/sim/src/defs/map/<object>.rs` (the kind registry, `sc_sim::defs`), but for the forms a kind turns into,
//!   which have a file of their own beside the module they share (`peacekeeper_shielded.json`);
//! - `defs/<faction>.json`: a faction's own rules (`Ruleset::factions`: the Cults' with their technologies and
//!   abilities);
//! - `defs/bases/<base>.json`: a unit base (`Ruleset::bases`);
//! - `tuning/<system>.json`: a system's tunables (`Ruleset::tuning`, `sc_sim::tuning`);
//! - `client/<module>.json`: the client's feel tunables (`crate::tuning::ClientTuning`).
//!
//! The layout is read off the JSON of the built-in rules and tuning, so a kind, a base, a faction or a module that a
//! later slice adds has its file with no change here.

use std::collections::BTreeMap;

use serde_json::Value;

/// Where a file's part lies: the rules' part (`defs`, `bases`, `factions` or `tuning`) and its key there, or the
/// client's tuning's module.
#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub enum Part {
    Rules(&'static str, String),
    Client(String),
}

/// Every data file, by its path in the data directory (`/`-separated), and its part (module docs).
#[derive(Clone, Debug, Default)]
pub struct Layout {
    files: BTreeMap<String, Part>,
}

impl Layout {
    /// The layout of `rules` and `client`, the rules' and the client's tuning's JSON (module docs). Panics if two
    /// parts would share a file, which a test rules out.
    pub fn of(rules: &Value, client: &Value) -> Layout {
        let mut layout = Layout::default();
        for (kind, def) in members(rules, "defs") {
            let faction = def.get("faction").and_then(Value::as_str).map(snake_case);
            let path = match faction {
                Some(faction) => {
                    let folder = if def.get("unit").is_some_and(|unit| !unit.is_null()) {
                        "units"
                    } else {
                        "structures"
                    };
                    format!("defs/{faction}/{folder}/{}.json", snake_case(kind))
                }
                None => format!("defs/map/{}.json", snake_case(kind)),
            };
            layout.add(path, Part::Rules("defs", kind.clone()));
        }
        for (base, _) in members(rules, "bases") {
            layout.add(
                format!("defs/bases/{}.json", snake_case(base)),
                Part::Rules("bases", base.clone()),
            );
        }
        for (faction, _) in members(rules, "factions") {
            layout.add(format!("defs/{faction}.json"), Part::Rules("factions", faction.clone()));
        }
        for (system, _) in members(rules, "tuning") {
            layout.add(format!("tuning/{system}.json"), Part::Rules("tuning", system.clone()));
        }
        for (module, _) in client.as_object().into_iter().flatten() {
            layout.add(format!("client/{module}.json"), Part::Client(module.clone()));
        }
        layout
    }

    fn add(&mut self, path: String, part: Part) {
        if let Some(other) = self.files.insert(path.clone(), part.clone()) {
            panic!("{path} would hold both {other:?} and {part:?}");
        }
    }

    /// Every file and its part, by path.
    pub fn files(&self) -> impl Iterator<Item = (&str, &Part)> {
        self.files.iter().map(|(path, part)| (path.as_str(), part))
    }

    /// The part of the file at `path` (`/`-separated, in the data directory), if it is a data file.
    pub fn part(&self, path: &str) -> Option<&Part> {
        self.files.get(path)
    }

    /// The file of `part`.
    pub fn file_of(&self, part: &Part) -> Option<&str> {
        self.files().find(|(_, p)| *p == part).map(|(path, _)| path)
    }
}

/// The members of the rules' part `part`, a map.
fn members<'a>(rules: &'a Value, part: &str) -> impl Iterator<Item = (&'a String, &'a Value)> {
    (rules.get(part).and_then(Value::as_object)).into_iter().flatten()
}

/// `name` in snake case: `PeacekeeperShielded`'s `peacekeeper_shielded`, `Gdo`'s `gdo`.
pub fn snake_case(name: &str) -> String {
    let mut snake = String::with_capacity(name.len() + 4);
    let mut previous: Option<char> = None;
    for c in name.chars() {
        if c.is_ascii_uppercase() {
            if previous.is_some_and(|p| p.is_ascii_lowercase() || p.is_ascii_digit()) {
                snake.push('_');
            }
            snake.push(c.to_ascii_lowercase());
        } else {
            snake.push(c);
        }
        previous = Some(c);
    }
    snake
}
