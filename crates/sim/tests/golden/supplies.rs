//! The GDO's supply golden game (`factions/gdo/economy.md > SupplyTower`, `factions/gdo/units/supply_chopper.md`):
//! the GDO's starting Supply buys a Field
//! Engineer; the Deployment Center constructs a Supply Tower, which comes with its chopper and is scheduled on a far
//! station, the chopper down there as each delivery comes and back home with it; the Tower trains a second chopper,
//! which attaches to a Tower without one and runs that Tower's schedule on a station a Syndicate Agent picks up at too;
//! the Supplies brought train a Drone Operator.

use std::cell::RefCell;

use sc_sim::defs::Resource;
use sc_sim::landing;
use sc_sim::{ConstructionCommand, DebugCommand, ResourceState, SimConfig, SpawnSettings, supply_run};

use super::*;

// The objects, by id: 1 player 1's Deployment Center, on tiles (2, 2) to (5, 5); 2 to 5 its Generators, 6 its Radar
// Station, 7 its Barracks, 8 its Career Campus (Power 110 generated, 100 used with both Towers: full speed); 9 its
// Supply Tower B, which the map places, without a chopper; 10 and 11 the stations; 12 player 2's Tunnel, far off in
// the south-east, so that player 2 owns a structure all game; 13 their Agent; then player 1's Field Engineer, Tower A
// and its chopper, the chopper Tower A trains, and the Drone Operator, as they come.
const DC: ObjectId = ObjectId(1);
const BARRACKS: ObjectId = ObjectId(7);
/// Tower B, on tiles (2, 13) to (4, 15).
const TOWER_B: ObjectId = ObjectId(9);
/// The far station, on tiles (36, 4) to (37, 5): empty at the start, 4 Supplies once it has been empty for 960
/// frames (`core/map.md > SupplyDeliveryStation`).
const FAR: ObjectId = ObjectId(10);
/// The near one, on tiles (22, 22) to (23, 23): 4 Supplies at the start, 4 more once it has been empty for 800 frames.
const NEAR: ObjectId = ObjectId(11);
const TUNNEL: ObjectId = ObjectId(12);
const AGENT: ObjectId = ObjectId(13);
const ENGINEER: ObjectId = ObjectId(14);
/// Tower A, placed on tiles (2, 8) to (4, 10) at [`PLACE`], and its chopper, which comes with it.
const TOWER_A: ObjectId = ObjectId(15);
const FIRST: ObjectId = ObjectId(16);
/// The chopper Tower A trains, queued as it is placed and out over it 160 frames on, at [`TRAINED`].
const SECOND: ObjectId = ObjectId(17);
const OPERATOR: ObjectId = ObjectId(18);
/// The Deployment Center queues the Tower in tick 1 (320 frames), and places it here.
const PLACE: Tick = 330;
const TRAINED: Tick = 490;
/// The second chopper is given an Attach to Tower B.
const ATTACH: Tick = 500;
/// The Barracks trains a Drone Operator, once the first run has brought its Supply.
const OPERATOR_AT: Tick = 1300;
/// The game's last tick, as the first chopper is back from its second run.
const END: Tick = 2240;

/// What the game did, by tick.
#[derive(Default, Debug)]
struct Seen {
    /// The Supplies each station held at the end of the tick before, and the ticks its deliveries came.
    held: BTreeMap<ObjectId, u32>,
    arrivals: Vec<(Tick, ObjectId)>,
    /// Each chopper's load, and its landing (where, and whether down), at the end of the tick before.
    load: BTreeMap<ObjectId, u32>,
    on: BTreeMap<ObjectId, Option<(ObjectId, bool)>>,
    /// The landings begun from the air, the touch-downs and the loads taken: (tick, chopper, where or how many). A
    /// chopper takes its load as it touches down on a station, and lifts off in the same tick.
    landings: Vec<(Tick, ObjectId, ObjectId)>,
    downs: Vec<(Tick, ObjectId, ObjectId)>,
    loads: Vec<(Tick, ObjectId, u32)>,
    /// The Supplies paid in: (tick, worker, drop-off, amount).
    paid: Vec<(Tick, ObjectId, ObjectId, u32)>,
    /// The units produced: (tick, producer, unit).
    produced: Vec<(Tick, ObjectId, ObjectId)>,
}

