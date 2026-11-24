//! The two unit types and the toggle's window (design changes 0171 to 0174, which answer question Q0020;
//! `factions/gdo/units.md > Peacekeeper > Riot Shield > Two unit types`, `> During the toggle`, `> Toggle time`):
//! the type switches when the toggle starts; while the shield goes up or down the unit is slow and doesn't attack,
//! and only a shield all the way up protects; a press mid-way turns back from there; a raise pressed mid-attack
//! waits for the shot; the Barracks trains the Peacekeeper, and both types cost the same Unit Control.

use sc_sim::testing::ScenarioBuilder;
use sc_sim::{DebugCommand, ProductionCommand, SpawnAt, TilePos};

use super::*;

/// The `KindChanged` events of the game: `(tick, unit, from, to)`.
fn kind_changes(t: &TestSim) -> Vec<(Tick, u32, ObjectKind, ObjectKind)> {
    t.events()
        .iter()
        .filter_map(|(tick, e)| match e {
            SimEvent::KindChanged { id, from, to } => Some((*tick, id.0, *from, *to)),
            _ => None,
        })
        .collect()
}

/// Design change 0173 (`> Two unit types`): "Raising or lowering the shield switches the unit's type", "**when the
/// toggle starts**". A walking Peacekeeper raised in tick 3 is a Peacekeeper (Shielded) from that tick on, its shield
/// on its way up; lowered in tick 12, a Peacekeeper again from that tick, its shield on its way down. It stays the
/// same unit: its id, HP and orders, and its walk goes on: 4 su in tick 2, 2 a tick from the raise to the end of the
/// lowering (ticks 3 to 12 + T − 2), and 4 again from then on. "Cost and Unit Control are the same for both types":
/// player 1's Unit Control stays 1 all along.
#[test]
fn the_type_switches_when_the_toggle_starts() {
    let mut t = spawned(&[(PK, P1, Vec2::new(100.0, 300.0))], false);
    let to = OrderKind::Move(Target::Point(Vec2::new(900.0, 300.0)));
    t.order_at(2, P1, &ids(&[1]), to, false)
        .ability_at(3, P1, &ids(&[1]), RAISE)
        .ability_at(12, P1, &ids(&[1]), LOWER)
        .run_until(2);
    let hp = t.sim().object(ObjectId(1)).unwrap().hp;
    let mut seen = Vec::new();
    for tick in 3..=20 {
        t.run_until(tick);
        let unit = t.sim().object(ObjectId(1)).unwrap();
        let current = unit
            .orders
            .as_ref()
            .unwrap()
            .current
            .as_ref()
            .map(|active| active.order.kind);
        assert_eq!((unit.hp, current), (hp, Some(to)), "tick {tick}");
        assert_eq!(t.sim().unit_control(P1), 1, "tick {tick}");
        seen.push((tick, kind(&t, 1), shield(&t, 1)));
    }
    assert_eq!(
        seen[0],
        (3, SHIELDED, ShieldMode::Raising { left: T - 1 }),
        "the tick of the raise"
    );
    assert_eq!(seen[8], (11, SHIELDED, ShieldMode::Raised));
    assert_eq!(
        seen[9],
        (12, PK, ShieldMode::Lowering { left: T - 1 }),
        "the tick of the lowering"
    );
    assert_eq!(
        kind_changes(&t),
        [(3, 1, PK, SHIELDED), (12, 1, SHIELDED, PK)],
        "once each way, as each toggle starts"
    );
    let slow = 12 + T - 2 - 3 + 1;
    let walked = 4.0 + 2.0 * slow as f32 + 4.0 * (20 - (12 + T - 2)) as f32;
    assert_eq!(t.pos(1), Vec2::new(100.0 + walked, 300.0), "it walked on");
}

/// Design change 0171 (`> During the toggle`): "While raising **or** lowering, the Peacekeeper moves at half speed
/// (as with the shield up) and **cannot attack**." A walking Peacekeeper raised in tick 3 and lowered in tick 12
/// covers 2 su a tick from tick 3 to the end of the lowering, tick 12 + T − 1, and 4 from then on; and all that
/// while, the gate takes no Attack order for it (`Reject::CannotAttack`), which it takes once the shield is down.
#[test]
fn while_the_shield_goes_up_or_down_the_unit_is_slow_and_does_not_attack() {
    let mut t = spawned(
        &[
            (PK, P1, Vec2::new(100.0, 300.0)), // 1
            (PK, P2, Vec2::new(250.0, 420.0)), // 2, in sight
        ],
        false,
    );
    let to = OrderKind::Move(Target::Point(Vec2::new(900.0, 300.0)));
    t.order_at(2, P1, &ids(&[1]), to, false)
        .ability_at(3, P1, &ids(&[1]), RAISE)
        .ability_at(12, P1, &ids(&[1]), LOWER)
        .run_until(2);
    let down = 12 + T - 1;
    for tick in 3..=down + 3 {
        let before = t.pos(1);
        t.run_until(tick);
        let attack = t.sim().validate(&attack_order(&[1], 2, false));
        if tick <= down - 1 {
            assert_ne!(shield(&t, 1), ShieldMode::Lowered, "tick {tick}");
        }
        let expected = if tick < down {
            (2.0, Err(Reject::CannotAttack))
        } else {
            (4.0, Ok(()))
        };
        assert_eq!((t.pos(1).distance(before), attack), expected, "tick {tick}");
    }
}

