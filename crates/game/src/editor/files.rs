//! Map files: the `maps/` directory at the repository root, listing it, and loading and saving scenarios in the
//! sim's JSON scenario format (`ARCHITECTURE.md > Sim scenarios and tests > Scenario files`).

use std::path::{Path, PathBuf};

use sc_sim::{Scenario, scenarios};

use crate::cli::load_scenario_file;

/// Size of a new map (placeholder until Q0004 settles map sizes).
pub const NEW_MAP_SIZE: (u32, u32) = (64, 48);
/// The largest map side the editor makes (placeholder until Q0004 settles map sizes).
pub const MAX_MAP_SIDE: u32 = 256;
/// The name of a new map.
pub const NEW_MAP_NAME: &str = "untitled";

/// `maps/` at the repository root, where new maps are saved and the files panel looks.
pub fn default_maps_dir() -> PathBuf {
    // The client crate lives in `crates/game`.
    let manifest = Path::new(env!("CARGO_MANIFEST_DIR"));
    manifest
        .parent()
        .and_then(Path::parent)
        .map_or_else(|| PathBuf::from("maps"), |root| root.join("maps"))
}

/// The `.json` files in `dir`, sorted by name. A missing directory has none.
pub fn list_maps(dir: &Path) -> Vec<PathBuf> {
    let Ok(entries) = std::fs::read_dir(dir) else {
        return Vec::new();
    };
    let mut maps: Vec<PathBuf> = entries
        .filter_map(|e| Some(e.ok()?.path()))
        .filter(|p| p.is_file() && p.extension().is_some_and(|x| x == "json"))
        .collect();
    maps.sort();
    maps
}

/// Reads a scenario file. It may still be invalid: check it with `Scenario::validate`.
pub fn load(path: &Path) -> Result<Scenario, String> {
    load_scenario_file(path)
}

/// Writes `scenario` as a pretty-printed scenario file, creating the directory if needed.
pub fn save(path: &Path, scenario: &Scenario) -> Result<(), String> {
    if let Some(dir) = path.parent().filter(|d| !d.as_os_str().is_empty()) {
        std::fs::create_dir_all(dir).map_err(|e| format!("can't create {}: {e}", dir.display()))?;
    }
    let mut json = serde_json::to_string_pretty(scenario).map_err(|e| format!("can't serialize the scenario: {e}"))?;
    json.push('\n');
    std::fs::write(path, json).map_err(|e| format!("can't write {}: {e}", path.display()))
}

/// A file name for a map called `name`: lower case letters, digits, `-` and `_`.
pub fn slug(name: &str) -> String {
    let mut slug = String::new();
    for c in name.trim().chars() {
        if c.is_ascii_alphanumeric() || c == '-' || c == '_' {
            slug.push(c.to_ascii_lowercase());
        } else if !slug.ends_with('_') {
            slug.push('_');
        }
    }
    let slug = slug.trim_matches('_');
    if slug.is_empty() {
        NEW_MAP_NAME.into()
    } else {
        slug.into()
    }
}

/// Where a map called `name` that has no file yet is saved: `dir/<slug>.json`, numbered if that file exists.
pub fn path_for_new_map(dir: &Path, name: &str) -> PathBuf {
    let slug = slug(name);
    (1..)
        .map(|n| match n {
            1 => dir.join(format!("{slug}.json")),
            n => dir.join(format!("{slug}-{n}.json")),
        })
        .find(|p| !p.exists())
        .expect("some file name is free")
}

/// A new flat map with the two player slots of `scenarios::empty`.
pub fn new_scenario(name: &str, width: u32, height: u32) -> Scenario {
    Scenario {
        name: name.into(),
        ..scenarios::empty(width, height)
    }
}

/// `path` relative to the repository root when it lies inside it, for display.
pub fn display_path(path: &Path) -> String {
    let root = default_maps_dir();
    let root = root.parent().unwrap_or(Path::new(""));
    path.strip_prefix(root).unwrap_or(path).display().to_string()
}

#[cfg(test)]
mod tests {
    use super::*;

    fn temp_dir(tag: &str) -> PathBuf {
        let dir = std::env::temp_dir().join(format!("sc_editor_files_{tag}_{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&dir);
        dir
    }

    #[test]
    fn slugs_are_file_name_safe() {
        assert_eq!(slug("Dev Skirmish"), "dev_skirmish");
        assert_eq!(slug("  ../../etc/passwd "), "etc_passwd");
        assert_eq!(slug("a--b__c"), "a--b__c");
        assert_eq!(slug("???"), NEW_MAP_NAME);
    }

    #[test]
    fn new_maps_get_a_free_file_name_and_save_and_list() {
        let dir = temp_dir("new");
        assert!(list_maps(&dir).is_empty(), "a missing directory lists nothing");
        let path = path_for_new_map(&dir, "My Map");
        assert_eq!(path, dir.join("my_map.json"));
        let scenario = new_scenario("My Map", 12, 9);
        save(&path, &scenario).unwrap();
        assert_eq!(path_for_new_map(&dir, "My Map"), dir.join("my_map-2.json"));
        std::fs::write(dir.join("notes.txt"), "not a map").unwrap();
        assert_eq!(list_maps(&dir), vec![path.clone()]);
        assert_eq!(load(&path), Ok(scenario));
        std::fs::remove_dir_all(&dir).unwrap();
    }

    #[test]
    fn the_maps_directory_is_at_the_repository_root() {
        let dir = default_maps_dir();
        assert!(dir.ends_with("maps"));
        assert!(dir.parent().unwrap().join("Cargo.toml").is_file());
        assert_eq!(display_path(&dir.join("x.json")), "maps/x.json");
    }
}