thread_local! {
    static SEEN: RefCell<Seen> = RefCell::new(Seen::default());
}

fn p1(command: Command) -> PlayerCommand {
    PlayerCommand { player: P1, command }
}

/// A Supply Delivery Station holding `supplies`, delivering 4 once it has been empty for `interval` frames.
fn station(supplies: u32, interval: Tick) -> SpawnSettings {
    SpawnSettings::SupplyDeliveryStation {
        starting_supplies: supplies,
        delivery_size: 4,
        delivery_interval: interval,
    }
}

fn held(sim: &Sim, station: ObjectId) -> u32 {
    match sim.object(station).and_then(|o| o.resource.as_ref()) {
        Some(ResourceState::SupplyStation { current_supplies, .. }) => *current_supplies,
        _ => panic!("tick {}: {station:?} is no station", sim.tick()),
    }
}

fn train(producer: ObjectId, kind: ObjectKind) -> PlayerCommand {
    p1(Command::Production {
        objects: vec![producer],
        command: ProductionCommand::Train(kind),
    })
}

fn schedule(tower: ObjectId, station: ObjectId) -> PlayerCommand {
    p1(Command::Schedule {
        towers: vec![tower],
        station,
    })
}

fn bank(sim: &Sim) -> (u32, u32) {
    let player = sim.player(P1).expect("player 1 plays");
    (player.crystals, player.supplies)
}

/// Chopper `n` is down on Tower `tower` and attached to it.
fn home(sim: &Sim, n: ObjectId, tower: ObjectId) {
    let tick = sim.tick();
    let o = sim.object(n).unwrap_or_else(|| panic!("tick {tick}: {n:?} gone"));
    let on = landing::of(o)
        .zip(landing::at(o))
        .map(|(landing, at)| (at, landing.descent.done()));
    assert_eq!(on, Some((tower, true)), "tick {tick}: {n:?} down on {tower:?}");
    assert_eq!(supply_run::attached_to(o), Some(tower), "tick {tick}: {n:?}");
}

