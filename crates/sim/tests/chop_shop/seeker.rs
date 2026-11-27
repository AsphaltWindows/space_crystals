//! The Seeker trained (`factions/cults/units/seeker.md`): of two Recruits, the first of which it is, at full HP and
//! with no augmentation (design change 0330), and its Unit Control, a point for each of its Recruits' Halls
//! (`factions/cults/structures/recruitment_hall.md > Unit Control Tracking`: "A trained unit's Unit Control cost equals
//! the number of Recruits consumed to produce it, with each point of cost attributed to the originating hall of the
//! respective Recruit").

use sc_sim::recruitment::Lineages;

use super::*;

/// Player 1's Recruit `unit`, hurt by one blow of player 2's Recruit `attacker` (3 damage,
/// `factions/cults/units/recruit.md`), which then leaves the game: from tick `tick` on, returned once the blow landed.
fn hurt(t: &mut TestSim, tick: Tick, unit: ObjectId, attacker: ObjectId) {
    t.at(
        tick,
        P2,
        Command::Order {
            units: vec![attacker],
            kind: OrderKind::Attack(unit),
            queue: false,
        },
    );
    run_checked(t, tick);
    let max = t.sim().object(unit).and_then(|o| o.max_hp_with(t.sim().research()));
    while t.sim().object(unit).and_then(|o| o.hp) == max {
        assert!(t.sim().tick() < tick + 60, "a blow lands");
        step_checked(t);
    }
    let removed = t.sim().tick() + 1;
    remove_at(t, removed, attacker);
    run_checked(t, removed);
}

/// Design change 0330 (`factions/cults/structures/chop_shop.md`: "**Full HP** (user, 2026-11-26; Q0121): a vehicle comes
/// out at **full HP**, whatever damage its Recruits had"; `factions/cults/augmentation.md > Persistence`: "if augmented
/// Recruits are consumed to produce a vehicle, their Augmentations are lost"). Two Recruits with both Augmentations
/// (MaxHP 50 at no research), each hit once, 47 HP, are trained in KindMenu\[Both\]: the Seeker is the first of them, at
/// its full MaxHP of 120, with no augmentation, and its own ForwardMaxSpeed and machine gun.
#[test]
fn two_damaged_augmented_recruits_make_a_seeker_at_full_hp_with_no_augmentation() {
    let mut t = base().start();
    t.without_auto_targeting();
    t.debug_at(1, crystals(P1, BANK));
    let first = next_id(&t);
    let recruits = [id(first), id(first + 1)];
    let attackers = [id(first + 2), id(first + 3)];
    t.spawn_unit_at(1, RECRUIT, P1, Vec2::new(1150.0, 704.0));
    t.spawn_unit_at(1, RECRUIT, P1, Vec2::new(1150.0, 800.0));
    t.spawn_unit_at(1, RECRUIT, P2, Vec2::new(1150.0, 740.0));
    t.spawn_unit_at(1, RECRUIT, P2, Vec2::new(1150.0, 836.0));
    for recruit in recruits {
        augment(&mut t, 2, recruit, RecruitKind::Both);
    }
    hurt(&mut t, 3, recruits[0], attackers[0]);
    let next = t.sim().tick() + 1;
    hurt(&mut t, next, recruits[1], attackers[1]);
    let tick = t.sim().tick() + 1;
    t.at(tick, P1, enter(&recruits, CS));
    run_checked(&mut t, tick);
    fill(&mut t, CS, 2);
    for recruit in recruits {
        let held = t.sim().held_unit(recruit).expect("inside");
        assert_eq!(held.object.hp, Some(47.0), "50, less a blow of 3");
        assert_eq!(held.object.augmentation, Augmentation::of(RecruitKind::Both));
    }
    assert_eq!(t.sim().pool_counts(CS), [0, 0, 0, 2]);
    let start = t.sim().tick() + 1;
    t.at(start, P1, train(&[CS], RecruitKind::Both));
    let (_, seeker) = until_produced(&mut t, CS, FRAMES + 2);
    assert!(recruits.contains(&seeker), "one of the two");
    let research = t.sim().research();
    let o = t.sim().object(seeker).expect("out");
    assert_eq!(o.kind, SEEKER);
    assert_eq!(o.augmentation, Augmentation::NONE, "lost: it isn't infantry");
    assert_eq!(
        (o.hp, o.max_hp_with(research)),
        (Some(SEEKER_HP), Some(SEEKER_HP)),
        "full HP"
    );
    assert_eq!(o.max_speed(research), TOP);
    let weapon = sc_sim::combat::weapon(o, research).expect("its machine gun");
    assert_eq!(weapon.damage, 3.0);
    let other = recruits.iter().find(|&&r| r != seeker).copied().expect("the other");
    assert!(t.sim().object(other).is_none() && t.sim().held_unit(other).is_none());
}

