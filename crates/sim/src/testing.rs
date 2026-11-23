//! Test helpers, public so client tests can reuse them (`ARCHITECTURE.md > Sim scenarios
//! and tests`). They panic on misuse, since they are meant for tests only.
//!
//! ```
//! use sc_sim::testing::ScenarioBuilder;
//! use sc_sim::{DebugCommand, ObjectKind, SpawnAt, TilePos};
//!
//! let mut test = ScenarioBuilder::new("
//!     ....
//!     .~~.
//! ")
//! .spawn(ObjectKind::CrystalPatch, None, SpawnAt::Tile(TilePos::new(0, 0)))
//! .start();
//! test.debug_at(3, DebugCommand::Spawn {
//!     kind: ObjectKind::CrystalPatch,
//!     owner: None,
//!     at: SpawnAt::Tile(TilePos::new(3, 0)),
//! });
//! test.run(5);
//! assert_eq!(test.sim().tick(), 5);
//! assert_eq!(test.sim().objects().count(), 2);
//! assert_eq!(test.replay().run().state_hash(), test.sim().state_hash());
//! ```

use std::collections::BTreeMap;

use glam::Vec2;

use crate::command::{Command, DebugCommand, PlayerCommand, TickInputs};
use crate::defs::ObjectKind;
use crate::events::SimEvent;
use crate::ids::{ObjectId, PlayerId, SlotId};
use crate::map::Map;
use crate::orders::OrderKind;
use crate::replay::Replay;
use crate::scenario::{Scenario, Spawn, SpawnAt, SpawnSettings};
use crate::scenarios;
use crate::sim::{Sim, SimConfig};
use crate::units::Tick;

/// Builds a test [`Scenario`] from an ASCII map ([`Map::from_ascii`]) plus spawns. It has the two slots of
/// [`scenarios::empty`] and its default setup: players 1 and 2, GDO, in slots 1 and 2, and seed 1.
pub struct ScenarioBuilder {
    scenario: Scenario,
}

impl ScenarioBuilder {
    /// Panics with the parse error if `ascii_map` is invalid.
    pub fn new(ascii_map: &str) -> Self {
        let map = Map::from_ascii(ascii_map).unwrap_or_else(|e| panic!("invalid test map: {e}"));
        let mut scenario = scenarios::empty(map.width(), map.height());
        scenario.map.name = "test".into();
        scenario.map.terrain = map;
        Self { scenario }
    }

    pub fn seed(mut self, seed: u64) -> Self {
        self.scenario.setup.seed = seed;
        self
    }

    /// Sets the game options (by default none: debug commands off, as in a real match).
    pub fn config(mut self, config: SimConfig) -> Self {
        self.scenario.setup.config = config;
        self
    }

    /// Adds a spawn with the default per-object values of its kind; spawns get ascending ids in the order they
    /// are added. An owner is a player of the default setup, which takes the slot with its number.
    pub fn spawn(mut self, kind: ObjectKind, owner: Option<PlayerId>, at: SpawnAt) -> Self {
        let slot = owner.map(|p| SlotId(p.0));
        self.scenario.map.spawns.push(Spawn::new(kind, slot, at));
        self
    }

    /// Adds a spawn with its own per-object values, which decide its kind.
    pub fn spawn_with(mut self, settings: SpawnSettings, at: SpawnAt) -> Self {
        let mut spawn = Spawn::new(settings.kind(), None, at);
        spawn.settings = Some(settings);
        self.scenario.map.spawns.push(spawn);
        self
    }

    pub fn build(self) -> Scenario {
        self.scenario
    }

    /// Builds the scenario and starts a [`TestSim`] on it, with debug commands on.
    pub fn start(self) -> TestSim {
        TestSim::new(self.build())
    }
}

