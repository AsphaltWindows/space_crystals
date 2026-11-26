//! Health regeneration, the Shrine's Q (`factions/cults/structures/shrine.md > Abilities`: "Health regeneration (name
//! TBD) | Own Cults infantry | Grants health regeneration (+2 HP/s, placeholder) for the duration — Cults infantry have
//! no natural regeneration | 75 | radius 4 grid units | 15 s"; "The effect is a **buff applied to units at the moment
//! of casting**: qualifying units inside the area when the ability is cast receive the buff for its duration and keep
//! it even if they leave the area. Units that enter the area afterward do not receive it. Only the caster's own units
//! are affected"; `> Placeholder stats`: "**In the circle** (user, 2026-11-26; Q0118): a unit is affected when its
//! **centre** is inside. **Recasting** on units that already have the effect **restarts its duration**; effects of the
//! same ability **don't stack**").

use super::*;

/// The plan's acceptance: "Q heals each own Cults infantry unit inside the circle by 30 HP over 15 s", and "It doesn't
/// heal enemies, other factions' units, or units that arrive after the cast". Player 1's Recruit, Militant, Gunner and
/// Disciple, each at 8 HP of 40 in the circle, get the buff from the cast's tick for 240 ticks, and 0.125 HP a tick:
/// 38 HP 15 s on, and no more after. Player 2's Militant in the circle, player 1's Peacekeeper (a GDO type) and Seeker
/// (no infantry) in it, and player 1's Recruit that walks into it after the cast get nothing.
#[test]
fn q_heals_own_cults_infantry_in_the_circle_30_hp_over_15_s_and_no_one_else() {
    let (mut t, _) = manned(8);
    let next = |t: &TestSim| t.sim().tick() + 1;
    let recruit = spawn(&mut t, RECRUIT, P1, Vec2::new(900.0, 520.0));
    let militant = spawn(&mut t, MILITANT, P1, Vec2::new(900.0, 580.0));
    let gunner = spawn(&mut t, GUNNER, P1, Vec2::new(900.0, 640.0));
    let disciple = spawn(&mut t, DISCIPLE, P1, Vec2::new(900.0, 700.0));
    let enemy = spawn(&mut t, MILITANT, P2, Vec2::new(900.0, 760.0));
    let gdo = spawn(&mut t, ObjectKind::Peacekeeper, P1, Vec2::new(800.0, 830.0));
    let seeker = spawn(&mut t, ObjectKind::Seeker, P1, Vec2::new(760.0, 540.0));
    let late = spawn(&mut t, RECRUIT, P1, Vec2::new(900.0, 1000.0));
    let infantry = [recruit, militant, gunner, disciple];
    let mut wounds: Vec<(ObjectId, f32)> = infantry.iter().map(|&unit| (unit, 8.0)).collect();
    wounds.extend([(enemy, 32.0), (gdo, 44.0), (late, 8.0)]);
    wound(&mut t, &wounds);
    assert!(infantry.iter().all(|&unit| hp(&t, unit) == (8.0, 40.0)));
    let (enemy_hp, gdo_hp) = (hp(&t, enemy).0, hp(&t, gdo).0);
    assert!(enemy_hp < 40.0 && gdo_hp < 50.0, "{enemy_hp} {gdo_hp}");
    charge(&mut t, 75);
    let c = next(&t);
    t.at(c, P1, cast(&[S1], Q, AT));
    t.at(c + 1, P1, walk(&[late], Vec2::new(960.0, 640.0)));
    run_checked(&mut t, c);
    assert_eq!(casts(&t), [(c, S1, Q, AT)]);
    for unit in infantry {
        assert_eq!(buffs(&t, unit), buff(Q, c + Q_TICKS), "{unit:?}");
        assert_eq!(hp(&t, unit).0, 8.125, "{unit:?}: healed in the cast's tick");
    }
    for unit in [enemy, gdo, seeker, late] {
        assert_eq!(buffs(&t, unit), [], "{unit:?}");
    }
    run_checked(&mut t, c + Q_TICKS - 1);
    for unit in infantry {
        assert_eq!(hp(&t, unit), (38.0, 40.0), "{unit:?}: 30 HP in 240 ticks");
        assert_eq!(buffs(&t, unit), [], "{unit:?}: over");
    }
    let late_pos = t.sim().object(late).unwrap().pos;
    assert!(
        late_pos.distance(AT) < 100.0,
        "the late Recruit is in the circle, at {late_pos}"
    );
    assert_eq!(
        (hp(&t, late).0, buffs(&t, late)),
        (8.0, vec![]),
        "arrived after the cast"
    );
    assert_eq!((hp(&t, enemy).0, hp(&t, gdo).0), (enemy_hp, gdo_hp));
    run_for(&mut t, 60);
    assert!(
        infantry.iter().all(|&unit| hp(&t, unit) == (38.0, 40.0)),
        "no more after"
    );
}

