//! Attack sequences (`systems/combat.md > AttackPhase`, `ARCHITECTURE.md > Combat`): each phase lasts exactly its
//! duration in ticks, the unit faces its target before it fires, and only Aiming and Reloading can be interrupted.
//!
//! The target holds its position (`systems/controls/behaviors.md > HoldingPosition`), so it never moves; it may fire
//! back, which changes nothing here as long as the attacker lives (a test that runs longer than a Guard target takes
//! to kill the attacker keeps it beyond the Guard's Range). A unit left without an order looks for a target by itself
//! (`systems/controls/behaviors.md > BaseAutoTargeting`), so the tests that need the attacker to stay disengaged send
//! it away with a Move.

use sc_sim::combat::{self, AttackPhase};
use sc_sim::testing::{ScenarioBuilder, TestSim};
use sc_sim::{
    DebugCommand, MoveGoal, ObjectId, ObjectKind, OrderKind, PlayerId, SimEvent, Target, Tick, Vec2, gu, math,
};

const P1: PlayerId = PlayerId(1);
const P2: PlayerId = PlayerId(2);
const ATTACKER: ObjectId = ObjectId(1);
const TARGET: ObjectId = ObjectId(2);

/// 16 × 8 tiles of open ground (1024 × 512 su).
const MAP: &str = "
    ................
    ................
    ................
    ................
    ................
    ................
    ................
    ................
";

/// An `attacker` of player 1 (id 1) at (200, 250), facing east, and a `target` of player 2 (id 2) at `target_at`,
/// both spawned in tick 1; in tick 2 the attacker is ordered to attack and the target to hold its position.
fn engage(attacker: ObjectKind, target: ObjectKind, target_at: Vec2) -> TestSim {
    let mut t = ScenarioBuilder::new(MAP).start();
    t.spawn_unit_at(1, attacker, P1, Vec2::new(200.0, 250.0))
        .spawn_unit_at(1, target, P2, target_at)
        .order_at(2, P1, &[ATTACKER], OrderKind::Attack(TARGET), false)
        .order_at(2, P2, &[TARGET], OrderKind::Hold, false);
    t
}

/// The ticks in which `attacker`'s attacks took effect.
fn attack_ticks(t: &TestSim, attacker: ObjectId) -> Vec<Tick> {
    t.events()
        .iter()
        .filter(|(_, e)| matches!(e, SimEvent::Attacked { attacker: a, .. } if *a == attacker))
        .map(|&(tick, _)| tick)
        .collect()
}

fn phase(t: &TestSim, id: ObjectId) -> (Option<AttackPhase>, u32) {
    let state = t
        .sim()
        .object(id)
        .and_then(|o| o.attack.clone())
        .expect("a unit that attacks");
    (state.phase, state.elapsed)
}

fn may_move(t: &TestSim, id: ObjectId) -> bool {
    combat::may_move(t.sim().object(id).expect("exists"))
}

/// `factions/gdo/units/peacekeeper.md`: Aim 2, Firing 1, Cooldown 2, Reload 12, a period of 17 ticks. Ordered in
/// tick 2, it aims in ticks 2–3, fires in 4 (the hit lands at its end), cools down in 5–6 and reloads in 7–18; the
/// next sequence starts in 19. The Guard holds 200 su off, beyond its own Range (3 grid units): its hits of 17 would
/// kill the Peacekeeper before its fourth shot.
#[test]
fn peacekeeper_attacks_every_17_ticks() {
    let mut t = engage(ObjectKind::Peacekeeper, ObjectKind::Guard, Vec2::new(430.0, 250.0));
    let expected = [
        (2, Some(AttackPhase::Aiming { target: TARGET }), 1),
        (3, Some(AttackPhase::Firing { target: Some(TARGET) }), 0),
        (4, Some(AttackPhase::Cooldown), 0),
        (5, Some(AttackPhase::Cooldown), 1),
        (6, Some(AttackPhase::Reloading), 0),
        (17, Some(AttackPhase::Reloading), 11),
        (18, None, 0),
        (19, Some(AttackPhase::Aiming { target: TARGET }), 1),
    ];
    for (tick, phase_after, elapsed) in expected {
        t.run_until(tick);
        assert_eq!(phase(&t, ATTACKER), (phase_after, elapsed), "after tick {tick}");
    }
    t.run_until(60);
    assert_eq!(attack_ticks(&t, ATTACKER), [4, 21, 38, 55]);
}

