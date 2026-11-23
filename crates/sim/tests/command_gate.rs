//! The command gate (`ARCHITECTURE.md > Commands, events and replays`): every command passes `Sim::validate` before
//! it is applied, and a rejected one changes nothing.

use sc_sim::testing::{ScenarioBuilder, TestSim};
use sc_sim::{
    Command, DebugCommand, ObjectId, ObjectKind, PlayerCommand, PlayerId, Reject, SimConfig, SpawnAt, SpawnProblem,
    TickInputs, TilePos, Vec2,
};

/// 6 × 4 tiles. `ScenarioBuilder` gives it players 1 and 2.
const MAP: &str = "
    ......
    ......
    ......
    ......
";

fn patch_at(x: i32, y: i32) -> DebugCommand {
    DebugCommand::Spawn {
        kind: ObjectKind::CrystalPatch,
        owner: None,
        at: SpawnAt::Tile(TilePos::new(x, y)),
    }
}

fn from(player: u8, command: DebugCommand) -> PlayerCommand {
    PlayerCommand {
        player: PlayerId(player),
        command: Command::Debug(command),
    }
}

/// Patches 1 and 2 in the top-left corner.
fn two_patches(config: SimConfig) -> TestSim {
    let scenario = ScenarioBuilder::new(MAP)
        .spawn(ObjectKind::CrystalPatch, None, SpawnAt::Tile(TilePos::new(0, 0)))
        .spawn(ObjectKind::CrystalPatch, None, SpawnAt::Tile(TilePos::new(1, 0)))
        .build();
    TestSim::with_config(scenario, config)
}

/// Schedules `commands` for the next tick, runs it, and checks that it changed exactly what the same tick without
/// them changes.
fn assert_ignored(t: &mut TestSim, commands: &[PlayerCommand]) {
    let mut without = t.sim().clone();
    without.step(&TickInputs::default());
    let tick = t.sim().tick() + 1;
    for cmd in commands {
        t.at(tick, cmd.player, cmd.command.clone());
    }
    t.step();
    assert_eq!(t.sim().state_hash(), without.state_hash());
    assert_eq!(t.sim(), &without);
    assert!(t.sim().events().is_empty(), "{:?}", t.sim().events());
}

#[test]
fn commands_from_players_not_in_the_game_are_rejected() {
    let mut t = two_patches(SimConfig::DEBUG);
    let commands = [
        patch_at(4, 2),
        DebugCommand::Remove { id: ObjectId(1) },
        DebugCommand::SetPosition {
            id: ObjectId(2),
            pos: Vec2::new(100.0, 100.0),
        },
    ];
    let mut rejected = Vec::new();
    for player in [0, 3, 9, u8::MAX] {
        for command in &commands {
            let cmd = from(player, command.clone());
            assert_eq!(t.sim().validate(&cmd), Err(Reject::UnknownPlayer(PlayerId(player))));
            rejected.push(cmd);
        }
    }
    assert_ignored(&mut t, &rejected);

    // From the game's players, the same spawn and removal go through.
    for player in [1, 2] {
        assert_eq!(t.sim().validate(&from(player, commands[1].clone())), Ok(()));
    }
    let next = t.sim().tick() + 1;
    t.debug_at(next, commands[0].clone())
        .at(next, PlayerId(2), Command::Debug(commands[1].clone()));
    t.step();
    let ids: Vec<u32> = t.sim().objects().map(|o| o.id.0).collect();
    assert_eq!(ids, [2, 3]);
}

#[test]
fn commands_naming_missing_objects_are_rejected() {
    let mut t = two_patches(SimConfig::DEBUG);
    t.debug_at(1, DebugCommand::Remove { id: ObjectId(1) });
    t.step();
    // 0 is never used, 1 is gone, 3 is the next id, the rest were never allocated.
    let mut rejected = Vec::new();
    for id in [0, 1, 3, 99, u32::MAX].map(ObjectId) {
        for command in [
            DebugCommand::Remove { id },
            DebugCommand::SetPosition {
                id,
                pos: Vec2::new(100.0, 100.0),
            },
        ] {
            let cmd = from(1, command);
            assert_eq!(t.sim().validate(&cmd), Err(Reject::UnknownObject(id)));
            rejected.push(cmd);
        }
    }
    assert_ignored(&mut t, &rejected);
    assert!(t.sim().object(ObjectId(2)).is_some());
}

#[test]
fn debug_commands_are_rejected_while_debug_is_off() {
    let mut off = two_patches(SimConfig::default());
    let mut on = two_patches(SimConfig::DEBUG);
    assert_eq!(off.sim().config(), SimConfig { debug: false });
    // The option is part of the state.
    assert_ne!(off.sim().state_hash(), on.sim().state_hash());

    let commands = [patch_at(4, 2), DebugCommand::Remove { id: ObjectId(1) }].map(|c| from(1, c));
    for cmd in &commands {
        assert_eq!(off.sim().validate(cmd), Err(Reject::DebugDisabled));
        assert_eq!(on.sim().validate(cmd), Ok(()));
    }
    assert_ignored(&mut off, &commands);
    for cmd in &commands {
        on.at(1, cmd.player, cmd.command.clone());
    }
    on.step();
    assert_eq!(on.sim().events().len(), 2);

    // The replay keeps the option, so it rejects the same commands.
    assert_eq!(off.replay().config, SimConfig::default());
    assert_eq!(off.replay().run(), *off.sim());
}

#[test]
fn the_gate_checks_the_player_then_the_debug_option_then_the_command() {
    let off = two_patches(SimConfig::default());
    let on = two_patches(SimConfig::DEBUG);
    let remove_missing = DebugCommand::Remove { id: ObjectId(7) };
    assert_eq!(
        off.sim().validate(&from(5, remove_missing.clone())),
        Err(Reject::UnknownPlayer(PlayerId(5)))
    );
    assert_eq!(
        off.sim().validate(&from(1, remove_missing.clone())),
        Err(Reject::DebugDisabled)
    );
    assert_eq!(
        on.sim().validate(&from(1, remove_missing)),
        Err(Reject::UnknownObject(ObjectId(7)))
    );
    // A debug command's own rules are part of the gate.
    assert_eq!(
        on.sim().validate(&from(1, patch_at(6, 0))),
        Err(Reject::Spawn(SpawnProblem::FootprintOffMap))
    );
    let move_patch = DebugCommand::SetPosition {
        id: ObjectId(1),
        pos: Vec2::new(100.0, 100.0),
    };
    assert_eq!(
        on.sim().validate(&from(2, move_patch)),
        Err(Reject::TileObject(ObjectId(1)))
    );
    assert_eq!(
        on.sim().validate(&from(1, patch_at(0, 0))),
        Ok(()),
        "overlapping footprints are fine for debug spawns"
    );
}
