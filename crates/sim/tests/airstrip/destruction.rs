//! A destroyed Airstrip (`super`; `factions/cults/structures/README.md > Building Mechanics > Evacuation on
//! Destruction`, design change 0349): everyone inside comes out alive, the orders are cancelled with a full refund;
//! and the Sparrows it trained, in the air, never needed it again. One still on its takeoff run flies on: it took off
//! along the runway, through no doorway (`Mover::exit`), so it is none of the units walking out of a destroyed
//! building's doorway with no goal, which stand where they are (slice A2a's reading, `sim/removal.rs::forget`); a
//! Glider never stands (`systems/units.md > GliderMovement`: "Unit must always maintain movement to stay airborne").

use sc_sim::mover::Locomotion;

use super::*;

/// Asserts that Sparrow `sparrow` flies in tick `tick`: Locomotion Moving, its own move in the tick its flight
/// (`Mover::velocity` is its `Mover::momentum`), at its IdleSpeed or more, and through no doorway: it doesn't stand.
fn flying(t: &TestSim, sparrow: ObjectId, tick: Tick) {
    let mover = t.mover(sparrow.0);
    assert_eq!(mover.locomotion, Locomotion::Moving, "tick {tick}: {mover:?}");
    assert_eq!(mover.exit, None, "tick {tick}: in a doorway");
    assert!(
        mover.velocity != Vec2::ZERO && mover.velocity == mover.momentum,
        "tick {tick}: {mover:?}"
    );
    let speed = speed(t, sparrow);
    assert!(speed >= IDLE_SPEED - 1e-3, "tick {tick}: {speed} su a tick");
}

/// Design change 0349 ("**Everyone inside is evacuated alive** [...] a Recruit mid-training comes out as a Recruit";
/// "**Orders are cancelled with a full refund**"), and the aircraft it let go: three Sparrows queued on four Recruits.
/// The first takes off and circles past C; the second has just taken off, still on its takeoff run, and the third is
/// mid-training when the Airstrip is destroyed (a debug removal). The third's Recruit and the free one come out at once
/// on its footprint as Recruits, the third's 75 Space Crystals back; the two Sparrows fly on, untouched, every tick
/// from the destruction on: the first circling, the second on along its run, straight, and then circling too, both
/// for good.
#[test]
fn a_destroyed_airstrip_evacuates_its_recruits_and_leaves_its_sparrows_in_the_air_alone() {
    let (mut t, order) = filled(4);
    let start = t.sim().tick() + 1;
    for _ in 0..3 {
        t.at(start, P1, train(&[A1], RecruitKind::Plain));
    }
    let (_, first) = until_produced(&mut t, FRAMES + 2);
    let (_, second) = until_produced(&mut t, FRAMES + 2);
    assert_eq!((first, second), (order[0], order[1]));
    run_for(&mut t, 5);
    assert!(t.mover(second.0).takeoff.is_some(), "on its run");
    assert_eq!(bank(&t, P1), BANK - 3 * PRICE);
    let (circling, run) = (pos(&t, first), pos(&t, second));
    assert!(circling.distance(CIRCLE) < IDLE_RADIUS + 1.0, "{circling}");
    let gone = t.sim().tick() + 1;
    remove_at(&mut t, gone, A1);
    run_checked(&mut t, gone);
    assert!(t.sim().object(A1).is_none());
    for sparrow in [first, second] {
        flying(&t, sparrow, gone);
    }
    let evacuated: Vec<ObjectId> = (t.events().iter())
        .filter_map(|(at, e)| match *e {
            SimEvent::Evacuated { unit, building } if *at == gone && building == A1 => Some(unit),
            _ => None,
        })
        .collect();
    assert_eq!(evacuated, [order[2], order[3]], "at once, longest inside first");
    for unit in [order[2], order[3]] {
        let o = t.sim().object(unit).expect("evacuated");
        assert_eq!(o.kind, RECRUIT, "{unit:?}");
        assert!(
            (768.0..=1024.0).contains(&o.pos.x) && (640.0..=768.0).contains(&o.pos.y),
            "{}",
            o.pos
        );
    }
    assert_eq!(bank(&t, P1), BANK - 2 * PRICE, "the third's Space Crystals back");
    assert!(t.mover(second.0).takeoff.is_some(), "still on its run");
    assert!(
        pos(&t, second).x < run.x && near(pos(&t, second).y, run.y, 1e-3),
        "on along it"
    );
    for _ in 0..16 * 20 {
        step_checked(&mut t);
        for sparrow in [first, second] {
            let o = t.sim().object(sparrow).expect("in the air");
            assert_eq!(o.kind, SPARROW);
            flying(&t, sparrow, t.sim().tick());
        }
    }
    for sparrow in [first, second] {
        assert!(
            near(pos(&t, sparrow).distance(CIRCLE), IDLE_RADIUS, 1.5),
            "{sparrow:?} circles past C: {}",
            pos(&t, sparrow)
        );
        assert_eq!(current(&t, sparrow), None, "{sparrow:?} idle");
    }
}

