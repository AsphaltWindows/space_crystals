//! An AI-against-AI game, headless ([`play`]): each computer player thinks before each step, on the game as the last
//! tick left it, as the client runs them (`ARCHITECTURE.md > Computer players`), and after every step the game is
//! checked:
//! - `sc_sim::testing`'s invariants ([`Invariants::check`], as the sim's pairings check them, or the rules' alone for
//!   [`Checks::Rules`]), and no object names a removed one;
//! - the gate takes every command a computer player gives (`core/game_rules.md > Computer players (playtest tool)`:
//!   "It issues the same commands a player can, under the same rules");
//! - design change 0402's Verification: "In an AI-vs-AI game, an AI doesn't attack a building it has never seen or
//!   scouted, apart from the known start locations" ([`Knowledge::aim`]). Every Attack's target is a building its
//!   player knew of by then (seen or remembered: what its View showed in some tick up to this one) or its own, and
//!   every Attack Move's point is where it knew one to stand, a start location, a Space Crystals patch the map placed
//!   (`core/game_rules.md > Computer players (playtest tool)`: "It also knows the map"), where a wave searches for the
//!   last buildings, a point of the ground a wave searches around a start (`sc_ai::army::search_points`, slice A2a), or
//!   a point of its own: its staging point or where one of its buildings stands or stood.

use std::collections::BTreeMap;
use std::time::{Duration, Instant};

use sc_ai::army::search_points;
use sc_ai::{Ai, Profile, Start, View};
use sc_sim::testing::{self, invariants::Invariants};
use sc_sim::{
    Category, Command, Object, ObjectId, ObjectKind, OrderKind, PlayerCommand, PlayerId, Scenario, Sim, SimEvent, Tick,
    TickInputs, Vec2,
};

/// What a game came to.
#[derive(Clone, Debug)]
pub struct Report {
    /// The ticks played: to the end of the game, or the limit.
    pub ticks: Tick,
    /// The winners, if the game ended; empty for a draw.
    pub winners: Option<Vec<PlayerId>>,
    /// Each player's side, in player order.
    pub sides: Vec<Side>,
    /// The state hash at the end.
    pub hash: u64,
    /// What `Ai::think` cost: each call's time.
    pub think: Cost,
}

impl Report {
    /// `player`'s side.
    pub fn side(&self, player: PlayerId) -> &Side {
        (self.sides.iter().find(|s| s.player == player)).expect("a player of the game")
    }

    /// The only winner, if the game ended with one.
    pub fn winner(&self) -> Option<PlayerId> {
        match self.winners.as_deref() {
            Some(&[winner]) => Some(winner),
            _ => None,
        }
    }
}

/// One player's side of a game.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Side {
    pub player: PlayerId,
    /// The units their buildings trained.
    pub trained: u32,
    /// Their units and buildings destroyed.
    pub lost: u32,
    /// The waves their computer player sent.
    pub waves: u32,
    /// The commands their computer player gave.
    pub commands: u32,
    /// Their computer player's Attacks and Attack Moves aimed at an enemy building it knew of ([`Aim::Building`]).
    pub at_buildings: u32,
    /// Their buildings at the end.
    pub buildings: u32,
}

/// The time of each call to `Ai::think`.
#[derive(Clone, Debug, Default)]
pub struct Cost {
    pub calls: u32,
    pub total: Duration,
    pub worst: Duration,
}

impl Cost {
    pub fn mean(&self) -> Duration {
        self.total.checked_div(self.calls).unwrap_or_default()
    }

    /// Adds `other`'s calls.
    pub fn add(&mut self, other: &Cost) {
        self.calls += other.calls;
        self.total += other.total;
        self.worst = self.worst.max(other.worst);
    }
}

/// How much to check after each step.
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum Checks {
    /// Every invariant (`Invariants::check`).
    All,
    /// The invariants of the game's rules alone (`Invariants::check_rules`), which cost little: for a game whose
    /// every check another test makes.
    Rules,
}

/// Plays `scenario` with a computer player of each of `ais`' profiles in each of its players' slots, at most `limit`
/// ticks, checking it after every step (module docs). Panics at the first check that fails.
pub fn play(scenario: &Scenario, ais: &[(PlayerId, Profile)], limit: Tick, checks: Checks) -> Report {
    let mut sim = Sim::new(scenario);
    let starts = Start::all(scenario);
    let mut players: Vec<Ai> = (ais.iter())
        .map(|&(player, profile)| Ai::new(scenario, &sim, player, profile).expect("a computer player"))
        .collect();
    let mut invariants = Invariants::new(&sim);
    let mut sides: Vec<Side> = (sim.players().iter())
        .map(|p| Side {
            player: p.id,
            trained: 0,
            lost: 0,
            waves: 0,
            commands: 0,
            at_buildings: 0,
            buildings: 0,
        })
        .collect();
    let mut knowledge: BTreeMap<PlayerId, Knowledge> = BTreeMap::new();
    let mut think = Cost::default();
    while sim.tick() < limit && sim.outcome().is_none() {
        let tick = sim.tick() + 1;
        let mut commands = Vec::new();
        for ai in &mut players {
            let known = knowledge.entry(ai.player()).or_default();
            known.learn(&View::new(&sim, ai.player(), &starts));
            let (given, spent) = timed(|| ai.think(&sim));
            think.calls += 1;
            think.total += spent;
            think.worst = think.worst.max(spent);
            let side = (sides.iter_mut().find(|s| s.player == ai.player())).expect("a computer player's side");
            for c in &given {
                assert_eq!(c.player, ai.player(), "tick {tick}: a command in another player's name");
                if let Err(reject) = sim.validate(c) {
                    panic!("tick {tick}: the gate refuses {c:?}: {reject:?}");
                }
                match known.aim(&sim, &starts, ai.army().staging(), c) {
                    Ok(aim) => side.at_buildings += u32::from(aim == Some(Aim::Building)),
                    Err(unfair) => panic!("tick {tick}: {unfair}"),
                }
            }
            side.commands += given.len() as u32;
            commands.extend(given);
        }
        let inputs = TickInputs { commands };
        sim.step(&inputs);
        let checked = match checks {
            Checks::All => invariants.check(&sim, &inputs),
            Checks::Rules => invariants.check_rules(&sim, &inputs),
        };
        if let Err(violation) = checked {
            panic!("tick {tick}: {violation:?}");
        }
        assert_eq!(
            testing::dangling_references(&sim),
            [],
            "tick {tick}: objects name removed ones"
        );
        count(&sim, &mut sides);
    }
    for side in &mut sides {
        let building = |o: &&Object| o.owner == Some(side.player) && o.kind.def().category == Category::Structure;
        side.buildings = sim.objects().filter(building).count() as u32;
        side.waves = (players.iter())
            .find(|ai| ai.player() == side.player)
            .map_or(0, |ai| ai.army().waves_sent());
    }
    Report {
        ticks: sim.tick(),
        winners: sim.outcome().map(|o| o.winners.clone()),
        sides,
        hash: sim.state_hash(),
        think,
    }
}

