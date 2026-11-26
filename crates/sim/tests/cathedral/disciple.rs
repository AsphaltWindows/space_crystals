//! The Disciple the Cathedral trains (`factions/cults/units/disciple.md`): it keeps its Recruit's damage and its
//! Augmentations (`factions/cults/structures/cathedral.md > Training`: "The trained unit inherits the Recruit's
//! AugmentationState"; design change 0332: infantry trained from a Recruit "keep the Recruit's damage"), of which it
//! gets only the MaxHP and speed parts ("AttackAttributes - None (placeholder: no weapon; for Augmentation it is
//! **armed**, i.e. it gets only the MaxHP and speed parts of increments"); and without a weapon it never attacks by
//! itself, a right-click on an enemy moves it, as for any unit without an attack, and it ignores an Attack order.

use sc_sim::{Level, Levels, Research, combat};

use super::*;

/// Player 1's Recruit `unit` with the augmentations of kind `kind`, hurt by one blow of player 2's Recruit `attacker`
/// (3 damage, `factions/cults/units/recruit.md`), which then leaves the game; then into the Cathedral. Its HP once
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
    t.at(removed, P1, enter(&[unit], C1));
    run_checked(t, removed);
    fill(t, C1, 1);
    t.sim().held_unit(unit).and_then(|held| held.object.hp).expect("hurt")
}

/// The plan's acceptance: "the trained Disciple keeps its Recruit's damage and augmentation", and "a Disciple gets only
/// its MaxHP and speed parts, through the Cathedral". A Recruit with both Augmentations (+1 Constitution, +1 Metabolism
/// at no research: MaxHP 50, Damage 4, FiringDuration 7, MaxSpeed 4.5, `factions/cults/augmentation.md > Increments`)
/// hit once, trained in KindMenu\[Both\]: the Disciple keeps the 3 HP it lost of the same MaxHP 50 (40, +10) and its
/// MaxSpeed 4.5 (4, +0.5), and has no weapon to take the damage and firing parts; at Level 2 of both lines (+3/+3) it
/// would have MaxHP 70 and MaxSpeed 5.5. Its augmentations stay its own.
#[test]
fn a_disciple_keeps_its_recruit_s_damage_and_gets_only_the_max_hp_and_speed_parts() {
    let mut t = base().start();
    t.without_auto_targeting();
    t.debug_at(1, crystals(P1, BANK));
    let recruit = id(next_id(&t));
    t.spawn_unit_at(1, RECRUIT, P1, Vec2::new(1150.0, 704.0));
    t.spawn_unit_at(1, RECRUIT, P2, Vec2::new(1150.0, 740.0));
    let hp = hurt_and_inside(&mut t, recruit, id(recruit.0 + 1), RecruitKind::Both);
    assert_eq!(hp, 47.0, "50, less a blow of 3");
    let recruit_weapon = (t.sim().held_unit(recruit)).and_then(|held| combat::weapon(&held.object, t.sim().research()));
    assert_eq!(recruit_weapon.map(|w| (w.damage, w.firing_ticks)), Some((4.0, 7)));
    assert_eq!(t.sim().pool_counts(C1), [0, 0, 0, 1]);
    let start = t.sim().tick() + 1;
    t.at(start, P1, train(&[C1], RecruitKind::Both));
    run_checked(&mut t, start + FRAMES - 1);
    assert_eq!(
        produced(&t).iter().map(|&(_, _, unit)| unit).collect::<Vec<_>>(),
        [recruit]
    );
    let research = t.sim().research();
    let o = t.sim().object(recruit).expect("out");
    assert_eq!(o.kind, DISCIPLE);
    assert_eq!(o.augmentation, Augmentation::of(RecruitKind::Both), "inherited");
    assert_eq!(
        (o.hp, o.max_hp_with(research)),
        (Some(47.0), Some(50.0)),
        "the damage kept"
    );
    assert_eq!(o.max_speed(research), 4.5);
    assert_eq!(
        combat::weapon(o, research),
        None,
        "no weapon, so no damage nor firing part"
    );
    let l2 = Research::at(Levels {
        constitution: Level::L2,
        metabolism: Level::L2,
    });
    assert_eq!((o.max_hp_with(l2), o.max_speed(l2)), (Some(70.0), 5.5));
}

