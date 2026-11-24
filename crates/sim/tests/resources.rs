//! Map resources: `core/map.md > SpaceCrystalsPatch` and `> SupplyDeliveryStation`, with the per-object values a
//! map sets on them (`core/map_editor.md > Per-object Resource Values`).

use sc_sim::scenario::{DEFAULT_DELIVERY_INTERVAL, DEFAULT_DELIVERY_SIZE, DEFAULT_STARTING_AMOUNT};
use sc_sim::testing::{ScenarioBuilder, TestSim};
use sc_sim::{
    DebugCommand, ObjectId, ObjectKind, ResourceState, Sim, SimEvent, SpawnAt, SpawnSettings, TilePos, scenarios,
};

fn tile(x: i32, y: i32) -> SpawnAt {
    SpawnAt::Tile(TilePos::new(x, y))
}

fn station_test() -> TestSim {
    ScenarioBuilder::new("....\n....")
        .spawn(ObjectKind::SupplyDeliveryStation, None, tile(1, 0))
        .start()
}

/// `(CurrentSupplies, NextDeliveryTime)` of the station with this id.
fn supplies_of(t: &TestSim, id: u32) -> (u32, Option<u32>) {
    match &t.sim().object(ObjectId(id)).unwrap().resource {
        Some(ResourceState::SupplyStation {
            current_supplies,
            next_delivery_in,
            ..
        }) => (*current_supplies, *next_delivery_in),
        other => panic!("not a station: {other:?}"),
    }
}

/// `(CurrentSupplies, NextDeliveryTime)` of the station with id 1.
fn supplies(t: &TestSim) -> (u32, Option<u32>) {
    supplies_of(t, 1)
}

fn delivered(tick: u32, station: u32, amount: u32) -> (u32, SimEvent) {
    let event = SimEvent::SuppliesDelivered {
        station: ObjectId(station),
        amount,
    };
    (tick, event)
}

fn delivered_at(tick: u32) -> (u32, SimEvent) {
    delivered(tick, 1, DEFAULT_DELIVERY_SIZE)
}

#[test]
fn an_empty_station_delivers_first_at_tick_960() {
    let interval = DEFAULT_DELIVERY_INTERVAL;
    assert_eq!(interval, 960); // 60 s at 16 ticks per second (placeholders).
    let mut t = station_test();
    assert_eq!(supplies(&t), (0, Some(interval)));
    t.step();
    assert_eq!(supplies(&t), (0, Some(interval - 1)));
    t.run_until(interval - 1);
    assert_eq!(supplies(&t), (0, Some(1)));
    assert!(t.events().is_empty());
    t.step();
    assert_eq!(t.events(), [delivered_at(960)]);
    assert_eq!(t.sim().events(), [delivered_at(960).1]);
    assert_eq!(supplies(&t), (DEFAULT_DELIVERY_SIZE, None));
}

#[test]
fn no_countdown_while_stocked() {
    let interval = DEFAULT_DELIVERY_INTERVAL;
    let mut t = station_test();
    t.run_until(interval);
    for _ in 0..3 * interval {
        t.step();
        assert_eq!(supplies(&t), (DEFAULT_DELIVERY_SIZE, None), "tick {}", t.sim().tick());
    }
    assert_eq!(t.events(), [delivered_at(interval)]);
}

#[test]
fn station_values_come_from_the_spawn() {
    // A stocked station waits; an empty one with its own size and interval counts down from the start.
    let stocked = SpawnSettings::SupplyDeliveryStation {
        starting_supplies: 7,
        delivery_size: 2,
        delivery_interval: 5,
    };
    let fast = SpawnSettings::SupplyDeliveryStation {
        starting_supplies: 0,
        delivery_size: 9,
        delivery_interval: 30,
    };
    let mut t = ScenarioBuilder::new("....\n....")
        .spawn_with(stocked, tile(0, 0))
        .spawn_with(fast, tile(2, 0))
        .start();
    assert_eq!(supplies_of(&t, 1), (7, None));
    assert_eq!(supplies_of(&t, 2), (0, Some(30)));
    t.run(100);
    assert_eq!(supplies_of(&t, 1), (7, None), "waits until emptied");
    assert_eq!(supplies_of(&t, 2), (9, None));
    assert_eq!(t.events(), [delivered(30, 2, 9)]);
    match &t.sim().object(ObjectId(1)).unwrap().resource {
        Some(ResourceState::SupplyStation {
            delivery_size,
            delivery_interval,
            ..
        }) => assert_eq!((*delivery_size, *delivery_interval), (2, 5)),
        other => panic!("not a station: {other:?}"),
    }
}

