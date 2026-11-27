//! The soak's fight (`super`, module docs, 2): each player's [`EXTRAS`], debug spawns by their home, sent at the
//! point halfway to the next player's home, where they meet that player's, with the casts, channels and locks of a
//! player who uses them:
//! - the Cults' **Hand of Heaven** drives there on a Move, through whatever is in its way (a crusher crushes only on
//!   the move, `movement::crush`), then back and forth across it;
//! - their first **Sparrow** circles a map corner first (the seed's), off the map's edge as its turns take it
//!   (`systems/units.md > Air Collision`);
//! - their **Prayer-mobile** casts Suppress at enemies in its arc and Inspire on its own infantry around it, the seed
//!   choosing which comes first (it starts with 50 energy, regains 0.5 a second, and they cost 100 and 75): for
//!   Suppress first it waits at home until it can, then drives to the meeting point;
//! - their **Shrine**, spawned with the Recruits that charge it (`EnergyRegen::PerUnitInside`), casts Health
//!   regeneration on their army and Energy regeneration on their casters, in turn;
//! - their **Disciples** channel Shroud with enemies near, and the GDO's **Rocketmen** lock on.
//!
//! Each command is one the gate takes as the game stands before its step, a caster's one a tick.

use std::collections::BTreeMap;

use sc_sim::buffs;
use sc_sim::{
    AreaAbility, Command, DebugCommand, Faction, Object, ObjectId, ObjectKind, OrderKind, PlayerCommand, PlayerId, Rng,
    SU_PER_GU, Sim, SimEvent, SpawnAt, TICKS_PER_SECOND, Target, Tick, TilePos, Vec2,
};

use super::Tally;

use ObjectKind::{
    AssaultBike, Disciple, FieldEngineer, Grenadier, Guard, Gunner, HandOfHeaven, Militant, Peacekeeper, PrayerMobile,
    Recruit, RifleLegionnaire, Rocketman, SabreLegionnaire, Seeker, Shrine, Sparrow,
};

/// Each faction's extras for the fight, spawned by their home: the Cults' Recruits go into their Shrine.
const EXTRAS: &[(Faction, &[(ObjectKind, usize)])] = &[
    (Faction::Gdo, &[(Peacekeeper, 4), (Rocketman, 2), (FieldEngineer, 1)]),
    (Faction::Colonists, &[(SabreLegionnaire, 3), (RifleLegionnaire, 2)]),
    (Faction::Syndicate, &[(Guard, 2), (Grenadier, 2), (AssaultBike, 2)]),
    (
        Faction::Cults,
        &[
            (HandOfHeaven, 1),
            (PrayerMobile, 1),
            (Seeker, 1),
            (Sparrow, 2),
            (Militant, 3),
            (Gunner, 1),
            (Disciple, 2),
            (Recruit, 8),
        ],
    ),
];
/// The lattice the extras' spots are on, around their home, and how far out it reaches, in steps.
const STEP: f32 = 2.0 * SU_PER_GU;
const REACH: i32 = 7;
/// The nearest to their home's centre an extra spawns.
const CLEAR: f32 = 4.0 * SU_PER_GU;
/// How often idle extras are sent at the meeting point again, and how long the first Sparrow circles its corner.
const REORDER: Tick = 4 * TICKS_PER_SECOND;
const CORNER_TIME: Tick = 15 * TICKS_PER_SECOND;
/// How far an enemy may be for a Disciple to channel Shroud, or a Rocketman to lock on, and how long a Disciple
/// waits between two channels.
const SHROUD_RANGE: f32 = 6.0 * SU_PER_GU;
const LOCK_RANGE: f32 = 12.0 * SU_PER_GU;
const SHROUD_GAP: Tick = 10 * TICKS_PER_SECOND;

