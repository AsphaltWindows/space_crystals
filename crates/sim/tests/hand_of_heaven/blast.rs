//! The missile's blast (`factions/cults/units/hand_of_heaven.md > Placeholder stats`: "Damage **600** total over the
//! circle (AoEDamage; a 3x3 building wholly inside takes it all, a lone 24x24 infantry about 12, so it is a siege
//! weapon, poor against spread or moving units); AoERadius **96 su (1.5 GU)**"), by `systems/combat.md > AoEDamage`'s
//! rules: "A unit's damage share = Attack Damage × (unit overlap area / AoE area). Effective armor = FullArmor × (unit
//! overlap area / unit total area)", and "**Friendly fire** ... area damage hits **every** object in the circle whose
//! domain matches, **whoever owns it**".

use std::f32::consts::PI;

use super::verification::shooting_range;
use super::*;

/// The circle's area, π × 96² su².
const CIRCLE: f32 = PI * RADIUS * RADIUS;

/// The Chop Shop's FullArmor (`factions/cults/structures/chop_shop.md > Placeholder stats`: "PointArmor 1, FullArmor
/// 9"), and the share of its 192 × 192 su footprint the circle of 96 su covers, a quarter of π.
const CHOP_SHOP_FULL_ARMOR: f32 = 9.0;
const BUILDING_SHARE: f32 = CIRCLE / (192.0 * 192.0);

/// A lone infantry's share of the blast: its 24 × 24 su Silhouette wholly inside the circle.
const INFANTRY: f32 = DAMAGE * 24.0 * 24.0 / CIRCLE;

/// "a 3x3 building wholly inside takes it all": the circle of 96 su, on the Chop Shop's centre, lies wholly inside its
/// 192 × 192 su footprint, so its share is all 600; its effective FullArmor is 9 × π/4 ≈ 7.07, the part of the
/// footprint the circle covers, so it takes 600 − 7.07 ≈ 592.93 (more than its 450 HP: it falls to one missile).
#[test]
fn a_3x3_building_takes_all_600_less_its_armour() {
    let mut t = shooting_range();
    order(&mut t, 1, P1, &[1], OrderKind::Attack(id(2)));
    until(&mut t, 200, "the landing", |t| !blasts(t).is_empty());
    let [(_, _, taken)] = hits_on(&t, 2)[..] else {
        panic!("one hit: {:?}", hits_on(&t, 2));
    };
    let expected = DAMAGE - CHOP_SHOP_FULL_ARMOR * BUILDING_SHARE;
    assert!((taken - expected).abs() < 1e-2, "{taken} for {expected}");
    assert!((expected - 592.93).abs() < 1e-2);
    assert!(t.sim().object(id(2)).is_none(), "450 HP: it fell");
}

/// "a lone 24x24 infantry about 12", and friendly fire. Player 2's Militant (2) stands 12 tiles east of the Hand of
/// Heaven, and player 1's own Recruit (3), its spotter, 40 su south of it, both wholly inside the circle as the
/// missile lands on the Militant's centre: each takes 600 × 576 / (π × 96²) ≈ 11.94, its FullArmor 0 taking nothing
/// off. The Hand of Heaven, far off, takes nothing.
#[test]
fn a_lone_infantry_takes_about_12_and_own_units_in_the_circle_are_hit_too() {
    let target = centre(18, 12);
    let mut t = cults(40, 24)
        .spawn(HAND, Some(P1), point(centre(6, 12)))
        .spawn(ObjectKind::Militant, Some(P2), point(target))
        .spawn(ObjectKind::Recruit, Some(P1), point(target + Vec2::new(0.0, 40.0)))
        .spawn(ObjectKind::Storage, Some(P2), tile(36, 1))
        .start();
    t.without_auto_targeting();
    order(&mut t, 1, P1, &[1], OrderKind::Attack(id(2)));
    until(&mut t, 200, "the landing", |t| !blasts(t).is_empty());
    assert!((INFANTRY - 11.94).abs() < 5e-3, "{INFANTRY}");
    for n in [2, 3] {
        let [(_, attacker, taken)] = hits_on(&t, n)[..] else {
            panic!("#{n} hit once: {:?}", hits_on(&t, n));
        };
        assert_eq!(attacker, id(1));
        assert!((taken - INFANTRY).abs() < 1e-3, "#{n}: {taken}");
    }
    assert_eq!(hits_on(&t, 1), [], "not the Hand of Heaven");
}

/// "the missile flies to where the target was at the end of Aiming, so targets can dodge": player 2's Militant walks
/// off north as the missile leaves, and is out of the circle when it lands, 96 ticks later, where the Militant stood;
/// it takes nothing.
#[test]
fn a_target_that_walks_off_in_the_flight_dodges_the_missile() {
    let target = centre(18, 12);
    let mut t = cults(40, 24)
        .spawn(HAND, Some(P1), point(centre(6, 12)))
        .spawn(ObjectKind::Militant, Some(P2), point(target))
        .spawn(ObjectKind::Recruit, Some(P1), point(centre(22, 12)))
        .spawn(ObjectKind::Storage, Some(P2), tile(36, 1))
        .start();
    t.without_auto_targeting();
    order(&mut t, 1, P1, &[1], OrderKind::Attack(id(2)));
    until(&mut t, 80, "the launch", |t| !launches(t).is_empty());
    let launch = launches(&t)[0];
    assert_eq!(launch.to, target);
    order(
        &mut t,
        launch.tick + 1,
        P2,
        &[2],
        OrderKind::Move(sc_sim::Target::Point(centre(18, 2))),
    );
    until(&mut t, 100, "the landing", |t| !blasts(t).is_empty());
    assert_eq!(blasts(&t), [(launch.lands_at, Some(id(1)), target)]);
    assert!(
        t.pos(2).distance(target) > RADIUS + 12.0,
        "out of the circle: {}",
        t.pos(2)
    );
    assert_eq!(hits_on(&t, 2), [], "dodged");
}
