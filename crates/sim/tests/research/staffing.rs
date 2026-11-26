//! Staffing (`core/tech_and_research.md > Research > Faction Exceptions > Cults — research is staffed by Recruits`): "A
//! Cults research building only makes research progress while Recruits are inside it"; "**BaseResearchTime**: each
//! Cults technology's research time is defined as the time with exactly 1 Recruit inside"; "**Linear scaling**:
//! effective research time scales down linearly with the number of Recruits inside — 2 Recruits halve it, 3 reduce it
//! to a third, and so on (limited only by the building's staffing capacity)"; "**Not consumed**: staffing Recruits are
//! not used up by research"; "**Pauses when empty**: if no Recruits are inside, the in-progress technology pauses at
//! its current progress (no decay) until Recruits return. Queued research waits likewise"; and the queue
//! (`core/tech_and_research.md > Research > Shared Rules` 3: "a research structure researches one technology at a
//! time. Additional technologies can be queued behind it and are processed in queue order").

use super::*;

/// The ticks Enhanced Constitution Level 1 takes with `staff` Recruits inside, from the tick it is queued, which counts
/// as a producer's, to the tick it completes, both counted. Checks that it is queued and paid at once, that the staff
/// are all still inside once it is done, and that player 1 has it.
fn time_with(staff: usize) -> Tick {
    let (mut t, order) = staffed(staff);
    let start = t.sim().tick() + 1;
    t.at(start, P1, research(&[T1], EC1));
    run_checked(&mut t, start);
    assert_eq!(queued(&t), [(start, T1, EC1)]);
    assert_eq!(bank(&t, P1), BANK - L1_PRICE, "paid as it is queued");
    assert_eq!(progress(&t, T1), (staff as u32, true), "the command's tick counts");
    while completed(&t).is_empty() {
        assert!(t.sim().tick() < start + L1_FRAMES, "{staff} Recruits: not done");
        step_checked(&mut t);
    }
    let [(done, producer, tech)] = completed(&t)[..] else {
        panic!("{:?}", completed(&t));
    };
    assert_eq!((producer, tech), (T1, EC1));
    assert_eq!(techs(&t, P1), [EC1]);
    assert_eq!(queue(&t, T1), [], "it left the queue");
    assert_eq!(inside(&t, T1), order, "not consumed");
    done - start + 1
}

/// The plan's acceptance (the M5 addendum's N3): "With one Recruit inside, Enhanced Constitution L1 takes 960 frames;
/// with three, 320". With 1 Recruit inside, the BaseResearchTime, 960 frames; 2 halve it, 3 make it a third, and 6, the
/// Temple's staffing capacity (`factions/cults/structures/temple.md > Recruit Staffing`: "Recruit staffing capacity
/// 6"), a sixth, each exact in whole frames (`sim/producers/research.rs`: a `RATIO_ONE` a frame for each Recruit).
#[test]
fn level_1_takes_960_ticks_with_one_recruit_inside_and_a_third_of_that_with_three() {
    assert_eq!(time_with(1), L1_FRAMES);
    assert_eq!(time_with(2), L1_FRAMES / 2);
    assert_eq!(time_with(3), L1_FRAMES / 3);
    assert_eq!(time_with(6), L1_FRAMES / 6);
}

/// "**Pauses when empty**": with two Recruits inside, Enhanced Constitution Level 1 makes two frames of progress a
/// tick; Eject All 100 ticks in stops it in its own tick, the Recruits on their way out staffing no more, and it stays
/// at 200 frames while they walk out one at a time and wait outside ("no decay"). Sent back in, they restart it where
/// it stopped, a frame a tick for the first one in and two for both, and it completes once the 960 frames are made up.
/// Every tick its progress grows by as many frames as Recruits staff the Temple at its end (`Sim::staff`).
#[test]
fn research_pauses_without_staff_and_resumes_where_it_stopped() {
    let (mut t, order) = staffed(2);
    let start = t.sim().tick() + 1;
    t.at(start, P1, research(&[T1], EC1));
    let check = |t: &mut TestSim, until: Tick| {
        while t.sim().tick() < until && completed(t).is_empty() {
            let (before, _) = progress(t, T1);
            step_checked(t);
            if completed(t).is_empty() {
                let (now, whole) = progress(t, T1);
                assert!(whole, "tick {}", t.sim().tick());
                assert_eq!(now - before, t.sim().staff(T1), "tick {}", t.sim().tick());
            }
        }
    };
    run_checked(&mut t, start);
    check(&mut t, start + 99);
    assert_eq!(progress(&t, T1), (200, true), "two frames a tick");
    t.at(start + 100, P1, eject_all(&[T1]));
    run_checked(&mut t, start + 100);
    assert_eq!(t.sim().staff(T1), 0, "on their way out");
    assert_eq!(progress(&t, T1), (200, true), "stopped in the Eject All's tick");
    while !inside(&t, T1).is_empty() {
        let next = t.sim().tick() + 1;
        assert!(next <= start + 200, "they come out");
        check(&mut t, next);
    }
    let out: Vec<Tick> = exits(&t).iter().map(|&(tick, _)| tick).collect();
    assert_eq!(out.len(), 2);
    assert!(out[0] < out[1], "one at a time: {out:?}");
    run_for(&mut t, 50);
    assert_eq!(progress(&t, T1), (200, true), "no decay");
    assert_eq!(queue(&t, T1), [EC1], "still queued");
    let back = t.sim().tick() + 1;
    t.at(back, P1, enter(&order, T1));
    check(&mut t, back + 300);
    assert_eq!(inside(&t, T1).len(), 2, "both back in");
    check(&mut t, back + 1000);
    let [(done, ..)] = completed(&t)[..] else {
        panic!("{:?}", completed(&t));
    };
    assert!(
        done > back + 380,
        "760 frames left, at two a tick at most: done in {}",
        done - back
    );
    assert_eq!(techs(&t, P1), [EC1]);
}

