//! The exit doorway (`core/entities.md > Structure Type > Exit Doorway`, design changes 0080 and 0086): a produced
//! unit appears half inside its Barracks at the middle of side B and walks out. The doorway is the one place where a
//! unit overlaps a structure, and only on its way out. And a Barracks destroyed in combat while a unit walks out of it
//! (the Director's reading, slice A2a).

use sc_sim::mover::Locomotion;
use sc_sim::testing::{ScenarioBuilder, TestSim};
use sc_sim::{
    BuildingOrientation, DebugCommand, MoveGoal, MoveStatus, ObjectId, ObjectKind, OrderKind, ProductionCommand,
    Rotation, SimEvent, Target, Tick, Vec2, math,
};

use super::doors::{DOOR, OUTSIDE, door_of, exit_of, produced, queue, rally, run_checked, step_checked};
use super::queue::crystals;
use super::{BARRACKS, P1, P2, PK, at, base, flat, production, tile};

/// Design change 0080's Verification 1: "Queue 5 Peacekeepers at a Barracks with open ground in front of side B.
/// Each one appears half inside the right end and walks out."
#[test]
fn five_units_appear_half_inside_the_exit_side_and_walk_out() {
    let mut t = base().start();
    queue(&mut t, BARRACKS, 5);
    let point = Vec2::new(1300.0, 600.0);
    rally(&mut t, 1, BARRACKS, point);
    let mut out = Vec::new();
    let mut left_after = Vec::new();
    for tick in 1..=700 {
        step_checked(&mut t, &mut out);
        if let Some(&(made, unit)) = produced(&t).last() {
            if made == tick {
                let o = t.sim().object(unit).unwrap();
                assert_eq!(o.pos, DOOR, "half inside, at the middle of side B");
                assert_eq!(o.facing, 0.0, "facing out, east");
                assert_eq!(exit_of(t.sim(), unit), Some(BARRACKS));
                assert!(
                    o.mover.as_ref().unwrap().velocity == Vec2::ZERO && o.orders.as_ref().unwrap().current.is_none()
                );
            } else if exit_of(t.sim(), unit).is_none() && left_after.len() < produced(&t).len() {
                left_after.push(tick - made);
            }
        }
    }
    let made = produced(&t);
    assert_eq!(
        made.iter().map(|m| m.0).collect::<Vec<_>>(),
        [80, 160, 240, 320, 400],
        "the doorway is free again long before the next unit: production never pauses"
    );
    // 12 su out at 4 su a tick, the last step already towards the rally point: clear of the footprint in the
    // fourth tick after it appears.
    assert_eq!(left_after, [4; 5]);
    for (_, unit) in made {
        assert_eq!(t.status(unit.0), MoveStatus::Arrived, "#{}", unit.0);
        assert!(
            t.pos(unit.0).distance(point) < 130.0,
            "near the rally point: {}",
            t.pos(unit.0)
        );
    }
}

