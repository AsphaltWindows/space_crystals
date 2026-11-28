use glam::Vec2;

use super::*;
use crate::defs::ObjectKind;
use crate::orders::Target;
use crate::scenario::SpawnAt;
use crate::sim::Sim;
use crate::units::TilePos;

/// An Airfield (1) of player 1 on tiles (1, 1) to (3, 3), its centre at (160, 160), and four Hornets of player 1 (2 to
/// 5), as they come into a game, unassigned and in the air.
fn objects() -> BTreeMap<ObjectId, Object> {
    let mut builder = crate::testing::ScenarioBuilder::new(&"....................\n".repeat(10)).spawn(
        ObjectKind::Airfield,
        Some(PlayerId(1)),
        SpawnAt::Tile(TilePos::new(1, 1)),
    );
    for x in [400.0, 500.0, 600.0, 700.0] {
        builder = builder.spawn(
            ObjectKind::Hornet,
            Some(PlayerId(1)),
            SpawnAt::Point(Vec2::new(x, 100.0)),
        );
    }
    let sim = Sim::new(&builder.build());
    sim.objects().map(|o| (o.id, o.clone())).collect()
}

const AIRFIELD: ObjectId = ObjectId(1);
const CENTRE: Vec2 = Vec2::new(160.0, 160.0);

fn state(objects: &mut BTreeMap<ObjectId, Object>, n: u32) -> &mut aircraft::AircraftState {
    objects
        .get_mut(&ObjectId(n))
        .and_then(|o| o.aircraft.as_mut())
        .expect("an aircraft")
}

/// Object `n`'s landing (`Object::landing`).
fn landing_of(objects: &mut BTreeMap<ObjectId, Object>, n: u32) -> &mut Option<Landing> {
    &mut objects.get_mut(&ObjectId(n)).expect("an object").landing
}

/// The Airfield's slots, the centres of its footprint's quarters, 48 su off its centre along both axes, in the order
/// they are claimed: north-west, north-east, south-west, south-east. A capacity of 3 has the first three; an object
/// that is no Airfield has none.
#[test]
fn an_airfield_has_a_slot_for_each_assignment_in_its_quarters() {
    let objects = objects();
    let airfield = &objects[&AIRFIELD];
    assert_eq!(airfield.pos, CENTRE);
    let found: Vec<Vec2> = slots(airfield).collect();
    let quarter = |x: f32, y: f32| CENTRE + Vec2::new(x, y) * 48.0;
    assert_eq!(
        found,
        [
            quarter(-1.0, -1.0),
            quarter(1.0, -1.0),
            quarter(-1.0, 1.0),
            quarter(1.0, 1.0)
        ]
    );
    assert_eq!(slots(&objects[&ObjectId(2)]).count(), 0, "a Hornet has none");
    let three = AirfieldDef {
        capacity: 3,
        landing_ticks: 16,
        slot_offset_su: 40.0,
    };
    let laid: Vec<Vec2> = layout(Vec2::ZERO, Some(three)).collect();
    assert_eq!(
        laid,
        [Vec2::new(-40.0, -40.0), Vec2::new(40.0, -40.0), Vec2::new(-40.0, 40.0)]
    );
}

