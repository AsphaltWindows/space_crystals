//! A toggle sets a target state (design change 0198, `factions/gdo/units.md > Peacekeeper > Riot Shield > Two unit
//! types`: "pressing R flips the Peacekeeper's target (shield up or down), and the unit works to reach it. The type,
//! and so the SelectionGroup, switches **at the press**, including when the raise is deferred behind a shot"), and
//! the orders the unit takes meanwhile follow its type (design change 0202, `> During the toggle > Orders given
//! during the toggle`).

use super::*;

/// How far up the shield of unit `id` is (`RiotShield::up`).
fn up(t: &TestSim, id: u32) -> u32 {
    t.sim().object(ObjectId(id)).unwrap().riot_shield.unwrap().up
}

fn current(t: &TestSim, id: u32) -> Option<OrderKind> {
    let orders = t.sim().object(ObjectId(id)).unwrap().orders.as_ref().unwrap();
    orders.current.as_ref().map(|active| active.order.kind)
}

/// Five Peacekeepers of player 1 (ids 1 to 5), ordered to attack player 2's Peacekeeper (Shielded) 6 a tick apart (as
/// in `toggle::fighting_peacekeepers_all_raise_some_a_few_frames_later`: in tick 7, units 2 to 4 are mid-shot), and
/// player 1's Peacekeeper (Shielded) 7, its shield up, standing by.
fn fighting() -> TestSim {
    let mut t = spawned(
        &[
            (PK, P1, Vec2::new(300.0, 200.0)),       // 1
            (PK, P1, Vec2::new(300.0, 260.0)),       // 2
            (PK, P1, Vec2::new(300.0, 320.0)),       // 3
            (PK, P1, Vec2::new(300.0, 380.0)),       // 4
            (PK, P1, Vec2::new(300.0, 440.0)),       // 5
            (SHIELDED, P2, Vec2::new(500.0, 320.0)), // 6, its shield up: it never fires back
            (SHIELDED, P1, Vec2::new(150.0, 470.0)), // 7
        ],
        false,
    );
    for unit in 1..=5 {
        t.at(unit + 1, P1, attack_order(&[unit], 6, false).command);
    }
    t
}

/// 0198's Verification: "Peacekeepers mid-shot: press R. The panel shows Lower Shield at once. Press R again before
/// the shot ends: none raise." R in tick 7 raises the five (the Peacekeepers' ActiveGroup), all of them Peacekeepers
/// (Shielded) from that press, so the panel is the Shielded type's; R again in tick 8 lowers the merged group, unit 7
/// included ("A second press therefore reaches everyone, original members included"). Per unit (`> Two unit types`):
/// units 2 to 4, whose raise waited for their shot, take the new target and never raise ("Lower pressed while a raise
/// is still deferred [...] the shield never starts rising"), their shots landing; units 1 and 5, a tick up, turn back
/// from there (design change 0171); unit 7, all the way up, lowers. All six end as Peacekeepers, shields down, and the
/// five fight on, their Attack orders kept (design change 0150).
#[test]
fn peacekeepers_mid_shot_r_then_r_again_before_the_shot_ends_none_raise() {
    let mut t = fighting();
    t.ability_at(7, P1, &ids(&[1, 2, 3, 4, 5]), RAISE)
        .ability_at(8, P1, &ids(&[1, 2, 3, 4, 5, 7]), LOWER)
        .run_until(6);
    t.step();
    let mid_shot = [2, 3, 4];
    for unit in 1..=5 {
        assert_eq!(kind(&t, unit), SHIELDED, "#{unit}: the Shielded type at the press");
    }
    for unit in mid_shot {
        // Deferred, or, its Cooldown over in this tick, to begin in the next.
        let waits = matches!(shield(&t, unit), ShieldMode::Deferred | ShieldMode::Raising { left: T });
        assert!(waits && up(&t, unit) == 0, "#{unit}: {:?}", shield(&t, unit));
    }
    assert_eq!(shield(&t, 4), ShieldMode::Deferred, "#4 cools down");
    assert_eq!([up(&t, 1), up(&t, 5)], [1, 1], "off the ground at once");
    let mut ups = Vec::new();
    for _ in 8..=8 + T {
        t.step();
        for unit in 1..=7 {
            if unit != 6 {
                assert_eq!(kind(&t, unit), PK, "#{unit} in tick {}", t.sim().tick());
            }
        }
        for unit in mid_shot {
            assert_eq!(up(&t, unit), 0, "#{unit} in tick {}: never raises", t.sim().tick());
        }
        ups.push([1, 5, 7].map(|unit| up(&t, unit)));
    }
    assert_eq!(ups[0], [0, 0, T - 1], "tick 8: 1 and 5 back down, 7 on its way");
    assert_eq!(ups[T as usize - 1], [0, 0, 0], "tick {}: 7 down", 8 + T - 1);
    assert_eq!(
        hits_on(&t, 6)[..4],
        [(4, 4.0), (5, 4.0), (6, 4.0), (7, 4.0)],
        "the shots under way land"
    );
    for unit in 1..=5 {
        assert_eq!(shield(&t, unit), ShieldMode::Lowered, "#{unit}");
        assert_eq!(current(&t, unit), Some(OrderKind::Attack(ObjectId(6))), "#{unit}");
    }
    t.run_until(40);
    for unit in 1..=5 {
        assert!(attacks_by(&t, unit).iter().any(|&tick| tick > 8), "#{unit} fights on");
    }
}

