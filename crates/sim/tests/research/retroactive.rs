//! A completed level changes everyone (`factions/cults/augmentation.md > Research applies to everyone`): "Temple
//! research is **retroactive**: when a level completes, **every unit that has that Augmentation** gets the new amounts
//! at once, along with units augmented later. HP follows the MaxHP rule (Increments > HP when MaxHP changes): if MaxHP
//! goes up, HP rises by the same amount"; `factions/cults/augmentation.md > Increments`: "**MaxHP goes down**: HP is
//! **unchanged**, unless it is now above the new MaxHP, in which case it becomes the new MaxHP"; and the amounts
//! (`factions/cults/augmentation.md > Augmentation Effects`): "Enhanced Constitution | +2 Constitution / −1 Metabolism
//! | +4 Constitution / −2 Metabolism", "Improved Metabolism | +2 Metabolism / −1 Constitution | +4 Metabolism / −2
//! Constitution", a Constitution increment "+10 MaxHP; +1 melee Damage (unarmed units only)", a Metabolism increment
//! "+0.5 space units/frame MaxSpeed", for an unarmed unit also "−1 frame of FiringDuration" ("a negative increment adds
//! 1 frame"). Every stat is read through the accessors, at its owner's research levels (`Sim::research`).

use sc_sim::{AugmentOrder, combat};

use super::*;

/// Unit `unit`'s melee Damage, FiringDuration and MaxSpeed at its owner's research levels, on the map.
fn stats(t: &TestSim, unit: ObjectId) -> (f32, u32, f32) {
    let research = t.sim().research();
    let o = t.sim().object(unit).expect("on the map");
    let weapon = combat::weapon(o, research).expect("it strikes");
    (weapon.damage, weapon.firing_ticks, o.max_speed(research))
}

/// Player 1's units of `kinds`, spawned in a column at x = 1500 around y = 300 in the next tick, each given the
/// Augmentations of its kind (a debug command, the tick after): their ids, with that tick run.
fn spawn_augmented(t: &mut TestSim, kinds: &[(ObjectKind, RecruitKind)]) -> Vec<ObjectId> {
    let now = t.sim().tick() + 1;
    let first = next_id(t);
    for (i, &(kind, _)) in (0..).zip(kinds) {
        t.spawn_unit_at(now, kind, P1, Vec2::new(1500.0, 200.0 + 64.0 * i as f32));
    }
    let units: Vec<ObjectId> = (0..kinds.len() as u32).map(|i| id(first + i)).collect();
    for (&unit, &(_, augmentation)) in units.iter().zip(kinds) {
        if augmentation != RecruitKind::Plain {
            set_augmentation(t, now + 1, unit, augmentation);
        }
    }
    run_checked(t, now + 1);
    units
}

/// Queues `tech` at the Temple in the next tick and runs until it completes: the tick it does.
fn complete(t: &mut TestSim, tech: Tech) -> Tick {
    let now = t.sim().tick() + 1;
    t.at(now, P1, research(&[T1], tech));
    let before = completed(t).len();
    while completed(t).len() == before {
        assert!(t.sim().tick() < now + L2_FRAMES, "{tech:?} never completes");
        step_checked(t);
    }
    let (done, _, completed_tech) = *completed(t).last().unwrap();
    assert_eq!(completed_tech, tech);
    done
}

