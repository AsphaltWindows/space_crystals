//! Player input never panics the sim (`ARCHITECTURE.md > Commands, events and replays`):
//! a long stream of random, mostly invalid debug commands, orders and abilities runs deterministically,
//! the commands the gate rejects change nothing, and any per-object values a file holds are
//! handled.
//!
//! Each player has a base the stream never removes ([`BASES`]): a player who loses their last structure is defeated,
//! and in a two-player game that ends it, after which every command is refused (`core/game_rules.md > End of Game`).
//! The structures the stream spawns for them come and go around it. The stream's fights can still take a base: the
//! game then ends, the tick after it checks that the gate refuses every command and that they change nothing, and the
//! stream goes on in a new game ([`fuzz_run`]; [`SEED`] says why).

use sc_sim::testing::invariants::Invariants;
use sc_sim::testing::{self, ScenarioBuilder};
use sc_sim::{
    Ability, BuildingOrientation, Command, DebugCommand, ExitDoor, ObjectId, ObjectKind, OrderKind, PlayerCommand,
    PlayerId, Reject, Replay, Rng, Rotation, SU_PER_GU, Scenario, Sim, SimConfig, SimEvent, Spawn, SpawnAt,
    SpawnSettings, Target, TickInputs, TilePos, Vec2, nav,
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
/// a player isn't defeated (plan decision T8) as soon as a debug removal takes the last structure the stream gave
/// them, and a game lasts. Only a fight can take a base ([`SEED`]).
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
/// overlaps no blocked tile and stays inside the map; I3, no unit moves more than its top speed in a tick. (M5-M1) And
/// the invariants of the game's rules hold, whatever the input (`Invariants::check_rules`: P1, O1, R1, and H1, B1 and
/// A1, the HP, the banks and the Augmentations), `rules` having seen the tick before the step that applied `inputs`.
/// Each message begins with what broke and the tick, so that [`seed_sweep`]'s list reads at a glance.
fn check_invariants(sim: &Sim, inputs: &TickInputs, rules: &mut Invariants) {
    let (map, tick) = (sim.map(), sim.tick());
    if let Err(violation) = rules.check_rules(sim, inputs) {
        panic!("{} in tick {tick}: {}", violation.invariant, violation.message);
    }
    for o in sim.objects() {
        assert!(
            o.pos.is_finite() && o.prev_pos.is_finite(),
            "a position not finite in tick {tick}: {o:?}"
        );
        match &o.placement {
            Some(p) => assert!(
                p.tiles().all(|t| map.in_bounds(t)),
                "a footprint off the map in tick {tick}: {o:?}"
            ),
            None => assert!(map.contains_point(o.pos), "off the map in tick {tick}: {o:?}"),
        }
        let def = o.def();
        if let (Some(unit), Some(silhouette)) = (&def.unit, def.silhouette()) {
            let half = silhouette / 2.0;
            if let Some(class) = unit.base_def().nav_class() {
                let grid = sim
                    .nav()
                    .grid(class, nav::size_class(silhouette))
                    .expect("a grid for each unit in the game (slice M5-Z2)");
                // A unit in the exit doorway of the building that released it (design change 0080) overlaps that
                // footprint's tiles but no other blocked tile, nor any ground of the footprint it can't stand on
                // (`ExitDoor::box_clear`; `testing::invariants` checks the rest of the doorway's rules).
                let door = (o.mover.as_ref().and_then(|mover| mover.exit))
                    .and_then(|building| ExitDoor::of(sim.object(building)?));
                let clear = match door {
                    Some(door) => door.box_clear(grid, half, o.pos),
                    None => nav::box_clear(grid, half, o.pos),
                };
                assert!(clear, "I2 in tick {tick}: {o:?}");
            }
            let moved = o.pos.distance(o.prev_pos);
            assert!(
                moved <= o.max_speed(sim.research()) + 1e-3,
                "I3 in tick {tick}: {moved} su: {o:?}"
            );
            let mover = o.mover.as_ref().expect("units move");
            assert!(
                mover.velocity.is_finite() && mover.path.iter().all(|p| p.is_finite()),
                "a velocity or path not finite in tick {tick}: {o:?}"
            );
        }
    }
    // The removal path (`ARCHITECTURE.md > Combat`) leaves no reference to an object that is gone.
    assert_eq!(
        testing::dangling_references(sim),
        [],
        "dangling references in tick {tick}"
    );
}

/// A game the stream played: its replay from the start, and the state it ended in (at the end of the stream, or in the
/// tick after the game was over).
struct Game {
    replay: Replay,
    end: Sim,
}

struct Run {
    hashes: Vec<u64>,
    /// In order: a game that ends gives way to a new one ([`fuzz_run`]).
    games: Vec<Game>,
    spawned: usize,
    removed: usize,
    /// Unit steps: a unit's position changed during a tick.
    unit_steps: usize,
}

/// The tick after a game's end (`core/game_rules.md > End of Game`): the gate refuses each command of the tick, as the
/// game is over, but for those of a player who isn't in it (`Sim::validate` checks the player first), and the tick runs
/// as one without them.
fn check_refused(sim: &Sim, inputs: &TickInputs) {
    for c in &inputs.commands {
        let known = sim.players().iter().any(|p| p.id == c.player);
        let reason = if known {
            Reject::GameOver
        } else {
            Reject::UnknownPlayer(c.player)
        };
        assert_eq!(sim.validate(c), Err(reason), "{c:?}");
    }
    let (mut with, mut without) = (sim.clone(), sim.clone());
    with.step(inputs);
    without.step(&TickInputs::default());
    assert_eq!(with.state_hash(), without.state_hash());
    assert_eq!(with, without);
}

/// [`TICKS`] ticks of the stream from `seed`, the invariants checked after each. A game that ends plays one more tick
/// of the stream, which [`check_refused`] checks, and the stream goes on in a new game of the same scenario.
fn fuzz_run(seed: u64) -> Run {
    let scenario = scenario(SimConfig::DEBUG);
    let mut sim = Sim::new(&scenario);
    let mut rules = Invariants::new(&sim);
    let mut replay = Replay::new(scenario.clone());
    let mut games = Vec::new();
    let mut rng = Rng::from_seed(seed);
    let (mut spawned, mut removed, mut unit_steps) = (0, 0, 0);
    let mut hashes = Vec::new();
    for _ in 0..TICKS {
        // A third are orders, the rest debug commands.
        let commands = (0..rng.below(8)).map(|_| random_command(&mut rng, &sim)).collect();
        let inputs = TickInputs { commands };
        let over = sim.outcome().is_some();
        if over {
            check_refused(&sim, &inputs);
        }
        sim.step(&inputs);
        check_invariants(&sim, &inputs, &mut rules);
        replay.record(inputs);
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
                | SimEvent::EnterFailed { .. }
                | SimEvent::EnteredBuilding { .. }
                | SimEvent::LeftBuilding { .. }
                | SimEvent::Evacuated { .. }
                | SimEvent::ProjectileLaunched { .. }
                | SimEvent::Blast { .. }
                | SimEvent::Missed { .. }
                | SimEvent::LocationTooClose { .. }
                | SimEvent::BuildFailed { .. }
                | SimEvent::BuildRefunded { .. }
                | SimEvent::SiteStarted { .. }
                | SimEvent::SiteCompleted { .. }
                | SimEvent::SiteBuilt { .. }
                | SimEvent::SiteCancelled { .. }
                | SimEvent::LeftSite { .. }
                | SimEvent::ExpansionStarted { .. }
                | SimEvent::ExpansionCompleted { .. }
                | SimEvent::ExpansionCancelled { .. }
                | SimEvent::UpgradeStarted { .. }
                | SimEvent::UpgradeCompleted { .. }
                | SimEvent::UpgradeCancelled { .. }
                | SimEvent::ReleaseQueued { .. }
                | SimEvent::ReleaseCancelled { .. }
                | SimEvent::AugmentQueued { .. }
                | SimEvent::AugmentCancelled { .. }
                | SimEvent::ResearchQueued { .. }
                | SimEvent::ResearchCancelled { .. }
                | SimEvent::ResearchCompleted { .. } => {}
            }
        }
        unit_steps += sim
            .objects()
            .filter(|o| o.mover.is_some() && o.pos != o.prev_pos)
            .count();
        hashes.push(sim.state_hash());
        if over {
            let end = std::mem::replace(&mut sim, Sim::new(&scenario));
            rules = Invariants::new(&sim);
            let replay = std::mem::replace(&mut replay, Replay::new(scenario.clone()));
            games.push(Game { replay, end });
        }
    }
    games.push(Game { replay, end: sim });
    Run {
        hashes,
        games,
        spawned,
        removed,
        unit_steps,
    }
}

