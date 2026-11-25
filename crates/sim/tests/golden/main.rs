//! Golden hashes (`ARCHITECTURE.md > Replays and golden hashes`): nine small games, defined below and in
//! `colonists.rs` and recorded as replays in `tests/golden/`, each with the state hash at tick 0, every 32 ticks and
//! at its end (`harness.rs`). A run must give the blessed hashes: a change that alters how any of these games plays
//! out fails here, at the first checkpoint that differs. The replays play in the client too (`cargo run -- --replay
//! crates/sim/tests/golden/<name>.json`).
//!
//! When the change is intended, re-bless the files and review the diff (the checkpoint hashes) before committing:
//!
//! ```sh
//! SC_BLESS=1 cargo test -p sc_sim golden
//! ```

mod colonists;
mod harness;

use std::collections::BTreeMap;

use sc_sim::testing::ScenarioBuilder;
use sc_sim::testing::build_order::{Books, BuildOrder, Order};
use sc_sim::{
    Ability, Command, Faction, ObjectId, ObjectKind, OrderKind, PlayerCommand, PlayerId, ProductionCommand, SU_PER_GU,
    Scenario, ShieldMode, Sim, SimEvent, SpawnAt, Target, Tick, TilePos, Vec2, riot_shield, scenarios,
};

use harness::{Game, Script, golden};

const P1: PlayerId = PlayerId(1);
const P2: PlayerId = PlayerId(2);
const PK: ObjectKind = ObjectKind::Peacekeeper;
const GUARD: ObjectKind = ObjectKind::Guard;

/// Open Plane ground, `w` × `h` tiles.
fn open(w: usize, h: usize) -> String {
    format!("{}\n", ".".repeat(w)).repeat(h)
}

/// A unit's spawn at the centre of tile `(x, y)`.
fn at(x: i32, y: i32) -> SpawnAt {
    SpawnAt::Point(TilePos::new(x, y).center())
}

fn point(x: i32, y: i32) -> Vec2 {
    TilePos::new(x, y).center()
}

fn order(player: PlayerId, units: impl IntoIterator<Item = u32>, kind: OrderKind) -> PlayerCommand {
    PlayerCommand {
        player,
        command: Command::Order {
            units: units.into_iter().map(ObjectId).collect(),
            kind,
            queue: false,
        },
    }
}

/// Player 2 plays the Syndicate, whose unit the Guard is.
fn syndicate_p2(builder: ScenarioBuilder) -> Scenario {
    let mut scenario = builder.build();
    scenario.setup.players[1].faction = Faction::Syndicate;
    scenario
}

/// Player 1's `units` raise (`raise`) or lower their Riot Shields.
fn shield(units: impl IntoIterator<Item = u32>, raise: bool) -> PlayerCommand {
    PlayerCommand {
        player: P1,
        command: Command::Ability {
            units: units.into_iter().map(ObjectId).collect(),
            ability: Ability::RiotShield { raise },
        },
    }
}

fn units_of(sim: &Sim, player: PlayerId) -> Vec<&sc_sim::Object> {
    sim.objects().filter(|o| o.owner == Some(player)).collect()
}

/// Three Peacekeepers (player 1, ids 1–3) against three Guards (player 2, ids 4–6), in two columns 4 tiles apart:
/// the Peacekeepers are ordered to attack the middle Guard, the Guards acquire their targets by themselves. It ends
/// with one side gone.
fn duel_3v3() -> Game {
    let mut builder = ScenarioBuilder::new(&open(16, 10));
    for y in [3, 5, 7] {
        builder = builder.spawn(PK, Some(P1), at(5, y));
    }
    for y in [3, 5, 7] {
        builder = builder.spawn(GUARD, Some(P2), at(9, y));
    }
    Game {
        scenario: syndicate_p2(builder),
        commands: vec![(1, order(P1, 1..=3, OrderKind::Attack(ObjectId(5))))],
        script: None,
        end: 240,
        during: |_, _| {},
        expect: |sim| {
            let (p1, p2) = (units_of(sim, P1).len(), units_of(sim, P2).len());
            assert!(p1 == 0 || p2 == 0, "one side is gone: {p1} against {p2}");
        },
    }
}

