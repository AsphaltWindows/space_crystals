//! The golden games' harness: a game's definition ([`Game`], [`Script`]), its recording, and the comparison with its
//! blessed file ([`golden`]).

use std::path::PathBuf;

use sc_sim::{PlayerCommand, Replay, SU_PER_GU, Scenario, Sim, Tick, TickInputs, Vec2};

/// A golden replay notes the state hash every this many ticks (2 s), at tick 0 and at its end.
const EVERY: Tick = 32;

/// A golden game: a scenario, the commands given at each tick, and the tick it is played to. `expect` checks that
/// the game still does what it is there for (its doc comment) at its end, and `during` along the way, after every
/// tick, so that a re-bless can't quietly fix a game that no longer fights, moves or chases. A game its players play
/// by script has a [`Script`] too.
pub(crate) struct Game {
    pub(crate) scenario: Scenario,
    pub(crate) commands: Vec<(Tick, PlayerCommand)>,
    pub(crate) script: Option<Box<dyn Script>>,
    pub(crate) end: Tick,
    pub(crate) during: fn(Tick, &Sim),
    pub(crate) expect: fn(&Sim),
}

/// The players of a golden game that play by script, the way a player would (`sc_sim::testing::build_order`,
/// `sc_sim::testing::colonists`): asked before each step for their commands, from the game as the last step left it,
/// and told when it is over, to check what they saw along the way. Their commands are recorded with the game's own,
/// so the blessed file plays back without them; as they follow the game, they are part of how it plays out, and are
/// compared as such.
pub(crate) trait Script {
    /// The commands for the step to tick `sim.tick() + 1`, after the game's own for it.
    fn commands(&mut self, sim: &Sim) -> Vec<PlayerCommand>;
    /// The game is over, as its last step left it.
    fn end(self: Box<Self>, sim: &Sim);
}

/// Plays `game` and records it: the replay with its checkpoints, and the game at its end. A `script` gives its
/// commands before each step.
fn record(game: &Game, mut script: Option<Box<dyn Script>>) -> (Replay, Sim) {
    let mut inputs = vec![TickInputs::default(); game.end as usize];
    for (tick, command) in &game.commands {
        inputs[*tick as usize - 1].commands.push(command.clone());
    }
    let mut replay = Replay::new(game.scenario.clone());
    let mut sim = Sim::new(&game.scenario);
    replay.checkpoint(&sim);
    for mut step in inputs {
        if let Some(script) = &mut script {
            step.commands.extend(script.commands(&sim));
        }
        sim.step(&step);
        replay.record(step);
        (game.during)(sim.tick(), &sim);
        if sim.tick().is_multiple_of(EVERY) || sim.tick() == game.end {
            replay.checkpoint(&sim);
        }
    }
    if let Some(script) = script {
        script.end(&sim);
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
/// `SC_BLESS=1`. The map takes the game's name, and the camera starts over its middle unless the game says where,
/// for watching it in the client. A scripted game's commands come from the game as it goes: they are compared after
/// its checkpoints, as a part of how it plays out, not of its definition.
pub(crate) fn golden(name: &str, game: fn() -> Game) {
    let mut game = game();
    let map = &mut game.scenario.map;
    map.name = name.to_string();
    let middle = Vec2::new(map.terrain.width() as f32, map.terrain.height() as f32) * SU_PER_GU / 2.0;
    for slot in &mut map.slots {
        slot.camera_start.get_or_insert(middle);
    }
    let script = game.script.take();
    let scripted = script.is_some();
    let (replay, sim) = record(&game, script);
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
        blessed.scenario == replay.scenario && (scripted || blessed.inputs == replay.inputs),
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
    let commands =
        (0..replay.inputs.len().max(blessed.inputs.len())).find(|&i| replay.inputs.get(i) != blessed.inputs.get(i));
    if let Some(i) = commands {
        panic!(
            "golden {name}: the scripts' commands for tick {} differ from the blessed ones; {BLESS}",
            i + 1
        );
    }
}
