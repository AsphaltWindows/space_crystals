//! Design change 0347 (`factions/cults/structures/README.md > Choosing Recruits, Release and Eject All`, "Entering pool
//! buildings"): point 5, "**Capacity counts everyone inside**, reserved units included: the queue can never hold more
//! orders than the building's capacity allows (e.g. the Armory: 10 Recruits inside in all, so at most 10 queued
//! orders)"; point 3, "a Recruit carrying Space Crystals **keeps its load** inside and comes out with it; a Recruit
//! consumed by training loses it". (M5-S2b) And design change 0378: a right-click on a full damaged Armory, its
//! reserved Recruits counted, is a Repair.

use sc_sim::SpawnSettings;

use super::*;

/// Design change 0347's Verification: "An Armory with 10 Recruits inside and 10 queued refuses an 11th Recruit, which
/// idles beside it." Ten Militants queued reserve the ten Recruits, and an eleventh Militant finds none; an eleventh
/// Recruit walking in has its Enter fail for want of room (`SimEvent::EnterFailed` with `full`) and idles by the
/// entrance. Once the first Militant is out, there is room for one again.
#[test]
fn an_armory_with_ten_inside_and_ten_queued_refuses_an_eleventh_recruit() {
    let (mut t, order) = filled(10);
    let start = t.sim().tick() + 1;
    for _ in 0..10 {
        t.at(start, P1, train(&[A1], RecruitKind::Plain));
    }
    let eleventh = id(next_id(&t));
    t.spawn_unit_at(start, RECRUIT, P1, Vec2::new(1060.0, 704.0));
    t.at(start + 1, P1, enter(&[eleventh], A1));
    run_checked(&mut t, start + 1);
    assert_eq!(
        reserved(&t, A1),
        order.iter().map(|&unit| vec![unit]).collect::<Vec<_>>()
    );
    assert_eq!(bank(&t, P1), BANK - 200);
    assert_eq!(t.sim().room(A1), 0, "the reserved ones count");
    assert_eq!(
        validate(&t, train(&[A1], RecruitKind::Plain)),
        Err(Reject::NoRecruit(RecruitKind::Plain))
    );
    let failed = |t: &TestSim| {
        (t.events().iter()).any(|(_, e)| {
            *e == SimEvent::EnterFailed {
                unit: eleventh,
                target: A1,
                full: true,
            }
        })
    };
    while !failed(&t) {
        assert!(
            t.sim().tick() < start + 30,
            "the eleventh's Enter fails before a Militant is out"
        );
        step_checked(&mut t);
    }
    assert!(produced(&t).is_empty());
    let o = t.sim().object(eleventh).expect("on the map");
    assert!(o.orders.as_ref().unwrap().is_idle());
    assert!(
        o.pos.distance(Vec2::new(972.0, 704.0)) < 24.0,
        "by the entrance: {}",
        o.pos
    );
    run_checked(&mut t, start + 31);
    assert_eq!(produced(&t).len(), 1);
    assert_eq!(t.sim().room(A1), 1);
    let again = t.sim().tick() + 1;
    t.at(again, P1, enter(&[eleventh], A1));
    run_checked(&mut t, again);
    fill(&mut t, A1, 10);
    assert!(inside(&t, A1).contains(&eleventh));
}

