//! Computer players (`core/game_rules.md > Computer players (playtest tool)`): "For now, computer players are a
//! **playtest tool** ... started from the command line, taking any player's slot. Their behavior (profiles such as
//! standard, rush and passive; build orders) is the Director's placeholder" (design change 0402, answering Q0147).
//! Every behavior here is a placeholder of that kind, its numbers in `tuning` (`ARCHITECTURE.md > Computer players`).
//!
//! An [`Ai`] plays one player. Once a tick, before the step, [`Ai::think`] looks at the game as the last tick left it
//! and gives the player's commands for the next one, as `testing`'s scripted openings do: the same commands a player
//! gives, through the same gate (`core/game_rules.md > Computer players (playtest tool)`: "It issues the same commands a
//! player can, under the same rules"), which the replays record. It knows only what the player's fog of war shows
//! ([`View`]: "Fair knowledge").
//!
//! - **Layers**: [`view`] (what the player knows), [`turn`] (one tick's commands, each checked by the gate, and what the
//!   bank still pays), and each faction's own module behind [`FactionAi`] ([`factions`]).
//! - **Deterministic**, like the sim: the same game gives the same commands on every machine. No hash containers, no
//!   clock, no randomness of its own, ties broken by `ObjectId`, floats compared with `total_cmp` (`clippy.toml`).
//! - **Never panics**, whatever the game's state: an assumption that fails skips the action.

pub mod factions;
pub mod profile;
pub mod turn;
pub mod view;

use std::fmt;

use sc_sim::{Faction, PlayerCommand, PlayerId, Scenario, Sim};

pub use factions::FactionAi;
pub use profile::Profile;
pub use turn::Turn;
pub use view::{Known, Start, View};

/// A computer player: one player's, playing a profile.
#[derive(Debug)]
pub struct Ai {
    player: PlayerId,
    profile: Profile,
    /// Every player's start location, from the scenario (the sim doesn't keep the slots).
    starts: Vec<Start>,
    faction: Box<dyn FactionAi>,
}

/// Why a computer player can't take a player's slot.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum AiError {
    /// There is no such player in the game.
    UnknownPlayer(PlayerId),
    /// The player's faction has no computer player yet.
    NoAi { player: PlayerId, faction: Faction },
}

impl fmt::Display for AiError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            AiError::UnknownPlayer(player) => write!(f, "there is no player {} in this game", player.0),
            AiError::NoAi { player, faction } => {
                let with: Vec<String> = factions::with_ai().into_iter().map(factions::name).collect();
                write!(
                    f,
                    "player {} plays {}, which has no computer player yet (computer players play: {})",
                    player.0,
                    factions::name(*faction),
                    with.join(", ")
                )
            }
        }
    }
}

impl std::error::Error for AiError {}

/// Whether a computer player can take `player`'s slot in the game `scenario` starts: the game has the player, and the
/// player's faction has a computer player. What the command line checks before the game starts.
pub fn can_play(scenario: &Scenario, player: PlayerId) -> Result<(), AiError> {
    let faction = scenario
        .setup
        .player(player)
        .ok_or(AiError::UnknownPlayer(player))?
        .faction;
    match factions::hook(faction) {
        Some(_) => Ok(()),
        None => Err(AiError::NoAi { player, faction }),
    }
}

impl Ai {
    /// A computer player for `player` in `sim`, the game `scenario` starts, playing `profile`; an error if the game has
    /// no such player or its faction has no computer player yet.
    pub fn new(scenario: &Scenario, sim: &Sim, player: PlayerId, profile: Profile) -> Result<Self, AiError> {
        let faction = sim.player(player).ok_or(AiError::UnknownPlayer(player))?.faction;
        let hook = factions::hook(faction).ok_or(AiError::NoAi { player, faction })?;
        let starts = Start::all(scenario);
        let faction = hook(&View::new(sim, player, &starts), profile);
        Ok(Self {
            player,
            profile,
            starts,
            faction,
        })
    }

    /// The player it plays.
    pub fn player(&self) -> PlayerId {
        self.player
    }

    pub fn profile(&self) -> Profile {
        self.profile
    }

    /// The player's commands for the step to tick `sim.tick() + 1`, from the game as it is now (module docs). None once
    /// the player is defeated or the game is over: the gate would refuse them.
    pub fn think(&mut self, sim: &Sim) -> Vec<PlayerCommand> {
        let view = View::new(sim, self.player, &self.starts);
        if !view.in_game() {
            return Vec::new();
        }
        let mut turn = Turn::new(&view, self.profile, None);
        self.faction.think(&mut turn);
        turn.into_commands()
    }
}