/// The fight (module docs): spawned in its first two ticks (the Shrines in the first, the units in the second), then
/// played from its third.
pub(super) struct Fight {
    from: Tick,
    /// Each player, and the player whose extras theirs meet.
    players: Vec<(PlayerId, Faction, PlayerId)>,
    /// The spawns not yet seen in the game: whose, what, where.
    pending: Vec<(PlayerId, ObjectKind, Vec2)>,
    /// Each player's extras and Shrine, once spawned.
    army: BTreeMap<PlayerId, Vec<ObjectId>>,
    shrines: BTreeMap<PlayerId, ObjectId>,
    /// The Sparrow that circles a corner first, and the corner the seed picked.
    corner: Option<ObjectId>,
    corner_pick: u32,
    /// Whether a Prayer-mobile saves its energy for Suppress until it has cast one (the seed's pick), and those that
    /// have.
    suppress_first: bool,
    suppressed: Vec<ObjectId>,
    /// The ability each Shrine casts next.
    next: BTreeMap<ObjectId, AreaAbility>,
    /// How many times each Hand of Heaven has crossed the meeting point.
    crossings: BTreeMap<ObjectId, u32>,
    /// The tick each Disciple last channelled Shroud from.
    channelled: BTreeMap<ObjectId, Tick>,
}

impl Fight {
    pub(super) fn new(players: &[(PlayerId, Faction)], from: Tick, rng: &mut Rng) -> Self {
        let n = players.len();
        let players = (players.iter().enumerate())
            .map(|(i, &(player, faction))| (player, faction, players[(i + 1) % n].0))
            .collect();
        Self {
            from,
            players,
            pending: Vec::new(),
            army: BTreeMap::new(),
            shrines: BTreeMap::new(),
            corner: None,
            corner_pick: rng.below(4),
            suppress_first: rng.below(2) == 1,
            suppressed: Vec::new(),
            next: BTreeMap::new(),
            crossings: BTreeMap::new(),
            channelled: BTreeMap::new(),
        }
    }