/// Twenty units a side, five rows of Peacekeepers and Guards in turn, in blocks at opposite ends of the map, are
/// ordered to AttackMove to the other block's place: they meet in the middle and fight.
fn attack_move_battle() -> Game {
    let mut builder = ScenarioBuilder::new(&open(40, 20));
    for (player, x0, y0) in [(P1, 3, 6), (P2, 33, 9)] {
        for y in 0..5 {
            for x in 0..4 {
                let kind = if y % 2 == 0 { PK } else { GUARD };
                builder = builder.spawn(kind, Some(player), at(x0 + x, y0 + y));
            }
        }
    }
    Game {
        scenario: syndicate_p2(builder),
        commands: vec![
            (1, order(P1, 1..=20, OrderKind::AttackMove(point(35, 11)))),
            (1, order(P2, 21..=40, OrderKind::AttackMove(point(4, 8)))),
        ],
        script: None,
        end: 480,
        during: |_, _| {},
        expect: |sim| {
            let left = sim.objects().count();
            assert!(left <= 30, "at least ten died: {left} are left");
        },
    }
}

/// Twelve Peacekeepers in a block west of a mountain wall, ordered as one group to a point east of it, cross it
/// through a gap two tiles wide.
fn choke_group_move() -> Game {
    let wall = format!("{}M{}\n", ".".repeat(15), ".".repeat(16));
    let gap = open(32, 1);
    let rows: Vec<&str> = (0..16)
        .map(|y| if y == 7 || y == 8 { &gap } else { &wall })
        .map(String::as_str)
        .collect();
    let mut builder = ScenarioBuilder::new(&rows.concat());
    for y in 6..=8 {
        for x in 3..=6 {
            builder = builder.spawn(PK, Some(P1), at(x, y));
        }
    }
    Game {
        scenario: builder.build(),
        commands: vec![(1, order(P1, 1..=12, OrderKind::Move(Target::Point(point(26, 8)))))],
        script: None,
        end: 480,
        during: |_, _| {},
        expect: |sim| {
            let wall_east = 16.0 * SU_PER_GU;
            for unit in units_of(sim, P1) {
                assert!(
                    unit.pos.x > wall_east,
                    "#{} is through the gap: {}",
                    unit.id.0,
                    unit.pos
                );
            }
        },
    }
}

/// Three Peacekeepers (ids 1–3) patrol between their place and a point 14 tiles east. A Guard of player 2 (id 4)
/// stands near the route: the patrol engages it, kills it and walks on.
fn patrol() -> Game {
    let mut builder = ScenarioBuilder::new(&open(32, 12));
    for y in [4, 5, 6] {
        builder = builder.spawn(PK, Some(P1), at(4, y));
    }
    builder = builder.spawn(GUARD, Some(P2), at(14, 9));
    Game {
        scenario: syndicate_p2(builder),
        commands: vec![(1, order(P1, 1..=3, OrderKind::Patrol(point(18, 5))))],
        script: None,
        end: 640,
        during: |_, _| {},
        expect: |sim| {
            assert!(sim.object(ObjectId(4)).is_none(), "the Guard is dead");
            let patrolling = units_of(sim, P1)
                .iter()
                .filter(|u| {
                    let current = u.orders.as_ref().and_then(|o| o.current.as_ref());
                    current.is_some_and(|c| matches!(c.order.kind, OrderKind::Patrol(_)))
                })
                .count();
            assert!(patrolling > 0, "the patrol goes on");
        },
    }
}

