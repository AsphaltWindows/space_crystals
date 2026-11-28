//! `tests/landing`'s rules (slice M6-GH2), for a Dragonfly: design change 0428 point 1
//! (`systems/mechanics/aircraft_landing.md > Landed aircraft (every archetype)`), "**Landed aircraft are ground
//! targets**", and "**During landing and take-off** (16 frames each), the aircraft is an **air target**"; point 3
//! (`systems/mechanics/aircraft_landing.md > Assigned base`), "**Aircraft landed on it are destroyed with it** [...] an
//! aircraft still **landing** (not yet down) counts as landed and is destroyed; one already **taking off** escapes";
//! point 4, "**An order during rearming**: the aircraft takes off at once **with the ammo rearmed so far**".

use super::*;

/// Player 2's Twister (7), whose machine gun hits air units only (`factions/gdo/units/twister.md`: "TargetDomain:
/// **Air** (anti-air only)"), and Peacekeeper (8), whose rifle hits ground units only ("TargetDomain: Ground"), both
/// within their Range of [`A`]'s first slot.
fn guarded() -> TestSim {
    field(|builder| {
        builder.spawn(ObjectKind::Twister, Some(P2), at(700.0, 100.0)).spawn(
            ObjectKind::Peacekeeper,
            Some(P2),
            at(720.0, 240.0),
        )
    })
}

const TWISTER: u32 = 7;
const PEACEKEEPER: u32 = 8;

fn attack(units: &[u32], target: u32) -> Command {
    order(units, OrderKind::Attack(id(target)))
}

fn remove(t: &mut TestSim, n: u32) {
    debug(t, DebugCommand::Remove { id: id(n) });
    assert!(t.sim().object(id(n)).is_none(), "#{n} gone");
}

/// A Dragonfly landing on [`A`] is an air target, as a Hornet is (`tests/landing`'s `domains`): over its slot and
/// landing, the Twister can hit it, and the Peacekeeper can't; down, the Peacekeeper can and the Twister can't; and
/// rising off it, the Twister again from the tick its take-off begins.
#[test]
fn a_dragonfly_is_an_air_target_landing_and_taking_off_and_a_ground_target_down() {
    let mut t = guarded();
    t.spawn_unit_at(2, DRAGONFLY, P1, Vec2::new(816.0, 760.0));
    t.step();
    let n = 9;
    next(&mut t, P1, order(&[n], OrderKind::Rebase(id(A))));
    let cannot = Err(Reject::CannotHit(id(n)));

    // Landing: the Twister hits it until it is down.
    step_until(&mut t, 300, "landing", |sim| landing_of(sim, n).is_some());
    let began = t.sim().tick();
    assert_eq!(
        gate(t.sim(), P2, &attack(&[PEACEKEEPER], n)),
        cannot,
        "landing, in the air"
    );
    next(&mut t, P2, attack(&[TWISTER], n));
    step_until(&mut t, LANDING, "down", |sim| down(sim, n));
    let down_at = t.sim().tick();
    assert_eq!(down_at, began + LANDING);
    let hits = hit_ticks(&t, TWISTER, n);
    assert!(!hits.is_empty(), "the Twister hits it landing");
    assert!(
        hits.iter().all(|&tick| tick > began && tick < down_at),
        "{hits:?}, down at {down_at}"
    );

    // Down: a ground target.
    assert_eq!(gate(t.sim(), P2, &attack(&[TWISTER], n)), cannot, "down");
    assert_eq!(gate(t.sim(), P2, &attack(&[PEACEKEEPER], n)), Ok(()));
    t.run(32);
    assert_eq!(hit_ticks(&t, TWISTER, n), hits, "the Twister no more");
    next(&mut t, P2, attack(&[PEACEKEEPER], n));
    let until = t.sim().tick() + 32;
    while hit_ticks(&t, PEACEKEEPER, n).is_empty() {
        assert!(t.sim().tick() < until, "the Peacekeeper hits it down");
        t.step();
    }
    assert!(down(t.sim(), n), "still down");

    // Taking off: an air target again from the tick its take-off begins.
    let slot = t.pos(n);
    next(&mut t, P1, move_to(&[n], Vec2::new(816.0, 760.0)));
    let rose = t.sim().tick();
    assert_eq!(gate(t.sim(), P2, &attack(&[TWISTER], n)), Ok(()));
    assert_eq!(gate(t.sim(), P2, &attack(&[PEACEKEEPER], n)), cannot);
    next(&mut t, P2, attack(&[TWISTER], n));
    while t.sim().tick() < rose + LANDING - 2 {
        t.step();
    }
    assert_eq!(t.pos(n), slot, "rising, it holds still");
    let rising: Vec<Tick> = (hit_ticks(&t, TWISTER, n).into_iter())
        .filter(|&tick| tick > rose)
        .collect();
    assert!(!rising.is_empty(), "the Twister hits it rising");
    let peacekeeper = hit_ticks(&t, PEACEKEEPER, n);
    assert!(
        peacekeeper.iter().all(|&tick| tick < rose),
        "the Peacekeeper no more: {peacekeeper:?}"
    );
}