    /// The fight's commands for the step to the next tick, each one the gate takes now (module docs).
    pub(super) fn commands(&mut self, sim: &Sim) -> Vec<PlayerCommand> {
        let tick = sim.tick() + 1;
        if tick < self.from {
            return Vec::new();
        }
        let mut commands = Vec::new();
        let players = self.players.clone();
        for &(player, faction, enemy) in &players {
            let (Some(home), Some(theirs)) = (home(sim, player), home(sim, enemy)) else {
                continue;
            };
            let meet = (home + theirs) / 2.0;
            let across = (theirs - home).normalize_or_zero().perp() * 4.0 * SU_PER_GU;
            let mut give = |command: Command| {
                let c = PlayerCommand { player, command };
                let takes = sim.validate(&c).is_ok();
                if takes {
                    commands.push(c);
                }
                takes
            };
            if tick == self.from {
                if faction == Faction::Cults
                    && let Some(origin) = shrine_site(sim, player, home)
                    && give(spawn(Shrine, player, SpawnAt::Tile(origin)))
                {
                    self.pending.push((player, Shrine, Vec2::ZERO));
                }
                continue;
            }
            if tick == self.from + 1 {
                for (kind, pos) in spots(sim, player, home, meet) {
                    if give(spawn(kind, player, SpawnAt::Point(pos))) {
                        self.pending.push((player, kind, pos));
                    }
                }
                continue;
            }
            let army: Vec<&Object> = (self.army.get(&player).into_iter().flatten())
                .filter_map(|&id| sim.object(id))
                .collect();
            let first = tick == self.from + 2;
            let shrine = self.shrines.get(&player).copied();
            let mut attackers = Vec::new();
            let mut charging = Vec::new();
            for o in &army {
                let current = (o.orders.as_ref())
                    .and_then(|orders| orders.current.as_ref())
                    .map(|a| a.order.kind);
                let idle = o.orders.as_ref().is_none_or(|orders| orders.is_idle());
                let corner_ends = self.from + 2 + CORNER_TIME;
                let cornered = self.corner == Some(o.id);
                if o.kind == Recruit {
                    if first {
                        charging.push(o.id);
                    }
                    continue;
                }
                let reorder = idle && (tick - self.from).is_multiple_of(REORDER);
                let waits = o.kind == PrayerMobile && self.suppress_first && !self.suppressed.contains(&o.id);
                if o.kind == HandOfHeaven {
                    if first || reorder {
                        let n = self.crossings.entry(o.id).or_default();
                        let side = [0.0, 1.0, -1.0][(*n as usize).min(1) + (*n as usize % 2)];
                        *n += 1;
                        give(order(o.id, OrderKind::Move(Target::Point(meet + across * side))));
                    }
                } else if waits {
                    // At home until it can cast Suppress, then to the meeting point.
                    let ready = sim.can_cast(player, &[o.id], AreaAbility::Suppress).is_ok();
                    if ready && current.is_none() && (idle || first) {
                        give(order(o.id, OrderKind::Move(Target::Point(meet))));
                    }
                } else if first && self.corner.is_none() && o.kind == Sparrow {
                    self.corner = Some(o.id);
                    give(order(
                        o.id,
                        OrderKind::Move(Target::Point(corner(sim, self.corner_pick))),
                    ));
                } else if cornered && tick < corner_ends {
                    // Circling its corner until then.
                } else if (cornered && tick == corner_ends) || first || reorder {
                    attackers.push(o.id);
                }
                match o.kind {
                    Disciple
                        if current != Some(OrderKind::Shroud)
                            && (self.channelled.get(&o.id)).is_none_or(|&at| tick >= at + SHROUD_GAP)
                            && nearest_enemy(sim, player, o.pos, SHROUD_RANGE).is_some() =>
                    {
                        if give(order(o.id, OrderKind::Shroud)) {
                            self.channelled.insert(o.id, tick);
                        }
                    }
                    Rocketman if !matches!(current, Some(OrderKind::TargetLock(_))) => {
                        if let Some(enemy) = nearest_enemy(sim, player, o.pos, LOCK_RANGE) {
                            give(order(o.id, OrderKind::TargetLock(enemy)));
                        }
                    }
                    PrayerMobile => {
                        let ahead = (sim.objects()).any(|e| {
                            enemy_unit(e, player) && buffs::in_area(AreaAbility::Suppress, o.pos, o.facing, e.pos)
                        });
                        let around = (sim.objects()).any(|u| {
                            u.id != o.id
                                && u.owner == Some(player)
                                && AreaAbility::Inspire.affects(u.kind)
                                && buffs::in_area(AreaAbility::Inspire, o.pos, o.facing, u.pos)
                        });
                        let saving = self.suppress_first && !self.suppressed.contains(&o.id);
                        if ahead && sim.can_cast(player, &[o.id], AreaAbility::Suppress).is_ok() {
                            if give(cast(o.id, AreaAbility::Suppress, o.pos)) {
                                self.suppressed.push(o.id);
                            }
                        } else if !saving && around && sim.can_cast(player, &[o.id], AreaAbility::Inspire).is_ok() {
                            give(cast(o.id, AreaAbility::Inspire, o.pos));
                        }
                    }
                    _ => {}
                }
            }
            if !attackers.is_empty() {
                give(Command::Order {
                    units: attackers,
                    kind: OrderKind::AttackMove(meet),
                    queue: false,
                });
            }
            if let Some(shrine) = shrine {
                if !charging.is_empty() {
                    give(Command::Order {
                        units: charging,
                        kind: OrderKind::Enter(shrine),
                        queue: false,
                    });
                }
                let next = *self.next.entry(shrine).or_insert(AreaAbility::HealthRegeneration);
                let on = (army.iter())
                    .find(|o| o.kind != Recruit && next.affects(o.kind))
                    .map(|o| o.pos);
                if let Some(at) = on
                    && sim.sees_point(player, at)
                    && sim.can_cast(player, &[shrine], next).is_ok()
                    && give(cast(shrine, next, at))
                {
                    let other = match next {
                        AreaAbility::HealthRegeneration => AreaAbility::EnergyRegeneration,
                        _ => AreaAbility::HealthRegeneration,
                    };
                    self.next.insert(shrine, other);
                }
            }
        }
        commands
    }