/// 0198 (`> Two unit types`: "However fast the button is pressed, every unit that received the presses ends up in the
/// same state"): R pressed on fighting Peacekeepers on irregular ticks, twice in a tick too, sends Raise Shield to the
/// group while it is of the Peacekeeper's type and Lower Shield while it is the Shielded one's, as its panel shows.
/// The five are of one type at every tick, and once the shields are done, theirs are all up after an odd count of
/// presses, all down after an even one.
#[test]
fn spam_r_on_fighting_peacekeepers_all_end_in_the_state_of_the_presses_parity() {
    for presses in [[7, 7, 8, 10, 11, 11, 12].as_slice(), [7, 8, 8, 9, 12, 13].as_slice()] {
        let mut t = fighting();
        let five = ids(&[1, 2, 3, 4, 5]);
        for (n, &tick) in presses.iter().enumerate() {
            let ability = if n % 2 == 0 { RAISE } else { LOWER };
            t.ability_at(tick, P1, &five, ability);
        }
        let last = *presses.last().unwrap();
        while t.sim().tick() < last + 2 * T + 4 {
            t.step();
            let types = [1, 2, 3, 4, 5].map(|unit| kind(&t, unit));
            assert!(
                types.iter().all(|&k| k == types[0]),
                "tick {}: {types:?}",
                t.sim().tick()
            );
        }
        let (expected, mode) = match presses.len() % 2 {
            1 => (SHIELDED, ShieldMode::Raised),
            _ => (PK, ShieldMode::Lowered),
        };
        for unit in 1..=5 {
            assert_eq!(
                (kind(&t, unit), shield(&t, unit)),
                (expected, mode),
                "#{unit}, {presses:?}"
            );
            assert_eq!(current(&t, unit), Some(OrderKind::Attack(ObjectId(6))), "#{unit}");
        }
    }
}

