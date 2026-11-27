//! The army layer: where a computer player's army gathers, when and where it attacks, and how it defends (Q0147 point
//! 2's standard profile: "attacks in waves of about 8 units on the nearest enemy building it knows, then reinforces;
//! pulls its army home when its base is attacked"; its passive profile "never attacks"). Every rule here is that
//! placeholder (`core/game_rules.md > Computer players (playtest tool)`: "Their behavior ... is the Director's
//! placeholder"), its numbers in `crate::tuning::army`. The faction's module says which units fight and which
//! buildings rally (`FactionAi::fights`, `FactionAi::rallies`).
//!
//! - **Staging**: a point between the player's start and the map's centre ([`staging_point`]). The buildings that
//!   rally send their units there; a unit at home that stands idle elsewhere is sent back.
//! - **Waves** (a profile that attacks, `Profile::attacks`): once [`FIRST_WAVE_SIZE`] units at home wait at staging,
//!   they Attack Move together ([`waves`]) to the nearest enemy building the player knows of, seen or remembered (the
//!   View: "Fair knowledge"), else to the nearest start of an enemy still in the game that no wave has reached. Later
//!   units form the next wave, of [`WAVE_SIZE`] (or however many have waited [`WAVE_PATIENCE_TICKS`]), which goes to
//!   the nearest building it knows of too: it reinforces. A faction may send waves of other sizes, or wait for them
//!   otherwise long (`FactionAi::waves`, [`Waves`]). A wave whose building is
//!   gone (the player saw it go) retargets at once; one that comes to rest retargets too, giving up a building it
//!   still knows of there. With no building known and every enemy start reached, it searches the patches near them
//!   (a GDO's last buildings are often plates), then the ground around them ([`search_points`]; slice A2a, for a Cults
//!   player's Halls on open land, and since slice A2d in every game, for a Colonists' Beacon or a Syndicate's Tunnel
//!   away from the patches too); with
//!   nothing left to search, it goes home and its units are at home again.
//! - **Defense**: when one of the player's buildings loses HP, the units at home (not in a wave) Attack Move to it, and
//!   go back to staging [`DEFENSE_TICKS`] after the last damage. No wave forms meanwhile.
//!
//! Every target is a building the player knows of, a start location, a patch of the map, a point of the ground around
//! a start (all four known by the rules of fair knowledge), or a point of its own: design change 0402's Verification,
//! "an AI doesn't attack a building it has never seen or scouted, apart from the known start locations".

mod waves;

use std::collections::BTreeMap;

use sc_sim::Map;
use sc_sim::orders::state::Auto;
use sc_sim::{
    Category, Command, Object, ObjectId, OrderKind, PlayerId, ProductionCommand, Rally, SU_PER_GU, Target, Tick,
    TilePos, Vec2, gu,
};

use crate::build::open;
use crate::factions::FactionAi;
use crate::tuning::army::{
    DEFENSE_MOVE_GU, DEFENSE_TICKS, FIRST_WAVE_SIZE, GATHER_RETRY_TICKS, SEARCH_RADIUS_GU, SEARCH_STEP_GU,
    STAGING_DISTANCE_GU, STAGING_RADIUS_GU, STAGING_SHARE, WAVE_PATIENCE_TICKS, WAVE_SIZE,
};
use crate::{Turn, View};
use waves::Wave;

/// A computer player's army (module docs): what each of its units does, its waves and its defense.
#[derive(Clone, Debug, Default)]
pub struct Army {
    staging: Option<Vec2>,
    /// What each of its fighting units does, by id.
    duties: BTreeMap<ObjectId, Duty>,
    waves: Vec<Wave>,
    /// How many waves it has sent.
    sent: u32,
    /// Since when units have waited at staging for the next wave.
    waiting: Option<Tick>,
    alarm: Option<Alarm>,
    /// Its buildings' HP at the end of the latest tick it looked at, by id.
    buildings: BTreeMap<ObjectId, f32>,
    /// The enemies whose start a wave has reached since the player last knew of nothing to attack.
    visited: Vec<PlayerId>,
}

