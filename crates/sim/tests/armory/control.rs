//! A trained unit and its Recruitment Hall's Unit Control (`factions/cults/structures/recruitment_hall.md > Unit
//! Control Tracking`: "Each Recruit tracks which Recruitment Hall produced it"; slice M5-C's lineages): the Militant is
//! the Recruit it was trained of, its lineage with it, so the Unit Control it uses is the Recruit's, its Hall's while
//! the Hall stands and an orphaned point once it is gone. An Armory's queue takes no Unit Control of its own: its units
//! are counted already (`ArmyResource::taken_by`).

use sc_sim::HallState;

use super::*;

/// The Hall of [`hall_and_armory`].
const HALL: ObjectId = ObjectId(1);
/// Its Armory.
const ARMORY_2: ObjectId = ObjectId(2);

/// Player 1's Recruitment Hall (1) at tiles (4, 4) to (7, 7), its exit on the west side, and an Armory (2) at tiles
/// (1, 9) to (3, 10), its entrance, B, facing east, on a 30 × 22 map, with a Storage (3), which keeps player 1 in the
/// game when the others go, and [`BANK`] Space Crystals. The Hall's first Recruit is ordered into the Armory as it
/// comes out; this returns it once it is inside, still the Hall's only Recruit (the next one comes 192 ticks after the
/// first, and is in production meanwhile).
fn hall_and_armory() -> (TestSim, ObjectId) {
    let mut t = cults(&flat(30, 22))
        .spawn(ObjectKind::RecruitmentHall, Some(P1), tile(4, 4))
        .spawn(ARMORY, Some(P1), tile(1, 9))
        .spawn(ObjectKind::Storage, Some(P1), tile(24, 16))
        .start();
    t.without_auto_targeting();
    t.debug_at(1, crystals(P1, BANK));
    while produced(&t).is_empty() {
        assert!(t.sim().tick() < 250, "the Hall's first Recruit");
        step_checked(&mut t);
    }
    let [(_, HALL, recruit)] = produced(&t)[..] else {
        panic!("one Recruit: {:?}", produced(&t));
    };
    let order = t.sim().tick() + 1;
    t.at(order, P1, enter(&[recruit], ARMORY_2));
    run_checked(&mut t, order);
    fill(&mut t, ARMORY_2, 1);
    assert_eq!(produced(&t).len(), 1, "still the Hall's only one");
    (t, recruit)
}

/// `player`'s Cults Unit Control as the HUD shows it: `(used, available)`.
fn unit_control(t: &TestSim, player: PlayerId) -> (u32, u32) {
    let cap = t.sim().army_cap(player).expect("in the game");
    (cap.used, cap.available)
}

fn hall(t: &TestSim) -> HallState {
    t.sim().hall_state(HALL).expect("the Hall")
}

/// The plan's acceptance (slice M5-F2): "Unit Control is unchanged through training." The Hall's Recruit inside the
/// Armory is trained into a Militant: the Hall's usage and the player's Unit Control stay as they were every tick of the
/// training and after it (the Recruit and the Hall's next one in production), and the Militant names the Hall.
#[test]
fn unit_control_is_unchanged_through_training() {
    let (mut t, recruit) = hall_and_armory();
    let before = (hall(&t).usage, unit_control(&t, P1));
    assert_eq!(before, (2, (2, 20)), "it and the next in production");
    let start = t.sim().tick() + 1;
    t.at(start, P1, train(&[ARMORY_2], RecruitKind::Plain));
    while !produced(&t).iter().any(|&(_, producer, _)| producer == ARMORY_2) {
        assert!(t.sim().tick() < start + 40, "training");
        step_checked(&mut t);
        assert_eq!(
            (hall(&t).usage, unit_control(&t, P1)),
            before,
            "tick {}",
            t.sim().tick()
        );
    }
    let o = t.sim().object(recruit).expect("out");
    assert_eq!((o.kind, &o.lineage[..]), (MILITANT, &[HALL][..]));
    run_for(&mut t, 16);
    assert_eq!((hall(&t).usage, unit_control(&t, P1)), before);
}

/// The plan's acceptance: "A Militant from a destroyed Hall's Recruit is an orphan." The Hall goes while its Recruit
/// is inside (an orphaned point, `tests/holding/control.rs`); trained, the Militant names the Hall that is gone, and is
/// the same orphaned point.
#[test]
fn a_militant_trained_of_a_destroyed_halls_recruit_is_an_orphan() {
    let (mut t, recruit) = hall_and_armory();
    let gone = t.sim().tick() + 1;
    remove_at(&mut t, gone, HALL);
    run_checked(&mut t, gone);
    assert_eq!(unit_control(&t, P1), (1, 0));
    let start = gone + 1;
    t.at(start, P1, train(&[ARMORY_2], RecruitKind::Plain));
    run_checked(&mut t, start + 31);
    assert_eq!(produced(&t).last(), Some(&(start + 31, ARMORY_2, recruit)));
    let o = t.sim().object(recruit).expect("out");
    assert_eq!(
        (o.kind, &o.lineage[..]),
        (MILITANT, &[HALL][..]),
        "it names the Hall that is gone"
    );
    assert_eq!(unit_control(&t, P1), (1, 0));
}
