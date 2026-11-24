//! [`ScenarioBuilder`]: test scenarios from an ASCII map.

use crate::defs::ObjectKind;
use crate::ids::{PlayerId, SlotId};
use crate::map::Map;
use crate::player::Faction;
use crate::scenario::{Scenario, Spawn, SpawnAt, SpawnSettings};
use crate::scenarios;
use crate::sim::SimConfig;

use super::TestSim;

/// Builds a test [`Scenario`] from an ASCII map ([`Map::from_ascii`]) plus spawns. It has the two slots of
/// [`scenarios::empty`] and its default setup: players 1 and 2, GDO unless [`ScenarioBuilder::faction`] says
/// otherwise, in slots 1 and 2, and seed 1. The map is a Custom one, so no Starting Layout is required of it.
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

    /// Makes `player` (1 or 2) play `faction` (plan decision T27): their banks start at its Starting Conditions, and
    /// their army cap is its own (`crate::economy::army`). Panics for another player.
    pub fn faction(mut self, player: PlayerId, faction: Faction) -> Self {
        let setup = player
            .index()
            .and_then(|i| self.scenario.setup.players.get_mut(i))
            .unwrap_or_else(|| panic!("the test scenarios have players 1 and 2, not {}", player.0));
        setup.faction = faction;
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
