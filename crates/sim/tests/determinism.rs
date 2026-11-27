//! Determinism (`ARCHITECTURE.md > Determinism`): same scenario plus same inputs gives the
//! same state hash, also after a JSON round trip and when played back from a replay, whose checkpoints catch a game
//! that diverges from the recording.

use sc_sim::replay::CHECKPOINT_INTERVAL;
use sc_sim::testing::{TestSim, every_command};
use sc_sim::{
    Checkpoint, Command, DebugCommand, Desync, ObjectId, ObjectKind, PlayerCommand, PlayerId, Replay, ReplayError, Sim,
    SimConfig, SpawnAt, TickInputs, TilePos, scenarios,
};

fn spawn(kind: ObjectKind, x: i32, y: i32) -> DebugCommand {
    DebugCommand::Spawn {
        kind,
        owner: None,
        at: SpawnAt::Tile(TilePos::new(x, y)),
        orientation: Default::default(),
    }
}

/// `dev_skirmish` with debug commands, valid and invalid, scheduled over its first 1100
/// ticks (the station's first delivery comes at tick 960).
fn scripted_skirmish() -> TestSim {
    let mut t = TestSim::new(scenarios::dev_skirmish());
    t.debug_at(1, spawn(ObjectKind::CrystalPatch, 20, 20))
        .debug_at(5, DebugCommand::Remove { id: ObjectId(1) })
        .debug_at(10, spawn(ObjectKind::SupplyDeliveryStation, 12, 40))
        .debug_at(10, spawn(ObjectKind::SupplyDeliveryStation, 63, 47)) // off the map: ignored
        .debug_at(400, DebugCommand::Remove { id: ObjectId(17) })
        .debug_at(
            700,
            DebugCommand::SetPosition {
                id: ObjectId(2),
                pos: sc_sim::Vec2::new(100.0, 100.0),
            },
        );
    t
}

const TICKS: u32 = 1100;

#[test]
fn identical_runs_give_identical_hashes_every_tick() {
    let run = || {
        let mut t = scripted_skirmish();
        let mut hashes = vec![t.sim().state_hash()];
        for _ in 0..TICKS {
            t.step();
            hashes.push(t.sim().state_hash());
        }
        hashes
    };
    let first = run();
    assert_eq!(first, run());
    // The state changes every tick (at least the tick counter), and so does the hash.
    assert!(first.windows(2).all(|w| w[0] != w[1]));
}

#[test]
fn the_seed_is_part_of_the_state() {
    let mut reseeded = scenarios::dev_skirmish();
    reseeded.setup.seed = 2;
    let a = Sim::new(&scenarios::dev_skirmish());
    let b = Sim::new(&reseeded);
    assert_ne!(a.state_hash(), b.state_hash());
}

#[test]
fn json_round_trip_preserves_the_state() {
    let mut t = scripted_skirmish();
    t.run(TICKS);
    let json = serde_json::to_string(t.sim()).unwrap();
    let mut loaded: Sim = serde_json::from_str(&json).unwrap();
    assert_eq!(loaded.state_hash(), t.sim().state_hash());
    assert_eq!(&loaded, t.sim());
    // A loaded save continues exactly like the original game.
    let mut original = t.sim().clone();
    let inputs = TickInputs {
        commands: vec![PlayerCommand {
            player: PlayerId(2),
            command: Command::Debug(spawn(ObjectKind::CrystalPatch, 40, 2)),
        }],
    };
    for _ in 0..1000 {
        original.step(&inputs);
        loaded.step(&inputs);
    }
    assert_eq!(loaded.state_hash(), original.state_hash());
}

#[test]
fn replays_reproduce_the_run() {
    let mut t = scripted_skirmish();
    t.run(TICKS);
    let replay = t.replay();
    assert_eq!(replay.end_tick(), TICKS);
    let json = serde_json::to_string(replay).unwrap();
    let loaded: Replay = serde_json::from_str(&json).unwrap();
    assert_eq!(&loaded, replay);
    assert_eq!(loaded.run().state_hash(), t.sim().state_hash());
    assert_eq!(loaded.try_run().unwrap().state_hash(), t.sim().state_hash());

    // Step by step, as a client plays a replay back.
    let mut sim = loaded.start();
    assert_eq!(sim.config(), SimConfig::DEBUG, "the replay keeps the game's options");
    while loaded.step(&mut sim) {}
    assert_eq!(sim.tick(), TICKS);
    assert_eq!(sim.state_hash(), t.sim().state_hash());
    assert!(!loaded.step(&mut sim));
    assert_eq!(sim.tick(), TICKS);
}

#[test]
fn recording_alongside_the_sim() {
    // The client's way: step the sim and record the same inputs, empty ticks included.
    let mut scenario = scenarios::empty(16, 12);
    scenario.setup.config = SimConfig::DEBUG;
    let mut sim = Sim::new(&scenario);
    let mut replay = Replay::new(scenario);
    for tick in 1..=50 {
        let mut inputs = TickInputs::default();
        if tick % 10 == 0 {
            inputs.commands.push(PlayerCommand {
                player: PlayerId(1),
                command: Command::Debug(spawn(ObjectKind::CrystalPatch, tick / 10, 3)),
            });
        }
        sim.step(&inputs);
        replay.record(inputs);
    }
    assert_eq!(sim.objects().count(), 5);
    assert_eq!(replay.end_tick(), 50);
    assert_eq!(replay.run().state_hash(), sim.state_hash());
}