/// `factions/syndicate/units/guard.md`, its laser rifle (design change 0186): Aim 4, Firing 1, Cooldown 3, Reload
/// 16, a period of 24 ticks. Ordered in tick 2, it aims in ticks 2–5, fires in 6, cools down in 7–9 and reloads in
/// 10–25; the next sequence starts in 26. Its third hit of 17 kills the Peacekeeper (50 HP, PointArmor 1).
#[test]
fn guard_attacks_every_24_ticks() {
    let mut t = engage(ObjectKind::Guard, ObjectKind::Peacekeeper, Vec2::new(350.0, 250.0));
    let expected = [
        (2, Some(AttackPhase::Aiming { target: TARGET }), 1),
        (4, Some(AttackPhase::Aiming { target: TARGET }), 3),
        (5, Some(AttackPhase::Firing { target: Some(TARGET) }), 0),
        (6, Some(AttackPhase::Cooldown), 0),
        (8, Some(AttackPhase::Cooldown), 2),
        (9, Some(AttackPhase::Reloading), 0),
        (24, Some(AttackPhase::Reloading), 15),
        (25, None, 0),
        (26, Some(AttackPhase::Aiming { target: TARGET }), 1),
    ];
    for (tick, phase_after, elapsed) in expected {
        t.run_until(tick);
        assert_eq!(phase(&t, ATTACKER), (phase_after, elapsed), "after tick {tick}");
    }
    t.run_until(60);
    assert_eq!(attack_ticks(&t, ATTACKER), [6, 30, 54]);
    assert!(t.sim().object(TARGET).is_none(), "dead");
}

/// `systems/controls/behaviors.md > AttackingObject` 2: a unit attacking from its base turns to face its target
/// while it aims (at its TurnRate, 180° per tick for infantry), and fires facing it.
#[test]
fn the_unit_turns_to_face_its_target_while_aiming() {
    // Due west of a unit facing east: half a turn.
    let mut t = engage(ObjectKind::Peacekeeper, ObjectKind::Guard, Vec2::new(60.0, 250.0));
    t.run_until(2);
    let unit = t.sim().object(ATTACKER).unwrap();
    assert_eq!((unit.prev_facing, unit.facing), (0.0, math::PI), "one tick's turn");
    t.run_until(4);
    assert_eq!(attack_ticks(&t, ATTACKER), [4]);
    // North-east: it turns back and fires facing it.
    t.debug_at(
        10,
        DebugCommand::SetPosition {
            id: TARGET,
            pos: Vec2::new(300.0, 150.0),
        },
    );
    t.run_until(21);
    let facing = t.sim().object(ATTACKER).unwrap().facing;
    assert!(
        (facing - math::vector_angle(Vec2::new(100.0, -100.0))).abs() < 1e-6,
        "{facing}"
    );
    assert_eq!(attack_ticks(&t, ATTACKER), [4, 21]);
}

/// `systems/combat.md > Aiming > Interruptible - true`: another command cancels the attack before anything is
/// fired. A Move takes the unit away without a shot (`systems/controls/behaviors.md > BaseAutoTargeting > Not active
/// during`: Move). After a Stop the unit has no order, acquires the enemy in sight by itself in the same tick, and
/// aims afresh: its shot lands a tick later than the one the Stop interrupted.
#[test]
fn a_new_order_interrupts_aiming() {
    let mut t = engage(ObjectKind::Peacekeeper, ObjectKind::Guard, Vec2::new(350.0, 250.0));
    let away = OrderKind::Move(Target::Point(Vec2::new(200.0, 480.0)));
    t.order_at(3, P1, &[ATTACKER], away, false);
    t.run_until(3);
    assert_eq!(phase(&t, ATTACKER), (None, 0));
    t.run_until(40);
    assert_eq!(attack_ticks(&t, ATTACKER), [] as [Tick; 0]);
    assert_eq!(t.sim().object(ATTACKER).unwrap().attack.as_ref().unwrap().target, None);

    let mut t = engage(ObjectKind::Peacekeeper, ObjectKind::Guard, Vec2::new(350.0, 250.0));
    t.order_at(3, P1, &[ATTACKER], OrderKind::Stop, false);
    t.run_until(3);
    assert!(t.sim().object(ATTACKER).unwrap().orders.as_ref().unwrap().is_idle());
    assert_eq!(
        phase(&t, ATTACKER),
        (Some(AttackPhase::Aiming { target: TARGET }), 1),
        "a new aim, begun in the tick of the Stop"
    );
    t.run_until(10);
    assert_eq!(attack_ticks(&t, ATTACKER), [5]);
}

