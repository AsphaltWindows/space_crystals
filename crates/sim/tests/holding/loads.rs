//! A Recruit's load inside a building (Placeholder (Q0104 point 3): "A unit's load is kept inside and comes out with
//! it"; `factions/cults/economy.md > Cults Building Mechanics > Evacuation on Destruction`: "Units come out
//! unchanged"), with real gathering (slice M5-D).

use sc_sim::Target;

use super::*;

/// The load Recruit `unit` carries, on the map or inside.
fn load(t: &TestSim, unit: ObjectId) -> u32 {
    let o = (t.sim().object(unit))
        .or_else(|| t.sim().held_unit(unit).map(|held| &held.object))
        .expect("in the game");
    o.worker.expect("a worker").load
}

/// The plan's acceptance (slice M5-F1): "A carrying Recruit keeps its load inside and out again." A Recruit mines a
/// patch until it carries a full load (`factions/cults/units/recruit.md > Gathering`: CarryCapacity 2), is ordered
/// into the Armory by a right-click ("Own Armory → Enter Armory": the Armory takes no load, so the right-click is no
/// Drop Off), keeps its load inside, comes out with it by Eject All, and delivers it at a Storage afterwards.
#[test]
fn a_carrying_recruit_keeps_its_load_inside_and_out_again() {
    // The Armory (1); a patch (2) and a Storage (3) north-east of it; the Recruit (4) by the patch.
    let mut t = base()
        .spawn_with(
            sc_sim::SpawnSettings::CrystalPatch { starting_amount: 1_000 },
            tile(20, 4),
        )
        .spawn(ObjectKind::Storage, Some(P1), tile(24, 3))
        .start();
    t.without_auto_targeting()
        .spawn_unit_at(1, RECRUIT, P1, Vec2::new(1312.0, 352.0));
    let (patch, storage, recruit) = (id(2), id(3), id(4));
    t.order_at(2, P1, &[recruit], OrderKind::Gather(patch), false);
    run_checked(&mut t, 2);
    while load(&t, recruit) < 2 {
        assert!(t.sim().tick() < 300, "mining\n{}", t.ascii(None));
        step_checked(&mut t);
    }
    let carrying = t.sim().tick() + 1;
    let right_click = t.sim().default_order(recruit, Target::Object(A1));
    assert_eq!(right_click, Some(OrderKind::Enter(A1)));
    t.order_at(carrying, P1, &[recruit], OrderKind::Enter(A1), false);
    while inside(&t, A1).is_empty() {
        assert!(t.sim().tick() < carrying + 300, "walking in\n{}", t.ascii(None));
        step_checked(&mut t);
    }
    assert_eq!(load(&t, recruit), 2, "inside");
    let bank = |t: &TestSim| t.sim().player(P1).expect("in the game").crystals;
    let banked = bank(&t);
    run_for(&mut t, 32);
    assert_eq!(
        (load(&t, recruit), bank(&t)),
        (2, banked),
        "kept inside, delivered nowhere"
    );
    let eject = t.sim().tick() + 1;
    t.at(eject, P1, eject_all(&[A1]));
    run_checked(&mut t, eject + 16);
    let o = t.sim().object(recruit).expect("out again");
    assert!(o.orders.as_ref().unwrap().is_idle());
    assert_eq!(load(&t, recruit), 2, "out again");
    let deliver = t.sim().tick() + 1;
    assert_eq!(
        t.sim().default_order(recruit, Target::Object(storage)),
        Some(OrderKind::DropOff(storage))
    );
    t.order_at(deliver, P1, &[recruit], OrderKind::DropOff(storage), false);
    while load(&t, recruit) > 0 {
        assert!(t.sim().tick() < deliver + 400, "delivering\n{}", t.ascii(None));
        step_checked(&mut t);
    }
    assert_eq!(bank(&t), banked + 2);
}

/// An evacuee carries what it carried: a destroyed Armory's Recruit comes out with its load.
#[test]
fn an_evacuated_recruit_keeps_its_load() {
    let mut t = base()
        .spawn_with(
            sc_sim::SpawnSettings::CrystalPatch { starting_amount: 1_000 },
            tile(20, 10),
        )
        .spawn(ObjectKind::Storage, Some(P1), tile(2, 2))
        .start();
    t.without_auto_targeting()
        .spawn_unit_at(1, RECRUIT, P1, Vec2::new(1312.0, 736.0)); // 4
    let recruit = id(4);
    t.order_at(2, P1, &[recruit], OrderKind::Gather(id(2)), false);
    run_checked(&mut t, 2);
    while load(&t, recruit) < 2 {
        assert!(t.sim().tick() < 300, "mining");
        step_checked(&mut t);
    }
    let carrying = t.sim().tick() + 1;
    t.order_at(carrying, P1, &[recruit], OrderKind::Enter(A1), false);
    while inside(&t, A1).is_empty() {
        assert!(t.sim().tick() < carrying + 300, "walking in");
        step_checked(&mut t);
    }
    let gone = t.sim().tick() + 1;
    remove_at(&mut t, gone, A1);
    run_checked(&mut t, gone);
    assert!(t.sim().object(recruit).is_some(), "evacuated");
    assert_eq!(load(&t, recruit), 2);
}
