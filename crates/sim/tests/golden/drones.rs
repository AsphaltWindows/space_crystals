//! The GDO's drones golden game (slice M6-GD; `factions/gdo/units/drone_operator.md`): a Barracks trains a Drone
//! Operator for 100 Space Crystals and 1 Supply, and its four drones end each in its own way: the first explodes on
//! a group of Peacekeepers, the second falls past its leash and explodes there, the third runs out of battery, and the
//! fourth falls with its operator. Between them the operator buys the next drone, once the last has ended.

use std::cell::RefCell;

use sc_sim::tether::Fall;
use sc_sim::{DebugCommand, Reject, SimConfig};

use super::*;

// The objects, by id: 1 to 4 player 1's buildings, its Deployment Center (1) on tiles (2, 7) to (5, 10) and the
// three below; 5 to 8 player 2's Peacekeepers, on Hold east of the operator's spot; 9 the Drone Operator, as it comes
// out; 10 to 13 its drones, as they are launched.
/// Player 1's Barracks, on tiles (8, 12) to (10, 13); a Generator (2) on tiles (8, 3) to (9, 4) and a Career Campus (4)
/// on tiles (8, 7) to (9, 8), which the Barracks' R requires.
const BARRACKS: ObjectId = ObjectId(3);
const PEACEKEEPERS: [u32; 4] = [5, 6, 7, 8];
const OPERATOR: ObjectId = ObjectId(9);
const DRONES: [u32; 4] = [10, 11, 12, 13];
/// Where the operator stands, the Barracks' rally point.
const SPOT: (i32, i32) = (16, 10);
/// The operator comes out of the Barracks 160 frames after its R in tick 1 (`factions/gdo/structures/barracks.md >
/// Produces`: "Drone Operator: 100 Space Crystals, 160 frames").
const PRODUCED: Tick = 160;
/// The four launches (R); the first drone attack-moves onto the Peacekeepers, the second is sent past its leash of 8
/// grid units, the third left hovering, and the fourth held over its operator, which player 2's Peacekeepers then
/// attack.
const LAUNCHES: [Tick; 4] = [280, 600, 880, 1380];
/// The three Build Drones (F), each once the drone before has ended: 50 Space Crystals, 160 frames.
const BUYS: [Tick; 3] = [400, 700, 1210];
/// `factions/gdo/units/drone_operator.md > Placeholder stats`: "**battery 20 s** (320 frames)".
const BATTERY: Tick = 320;
/// Player 2's Peacekeepers attack-move onto the operator's spot, out of their sight until then.
const ASSAULT: Tick = 1382;
/// The game's last tick, a few seconds after the operator fell.
const END: Tick = 1520;

/// What the drones did: (tick, drone) of each launch, (tick, drone, why) of each fall, (tick, drone, targets hit) of
/// each blast, the ticks of the purchases, and the tick the operator died.
#[derive(Default)]
struct Log {
    launched: Vec<(Tick, u32)>,
    fell: Vec<(Tick, u32, Fall)>,
    blasts: Vec<(Tick, u32, Vec<u32>)>,
    bought: Vec<Tick>,
    operator_died: Option<Tick>,
}

thread_local! {
    static LOG: RefCell<Log> = RefCell::new(Log::default());
}

/// Player 1's `command`.
fn p1(command: Command) -> PlayerCommand {
    PlayerCommand { player: P1, command }
}