/// Design change 0336's Verification: "Augment a Recruit, then complete Enhanced Constitution L1: the existing
/// Recruit's MaxHP rises from +20 to +40, and its HP rises by 20." Two Enhanced Constitution Recruits on the map, one
/// hurt and one at full HP, 60 MaxHP (40, and +2 Constitution): in the tick Level 1 completes they go to 80, +4
/// Constitution, their HP 20 up, so the hurt one keeps its damage and the other stays full; their melee Damage from 5
/// to 7, their FiringDuration a frame longer and their MaxSpeed 0.5 lower (−2 Metabolism for −1), as an unarmed unit's.
/// A Plain Recruit doesn't change.
#[test]
fn level_1_raises_an_enhanced_constitution_recruits_max_hp_from_plus_20_to_plus_40_and_its_hp_by_20() {
    let (mut t, _) = staffed(6);
    let constitution = (RECRUIT, RecruitKind::Constitution);
    let units = spawn_augmented(&mut t, &[constitution, constitution, (RECRUIT, RecruitKind::Plain)]);
    let (hurt_one, full, plain) = (units[0], units[1], units[2]);
    let left = hurt(&mut t, hurt_one, 45.0);
    assert!(left > 20.0 && left <= 45.0, "{left}");
    assert_eq!(
        [hp(&t, hurt_one), hp(&t, full), hp(&t, plain)],
        [(left, 60.0), (60.0, 60.0), (40.0, 40.0)]
    );
    let (damage, firing, speed) = stats(&t, full);
    assert_eq!(damage, 5.0, "3, and +2 Constitution");
    let plain_stats = stats(&t, plain);
    let now = t.sim().tick() + 1;
    t.at(now, P1, research(&[T1], EC1));
    run_checked(&mut t, now + L1_FRAMES / 6 - 2);
    assert_eq!(hp(&t, hurt_one), (left, 60.0), "the tick before");
    step_checked(&mut t);
    assert_eq!(completed(&t), [(now + L1_FRAMES / 6 - 1, T1, EC1)]);
    assert_eq!(hp(&t, hurt_one), (left + 20.0, 80.0), "at once, its HP 20 up");
    assert_eq!(hp(&t, full), (80.0, 80.0), "full, it stays full");
    assert_eq!(stats(&t, full), (7.0, firing + 1, speed - 0.5));
    assert_eq!(hp(&t, plain), (40.0, 40.0));
    assert_eq!(stats(&t, plain), plain_stats, "no Augmentation, no change");
}

/// Improved Metabolism Level 1 lowers the MaxHP of every unit with Improved Metabolism, from −1 Constitution to −2, and
/// HP above the new MaxHP becomes it ("40/40 → 10/10; 8/40 → 8/10"): an Improved Metabolism Recruit at full HP goes
/// from 30/30 to 20/20, one hurt to 25 or below but above 20 to 20/20, one at 15 or below keeps its HP, of 20; a
/// Recruit with Both from 50/50 (+1 Constitution) to 40/40 (+0), its MaxSpeed 1 higher (+3 Metabolism for +1); an
/// Improved Metabolism Militant, armed, from 30/30 to 20/20 and 1 su a frame faster, its weapon unchanged ("Armed
/// infantry (Militant, Gunner) receive only the MaxHP part of Constitution increments and only the movement-speed part
/// of Metabolism increments"). An Enhanced Constitution Recruit, of the other line, doesn't change.
#[test]
fn improved_metabolism_level_1_lowers_max_hp_and_caps_hp_of_its_units_with_both_too() {
    let (mut t, _) = staffed(6);
    let metabolism = (RECRUIT, RecruitKind::Metabolism);
    let kinds = [
        metabolism,
        metabolism,
        metabolism,
        (RECRUIT, RecruitKind::Both),
        (MILITANT, RecruitKind::Metabolism),
        (RECRUIT, RecruitKind::Constitution),
    ];
    let units = spawn_augmented(&mut t, &kinds);
    let middle = hurt(&mut t, units[1], 25.0);
    assert!(middle > 20.0, "{middle}");
    let low = hurt(&mut t, units[2], 15.0);
    let before: Vec<(f32, f32)> = units.iter().map(|&unit| hp(&t, unit)).collect();
    assert_eq!(
        before,
        [
            (30.0, 30.0),
            (middle, 30.0),
            (low, 30.0),
            (50.0, 50.0),
            (30.0, 30.0),
            (60.0, 60.0)
        ]
    );
    let speeds = |t: &TestSim| units.iter().map(|&unit| stats(t, unit).2).collect::<Vec<f32>>();
    let (speed, militant) = (speeds(&t), stats(&t, units[4]));
    complete(&mut t, IM1);
    let after: Vec<(f32, f32)> = units.iter().map(|&unit| hp(&t, unit)).collect();
    assert_eq!(
        after,
        [
            (20.0, 20.0),
            (20.0, 20.0),
            (low, 20.0),
            (40.0, 40.0),
            (20.0, 20.0),
            (60.0, 60.0)
        ]
    );
    let faster: Vec<f32> = speeds(&t).iter().zip(&speed).map(|(now, then)| now - then).collect();
    assert_eq!(faster, [1.0, 1.0, 1.0, 1.0, 1.0, 0.0], "+0.5 a Metabolism increment");
    let (damage, firing, _) = stats(&t, units[4]);
    assert_eq!((damage, firing), (militant.0, militant.1), "the Militant's weapon");
}

