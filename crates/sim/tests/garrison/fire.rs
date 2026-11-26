//! The occupants' fire (`factions/cults/structures/garrison.md`: "Infantry enter the Garrison and fire from inside it:
//! Recruits and Disciples fire **as if they were Militants** [...], while Militants and Gunners fire their own weapons.
//! Without units inside, the Garrison does not attack"; `> Garrisoning > Range bonus`, `> Targets`), from its footprint
//! (`combat::occupants`).
//!
//! The targets are player 2's Recruits holding position, harmless at a distance (a melee attack), each seen by a
//! "spotter" of player 1's 2 grid units beyond it (a Recruit holding position too), so that what the occupants fire at
//! is decided by their range, not by the Garrison's SightRange of 6 grid units.

use sc_sim::Augmentation;

use super::*;

/// A unit `gap` grid units east of the Garrison's footprint (its east edge, x = 896), level with its middle.
fn east(gap: f32) -> Vec2 {
    Vec2::new(HI.x + gap * GU + HALF.x, DOOR.y)
}

/// A unit `gap` grid units south of the footprint (its south edge, y = 768), below its middle.
fn south(gap: f32) -> Vec2 {
    Vec2::new((LO.x + HI.x) / 2.0, HI.y + gap * GU + HALF.y)
}

/// A unit `gap` grid units north of the footprint (its north edge, y = 640), above its middle.
fn north(gap: f32) -> Vec2 {
    Vec2::new((LO.x + HI.x) / 2.0, LO.y - gap * GU - HALF.y)
}

/// A target at each of `at`, spawned at `tick` with no spawn scheduled before it: player 2's Recruit there, holding
/// position from the next tick, and its spotter, player 1's Recruit 2 grid units farther from the Garrison, holding
/// position too. Returns the targets.
pub(super) fn targets(t: &mut TestSim, tick: Tick, at: &[Vec2]) -> Vec<ObjectId> {
    let centre = (LO + HI) / 2.0;
    let first = next_id(t);
    let mut targets = Vec::new();
    for (i, &pos) in at.iter().enumerate() {
        let (target, spotter) = (id(first + 2 * i as u32), id(first + 2 * i as u32 + 1));
        t.spawn_unit_at(tick, RECRUIT, P2, pos);
        t.spawn_unit_at(tick, RECRUIT, P1, pos + (pos - centre).normalize() * 2.0 * GU);
        t.order_at(tick + 1, P2, &[target], OrderKind::Hold, false);
        t.order_at(tick + 1, P1, &[spotter], OrderKind::Hold, false);
        targets.push(target);
    }
    targets
}

/// The shots so far, as `(tick, attacker, target, from)`: a hit's or a TailDisjointed miss's path from where it left.
fn shots(t: &TestSim) -> Vec<(Tick, ObjectId, Option<ObjectId>, Vec2)> {
    (t.events().iter())
        .filter_map(|(tick, e)| match *e {
            SimEvent::Attacked {
                attacker, target, from, ..
            } => Some((*tick, attacker, Some(target), from)),
            SimEvent::Missed {
                attacker, target, from, ..
            } => Some((*tick, attacker, target, from)),
            _ => None,
        })
        .collect()
}

/// The damage each hit on `id` took so far, as `(tick, attacker, damage)`.
fn hits_on(t: &TestSim, id: ObjectId) -> Vec<(Tick, ObjectId, f32)> {
    (t.events().iter())
        .filter_map(|(tick, e)| match *e {
            SimEvent::Hit {
                attacker,
                target,
                damage,
                ..
            } if target == id => Some((*tick, attacker, damage)),
            _ => None,
        })
        .collect()
}

fn died(t: &TestSim, id: ObjectId) -> bool {
    t.events()
        .iter()
        .any(|(_, e)| matches!(e, SimEvent::Died { id: dead, .. } if *dead == id))
}

