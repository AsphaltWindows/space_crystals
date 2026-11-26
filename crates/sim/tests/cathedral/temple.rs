//! The Cathedral's TechPrerequisite (`factions/cults/structures/cathedral.md > TechPrerequisite - player owns at least
//! one Temple`; `factions/cults/units/recruit.md > Constructs`: "Cathedral: 250 Space Crystals, 640 frames (40
//! seconds) BaseConstructionTime (placeholder). Requires: Temple."), checked as the Build is given
//! (`core/tech_and_research.md > Tech Prerequisites`), and design change 0335: "**losing a prerequisite never cancels or
//! disables anything already started or finished**, for structures, units and research".

use super::*;

/// Where the Recruits of these tests build their Cathedral: tiles (16, 4) to (18, 5).
const SITE: TilePos = TilePos::new(16, 4);

/// The plan's acceptance: "without a Temple the Construct entry is greyed and the gate refuses the build" (the entry's
/// greying is the client's, `crates/game/tests/commands/`). A Recruit's Build of a Cathedral is refused while its
/// player owns no Temple (`Reject::Requires`), another player's not counting; once theirs stands it is taken and paid,
/// 250 Space Crystals, and the Recruit builds the Cathedral, which then trains a Disciple.
#[test]
fn recruits_build_a_cathedral_only_while_their_player_owns_a_temple() {
    let mut t = cults(&flat(30, 22))
        .spawn(TEMPLE, Some(P2), tile(24, 16)) // 1
        .spawn(ObjectKind::Storage, Some(P1), tile(2, 16)) // 2
        .start();
    t.without_auto_targeting();
    t.debug_at(1, crystals(P1, BANK));
    let recruit = id(next_id(&t));
    t.spawn_unit_at(1, RECRUIT, P1, Vec2::new(1000.0, 600.0));
    run_checked(&mut t, 1);
    let order = build(&[recruit], CATHEDRAL, SITE);
    assert_eq!(
        validate(&t, order.clone()),
        Err(Reject::Requires(TEMPLE)),
        "player 2's Temple"
    );
    let temple = id(next_id(&t));
    t.debug_at(
        2,
        DebugCommand::Spawn {
            kind: TEMPLE,
            owner: Some(P1),
            at: tile(2, 2),
            orientation: BuildingOrientation::default(),
        },
    );
    run_checked(&mut t, 2);
    assert!(t.sim().object(temple).is_some_and(|o| o.kind == TEMPLE));
    assert_eq!(validate(&t, order.clone()), Ok(()));
    t.at(3, P1, order);
    run_checked(&mut t, 3);
    assert_eq!(bank(&t, P1), BANK - 250, "paid as the order is given");
    let built = |t: &TestSim| {
        (t.events().iter()).find_map(|(tick, e)| match *e {
            SimEvent::SiteBuilt { site } => Some((*tick, site)),
            _ => None,
        })
    };
    while built(&t).is_none() {
        assert!(t.sim().tick() < 1200, "the Cathedral is built\n{}", t.ascii(None));
        step_checked(&mut t);
    }
    let (_, cathedral) = built(&t).unwrap();
    let o = t.sim().object(cathedral).expect("built");
    assert!(o.kind == CATHEDRAL && o.is_operational() && o.production.is_some());
    let now = t.sim().tick() + 1;
    t.at(now, P1, enter(&[recruit], cathedral));
    run_checked(&mut t, now);
    fill(&mut t, cathedral, 1);
    let queue = t.sim().tick() + 1;
    t.at(queue, P1, train(&[cathedral], RecruitKind::Plain));
    run_checked(&mut t, queue + FRAMES - 1);
    assert_eq!(produced(&t), [(queue + FRAMES - 1, cathedral, recruit)]);
    assert_eq!(bank(&t, P1), BANK - 250 - PRICE);
}

/// The plan's acceptance: "Losing the Temple never cancels a Disciple already queued (0335)". Two Disciples queued,
/// the Temple goes 10 ticks into the first: both come out in their time, and the Cathedral, finished, is not disabled
/// by it either ("never cancels or disables anything already started or finished"): a third is taken, paid and
/// trained. Only a new Cathedral is refused.
#[test]
fn losing_the_temple_cancels_no_disciple() {
    let (mut t, order) = filled(3);
    let start = t.sim().tick() + 1;
    t.at(start, P1, train(&[C1], RecruitKind::Plain));
    t.at(start, P1, train(&[C1], RecruitKind::Plain));
    remove_at(&mut t, start + 10, T1);
    run_checked(&mut t, start + 10);
    assert!(!t.sim().requirement_met(P1, TEMPLE), "the Temple is gone");
    assert_eq!(reserved(&t, C1), [[order[0]], [order[1]]], "nothing cancelled");
    assert!(!(t.events().iter()).any(|(_, e)| matches!(e, SimEvent::ProductionCancelled { .. })));
    assert_eq!(validate(&t, train(&[C1], RecruitKind::Plain)), Ok(()), "not disabled");
    t.at(start + 11, P1, train(&[C1], RecruitKind::Plain));
    run_for(&mut t, 3 * FRAMES);
    let out: Vec<(Tick, ObjectId)> = produced(&t).iter().map(|&(tick, _, unit)| (tick, unit)).collect();
    assert_eq!(
        out,
        [
            (start + FRAMES - 1, order[0]),
            (start + 2 * FRAMES - 1, order[1]),
            (start + 3 * FRAMES - 1, order[2])
        ]
    );
    assert_eq!(bank(&t, P1), BANK - 3 * PRICE);
    let builder = id(next_id(&t));
    let later = t.sim().tick() + 1;
    t.spawn_unit_at(later, RECRUIT, P1, Vec2::new(1150.0, 300.0));
    run_checked(&mut t, later);
    assert_eq!(
        validate(&t, build(&[builder], CATHEDRAL, SITE)),
        Err(Reject::Requires(TEMPLE)),
        "a new Cathedral wants a Temple"
    );
}