    /// After the step to `sim.tick()`: the extras spawned in it, by their spots, and what the fight did in it.
    pub(super) fn after(&mut self, sim: &Sim, tally: &mut Tally) {
        if sim.tick() < self.from {
            return;
        }
        for e in sim.events() {
            match *e {
                SimEvent::Spawned(id) => {
                    let Some(o) = sim.object(id) else { continue };
                    let Some(owner) = o.owner else { continue };
                    let found = (self.pending.iter()).position(|&(player, kind, pos)| {
                        player == owner && kind == o.kind && (kind == Shrine || o.pos.distance(pos) < 1.0)
                    });
                    if let Some(i) = found {
                        self.pending.swap_remove(i);
                        if o.kind == Shrine {
                            self.shrines.insert(owner, id);
                        } else {
                            self.army.entry(owner).or_default().push(id);
                        }
                    }
                }
                SimEvent::Cast { ability, .. } => {
                    let i = match ability {
                        AreaAbility::HealthRegeneration => 0,
                        AreaAbility::EnergyRegeneration => 1,
                        AreaAbility::Inspire => 2,
                        AreaAbility::Suppress => 3,
                    };
                    tally.casts[i] += 1;
                }
                SimEvent::ShroudStarted { .. } => tally.shrouds += 1,
                SimEvent::Attacked { .. } => tally.attacks += 1,
                SimEvent::Died { .. } | SimEvent::Crushed { .. } => tally.deaths += 1,
                _ => {}
            }
        }
        if sim.tick() == self.from + 1 {
            assert_eq!(self.pending, [], "tick {}: spawns that didn't come", sim.tick());
            for &(player, faction, _) in &self.players {
                let army = self.army.get(&player).map_or(0, Vec::len);
                let wanted: usize = (EXTRAS.iter())
                    .filter(|&&(f, _)| f == faction)
                    .flat_map(|&(_, extras)| extras.iter().map(|&(_, n)| n))
                    .sum();
                assert_eq!(army, wanted, "player {}'s extras", player.0);
                if faction == Faction::Cults {
                    assert!(self.shrines.contains_key(&player), "player {}'s Shrine", player.0);
                }
            }
        }
    }
}

/// A player's home: their first structure, by id. `None` if they own none.
fn home(sim: &Sim, player: PlayerId) -> Option<Vec2> {
    let structures = (sim.objects()).filter(|o| o.owner == Some(player) && o.def().structure.is_some());
    structures.map(|o| o.pos).next()
}

/// Whether `o` is a unit of an enemy of `player`'s (another player's).
fn enemy_unit(o: &Object, player: PlayerId) -> bool {
    o.owner.is_some_and(|owner| owner != player) && o.def().unit.is_some()
}

/// The enemy unit nearest `at` within `range`, if any.
fn nearest_enemy(sim: &Sim, player: PlayerId, at: Vec2, range: f32) -> Option<ObjectId> {
    let enemies = (sim.objects()).filter(|o| enemy_unit(o, player) && o.pos.distance(at) <= range);
    enemies
        .min_by(|a, b| (a.pos.distance_squared(at)).total_cmp(&b.pos.distance_squared(at)))
        .map(|o| o.id)
}

/// The map corner the seed picked: 0 the north-west, 1 the north-east, 2 the south-west, 3 the south-east, a tile in.
fn corner(sim: &Sim, pick: u32) -> Vec2 {
    let size = Vec2::new(sim.map().width() as f32, sim.map().height() as f32) * SU_PER_GU;
    let x = if pick.is_multiple_of(2) {
        SU_PER_GU
    } else {
        size.x - SU_PER_GU
    };
    let y = if pick < 2 { SU_PER_GU } else { size.y - SU_PER_GU };
    Vec2::new(x, y)
}