/// Design change 0319's verification: "A Garrison with 4 Militants fires at range 5." The Militant's Range of 4 grid
/// units with the Garrison's bonus of 1 (`> Range bonus`: "**Militants and Gunners** inside get **+1 grid unit of
/// Range**"), from the footprint's edge ("**Range** is measured **from the Garrison's edge** (its footprint)"): a target
/// 4.9 grid units off it is hit, by the four, until it dies; one 5.1 grid units off it, which player 1 sees, never.
/// Every shot leaves from the footprint's edge nearest its target (plan decision T72), the east edge level with the
/// first.
#[test]
fn a_garrison_with_four_militants_fires_at_range_5() {
    let mut t = base().start();
    let militants = fill(&mut t, &[MILITANT; 4]);
    let now = t.sim().tick() + 1;
    let [near, far] = targets(&mut t, now, &[east(4.9), south(5.1)])[..] else {
        unreachable!()
    };
    run_for(&mut t, 300);
    assert!(died(&t, near), "the near one is shot dead");
    let hits = hits_on(&t, near);
    assert!(hits.len() >= 10, "40 HP at 4 a hit: {hits:?}");
    assert!(
        hits.iter()
            .all(|&(_, attacker, damage)| militants.contains(&attacker) && damage == 4.0)
    );
    let shooters: std::collections::BTreeSet<ObjectId> = hits.iter().map(|&(_, attacker, _)| attacker).collect();
    assert_eq!(shooters.len(), 4, "the four fire: {shooters:?}");
    assert_eq!(hits_on(&t, far), [], "never in range");
    let far_o = t.sim().object(far).expect("alive");
    assert!(t.sim().is_visible(P1, far_o), "seen all the while");
    for (tick, attacker, target, from) in shots(&t) {
        assert_eq!(target, Some(near), "tick {tick}: {attacker:?}");
        assert_eq!(
            from,
            Vec2::new(HI.x, DOOR.y),
            "from the east edge, level with the target"
        );
    }
    assert_eq!(inside(&t, G1), militants, "they fire from inside");
}

/// `> Garrisoning > Weapons`: "a Recruit or Disciple inside attacks with the Militant's AttackAttributes (see
/// Militant)", at "the Militant's normal Range" of 4 grid units, no bonus, and "**Augmentations don't change the
/// borrowed attack**: a Recruit or Disciple inside fires the Militant's attack unchanged": a Recruit with both
/// augmentations deals the Militant's 4 a hit (its own blow is 3), a shot every 8 ticks (Aim 2, Firing 2, Cooldown 1,
/// Reload 3), on a target 3.9 grid units off the footprint, and never on one 4.1 off it. So does a Disciple, whose type
/// has no attack of its own.
#[test]
fn a_recruit_or_a_disciple_inside_fires_the_militants_attack_unchanged() {
    for kind in [RECRUIT, DISCIPLE] {
        let mut t = base().start();
        let [unit] = column(&mut t, 1, &[kind], DOOR - Vec2::new(60.0, 0.0))[..] else {
            unreachable!()
        };
        let augmentation = Augmentation {
            constitution: true,
            metabolism: true,
        };
        t.debug_at(2, DebugCommand::SetAugmentation { id: unit, augmentation });
        t.at(3, P1, enter(&[unit], G1));
        while inside(&t, G1).is_empty() {
            assert!(t.sim().tick() < 100, "{kind:?} going in");
            step_checked(&mut t);
        }
        let augmented = t.sim().held_unit(unit).expect("inside").object.augmentation;
        assert_eq!(augmented, augmentation, "{kind:?}");
        let now = t.sim().tick() + 1;
        let [near, far] = targets(&mut t, now, &[east(3.9), south(4.1)])[..] else {
            unreachable!()
        };
        run_for(&mut t, 120);
        let hits = hits_on(&t, near);
        assert!(hits.len() >= 10, "{kind:?}: {hits:?}");
        assert!(
            hits.iter()
                .all(|&(_, attacker, damage)| attacker == unit && damage == 4.0),
            "{kind:?}: {hits:?}"
        );
        let gaps: Vec<Tick> = hits.windows(2).map(|w| w[1].0 - w[0].0).collect();
        assert!(
            gaps.iter().all(|&gap| gap == 8),
            "{kind:?}: a shot every 8 ticks: {gaps:?}"
        );
        assert_eq!(hits_on(&t, far), [], "{kind:?}: beyond the Militant's Range");
    }
}

