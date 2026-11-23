//! Player input never panics the sim (`ARCHITECTURE.md > Commands, events and replays`):
//! a long stream of random, mostly invalid debug commands runs deterministically, the
//! commands the gate rejects change nothing, and any per-object values a file holds are
//! handled.

use sc_sim::testing::ScenarioBuilder;
use sc_sim::{
    Command, DebugCommand, ObjectId, ObjectKind, PlayerCommand, PlayerId, Replay, Rng, SU_PER_GU, Sim, SimConfig,
    SimEvent, Spawn, SpawnAt, SpawnSettings, TickInputs, TilePos, Vec2,
};

const TICKS: u32 = 2000;

/// 24 × 16 tiles.
const MAP: &str = "
    ........................
    ....rrrr........~~~~....
    ....rrrr........~~~~....
    ........cccc............
    ........c..c.....MMM....
    ........cccc.....MMM....
    ........................
    ........................
    ........................
    ....~~~~................
    ....~~~~.......rrrr.....
    ...............rrrr.....
    ........................
    ..........MMMM..........
    ........................
    ........................
";

/// A coordinate in tiles: usually on the map or just off its edges, sometimes extreme.
fn tile_coord(rng: &mut Rng, size: u32) -> i32 {
    match rng.below(10) {
        0 => i32::MIN,
        1 => i32::MAX,
        2 => -1 - rng.below(3) as i32,
        _ => rng.below(size + 3) as i32,
    }
}

/// A coordinate in space units: usually on or near the map, sometimes not finite or huge.
fn point_coord(rng: &mut Rng, size: u32) -> f32 {
    match rng.below(12) {
        0 => f32::NAN,
        1 => f32::INFINITY,
        2 => f32::NEG_INFINITY,
        3 => f32::MAX,
        4 => -rng.next_f32() * 100.0,
        _ => rng.next_f32() * (size + 2) as f32 * SU_PER_GU,
    }
}

fn random_point(rng: &mut Rng, sim: &Sim) -> Vec2 {
    let map = sim.map();
    Vec2::new(point_coord(rng, map.width()), point_coord(rng, map.height()))
}

/// An id that often exists, has been removed, or was never allocated.
fn random_id(rng: &mut Rng, sim: &Sim) -> ObjectId {
    let highest = sim.objects().last().map_or(0, |o| o.id.0);
    match rng.below(5) {
        0 => ObjectId(rng.next_u32()),
        _ => ObjectId(rng.below(highest + 3)),
    }
}

fn random_command(rng: &mut Rng, sim: &Sim) -> PlayerCommand {
    let player = PlayerId(rng.below(4) as u8);
    let command = match rng.below(3) {
        0 => {
            let kind = ObjectKind::ALL[rng.below(ObjectKind::ALL.len() as u32) as usize];
            let owner = (rng.below(3) == 0).then(|| PlayerId(rng.below(4) as u8));
            let at = if rng.below(4) == 0 {
                SpawnAt::Point(random_point(rng, sim))
            } else {
                let map = sim.map();
                SpawnAt::Tile(TilePos::new(
                    tile_coord(rng, map.width()),
                    tile_coord(rng, map.height()),
                ))
            };
            DebugCommand::Spawn { kind, owner, at }
        }
        1 => DebugCommand::Remove {
            id: random_id(rng, sim),
        },
        _ => DebugCommand::SetPosition {
            id: random_id(rng, sim),
            pos: random_point(rng, sim),
        },
    };
    PlayerCommand {
        player,
        command: Command::Debug(command),
    }
}

/// Every object stays where the sim's rules allow it.
fn check_invariants(sim: &Sim) {
    let map = sim.map();
    for o in sim.objects() {
        assert!(o.pos.is_finite() && o.prev_pos.is_finite(), "{o:?}");
        match &o.placement {
            Some(p) => assert!(p.tiles().all(|t| map.in_bounds(t)), "{o:?}"),
            None => assert!(map.contains_point(o.pos), "{o:?}"),
        }
    }
}

struct Run {
    hashes: Vec<u64>,
    replay: Replay,
    final_state: Sim,
    spawned: usize,
    removed: usize,
}

fn fuzz_run(seed: u64) -> Run {
    let scenario = ScenarioBuilder::new(MAP)
        .spawn(
            ObjectKind::SupplyDeliveryStation,
            None,
            SpawnAt::Tile(TilePos::new(9, 4)),
        )
        .build();
    let mut sim = Sim::with_config(&scenario, SimConfig::DEBUG);
    let mut replay = Replay::new(scenario, SimConfig::DEBUG);
    let mut rng = Rng::from_seed(seed);
    let (mut spawned, mut removed) = (0, 0);
    let mut hashes = Vec::new();
    for _ in 0..TICKS {
        let commands = (0..rng.below(6)).map(|_| random_command(&mut rng, &sim)).collect();
        let inputs = TickInputs { commands };
        sim.step(&inputs);
        replay.record(inputs);
        check_invariants(&sim);
        for e in sim.events() {
            match e {
                SimEvent::Spawned(_) => spawned += 1,
                SimEvent::Removed(_) => removed += 1,
                SimEvent::SuppliesDelivered { .. } => {}
            }
        }
        hashes.push(sim.state_hash());
    }
    Run {
        hashes,
        replay,
        final_state: sim,
        spawned,
        removed,
    }
}

