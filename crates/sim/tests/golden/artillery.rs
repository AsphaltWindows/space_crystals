//! The Artillery Rig golden game (slice M6-GA; `factions/gdo/units/artillery_rig.md`, plan decisions T44 and T87, the
//! vehicle deploy model): a Factory trains an Artillery Rig, which drives out to its rally point, then on a Move and a
//! Shift-Deploy deploys where the Move ends; an Attack given while it deploys is held and a Shift-Move queued behind it,
//! which is dropped as it deploys; deployed, it shells three Militants on Hold 10 grid units off, out of its own sight,
//! which a Peacekeeper on Hold spots; at last it undeploys, and drives off on the Move held meanwhile.

use std::cell::RefCell;

use sc_sim::{DebugCommand, SimConfig};

use super::*;

// The objects, by id: 1 player 1's Deployment Center, on tiles (2, 7) to (5, 10), 2 its Factory, on tiles (8, 8) to
// (10, 10), 3 its Advanced Mechanics and 4 its Generator (Power 50 generated, 50 used); 5 player 2's Storage, far off
// in the south-east, so that player 2 owns a structure all game; 6 to 8 player 2's Militants, on Hold; 9 player 1's
// Peacekeeper, on Hold, 4.3 grid units from the nearest Militant, out of its Range of 4; 10 the Rig, as it comes out.
const FACTORY: ObjectId = ObjectId(2);
const MILITANTS: [u32; 3] = [6, 7, 8];
const SPOTTER: u32 = 9;
const RIG: ObjectId = ObjectId(10);
/// The Rig comes out of the Factory 480 frames after its production starts, in tick 1.
const OUT: Tick = 480;
/// The tick of the Rig's Move to [`DEPLOY_AT`] and its Shift-Deploy, parked on its rally point by then.
const DRIVE: Tick = 560;
/// Where it deploys: 10 grid units west of the Militants, edge to edge, beyond its sight of 6 and within its Range of
/// 12.
const DEPLOY_AT: Vec2 = Vec2::new(1270.0, 608.0);
/// While it deploys: its Attack on the first Militant, held, and a Shift-Move behind it, which it can't carry out
/// deployed.
const HOLD_ATTACK: Tick = 700;
const SHIFT_MOVE: Tick = 701;
/// Its Undeploy, and the Move it holds while it undeploys.
const UNDEPLOY: Tick = 1400;
const HOLD_MOVE: Tick = 1410;
/// Where that Move takes it.
const BACK_TO: (i32, i32) = (14, 12);
/// The game's last tick.
const END: Tick = 1650;

/// What the game did, from its events: the Rig's changes of type, its dropped orders, its blasts, and the hits of its
/// shells, by target.
#[derive(Default)]
struct Log {
    kinds: Vec<(Tick, ObjectKind)>,
    dropped: Vec<(Tick, OrderKind)>,
    blasts: Vec<Tick>,
    hits: Vec<(Tick, ObjectId)>,
}

thread_local! {
    static LOG: RefCell<Log> = RefCell::new(Log::default());
}

/// Player 1's `command`.
fn p1(command: Command) -> PlayerCommand {
    PlayerCommand { player: P1, command }
}

fn p1_order(kind: OrderKind, queue: bool) -> PlayerCommand {
    p1(Command::Order {
        units: vec![RIG],
        kind,
        queue,
    })
}

