//! The army's waves (`super` module docs, Q0147 point 2: "attacks in waves of about 8 units on the nearest enemy
//! building it knows, then reinforces"): what each one attacks, and when it retargets, searches or goes home.

use sc_sim::{Object, ObjectId, PlayerId, Tick, TilePos, Vec2, VisibilityState, gu};

use super::{Army, Duty, at_rest, attack_move, search_points};
use crate::tuning::army::SEARCH_RADIUS_GU;
use crate::{Turn, View};

/// A wave: its units, what it attacks, and what it gave up on or searched.
#[derive(Clone, Debug)]
pub(super) struct Wave {
    number: u32,
    members: Vec<ObjectId>,
    goal: Goal,
    /// The tick its latest order applies in: until then its units show what they did before.
    applies: Tick,
    /// What it is done with: the enemy buildings it came to rest by and still knew of, and the places it searched.
    done: Done,
}

/// What a wave is done with (see [`Wave`]).
#[derive(Clone, Debug, Default)]
pub(super) struct Done {
    given_up: Vec<ObjectId>,
    searched: Vec<Vec2>,
}

impl Wave {
    /// Wave `number`, of `members`, sent to `goal` in the turn of tick `now`.
    pub(super) fn new(number: u32, members: Vec<ObjectId>, goal: Goal, now: Tick) -> Self {
        Self {
            number,
            members,
            goal,
            applies: now + 1,
            done: Done::default(),
        }
    }
}

/// What a wave Attack Moves to.
#[derive(Copy, Clone, Debug, PartialEq)]
pub(super) enum Goal {
    /// An enemy building the player knows of, where it saw it last.
    Building { id: ObjectId, at: Vec2 },
    /// The start location of an enemy still in the game.
    Start { player: PlayerId, at: Vec2 },
    /// A Space Crystals patch near an enemy's start, or a point of the ground around it ([`search_points`]), where a
    /// building it hasn't seen may stand (the map's patches, the starts and the factions are known from the start).
    Search { at: Vec2 },
    /// Home: the staging point.
    Home { at: Vec2 },
}

impl Goal {
    pub(super) fn point(&self) -> Vec2 {
        match *self {
            Goal::Building { at, .. } | Goal::Start { at, .. } | Goal::Search { at } | Goal::Home { at } => at,
        }
    }

    pub(super) fn is_home(&self) -> bool {
        matches!(self, Goal::Home { .. })
    }
}

impl Army {
    /// What a wave at `from` attacks: the nearest building of an enemy still in the game that the player knows of,
    /// seen or remembered, but those it gave up on; else the nearest start of an enemy still in the game that no wave
    /// has reached; else the nearest patch within [`SEARCH_RADIUS_GU`] of such a start that it hasn't searched and the
    /// player doesn't see now; else the nearest such point of the ground around its start ([`search_points`]); else
    /// home. A `fresh` wave, about to form, tries every enemy start again once waves have reached them all.
    pub(super) fn choose(&mut self, view: &View<'_>, from: Vec2, done: &Done, fresh: bool) -> Goal {
        let standing: Vec<PlayerId> = view.enemy_starts().map(|s| s.player).collect();
        let enemy = |owner: Option<PlayerId>| owner.is_some_and(|p| standing.contains(&p));
        let nearest = |a: Vec2, b: Vec2| a.distance_squared(from).total_cmp(&b.distance_squared(from));
        let building = (view.enemy_structures())
            .filter(|k| k.kind.def().destructible && enemy(k.owner) && !done.given_up.contains(&k.id))
            .min_by(|a, b| nearest(a.pos, b.pos).then(a.id.cmp(&b.id)));
        if let Some(k) = building {
            return Goal::Building { id: k.id, at: k.pos };
        }
        if fresh && view.enemy_starts().all(|s| self.visited.contains(&s.player)) {
            self.visited.clear();
        }
        let start = (view.enemy_starts())
            .filter(|s| !self.visited.contains(&s.player))
            .min_by(|a, b| nearest(a.point(), b.point()).then(a.player.cmp(&b.player)));
        if let Some(s) = start {
            return Goal::Start {
                player: s.player,
                at: s.point(),
            };
        }
        let radius = gu(SEARCH_RADIUS_GU) * gu(SEARCH_RADIUS_GU);
        let near_a_start = |p: Vec2| view.enemy_starts().any(|s| s.point().distance_squared(p) <= radius);
        let unseen = |p: Vec2| {
            !done.searched.contains(&p) && view.visibility(TilePos::containing(p)) != VisibilityState::Visible
        };
        let search = (view.patches())
            .filter(|k| near_a_start(k.pos) && unseen(k.pos))
            .min_by(|a, b| nearest(a.pos, b.pos).then(a.id.cmp(&b.id)));
        if let Some(k) = search {
            return Goal::Search { at: k.pos };
        }
        // (A2a) Then the ground around the starts, where a building may stand away from any patch (slice A2d: in every
        // game).
        let around = (view.enemy_starts())
            .flat_map(|s| search_points(view.map(), s.point()))
            .filter(|&p| unseen(p))
            .min_by(|&a, &b| nearest(a, b));
        match around {
            Some(at) => Goal::Search { at },
            None => Goal::Home {
                at: self.staging.unwrap_or(from),
            },
        }
    }

    /// Leads each wave (`super` module docs): drops its lost units, and retargets it once its building is gone or it
    /// comes to rest. A wave that comes to rest at home is over: its units are at home again.
    pub(super) fn lead_waves(&mut self, turn: &mut Turn<'_, '_>, units: &[&Object], now: Tick) {
        let view = turn.view();
        let unit = |id: &ObjectId| units.binary_search_by_key(id, |o| o.id).ok().map(|i| units[i]);
        let mut waves = std::mem::take(&mut self.waves);
        for wave in &mut waves {
            wave.members.retain(|id| unit(id).is_some());
            let members: Vec<&Object> = wave.members.iter().filter_map(unit).collect();
            let known = match wave.goal {
                Goal::Building { id, .. } => view.enemy_structures().any(|k| k.id == id),
                Goal::Start { player, .. } => view.enemy_starts().any(|s| s.player == player),
                Goal::Search { .. } | Goal::Home { .. } => true,
            };
            let rests = now >= wave.applies && members.iter().all(|o| at_rest(o));
            if members.is_empty() || (known && !rests) {
                continue;
            }
            if rests {
                match wave.goal {
                    Goal::Building { id, .. } if known => wave.done.given_up.push(id),
                    Goal::Start { player, .. } => self.visited.push(player),
                    Goal::Search { at } => wave.done.searched.push(at),
                    Goal::Home { .. } => {
                        for &id in &wave.members {
                            self.duties.insert(id, Duty::Home { sent: Some(now) });
                        }
                        wave.members.clear();
                        continue;
                    }
                    Goal::Building { .. } => {}
                }
            }
            let sum = members.iter().fold(Vec2::ZERO, |sum, o| sum + o.pos);
            let from = sum / members.len() as f32;
            let goal = self.choose(view, from, &wave.done, false);
            attack_move(turn, &wave.members, goal.point());
            wave.goal = goal;
            wave.applies = now + 1;
        }
        waves.retain(|wave| !wave.members.is_empty());
        self.waves = waves;
        let numbers: Vec<u32> = self.waves.iter().map(|w| w.number).collect();
        // A unit whose wave is over without it (every other unit lost) is at home again.
        for duty in self.duties.values_mut() {
            if let Duty::Wave(n) = *duty
                && !numbers.contains(&n)
            {
                *duty = Duty::Home { sent: None };
            }
        }
    }
}