/// Design change 0171, Verification: "Raise the shield and get hit mid-raise: full damage." And mid-lowering: the 50%
/// reduction applies "**only once the shield is fully up**". Ordered in tick 10, the attackers hit in ticks 12 and 29
/// (Damage 10 on PointArmor 1: 9 in full, 4 behind a raised shield). Unit 1, raised in tick 11, is on its way up in
/// tick 12 (9) and up in tick 29 (4); unit 3, raised in tick 2 and lowered in tick 28, is up in tick 12 (4) and on
/// its way down in tick 29 (9).
#[test]
fn a_hit_mid_toggle_takes_full_damage() {
    let mut t = spawned(
        &[
            (PK, P1, Vec2::new(300.0, 300.0)), // 1
            (PK, P2, Vec2::new(300.0, 450.0)), // 2
            (PK, P1, Vec2::new(700.0, 300.0)), // 3
            (PK, P2, Vec2::new(700.0, 450.0)), // 4
        ],
        false,
    );
    t.ability_at(11, P1, &ids(&[1]), RAISE)
        .ability_at(2, P1, &ids(&[3]), RAISE)
        .ability_at(28, P1, &ids(&[3]), LOWER);
    for (attacker, target) in [(2, 1), (4, 3)] {
        t.at(10, P2, attack_order(&[attacker], target, false).command);
    }
    t.run_until(30);
    assert_eq!(hits_on(&t, 1), [(12, 9.0), (29, 4.0)], "mid-raise, then up");
    assert_eq!(hits_on(&t, 3), [(12, 4.0), (29, 9.0)], "up, then mid-lowering");
}

/// Design change 0171, Verification: "Press again halfway: lowering takes half the toggle time." Raised in tick 3,
/// the shield is half-way up at the end of tick 2 + T/2; lowered in the next tick, it is down at the end of tick
/// 3 + T/2 + T/2 − 1, after T/2 ticks. Raised again half-way down, it turns back up from there too.
#[test]
fn a_press_mid_way_turns_back_from_where_the_shield_is() {
    const HALF: Tick = T / 2;
    const { assert!(HALF * 2 == T) };
    let mut t = spawned(&[(PK, P1, Vec2::new(300.0, 300.0))], false);
    let lowered = 3 + HALF;
    t.ability_at(3, P1, &ids(&[1]), RAISE)
        .ability_at(lowered, P1, &ids(&[1]), LOWER)
        .run_until(lowered - 1);
    assert_eq!(shield(&t, 1), ShieldMode::Raising { left: HALF }, "half-way up");
    let mut down = lowered;
    while shield(t.step(), 1) != ShieldMode::Lowered {
        down += 1;
    }
    assert_eq!(down, lowered + HALF - 1, "down after T/2 ticks");

    // Up all the way, then down half-way, then up again: T/2 ticks to the top.
    t.ability_at(20, P1, &ids(&[1]), RAISE)
        .ability_at(30, P1, &ids(&[1]), LOWER)
        .ability_at(30 + HALF, P1, &ids(&[1]), RAISE)
        .run_until(30 + HALF - 1);
    assert_eq!(shield(&t, 1), ShieldMode::Lowering { left: HALF }, "half-way down");
    t.run_until(30 + HALF + HALF - 2);
    assert_eq!(shield(&t, 1), ShieldMode::Raising { left: 1 });
    assert_eq!(shield(t.step(), 1), ShieldMode::Raised, "up after T/2 ticks");
}

