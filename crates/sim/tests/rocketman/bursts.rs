//! (M5-S8) Design change 0445's Verification (answering Q0173): the Rocketman's missiles, normal and locked, burst as
//! area damage (`factions/gdo/units/rocketman.md > AttackAttributes (normal attack)`: "Damage **40** total over an
//! AoERadius of **32 su**", `systems/combat.md > AoEDamage`), keeping the domain of what they were fired at. Units are
//! spawned directly, with auto-targeting off, so that nobody fights but on orders.

use std::collections::BTreeMap;

use super::*;

/// A missile's burst of radius 32 su (`SimEvent::Blast`) so far: (tick, shooter, centre).
fn bursts(t: &TestSim) -> Vec<(Tick, u32, Vec2)> {
    (t.events().iter())
        .filter_map(|(tick, e)| match *e {
            SimEvent::Blast {
                at,
                radius,
                attacker: Some(attacker),
            } => {
                assert_eq!(radius, 32.0, "an AoERadius of 32 su");
                Some((*tick, attacker.0, at))
            }
            _ => None,
        })
        .collect()
}

/// What `shooter`'s hits in tick `tick` took: HP by object.
fn struck(t: &TestSim, tick: Tick, shooter: u32) -> BTreeMap<u32, f32> {
    (t.events().iter())
        .filter_map(|(at, e)| match *e {
            SimEvent::Hit {
                attacker,
                target,
                damage,
                ..
            } if *at == tick && attacker == id(shooter) => Some((target.0, damage)),
            _ => None,
        })
        .collect()
}

/// "Rocketmen fire at a tight clump of enemy infantry: several take damage per missile; own infantry next to the target
/// take some too." Player 1's two Rocketmen (1, 2) attack player 2's Militant (3) in the middle of a clump of five, the
/// others (4 to 7) 26 su off it each way, their 24 su boxes 2 su apart, and player 1's own Peacekeeper (8) stands at
/// the clump's corner, diagonally next to the target. Each missile lands on the target's spot, and its burst hits every
/// one of the five, and the Peacekeeper: friendly fire, as all area damage. The Rocketmen, 280 su off, take nothing.
#[test]
fn a_missile_into_a_clump_of_infantry_hits_several_and_own_infantry_too() {
    let centre = Vec2::new(880.0, 640.0);
    let mut scenario =
        field(30, 20)
            .spawn(ROCKETMAN, Some(P1), point(600.0, 600.0))
            .spawn(ROCKETMAN, Some(P1), point(600.0, 680.0));
    let clump = [(0.0, 0.0), (-26.0, 0.0), (26.0, 0.0), (0.0, -26.0), (0.0, 26.0)];
    for (dx, dy) in clump {
        scenario = scenario.spawn(ObjectKind::Militant, Some(P2), point(centre.x + dx, centre.y + dy));
    }
    let own = 8;
    let mut t = scenario
        .spawn(
            ObjectKind::Peacekeeper,
            Some(P1),
            point(centre.x + 26.0, centre.y + 26.0),
        )
        .start();
    t.without_auto_targeting();
    order(&mut t, 2, P1, &[1, 2], OrderKind::Attack(id(3)), false);
    t.run_until(80);
    let landed = bursts(&t);
    assert!(landed.len() >= 4, "two missiles each: {landed:?}");
    for &(tick, shooter, at) in &landed {
        assert!((at - centre).length() < 1.0, "on the target's spot: {at}");
        let hit = struck(&t, tick, shooter);
        let enemies = (3..=7).filter(|n| hit.contains_key(n)).count();
        assert_eq!(enemies, 5, "tick {tick}: all five of the clump: {hit:?}");
        assert!(hit.contains_key(&own), "tick {tick}: the own Peacekeeper too: {hit:?}");
        assert!(hit[&3] > hit[&own], "the target, wholly inside, takes most: {hit:?}");
        assert!(!hit.contains_key(&1) && !hit.contains_key(&2), "not the Rocketmen");
    }
}