/// The claims (plan decision T88: "A landing aircraft claims the lowest free pad, derived each tick"): the aircraft on
/// the Airfield, landing or down, each keep the free slot nearest it, in id order; then those flying home to it, in id
/// order, each the first one left free; one taking off over the footprint keeps its slot too, one taking off off it
/// none. With every slot taken, a fifth flying home has none.
#[test]
fn the_aircraft_on_an_airfield_keep_their_slots_and_those_flying_home_take_the_first_free() {
    let objects = objects();
    let airfield = &objects[&AIRFIELD];
    let slot: Vec<Vec2> = slots(airfield).collect();
    let user = |n: u32, pos: Vec2| User {
        id: ObjectId(n),
        pos,
        on: None,
        rising: false,
        home: None,
    };
    let on = |n: u32, pos: Vec2| User {
        on: Some(AIRFIELD),
        ..user(n, pos)
    };
    let home = |n: u32| User {
        home: Some(AIRFIELD),
        ..user(n, Vec2::new(900.0, 900.0))
    };
    // #3 is down on the south-east slot, #4 lands on the north-east one: they keep theirs. #2 and #5, flying home,
    // take the first two left, in id order.
    let users = [home(2), on(3, slot[3]), on(4, slot[1] + Vec2::new(3.0, -2.0)), home(5)];
    let id = |n: u32| Some(ObjectId(n));
    assert_eq!(claims(airfield, &users), [id(2), id(4), id(5), id(3)]);
    // An aircraft on another Airfield, or flying home to another, takes none of these.
    let elsewhere = User {
        on: Some(ObjectId(99)),
        ..user(6, slot[0])
    };
    assert_eq!(claims(airfield, &[elsewhere, home(2)]), [id(2), None, None, None]);
    // One taking off over the footprint keeps the slot nearest it; one taking off off it keeps none.
    let rising = |n: u32, pos: Vec2| User {
        rising: true,
        ..user(n, pos)
    };
    assert_eq!(
        claims(airfield, &[home(2), rising(7, slot[0])]),
        [id(7), id(2), None, None]
    );
    assert_eq!(
        claims(airfield, &[home(2), rising(7, CENTRE + Vec2::new(200.0, 0.0))]),
        [id(2), None, None, None]
    );
    // Rising over it while flying home to it again: one slot, the nearest.
    let back = User {
        home: Some(AIRFIELD),
        ..rising(7, slot[2])
    };
    assert_eq!(claims(airfield, &[home(2), back]), [id(2), None, id(7), None]);
    // Four on it: a fifth flying home has none, and waits.
    let full = [on(2, slot[0]), on(3, slot[1]), on(4, slot[2]), on(5, slot[3]), home(6)];
    assert_eq!(claims(airfield, &full), [id(2), id(3), id(4), id(5)]);
}

/// Who takes a slot or will (`users`): an aircraft landed, one flying home to its base (a Return to Base, a Rebase),
/// one rising (`Mover::rise`); not one in the air on another order, nor one flying home without a base.
#[test]
fn users_are_the_aircraft_on_a_slot_rising_or_flying_home() {
    let mut objects = objects();
    assert_eq!(users(objects.values()), []);
    *landing_of(&mut objects, 2) = Some(down_on(AIRFIELD, 16));
    state(&mut objects, 3).base = Some(AIRFIELD);
    let give = |o: &mut Object, kind: OrderKind| {
        let orders = o.orders.as_mut().expect("orders");
        orders.give(
            crate::orders::Order {
                kind,
                group: crate::orders::OrderGroup(0),
                slot: None,
            },
            false,
        );
        let order = orders.queue.pop_front().expect("given");
        orders.current = Some(crate::orders::ActiveOrder {
            order,
            since: 0,
            state: crate::orders::Behaviour::Plain,
        });
    };
    give(objects.get_mut(&ObjectId(3)).unwrap(), OrderKind::ReturnToBase);
    give(objects.get_mut(&ObjectId(4)).unwrap(), OrderKind::ReturnToBase);
    give(
        objects.get_mut(&ObjectId(5)).unwrap(),
        OrderKind::Move(Target::Point(Vec2::ZERO)),
    );
    objects
        .get_mut(&ObjectId(5))
        .and_then(|o| o.mover.as_mut())
        .unwrap()
        .rise = Some(Progress::new(16));
    let found: Vec<(u32, Option<ObjectId>, bool, Option<ObjectId>)> = (users(objects.values()).iter())
        .map(|user| (user.id.0, user.on, user.rising, user.home))
        .collect();
    assert_eq!(
        found,
        [
            (2, Some(AIRFIELD), false, None),
            (3, None, false, Some(AIRFIELD)),
            (5, None, true, None)
        ],
        "#4 has no base"
    );
    assert!(flies_home(&objects[&ObjectId(3)]) && !flies_home(&objects[&ObjectId(5)]));
}

