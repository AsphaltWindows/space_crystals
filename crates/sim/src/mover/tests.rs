use super::*;

#[test]
fn go_to_wants_a_path_and_stop_drops_it() {
    let mut mover = Mover {
        path: vec![Vec2::ZERO, Vec2::ONE],
        next: 1,
        path_state: PathState::Ready,
        velocity: Vec2::X,
        ..Mover::default()
    };
    assert_eq!(mover.remaining_path(), [Vec2::ONE]);
    let goal = MoveGoal::Point(Vec2::new(5.0, 6.0));
    mover.go_to(goal, None, 7);
    assert_eq!((mover.goal, mover.status), (goal, MoveStatus::Moving));
    assert_eq!(mover.path_state, PathState::Wanted { since: 7, group: None });
    assert!(mover.path.is_empty() && mover.remaining_path().is_empty());
    mover.stop();
    assert_eq!(
        (mover.goal, mover.status, mover.path_state),
        (MoveGoal::None, MoveStatus::Idle, PathState::None)
    );
    assert_eq!(mover.velocity, Vec2::ZERO);
    mover.finish(MoveStatus::Arrived);
    assert_eq!(mover.status, MoveStatus::Arrived);
    // Without a goal there is nothing to plan.
    mover.repath(9);
    assert_eq!(mover.path_state, PathState::None);
    assert!(!mover.counted_repath(9));
}

/// Design change 0024, `docs/movement.md > 5.6`: three repaths per leg, after waits of 8, 16 and 32 ticks.
#[test]
fn counted_repaths_are_capped_and_wait() {
    let mut mover = Mover::default();
    mover.go_to(MoveGoal::Point(Vec2::ONE), None, 10);
    let mut at = Vec::new();
    for tick in 10..200 {
        if mover.counted_repath(tick) {
            at.push(tick);
            assert_eq!(
                mover.path_state,
                PathState::Wanted {
                    since: tick,
                    group: None
                }
            );
        }
    }
    assert_eq!(
        at,
        [18, 34, 66],
        "8, 16 and 32 ticks after the leg began or the repath before"
    );
    assert_eq!((mover.last_repath, mover.stuck_clock()), (66, 66));
    mover.follow(vec![Vec2::ZERO, Vec2::ONE], Vec2::ONE, 0);
    assert!(!mover.is_held_up(66 + HALF_STUCK_TICKS - 1) && mover.is_held_up(66 + HALF_STUCK_TICKS));
    mover.last_progress = 70;
    assert_eq!(mover.stuck_clock(), 70);
    assert_eq!(mover.repaths_left, 0);
    // A new leg starts afresh.
    mover.go_to(MoveGoal::Point(Vec2::ONE), None, 200);
    assert_eq!((mover.repaths_left, mover.may_repath(208)), (MAX_REPATHS, true));
    assert_eq!(mover.stuck_clock(), 200);
}

/// Design change 0024: stuck after 24 ticks without getting closer; another route after waits of 8, 16 and 32
/// ticks; then it gives up, 104 ticks (6.5 s, "about 7 s") after its last progress.
#[test]
fn a_unit_that_never_gets_on_follows_the_schedule_of_0024() {
    let mut mover = Mover::default();
    mover.go_to(MoveGoal::Point(Vec2::ONE), None, 100);
    assert!(!mover.is_stuck(123) && mover.is_stuck(124));
    let mut due = Vec::new();
    for _ in 0..MAX_REPATHS {
        let tick = mover.recovery_due();
        assert!(mover.counted_repath(tick));
        due.push(tick - 100);
    }
    assert_eq!(due, [32, 48, 80]);
    assert_eq!(mover.recovery_due() - 100, GIVE_UP_TICKS);
    assert_eq!(GIVE_UP_TICKS, 104);
    // Progress in between: stuck again 24 ticks later, and the next attempt waits from then.
    let mut mover = Mover::default();
    mover.go_to(MoveGoal::Point(Vec2::ONE), None, 100);
    assert!(mover.counted_repath(132));
    mover.last_progress = 150;
    assert_eq!(mover.recovery_due(), 150 + 24 + 16);
}

