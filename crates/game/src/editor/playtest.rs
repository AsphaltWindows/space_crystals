//! Play-testing (F5): the edited map runs in game mode, in-process, with the play-test's game setup
//! ([`PlayTestSetup`]: each slot's faction, the slot you play, the seed); F5 again returns to the editor with the
//! document, its undo history and the editor's view as they were. (Esc doesn't: in the game it cancels,
//! `systems/controls/selection_and_hud.md`.)
//!
//! The setup is no part of the map: factions are chosen per game (`core/map_editor.md > Map Types > Skirmish
//! Default`). Play-tests run with debug commands unless the editor was started with `--no-debug`.

use bevy::ecs::system::SystemParam;
use bevy::prelude::*;
use sc_sim::{Faction, GameSetup, MapDef, PlayerId, Scenario, ScenarioProblem, Sim, SimConfig, SlotId};

use super::actions::{Editor, PlayTestRequest};
use super::doc::EditorDoc;
use super::layouts::problems::problem_lines;
use super::objects::faction_name;
use super::state::EditorStatus;
use crate::ClientMode;
use crate::camera::{CameraView, CameraZoom, start_view};
use crate::sim_driver::{LoadedScenario, LocalPlayer, PendingCommands, RecordedInputs, SimControl, SimRes};

/// The game setup play-tests start with: a player in every slot of the map.
#[derive(Resource, Clone, Debug, PartialEq)]
pub struct PlayTestSetup {
    /// Each slot's faction, by slot index; slots past its end play GDO.
    pub factions: Vec<Faction>,
    /// The slot the local player takes.
    pub slot: SlotId,
    pub seed: u64,
    /// The game options: debug commands on, unless `--no-debug`.
    pub config: SimConfig,
}

impl Default for PlayTestSetup {
    fn default() -> Self {
        Self {
            factions: Vec::new(),
            slot: SlotId(1),
            seed: sc_sim::scenario::DEFAULT_SEED,
            config: SimConfig::DEBUG,
        }
    }
}

impl PlayTestSetup {
    /// The faction of `slot`'s player.
    pub fn faction(&self, slot: SlotId) -> Faction {
        slot.index()
            .and_then(|i| self.factions.get(i))
            .copied()
            .unwrap_or(Faction::Gdo)
    }

    /// The setup of a play-test of `map`: player `n` in slot `n`, each with its slot's faction.
    pub fn game_setup(&self, map: &MapDef) -> GameSetup {
        let mut setup = GameSetup::per_slot(map, &self.factions, self.seed);
        setup.config = self.config;
        setup
    }

    /// The local player: the one in the chosen slot (player `n` takes slot `n`).
    pub fn local_player(&self) -> PlayerId {
        PlayerId(self.slot.0)
    }

    /// A play-test's game of `map`: the map with [`PlayTestSetup::game_setup`].
    pub fn scenario(&self, map: &MapDef) -> Scenario {
        Scenario {
            map: map.clone(),
            setup: self.game_setup(map),
        }
    }

    /// Everything that keeps a play-test of `map` from starting, which the Problems list shows
    /// ([`Scenario::validate`]): the map's problems ([`MapDef::validate`]), then the game's, such as the layouts of
    /// different slots in each other's way (`ScenarioProblem::LayoutOverlap`), which the sim lists once the map has
    /// none of its own. So they show as soon as they are found, not only when F5 is refused (slice M2-J2, M2-I's
    /// finding).
    pub fn problems(&self, map: &MapDef) -> Vec<ScenarioProblem> {
        self.scenario(map).validate()
    }
}