/// The waves a faction's army sends (`FactionAi::waves`): how many units at staging the first wave and each later one
/// wait for, and how long units wait there before they go however few they are. Its default is
/// `crate::tuning::army`'s: [`FIRST_WAVE_SIZE`], [`WAVE_SIZE`] and [`WAVE_PATIENCE_TICKS`].
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub struct Waves {
    pub first: usize,
    pub later: usize,
    pub patience: Tick,
}

impl Default for Waves {
    fn default() -> Self {
        Self {
            first: FIRST_WAVE_SIZE,
            later: WAVE_SIZE,
            patience: WAVE_PATIENCE_TICKS,
        }
    }
}

/// What one of the army's units does.
#[derive(Copy, Clone, Debug, PartialEq)]
enum Duty {
    /// At home: waiting at staging or on its way there. `sent`: the tick it was last sent back there.
    Home { sent: Option<Tick> },
    /// Defending the base: Attack Moving to the attacked building at `to`.
    Defend { to: Vec2 },
    /// In the wave of this number.
    Wave(u32),
}

/// The defense: where one of the player's buildings was last attacked, and until when the units at home defend it.
#[derive(Copy, Clone, Debug, PartialEq)]
struct Alarm {
    at: Vec2,
    until: Tick,
}

impl Army {
    /// The army of the player whose View it is, at the start: its staging point, and no units yet.
    pub fn new(view: &View<'_>) -> Self {
        Self {
            staging: staging_point(view),
            ..Self::default()
        }
    }

    /// Where the army gathers ([`staging_point`]).
    pub fn staging(&self) -> Option<Vec2> {
        self.staging
    }

    /// How many waves it has sent.
    pub fn waves_sent(&self) -> u32 {
        self.sent
    }

    /// The turn's orders to the units `faction` fights with and its rally points (module docs).
    pub fn think(&mut self, turn: &mut Turn<'_, '_>, faction: &dyn FactionAi) {
        let view = turn.view();
        let now = view.tick();
        let units: Vec<&Object> = view.own().filter(|o| faction.fights(o)).collect();
        self.rally(turn, faction);
        self.watch(view, now);
        self.duties
            .retain(|id, _| units.binary_search_by_key(id, |o| o.id).is_ok());
        for o in &units {
            self.duties.entry(o.id).or_insert(Duty::Home { sent: None });
        }
        self.defend(turn, now);
        if turn.profile().attacks() && self.alarm.is_none() {
            self.form_wave(turn, &units, now, faction.waves());
        }
        self.lead_waves(turn, &units, now);
        if self.alarm.is_none() {
            self.gather(turn, &units, now);
        }
    }

    /// Points every building that rallies at the staging point, if it rallies elsewhere.
    fn rally(&self, turn: &mut Turn<'_, '_>, faction: &dyn FactionAi) {
        let Some(staging) = self.staging else {
            return;
        };
        let wanted = Some(Rally::Point(staging));
        let producers: Vec<ObjectId> = (turn.view().own())
            .filter(|o| faction.rallies(o) && o.production.as_ref().is_some_and(|p| p.rally != wanted))
            .map(|o| o.id)
            .collect();
        if !producers.is_empty() {
            turn.give(Command::Production {
                objects: producers,
                command: ProductionCommand::SetRally(Target::Point(staging)),
            });
        }
    }

    /// Raises the alarm when one of the player's buildings has lost HP since the latest tick it looked at: at the
    /// first of them by id, for [`DEFENSE_TICKS`]. Damage near the alarm's building extends it; damage farther away
    /// moves it there.
    fn watch(&mut self, view: &View<'_>, now: Tick) {
        let mut attacked = None;
        let mut buildings = BTreeMap::new();
        for o in view.own().filter(|o| o.kind.def().category == Category::Structure) {
            let Some(hp) = o.hp else {
                continue;
            };
            if attacked.is_none() && self.buildings.get(&o.id).is_some_and(|&was| hp < was) {
                attacked = Some(o.pos);
            }
            buildings.insert(o.id, hp);
        }
        self.buildings = buildings;
        let Some(at) = attacked else {
            return;
        };
        let until = now + DEFENSE_TICKS;
        match &mut self.alarm {
            Some(alarm) if alarm.at.distance_squared(at) < gu(DEFENSE_MOVE_GU) * gu(DEFENSE_MOVE_GU) => {
                alarm.until = until;
            }
            alarm => *alarm = Some(Alarm { at, until }),
        }
    }

