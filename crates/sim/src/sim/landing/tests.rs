//! The landing capability (`crate::landing`, [`Sim::step_landings`]): a landing and a take-off in the unit's def's
//! times, the landed rules for any landed unit, and Field landing (`systems/mechanics/aircraft_landing.md > Field
//! landing`), asked for as an order would ask (`Mover::land_at`, in each tick until the unit has landed).

use glam::Vec2;

use crate::aircraft;
use crate::combat::object_domain;
use crate::command::{Command, DebugCommand};
use crate::defs::{Domain, ObjectKind};
use crate::ids::{ObjectId, PlayerId};
use crate::landing::{self, Spot};
use crate::orders::{OrderKind, Target};
use crate::scenario::SpawnAt;
use crate::shape::Shape;
use crate::supply_run;
use crate::testing::{ScenarioBuilder, TestSim};
use crate::units::TilePos;

const P1: PlayerId = PlayerId(1);
const AIRFIELD: ObjectId = ObjectId(1);
const TOWER: ObjectId = ObjectId(2);
const HORNET: ObjectId = ObjectId(3);
const CHOPPER: ObjectId = ObjectId(4);
const OTHER_CHOPPER: ObjectId = ObjectId(5);
const PEACEKEEPER: ObjectId = ObjectId(6);
const PEACEKEEPER_AT: Vec2 = Vec2::new(1300.0, 600.0);

/// A flat map, 30 by 20 tiles: player 1's Airfield (1) on tiles (2, 2) to (4, 4), its Supply Tower (2) on tile
/// (10, 2), a Hornet (3) over the Airfield's first slot, a Supply Chopper (4) over the Tower's spot and another (5), in
/// the air, and a Peacekeeper (6) standing on open ground.
fn game() -> TestSim {
    let mut t = ScenarioBuilder::new(&format!("{}\n", ".".repeat(30)).repeat(20))
        .spawn(ObjectKind::Airfield, Some(P1), SpawnAt::Tile(TilePos::new(2, 2)))
        .spawn(ObjectKind::SupplyTower, Some(P1), SpawnAt::Tile(TilePos::new(10, 2)))
        .spawn(ObjectKind::Hornet, Some(P1), SpawnAt::Point(Vec2::new(176.0, 176.0)))
        .spawn(
            ObjectKind::SupplyChopper,
            Some(P1),
            SpawnAt::Point(Vec2::new(736.0, 224.0)),
        )
        .spawn(
            ObjectKind::SupplyChopper,
            Some(P1),
            SpawnAt::Point(Vec2::new(1100.0, 900.0)),
        )
        .spawn(ObjectKind::Peacekeeper, Some(P1), SpawnAt::Point(PEACEKEEPER_AT))
        .start();
    t.run_until(1);
    t
}

/// Steps `t` once, unit `id`'s order asking it to land at `point` on open ground in the tick.
fn step_asking(t: &mut TestSim, id: ObjectId, point: Vec2) {
    if let Some(mover) = t.sim_mut().objects.get_mut(&id).and_then(|o| o.mover.as_mut()) {
        mover.land_at(point);
    }
    t.step();
}

/// Steps `t`, `id` asking to land at `point`, until its landing begins: the tick it begins in.
fn until_landing(t: &mut TestSim, id: ObjectId, point: Vec2) -> landing::Landing {
    for _ in 0..600 {
        step_asking(t, id, point);
        if let Some(landing) = landing::of(t.sim().object(id).unwrap()) {
            return landing;
        }
    }
    panic!("#{} never lands", id.0);
}

fn landed_rules_hold(t: &TestSim, id: ObjectId) {
    let o = t.sim().object(id).unwrap();
    assert!(landing::down(o), "#{} down", id.0);
    assert!(!aircraft::may_fire(o), "#{} holds fire", id.0);
    assert_eq!(object_domain(o), Domain::Ground, "#{} a ground target", id.0);
}

