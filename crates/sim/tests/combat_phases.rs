//! Attack sequences (`systems/combat.md > AttackPhase`, `ARCHITECTURE.md > Combat`): each phase lasts exactly its
//! duration in ticks, the unit faces its target before it fires, and only Aiming and Reloading can be interrupted.
//! Attack orders without the approach of slice E: a unit attacks a target in range and stands still otherwise.

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
/// both spawned in tick 1; the attacker is ordered to attack in tick 2.
fn engage(attacker: ObjectKind, target: ObjectKind, target_at: Vec2) -> TestSim {
    let mut t = ScenarioBuilder::new(MAP).start();
    t.spawn_unit_at(1, attacker, P1, Vec2::new(200.0, 250.0))
        .spawn_unit_at(1, target, P2, target_at)
        .order_at(2, P1, &[ATTACKER], OrderKind::Attack(TARGET), false);
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

/// `factions/gdo/units.md > Peacekeeper`: Aim 2, Firing 1, Cooldown 2, Reload 12, a period of 17 ticks. Ordered in
/// tick 2, it aims in ticks 2–3, fires in 4 (the hit lands at its end), cools down in 5–6 and reloads in 7–18; the
/// next sequence starts in 19.
#[test]
fn peacekeeper_attacks_every_17_ticks() {
    let mut t = engage(ObjectKind::Peacekeeper, ObjectKind::Guard, Vec2::new(350.0, 250.0));
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

/// `factions/syndicate/units.md > Guard`: Aim 2, Firing 1, Cooldown 1, Reload 4, a period of 8 ticks.
#[test]
fn guard_attacks_every_8_ticks() {
    let mut t = engage(ObjectKind::Guard, ObjectKind::Peacekeeper, Vec2::new(350.0, 250.0));
    t.run_until(36);
    assert_eq!(attack_ticks(&t, ATTACKER), [4, 12, 20, 28, 36]);
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
/// fired.
#[test]
fn a_new_order_interrupts_aiming() {
    let mut t = engage(ObjectKind::Peacekeeper, ObjectKind::Guard, Vec2::new(350.0, 250.0));
    t.order_at(3, P1, &[ATTACKER], OrderKind::Stop, false);
    t.run_until(40);
    assert_eq!(attack_ticks(&t, ATTACKER), [] as [Tick; 0]);
    assert_eq!(phase(&t, ATTACKER), (None, 0));
    assert!(t.sim().object(ATTACKER).unwrap().orders.as_ref().unwrap().is_idle());
}

/// `systems/combat.md > Firing`, `> Cooldown`: neither can be interrupted. A Stop given in the tick the unit fires
/// doesn't stop the shot, nor the cooldown after it; the unit can't move until it reloads (`> UnitBaseSource`): the
/// movement step holds it with its goal. But the attack is disengaged: no further sequence follows.
#[test]
fn firing_and_cooldown_run_on_through_a_new_order() {
    let mut t = engage(ObjectKind::Peacekeeper, ObjectKind::Guard, Vec2::new(350.0, 250.0));
    t.order_at(4, P1, &[ATTACKER], OrderKind::Stop, false);
    t.order_at(
        5,
        P1,
        &[ATTACKER],
        OrderKind::Move(Target::Point(Vec2::new(200.0, 400.0))),
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
    assert_eq!(mover.goal, MoveGoal::Point(Vec2::new(200.0, 400.0)), "the move waits");
    t.run_until(7);
    assert!(pos(&t).y > start.y, "it leaves once it reloads: {}", pos(&t));
    t.run_until(60);
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
/// Until slice E the unit doesn't follow (TODO(E): chase): the order waits, and the attack resumes when the target
/// is back in range.
#[test]
fn a_target_leaving_range_cancels_the_aim_and_the_order_waits() {
    let mut t = engage(ObjectKind::Peacekeeper, ObjectKind::Guard, Vec2::new(350.0, 250.0));
    let away = |pos| DebugCommand::SetPosition { id: TARGET, pos };
    t.debug_at(3, away(Vec2::new(900.0, 250.0)));
    t.debug_at(30, away(Vec2::new(350.0, 250.0)));
    t.run_until(29);
    assert_eq!(attack_ticks(&t, ATTACKER), [] as [Tick; 0]);
    assert_eq!(phase(&t, ATTACKER), (None, 0));
    let unit = t.sim().object(ATTACKER).unwrap();
    assert_eq!(unit.pos, Vec2::new(200.0, 250.0), "it stands");
    assert_eq!(
        unit.orders.as_ref().unwrap().current.as_ref().map(|a| a.order.kind),
        Some(OrderKind::Attack(TARGET)),
        "the order waits"
    );
    t.run_until(40);
    assert_eq!(attack_ticks(&t, ATTACKER), [32]);
}

/// Out of range from the start: the unit stands and waits (TODO(E): chase); placed in range, it attacks.
#[test]
fn out_of_range_the_unit_stands_in_range_it_attacks() {
    let mut far = engage(
        ObjectKind::Peacekeeper,
        ObjectKind::Guard,
        Vec2::new(200.0 + gu(6.0), 250.0),
    );
    far.run_until(50);
    assert_eq!(attack_ticks(&far, ATTACKER), [] as [Tick; 0]);
    let unit = far.sim().object(ATTACKER).unwrap();
    assert_eq!(
        (unit.pos, unit.prev_pos),
        (Vec2::new(200.0, 250.0), Vec2::new(200.0, 250.0))
    );
    assert!(!unit.orders.as_ref().unwrap().is_idle(), "the order waits");

    let mut near = engage(
        ObjectKind::Peacekeeper,
        ObjectKind::Guard,
        Vec2::new(200.0 + gu(3.0), 250.0),
    );
    near.run_until(50);
    assert_eq!(attack_ticks(&near, ATTACKER), [4, 21, 38]);
}

/// `systems/combat.md > ValidTarget`: a unit can't attack itself; the order completes at once.
#[test]
fn a_unit_ordered_to_attack_itself_drops_the_order() {
    let mut t = engage(ObjectKind::Peacekeeper, ObjectKind::Guard, Vec2::new(350.0, 250.0));
    t.order_at(3, P1, &[ATTACKER], OrderKind::Attack(ATTACKER), false);
    t.run_until(10);
    let unit = t.sim().object(ATTACKER).unwrap();
    assert!(unit.orders.as_ref().unwrap().is_idle());
    assert_eq!(unit.attack.as_ref().unwrap().target, None);
    assert_eq!(
        attack_ticks(&t, ATTACKER),
        [] as [Tick; 0],
        "the aim at the Guard was interrupted"
    );
}
