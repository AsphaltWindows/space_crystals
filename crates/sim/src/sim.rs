//! The simulation: state plus the fixed-order tick pipeline.

use std::collections::BTreeMap;

use glam::Vec2;
use serde::{Deserialize, Serialize, Serializer};

use crate::command::{PlayerCommand, TickInputs};
use crate::defs::ObjectKind;
use crate::events::SimEvent;
use crate::hash::{StateHash, StateHasher};
use crate::ids::{ObjectId, PlayerId};
use crate::map::Map;
use crate::mover::Mover;
use crate::nav::{Nav, NavCounters};
use crate::object::{Object, Placement, ResourceState};
use crate::orders::{Order, OrderGroup, OrderKind, Orders, Target, slots};
use crate::paths::{self, NavState};
use crate::player::Player;
use crate::rng::Rng;
use crate::scenario::{Scenario, ScenarioProblem, SpawnSettings};
use crate::units::Tick;
use crate::{combat, movement, orders, vision};

use gate::Checked;
pub use gate::Reject;

/// Game options, chosen with the rest of the game setup ([`crate::GameSetup::config`]) and fixed when the game
/// starts: stored and hashed in the `Sim`, so a [`crate::Replay`] (map, setup and inputs) brings them back.
#[derive(Copy, Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct SimConfig {
    /// `Command::Debug` is accepted (development and QA, `ARCHITECTURE.md > Commands, events and replays`). Off
    /// by default: a real match never offers debug commands.
    pub debug: bool,
}

impl SimConfig {
    /// Debug commands on: development, QA and tests.
    pub const DEBUG: SimConfig = SimConfig { debug: true };
}

/// The whole game state. Serde (de)serializes all of it, e.g. as a JSON state dump or a
/// save; loading checks that object ids are unique and below the next id.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(try_from = "SimState")]
pub struct Sim {
    tick: Tick,
    map: Map,
    players: Vec<Player>,
    /// Keyed by id ⇒ iteration is always in ascending `ObjectId` order. Serialized as a
    /// list in that order.
    #[serde(serialize_with = "serialize_objects")]
    objects: BTreeMap<ObjectId, Object>,
    next_id: u32,
    /// The only source of randomness (`ARCHITECTURE.md > Determinism`), seeded from
    /// `GameSetup::seed`.
    rng: Rng,
    config: SimConfig,
    /// The next order command's group (`docs/movement.md > 3`, `OrderGroup`).
    next_group: u32,
    /// Bumped by every rebuild of the navigation grids (phase 5b): paths planned on an older version are stale.
    nav_version: u32,
    /// Output of the latest step only; not part of [`Sim::state_hash`].
    events: Vec<SimEvent>,
    /// Derived from the map and the objects (`docs/movement.md > 3`): not hashed, serialized or compared, and
    /// rebuilt on load.
    #[serde(skip_serializing)]
    nav: NavState,
}

impl Sim {
    /// Panics if the scenario is invalid, listing its problems. Use [`Sim::try_new`] for
    /// scenarios from files or other input.
    pub fn new(scenario: &Scenario) -> Self {
        Self::try_new(scenario).unwrap_or_else(|problems| {
            let problems: Vec<String> = problems.iter().map(ToString::to_string).collect();
            panic!("invalid scenario {:?}: {}", scenario.name(), problems.join("; "))
        })
    }

    /// Starts a game of the scenario's map with its game setup (players, seed and game options), or lists everything
    /// wrong with either ([`Scenario::validate`]). The map's pre-placed objects come first, those of slots that no
    /// player takes left out, then each player's Starting Layout for their faction at their slot; objects a slot
    /// owns belong to the player who takes it. Never panics.
    pub fn try_new(scenario: &Scenario) -> Result<Self, Vec<ScenarioProblem>> {
        let objects = scenario.start_objects()?;
        let mut sim = Self::empty(scenario);
        for o in objects {
            sim.insert_object(o.kind, o.owner, o.pos, o.placement, o.settings.as_ref());
        }
        sim.nav = NavState::build(&sim.map, &sim.objects);
        // Scenario spawns emit no events (`ARCHITECTURE.md > Commands, events and replays`).
        sim.events.clear();
        Ok(sim)
    }

    /// The scenario's terrain and players at tick 0, without objects. Players beyond the last `PlayerId` are left
    /// out (only an invalid setup has them). Its grids are built once the objects are in
    /// (`NavState::build`).
    fn empty(scenario: &Scenario) -> Self {
        let players = scenario
            .setup
            .player_ids()
            .zip(&scenario.setup.players)
            .map(|(id, p)| Player {
                id,
                name: p.name.clone(),
                faction: p.faction,
            })
            .collect();
        Self {
            tick: 0,
            map: scenario.map.terrain.clone(),
            players,
            objects: BTreeMap::new(),
            next_id: 1,
            rng: Rng::from_seed(scenario.setup.seed),
            config: scenario.setup.config,
            next_group: 0,
            nav_version: 0,
            events: Vec::new(),
            nav: NavState::unbuilt(&scenario.map.terrain),
        }
    }

