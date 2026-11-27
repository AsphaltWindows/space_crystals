//! A destroyed Chop Shop (`factions/cults/economy.md > Cults Building Mechanics > Evacuation on Destruction`, design
//! change 0349): "**Everyone inside is evacuated alive**", "**Orders are cancelled with a full refund**", and "**A unit
//! that had finished training** and was waiting for the doorway comes out **as the trained unit**, with no refund".

use super::*;

/// A Chop Shop whose exit is walled off: the three tiles in front of side C are cliffs, so a Recruit released comes
/// out fully inside its doorway and stands there for good (`core/entities.md > Structure Type > Exit Doorway`: "If the
/// way is blocked, it stops in the doorway"), the doorway occupied. Six Recruits inside: a Release, then two Seekers.
/// The first Seeker, trained, waits for the doorway, the second is in training, when the Chop Shop is destroyed (a debug
/// removal): the first comes out a Seeker, of its two Recruits, inside the former footprint with its whole box, the
/// second's Recruits and the free one as Recruits, all in the same tick; the second's 75 Space Crystals come back, the
/// first's don't.
#[test]
fn a_destroyed_chop_shop_evacuates_its_recruits_and_a_finished_seeker_as_a_seeker() {
    let map: Vec<String> = (0..22)
        .map(|y| {
            (0..30)
                .map(|x| if x == 11 && (10..13).contains(&y) { 'c' } else { '.' })
                .collect()
        })
        .collect();
    let (mut t, order) = filled_on(on(&map.join("\n")), 6);
    let start = t.sim().tick() + 1;
    t.at(start, P1, release(&[CS], RecruitKind::Plain));
    t.at(start, P1, train(&[CS], RecruitKind::Plain));
    t.at(start, P1, train(&[CS], RecruitKind::Plain));
    run_checked(&mut t, start);
    assert_eq!(
        reserved(&t, CS),
        [vec![order[0]], vec![order[1], order[2]], vec![order[3], order[4]]]
    );
    while exits(&t).is_empty() {
        assert!(t.sim().tick() < start + 40, "the Release");
        step_checked(&mut t);
    }
    assert_eq!(exits(&t)[0].1, order[0]);
    run_for(&mut t, FRAMES + 40);
    assert_eq!(produced(&t), [], "the first Seeker waits");
    assert_eq!(t.mover(order[0].0).exit, Some(CS), "the Recruit stands in the doorway");
    let status = t.sim().production_status(CS).expect("a producer");
    assert!(status.waiting, "finished, waiting for the doorway: {status:?}");
    assert_eq!(bank(&t, P1), BANK - 2 * PRICE);
    let gone = t.sim().tick() + 1;
    remove_at(&mut t, gone, CS);
    run_checked(&mut t, gone);
    let evacuated: Vec<ObjectId> = (t.events().iter())
        .filter_map(|(at, e)| match *e {
            SimEvent::Evacuated { unit, building } if *at == gone && building == CS => Some(unit),
            _ => None,
        })
        .collect();
    assert_eq!(
        evacuated,
        [order[1], order[3], order[4], order[5]],
        "longest inside first"
    );
    let seeker = t.sim().object(order[1]).expect("evacuated");
    assert_eq!((seeker.kind, seeker.hp), (SEEKER, Some(SEEKER_HP)), "the trained unit");
    assert!(
        t.sim().object(order[2]).is_none() && t.sim().held_unit(order[2]).is_none(),
        "gone into it"
    );
    let (lo, hi) = shape(&t, order[1]).bounds();
    let footprint = (Vec2::new(768.0, 640.0), Vec2::new(960.0, 832.0));
    assert!(
        lo.cmpge(footprint.0 - 1e-3).all() && hi.cmple(footprint.1 + 1e-3).all(),
        "inside the footprint: {lo} to {hi}"
    );
    for unit in [order[3], order[4], order[5]] {
        let o = t.sim().object(unit).expect("evacuated");
        assert_eq!(o.kind, RECRUIT, "{unit:?}");
        assert!(
            o.pos.cmpge(footprint.0).all() && o.pos.cmple(footprint.1).all(),
            "{}",
            o.pos
        );
    }
    assert_eq!(
        bank(&t, P1),
        BANK - PRICE,
        "the second's Space Crystals back, the first's not"
    );
    assert!(t.sim().held().next().is_none(), "no one left inside");
    run_for(&mut t, 60);
}
