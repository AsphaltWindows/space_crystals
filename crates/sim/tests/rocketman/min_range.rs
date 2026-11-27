//! The Rocketman's MinRange (`factions/gdo/units/rocketman.md > Placeholder stats`: "MinRange 2"), under the rules
//! the Gunner's tests check (`tests/cults_combat.rs`): `systems/controls/command_behaviors.md > AttackingObject >
//! Target inside MinRange` and `systems/controls/behaviors.md > BaseAutoTargeting > Targets inside MinRange`; and
//! Target Lock's exception to it (`factions/gdo/units/rocketman.md > Target Lock`: "**No minimum range when locked**").

use sc_sim::gu;

use super::*;

/// Half a LightInfantry's 24 su Silhouette: the Rocketman's and the Peacekeeper's.
const HALF: f32 = 12.0;

/// The Rocketman's launches so far (`SimEvent::ProjectileLaunched`): where it stood, and at whom.
fn launches(t: &TestSim, shooter: u32) -> Vec<(Vec2, Option<ObjectId>)> {
    (t.events().iter())
        .filter_map(|(_, e)| match *e {
            SimEvent::ProjectileLaunched {
                shooter: s,
                from,
                target,
                ..
            } if s == id(shooter) => Some((from, target)),
            _ => None,
        })
        .collect()
}

/// The gap, edge to edge, between two LightInfantry units whose centres are `a` and `b` (`systems/combat.md > Range
/// Measurement`), side by side on a row.
fn gap(a: Vec2, b: Vec2) -> f32 {
    a.distance(b) - 2.0 * HALF
}

/// Player 1's Rocketman (1) and player 2's Peacekeeper (2), 1 grid unit east of its edge: inside its MinRange.
/// Auto-targeting off: the Peacekeeper doesn't fight back, and nobody moves but on orders.
fn hugged() -> TestSim {
    let at = Vec2::new(400.0, 320.0);
    let mut t = ScenarioBuilder::new(&flat(20, 10))
        .spawn(ROCKETMAN, Some(P1), SpawnAt::Point(at))
        .spawn(
            ObjectKind::Peacekeeper,
            Some(P2),
            SpawnAt::Point(at + Vec2::new(2.0 * HALF + gu(1.0), 0.0)),
        )
        .start();
    t.without_auto_targeting();
    t
}

/// "MinRange 2": ordered to attack the Peacekeeper inside its MinRange, the Rocketman "backs off until the target is
/// outside MinRange, then attacks", its Attack running all the while: every missile it fires in 10 seconds leaves from
/// 2 grid units off or more, at the Peacekeeper, which stands. Ordered instead to lock onto it, it stands where it is
/// and fires at it from there, 1 grid unit off, its missiles homing: "locked homing missiles can be fired at **any
/// distance** up to LockRange, even inside the normal attack's MinRange".
#[test]
fn the_normal_attack_backs_off_out_of_min_range_and_a_lock_fires_where_it_stands() {
    let mut t = hugged();
    let target = t.pos(2);
    order(&mut t, 2, P1, &[1], OrderKind::Attack(id(2)), false);
    t.run_until(2);
    while launches(&t, 1).is_empty() {
        assert!(t.sim().tick() < 16 * 10, "it fires once out of MinRange");
        assert_eq!(
            current(&t, 1),
            Some(OrderKind::Attack(id(2))),
            "its Attack runs all the while"
        );
        t.step();
    }
    t.run_until(16 * 10);
    let fired = launches(&t, 1);
    for &(from, aimed) in &fired {
        assert_eq!(aimed, Some(id(2)));
        assert!(gap(from, target) >= gu(2.0), "fired from {} off", gap(from, target));
    }

    let mut t = hugged();
    let at = t.pos(1);
    order(&mut t, 2, P1, &[1], OrderKind::TargetLock(id(2)), false);
    t.run_until(2 + LOCK_ON + 16);
    let fired = launches(&t, 1);
    assert!(!fired.is_empty(), "it fires once locked");
    for &(from, aimed) in &fired {
        assert_eq!((from, aimed), (at, Some(id(2))), "from where it stands");
        assert!(gap(from, target) < gu(2.0));
    }
    assert!(missiles_at(&t, 2).iter().all(|m| m.homing));
    assert!(!hits_on(&t, 2).is_empty(), "and hits");
}

/// "Auto-targeting [...] **skips** enemies inside the weapon's MinRange and picks one it can hit": player 1's
/// Rocketman (1) on Hold, which keeps it where it stands, and two Peacekeepers of player 2 on Hold, one (2) 1 grid unit
/// east of its edge, inside its MinRange, the other (3) 5 grid units west of it, within its Range of 6 and beyond the
/// Peacekeeper's of 4; auto-targeting on. The Rocketman fires only at the farther one, though the nearer one shoots it.
#[test]
fn auto_targeting_skips_an_enemy_inside_min_range() {
    let at = Vec2::new(600.0, 320.0);
    let mut t = ScenarioBuilder::new(&flat(20, 10))
        .spawn(ROCKETMAN, Some(P1), SpawnAt::Point(at))
        .spawn(
            ObjectKind::Peacekeeper,
            Some(P2),
            SpawnAt::Point(at + Vec2::new(2.0 * HALF + gu(1.0), 0.0)),
        )
        .spawn(
            ObjectKind::Peacekeeper,
            Some(P2),
            SpawnAt::Point(at - Vec2::new(2.0 * HALF + gu(5.0), 0.0)),
        )
        .start();
    order(&mut t, 1, P1, &[1], OrderKind::Hold, false);
    order(&mut t, 1, P2, &[2, 3], OrderKind::Hold, false);
    t.run_until(40);
    assert!(alive(&t, 1), "still standing");
    let fired = launches(&t, 1);
    assert!(!fired.is_empty(), "it fires");
    assert!(fired.iter().all(|&(_, aimed)| aimed == Some(id(3))), "{fired:?}");
    assert!(!hits_on(&t, 1).is_empty(), "the nearer one shoots it");
}