/// "Queued research waits likewise" and Shared Rules 3, "processed in queue order": Improved Metabolism Level 1, queued
/// behind Enhanced Constitution Level 1, makes no progress while it waits, and starts in the tick after the first
/// completes, taking its own 960 frames with the one Recruit inside. Both were paid as they were queued.
#[test]
fn a_research_queued_behind_another_waits_for_it() {
    let (mut t, _) = staffed(1);
    let start = t.sim().tick() + 1;
    t.at(start, P1, research(&[T1], EC1));
    t.at(start, P1, research(&[T1], IM1));
    run_checked(&mut t, start);
    assert_eq!(queue(&t, T1), [EC1, IM1]);
    assert_eq!(bank(&t, P1), BANK - 2 * L1_PRICE);
    let status = t.sim().production_status(T1).expect("researching");
    assert_eq!(status.entry.tech(), Some(EC1), "the first queued first");
    assert_eq!(status.total, L1_FRAMES * RATIO_ONE);
    run_checked(&mut t, start + L1_FRAMES - 1);
    assert_eq!(completed(&t), [(start + L1_FRAMES - 1, T1, EC1)]);
    assert_eq!(queue(&t, T1), [IM1]);
    assert_eq!(progress(&t, T1), (0, true), "it starts in the next tick");
    run_checked(&mut t, start + 2 * L1_FRAMES - 1);
    assert_eq!(
        completed(&t),
        [(start + L1_FRAMES - 1, T1, EC1), (start + 2 * L1_FRAMES - 1, T1, IM1)]
    );
    assert_eq!(techs(&t, P1), [EC1, IM1]);
    assert_eq!(t.sim().production_status(T1), None, "nothing left");
}

/// "A Cults research building only makes research progress while Recruits are inside it": with three Recruits inside,
/// a Release of one (`HoldCommand::ReleaseOne`) takes it off the staff in the command's tick, before it is out, so the
/// research makes two frames in that tick; a Recruit walking in staffs from the tick it is inside. A Temple without a
/// queue does nothing with its staff.
#[test]
fn a_recruit_staffs_from_the_tick_it_is_inside_until_it_is_on_its_way_out() {
    let (mut t, order) = staffed(3);
    assert_eq!(t.sim().staff(T1), 3);
    assert_eq!(t.sim().production_status(T1), None, "nothing to research");
    let start = t.sim().tick() + 1;
    t.at(start, P1, research(&[T1], EC1));
    run_checked(&mut t, start + 9);
    assert_eq!(progress(&t, T1), (30, true));
    t.at(start + 10, P1, release_one(&[T1], RecruitKind::Plain));
    run_checked(&mut t, start + 10);
    assert_eq!(t.sim().staff(T1), 2);
    assert_eq!(progress(&t, T1), (32, true), "two frames in the Release's tick");
    assert_eq!(exits(&t), [(start + 10, order[0])], "the first in, out at once");
    let back = t.sim().tick() + 1;
    t.at(back, P1, enter(&order[..1], T1));
    while t.sim().staff(T1) < 3 {
        assert!(t.sim().tick() < back + 300, "it goes back in");
        let (before, _) = progress(&t, T1);
        step_checked(&mut t);
        let (now, _) = progress(&t, T1);
        assert_eq!(now - before, t.sim().staff(T1), "tick {}", t.sim().tick());
    }
    let (before, _) = progress(&t, T1);
    step_checked(&mut t);
    assert_eq!(progress(&t, T1).0 - before, 3, "three again");
}
