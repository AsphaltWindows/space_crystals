//! `factions/gdo/economy.md > SupplyTower > Scheduled Deliveries`: "The attached chopper automatically departs at a
//! time calculated so it arrives at the SDS simultaneously with the next expected supply delivery", and the rules
//! after it.

use super::*;

/// The station of these tests: on tiles (30, 8) to (31, 9), its spot at (1984, 576), 1760 su east of the spot of a
/// Tower placed on tiles (2, 8) to (4, 10).
const S: u32 = FIRST;

/// [`base`] with [`S`], empty, delivering 4 Supplies every `interval` ticks.
fn far_station(interval: Tick) -> TestSim {
    base(|b| b.spawn_with(station(0, interval), tile(30, 8)))
}

/// Whether chopper `n`'s current order is a Pick Up Supplies at [`S`]: it flies there.
fn outbound(t: &TestSim, n: u32) -> bool {
    let current = object(t, n).orders.as_ref().and_then(|orders| orders.current.as_ref());
    current.is_some_and(|active| active.order.kind == OrderKind::PickUpSupplies(id(S)))
}

/// Whether chopper `n` is home: down on its Tower `tower`, attached to it, carrying nothing, idle.
fn home(t: &TestSim, n: u32, tower: u32) -> bool {
    down_on(t, n, tower) && attached(t, n) == Some(tower) && carried(t, n) == 0 && doing(t, n).is_none()
}

/// A scheduled run (`factions/gdo/economy.md > SupplyTower > Scheduled Deliveries`): the chopper waits on its Tower
/// until its time, flies to the station, is down on it in the first tick the delivery is in it, takes all 4 Supplies
/// and lands back on its Tower, which pays them in, twice over; it stays attached throughout, the schedule's orders
/// breaking nothing. It leaves early enough to be over the station before it must land, and waits there no longer than
/// the schedule's margin.
#[test]
fn a_scheduled_run_is_down_on_its_station_as_the_delivery_comes() {
    let mut t = far_station(960);
    let (tower, chopper) = placed_tower(&mut t, TilePos::new(2, 8));
    next(&mut t, schedule(&[tower], S));
    assert_eq!(object(&t, tower).schedule, Some(id(S)));
    for run in 1..=2 {
        assert!(home(&t, chopper, tower), "run {run}");
        step_until(&mut t, 2000, "it leaves", |t| outbound(t, chopper));
        assert_eq!(held(&t, S), 0);
        let over = step_until(&mut t, 600, "it comes to rest over the station", |t| {
            t.status(chopper) != sc_sim::MoveStatus::Moving && object(t, chopper).pos.distance(spot(t, S)) <= ON_SPOT
        });
        let began = step_until(&mut t, 600, "it lands on the station", |t| {
            landing_of(t, chopper).is_some_and(|landing| landing.at == sc_sim::landing::Spot::Structure(id(S)))
        });
        let delivered = step_until(&mut t, 600, "the delivery", |t| held(t, S) > 0);
        t.step();
        let down = t.sim().tick();
        assert_eq!(down, began + LANDING, "run {run}: down in its landing's 16 frames");
        assert_eq!(down, delivered + 1, "run {run}: down as the delivery is in the station");
        assert_eq!(
            (carried(&t, chopper), held(&t, S)),
            (DELIVERY_SIZE, 0),
            "run {run}: it takes all"
        );
        assert!(
            over < began && began - over <= 24,
            "run {run}: over it at {over}, landing at {began}"
        );
        let supplies = banks(&t, P1).1;
        step_until(&mut t, 600, "it is home", |t| home(t, chopper, tower));
        assert_eq!(
            banks(&t, P1).1,
            supplies + DELIVERY_SIZE,
            "run {run}: paid in at the Tower"
        );
    }
}

/// "If the travel distance is too long or deliveries at the SDS are too frequent, the chopper departs immediately
/// after dropping off supplies": a delivery every 480 ticks, its round trip longer; each drop-off is followed by the
/// next departure in the next tick, and each run still brings a whole delivery.
#[test]
fn too_far_or_too_frequent_the_chopper_leaves_as_soon_as_it_is_home() {
    let mut t = far_station(480);
    let (tower, chopper) = placed_tower(&mut t, TilePos::new(2, 8));
    next(&mut t, schedule(&[tower], S));
    step_until(&mut t, 2000, "it leaves", |t| outbound(t, chopper));
    for run in 1..=3 {
        let supplies = banks(&t, P1).1;
        let home_at = step_until(&mut t, 1000, "it is home", |t| home(t, chopper, tower));
        assert_eq!(banks(&t, P1).1, supplies + DELIVERY_SIZE, "run {run}");
        let left = step_until(&mut t, 10, "it leaves again", |t| outbound(t, chopper));
        assert_eq!(left, home_at + 1, "run {run}: at once");
    }
}

/// "If multiple Supply Towers have attached choppers with scheduled deliveries from the same SDS, only one chopper
/// flies to that SDS at a time. Priority: closest tower with a chopper ready to depart goes first": Towers on tiles (2,
/// 8) and (6, 8), the second the closer to the station, both scheduled on it, deliveries every 480 ticks (too frequent
/// for either alone). The closer Tower's chopper leaves first, though its id is the higher; never are both on their way
/// to the station; and both fly, in turns.
#[test]
fn two_towers_on_one_station_take_turns_the_closer_first() {
    let mut t = far_station(480);
    let (far, far_chopper) = placed_tower(&mut t, TilePos::new(2, 8));
    let (near, near_chopper) = placed_tower(&mut t, TilePos::new(6, 8));
    assert!(spot(&t, near).distance(spot(&t, S)) < spot(&t, far).distance(spot(&t, S)));
    next(&mut t, schedule(&[far, near], S));
    assert_eq!(
        (object(&t, near).schedule, object(&t, far).schedule),
        (Some(id(S)), Some(id(S)))
    );
    step_until(&mut t, 2000, "one leaves", |t| {
        outbound(t, near_chopper) || outbound(t, far_chopper)
    });
    assert!(
        outbound(&t, near_chopper) && !outbound(&t, far_chopper),
        "the closer first"
    );
    let mut departures = vec![near_chopper];
    let mut was = (true, false);
    for _ in 0..3000 {
        t.step();
        let now = (outbound(&t, near_chopper), outbound(&t, far_chopper));
        assert!(!(now.0 && now.1), "both on their way in tick {}", t.sim().tick());
        if now.0 && !was.0 {
            departures.push(near_chopper);
        }
        if now.1 && !was.1 {
            departures.push(far_chopper);
        }
        was = now;
    }
    assert!(departures.len() >= 4, "{departures:?}");
    for pair in departures.windows(2) {
        assert_ne!(pair[0], pair[1], "in turns: {departures:?}");
    }
}