/// A debug spawn of `kind` for `owner` at `at`.
fn spawn(kind: ObjectKind, owner: PlayerId, at: SpawnAt) -> Command {
    Command::Debug(DebugCommand::Spawn {
        kind,
        owner: Some(owner),
        at,
        orientation: Default::default(),
    })
}

/// An order to one unit, not queued.
fn order(unit: ObjectId, kind: OrderKind) -> Command {
    Command::Order {
        units: vec![unit],
        kind,
        queue: false,
    }
}

/// A cast of `ability` by `caster` at `at`.
fn cast(caster: ObjectId, ability: AreaAbility, at: Vec2) -> Command {
    Command::Cast {
        casters: vec![caster],
        ability,
        at,
    }
}

/// Where `player`'s Shrine spawns: the first origin the gate takes, nearest their home first, within the extras'
/// reach, whose four diagonal neighbours it takes too, so that a tile all round it is clear.
fn shrine_site(sim: &Sim, player: PlayerId, home: Vec2) -> Option<TilePos> {
    let centre = TilePos::containing(home);
    let span = REACH * 2;
    let mut origins: Vec<TilePos> = ((-span..=span).flat_map(|j| (-span..=span).map(move |i| (i, j))))
        .map(|(i, j)| TilePos::new(centre.x + i, centre.y + j))
        .collect();
    origins.sort_by(|a, b| {
        let d = |t: &TilePos| t.center().distance_squared(home);
        d(a).total_cmp(&d(b)).then((a.y, a.x).cmp(&(b.y, b.x)))
    });
    let takes = |origin: TilePos| {
        let c = PlayerCommand {
            player,
            command: spawn(Shrine, player, SpawnAt::Tile(origin)),
        };
        sim.validate(&c).is_ok()
    };
    origins.into_iter().find(|&origin| {
        takes(origin)
            && [(-1, -1), (1, 1), (-1, 1), (1, -1)]
                .iter()
                .all(|&(dx, dy)| takes(TilePos::new(origin.x + dx, origin.y + dy)))
    })
}

/// The spots `player`'s extras spawn at (their faction's [`EXTRAS`], in order): points of a lattice of [`STEP`]
/// round their `home`, at least [`CLEAR`] from it, nearest `toward` first, each the first the gate takes for its kind,
/// and none within a step of another.
fn spots(sim: &Sim, player: PlayerId, home: Vec2, toward: Vec2) -> Vec<(ObjectKind, Vec2)> {
    let Some(faction) = sim.player(player).map(|p| p.faction) else {
        return Vec::new();
    };
    let kinds = (EXTRAS.iter())
        .filter(|&&(f, _)| f == faction)
        .flat_map(|&(_, extras)| extras.iter().flat_map(|&(kind, n)| std::iter::repeat_n(kind, n)));
    let mut lattice: Vec<Vec2> = ((-REACH..=REACH).flat_map(|j| (-REACH..=REACH).map(move |i| (i, j))))
        .map(|(i, j)| home + Vec2::new(i as f32, j as f32) * STEP)
        .filter(|p| p.distance(home) >= CLEAR && sim.map().contains_point(*p))
        .collect();
    lattice.sort_by(|a, b| (a.distance_squared(toward)).total_cmp(&b.distance_squared(toward)));
    let mut taken: Vec<(ObjectKind, Vec2)> = Vec::new();
    for kind in kinds {
        let fits = |p: Vec2| {
            let c = PlayerCommand {
                player,
                command: spawn(kind, player, SpawnAt::Point(p)),
            };
            sim.validate(&c).is_ok()
        };
        let free = (lattice.iter().copied()).find(|&p| taken.iter().all(|&(_, q)| q.distance(p) >= STEP) && fits(p));
        if let Some(p) = free {
            taken.push((kind, p));
        }
    }
    taken
}
