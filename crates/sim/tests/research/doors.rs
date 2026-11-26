//! The Temple's doors and its staff (`factions/cults/structures/temple.md > Recruit Staffing`: "SymmetryType ABAC,
//! **EntranceSide B, ExitSide C** (like the Armory; user rule 2026-11-26, Q0116: research buildings have an entrance
//! and an exit, and staff leave one at a time)"; "Recruit staffing capacity 6"; "Panel: [...] **A/S/D/F** release one
//! Recruit of a kind (README > Choosing Recruits, Release and Eject All), **V: Eject All** (staff leave) [...] Recruits
//! enter by right-click"; `factions/cults/structures/README.md > Choosing Recruits, Release and Eject All`: at a tech
//! building, "pressing one sends **one** Recruit of that kind out of the exit at once (no queue)";
//! `factions/cults/structures/temple.md > Recruit Staffing`: "Only Recruits can staff research").

use super::*;

/// The plan's acceptance: "Recruits staffing a Temple walk in at B, and Eject All lets them out of C one by one".
/// Three Recruits from the east go in at the entrance, in the middle of side B, their boxes against the side within
/// the entry path's 8 su of it; Eject All marks them all at once (no staff from its tick), and they come out of the
/// doorway of side C one at a time, in the order they went in, each once the one before has walked clear of it. With
/// no rally point, they stop outside.
#[test]
fn the_staff_go_in_at_b_and_eject_all_lets_them_out_of_c_one_by_one() {
    let (mut t, order) = staffed(3);
    for &unit in &order {
        let held = t.sim().held_unit(unit).expect("inside");
        let (at, half) = (held.object.pos, held.object.def().half_extents().unwrap());
        assert!(
            at.x - half.x >= ENTRANCE.x
                && at.x - half.x - ENTRANCE.x <= 8.0
                && (at.y - ENTRANCE.y).abs() - half.y <= 8.0,
            "{unit:?} went in at {at}, not at the entrance"
        );
    }
    assert_eq!(t.sim().staff(T1), 3);
    let start = t.sim().tick() + 1;
    t.at(start, P1, eject_all(&[T1]));
    run_checked(&mut t, start);
    assert_eq!(t.sim().staff(T1), 0, "all on their way out");
    while exits(&t).len() < 3 {
        assert!(t.sim().tick() < start + 200, "they come out");
        let before = exits(&t).len();
        step_checked(&mut t);
        if exits(&t).len() > before {
            let (_, unit) = *exits(&t).last().unwrap();
            let pos = t.sim().object(unit).expect("out").pos;
            assert!(in_the_exit(pos), "{unit:?} came out at {pos}, not at side C");
        }
    }
    let out = exits(&t);
    let units: Vec<ObjectId> = out.iter().map(|&(_, unit)| unit).collect();
    assert_eq!(units, order, "in the order they went in");
    assert!(
        out.windows(2).all(|pair| pair[0].0 < pair[1].0),
        "one at a time: {out:?}"
    );
    assert!(inside(&t, T1).is_empty());
    run_for(&mut t, 60);
    for unit in order {
        let o = t.sim().object(unit).expect("on the map");
        assert!(o.pos.x < EXIT.x, "{unit:?} stopped west of side C, at {}", o.pos);
        assert!(
            o.mover.as_ref().is_some_and(|m| m.goal == sc_sim::MoveGoal::None),
            "{unit:?} stands"
        );
    }
}