/// (M5-S2b) Design change 0378, answering Q0140 (`factions/cults/structures/README.md > Entering pool buildings`:
/// "**Right-click on a full building** [...]: if it is also damaged, Recruits' right-click resolves to **Repair**
/// instead of Enter"; 0378: "fullness, counting reserved units, is checked at click time", "This applies to every
/// building that holds units"): player 2's Militant damages the Armory, ten Recruits inside, and ten Militants queued
/// reserve them all. Nothing is left in its pool, yet it is full: an eleventh Recruit's right-click on it is a Repair.
/// Once the first Militant is out there is room for one, and the right-click enters (design change 0325: "**Enter**
/// wins").
#[test]
fn a_full_damaged_armory_is_repaired_on_a_right_click_its_reserved_recruits_counted() {
    let (mut t, _) = filled(10);
    let raider = id(next_id(&t));
    let now = t.sim().tick() + 1;
    t.spawn_unit_at(now, MILITANT, P2, Vec2::new(864.0, 820.0));
    t.order_at(now + 1, P2, &[raider], OrderKind::Attack(A1), false);
    let max = t.sim().object(A1).and_then(|o| o.max_hp()).expect("it has HP");
    while t.sim().object(A1).and_then(|o| o.hp) == Some(max) {
        assert!(t.sim().tick() < now + 100, "the raider hits it");
        step_checked(&mut t);
    }
    let start = t.sim().tick() + 1;
    t.debug_at(start, DebugCommand::Remove { id: raider });
    for _ in 0..10 {
        t.at(start, P1, train(&[A1], RecruitKind::Plain));
    }
    let eleventh = id(next_id(&t));
    t.spawn_unit_at(start, RECRUIT, P1, Vec2::new(1060.0, 704.0));
    run_checked(&mut t, start);
    let right_click = |t: &TestSim| t.sim().default_order(eleventh, Target::Object(A1));
    assert_eq!(reserved(&t, A1).len(), 10);
    assert_eq!(
        (t.sim().pool(A1).len(), t.sim().room(A1)),
        (0, 0),
        "all reserved, and full"
    );
    assert_eq!(right_click(&t), Some(OrderKind::Repair(A1)));
    run_checked(&mut t, start + 31);
    assert_eq!(produced(&t).len(), 1);
    assert_eq!(t.sim().room(A1), 1);
    assert!(t.sim().object(A1).and_then(|o| o.hp) < Some(max), "still damaged");
    assert_eq!(right_click(&t), Some(OrderKind::Enter(A1)));
}

/// A Recruit that mines a full load (`factions/cults/units/recruit.md > Gathering`: CarryCapacity 2) and goes in
/// keeps it inside; trained into a Militant, which carries nothing (no worker), the load is gone and delivered
/// nowhere, while a second carrying Recruit, Released, comes out with its load.
#[test]
fn a_trained_recruit_loses_its_load_and_a_released_one_keeps_it() {
    // The Armory (1); a patch (2) and a Storage (3) north-east of it; the Recruits (4, 5) by the patch.
    let mut t = base()
        .spawn_with(SpawnSettings::CrystalPatch { starting_amount: 1_000 }, tile(20, 4))
        .spawn(ObjectKind::Storage, Some(P1), tile(24, 3))
        .start();
    t.without_auto_targeting()
        .spawn_unit_at(1, RECRUIT, P1, Vec2::new(1312.0, 352.0))
        .spawn_unit_at(1, RECRUIT, P1, Vec2::new(1312.0, 224.0));
    t.debug_at(1, crystals(P1, BANK));
    let (patch, recruits) = (id(2), [id(4), id(5)]);
    t.order_at(2, P1, &recruits, OrderKind::Gather(patch), false);
    run_checked(&mut t, 2);
    let load = |t: &TestSim, unit: ObjectId| {
        let o = (t.sim().object(unit)).or_else(|| t.sim().held_unit(unit).map(|held| &held.object));
        o.expect("in the game").worker.map(|worker| worker.load)
    };
    // Each goes in once it carries a full load.
    for unit in recruits {
        while load(&t, unit) < Some(2) {
            assert!(t.sim().tick() < 600, "mining\n{}", t.ascii(None));
            step_checked(&mut t);
        }
        let carrying = t.sim().tick() + 1;
        t.order_at(carrying, P1, &[unit], OrderKind::Enter(A1), false);
        run_checked(&mut t, carrying);
    }
    fill(&mut t, A1, 2);
    let order = inside(&t, A1);
    assert!(order.iter().all(|&unit| load(&t, unit) == Some(2)), "kept inside");
    let banked = bank(&t, P1);
    let start = t.sim().tick() + 1;
    t.at(start, P1, train(&[A1], RecruitKind::Plain));
    t.at(start, P1, release(&[A1], RecruitKind::Plain));
    run_for(&mut t, 64);
    assert_eq!(
        produced(&t).iter().map(|&(_, _, unit)| unit).collect::<Vec<_>>(),
        [order[0]]
    );
    assert_eq!(exits(&t).iter().map(|&(_, unit)| unit).collect::<Vec<_>>(), [order[1]]);
    let militant = t.sim().object(order[0]).expect("out");
    assert_eq!(
        (militant.kind, load(&t, order[0])),
        (MILITANT, None),
        "a Militant carries nothing"
    );
    assert_eq!(load(&t, order[1]), Some(2), "Released with its load");
    assert_eq!(bank(&t, P1), banked - 20, "the load delivered nowhere");
}
