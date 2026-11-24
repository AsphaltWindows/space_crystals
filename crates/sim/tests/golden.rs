//! Golden hashes (`ARCHITECTURE.md > Replays and golden hashes`): six small games, defined below and recorded as
//! replays in `tests/golden/`, each with the state hash at tick 0, every [`EVERY`] ticks and at its end. A run must
//! give the blessed hashes: a change that alters how any of these games plays out fails here, at the first checkpoint
//! that differs. The replays play in the client too (`cargo run -- --replay crates/sim/tests/golden/<name>.json`).
//!
//! When the change is intended, re-bless the files and review the diff (the checkpoint hashes) before committing:
//!
//! ```sh
//! SC_BLESS=1 cargo test -p sc_sim golden
//! ```

use std::path::PathBuf;

use sc_sim::testing::ScenarioBuilder;
use sc_sim::{
    Ability, Command, Faction, ObjectId, ObjectKind, OrderKind, PlayerCommand, PlayerId, Replay, RiotShield, SU_PER_GU,
    Scenario, Sim, SpawnAt, Target, Tick, TickInputs, TilePos, Vec2,
};

const P1: PlayerId = PlayerId(1);
const P2: PlayerId = PlayerId(2);
const PK: ObjectKind = ObjectKind::Peacekeeper;
const GUARD: ObjectKind = ObjectKind::Guard;
/// A golden replay notes the state hash every this many ticks (2 s), at tick 0 and at its end.
const EVERY: Tick = 32;

/// A golden game: a scenario, the commands given at each tick, and the tick it is played to. `expect` checks that
/// the game still does what it is there for (its doc comment) at its end, and `during` along the way, after every
/// tick, so that a re-bless can't quietly fix a game that no longer fights, moves or chases.
struct Game {
    scenario: Scenario,
    commands: Vec<(Tick, PlayerCommand)>,
    end: Tick,
    during: fn(Tick, &Sim),
    expect: fn(&Sim),
}

/// Open Plane ground, `w` × `h` tiles.
fn open(w: usize, h: usize) -> String {
    format!("{}\n", ".".repeat(w)).repeat(h)
}

/// A unit's spawn at the centre of tile `(x, y)`.
fn at(x: i32, y: i32) -> SpawnAt {
    SpawnAt::Point(TilePos::new(x, y).center())
}

fn point(x: i32, y: i32) -> Vec2 {
    TilePos::new(x, y).center()
}

fn order(player: PlayerId, units: impl IntoIterator<Item = u32>, kind: OrderKind) -> PlayerCommand {
    PlayerCommand {
        player,
        command: Command::Order {
            units: units.into_iter().map(ObjectId).collect(),
            kind,
            queue: false,
        },
    }
}

/// Player 2 plays the Syndicate, whose unit the Guard is.
fn syndicate_p2(builder: ScenarioBuilder) -> Scenario {
    let mut scenario = builder.build();
    scenario.setup.players[1].faction = Faction::Syndicate;
    scenario
}

/// Player 1's `units` raise (`raise`) or lower their Riot Shields.
fn shield(units: impl IntoIterator<Item = u32>, raise: bool) -> PlayerCommand {
    PlayerCommand {
        player: P1,
        command: Command::Ability {
            units: units.into_iter().map(ObjectId).collect(),
            ability: Ability::RiotShield { raise },
        },
    }
}

fn units_of(sim: &Sim, player: PlayerId) -> Vec<&sc_sim::Object> {
    sim.objects().filter(|o| o.owner == Some(player)).collect()
}

/// Three Peacekeepers (player 1, ids 1–3) against three Guards (player 2, ids 4–6), in two columns 4 tiles apart:
/// the Peacekeepers are ordered to attack the middle Guard, the Guards acquire their targets by themselves. It ends
/// with one side gone.
fn duel_3v3() -> Game {
    let mut builder = ScenarioBuilder::new(&open(16, 10));
    for y in [3, 5, 7] {
        builder = builder.spawn(PK, Some(P1), at(5, y));
    }
    for y in [3, 5, 7] {
        builder = builder.spawn(GUARD, Some(P2), at(9, y));
    }
    Game {
        scenario: syndicate_p2(builder),
        commands: vec![(1, order(P1, 1..=3, OrderKind::Attack(ObjectId(5))))],
        end: 240,
        during: |_, _| {},
        expect: |sim| {
            let (p1, p2) = (units_of(sim, P1).len(), units_of(sim, P2).len());
            assert!(p1 == 0 || p2 == 0, "one side is gone: {p1} against {p2}");
        },
    }
}