/// Player 1 plays the GDO with a Barracks, a Career Campus and power for both, and gets by a debug command the
/// operator's 100 Space Crystals and 1 Supply and the three drones' 150 and more (the GDO starts with no Supply until
/// slice M6-GS). At tick 1 the Barracks trains a Drone Operator (R), which walks to its rally point [`SPOT`], and
/// player 2's four Peacekeepers hold 7 grid units east of it. At the [`LAUNCHES`]:
/// 1. the first drone attack-moves east, finds the Peacekeepers and explodes on the first it reaches (150 over 48
///    su), hurting those round it; a Move given to the operator while it is out is refused (`Reject::DroneOut`);
/// 2. the second, sent 10 grid units east over the Peacekeepers, falls past its leash, at 8 grid units edge to edge,
///    and explodes there, on the Peacekeepers under it;
/// 3. the third hovers over the operator until its battery runs out, 320 ticks on, and falls without exploding;
/// 4. the fourth holds over the operator, which player 2's Peacekeepers attack-move onto and shoot down: the drone
///    falls with it, without exploding.
///
/// Each Build Drone (F) comes after the drone before has ended, and the drone is in stock 160 frames later.
pub(crate) fn gdo_drones() -> Game {
    let builder = ScenarioBuilder::new(&open(40, 20))
        .spawn(
            ObjectKind::DeploymentCenter,
            Some(P1),
            SpawnAt::Tile(TilePos::new(2, 7)),
        )
        .spawn(ObjectKind::Generator, Some(P1), SpawnAt::Tile(TilePos::new(8, 3)))
        .spawn(ObjectKind::Barracks, Some(P1), SpawnAt::Tile(TilePos::new(8, 12)))
        .spawn(ObjectKind::CareerCampus, Some(P1), SpawnAt::Tile(TilePos::new(8, 7)))
        .spawn(PK, Some(P2), SpawnAt::Point(Vec2::new(1504.0, 640.0)))
        .spawn(PK, Some(P2), SpawnAt::Point(Vec2::new(1504.0, 704.0)))
        .spawn(PK, Some(P2), SpawnAt::Point(Vec2::new(1536.0, 672.0)))
        .spawn(PK, Some(P2), SpawnAt::Point(Vec2::new(1600.0, 700.0)));
    let bank = p1(Command::Debug(DebugCommand::SetResources {
        player: P1,
        crystals: 300,
        supplies: 1,
        alloys: 0,
        essence: 0,
        flux: 0,
    }));
    let rally = p1(Command::Production {
        objects: vec![BARRACKS],
        command: ProductionCommand::SetRally(Target::Point(point(SPOT.0, SPOT.1))),
    });
    let train = p1(Command::Production {
        objects: vec![BARRACKS],
        command: ProductionCommand::Train(ObjectKind::DroneOperator),
    });
    let launch = || order(P1, [OPERATOR.0], OrderKind::Launch);
    let buy = || {
        p1(Command::Ability {
            units: vec![OPERATOR],
            ability: Ability::Buy,
        })
    };
    let spot = point(SPOT.0, SPOT.1);
    let east = |gu: f32| Target::Point(spot + Vec2::new(gu * SU_PER_GU + 24.0, 0.0));
    let mut commands = vec![
        (1, bank),
        (1, rally),
        (1, train),
        (1, order(P2, PEACEKEEPERS, OrderKind::Hold)),
        (
            LAUNCHES[0] + 1,
            order(P1, [OPERATOR.0], OrderKind::Move(Target::Point(point(SPOT.0, 4)))),
        ),
        (
            LAUNCHES[0] + 2,
            order(P1, [DRONES[0]], OrderKind::AttackMove(point(24, 10))),
        ),
        (LAUNCHES[1] + 2, order(P1, [DRONES[1]], OrderKind::Move(east(10.0)))),
        (LAUNCHES[3] + 1, order(P1, [DRONES[3]], OrderKind::Hold)),
        (ASSAULT, order(P2, PEACEKEEPERS, OrderKind::AttackMove(spot))),
    ];
    commands.extend(LAUNCHES.map(|tick| (tick, launch())));
    commands.extend(BUYS.map(|tick| (tick, buy())));
    commands.sort_by_key(|(tick, _)| *tick);
    Game {
        scenario: builder.config(SimConfig::DEBUG).build(),
        commands,
        script: None,
        end: END,
        during: |tick, sim| {
            if tick == 1 {
                LOG.set(Log::default());
            }
            if tick == PRODUCED {
                let operator = sim.object(OPERATOR).expect("out of the Barracks");
                assert_eq!(operator.kind, ObjectKind::DroneOperator);
                let bank = sim.player(P1).map(|p| (p.crystals, p.supplies));
                assert_eq!(bank, Some((200, 0)), "100 Space Crystals and 1 Supply");
            }
            if tick == LAUNCHES[0] {
                let drone = sim.object(ObjectId(DRONES[0])).expect("launched");
                assert_eq!(
                    drone.pos,
                    sim.object(OPERATOR).expect("the operator").pos,
                    "over the operator"
                );
                let walk = PlayerCommand {
                    player: P1,
                    command: Command::Order {
                        units: vec![OPERATOR],
                        kind: OrderKind::Move(Target::Point(point(SPOT.0, 4))),
                        queue: false,
                    },
                };
                assert_eq!(sim.validate(&walk), Err(Reject::DroneOut), "it can't move");
            }
            let drone = |id: ObjectId| DRONES.contains(&id.0);
            LOG.with_borrow_mut(|log| {
                let mut blast = None;
                for e in sim.events() {
                    match *e {
                        SimEvent::DroneLaunched { operator, drone } => {
                            assert_eq!(operator, OPERATOR);
                            log.launched.push((tick, drone.0));
                        }
                        SimEvent::DroneFell { drone, why, .. } => log.fell.push((tick, drone.0, why)),
                        SimEvent::Blast {
                            attacker: Some(attacker),
                            ..
                        } => {
                            assert!(drone(attacker), "tick {tick}: {e:?}");
                            blast = Some(attacker.0);
                            log.blasts.push((tick, attacker.0, Vec::new()));
                        }
                        SimEvent::Hit { attacker, target, .. } if drone(attacker) => {
                            assert_eq!(blast, Some(attacker.0), "tick {tick}: a hit of the drone's blast");
                            if let Some((_, _, hit)) = log.blasts.last_mut() {
                                hit.push(target.0);
                            }
                        }
                        SimEvent::StockBought { unit } => {
                            assert_eq!(unit, OPERATOR);
                            log.bought.push(tick);
                        }
                        SimEvent::Died { id, .. } if id == OPERATOR => log.operator_died = Some(tick),
                        _ => {}
                    }
                }
            });
        },
        expect: |sim| {
            assert!(sim.object(OPERATOR).is_none(), "the operator fell");
            assert!(
                DRONES.iter().all(|&n| sim.object(ObjectId(n)).is_none()),
                "every drone ended"
            );
            let bank = sim.player(P1).map(|p| (p.crystals, p.supplies));
            assert_eq!(bank, Some((50, 0)), "the operator's 100 and three drones' 50");
            LOG.with_borrow(|log| {
                let launched: Vec<(Tick, u32)> = LAUNCHES.into_iter().zip(DRONES).collect();
                assert_eq!(log.launched, launched);
                assert_eq!(log.bought, BUYS.map(|tick| tick + 159));
                let died = log.operator_died.expect("the operator died");
                assert!(died > ASSAULT, "{died}");
                let [
                    (leash, 11, Fall::Leash),
                    (battery, 12, Fall::Battery),
                    (with, 13, Fall::Operator),
                ] = log.fell[..]
                else {
                    panic!("the falls: {:?}", log.fell);
                };
                assert!(leash > LAUNCHES[1] && leash < LAUNCHES[1] + 100, "{leash}");
                assert_eq!((battery, with), (LAUNCHES[2] + BATTERY, died));
                let [(_, 10, on_peacekeepers), (at_leash, 11, under_it)] = &log.blasts[..] else {
                    panic!("the blasts: {:?}", log.blasts);
                };
                assert_eq!(*at_leash, leash, "it exploded as it fell");
                assert!(
                    on_peacekeepers.len() >= 2,
                    "the explosion's share on the group: {on_peacekeepers:?}"
                );
                assert!(!under_it.is_empty(), "the leash's blast on the Peacekeepers under it");
                assert!(
                    (on_peacekeepers.iter().chain(under_it)).all(|n| PEACEKEEPERS.contains(n)),
                    "{:?}",
                    log.blasts
                );
            });
        },
    }
}
