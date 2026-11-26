//! The Shrine's doors and its participants (`factions/cults/structures/shrine.md > Ritual Participation`: "Recruits
//! enter the Shrine to take part in the ritual (right-click own Shrine with Recruits selected → Enter Shrine). Only
//! Recruits can participate. Participating Recruits are not consumed. Evacuation on Destruction applies";
//! `factions/cults/structures/shrine.md > Placeholder stats`: "SymmetryType ABAC, **EntranceSide B, ExitSide C** (user, 2026-11-26: like the production and research
//! buildings; participants leave one at a time)"; "Recruit capacity 8"; "**A/S/D/F** release one Recruit of a kind,
//! **V: Eject All** (participants leave; README > Choosing Recruits, Release and Eject All)"), and its destruction
//! (`factions/cults/economy.md > Cults Building Mechanics > Evacuation on Destruction`: "Everyone inside is evacuated
//! alive: [...] Recruits powering the Shrine").

use sc_sim::ProductionCommand;

use super::*;

/// The plan's acceptance: "enter at B; [...] V ejects through C one by one". Three Recruits from the east go in at the
/// entrance, in the middle of side B, their boxes against the side within the entry path's 8 su of it; Eject All lets
/// them out of the doorway of side C one at a time, in the order they went in, each once the one before has walked
/// clear of it, and they stand outside.
#[test]
fn the_recruits_go_in_at_b_and_eject_all_lets_them_out_of_c_one_by_one() {
    let (mut t, order) = manned(3);
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
    let start = t.sim().tick() + 1;
    t.at(start, P1, eject_all(&[S1]));
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
    assert!(inside(&t, S1).is_empty());
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

/// The plan's acceptance: "A/S/D/F release one of a kind" (`HoldCommand::ReleaseOne`; the README: "pressing one sends
/// **one** Recruit of that kind out of the exit at once (no queue)"). Of a Plain, an Enhanced Constitution, another
/// Plain and an Improved Metabolism Recruit inside, A sends the first Plain one out of side C in its own tick, and A
/// again the second; a third A, with no Plain one left, is refused (`Reject::NoRecruit`), as F is with none with Both.
/// S sends the Enhanced Constitution one, and the Improved Metabolism one stays, charging.
#[test]
fn a_release_sends_out_one_recruit_of_its_kind_the_first_in_first() {
    use RecruitKind::{Both, Constitution, Metabolism, Plain};
    let (mut t, order) = manned_with(base(), &[Plain, Constitution, Plain, Metabolism]);
    let kind_of = |t: &TestSim, unit: ObjectId| t.sim().held_unit(unit).unwrap().object.augmentation.kind();
    let plain: Vec<ObjectId> = (order.iter().copied())
        .filter(|&unit| kind_of(&t, unit) == Plain)
        .collect();
    assert_eq!(plain.len(), 2);
    assert_eq!(validate(&t, release_one(&[S1], Both)), Err(Reject::NoRecruit(Both)));
    let start = t.sim().tick() + 1;
    t.at(start, P1, release_one(&[S1], Plain));
    run_checked(&mut t, start);
    assert_eq!(exits(&t), [(start, plain[0])], "the first in, out at once");
    assert!(in_the_exit(t.sim().object(plain[0]).unwrap().pos));
    run_for(&mut t, 30);
    let again = t.sim().tick() + 1;
    t.at(again, P1, release_one(&[S1], Plain));
    run_checked(&mut t, again);
    assert_eq!(exits(&t)[1..], [(again, plain[1])]);
    assert_eq!(validate(&t, release_one(&[S1], Plain)), Err(Reject::NoRecruit(Plain)));
    run_for(&mut t, 30);
    let third = t.sim().tick() + 1;
    t.at(third, P1, release_one(&[S1], Constitution));
    run_checked(&mut t, third);
    let [.., (_, unit)] = exits(&t)[..] else { unreachable!() };
    assert_eq!(t.sim().object(unit).map(|o| o.augmentation.kind()), Some(Constitution));
    let left: Vec<RecruitKind> = inside(&t, S1).iter().map(|&unit| kind_of(&t, unit)).collect();
    assert_eq!(left, [Metabolism]);
    let before = energy(&t, S1);
    step_checked(&mut t);
    assert_eq!(energy(&t, S1) - before, 1024, "one inside");
}

/// "Only Recruits can participate", and "Recruit capacity 8": a Militant's Enter is refused (`Reject::NotAccepted`), a
/// Recruit's of the two taken; with eight inside there is no room left (`Sim::room`). The Shrine has no rally point
/// (its panel has no Set Rally Point; `Reject::NoProducers` for a SetRally to it).
#[test]
fn the_shrine_takes_eight_recruits_only_and_has_no_rally_point() {
    let (mut t, _) = manned(8);
    assert_eq!(t.sim().room(S1), 0, "eight inside");
    let now = t.sim().tick() + 1;
    let first = next_id(&t);
    t.spawn_unit_at(now, MILITANT, P1, Vec2::new(900.0, 400.0));
    t.spawn_unit_at(now, RECRUIT, P1, Vec2::new(900.0, 300.0));
    run_checked(&mut t, now);
    let (militant, recruit) = (id(first), id(first + 1));
    assert_eq!(validate(&t, enter(&[militant], S1)), Err(Reject::NotAccepted(S1)));
    assert_eq!(
        validate(&t, enter(&[militant, recruit], S1)),
        Ok(()),
        "the Recruit of the two"
    );
    let rally = Command::Production {
        objects: vec![S1],
        command: ProductionCommand::SetRally(Target::Point(Vec2::new(200.0, 640.0))),
    };
    assert_eq!(validate(&t, rally), Err(Reject::NoProducers));
}

/// The plan's acceptance: "Destruction evacuates the Recruits and the energy is lost". The Shrine destroyed with its
/// eight Recruits and energy for a Q lets all eight out onto the map (`SimEvent::Evacuated`), player 1's Recruits
/// still; its energy goes with it, and a new Shrine starts at 0. A Recruit healing under its Q keeps its buff and heals
/// on: the buff was given at the moment of casting (`factions/cults/structures/shrine.md > Abilities`: "receive the
/// buff for its duration"; the most literal reading, **Placeholder (not in the design: the caster's destruction)**).
#[test]
fn its_destruction_lets_the_recruits_out_and_its_energy_goes_with_it() {
    let (mut t, order) = manned(8);
    let healing = spawn(&mut t, RECRUIT, P1, AT);
    wound(&mut t, &[(healing, 20.0)]);
    charge(&mut t, 80);
    let c = t.sim().tick() + 1;
    t.at(c, P1, cast(&[S1], Q, AT));
    remove_at(&mut t, c + 10, S1);
    run_checked(&mut t, c + 10);
    assert!(t.sim().object(S1).is_none(), "destroyed");
    let mut evacuated: Vec<ObjectId> = (t.events().iter())
        .filter_map(|(_, e)| match *e {
            SimEvent::Evacuated { unit, building } if building == S1 => Some(unit),
            _ => None,
        })
        .collect();
    evacuated.sort();
    let mut participants = order.clone();
    participants.sort();
    assert_eq!(evacuated, participants, "all eight out");
    for &unit in &order {
        let o = t.sim().object(unit).expect("on the map");
        assert_eq!((o.kind, o.owner), (RECRUIT, Some(P1)));
    }
    assert_eq!(buffs(&t, healing), buff(Q, c + Q_TICKS), "kept");
    let hp_then = hp(&t, healing).0;
    run_for(&mut t, 16);
    assert_eq!(hp(&t, healing).0, hp_then + 2.0, "2 HP a second still");
    let again = spawn_structure(&mut t, SHRINE, P1, tile(20, 14));
    assert_eq!(energy(&t, again), 0, "the energy went with the Shrine");
}