/// Design change 0202's Verification: "Press R on shielded Peacekeepers, then right-click an enemy at once. They
/// finish lowering (6 frames) and then attack, with no second click needed." Lowered in tick 5, the three are
/// Peacekeepers from the press, whose default command against player 2's Peacekeeper 4 is an Attack (`Sim::
/// default_order`, the right-click's rule; design change 0202: "an explicit Attack or a right-click on an enemy is
/// **accepted and held**"), and the gate takes an explicit Attack too (the panel's Attack, which "must not silently
/// skip the unit"). Ordered in tick 6, units 1 and 2 by the right-click and unit 3 by the Attack, they hold their
/// fire until their shields are down at the end of tick 5 + T − 1, then shoot, their orders as given.
#[test]
fn lowered_then_ordered_to_attack_at_once_they_attack_once_the_shield_is_down() {
    let mut t = spawned(
        &[
            (SHIELDED, P1, Vec2::new(300.0, 200.0)), // 1
            (SHIELDED, P1, Vec2::new(300.0, 260.0)), // 2
            (SHIELDED, P1, Vec2::new(300.0, 320.0)), // 3
            (SHIELDED, P2, Vec2::new(480.0, 260.0)), // 4, in range, its shield up: it never fires back
        ],
        false,
    );
    t.ability_at(5, P1, &ids(&[1, 2, 3]), LOWER).run_until(5);
    let enemy = ObjectId(4);
    let right_click = |t: &TestSim, unit: u32| t.sim().default_order(ObjectId(unit), Target::Object(enemy));
    assert_eq!(shield(&t, 1), ShieldMode::Lowering { left: T - 1 });
    assert_eq!(
        [right_click(&t, 1), right_click(&t, 2)],
        [Some(OrderKind::Attack(enemy)); 2]
    );
    assert_eq!(t.sim().validate(&attack_order(&[1, 2, 3], 4, false)), Ok(()));
    t.at(6, P1, attack_order(&[1, 2], 4, false).command)
        .at(6, P1, attack_order(&[3], 4, false).command)
        .run_until(6);
    for unit in 1..=3 {
        assert_eq!(current(&t, unit), Some(OrderKind::Attack(enemy)), "#{unit}: accepted");
    }
    let down = 5 + T - 1;
    t.run_until(down - 1);
    assert_eq!(shield(&t, 1), ShieldMode::Lowering { left: 1 });
    t.run_until(down + 10);
    for unit in 1..=3 {
        let fired = attacks_by(&t, unit);
        assert!(!fired.is_empty(), "#{unit} attacks with no second click");
        assert!(fired[0] > down, "#{unit} only once its shield is down: {fired:?}");
        assert_eq!(current(&t, unit), Some(OrderKind::Attack(enemy)), "#{unit}");
    }
}

/// Design change 0202: "While raising (already Shielded): no Attack; a right-click follows, as with the shield up."
/// Raised in tick 5, the Peacekeeper takes no Attack order (`Reject::CannotAttack`), and its default command against
/// player 2's Peacekeeper is a Move to it, a follow, which it carries out while its shield goes up, never firing.
#[test]
fn raising_takes_no_attack_and_a_right_click_follows() {
    let mut t = spawned(
        &[
            (PK, P1, Vec2::new(200.0, 300.0)),       // 1
            (SHIELDED, P2, Vec2::new(480.0, 300.0)), // 2, in sight, out of range
        ],
        false,
    );
    t.ability_at(5, P1, &ids(&[1]), RAISE).run_until(5);
    let enemy = ObjectId(2);
    assert_eq!(shield(&t, 1), ShieldMode::Raising { left: T - 1 });
    assert_eq!(
        t.sim().validate(&attack_order(&[1], 2, false)),
        Err(Reject::CannotAttack)
    );
    let follow = OrderKind::Move(Target::Object(enemy));
    assert_eq!(t.sim().default_order(ObjectId(1), Target::Object(enemy)), Some(follow));
    let start = t.pos(1);
    t.order_at(6, P1, &ids(&[1]), follow, false).run_until(40);
    assert_eq!(current(&t, 1), Some(follow));
    assert!(t.pos(1).x > start.x + 40.0, "it follows: {}", t.pos(1));
    assert_eq!(attacks_by(&t, 1), Vec::<Tick>::new());
}
