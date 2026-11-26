//! A Convert flips the unit's target form (design change 0198, `factions/colonists/units/weapon_conversion.md > A
//! toggle sets a target form`): "Pressing R **flips the unit's target form**; the unit then works to get itself into
//! that form. So however fast R is pressed, every unit that received the presses ends up in the same form". Its type
//! switches at the press, its weapon when its conversion ends (`ARCHITECTURE.md > Object model > Type and form`).

use super::*;

/// The forms of units `ns` (`conversion::form`): the weapons they have.
fn forms(sim: &Sim, ns: &[u32]) -> Vec<ObjectKind> {
    ns.iter().map(|&n| conversion::form(object(sim, n))).collect()
}

/// The types of units `ns` (`Object::kind`): their SelectionGroups.
fn kinds(sim: &Sim, ns: &[u32]) -> Vec<ObjectKind> {
    ns.iter().map(|&n| object(sim, n).kind).collect()
}

/// The current order of unit `n`.
fn current(sim: &Sim, n: u32) -> Option<OrderKind> {
    let orders = object(sim, n).orders.as_ref()?;
    orders.current.as_ref().map(|active| active.order.kind)
}

/// 0198's example (`> A toggle sets a target form`): "4 Sabres (active) and 6 Rifles selected; R → 10 Rifles in one
/// active group; R again at once → all 10 target Sabre, including the original 6." The first R goes to the Sabres (the
/// ActiveGroup, the client's choice), and all ten are of the Rifle's type at once; the second, to all ten in the next
/// tick, makes all ten target the Sabre: the six Rifles convert, and the four, a frame into their conversion, finish
/// it and convert back (8 frames). All ten end as Sabres. Pressed twice in one tick, the four never convert at all.
#[test]
fn four_sabres_and_six_rifles_r_then_r_again_all_end_as_sabres() {
    let sabres = [1, 2, 3, 4];
    let all: Vec<u32> = (1..=10).collect();
    let game = || {
        let mut builder = builder();
        for n in all.iter().copied() {
            let kind = if sabres.contains(&n) { SABRE } else { RIFLE };
            builder = builder.spawn(kind, Some(P1), point(80.0 + 80.0 * n as f32, 192.0));
        }
        builder.start()
    };

    let mut t = game();
    t.at(2, P1, convert(&sabres)).run_until(2);
    assert_eq!(kinds(t.sim(), &all), [RIFLE; 10], "one Rifle group");
    t.at(3, P1, convert(&all)).step();
    assert_eq!(kinds(t.sim(), &all), [SABRE; 10], "all ten target the Sabre");
    assert_eq!(forms(t.sim(), &sabres), [SABRE; 4], "the four, converting still");
    while t.sim().tick() < 1 + 2 * CONVERT_TICKS {
        t.step();
        let tick = t.sim().tick();
        let four = if tick < 2 + CONVERT_TICKS { SABRE } else { RIFLE };
        let six = if tick < 3 + CONVERT_TICKS { RIFLE } else { SABRE };
        assert_eq!(forms(t.sim(), &sabres), [four; 4], "tick {tick}: the four's round trip");
        assert_eq!(forms(t.sim(), &all[4..]), [six; 6], "tick {tick}: the six convert");
        assert_eq!(kinds(t.sim(), &all), [SABRE; 10], "tick {tick}");
    }
    t.step();
    assert_eq!(forms(t.sim(), &all), [SABRE; 10], "tick {}: all Sabres", t.sim().tick());
    assert!(all.iter().all(|&n| object(t.sim(), n).conversion.is_none()));

    let mut t = game();
    t.at(2, P1, convert(&sabres)).at(2, P1, convert(&all)).run_until(2);
    assert_eq!(kinds(t.sim(), &all), [SABRE; 10]);
    assert!(
        sabres.iter().all(|&n| object(t.sim(), n).conversion.is_none()),
        "the four never convert"
    );
    t.run(CONVERT_TICKS);
    assert_eq!(forms(t.sim(), &all), [SABRE; 10]);
}

