//! The simulation: its state, and the fixed-order tick pipeline that advances it ([`Sim::step`], `sim/phase.rs`).

use std::collections::BTreeMap;

use glam::Vec2;
use serde::{Deserialize, Serialize};

use crate::augmentation::Research;
use crate::combat::Projectile;
use crate::defs::ObjectKind;
use crate::economy;
use crate::economy::Outcome;
use crate::economy::defeat;
use crate::events::SimEvent;
use crate::hash::{StateHash, StateHasher};
use crate::ids::{ObjectId, PlayerId};
use crate::map::Map;
use crate::network::Held;
use crate::object::{Object, Placement, ResourceState};
use crate::paths::NavState;
use crate::paths::knowledge::PathKnowledge;
use crate::player::Player;
use crate::rng::Rng;
use crate::ruleset::{self, Generation};
use crate::scenario::{Scenario, ScenarioProblem, SpawnSettings};
use crate::shroud::Clouds;
use crate::units::{Tick, TilePos};
use crate::vision::{VisibilityState, Vision};

pub use gate::Reject;
use load::{SimState, serialize_held, serialize_next_rules, serialize_objects, serialize_rules};
pub use phase::Phase;

/// Game options, chosen with the rest of the game setup ([`crate::GameSetup::config`]) and fixed when the game
/// starts: stored and hashed in the `Sim`, so a [`crate::Replay`] (map, setup and inputs) brings them back.
#[derive(Copy, Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct SimConfig {
    /// `Command::Debug` is accepted (development and QA, `ARCHITECTURE.md > Commands, events and replays`). Off
    /// by default: a real match never offers debug commands.
    pub debug: bool,
    /// (Slice M5-S7, design change 0416) The map starts explored (`core/vision.md > Starting exploration`): every
    /// tile starts Explored for every player, the fog of war still on, and each remembers the terrain and the neutral
    /// objects as at the start (their records, `Vision::seed`), none of the players' objects. Off here, and in files
    /// that don't have it (the replays recorded before it existed, which then replay as recorded); the client turns
    /// it on for the games it launches, as the design's default is on (`--unexplored` turns it off). Left out of files
    /// and of the hash while off.
    #[serde(default, skip_serializing_if = "is_false")]
    pub start_explored: bool,
}

impl SimConfig {
    /// Debug commands on: development, QA and tests.
    pub const DEBUG: SimConfig = SimConfig {
        debug: true,
        start_explored: false,
    };
}

fn is_false(value: &bool) -> bool {
    !*value
}