/// `systems/combat.md > Firing`, `systems/combat.md > Cooldown`: neither can be interrupted. A Stop given in the tick
/// the unit fires doesn't stop the shot, nor the cooldown after it; the unit can't move until it reloads
/// (`systems/combat.md > UnitBaseSource`): the movement step holds it with its goal. But the attack is disengaged: no
/// further sequence follows while the unit walks away.
#[test]
fn firing_and_cooldown_run_on_through_a_new_order() {
    let mut t = engage(ObjectKind::Peacekeeper, ObjectKind::Guard, Vec2::new(350.0, 250.0));
    t.order_at(4, P1, &[ATTACKER], OrderKind::Stop, false);
    t.order_at(
        5,
        P1,
        &[ATTACKER],
        OrderKind::Move(Target::Point(Vec2::new(900.0, 450.0))),
        false,
    );
    t.run_until(2);
    assert!(!may_move(&t, ATTACKER), "aiming");
    t.run_until(3);
    assert_eq!(
        phase(&t, ATTACKER).0,
        Some(AttackPhase::Firing { target: Some(TARGET) })
    );
    assert!(!may_move(&t, ATTACKER), "firing next");
    t.run_until(4);
    assert_eq!(attack_ticks(&t, ATTACKER), [4], "the shot lands");
    assert_eq!(phase(&t, ATTACKER), (Some(AttackPhase::Cooldown), 0));
    assert!(!may_move(&t, ATTACKER) && !combat::may_turn(t.sim().object(ATTACKER).unwrap()));
    let start = Vec2::new(200.0, 250.0);
    let pos = |t: &TestSim| t.sim().object(ATTACKER).unwrap().pos;
    t.run_until(5);
    assert_eq!(
        phase(&t, ATTACKER),
        (Some(AttackPhase::Cooldown), 1),
        "the move order doesn't cut it short"
    );
    assert_eq!(pos(&t), start, "held in place");
    t.run_until(6);
    assert_eq!(phase(&t, ATTACKER), (Some(AttackPhase::Reloading), 0));
    assert!(may_move(&t, ATTACKER), "free while reloading");
    assert_eq!(pos(&t), start, "held in place");
    let mover = t.sim().object(ATTACKER).unwrap().mover.as_ref().unwrap();
    assert_eq!(mover.goal, MoveGoal::Point(Vec2::new(900.0, 450.0)), "the move waits");
    t.run_until(7);
    assert!(pos(&t).x > start.x, "it leaves once it reloads: {}", pos(&t));
    // Three periods: no sequence followed. (Not much longer: the Guard on Hold, which it walks past, shoots it with
    // its hits of 17, design change 0186.)
    t.run_until(53);
    assert_eq!(attack_ticks(&t, ATTACKER), [4], "disengaged");
    assert_eq!(t.sim().object(ATTACKER).unwrap().attack.as_ref().unwrap().target, None);
}

/// `systems/combat.md > Reloading > Interruptible - true`: the unit is free to do other things, but its weapon isn't
/// ready any sooner: attacking again after a Stop fires when the reload is over, as if nothing had happened.
#[test]
fn an_interrupted_reload_keeps_counting() {
    let mut t = engage(ObjectKind::Peacekeeper, ObjectKind::Guard, Vec2::new(350.0, 250.0));
    t.order_at(8, P1, &[ATTACKER], OrderKind::Stop, false);
    t.order_at(10, P1, &[ATTACKER], OrderKind::Attack(TARGET), false);
    t.run_until(9);
    assert_eq!(phase(&t, ATTACKER), (Some(AttackPhase::Reloading), 3));
    t.run_until(40);
    assert_eq!(attack_ticks(&t, ATTACKER), [4, 21, 38]);
}

