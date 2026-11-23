//! Map resources: `core/map.md > SpaceCrystalsPatch` and `> SupplyDeliveryStation`.

use sc_sim::sim::{DEFAULT_PATCH_AMOUNT, DEFAULT_SDS_DELIVERY_INTERVAL_SECONDS, DEFAULT_SDS_DELIVERY_SIZE};
use sc_sim::testing::{ScenarioBuilder, TestSim};
use sc_sim::{DebugCommand, ObjectId, ObjectKind, ResourceState, Sim, SimEvent, SpawnAt, TilePos, scenarios, seconds};

fn interval() -> u32 {
    seconds(DEFAULT_SDS_DELIVERY_INTERVAL_SECONDS)
}

fn station_test() -> TestSim {
    ScenarioBuilder::new("....\n....")
        .spawn(
            ObjectKind::SupplyDeliveryStation,
            None,
            SpawnAt::Tile(TilePos::new(1, 0)),
        )
        .start()
}

/// `(CurrentSupplies, NextDeliveryTime)` of the station with id 1.
fn supplies(t: &TestSim) -> (u32, Option<u32>) {
    match &t.sim().object(ObjectId(1)).unwrap().resource {
        Some(ResourceState::SupplyStation {
            current_supplies,
            next_delivery_in,
            ..
        }) => (*current_supplies, *next_delivery_in),
        other => panic!("not a station: {other:?}"),
    }
}

fn delivered_at(tick: u32) -> (u32, SimEvent) {
    let event = SimEvent::SuppliesDelivered {
        station: ObjectId(1),
        amount: DEFAULT_SDS_DELIVERY_SIZE,
    };
    (tick, event)
}

#[test]
fn first_delivery_arrives_exactly_one_interval_after_the_start() {
    let interval = interval();
    assert_eq!(interval, 960); // 60 s at 16 ticks per second (placeholders, Q0003).
    let mut t = station_test();
    assert_eq!(supplies(&t), (0, Some(interval)));
    t.step();
    assert_eq!(supplies(&t), (0, Some(interval - 1)));
    t.run_until(interval - 1);
    assert_eq!(supplies(&t), (0, Some(1)));
    assert!(t.events().is_empty());
    t.step();
    assert_eq!(t.events(), [delivered_at(interval)]);
    assert_eq!(t.sim().events(), [delivered_at(interval).1]);
    assert_eq!(supplies(&t), (DEFAULT_SDS_DELIVERY_SIZE, None));
}

#[test]
fn no_countdown_while_stocked() {
    let interval = interval();
    let mut t = station_test();
    t.run_until(interval);
    for _ in 0..3 * interval {
        t.step();
        assert_eq!(
            supplies(&t),
            (DEFAULT_SDS_DELIVERY_SIZE, None),
            "tick {}",
            t.sim().tick()
        );
    }
    assert_eq!(t.events(), [delivered_at(interval)]);
}

#[test]
fn patches_spawn_with_the_placeholder_amount() {
    let mut t = ScenarioBuilder::new("...\n...")
        .spawn(ObjectKind::CrystalPatch, None, SpawnAt::Tile(TilePos::new(0, 0)))
        .start();
    t.debug_at(
        1,
        DebugCommand::Spawn {
            kind: ObjectKind::CrystalPatch,
            owner: None,
            at: SpawnAt::Tile(TilePos::new(2, 1)),
        },
    );
    t.run(100);
    let remaining = DEFAULT_PATCH_AMOUNT;
    for o in t.sim().objects() {
        assert_eq!(
            o.resource,
            Some(ResourceState::CrystalPatch { remaining }),
            "{:?}",
            o.id
        );
        assert_eq!((o.owner, o.hp), (None, None));
    }
    assert_eq!(t.sim().objects().count(), 2);
}

#[test]
fn dev_skirmish_resources_start_full_and_empty() {
    let sim = Sim::new(&scenarios::dev_skirmish());
    let mut patches = 0;
    let mut stations = 0;
    for o in sim.objects() {
        match &o.resource {
            Some(ResourceState::CrystalPatch { remaining }) => {
                assert_eq!(*remaining, DEFAULT_PATCH_AMOUNT);
                patches += 1;
            }
            Some(ResourceState::SupplyStation {
                current_supplies,
                next_delivery_in,
                ..
            }) => {
                assert_eq!((*current_supplies, *next_delivery_in), (0, Some(interval())));
                stations += 1;
            }
            None => panic!("{:?} has no resource state", o.id),
        }
    }
    assert_eq!((patches, stations), (16, 1));
}
