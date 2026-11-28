//! Which grids a game has (slice M5-Z2, the module docs of `paths`, *Which grids*): those of the units in it, each
//! built as the first unit that walks on it comes, and from then on what it would have been from the start, in the
//! true grids and in each player's; and, the grids being derived, a save loads with those of the units it has.

use glam::Vec2;

use crate::augmentation::RecruitKind;
use crate::command::{Command, DebugCommand, PlayerCommand};
use crate::defs::test_defs::TestDef;
use crate::defs::{ObjectKind, UnitBase};
use crate::events::SimEvent;
use crate::ids::{ObjectId, PlayerId};
use crate::mover::MoveStatus;
use crate::nav::{Nav, NavClass};
use crate::orders::{OrderKind, Target};
use crate::player::Faction;
use crate::production::ProductionCommand;
use crate::scenario::SpawnAt;
use crate::sim::{Reject, Sim};
use crate::testing::{ScenarioBuilder, TestSim};
use crate::units::{Tick, TilePos};

const P1: PlayerId = PlayerId(1);
const P2: PlayerId = PlayerId(2);
const PK: ObjectKind = ObjectKind::Peacekeeper;
/// A wheeled vehicle (`systems/units.md > WheeledVehicle`).
const SEEKER: ObjectKind = ObjectKind::Seeker;
/// The infantry's grid, the Peacekeeper's.
const INFANTRY: (NavClass, u8) = (NavClass::GroundRugged, 1);
/// The wheeled and tracked vehicles' grid: they don't cross Rugged terrain.
const VEHICLES: (NavClass, u8) = (NavClass::Ground, 1);
/// The tick the game's first vehicle comes in ([`game`]).
const SPAWN: Tick = 30;
/// That vehicle, player 1's Seeker.
const VEHICLE: ObjectId = ObjectId(4);
/// Where it comes in, west of the Rugged block, and where it is sent, east of it and of player 2's Generator.
const START: TilePos = TilePos::new(2, 5);
const GOAL: TilePos = TilePos::new(21, 5);
/// Player 2's Generator, `(14, 4)` to `(15, 5)`, which player 1 has never seen.
const GENERATOR: TilePos = TilePos::new(14, 4);

/// 24 × 12 tiles, a Rugged block x 6..10 and y 2..8 in the west: the infantry crosses it, a vehicle goes around.
fn map() -> String {
    let row = |y| match y {
        2..8 => "......rrrr..............",
        _ => "........................",
    };
    (0..12).map(row).collect::<Vec<_>>().join("\n")
}

fn centre(x: i32, y: i32) -> SpawnAt {
    SpawnAt::Point(TilePos::new(x, y).center())
}

fn go_to(tile: TilePos) -> OrderKind {
    OrderKind::Move(Target::Point(tile.center()))
}

/// Player 2's Generator (1); a Peacekeeper of each player (2, 3), far apart and far from it, so that both players
/// have their grids; and, if `vehicle`, player 1's Seeker (4), the game's first vehicle, spawned in tick [`SPAWN`] at
/// [`START`] and sent in that tick to [`GOAL`].
fn game(vehicle: bool) -> TestSim {
    let mut t = ScenarioBuilder::new(&map())
        .spawn(ObjectKind::Generator, Some(P2), SpawnAt::Tile(GENERATOR)) // 1
        .spawn(PK, Some(P1), centre(1, 10)) // 2
        .spawn(PK, Some(P2), centre(22, 10)) // 3
        .start();
    t.without_auto_targeting();
    if vehicle {
        t.spawn_unit_at(SPAWN, SEEKER, P1, START.center())
            .order_at(SPAWN, P1, &[VEHICLE], go_to(GOAL), false);
    }
    t
}

fn keys(nav: &Nav) -> Vec<(NavClass, u8)> {
    nav.grids().iter().map(|grid| (grid.class(), grid.size())).collect()
}

/// The sets of grids of `sim`: the true ones, then player 1's and player 2's (both have a unit that walks).
fn sets(sim: &Sim) -> [&Nav; 3] {
    let known = |player| sim.known_nav(player).expect("a player with a unit that walks");
    [sim.nav(), known(P1), known(P2)]
}

/// Whether every set of grids of `sim` holds the grids of `expected`, and no other.
fn has_grids(sim: &Sim, expected: &[(NavClass, u8)]) -> bool {
    sets(sim).iter().all(|nav| keys(nav) == expected)
}