/// Player 1 plays the GDO with a Deployment Center, a Factory, Advanced Mechanics and a Generator, and gets by a debug
/// command the Rig's 250 Space Crystals and 5 Supplies (set over the GDO's starting 1); player 2
/// plays the Cults. At tick 1 the Factory queues the Rig, which comes out at [`OUT`] and drives to the rally point. At
/// [`DRIVE`] it drives to [`DEPLOY_AT`] and deploys there (Shift); at [`HOLD_ATTACK`] and [`SHIFT_MOVE`], while it
/// deploys, it is given an Attack on the first Militant and a Shift-Move. Deployed, the Shift-Move is dropped
/// (`SimEvent::OrderDropped`) and the Attack runs: the shells fall on the Militants, which the Peacekeeper sees for
/// it. At [`UNDEPLOY`] it undeploys, at [`HOLD_MOVE`] it is given a Move, held until it is mobile again and then
/// carried out.
pub(crate) fn gdo_artillery() -> Game {
    let builder = ScenarioBuilder::new(&open(40, 20))
        .faction(P1, Faction::Gdo)
        .faction(P2, Faction::Cults)
        .spawn(
            ObjectKind::DeploymentCenter,
            Some(P1),
            SpawnAt::Tile(TilePos::new(2, 7)),
        )
        .spawn(ObjectKind::Factory, Some(P1), SpawnAt::Tile(TilePos::new(8, 8)))
        .spawn(
            ObjectKind::AdvancedMechanics,
            Some(P1),
            SpawnAt::Tile(TilePos::new(2, 2)),
        )
        .spawn(ObjectKind::Generator, Some(P1), SpawnAt::Tile(TilePos::new(2, 13)))
        .spawn(ObjectKind::Storage, Some(P2), SpawnAt::Tile(TilePos::new(36, 17)))
        .spawn(ObjectKind::Militant, Some(P2), at(30, 9))
        .spawn(ObjectKind::Militant, Some(P2), at(31, 9))
        .spawn(ObjectKind::Militant, Some(P2), at(30, 10))
        .spawn(PK, Some(P1), SpawnAt::Point(Vec2::new(1650.0, 608.0)));
    let bank = p1(Command::Debug(DebugCommand::SetResources {
        player: P1,
        crystals: 250,
        supplies: 5,
        alloys: 0,
        essence: 0,
        flux: 0,
    }));
    let train = p1(Command::Production {
        objects: vec![FACTORY],
        command: ProductionCommand::Train(ObjectKind::ArtilleryRig),
    });
    let deployed = OrderKind::VehicleDeploy {
        into: ObjectKind::ArtilleryRigDeployed,
    };
    let mobile = OrderKind::VehicleDeploy {
        into: ObjectKind::ArtilleryRig,
    };
    let commands = vec![
        (1, bank),
        (1, train),
        (1, order(P2, MILITANTS, OrderKind::Hold)),
        (1, order(P1, [SPOTTER], OrderKind::Hold)),
        (DRIVE, p1_order(OrderKind::Move(Target::Point(DEPLOY_AT)), false)),
        (DRIVE, p1_order(deployed, true)),
        (HOLD_ATTACK, p1_order(OrderKind::Attack(ObjectId(MILITANTS[0])), false)),
        (SHIFT_MOVE, p1_order(OrderKind::Move(Target::Point(point(20, 3))), true)),
        (UNDEPLOY, p1_order(mobile, false)),
        (
            HOLD_MOVE,
            p1_order(OrderKind::Move(Target::Point(point(BACK_TO.0, BACK_TO.1))), false),
        ),
    ];
    Game {
        scenario: builder.config(SimConfig::DEBUG).build(),
        commands,
        script: None,
        end: END,
        during: |tick, sim| {
            if tick == 1 {
                LOG.set(Log::default());
                let bank = sim.player(P1).map(|p| (p.crystals, p.supplies));
                assert_eq!(bank, Some((0, 0)), "the Rig paid");
            }
            if tick == OUT {
                let rig = sim.object(RIG).expect("out");
                assert_eq!(rig.kind, ObjectKind::ArtilleryRig);
            }
            LOG.with_borrow_mut(|log| {
                for e in sim.events() {
                    match *e {
                        SimEvent::KindChanged { id, to, .. } if id == RIG => log.kinds.push((tick, to)),
                        SimEvent::OrderDropped { unit, order } if unit == RIG => log.dropped.push((tick, order)),
                        SimEvent::Blast { attacker, .. } if attacker == Some(RIG) => log.blasts.push(tick),
                        SimEvent::Hit { attacker, target, .. } if attacker == RIG => log.hits.push((tick, target)),
                        _ => {}
                    }
                }
            });
            if tick == HOLD_ATTACK {
                let rig = sim.object(RIG).expect("on the map");
                assert!(
                    sc_sim::vehicle_deploy::is_deploying(rig),
                    "deploying as the Attack comes"
                );
            }
            if (HOLD_ATTACK..UNDEPLOY).contains(&tick) {
                assert!(
                    sim.object(RIG).unwrap().pos.distance(DEPLOY_AT) < 8.0,
                    "tick {tick}: it stands"
                );
            }
        },
        // As the game played when it was made: it deployed in tick 708, its deploy started as its Move ended in 660;
        // its five shells, 104 frames apart, killed the first Militant (the Attack) and then the third, which it picked
        // by itself; the second, out of the Peacekeeper's sight of 5 grid units, it never fired at.
        expect: |sim| {
            let rig = sim.object(RIG).expect("the Rig stands");
            assert_eq!(rig.kind, ObjectKind::ArtilleryRig, "undeployed");
            assert!(
                rig.pos.distance(point(BACK_TO.0, BACK_TO.1)) < 8.0,
                "it drove off: {}",
                rig.pos
            );
            let alive: Vec<u32> = MILITANTS
                .into_iter()
                .filter(|&n| sim.object(ObjectId(n)).is_some())
                .collect();
            assert_eq!(alive, [7], "the second Militant, out of sight");
            LOG.with_borrow(|log| {
                let deployed = log.kinds[0].0;
                assert_eq!(
                    log.kinds,
                    [
                        (deployed, ObjectKind::ArtilleryRigDeployed),
                        (UNDEPLOY + 48, ObjectKind::ArtilleryRig)
                    ]
                );
                assert!(deployed < HOLD_ATTACK + 48, "deploying as the Attack came: {deployed}");
                assert_eq!(log.dropped, [(deployed, OrderKind::Move(Target::Point(point(20, 3))))]);
                assert_eq!(log.blasts.len(), 5, "{:?}", log.blasts);
                assert!(
                    log.blasts.windows(2).all(|pair| pair[1] - pair[0] == 104),
                    "{:?}",
                    log.blasts
                );
                assert!(deployed < log.blasts[0] && log.blasts[4] < UNDEPLOY, "{:?}", log.blasts);
                assert!(
                    log.hits.iter().all(|&(_, target)| MILITANTS.contains(&target.0)),
                    "{:?}",
                    log.hits
                );
            });
        },
    }
}