    pub fn tick(&self) -> Tick {
        self.tick
    }

    /// The game options this sim started with.
    pub fn config(&self) -> SimConfig {
        self.config
    }

    pub fn map(&self) -> &Map {
        &self.map
    }

    pub fn players(&self) -> &[Player] {
        &self.players
    }

    pub fn player(&self, id: PlayerId) -> Option<&Player> {
        self.players.iter().find(|p| p.id == id)
    }

    /// All objects, in ascending `ObjectId` order.
    pub fn objects(&self) -> impl Iterator<Item = &Object> + '_ {
        self.objects.values()
    }

    pub fn object(&self, id: ObjectId) -> Option<&Object> {
        self.objects.get(&id)
    }

    /// Events emitted during the most recent [`Sim::step`].
    pub fn events(&self) -> &[SimEvent] {
        &self.events
    }

    /// The navigation grids units path on (`docs/movement.md > 5.2`): derived from the map and the blocking
    /// footprints, and in step with them between ticks.
    pub fn nav(&self) -> &Nav {
        self.nav.nav()
    }

    /// How often the grids were rebuilt since the game started (phase 5b).
    pub fn nav_version(&self) -> u32 {
        self.nav_version
    }

    /// The path searches of the most recent [`Sim::step`]: output for tests and overlays, not state.
    pub fn nav_counters(&self) -> NavCounters {
        self.nav.counters()
    }

    /// FNV-1a 64 hash of the whole state: tick, map, players, objects in id order, next id,
    /// PRNG state, config, next order group and nav version (`f32` by its bits; encoding in
    /// `hash.rs`). Events are output and the grids derived data, so they are left out. Equal
    /// states give equal hashes on every machine, so comparing hashes checks determinism.
    pub fn state_hash(&self) -> u64 {
        // Destructured without `..`: a new field fails to compile here until it is hashed.
        let Sim {
            tick,
            map,
            players,
            objects,
            next_id,
            rng,
            config,
            next_group,
            nav_version,
            events: _,
            nav: _,
        } = self;
        let mut h = StateHasher::new();
        tick.hash(&mut h);
        map.hash(&mut h);
        players.hash(&mut h);
        h.len(objects.len());
        for o in objects.values() {
            o.hash(&mut h);
        }
        next_id.hash(&mut h);
        rng.hash(&mut h);
        config.hash(&mut h);
        next_group.hash(&mut h);
        nav_version.hash(&mut h);
        h.finish()
    }

    /// Advance exactly one tick. Phase order is fixed and documented in ARCHITECTURE.md.
    pub fn step(&mut self, inputs: &TickInputs) {
        self.events.clear();
        self.tick += 1;
        for o in self.objects.values_mut() {
            o.prev_pos = o.pos;
            o.prev_facing = o.facing;
        }
        // `ARCHITECTURE.md > Tick pipeline`: objects that commands create during this tick are first updated in
        // the next one. Ids only grow, so they are exactly the ids from here on.
        let first_new = ObjectId(self.next_id);
        for cmd in &inputs.commands {
            self.apply_command(cmd);
        }
        // Phases 5a–5f, `docs/movement.md > 4`.
        // 5a: after the commands, so an order acts in its own tick; before movement, so goals have one writer.
        orders::update(&mut self.objects, first_new, self.tick);
        // 5b: the paths below must see this tick's obstacles.
        paths::sync(&mut self.nav, &self.map, &self.objects, &mut self.nav_version);
        // 5c: after the goals, before the movement that follows the paths.
        paths::serve(
            &mut self.nav,
            &mut self.objects,
            first_new,
            self.nav_version,
            self.tick,
            &mut self.events,
        );
        // 5d: the one movement pipeline, the single place that moves units.
        movement::step(&mut self.objects, first_new);
        // 5e: shoots from the positions movement committed.
        combat::step(&mut self.objects, first_new);
        // 5f: sees the tick's final positions.
        vision::update(&self.objects);
        self.update_supply_stations(first_new);
        debug_assert!(
            self.nav.matches(&self.objects),
            "a blocking footprint changed after phase 5b: sync the grids again (`paths::sync`)"
        );
    }

    /// `ARCHITECTURE.md > Commands, events and replays`: every command passes the gate ([`Sim::validate`]) first.
    /// A rejected one changes nothing; player input never panics the sim.
    fn apply_command(&mut self, cmd: &PlayerCommand) {
        let Ok(checked) = self.check(cmd) else {
            return;
        };
        match checked {
            Checked::Order { units, kind, queue } => self.give_order(&units, kind, queue),
            Checked::Spawn {
                kind,
                owner,
                pos,
                placement,
            } => {
                self.insert_object(kind, owner, pos, placement, None);
            }
            Checked::Remove(id) => {
                if self.objects.remove(&id).is_some() {
                    self.events.push(SimEvent::Removed(id));
                }
            }
            Checked::Teleport { id, pos } => {
                if let Some(o) = self.objects.get_mut(&id) {
                    // No interpolation from the old position.
                    o.pos = pos;
                    o.prev_pos = pos;
                    // A path from the old position leads nowhere now.
                    if let Some(mover) = &mut o.mover {
                        mover.repath(self.tick);
                    }
                }
            }
        }
    }

    /// `systems/controls/commands.md > Unit Command`: one order for each of `units` (checked: unique existing
    /// units), sharing a new order group. A group's move to a point gives each unit its own slot
    /// (`docs/movement.md > 6.4`). Orders only fill queues here; phase 5a starts them.
    fn give_order(&mut self, units: &[ObjectId], kind: OrderKind, queue: bool) {
        let group = OrderGroup(self.next_group);
        self.next_group = self.next_group.wrapping_add(1);
        let slots = match kind {
            OrderKind::Move(Target::Point(click)) => slots::assign(self.nav.nav(), &self.objects, units, click),
            _ => vec![None; units.len()],
        };
        for (id, slot) in units.iter().zip(slots) {
            if let Some(orders) = self.objects.get_mut(id).and_then(|o| o.orders.as_mut()) {
                // A full queue drops the order for this unit only.
                orders.give(Order { kind, group, slot }, queue);
            }
        }
    }

    /// Insert an object at a checked spot, with its type's initial state. Its per-object values are `settings`,
    /// or the kind's defaults if those are missing or belong to another kind ([`SpawnSettings::resolve`]).
    fn insert_object(
        &mut self,
        kind: ObjectKind,
        owner: Option<PlayerId>,
        pos: Vec2,
        placement: Option<Placement>,
        settings: Option<&SpawnSettings>,
    ) -> ObjectId {
        let id = ObjectId(self.next_id);
        self.next_id += 1;
        let def = kind.def();
        let resource = SpawnSettings::resolve(kind, settings).map(|s| s.starting_resource());
        // Units take orders and move (`systems/units.md > Unit`); they start idle.
        let is_unit = def.unit.is_some();
        let object = Object {
            id,
            kind,
            owner,
            pos,
            prev_pos: pos,
            facing: 0.0,
            prev_facing: 0.0,
            hp: def.max_hp,
            placement,
            resource,
            orders: is_unit.then(Orders::default),
            mover: is_unit.then(Mover::default),
        };
        self.objects.insert(id, object);
        self.events.push(SimEvent::Spawned(id));
        id
    }

    /// `core/map.md > SupplyDeliveryStation`: the countdown runs only while empty. Stations from `first_new` on
    /// were created this tick and wait for the next one.
    fn update_supply_stations(&mut self, first_new: ObjectId) {
        for o in self.objects.range_mut(..first_new).map(|(_, o)| o) {
            let Some(ResourceState::SupplyStation {
                delivery_size,
                delivery_interval,
                current_supplies,
                next_delivery_in,
            }) = &mut o.resource
            else {
                continue;
            };
            if *current_supplies > 0 {
                *next_delivery_in = None;
                continue;
            }
            let remaining = next_delivery_in.get_or_insert(*delivery_interval);
            *remaining = remaining.saturating_sub(1);
            if *remaining == 0 {
                *current_supplies = *delivery_size;
                *next_delivery_in = None;
                self.events.push(SimEvent::SuppliesDelivered {
                    station: o.id,
                    amount: *delivery_size,
                });
            }
        }
    }

    /// Convenience for tests and the client: the object nearest to `p` matching `filter`.
    pub fn nearest_object(&self, p: Vec2, filter: impl Fn(&Object) -> bool) -> Option<&Object> {
        self.objects
            .values()
            .filter(|o| filter(o))
            .min_by(|a, b| a.pos.distance_squared(p).total_cmp(&b.pos.distance_squared(p)))
    }
}