/// A landing's frames: an aircraft is down, a ground target, once all are done (`down`); it is drawn from 1, up, to
/// 0, down (`lift`). A new aircraft's is done already (`down_on`).
#[test]
fn down_once_every_frame_of_the_landing_is_done() {
    let mut objects = objects();
    assert_eq!(
        (lift(&objects[&ObjectId(2)]), down(&objects[&ObjectId(2)])),
        (None, false)
    );
    *landing_of(&mut objects, 2) = Some(Landing {
        at: Spot::Structure(AIRFIELD),
        descent: Progress::new(16),
        rearm: None,
    });
    let hornet = |objects: &BTreeMap<ObjectId, Object>| (lift(&objects[&ObjectId(2)]), down(&objects[&ObjectId(2)]));
    assert_eq!(hornet(&objects), (Some(1.0), false));
    for _ in 0..4 {
        let landing = landing_of(&mut objects, 2).as_mut().unwrap();
        landing.descent.step(RATIO_ONE);
    }
    assert_eq!(hornet(&objects), (Some(0.75), false));
    *landing_of(&mut objects, 2) = Some(down_on(AIRFIELD, 16));
    assert_eq!(hornet(&objects), (Some(0.0), true));
    assert_eq!(at(&objects[&ObjectId(2)]), Some(AIRFIELD));
}

/// A take-off from a landing rises from as low as the aircraft has come: all of a rise from down, the frames it came
/// down from part of a landing, and none from a landing not yet begun, still in the air.
#[test]
fn a_take_off_rises_from_as_low_as_it_came() {
    assert_eq!(rise_from(down_on(AIRFIELD, 16).descent, 16), Some(Progress::new(16)));
    let mut descent = Progress::new(16);
    assert_eq!(rise_from(descent, 16), None, "not a frame down");
    for _ in 0..4 {
        descent.step(RATIO_ONE);
    }
    let rise = rise_from(descent, 16).expect("4 frames down");
    assert_eq!((rise.ticks_done(), rise.total), (12, 16), "4 frames to rise");
}

/// "Any order lifts it off" (plan §GH2) but its flight home to the Airfield it is on: a Move, an Attack, a Stop, or a
/// Rebase onto another Airfield lift it off, its landing over and its rise begun; a Return to Base to its base there,
/// or a Rebase onto that Airfield, don't. An aircraft in the air takes off from nowhere.
#[test]
fn any_order_but_its_flight_home_lifts_it_off() {
    let (hornet, base) = (ObjectKind::Hornet, Some(AIRFIELD));
    let lifts = |kind: OrderKind| {
        let (mut landing, mut rise) = (Some(down_on(AIRFIELD, 16)), None);
        let took_off = take_off(kind, hornet, base, &mut landing, &mut rise);
        assert_eq!(took_off, landing.is_none(), "{kind:?}");
        (took_off, rise)
    };
    let full_rise = Some(Progress::new(16));
    assert_eq!(lifts(OrderKind::Move(Target::Point(Vec2::ZERO))), (true, full_rise));
    assert_eq!(lifts(OrderKind::Attack(ObjectId(9))), (true, full_rise));
    assert_eq!(lifts(OrderKind::Stop), (true, full_rise));
    assert_eq!(lifts(OrderKind::Rebase(ObjectId(9))), (true, full_rise));
    assert_eq!(lifts(OrderKind::ReturnToBase), (false, None));
    assert_eq!(lifts(OrderKind::Rebase(AIRFIELD)), (false, None));
    let (mut flying, mut rise) = (None, None);
    assert!(!take_off(OrderKind::Stop, hornet, base, &mut flying, &mut rise) && rise.is_none());
}

