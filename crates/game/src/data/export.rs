//! (M5-X2) `--export-data <dir>`: the built-in rules and the client's tuning written as data files
//! (`super::layout`), each pretty-printed with its members in their structs' order (`super::ordered`), so that a
//! file reads as its def module does and a change to it is a small diff. Each is a valid patch that changes nothing
//! (`super::load`).

use std::fs;
use std::path::Path;

use sc_sim::Ruleset;

use super::layout::Part;
use super::load::builtin;
use super::ordered::Ordered;
use crate::tuning::ClientTuning;

/// Writes every data file into `dir`, creating it and its folders; the files' count, or what failed.
pub fn export(dir: &Path) -> Result<usize, String> {
    let rules = Ordered::of(Ruleset::builtin_ref());
    let client = Ordered::of(&ClientTuning::BUILTIN);
    let mut count = 0;
    for (path, part) in builtin().layout.files() {
        let value = match part {
            Part::Rules(section, key) => rules.get(section).and_then(|section| section.get(key)),
            Part::Client(module) => client.get(module),
        };
        let value = value.expect("the layout is the built-in data's");
        let file = dir.join(path);
        if let Some(folder) = file.parent() {
            fs::create_dir_all(folder).map_err(|error| format!("{}: {error}", folder.display()))?;
        }
        let mut text = serde_json::to_string_pretty(value).expect("JSON serializes");
        text.push('\n');
        fs::write(&file, text).map_err(|error| format!("{}: {error}", file.display()))?;
        count += 1;
    }
    Ok(count)
}
