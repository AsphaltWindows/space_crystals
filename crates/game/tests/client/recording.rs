//! What the client records (`ARCHITECTURE.md > Commands, events and replays`) replays to the same state.

use std::collections::BTreeMap;

use bevy::prelude::*;

use sc_sim::{
    Command, DebugCommand, ObjectId, ObjectKind, PlayerCommand, PlayerId, Replay, SimConfig, Spawn, SpawnAt,
    SpawnSettings, TilePos,
};
use space_crystals::AppConfig;
use space_crystals::sim_driver::{LoadedScenario, PendingCommands, RecordedInputs, SimRes};

use super::{debug, headless, input, sim_ids, small_scenario, view_middle};

/// The commands the replay test's client submits at each frame: accepted ones and rejected ones.
fn scripted_commands(frame: u32) -> Vec<PlayerCommand> {
    let spawn = |kind, x, y| {
        debug(DebugCommand::Spawn {
            kind,
            owner: None,
            at: SpawnAt::Tile(TilePos::new(x, y)),
        })
    };
    let stranger = PlayerCommand {
        player: PlayerId(9),
        command: Command::Debug(DebugCommand::Remove { id: ObjectId(1) }),
    };
    match frame {
        3 => vec![spawn(ObjectKind::CrystalPatch, 12, 8)],
        7 => vec![
            debug(DebugCommand::Remove { id: ObjectId(2) }),
            spawn(ObjectKind::SupplyDeliveryStation, 1, 8),
        ],
        9 => vec![debug(DebugCommand::Remove { id: ObjectId(99) }), stranger],
        15 => vec![debug(DebugCommand::Remove { id: ObjectId(1) })],
        _ => Vec::new(),
    }
}

/// `ARCHITECTURE.md > Commands, events and replays`: what the client records (`LoadedScenario` and
/// `RecordedInputs`) plays back in a bare `Sim` to the same state hash at every tick, whether debug commands are on
/// (the default) or off (`--no-debug`). The player's input that is not a command never reaches the sim.
#[test]
fn the_clients_recording_replays_to_the_same_state_hashes() {
    for sim_config in [SimConfig::DEBUG, SimConfig::default()] {
        let mut scenario = small_scenario();
        // A station waiting for its first delivery, so that the state changes by itself too.
        scenario.map.spawns.push(Spawn {
            settings: Some(SpawnSettings::SupplyDeliveryStation {
                starting_supplies: 0,
                delivery_size: 30,
                delivery_interval: 5,
            }),
            ..Spawn::new(
                ObjectKind::SupplyDeliveryStation,
                None,
                SpawnAt::Tile(TilePos::new(12, 2)),
            )
        });
        assert_eq!(scenario.validate(), vec![]);
        let mut config = AppConfig::new(scenario);
        config.scenario.setup.config = sim_config;
        let mut app = headless(config);
        let initial_ids = sim_ids(&app);
        let middle = view_middle(&app);
        let mut hashes = BTreeMap::new();
        for frame in 0..40 {
            app.world_mut()
                .resource_mut::<PendingCommands>()
                .0
                .extend(scripted_commands(frame));
            let mut input = input(&mut app);
            match frame {
                2 => input.keys.press(KeyCode::ArrowRight),
                4 => input.tap_key(KeyCode::KeyG),
                5 => input.keys.release(KeyCode::ArrowRight),
                6 => {
                    input.cursor = Some(middle);
                    input.keys.press(KeyCode::ControlLeft);
                    input.tap_key(KeyCode::Digit1);
                    input.mouse.press(MouseButton::Left);
                }
                8 => input.mouse.release(MouseButton::Left),
                10 => input.mouse.press(MouseButton::Right),
                _ => {}
            }
            app.update();
            let sim = app.world().resource::<SimRes>();
            hashes.insert(sim.tick(), sim.state_hash());
        }
        let sim = app.world().resource::<SimRes>().sim().clone();
        assert_eq!(sim.config(), sim_config);
        assert_eq!(sim.tick(), 39);
        if sim_config.debug {
            assert_eq!(sim_ids(&app).len(), initial_ids.len());
            assert!(!sim_ids(&app).contains(&ObjectId(1)) && sim_ids(&app).contains(&ObjectId(7)));
        } else {
            assert_eq!(sim_ids(&app), initial_ids, "every debug command was rejected");
        }

        // A replay is the map, the game setup (game options included) and the inputs.
        let replay = Replay {
            scenario: app.world().resource::<LoadedScenario>().0.clone(),
            inputs: app.world().resource::<RecordedInputs>().ticks.clone(),
            checkpoints: Vec::new(),
        };
        assert_eq!(replay.scenario.setup.config, sim_config);
        let mut bare = replay.start();
        loop {
            assert_eq!(
                Some(&bare.state_hash()),
                hashes.get(&bare.tick()),
                "tick {}",
                bare.tick()
            );
            if !replay.step(&mut bare) {
                break;
            }
        }
        assert_eq!(bare.tick(), sim.tick());
        assert_eq!(bare, sim);
    }
}
