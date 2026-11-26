//! The levels and the duplicates (`core/tech_and_research.md > Research > Shared Rules`: "4. **No duplicates**: a
//! technology cannot be queued or in progress at more than one structure at the same time. Once completed, it no
//! longer appears in any research menu"; "5. **Levels**: a multi-level research line is a chain of separate
//! technologies, each requiring the previous level"; `core/tech_and_research.md > Tech Prerequisites > Research
//! Prerequisites`: "A technology's prerequisites (previous level, required structures) follow the same rules:
//! **checked only when it is queued**; losing a prerequisite structure afterwards **never cancels** research already
//! queued or in progress. A next level can only be queued once the previous level has **completed**";
//! `factions/cults/structures/temple.md > Recruit Staffing`: "a Level 2 can't be queued until its Level 1 has
//! **completed** (its key shows "In progress" meanwhile). Level 2's Cathedral prerequisite is checked only when it is
//! queued; losing the Cathedral later doesn't cancel it"), checked in plan decision T74's order (`Sim::plan_research`).

use sc_sim::ResearchLine;

use super::*;

/// The plan's acceptance (the M5 addendum's N3): "**Level 2** needs L1 complete and a complete Cathedral
/// (`Reject::Requires`). Losing the Cathedral after queuing cancels nothing". With six Recruits inside, Level 1 takes
/// 160 ticks: until it completes, Level 2 is refused for it (`Reject::RequiresTech`), and Level 1 itself, queued, is
/// under way (`Reject::ResearchUnderWay`, the key's "In progress"). Once it is done it is `Reject::Researched`, and
/// Level 2 wants a Cathedral (`Reject::Requires`); with one, spawned complete, it is taken and paid, 150 Space
/// Crystals; the Cathedral destroyed 10 ticks later cancels nothing, and Level 2 completes in its 1440 frames over six,
/// 240 ticks. The line has no next level then (`Sim::next_tech`), and the other line is still at its Level 1.
#[test]
fn level_2_waits_for_level_1_to_complete_then_wants_a_cathedral_at_queue_only() {
    let (mut t, _) = staffed(6);
    let constitution = ResearchLine::EnhancedConstitution;
    assert_eq!(t.sim().next_tech(P1, constitution), Some(EC1));
    assert_eq!(validate(&t, research(&[T1], EC2)), Err(Reject::RequiresTech(EC1)));
    let start = t.sim().tick() + 1;
    t.at(start, P1, research(&[T1], EC1));
    run_checked(&mut t, start);
    while completed(&t).is_empty() {
        assert_eq!(validate(&t, research(&[T1], EC1)), Err(Reject::ResearchUnderWay(EC1)));
        assert_eq!(validate(&t, research(&[T1], EC2)), Err(Reject::RequiresTech(EC1)));
        assert_eq!(t.sim().research_site(P1, EC1), Some(T1));
        assert_eq!(t.sim().next_tech(P1, constitution), Some(EC1), "the key's In progress");
        step_checked(&mut t);
    }
    assert_eq!(completed(&t), [(start + L1_FRAMES / 6 - 1, T1, EC1)]);
    assert_eq!(validate(&t, research(&[T1], EC1)), Err(Reject::Researched(EC1)));
    assert_eq!(t.sim().research_site(P1, EC1), None);
    assert_eq!(t.sim().next_tech(P1, constitution), Some(EC2));
    assert_eq!(validate(&t, research(&[T1], EC2)), Err(Reject::Requires(CATHEDRAL)));
    let now = t.sim().tick() + 1;
    let cathedral = spawn_structure(&mut t, now, CATHEDRAL, tile(20, 16));
    run_checked(&mut t, now);
    assert!(
        t.sim()
            .object(cathedral)
            .is_some_and(|o| o.kind == CATHEDRAL && o.is_operational())
    );
    assert_eq!(validate(&t, research(&[T1], EC2)), Ok(()));
    let queue_at = now + 1;
    t.at(queue_at, P1, research(&[T1], EC2));
    remove_at(&mut t, queue_at + 10, cathedral);
    run_checked(&mut t, queue_at + 10);
    assert!(t.sim().object(cathedral).is_none());
    assert_eq!(bank(&t, P1), BANK - L1_PRICE - L2_PRICE);
    assert_eq!(queue(&t, T1), [EC2], "nothing cancelled");
    assert_eq!(cancelled(&t), []);
    run_checked(&mut t, queue_at + L2_FRAMES / 6 - 1);
    assert_eq!(completed(&t)[1..], [(queue_at + L2_FRAMES / 6 - 1, T1, EC2)]);
    assert_eq!(techs(&t, P1), [EC1, EC2]);
    assert_eq!(validate(&t, research(&[T1], EC2)), Err(Reject::Researched(EC2)));
    assert_eq!(t.sim().next_tech(P1, constitution), None, "the line is done");
    let metabolism = ResearchLine::ImprovedMetabolism;
    assert_eq!(t.sim().next_tech(P1, metabolism), Some(IM1));
    assert_eq!(validate(&t, research(&[T1], IM2)), Err(Reject::RequiresTech(IM1)));
}