/// `> Exit Doorway` in all 8 orientations (plan acceptance: "Units leave on the correct side in all 8
/// orientations"): the unit appears on the middle of side B, wherever that faces, faces out, walks straight out
/// with no rally point and stops touching the side; with one, it walks there.
#[test]
fn units_leave_through_side_b_in_all_eight_orientations() {
    // The Barracks at tile (12, 6): 3 × 2 unturned (x 768..960, y 384..512), 2 × 3 on its side (x 768..896,
    // y 384..576).
    let cases = [
        (Rotation::R0, false, Vec2::new(960.0, 448.0), Vec2::X),
        (Rotation::R90, false, Vec2::new(832.0, 576.0), Vec2::Y),
        (Rotation::R180, false, Vec2::new(768.0, 448.0), -Vec2::X),
        (Rotation::R270, false, Vec2::new(832.0, 384.0), -Vec2::Y),
        (Rotation::R0, true, Vec2::new(768.0, 448.0), -Vec2::X),
        (Rotation::R90, true, Vec2::new(832.0, 384.0), -Vec2::Y),
        (Rotation::R180, true, Vec2::new(960.0, 448.0), Vec2::X),
        (Rotation::R270, true, Vec2::new(832.0, 576.0), Vec2::Y),
    ];
    for (rotation, flip_h, middle, outward) in cases {
        for rally_point in [None, Some(Vec2::new(1700.0, 900.0)), Some(Vec2::new(300.0, 80.0))] {
            let mut scenario = ScenarioBuilder::new(&flat(30, 16))
                .spawn(ObjectKind::DeploymentCenter, Some(P1), tile(1, 5))
                .spawn(ObjectKind::Barracks, Some(P1), tile(12, 6))
                .build();
            scenario.map.spawns[1].orientation = BuildingOrientation {
                rotation,
                flip_h,
                flip_v: false,
            };
            let mut t = TestSim::new(scenario);
            let what = format!("{rotation:?}, flipped {flip_h}, rally {rally_point:?}");
            let door = door_of(t.sim(), BARRACKS);
            assert_eq!((door.middle(), door.outward()), (middle, outward), "{what}");
            queue(&mut t, BARRACKS, 1);
            if let Some(point) = rally_point {
                rally(&mut t, 1, BARRACKS, point);
            }
            run_checked(&mut t, 80);
            let (_, unit) = produced(&t)[0];
            let o = t.sim().object(unit).unwrap();
            assert_eq!(o.pos, middle, "{what}");
            assert_eq!(math::facing_vector(o.facing).round(), outward, "{what}");
            assert_eq!(exit_of(t.sim(), unit), Some(BARRACKS), "{what}");
            // Out within a few ticks, whichever way the rally point lies.
            run_checked(&mut t, 100);
            assert_eq!(exit_of(t.sim(), unit), None, "{what}: {}", t.ascii(Some(middle)));
            run_checked(&mut t, 700);
            match rally_point {
                None => {
                    assert_eq!(t.pos(unit.0), middle + outward * 12.0, "{what}");
                    assert!(t.sim().object(unit).unwrap().orders.as_ref().unwrap().is_idle());
                }
                Some(point) => {
                    assert_eq!(t.pos(unit.0), point, "{what}");
                    assert_eq!(t.status(unit.0), MoveStatus::Arrived, "{what}");
                }
            }
        }
    }
}

/// Plan acceptance: the units come out "even when 10 earlier units with no rally crowd the exit, and the
/// invariant harness finds no overlap". Design change 0080, point 4: "Own idle units outside step aside for the
/// unit walking out".
#[test]
fn a_crowd_of_idle_units_in_front_of_the_doorway_makes_way() {
    let mut t = base().start();
    queue(&mut t, BARRACKS, 14);
    run_checked(&mut t, 14 * 80 + 40);
    let made = produced(&t);
    assert_eq!(made.len(), 14);
    assert_eq!(
        made.iter().map(|m| m.0).collect::<Vec<_>>(),
        (1..=14).map(|k| k * 80).collect::<Vec<_>>(),
        "the idle units step aside, so production never pauses"
    );
    for &(_, unit) in &made {
        assert_eq!(exit_of(t.sim(), unit), None);
        assert!(t.sim().object(unit).unwrap().orders.as_ref().unwrap().is_idle());
    }
    // The last one stands right outside the doorway; the others made room around it.
    assert_eq!(t.pos(made[13].1.0), OUTSIDE);
    let far = made
        .iter()
        .map(|&(_, unit)| t.pos(unit.0).distance(OUTSIDE))
        .fold(0.0, f32::max);
    assert!(far < 200.0, "they stay around the doorway: {far}");
}

/// Design change 0080's Verification 4: "Stand an own idle unit in front of side B. It steps aside when the new
/// unit comes out."
#[test]
fn an_own_idle_unit_in_front_of_the_doorway_steps_aside() {
    let mut t = base().spawn(PK, Some(P1), at(716.0, 320.0)).start();
    let idle = ObjectId(3);
    queue(&mut t, BARRACKS, 1);
    run_checked(&mut t, 80);
    let (_, unit) = produced(&t)[0];
    // Something stands flush against the exit side: the unit appears fully inside, against that side.
    let pos = t.pos(unit.0);
    assert!(pos.y == 320.0 && pos.x < 692.0 && pos.x > 691.9, "{pos}");
    assert_eq!(t.pos(idle.0), OUTSIDE, "it hasn't moved yet");
    run_checked(&mut t, 100);
    assert_ne!(t.pos(idle.0), OUTSIDE, "it stepped aside");
    assert_eq!((t.pos(unit.0), exit_of(t.sim(), unit)), (OUTSIDE, None));
}

