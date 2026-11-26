//! Computer players in the client (`core/game_rules.md > Computer players (playtest tool)`: "started from the command
//! line, taking any player's slot"; `ARCHITECTURE.md > Computer players`): `--ai 2`, `--ai 1,2:passive`.
//!
//! - [`AiPlayers`]: one `sc_ai::Ai` for each player of `--ai`, made from the game as it starts.
//! - **Thinking**: in `FixedUpdate`, before the step (and after QA's `--commands`), each computer player gives its
//!   commands for the coming tick into [`PendingCommands`], once for each tick the sim steps: none while it is paused or
//!   held, and none once its player is defeated or the game is over (`sc_ai::Ai::think`). The step records them with
//!   every other command ([`crate::sim_driver::RecordedInputs`]), so the replay plays them back. A replay's playback
//!   never has computer players: its recorded commands are theirs (`crate::cli` refuses `--ai` with `--replay`).
//! - **The local player's input**: while the local player is a computer player's, the command flow's input
//!   (`CommandUiSystems::Input`: keyboard, mouse and the command panel) doesn't run, so the client gives that player no
//!   command; selecting and looking still work. The debug player switch (F2) still changes the local player, and the top
//!   bar says the player is a computer's (`crate::hud::TopBarNote::Computer`).

use bevy::prelude::*;
use sc_ai::{Ai, Profile};
use sc_sim::PlayerId;

use crate::command_ui::CommandUiSystems;
use crate::qa::inject_commands;
use crate::sim_driver::{LoadedScenario, LocalPlayer, PendingCommands, SimControl, SimRes, StepSim};

/// One computer player of `--ai`: the player it plays, and how.
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub struct AiSlot {
    pub player: PlayerId,
    pub profile: Profile,
}

/// The game's computer players (module docs), in `--ai`'s order.
#[derive(Resource, Debug, Default)]
pub struct AiPlayers {
    players: Vec<Ai>,
}

impl AiPlayers {
    /// Whether a computer player plays `player`.
    pub fn controls(&self, player: PlayerId) -> bool {
        self.players.iter().any(|ai| ai.player() == player)
    }

    /// The computer players' slots, in `--ai`'s order.
    pub fn slots(&self) -> Vec<AiSlot> {
        (self.players.iter())
            .map(|ai| AiSlot {
                player: ai.player(),
                profile: ai.profile(),
            })
            .collect()
    }
}

/// Plays the players of `slots` by computer (module docs). Added after the sim driver, whose game they join.
pub struct AiPlayersPlugin {
    pub slots: Vec<AiSlot>,
}

impl Plugin for AiPlayersPlugin {
    fn build(&self, app: &mut App) {
        let world = app.world();
        let (sim, scenario) = (world.resource::<SimRes>(), &world.resource::<LoadedScenario>().0);
        let mut players = Vec::new();
        for slot in &self.slots {
            // The command line checked the slots against the game (`sc_ai::can_play`).
            match Ai::new(scenario, sim, slot.player, slot.profile) {
                Ok(ai) => players.push(ai),
                Err(error) => error!("--ai: {error}"),
            }
        }
        app.insert_resource(AiPlayers { players })
            .configure_sets(Update, CommandUiSystems::Input.run_if(local_player_is_human))
            .add_systems(FixedUpdate, think.before(StepSim).after(inject_commands));
    }
}

/// Whether the local player is nobody's computer player's: only then does the command flow's input run.
pub fn local_player_is_human(player: Res<LocalPlayer>, ais: Res<AiPlayers>) -> bool {
    !ais.controls(player.0)
}

/// Each computer player's commands for the coming tick, if the sim steps to it now (module docs).
fn think(mut ais: ResMut<AiPlayers>, sim: Res<SimRes>, control: Res<SimControl>, mut pending: ResMut<PendingCommands>) {
    if !control.may_step(&sim) {
        return;
    }
    for ai in &mut ais.players {
        pending.0.extend(ai.think(&sim));
    }
}
