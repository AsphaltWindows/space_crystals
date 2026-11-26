//! `--ai <player>[:<profile>][,<player>[:<profile>]...]`: the players computers play (`core/game_rules.md > Computer
//! players (playtest tool)`: "started from the command line, taking any player's slot"; `crate::ai_players`).

#[cfg(test)]
mod tests;

use sc_ai::Profile;
use sc_sim::{PlayerId, Scenario};

use super::values::parse_player;
use crate::ai_players::AiSlot;

/// `--ai`'s value: players by number, comma-separated, each with a profile after a colon or the standard one
/// (`2`, `1:passive`, `1,2:passive`). Each player at most once.
pub(super) fn parse_ai(s: &str) -> Result<Vec<AiSlot>, String> {
    let mut slots: Vec<AiSlot> = Vec::new();
    for part in s.split(',') {
        let (player, profile) = match part.split_once(':') {
            Some((player, profile)) => (player, profile.parse()?),
            None => (part, Profile::default()),
        };
        let player = PlayerId(parse_player(player.trim())?);
        if slots.iter().any(|slot| slot.player == player) {
            return Err(format!("--ai names player {} twice", player.0));
        }
        slots.push(AiSlot { player, profile });
    }
    Ok(slots)
}

/// That a computer can play each of `slots` in the game `scenario` starts: the game has the player, whose faction has a
/// computer player (`sc_ai::can_play`).
pub(super) fn check(slots: &[AiSlot], scenario: &Scenario) -> Result<(), String> {
    for slot in slots {
        sc_ai::can_play(scenario, slot.player).map_err(|error| format!("--ai: {error}"))?;
    }
    Ok(())
}