/// Shared Rules 4: with two Temples of player 1's, Enhanced Constitution Level 1 queued at one is under way for the
/// other (`Reject::ResearchUnderWay`), and so is it for a group of both. Player 2's own research is theirs: their
/// Temple may queue the same technology.
#[test]
fn a_technology_is_queued_at_one_temple_only() {
    let t2 = id(3);
    let theirs = id(4);
    let scenario = base()
        .spawn(TEMPLE, Some(P1), tile(12, 2)) // 3
        .spawn(TEMPLE, Some(P2), tile(24, 16)); // 4
    let (mut t, _) = staffed_with(scenario, &[RecruitKind::Plain]);
    let now = t.sim().tick() + 1;
    t.at(now, P1, research(&[T1], EC1));
    t.debug_at(now, crystals(P2, BANK));
    run_checked(&mut t, now);
    assert_eq!(validate(&t, research(&[t2], EC1)), Err(Reject::ResearchUnderWay(EC1)));
    assert_eq!(
        validate(&t, research(&[T1, t2], EC1)),
        Err(Reject::ResearchUnderWay(EC1))
    );
    assert_eq!(validate(&t, research(&[t2], IM1)), Ok(()), "another technology");
    let p2 = |command| PlayerCommand { player: P2, command };
    assert_eq!(
        t.sim().validate(&p2(research(&[theirs], EC1))),
        Ok(()),
        "player 2's own"
    );
    t.at(now + 1, P2, research(&[theirs], EC1));
    run_checked(&mut t, now + 1);
    assert_eq!(queue(&t, theirs), [EC1]);
    assert_eq!(t.sim().research_site(P2, EC1), Some(theirs));
    assert_eq!(t.sim().research_site(P1, EC1), Some(T1));
}

/// A group's research goes to one Temple (`ProductionCommand::Research`, design change 0083's order, `Sim::serve`'s):
/// the one with the least work queued, ties by id. Of two idle Temples the first takes Enhanced Constitution Level 1;
/// then Improved Metabolism Level 1 goes to the second, which has none queued; each is paid once.
#[test]
fn a_groups_research_goes_to_the_temple_with_the_least_work() {
    let t2 = id(3);
    let scenario = base().spawn(TEMPLE, Some(P1), tile(12, 2)); // 3
    let (mut t, _) = staffed_with(scenario, &[RecruitKind::Plain]);
    let now = t.sim().tick() + 1;
    t.at(now, P1, research(&[t2, T1], EC1));
    t.at(now, P1, research(&[t2, T1], IM1));
    run_checked(&mut t, now);
    assert_eq!(queued(&t), [(now, T1, EC1), (now, t2, IM1)]);
    assert_eq!((queue(&t, T1), queue(&t, t2)), (vec![EC1], vec![IM1]));
    assert_eq!(bank(&t, P1), BANK - 2 * L1_PRICE, "each paid once");
    run_for(&mut t, 100);
    assert_eq!(progress(&t, t2), (0, true), "no staff, no progress");
}