/// Twenty units a side, five rows of Peacekeepers and Guards in turn, in blocks at opposite ends of the map, are
/// ordered to AttackMove to the other block's place: they meet in the middle and fight.
fn attack_move_battle() -> Game {
    let mut builder = ScenarioBuilder::new(&open(40, 20));
    for (player, x0, y0) in [(P1, 3, 6), (P2, 33, 9)] {
        for y in 0..5 {
            for x in 0..4 {
                let kind = if y % 2 == 0 { PK } else { GUARD };
                builder = builder.spawn(kind, Some(player), at(x0 + x, y0 + y));
            }
        }
    }
    Game {
        scenario: syndicate_p2(builder),
        commands: vec![
            (1, order(P1, 1..=20, OrderKind::AttackMove(point(35, 11)))),
            (1, order(P2, 21..=40, OrderKind::AttackMove(point(4, 8)))),
        ],
        end: 480,
        during: |_, _| {},
        expect: |sim| {
            let left = sim.objects().count();
            assert!(left <= 30, "at least ten died: {left} are left");
        },
    }
}

/// Twelve Peacekeepers in a block west of a mountain wall, ordered as one group to a point east of it, cross it
/// through a gap two tiles wide.
fn choke_group_move() -> Game {
    let wall = format!("{}M{}\n", ".".repeat(15), ".".repeat(16));
    let gap = open(32, 1);
    let rows: Vec<&str> = (0..16)
        .map(|y| if y == 7 || y == 8 { &gap } else { &wall })
        .map(String::as_str)
        .collect();
    let mut builder = ScenarioBuilder::new(&rows.concat());
    for y in 6..=8 {
        for x in 3..=6 {
            builder = builder.spawn(PK, Some(P1), at(x, y));
        }
    }
    Game {
        scenario: builder.build(),
        commands: vec![(1, order(P1, 1..=12, OrderKind::Move(Target::Point(point(26, 8)))))],
        end: 480,
        during: |_, _| {},
        expect: |sim| {
            let wall_east = 16.0 * SU_PER_GU;
            for unit in units_of(sim, P1) {
                assert!(
                    unit.pos.x > wall_east,
                    "#{} is through the gap: {}",
                    unit.id.0,
                    unit.pos
                );
            }
        },
    }
}

/// Three Peacekeepers (ids 1–3) patrol between their place and a point 14 tiles east. A Guard of player 2 (id 4)
/// stands near the route: the patrol engages it, kills it and walks on.
fn patrol() -> Game {
    let mut builder = ScenarioBuilder::new(&open(32, 12));
    for y in [4, 5, 6] {
        builder = builder.spawn(PK, Some(P1), at(4, y));
    }
    builder = builder.spawn(GUARD, Some(P2), at(14, 9));
    Game {
        scenario: syndicate_p2(builder),
        commands: vec![(1, order(P1, 1..=3, OrderKind::Patrol(point(18, 5))))],
        end: 640,
        during: |_, _| {},
        expect: |sim| {
            assert!(sim.object(ObjectId(4)).is_none(), "the Guard is dead");
            let patrolling = units_of(sim, P1)
                .iter()
                .filter(|u| {
                    let current = u.orders.as_ref().and_then(|o| o.current.as_ref());
                    current.is_some_and(|c| matches!(c.order.kind, OrderKind::Patrol(_)))
                })
                .count();
            assert!(patrolling > 0, "the patrol goes on");
        },
    }
}