/// The stream's seed: any seed does. A stream can end a game: its Grenadiers (slice M4-U: four grenades of 98.8
/// against the Generator's 350 HP) and Laser Turrets (M4-S1; one in its owner's Tunnel Area, since M5-Y) can destroy a
/// base. 9 of the first 256 seeds' streams do within [`TICKS`] ([`seed_sweep`]'s, slice M5-Z1; 8 at slice M4-M), but
/// this one no longer does: its one game lasts the whole stream (it ended in tick 1367 at slice M4-M, before later
/// kinds reshuffled the streams), so of this file's tests only the ignored sweep now reaches a game's end and
/// [`check_refused`].
///
/// The fuzz accepts that end rather than preventing it. Keeping the bases out of the fight would take a rule the sim
/// hasn't (a structure no damage reaches), or a stream that knows every way a unit gets within reach of a base (each
/// weapon's Range, area damage, paths, chases), which each new kind would change. And a seed whose stream happens to
/// keep both bases holds only until the next kind reshuffles the streams. So a game's end is one more thing the stream
/// does: [`check_refused`] checks the tick after it, and the stream goes on in a new game, so that every seed plays
/// all of its commands. The counts the test asserts lie well under those of any of the 256 streams (spawned 62 to
/// 116, removed 59 to 103, unit steps 571 to 4239, commands 6684 to 7273; slice M5-Z1), so that a reshuffle keeps
/// them too.
const SEED: u64 = 0x5eed;

