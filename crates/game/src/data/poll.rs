//! (M5-X2) Watching a data directory for changes with no new dependency ([`Poll`]): every [`POLL_SECONDS`], each of
//! its files' modification time and length ([`Snapshot`]) against the last look's. The poll's clock is its caller's,
//! so that a test drives it with no sleep: the game's is its real time (`Time<Real>`).

use std::collections::BTreeMap;
use std::fs;
use std::path::{Path, PathBuf};
use std::time::SystemTime;

/// How often the directory is looked at, seconds (slice M5-X2's brief: "polled every 0.5 s by modification time").
pub const POLL_SECONDS: f64 = 0.5;

/// What a directory's files were at a look: each file's path, modification time and length, its folders' included.
/// An editor that saves a file changes its time, its length or both; a file added or removed changes the set.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct Snapshot(BTreeMap<PathBuf, (Option<SystemTime>, u64)>);

impl Snapshot {
    /// The files under `dir` now; none for a directory that can't be read.
    pub fn of(dir: &Path) -> Snapshot {
        let mut files = BTreeMap::new();
        let mut folders = vec![dir.to_path_buf()];
        while let Some(folder) = folders.pop() {
            let Ok(entries) = fs::read_dir(&folder) else {
                continue;
            };
            for entry in entries.flatten() {
                let Ok(meta) = entry.metadata() else {
                    continue;
                };
                if meta.is_dir() {
                    folders.push(entry.path());
                } else {
                    files.insert(entry.path(), (meta.modified().ok(), meta.len()));
                }
            }
        }
        Snapshot(files)
    }
}

/// The poll of one directory (module docs).
#[derive(Clone, Debug)]
pub struct Poll {
    dir: PathBuf,
    /// The clock's time of the next look, seconds.
    next: f64,
    last: Snapshot,
}

impl Poll {
    /// A poll of `dir`, which looks first [`POLL_SECONDS`] after `now`, against the files as they are now.
    pub fn new(dir: &Path, now: f64) -> Poll {
        Poll {
            dir: dir.to_path_buf(),
            next: now + POLL_SECONDS,
            last: Snapshot::of(dir),
        }
    }

    /// Whether the directory changed since the last look, looked at if it is time to at `now`, the clock's time in
    /// seconds: `false` until then.
    pub fn changed(&mut self, now: f64) -> bool {
        if now < self.next {
            return false;
        }
        self.next = now + POLL_SECONDS;
        let snapshot = Snapshot::of(&self.dir);
        if snapshot == self.last {
            return false;
        }
        self.last = snapshot;
        true
    }

    /// Takes the files as they are now as the last look's, so that a change already read (by a reload of F8) isn't
    /// read again.
    pub fn rebase(&mut self) {
        self.last = Snapshot::of(&self.dir);
    }
}