/// Design change 0172, Verification: "Peacekeepers fighting, press the shield key: every one raises its shield, some
/// a few frames after the others." Five Peacekeepers, ordered to attack one tick apart (unit `n` in tick `n + 1`),
/// aim for 2 ticks, fire in the next (Aim 2, Firing 1: unit `n` in tick `n + 3`), then cool down for 2 ticks. In tick
/// 7 all five are given the raise. Unit 1, its Cooldown over (ticks 5 and 6), and unit 5, still aiming (an aim can be
/// interrupted), raise it at once; units 2 to 4, in Cooldown or Firing (`combat::mid_attack`), finish their shot
/// first, and become Peacekeepers (Shielded) in the first tick after their Cooldown: units 2 and 3 in ticks 8 and 9,
/// and unit 4, whose aim was done and whose shot goes out in tick 7, in tick 10. Each shot fired lands, unit 5's aim
/// is cancelled, and nobody fires once shielded.
#[test]
fn fighting_peacekeepers_all_raise_some_a_few_frames_later() {
    let mut t = spawned(
        &[
            (PK, P1, Vec2::new(300.0, 200.0)),       // 1
            (PK, P1, Vec2::new(300.0, 260.0)),       // 2
            (PK, P1, Vec2::new(300.0, 320.0)),       // 3
            (PK, P1, Vec2::new(300.0, 380.0)),       // 4
            (PK, P1, Vec2::new(300.0, 440.0)),       // 5
            (SHIELDED, P2, Vec2::new(500.0, 320.0)), // 6, its shield up: it never fires back
        ],
        false,
    );
    for unit in 1..=5 {
        t.at(unit + 1, P1, attack_order(&[unit], 6, false).command);
    }
    t.ability_at(7, P1, &ids(&[1, 2, 3, 4, 5]), RAISE).run_until(40);
    let raised: Vec<(Tick, u32)> = kind_changes(&t).iter().map(|&(tick, id, _, _)| (tick, id)).collect();
    assert_eq!(raised, [(7, 1), (7, 5), (8, 2), (9, 3), (10, 4)]);
    let fired: Vec<(Tick, u32)> = t
        .events()
        .iter()
        .filter_map(|(tick, e)| match e {
            SimEvent::Attacked { attacker, .. } => Some((*tick, attacker.0)),
            _ => None,
        })
        .collect();
    assert_eq!(
        fired,
        [(4, 1), (5, 2), (6, 3), (7, 4)],
        "the shots under way, and no other"
    );
    assert_eq!(
        hits_on(&t, 6),
        [(4, 4.0), (5, 4.0), (6, 4.0), (7, 4.0)],
        "each one lands"
    );
    for unit in 1..=5 {
        assert_eq!(
            (kind(&t, unit), shield(&t, unit)),
            (SHIELDED, ShieldMode::Raised),
            "#{unit}"
        );
    }
}

/// Design change 0173 (`> Two unit types`): "The Barracks produces the unshielded Peacekeeper. Cost and Unit Control
/// are the same for both types." A Peacekeeper trained at a Barracks comes out a Peacekeeper with its shield down,
/// and the gate refuses to train a Peacekeeper (Shielded) (`Reject::CannotProduce`). Raised, it costs the Unit
/// Control it did.
#[test]
fn the_barracks_trains_the_peacekeeper() {
    let flat = vec![".".repeat(30); 16].join("\n");
    let tile = |x, y| SpawnAt::Tile(TilePos::new(x, y));
    let mut t = ScenarioBuilder::new(&flat)
        .spawn(ObjectKind::DeploymentCenter, Some(P1), tile(1, 1))
        .spawn(ObjectKind::Barracks, Some(P1), tile(8, 4))
        .start();
    let barracks = ObjectId(2);
    let train = |kind| Command::Production {
        objects: vec![barracks],
        command: ProductionCommand::Train(kind),
    };
    t.debug_at(
        1,
        DebugCommand::SetResources {
            player: P1,
            crystals: 500,
            supplies: 0,
            alloys: 0,
            essence: 0,
            flux: 0,
        },
    )
    .at(1, P1, train(PK))
    .run_until(1);
    let refused = PlayerCommand {
        player: P1,
        command: train(SHIELDED),
    };
    assert_eq!(t.sim().validate(&refused), Err(Reject::CannotProduce(SHIELDED)));
    t.run_until(80);
    let trained = t
        .events()
        .iter()
        .find_map(|(_, e)| match e {
            SimEvent::UnitProduced { unit, .. } => Some(unit.0),
            _ => None,
        })
        .expect("a unit trained");
    assert_eq!((kind(&t, trained), shield(&t, trained)), (PK, ShieldMode::Lowered));
    assert_eq!(t.sim().unit_control(P1), 1);
    t.ability_at(81, P1, &ids(&[trained]), RAISE).run_until(81);
    assert_eq!((kind(&t, trained), t.sim().unit_control(P1)), (SHIELDED, 1));
}
