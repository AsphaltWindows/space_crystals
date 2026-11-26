//! A Convert pressed mid-attack (design change 0153, `factions/colonists/units/weapon_conversion.md >
//! Eligibility`: "deferred, not rejected"): the gate takes it in every attack phase, and a unit Firing or in Cooldown
//! converts once its Cooldown is over, alone or in a group in a fight.

use super::*;

/// The plan's acceptance, as design change 0153 changed it (`> Eligibility`: it "may be issued freely while the unit
/// is idle, moving, or in the Aiming or Reloading phases [...], interrupting that phase immediately"; "Pressed during
/// those phases [Firing and Cooldown], it is deferred, not rejected"). Player 1's Rifle 1 shoots player 2's
/// Peacekeeper 2, which stands by, through a whole attack sequence: the gate takes a Convert of it in every phase.
/// Idle, Aiming or Reloading, it converts at once; Firing or in Cooldown, its conversion waits. A group's Convert with
/// an idle Sabre 3 goes to both, the Sabre converting at once (question Q0042 point 9, as 0153 answers it). Converted
/// while Reloading, its reload ends at once.
#[test]
fn convert_is_taken_in_every_phase_and_waits_in_firing_or_cooldown() {
    let mut t = builder()
        .spawn(RIFLE, Some(P1), point(200.0, 192.0)) // 1
        .spawn(PK, Some(P2), point(400.0, 192.0)) // 2, in range
        .spawn(SABRE, Some(P1), point(200.0, 300.0)) // 3
        .start();
    t.without_auto_targeting()
        .order_at(2, P1, &[id(1)], OrderKind::Attack(id(2)), false);
    t.run_until(1);
    assert_eq!(check(t.sim(), &[1]), Ok(()), "idle");
    let mut seen = Vec::new();
    while seen.len() < 4 {
        assert!(t.sim().tick() < 40, "a whole sequence: {seen:?}");
        t.step();
        let Some(phase) = phase(t.sim(), 1) else {
            continue;
        };
        assert_eq!(check(t.sim(), &[1]), Ok(()), "{phase:?}");
        // The group's Convert: the Sabre converts, and the Rifle too, or, mid-attack, its conversion waits.
        let mut sim = t.sim().clone();
        step(&mut sim, vec![convert(&[1, 3])]);
        let mid_attack = matches!(phase, AttackPhase::Firing { .. } | AttackPhase::Cooldown);
        let deferred = [1, 3].map(|n| waits(object(&sim, n)));
        assert_eq!(deferred, [Some(mid_attack), Some(false)], "{phase:?}");
        assert_eq!(conversion::is_converting(object(&sim, 1)), !mid_attack, "{phase:?}");
        let name = format!("{phase:?}")
            .chars()
            .take_while(|c| c.is_alphabetic())
            .collect::<String>();
        if !seen.contains(&name) {
            seen.push(name);
        }
    }
    assert_eq!(seen, ["Aiming", "Firing", "Cooldown", "Reloading"]);
    // In Reloading now: the group's Convert goes to both, and the reload ends.
    let tick = t.sim().tick() + 1;
    t.at(tick, P1, convert(&[1, 3])).step();
    let sim = t.sim();
    assert!(object(sim, 1).conversion.is_some() && object(sim, 3).conversion.is_some());
    assert_eq!(phase(sim, 1), None, "the reload is over");
}

