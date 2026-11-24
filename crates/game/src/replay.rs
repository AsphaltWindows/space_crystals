//! Replays (`ARCHITECTURE.md > Replays and golden hashes`): the client records every game it plays ([`record`]) and
//! plays recordings back (`--replay`, [`playback`]).
//!
//! A replay file is the sim's [`Replay`] as JSON: the scenario (the map and the game setup, game options included),
//! the inputs of every tick and the state-hash checkpoints that tell a playback whether it still matches the
//! recorded game. [`replay_json`] writes it with one line per tick, the layout of the sim's golden files
//! (`crates/sim/tests/golden/`), which the client plays back too.

pub mod playback;
pub mod record;

use std::path::{Path, PathBuf};
use std::time::{SystemTime, UNIX_EPOCH};

use sc_sim::Replay;

pub use playback::{Playback, PlaybackPlugin};
pub use record::{RecordPlugin, Recorder};

use crate::editor::files;

/// Where games are recorded unless `--record` names a file: `replays/` under the working directory.
pub const REPLAYS_DIR: &str = "replays";

/// `replay` as JSON: the scenario pretty-printed, then one line per tick's inputs and one per checkpoint.
pub fn replay_json(replay: &Replay) -> Result<String, String> {
    let error = |e: serde_json::Error| format!("can't serialize the replay: {e}");
    let scenario = serde_json::to_string_pretty(&replay.scenario)
        .map_err(error)?
        .replace('\n', "\n  ");
    let lines = |items: Vec<String>| items.join(",\n    ");
    let inputs = replay
        .inputs
        .iter()
        .map(serde_json::to_string)
        .collect::<Result<_, _>>()
        .map_err(error)?;
    let checkpoints = replay
        .checkpoints
        .iter()
        .map(serde_json::to_string)
        .collect::<Result<_, _>>()
        .map_err(error)?;
    Ok(format!(
        "{{\n  \"scenario\": {scenario},\n  \"inputs\": [\n    {}\n  ],\n  \"checkpoints\": [\n    {}\n  ]\n}}\n",
        lines(inputs),
        lines(checkpoints)
    ))
}

/// Writes `replay` to `path` ([`replay_json`]), atomically ([`files::write_atomically`]): a crash while saving
/// leaves the previous save whole.
pub fn save_replay(path: &Path, replay: &Replay) -> Result<(), String> {
    files::write_atomically(path, &replay_json(replay)?)
}

/// Reads a replay file. Its checkpoints must be in ascending tick order and none after its end; its scenario may
/// still be invalid (`Scenario::validate`).
pub fn load_replay(path: &Path) -> Result<Replay, String> {
    let text = std::fs::read_to_string(path).map_err(|e| format!("can't read {}: {e}", path.display()))?;
    let replay: Replay =
        serde_json::from_str(&text).map_err(|e| format!("{} is not a replay file: {e}", path.display()))?;
    let ordered = replay.checkpoints.windows(2).all(|pair| pair[0].tick < pair[1].tick);
    if !ordered || replay.checkpoints.last().is_some_and(|c| c.tick > replay.end_tick()) {
        return Err(format!(
            "{}: the checkpoints must be in ascending tick order, none after the last tick ({})",
            path.display(),
            replay.end_tick()
        ));
    }
    Ok(replay)
}

/// The file a game on the map `map_name` that starts at `now` is recorded to, unless `--record` names one:
/// `replays/<UTC date and time>_<map>.json` ([`utc_timestamp`], [`files::slug`]), with `_2`, `_3`, … added if
/// that file exists already.
pub fn recording_path(now: SystemTime, map_name: &str) -> PathBuf {
    let stem = format!("{}_{}", utc_timestamp(now), files::slug(map_name));
    let dir = Path::new(REPLAYS_DIR);
    let mut path = dir.join(format!("{stem}.json"));
    let mut n = 2;
    while path.exists() {
        path = dir.join(format!("{stem}_{n}.json"));
        n += 1;
    }
    path
}

/// `now` as a UTC date and time for a file name: `2026-11-23_20-43-05`.
pub fn utc_timestamp(now: SystemTime) -> String {
    let seconds = now.duration_since(UNIX_EPOCH).map_or(0, |d| d.as_secs());
    let (days, time) = (seconds / 86_400, seconds % 86_400);
    let (year, month, day) = civil_from_days(days);
    format!(
        "{year:04}-{month:02}-{day:02}_{:02}-{:02}-{:02}",
        time / 3600,
        time % 3600 / 60,
        time % 60
    )
}