/// 0428's Verification, for Dragonflies: "Destroy an Airfield with two Hornets parked and one taking off: the two die,
/// the third flies on, unassigned". The third's Move began in the tick before; it rises on, and flies to its point,
/// where it circles.
#[test]
fn an_airfield_destroyed_takes_its_parked_dragonflies_along_and_one_taking_off_escapes() {
    let mut t = field(|builder| builder);
    let points = [
        Vec2::new(760.0, 760.0),
        Vec2::new(864.0, 760.0),
        Vec2::new(968.0, 760.0),
    ];
    let [parked, parked_too, leaving] = landed_dragonflies(&mut t, 7, &points, A)[..] else {
        panic!("three Dragonflies");
    };
    let to = Vec2::new(1600.0, 900.0);
    next(&mut t, P1, move_to(&[leaving], to));
    assert_eq!(landing_of(t.sim(), leaving), None, "taking off");
    remove(&mut t, A);
    let tick = t.sim().tick();
    assert_eq!(
        (died(&t, parked), died(&t, parked_too)),
        (Some(tick), Some(tick)),
        "the two die with it"
    );
    assert_eq!(died(&t, leaving), None);
    assert_eq!(base(t.sim(), leaving), None, "unassigned");
    assert_eq!(doing(t.sim(), leaving), Some(OrderKind::Move(Target::Point(to))));
    step_until(&mut t, 400, "circling its point", |sim| {
        mover(sim, leaving).orbit == Some(to)
    });
}

/// The designer's reading: "an aircraft still **landing** (not yet down) counts as landed and is destroyed". A
/// Dragonfly a frame into its landing on [`A`] dies with it; another flying its approach to it flies on, unassigned,
/// its flight home over (`orders::air_base`: with no base, "The order ends where the unit is, at once"), circling
/// where it is, never stopping in the air.
#[test]
fn one_still_landing_counts_as_landed_and_one_on_its_approach_flies_on_unassigned() {
    let mut t = field(|builder| builder);
    t.spawn_unit_at(2, DRAGONFLY, P1, Vec2::new(816.0, 760.0));
    t.spawn_unit_at(2, DRAGONFLY, P1, Vec2::new(912.0, 1500.0));
    t.step();
    let (landing, flying) = (7, 8);
    next(&mut t, P1, order(&[landing, flying], OrderKind::Rebase(id(A))));
    step_until(&mut t, 300, "landing", |sim| landing_of(sim, landing).is_some());
    t.step();
    let frames = landing_of(t.sim(), landing).expect("landing").descent.ticks_done();
    assert_eq!(frames, 1);
    let approach = mover(t.sim(), flying).approach;
    assert_eq!(
        approach,
        Some(sc_sim::mover::Approach::Flying),
        "the other on its approach"
    );
    remove(&mut t, A);
    let tick = t.sim().tick();
    assert_eq!(died(&t, landing), Some(tick), "landing, it dies with its Airfield");
    assert_eq!(died(&t, flying), None);
    assert_eq!(base(t.sim(), flying), None, "unassigned");
    t.step();
    assert_eq!(doing(t.sim(), flying), None, "its flight home over");
    assert_eq!(mover(t.sim(), flying).approach, None);
    for _ in 0..80 {
        t.step();
        flies_or_is_held(frame(t.sim(), flying));
        assert!(!frame(t.sim(), flying).over);
    }
    let orbit = mover(t.sim(), flying).orbit.expect("circling");
    assert!(t.pos(flying).distance(orbit) < 2.0 * 64.0, "where it was");
}

/// Design change 0428 point 4 for a Dragonfly: emptied on a Sparrow, home and down on its slot, it is given a Move 32
/// frames into its 64-frame rearm: it takes off at once with the ammo rearmed so far, none, the round under way lost.
/// Sent home again (F), it lands and rearms its missile in 64 frames from the start.
#[test]
fn an_order_during_its_rearm_lifts_it_off_with_the_ammo_rearmed_so_far() {
    let mut t = field(|builder| builder.spawn(ObjectKind::Sparrow, Some(P2), at(1100.0, 500.0)));
    let n = 8;
    one_landed(&mut t, n);
    shoots_down(&mut t, n, 7);
    fly_down(&mut t, n, 600);
    let down_at = t.sim().tick();
    // Its round begun in the frame it came down.
    t.run(REARM / 2 - 1);
    let rearm = landing_of(t.sim(), n)
        .and_then(|landing| landing.rearm)
        .expect("rearming");
    assert_eq!(rearm.ticks_done(), REARM / 2);
    assert!(t.sim().tick() < down_at + REARM);
    takes_off(&mut t, n, move_to(&[n], Vec2::new(1600.0, 900.0)));
    assert_eq!(ammo(t.sim(), n), 0, "none rearmed");

    next(&mut t, P1, order(&[n], OrderKind::ReturnToBase));
    fly_down(&mut t, n, 600);
    let down_again = t.sim().tick();
    step_until(&mut t, 2 * REARM, "rearmed", |sim| ammo(sim, n) == AMMO);
    assert_eq!(t.sim().tick() - down_again + 1, REARM, "64 frames from the start");
}