    /// While the alarm lasts, sends the units at home, and the defenders sent to another building, to the attacked
    /// one; once it is over, sends the defenders back to staging.
    fn defend(&mut self, turn: &mut Turn<'_, '_>, now: Tick) {
        let Some(alarm) = self.alarm else {
            return;
        };
        if now > alarm.until {
            self.alarm = None;
            let back = self.with(|duty| matches!(duty, Duty::Defend { .. }));
            if let Some(staging) = self.staging {
                attack_move(turn, &back, staging);
            }
            for id in back {
                self.duties.insert(id, Duty::Home { sent: Some(now) });
            }
            return;
        }
        let near = |to: Vec2| to.distance_squared(alarm.at) < gu(DEFENSE_MOVE_GU) * gu(DEFENSE_MOVE_GU);
        let to = self.with(|duty| match duty {
            Duty::Home { .. } => true,
            Duty::Defend { to } => !near(to),
            Duty::Wave(_) => false,
        });
        attack_move(turn, &to, alarm.at);
        for id in to {
            self.duties.insert(id, Duty::Defend { to: alarm.at });
        }
    }

    /// Sends the units at home that wait at staging as a wave once there are as many as it waits for
    /// ([`Army::wave_size`]), or however many once units have waited for its faction's patience ([`Waves::patience`]),
    /// if it knows of something to attack or to search.
    fn form_wave(&mut self, turn: &mut Turn<'_, '_>, units: &[&Object], now: Tick, waves: Waves) {
        let Some(staging) = self.staging else {
            return;
        };
        let radius = gu(STAGING_RADIUS_GU) * gu(STAGING_RADIUS_GU);
        let waiting: Vec<ObjectId> = (units.iter())
            .filter(|o| matches!(self.duties.get(&o.id), Some(Duty::Home { .. })))
            .filter(|o| at_rest(o) && o.pos.distance_squared(staging) <= radius)
            .map(|o| o.id)
            .collect();
        if waiting.is_empty() {
            self.waiting = None;
            return;
        }
        let since = *self.waiting.get_or_insert(now);
        if waiting.len() < self.wave_size(waves) && now < since + waves.patience {
            return;
        }
        let goal = self.choose(turn.view(), staging, &waves::Done::default(), true);
        if goal.is_home() {
            return;
        }
        attack_move(turn, &waiting, goal.point());
        self.waiting = None;
        self.sent += 1;
        for &id in &waiting {
            self.duties.insert(id, Duty::Wave(self.sent));
        }
        self.waves.push(Wave::new(self.sent, waiting, goal, now));
    }

    /// Sends the units at home that stand idle away from staging back to it, each at most every
    /// [`GATHER_RETRY_TICKS`].
    fn gather(&mut self, turn: &mut Turn<'_, '_>, units: &[&Object], now: Tick) {
        let Some(staging) = self.staging else {
            return;
        };
        let radius = gu(STAGING_RADIUS_GU) * gu(STAGING_RADIUS_GU);
        let away: Vec<ObjectId> = (units.iter())
            .filter(|o| match self.duties.get(&o.id) {
                Some(Duty::Home { sent }) => sent.is_none_or(|t| now >= t + GATHER_RETRY_TICKS),
                _ => false,
            })
            .filter(|o| at_rest(o) && o.pos.distance_squared(staging) > radius)
            .map(|o| o.id)
            .collect();
        attack_move(turn, &away, staging);
        for id in away {
            self.duties.insert(id, Duty::Home { sent: Some(now) });
        }
    }