/// The proleptic Gregorian date `days` days after 1970-01-01 (Howard Hinnant's `civil_from_days`).
fn civil_from_days(days: u64) -> (u64, u64, u64) {
    let z = days + 719_468;
    let era = z / 146_097;
    let doe = z % 146_097;
    let yoe = (doe - doe / 1460 + doe / 36_524 - doe / 146_096) / 365;
    let doy = doe - (365 * yoe + yoe / 4 - yoe / 100);
    let mp = (5 * doy + 2) / 153;
    let day = doy - (153 * mp + 2) / 5 + 1;
    let month = if mp < 10 { mp + 3 } else { mp - 9 };
    let year = yoe + era * 400 + u64::from(month <= 2);
    (year, month, day)
}

#[cfg(test)]
mod tests {
    use std::time::Duration;

    use sc_sim::testing::every_command;
    use sc_sim::{Checkpoint, PlayerCommand, PlayerId, TickInputs, scenarios};

    use super::*;

    fn at(seconds: u64) -> SystemTime {
        UNIX_EPOCH + Duration::from_secs(seconds)
    }

    #[test]
    fn timestamps_are_utc_dates_and_times() {
        assert_eq!(utc_timestamp(at(0)), "1970-01-01_00-00-00");
        assert_eq!(utc_timestamp(at(1_700_000_000)), "2023-11-14_22-13-20");
        assert_eq!(utc_timestamp(at(951_782_400)), "2000-02-29_00-00-00");
        assert_eq!(utc_timestamp(at(951_868_799)), "2000-02-29_23-59-59");
        assert_eq!(utc_timestamp(at(4_107_542_400)), "2100-03-01_00-00-00");
    }

    #[test]
    fn recordings_are_named_after_the_time_and_the_map() {
        let path = recording_path(at(1_700_000_000), "Frozen Lake");
        assert_eq!(path, Path::new("replays/2023-11-14_22-13-20_frozen_lake.json"));
    }

    #[test]
    fn replay_files_round_trip_and_checkpoints_are_checked() {
        let dir = std::env::temp_dir().join(format!("sc_replay_files_{}", std::process::id()));
        let path = dir.join("game.json");
        let mut replay = Replay::new(scenarios::empty(6, 4));
        let start = replay.start();
        replay.record_checked(None, &start);
        replay.record(Default::default());
        replay.record(Default::default());
        replay.checkpoints.push(Checkpoint { tick: 2, hash: 7 });
        save_replay(&path, &replay).unwrap();
        let text = std::fs::read_to_string(&path).unwrap();
        let loaded = load_replay(&path);
        replay.checkpoints.push(Checkpoint { tick: 1, hash: 7 });
        save_replay(&path, &replay).unwrap();
        let unordered = load_replay(&path);
        replay.checkpoints.pop();
        replay.checkpoints.push(Checkpoint { tick: 3, hash: 7 });
        save_replay(&path, &replay).unwrap();
        let late = load_replay(&path);
        std::fs::remove_dir_all(&dir).unwrap();
        assert!(
            text.contains("\n    {\"commands\":[]},\n    {\"commands\":[]}\n"),
            "{text}"
        );
        let loaded = loaded.unwrap();
        let first = Checkpoint {
            tick: 0,
            hash: start.state_hash(),
        };
        assert_eq!(loaded.checkpoints, [first, Checkpoint { tick: 2, hash: 7 }]);
        assert_eq!(loaded.end_tick(), 2);
        for error in [unordered, late] {
            assert!(error.unwrap_err().contains("ascending tick order"));
        }
    }

    /// A replay file keeps every kind of command a player can give (`testing::every_command`), one tick a line.
    #[test]
    fn replay_files_keep_every_kind_of_command() {
        let dir = std::env::temp_dir().join(format!("sc_replay_commands_{}", std::process::id()));
        let path = dir.join("game.json");
        let mut replay = Replay::new(scenarios::empty(6, 4));
        for command in every_command() {
            let player = PlayerId(2);
            replay.record(TickInputs {
                commands: vec![PlayerCommand { player, command }],
            });
        }
        save_replay(&path, &replay).unwrap();
        let text = std::fs::read_to_string(&path).unwrap();
        let loaded = load_replay(&path);
        std::fs::remove_dir_all(&dir).unwrap();
        assert_eq!(loaded.unwrap(), replay);
        let lines = text.lines().filter(|line| line.contains("{\"commands\":[{")).count();
        assert_eq!(lines, every_command().len());
    }
}