/// Each grid `sim` has is the grid of its key that `other` has, in the true grids and in each player's, tile for tile.
fn same_grids(sim: &Sim, other: &Sim, when: &str) {
    for (set, (nav, other)) in sets(sim).into_iter().zip(sets(other)).enumerate() {
        for grid in nav.grids() {
            let key = (grid.class(), grid.size());
            assert!(other.grid(key.0, key.1) == Some(grid), "{when}: set {set}, {key:?}");
        }
    }
}

fn path(sim: &Sim, unit: ObjectId) -> Option<Vec<Vec2>> {
    let mover = sim.object(unit)?.mover.as_ref()?;
    Some(mover.path.clone())
}

/// Without a vehicle, no grid of the vehicles' class, true or known, however long the game runs: a set holds the
/// grids of the units in the game, here the Peacekeepers'.
#[test]
fn a_game_without_a_vehicle_has_no_grid_of_theirs() {
    let mut t = game(false);
    assert!(has_grids(t.sim(), &[INFANTRY]));
    for _ in 0..60 {
        t.step();
        assert!(has_grids(t.sim(), &[INFANTRY]), "tick {}", t.sim().tick());
    }
}

/// The plan's acceptance for the late grids: a game whose first vehicle comes in tick [`SPAWN`] by a debug spawn,
/// against the same game with every grid built from tick 0. The vehicles' grid exists from the vehicle's tick on, in
/// the true grids and in each player's, and is then the eager game's tile for tile: the true footprints in the true
/// one, each player's in its own (player 1's without the Generator it has never seen); and the two games play the
/// same, the vehicle's path tick for tick, as it goes around the block and learns of the Generator on its way.
#[test]
fn a_grid_built_with_the_first_vehicle_is_the_one_built_from_the_start() {
    let mut t = game(true);
    let mut eager = t.replay().start();
    eager.build_every_grid();
    assert!(
        sets(&eager)
            .iter()
            .all(|nav| nav.grid(VEHICLES.0, VEHICLES.1).is_some())
    );
    while t.sim().tick() < 600 {
        t.step();
        let tick = t.sim().tick();
        eager.step(&t.replay().inputs[tick as usize - 1]);
        let lazy = t.sim();
        let expected: &[_] = if tick < SPAWN {
            &[INFANTRY]
        } else {
            &[VEHICLES, INFANTRY]
        };
        assert!(has_grids(lazy, expected), "tick {tick}");
        same_grids(lazy, &eager, &format!("tick {tick}"));
        assert_eq!(lazy.state_hash(), eager.state_hash(), "tick {tick}");
        assert_eq!(path(lazy, VEHICLE), path(&eager, VEHICLE), "tick {tick}");
        if tick == SPAWN {
            let [truth, own, _] = sets(lazy).map(|nav| nav.grid(VEHICLES.0, VEHICLES.1).unwrap());
            assert!(truth.is_blocked(GENERATOR) && !own.is_blocked(GENERATOR));
            assert!(own.is_blocked(TilePos::new(7, 4)), "Rugged ground");
        }
    }
    assert_eq!(t.status(VEHICLE.0), MoveStatus::Arrived);
    assert_eq!(t.pos(VEHICLE.0), GOAL.center());
    assert!(
        t.sim().path_knowledge().version(P1) > Some(0),
        "player 1 saw the Generator"
    );
}

