//! The takeoff (`super`; `factions/cults/units/sparrow.md > Flight rules`, design change 0354): "**Takeoff**: it
//! appears over the runway's B end facing C at IdleSpeed and speeds up along the runway"; "**No rally point**: it
//! circles 2 grid units past the runway's C end"; `factions/cults/structures/airstrip.md > Production`: "A finished
//! aircraft **takes off from the Airstrip** to its RallyPoint".

use super::*;

/// A rally point north-east of the Airstrip, behind its B end: the Sparrow has to turn about to reach it.
const RALLY: Vec2 = Vec2::new(1400.0, 400.0);

/// Steps the Sparrow `sparrow` along the runway, from the tick it appeared in, until its takeoff run is over, checking
/// each tick: straight along it toward C, never turning, by its Acceleration of 0.5 su a tick from IdleSpeed up to
/// MaxSpeed. Returns the ticks the run took.
fn along_the_runway(t: &mut TestSim, sparrow: ObjectId) -> Tick {
    let mut ticks = 0;
    let mut was = (pos(t, sparrow), speed(t, sparrow));
    while t.mover(sparrow.0).takeoff.is_some() {
        assert!(ticks < 40, "still on its run at {}", pos(t, sparrow));
        step_checked(t);
        ticks += 1;
        let (at, now) = (pos(t, sparrow), speed(t, sparrow));
        if t.mover(sparrow.0).takeoff.is_some() {
            assert!(near(at.y, START.y, 1e-3), "tick {ticks}: {at}");
            assert!(near(heading(t, sparrow).x, -1.0, 1e-6), "tick {ticks}: it turned");
            assert!(near(now, (was.1 + 0.5).min(MAX_SPEED), 1e-3), "tick {ticks}: {now}");
            assert!(near(was.0.x - at.x, now, 1e-3), "tick {ticks}: flew {}", was.0.x - at.x);
            assert!(was.0.x > C_END.x, "tick {ticks}: on its run past C");
        }
        was = (at, now);
    }
    ticks
}

/// The takeoff to a rally point. Player 1 sets the Airstrip's rally point north-east of it and queues a Sparrow (Q),
/// which reserves the Recruit inside the longest. 320 ticks on, that Recruit comes out the Sparrow (the same object),
/// over the runway's B end, its 40 × 40 wholly over the runway and against B, facing C, flying at IdleSpeed, on its
/// takeoff run. Along the runway it flies straight toward C, speeding up to MaxSpeed; once its centre is past C its run
/// is over, 28 ticks on, and it turns about for its rally point, which it reaches (its Move completes) and circles.
#[test]
fn a_sparrow_appears_over_the_b_end_takes_off_toward_c_and_flies_to_its_rally_point() {
    let (mut t, order) = filled(2);
    let start = t.sim().tick() + 1;
    t.at(start, P1, rally(&[A1], RALLY));
    t.at(start, P1, train(&[A1], RecruitKind::Plain));
    run_checked(&mut t, start);
    assert_eq!(bank(&t, P1), BANK - PRICE);
    assert_eq!(reserved(&t, A1), [vec![order[0]]], "the Recruit inside the longest");
    let (tick, sparrow) = until_produced(&mut t, FRAMES + 2);
    assert_eq!(
        tick,
        start + FRAMES - 1,
        "trained in 320 frames, the tick of the command counting"
    );
    assert_eq!(sparrow, order[0]);
    let o = t.sim().object(sparrow).expect("out");
    assert_eq!((o.kind, o.owner), (SPARROW, Some(P1)));
    assert_eq!((o.pos, o.prev_pos), (START, START));
    assert!(near(heading(&t, sparrow).x, -1.0, 1e-6), "facing C");
    assert_eq!(t.mover(sparrow.0).momentum, Vec2::new(-IDLE_SPEED, 0.0), "at IdleSpeed");
    assert!(t.mover(sparrow.0).takeoff.is_some());
    assert_eq!(
        queued(&t, sparrow),
        [OrderKind::Move(Target::Point(RALLY))],
        "the rally's order, from the next tick"
    );
    assert_eq!(inside(&t, A1), [order[1]]);

    assert_eq!(along_the_runway(&mut t, sparrow), 28);
    assert!(pos(&t, sparrow).x < C_END.x, "past C: {}", pos(&t, sparrow));
    assert!(near(speed(&t, sparrow), MAX_SPEED, 1e-3));
    let mut turned = false;
    while current(&t, sparrow).is_some() {
        assert!(t.sim().tick() < tick + 16 * 30, "on its way at {}", pos(&t, sparrow));
        step_checked(&mut t);
        turned |= heading(&t, sparrow).x > 0.0;
    }
    assert!(turned, "it turned about for its rally point");
    assert!(pos(&t, sparrow).distance(RALLY) <= IDLE_RADIUS + 1.0);
    run_for(&mut t, 16 * 15);
    assert!(near(pos(&t, sparrow).distance(RALLY), IDLE_RADIUS, 1.0), "circling it");
    assert!(near(speed(&t, sparrow), IDLE_SPEED, 1e-3));
}

