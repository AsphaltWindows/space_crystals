//! The amounts as the accessors read them (`factions/cults/augmentation.md > Increments`: Constitution "+10 MaxHP;
//! +1 melee Damage (unarmed units only)", Metabolism "+0.5 space units/frame MaxSpeed" and "for unarmed units also −1
//! frame of FiringDuration"), and phases 5d and 5e moving and striking by them.

use sc_sim::combat;
use sc_sim::{Increments, Level, Levels, Research, SimEvent, Target};

use super::*;

const RECRUIT: ObjectId = ObjectId(1);
const PLAIN: ObjectId = ObjectId(2);
const MILITANT: ObjectId = ObjectId(3);
const GUNNER: ObjectId = ObjectId(4);
const DISCIPLE: ObjectId = ObjectId(5);

/// Player 1's two Recruits, a Militant, a Gunner and a Disciple, ids 1 to 5, in a row.
fn infantry() -> TestSim {
    let kinds = [
        ObjectKind::Recruit,
        ObjectKind::Recruit,
        ObjectKind::Militant,
        ObjectKind::Gunner,
        ObjectKind::Disciple,
    ];
    let row = (kinds.iter().enumerate()).fold(cults(20, 8), |builder, (i, &kind)| {
        builder.spawn(kind, Some(P1), point(100.0 + 64.0 * i as f32, 200.0))
    });
    row.start()
}

fn levels(constitution: Level, metabolism: Level) -> Research<'static> {
    Research::at(Levels {
        constitution,
        metabolism,
    })
}

/// A Recruit's MaxHP, Damage, FiringDuration and MaxSpeed at `research`.
fn recruit_stats(o: &Object, research: Research) -> (Option<f32>, f32, u32, f32) {
    let weapon = combat::weapon(o, research).expect("a Recruit strikes");
    (
        o.max_hp_with(research),
        weapon.damage,
        weapon.firing_ticks,
        o.max_speed(research),
    )
}

/// "Militant (MaxHP 40) with Enhanced Constitution at no research: 60; at Level 2: 100": +2 Constitution increments,
/// +6 at Level 2 (`> Augmentation Effects`: "+6 Constitution / −3 Metabolism"). A full unit stays full. Its kind's
/// MaxHP, before Augmentations, is still 40.
#[test]
fn a_militant_with_enhanced_constitution_has_60_max_hp_and_100_at_level_2() {
    let mut t = infantry();
    augment(&mut t, MILITANT, RecruitKind::Constitution);
    assert_eq!(hp(&t, MILITANT), (60.0, 60.0));
    let o = object(&t, MILITANT);
    let l2 = levels(Level::L2, Level::NoResearch);
    assert_eq!(
        (o.increments(l2), o.max_hp_with(l2)),
        (Increments::new(6, -3), Some(100.0))
    );
    assert_eq!((o.max_hp(), o.max_hp_with(Research::NONE)), (Some(40.0), Some(60.0)));
}

/// "Both at no research: +1 Constitution / +1 Metabolism": a Recruit of MaxHP 50, Damage 4, FiringDuration 7 and
/// MaxSpeed 4.5 (a Plain one's 40, 3, 8 and 4). With Improved Metabolism at Level 2 instead ("+6 Metabolism / −3
/// Constitution"), MaxHP 10, Damage 0, FiringDuration 2 and MaxSpeed 7: "No floors on the results".
#[test]
fn a_recruit_s_amounts_follow_its_augmentations() {
    let mut t = infantry();
    augment(&mut t, RECRUIT, RecruitKind::Both);
    let research = t.sim().research();
    assert_eq!(object(&t, RECRUIT).increments(research), Increments::new(1, 1));
    assert_eq!(recruit_stats(object(&t, RECRUIT), research), (Some(50.0), 4.0, 7, 4.5));
    assert_eq!(recruit_stats(object(&t, PLAIN), research), (Some(40.0), 3.0, 8, 4.0));
    augment(&mut t, PLAIN, RecruitKind::Metabolism);
    let l2 = levels(Level::NoResearch, Level::L2);
    assert_eq!(recruit_stats(object(&t, PLAIN), l2), (Some(10.0), 0.0, 2, 7.0));
    assert_eq!(hp(&t, PLAIN), (30.0, 30.0), "at no research, −1 Constitution increment");
}