/// The same for a vehicle a producer makes, a test def's Sabre Legionnaire, wheeled, from player 1's Bio Portal (an
/// energy portal, whose way out is checked on the unit's grid, `Sim::release_unit`): its grid comes with its entry in
/// the queue, true and known, before the unit comes out onto it, and is the eager game's from then on; and the two
/// games play the same as it comes out and walks to the rally point, around the block.
#[test]
fn a_produced_vehicle_has_its_grid_from_its_queueing_on() {
    let _vehicle = TestDef::unit(ObjectKind::SabreLegionnaire, |unit| {
        unit.base = UnitBase::WheeledVehicle
    });
    let (portal, queued) = (ObjectId(4), 5);
    let mut t = ScenarioBuilder::new(&map())
        .spawn(ObjectKind::Generator, Some(P2), SpawnAt::Tile(GENERATOR)) // 1
        .spawn(PK, Some(P1), centre(1, 10)) // 2
        .spawn(PK, Some(P2), centre(22, 10)) // 3
        .spawn(ObjectKind::BioPortal, Some(P1), SpawnAt::Tile(TilePos::new(1, 4))) // 4: (1, 4) to (3, 6)
        .start();
    t.without_auto_targeting();
    let crystals = DebugCommand::SetResources {
        player: P1,
        crystals: 1000,
        supplies: 0,
        alloys: 0,
        essence: 0,
        flux: 0,
    };
    let production = |command| Command::Production {
        objects: vec![portal],
        command,
    };
    t.debug_at(1, crystals)
        .at(
            queued,
            P1,
            production(ProductionCommand::SetRally(Target::Point(GOAL.center()))),
        )
        .at(
            queued,
            P1,
            production(ProductionCommand::Train(ObjectKind::SabreLegionnaire)),
        );
    let mut eager = t.replay().start();
    eager.build_every_grid();
    let mut produced = None;
    while t.sim().tick() < 900 {
        t.step();
        let tick = t.sim().tick();
        eager.step(&t.replay().inputs[tick as usize - 1]);
        let lazy = t.sim();
        let expected: &[_] = if tick < queued {
            &[INFANTRY]
        } else {
            &[VEHICLES, INFANTRY]
        };
        assert!(has_grids(lazy, expected), "tick {tick}");
        same_grids(lazy, &eager, &format!("tick {tick}"));
        assert_eq!(lazy.state_hash(), eager.state_hash(), "tick {tick}");
        produced = produced.or_else(|| {
            t.events().iter().find_map(|(_, event)| match *event {
                SimEvent::UnitProduced { producer, unit } if producer == portal => Some(unit),
                _ => None,
            })
        });
        if let Some(unit) = produced {
            assert_eq!(path(lazy, unit), path(&eager, unit), "tick {tick}");
        }
    }
    let unit = produced.expect("the vehicle came out");
    assert_eq!(t.sim().object(unit).map(|o| o.kind), Some(ObjectKind::SabreLegionnaire));
    assert_eq!(t.status(unit.0), MoveStatus::Arrived);
    assert_eq!(t.pos(unit.0), GOAL.center());
}

/// Determinism: the grids are derived, and a save loads with those of the units in it (and of those about to come
/// in). A save before the first vehicle, in its tick, while it is in the game, and after it left (the running game
/// keeps its grid, and the loaded one builds it again with the next vehicle) loads equal, with the same hash, each
/// grid it has the running game's; and plays on tick for tick like the game it was taken from, the vehicles' paths
/// included.
#[test]
fn a_save_before_or_after_the_first_vehicle_loads_and_plays_on_the_same() {
    const GONE: Tick = 200;
    const AGAIN: Tick = 260;
    const SECOND: ObjectId = ObjectId(5);
    let game = || {
        let mut t = game(true);
        t.debug_at(GONE, DebugCommand::Remove { id: VEHICLE })
            .spawn_unit_at(AGAIN, SEEKER, P1, START.center())
            .order_at(AGAIN, P1, &[SECOND], go_to(GOAL), false);
        t
    };
    let mut t = game();
    for k in [5, SPAWN - 1, SPAWN, 120, GONE, GONE + 20, AGAIN, AGAIN + 60] {
        t.run_until(k);
        let json = serde_json::to_string(t.sim()).unwrap();
        let mut loaded: Sim = serde_json::from_str(&json).unwrap();
        assert_eq!(&loaded, t.sim(), "tick {k}");
        assert_eq!(loaded.state_hash(), t.sim().state_hash(), "tick {k}");
        let vehicle_in = (SPAWN..GONE).contains(&k) || k >= AGAIN;
        let expected: &[_] = if vehicle_in { &[VEHICLES, INFANTRY] } else { &[INFANTRY] };
        assert!(has_grids(&loaded, expected), "tick {k}");
        let kept: &[_] = if k < SPAWN { &[INFANTRY] } else { &[VEHICLES, INFANTRY] };
        assert!(has_grids(t.sim(), kept), "tick {k}: a grid is never dropped");
        same_grids(&loaded, t.sim(), &format!("loaded at {k}"));
        let mut twin = game();
        twin.run_until(k);
        for _ in 0..90 {
            twin.step();
            let tick = twin.sim().tick();
            loaded.step(&twin.replay().inputs[tick as usize - 1]);
            let when = format!("from tick {k}, at {tick}");
            assert_eq!(loaded.state_hash(), twin.sim().state_hash(), "{when}");
            for unit in [VEHICLE, SECOND] {
                assert_eq!(path(&loaded, unit), path(twin.sim(), unit), "{when}: {unit:?}");
            }
            same_grids(&loaded, twin.sim(), &when);
        }
    }
}