#[test]
fn the_patch_amount_comes_from_the_spawn() {
    let mut t = ScenarioBuilder::new("...\n...")
        .spawn_with(SpawnSettings::CrystalPatch { starting_amount: 777 }, tile(0, 0))
        .spawn(ObjectKind::CrystalPatch, None, tile(1, 0))
        .start();
    // Debug spawns get the defaults.
    t.debug_at(
        1,
        DebugCommand::Spawn {
            kind: ObjectKind::CrystalPatch,
            owner: None,
            at: tile(2, 1),
            orientation: Default::default(),
        },
    );
    t.run(100);
    let remaining: Vec<u32> = t
        .sim()
        .objects()
        .map(|o| match o.resource {
            Some(ResourceState::CrystalPatch { remaining }) => remaining,
            ref other => panic!("{:?} is not a patch: {other:?}", o.id),
        })
        .collect();
    assert_eq!(remaining, [777, DEFAULT_STARTING_AMOUNT, DEFAULT_STARTING_AMOUNT]);
    assert_eq!(DEFAULT_STARTING_AMOUNT, 1500);
    for o in t.sim().objects() {
        assert_eq!((o.owner, o.hp), (None, None));
    }
}

/// `ARCHITECTURE.md > Tick pipeline`: an object a command creates during tick t is first updated in tick t + 1,
/// so a station spawned empty during tick t delivers at t + its interval, like one placed at game start (tick 0).
#[test]
fn objects_spawned_by_a_command_are_first_updated_in_the_next_tick() {
    let mut t = ScenarioBuilder::new("......\n......").start();
    let spawned_in = 37;
    t.debug_at(
        spawned_in,
        DebugCommand::Spawn {
            kind: ObjectKind::SupplyDeliveryStation,
            owner: None,
            at: tile(2, 0),
            orientation: Default::default(),
        },
    );
    t.run_until(spawned_in);
    assert_eq!(t.events(), [(spawned_in, SimEvent::Spawned(ObjectId(1)))]);
    assert_eq!(
        supplies(&t),
        (0, Some(DEFAULT_DELIVERY_INTERVAL)),
        "untouched in its first tick"
    );
    t.step();
    assert_eq!(supplies(&t), (0, Some(DEFAULT_DELIVERY_INTERVAL - 1)));
    t.run_until(spawned_in + DEFAULT_DELIVERY_INTERVAL);
    assert_eq!(t.events().last(), Some(&delivered_at(spawned_in + 960)));
    assert_eq!(t.events().len(), 2);
}

#[test]
fn dev_skirmish_resources_start_full_and_empty() {
    let sim = Sim::new(&scenarios::dev_skirmish());
    let mut patches = 0;
    let mut stations = 0;
    for o in sim.objects() {
        match &o.resource {
            Some(ResourceState::CrystalPatch { remaining }) => {
                assert_eq!(*remaining, DEFAULT_STARTING_AMOUNT);
                patches += 1;
            }
            Some(ResourceState::SupplyStation {
                current_supplies,
                next_delivery_in,
                ..
            }) => {
                assert_eq!(
                    (*current_supplies, *next_delivery_in),
                    (0, Some(DEFAULT_DELIVERY_INTERVAL))
                );
                stations += 1;
            }
            None => panic!("{:?} has no resource state", o.id),
        }
    }
    assert_eq!((patches, stations), (16, 1));
}
