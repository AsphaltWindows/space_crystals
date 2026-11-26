//! A Recruit inside an Armory and its Recruitment Hall's Unit Control (`factions/cults/economy.md > RecruitmentHall >
//! Unit Control Tracking`: "Each Recruit tracks which Recruitment Hall produced it"; slice M5-C's lineages): inside, it
//! is held off the map, and the units held count as those on it (`economy::army::cults`), so it still uses its Hall's
//! Unit Control, and after its evacuation too (`factions/cults/economy.md > Cults Building Mechanics > Evacuation on
//! Destruction`: "Units come out unchanged"). Its Hall gone while it is inside, it is an orphaned point
//! (`factions/cults/economy.md > RecruitmentHall > On Destruction`: "Surviving units produced by this hall remain on
//! the field and continue to count toward the global usage sum").

use super::*;

/// The Hall of [`hall_and_armory`].
const HALL: ObjectId = ObjectId(1);
/// Its Armory.
const ARMORY_2: ObjectId = ObjectId(2);

/// Player 1's Recruitment Hall (1) at tiles (4, 4) to (7, 7), its exit, B, the west side, its door's middle at (256,
/// 384), and an Armory (2) at tiles (1, 9) to (3, 10), its entrance, B, facing east at (256, 640), on a 30 × 22 map;
/// and a Storage (3), which keeps player 1 in the game when the others go. The Hall's first Recruit comes out of side
/// B (design change 0342: "It becomes AAAB with the exit on B") and is ordered into the Armory in the next tick; this
/// returns it once it is inside, within 80 ticks, still the Hall's only Recruit (the next one comes 192 ticks after
/// the first, and is in production meanwhile).
fn hall_and_armory() -> (TestSim, ObjectId) {
    let mut t = cults(&flat(30, 22))
        .spawn(ObjectKind::RecruitmentHall, Some(P1), tile(4, 4))
        .spawn(ARMORY, Some(P1), tile(1, 9))
        .spawn(ObjectKind::Storage, Some(P1), tile(24, 16))
        .start();
    t.without_auto_targeting();
    let made = |t: &TestSim| {
        (t.events().iter())
            .filter_map(|(_, e)| match *e {
                SimEvent::UnitProduced { producer, unit } if producer == HALL => Some(unit),
                _ => None,
            })
            .collect::<Vec<ObjectId>>()
    };
    while made(&t).is_empty() {
        assert!(t.sim().tick() < 250, "the Hall's first Recruit");
        step_checked(&mut t);
    }
    let [recruit] = made(&t)[..] else {
        panic!("one Recruit: {:?}", made(&t));
    };
    let order = t.sim().tick() + 1;
    t.at(order, P1, enter(&[recruit], ARMORY_2));
    while inside(&t, ARMORY_2).is_empty() {
        assert!(t.sim().tick() < order + 80, "in by now\n{}", t.ascii(None));
        step_checked(&mut t);
    }
    assert_eq!(inside(&t, ARMORY_2), [recruit]);
    assert_eq!(made(&t), [recruit], "still the Hall's only one");
    (t, recruit)
}

/// `player`'s Cults Unit Control as the HUD shows it: `(used, available)`.
fn unit_control(t: &TestSim, player: PlayerId) -> (u32, u32) {
    let cap = t.sim().army_cap(player).expect("in the game");
    (cap.used, cap.available)
}

/// The Hall's LocalUnitControlUsage.
fn usage(t: &TestSim) -> u32 {
    t.sim().hall_state(HALL).expect("the Hall").usage
}

/// A Recruit inside an Armory still uses its Hall's Unit Control, and still does once it is evacuated, its lineage
/// unchanged; the Hall's next Recruit, in production, counts as well (design change 0342: "The Recruit in production
/// counts toward LocalUnitControlUsage from the start of its production").
#[test]
fn a_recruit_inside_an_armory_still_counts_in_its_halls_usage() {
    let (mut t, recruit) = hall_and_armory();
    assert!(t.sim().object(recruit).is_none(), "off the map");
    assert_eq!(usage(&t), 2, "it and the next in production");
    assert_eq!(unit_control(&t, P1), (2, 20));
    let gone = t.sim().tick() + 1;
    remove_at(&mut t, gone, ARMORY_2);
    run_checked(&mut t, gone);
    let o = t.sim().object(recruit).expect("evacuated");
    assert_eq!(o.lineage, [HALL]);
    assert_eq!(usage(&t), 2);
    assert_eq!(unit_control(&t, P1), (2, 20));
}

/// Its Hall destroyed while it is inside, the Recruit is an orphaned point: the Hall's capacity leaves the sum, and
/// its Recruit in production with it; the Recruit's usage stays in it, and stays there once the Armory goes too and it
/// is evacuated.
#[test]
fn a_recruit_inside_when_its_hall_goes_counts_as_an_orphan() {
    let (mut t, recruit) = hall_and_armory();
    let hall_gone = t.sim().tick() + 1;
    remove_at(&mut t, hall_gone, HALL);
    run_checked(&mut t, hall_gone);
    assert_eq!(inside(&t, ARMORY_2), [recruit]);
    assert_eq!(unit_control(&t, P1), (1, 0));
    let armory_gone = hall_gone + 1;
    remove_at(&mut t, armory_gone, ARMORY_2);
    run_checked(&mut t, armory_gone);
    let o = t.sim().object(recruit).expect("evacuated");
    assert_eq!(o.lineage, [HALL], "it names the Hall that is gone");
    assert_eq!(unit_control(&t, P1), (1, 0));
}