/// The whole game state. Serde (de)serializes all of it, e.g. as a JSON state dump or a
/// save; loading checks that object ids are unique and below the next id.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(try_from = "SimState")]
pub struct Sim {
    tick: Tick,
    map: Map,
    players: Vec<Player>,
    /// How the game ended, once it has (`core/game_rules.md > End of Game`, `economy::defeat`): hashed and
    /// serialized. From then on the gate refuses every command.
    outcome: Option<Outcome>,
    /// Keyed by id ⇒ iteration is always in ascending `ObjectId` order. Serialized as a
    /// list in that order.
    #[serde(serialize_with = "serialize_objects")]
    objects: BTreeMap<ObjectId, Object>,
    /// The units off the map (plan decision T30, `crate::network`): those inside a Tunnel Network, each with its id,
    /// which no object has. Not among `objects`, so nothing that walks the map sees, moves or hits them. Hashed and
    /// serialized, as a list in id order; left out of the hash and of files while empty, so that the states without a
    /// held unit are as they were before the store existed.
    #[serde(serialize_with = "serialize_held", skip_serializing_if = "BTreeMap::is_empty")]
    held: BTreeMap<ObjectId, Held>,
    /// (M4-O) The projectiles in flight, in launch order (plan decision T45, `combat::projectiles`): not objects.
    /// Hashed and serialized; left out of the hash and of files while empty, so that the states without one are as
    /// they were before the store existed.
    #[serde(skip_serializing_if = "Vec::is_empty")]
    projectiles: Vec<Projectile>,
    /// (M5-N4b) Shroud's clouds and their buffers (`crate::shroud`, plan decision T77): history, hashed and serialized;
    /// left out of the hash and of files while there is no cloud and no buffer waiting for its release, so that the
    /// states without one are as they were before Shroud.
    #[serde(skip_serializing_if = "Clouds::is_empty")]
    clouds: Clouds,
    next_id: u32,
    /// The only source of randomness (`ARCHITECTURE.md > Determinism`), seeded from
    /// `GameSetup::seed`.
    rng: Rng,
    config: SimConfig,
    /// The next order command's group (`docs/movement.md > 3`, `OrderGroup`).
    next_group: u32,
    /// Bumped by every rebuild of the navigation grids (phase 5b): paths planned on an older version are stale.
    nav_version: u32,
    /// Whether units acquire targets by themselves (`systems/controls/behaviors.md > BaseAutoTargeting`): on,
    /// unless a debug command turned it off (`DebugCommand::SetAutoTargeting`).
    auto_targeting: bool,
    /// Output of the latest step only; not part of [`Sim::state_hash`].
    events: Vec<SimEvent>,
    /// Each player's vision as of the end of the latest tick (`vision`): the Explored tiles are history, hashed and
    /// serialized; the Visible ones follow from the objects and are rebuilt on load.
    vision: Vision,
    /// What each player knows to stand in the way of its units' paths, beyond its vision's records
    /// (`systems/controls/behaviors.md > PathKnowledge`): history, hashed and serialized.
    path_knowledge: PathKnowledge,
    /// Derived from the map, the objects and what each player knows (`docs/movement.md > 3`, `paths`): not hashed,
    /// serialized or compared, and rebuilt on load.
    #[serde(skip_serializing)]
    nav: NavState,
    /// (M5-X1b) The rules the game plays by (plan decision T66, `sim/rules.rs`): its generation, made the current one
    /// as each tick starts. Not part of [`Sim::state_hash`], so that a golden fails only when behaviour does.
    /// Serialized as the rules themselves while they aren't the built-in ones, and left out of files while they are,
    /// so that the states of games on the built-in rules are as they were before rules could change.
    #[serde(serialize_with = "serialize_rules", skip_serializing_if = "Generation::is_builtin")]
    rules: Generation,
    /// (M5-X1b) The rules of the last tick's `DebugCommand::SetRules`, which the next tick starts on (`sim/rules.rs`).
    /// Neither hashed, as [`Sim::rules`], nor serialized while there are none; serialized as the rules themselves.
    #[serde(serialize_with = "serialize_next_rules", skip_serializing_if = "Option::is_none")]
    next_rules: Option<Generation>,
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
    /// owns belong to the player who takes it, and the banks start at their faction's Starting Conditions
    /// ([`Player::starting`]). A layout's Cults units belong to its Recruitment Hall (`Object::lineage`, slice M5-B).
    /// Never panics.
    pub fn try_new(scenario: &Scenario) -> Result<Self, Vec<ScenarioProblem>> {
        let objects = scenario.start_objects()?;
        let mut sim = Self::empty(scenario);
        let ids: Vec<ObjectId> = (objects.iter())
            .map(|o| sim.insert_object(o.kind, o.owner, o.pos, o.placement, o.settings.as_ref()))
            .collect();
        for (o, id) in objects.iter().zip(&ids) {
            if let Some(object) = sim.objects.get_mut(id) {
                object.lineage = o.lineage.iter().filter_map(|&i| ids.get(i).copied()).collect();
            }
        }
        sim.settle_start();
        Ok(sim)
    }