/// A held unit has its grid for the spot it comes out at (`docs/architecture/object-model-holding.md > Object model > Units off the map: the held
/// store`): the game's one vehicle, a test def's wheeled Recruit, inside player 1's Armory, and a save taken while it is
/// inside loads with its grid, though no unit on the map walks on it; Released, it comes out at the Armory's door the
/// same in the loaded game as in the game it was taken from.
#[test]
fn a_held_vehicle_keeps_its_grid_and_a_save_loads_it() {
    const ARMORY: ObjectId = ObjectId(1);
    const RECRUIT: ObjectId = ObjectId(2);
    const INSIDE: Tick = 61;
    let _vehicle = TestDef::unit(ObjectKind::Recruit, |unit| unit.base = UnitBase::WheeledVehicle);
    let game = || {
        // The Armory on tiles (12, 10) to (14, 11), its entrance facing east, the Recruit east of it.
        let mut t = ScenarioBuilder::new(&"..............................\n".repeat(22))
            .faction(P1, Faction::Cults)
            .spawn(ObjectKind::Armory, Some(P1), SpawnAt::Tile(TilePos::new(12, 10))) // 1
            .spawn(ObjectKind::Recruit, Some(P1), SpawnAt::Point(Vec2::new(1050.0, 704.0))) // 2
            .start();
        let release = Command::Production {
            objects: vec![ARMORY],
            command: ProductionCommand::Release(RecruitKind::Plain),
        };
        t.without_auto_targeting()
            .order_at(1, P1, &[RECRUIT], OrderKind::Enter(ARMORY), false)
            .at(INSIDE + 1, P1, release);
        t
    };
    let mut t = game();
    t.run_until(INSIDE);
    assert!(t.sim().object(RECRUIT).is_none(), "inside");
    let json = serde_json::to_string(t.sim()).unwrap();
    let mut loaded: Sim = serde_json::from_str(&json).unwrap();
    assert_eq!(loaded.state_hash(), t.sim().state_hash());
    assert!(loaded.nav().grid(VEHICLES.0, VEHICLES.1) == t.sim().nav().grid(VEHICLES.0, VEHICLES.1));
    assert!(
        loaded.nav().grid(VEHICLES.0, VEHICLES.1).is_some(),
        "built for the held vehicle"
    );
    for _ in 0..120 {
        t.step();
        let tick = t.sim().tick();
        loaded.step(&t.replay().inputs[tick as usize - 1]);
        assert_eq!(loaded.state_hash(), t.sim().state_hash(), "tick {tick}");
    }
    assert!(loaded.object(RECRUIT).is_some(), "out");
}

/// The gate asked about a debug spawn of the first vehicle (`Sim::validate`, as the client asks) checks its spot on
/// the grid it would walk on, made for the answer: a wheeled vehicle can't stand on Rugged ground, a Peacekeeper can.
/// Asking builds no grid, nor does a spawn the gate refuses: a grid comes with its first unit.
#[test]
fn the_gate_checks_the_first_vehicle_on_the_grid_it_will_have() {
    let mut t = game(false);
    t.run_until(1);
    let spawn = |kind, tile: TilePos| {
        Command::Debug(DebugCommand::Spawn {
            kind,
            owner: Some(P1),
            at: SpawnAt::Point(tile.center()),
            orientation: Default::default(),
        })
    };
    let ask = |t: &TestSim, kind, tile| {
        let command = spawn(kind, tile);
        t.sim().validate(&PlayerCommand { player: P1, command })
    };
    let rugged = TilePos::new(7, 4);
    assert_eq!(ask(&t, SEEKER, rugged), Err(Reject::Blocked));
    assert_eq!((ask(&t, PK, rugged), ask(&t, SEEKER, START)), (Ok(()), Ok(())));
    assert!(has_grids(t.sim(), &[INFANTRY]), "asking builds nothing");
    t.at(2, P1, spawn(SEEKER, rugged)).step();
    assert_eq!(t.sim().objects().count(), 3, "refused");
    assert!(has_grids(t.sim(), &[INFANTRY]), "a refused spawn builds nothing");
    t.at(3, P1, spawn(SEEKER, START)).step();
    assert_eq!(t.sim().objects().count(), 4);
    assert!(has_grids(t.sim(), &[VEHICLES, INFANTRY]));
}

/// A unit's grid missing is a bug, never a quiet "no grid" (a unit planning or moving as if its ground were all
/// open): a debug build fails on it.
#[test]
#[cfg(debug_assertions)]
#[should_panic(expected = "no grid of (Ground, 1)")]
fn a_missing_grid_fails_loudly_in_debug_builds() {
    let t = game(false);
    let _ = t
        .sim()
        .nav_state()
        .plan_grid(Some(P1), VEHICLES.0, VEHICLES.1, START.center());
}