/// A Peacekeeper (player 1, id 1) attacks a Guard (player 2, id 2) that runs away east, faster than it: the
/// Peacekeeper shoots while it can, chases, loses the Guard in the fog, walks to where it last saw it and stops
/// there (design change 0034).
fn fog_chase() -> Game {
    let builder = ScenarioBuilder::new(&open(48, 10))
        .spawn(PK, Some(P1), at(4, 5))
        .spawn(GUARD, Some(P2), at(8, 5));
    Game {
        scenario: syndicate_p2(builder),
        commands: vec![
            (1, order(P2, [2], OrderKind::Move(Target::Point(point(44, 5))))),
            (1, order(P1, [1], OrderKind::Attack(ObjectId(2)))),
        ],
        script: None,
        end: 400,
        during: |_, _| {},
        expect: |sim| {
            let (chaser, runner) = (sim.object(ObjectId(1)).unwrap(), sim.object(ObjectId(2)).unwrap());
            assert!(runner.hp < Some(80.0), "the chaser hit it before it got away");
            assert!(!sim.is_visible(P1, runner), "out of the chaser's sight");
            assert!(chaser.orders.as_ref().unwrap().is_idle(), "the chase ended");
            assert!(
                chaser.pos.x > 9.0 * SU_PER_GU,
                "it chased past the Guard's start: {}",
                chaser.pos
            );
        },
    }
}

/// Player 1 raises the Riot Shields of three Peacekeepers (ids 1–3, `Command::Ability`) and walks them, at half
/// speed, up to a Guard (player 2, id 4), which shoots at them: behind their shields they don't shoot back. On the
/// way #3 lowers its shield and raises it again before it is down. At tick [`SHIELDS_DOWN`] all three lower their
/// shields, and then they kill the Guard; with its laser rifle (design change 0186) it takes #2 with it.
fn riot_shield() -> Game {
    let mut builder = ScenarioBuilder::new(&open(16, 10));
    for y in [4, 5, 6] {
        builder = builder.spawn(PK, Some(P1), at(3, y));
    }
    builder = builder.spawn(GUARD, Some(P2), at(11, 5));
    Game {
        scenario: syndicate_p2(builder),
        commands: vec![
            (1, shield(1..=3, true)),
            (1, order(P1, 1..=3, OrderKind::Move(Target::Point(point(8, 5))))),
            (30, shield([3], false)),
            (32, shield([3], true)),
            (SHIELDS_DOWN, shield(1..=3, false)),
        ],
        script: None,
        end: 240,
        during: |tick, sim| {
            if tick == SHIELDS_DOWN - 1 {
                let guard = sim.object(ObjectId(4)).expect("the Guard lives");
                assert_eq!(guard.hp, Some(80.0), "the shielded Peacekeepers didn't shoot");
                let units = units_of(sim, P1);
                assert!(units.iter().any(|u| u.hp < Some(50.0)), "the Guard shot at them");
                for unit in units {
                    assert_eq!(unit.kind, ObjectKind::PeacekeeperShielded, "#{}", unit.id.0);
                    assert_eq!(riot_shield::mode(unit), Some(ShieldMode::Raised), "#{}", unit.id.0);
                }
            }
        },
        expect: |sim| {
            assert!(sim.object(ObjectId(4)).is_none(), "the Guard is dead");
            let units = units_of(sim, P1);
            let ids: Vec<u32> = units.iter().map(|unit| unit.id.0).collect();
            assert_eq!(ids, [1, 3], "the Guard killed #2");
            for unit in units {
                assert_eq!(unit.kind, PK, "#{}", unit.id.0);
                assert_eq!(riot_shield::mode(unit), Some(ShieldMode::Lowered), "#{}", unit.id.0);
            }
        },
    }
}

/// [`riot_shield`]: the Peacekeepers lower their shields in this tick.
const SHIELDS_DOWN: Tick = 120;