/// 0198's three cases, one press reaching each (`> A toggle sets a target form`): player 1's Rifle 1, which shoots
/// player 2's Peacekeeper 2, was pressed while Firing and waits ("it just takes the new target; if that is its current
/// weapon again, it doesn't convert at all"); Sabre 3 is a frame into its conversion ("it finishes the 4 frames, then
/// converts back"); Sabre 4 has its rifle already ("it converts back"). The Rifle's shot under way lands, with its
/// rifle (`> Eligibility`: "The shot already under way still finishes"), and it attacks on; each unit keeps its orders
/// throughout (design change 0150), and one converting holds still.
#[test]
fn a_press_on_a_waiting_a_converting_and_a_converted_unit() {
    let mut t = builder()
        .spawn(RIFLE, Some(P1), point(200.0, 192.0)) // 1
        .spawn(PK, Some(P2), point(400.0, 192.0)) // 2, in range
        .spawn(SABRE, Some(P1), point(200.0, 330.0)) // 3
        .spawn(SABRE, Some(P1), point(700.0, 330.0)) // 4
        .start();
    t.without_auto_targeting()
        .order_at(2, P1, &[id(1)], OrderKind::Attack(id(2)), false)
        .at(2, P1, convert(&[4]));
    while !(matches!(phase(t.sim(), 1), Some(AttackPhase::Firing { .. })) && t.sim().tick() > 2 + CONVERT_TICKS) {
        assert!(t.sim().tick() < 60, "the Rifle should fire by now");
        t.step();
    }
    assert_eq!((forms(t.sim(), &[4]), kinds(t.sim(), &[4])), (vec![RIFLE], vec![RIFLE]));
    let rifle = RIFLE.def().unit.as_ref().unwrap().attack.as_ref().unwrap();
    let elapsed = object(t.sim(), 1).attack.as_ref().unwrap().elapsed;
    let b = t.sim().tick() + 1;
    let hit = b + rifle.firing_ticks - elapsed - 1;
    let shots = hits(&t).len();
    t.at(b, P1, convert(&[1, 3])).step();
    assert_eq!(kinds(t.sim(), &[1, 3]), [SABRE, RIFLE]);
    assert_eq!(
        (waits(object(t.sim(), 1)), waits(object(t.sim(), 3))),
        (Some(true), Some(false))
    );
    let at = [3, 4].map(|n| t.pos(n));
    t.at(b + 1, P1, convert(&[1, 3, 4]));
    for tick in b + 1..=b + 2 * CONVERT_TICKS {
        t.step();
        let sim = t.sim();
        assert_eq!(kinds(sim, &[1, 3, 4]), [RIFLE, SABRE, SABRE], "tick {tick}");
        assert_eq!(
            object(sim, 1).conversion,
            None,
            "tick {tick}: the Rifle has nothing to do"
        );
        let three = if (b + CONVERT_TICKS..b + 2 * CONVERT_TICKS).contains(&tick) {
            RIFLE
        } else {
            SABRE
        };
        let four = if tick < b + 1 + CONVERT_TICKS { RIFLE } else { SABRE };
        assert_eq!(forms(sim, &[1, 3, 4]), [RIFLE, three, four], "tick {tick}");
        assert_eq!(
            conversion::is_converting(object(sim, 3)),
            tick < b + 2 * CONVERT_TICKS,
            "tick {tick}"
        );
        assert_eq!(
            conversion::is_converting(object(sim, 4)),
            tick < b + 1 + CONVERT_TICKS,
            "tick {tick}"
        );
        assert_eq!([3, 4].map(|n| t.pos(n)), at, "tick {tick}: they hold still");
        assert_eq!(current(sim, 1), Some(OrderKind::Attack(id(2))), "tick {tick}");
    }
    let rifle_hits: Vec<_> = t.events()[..]
        .iter()
        .filter_map(|(tick, e)| match e {
            SimEvent::Hit { attacker, damage, .. } if *attacker == id(1) => Some((*tick, *damage)),
            _ => None,
        })
        .collect();
    assert!(rifle_hits.contains(&(hit, 9.0)), "its rifle's shot: {rifle_hits:?}");
    assert!(hits(&t).len() > shots, "it fought on");
    assert!(!conversion::is_converting(object(t.sim(), 1)));
}

