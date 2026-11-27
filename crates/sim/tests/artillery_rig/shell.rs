//! The deployed Rig's shell (`factions/gdo/units/artillery_rig.md > AttackAttributes (Deployed only)`): "AttackType:
//! **TailDisjointed** — the target location is locked at the end of Aiming; the blast lands there at the end of Firing
//! (no tracking projectile) [...] Can also be fired at a ground location"; "TargetType: AoE — **friendly fire**";
//! "TargetDomain: Ground"; Range 12 and MinRange 3 grid units, a circle of 64 su; "It can fire at any enemy visible to
//! its owner". The deployed Rig is spawned deployed.

use super::*;

/// Acceptance, "an enemy at 2.5 GU can't be targeted (MinRange)" (`systems/controls/command_behaviors.md >
/// AttackingObject > Target inside MinRange`: "A unit that can't move (e.g. a deployed Artillery Rig) can't carry it
/// out: the order fails with feedback ("Target too close"; same channel as other refusals)"): an Attack on a
/// Peacekeeper 2.5 grid units from the deployed Rig, edge to edge, ends in its own tick with a
/// `SimEvent::TargetTooClose`, and no shell is fired.
#[test]
fn an_attack_inside_min_range_fails_too_close() {
    let pk = id(2);
    let mut t = field(DEPLOYED, |b| {
        b.spawn(PK, Some(P2), SpawnAt::Point(east_of_rig(2.5, half(PK), 0.0)))
    });
    let given = give(&mut t, P1, order(&[UNIT], OrderKind::Attack(pk)));
    let too_close = |(tick, e): &(Tick, SimEvent)| {
        *tick == given && matches!(*e, SimEvent::TargetTooClose { unit, target } if unit == UNIT && target == pk)
    };
    assert!(t.events().iter().any(too_close), "Target too close");
    assert_eq!(current(&t, UNIT), None);
    t.run(200);
    assert!(!ever_attacked(&t, UNIT));
}

/// "Auto-targeting never picks such an enemy": with auto-targeting on, the deployed Rig leaves alone a Disciple (which
/// has no attack, so it never fires back) 2.5 grid units off, for 400 frames; one 4.5 grid units off it picks and
/// shells.
#[test]
fn auto_targeting_never_picks_an_enemy_inside_min_range() {
    let disciple = ObjectKind::Disciple;
    let scene = |gap: f32| {
        let mut t = field(DEPLOYED, |b| {
            b.spawn(
                disciple,
                Some(P2),
                SpawnAt::Point(east_of_rig(gap, half(disciple), 0.0)),
            )
        });
        auto_targeting(&mut t);
        t
    };
    let mut near = scene(2.5);
    near.run(400);
    assert!(!ever_attacked(&near, UNIT), "inside MinRange");
    let mut far = scene(4.5);
    far.run(100);
    assert!(
        !blasts_by(&far, UNIT).is_empty(),
        "it shells the one 4.5 grid units off"
    );
}

/// Player 2's Peacekeeper (2) `gap` grid units east of the deployed Rig, edge to edge, and, with `spotter`, player 1's
/// Peacekeeper (3) 2 grid units south of it, which sees it.
fn spotted(gap: f32, spotter: bool) -> TestSim {
    let there = east_of_rig(gap, half(PK), 0.0);
    field(DEPLOYED, |b| {
        let b = b.spawn(PK, Some(P2), SpawnAt::Point(there));
        match spotter {
            true => b.spawn(PK, Some(P1), SpawnAt::Point(there + Vec2::new(0.0, gu(2.0) + 24.0))),
            false => b,
        }
    })
}

