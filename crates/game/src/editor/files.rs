//! Map files: the `maps/` directory at the repository root, listing it, and loading and saving maps in the sim's
//! versioned JSON map format (`ARCHITECTURE.md > Sim scenarios and tests > Map files`).

use std::io::Write;
use std::path::{Path, PathBuf};

use sc_sim::{MapDef, MapSettings};

use crate::cli::load_map_file;

/// The largest map side: the sim's, which loading and validation enforce too.
pub use sc_sim::map::MAX_MAP_SIDE;
/// The shortest side the editor creates or resizes to (a placeholder, see its docs).
pub use sc_sim::scenario::MIN_MAP_SIDE;
/// The name of a new map.
pub const NEW_MAP_NAME: &str = "untitled";
/// Size of the map the editor starts with when it has no file: the creation dialog's default
/// ([`MapSettings::default`]).
pub const NEW_MAP_SIZE: (u32, u32) = (64, 48);

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

/// Reads a map file. The map may still be invalid: check it with `MapDef::validate`.
pub fn load(path: &Path) -> Result<MapDef, String> {
    load_map_file(path)
}

/// Writes `map` as a pretty-printed map file, creating the directory if needed. Atomically (review finding,
/// [`write_atomically`]).
pub fn save(path: &Path, map: &MapDef) -> Result<(), String> {
    let mut json = serde_json::to_string_pretty(map).map_err(|e| format!("can't serialize the map: {e}"))?;
    json.push('\n');
    write_atomically(path, &json)
}

/// Writes `text` to `path`, creating the directory if needed. Atomically: the file is written and flushed to disk
/// under a temporary name next to `path` first ([`temp_path`]), then renamed over `path`, so a crash or a full disk
/// leaves the old file whole instead of a truncated one, and a reader finds the old file or the new one, never a
/// part. Map files and replays (`crate::replay`) are written so.
pub fn write_atomically(path: &Path, text: &str) -> Result<(), String> {
    if let Some(dir) = path.parent().filter(|d| !d.as_os_str().is_empty()) {
        std::fs::create_dir_all(dir).map_err(|e| format!("can't create {}: {e}", dir.display()))?;
    }
    let temp = temp_path(path);
    let written = std::fs::File::create(&temp)
        .and_then(|mut file| {
            file.write_all(text.as_bytes())?;
            file.sync_all()
        })
        .and_then(|()| std::fs::rename(&temp, path));
    written.map_err(|e| {
        let _ = std::fs::remove_file(&temp);
        format!("can't write {}: {e}", path.display())
    })
}

/// Where [`write_atomically`] writes before renaming: a hidden file next to `path` that [`list_maps`] doesn't list,
/// named after this process too, so that two processes writing the same file at once (two games recording to one
/// replay file, say) never write into each other's temporary file: the file is always one of theirs, whole.
pub fn temp_path(path: &Path) -> PathBuf {
    let name = path.file_name().map_or_else(|| "map".into(), |n| n.to_string_lossy());
    path.with_file_name(format!(".{name}.{}.saving", std::process::id()))
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

/// The map the editor starts with when it has no file: the creation dialog's default settings
/// ([`MapSettings::default`]) under the name `name` (or [`NEW_MAP_NAME`] if `name` is blank).
pub fn new_map(name: &str) -> MapDef {
    let name = if name.trim().is_empty() { NEW_MAP_NAME } else { name };
    MapSettings {
        name: name.into(),
        ..MapSettings::default()
    }
    .create()
    .expect("the default settings make a map")
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
    use sc_sim::{Faction, MapType, ObjectKind, ScenarioProblem, Symmetry, scenarios};

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
        let map = new_map("My Map");
        save(&path, &map).unwrap();
        assert_eq!(path_for_new_map(&dir, "My Map"), dir.join("my_map-2.json"));
        std::fs::write(dir.join("notes.txt"), "not a map").unwrap();
        assert_eq!(list_maps(&dir), vec![path.clone()]);
        assert_eq!(load(&path), Ok(map));
        std::fs::remove_dir_all(&dir).unwrap();
    }

    /// The map the editor starts with is the creation dialog's default: a 64x48 Skirmish Default map for two with
    /// 180 degree symmetry. Its Starting Layouts start empty (design change 0009), so all it lacks to be played is
    /// the GDO's, the Syndicate's and the Colonists' at each start (0012; plan decision T24; slice M4-B).
    #[test]
    fn a_new_map_has_the_default_settings() {
        let map = new_map(" ");
        assert_eq!(map.name, NEW_MAP_NAME);
        assert_eq!((map.terrain.width(), map.terrain.height()), NEW_MAP_SIZE);
        let defaults = MapSettings::default();
        assert_eq!(NEW_MAP_SIZE, (defaults.width, defaults.height));
        assert_eq!(
            (map.map_type, map.symmetry, map.slots.len()),
            (MapType::SkirmishDefault, Symmetry::Rotate180, 2)
        );
        let missing: Vec<_> = map
            .validate()
            .into_iter()
            .map(|p| match p {
                ScenarioProblem::LayoutMissing {
                    slot, faction, kind, ..
                } => (slot.0, faction, kind),
                other => panic!("{other}"),
            })
            .collect();
        let (dc, ef, plate) = (
            ObjectKind::DeploymentCenter,
            ObjectKind::ExtractionFacility,
            ObjectKind::ExtractionPlate,
        );
        let (node, portal, drone) = (ObjectKind::EgressNode, ObjectKind::RoboPortal, ObjectKind::Drone);
        let (tunnel, hideout, agent) = (ObjectKind::Tunnel, ObjectKind::Hideout, ObjectKind::Agent);
        let (gdo, syndicate, colonists) = (Faction::Gdo, Faction::Syndicate, Faction::Colonists);
        assert_eq!(
            missing,
            [
                (1, gdo, dc),
                (1, gdo, ef),
                (1, gdo, plate),
                (1, syndicate, tunnel),
                (1, syndicate, hideout),
                (1, syndicate, agent),
                (1, colonists, node),
                (1, colonists, portal),
                (1, colonists, drone),
                (2, gdo, dc),
                (2, gdo, ef),
                (2, gdo, plate),
                (2, syndicate, tunnel),
                (2, syndicate, hideout),
                (2, syndicate, agent),
                (2, colonists, node),
                (2, colonists, portal),
                (2, colonists, drone)
            ]
        );
    }

    /// Review finding: saving replaces the file whole or not at all, and leaves no temporary file.
    #[test]
    fn saving_is_atomic() {
        let dir = temp_dir("atomic");
        let path = dir.join("map.json");
        let first = scenarios::empty_map(20, 16);
        save(&path, &first).unwrap();
        let mut second = first.clone();
        second.description = "second".into();
        save(&path, &second).unwrap();
        assert_eq!(load(&path), Ok(second.clone()));
        assert!(!temp_path(&path).exists());
        assert_eq!(list_maps(&dir), vec![path.clone()]);

        // When the new file can't be written, the old one stays as it was.
        std::fs::create_dir_all(temp_path(&path).join("blocked")).unwrap();
        let mut third = second.clone();
        third.description = "third".into();
        let error = save(&path, &third).unwrap_err();
        assert!(error.contains("can't write"), "{error}");
        assert_eq!(load(&path), Ok(second));
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