/// 0198's Verification: "Spam R on a fighting Legionnaire group. When you stop, all units share one form, the one
/// matching the parity of the presses." Player 1's six Sabres attack player 2's two Deployment Centers, set off a
/// tick apart; once each has struck, R goes to the six on irregular ticks, twice in one tick too. Their types are one
/// at every tick (one group); once their conversions are over, their forms are too, the Rifle's after an odd count of
/// presses, the Sabre's after an even one, and they still attack the same Deployment Centers.
#[test]
fn spam_r_on_a_fighting_group_all_end_in_the_form_of_the_presses_parity() {
    for presses in [[0, 0, 1, 3, 4, 4, 5].as_slice(), [0, 1, 1, 2, 5, 6].as_slice()] {
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
        for (x, y) in [
            (176.0, 300.0),
            (176.0, 400.0),
            (592.0, 300.0),
            (816.0, 300.0),
            (1232.0, 300.0),
            (1232.0, 420.0),
        ] {
            builder = builder.spawn(SABRE, Some(P1), point(x, y));
        }
        let mut t = builder.start();
        let sabres: Vec<u32> = (3..=8).collect();
        let target = |n: u32| id(if n < 6 { 1 } else { 2 });
        for (i, &n) in sabres.iter().enumerate() {
            t.order_at(2 + i as Tick, P1, &[id(n)], OrderKind::Attack(target(n)), false);
        }
        let struck = |t: &TestSim, n: u32| hits(t).iter().any(|&(_, attacker, _)| attacker == id(n));
        // (M4-W3) With automatic targeting on, and a longer wait (160 ticks until design change 0323). The
        // Sabres of one building all head for one point by its corner. The second there is stopped by the first, which
        // strikes and doesn't yield; its contact step takes it a whole width round it on its near side, away from the
        // building's face (`docs/movement.md > 6.3`), where it used to shuffle along the first into reach. It then counts
        // as arrived behind it, out of reach, and its Attack ends (`systems/controls/behaviors.md > AttackingObject >
        // Failure`); idle, it takes the building up again by itself and gets to it.
        while !sabres.iter().all(|&n| struck(&t, n)) {
            assert!(t.sim().tick() < 240, "the Sabres should all fight by now");
            t.step();
        }
        let first = t.sim().tick() + 1;
        for &press in presses {
            t.at(first + press, P1, convert(&sabres));
        }
        let last = first + presses.iter().max().unwrap();
        while t.sim().tick() < last || sabres.iter().any(|&n| object(t.sim(), n).conversion.is_some()) {
            assert!(t.sim().tick() < last + 40, "the conversions should be over by now");
            t.step();
            let types = kinds(t.sim(), &sabres);
            assert!(
                types.iter().all(|&kind| kind == types[0]),
                "tick {}: {types:?}",
                t.sim().tick()
            );
        }
        let form = if presses.len() % 2 == 1 { RIFLE } else { SABRE };
        assert_eq!(forms(t.sim(), &sabres), [form; 6], "{} presses", presses.len());
        assert_eq!(kinds(t.sim(), &sabres), [form; 6], "{} presses", presses.len());
        // Under its Attack order still, or, for one whose Attack ended behind another Sabre (above), by itself.
        for &n in &sabres {
            let engaged = object(t.sim(), n).attack.as_ref().and_then(|attack| attack.target);
            let ordered = current(t.sim(), n) == Some(OrderKind::Attack(target(n)));
            assert!(ordered || engaged == Some(target(n)), "Sabre {n}");
        }
    }
}