/// "every unit that has that Augmentation": held ones too. The Temple's six staff, Enhanced Constitution Recruits at
/// 60/60, are 80/80 once their own research is done, and an Enhanced Constitution Recruit hurt and then sent into an
/// Armory (3) of player 1's comes out of it later with its HP 20 up and its damage kept.
#[test]
fn held_units_take_the_new_amounts_too() {
    let armory = id(3);
    let scenario = base().spawn(ARMORY, Some(P1), tile(20, 14)); // 3
    let (mut t, staff) = staffed_with(scenario, &[RecruitKind::Constitution; 6]);
    for &unit in &staff {
        assert_eq!(hp(&t, unit), (60.0, 60.0));
    }
    let [held] = spawn_augmented(&mut t, &[(RECRUIT, RecruitKind::Constitution)])[..] else {
        unreachable!()
    };
    let left = hurt(&mut t, held, 45.0);
    let now = t.sim().tick() + 1;
    t.at(now, P1, enter(&[held], armory));
    run_checked(&mut t, now);
    fill(&mut t, armory, 1);
    assert_eq!(hp(&t, held), (left, 60.0), "inside the Armory");
    complete(&mut t, EC1);
    for &unit in &staff {
        assert!(t.sim().held_unit(unit).is_some(), "still inside");
        assert_eq!(hp(&t, unit), (80.0, 80.0), "#{}", unit.0);
    }
    assert_eq!(hp(&t, held), (left + 20.0, 80.0), "held in the Armory");
    let now = t.sim().tick() + 1;
    t.at(now, P1, eject_all(&[armory]));
    while t.sim().object(held).is_none() {
        assert!(t.sim().tick() < now + 100, "it comes out");
        step_checked(&mut t);
    }
    assert_eq!(hp(&t, held), (left + 20.0, 80.0), "out, as it was");
}

/// "along with units augmented later": after Enhanced Constitution Level 1, an Altar's Enhanced Constitution order (25
/// Space Crystals, 48 frames since design change 0391) on a Plain Recruit at full HP gives it +4 Constitution: it comes
/// out at 80/80.
#[test]
fn a_unit_augmented_after_level_1_gets_its_amounts() {
    let altar = id(3);
    let scenario = base().spawn(ALTAR, Some(P1), tile(20, 4)); // 3
    let (mut t, _) = staffed_with(scenario, &[RecruitKind::Plain; 6]);
    complete(&mut t, EC1);
    let [recruit] = spawn_augmented(&mut t, &[(RECRUIT, RecruitKind::Plain)])[..] else {
        unreachable!()
    };
    let now = t.sim().tick() + 1;
    t.at(now, P1, enter(&[recruit], altar));
    run_checked(&mut t, now);
    fill(&mut t, altar, 1);
    let order = ProductionCommand::Augment {
        kind: RECRUIT,
        from: RecruitKind::Plain,
        order: AugmentOrder::EnhancedConstitution,
    };
    let start = t.sim().tick() + 1;
    t.at(start, P1, production(&[altar], order));
    while t.sim().object(recruit).is_none() {
        assert!(t.sim().tick() < start + 200, "augmented and out");
        step_checked(&mut t);
    }
    assert_eq!(
        t.sim().object(recruit).map(|o| o.augmentation),
        Some(Augmentation::of(RecruitKind::Constitution))
    );
    assert_eq!(hp(&t, recruit), (80.0, 80.0));
}