/// Design change 0153 for one unit (`> Eligibility`: "the unit finishes Firing and Cooldown, then converts
/// immediately (instead of starting its Reloading phase)"; "Applies to single units too"). Player 1's Rifle 1 shoots
/// player 2's Peacekeeper 2 and is sent a Convert while it is Firing: its type is the Sabre's from the press (design
/// change 0198). It fires on with its rifle, its hit landing at Firing's end, and cools down, its conversion waiting;
/// the gate takes a second Convert meanwhile, which would flip its target back (`target`), and one to a group with an
/// idle Sabre 3 in the Cooldown's last tick flips the Rifle back, which then doesn't convert at all ("if that is its
/// current weapon again, it doesn't convert at all"), and converts the Sabre. Left alone, in the tick after its
/// Cooldown's last it converts, never having reloaded, and 4 ticks later it has its sabre, still attacking the
/// Peacekeeper.
#[test]
fn a_rifle_sent_a_convert_while_firing_converts_after_its_cooldown() {
    let mut t = builder()
        .spawn(RIFLE, Some(P1), point(200.0, 192.0)) // 1
        .spawn(PK, Some(P2), point(400.0, 192.0)) // 2, in range
        .spawn(SABRE, Some(P1), point(200.0, 300.0)) // 3
        .start();
    t.without_auto_targeting()
        .order_at(2, P1, &[id(1)], OrderKind::Attack(id(2)), false);
    while !matches!(phase(t.sim(), 1), Some(AttackPhase::Firing { .. })) {
        assert!(t.sim().tick() < 40, "the Rifle should fire by now");
        t.step();
    }
    let rifle = RIFLE.def().unit.as_ref().unwrap().attack.as_ref().unwrap();
    let elapsed = object(t.sim(), 1).attack.as_ref().unwrap().elapsed;
    let press = t.sim().tick() + 1;
    let hit = press + rifle.firing_ticks - elapsed - 1;
    let cooled = hit + rifle.cooldown_ticks;
    assert_eq!(check(t.sim(), &[1]), Ok(()), "Firing: taken");
    t.at(press, P1, convert(&[1])).step();
    while t.sim().tick() < cooled {
        let o = object(t.sim(), 1);
        assert_eq!(waits(o), Some(true), "tick {}", t.sim().tick());
        assert!(combat::mid_attack(o) && !conversion::is_converting(o));
        assert_eq!((o.kind, conversion::form(o)), (SABRE, RIFLE), "tick {}", t.sim().tick());
        assert_eq!(check(t.sim(), &[1]), Ok(()), "a second Convert is taken");
        t.step();
    }
    assert!(hits(&t).contains(&(hit, id(1), id(2))), "it fired on: {:?}", hits(&t));
    // The Cooldown's last tick: its combat phase put the Rifle into Reloading, which doesn't run.
    assert_eq!(phase(t.sim(), 1), Some(AttackPhase::Reloading));
    let mut sim = t.sim().clone();
    step(&mut sim, vec![convert(&[1, 3])]);
    let deferred = [1, 3].map(|n| waits(object(&sim, n)));
    assert_eq!(
        deferred,
        [None, Some(false)],
        "the Rifle flipped back, the Sabre converting"
    );
    assert_eq!([1, 3].map(|n| object(&sim, n).kind), [RIFLE, RIFLE]);
    assert_eq!(phase(&sim, 1), Some(AttackPhase::Reloading), "the Rifle reloads on");
    assert_eq!(object(&sim, 3).conversion.unwrap().frames_done(), Some(1));
    t.step();
    let o = object(t.sim(), 1);
    assert!(conversion::is_converting(o), "tick {}", t.sim().tick());
    assert_eq!(o.conversion.unwrap().frames_done(), Some(1));
    assert_eq!(phase(t.sim(), 1), None, "no reload");
    t.run(CONVERT_TICKS - 1);
    assert_eq!(
        conversion::form(object(t.sim(), 1)),
        RIFLE,
        "converting through tick {}",
        t.sim().tick()
    );
    t.step();
    let o = object(t.sim(), 1);
    assert_eq!(
        (o.kind, conversion::form(o), o.conversion),
        (SABRE, SABRE, None),
        "tick {}",
        t.sim().tick()
    );
    assert_eq!(o.attack.as_ref().unwrap().target, Some(id(2)));
    let current = o.orders.as_ref().unwrap().current.as_ref().map(|a| a.order.kind);
    assert_eq!(current, Some(OrderKind::Attack(id(2))));
}