/// A Peacekeeper (player 1, id 1) attacks a Guard (player 2, id 2) that runs away east, faster than it: the
/// Peacekeeper shoots while it can, chases, loses the Guard in the fog, walks to where it last saw it and stops
/// there (design change 0034).
fn fog_chase() -> Game {
    let builder = ScenarioBuilder::new(&open(48, 10))
        .spawn(PK, Some(P1), at(4, 5))
        .spawn(GUARD, Some(P2), at(8, 5));
    Game {
        scenario: syndicate_p2(builder),
        commands: vec![
            (1, order(P2, [2], OrderKind::Move(Target::Point(point(44, 5))))),
            (1, order(P1, [1], OrderKind::Attack(ObjectId(2)))),
        ],
        end: 400,
        during: |_, _| {},
        expect: |sim| {
            let (chaser, runner) = (sim.object(ObjectId(1)).unwrap(), sim.object(ObjectId(2)).unwrap());
            assert!(runner.hp < Some(80.0), "the chaser hit it before it got away");
            assert!(!sim.is_visible(P1, runner), "out of the chaser's sight");
            assert!(chaser.orders.as_ref().unwrap().is_idle(), "the chase ended");
            assert!(
                chaser.pos.x > 9.0 * SU_PER_GU,
                "it chased past the Guard's start: {}",
                chaser.pos
            );
        },
    }
}

/// Player 1 raises the Riot Shields of three Peacekeepers (ids 1–3, `Command::Ability`) and walks them, at half
/// speed, up to a Guard (player 2, id 4), which shoots at them: behind their shields they don't shoot back. On the
/// way #3 lowers its shield and raises it again before it is down. At tick [`SHIELDS_DOWN`] all three lower their
/// shields, and then they kill the Guard.
fn riot_shield() -> Game {
    let mut builder = ScenarioBuilder::new(&open(16, 10));
    for y in [4, 5, 6] {
        builder = builder.spawn(PK, Some(P1), at(3, y));
    }
    builder = builder.spawn(GUARD, Some(P2), at(11, 5));
    Game {
        scenario: syndicate_p2(builder),
        commands: vec![
            (1, shield(1..=3, true)),
            (1, order(P1, 1..=3, OrderKind::Move(Target::Point(point(8, 5))))),
            (30, shield([3], false)),
            (32, shield([3], true)),
            (SHIELDS_DOWN, shield(1..=3, false)),
        ],
        end: 240,
        during: |tick, sim| {
            if tick == SHIELDS_DOWN - 1 {
                let guard = sim.object(ObjectId(4)).expect("the Guard lives");
                assert_eq!(guard.hp, Some(80.0), "the shielded Peacekeepers didn't shoot");
                let units = units_of(sim, P1);
                assert!(units.iter().any(|u| u.hp < Some(50.0)), "the Guard shot at them");
                for unit in units {
                    assert_eq!(unit.riot_shield, Some(RiotShield::Raised), "#{}", unit.id.0);
                }
            }
        },
        expect: |sim| {
            assert!(sim.object(ObjectId(4)).is_none(), "the Guard is dead");
            let units = units_of(sim, P1);
            assert_eq!(units.len(), 3, "no Peacekeeper died");
            for unit in units {
                assert_eq!(unit.riot_shield, Some(RiotShield::Lowered), "#{}", unit.id.0);
            }
        },
    }
}

/// [`riot_shield`]: the Peacekeepers lower their shields in this tick.
const SHIELDS_DOWN: Tick = 120;

/// Plays `game` and records it: the replay with its checkpoints, and the game at its end.
fn record(game: &Game) -> (Replay, Sim) {
    let mut inputs = vec![TickInputs::default(); game.end as usize];
    for (tick, command) in &game.commands {
        inputs[*tick as usize - 1].commands.push(command.clone());
    }
    let mut replay = Replay::new(game.scenario.clone());
    let mut sim = Sim::new(&game.scenario);
    replay.checkpoint(&sim);
    for step in inputs {
        sim.step(&step);
        replay.record(step);
        (game.during)(sim.tick(), &sim);
        if sim.tick().is_multiple_of(EVERY) || sim.tick() == game.end {
            replay.checkpoint(&sim);
        }
    }
    (replay, sim)
}

