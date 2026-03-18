use crate::helpers::*;
use bevy::ecs::system::RunSystemOnce;
use space_crystals::game::types::structures::ConstructionHP;
use space_crystals::game::types::objects::ObjectInstance;

/// Helper: run construction HP tick manually (the actual system is pub(crate)).
/// Replicates the logic: progress += 1/build_frames, HP = max_hp * (0.10 + 0.90 * progress).
/// Collects entities that complete construction and removes ConstructionHP directly.
fn tick_construction(app: &mut bevy::app::App) {
    let completed: Vec<bevy::prelude::Entity> = app.world_mut().run_system_once(
        |mut query: bevy::prelude::Query<(
            bevy::prelude::Entity,
            &mut ObjectInstance,
            &mut ConstructionHP,
        )>| {
            let mut completed = Vec::new();
            for (entity, mut obj, mut construction) in query.iter_mut() {
                let increment = 1.0 / construction.build_frames as f32;
                construction.progress = (construction.progress + increment).min(1.0);

                if let Some(max_hp) = obj.max_hp {
                    let new_hp = max_hp * ConstructionHP::hp_fraction(construction.progress);
                    if let Some(current_hp) = obj.hp {
                        if current_hp < new_hp {
                            let prev_progress = (construction.progress - increment).max(0.0);
                            let prev_expected = max_hp * ConstructionHP::hp_fraction(prev_progress);
                            if current_hp >= prev_expected - 0.001 {
                                obj.hp = Some(new_hp.min(max_hp));
                            }
                        }
                    }
                }

                if construction.is_complete() {
                    completed.push(entity);
                }
            }
            completed
        },
    ).unwrap();
    // Remove ConstructionHP directly (Commands are deferred, so we do it here)
    for entity in completed {
        app.world_mut().entity_mut(entity).remove::<ConstructionHP>();
    }
}

/// QA Step 1 [auto]: Begin constructing a Tunnel with a Syndicate Agent.
/// QA Step 2 [auto]: Immediately after construction starts, inspect HP — verify ~10% of MaxHP.
/// (Combined: spawn a tunnel under construction and verify initial HP)
#[test]
fn step_1_2_initial_construction_hp_is_ten_percent() {
    let mut test_app = TestApp::new();
    test_app.step();

    // Spawn a tunnel and manually set it under construction.
    // Tunnel T1 max_hp = 600.0
    let max_hp = 600.0_f32;
    let build_frames = 100_u32;

    let tunnel_entity = {
        let mut harness = TestHarness::new(&mut test_app.app);
        harness.spawn_structure_at_grid(ObjectEnum::Tunnel, 20, 20, Owner(Some(0)))
    };
    test_app.step(); // flush spawn

    // Replace its ObjectInstance with under_construction version and add ConstructionHP
    test_app.app.world_mut().entity_mut(tunnel_entity)
        .insert(ObjectInstance::under_construction(ObjectEnum::Tunnel, max_hp))
        .insert(ConstructionHP::new(build_frames));

    // Verify initial HP is 10% of max
    let (hp, _max) = TestHarness::new(&mut test_app.app).get_health(tunnel_entity).unwrap();
    let expected_hp = max_hp * 0.10;
    assert!(
        (hp - expected_hp).abs() < 1.0,
        "Initial construction HP should be ~{} (10% of {}), got {}",
        expected_hp, max_hp, hp
    );
}

/// QA Step 3 [auto]: At approximately 50% construction progress, verify HP is ~55% of MaxHP.
#[test]
fn step_3_hp_at_fifty_percent_progress() {
    let mut test_app = TestApp::new();
    test_app.step();

    let max_hp = 600.0_f32;
    let build_frames = 100_u32;

    let tunnel_entity = {
        let mut harness = TestHarness::new(&mut test_app.app);
        harness.spawn_structure_at_grid(ObjectEnum::Tunnel, 20, 20, Owner(Some(0)))
    };
    test_app.step();

    test_app.app.world_mut().entity_mut(tunnel_entity)
        .insert(ObjectInstance::under_construction(ObjectEnum::Tunnel, max_hp))
        .insert(ConstructionHP::new(build_frames));

    // Run construction tick 50 times (50% of 100 build_frames)
    for _ in 0..50 {
        tick_construction(&mut test_app.app);
    }

    let (hp, _max) = TestHarness::new(&mut test_app.app).get_health(tunnel_entity).unwrap();
    // At 50% progress: HP = 600 * (0.10 + 0.90 * 0.50) = 600 * 0.55 = 330
    let expected_hp = max_hp * 0.55;
    assert!(
        (hp - expected_hp).abs() < 5.0,
        "HP at 50% progress should be ~{} (55% of {}), got {}",
        expected_hp, max_hp, hp
    );
}

