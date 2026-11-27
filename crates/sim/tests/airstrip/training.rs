//! The Airstrip's training (`super`; `factions/cults/structures/airstrip.md > Production`: "**Production works like the
//! Armory** [...]: Recruits enter an internal pool, orders are serial and queueable and reserve the aircraft's number of
//! Recruits, which are consumed on completion"), its pool's way out on the ground, and the right-click that sends
//! Recruits in.

use super::*;

/// Player 1's Recruit `unit` with the augmentations of kind `kind`, hurt by one blow of player 2's Recruit `attacker`
/// (3 damage, `factions/cults/units/recruit.md`), which then leaves the game; then into the Airstrip. Its HP once
/// inside.
fn hurt_and_inside(t: &mut TestSim, unit: ObjectId, attacker: ObjectId, kind: RecruitKind) -> f32 {
    augment(t, 2, unit, kind);
    t.at(
        3,
        P2,
        Command::Order {
            units: vec![attacker],
            kind: OrderKind::Attack(unit),
            queue: false,
        },
    );
    run_checked(t, 3);
    let max = t.sim().object(unit).and_then(|o| o.max_hp_with(t.sim().research()));
    while t.sim().object(unit).and_then(|o| o.hp) == max {
        assert!(t.sim().tick() < 60, "a blow lands");
        step_checked(t);
    }
    let removed = t.sim().tick() + 1;
    remove_at(t, removed, attacker);
    t.at(removed, P1, enter(&[unit], A1));
    run_checked(t, removed);
    fill(t, A1, 1);
    t.sim().held_unit(unit).and_then(|held| held.object.hp).expect("hurt")
}

/// Design change 0330 (`factions/cults/structures/airstrip.md`: "**Full HP** (user, 2026-11-26; Q0121): a vehicle or
/// aircraft comes out at **full HP**, whatever damage its Recruits had"; `factions/cults/augmentation.md >
/// Persistence`: "if augmented Recruits are consumed to produce a vehicle, their Augmentations are lost"). A Recruit
/// with both Augmentations (MaxHP 50 at no research) hit once, 47 HP, is trained in KindMenu\[Both\]: the Sparrow is the
/// same object, at its full MaxHP of 50, with no augmentation, and its own speeds and weapon.
#[test]
fn a_damaged_augmented_recruit_comes_out_a_sparrow_at_full_hp_its_augmentations_lost() {
    let mut t = base().start();
    t.without_auto_targeting();
    t.debug_at(1, crystals(P1, BANK));
    let recruit = id(next_id(&t));
    t.spawn_unit_at(1, RECRUIT, P1, Vec2::new(1150.0, 704.0));
    t.spawn_unit_at(1, RECRUIT, P2, Vec2::new(1150.0, 740.0));
    let hp = hurt_and_inside(&mut t, recruit, id(recruit.0 + 1), RecruitKind::Both);
    assert_eq!(hp, 47.0, "50, less a blow of 3");
    assert_eq!(t.sim().pool_counts(A1), [0, 0, 0, 1]);
    let start = t.sim().tick() + 1;
    t.at(start, P1, train(&[A1], RecruitKind::Both));
    let (_, sparrow) = until_produced(&mut t, FRAMES + 2);
    assert_eq!(sparrow, recruit, "the same object");
    let research = t.sim().research();
    let o = t.sim().object(sparrow).expect("out");
    assert_eq!(o.kind, SPARROW);
    assert_eq!(o.augmentation, Augmentation::NONE, "lost: it isn't infantry");
    assert_eq!((o.hp, o.max_hp_with(research)), (Some(50.0), Some(50.0)), "full HP");
    assert_eq!(o.max_speed(research), MAX_SPEED);
    let weapon = sc_sim::combat::weapon(o, research).expect("its machine gun");
    assert_eq!(weapon.damage, 3.0);
}

