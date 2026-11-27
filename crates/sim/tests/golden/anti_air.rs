//! The GDO's anti-air golden game (slice M5-K, design change 0356; `factions/gdo/structures/arsenal.md`,
//! `factions/gdo/units/rocketman.md`): a Deployment Center constructs and places an Arsenal, the Barracks trains three
//! Rocketmen, and they shoot down two Sparrows, spawned directly, first with normal missiles, which the circling
//! Sparrows dodge (almost always, design change 0445: a burst may graze one), then with Target Locks, one after the
//! other, whose bursts (M5-S8) land on them.

use std::cell::RefCell;

use sc_sim::{BuildingOrientation, ConstructionCommand, DebugCommand, Reject, SimConfig};

use super::*;

// The objects, by id: 1 to 3 player 1's, below, 4 and 5 player 2's Sparrows, on Hold, circling over tiles (22, 6) and
// (22, 12), 6 player 2's Airstrip, far off in the south-east, so that player 2 owns a structure all game
// (`core/game_rules.md > Defeat Condition`); 7 the Arsenal, placed; 8 to 10 the Rocketmen, as they come out.
/// Player 1's Deployment Center, on tiles (2, 6) to (5, 9).
const DC: ObjectId = ObjectId(1);
/// Player 1's Barracks, on tiles (8, 11) to (10, 12); a Generator (2) on tiles (8, 3) to (9, 4) powers the Arsenal.
const BARRACKS: ObjectId = ObjectId(3);
const SPARROWS: [u32; 2] = [4, 5];
const ARSENAL: ObjectId = ObjectId(7);
const ROCKETMEN: [u32; 3] = [8, 9, 10];
/// The Arsenal's tiles' corner: in the Deployment Center's sight, as placing needs (`Reject::Unplaceable`).
const ARSENAL_AT: (i32, i32) = (7, 7);
/// The Arsenal is ready in the Deployment Center in this tick, 320 frames after its Construct
/// (`factions/gdo/structures/deployment_center.md > Constructs`: "Arsenal: 150 Space Crystals, 320 frames"), and
/// placed in the next.
const READY: Tick = 320;
const PLACE: Tick = READY + 1;
/// The Barracks queues three Rocketmen in this tick, the Arsenal standing, and each comes out 160 frames after the one
/// before (`factions/gdo/structures/barracks.md > Produces`: "Rocketman: 100 Space Crystals, 160 frames").
const TRAIN: Tick = PLACE + 1;
const PRODUCED: [Tick; 3] = [TRAIN + 159, TRAIN + 319, TRAIN + 479];
/// The Rocketmen walk from the rally point to within their Range of both Sparrows, and fire normal missiles once there.
const MOVE: Tick = 820;
/// All three lock onto Sparrow 4, and, Shift-queued, onto Sparrow 5.
const LOCK: Tick = 1000;
/// The game's last tick, a few seconds after the second Sparrow fell.
const END: Tick = 1200;

/// What the missiles did: the normal missiles fired at the Sparrows (shooter, landing tick), their hits and the HP
/// those took; the locked ones, their hits and the HP each took; and the Sparrows' hits on the Rocketmen.
#[derive(Default)]
struct Missiles {
    normal: Vec<(u32, Tick)>,
    normal_hits: u32,
    normal_taken: f32,
    locked: u32,
    locked_hits: u32,
    locked_taken: Vec<f32>,
    on_rocketmen: u32,
}

/// (M5-S8) A missile's burst on a Sparrow's centre, as a locked one lands: its Damage of 40 over its AoERadius of 32 su
/// (`factions/gdo/units/rocketman.md > AttackAttributes (normal attack)`, design change 0445), all of the Sparrow's 40 ×
/// 40 Silhouette inside the circle (its corners 28.3 su out), no armour: 40 × 1600 / (π × 32²) ≈ 19.9.
const LOCKED_BURST: f32 = 40.0 * 1600.0 / (std::f32::consts::PI * 32.0 * 32.0);

thread_local! {
    static MISSILES: RefCell<Missiles> = RefCell::new(Missiles::default());
}