#[test]
fn replays_of_invalid_scenarios_fail_without_panicking() {
    let mut scenario = scenarios::empty(4, 4);
    scenario.setup.players[1].slot = sc_sim::SlotId(1);
    let replay = Replay::new(scenario.clone());
    assert_eq!(replay.try_run(), Err(scenario.validate()));
    assert_eq!(replay.run_checked(), Err(ReplayError::Invalid(scenario.validate())));
}

/// The scripted skirmish recorded the client's way ([`Replay::record_checked`]), with a checkpoint at its end too.
fn recorded_skirmish() -> (Replay, Sim) {
    let mut t = scripted_skirmish();
    t.run(TICKS);
    let mut sim = t.replay().start();
    let mut replay = Replay::new(t.replay().scenario.clone());
    replay.record_checked(None, &sim);
    for inputs in &t.replay().inputs {
        sim.step(inputs);
        replay.record_checked(Some(inputs.clone()), &sim);
    }
    replay.checkpoint(&sim);
    assert_eq!(sim.state_hash(), t.sim().state_hash());
    (replay, sim)
}

/// A recording notes the state hash at tick 0, every `CHECKPOINT_INTERVAL` ticks and where it was asked to; played
/// back, every checkpoint matches, also after a JSON round trip.
#[test]
fn recordings_note_checkpoints_that_playback_matches() {
    let (replay, sim) = recorded_skirmish();
    let mut ticks: Vec<u32> = (0..=TICKS).step_by(CHECKPOINT_INTERVAL as usize).collect();
    ticks.push(TICKS);
    assert_eq!(replay.checkpoints.iter().map(|c| c.tick).collect::<Vec<_>>(), ticks);
    let end = Checkpoint {
        tick: TICKS,
        hash: sim.state_hash(),
    };
    assert_eq!(replay.checkpoint_at(TICKS), Some(end));
    assert_eq!(replay.checkpoint_at(TICKS - 1), None);
    // Noting one twice, or anywhere but at the end of the recording, notes nothing.
    let mut again = replay.clone();
    again.checkpoint(&sim);
    again.checkpoint(&replay.start());
    assert_eq!(again, replay);

    let json = serde_json::to_string(&replay).unwrap();
    let loaded: Replay = serde_json::from_str(&json).unwrap();
    assert_eq!(loaded, replay);
    assert_eq!(loaded.run_checked().unwrap(), sim);
    // Step by step, as the client plays a replay back.
    let mut played = loaded.start();
    assert_eq!(loaded.verify(&played), Ok(()));
    while loaded.step(&mut played) {
        assert_eq!(loaded.verify(&played), Ok(()), "tick {}", played.tick());
    }
}

/// A game that diverges from its recording fails at the first checkpoint after the change, which names the
/// checkpoint before it (where it still matched); a recording whose hash was changed fails at that checkpoint.
#[test]
fn checkpoints_catch_a_game_that_diverges_from_its_recording() {
    let (replay, _) = recorded_skirmish();
    // Without the debug removal of tick 400, the game differs from that tick on.
    let mut changed = replay.clone();
    assert_eq!(changed.inputs[399].commands.len(), 1);
    changed.inputs[399].commands.clear();
    let Err(ReplayError::Desync(desync)) = changed.run_checked() else {
        panic!("the change went unnoticed");
    };
    assert_eq!((desync.tick, desync.previous), (448, Some(384)));
    assert_ne!(desync.expected, desync.actual);
    assert_eq!(desync.expected, replay.checkpoint_at(448).unwrap().hash);
    let message = desync.to_string();
    assert!(
        message.contains("desync at tick 448") && message.contains("at tick 384, matched"),
        "{message}"
    );

    let mut tampered = replay.clone();
    tampered.checkpoints[0].hash ^= 1;
    let Err(ReplayError::Desync(Desync {
        tick: 0,
        previous: None,
        ..
    })) = tampered.run_checked()
    else {
        panic!("a wrong first checkpoint went unnoticed");
    };
}

/// Replays without checkpoints (written before they existed) still load and play: there is just nothing to compare.
#[test]
fn a_replay_without_checkpoints_plays_unchecked() {
    let mut t = scripted_skirmish();
    t.run(100);
    let mut json = serde_json::to_value(t.replay()).unwrap();
    json.as_object_mut().unwrap().remove("checkpoints");
    let loaded: Replay = serde_json::from_value(json).unwrap();
    assert!(loaded.checkpoints.is_empty());
    assert_eq!(loaded.run_checked().unwrap().state_hash(), t.sim().state_hash());
}

/// A replay file keeps every kind of command (`testing::every_command`: every order, ability and debug command), so
/// nothing a player did is lost in a recording; the game plays back the same from the file. It runs past the last of
/// them, one a tick from tick 20.
#[test]
fn replay_files_keep_every_kind_of_command() {
    let mut t = scripted_skirmish();
    let commands = every_command();
    let last = 20 + commands.len() as u32;
    for (i, command) in commands.into_iter().enumerate() {
        let player = PlayerId(1 + (i % 2) as u8);
        t.at(20 + i as u32, player, command);
    }
    t.run(last.max(100));
    let replay = t.replay();
    let recorded: Vec<Command> = replay.inputs[19..]
        .iter()
        .flat_map(|inputs| inputs.commands.iter().map(|c| c.command.clone()))
        .collect();
    assert_eq!(recorded, every_command());
    let json = serde_json::to_string(replay).unwrap();
    let loaded: Replay = serde_json::from_str(&json).unwrap();
    assert_eq!(&loaded, replay);
    assert_eq!(loaded.run().state_hash(), t.sim().state_hash());
}