/// The plan's acceptance: "A Disciple never auto-targets, and a right-click on an enemy moves it." With auto-targeting
/// on, a Disciple trained beside an enemy Disciple and an enemy Storage stays where it is and attacks nothing, having
/// no attack (`Object::attack`); its default order on either is a Move to it (`Sim::default_order`, the sim's one rule
/// of the right-click: an Attack wants a weapon), where a Militant's is an Attack. Given that Move, it walks up to the
/// enemy Disciple, and an AttackMove plays out as a move: it walks past the Storage to the point, attacking nothing.
#[test]
fn a_disciple_never_attacks_and_a_right_click_on_an_enemy_moves_it() {
    let mut t = base()
        .spawn(ObjectKind::Storage, Some(P2), tile(5, 10)) // 3: x 320..448, y 640..768
        .start();
    t.debug_at(1, crystals(P1, BANK));
    let recruit = id(next_id(&t));
    t.spawn_unit_at(1, RECRUIT, P1, Vec2::new(1150.0, 704.0));
    let (enemy, militant) = (id(recruit.0 + 1), id(recruit.0 + 2));
    t.spawn_unit_at(1, DISCIPLE, P2, Vec2::new(600.0, 600.0));
    t.spawn_unit_at(1, ObjectKind::Militant, P1, Vec2::new(1700.0, 1250.0));
    t.at(2, P1, enter(&[recruit], C1));
    t.at(2, P1, rally(&[C1], Vec2::new(560.0, 700.0)));
    run_checked(&mut t, 2);
    fill(&mut t, C1, 1);
    let start = t.sim().tick() + 1;
    t.at(start, P1, train(&[C1], RecruitKind::Plain));
    run_checked(&mut t, start + FRAMES - 1);
    let disciple = recruit;
    assert_eq!(t.sim().object(disciple).map(|o| o.kind), Some(DISCIPLE));
    run_for(&mut t, 120);
    let o = t.sim().object(disciple).expect("out");
    assert!(
        o.pos.distance(Vec2::new(560.0, 700.0)) < 40.0,
        "at its rally point: {}",
        o.pos
    );
    assert!(o.attack.is_none(), "no attack to carry out");
    let at = o.pos;
    run_for(&mut t, 64);
    let o = t.sim().object(disciple).unwrap();
    assert_eq!(o.pos, at, "it stays where it is");
    assert!(o.orders.as_ref().unwrap().is_idle());
    let attacked_by = |t: &TestSim, unit: ObjectId| {
        (t.events().iter()).any(|(_, e)| matches!(*e, SimEvent::Attacked { attacker, .. } if attacker == unit))
    };
    assert!(!attacked_by(&t, disciple), "it never attacks");
    let storage = id(3);
    for target in [enemy, storage] {
        assert_eq!(
            t.sim().default_order(disciple, Target::Object(target)),
            Some(OrderKind::Move(Target::Object(target))),
            "a right-click on #{}",
            target.0
        );
        assert_eq!(
            t.sim().default_order(militant, Target::Object(target)),
            Some(OrderKind::Attack(target)),
            "an armed unit's"
        );
    }
    let now = t.sim().tick() + 1;
    t.order_at(now, P1, &[disciple], OrderKind::Move(Target::Object(enemy)), false);
    run_for(&mut t, 48);
    let (o, them) = (t.sim().object(disciple).unwrap(), t.sim().object(enemy).unwrap());
    assert!(o.pos.distance(them.pos) < 64.0, "up to it: {} and {}", o.pos, them.pos);
    let beyond = Vec2::new(250.0, 1000.0);
    let now = t.sim().tick() + 1;
    t.order_at(now, P1, &[disciple], OrderKind::AttackMove(beyond), false);
    run_for(&mut t, 160);
    let o = t.sim().object(disciple).unwrap();
    assert!(o.pos.distance(beyond) < 40.0, "at the point: {}", o.pos);
    assert!(!attacked_by(&t, disciple), "it never attacks");
    assert!(t.sim().object(storage).is_some_and(|s| s.hp == s.max_hp()));
}

/// A Disciple takes no Attack order (`combat::takes_attack_orders`): it has nothing to attack with, so it ignores an
/// explicit Attack as a unit whose attack can't target the ground ignores an Attack Ground. Alone, the gate refuses the
/// Attack (`Reject::CannotAttack`), which greys the panel's Attack (plan decision T10, the client's
/// `command_ui::interface::conditions`); beside a Militant, the Militant takes it and attacks, and the Disciple keeps its
/// own order, a Move.
#[test]
fn a_disciple_ignores_an_attack_order_a_militant_beside_it_takes() {
    let mut t = cults(&flat(30, 22)).start();
    t.without_auto_targeting();
    let (disciple, militant, enemy) = (id(1), id(2), id(3));
    t.spawn_unit_at(1, DISCIPLE, P1, Vec2::new(300.0, 300.0));
    t.spawn_unit_at(1, ObjectKind::Militant, P1, Vec2::new(300.0, 360.0));
    t.spawn_unit_at(1, RECRUIT, P2, Vec2::new(500.0, 330.0));
    run_checked(&mut t, 1);
    let attack = |units: &[ObjectId]| Command::Order {
        units: units.to_vec(),
        kind: OrderKind::Attack(enemy),
        queue: false,
    };
    assert_eq!(validate(&t, attack(&[disciple])), Err(Reject::CannotAttack));
    assert_eq!(validate(&t, attack(&[disciple, militant])), Ok(()));
    let away = Vec2::new(300.0, 1000.0);
    t.order_at(2, P1, &[disciple], OrderKind::Move(Target::Point(away)), false);
    t.at(3, P1, attack(&[disciple, militant]));
    run_for(&mut t, 40);
    let current = |t: &TestSim, unit: ObjectId| {
        let orders = t.sim().object(unit).expect("on the map").orders.as_ref()?;
        orders.current.as_ref().map(|active| active.order.kind)
    };
    assert_eq!(current(&t, militant), Some(OrderKind::Attack(enemy)));
    assert_eq!(
        current(&t, disciple),
        Some(OrderKind::Move(Target::Point(away))),
        "its own order"
    );
    let hit =
        (t.events().iter()).any(|(_, e)| matches!(*e, SimEvent::Attacked { attacker, .. } if attacker == militant));
    assert!(hit, "the Militant attacks");
}