#[test]
fn random_debug_commands_never_panic_and_stay_deterministic() {
    let first = fuzz_run(SEED);
    // The stream exercises the valid paths too, not only rejections.
    assert!(
        first.spawned > 40 && first.removed > 40,
        "{} spawned, {} removed",
        first.spawned,
        first.removed
    );
    assert!(first.unit_steps > 400, "{} unit steps", first.unit_steps);
    let commands: usize = (first.games.iter())
        .flat_map(|g| &g.replay.inputs)
        .map(|i| i.commands.len())
        .sum();
    assert!(commands > 4000, "{commands} commands");

    let second = fuzz_run(SEED);
    assert_eq!(first.hashes, second.hashes);
    assert_eq!(first.games.len(), second.games.len());
    for (a, b) in first.games.iter().zip(&second.games) {
        assert_eq!(a.end, b.end);
    }

    // Each game's recorded stream replays to the same state. (Not through JSON: it has no
    // numbers for the NaN and infinite positions in this stream, and serde_json writes them as
    // `null`, which it can't read back. The state itself never holds such values.)
    for game in &first.games {
        assert_eq!(game.replay.run().state_hash(), game.end.state_hash());
        let state: Sim = serde_json::from_str(&serde_json::to_string(&game.end).unwrap()).unwrap();
        assert_eq!(state.state_hash(), game.end.state_hash());
    }

    let other = fuzz_run(0x0dd);
    assert_ne!(other.hashes.last(), first.hashes.last());
}

/// How many seeds [`seed_sweep`] plays: 0 to 255.
const SWEEP_SEEDS: u64 = 256;

/// [`fuzz_run`] from each of the first [`SWEEP_SEEDS`] seeds, every one that panics listed with its message. [`SEED`]
/// is one stream, which the next kind or rule reshuffles (slice M5-Z1: seed 155 broke I2 for a Recruit while the
/// fixed seed passed), so a slice that changes the sim runs this before it reports, in release (4.1 to 4.6 s at load
/// 4 to 9, slice M5-Z1):
/// `cargo test -p sc_sim --release --test sim robustness::seed_sweep -- --ignored --nocapture`.
#[test]
#[ignore]
fn seed_sweep() {
    let failed: Vec<(u64, String)> = (0..SWEEP_SEEDS)
        .filter_map(|seed| {
            let panic = std::panic::catch_unwind(|| fuzz_run(seed)).err()?;
            let message = (panic.downcast_ref::<String>().cloned())
                .or_else(|| panic.downcast_ref::<&str>().map(|s| s.to_string()))
                .unwrap_or_else(|| "a panic without a message".to_string());
            Some((seed, message))
        })
        .collect();
    for (seed, message) in &failed {
        eprintln!("seed {seed}: {message}");
    }
    let seeds: Vec<u64> = failed.iter().map(|&(seed, _)| seed).collect();
    assert!(
        failed.is_empty(),
        "{} of {SWEEP_SEEDS} seeds failed: {seeds:?}",
        failed.len()
    );
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
        let mut rules = Invariants::new(&sim);
        let mut rng = Rng::from_seed(0x9a7e);
        let (mut accepted, mut rejected) = (0, 0);
        for _ in 0..TICKS {
            let cmd = random_command(&mut rng, &sim);
            let verdict = sim.validate(&cmd);
            let mut without = sim.clone();
            without.step(&TickInputs::default());
            let inputs = TickInputs {
                commands: vec![cmd.clone()],
            };
            sim.step(&inputs);
            match verdict {
                Ok(()) => accepted += 1,
                Err(reason) => {
                    rejected += 1;
                    assert_eq!(sim.state_hash(), without.state_hash(), "{cmd:?}: {reason}");
                    assert_eq!(sim, without, "{cmd:?}: {reason}");
                }
            }
            check_invariants(&sim, &inputs, &mut rules);
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
                | ObjectKind::Hideout
                | ObjectKind::OpticsLab
                | ObjectKind::ChemicalPlant
                | ObjectKind::Grenadier
                | ObjectKind::LaserTurret
                | ObjectKind::Recruit
                | ObjectKind::Militant
                | ObjectKind::Gunner
                | ObjectKind::RecruitmentHall
                | ObjectKind::Storage
                | ObjectKind::Armory
                | ObjectKind::Gunworks
                | ObjectKind::Garrison
                | ObjectKind::ChopShop
                | ObjectKind::MachineShop
                | ObjectKind::Airstrip
                | ObjectKind::ScrapHangar
                | ObjectKind::Altar
                | ObjectKind::Temple
                | ObjectKind::Cathedral
                | ObjectKind::Shrine
                | ObjectKind::Disciple
                | ObjectKind::Seeker
                | ObjectKind::Sparrow
                | ObjectKind::HandOfHeaven
                | ObjectKind::PrayerMobile
                | ObjectKind::CareerCampus
                | ObjectKind::FieldEngineer => None,
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
