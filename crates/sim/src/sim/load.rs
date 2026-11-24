//! A `Sim`'s serialized form: the objects as a list in id order, and a save checked before it becomes a `Sim`
//! (unique ids below the next id, the vision's records); the derived data (the grids, the Visible tiles) is
//! rebuilt from the objects.

use std::collections::BTreeMap;

use serde::{Deserialize, Serializer};

use super::{Sim, SimConfig};
use crate::economy::Outcome;
use crate::events::SimEvent;
use crate::ids::{ObjectId, PlayerId};
use crate::map::Map;
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
            if o.id.0 >= next_id {
                return Err(format!("object id {} is not below next_id {next_id}", o.id.0));
            }
            if let Some(duplicate) = objects.insert(o.id, o) {
                return Err(format!("object id {} appears more than once", duplicate.id.0));
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
        let nav = NavState::build(&map, &objects, &vision, &path_knowledge);
        Ok(Sim {
            tick,
            map,
            players,
            outcome,
            objects,
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