/// The Rig's Attack on player 2's Peacekeeper, given in the next tick: its first shell lands on the Peacekeeper,
/// which it hurts, and the Rig hasn't moved.
fn shells_the_peacekeeper(t: &mut TestSim) {
    let pk = id(2);
    let at = object(t, UNIT).pos;
    give(t, P1, order(&[UNIT], OrderKind::Attack(pk)));
    t.run(80);
    let blasts = blasts_by(t, UNIT);
    assert_eq!(blasts.len(), 1, "{blasts:?}");
    let (tick, centre, radius) = blasts[0];
    assert_eq!((centre, radius), (object(t, pk).pos, RADIUS), "on the Peacekeeper");
    let hit = hits_on(t, pk);
    assert!(
        hit.iter()
            .any(|&(at, by, damage)| at == tick && by == UNIT && damage > 0.0),
        "{hit:?}"
    );
    assert_eq!(object(t, UNIT).pos, at, "from where it stands");
}

/// Acceptance, "one at 12 GU can [be targeted]": player 2's Peacekeeper at the Rig's Range of 12 grid units, edge to
/// edge, seen by a spotter, takes a shell.
#[test]
fn an_enemy_at_range_12_is_hit() {
    let mut t = spotted(12.0, true);
    shells_the_peacekeeper(&mut t);
}

/// `> AttackAttributes`, Vision: "It can fire at any enemy visible to its owner" (its Range of 12 is twice its sight of
/// 6): an enemy 11 grid units off, out of the Rig's own sight, can't be attacked (`Reject::NotVisible`); seen by a
/// friendly Peacekeeper, it is attacked, and hit.
#[test]
fn a_spotters_target_11_grid_units_off_is_hit() {
    let pk = id(2);
    let alone = spotted(11.0, false);
    assert_eq!(
        gate(&alone, P1, order(&[UNIT], OrderKind::Attack(pk))),
        Err(Reject::NotVisible(pk))
    );
    let mut t = spotted(11.0, true);
    shells_the_peacekeeper(&mut t);
}

/// An Agent of player 2's (6 su a frame) at `from`, walking south for 12 grid units from tick 2 on if `walks`, and the
/// deployed Rig's Attack on it in tick 2.
fn agent(from: Vec2, walks: bool) -> TestSim {
    let agent = id(2);
    let mut t = field(DEPLOYED, |b| b.spawn(ObjectKind::Agent, Some(P2), SpawnAt::Point(from)));
    if walks {
        let to = OrderKind::Move(Target::Point(from + Vec2::new(0.0, gu(12.0))));
        t.order_at(2, P2, &[agent], to, false);
    }
    t.order_at(2, P1, &[UNIT], OrderKind::Attack(agent), false);
    t
}

/// Acceptance, "a target that moves 1.5 GU during the 16-frame Firing is missed" (`systems/combat.md >
/// TailDisjointed`): an Agent walking south across the Rig's line of fire, 4.5 grid units off, at its 6 su a frame, is
/// locked where it is as the Rig's Aiming ends; 16 frames on, the blast lands there, the Agent 96 su (1.5 grid units)
/// on, its 36 su Silhouette well out of the 64 su circle, and hurts no one.
#[test]
fn a_target_that_walks_1_5_grid_units_during_firing_is_missed() {
    let agent = id(2);
    let mut t = agent_at(true);
    let (locked, at) = until_locked(&mut t);
    assert_eq!(at, object(&t, agent).pos, "where the Agent is as the Aiming ends");
    t.run_until(locked + FIRING - 1);
    assert!(blasts_by(&t, UNIT).is_empty(), "not before the end of Firing");
    t.step();
    assert_eq!(blasts_by(&t, UNIT), [(locked + FIRING, at, RADIUS)]);
    let walked = object(&t, agent).pos.distance(at);
    assert!(walked > gu(1.5) - 0.01, "{walked} su");
    assert!(hits_on(&t, agent).is_empty(), "missed");
    assert!(
        !t.events().iter().any(|(_, e)| matches!(e, SimEvent::Hit { .. })),
        "and no one else hit"
    );
}

/// The same Agent standing still where the walking one was locked: the blast lands on it, 16 frames after the end of
/// the Aiming, and hurts it.
#[test]
fn a_target_that_stands_still_is_hit() {
    let agent = id(2);
    let mut t = agent_at(false);
    let (locked, at) = until_locked(&mut t);
    t.run_until(locked + FIRING);
    assert_eq!(blasts_by(&t, UNIT), [(locked + FIRING, at, RADIUS)]);
    let hit = hits_on(&t, agent);
    assert!(
        matches!(hit[..], [(tick, UNIT, damage)] if tick == locked + FIRING && damage > 0.0),
        "{hit:?}"
    );
}