    /// The scenario's terrain and players at tick 0, without objects. Players beyond the last `PlayerId` are left
    /// out (only an invalid setup has them). Its grids are built once the objects are in
    /// (`NavState::build`).
    fn empty(scenario: &Scenario) -> Self {
        // A new game plays by the built-in rules (`sim/rules.rs`): the current ones from here on, whatever a game
        // before it played by, for the objects it starts with.
        ruleset::select(Generation::BUILTIN);
        let players = scenario
            .setup
            .player_ids()
            .zip(&scenario.setup.players)
            .map(|(id, p)| Player::starting(id, p.name.clone(), p.faction))
            .collect::<Vec<Player>>();
        Self {
            tick: 0,
            map: scenario.map.terrain.clone(),
            vision: Vision::new(&scenario.map.terrain, players.iter().map(|p| p.id)),
            players,
            outcome: None,
            objects: BTreeMap::new(),
            held: BTreeMap::new(),
            projectiles: Vec::new(),
            clouds: Clouds::default(),
            next_id: 1,
            rng: Rng::from_seed(scenario.setup.seed),
            config: scenario.setup.config,
            next_group: 0,
            nav_version: 0,
            auto_targeting: true,
            events: Vec::new(),
            path_knowledge: PathKnowledge::default(),
            nav: NavState::unbuilt(&scenario.map.terrain),
            rules: Generation::BUILTIN,
            next_rules: None,
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

    /// (M5-N1a) Every player's research, which the units' augmented stats follow (`augmentation::Research`): what to
    /// pass to `Object::max_hp_with`, `Object::movement`, `Object::max_speed` and `combat::weapon`.
    pub fn research(&self) -> Research<'_> {
        Research::new(&self.players)
    }

    pub fn player(&self, id: PlayerId) -> Option<&Player> {
        self.players.iter().find(|p| p.id == id)
    }

    /// How the game ended (`core/game_rules.md > End of Game`): `None` while it goes on. Set in the tick of the
    /// defeat that leaves at most one side standing (`economy::defeat`), and final: the gate refuses every command
    /// from then on ([`Reject::GameOver`]). The sim itself goes on stepping if asked; the client stops the game.
    pub fn outcome(&self) -> Option<&Outcome> {
        self.outcome.as_ref()
    }

    /// The other players on `player`'s side (design change 0102: a defeated player watches with their allies'
    /// vision), in id order. None: the sim has no team or alliance data, so every player is their own side
    /// ([`defeat::allied`]).
    pub fn allies(&self, player: PlayerId) -> Vec<PlayerId> {
        self.players
            .iter()
            .map(|p| p.id)
            .filter(|&id| id != player && defeat::allied(player, id))
            .collect()
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

    /// (M4-O) The projectiles in flight, in launch order (`combat::projectiles`). Each lands in phase 5e of its
    /// arrival tick, which is always after the latest tick.
    pub fn projectiles(&self) -> &[Projectile] {
        &self.projectiles
    }

    /// (M5-N4b) Shroud's clouds as the latest tick left them (`crate::shroud`), with the buffers of those that ended
    /// since its phase 5e, which the next one releases (`Clouds::ended`).
    pub fn clouds(&self) -> &Clouds {
        &self.clouds
    }

    /// Every player's vision at the end of the latest tick (`core/vision.md`): tile states, which objects each
    /// player sees, and a revision per player that changes with its tiles.
    pub fn vision(&self) -> &Vision {
        &self.vision
    }

    /// `core/vision.md > VisibilityState`: how `player` sees `tile` at the end of the latest tick. Unexplored off
    /// the map and for a player who isn't in the game.
    pub fn visibility(&self, player: PlayerId, tile: TilePos) -> VisibilityState {
        self.vision.state(player, tile)
    }

    /// Whether `player` sees `object` at the end of the latest tick: a tile it covers is Visible
    /// ([`crate::vision::PlayerVision::object_state`]).
    pub fn is_visible(&self, player: PlayerId, object: &Object) -> bool {
        self.vision.sees(player, object)
    }

    /// Whether units acquire targets by themselves: on, unless `DebugCommand::SetAutoTargeting` turned it off.
    pub fn auto_targeting(&self) -> bool {
        self.auto_targeting
    }

    /// FNV-1a 64 hash of the whole state: tick, map, players, the outcome, objects in id order, next id,
    /// PRNG state, config, next order group, nav version, each player's Explored tiles and records and its path
    /// knowledge, the held units, the projectiles in flight and (M5-N4b) the clouds while there are any (`f32` by its
    /// bits; encoding in `hash.rs`). Events are output, and the grids and the Visible tiles
    /// derived data, so they are left out. Equal states give equal hashes on every machine, so
    /// comparing hashes checks determinism.
    pub fn state_hash(&self) -> u64 {
        // Destructured without `..`: a new field fails to compile here until it is hashed.
        let Sim {
            tick,
            map,
            players,
            outcome,
            objects,
            held,
            projectiles,
            clouds,
            next_id,
            rng,
            config,
            next_group,
            nav_version,
            auto_targeting,
            events: _,
            vision,
            path_knowledge,
            nav: _,
            // (M5-X1b) The rules aren't the state (plan decision T66): a golden on the built-in rules fails only when
            // behaviour changes, and a tuned game's state hashes by what the rules did to it.
            rules: _,
            next_rules: _,
        } = self;
        let mut h = StateHasher::new();
        tick.hash(&mut h);
        map.hash(&mut h);
        players.hash(&mut h);
        outcome.hash(&mut h);
        h.len(objects.len());
        for o in objects.values() {
            o.hash(&mut h);
        }
        next_id.hash(&mut h);
        rng.hash(&mut h);
        config.hash(&mut h);
        next_group.hash(&mut h);
        nav_version.hash(&mut h);
        auto_targeting.hash(&mut h);
        vision.hash(&mut h);
        path_knowledge.hash(&mut h);
        // (M4-C) The held units, last and only while there are any, under a tag of their own: the states of games in
        // which no unit ever left the map hash as they did before the store existed, so the goldens stand.
        if !held.is_empty() {
            h.tag(1);
            h.len(held.len());
            for unit in held.values() {
                unit.hash(&mut h);
            }
        }
        // (M4-O) The projectiles in flight, the same way, under a tag of their own.
        if !projectiles.is_empty() {
            h.tag(2);
            h.len(projectiles.len());
            for p in projectiles {
                p.hash(&mut h);
            }
        }
        // (M5-N4b) The clouds and the buffers waiting for their release, the same way, under a tag of their own.
        if !clouds.is_empty() {
            h.tag(3);
            clouds.hash(&mut h);
        }
        h.finish()
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
        let object = self.new_object(id, kind, owner, pos, placement, settings);
        let is_patch = matches!(object.resource, Some(ResourceState::CrystalPatch { .. }));
        self.objects.insert(id, object);
        if is_patch {
            // A plate may be in the game before the patch under it (a map lists them in any order).
            economy::mining::link_patch(&mut self.objects, id);
        }
        self.events.push(SimEvent::Spawned(id));
        id
    }

    /// A new object `id` of `kind` with its type's initial state, not yet in the game: what [`Sim::insert_object`]
    /// inserts, and what a producer releases into the Tunnel Network (`sim/network.rs`). (M5-S8) A vehicle building
    /// starts with its rally point out of its exit side (`production::default_rally`, design change 0446: "it is set at
    /// placement, so it exists during construction"): a site keeps it while its production waits
    /// (`transform::hold_back`), and its completion keeps it (`Sim::start_functions`).
    fn new_object(
        &self,
        id: ObjectId,
        kind: ObjectKind,
        owner: Option<PlayerId>,
        pos: Vec2,
        placement: Option<Placement>,
        settings: Option<&SpawnSettings>,
    ) -> Object {
        let resource = SpawnSettings::resolve(kind, settings).map(|s| s.starting_resource());
        // Its capabilities, as a changed object gets them (`Sim::change_kind`).
        let kinds::Capabilities {
            orders,
            mover,
            attack,
            riot_shield,
            extractor,
            production,
            constructor,
            worker,
            tunnel,
        } = kinds::Capabilities::fresh(&self.objects, kind, placement.as_ref(), self.tick);
        let mut object = Object {
            id,
            kind,
            owner,
            pos,
            prev_pos: pos,
            facing: 0.0,
            prev_facing: 0.0,
            hp: kind.def().max_hp,
            // `systems/combat.md > Shields`: full, as the HP (`shields`).
            shields: kind.def().max_shields,
            placement,
            resource,
            orders,
            mover,
            attack,
            riot_shield,
            extractor,
            production,
            constructor,
            worker,
            transform: None,
            conversion: None,
            tunnel,
            construction: None,
            decay: None,
            // A turret is a pop-up from its first tick, lowered (`crate::popup`, design change 0360).
            popup: crate::popup::def_of(kind).map(|_| crate::popup::PopUp::LOWERED),
            lineage: Vec::new(),
            augmentation: Default::default(),
            // (M5-N4a) Its type's start (`crate::energy`).
            energy: crate::energy::start(kind),
            kept_research: None,
            buffs: Vec::new(),
            // (M6-GD) Its type's start (`crate::stock`). No drone is out, and a drone a debug command spawns is on no
            // leash (`crate::tether`): a launch tethers it.
            stock: crate::stock::fresh(kind),
            tether: None,
            drone: None,
        };
        if let Some(rally) = crate::production::default_rally(&self.map, &object)
            && let Some(production) = &mut object.production
        {
            production.rally = Some(rally);
        }
        object
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

mod builds;
mod casts;
mod commands;
mod construction;
mod deploy;
mod drones;
mod expansions;
mod gate;
mod holding;
mod kinds;
mod load;
mod modes;
mod navigation;
mod network;
mod phase;
mod popups;
mod preview;
mod producers;
mod recruitment;
mod removal;
mod repairs;
mod resources;
mod rules;
mod sites;
mod spot_work;
#[cfg(test)]
mod tests;
mod too_close;
mod transform;
mod tunnels;
mod upgrades;
mod workers;
