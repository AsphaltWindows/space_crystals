//! The turns (`factions/syndicate/structures/null_chamber.md > Placeholder stats`: "each Null Chamber buffs **4 units
//! at a time**, each buff taking **2 s** inside before the unit can leave buffed"; `network::services`): a unit's turn
//! from the tick it comes in, four at a time for each chamber, first come first served in the order the units came in
//! (D48), a unit trained inside from its production, and a turn done kept while the unit stays.

use sc_sim::Target;

use super::*;

/// Player 1's Hideout in [`with_hideout`], after [`rows`]' Tunnels and chamber.
const ROW_HIDEOUT: ObjectId = ObjectId(7);

/// [`rows`] of five Tunnels and one chamber (id 6), with player 1's Hideout ([`ROW_HIDEOUT`]) on (18, 14) to (19, 15)
/// and its Optics Lab (id 8), the Guard's prerequisite, on (24, 14) to (25, 15).
fn with_hideout() -> ScenarioBuilder {
    rows(5, 1)
        .spawn(ObjectKind::Hideout, Some(P1), tile(18, 14))
        .spawn(ObjectKind::OpticsLab, Some(P1), tile(24, 14))
}

/// The plan's six Guards in together, one chamber in the game: each by a Tunnel of its own, from the same place before
/// it, so that all six go in in one tick, which has run. The game, the Guards and that tick.
fn six_guards() -> (TestSim, Vec<ObjectId>, Tick) {
    let mut t = started(rows(6, 1), 1);
    let guards = row_guards(&mut t, &[42.0; 6]);
    let came_in: Vec<Tick> = guards.iter().map(|&guard| entered(&t, guard)).collect();
    assert!(
        came_in.iter().all(|&tick| tick == came_in[0]),
        "in together: {came_in:?}"
    );
    assert_eq!(t.sim().tick(), came_in[0]);
    (t, guards, came_in[0])
}

/// A Guard that walked in takes a tick of its turn in each tick from the one it went in (the services' step comes
/// after the units' entries, `Sim::step_network`), and is done 2 s later, in its 32nd tick inside: the tick before it
/// may leave. Done, it keeps its turn while it stays, however long, and takes no slot.
#[test]
fn a_guard_takes_its_turn_from_the_tick_it_goes_in_and_is_done_2_s_later() {
    let mut t = started(with_chamber(), 1);
    let guard = guards_in(&mut t, 1)[0];
    let came_in = entered(&t, guard);
    assert_eq!(
        (t.sim().tick(), turn(&t, guard)),
        (came_in, Some(1)),
        "from the tick it went in"
    );
    for done in 2..=TURN {
        step_checked(&mut t);
        let held = t.sim().held_unit(guard).expect("inside");
        let tick = t.sim().tick();
        assert_eq!(turn(&t, guard), Some(done), "tick {tick}");
        assert_eq!(held.service.charged(), done == TURN, "tick {tick}");
        assert_eq!(held.may_exit(tick + 1), done == TURN, "tick {tick}: the transit delay");
    }
    assert_eq!(t.sim().tick(), came_in + TURN - 1, "2 s");
    run_checked(&mut t, came_in + 10 * TURN);
    assert_eq!(turn(&t, guard), Some(TURN), "kept");
}

/// The plan's acceptance: "Six Guards enter together, with one chamber. After 2 s four are done, and after 4 s all
/// six." In together, the four first by id take the four slots ("first come first served", ties by id); at 2 s they are
/// done and the other two begin, done at 4 s.
#[test]
fn six_guards_with_one_chamber_four_are_done_after_2_s_and_all_six_after_4_s() {
    let (mut t, guards, came_in) = six_guards();
    let one = Some(1);
    assert_eq!(turns(&t, &guards), [one, one, one, one, None, None]);
    run_checked(&mut t, came_in + TURN - 2);
    let almost = Some(TURN - 1);
    assert_eq!(turns(&t, &guards), [almost, almost, almost, almost, None, None]);
    step_checked(&mut t);
    let done = Some(TURN);
    assert_eq!(turns(&t, &guards), [done, done, done, done, None, None], "2 s");
    step_checked(&mut t);
    assert_eq!(turns(&t, &guards), [done, done, done, done, one, one]);
    run_checked(&mut t, came_in + 2 * TURN - 2);
    assert_eq!(turns(&t, &guards), [done, done, done, done, almost, almost]);
    step_checked(&mut t);
    assert_eq!(turns(&t, &guards), [done; 6], "4 s");
}