/// `> Garrisoning > Weapons`: "Militants and Gunners use their own AttackAttributes", the Gunner's TailDisjointed shot
/// (`factions/cults/units/gunner.md`: Range 5, MinRange 2), with the bonus (Range 6), "**Range** is measured **from the
/// Garrison's edge** (its footprint), including a Gunner's MinRange": a target 5.9 grid units off the footprint is hit,
/// 16 a shot, its shots leaving from the footprint's edge nearest it; one 1.9 grid units off it (too close) and one 6.1
/// off it (too far), both seen, never.
#[test]
fn a_gunner_inside_fires_its_own_shots_from_the_footprint() {
    let mut t = base().start();
    let [gunner] = fill(&mut t, &[GUNNER])[..] else {
        unreachable!()
    };
    let now = t.sim().tick() + 1;
    let [close, near, far] = targets(&mut t, now, &[north(1.9), east(5.9), south(6.1)])[..] else {
        unreachable!()
    };
    run_for(&mut t, 200);
    let hits = hits_on(&t, near);
    assert!(hits.len() >= 3 && died(&t, near), "40 HP at 16 a shot: {hits:?}");
    assert!(
        hits.iter()
            .all(|&(_, attacker, damage)| attacker == gunner && damage == 16.0)
    );
    assert_eq!((hits_on(&t, close), hits_on(&t, far)), (vec![], vec![]));
    for id in [close, far] {
        let o = t.sim().object(id).expect("alive");
        assert!(t.sim().is_visible(P1, o), "{id:?} seen");
    }
    for (tick, attacker, target, from) in shots(&t) {
        assert_eq!((attacker, target), (gunner, Some(near)), "tick {tick}");
        assert_eq!(from, Vec2::new(HI.x, DOOR.y), "tick {tick}: from the east edge");
    }
}

/// "Without units inside, the Garrison does not attack": a target right beside it is never shot at while it is empty,
/// and is as soon as a Militant is inside.
#[test]
fn an_empty_garrison_does_not_attack() {
    let mut t = base().start();
    let [near] = targets(&mut t, 1, &[east(1.0)])[..] else {
        unreachable!()
    };
    run_for(&mut t, 100);
    assert_eq!(shots(&t), []);
    let militant = id(next_id(&t));
    let now = t.sim().tick() + 1;
    t.spawn_unit_at(now, MILITANT, P1, DOOR - Vec2::new(60.0, 0.0));
    t.at(now + 1, P1, enter(&[militant], G1));
    while inside(&t, G1).is_empty() {
        assert!(t.sim().tick() < now + 100, "going in");
        step_checked(&mut t);
    }
    run_for(&mut t, 20);
    assert!(hits_on(&t, near).iter().any(|&(_, attacker, _)| attacker == militant));
}

/// `> Targets`: "each unit inside **picks its own target** by the normal targeting rules (as the Laser Turret:
/// factions/syndicate/structures/laser_turret.md > Targeting)", which "**keeps its target** while the target stays
/// valid and in range". A Militant and a Recruit inside, and a target 4.5 grid units off the footprint, which only the
/// Militant reaches (5 to the Recruit's 4): the Militant fires at it, the Recruit at nothing. A second target 3.5 off
/// it comes: the Recruit takes it, and the Militant keeps its own, though the new one is closer.
#[test]
fn each_unit_inside_picks_its_own_target() {
    let mut t = base().start();
    let pool = fill(&mut t, &[MILITANT, RECRUIT]);
    let (militant, recruit) = (pool[0], pool[1]);
    let now = t.sim().tick() + 1;
    let [first] = targets(&mut t, now, &[east(4.5)])[..] else {
        unreachable!()
    };
    run_for(&mut t, 30);
    let by = |t: &TestSim, unit: ObjectId| -> Vec<Option<ObjectId>> {
        let mut targets: Vec<Option<ObjectId>> = shots(t).iter().filter(|s| s.1 == unit).map(|s| s.2).collect();
        targets.dedup();
        targets
    };
    assert_eq!(by(&t, militant), [Some(first)]);
    assert_eq!(by(&t, recruit), []);
    let later = t.sim().tick() + 1;
    let [second] = targets(&mut t, later, &[south(3.5)])[..] else {
        unreachable!()
    };
    run_for(&mut t, 30);
    assert!(!died(&t, first), "40 HP at 4 every 8 ticks lasts 80 ticks");
    assert_eq!(by(&t, militant), [Some(first)], "it keeps its target");
    assert_eq!(by(&t, recruit), [Some(second)]);
}
