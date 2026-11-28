//! Design change 0428 point 1 (`factions/gdo/structures/airfield.md > Returning and rearming`): "**Landed aircraft are
//! ground targets**: once an aircraft has touched down, it can be attacked by ground-targeting weapons. Air-only
//! weapons (TargetDomain Air) can't hit it. Once it takes off it is an air target again"; "**During landing and
//! take-off** (16 frames each), the aircraft is an **air target**: anti-air can hit it until it has fully landed, and
//! again from the moment it starts to take off" (`sc_sim::landing::down`, `combat::object_domain`).

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

/// The plan's acceptance, "During the 16-tick landing a Twister can hit a Hornet. Once it has landed, the Twister
/// can't, and a Peacekeeper can", and 0428's Verification, "A Hornet landing on its Airfield: an Air-only weapon can
/// hit it until it touches down, not after". Then, as the Hornet takes off, the Twister hits it again from the tick its
/// take-off begins, while it rises, and the Peacekeeper no more. The gate refuses each an Attack on the Hornet that
/// its weapon can't hit (`Reject::CannotHit`, slice M6-GF); a selection of both takes the Attack, the one that can hit
/// it fighting.
#[test]
fn a_hornet_is_an_air_target_landing_and_taking_off_and_a_ground_target_down() {
    let mut t = guarded();
    t.spawn_unit_at(2, HORNET, P1, Vec2::new(816.0, 760.0));
    t.step();
    let hornet = 9;
    next(&mut t, P1, order(&[hornet], OrderKind::Rebase(id(A))));
    let cannot = Err(Reject::CannotHit(id(hornet)));

    // Landing: the Twister hits it until it is down.
    step_until(&mut t, 200, "landing", |sim| landing_of(sim, hornet).is_some());
    let began = t.sim().tick();
    assert_eq!(
        gate(t.sim(), P2, &attack(&[PEACEKEEPER], hornet)),
        cannot,
        "landing, in the air"
    );
    assert_eq!(gate(t.sim(), P2, &attack(&[TWISTER], hornet)), Ok(()));
    next(&mut t, P2, attack(&[TWISTER], hornet));
    step_until(&mut t, LANDING, "down", |sim| down(sim, hornet));
    let down_at = t.sim().tick();
    assert_eq!(down_at, began + LANDING);
    let hits = hit_ticks(&t, TWISTER, hornet);
    assert!(!hits.is_empty(), "the Twister hits it landing");
    assert!(
        hits.iter().all(|&tick| tick > began && tick < down_at),
        "{hits:?}, down at {down_at}"
    );

    // Down: a ground target.
    assert_eq!(gate(t.sim(), P2, &attack(&[TWISTER], hornet)), cannot, "down");
    assert_eq!(gate(t.sim(), P2, &attack(&[PEACEKEEPER], hornet)), Ok(()));
    assert_eq!(gate(t.sim(), P2, &attack(&[TWISTER, PEACEKEEPER], hornet)), Ok(()));
    t.run(2 * REARM);
    assert_eq!(hit_ticks(&t, TWISTER, hornet), hits, "the Twister no more");
    next(&mut t, P2, attack(&[PEACEKEEPER], hornet));
    let until = t.sim().tick() + 32;
    while hit_ticks(&t, PEACEKEEPER, hornet).is_empty() {
        assert!(t.sim().tick() < until, "the Peacekeeper hits it down");
        t.step();
    }
    assert!(down(t.sim(), hornet), "still down");

    // Taking off: an air target again from the tick its take-off begins.
    let held = t.pos(hornet);
    next(
        &mut t,
        P1,
        order(&[hornet], OrderKind::Move(Target::Point(Vec2::new(816.0, 760.0)))),
    );
    let rose = t.sim().tick();
    assert_eq!(landing_of(t.sim(), hornet), None, "taking off");
    assert_eq!(gate(t.sim(), P2, &attack(&[TWISTER], hornet)), Ok(()));
    assert_eq!(gate(t.sim(), P2, &attack(&[PEACEKEEPER], hornet)), cannot);
    next(&mut t, P2, attack(&[TWISTER], hornet));
    while t.sim().tick() < rose + LANDING - 2 {
        t.step();
    }
    assert_eq!(t.pos(hornet), held, "rising, it holds still");
    let rising: Vec<Tick> = (hit_ticks(&t, TWISTER, hornet).into_iter())
        .filter(|&tick| tick > rose)
        .collect();
    assert!(!rising.is_empty(), "the Twister hits it rising");
    let peacekeeper = hit_ticks(&t, PEACEKEEPER, hornet);
    assert!(
        peacekeeper.iter().all(|&tick| tick < rose),
        "the Peacekeeper no more: {peacekeeper:?}"
    );
}