/// [`agent`]: the walking one starts 3 grid units north of the Rig's line, 4.5 grid units off it, and crosses it about
/// as the Rig locks; the standing one stands on that line.
fn agent_at(walks: bool) -> TestSim {
    let dy = if walks { -gu(3.0) } else { 0.0 };
    agent(east_of_rig(4.5, half(ObjectKind::Agent), dy), walks)
}

/// Acceptance, "own units in the blast are hurt" (`> AttackAttributes`: "TargetType: AoE — **friendly fire**: the
/// blast damages friendly units too"): the shell at player 2's Peacekeeper hurts player 1's Peacekeeper beside it,
/// inside the circle, as it hurts its target.
#[test]
fn own_units_in_the_blast_are_hurt() {
    let (enemy, own) = (id(2), id(3));
    let there = east_of_rig(5.0, half(PK), 0.0);
    let mut t = field(DEPLOYED, |b| {
        b.spawn(PK, Some(P2), SpawnAt::Point(there))
            .spawn(PK, Some(P1), SpawnAt::Point(there + Vec2::new(0.0, 30.0)))
    });
    give(&mut t, P1, order(&[UNIT], OrderKind::Attack(enemy)));
    t.run(80);
    let blasts = blasts_by(&t, UNIT);
    assert_eq!(blasts.len(), 1, "{blasts:?}");
    let blast = blasts[0].0;
    for n in [enemy, own] {
        let hit = hits_on(&t, n);
        assert!(
            hit.iter()
                .any(|&(tick, by, damage)| tick == blast && by == UNIT && damage > 0.0),
            "#{}: {hit:?}",
            n.0
        );
    }
}

/// Acceptance, "Attack Ground works" (`systems/controls/command_behaviors.md > AttackingLocation`): the deployed Rig
/// fires at a point 6 grid units east, where player 2's Peacekeeper stands, each blast centred there, and keeps firing
/// at it, a shell every 104 frames, each hurting the Peacekeeper.
#[test]
fn attack_ground_shells_the_spot() {
    let pk = id(2);
    let spot = east_of_rig(6.0, 0.0, 0.0);
    let mut t = field(DEPLOYED, |b| b.spawn(PK, Some(P2), SpawnAt::Point(spot)));
    give(&mut t, P1, order(&[UNIT], OrderKind::AttackGround(spot)));
    t.run(150);
    let blasts = blasts_by(&t, UNIT);
    assert_eq!(blasts.len(), 2, "{blasts:?}");
    assert!(
        blasts.iter().all(|&(_, at, radius)| (at, radius) == (spot, RADIUS)),
        "{blasts:?}"
    );
    assert_eq!(blasts[1].0 - blasts[0].0, 104, "a shell every 104 frames");
    let hit = hits_on(&t, pk);
    let ticks: Vec<Tick> = hit
        .iter()
        .filter(|&&(_, by, damage)| by == UNIT && damage > 0.0)
        .map(|h| h.0)
        .collect();
    assert_eq!(ticks, [blasts[0].0, blasts[1].0], "{hit:?}");
}

/// Slice M6-GF's `Reject::CannotHit` (`systems/combat.md > ValidTarget` 3), for the shell's "TargetDomain: Ground": an
/// Attack on a Sparrow is refused.
#[test]
fn an_attack_on_an_aircraft_is_refused() {
    let sparrow = id(2);
    let t = field(DEPLOYED, |b| {
        b.spawn(ObjectKind::Sparrow, Some(P2), SpawnAt::Point(from_home(gu(4.0), 0.0)))
    });
    assert_eq!(
        gate(&t, P1, order(&[UNIT], OrderKind::Attack(sparrow))),
        Err(Reject::CannotHit(sparrow))
    );
}
