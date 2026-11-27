//! The Phantom's laser (`factions/syndicate/units/phantom.md`: "Fires a **red laser** that hits **both air and
//! ground**"; "AttackType: FullyConnected (laser like other Syndicate lasers: instant, cannot miss)", "TargetDomain:
//! Universal (ground and air)", "Laser: Damage 12; Range 5 grid units; Aim 2, Firing 1, Cooldown 3, Reload 10 frames").

use super::*;

/// A flat 30 × 16 map, player 1 the Syndicate's and player 2 the GDO's, auto-targeting off: player 1's Phantom (1) at
/// (400, 500), player 2's Peacekeeper (2) 4 grid units east of it, and player 2's Sparrow (3), a Glider, circling a
/// point 4 grid units north of it.
fn duel() -> TestSim {
    let mut t = ScenarioBuilder::new(&flat(30, 16))
        .faction(P1, Faction::Syndicate)
        .faction(P2, Faction::Gdo)
        .spawn(PHANTOM, Some(P1), SpawnAt::Point(Vec2::new(400.0, 500.0)))
        .spawn(
            ObjectKind::Peacekeeper,
            Some(P2),
            SpawnAt::Point(Vec2::new(656.0, 500.0)),
        )
        .spawn(ObjectKind::Sparrow, Some(P2), SpawnAt::Point(Vec2::new(400.0, 244.0)))
        .start();
    t.without_auto_targeting();
    t
}

/// The plan's acceptance, "The Phantom hits a Sparrow and a Peacekeeper": ordered to attack player 2's Peacekeeper, a
/// ground unit, the Phantom lands its laser on it, 12 less the Peacekeeper's armour each, until it dies; ordered then
/// to attack the circling Sparrow, an air unit, it lands its laser on it too, until it dies. Every hit is the
/// Phantom's, instant (no projectile), none on anything else.
#[test]
fn the_phantom_hits_a_peacekeeper_and_a_sparrow() {
    let mut t = duel();
    let (phantom, peacekeeper, sparrow) = (id(1), id(2), id(3));
    assert_eq!(
        (object(&t, peacekeeper).kind, object(&t, sparrow).kind),
        (ObjectKind::Peacekeeper, ObjectKind::Sparrow)
    );
    for target in [peacekeeper, sparrow] {
        let right_click = t.sim().default_order(phantom, Target::Object(target));
        assert_eq!(right_click, Some(OrderKind::Attack(target)), "{target:?}: Universal");
    }
    t.at(1, P1, order(&[phantom], OrderKind::Attack(peacekeeper)));
    step_until(&mut t, 600, "the Peacekeeper dies", |t| {
        t.sim().object(peacekeeper).is_none()
    });
    let on_ground = hits_on(&t, peacekeeper);
    assert!(on_ground.len() >= 4, "{on_ground:?}");
    assert!(
        on_ground
            .iter()
            .all(|&(_, by, damage)| by == phantom && damage > 0.0 && damage <= 12.0)
    );
    let next = t.sim().tick() + 1;
    t.at(next, P1, order(&[phantom], OrderKind::Attack(sparrow)));
    step_until(&mut t, 1200, "the Sparrow dies", |t| t.sim().object(sparrow).is_none());
    let in_air = hits_on(&t, sparrow);
    assert!(in_air.len() >= 4, "{in_air:?}");
    assert!(
        in_air
            .iter()
            .all(|&(tick, by, damage)| by == phantom && tick > next && damage > 0.0 && damage <= 12.0)
    );
    let launched = (t.events().iter()).any(|(_, e)| matches!(e, SimEvent::ProjectileLaunched { .. }));
    assert!(!launched, "a laser: no projectile");
}