/// `systems/combat.md > Aiming`: the target must stay valid throughout; one that leaves the range cancels the aim.
/// The unit goes after it (`systems/controls/behaviors.md > AttackingObject` 4) and attacks again once the gap
/// between their edges is back within its Range, 4 grid units.
#[test]
fn a_target_leaving_range_cancels_the_aim_and_the_unit_goes_after_it() {
    let mut t = engage(ObjectKind::Peacekeeper, ObjectKind::Guard, Vec2::new(350.0, 250.0));
    // A gap of 280 su: beyond its Range (256), still in its owner's sight.
    let away = Vec2::new(200.0 + 12.0 + 280.0 + 18.0, 250.0);
    t.debug_at(3, DebugCommand::SetPosition { id: TARGET, pos: away });
    t.run_until(3);
    assert_eq!(phase(&t, ATTACKER), (None, 0), "the aim is cancelled");
    assert_eq!(t.sim().object(ATTACKER).unwrap().pos, Vec2::new(200.0, 250.0));
    t.run_until(40);
    let unit = t.sim().object(ATTACKER).unwrap();
    // 4 su a tick from tick 4 on (in tick 3 it was still aiming): after 6 steps, in tick 9, the gap is 256, and
    // it aims at once.
    assert_eq!(unit.pos, Vec2::new(224.0, 250.0), "it stops as soon as it is in range");
    assert_eq!(
        unit.orders.as_ref().unwrap().current.as_ref().map(|a| a.order.kind),
        Some(OrderKind::Attack(TARGET)),
    );
    assert_eq!(attack_ticks(&t, ATTACKER), [11, 28]);
}

/// Out of range from the start (`systems/controls/behaviors.md > AttackingObject` 1): the unit approaches until the
/// target is in range, then attacks; in range from the start, it attacks from where it stands.
#[test]
fn out_of_range_the_unit_closes_in_and_in_range_it_attacks() {
    let mut far = engage(
        ObjectKind::Peacekeeper,
        ObjectKind::Guard,
        Vec2::new(200.0 + 12.0 + 270.0 + 18.0, 250.0),
    );
    far.run_until(50);
    let unit = far.sim().object(ATTACKER).unwrap();
    assert_eq!(unit.pos, Vec2::new(216.0, 250.0), "four steps close the gap to 254 su");
    // It moves in ticks 2 to 5 and aims in the tick it comes into range.
    assert_eq!(attack_ticks(&far, ATTACKER), [7, 24, 41]);
    assert!(!unit.orders.as_ref().unwrap().is_idle());

    let mut near = engage(
        ObjectKind::Peacekeeper,
        ObjectKind::Guard,
        Vec2::new(200.0 + gu(3.0), 250.0),
    );
    near.run_until(50);
    assert_eq!(attack_ticks(&near, ATTACKER), [4, 21, 38]);
    assert_eq!(near.sim().object(ATTACKER).unwrap().pos, Vec2::new(200.0, 250.0));
}

/// `systems/combat.md > ValidTarget`: a unit can't attack itself; the order completes at once.
#[test]
fn a_unit_ordered_to_attack_itself_drops_the_order() {
    let mut t = engage(ObjectKind::Peacekeeper, ObjectKind::Guard, Vec2::new(350.0, 250.0));
    t.order_at(3, P1, &[ATTACKER], OrderKind::Attack(ATTACKER), false);
    t.run_until(3);
    assert_eq!(phase(&t, ATTACKER), (None, 0), "the aim at the Guard was interrupted");
    t.run_until(4);
    let unit = t.sim().object(ATTACKER).unwrap();
    assert!(unit.orders.as_ref().unwrap().is_idle());
    // Without an order it picks the Guard again by itself, and its shot lands two ticks late.
    t.run_until(10);
    assert_eq!(attack_ticks(&t, ATTACKER), [6]);
}