/// Design change 0080's Verification 2: "Park one enemy Peacekeeper right in front of side B. The next produced
/// unit still appears in the doorway and attacks it." One enemy unit never stops production for good.
#[test]
fn an_enemy_in_front_of_the_doorway_is_shot_from_inside_it() {
    let mut t = base().spawn(PK, Some(P2), at(716.0, 320.0)).start();
    let enemy = ObjectId(3);
    t.order_at(1, P2, &[enemy], OrderKind::Hold, false);
    queue(&mut t, BARRACKS, 2);
    run_checked(&mut t, 80);
    let (tick, unit) = produced(&t)[0];
    assert_eq!(tick, 80, "the spawn spot can't be blocked from outside");
    assert_eq!(exit_of(t.sim(), unit), Some(BARRACKS));
    assert!(
        t.pos(unit.0).x < 692.0,
        "inside, against the exit side: {}",
        t.pos(unit.0)
    );
    let mut out = Vec::new();
    let killed = (81..3000).find(|_| {
        step_checked(&mut t, &mut out);
        t.sim().object(enemy).is_none() || t.sim().object(unit).is_none()
    });
    assert!(killed.is_some());
    assert!(
        t.events().iter().any(
            |(_, e)| matches!(e, SimEvent::Attacked { attacker, target, .. } if *attacker == unit && *target == enemy)
        ),
        "the unit in the doorway attacks the blocker"
    );
    // Whoever won the duel, the second unit follows and the doorway empties.
    run_checked(&mut t, 4000);
    assert_eq!(produced(&t).len(), 2);
    assert!(t.sim().object(enemy).is_none(), "two against one");
    assert!(
        t.sim()
            .objects()
            .all(|o| o.mover.as_ref().is_none_or(|m| m.exit.is_none()))
    );
}

/// Something blocks half of the exit side: a unit with somewhere to go comes out beside it. It starts fully
/// inside, plans its path from the free tile in front of the side, and slides along the inside of the doorway.
#[test]
fn a_unit_comes_out_beside_a_half_blocked_exit() {
    // A crystal patch on tile (11, 4), in front of the northern tile of side B.
    let mut t = base().spawn(ObjectKind::CrystalPatch, None, tile(11, 4)).start();
    queue(&mut t, BARRACKS, 2);
    let point = Vec2::new(1300.0, 200.0);
    rally(&mut t, 1, BARRACKS, point);
    run_checked(&mut t, 80);
    let (_, first) = produced(&t)[0];
    assert_eq!(
        t.pos(first.0),
        Vec2::new(692.0, 320.0),
        "the patch's tile is in the way of half inside"
    );
    run_checked(&mut t, 100);
    assert_eq!(exit_of(t.sim(), first), None);
    run_checked(&mut t, 500);
    assert_eq!(produced(&t).iter().map(|m| m.0).collect::<Vec<_>>(), [80, 160]);
    // Both reach the rally point: the first, idle there, makes way for the second.
    for (_, unit) in produced(&t) {
        assert!(t.pos(unit.0).distance(point) < 130.0, "{}", t.pos(unit.0));
        assert_eq!(t.status(unit.0), MoveStatus::Arrived);
    }
}