/// Design change 0153's Verification: "10 Sabres fighting, press R once: all 10 end up Rifles; a few convert 1–4
/// frames after the others." Player 1's ten Sabres attack player 2's two Deployment Centers, set off a tick apart so
/// that their attack sequences don't keep step. Once each has struck, R goes to all ten in a tick when some of them
/// are mid-swing (Firing or in Cooldown) and some aren't. All ten are of the Rifle's type from the press (design
/// change 0198), so one group; those that aren't mid-swing have their rifles 4 ticks after it, each of the others as
/// many ticks later as it had Firing and Cooldown ticks left (1 to 4: a Sabre fires 3, cools down 1).
#[test]
fn ten_sabres_fighting_all_convert_on_one_press_the_mid_swing_ones_a_little_later() {
    // Two Deployment Centers (1, 2) on tiles (4, 4) to (7, 7) and (14, 4) to (17, 7); five Sabres by each, a step
    // from its sides.
    let mut builder = ScenarioBuilder::new(&flat(24, 12))
        .faction(P1, Faction::Colonists)
        .spawn(
            ObjectKind::DeploymentCenter,
            Some(P2),
            SpawnAt::Tile(TilePos::new(4, 4)),
        )
        .spawn(
            ObjectKind::DeploymentCenter,
            Some(P2),
            SpawnAt::Tile(TilePos::new(14, 4)),
        );
    let near = [
        (176.0, 300.0),
        (176.0, 400.0),
        (176.0, 480.0),
        (592.0, 300.0),
        (592.0, 420.0),
        (816.0, 300.0),
        (816.0, 420.0),
        (1232.0, 300.0),
        (1232.0, 400.0),
        (1232.0, 480.0),
    ];
    for (x, y) in near {
        builder = builder.spawn(SABRE, Some(P1), point(x, y));
    }
    let mut t = builder.start();
    let sabres: Vec<u32> = (3..=12).collect();
    for (i, &n) in sabres.iter().enumerate() {
        let target = if i < 5 { 1 } else { 2 };
        t.order_at(2 + i as Tick, P1, &[id(n)], OrderKind::Attack(id(target)), false);
    }
    let struck = |t: &TestSim, n: u32| hits(t).iter().any(|&(_, attacker, _)| attacker == id(n));
    // (M4-W3) With automatic targeting on, and a longer wait (160 ticks until design change 0323). The
    // Sabres of one building all head for one point by its corner. The second there is stopped by the first, which
    // strikes and doesn't yield; its contact step takes it a whole width round it on its near side, away from the
    // building's face (`docs/movement.md > 6.3`), where it used to shuffle along the first into reach. It then counts
    // as arrived behind it, out of reach, and its Attack ends (`systems/controls/command_behaviors.md >
    // AttackingObject > Failure`); idle, it takes the building up again by itself and gets to it.
    while !sabres.iter().all(|&n| struck(&t, n)) {
        assert!(t.sim().tick() < 240, "the Sabres should all fight by now");
        t.step();
    }
    let mid_swing = |t: &TestSim| -> Vec<u32> {
        (sabres.iter().copied())
            .filter(|&n| combat::mid_attack(object(t.sim(), n)))
            .collect()
    };
    while !(2..=8).contains(&mid_swing(&t).len()) {
        assert!(t.sim().tick() < 190, "some mid-swing, some not");
        t.step();
    }
    // The ticks each one's conversion waits: its Firing and Cooldown ticks left.
    let sabre = SABRE.def().unit.as_ref().unwrap().attack.as_ref().unwrap();
    let wait = |n: u32| {
        let attack = object(t.sim(), n).attack.as_ref().unwrap();
        match attack.phase {
            Some(AttackPhase::Firing { .. }) => sabre.firing_ticks - attack.elapsed + sabre.cooldown_ticks,
            Some(AttackPhase::Cooldown) => sabre.cooldown_ticks - attack.elapsed,
            _ => 0,
        }
    };
    let press = t.sim().tick() + 1;
    let expected: Vec<(ObjectId, Tick)> = (sabres.iter())
        .map(|&n| (id(n), press + CONVERT_TICKS + wait(n)))
        .collect();
    let late: Vec<Tick> = (sabres.iter()).map(|&n| wait(n)).filter(|&w| w > 0).collect();
    assert!(late.len() >= 2 && late.iter().all(|w| (1..=4).contains(w)), "{late:?}");
    t.at(press, P1, convert(&sabres));
    let mut armed: Vec<(ObjectId, Tick)> = Vec::new();
    while t.sim().tick() < press + CONVERT_TICKS + 4 {
        t.step();
        for &n in &sabres {
            let o = object(t.sim(), n);
            assert_eq!(o.kind, RIFLE, "tick {}: the type from the press", t.sim().tick());
            if conversion::form(o) == RIFLE && !armed.iter().any(|&(armed, _)| armed == id(n)) {
                armed.push((id(n), t.sim().tick()));
            }
        }
    }
    armed.sort();
    assert_eq!(armed, expected, "pressed in tick {press}");
    let mut changed: Vec<(ObjectId, Tick)> = (t.events().iter())
        .filter_map(|(tick, e)| match e {
            SimEvent::KindChanged { id, from, to } => {
                assert_eq!((*from, *to), (SABRE, RIFLE));
                Some((*id, *tick))
            }
            _ => None,
        })
        .collect();
    changed.sort();
    let at_the_press: Vec<(ObjectId, Tick)> = sabres.iter().map(|&n| (id(n), press)).collect();
    assert_eq!(changed, at_the_press, "one KindChanged each, at the press");
    for &n in &sabres {
        assert_eq!(object(t.sim(), n).conversion, None, "all 10 end up Rifles");
    }
}