/// A sim driven by a schedule of commands. It records a [`Replay`] of everything it runs
/// and keeps every event, since [`Sim::events`] holds only the latest step's. Debug commands
/// are on unless it starts with another [`SimConfig`] ([`TestSim::with_config`]).
pub struct TestSim {
    sim: Sim,
    replay: Replay,
    schedule: BTreeMap<Tick, Vec<PlayerCommand>>,
    events: Vec<(Tick, SimEvent)>,
}

impl TestSim {
    /// Panics if the scenario is invalid, listing its problems. Debug commands are on
    /// ([`SimConfig::DEBUG`]), whatever the scenario's setup says.
    pub fn new(scenario: Scenario) -> Self {
        Self::with_config(scenario, SimConfig::DEBUG)
    }

    /// [`TestSim::new`] with the game options `config` in the scenario's setup, in place of its own.
    pub fn with_config(mut scenario: Scenario, config: SimConfig) -> Self {
        scenario.setup.config = config;
        Self {
            sim: Sim::new(&scenario),
            replay: Replay::new(scenario),
            schedule: BTreeMap::new(),
            events: Vec::new(),
        }
    }

    /// Schedules `command` from `player` for the step that advances the sim to `tick`
    /// (commands apply at the start of a tick, so the first possible tick is 1). Commands
    /// for the same tick apply in the order they were scheduled. Panics if that step has
    /// already run.
    pub fn at(&mut self, tick: Tick, player: PlayerId, command: Command) -> &mut Self {
        assert!(
            tick > self.sim.tick(),
            "can't schedule a command for tick {tick}: the sim is already at tick {}",
            self.sim.tick()
        );
        self.schedule
            .entry(tick)
            .or_default()
            .push(PlayerCommand { player, command });
        self
    }

    /// [`TestSim::at`] for a debug command from player 1.
    pub fn debug_at(&mut self, tick: Tick, command: DebugCommand) -> &mut Self {
        self.at(tick, PlayerId(1), Command::Debug(command))
    }

    /// [`TestSim::debug_at`] for spawning a unit of `kind` for `owner`, centred on `pos`.
    pub fn spawn_unit_at(&mut self, tick: Tick, kind: ObjectKind, owner: PlayerId, pos: Vec2) -> &mut Self {
        self.debug_at(
            tick,
            DebugCommand::Spawn {
                kind,
                owner: Some(owner),
                at: SpawnAt::Point(pos),
            },
        )
    }

    /// [`TestSim::at`] for `player`'s order to `units` (`Command::Order`; `queue` is Shift).
    pub fn order_at(
        &mut self,
        tick: Tick,
        player: PlayerId,
        units: &[ObjectId],
        kind: OrderKind,
        queue: bool,
    ) -> &mut Self {
        let units = units.to_vec();
        self.at(tick, player, Command::Order { units, kind, queue })
    }

    /// Advances one tick, applying the commands scheduled for it.
    pub fn step(&mut self) -> &mut Self {
        let tick = self.sim.tick() + 1;
        let inputs = TickInputs {
            commands: self.schedule.remove(&tick).unwrap_or_default(),
        };
        self.sim.step(&inputs);
        self.replay.record(inputs);
        self.events.extend(self.sim.events().iter().map(|e| (tick, e.clone())));
        self
    }

    /// Advances `ticks` ticks.
    pub fn run(&mut self, ticks: u32) -> &mut Self {
        for _ in 0..ticks {
            self.step();
        }
        self
    }

    /// Advances until the sim reaches `tick`. Panics if it is already past it.
    pub fn run_until(&mut self, tick: Tick) -> &mut Self {
        assert!(
            tick >= self.sim.tick(),
            "can't run until tick {tick}: the sim is already at tick {}",
            self.sim.tick()
        );
        while self.sim.tick() < tick {
            self.step();
        }
        self
    }

    pub fn sim(&self) -> &Sim {
        &self.sim
    }

    /// Everything run so far, ready to play back.
    pub fn replay(&self) -> &Replay {
        &self.replay
    }

    /// Every event so far, with the tick whose step emitted it.
    pub fn events(&self) -> &[(Tick, SimEvent)] {
        &self.events
    }
}