    /// How many units the next wave of a faction sending `waves` waits for: [`Waves::first`] for the first,
    /// [`Waves::later`] for each one after.
    pub fn wave_size(&self, waves: Waves) -> usize {
        if self.sent == 0 { waves.first } else { waves.later }
    }

    /// The units whose duty `which` picks, by id.
    fn with(&self, which: impl Fn(Duty) -> bool) -> Vec<ObjectId> {
        (self.duties.iter())
            .filter(|&(_, &duty)| which(duty))
            .map(|(&id, _)| id)
            .collect()
    }
}

/// Whether `o` has nothing to do: no order, and not fighting or returning by itself.
fn at_rest(o: &Object) -> bool {
    let idle =
        (o.orders.as_ref()).is_none_or(|orders| orders.is_idle() && matches!(orders.auto, Auto::Watching { .. }));
    idle && o.attack.as_ref().is_none_or(|attack| attack.target.is_none())
}

/// Gives `units` an Attack Move to `to`, if there are any.
fn attack_move(turn: &mut Turn<'_, '_>, units: &[ObjectId], to: Vec2) {
    if !units.is_empty() {
        turn.give(Command::Order {
            units: units.to_vec(),
            kind: OrderKind::AttackMove(to),
            queue: false,
        });
    }
}

/// The army's staging point (module docs): from the player's start toward the map's centre, [`STAGING_DISTANCE_GU`]
/// and at most [`STAGING_SHARE`] of the way, at the centre of the nearest tile of open ground. `None` for a player
/// without a start location.
pub fn staging_point(view: &View<'_>) -> Option<Vec2> {
    let home = view.my_start()?.point();
    let map = view.map();
    let centre = Vec2::new(map.width() as f32, map.height() as f32) * (SU_PER_GU / 2.0);
    let way = centre - home;
    let length = way.length();
    let point = if length > 0.0 {
        home + way * (gu(STAGING_DISTANCE_GU).min(length * STAGING_SHARE) / length)
    } else {
        home
    };
    Some(open_near(map, point))
}

/// The points a wave searches around an enemy's start at `start` once it knows of nothing to attack and has searched
/// the patches there (`waves`; slice A2a): a square grid of [`SEARCH_STEP_GU`] centred on it, the points within
/// [`SEARCH_RADIUS_GU`] of it, each moved to the tile of open ground nearest it ([`open_near`]), row by row from the
/// north and the west, without repeats. They depend only on the map and the start, both known from the start of the
/// game (`core/game_rules.md > Computer players (playtest tool)`: "It also knows the map and every player's starting
/// location").
pub fn search_points(map: &Map, start: Vec2) -> Vec<Vec2> {
    let steps = (SEARCH_RADIUS_GU / SEARCH_STEP_GU).floor() as i32;
    let radius = gu(SEARCH_RADIUS_GU) * gu(SEARCH_RADIUS_GU);
    let mut points = Vec::new();
    for dy in -steps..=steps {
        for dx in -steps..=steps {
            let offset = Vec2::new(dx as f32, dy as f32) * gu(SEARCH_STEP_GU);
            if offset.length_squared() > radius {
                continue;
            }
            let point = open_near(map, start + offset);
            if !points.contains(&point) {
                points.push(point);
            }
        }
    }
    points
}

/// The centre of the tile of open ground nearest `point` within 8 tiles of it (ties to the north, then the west), or
/// `point` on the map if there is none.
fn open_near(map: &Map, point: Vec2) -> Vec2 {
    let point = map.clamp_point(point);
    let tile = TilePos::containing(point);
    let mut best: Option<(f32, TilePos)> = None;
    for y in tile.y - 8..=tile.y + 8 {
        for x in tile.x - 8..=tile.x + 8 {
            let t = TilePos::new(x, y);
            if !open(map, t) {
                continue;
            }
            let d = t.center().distance_squared(point);
            if best.is_none_or(|(b, _)| d < b) {
                best = Some((d, t));
            }
        }
    }
    best.map_or(point, |(_, t)| t.center())
}