/// The plan's acceptance: "Ejected at 3 s, four leave buffed and two unbuffed." Each ejected by its own Tunnel, the six
/// come out together: the four done leave buffed for 20 s, the two half way through their turns unbuffed ("a unit that
/// leaves before its turn leaves unbuffed").
#[test]
fn six_guards_ejected_at_3_s_four_leave_buffed_and_two_unbuffed() {
    let (mut t, guards, came_in) = six_guards();
    let out = came_in + 3 * 16;
    for k in 0..6 {
        t.at(out, P1, eject(row_tunnel(k), GUARD));
    }
    run_checked(&mut t, out);
    assert_eq!(inside(&t), []);
    let out_at: Vec<Tick> = guards.iter().map(|&guard| left(&t, guard)).collect();
    assert_eq!(out_at, [out; 6], "out together");
    assert_eq!(buffed(&t, &guards), [true, true, true, true, false, false]);
    for &guard in &guards[..4] {
        assert_eq!(buff_until(&t, guard), Some(out + DURATION), "{guard:?}");
    }
}

/// First come first served by the tick each came in (D48: "the queue runs in the order units entered the network"),
/// not by id: five Guards, each a few ticks behind the one before, the lowest id the last in. The first four in take
/// the four slots, each from its own tick; the last waits, its id the lowest, until the first in is done, and takes its
/// slot in the tick after.
#[test]
fn the_first_in_take_the_slots_whatever_their_ids() {
    let mut t = started(rows(5, 1), 1);
    let guards = row_guards(&mut t, &[82.0, 72.0, 62.0, 52.0, 42.0]);
    let came_in: Vec<Tick> = guards.iter().map(|&guard| entered(&t, guard)).collect();
    assert!(
        came_in.windows(2).all(|w| w[0] > w[1]),
        "the lowest id the last in: {came_in:?}"
    );
    let first = came_in[4];
    assert!(
        came_in[0] < first + TURN - 1,
        "the last in before the first is done: {came_in:?}"
    );
    run_checked(&mut t, first + TURN - 1);
    let now = t.sim().tick();
    let expected: Vec<Option<u32>> = (came_in.iter())
        .map(|&tick| (tick != came_in[0]).then_some(now - tick + 1))
        .collect();
    assert_eq!(turns(&t, &guards), expected, "the first in done; the last waits");
    step_checked(&mut t);
    assert_eq!(turn(&t, guards[0]), Some(1), "in the first one's slot");
    run_checked(&mut t, first + 2 * TURN - 1);
    assert_eq!(turn(&t, guards[0]), Some(TURN));
}