/// QA Step 4 [auto]: Attack the partially-built structure — verify it takes damage.
#[test]
fn step_4_partially_built_takes_damage() {
    let mut test_app = TestApp::new();
    test_app.step();

    let max_hp = 600.0_f32;
    let build_frames = 100_u32;

    let tunnel_entity = {
        let mut harness = TestHarness::new(&mut test_app.app);
        harness.spawn_structure_at_grid(ObjectEnum::Tunnel, 20, 20, Owner(Some(0)))
    };
    test_app.step();

    test_app.app.world_mut().entity_mut(tunnel_entity)
        .insert(ObjectInstance::under_construction(ObjectEnum::Tunnel, max_hp))
        .insert(ConstructionHP::new(build_frames));

    // Run 20 ticks (20% progress → HP = 600 * (0.10 + 0.90 * 0.20) = 600 * 0.28 = 168)
    for _ in 0..20 {
        tick_construction(&mut test_app.app);
    }

    let (hp_before, _) = TestHarness::new(&mut test_app.app).get_health(tunnel_entity).unwrap();

    // Apply damage directly
    let damage = 50.0_f32;
    {
        let world = test_app.app.world_mut();
        let mut obj = world.get_mut::<ObjectInstance>(tunnel_entity).unwrap();
        obj.apply_damage(damage);
    }

    let (hp_after, _) = TestHarness::new(&mut test_app.app).get_health(tunnel_entity).unwrap();
    assert!(
        (hp_before - hp_after - damage).abs() < 1.0,
        "HP should decrease by {} from {} to {}, got {}",
        damage, hp_before, hp_before - damage, hp_after
    );
}

/// QA Step 5 [auto]: Destroy the partially-built structure before completion.
#[test]
fn step_5_destroy_partially_built() {
    let mut test_app = TestApp::new();
    test_app.step();

    let max_hp = 600.0_f32;
    let build_frames = 100_u32;

    let tunnel_entity = {
        let mut harness = TestHarness::new(&mut test_app.app);
        harness.spawn_structure_at_grid(ObjectEnum::Tunnel, 20, 20, Owner(Some(0)))
    };
    test_app.step();

    test_app.app.world_mut().entity_mut(tunnel_entity)
        .insert(ObjectInstance::under_construction(ObjectEnum::Tunnel, max_hp))
        .insert(ConstructionHP::new(build_frames));

    // Run 10 ticks → HP ≈ 600 * 0.19 ≈ 114
    for _ in 0..10 {
        tick_construction(&mut test_app.app);
    }

    // Deal massive damage to destroy it
    {
        let world = test_app.app.world_mut();
        let mut obj = world.get_mut::<ObjectInstance>(tunnel_entity).unwrap();
        let destroyed = obj.apply_damage(1000.0);
        assert!(destroyed, "Structure should be destroyed by 1000 damage");
    }

    let harness = TestHarness::new(&mut test_app.app);
    assert!(!harness.is_alive(tunnel_entity), "Destroyed structure should not be alive");
}

/// QA Step 6 [auto]: Build structure to full completion — verify HP reaches full MaxHP.
#[test]
fn step_6_full_completion_reaches_max_hp() {
    let mut test_app = TestApp::new();
    test_app.step();

    let max_hp = 600.0_f32;
    // Use 10 build_frames for clean float arithmetic (0.1 * 10 = 1.0)
    let build_frames = 10_u32;

    let tunnel_entity = {
        let mut harness = TestHarness::new(&mut test_app.app);
        harness.spawn_structure_at_grid(ObjectEnum::Tunnel, 20, 20, Owner(Some(0)))
    };
    test_app.step();

    test_app.app.world_mut().entity_mut(tunnel_entity)
        .insert(ObjectInstance::under_construction(ObjectEnum::Tunnel, max_hp))
        .insert(ConstructionHP::new(build_frames));

    // Run enough ticks to complete construction (a few extra to handle float rounding)
    for _ in 0..(build_frames + 2) {
        tick_construction(&mut test_app.app);
    }

    let (hp, max) = TestHarness::new(&mut test_app.app).get_health(tunnel_entity).unwrap();
    assert!(
        (hp - max_hp).abs() < 1.0,
        "HP at completion should be ~{}, got {}",
        max_hp, hp
    );
    assert!(
        (max - max_hp).abs() < 0.01,
        "Max HP should be {}, got {}",
        max_hp, max
    );

    // ConstructionHP component should be removed
    let world = test_app.app.world();
    assert!(
        world.get::<ConstructionHP>(tunnel_entity).is_none(),
        "ConstructionHP component should be removed after completion"
    );
}

/// QA Step 7 [auto]: Verify GDO buildings NOT using ConstructionHP spawn at full HP.
#[test]
fn step_7_gdo_buildings_unaffected() {
    let mut test_app = TestApp::new();
    test_app.step();

    let pp;
    let bk;
    {
        let mut harness = TestHarness::new(&mut test_app.app);
        pp = harness.spawn_structure_at_grid(ObjectEnum::PowerPlant, 15, 15, Owner(Some(0)));
        bk = harness.spawn_structure_at_grid(ObjectEnum::Barracks, 20, 15, Owner(Some(0)));
    }
    test_app.step();

    // Power Plant
    {
        let harness = TestHarness::new(&mut test_app.app);
        let (pp_hp, pp_max) = harness.get_health(pp).unwrap();
        assert!(
            (pp_hp - pp_max).abs() < 0.01,
            "PowerPlant should spawn at full HP: {}/{}",
            pp_hp, pp_max
        );
    }
    assert!(
        test_app.app.world().get::<ConstructionHP>(pp).is_none(),
        "PowerPlant should not have ConstructionHP"
    );

    // Barracks
    {
        let harness = TestHarness::new(&mut test_app.app);
        let (bk_hp, bk_max) = harness.get_health(bk).unwrap();
        assert!(
            (bk_hp - bk_max).abs() < 0.01,
            "Barracks should spawn at full HP: {}/{}",
            bk_hp, bk_max
        );
    }
    assert!(
        test_app.app.world().get::<ConstructionHP>(bk).is_none(),
        "Barracks should not have ConstructionHP"
    );
}
