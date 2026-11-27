use sc_ai::{Profile, factions};
use sc_sim::{Faction, PlayerId};

use crate::ai_players::AiSlot;
use crate::cli::{into_config, parse};

fn args(s: &str) -> Vec<String> {
    s.split_whitespace().map(str::to_string).collect()
}

fn slot(player: u8, profile: Profile) -> AiSlot {
    AiSlot {
        player: PlayerId(player),
        profile,
    }
}

/// `--ai`: players by number, each with a profile after a colon (any case: standard, rush or passive) or the standard
/// one.
#[test]
fn ai_flags_parse() {
    let ai = |flags: &str| parse(&args(flags)).unwrap().ai;
    assert_eq!(ai(""), []);
    assert_eq!(ai("--ai 2"), [slot(2, Profile::Standard)]);
    assert_eq!(
        ai("--ai 1,2:passive"),
        [slot(1, Profile::Standard), slot(2, Profile::Passive)]
    );
    assert_eq!(
        ai("--ai 2:PASSIVE,1:standard"),
        [slot(2, Profile::Passive), slot(1, Profile::Standard)]
    );
    assert_eq!(
        ai("--ai 1:rush,2:Rush"),
        [slot(1, Profile::Rush), slot(2, Profile::Rush)]
    );
}

/// The help's `--ai` lines name every profile (slice A2d added the rush).
#[test]
fn the_usage_names_every_profile() {
    let text = crate::cli::usage();
    let from = text.find("--ai <n>").unwrap();
    let to = from + text[from..].find("--window-size").unwrap();
    let lines = text[from..to].split_whitespace().collect::<Vec<_>>().join(" ");
    assert!(
        lines.contains("each with a profile: standard, rush or passive (default standard); e.g. --ai 2, --ai 2:rush"),
        "{lines}"
    );
}

/// A player that isn't a number, a profile there isn't (the error lists those there are), and a player named twice.
#[test]
fn bad_ai_flags_are_errors() {
    let error = |flags: &str| parse(&args(flags)).unwrap_err();
    for bad in ["--ai", "--ai 0", "--ai x", "--ai 2:", "--ai ,2", "--ai 1,"] {
        assert!(parse(&args(bad)).is_err(), "{bad}");
    }
    let blitz = error("--ai 2:blitz");
    assert!(
        blitz.contains("\"blitz\"") && blitz.contains("standard, rush, passive"),
        "{blitz}"
    );
    assert_eq!(error("--ai 1,2,1:passive"), "--ai names player 1 twice");
}

/// The computer players are checked against the game: its players, and their factions' computer players. `--ai`
/// needs a game: not the editor, nor a replay's playback, whose recorded commands are the computers' own.
#[test]
fn ai_players_are_checked_against_the_game() {
    let config = |flags: &str| into_config(parse(&args(flags)).unwrap());
    let ai = config("--factions cults,gdo --ai 2").unwrap().ai;
    assert_eq!(ai, [slot(2, Profile::Standard)]);
    assert_eq!(
        config("--factions gdo,gdo --ai 1,2:passive").unwrap().ai,
        [slot(1, Profile::Standard), slot(2, Profile::Passive)]
    );
    if let Some(lacking) = Faction::ALL.iter().copied().find(|&f| !factions::has_ai(f)) {
        let name = factions::name(lacking);
        let error = config(&format!("--factions {name},gdo --ai 1")).unwrap_err();
        let expected = format!("--ai: player 1 plays {name}, which has no computer player yet");
        assert!(error.starts_with(&expected), "{error}");
    }
    assert_eq!(config("--ai 3").unwrap_err(), "--ai: there is no player 3 in this game");
    assert!(config("--editor --ai 1").unwrap_err().contains("--ai needs a game"));
    let replay = config("--replay nowhere.json --ai 2").unwrap_err();
    assert!(replay.contains("not with --ai"), "{replay}");
    assert_eq!(config("").unwrap().ai, [], "no computer players by default");
}