/// The Airstrip destroyed with its rally point set while its Sparrow is on its takeoff run (the coordinator's case for
/// slice A2a's fix, module docs): in the tick after the Sparrow appears, and in another game 12 ticks into its run of
/// 28. Each tick after its takeoff, the Sparrow flies, at its IdleSpeed or more; it goes on along the runway,
/// straight, until its run is over, after the destruction, then to the rally point, south-west of the Airstrip, with
/// the Move its rally gave it, which it reaches and circles, idle.
#[test]
fn a_sparrow_on_its_takeoff_run_flies_on_to_its_rally_point_when_the_airstrip_goes() {
    const RALLY_POINT: Vec2 = Vec2::new(400.0, 1000.0);
    for after in [1, 12] {
        let (mut t, order) = filled(1);
        let start = t.sim().tick() + 1;
        t.at(start, P1, rally(&[A1], RALLY_POINT));
        t.at(start, P1, train(&[A1], RecruitKind::Plain));
        let (took_off, sparrow) = until_produced(&mut t, FRAMES + 2);
        assert_eq!(sparrow, order[0]);
        // Out over the runway in this tick, through no doorway; it flies from the next (G1: one that came in this tick
        // hasn't flown yet).
        let mover = t.mover(sparrow.0);
        assert!(mover.exit.is_none() && mover.takeoff.is_some(), "{after}: {mover:?}");
        assert!(near(speed(&t, sparrow), IDLE_SPEED, 1e-3), "{after}");
        let gone = took_off + after;
        remove_at(&mut t, gone, A1);
        let mut run_over = None;
        while t.sim().tick() < took_off + 16 * 20 {
            step_checked(&mut t);
            let tick = t.sim().tick();
            assert_eq!(t.sim().object(A1).is_none(), tick >= gone, "{after}: tick {tick}");
            flying(&t, sparrow, tick);
            if t.mover(sparrow.0).takeoff.is_some() {
                let at = pos(&t, sparrow);
                assert!(
                    near(at.y, START.y, 1e-3),
                    "{after}, tick {tick}: off the runway at {at}"
                );
            } else if run_over.is_none() {
                run_over = Some(tick);
                assert_eq!(queued(&t, sparrow), [], "{after}, tick {tick}");
                let to_rally = OrderKind::Move(Target::Point(RALLY_POINT));
                assert_eq!(current(&t, sparrow), Some(to_rally), "{after}, tick {tick}");
            }
        }
        assert!(
            run_over.is_some_and(|tick| tick > gone),
            "{after}: its run went on after the destruction, until {run_over:?}"
        );
        let off = pos(&t, sparrow).distance(RALLY_POINT);
        assert!(near(off, IDLE_RADIUS, 1.5), "{after}: {off} su off the rally point");
        assert_eq!(current(&t, sparrow), None, "{after}: idle, its Move done");
    }
}