/// One take-off for every aircraft (`take_off`): a Supply Chopper on a spot takes off for any order but one for that
/// spot's object, its landing cleared and its rise begun in its own take-off time (`LandingDef::takeoff_ticks`); an
/// order that lands it on another object lifts it off too.
#[test]
fn a_chopper_takes_off_by_the_same_rule() {
    let (chopper, tower) = (ObjectKind::SupplyChopper, ObjectId(7));
    let takeoff = def_of(chopper).expect("its own times").takeoff_ticks;
    let lifts = |kind: OrderKind| {
        let (mut landing, mut rise) = (Some(down_on(tower, 16)), None);
        let took_off = take_off(kind, chopper, None, &mut landing, &mut rise);
        assert_eq!(took_off, landing.is_none(), "{kind:?}");
        (took_off, rise)
    };
    let full_rise = Some(Progress::new(takeoff));
    assert_eq!(lifts(OrderKind::Move(Target::Point(Vec2::ZERO))), (true, full_rise));
    assert_eq!(lifts(OrderKind::PickUpSupplies(ObjectId(9))), (true, full_rise));
    assert_eq!(lifts(OrderKind::ReturnToBase), (true, full_rise), "it has no base");
    assert_eq!(lifts(OrderKind::DropOffSupplies(tower)), (false, None));
    assert_eq!(lifts(OrderKind::AttachToTower(tower)), (false, None));
}

/// "Rearming an aircraft takes RearmTimePerAmmo × missing ammo" (`systems/mechanics/aircraft_landing.md > Assigned
/// base`; "**RearmTimePerAmmo 32 frames**", `factions/gdo/units/hornet.md`), from the frame it is down, at its
/// Airfield's speed (design change 0423: "Each frame, these timers advance by the speed ratio (generated ÷ consumed,
/// capped at 100%)"): at full Power a round every 32 frames, 3 rounds in 96; at 30 of 51 a full rearm of 5 in 272
/// frames, about 17 s (its Verification), what a frame has left over going on to the next round. Full, it rearms no
/// more; one still landing, not down, rearms nothing yet.
#[test]
fn a_round_every_rearm_time_at_its_airfields_speed() {
    let mut objects = objects();
    let power = |generated: u32, consumed: u32| BTreeMap::from([(PlayerId(1), PowerSummary { generated, consumed })]);
    let (full, low) = (power(60, 30), power(30, 51));
    // Steps the rearm until #2 is full: the frames it took.
    let until_full = |objects: &mut BTreeMap<ObjectId, Object>, power: &BTreeMap<PlayerId, PowerSummary>| {
        let mut frames = 0;
        while state(objects, 2).ammo < 5 {
            rearm(objects, power);
            frames += 1;
            assert!(frames <= 400, "rearmed");
            let landing = landing_of(objects, 2).expect("landed");
            assert_eq!(
                landing.rearm.is_some(),
                state(objects, 2).ammo < 5,
                "a round under way until full"
            );
            if frames == 32 && power == &full {
                assert_eq!(state(objects, 2).ammo, 3, "a round in 32 frames");
            }
        }
        frames
    };

    let state2 = state(&mut objects, 2);
    state2.ammo = 2;
    *landing_of(&mut objects, 2) = Some(Landing {
        at: Spot::Structure(AIRFIELD),
        descent: Progress::new(16),
        rearm: None,
    });
    rearm(&mut objects, &full);
    assert_eq!(
        landing_of(&mut objects, 2).and_then(|landing| landing.rearm),
        None,
        "not down yet"
    );
    *landing_of(&mut objects, 2) = Some(down_on(AIRFIELD, 16));
    assert_eq!(until_full(&mut objects, &full), 96, "3 missing, 96 frames");
    rearm(&mut objects, &full);
    assert_eq!(state(&mut objects, 2).ammo, 5, "full, it stays so");

    state(&mut objects, 2).ammo = 0;
    assert_eq!(until_full(&mut objects, &low), 272, "about 17 s at 59%");
}
