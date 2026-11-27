//! A `Sim`'s serialized form: the objects as a list in id order, (M5-X1b) the rules as themselves while they aren't
//! the built-in ones, and a save checked before it becomes a `Sim` (unique ids below the next id, the vision's records,
//! rules of this build's structure); the derived data (the grids, the Visible tiles) is rebuilt from the objects.

use std::collections::BTreeMap;

use serde::{Deserialize, Serialize, Serializer};

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
use crate::ruleset::{self, Generation, Ruleset};
use crate::shroud::Clouds;
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

/// (M5-X1b) A game's rules (`Sim::rules`) as the rules themselves: a generation's number is the process's.
pub(super) fn serialize_rules<S: Serializer>(rules: &Generation, serializer: S) -> Result<S::Ok, S::Error> {
    rules.rules().serialize(serializer)
}

/// (M5-X1b) The rules the next tick starts on (`Sim::next_rules`), as [`serialize_rules`].
pub(super) fn serialize_next_rules<S: Serializer>(
    rules: &Option<Generation>,
    serializer: S,
) -> Result<S::Ok, S::Error> {
    rules.map(Generation::rules).serialize(serializer)
}

/// (M5-X1b) A save's rules as a generation, installed (`crate::ruleset`): the built-in rules' for none. An error for
/// rules whose structure isn't this build's (a save of a build with other kinds or footprints,
/// `Ruleset::structural_diff`) and when the process has no generation left for them.
fn generation(rules: Option<&Ruleset>) -> Result<Generation, String> {
    let Some(rules) = rules else {
        return Ok(Generation::BUILTIN);
    };
    if let Some(change) = Ruleset::builtin_ref().structural_diff(rules) {
        return Err(format!(
            "the saved rules change {change}, which this build's rules don't"
        ));
    }
    ruleset::install(rules).ok_or_else(|| "no generation of rules is left for the saved rules".to_owned())
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
    /// (M5-N4b) Optional, so that states saved before Shroud still load, and those without a cloud.
    #[serde(default)]
    clouds: Clouds,
    next_id: u32,
    rng: Rng,
    config: SimConfig,
    next_group: u32,
    nav_version: u32,
    auto_targeting: bool,
    events: Vec<SimEvent>,
    vision: Vec<vision::SavedVision>,
    path_knowledge: PathKnowledge,
    /// (M5-X1b) The rules the game plays by, while they aren't the built-in ones (`Sim::rules`). Optional, so that the
    /// states saved before rules could change still load, and those of games on the built-in rules.
    #[serde(default)]
    rules: Option<Box<Ruleset>>,
    /// (M5-X1b) The rules of the last tick's `DebugCommand::SetRules`, which the next one starts on; optional, as
    /// `rules`.
    #[serde(default)]
    next_rules: Option<Box<Ruleset>>,
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
            clouds,
            next_id,
            rng,
            config,
            next_group,
            nav_version,
            auto_targeting,
            events,
            vision: saved,
            path_knowledge,
            rules,
            next_rules,
        } = state;
        // (M5-X1b) The rules first, the current ones from here on: what follows reads the defs (`sim/rules.rs`).
        let rules = generation(rules.as_deref())?;
        let next_rules = next_rules.as_deref().map(|rules| generation(Some(rules))).transpose()?;
        ruleset::select(rules);
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
        // (M5-N4b) A cloud names its units by their ids, which a buffer waiting for its release may outlive.
        if let Some(id) = clouds.ids().find(|id| id.0 >= next_id) {
            return Err(format!("cloud member id {} is not below next_id {next_id}", id.0));
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
            clouds,
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
            rules,
            next_rules,
        })
    }
}
