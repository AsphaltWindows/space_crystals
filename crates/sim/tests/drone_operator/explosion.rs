//! The drone's explosion (`factions/gdo/units/drone_operator.md > Launching and piloting`): "**Detonation:** the drone
//! behaves like a combat unit with a **melee-range attack that it can use only once** — the attack is its explosion.
//! It takes normal Attack orders and attacks like any melee unit; using its attack destroys it"; "**Explosion targets
//! ground units only** (TargetDomain: Ground) — the drone cannot attack air units"; "The drone **can be shot down** —
//! but only by attacks that can hit air (Air or Universal TargetDomain). It is an Air-domain unit"; "Explosion: **AoE**
//! (user, 2026-11-27), Damage **150** total, AoERadius **48 su**, melee range".

use super::*;

/// The Peacekeepers of [`a_drone_explodes_on_a_peacekeeper_group`], by id (the operator is 1, the drone 7): player 2's
/// target (2) 4 grid units east of the operator, two more 30 su north and south of it (3, 4) and one 80 su beyond it
/// (5), and player 1's own (6) 30 su behind the target, every one facing east.
const GROUP: [(u32, f32, f32); 5] = [
    (2, 256.0, 0.0),
    (3, 256.0, -30.0),
    (4, 256.0, 30.0),
    (5, 336.0, 0.0),
    (6, 226.0, 0.0),
];

/// The drone ordered to Attack player 2's Peacekeeper 2 flies to it, and explodes: one blast of 48 su round itself
/// (`SimEvent::Blast`), whose 150 the area rule shares among the ground units in the circle, whoever owns them
/// (`systems/combat.md > AoEDamage`, friendly fire): each Peacekeeper takes [`explosion_share`] (one beyond the circle
/// nothing), and neither the drone, in the air, nor its operator, far off, takes any. The explosion spends the drone,
/// which leaves the game in its tick (`SimEvent::Died`, no fall), and its operator is free: it walks, and F buys
/// another.
#[test]
fn a_drone_explodes_on_a_peacekeeper_group() {
    let mut t = field(|mut builder| {
        for (n, dx, dy) in GROUP {
            let owner = if n == 6 { P1 } else { P2 };
            builder = builder.spawn(PK, Some(owner), SpawnAt::Point(from_home(dx, dy)));
        }
        builder
    });
    let drone = 7;
    next(&mut t, P1, launch(&[1]));
    next(&mut t, P1, order(&[drone], OrderKind::Attack(id(2))));
    while died(&t, drone).is_none() {
        assert!(t.sim().tick() < 200, "no explosion\n{}", t.ascii(None));
        t.step();
    }
    let gone = died(&t, drone).expect("spent");
    let [(tick, centre, radius, attacker)] = blasts(&t)[..] else {
        panic!("one blast: {:?}", blasts(&t));
    };
    assert_eq!((tick, radius, attacker), (gone, RADIUS, Some(id(drone))));
    assert!(t.sim().object(id(drone)).is_none(), "the explosion spent it");
    assert_eq!(falls(&t), [], "it didn't fall");
    let target = object(t.sim(), 2).pos;
    let gap = (centre - target).abs() - Vec2::splat(24.0);
    assert!(
        gap.x <= 8.0 + 1e-3 && gap.y <= 8.0 + 1e-3,
        "in melee reach of its target: {centre}"
    );

    for (n, _, _) in GROUP {
        let taken: f32 = hits_on(&t, n)
            .iter()
            .map(|&(tick, attacker, damage)| {
                assert_eq!((tick, attacker), (gone, id(drone)), "#{n}");
                damage
            })
            .sum();
        let expected = explosion_share(centre, object(t.sim(), n).pos);
        assert!((taken - expected).abs() < 1e-4, "#{n}: {taken}, by the rule {expected}");
        assert_eq!(object(t.sim(), n).hp, Some(50.0 - taken), "#{n}");
    }
    assert!(
        hits_on(&t, 2)[0].2 > 8.0,
        "the target, inside the circle: {:?}",
        hits_on(&t, 2)
    );
    assert!(!hits_on(&t, 6).is_empty(), "friendly fire on player 1's own");
    assert_eq!(hits_on(&t, 5), [], "beyond the circle");
    assert_eq!(hits_on(&t, 1), [], "the operator, far off");

    assert_eq!(object(t.sim(), 1).drone, None, "the operator is free");
    assert_eq!(gate(t.sim(), P1, buy(&[1])), Ok(()));
    next(&mut t, P1, move_to(&[1], from_home(-200.0, 0.0)));
    t.run(20);
    assert!(t.pos(1).x < HOME.x - 40.0, "it walks: {}", t.pos(1));
}

/// "The drone cannot attack air units": ordered to Attack player 2's Supply Chopper (2), an air unit, hovering beside
/// it, the drone does nothing; and the chopper stays untouched.
#[test]
fn the_drone_attacks_no_air_unit() {
    let mut t = field(|builder| {
        builder.spawn(
            ObjectKind::SupplyChopper,
            Some(P2),
            SpawnAt::Point(from_home(150.0, 0.0)),
        )
    });
    let drone = 3;
    next(&mut t, P1, launch(&[1]));
    next(&mut t, P1, order(&[drone], OrderKind::Attack(id(2))));
    t.run(100);
    assert_eq!(blasts(&t), [], "no explosion");
    assert_eq!(hits_on(&t, 2), []);
    assert_eq!(object(t.sim(), 2).hp, Some(150.0));
    assert_eq!(object(t.sim(), drone).hp, Some(30.0), "still out");
}

/// "The drone **can be shot down** — but only by attacks that can hit air": player 2's Peacekeeper (2), whose rifle
/// hits the ground only, ordered to Attack the hovering drone, never hits it; three Rocketmen (3 to 5), Universal, shoot
/// it down. It dies of its wounds (`SimEvent::Died`, no fall and no explosion), and its operator is free.
#[test]
fn rocketmen_shoot_the_drone_down_and_peacekeepers_cant() {
    let mut t = field(|mut builder| {
        builder = builder.spawn(PK, Some(P2), SpawnAt::Point(from_home(0.0, 150.0)));
        for dy in [-60.0, 0.0, 60.0] {
            builder = builder.spawn(ObjectKind::Rocketman, Some(P2), SpawnAt::Point(from_home(320.0, dy)));
        }
        builder
    });
    let drone = 6;
    next(&mut t, P1, launch(&[1]));
    next(&mut t, P2, order(&[2], OrderKind::Attack(id(drone))));
    t.run(80);
    assert_eq!(hits_on(&t, drone), [], "the Peacekeeper's rifle doesn't reach the air");
    assert_eq!(object(t.sim(), drone).hp, Some(30.0));

    next(&mut t, P2, order(&[3, 4, 5], OrderKind::Attack(id(drone))));
    while died(&t, drone).is_none() {
        assert!(t.sim().tick() < 2 + BATTERY, "not shot down\n{}", t.ascii(None));
        t.step();
    }
    let shooters: Vec<ObjectId> = hits_on(&t, drone).iter().map(|&(_, attacker, _)| attacker).collect();
    assert!(
        shooters.iter().all(|s| [id(3), id(4), id(5)].contains(s)),
        "{shooters:?}"
    );
    assert_eq!(
        (falls(&t), blasts(&t).iter().filter(|b| b.3 == Some(id(drone))).count()),
        (vec![], 0)
    );
    assert_eq!(object(t.sim(), 1).drone, None, "the operator is free");
    assert_eq!(hits_on(&t, 1), [], "the bursts in the air");
}