/// Design change 0023: modes that slow a unit leave it able to move; a unit that can't move is the hook's
/// `false`.
#[test]
fn a_unit_can_move_unless_its_mode_stops_it() {
    let mut mover = Mover::default();
    assert!(mover.can_move(4.0));
    mover.speed_factor = 0.5;
    assert!(mover.can_move(4.0), "a raised Riot Shield only slows it");
    mover.speed_factor = 0.0;
    assert!(!mover.can_move(4.0));
    assert_eq!(
        Sidestep::new(Vec2::X, 24.0, 2.0, 0, None).ticks_left,
        12 + YIELD_WAIT_TICKS
    );
}

#[test]
fn a_new_path_measures_afresh_but_the_stuck_clock_runs_on() {
    let mut mover = Mover::default();
    mover.go_to(MoveGoal::Point(Vec2::X), None, 1);
    mover.best_remaining = 3.0;
    mover.blocked_ticks = JAMMED_TICKS;
    mover.locomotion = Locomotion::Moving;
    assert!(mover.is_stationary());
    mover.follow(vec![Vec2::ZERO, Vec2::X], Vec2::X, 2);
    assert_eq!((mover.best_remaining, mover.last_progress), (f32::MAX, 1));
    assert!(!mover.is_stationary());
    assert_eq!(mover.path_end(), Some(Vec2::X));
}

/// (M3-G2) `Mover::go_round`: a detour round standing units goes in front of what is left of the path, counts
/// against the leg's cap and waits like a repath, but asks for no path. A free one (the unit lost its place to a
/// unit that got there first) doesn't count. What is left of an earlier detour goes; a new path ends it.
#[test]
fn a_detour_goes_in_front_of_the_path_and_counts_like_a_repath() {
    let v = Vec2::new;
    let mut mover = Mover::default();
    mover.go_to(MoveGoal::Point(v(400.0, 0.0)), None, 10);
    mover.follow(vec![v(0.0, 0.0), v(200.0, 0.0), v(400.0, 0.0)], v(400.0, 0.0), 0);
    let at = v(100.0, 0.0);
    // The wait before a leg's first attempt is 8 ticks.
    assert!(!mover.go_round(at, &[v(100.0, 30.0)], 17, false));
    assert!(mover.go_round(at, &[v(100.0, 30.0), v(150.0, 30.0)], 18, false));
    assert_eq!(
        mover.path,
        [at, v(100.0, 30.0), v(150.0, 30.0), v(200.0, 0.0), v(400.0, 0.0)]
    );
    assert_eq!((mover.next, mover.detour_end), (1, 3));
    assert_eq!((mover.repaths_left, mover.last_repath), (MAX_REPATHS - 1, 18));
    assert_eq!(mover.path_state, PathState::Ready, "no path is asked for");
    assert_eq!(mover.best_remaining, f32::MAX, "progress is measured afresh");
    // A free one, at the first waypoint of that detour: the rest of the old detour goes, and no attempt is used.
    assert!(
        !mover.go_round(v(100.0, 30.0), &[v(90.0, 60.0)], 33, true),
        "the wait is 16 ticks now"
    );
    assert!(mover.go_round(v(100.0, 30.0), &[v(90.0, 60.0)], 34, true));
    assert_eq!(
        mover.path,
        [v(100.0, 30.0), v(90.0, 60.0), v(200.0, 0.0), v(400.0, 0.0)]
    );
    assert_eq!(
        (mover.next, mover.detour_end, mover.repaths_left),
        (1, 2, MAX_REPATHS - 1)
    );
    // With nothing to go round by, only the attempt is counted.
    assert!(mover.go_round(v(100.0, 30.0), &[], 50, false));
    assert_eq!(
        (mover.path.len(), mover.repaths_left, mover.last_repath),
        (4, MAX_REPATHS - 2, 50)
    );
    // The last one; then only a free attempt is left.
    assert!(mover.go_round(v(100.0, 30.0), &[], 82, false));
    assert!(!mover.may_go_round(200, false) && mover.may_go_round(200, true));
    // A new path has no detour.
    mover.follow(vec![v(0.0, 0.0), v(400.0, 0.0)], v(400.0, 0.0), 0);
    assert_eq!(mover.detour_end, 0);
}