/// A/S/D/F (`HoldCommand::ReleaseOne`): of a Plain, an Enhanced Constitution, another Plain and an Improved Metabolism
/// Recruit inside, A sends the first Plain one in out of side C in its own tick, and A again the second; a third A,
/// with no Plain one left, is refused (`Reject::NoRecruit`), as F is with none with Both. S sends the Enhanced
/// Constitution one. The others stay, staffing.
#[test]
fn a_release_sends_out_one_recruit_of_its_kind_the_first_in_first() {
    use RecruitKind::{Both, Constitution, Metabolism, Plain};
    let (mut t, order) = staffed_with(base(), &[Plain, Constitution, Plain, Metabolism]);
    let kind_of = |t: &TestSim, unit: ObjectId| t.sim().held_unit(unit).unwrap().object.augmentation.kind();
    let plain: Vec<ObjectId> = order
        .iter()
        .copied()
        .filter(|&unit| kind_of(&t, unit) == Plain)
        .collect();
    assert_eq!(plain.len(), 2);
    assert_eq!(validate(&t, release_one(&[T1], Both)), Err(Reject::NoRecruit(Both)));
    let start = t.sim().tick() + 1;
    t.at(start, P1, release_one(&[T1], Plain));
    run_checked(&mut t, start);
    assert_eq!(exits(&t), [(start, plain[0])], "the first in, out at once");
    assert!(in_the_exit(t.sim().object(plain[0]).unwrap().pos));
    assert_eq!(t.sim().staff(T1), 3);
    run_for(&mut t, 30);
    let again = t.sim().tick() + 1;
    t.at(again, P1, release_one(&[T1], Plain));
    run_checked(&mut t, again);
    assert_eq!(exits(&t)[1..], [(again, plain[1])]);
    assert_eq!(validate(&t, release_one(&[T1], Plain)), Err(Reject::NoRecruit(Plain)));
    run_for(&mut t, 30);
    let third = t.sim().tick() + 1;
    t.at(third, P1, release_one(&[T1], Constitution));
    run_checked(&mut t, third);
    let [.., (_, unit)] = exits(&t)[..] else { unreachable!() };
    assert_eq!(t.sim().object(unit).map(|o| o.augmentation.kind()), Some(Constitution));
    let left: Vec<RecruitKind> = inside(&t, T1).iter().map(|&unit| kind_of(&t, unit)).collect();
    assert_eq!(left, [Metabolism]);
    assert_eq!(t.sim().staff(T1), 1);
}

/// A Release of one is a tech building's (`sc_sim::holding::staffed`): an Armory, which holds a pool, refuses it
/// (`Reject::NoHolders`; its own Release is a queue entry, `ProductionCommand::Release`), as the Storage does, which
/// holds no one. A group of the Temple and the Armory releases one from the Temple alone.
#[test]
fn a_release_of_one_is_a_tech_buildings() {
    let armory = id(3);
    let scenario = base().spawn(ARMORY, Some(P1), tile(20, 14)); // 3
    let (mut t, order) = staffed_with(scenario, &[RecruitKind::Plain; 2]);
    let plain = RecruitKind::Plain;
    assert_eq!(validate(&t, release_one(&[armory], plain)), Err(Reject::NoHolders));
    assert_eq!(validate(&t, release_one(&[id(2)], plain)), Err(Reject::NoHolders));
    let now = t.sim().tick() + 1;
    t.at(now, P1, release_one(&[armory, T1], plain));
    run_checked(&mut t, now);
    assert_eq!(exits(&t), [(now, order[0])]);
}

/// The Temple has no rally point (its panel has no Set Rally Point; `Reject::NoProducers` for a SetRally to it
/// alone), and takes Recruits only: a Militant's Enter is refused (`Reject::NotAccepted`), a Recruit's taken. It holds
/// six ("Recruit staffing capacity 6"): with six inside there is no room left (`Sim::room`).
#[test]
fn the_temple_has_no_rally_point_and_takes_six_recruits_only() {
    let (mut t, _) = staffed(6);
    assert_eq!(
        validate(&t, rally(&[T1], Vec2::new(400.0, 736.0))),
        Err(Reject::NoProducers)
    );
    assert_eq!(t.sim().room(T1), 0, "six inside");
    let now = t.sim().tick() + 1;
    let first = next_id(&t);
    t.spawn_unit_at(now, MILITANT, P1, Vec2::new(1150.0, 400.0));
    t.spawn_unit_at(now, RECRUIT, P1, Vec2::new(1150.0, 300.0));
    run_checked(&mut t, now);
    let (militant, recruit) = (id(first), id(first + 1));
    assert_eq!(validate(&t, enter(&[militant], T1)), Err(Reject::NotAccepted(T1)));
    assert_eq!(
        validate(&t, enter(&[militant, recruit], T1)),
        Ok(()),
        "the Recruit of the two"
    );
}
