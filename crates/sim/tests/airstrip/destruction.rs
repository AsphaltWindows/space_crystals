//! A destroyed Airstrip (`super`; `factions/cults/structures/README.md > Building Mechanics > Evacuation on
//! Destruction`, design change 0349): everyone inside comes out alive, the orders are cancelled with a full refund;
//! and the Sparrows it trained, in the air, never needed it again.

use super::*;

/// Design change 0349 ("**Everyone inside is evacuated alive** [...] a Recruit mid-training comes out as a Recruit";
/// "**Orders are cancelled with a full refund**"), and the aircraft it let go: three Sparrows queued on four Recruits.
/// The first takes off and circles past C; the second has just taken off, still on its takeoff run, and the third is
/// mid-training when the Airstrip is destroyed (a debug removal). The third's Recruit and the free one come out at once
/// on its footprint as Recruits, the third's 75 Space Crystals back; the two Sparrows fly on, untouched: the first
/// circling, the second on along its run, straight, and then circling too, both for good.
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
            assert!(speed(&t, sparrow) >= IDLE_SPEED - 1e-3, "flying");
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
