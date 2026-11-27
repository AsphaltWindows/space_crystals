//! (M5-X2) A game's data directory (`--tuning <dir>`, games only, with debug commands on): read before the first
//! tick, then whenever it changes, its rules given to the game and its client tuning to the client ([`TuningData`]).
//!
//! - **The rules** go to the game as a `DebugCommand::SetRules` of the local player (or, once it is defeated, of a
//!   player still in the game), checked by the gate first (`Sim::validate`), whenever they differ from those last
//!   given to it: the first one in the first tick's inputs, so that the game plays by them from its second tick on
//!   (`sim/rules.rs`). The replay records it, and plays the tuned game back without the directory.
//! - **The client's tuning** replaces the `ClientTuning` resource at once.
//! - **A refusal** ([`super::Refusal`], or the gate's) keeps both as they were, and the feedback line says it
//!   (`crate::command_ui::DataRefused`), as the log does.
//! - **Polled** every half second of real time (`super::poll`), the pause and the playtest menu's included.
//! - **A restart** (the playtest menu's Restart, F10) gives the new game the rules of the last read that passed, in its
//!   first tick; F8 ([`InputAction::DebugRestartWithData`]) restarts the same way after reading the directory again.
//!   A structural change is refused live and after F8 alike, its file and member named: it needs a change to the Rust
//!   default and a rebuild (`docs/tuning.md`).
//! - **At most 63 sets of rules a run** (`sc_sim::ruleset::GENERATION_COUNT`, the built-in ones besides), each set of
//!   rules installed once whichever game takes it: a change beyond them is refused until the game is launched again.

use std::collections::BTreeSet;
use std::path::PathBuf;
use std::sync::Mutex;

use bevy::app::{RunFixedMainLoop, RunFixedMainLoopSystems};
use bevy::prelude::*;
use sc_sim::ruleset;
use sc_sim::{Command, DebugCommand, PlayerCommand, PlayerId, Reject, Ruleset, Sim};

use super::load::{Data, load};
use super::poll::Poll;
use crate::ClientSystems;
use crate::command_ui::DataRefused;
use crate::input::{ActionState, InputAction, ReadActions};
use crate::playtest_menu::Launch;
use crate::playtest_menu::restart::restart;
use crate::sim_driver::{LocalPlayer, PendingCommands, SimRes};
use crate::tuning::ClientTuning;

/// The data directory of a game (module docs). Add it after `SimDriverPlugin`, `CommandUiPlugin` and, in a game,
/// `PlaytestMenuPlugin`.
pub struct DataPlugin {
    pub dir: PathBuf,
}

impl Plugin for DataPlugin {
    fn build(&self, app: &mut App) {
        app.insert_resource(TuningData::new(self.dir.clone()))
            .add_systems(Startup, read_at_start)
            .add_systems(PreUpdate, restart_with_data.after(ReadActions))
            .add_systems(
                RunFixedMainLoop,
                follow_restarts.in_set(RunFixedMainLoopSystems::BeforeFixedMainLoop),
            )
            .add_systems(Update, poll_data.in_set(ClientSystems::SyncSim));
    }
}

/// The data directory (module docs) and what it last gave.
#[derive(Resource, Debug)]
pub struct TuningData {
    pub dir: PathBuf,
    /// Its poll, from the first read on.
    poll: Option<Poll>,
    /// What the last read that passed gave: the built-in data until one does.
    pub data: Data,
    /// The game (`SimRes::generation`) the rules were last given to, and those rules.
    sent: Option<(u32, Ruleset)>,
}

impl TuningData {
    fn new(dir: PathBuf) -> TuningData {
        TuningData {
            dir,
            poll: None,
            data: Data {
                rules: Ruleset::builtin(),
                client: ClientTuning::BUILTIN,
                tuned: Vec::new(),
            },
            sent: None,
        }
    }

    /// How many files change their counterpart in the data the game plays by: "Tuned: N files" (`crate::hud`).
    pub fn tuned(&self) -> usize {
        self.data.tuned.len()
    }

    /// Reads the directory again: its data, or the refusal's text (module docs).
    fn read(&mut self) -> Result<(), String> {
        match load(&self.dir) {
            Ok(data) => {
                if data.tuned != self.data.tuned {
                    info!("Tuning: {} file(s) tuned: {}", data.tuned.len(), data.tuned.join(", "));
                }
                self.data = data;
                Ok(())
            }
            Err(refusal) => Err(refusal.to_string()),
        }
    }