/// A unit trained into the network (no rally point) takes its turn from the tick it is finished, as one that walked in
/// does from the tick it went in, and leaves buffed 2 s later; one trained with a surface rally point comes out in
/// the tick it is finished (`sim/network/produced.rs`), before its turn: unbuffed.
#[test]
fn a_unit_trained_into_the_network_takes_its_turn_from_the_tick_it_is_finished() {
    let mut t = started(with_hideout(), 1);
    t.at(2, P1, train(ROW_HIDEOUT, GUARD));
    let finished = 2 + GUARD_FRAMES - 1;
    run_checked(&mut t, finished);
    let [(tick, inside_one)] = produced(&t, ROW_HIDEOUT)[..] else {
        panic!("one Guard: {:?}", produced(&t, ROW_HIDEOUT));
    };
    assert_eq!(tick, finished);
    assert_eq!(inside(&t), [inside_one]);
    assert_eq!(turn(&t, inside_one), Some(1), "from the tick it is finished");
    run_checked(&mut t, finished + TURN - 1);
    assert_eq!(turn(&t, inside_one), Some(TURN));
    let out = finished + TURN;
    t.at(out, P1, eject(row_tunnel(0), GUARD));
    run_checked(&mut t, out);
    assert_eq!(buff_until(&t, inside_one), Some(out + DURATION), "buffed");
    // With a rally point on the surface.
    let rally = Command::Production {
        objects: vec![ROW_HIDEOUT],
        command: ProductionCommand::SetRally(Target::Point(before_row(2, 200.0))),
    };
    t.at(out + 1, P1, rally);
    t.at(out + 1, P1, train(ROW_HIDEOUT, GUARD));
    let finished = out + GUARD_FRAMES;
    run_checked(&mut t, finished);
    let rallied = produced(&t, ROW_HIDEOUT)[1];
    assert_eq!(rallied.0, finished);
    assert_eq!(left(&t, rallied.1), finished, "out in the tick it is finished");
    assert_eq!(buff_until(&t, rallied.1), None, "unbuffed");
}

/// A unit trained into the network has its place in the queue from the tick it is finished (D48: "a produced unit from
/// its production"): four Guards go in a tick apart and take the four slots; the Hideout finishes a Guard while they
/// take their turns, and a fifth Guard goes in after it. When the first in is done its slot goes to the trained one,
/// ahead of the fifth, whose id is the lower; the second's, a tick later, to the fifth.
#[test]
fn a_unit_trained_into_the_network_queues_from_the_tick_it_is_finished() {
    let mut t = started(with_hideout(), 1);
    let first = next_id(&t);
    let walkers: Vec<ObjectId> = (0..5).map(|k| id(first + k)).collect();
    for k in 0..5 {
        t.spawn_unit_at(2, GUARD, P1, before_row(k, 42.0));
    }
    t.at(2, P1, train(ROW_HIDEOUT, GUARD));
    // The four in a tick apart, from tick 95 on; the fifth sent in once the Guard is trained.
    let finished = 2 + GUARD_FRAMES - 1;
    for k in 0..4 {
        t.at(
            95 + k,
            P1,
            order(&[walkers[k as usize]], OrderKind::Enter(row_tunnel(k))),
        );
    }
    t.at(finished + 1, P1, order(&[walkers[4]], OrderKind::Enter(row_tunnel(4))));
    step_until(&mut t, 200, "all five in", |t| {
        walkers.iter().all(|&walker| t.sim().held_unit(walker).is_some())
    });
    let [(tick, trained)] = produced(&t, ROW_HIDEOUT)[..] else {
        panic!("one Guard: {:?}", produced(&t, ROW_HIDEOUT));
    };
    assert_eq!(tick, finished);
    assert!(trained.0 > walkers[4].0, "{trained:?} after {:?}", walkers[4]);
    let came_in: Vec<Tick> = walkers.iter().map(|&walker| entered(&t, walker)).collect();
    let e0 = came_in[0];
    assert_eq!(came_in[..4], [e0, e0 + 1, e0 + 2, e0 + 3], "a tick apart");
    assert!(
        e0 + 3 < finished && finished < came_in[4],
        "the trained one between: {came_in:?}"
    );
    assert!(
        came_in[4] < e0 + TURN - 1,
        "all in before the first is done: {came_in:?}"
    );
    run_checked(&mut t, e0 + TURN - 1);
    assert_eq!(turns(&t, &[trained, walkers[4]]), [None, None], "the four slots taken");
    assert_eq!(turn(&t, walkers[0]), Some(TURN));
    step_checked(&mut t);
    assert_eq!(turns(&t, &[trained, walkers[4]]), [Some(1), None], "the first slot");
    step_checked(&mut t);
    assert_eq!(turns(&t, &[trained, walkers[4]]), [Some(2), Some(1)], "the second");
}