/// The path of a unit in the doorway starts on a blocked tile, the Barracks' own, when the exit faces west or
/// north (its centre lies in the footprint) and whenever it stands fully inside. It still plans, and walks round
/// its own Barracks to a rally point behind it.
#[test]
fn a_unit_plans_its_way_out_of_the_footprint_to_a_rally_point_behind_the_building() {
    for (rotation, behind) in [
        (Rotation::R0, Vec2::new(500.0, 448.0)),
        (Rotation::R180, Vec2::new(1200.0, 448.0)),
        (Rotation::R90, Vec2::new(832.0, 200.0)),
        (Rotation::R270, Vec2::new(832.0, 800.0)),
    ] {
        let mut scenario = ScenarioBuilder::new(&flat(30, 16))
            .spawn(ObjectKind::DeploymentCenter, Some(P1), tile(1, 10))
            .spawn(ObjectKind::Barracks, Some(P1), tile(12, 6))
            .build();
        scenario.map.spawns[1].orientation.rotation = rotation;
        let mut t = TestSim::new(scenario);
        queue(&mut t, BARRACKS, 1);
        rally(&mut t, 1, BARRACKS, behind);
        run_checked(&mut t, 81);
        let (_, unit) = produced(&t)[0];
        let mover = t.mover(unit.0);
        let door = door_of(t.sim(), BARRACKS);
        assert_eq!(mover.status, MoveStatus::Moving, "{rotation:?}: it has a path");
        assert_eq!(
            mover.path[0],
            door.spot(12.0),
            "{rotation:?}: which starts just outside the doorway"
        );
        run_checked(&mut t, 400);
        assert_eq!(t.pos(unit.0), behind, "{rotation:?}");
        assert_eq!(t.status(unit.0), MoveStatus::Arrived);
    }
}

/// "Once the unit is fully out, it can't re-enter that space": a unit sent back through its Barracks walks round
/// it, and one sent onto it stops at the wall.
#[test]
fn a_unit_that_is_out_cannot_go_back_in() {
    let mut t = base().start();
    queue(&mut t, BARRACKS, 1);
    run_checked(&mut t, 100);
    let (_, unit) = produced(&t)[0];
    assert_eq!((t.pos(unit.0), exit_of(t.sim(), unit)), (OUTSIDE, None));
    // Onto the Barracks: it goes to a point beside it (`run_checked` sees that it never overlaps the footprint).
    t.order_at(
        101,
        P1,
        &[unit],
        OrderKind::Move(Target::Point(Vec2::new(650.0, 320.0))),
        false,
    );
    run_checked(&mut t, 200);
    assert!(
        t.pos(unit.0).x >= 716.0 && t.pos(unit.0).distance(OUTSIDE) < 80.0,
        "{}",
        t.pos(unit.0)
    );
    assert!(t.sim().object(unit).unwrap().orders.as_ref().unwrap().is_idle());
    // To the other side of it.
    t.order_at(
        201,
        P1,
        &[unit],
        OrderKind::Move(Target::Point(Vec2::new(480.0, 320.0))),
        false,
    );
    run_checked(&mut t, 400);
    assert_eq!(t.pos(unit.0), Vec2::new(480.0, 320.0));
    // A debug teleport takes a unit out of a doorway, and the exception with it.
    let mut t = base().spawn(ObjectKind::Generator, Some(P1), tile(11, 4)).start();
    queue(&mut t, BARRACKS, 2);
    run_checked(&mut t, 90);
    let (_, unit) = produced(&t)[0];
    assert_eq!(exit_of(t.sim(), unit), Some(BARRACKS));
    t.debug_at(
        91,
        DebugCommand::SetPosition {
            id: unit,
            pos: Vec2::new(1000.0, 700.0),
        },
    );
    run_checked(&mut t, 200);
    assert_eq!(
        (t.pos(unit.0), exit_of(t.sim(), unit)),
        (Vec2::new(1000.0, 700.0), None)
    );
    assert_eq!(produced(&t).len(), 2, "and the doorway is free for the next unit");
}

/// [`base`] with six of player 2's Peacekeepers south of the Barracks (3 to 8), in range of its wall, which take an
/// Attack on it in tick 2 and destroy it in combat. Auto-targeting is off: nothing else fights.
fn under_fire() -> TestSim {
    let mut builder = base();
    for i in 0..6 {
        builder = builder.spawn(PK, Some(P2), at(530.0 + 30.0 * i as f32, 500.0));
    }
    let mut t = builder.start();
    t.without_auto_targeting();
    let attackers: Vec<ObjectId> = (3..9).map(ObjectId).collect();
    t.order_at(2, P2, &attackers, OrderKind::Attack(BARRACKS), false);
    t
}