/// The Director's note on the same tick (plan decision T74: "the research level in effect at that moment" includes
/// research finishing that frame; with fixed amounts either order gives the same): an Altar's Enhanced Constitution
/// order on a hurt Plain Recruit (h/40) and Enhanced Constitution Level 1 finish in the same tick, 160 ticks after the
/// research is queued: the order, 48 ticks long (design change 0391), is queued 112 ticks after it. The Recruit comes
/// out with +4 Constitution, at 80 MaxHP, its HP 40 up.
#[test]
fn an_augmentation_finishing_in_the_tick_level_1_completes_gets_level_1() {
    let altar = id(3);
    let scenario = base().spawn(ALTAR, Some(P1), tile(20, 4)); // 3
    let (mut t, _) = staffed_with(scenario, &[RecruitKind::Plain; 6]);
    let [recruit] = spawn_augmented(&mut t, &[(RECRUIT, RecruitKind::Plain)])[..] else {
        unreachable!()
    };
    let left = hurt(&mut t, recruit, 30.0);
    let now = t.sim().tick() + 1;
    t.at(now, P1, enter(&[recruit], altar));
    run_checked(&mut t, now);
    fill(&mut t, altar, 1);
    let order = ProductionCommand::Augment {
        kind: RECRUIT,
        from: RecruitKind::Plain,
        order: AugmentOrder::EnhancedConstitution,
    };
    let start = t.sim().tick() + 1;
    t.at(start, P1, research(&[T1], EC1));
    t.at(start + 112, P1, production(&[altar], order));
    run_checked(&mut t, start + 158);
    assert!(
        t.sim().object(recruit).is_none() && completed(&t).is_empty(),
        "neither yet"
    );
    step_checked(&mut t);
    assert_eq!(completed(&t), [(start + 159, T1, EC1)]);
    let out = (t.events().iter()).any(|(tick, e)| {
        *tick == start + 159
            && *e
                == SimEvent::UnitProduced {
                    producer: altar,
                    unit: recruit,
                }
    });
    assert!(out, "out in the same tick");
    assert_eq!(hp(&t, recruit), (left + 40.0, 80.0), "+4 Constitution, its damage kept");
}

/// "**Speed and weapon** are read live through the accessors" (the Director's brief), so a level completing mid-move
/// breaks nothing: an Enhanced Constitution Militant and an Improved Metabolism Militant walk east along the south of
/// the map while Enhanced Constitution Level 1 and then Improved Metabolism Level 1 complete; `TestSim`'s invariants,
/// I3 (a unit moves at most its speed a tick) among them, hold every tick, and each walks at its new speed from the
/// tick after: the first 0.5 su a frame slower (3.5 to 3.0), the second 1 faster (5.0 to 6.0).
#[test]
fn walking_units_take_their_new_speed_from_the_next_tick() {
    let (mut t, _) = staffed(6);
    let now = t.sim().tick() + 1;
    let first = next_id(&t);
    t.spawn_unit_at(now, MILITANT, P1, Vec2::new(60.0, 1250.0));
    t.spawn_unit_at(now, MILITANT, P1, Vec2::new(60.0, 1320.0));
    let (constitution, metabolism) = (id(first), id(first + 1));
    set_augmentation(&mut t, now + 1, constitution, RecruitKind::Constitution);
    set_augmentation(&mut t, now + 1, metabolism, RecruitKind::Metabolism);
    run_checked(&mut t, now + 1);
    let walk = |unit: ObjectId, y: f32| Command::Order {
        units: vec![unit],
        kind: OrderKind::Move(Target::Point(Vec2::new(1850.0, y))),
        queue: false,
    };
    let start = t.sim().tick() + 1;
    t.at(start, P1, walk(constitution, 1250.0));
    t.at(start, P1, walk(metabolism, 1320.0));
    t.at(start + 1, P1, research(&[T1], EC1));
    t.at(start + 1, P1, research(&[T1], IM1));
    let speed = |t: &TestSim, unit| t.sim().object(unit).unwrap().max_speed(t.sim().research());
    assert_eq!((speed(&t, constitution), speed(&t, metabolism)), (3.5, 5.0));
    let mut moved = Vec::new();
    while completed(&t).len() < 2 || moved.len() < 4 {
        assert!(t.sim().tick() < start + 400, "both done");
        let pos = |t: &TestSim, unit| t.sim().object(unit).unwrap().pos;
        let before = [pos(&t, constitution), pos(&t, metabolism)];
        let done = completed(&t).len();
        step_checked(&mut t);
        if completed(&t).len() > done || moved.len() % 2 == 1 {
            let after = [pos(&t, constitution), pos(&t, metabolism)];
            moved.push([before[0].distance(after[0]), before[1].distance(after[1])]);
        }
    }
    assert_eq!((speed(&t, constitution), speed(&t, metabolism)), (3.0, 6.0));
    let near = |a: f32, b: f32| (a - b).abs() < 1e-3;
    // The tick Enhanced Constitution Level 1 completes, the moves are those of the speeds before; the tick after, the
    // Enhanced Constitution Militant's is its new speed.
    assert!(near(moved[0][0], 3.5) && near(moved[1][0], 3.0), "{moved:?}");
    assert!(near(moved[2][1], 5.0) && near(moved[3][1], 6.0), "{moved:?}");
}
