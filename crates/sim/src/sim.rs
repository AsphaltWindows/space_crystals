//! The simulation: state plus the fixed-order tick pipeline.

use std::collections::BTreeMap;

use glam::Vec2;
use serde::{Deserialize, Serialize, Serializer};

use crate::command::{Command, DebugCommand, PlayerCommand, TickInputs};
use crate::defs::ObjectKind;
use crate::events::SimEvent;
use crate::hash::{StateHash, StateHasher};
use crate::ids::{ObjectId, PlayerId};
use crate::map::Map;
use crate::object::{Object, Placement, ResourceState};
use crate::player::Player;
use crate::rng::Rng;
use crate::scenario::{Scenario, ScenarioProblem, SpawnAt, place_spawn};
use crate::units::{Tick, seconds};

/// Placeholder until the design gives one (question Q0003).
pub const DEFAULT_PATCH_AMOUNT: u32 = 1500;
/// Placeholders until the design gives them (question Q0003).
pub const DEFAULT_SDS_DELIVERY_SIZE: u32 = 4;
pub const DEFAULT_SDS_DELIVERY_INTERVAL_SECONDS: f32 = 60.0;

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
    /// `Scenario::seed`.
    rng: Rng,
    /// Output of the latest step only; not part of [`Sim::state_hash`].
    events: Vec<SimEvent>,
}

impl Sim {
    /// Panics if the scenario is invalid, listing its problems. Use [`Sim::try_new`] for
    /// scenarios from files or other input.
    pub fn new(scenario: &Scenario) -> Self {
        Self::try_new(scenario).unwrap_or_else(|problems| {
            let problems: Vec<String> = problems.iter().map(ToString::to_string).collect();
            panic!("invalid scenario {:?}: {}", scenario.name, problems.join("; "))
        })
    }

    /// Starts a game, or lists everything wrong with the scenario
    /// ([`Scenario::validate`]). Never panics.
    pub fn try_new(scenario: &Scenario) -> Result<Self, Vec<ScenarioProblem>> {
        let spots = scenario.spawn_spots()?;
        let players = scenario
            .players
            .iter()
            .map(|p| Player {
                id: p.id,
                name: p.name.clone(),
                faction: p.faction,
            })
            .collect();
        let mut sim = Self {
            tick: 0,
            map: scenario.map.clone(),
            players,
            objects: BTreeMap::new(),
            next_id: 1,
            rng: Rng::from_seed(scenario.seed),
            events: Vec::new(),
        };
        for (s, (pos, placement)) in scenario.spawns.iter().zip(spots) {
            sim.insert_object(s.kind, s.owner, pos, placement);
        }
        // Scenario spawns emit no events (`ARCHITECTURE.md > Commands, events and replays`).
        sim.events.clear();
        Ok(sim)
    }

    pub fn tick(&self) -> Tick {
        self.tick
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

    /// FNV-1a 64 hash of the whole state: tick, map, players, objects in id order, next id
    /// and PRNG state (`f32` by its bits; encoding in `hash.rs`). Events are output, not
    /// state, so they are left out. Equal states give equal hashes on every machine, so
    /// comparing hashes checks determinism.
    pub fn state_hash(&self) -> u64 {
        // Destructured without `..`: a new field fails to compile here until it is hashed.
        let Sim {
            tick,
            map,
            players,
            objects,
            next_id,
            rng,
            events: _,
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
        for cmd in &inputs.commands {
            self.apply_command(cmd);
        }
        self.update_supply_stations();
    }

    /// Player input must never panic the sim: invalid commands are ignored.
    fn apply_command(&mut self, cmd: &PlayerCommand) {
        match &cmd.command {
            Command::Debug(DebugCommand::Spawn { kind, owner, at }) => {
                self.spawn(*kind, *owner, *at);
            }
            Command::Debug(DebugCommand::Remove { id }) => {
                if self.objects.remove(id).is_some() {
                    self.events.push(SimEvent::Removed(*id));
                }
            }
            Command::Debug(DebugCommand::SetPosition { id, pos }) => {
                // `contains_point` also rejects NaN and infinite positions.
                let on_map = self.map.contains_point(*pos);
                if let Some(o) = self.objects.get_mut(id)
                    && o.placement.is_none()
                    && on_map
                {
                    // A teleport: no interpolation from the old position.
                    o.pos = *pos;
                    o.prev_pos = *pos;
                }
            }
        }
    }

    /// Create an object with its type's initial state. Returns `None`, changing
    /// nothing, if the location doesn't suit the type, lies off the map, the owner is
    /// invalid ([`place_spawn`]), or every object id is used up.
    fn spawn(&mut self, kind: ObjectKind, owner: Option<PlayerId>, at: SpawnAt) -> Option<ObjectId> {
        if self.next_id == u32::MAX {
            return None;
        }
        let (pos, placement) = place_spawn(&self.map, kind, owner, at, |p| self.player(p).is_some()).ok()?;
        Some(self.insert_object(kind, owner, pos, placement))
    }

    /// Insert an object at a checked spot, with its type's initial state.
    fn insert_object(
        &mut self,
        kind: ObjectKind,
        owner: Option<PlayerId>,
        pos: Vec2,
        placement: Option<Placement>,
    ) -> ObjectId {
        let id = ObjectId(self.next_id);
        self.next_id += 1;
        let resource = match kind {
            ObjectKind::CrystalPatch => Some(ResourceState::CrystalPatch {
                remaining: DEFAULT_PATCH_AMOUNT,
            }),
            ObjectKind::SupplyDeliveryStation => Some(ResourceState::SupplyStation {
                delivery_size: DEFAULT_SDS_DELIVERY_SIZE,
                delivery_interval: seconds(DEFAULT_SDS_DELIVERY_INTERVAL_SECONDS),
                current_supplies: 0,
                next_delivery_in: Some(seconds(DEFAULT_SDS_DELIVERY_INTERVAL_SECONDS)),
            }),
        };
        let object = Object {
            id,
            kind,
            owner,
            pos,
            prev_pos: pos,
            facing: 0.0,
            prev_facing: 0.0,
            hp: kind.def().max_hp,
            placement,
            resource,
        };
        self.objects.insert(id, object);
        self.events.push(SimEvent::Spawned(id));
        id
    }

    /// `core/map.md > SupplyDeliveryStation`: the countdown runs only while empty.
    fn update_supply_stations(&mut self) {
        for o in self.objects.values_mut() {
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
        Ok(Sim {
            tick,
            map,
            players,
            objects,
            next_id,
            rng,
            events,
        })
    }
}

#[cfg(test)]
mod tests;
