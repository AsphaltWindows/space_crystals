//! Design change 0428 point 3 (`factions/gdo/structures/airfield.md > Assignment`): "If the Airfield is destroyed,
//! its aircraft lose their assignment [...] **Aircraft landed on it are destroyed with it**: they don't take off by
//! themselves. Designer's reading: an aircraft still **landing** (not yet down) counts as landed and is destroyed; one
//! already **taking off** escapes" (`Sim::landed_on`). The Airfield goes by a debug removal, the removal path of a
//! destruction (`Sim::remove_objects`).

use super::*;

fn remove(t: &mut TestSim, n: u32) {
    debug(t, DebugCommand::Remove { id: id(n) });
    assert!(t.sim().object(id(n)).is_none(), "#{n} gone");
}

/// 0428's Verification: "Destroy an Airfield with two Hornets parked and one taking off: the two die, the third flies
/// on, unassigned". The third's Move began in the tick before; it rises on, and flies to its point.
#[test]
fn an_airfield_destroyed_takes_its_parked_hornets_along_and_one_taking_off_escapes() {
    let mut t = field(|builder| builder);
    let points = [
        Vec2::new(760.0, 760.0),
        Vec2::new(864.0, 760.0),
        Vec2::new(968.0, 760.0),
    ];
    let [parked, parked_too, leaving] = landed_hornets(&mut t, 7, &points, A)[..] else {
        panic!("three Hornets");
    };
    let to = Vec2::new(1600.0, 900.0);
    next(&mut t, P1, order(&[leaving], OrderKind::Move(Target::Point(to))));
    assert_eq!(landing_of(t.sim(), leaving), None, "taking off");
    remove(&mut t, A);
    let tick = t.sim().tick();
    assert_eq!(
        (died(&t, parked), died(&t, parked_too)),
        (Some(tick), Some(tick)),
        "the two die with it"
    );
    assert!(t.sim().object(id(parked)).is_none() && t.sim().object(id(parked_too)).is_none());
    assert_eq!(died(&t, leaving), None);
    assert_eq!(base(t.sim(), leaving), None, "unassigned");
    assert_eq!(doing(t.sim(), leaving), Some(OrderKind::Move(Target::Point(to))));
    step_until(&mut t, 300, "there", |sim| {
        sim.object(id(leaving)).unwrap().pos.distance(to) < 2.0
    });
}

/// The designer's reading: "an aircraft still **landing** (not yet down) counts as landed and is destroyed". A Hornet
/// a frame into its landing on [`A`] dies with it; another flying home to it, not landing yet, flies on, unassigned,
/// its flight home over (`orders::air_base`: with no base, "The order ends where the unit is, at once").
#[test]
fn one_still_landing_counts_as_landed_and_one_flying_home_flies_on_unassigned() {
    let mut t = field(|builder| builder);
    t.spawn_unit_at(2, HORNET, P1, Vec2::new(816.0, 760.0));
    t.spawn_unit_at(2, HORNET, P1, Vec2::new(912.0, 2000.0));
    t.step();
    let (landing, flying) = (7, 8);
    next(&mut t, P1, order(&[landing, flying], OrderKind::Rebase(id(A))));
    step_until(&mut t, 200, "landing", |sim| landing_of(sim, landing).is_some());
    t.step();
    let frames = landing_of(t.sim(), landing).expect("landing").descent.ticks_done();
    assert_eq!(frames, 1);
    assert_eq!(landing_of(t.sim(), flying), None, "the other flies home still");
    remove(&mut t, A);
    let tick = t.sim().tick();
    assert_eq!(died(&t, landing), Some(tick), "landing, it dies with its Airfield");
    assert_eq!(died(&t, flying), None);
    assert_eq!(base(t.sim(), flying), None, "unassigned");
    t.step();
    assert_eq!(doing(t.sim(), flying), None, "its flight home over");
}
