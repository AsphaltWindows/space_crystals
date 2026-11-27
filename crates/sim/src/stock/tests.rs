use glam::Vec2;

use super::*;
use crate::ids::PlayerId;
use crate::scenario::SpawnAt;
use crate::sim::Sim;
use crate::testing::ScenarioBuilder;

/// A Drone Operator (1) and a Peacekeeper (2) of player 1, as they come into a game.
fn fresh_units() -> (Object, Object) {
    let builder = ScenarioBuilder::new(&"..........\n".repeat(8))
        .spawn(
            ObjectKind::DroneOperator,
            Some(PlayerId(1)),
            SpawnAt::Point(Vec2::new(100.0, 100.0)),
        )
        .spawn(
            ObjectKind::Peacekeeper,
            Some(PlayerId(1)),
            SpawnAt::Point(Vec2::new(300.0, 100.0)),
        );
    let sim = Sim::new(&builder.build());
    let unit = |id| sim.object(ObjectId(id)).expect("spawned").clone();
    (unit(1), unit(2))
}

/// "Starts with **1** drone when trained; holds a **maximum of 1**": the operator comes in with its type's start, and a
/// type without a stock has none.
#[test]
fn a_unit_starts_with_its_types_stock() {
    let (operator, peacekeeper) = fresh_units();
    assert_eq!(operator.stock, Some(Stock { count: 1, buying: None }));
    assert_eq!(peacekeeper.stock, None);
    assert_eq!(fresh(ObjectKind::Peacekeeper), None);
}

/// Why a purchase can't start: the drone out first (design change 0421), then a purchase under way, then a full stock;
/// none once the drone has ended and the stock has room.
#[test]
fn a_purchase_starts_only_with_room_no_purchase_and_no_drone_out() {
    let (mut operator, _) = fresh_units();
    let stock_of = |o: &Object| o.stock.expect("a stock");
    assert_eq!(refusal(&operator, &stock_of(&operator)), Some(Unbuyable::Full));
    operator.drone = Some(ObjectId(9));
    operator.stock = Some(Stock {
        count: 0,
        buying: Some(Progress::new(160)),
    });
    assert_eq!(refusal(&operator, &stock_of(&operator)), Some(Unbuyable::DroneOut));
    operator.drone = None;
    assert_eq!(refusal(&operator, &stock_of(&operator)), Some(Unbuyable::Buying));
    operator.stock = Some(Stock { count: 0, buying: None });
    assert_eq!(refusal(&operator, &stock_of(&operator)), None);
}

/// "Build Drone (F): 50 Space Crystals, 160 frames": a purchase takes its frames at full speed, one a step, and adds
/// its item at the last; a stock already at its max (a lowered max's) takes none past it.
#[test]
fn a_purchase_adds_its_item_after_its_frames_and_never_past_the_max() {
    let (mut operator, _) = fresh_units();
    operator.stock = Some(Stock { count: 0, buying: None });
    assert!(start_buying(&mut operator));
    let id = operator.id;
    let mut objects = BTreeMap::from([(id, operator)]);
    let frames = def_of(ObjectKind::DroneOperator).expect("a stock").frames;
    assert_eq!(frames, 160);
    for frame in 1..frames {
        assert_eq!(step(&mut objects, ObjectId(u32::MAX)), [], "frame {frame}");
    }
    assert_eq!(step(&mut objects, ObjectId(u32::MAX)), [id]);
    assert_eq!(objects[&id].stock, Some(Stock { count: 1, buying: None }));
    let full = objects.get_mut(&id).unwrap();
    full.stock = Some(Stock {
        count: 1,
        buying: Some(Progress {
            elapsed: 159 * RATIO_ONE,
            total: 160,
        }),
    });
    assert_eq!(step(&mut objects, ObjectId(u32::MAX)), [id]);
    assert_eq!(objects[&id].stock.map(|s| s.count), Some(1), "never past the max");
    assert_eq!(
        step(&mut objects, ObjectId(1)),
        [],
        "only the units before the tick's new ones"
    );
}