/// Player 1 plays the GDO, player 2 the Cults, whose Sparrows they are, with debug commands on: a debug command gives
/// player 1 the Arsenal's and the three Rocketmen's 450 Space Crystals, so that the game is about the anti-air, not
/// about mining.
fn gdo_p1_cults_p2(builder: ScenarioBuilder) -> Scenario {
    let mut scenario = builder.config(SimConfig::DEBUG).build();
    scenario.setup.players[0].faction = Faction::Gdo;
    scenario.setup.players[1].faction = Faction::Cults;
    scenario
}

/// The Barracks' W, a Rocketman.
fn train() -> Command {
    Command::Production {
        objects: vec![BARRACKS],
        command: ProductionCommand::Train(ObjectKind::Rocketman),
    }
}

/// Player 2's Sparrows hold over their spots from tick 1, circling, firing only at what comes within their Range of 3
/// (`factions/cults/units/sparrow.md > Flight rules`). Player 1's Deployment Center constructs an Arsenal from tick 1
/// and places it at [`PLACE`], the Barracks refusing W until then ("Requires an Arsenal"); at [`TRAIN`] the Barracks
/// queues three Rocketmen, all the bank, with a rally point east of it. At [`MOVE`] the three walk east, within their
/// Range of both Sparrows and in sight of them, and fire normal missiles, which the circling Sparrows dodge (all but
/// one, as the game played when it was re-blessed for design change 0445 in slice M5-S8: that one's burst of 32 su,
/// in the air, grazed a Sparrow's corner for 0.34 HP), while the Sparrows' guns hit them when they circle within
/// range. At [`LOCK`] they lock onto Sparrow 4 and, Shift-queued, onto Sparrow 5 (a queued order waits behind the
/// lock, design change 0438): after LockOnDuration each fires locked missiles, which can't miss, and Sparrow 4 falls to
/// three of them, each bursting on its centre ([`LOCKED_BURST`]); the locks on Sparrow 5 start at once, and it falls
/// to three more. Every burst is in the air, fired at an aircraft, and none hits the Rocketmen under it. The Rocketmen
/// all stand.
pub(crate) fn gdo_anti_air() -> Game {
    let builder = ScenarioBuilder::new(&open(40, 18))
        .spawn(
            ObjectKind::DeploymentCenter,
            Some(P1),
            SpawnAt::Tile(TilePos::new(2, 6)),
        )
        .spawn(ObjectKind::Generator, Some(P1), SpawnAt::Tile(TilePos::new(8, 3)))
        .spawn(ObjectKind::Barracks, Some(P1), SpawnAt::Tile(TilePos::new(8, 11)))
        .spawn(ObjectKind::Sparrow, Some(P2), at(22, 6))
        .spawn(ObjectKind::Sparrow, Some(P2), at(22, 12))
        .spawn(ObjectKind::Airstrip, Some(P2), SpawnAt::Tile(TilePos::new(34, 15)));
    let p1 = |command| PlayerCommand { player: P1, command };
    let bank = p1(Command::Debug(DebugCommand::SetResources {
        player: P1,
        crystals: 450,
        supplies: 0,
        alloys: 0,
        essence: 0,
        flux: 0,
    }));
    let construction = |command| {
        p1(Command::Construction {
            objects: vec![DC],
            command,
        })
    };
    let place = ConstructionCommand::Place {
        origin: TilePos::new(ARSENAL_AT.0, ARSENAL_AT.1),
        orientation: BuildingOrientation::default(),
    };
    let rally = p1(Command::Production {
        objects: vec![BARRACKS],
        command: ProductionCommand::SetRally(Target::Point(point(15, 9))),
    });
    let mut lock_next = order(P1, ROCKETMEN, OrderKind::TargetLock(ObjectId(5)));
    if let Command::Order { queue, .. } = &mut lock_next.command {
        *queue = true;
    }
    let commands = vec![
        (1, bank),
        (1, construction(ConstructionCommand::Construct(ObjectKind::Arsenal))),
        (1, order(P2, SPARROWS, OrderKind::Hold)),
        (PLACE, construction(place)),
        (TRAIN, rally),
        (TRAIN, p1(train())),
        (TRAIN, p1(train())),
        (TRAIN, p1(train())),
        (MOVE, order(P1, ROCKETMEN, OrderKind::Move(Target::Point(point(19, 9))))),
        (LOCK, order(P1, ROCKETMEN, OrderKind::TargetLock(ObjectId(4)))),
        (LOCK, lock_next),
    ];
    Game {
        scenario: gdo_p1_cults_p2(builder),
        commands,
        script: None,
        end: END,
        during: |tick, sim| {
            if tick == 1 {
                MISSILES.set(Missiles::default());
            }
            if tick == READY {
                let dc = sim.object(DC).and_then(|o| o.constructor);
                assert_eq!(dc.and_then(|c| c.ready()), Some(ObjectKind::Arsenal), "ready");
                let w = sim.validate(&PlayerCommand {
                    player: P1,
                    command: train(),
                });
                assert_eq!(w, Err(Reject::Requires(ObjectKind::Arsenal)), "W greyed");
            }
            if tick == PLACE {
                let arsenal = sim.object(ARSENAL).expect("placed");
                assert_eq!((arsenal.kind, arsenal.owner), (ObjectKind::Arsenal, Some(P1)));
            }
            if tick == TRAIN {
                assert_eq!(sim.player(P1).map(|p| p.crystals), Some(0), "paid");
            }
            let produced: Vec<ObjectId> = (sim.events().iter())
                .filter_map(|e| match *e {
                    SimEvent::UnitProduced { producer, unit } if producer == BARRACKS => Some(unit),
                    _ => None,
                })
                .collect();
            match PRODUCED.iter().position(|&at| at == tick) {
                Some(i) => assert_eq!(produced, [ObjectId(ROCKETMEN[i])], "tick {tick}"),
                None => assert_eq!(produced, [], "tick {tick}"),
            }
            let rocketman = |id: ObjectId| ROCKETMEN.contains(&id.0);
            let sparrow = |id: ObjectId| SPARROWS.contains(&id.0);
            MISSILES.with_borrow_mut(|missiles| {
                for e in sim.events() {
                    match *e {
                        SimEvent::ProjectileLaunched {
                            shooter,
                            target,
                            homing,
                            lands_at,
                            ..
                        } => {
                            assert!(rocketman(shooter) && target.is_some_and(sparrow), "tick {tick}: {e:?}");
                            assert_eq!(homing, tick >= LOCK, "tick {tick}: locked from the lock on");
                            match homing {
                                true => missiles.locked += 1,
                                false => missiles.normal.push((shooter.0, lands_at)),
                            }
                        }
                        SimEvent::Hit {
                            attacker,
                            target,
                            damage,
                            ..
                        } if rocketman(attacker) && sparrow(target) => {
                            match missiles.normal.contains(&(attacker.0, tick)) {
                                true => {
                                    missiles.normal_hits += 1;
                                    missiles.normal_taken += damage;
                                }
                                false => {
                                    missiles.locked_hits += 1;
                                    missiles.locked_taken.push(damage);
                                }
                            }
                        }
                        SimEvent::Hit { attacker, target, .. } if sparrow(attacker) && rocketman(target) => {
                            missiles.on_rocketmen += 1;
                        }
                        SimEvent::Hit { .. } => panic!("tick {tick}: {e:?}"),
                        _ => {}
                    }
                }
            });
            if tick < LOCK {
                assert!(
                    SPARROWS.iter().all(|&n| sim.object(ObjectId(n)).is_some()),
                    "tick {tick}"
                );
            }
        },
        expect: |sim| {
            for n in SPARROWS {
                assert!(sim.object(ObjectId(n)).is_none(), "Sparrow {n} fell");
            }
            for n in ROCKETMEN {
                assert!(sim.object(ObjectId(n)).is_some(), "Rocketman {n} stands");
            }
            assert!(sim.object(ARSENAL).is_some(), "the Arsenal stands");
            MISSILES.with_borrow(|missiles| {
                assert!(missiles.normal.len() >= 3, "normal missiles: {}", missiles.normal.len());
                assert!(
                    missiles.normal_hits <= 1 && missiles.normal_taken < 1.0,
                    "the circling Sparrows dodged them, but for a graze: {} hits, {} HP",
                    missiles.normal_hits,
                    missiles.normal_taken
                );
                assert_eq!(missiles.locked_hits, 6, "three locked missiles a Sparrow");
                assert!(
                    (missiles.locked_taken.iter()).all(|&taken| (taken - LOCKED_BURST).abs() < 1e-3),
                    "each locked missile bursts on its Sparrow: {:?}",
                    missiles.locked_taken
                );
                assert!(missiles.locked >= missiles.locked_hits);
                assert!(missiles.on_rocketmen > 0, "the Sparrows hit back");
            });
        },
    }
}