/// Design change 0354 point 4: "with no rally point, circles 2 GU past the C end". The Airstrip has no rally point: its
/// Sparrow takes off the same way, and past C it circles the point 2 grid units on from the middle of side C, along the
/// runway, at IdleSpeed on its idle circle of 1 grid unit, idle.
#[test]
fn with_no_rally_point_it_circles_2_grid_units_past_the_c_end() {
    let (mut t, order) = filled(1);
    let start = t.sim().tick() + 1;
    t.at(start, P1, train(&[A1], RecruitKind::Plain));
    let (_, sparrow) = until_produced(&mut t, FRAMES + 2);
    assert_eq!(sparrow, order[0]);
    assert_eq!(pos(&t, sparrow), START);
    assert_eq!(t.mover(sparrow.0).orbit, Some(CIRCLE));
    assert_eq!((current(&t, sparrow), queued(&t, sparrow)), (None, vec![]), "idle");
    assert_eq!(along_the_runway(&mut t, sparrow), 28);
    run_for(&mut t, 16 * 15);
    for _ in 0..16 * 10 {
        step_checked(&mut t);
        assert!(
            near(pos(&t, sparrow).distance(CIRCLE), IDLE_RADIUS, 1.0),
            "{}",
            pos(&t, sparrow)
        );
        assert!(near(speed(&t, sparrow), IDLE_SPEED, 1e-3), "{}", speed(&t, sparrow));
    }
    assert_eq!(t.mover(sparrow.0).orbit, Some(CIRCLE));
}

/// "It never pauses" (the plan's N7): `core/entities.md > Structure Type > Exit Doorway`: "Aircraft and units released
/// underground ... are not covered by this rule". Mountain walls off side C, so the Recruit a Release lets out stays in
/// the doorway, and the Recruits Eject All lets out wait inside for it, the doorway occupied. A Sparrow trained behind
/// the Release takes off all the same, in the tick its training finishes, over the Recruit in the doorway and the
/// Mountain, and flies on; the Recruits still wait.
#[test]
fn a_sparrow_takes_off_while_its_recruits_wait_at_the_doorway() {
    let mut rows: Vec<Vec<u8>> = (0..22).map(|_| vec![b'.'; 30]).collect();
    rows[10][11] = b'M';
    rows[11][11] = b'M';
    let rows: Vec<String> = rows.into_iter().map(|row| String::from_utf8(row).unwrap()).collect();
    let (mut t, order) = filled_on(on(&rows.join("\n")), 4);
    let start = t.sim().tick() + 1;
    t.at(start, P1, release(&[A1], RecruitKind::Plain));
    t.at(start, P1, train(&[A1], RecruitKind::Plain));
    run_checked(&mut t, start + 8);
    assert_eq!(exits(&t).iter().map(|&(_, unit)| unit).collect::<Vec<_>>(), [order[0]]);
    let in_doorway = t.sim().object(order[0]).expect("out");
    assert_eq!(
        in_doorway.mover.as_ref().and_then(|mover| mover.exit),
        Some(A1),
        "in the doorway"
    );
    t.at(start + 9, P1, eject_all(&[A1]));
    let (tick, sparrow) = until_produced(&mut t, FRAMES + 8);
    assert_eq!(
        tick,
        start + 4 + FRAMES - 1,
        "the Release's 4 frames, then its 320, the command's tick counting"
    );
    assert_eq!(sparrow, order[1]);
    assert_eq!(pos(&t, sparrow), START);
    assert_eq!(inside(&t, A1), [order[2], order[3]], "ejected, waiting for the doorway");
    along_the_runway(&mut t, sparrow);
    run_for(&mut t, 16 * 5);
    assert_eq!(inside(&t, A1), [order[2], order[3]], "still waiting");
    assert_eq!(exits(&t).len(), 1);
    assert!(
        pos(&t, sparrow).distance(CIRCLE) < 3.0 * IDLE_RADIUS,
        "{}",
        pos(&t, sparrow)
    );
}