/// "Armed infantry (Militant, Gunner) receive only the MaxHP part of Constitution increments and only the
/// movement-speed part of Metabolism increments", and so does the Disciple, armed for Augmentation
/// (`factions/cults/units/disciple.md > AttackAttributes`): with both Augmentations at Level 2, +3/+3, MaxHP 70 and
/// MaxSpeed 5.5, their weapons their kinds'.
#[test]
fn armed_infantry_get_only_the_max_hp_and_the_speed() {
    let mut t = infantry();
    for unit in [MILITANT, GUNNER, DISCIPLE] {
        augment(&mut t, unit, RecruitKind::Both);
    }
    let l2 = levels(Level::L2, Level::L2);
    for unit in [MILITANT, GUNNER, DISCIPLE] {
        let o = object(&t, unit);
        let own = o.def().unit.as_ref().expect("a unit").attack;
        assert_eq!((o.max_hp_with(l2), o.max_speed(l2)), (Some(70.0), 5.5), "{:?}", o.kind);
        assert_eq!(combat::weapon(o, l2), own, "{:?}", o.kind);
    }
}

/// Phase 5d moves a unit at its top speed with its Augmentations': a Recruit with Improved Metabolism at no research
/// (+2 Metabolism increments) walks 5 space units a tick, a Plain one 4.
#[test]
fn a_recruit_with_improved_metabolism_walks_faster() {
    let mut t = infantry();
    augment(&mut t, RECRUIT, RecruitKind::Metabolism);
    for (unit, y) in [(RECRUIT, 300.0), (PLAIN, 400.0)] {
        let to = OrderKind::Move(Target::Point(Vec2::new(1000.0, y)));
        t.order_at(next(&t), P1, &[unit], to, false);
    }
    let mut fastest = [0.0f32; 2];
    for _ in 0..60 {
        t.step();
        for (most, unit) in fastest.iter_mut().zip([RECRUIT, PLAIN]) {
            let o = object(&t, unit);
            *most = most.max(o.pos.distance(o.prev_pos));
        }
    }
    assert!(
        (fastest[0] - 5.0).abs() < 1e-3 && (fastest[1] - 4.0).abs() < 1e-3,
        "{fastest:?}"
    );
}

/// The HP damage of each of `attacker`'s hits, and the ticks of its attacks.
fn blows(t: &TestSim, attacker: ObjectId) -> (Vec<f32>, Vec<Tick>) {
    let hits = (t.events().iter()).filter_map(|(_, e)| match *e {
        SimEvent::Hit {
            attacker: a, damage, ..
        } if a == attacker => Some(damage),
        _ => None,
    });
    let ticks = (t.events().iter()).filter_map(|&(tick, ref e)| match *e {
        SimEvent::Attacked { attacker: a, .. } if a == attacker => Some(tick),
        _ => None,
    });
    (hits.collect(), ticks.collect())
}

/// Phase 5e strikes with the weapon as the Augmentations make it: a Recruit with both at no research hits for 4 where
/// a Plain one, in the same place against the same kind of target, hits for 3, and strikes every 11 ticks rather
/// than every 12 (FiringDuration 7, not 8; `factions/cults/units/recruit.md > AttackAttributes`: Aim 2, Fire 8,
/// Cooldown 1, Reload 1).
#[test]
fn a_recruit_with_both_strikes_harder_and_faster() {
    let mut t = cults(20, 12)
        .spawn(ObjectKind::Recruit, Some(P1), point(200.0, 200.0))
        .spawn(ObjectKind::Recruit, Some(P2), point(240.0, 200.0))
        .spawn(ObjectKind::Recruit, Some(P1), point(200.0, 600.0))
        .spawn(ObjectKind::Recruit, Some(P2), point(240.0, 600.0))
        .start();
    t.without_auto_targeting();
    augment(&mut t, id(1), RecruitKind::Both);
    t.at(next(&t), P1, attack(&[id(1)], id(2)));
    t.at(next(&t), P1, attack(&[id(3)], id(4)));
    t.run(60);
    let ((both, both_at), (plain, plain_at)) = (blows(&t, id(1)), blows(&t, id(3)));
    assert!(both.len() >= 3 && plain.len() >= 3, "{both:?} {plain:?}");
    assert!(
        both.iter().all(|&d| (d * 3.0 - plain[0] * 4.0).abs() < 1e-3),
        "{both:?} {plain:?}"
    );
    assert!(plain.iter().all(|&d| d == plain[0]), "{plain:?}");
    assert!(both_at.windows(2).all(|w| w[1] - w[0] == 11), "{both_at:?}");
    assert!(plain_at.windows(2).all(|w| w[1] - w[0] == 12), "{plain_at:?}");
}