fn serialize_objects<S: Serializer>(objects: &BTreeMap<ObjectId, Object>, serializer: S) -> Result<S::Ok, S::Error> {
    serializer.collect_seq(objects.values())
}

/// The serialized form of [`Sim`], checked before it becomes one.
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct SimState {
    tick: Tick,
    map: Map,
    players: Vec<Player>,
    objects: Vec<Object>,
    next_id: u32,
    rng: Rng,
    config: SimConfig,
    next_group: u32,
    nav_version: u32,
    events: Vec<SimEvent>,
}

impl TryFrom<SimState> for Sim {
    type Error = String;

    fn try_from(state: SimState) -> Result<Self, String> {
        let SimState {
            tick,
            map,
            players,
            objects: list,
            next_id,
            rng,
            config,
            next_group,
            nav_version,
            events,
        } = state;
        let mut objects = BTreeMap::new();
        for o in list {
            if o.id.0 >= next_id {
                return Err(format!("object id {} is not below next_id {next_id}", o.id.0));
            }
            if let Some(duplicate) = objects.insert(o.id, o) {
                return Err(format!("object id {} appears more than once", duplicate.id.0));
            }
        }
        // Between ticks the grids match the objects (`paths`), so rebuilding them gives the running game's.
        let nav = NavState::build(&map, &objects);
        Ok(Sim {
            tick,
            map,
            players,
            objects,
            next_id,
            rng,
            config,
            next_group,
            nav_version,
            events,
            nav,
        })
    }
}

mod gate;
mod preview;
#[cfg(test)]
mod tests;