/// "A missile fired at a Sparrow that flies on bursts harmlessly; a ground unit under that spot is not hit." Player 2's
/// Sparrow (2) flies south past player 1's Rocketman (1), in its Range, and the Rocketman attacks it: its missile flies
/// to where the Sparrow was at the end of Aiming and bursts there, in the air, the domain of what it was fired at,
/// with the Sparrow long gone. Player 2's Militant (3), spawned on that spot as the missile flies, is not hit: nothing
/// is.
#[test]
fn a_missile_at_a_sparrow_that_flies_on_bursts_harmlessly() {
    let mut t = field(30, 20)
        .spawn(ROCKETMAN, Some(P1), point(600.0, 640.0))
        .spawn(SPARROW, Some(P2), point(880.0, 420.0))
        .start();
    t.without_auto_targeting();
    order(
        &mut t,
        2,
        P2,
        &[2],
        OrderKind::Move(sc_sim::Target::Point(Vec2::new(880.0, 1200.0))),
        false,
    );
    order(&mut t, 2, P1, &[1], OrderKind::Attack(id(2)), false);
    while missiles_at(&t, 2).is_empty() {
        assert!(t.sim().tick() < 100, "it fires\n{}", t.ascii(None));
        t.step();
    }
    let [missile] = missiles_at(&t, 2)[..] else {
        panic!("one missile");
    };
    assert!(!missile.homing);
    let spot = (t.sim().projectiles().iter())
        .find(|p| p.shooter == id(1))
        .map(|p| p.to)
        .expect("in flight");
    t.spawn_unit_at(missile.tick + 1, ObjectKind::Militant, P2, spot)
        .run_until(missile.lands_at);
    assert_eq!(t.sim().object(id(3)).map(|o| o.pos), Some(spot), "under the spot");
    assert_eq!(bursts(&t), [(missile.lands_at, 1, spot)]);
    let sparrow = t.sim().object(id(2)).expect("flying on");
    assert!(sparrow.pos.distance(spot) > 60.0, "long gone: {}", sparrow.pos);
    let hits = (t.events().iter()).filter(|(_, e)| matches!(e, SimEvent::Hit { .. }));
    assert_eq!(hits.count(), 0, "harmless: nothing is hit");
}

/// "A locked missile on a Sparrow flying in a group of Sparrows damages the neighbours in the 32 su radius." Player 2's
/// three Sparrows (2 to 4) circle side by side, spawned 44 su apart, and player 1's Rocketman (1) locks onto the middle
/// one. Its locked missile bursts on it, a whole burst (all of its 40 × 40 inside the circle), and the neighbours, whose
/// near edges lie within the 32 su, take their smaller shares in the same tick; nothing on the ground is hit.
#[test]
fn a_locked_missile_on_a_sparrow_in_a_group_hits_its_neighbours() {
    let mut t = field(30, 20)
        .spawn(ROCKETMAN, Some(P1), point(600.0, 640.0))
        .spawn(SPARROW, Some(P2), point(836.0, 640.0))
        .spawn(SPARROW, Some(P2), point(880.0, 640.0))
        .spawn(SPARROW, Some(P2), point(924.0, 640.0))
        .start();
    t.without_auto_targeting();
    order(&mut t, 2, P1, &[1], OrderKind::TargetLock(id(3)), false);
    while bursts(&t).is_empty() {
        assert!(t.sim().tick() < 200, "it fires\n{}", t.ascii(None));
        t.step();
    }
    let [(tick, 1, at)] = bursts(&t)[..] else {
        panic!("one burst: {:?}", bursts(&t));
    };
    assert!(missiles_at(&t, 3).iter().all(|m| m.homing), "a locked one");
    let hit = struck(&t, tick, 1);
    assert!(whole_burst(hit[&3]), "on the target's centre: {hit:?} at {at}");
    let neighbours: Vec<f32> = [2, 4].iter().filter_map(|n| hit.get(n).copied()).collect();
    assert!(!neighbours.is_empty(), "a neighbour in the radius: {hit:?}");
    assert!(
        neighbours.iter().all(|&share| share > 0.0 && share < hit[&3]),
        "their shares: {hit:?}"
    );
    assert!(hit.keys().all(|n| (2..=4).contains(n)), "only the Sparrows: {hit:?}");
}