/// The plan's acceptance: "capped at MaxHP" (with the Augmentations: the brief), and "A unit that leaves the circle
/// keeps the buff". An Enhanced Constitution Recruit, whose MaxHP is above 40, two hits below it heals to its MaxHP,
/// not to 40, and no further. A Recruit that walks out of the circle at once heals its 30 HP all the same, and so does
/// one that goes into the Shrine and heals there (**Placeholder (not in the design)**: a buff works wherever its unit
/// is).
#[test]
fn the_heal_stops_at_the_max_hp_with_augmentations_and_leaving_the_circle_keeps_it() {
    let (mut t, _) = manned(7);
    let next = |t: &TestSim| t.sim().tick() + 1;
    let augmented = spawn(&mut t, RECRUIT, P1, Vec2::new(900.0, 580.0));
    let now = next(&t);
    set_augmentation(&mut t, now, augmented, RecruitKind::Constitution);
    run_for(&mut t, 1);
    let leaver = spawn(&mut t, RECRUIT, P1, Vec2::new(900.0, 640.0));
    let goer = spawn(&mut t, RECRUIT, P1, Vec2::new(900.0, 700.0));
    let max = hp(&t, augmented).1;
    assert!(max > 40.0, "Enhanced Constitution raises it: {max}");
    wound(&mut t, &[(augmented, max - 8.0), (leaver, 8.0), (goer, 8.0)]);
    assert_eq!(hp(&t, augmented).0, max - 8.0);
    charge(&mut t, 75);
    let c = next(&t);
    t.at(c, P1, cast(&[S1], Q, AT));
    t.at(c + 1, P1, walk(&[leaver], Vec2::new(900.0, 1150.0)));
    t.at(c + 1, P1, enter(&[goer], S1));
    run_checked(&mut t, c + 64);
    assert_eq!(hp(&t, augmented), (max, max), "8 HP in 64 ticks, to its MaxHP");
    run_checked(&mut t, c + Q_TICKS - 1);
    assert_eq!(hp(&t, augmented), (max, max), "no further");
    let pos = t.sim().object(leaver).expect("on the map").pos;
    assert!(pos.distance(AT) > 256.0, "out of the circle, at {pos}");
    assert_eq!(hp(&t, leaver).0, 38.0, "its 30 HP all the same");
    assert!(inside(&t, S1).contains(&goer), "inside the Shrine");
    assert_eq!(hp(&t, goer).0, 38.0, "healed inside too");
    assert_eq!((buffs(&t, leaver), buffs(&t, goer)), (vec![], vec![]));
}

/// "**Recasting** on units that already have the effect **restarts its duration**; effects of the same ability **don't
/// stack**": a second Q 40 ticks after the first, on a Recruit at 4 HP, restarts its buff, which ends 240 ticks after
/// the recast; it heals 0.125 a tick all along, never twice that, 35 HP in all.
#[test]
fn a_recast_restarts_the_buff_and_the_rates_never_add() {
    let (mut t, _) = manned(8);
    let recruit = spawn(&mut t, RECRUIT, P1, Vec2::new(900.0, 640.0));
    wound(&mut t, &[(recruit, 4.0)]);
    assert_eq!(hp(&t, recruit).0, 4.0);
    charge(&mut t, 150);
    let c = t.sim().tick() + 1;
    t.at(c, P1, cast(&[S1], Q, AT));
    t.at(c + 40, P1, cast(&[S1], Q, AT));
    run_checked(&mut t, c);
    assert_eq!(buffs(&t, recruit), buff(Q, c + Q_TICKS));
    let mut gains = Vec::new();
    while t.sim().tick() < c + 40 + Q_TICKS + 10 {
        let before = hp(&t, recruit).0;
        step_checked(&mut t);
        gains.push(hp(&t, recruit).0 - before);
        if t.sim().tick() == c + 40 {
            assert_eq!(buffs(&t, recruit), buff(Q, c + 40 + Q_TICKS), "restarted, one buff");
        }
    }
    let (healing, after) = gains.split_at(40 + Q_TICKS as usize - 1);
    assert!(healing.iter().all(|&gain| gain == 0.125), "{healing:?}");
    assert!(after.iter().all(|&gain| gain == 0.0), "{after:?}");
    assert_eq!(hp(&t, recruit).0, 39.0, "35 HP in 280 ticks");
}