/// `dev_skirmish` from its real start, both players GDO, playing by script ([`Opening`]; slice M2-J1). Player 1 plays
/// the fastest clean Barracks (Plate, Generator, Generator, Barracks: `tests/economy_oracle/`), trains ten
/// Peacekeepers and rallies them to the edge of player 2's crystal field ([`RALLY`]), where they shoot what they find:
/// player 2's plates first. Player 2 plates its field and builds Generators. Both players' books balance every step;
/// at least one of player 2's plates is destroyed, its patch is mined no more and player 2's income falls; player 2
/// keeps its other structures, so it isn't defeated and the game goes on (`core/game_rules.md > Defeat Condition`) to
/// [`GDO_OPENING_END`].
fn gdo_opening() -> Game {
    let p1 = BuildOrder::new(P1).track(
        [
            Order::new(EXTRACTION_FACILITY, PLATE),
            Order::new(DEPLOYMENT_CENTER, GENERATOR),
            Order::new(DEPLOYMENT_CENTER, GENERATOR),
            Order::new(DEPLOYMENT_CENTER, BARRACKS),
        ]
        .into_iter()
        .chain([Order::new(BARRACKS, PK); 10]),
    );
    let p2 = BuildOrder::new(P2)
        .track([Order::new(EXTRACTION_FACILITY, PLATE); 6])
        .track([Order::new(DEPLOYMENT_CENTER, GENERATOR); 6]);
    Game {
        scenario: scenarios::dev_skirmish(),
        commands: Vec::new(),
        script: Some(Box::new(Opening {
            p1,
            p2,
            rallied: false,
            open: Vec::new(),
            plates: BTreeMap::new(),
            lost: BTreeMap::new(),
            destroyed: Vec::new(),
            mined: Vec::new(),
        })),
        end: GDO_OPENING_END,
        during: |_, sim| {
            assert!(sim.outcome().is_none(), "tick {}: the game ended", sim.tick());
        },
        expect: |sim| {
            let p2 = sim.player(P2).unwrap();
            assert!(!p2.is_defeated(), "player 2 isn't defeated");
            let structures = sim
                .objects()
                .filter(|o| o.owner == Some(P2) && o.def().structure.is_some())
                .count();
            assert!(structures > 0, "player 2 keeps structures");
            let peacekeepers = units_of(sim, P1).iter().filter(|o| o.kind == PK).count();
            assert!(peacekeepers > 0, "player 1's Peacekeepers");
        },
    }
}

const EXTRACTION_FACILITY: ObjectKind = ObjectKind::ExtractionFacility;
const DEPLOYMENT_CENTER: ObjectKind = ObjectKind::DeploymentCenter;
const PLATE: ObjectKind = ObjectKind::ExtractionPlate;
const GENERATOR: ObjectKind = ObjectKind::Generator;
const BARRACKS: ObjectKind = ObjectKind::Barracks;
/// [`gdo_opening`]'s last tick, 2:45: the Peacekeepers have destroyed four plates, and the Deployment Center, which
/// they shoot next, still stands.
const GDO_OPENING_END: Tick = 2640;
/// [`gdo_opening`]: player 1's rally point, on open ground at the south-east corner of player 2's crystal field
/// (`dev_skirmish`'s field of the second slot: patches (56, 45) to (59, 45), (60, 44), (61, 40) to (61, 42)).
const RALLY: TilePos = TilePos::new(60, 46);

/// [`gdo_opening`]'s players.
struct Opening {
    /// Player 1's line: the fastest clean Barracks, then ten Peacekeepers.
    p1: BuildOrder,
    /// Player 2's: its field plated, and Generators.
    p2: BuildOrder,
    /// Whether player 1 has set its Barracks' rally point.
    rallied: bool,
    /// The books of the step under way, each player's, with what its orders paid.
    open: Vec<(PlayerId, Books, u32)>,
    /// Player 2's plates, with the patch each covers.
    plates: BTreeMap<ObjectId, ObjectId>,
    /// The patches of player 2's destroyed plates that no plate covers again, with the tick their plate was
    /// destroyed in and the crystals they held then.
    lost: BTreeMap<ObjectId, (Tick, u32)>,
    /// Each of player 2's plates destroyed: the tick, and its patch.
    destroyed: Vec<(Tick, ObjectId)>,
    /// What player 2's plates mined, step by step: `mined[i]` in tick `i + 1`.
    mined: Vec<u32>,
}

impl Opening {
    /// Closes the books of the step that just ran, which must balance.
    fn close(&mut self, sim: &Sim) {
        for (player, books, spent) in std::mem::take(&mut self.open) {
            let entry = books.close(sim, spent);
            if player == P2 {
                self.mined.push(entry.mined);
            }
        }
    }
}