    /// Gives `sim`'s game the rules of the data unless it was last given them (module docs): the refusal's text if
    /// the gate refuses them.
    fn send(&mut self, sim: &SimRes, local: PlayerId, pending: &mut PendingCommands) -> Result<(), String> {
        let generation = sim.generation();
        let given = match &self.sent {
            Some((game, rules)) if *game == generation => rules,
            _ => Ruleset::builtin_ref(),
        };
        if *given == self.data.rules {
            return Ok(());
        }
        let command = PlayerCommand {
            player: sender(sim, local),
            command: Command::Debug(DebugCommand::SetRules(Box::new(self.data.rules.clone()))),
        };
        match sim.validate(&command) {
            Ok(()) => {}
            // The generations are the process's: no restart frees one.
            Err(Reject::RulesFull) => {
                return Err(format!(
                    "{}: the game took {} sets of rules in this run, the most it can: quit and launch it again",
                    self.dir.display(),
                    ruleset::GENERATION_COUNT - 1
                ));
            }
            Err(reject) => return Err(format!("{}: the game refused the rules: {reject}", self.dir.display())),
        }
        pending.0.push(command);
        self.sent = Some((generation, self.data.rules.clone()));
        Ok(())
    }
}

/// Who gives the game its rules: the local player, or the first player still in the game once it is defeated (the
/// gate takes no command of a defeated player).
fn sender(sim: &Sim, local: PlayerId) -> PlayerId {
    let in_game = |id: PlayerId| sim.player(id).is_some_and(|player| !player.is_defeated());
    if in_game(local) {
        return local;
    }
    (sim.players().iter().map(|player| player.id))
        .find(|&id| in_game(id))
        .unwrap_or(local)
}

/// Everything a read and a send change.
fn read_and_send(world: &mut World) {
    let local = world.resource::<LocalPlayer>().0;
    world.resource_scope(|world, mut tuning: Mut<TuningData>| {
        let read = tuning.read();
        if let Some(poll) = &mut tuning.poll {
            poll.rebase();
        }
        world.resource_mut::<ClientTuning>().set_if_neq(tuning.data.client);
        let sent = world.resource_scope(|world, mut pending: Mut<PendingCommands>| {
            tuning.send(world.resource::<SimRes>(), local, &mut pending)
        });
        for error in [read.err(), sent.err()].into_iter().flatten() {
            refuse(world, error);
        }
    });
}

/// Says a refusal in the feedback line and the log.
fn refuse(world: &mut World, text: String) {
    warn!("Tuning refused: {text}");
    let tick = world.resource::<SimRes>().tick();
    world.write_message(DataRefused {
        text: interned(text),
        tick,
    });
}

/// `text` as a `&'static str`, as the feedback line keeps its texts: each text once, however often it is refused.
fn interned(text: String) -> &'static str {
    static TEXTS: Mutex<BTreeSet<&'static str>> = Mutex::new(BTreeSet::new());
    let mut texts = TEXTS.lock().unwrap_or_else(std::sync::PoisonError::into_inner);
    if let Some(known) = texts.get(text.as_str()) {
        return known;
    }
    let leaked: &'static str = text.leak();
    texts.insert(leaked);
    leaked
}

/// Reads the directory before the first tick, its rules in the first tick's inputs (module docs), and starts the poll.
fn read_at_start(world: &mut World) {
    let now = world.resource::<Time<Real>>().elapsed_secs_f64();
    let mut tuning = world.resource_mut::<TuningData>();
    let poll = Poll::new(&tuning.dir, now);
    tuning.poll = Some(poll);
    read_and_send(world);
}

/// Reads the directory again when it changed, at most every half second of real time (module docs).
fn poll_data(world: &mut World) {
    let now = world.resource::<Time<Real>>().elapsed_secs_f64();
    // The poll's own state: the data are unchanged until a read (the top bar's note follows their changes).
    let mut tuning = world.resource_mut::<TuningData>();
    let changed = (tuning.bypass_change_detection().poll.as_mut()).is_some_and(|poll| poll.changed(now));
    if changed {
        read_and_send(world);
    }
}

/// F8 (module docs): the playtest menu's Restart, then the directory read again, its rules in the new game's first
/// tick. Only in a game, which the playtest menu's [`Launch`] says; with the menu open too, which F8 passes
/// (`InputAction::passes_modal`) and the restart closes.
fn restart_with_data(actions: Res<ActionState>, launch: Option<Res<Launch>>, mut commands: Commands) {
    if actions.just_pressed(InputAction::DebugRestartWithData) && launch.is_some() {
        info!("F8: restarting the game with the tuning directory's data");
        commands.queue(|world: &mut World| {
            restart(world);
            read_and_send(world);
        });
    }
}

/// Gives a game the playtest menu restarted the rules of the last read that passed, before its first tick
/// (module docs).
fn follow_restarts(
    sim: Res<SimRes>,
    local: Res<LocalPlayer>,
    mut tuning: ResMut<TuningData>,
    mut pending: ResMut<PendingCommands>,
    mut refused: MessageWriter<DataRefused>,
) {
    let new_game = tuning.sent.as_ref().is_some_and(|(game, _)| *game != sim.generation());
    if new_game && let Err(text) = tuning.send(&sim, local.0, &mut pending) {
        warn!("Tuning refused: {text}");
        refused.write(DataRefused {
            text: interned(text),
            tick: sim.tick(),
        });
    }
}
