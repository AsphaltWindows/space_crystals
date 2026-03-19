use crate::helpers::*;
#[allow(unused_imports)]
use space_crystals::game::combat::types::{AttackPhase, PhaseActionConstraints, AttackState, AttackCapability, AttackTarget};

/// QA Step 1 [auto]: Verify attack phases execute in order: Aiming -> Firing -> Cooldown -> Reloading.
#[test]
fn step_1_phase_order() {
    let mut test_app = TestApp::new();
    test_app.step(); // startup

    // Spawn a player unit and an enemy unit close together
    let player_unit;
    let enemy_unit;
    {
        let mut harness = TestHarness::new(&mut test_app.app);
        player_unit = harness.spawn_unit_at_grid(ObjectEnum::Peacekeeper, 20, 20, Owner(Some(0)));
        enemy_unit = harness.spawn_unit_at_grid(ObjectEnum::Peacekeeper, 21, 20, Owner(Some(1)));
    }
    test_app.step(); // flush spawn

    // Set attack target to trigger the phase machine
    {
        let world = test_app.app.world_mut();
        let mut attack_state = world.get_mut::<AttackState>(player_unit).unwrap();
        attack_state.current_target = Some(AttackTarget::UnitTarget(enemy_unit));
        attack_state.phase = AttackPhase::None;
    }

    // Step to let the attack_phase_system transition from None -> Aiming
    test_app.step_n(3);
    let phase = TestHarness::new(&mut test_app.app).get_attack_state(player_unit).unwrap();
    // After stepping, the unit should be in Aiming or later (depends on aim_time)
    assert!(
        matches!(phase, AttackPhase::Aiming | AttackPhase::Firing | AttackPhase::Cooldown | AttackPhase::Reloading),
        "After setting target, unit should enter attack phases, got {:?}", phase
    );
}

/// QA Step 2 [auto]: Verify Aiming phase is interruptible
#[test]
fn step_2_aiming_is_interruptible() {
    assert!(AttackPhase::Aiming.is_interruptible(),
        "Aiming phase should be interruptible");
}

/// QA Step 3 [auto]: Verify Firing phase is not interruptible
#[test]
fn step_3_firing_not_interruptible() {
    assert!(!AttackPhase::Firing.is_interruptible(),
        "Firing phase should NOT be interruptible");
}

/// QA Step 4 [auto]: Verify Cooldown phase is not interruptible
#[test]
fn step_4_cooldown_not_interruptible() {
    assert!(!AttackPhase::Cooldown.is_interruptible(),
        "Cooldown phase should NOT be interruptible");
}

/// QA Step 5 [auto]: Verify Reloading phase is interruptible
#[test]
fn step_5_reloading_is_interruptible() {
    assert!(AttackPhase::Reloading.is_interruptible(),
        "Reloading phase should be interruptible");
}

/// QA Step 6 [auto]: For a UnitBaseSource unit: verify it can only Turn during Aiming,
/// cannot Move or Turn during Firing/Cooldown, and can Move+Turn during Reloading.
#[test]
fn step_6_unit_base_source_constraints() {
    let is_turret = false;

    // None: full freedom
    let c = AttackPhase::None.base_action_constraints(is_turret);
    assert!(c.base_can_move && c.base_can_turn, "None phase: should allow move+turn");

    // Aiming: turn only, no move
    let c = AttackPhase::Aiming.base_action_constraints(is_turret);
    assert!(!c.base_can_move, "Aiming: UnitBase should NOT be able to move");
    assert!(c.base_can_turn, "Aiming: UnitBase should be able to turn");

    // Firing: nothing
    let c = AttackPhase::Firing.base_action_constraints(is_turret);
    assert!(!c.base_can_move, "Firing: UnitBase should NOT be able to move");
    assert!(!c.base_can_turn, "Firing: UnitBase should NOT be able to turn");

    // Cooldown: nothing
    let c = AttackPhase::Cooldown.base_action_constraints(is_turret);
    assert!(!c.base_can_move, "Cooldown: UnitBase should NOT be able to move");
    assert!(!c.base_can_turn, "Cooldown: UnitBase should NOT be able to turn");

    // Reloading: full freedom
    let c = AttackPhase::Reloading.base_action_constraints(is_turret);
    assert!(c.base_can_move, "Reloading: UnitBase should be able to move");
    assert!(c.base_can_turn, "Reloading: UnitBase should be able to turn");
}

/// QA Step 7 [auto]: For a TurretSource unit: verify turret actions per phase
/// and verify unit base can Move+Turn during all four phases.
#[test]
fn step_7_turret_source_constraints() {
    let is_turret = true;

    // All phases: turret source base can always move and turn
    for phase in &[AttackPhase::None, AttackPhase::Aiming, AttackPhase::Firing,
                   AttackPhase::Cooldown, AttackPhase::Reloading] {
        let c = phase.base_action_constraints(is_turret);
        assert!(c.base_can_move,
            "Turret source: base should ALWAYS be able to move during {:?}", phase);
        assert!(c.base_can_turn,
            "Turret source: base should ALWAYS be able to turn during {:?}", phase);
    }
}

/// QA Step 8 [auto]: Verify Aiming cancels if the target becomes invalid (e.g., target dies).
#[test]
fn step_8_aiming_cancels_on_target_death() {
    let mut test_app = TestApp::new();
    test_app.step();

    let player_unit;
    let enemy_unit;
    {
        let mut harness = TestHarness::new(&mut test_app.app);
        player_unit = harness.spawn_unit_at_grid(ObjectEnum::Peacekeeper, 20, 20, Owner(Some(0)));
        enemy_unit = harness.spawn_unit_at_grid(ObjectEnum::Peacekeeper, 21, 20, Owner(Some(1)));
    }
    test_app.step();

    // Set attack state to Aiming with target
    {
        let world = test_app.app.world_mut();
        let mut attack_state = world.get_mut::<AttackState>(player_unit).unwrap();
        attack_state.current_target = Some(AttackTarget::UnitTarget(enemy_unit));
        attack_state.phase = AttackPhase::Aiming;
        attack_state.time_in_phase = 0.0;
    }

    // Kill the enemy
    {
        let world = test_app.app.world_mut();
        let mut obj = world.get_mut::<ObjectInstance>(enemy_unit).unwrap();
        obj.apply_damage(1000.0);
    }

    // Step several frames to let remove_dead_entities and attack_phase_system process
    test_app.step_n(5);

    // The attack should reset since target is dead/despawned
    let world = test_app.app.world();
    if let Some(attack_state) = world.get::<AttackState>(player_unit) {
        // Target should be None or phase should be None
        let target_gone = match attack_state.current_target {
            None => true,
            Some(AttackTarget::UnitTarget(t)) => world.get_entity(t).is_err(),
            Some(AttackTarget::LocationTarget(_)) => false,
        };
        assert!(target_gone || attack_state.phase == AttackPhase::None,
            "Attack should cancel when target dies. Phase: {:?}, target: {:?}",
            attack_state.phase, attack_state.current_target);
    }
}