/// The crystals patch `id` holds, if it is one.
fn remaining(sim: &Sim, id: ObjectId) -> Option<u32> {
    match sim.object(id)?.resource.as_ref()? {
        sc_sim::ResourceState::CrystalPatch { remaining } => Some(*remaining),
        _ => None,
    }
}

impl Script for Opening {
    fn commands(&mut self, sim: &Sim) -> Vec<PlayerCommand> {
        self.close(sim);
        // A destroyed plate's patch stays, uncovered, with what it held (`factions/gdo/economy.md > ExtractionPlate >
        // On destruction`), and nothing mines it until a plate covers it again.
        for (&patch, &(since, left)) in &self.lost {
            assert_eq!(
                remaining(sim, patch),
                Some(left),
                "tick {}: patch #{}, whose plate was destroyed in tick {since}",
                sim.tick(),
                patch.0
            );
        }
        for e in sim.events() {
            if let SimEvent::Died {
                id,
                kind: PLATE,
                owner: Some(P2),
                ..
            } = e
            {
                let patch = self.plates[id];
                let left = remaining(sim, patch)
                    .filter(|&n| n > 0)
                    .expect("the patch stays, uncovered");
                self.lost.insert(patch, (sim.tick(), left));
                self.destroyed.push((sim.tick(), patch));
            }
        }
        self.plates = sim
            .objects()
            .filter(|o| o.owner == Some(P2))
            .filter_map(|o| Some((o.id, o.extractor?.patch)))
            .collect();
        let covered: Vec<ObjectId> = self.plates.values().copied().collect();
        self.lost.retain(|patch, _| !covered.contains(patch));

        let tick = sim.tick() + 1;
        let mut commands = self.p1.commands(sim);
        if !self.rallied
            && let Some(barracks) = sim.objects().find(|o| o.owner == Some(P1) && o.kind == BARRACKS)
        {
            commands.push(PlayerCommand {
                player: P1,
                command: Command::Production {
                    objects: vec![barracks.id],
                    command: ProductionCommand::SetRally(Target::Point(RALLY.center())),
                },
            });
            self.rallied = true;
        }
        commands.extend(self.p2.commands(sim));
        let paid = |line: &BuildOrder| line.issued().iter().filter(|i| i.tick == tick).map(|i| i.cost).sum();
        self.open = vec![
            (P1, Books::open(sim, P1), paid(&self.p1)),
            (P2, Books::open(sim, P2), paid(&self.p2)),
        ];
        commands
    }

    fn end(mut self: Box<Self>, sim: &Sim) {
        self.close(sim);
        let trained = self.p1.issued().iter().filter(|i| i.order.kind == PK).count();
        assert_eq!(trained, 10, "player 1 trained ten Peacekeepers");
        let Some(&(first, _)) = self.destroyed.first() else {
            panic!("player 1 destroyed none of player 2's plates");
        };
        // Player 2's income: the 9 seconds (3 cycles) before the first plate was lost, and the last 9.
        let window = |end: Tick| -> u32 { self.mined[(end - 144) as usize..end as usize].iter().sum() };
        let (before, last) = (window(first - 1), window(sim.tick()));
        assert!(
            last < before,
            "player 2 mined {last} in the last 9 s, {before} before it lost a plate"
        );
    }
}

#[test]
fn golden_gdo_opening() {
    golden("gdo_opening", gdo_opening);
}

#[test]
fn golden_duel_3v3() {
    golden("duel_3v3", duel_3v3);
}

#[test]
fn golden_attack_move_battle() {
    golden("attack_move_battle", attack_move_battle);
}

#[test]
fn golden_choke_group_move() {
    golden("choke_group_move", choke_group_move);
}

#[test]
fn golden_patrol() {
    golden("patrol", patrol);
}

#[test]
fn golden_fog_chase() {
    golden("fog_chase", fog_chase);
}

#[test]
fn golden_riot_shield() {
    golden("riot_shield", riot_shield);
}

#[test]
fn golden_colonists_opening() {
    golden("colonists_opening", colonists::opening);
}

#[test]
fn golden_colonists_vs_gdo() {
    golden("colonists_vs_gdo", colonists::versus_gdo);
}
