//! (Slice M5-X2) The data files' flags (`crate::data`, `docs/tuning.md`): `--export-data <dir>`, which writes the
//! built-in data and exits ([`export_data`]), and `--tuning <dir>`, a game's data directory ([`tuning_dir`]).

use std::path::{Path, PathBuf};

use super::CliOptions;

/// `--export-data <dir>`: every data file of the built-in rules and tuning, written into `dir`; what to print, or
/// what failed.
pub fn export_data(dir: &Path) -> Result<String, String> {
    let count = crate::data::export(dir)?;
    Ok(format!(
        "Exported {count} data files to {}: edit any, then play with --tuning {} (docs/tuning.md)",
        dir.display(),
        dir.display()
    ))
}

/// `--tuning <dir>`'s directory, checked against the other flags: a game's only (not the map editor's, whose
/// play-tests start over, nor a replay's, which brings its rules in its commands), with debug commands on (the rules
/// go to the game as a debug command), and a directory that exists.
pub fn tuning_dir(options: &CliOptions) -> Result<Option<PathBuf>, String> {
    let Some(dir) = &options.tuning else {
        return Ok(None);
    };
    let conflicts = [
        (options.editor, "--editor"),
        (options.replay.is_some(), "--replay"),
        (!options.debug, "--no-debug"),
    ];
    if let Some((_, flag)) = conflicts.into_iter().find(|(given, _)| *given) {
        return Err(format!(
            "--tuning gives a game its rules as a debug command, from its first tick on: not with {flag}"
        ));
    }
    if !dir.is_dir() {
        return Err(format!(
            "--tuning {}: no such directory (--export-data {} writes one)",
            dir.display(),
            dir.display()
        ));
    }
    Ok(Some(dir.clone()))
}
