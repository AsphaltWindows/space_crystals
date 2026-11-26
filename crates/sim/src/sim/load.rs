//! A `Sim`'s serialized form: the objects as a list in id order, and a save checked before it becomes a `Sim`
//! (unique ids below the next id, the vision's records); the derived data (the grids, the Visible tiles) is
//! rebuilt from the objects.

use std::collections::BTreeMap;

use serde::{Deserialize, Serializer};

use super::{Sim, SimConfig};
use crate::combat::Projectile;
use crate::economy::Outcome;
use crate::events::SimEvent;
use crate::ids::{ObjectId, PlayerId};
use crate::map::Map;
use crate::network::Held;
use crate::object::Object;
use crate::paths::NavState;
use crate::paths::knowledge::PathKnowledge;
use crate::player::Player;
use crate::rng::Rng;
use crate::units::Tick;
use crate::vision;

pub(super) fn serialize_objects<S: Serializer>(
    objects: &BTreeMap<ObjectId, Object>,
    serializer: S,
) -> Result<S::Ok, S::Error> {
    serializer.collect_seq(objects.values())
}

pub(super) fn serialize_held<S: Serializer>(held: &BTreeMap<ObjectId, Held>, serializer: S) -> Result<S::Ok, S::Error> {
    serializer.collect_seq(held.values())
}

/// The serialized form of [`Sim`], checked before it becomes one.
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
pub(super) struct SimState {
    tick: Tick,
    map: Map,
    players: Vec<Player>,
    /// Optional, so that states saved before the end of the game existed still load.
    #[serde(default)]
    outcome: Option<Outcome>,
    objects: Vec<Object>,
    /// Optional, so that states saved before units could leave the map still load, and those without one since.
    #[serde(default)]
    held: Vec<Held>,
    /// Optional, so that states saved before projectiles existed still load, and those without one in flight.
    #[serde(default)]
    projectiles: Vec<Projectile>,
    next_id: u32,
    rng: Rng,
    config: SimConfig,
    next_group: u32,
    nav_version: u32,
    auto_targeting: bool,
    events: Vec<SimEvent>,
    vision: Vec<vision::SavedVision>,
    path_knowledge: PathKnowledge,
}

impl TryFrom<SimState> for Sim {
    type Error = String;

    fn try_from(state: SimState) -> Result<Self, String> {
        let SimState {
            tick,
            map,
            players,
            outcome,
            objects: list,
            held: held_list,
            projectiles,
            next_id,
            rng,
            config,
            next_group,
            nav_version,
            auto_targeting,
            events,
            vision: saved,
            path_knowledge,
        } = state;
        let mut objects = BTreeMap::new();
        for mut o in list {
            // An object saved before it had Shields (slice M3-B, `Object::shields`) is at full Shields.
            if o.shields.is_none() {
                o.shields = o.max_shields();
            }
            // (M5-N4a) One saved before it had energy (`Object::energy`) has its type's start.
            if o.energy.is_none() {
                o.energy = crate::energy::start(o.kind);
            }
            if o.id.0 >= next_id {
                return Err(format!("object id {} is not below next_id {next_id}", o.id.0));
            }
            if let Some(duplicate) = objects.insert(o.id, o) {
                return Err(format!("object id {} appears more than once", duplicate.id.0));
            }
        }
        // The held units share the objects' ids (plan decision T30): each below the next id, none twice, none an
        // object's.
        let mut held = BTreeMap::new();
        for mut unit in held_list {
            // (M5-N4a) The same for a held unit.
            if unit.object.energy.is_none() {
                unit.object.energy = crate::energy::start(unit.object.kind);
            }
            let id = unit.object.id;
            if id.0 >= next_id {
                return Err(format!("held unit id {} is not below next_id {next_id}", id.0));
            }
            if objects.contains_key(&id) || held.insert(id, unit).is_some() {
                return Err(format!("held unit id {} appears more than once", id.0));
            }
        }
        // A projectile's objects have ids, which it may outlive (`combat::projectiles`); between ticks it is in flight.
        for p in &projectiles {
            if let Some(id) = [Some(p.shooter), p.target]
                .into_iter()
                .flatten()
                .find(|id| id.0 >= next_id)
            {
                return Err(format!("projectile object id {} is not below next_id {next_id}", id.0));
            }
            if p.lands_at <= tick {
                return Err(format!(
                    "projectile due in tick {} of a state at tick {tick}",
                    p.lands_at
                ));
            }
        }
        // The Visible tiles follow from the objects, which is how phase 5f left them.
        let ids: Vec<PlayerId> = players.iter().map(|p| p.id).collect();
        if let Some(winner) = outcome.iter().flat_map(|o| &o.winners).find(|w| !ids.contains(w)) {
            return Err(format!("winner {} is not in the game", winner.0));
        }
        let vision = vision::load(&map, &ids, saved, tick, objects.values())?;
        // Between ticks the true grids match the objects and each player's what it knows (`paths`), so rebuilding
        // them gives the running game's.
        path_knowledge.check(&ids, &map)?;
        // The grids of the units in the game and of those about to come into it (`paths`, *Which grids*). The running
        // game may have more, of units gone since, which no unit reads: the loaded game plays the same.
        let keys = super::navigation::walking_keys(&objects, &held);
        let nav = NavState::build(&map, &objects, &keys, &vision, &path_knowledge);
        Ok(Sim {
            tick,
            map,
            players,
            outcome,
            objects,
            held,
            projectiles,
            next_id,
            rng,
            config,
            next_group,
            nav_version,
            auto_targeting,
            events,
            vision,
            path_knowledge,
            nav,
        })
    }
}