/// Design change 0332 ("first in, first trained"; `factions/cults/structures/armory.md > Training`: "an order reserves
/// the matching unit(s) that **entered the building first**"), and the queue: three Sparrows queued on four Recruits pay
/// 75 Space Crystals each and reserve the three inside the longest, in order; they take off one after the other, 320
/// ticks apart, the first Recruit in first. X takes the last entry back before it starts: its 75 Space Crystals, and
/// its Recruit to the pool.
#[test]
fn sparrows_are_trained_of_the_recruits_inside_the_longest_first() {
    let (mut t, order) = filled(4);
    let start = t.sim().tick() + 1;
    for _ in 0..3 {
        t.at(start, P1, train(&[A1], RecruitKind::Plain));
    }
    run_checked(&mut t, start);
    assert_eq!(bank(&t, P1), BANK - 3 * PRICE);
    assert_eq!(reserved(&t, A1), [vec![order[0]], vec![order[1]], vec![order[2]]]);
    assert_eq!(t.sim().pool_counts(A1), [1, 0, 0, 0]);
    t.at(start + 1, P1, cancel(&[A1]));
    run_checked(&mut t, start + 1);
    assert_eq!(bank(&t, P1), BANK - 2 * PRICE, "refunded in full");
    assert_eq!(reserved(&t, A1), [vec![order[0]], vec![order[1]]]);
    assert_eq!(t.sim().pool_counts(A1), [2, 0, 0, 0], "back in the pool");
    let (first, a) = until_produced(&mut t, FRAMES + 2);
    let (second, b) = until_produced(&mut t, FRAMES + 2);
    assert_eq!((a, b), (order[0], order[1]));
    assert_eq!(second - first, FRAMES, "one after the other");
    for sparrow in [a, b] {
        assert_eq!(t.sim().object(sparrow).map(|o| o.kind), Some(SPARROW));
    }
    assert_eq!(inside(&t, A1), [order[2], order[3]]);
}

/// The Airstrip's pool on the ground: Eject All (`factions/cults/structures/README.md > Choosing Recruits, Release and
/// Eject All`: "every unreserved unit leaves, one at a time through the exit") lets its Recruits out of the doorway in
/// the middle of side C, the runway's far end (the def's Placeholder), as Recruits, each walking out west of it.
#[test]
fn eject_all_lets_the_recruits_out_of_side_c_on_the_ground() {
    let (mut t, order) = filled(3);
    let start = t.sim().tick() + 1;
    t.at(start, P1, eject_all(&[A1]));
    let mut seen = 0;
    while exits(&t).len() < 3 {
        assert!(t.sim().tick() < start + 200, "{:?}", exits(&t));
        step_checked(&mut t);
        for &(_, unit) in &exits(&t)[seen..] {
            let o = t.sim().object(unit).expect("out");
            assert_eq!(o.kind, RECRUIT);
            assert!(
                near(o.pos.y, C_END.y, 0.5) && near(o.pos.x, C_END.x, 16.0),
                "in side C's doorway: {}",
                o.pos
            );
            assert!(near(heading(&t, unit).x, -1.0, 1e-6), "facing out of C");
        }
        seen = exits(&t).len();
    }
    assert_eq!(exits(&t).iter().map(|&(_, unit)| unit).collect::<Vec<_>>(), order);
    run_for(&mut t, 32);
    let half = RECRUIT.def().half_extents().expect("a unit").x;
    for unit in order {
        assert!(
            pos(&t, unit).x + half <= C_END.x + 0.01,
            "{unit:?} walked out west: {}",
            pos(&t, unit)
        );
    }
}

/// `factions/cults/units/recruit.md > Right-Click Resolution` (`systems/controls/commands.md > Enter`, "an own
/// structure that can hold units"): a right-click on the Airstrip is a Recruit's Enter; the Airstrip takes Recruits
/// only, so a Sparrow's right-click on it moves it there, and an Enter of it is refused: it never lands
/// (`factions/cults/units/sparrow.md`: "On completion it **takes off from the Airstrip** and **never lands** again").
#[test]
fn a_right_click_on_it_is_a_recruit_s_enter_and_a_sparrow_never_lands() {
    let (mut t, order) = filled(1);
    let recruit = id(next_id(&t));
    t.spawn_unit_at(t.sim().tick() + 1, RECRUIT, P1, Vec2::new(1150.0, 704.0));
    let start = t.sim().tick() + 1;
    t.at(start, P1, train(&[A1], RecruitKind::Plain));
    let (_, sparrow) = until_produced(&mut t, FRAMES + 2);
    assert_eq!(sparrow, order[0]);
    let on_airstrip = |t: &TestSim, unit| t.sim().default_order(unit, Target::Object(A1));
    assert_eq!(on_airstrip(&t, recruit), Some(OrderKind::Enter(A1)));
    assert_eq!(on_airstrip(&t, sparrow), Some(OrderKind::Move(Target::Object(A1))));
    let refused = t.sim().validate(&sc_sim::PlayerCommand {
        player: P1,
        command: enter(&[sparrow], A1),
    });
    assert!(refused.is_err(), "{refused:?}");
    // Sent over it, it circles it, in the air, for good.
    t.order_at(
        t.sim().tick() + 1,
        P1,
        &[sparrow],
        OrderKind::Move(Target::Object(A1)),
        false,
    );
    for _ in 0..16 * 30 {
        step_checked(&mut t);
        assert!(t.sim().held_unit(sparrow).is_none(), "never inside");
        assert!(speed(&t, sparrow) >= IDLE_SPEED - 1e-3, "flying");
    }
    assert!(pos(&t, sparrow).distance(t.sim().object(A1).unwrap().pos) < 2.0 * IDLE_RADIUS + 64.0);
}