#[test]
fn random_debug_commands_never_panic_and_stay_deterministic() {
    let first = fuzz_run(0x5eed);
    // The stream exercises the valid paths too, not only rejections.
    assert!(
        first.spawned > 100 && first.removed > 50,
        "{} spawned, {} removed",
        first.spawned,
        first.removed
    );
    let commands: usize = first.replay.inputs.iter().map(|i| i.commands.len()).sum();
    assert!(commands > 4000, "{commands} commands");

    let second = fuzz_run(0x5eed);
    assert_eq!(first.hashes, second.hashes);
    assert_eq!(first.final_state, second.final_state);

    // The recorded stream replays to the same state. (Not through JSON: it has no numbers
    // for the NaN and infinite positions in this stream, and serde_json writes them as
    // `null`, which it can't read back. The state itself never holds such values.)
    assert_eq!(first.replay.run().state_hash(), first.final_state.state_hash());
    let state: Sim = serde_json::from_str(&serde_json::to_string(&first.final_state).unwrap()).unwrap();
    assert_eq!(state.state_hash(), first.final_state.state_hash());

    let other = fuzz_run(0x0dd);
    assert_ne!(other.hashes.last(), first.hashes.last());
}

/// The command gate (`Sim::validate`): a rejected command changes nothing, neither the state nor the events. Each
/// random command runs alone in a tick, next to a copy of the game that runs the same tick without it. With debug
/// commands off, the gate rejects every one of them.
#[test]
fn rejected_commands_change_nothing() {
    for config in [SimConfig::DEBUG, SimConfig::default()] {
        let scenario = ScenarioBuilder::new(MAP)
            .spawn(
                ObjectKind::SupplyDeliveryStation,
                None,
                SpawnAt::Tile(TilePos::new(9, 4)),
            )
            .build();
        let mut sim = Sim::with_config(&scenario, config);
        let mut rng = Rng::from_seed(0x9a7e);
        let (mut accepted, mut rejected) = (0, 0);
        for _ in 0..TICKS {
            let cmd = random_command(&mut rng, &sim);
            let verdict = sim.validate(&cmd);
            let mut without = sim.clone();
            without.step(&TickInputs::default());
            sim.step(&TickInputs {
                commands: vec![cmd.clone()],
            });
            match verdict {
                Ok(()) => accepted += 1,
                Err(reason) => {
                    rejected += 1;
                    assert_eq!(sim.state_hash(), without.state_hash(), "{cmd:?}: {reason}");
                    assert_eq!(sim, without, "{cmd:?}: {reason}");
                }
            }
            check_invariants(&sim);
        }
        if config.debug {
            assert!(
                accepted > 50 && rejected > 1000,
                "{accepted} accepted, {rejected} rejected"
            );
        } else {
            assert_eq!((accepted, rejected), (0, TICKS));
        }
    }
}

/// Per-object values (`core/map_editor.md > Per-object Resource Values`) come from files, so any of them, of any
/// kind or none, must never panic validation, the editor's preview or a running game.
#[test]
fn any_spawn_settings_are_handled() {
    let extremes = [0, 1, 2, 959, 960, u32::MAX - 1, u32::MAX];
    let mut rng = Rng::from_seed(5);
    let (mut games, mut rejected) = (0, 0);
    for _ in 0..200 {
        let mut scenario = ScenarioBuilder::new(MAP).build();
        for i in 0..3 {
            let kind = ObjectKind::ALL[rng.below(ObjectKind::ALL.len() as u32) as usize];
            // Mostly values of the spawn's own kind, sometimes none or another kind's.
            let settings_kind = match rng.below(6) {
                0 => None,
                1 => Some(ObjectKind::ALL[rng.below(ObjectKind::ALL.len() as u32) as usize]),
                _ => Some(kind),
            };
            let mut value = || extremes[rng.below(extremes.len() as u32) as usize];
            let settings = settings_kind.map(|k| match k {
                ObjectKind::CrystalPatch => SpawnSettings::CrystalPatch {
                    starting_amount: value(),
                },
                ObjectKind::SupplyDeliveryStation => SpawnSettings::SupplyDeliveryStation {
                    starting_supplies: value(),
                    delivery_size: value(),
                    delivery_interval: value(),
                },
            });
            let mut spawn = Spawn::new(kind, None, SpawnAt::Tile(TilePos::new(i * 6, 0)));
            spawn.settings = settings;
            scenario.spawns.push(spawn);
        }
        let problems = scenario.validate();
        let (mut preview, _) = Sim::preview(&scenario);
        preview.step(&TickInputs::default());
        match Sim::try_new(&scenario) {
            Ok(mut sim) => {
                assert!(problems.is_empty());
                for _ in 0..1000 {
                    sim.step(&TickInputs::default());
                }
                games += 1;
            }
            Err(p) => {
                assert_eq!(p, problems);
                rejected += 1;
            }
        }
    }
    assert!(games > 0 && rejected > 0, "{games} games, {rejected} rejected");
}
