//! Determinism (`ARCHITECTURE.md > Determinism`): same scenario plus same inputs gives the
//! same state hash, also after a JSON round trip and when played back from a replay.

use sc_sim::testing::TestSim;
use sc_sim::{
    Command, DebugCommand, ObjectId, ObjectKind, PlayerCommand, PlayerId, Replay, Sim, SpawnAt, TickInputs, TilePos,
    scenarios,
};

fn spawn(kind: ObjectKind, x: i32, y: i32) -> DebugCommand {
    DebugCommand::Spawn {
        kind,
        owner: None,
        at: SpawnAt::Tile(TilePos::new(x, y)),
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
    let mut sim = Sim::new(&loaded.scenario);
    while loaded.step(&mut sim) {}
    assert_eq!(sim.tick(), TICKS);
    assert_eq!(sim.state_hash(), t.sim().state_hash());
    assert!(!loaded.step(&mut sim));
    assert_eq!(sim.tick(), TICKS);
}

#[test]
fn recording_alongside_the_sim() {
    // The client's way: step the sim and record the same inputs, empty ticks included.
    let scenario = scenarios::empty(16, 12);
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
}