/// The tick in which [`under_fire`]'s Barracks is destroyed, with nothing in production.
fn destroyed_in() -> Tick {
    let mut t = under_fire();
    let gone = (1..3000).find(|_| {
        t.step();
        t.sim().object(BARRACKS).is_none()
    });
    gone.expect("the Barracks is destroyed")
}

/// [`under_fire`], with a Peacekeeper trained so that it appears in the doorway two ticks before the Barracks goes,
/// rallied to `rally_point` if there is one: it is still walking out when the Barracks is destroyed, in combat, after
/// the tick's movement (phase 5e). Runs the game to the end of that tick, and returns it with the unit.
fn destroyed_while_walking_out(rally_point: Option<Vec2>) -> (TestSim, ObjectId, Tick) {
    let gone = destroyed_in();
    let mut t = under_fire();
    t.debug_at(1, crystals(P1, 50));
    if let Some(point) = rally_point {
        rally(&mut t, 1, BARRACKS, point);
    }
    // 80 ticks in production.
    t.at(gone - 81, P1, production(&[BARRACKS], ProductionCommand::Train(PK)));
    run_checked(&mut t, gone - 1);
    let made = produced(&t);
    assert_eq!(made.len(), 1);
    let (appeared, unit) = made[0];
    assert_eq!(appeared, gone - 2);
    let mover = t.mover(unit.0);
    assert_eq!(mover.exit, Some(BARRACKS), "in the doorway");
    assert!(
        mover.locomotion == Locomotion::Moving && mover.velocity != Vec2::ZERO,
        "walking out: {mover:?}"
    );
    run_checked(&mut t, gone);
    assert!(t.sim().object(BARRACKS).is_none(), "destroyed in tick {gone}");
    (t, unit, gone)
}

/// `> Exit Doorway`, point 2: "the unit then walks out toward its RallyPoint (or straight out of the exit side if
/// there is none)". It doesn't say what happens when the building is destroyed meanwhile; the Director's reading
/// (slice A2a): a unit walking out with no goal stands where it is, as one that has walked out does. A unit with no
/// rally point, whose Barracks is destroyed as it walks out, comes to rest in that tick, part way out, and stays there,
/// idle, for the 100 ticks after, every invariant holding in each (I7: a unit without a goal is at rest).
#[test]
fn a_unit_walking_out_with_no_rally_point_stands_when_its_barracks_is_destroyed() {
    let (mut t, unit, gone) = destroyed_while_walking_out(None);
    let state = |t: &TestSim| {
        let mover = t.mover(unit.0);
        (mover.goal, mover.locomotion, mover.velocity, mover.exit)
    };
    let at_rest = (MoveGoal::None, Locomotion::Stationary, Vec2::ZERO, None);
    assert_eq!(state(&t), at_rest);
    let pos = t.pos(unit.0);
    assert!(
        pos.y == DOOR.y && pos.x > DOOR.x && pos.x < OUTSIDE.x,
        "part way out: {pos}"
    );
    run_checked(&mut t, gone + 100);
    assert_eq!(state(&t), at_rest);
    assert_eq!(t.pos(unit.0), pos, "where it stopped");
    assert!(t.sim().object(unit).unwrap().orders.as_ref().unwrap().is_idle());
}

/// The same with a rally point: one walking out toward its rally point keeps going there (the Director's reading).
/// Its Barracks destroyed as it walks out, the unit carries on to its rally point and arrives there.
#[test]
fn a_unit_walking_out_to_its_rally_point_goes_on_there_when_its_barracks_is_destroyed() {
    let point = Vec2::new(1300.0, 600.0);
    let (mut t, unit, gone) = destroyed_while_walking_out(Some(point));
    let mover = t.mover(unit.0);
    assert_eq!(
        (mover.goal, mover.locomotion, mover.exit),
        (MoveGoal::Point(point), Locomotion::Moving, None)
    );
    run_checked(&mut t, gone + 300);
    assert_eq!(t.pos(unit.0), point);
    assert_eq!(t.status(unit.0), MoveStatus::Arrived);
}