/// A golden file: the replay's JSON with the scenario pretty-printed (the map one line per row) and one line per
/// tick's inputs and per checkpoint, so that a re-bless shows as a short diff.
fn golden_json(replay: &Replay) -> String {
    let scenario = serde_json::to_string_pretty(&replay.scenario)
        .unwrap()
        .replace('\n', "\n  ");
    let lines = |items: Vec<String>| items.join(",\n    ");
    let inputs = replay
        .inputs
        .iter()
        .map(|i| serde_json::to_string(i).unwrap())
        .collect();
    let checkpoints = replay
        .checkpoints
        .iter()
        .map(|c| serde_json::to_string(c).unwrap())
        .collect();
    format!(
        "{{\n  \"scenario\": {scenario},\n  \"inputs\": [\n    {}\n  ],\n  \"checkpoints\": [\n    {}\n  ]\n}}\n",
        lines(inputs),
        lines(checkpoints)
    )
}

fn path(name: &str) -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("tests/golden")
        .join(format!("{name}.json"))
}

const BLESS: &str = "re-bless if the change is intended: SC_BLESS=1 cargo test -p sc_sim golden";

/// Plays the golden game `name` and compares its checkpoints with its blessed file, or rewrites the file when
/// `SC_BLESS=1`. The map takes the game's name, and the camera starts over its middle, for watching it in the
/// client.
fn golden(name: &str, game: fn() -> Game) {
    let mut game = game();
    let map = &mut game.scenario.map;
    map.name = name.to_string();
    let middle = Vec2::new(map.terrain.width() as f32, map.terrain.height() as f32) * SU_PER_GU / 2.0;
    for slot in &mut map.slots {
        slot.camera_start = Some(middle);
    }
    let (replay, sim) = record(&game);
    (game.expect)(&sim);
    let path = path(name);
    if std::env::var("SC_BLESS").is_ok_and(|v| v == "1") {
        let json = golden_json(&replay);
        assert_eq!(serde_json::from_str::<Replay>(&json).unwrap(), replay);
        std::fs::create_dir_all(path.parent().unwrap()).unwrap();
        std::fs::write(&path, json).unwrap();
        return;
    }
    let text = std::fs::read_to_string(&path).unwrap_or_else(|e| panic!("{}: {e}; {BLESS}", path.display()));
    let blessed: Replay = serde_json::from_str(&text).unwrap_or_else(|e| {
        panic!(
            "golden {name}: {} doesn't load as a replay ({e}); {BLESS}",
            path.display()
        )
    });
    assert!(
        blessed.scenario == replay.scenario && blessed.inputs == replay.inputs,
        "golden {name}: its definition changed since it was blessed (the scenario or the commands); {BLESS}"
    );
    let first_difference = (0..replay.checkpoints.len().max(blessed.checkpoints.len()))
        .find(|&i| replay.checkpoints.get(i) != blessed.checkpoints.get(i));
    if let Some(i) = first_difference {
        let tick = |list: &[sc_sim::Checkpoint]| list.get(i).map(|c| c.tick);
        let at = tick(&replay.checkpoints)
            .or(tick(&blessed.checkpoints))
            .unwrap_or_default();
        let hash = |list: &[sc_sim::Checkpoint]| {
            list.get(i)
                .map_or_else(|| "none".to_string(), |c| format!("{:#018x}", c.hash))
        };
        let before = match i.checked_sub(1) {
            Some(j) => format!(
                "the checkpoint before, at tick {}, still matches",
                replay.checkpoints[j].tick
            ),
            None => "its first checkpoint: the starting state differs".to_string(),
        };
        panic!(
            "golden {name}: the game diverges at tick {at}: state hash {}, blessed {} ({before}); {BLESS}",
            hash(&replay.checkpoints),
            hash(&blessed.checkpoints),
        );
    }
}

#[test]
fn golden_duel_3v3() {
    golden("duel_3v3", duel_3v3);
}

#[test]
fn golden_attack_move_battle() {
    golden("attack_move_battle", attack_move_battle);
}

#[test]
fn golden_choke_group_move() {
    golden("choke_group_move", choke_group_move);
}

#[test]
fn golden_patrol() {
    golden("patrol", patrol);
}

#[test]
fn golden_fog_chase() {
    golden("fog_chase", fog_chase);
}

#[test]
fn golden_riot_shield() {
    golden("riot_shield", riot_shield);
}