/// `f`'s result, and the time it took: the clock's one reading in the computer players' tests (`ARCHITECTURE.md >
/// Determinism`'s exception for a timing helper), which measures `Ai::think`. The time is printed, never asserted,
/// and nothing the game does depends on it.
#[expect(clippy::disallowed_methods)]
fn timed<T>(f: impl FnOnce() -> T) -> (T, Duration) {
    let started = Instant::now();
    let result = f();
    (result, started.elapsed())
}

/// What a player has known of the other players' buildings, each one's id and every point it knew it to stand at,
/// where the map's patches are, and where its own buildings have stood: its View's, tick by tick ([`Knowledge::learn`]).
#[derive(Clone, Debug, Default)]
pub struct Knowledge {
    buildings: BTreeMap<ObjectId, Vec<Vec2>>,
    patches: Vec<Vec2>,
    own: Vec<Vec2>,
}

/// What a computer player's Attack or Attack Move aims at, by the rules of 0402's Verification (module docs).
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum Aim {
    /// An enemy building the player knew of: its id, or a point it knew it to stand at.
    Building,
    /// A start location.
    Start,
    /// A Space Crystals patch of the map.
    Patch,
    /// A point of the ground a wave searches around a start location (`sc_ai::army::search_points`).
    Around,
    /// Something of the player's own: one of its objects, its staging point, or where one of its buildings stood.
    Own,
}

impl Knowledge {
    /// Adds what `view` shows now: the other players' buildings, the patches, and where the player's own buildings
    /// stand.
    pub fn learn(&mut self, view: &View<'_>) {
        for k in view.enemy_structures() {
            let points = self.buildings.entry(k.id).or_default();
            if !points.contains(&k.pos) {
                points.push(k.pos);
            }
        }
        for k in view.patches() {
            if !self.patches.contains(&k.pos) {
                self.patches.push(k.pos);
            }
        }
        for o in view.own().filter(|o| o.kind.def().category == Category::Structure) {
            if !self.own.contains(&o.pos) {
                self.own.push(o.pos);
            }
        }
    }

    /// What `c`, a command of a computer player whose staging point is `staging`, aims at if it is an Attack or an
    /// Attack Move; an error naming it if it aims at nothing 0402's Verification allows (module docs).
    pub fn aim(
        &self,
        sim: &Sim,
        starts: &[Start],
        staging: Option<Vec2>,
        c: &PlayerCommand,
    ) -> Result<Option<Aim>, String> {
        let Command::Order { kind, .. } = &c.command else {
            return Ok(None);
        };
        let knew_at = |point: Vec2| self.buildings.values().any(|points| points.contains(&point));
        let aim = match *kind {
            OrderKind::Attack(target) if self.buildings.contains_key(&target) => Aim::Building,
            OrderKind::Attack(target) if sim.object(target).is_some_and(|o| o.owner == Some(c.player)) => Aim::Own,
            OrderKind::AttackMove(point) if knew_at(point) => Aim::Building,
            OrderKind::AttackMove(point) if starts.iter().any(|s| s.point() == point) => Aim::Start,
            OrderKind::AttackMove(point) if self.patches.contains(&point) => Aim::Patch,
            OrderKind::AttackMove(point)
                if starts
                    .iter()
                    .any(|s| search_points(sim.map(), s.point()).contains(&point)) =>
            {
                Aim::Around
            }
            OrderKind::AttackMove(point) if staging == Some(point) || self.own.contains(&point) => Aim::Own,
            OrderKind::Attack(_) | OrderKind::AttackMove(_) => {
                return Err(format!("player {} aims {kind:?} at nothing it knew of", c.player.0));
            }
            _ => return Ok(None),
        };
        Ok(Some(aim))
    }
}

/// Counts the step's units trained and objects lost into `sides`.
fn count(sim: &Sim, sides: &mut [Side]) {
    let owner = |id: ObjectId| sim.object(id).and_then(|o| o.owner);
    for e in sim.events() {
        let (player, trained) = match *e {
            SimEvent::UnitProduced { producer, .. } => (owner(producer), true),
            SimEvent::Died { owner, kind, .. } if kind != ObjectKind::CrystalPatch => (owner, false),
            _ => continue,
        };
        let Some(side) = sides.iter_mut().find(|s| Some(s.player) == player) else {
            continue;
        };
        if trained {
            side.trained += 1;
        } else {
            side.lost += 1;
        }
    }
}