/// Player 1 plays the GDO, with a base that runs at full Power and a Supply Tower the map places (Tower B, without a
/// chopper: a map's Tower comes alone); a debug command gives them 3000 Space Crystals and leaves their starting
/// Supply (`factions/gdo/economy.md > Starting Conditions`). Player 2 plays the Syndicate, whose Agent picks up at the
/// near station from tick 1, a Supply a trip. In tick 1 the Barracks trains a Field Engineer with the starting Supply
/// and the Deployment Center queues a Supply Tower, placed at [`PLACE`] (Tower A):
/// its chopper comes with it, down on it and attached. Tower A is scheduled on the far station and trains a second
/// chopper; its chopper waits on it, takes off in time to touch down on the station in the tick after each delivery
/// came, takes the 4 and drops them off at home. The second chopper attaches to Tower B at [`ATTACH`] (a right-click
/// there, carrying nothing), which is scheduled on the near station: it flies there at once for the 2 the Agent hasn't
/// claimed, and for the next delivery once the station has been empty for its interval, waiting over it for the
/// delivery, of which the Agent claims 1. At [`OPERATOR_AT`] the Barracks trains a Drone Operator with a Supply the
/// runs brought.
pub(crate) fn gdo_supplies() -> Game {
    let mut builder = ScenarioBuilder::new(&open(48, 30))
        .faction(P1, Faction::Gdo)
        .faction(P2, Faction::Syndicate)
        .spawn(
            ObjectKind::DeploymentCenter,
            Some(P1),
            SpawnAt::Tile(TilePos::new(2, 2)),
        );
    for x in [8, 10, 12, 14] {
        builder = builder.spawn(ObjectKind::Generator, Some(P1), SpawnAt::Tile(TilePos::new(x, 2)));
    }
    let builder = builder
        .spawn(ObjectKind::RadarStation, Some(P1), SpawnAt::Tile(TilePos::new(8, 5)))
        .spawn(ObjectKind::Barracks, Some(P1), SpawnAt::Tile(TilePos::new(11, 5)))
        .spawn(ObjectKind::CareerCampus, Some(P1), SpawnAt::Tile(TilePos::new(15, 5)))
        .spawn(ObjectKind::SupplyTower, Some(P1), SpawnAt::Tile(TilePos::new(2, 13)))
        .spawn_with(station(0, 960), SpawnAt::Tile(TilePos::new(36, 4)))
        .spawn_with(station(4, 800), SpawnAt::Tile(TilePos::new(22, 22)))
        .spawn(ObjectKind::Tunnel, Some(P2), SpawnAt::Tile(TilePos::new(40, 24)))
        .spawn(ObjectKind::Agent, Some(P2), at(30, 22));
    let funds = p1(Command::Debug(DebugCommand::SetResources {
        player: P1,
        crystals: 3000,
        supplies: 1,
        alloys: 0,
        essence: 0,
        flux: 0,
    }));
    let construct = p1(Command::Construction {
        objects: vec![DC],
        command: ConstructionCommand::Construct(ObjectKind::SupplyTower),
    });
    let place = p1(Command::Construction {
        objects: vec![DC],
        command: ConstructionCommand::Place {
            origin: TilePos::new(2, 8),
            orientation: Default::default(),
        },
    });
    let commands = vec![
        (1, funds),
        (1, train(BARRACKS, ObjectKind::FieldEngineer)),
        (1, construct),
        (1, order(P2, [AGENT.0], OrderKind::Gather(NEAR))),
        (PLACE, place),
        (PLACE + 1, schedule(TOWER_A, FAR)),
        (PLACE + 1, train(TOWER_A, ObjectKind::SupplyChopper)),
        (ATTACH, order(P1, [SECOND.0], OrderKind::AttachToTower(TOWER_B))),
        (ATTACH + 1, schedule(TOWER_B, NEAR)),
        (OPERATOR_AT, train(BARRACKS, ObjectKind::DroneOperator)),
    ];
    Game {
        scenario: builder.config(SimConfig::DEBUG).build(),
        commands,
        script: None,
        end: END,
        during: |tick, sim| {
            if tick == 1 {
                SEEN.set(Seen::default());
                assert_eq!(bank(sim), (2775, 0), "the Field Engineer took the starting Supply");
            }
            if tick == PLACE {
                home(sim, FIRST, TOWER_A);
                assert_eq!(bank(sim), (2775, 0), "the chopper came free");
            }
            if tick == OPERATOR_AT - 1 {
                assert_eq!(bank(sim).1, 6, "the first two runs' Supplies");
            }
            SEEN.with_borrow_mut(|seen| {
                for e in sim.events() {
                    match *e {
                        SimEvent::ResourcesDelivered {
                            worker,
                            drop_off,
                            amount,
                            resource,
                        } => {
                            assert_eq!(resource, Resource::Supplies, "tick {tick}");
                            seen.paid.push((tick, worker, drop_off, amount));
                        }
                        SimEvent::UnitProduced { producer, unit } => seen.produced.push((tick, producer, unit)),
                        _ => {}
                    }
                }
                for station in [FAR, NEAR] {
                    let now = held(sim, station);
                    if seen.held.insert(station, now).is_some_and(|before| now > before) {
                        seen.arrivals.push((tick, station));
                    }
                }
                for chopper in [FIRST, SECOND] {
                    let Some(o) = sim.object(chopper) else { continue };
                    let load = supply_run::carried(o);
                    if load > seen.load.insert(chopper, load).unwrap_or(0) {
                        seen.loads.push((tick, chopper, load));
                    }
                    let on = landing::of(o)
                        .zip(landing::at(o))
                        .map(|(landing, at)| (at, landing.descent.done()));
                    let before = seen.on.insert(chopper, on);
                    match (before, on) {
                        (Some(None), Some((at, _))) => seen.landings.push((tick, chopper, at)),
                        (Some(Some((_, false))), Some((at, true))) => seen.downs.push((tick, chopper, at)),
                        _ => {}
                    }
                }
            });
        },
        expect: |sim| {
            SEEN.with_borrow(|seen| {
                let produced = [
                    (160, BARRACKS, ENGINEER),
                    (TRAINED, TOWER_A, SECOND),
                    (OPERATOR_AT + 159, BARRACKS, OPERATOR),
                ];
                assert_eq!(seen.produced, produced);
                // The deliveries: the far station's every 960 frames from the start, the near one's 800 frames after
                // the second chopper emptied it.
                assert_eq!(seen.arrivals, [(960, FAR), (1609, NEAR), (1920, FAR)]);
                let loads = [(810, SECOND, 2), (961, FIRST, 4), (1610, SECOND, 3), (1921, FIRST, 4)];
                assert_eq!(seen.loads, loads, "all the station held that no Agent had claimed");
                // Each delivery taken in the tick after it came, the landing on the station begun 16 frames before
                // the touch-down; the second chopper's first landing there in the run it began at once.
                let landings = [
                    (579, SECOND, TOWER_B),
                    (794, SECOND, NEAR),
                    (945, FIRST, FAR),
                    (1017, SECOND, TOWER_B),
                    (1248, FIRST, TOWER_A),
                    (1594, SECOND, NEAR),
                    (1817, SECOND, TOWER_B),
                    (1905, FIRST, FAR),
                    (2208, FIRST, TOWER_A),
                ];
                assert_eq!(seen.landings, landings);
                for &(tick, chopper, at) in &seen.landings {
                    let down = seen.downs.iter().any(|&down| down == (tick + 16, chopper, at));
                    let took = seen.loads.iter().any(|&(load, n, _)| (load, n) == (tick + 16, chopper));
                    assert!(down || took, "the landing in tick {tick}: down 16 frames on");
                }
                let paid = [
                    (331, AGENT, TUNNEL, 1),
                    (755, AGENT, TUNNEL, 1),
                    (1033, SECOND, TOWER_B, 2),
                    (1264, FIRST, TOWER_A, 4),
                    (1833, SECOND, TOWER_B, 3),
                    (1869, AGENT, TUNNEL, 1),
                    (2224, FIRST, TOWER_A, 4),
                ];
                assert_eq!(seen.paid, paid);
            });
            assert_eq!(bank(sim), (2575, 12), "1 − 1 + 2 + 4 − 1 + 3 + 4");
            home(sim, FIRST, TOWER_A);
            // The second chopper on its way to the near station's next delivery, still attached to Tower B.
            let second = sim.object(SECOND).expect("the second chopper");
            assert_eq!(supply_run::attached_to(second), Some(TOWER_B));
            let doing = second.orders.as_ref().and_then(|orders| orders.current.as_ref());
            assert_eq!(
                doing.map(|active| active.order.kind),
                Some(OrderKind::PickUpSupplies(NEAR))
            );
            let scheduled = |tower: ObjectId| sim.object(tower).and_then(|o| o.schedule);
            assert_eq!((scheduled(TOWER_A), scheduled(TOWER_B)), (Some(FAR), Some(NEAR)));
            for n in [ENGINEER, OPERATOR] {
                assert_eq!(sim.object(n).map(|o| o.owner), Some(Some(P1)), "{n:?}");
            }
        },
    }
}