/// Each chamber adds its four slots to the network's ("each Null Chamber buffs **4 units at a time**"): with two, nine
/// Guards in together, eight take their turns at once, and the ninth begins once they are done.
#[test]
fn two_chambers_take_eight_at_a_time() {
    let mut t = started(rows(9, 2), 1);
    let guards = row_guards(&mut t, &[42.0; 9]);
    let came_in = entered(&t, guards[0]);
    assert!(guards.iter().all(|&guard| entered(&t, guard) == came_in), "in together");
    run_checked(&mut t, came_in + TURN - 1);
    let mut expected = vec![Some(TURN); 8];
    expected.push(None);
    assert_eq!(turns(&t, &guards), expected);
    run_checked(&mut t, came_in + 2 * TURN - 1);
    assert_eq!(turns(&t, &guards), [Some(TURN); 9]);
}

/// A chamber serves while its owner owns it: lost (removed outright, as if destroyed), the units inside wait, keeping
/// what of their turns is done, and one whose turn is done keeps it and leaves buffed though its owner owns none: design
/// change 0463's Verification 2 (answering Q0187; `> Placeholder stats`: "**A finished turn always buffs**"): "Let a
/// unit finish its turn, destroy the player's only Null Chamber, then eject the unit. It comes out buffed", for 20 s
/// from the tick it comes out. A chamber again, the waiting go on from where they were, and the next one done leaves
/// buffed with no new turn.
#[test]
fn a_turn_done_is_kept_and_leaves_buffed_with_no_chamber_left() {
    let mut t = started(with_chamber(), 1);
    let guards = guards_in(&mut t, 6);
    // The first four done, the other two under way.
    step_until(&mut t, 100, "the first four done", |t| {
        turns(t, &guards[..4]).iter().all(|&turn| turn == Some(TURN))
    });
    let (fifth, sixth) = (turn(&t, guards[4]), turn(&t, guards[5]));
    let under_way = |turn: Option<u32>| turn.is_some_and(|done| done < TURN);
    assert!(under_way(fifth) && under_way(sixth), "{fifth:?}, {sixth:?}");
    let lost = t.sim().tick() + 1;
    t.debug_at(lost, DebugCommand::Remove { id: id(2) });
    t.at(lost + 1, P1, eject(T1, GUARD));
    run_checked(&mut t, lost + 40);
    assert!(t.sim().object(id(2)).is_none(), "the chamber is gone");
    assert_eq!(turns(&t, &guards[4..]), [fifth, sixth], "they wait");
    let first_out = guards[0];
    assert!(t.sim().object(first_out).is_some(), "out: {:?}", exits(&t));
    let out = left(&t, first_out);
    assert!(out > lost, "out after the chamber was lost");
    assert_eq!(
        buff_until(&t, first_out),
        Some(out + DURATION),
        "a finished turn always buffs"
    );
    assert_eq!(turns(&t, &guards[1..4]), [Some(TURN); 3], "kept");
    // A chamber again, serving from the tick it is spawned in.
    let again = t.sim().tick() + 1;
    t.debug_at(again, spawn(CHAMBER, P1, SpawnAt::Tile(CHAMBER_AT)));
    t.at(again + 1, P1, eject(T1, GUARD));
    run_checked(&mut t, again);
    let resumed = |turn: Option<u32>| turn.map(|done| done + 1);
    assert_eq!(
        turns(&t, &guards[4..]),
        [resumed(fifth), resumed(sixth)],
        "from where they were"
    );
    step_until(&mut t, 40, "the second out", |t| t.sim().object(guards[1]).is_some());
    let out = left(&t, guards[1]);
    assert_eq!(out, again + 1);
    assert_eq!(
        buff_until(&t, guards[1]),
        Some(out + DURATION),
        "buffed with no new turn"
    );
}
