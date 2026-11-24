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
//!     orientation: Default::default(),
//! });
//! test.run(5);
//! assert_eq!(test.sim().tick(), 5);
//! assert_eq!(test.sim().objects().count(), 2);
//! assert_eq!(test.replay().run().state_hash(), test.sim().state_hash());
//! ```

pub mod invariants;

use std::collections::BTreeMap;

use glam::Vec2;

use crate::combat::AttackPhase;
use crate::command::{Ability, Command, DebugCommand, PlayerCommand, TickInputs};
use crate::defs::ObjectKind;
use crate::events::SimEvent;
use crate::ids::{ObjectId, PlayerId, SlotId};
use crate::map::Map;
use crate::mover::{MoveGoal, MoveStatus, Mover};
use crate::object::Object;
use crate::orders::{OrderKind, Target};
use crate::replay::Replay;
use crate::scenario::{Scenario, Spawn, SpawnAt, SpawnSettings};
use crate::scenarios;
use crate::sim::{Sim, SimConfig};
use crate::units::Tick;

use invariants::Invariants;

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
///
/// After every tick it checks the movement invariants I1–I7 (`docs/movement.md > 8`, [`invariants`]) and panics
/// on the first violation, with a picture of the map ([`TestSim::ascii`]). [`TestSim::check_invariants`] is the
/// switch; it is on from the start.
pub struct TestSim {
    sim: Sim,
    replay: Replay,
    schedule: BTreeMap<Tick, Vec<PlayerCommand>>,
    events: Vec<(Tick, SimEvent)>,
    /// `None` while the invariant checks are off.
    invariants: Option<Invariants>,
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
        let sim = Sim::new(&scenario);
        Self {
            invariants: Some(Invariants::new(&sim)),
            sim,
            replay: Replay::new(scenario),
            schedule: BTreeMap::new(),
            events: Vec::new(),
        }
    }

    /// The switch for the invariant checks after every tick (`docs/movement.md > 8`): on by default. Turn them
    /// off only where their cost matters (timing runs); turning them on again starts from the current state.
    pub fn check_invariants(&mut self, on: bool) -> &mut Self {
        self.invariants = on.then(|| Invariants::new(&self.sim));
        self
    }

    /// The map with its units as letters, and a legend ([`invariants::ascii`]): the whole map, or for a large one
    /// the window around `focus`.
    pub fn ascii(&self, focus: Option<Vec2>) -> String {
        invariants::ascii(&self.sim, focus)
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

    /// Turns auto-targeting off from tick 1 on (`DebugCommand::SetAutoTargeting`), for a test of movement between
    /// the units of different players: left to themselves they would fight (`systems/controls/behaviors.md >
    /// BaseAutoTargeting`). Call it before the first step.
    pub fn without_auto_targeting(&mut self) -> &mut Self {
        self.debug_at(1, DebugCommand::SetAutoTargeting { on: false })
    }

    /// [`TestSim::debug_at`] for spawning a unit of `kind` for `owner`, centred on `pos`.
    pub fn spawn_unit_at(&mut self, tick: Tick, kind: ObjectKind, owner: PlayerId, pos: Vec2) -> &mut Self {
        self.debug_at(
            tick,
            DebugCommand::Spawn {
                kind,
                owner: Some(owner),
                at: SpawnAt::Point(pos),
                orientation: Default::default(),
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

    /// [`TestSim::at`] for `player`'s `units` using `ability` (`Command::Ability`).
    pub fn ability_at(&mut self, tick: Tick, player: PlayerId, units: &[ObjectId], ability: Ability) -> &mut Self {
        let units = units.to_vec();
        self.at(tick, player, Command::Ability { units, ability })
    }

    /// Advances one tick, applying the commands scheduled for it.
    pub fn step(&mut self) -> &mut Self {
        let tick = self.sim.tick() + 1;
        let inputs = TickInputs {
            commands: self.schedule.remove(&tick).unwrap_or_default(),
        };
        self.sim.step(&inputs);
        if let Some(invariants) = &mut self.invariants
            && let Err(violation) = invariants.check(&self.sim, &inputs)
        {
            panic!(
                "invariant {} broken in tick {tick}: {}\n{}",
                violation.invariant,
                violation.message,
                invariants::ascii(&self.sim, Some(violation.at))
            );
        }
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

    /// Where object `id` is. Panics if it doesn't exist.
    pub fn pos(&self, id: u32) -> Vec2 {
        self.object(id).pos
    }

    /// Unit `id`'s movement state. Panics if `id` isn't a unit.
    pub fn mover(&self, id: u32) -> &Mover {
        self.object(id)
            .mover
            .as_ref()
            .unwrap_or_else(|| panic!("#{id} is not a unit"))
    }

    /// How unit `id`'s move is going.
    pub fn status(&self, id: u32) -> MoveStatus {
        self.mover(id).status
    }

    fn object(&self, id: u32) -> &Object {
        self.sim
            .object(ObjectId(id))
            .unwrap_or_else(|| panic!("there is no object #{id}"))
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

/// The objects `o` refers to, in no particular order: those its orders (current and queued) name, those its
/// behaviours engaged by themselves, its move's goal object, and its attack's target and the target of its attack
/// phase.
pub fn referenced_ids(o: &Object) -> Vec<ObjectId> {
    let mut ids = Vec::new();
    if let Some(orders) = &o.orders {
        let kinds = orders
            .current
            .iter()
            .map(|active| active.order.kind)
            .chain(orders.queue.iter().map(|order| order.kind));
        for kind in kinds {
            if let OrderKind::Attack(id) | OrderKind::Move(Target::Object(id)) = kind {
                ids.push(id);
            }
        }
        ids.extend(orders.behaviour_targets());
    }
    if let Some(MoveGoal::Object { id, .. }) = o.mover.as_ref().map(|m| m.goal) {
        ids.push(id);
    }
    if let Some(attack) = &o.attack {
        ids.extend(attack.target);
        if let Some(AttackPhase::Aiming { target } | AttackPhase::Firing { target: Some(target) }) = attack.phase {
            ids.push(target);
        }
    }
    ids
}

/// Every reference to an object that no longer exists, as `(holder, missing)` pairs ([`referenced_ids`]). The
/// removal path (`Sim::remove_objects`) leaves none behind.
pub fn dangling_references(sim: &Sim) -> Vec<(ObjectId, ObjectId)> {
    sim.objects()
        .flat_map(|o| referenced_ids(o).into_iter().map(move |id| (o.id, id)))
        .filter(|&(_, id)| sim.object(id).is_none())
        .collect()
}

/// One command of every kind: every variant of [`Command`], [`OrderKind`], [`Target`], [`Ability`], [`DebugCommand`]
/// and [`SpawnAt`], queued and not, for tests that a replay file keeps everything a player did
/// (`ARCHITECTURE.md > Replays and golden hashes`). The ids needn't exist. The match below stops compiling when a
/// variant is added, as a reminder to add it here.
pub fn every_command() -> Vec<Command> {
    let units = vec![ObjectId(1), ObjectId(2)];
    let point = Vec2::new(48.5, 80.25);
    let kinds = [
        OrderKind::Move(Target::Point(point)),
        OrderKind::Move(Target::Object(ObjectId(3))),
        OrderKind::Attack(ObjectId(3)),
        OrderKind::AttackMove(point),
        OrderKind::Patrol(point),
        OrderKind::Hold,
        OrderKind::Stop,
    ];
    let mut commands: Vec<Command> = kinds
        .into_iter()
        .flat_map(|kind| {
            [false, true].map(|queue| Command::Order {
                units: units.clone(),
                kind,
                queue,
            })
        })
        .collect();
    commands.extend([true, false].map(|raise| Command::Ability {
        units: units.clone(),
        ability: Ability::RiotShield { raise },
    }));
    commands.extend(
        [
            DebugCommand::Spawn {
                kind: ObjectKind::Peacekeeper,
                owner: Some(PlayerId(1)),
                at: SpawnAt::Point(point),
                orientation: Default::default(),
            },
            DebugCommand::Spawn {
                kind: ObjectKind::SupplyDeliveryStation,
                owner: None,
                at: SpawnAt::Tile(crate::units::TilePos::new(3, 4)),
                orientation: Default::default(),
            },
            DebugCommand::Spawn {
                kind: ObjectKind::Barracks,
                owner: Some(PlayerId(2)),
                at: SpawnAt::Tile(crate::units::TilePos::new(6, 1)),
                orientation: crate::scenario::BuildingOrientation {
                    rotation: crate::object::Rotation::R90,
                    flip_h: true,
                    flip_v: false,
                },
            },
            DebugCommand::Remove { id: ObjectId(2) },
            DebugCommand::SetPosition {
                id: ObjectId(1),
                pos: point,
            },
            DebugCommand::SetAutoTargeting { on: false },
        ]
        .map(Command::Debug),
    );
    for command in &commands {
        match command {
            Command::Order { kind, .. } => match kind {
                OrderKind::Move(Target::Point(_) | Target::Object(_))
                | OrderKind::Attack(_)
                | OrderKind::AttackMove(_)
                | OrderKind::Patrol(_)
                | OrderKind::Hold
                | OrderKind::Stop => {}
            },
            Command::Ability {
                ability: Ability::RiotShield { .. },
                ..
            } => {}
            Command::Debug(
                DebugCommand::Spawn {
                    at: SpawnAt::Point(_) | SpawnAt::Tile(_),
                    ..
                }
                | DebugCommand::Remove { .. }
                | DebugCommand::SetPosition { .. }
                | DebugCommand::SetAutoTargeting { .. },
            ) => {}
        }
    }
    commands
}
