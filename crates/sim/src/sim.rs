//! The simulation: state plus the fixed-order tick pipeline.

use std::collections::BTreeMap;

use glam::Vec2;

use crate::command::{Command, DebugCommand, PlayerCommand, TickInputs};
use crate::defs::{Category, Footprint, ObjectKind};
use crate::events::SimEvent;
use crate::ids::{ObjectId, PlayerId};
use crate::map::Map;
use crate::object::{Object, Placement, ResourceState};
use crate::player::Player;
use crate::scenario::{Scenario, SpawnAt};
use crate::units::{Tick, seconds};

/// Placeholder until the design gives one (question Q0003).
pub const DEFAULT_PATCH_AMOUNT: u32 = 1500;
/// Placeholders until the design gives them (question Q0003).
pub const DEFAULT_SDS_DELIVERY_SIZE: u32 = 4;
pub const DEFAULT_SDS_DELIVERY_INTERVAL_SECONDS: f32 = 60.0;

pub struct Sim {
    tick: Tick,
    map: Map,
    players: Vec<Player>,
    /// Keyed by id ⇒ iteration is always in ascending `ObjectId` order.
    objects: BTreeMap<ObjectId, Object>,
    next_id: u32,
    events: Vec<SimEvent>,
}

impl Sim {
    pub fn new(scenario: &Scenario) -> Self {
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
            events: Vec::new(),
        };
        for s in &scenario.spawns {
            if sim.spawn(s.kind, s.owner, s.at).is_none() {
                panic!("scenario {:?}: invalid spawn {s:?}", scenario.name);
            }
        }
        sim.events.clear();
        sim
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
    /// nothing, if the location doesn't suit the type, lies off the map, or the
    /// owner is invalid.
    fn spawn(&mut self, kind: ObjectKind, owner: Option<PlayerId>, at: SpawnAt) -> Option<ObjectId> {
        let def = kind.def();
        let owner_ok = match owner {
            // `core/map.md > Resource`: resources are never owned.
            Some(p) => def.category != Category::Resource && self.player(p).is_some(),
            None => true,
        };
        if !owner_ok {
            return None;
        }
        let (pos, placement) = match (def.footprint, at) {
            (Footprint::Tiles { .. }, SpawnAt::Tile(origin)) => {
                let p = Placement::new(origin, def.footprint);
                if !p.tiles().all(|t| self.map.in_bounds(t)) {
                    return None;
                }
                (p.center(), Some(p))
            }
            (Footprint::Silhouette { .. }, SpawnAt::Point(pos)) if self.map.contains_point(pos) => (pos, None),
            _ => return None,
        };
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
            hp: def.max_hp,
            placement,
            resource,
        };
        self.objects.insert(id, object);
        self.events.push(SimEvent::Spawned(id));
        Some(id)
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