/// Player 1's two Recruitment Halls ([`H1`] and [`H2`]), the Chop Shop between them and a Storage, which keeps player 1
/// in the game, with [`BANK`] Space Crystals: each Hall's first Recruit is ordered into the Chop Shop as it comes out.
/// Returns the two once inside.
fn two_halls() -> (TestSim, [ObjectId; 2]) {
    let mut t = cults(&flat(30, 22))
        .spawn(CHOP_SHOP, Some(P1), tile(12, 10))
        .spawn(ObjectKind::RecruitmentHall, Some(P1), tile(20, 2))
        .spawn(ObjectKind::RecruitmentHall, Some(P1), tile(20, 16))
        .spawn(ObjectKind::Storage, Some(P1), tile(2, 2))
        .start();
    t.without_auto_targeting();
    t.debug_at(1, crystals(P1, BANK));
    let mut sent = Vec::new();
    while sent.len() < 2 {
        assert!(t.sim().tick() < 400, "the Halls' first Recruits");
        step_checked(&mut t);
        for (_, hall, recruit) in produced(&t) {
            if [H1, H2].contains(&hall) && !sent.iter().any(|&(h, _)| h == hall) {
                sent.push((hall, recruit));
                let tick = t.sim().tick() + 1;
                t.at(tick, P1, enter(&[recruit], CS));
            }
        }
    }
    let recruits = [sent[0].1, sent[1].1];
    fill(&mut t, CS, 2);
    for (hall, recruit) in sent {
        let held = t.sim().held_unit(recruit).expect("inside");
        assert_eq!(held.object.lineage, [hall], "its Hall's");
    }
    (t, recruits)
}

/// [`two_halls`]' Halls.
const H1: ObjectId = ObjectId(2);
const H2: ObjectId = ObjectId(3);

/// The Unit Control the units of `t` that are `units` use, by their lineages ([`Lineages`]): each Hall's points and
/// player 1's orphaned ones, with every Hall of the game standing by them, and none of the rest.
fn usage_of(t: &TestSim, units: &[ObjectId]) -> (u32, u32, u32) {
    let halls = [H1, H2].into_iter().filter_map(|hall| t.sim().object(hall));
    let alone = Lineages::of(halls.clone());
    let units = units.iter().filter_map(|&unit| t.sim().object(unit));
    let with = Lineages::of(halls.chain(units));
    let point = |hall| with.usage(hall) - alone.usage(hall);
    (point(H1), point(H2), with.orphaned(P1))
}

/// The plan's acceptance (slice M5-N6): "a Seeker counts 1 against each of its two Recruits' Halls. Losing one Hall
/// makes half of it an orphan." A Seeker trained of a Recruit of each of two Halls names them both (`Object::lineage`,
/// `Sim::train`: the second Recruit's entry appended), a point of its Unit Control cost of 2 at each; the player's
/// Unit Control is the same in the tick it comes out as in the tick before, its two Recruits' points now its own. Once
/// one Hall is gone, its point is an orphan, and the other Hall's stays.
#[test]
fn a_seeker_counts_one_against_each_of_its_recruits_halls_and_half_of_it_is_an_orphan_once_one_goes() {
    let (mut t, recruits) = two_halls();
    let start = t.sim().tick() + 1;
    t.at(start, P1, train(&[CS], RecruitKind::Plain));
    run_checked(&mut t, start);
    let reserved = reserved(&t, CS);
    assert_eq!(reserved.len(), 1);
    let mut both = reserved[0].clone();
    both.sort_unstable();
    let mut expected = recruits.to_vec();
    expected.sort_unstable();
    assert_eq!(both, expected);
    run_checked(&mut t, start + FRAMES - 2);
    let used = |t: &TestSim| t.sim().army_cap(P1).expect("in the game").used;
    let halls = |t: &TestSim| [H1, H2].map(|hall| t.sim().hall_state(hall).expect("the Hall").usage);
    let before = (used(&t), halls(&t));
    let made_before = produced(&t).len();
    step_checked(&mut t);
    let made: Vec<_> = produced(&t)[made_before..].to_vec();
    let [(_, CS, seeker)] = made[..] else {
        panic!("the Seeker alone in that tick: {made:?}");
    };
    let o = t.sim().object(seeker).expect("out");
    assert_eq!(o.kind, SEEKER);
    let mut lineage = o.lineage.clone();
    lineage.sort_unstable();
    assert_eq!(lineage, [H1, H2], "a Recruit of each");
    assert_eq!((used(&t), halls(&t)), before, "its two points are its Recruits'");
    assert_eq!(usage_of(&t, &[seeker]), (1, 1, 0), "one point at each Hall");
    let gone = t.sim().tick() + 1;
    remove_at(&mut t, gone, H2);
    run_checked(&mut t, gone);
    assert!(t.sim().hall_state(H2).is_none());
    assert_eq!(usage_of(&t, &[seeker]), (1, 0, 1), "half of it an orphan");
}
