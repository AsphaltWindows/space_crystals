//! The faction seam (`sc_ai::factions`): a computer player takes a player's slot only if the player's faction has one,
//! and says why not otherwise (`core/game_rules.md > Computer players (playtest tool)`: "taking any player's slot").

use sc_ai::{Ai, AiError, Profile, factions};
use sc_sim::{Faction, GameSetup, PlayerId, Scenario, Sim, scenarios};

/// `dev_skirmish` with player 1 playing `a` and player 2 `b`, as `--factions a,b` starts it.
fn game(a: Faction, b: Faction) -> Scenario {
    let mut scenario = scenarios::dev_skirmish();
    scenario.setup = GameSetup::per_slot(&scenario.map, &[a, b], scenario.setup.seed);
    scenario
}

/// A faction without a computer player (while there is one) is refused with its name and the names of those that
/// have one; the GDO's takes its slot, with its profile; a player the game lacks is refused.
#[test]
fn a_computer_player_takes_only_a_slot_whose_faction_has_one() {
    if let Some(lacking) = Faction::ALL.iter().copied().find(|&f| !factions::has_ai(f)) {
        let scenario = game(lacking, Faction::Gdo);
        let sim = Sim::new(&scenario);
        let refused = Ai::new(&scenario, &sim, PlayerId(1), Profile::Standard).unwrap_err();
        let faction = lacking;
        assert_eq!(
            refused,
            AiError::NoAi {
                player: PlayerId(1),
                faction
            }
        );
        let message = refused.to_string();
        let with: Vec<String> = factions::with_ai().into_iter().map(factions::name).collect();
        let expected = format!(
            "player 1 plays {}, which has no computer player yet (computer players play: {})",
            factions::name(lacking),
            with.join(", ")
        );
        assert_eq!(message, expected);
    }
    let scenario = game(Faction::Gdo, Faction::Gdo);
    let sim = Sim::new(&scenario);
    let ai = Ai::new(&scenario, &sim, PlayerId(2), Profile::Passive).expect("the GDO has a computer player");
    assert_eq!((ai.player(), ai.profile()), (PlayerId(2), Profile::Passive));
    let unknown = Ai::new(&scenario, &sim, PlayerId(3), Profile::Standard).unwrap_err();
    assert_eq!(unknown, AiError::UnknownPlayer(PlayerId(3)));
    assert_eq!(unknown.to_string(), "there is no player 3 in this game");
}

#[test]
fn the_factions_with_a_computer_player_are_listed_in_order() {
    assert!(factions::has_ai(Faction::Gdo));
    let listed: Vec<Faction> = Faction::ALL.iter().copied().filter(|&f| factions::has_ai(f)).collect();
    assert_eq!(factions::with_ai(), listed);
    let names: Vec<String> = Faction::ALL.iter().map(|&f| factions::name(f)).collect();
    assert_eq!(names, ["gdo", "syndicate", "cults", "colonists"]);
}