impl Editor<'_> {
    pub(super) fn set_play_test_faction(&mut self, slot: SlotId, faction: Faction) {
        let Some(index) = slot.index().filter(|_| self.doc.map().slot(slot).is_some()) else {
            self.status.warn(format!("There is no slot {}", slot.0));
            return;
        };
        if self.playtest.faction(slot) == faction {
            return;
        }
        let factions = &mut self.playtest.factions;
        if factions.len() <= index {
            factions.resize(index + 1, Faction::Gdo);
        }
        factions[index] = faction;
        self.status
            .info(format!("Play-tests: slot {} plays {}", slot.0, faction_name(faction)));
    }

    pub(super) fn set_play_test_slot(&mut self, slot: SlotId) {
        if self.doc.map().slot(slot).is_none() {
            self.status.warn(format!("There is no slot {}", slot.0));
            return;
        }
        if self.playtest.slot != slot {
            self.playtest.slot = slot;
        }
        self.status.info(format!("Play-tests: you play slot {}", slot.0));
    }
}

/// The editor's view while a play-test runs, to come back to.
#[derive(Resource, Default, Debug)]
pub struct PlayTestState {
    saved_view: Option<(Vec2, f32)>,
}

/// The client state a play-test switches.
#[derive(SystemParam)]
pub struct GameState<'w> {
    sim: ResMut<'w, SimRes>,
    loaded: ResMut<'w, LoadedScenario>,
    player: ResMut<'w, LocalPlayer>,
    control: ResMut<'w, SimControl>,
    pending: ResMut<'w, PendingCommands>,
    recorded: ResMut<'w, RecordedInputs>,
    view: ResMut<'w, CameraView>,
    zoom: ResMut<'w, CameraZoom>,
}

/// Starts or ends a play-test when one is requested.
/// Why a play-test can't start: the first line of the Problems list for `problems`, and how many more there are.
pub fn refusal(map: &MapDef, problems: &[ScenarioProblem]) -> String {
    let lines = problem_lines(map, problems);
    let first = match (lines.first(), problems.first()) {
        (Some(line), _) => line.text.clone(),
        (None, Some(problem)) => problem.to_string(),
        (None, None) => "the scenario is invalid".into(),
    };
    match lines.len() {
        0 | 1 => format!("Can't play-test: {first}"),
        n => format!("Can't play-test: {first}; and {} more, see Problems", n - 1),
    }
}

pub fn switch_play_test(
    mut request: ResMut<PlayTestRequest>,
    mut mode: ResMut<ClientMode>,
    mut state: ResMut<PlayTestState>,
    doc: Res<EditorDoc>,
    setup: Res<PlayTestSetup>,
    mut status: ResMut<EditorStatus>,
    mut game: GameState,
) {
    if !request.0 {
        return;
    }
    request.0 = false;
    match *mode {
        ClientMode::Editor => {
            let map = doc.map();
            let scenario = setup.scenario(map);
            // The game only runs valid scenarios. The refusal says what to fix first in the Problems list's words
            // (slice M2-I: on a new Skirmish Default map, exactly what each layout lacks).
            let sim = match Sim::try_new(&scenario) {
                Ok(sim) => sim,
                Err(problems) => {
                    status.error(refusal(map, &problems));
                    return;
                }
            };
            let player = setup.local_player();
            state.saved_view = Some((game.view.center, game.zoom.0));
            game.sim.replace(sim);
            game.player.0 = player;
            game.pending.0.clear();
            game.recorded.ticks.clear();
            game.control.paused = false;
            game.zoom.0 = 1.0;
            game.view.center = start_view(&scenario, player, &scenario.map.terrain);
            game.loaded.0 = scenario;
            *mode = ClientMode::PlayTest;
            status.info(format!(
                "Play-testing as player {} ({}): F5 returns",
                player.0,
                faction_name(setup.faction(setup.slot))
            ));
        }
        ClientMode::PlayTest => {
            let scenario = Scenario::new(doc.map().clone());
            game.sim.replace(Sim::preview(&scenario).0);
            game.loaded.0 = scenario;
            game.pending.0.clear();
            game.recorded.ticks.clear();
            game.control.paused = true;
            if let Some((center, zoom)) = state.saved_view.take() {
                game.view.center = center;
                game.zoom.0 = zoom;
            }
            *mode = ClientMode::Editor;
            status.info("Back in the editor");
        }
        ClientMode::Game => {}
    }
}
