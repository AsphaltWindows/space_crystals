//! Player input never panics the sim (`ARCHITECTURE.md > Commands, events and replays`):
//! a long stream of random, mostly invalid debug commands, orders and abilities runs deterministically,
//! the commands the gate rejects change nothing, and any per-object values a file holds are
//! handled.
//!
//! Each player has a base the stream never removes ([`BASES`]): a player who loses their last structure is defeated,
//! and in a two-player game that ends it, after which every command is refused (`core/game_rules.md > End of Game`).
//! The structures the stream spawns for them come and go around it.

use sc_sim::testing::{self, ScenarioBuilder};
use sc_sim::{
    Ability, BuildingOrientation, Command, DebugCommand, ObjectId, ObjectKind, OrderKind, PlayerCommand, PlayerId,
    Replay, Rng, Rotation, SU_PER_GU, Scenario, Sim, SimConfig, SimEvent, Spawn, SpawnAt, SpawnSettings, Target,
    TickInputs, TilePos, Vec2, nav,
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

/// The players' bases, a Generator each in a corner of [`MAP`] ([`scenario`]): the stream never removes them, so that
/// nobody is defeated and the game goes on to the end of the stream (plan decision T8).
const BASES: [ObjectId; 2] = [ObjectId(2), ObjectId(3)];

/// [`MAP`] with a Supply Delivery Station in the cliffs' ring, and the [`BASES`].
fn scenario(config: SimConfig) -> Scenario {
    ScenarioBuilder::new(MAP)
        .spawn(
            ObjectKind::SupplyDeliveryStation,
            None,
            SpawnAt::Tile(TilePos::new(9, 4)),
        )
        .spawn(
            ObjectKind::Generator,
            Some(PlayerId(1)),
            SpawnAt::Tile(TilePos::new(0, 14)),
        )
        .spawn(
            ObjectKind::Generator,
            Some(PlayerId(2)),
            SpawnAt::Tile(TilePos::new(22, 0)),
        )
        .config(config)
        .build()
}

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

/// An order to a few ids, often the issuing player's units, sometimes anyone's or none; one in six raises or
/// lowers their Riot Shields instead.
fn random_order(rng: &mut Rng, sim: &Sim, player: PlayerId) -> Command {
    let own: Vec<ObjectId> = sim
        .objects()
        .filter(|o| o.owner == Some(player) && o.orders.is_some())
        .map(|o| o.id)
        .collect();
    let units: Vec<ObjectId> = (0..rng.below(6))
        .map(|_| match (rng.below(3), own.is_empty()) {
            (0, _) | (_, true) => random_id(rng, sim),
            _ => own[rng.below(own.len() as u32) as usize],
        })
        .collect();
    if rng.below(6) == 0 {
        let ability = Ability::RiotShield {
            raise: rng.below(2) == 0,
        };
        return Command::Ability { units, ability };
    }
    // Attacks often go for the destructible object nearest to the first unit, which may be in range.
    let nearest = units.first().and_then(|&id| {
        let unit = sim.object(id)?;
        sim.nearest_object(unit.pos, |o| o.id != id && o.hp.is_some())
            .map(|o| o.id)
    });
    // Half of them moves to a point.
    let kind = match rng.below(12) {
        0 => OrderKind::Move(Target::Object(random_id(rng, sim))),
        1 => {
            // Drawn either way, so that the stream of random numbers stays the same.
            let random = random_id(rng, sim);
            OrderKind::Attack(match nearest {
                Some(id) if random.0.is_multiple_of(2) => id,
                _ => random,
            })
        }
        2 => OrderKind::AttackMove(random_point(rng, sim)),
        3 => OrderKind::Patrol(random_point(rng, sim)),
        4 => OrderKind::Hold,
        5 => OrderKind::Stop,
        _ => OrderKind::Move(Target::Point(random_point(rng, sim))),
    };
    Command::Order {
        units,
        kind,
        queue: rng.below(3) == 0,
    }
}

fn random_command(rng: &mut Rng, sim: &Sim) -> PlayerCommand {
    let player = PlayerId(rng.below(4) as u8);
    if rng.below(3) == 0 {
        return PlayerCommand {
            player,
            command: random_order(rng, sim, player),
        };
    }
    let command = match rng.below(3) {
        0 => {
            // A unit half the time, so that the stream keeps units moving; else any kind, structures included.
            let kind = if rng.below(2) == 0 {
                [ObjectKind::Peacekeeper, ObjectKind::Guard][rng.below(2) as usize]
            } else {
                ObjectKind::ALL[rng.below(ObjectKind::ALL.len() as u32) as usize]
            };
            // Units mostly belong to someone, so that orders find them.
            let unit = kind.def().unit.is_some();
            let owned = if unit { rng.below(4) != 0 } else { rng.below(3) == 0 };
            let owner = owned.then(|| PlayerId(rng.below(4) as u8));
            // Mostly the location type of the kind's footprint: units at points, the rest on tiles.
            let at = if unit == (rng.below(4) != 0) {
                SpawnAt::Point(random_point(rng, sim))
            } else {
                let map = sim.map();
                SpawnAt::Tile(TilePos::new(
                    tile_coord(rng, map.width()),
                    tile_coord(rng, map.height()),
                ))
            };
            // Any of the 16 ways to turn a footprint (structures turn; the rest ignore it or keep it).
            let orientation = BuildingOrientation {
                rotation: [Rotation::R0, Rotation::R90, Rotation::R180, Rotation::R270][rng.below(4) as usize],
                flip_h: rng.below(2) == 0,
                flip_v: rng.below(2) == 0,
            };
            DebugCommand::Spawn {
                kind,
                owner,
                at,
                orientation,
            }
        }
        1 => {
            // Never a base: an id that never exists instead.
            let id = random_id(rng, sim);
            DebugCommand::Remove {
                id: if BASES.contains(&id) { ObjectId(0) } else { id },
            }
        }
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

/// Every object stays where the sim's rules allow it. Units (`docs/movement.md > 8`): I2, a ground unit's box
/// overlaps no blocked tile and stays inside the map; I3, no unit moves more than its top speed in a tick.
fn check_invariants(sim: &Sim) {
    let map = sim.map();
    for o in sim.objects() {
        assert!(o.pos.is_finite() && o.prev_pos.is_finite(), "{o:?}");
        match &o.placement {
            Some(p) => assert!(p.tiles().all(|t| map.in_bounds(t)), "{o:?}"),
            None => assert!(map.contains_point(o.pos), "{o:?}"),
        }
        let def = o.def();
        if let (Some(unit), Some(silhouette)) = (&def.unit, def.silhouette()) {
            if let Some(class) = unit.base_def().nav_class() {
                let grid = sim
                    .nav()
                    .grid(class, nav::size_class(silhouette))
                    .expect("a grid per unit type");
                // But for a unit in the exit doorway of the building that released it (design change 0080;
                // `tests/production_exit.rs` checks those).
                let in_doorway = o.mover.as_ref().is_some_and(|mover| mover.exit.is_some());
                assert!(in_doorway || nav::box_clear(grid, silhouette / 2.0, o.pos), "I2: {o:?}");
            }
            let moved = o.pos.distance(o.prev_pos);
            assert!(moved <= unit.movement.max_speed() + 1e-3, "I3: {moved} su: {o:?}");
            let mover = o.mover.as_ref().expect("units move");
            assert!(
                mover.velocity.is_finite() && mover.path.iter().all(|p| p.is_finite()),
                "{o:?}"
            );
        }
    }
    // The removal path (`ARCHITECTURE.md > Combat`) leaves no reference to an object that is gone.
    assert_eq!(testing::dangling_references(sim), []);
}

struct Run {
    hashes: Vec<u64>,
    replay: Replay,
    final_state: Sim,
    spawned: usize,
    removed: usize,
    /// Unit steps: a unit's position changed during a tick.
    unit_steps: usize,
}

fn fuzz_run(seed: u64) -> Run {
    let scenario = scenario(SimConfig::DEBUG);
    let mut sim = Sim::new(&scenario);
    let mut replay = Replay::new(scenario);
    let mut rng = Rng::from_seed(seed);
    let (mut spawned, mut removed, mut unit_steps) = (0, 0, 0);
    let mut hashes = Vec::new();
    for _ in 0..TICKS {
        // A third are orders, the rest debug commands.
        let commands = (0..rng.below(8)).map(|_| random_command(&mut rng, &sim)).collect();
        let inputs = TickInputs { commands };
        sim.step(&inputs);
        replay.record(inputs);
        check_invariants(&sim);
        for e in sim.events() {
            match e {
                SimEvent::Spawned(_) => spawned += 1,
                SimEvent::Removed(_) => removed += 1,
                SimEvent::SuppliesDelivered { .. }
                | SimEvent::SupplyStationEmpty { .. }
                | SimEvent::MoveFailed { .. }
                | SimEvent::Attacked { .. }
                | SimEvent::Hit { .. }
                | SimEvent::Died { .. }
                | SimEvent::PatchDepleted { .. }
                | SimEvent::ProductionQueued { .. }
                | SimEvent::ProductionRefused { .. }
                | SimEvent::ProductionCancelled { .. }
                | SimEvent::ProductionWaiting { .. }
                | SimEvent::UnitProduced { .. }
                | SimEvent::PlayerDefeated { .. }
                | SimEvent::GameOver { .. }
                | SimEvent::ConstructionCompleted { .. }
                | SimEvent::ConstructionCancelled { .. }
                | SimEvent::StructurePlaced { .. }
                | SimEvent::PlacementRefused { .. }
                | SimEvent::KindChanged { .. }
                | SimEvent::ResourcesDelivered { .. }
                | SimEvent::TransformCompleted { .. }
                | SimEvent::TransformCancelled { .. }
                | SimEvent::DeployFailed { .. }
                | SimEvent::TargetTooClose { .. }
                | SimEvent::EnteredNetwork { .. }
                | SimEvent::LeftNetwork { .. }
                | SimEvent::EnterFailed { .. } => {}
            }
        }
        unit_steps += sim
            .objects()
            .filter(|o| o.mover.is_some() && o.pos != o.prev_pos)
            .count();
        hashes.push(sim.state_hash());
    }
    Run {
        hashes,
        replay,
        final_state: sim,
        spawned,
        removed,
        unit_steps,
    }
}

#[test]
fn random_debug_commands_never_panic_and_stay_deterministic() {
    let first = fuzz_run(0x5eed);
    // The stream exercises the valid paths too, not only rejections.
    assert!(
        first.spawned > 80 && first.removed > 50,
        "{} spawned, {} removed",
        first.spawned,
        first.removed
    );
    assert!(first.unit_steps > 1000, "{} unit steps", first.unit_steps);
    // The bases stood through it all: nobody was defeated, and every command had its chance.
    assert!(
        first.final_state.players().iter().all(|p| !p.is_defeated()) && first.final_state.outcome().is_none(),
        "{:?}",
        first.final_state.players()
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
/// commands off, the gate rejects every one of them. (The construction commands have their own such test, with the
/// one rejection the sim reports: `tests/construction/`.)
#[test]
fn rejected_commands_change_nothing() {
    for config in [SimConfig::DEBUG, SimConfig::default()] {
        let scenario = scenario(config);
        let mut sim = Sim::new(&scenario);
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
            let settings = settings_kind.and_then(|k| match k {
                ObjectKind::CrystalPatch => Some(SpawnSettings::CrystalPatch {
                    starting_amount: value(),
                }),
                ObjectKind::SupplyDeliveryStation => Some(SpawnSettings::SupplyDeliveryStation {
                    starting_supplies: value(),
                    delivery_size: value(),
                    delivery_interval: value(),
                }),
                // Units and structures have no per-object values.
                ObjectKind::Peacekeeper
                | ObjectKind::Guard
                | ObjectKind::DeploymentCenter
                | ObjectKind::Generator
                | ObjectKind::Barracks
                | ObjectKind::ExtractionFacility
                | ObjectKind::ExtractionPlate
                | ObjectKind::Drone
                | ObjectKind::Beacon
                | ObjectKind::EgressNode
                | ObjectKind::RoboPortal
                | ObjectKind::BioPortal
                | ObjectKind::SabreLegionnaire
                | ObjectKind::RifleLegionnaire
                | ObjectKind::PeacekeeperShielded
                | ObjectKind::Agent
                | ObjectKind::Tunnel
                | ObjectKind::Hideout => None,
            });
            let mut spawn = Spawn::new(kind, None, SpawnAt::Tile(TilePos::new(i * 6, 0)));
            spawn.settings = settings;
            scenario.map.spawns.push(spawn);
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