/// A Supply Chopper asked to land on a free point lands on it, in its def's landing time, holds fire and is a ground
/// target down; an order lifts it off, rising in its def's take-off time, and it flies off.
#[test]
fn a_chopper_lands_on_free_open_ground_in_its_defs_times_and_takes_off() {
    let mut t = game();
    let def = landing::def_of(ObjectKind::SupplyChopper).expect("its own times");
    let point = Vec2::new(900.0, 700.0);
    let begun = until_landing(&mut t, CHOPPER, point);
    assert_eq!(begun.at, Spot::Field(point), "the point itself, free");
    assert_eq!(begun.descent.ticks_done(), 0);
    for frame in 1..=def.landing_ticks {
        assert!(
            !landing::down(t.sim().object(CHOPPER).unwrap()),
            "frame {frame}: landing still"
        );
        step_asking(&mut t, CHOPPER, point);
    }
    landed_rules_hold(&t, CHOPPER);
    let to = Vec2::new(900.0, 300.0);
    let tick = t.sim().tick() + 1;
    let kind = OrderKind::Move(Target::Point(to));
    t.at(
        tick,
        P1,
        Command::Order {
            units: vec![CHOPPER],
            kind,
            queue: false,
        },
    );
    t.step();
    let o = t.sim().object(CHOPPER).unwrap();
    assert_eq!(landing::of(o), None, "taken off");
    let rise = o.mover.as_ref().and_then(|mover| mover.rise).expect("rising");
    // A whole take-off, its first frame risen in the tick the order starts in (phase 5d).
    assert_eq!(
        (rise.total, rise.ticks_done()),
        (def.takeoff_ticks, 1),
        "a whole take-off"
    );
    for _ in 0..def.takeoff_ticks + 60 {
        t.step();
    }
    let o = t.sim().object(CHOPPER).unwrap();
    assert!(o.mover.as_ref().is_some_and(|mover| mover.rise.is_none()), "risen");
    assert!(o.pos.distance(to) < point.distance(to) - 50.0, "on its way");
}

/// A point a unit stands on is blocked: the chopper lands on the nearest free one of the search's points, the first
/// whose Silhouette clears the unit's.
#[test]
fn a_field_landing_on_a_blocked_point_takes_the_nearest_free_one() {
    let mut t = game();
    let begun = until_landing(&mut t, OTHER_CHOPPER, PEACEKEEPER_AT);
    let spot = begun.at.field().expect("on open ground");
    let (chopper, peacekeeper) = (
        t.sim().object(OTHER_CHOPPER).unwrap(),
        t.sim().object(PEACEKEEPER).unwrap(),
    );
    let blocker = Shape::of(peacekeeper);
    let clear = |point: Vec2| !Shape::silhouette(chopper.kind, point, chopper.facing).overlaps(&blocker);
    assert_ne!(spot, PEACEKEEPER_AT);
    assert_eq!(
        landing::field_points(PEACEKEEPER_AT).find(|&point| clear(point)),
        Some(spot)
    );
    assert!(spot.distance(PEACEKEEPER_AT) <= landing::FIELD_SEARCH_SU);
    for _ in 0..landing::def_of(chopper.kind).unwrap().landing_ticks {
        step_asking(&mut t, OTHER_CHOPPER, PEACEKEEPER_AT);
    }
    landed_rules_hold(&t, OTHER_CHOPPER);
}

/// A Hornet down on its Airfield's slot and a chopper down on its Tower's spot hold fire and are ground targets, and
/// each dies with its structure.
#[test]
fn landed_on_a_structure_they_hold_fire_are_ground_targets_and_die_with_it() {
    let mut t = game();
    let sim = t.sim_mut();
    let slot = landing::slots(&sim.objects[&AIRFIELD]).next().unwrap();
    let spot = supply_run::spot(&sim.objects[&TOWER]).unwrap();
    let hornet = sim.objects.get_mut(&HORNET).unwrap();
    hornet.aircraft.as_mut().unwrap().base = Some(AIRFIELD);
    assert_eq!(hornet.pos, slot, "over its slot");
    hornet.landing = Some(landing::down_on(AIRFIELD, 16));
    let chopper = sim.objects.get_mut(&CHOPPER).unwrap();
    assert_eq!(chopper.pos, spot, "over its spot");
    chopper.landing = Some(landing::down_on(TOWER, 16));
    t.step();
    landed_rules_hold(&t, HORNET);
    landed_rules_hold(&t, CHOPPER);
    let tick = t.sim().tick() + 1;
    t.debug_at(tick, DebugCommand::Remove { id: AIRFIELD });
    t.debug_at(tick, DebugCommand::Remove { id: TOWER });
    t.step();
    assert!(t.sim().object(HORNET).is_none(), "the Hornet dies with its Airfield");
    assert!(t.sim().object(CHOPPER).is_none(), "the chopper dies with its Tower");
    assert!(t.sim().object(OTHER_CHOPPER).is_some());
}
