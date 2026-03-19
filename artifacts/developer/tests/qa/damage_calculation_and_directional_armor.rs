use crate::helpers::*;
use space_crystals::game::combat::types::{
    Armor, DamageEvent, Silhouette,
    DIRECTIONAL_ARMOR_FRONT_MULTIPLIER, DIRECTIONAL_ARMOR_REAR_MULTIPLIER,
};
use space_crystals::game::combat::utils::{circle_rect_overlap_area, is_domain_compatible};
use space_crystals::game::combat::systems::directional_armor_multiplier;
use space_crystals::types::{DomainEnum, TargetDomainEnum};
use std::f32::consts::PI;

/// QA Step 1 [auto]: SingleTarget — 10 damage vs 3 PointArmor => 7 damage taken.
#[test]
fn step_1_single_target_damage_minus_armor() {
    let mut test_app = TestApp::new();
    test_app.step(); // startup

    let target;
    let attacker;
    {
        let mut harness = TestHarness::new(&mut test_app.app);
        target = harness.spawn_unit_at_grid(ObjectEnum::Peacekeeper, 20, 20, Owner(Some(0)));
        attacker = harness.spawn_unit_at_grid(ObjectEnum::Peacekeeper, 25, 20, Owner(Some(1)));
    }
    test_app.step();

    // Record starting HP
    let start_hp = {
        let world = test_app.app.world();
        world.get::<ObjectInstance>(target).unwrap().hp.unwrap()
    };

    // Set armor and insert DamageEvent on target
    {
        let world = test_app.app.world_mut();
        world.entity_mut(target).insert(Armor {
            point_armor: 3.0,
            full_armor: 0.0,
            directional_armor: false,
        });

        let attacker_pos = world.get::<Transform>(attacker).unwrap().translation;
        world.entity_mut(target).insert(DamageEvent::SingleTarget {
            damage: 10.0,
            source: attacker,
            source_position: attacker_pos,
        });
    }
    test_app.step();

    let end_hp = {
        let world = test_app.app.world();
        world.get::<ObjectInstance>(target).unwrap().hp.unwrap()
    };

    let damage_taken = start_hp - end_hp;
    assert!(
        (damage_taken - 7.0).abs() < 0.01,
        "Expected 7 damage (10 - 3 armor), got {}", damage_taken
    );
}

/// QA Step 2 [auto]: SingleTarget — 5 damage vs 8 PointArmor => 0 damage (not negative).
#[test]
fn step_2_armor_exceeds_damage_floors_to_zero() {
    let mut test_app = TestApp::new();
    test_app.step();

    let target;
    let attacker;
    {
        let mut harness = TestHarness::new(&mut test_app.app);
        target = harness.spawn_unit_at_grid(ObjectEnum::Peacekeeper, 20, 20, Owner(Some(0)));
        attacker = harness.spawn_unit_at_grid(ObjectEnum::Peacekeeper, 25, 20, Owner(Some(1)));
    }
    test_app.step();

    let start_hp = {
        let world = test_app.app.world();
        world.get::<ObjectInstance>(target).unwrap().hp.unwrap()
    };

    {
        let world = test_app.app.world_mut();
        world.entity_mut(target).insert(Armor {
            point_armor: 8.0,
            full_armor: 0.0,
            directional_armor: false,
        });

        let attacker_pos = world.get::<Transform>(attacker).unwrap().translation;
        world.entity_mut(target).insert(DamageEvent::SingleTarget {
            damage: 5.0,
            source: attacker,
            source_position: attacker_pos,
        });
    }
    test_app.step();

    let end_hp = {
        let world = test_app.app.world();
        world.get::<ObjectInstance>(target).unwrap().hp.unwrap()
    };

    let damage_taken = start_hp - end_hp;
    assert!(
        damage_taken.abs() < 0.01,
        "Expected 0 damage (armor > attack), got {}", damage_taken
    );
}

/// QA Step 3 [auto]: AoE — unit fully inside AoE circle.
/// damage_share = attack_damage * (unit_area / aoe_area), effective_armor = full_armor,
/// actual_damage = (damage_share - full_armor).max(0).
#[test]
fn step_3_aoe_fully_inside() {
    let mut test_app = TestApp::new();
    test_app.step();

    let target;
    let attacker;
    {
        let mut harness = TestHarness::new(&mut test_app.app);
        target = harness.spawn_unit_at_grid(ObjectEnum::Peacekeeper, 20, 20, Owner(Some(0)));
        attacker = harness.spawn_unit_at_grid(ObjectEnum::Peacekeeper, 25, 25, Owner(Some(1)));
    }
    test_app.step();

    let start_hp = {
        let world = test_app.app.world();
        world.get::<ObjectInstance>(target).unwrap().hp.unwrap()
    };

    // Get target position so we can center AoE right on it (fully inside)
    let target_pos = {
        let world = test_app.app.world();
        world.get::<Transform>(target).unwrap().translation
    };

    let sil_w = 2.0_f32;
    let sil_h = 2.0_f32;
    let aoe_radius = 10.0_f32; // Large radius to fully contain the unit
    let aoe_damage = 50.0_f32;
    let full_armor = 3.0_f32;

    {
        let world = test_app.app.world_mut();
        world.entity_mut(target).insert(Silhouette { width: sil_w, height: sil_h });
        world.entity_mut(target).insert(Armor {
            point_armor: 0.0,
            full_armor,
            directional_armor: false,
        });
        world.entity_mut(target).insert(DamageEvent::AreaOfEffect {
            damage: aoe_damage,
            source: attacker,
            center: target_pos, // centered on unit
            radius: aoe_radius,
            source_owner: Owner(Some(1)),
        });
    }
    test_app.step();

    let end_hp = {
        let world = test_app.app.world();
        world.get::<ObjectInstance>(target).unwrap().hp.unwrap()
    };

    // Verify overlap: unit is fully inside circle, so overlap = unit_area
    let unit_area = sil_w * sil_h;
    let aoe_area = PI * aoe_radius * aoe_radius;
    let overlap = circle_rect_overlap_area(
        Vec2::new(target_pos.x, target_pos.z),
        aoe_radius,
        Vec2::new(target_pos.x, target_pos.z),
        sil_w,
        sil_h,
    );
    assert!(
        (overlap - unit_area).abs() < 0.1,
        "Unit should be fully inside AoE circle, overlap={}, unit_area={}", overlap, unit_area
    );

    let damage_share = aoe_damage * (overlap / aoe_area);
    // effective_armor = full_armor * (overlap / unit_area) = full_armor (since fully inside)
    let effective_armor = full_armor * (overlap / unit_area);
    let expected_damage = (damage_share - effective_armor).max(0.0);

    let damage_taken = start_hp - end_hp;
    assert!(
        (damage_taken - expected_damage).abs() < 0.5,
        "Expected ~{:.1} AoE damage, got {:.1}", expected_damage, damage_taken
    );
}

/// QA Step 4 [auto]: AoE — unit half inside AoE circle.
/// Verify damage_share uses partial overlap and effective_armor uses overlap fraction.
#[test]
fn step_4_aoe_partial_overlap() {
    let mut test_app = TestApp::new();
    test_app.step();

    let target;
    let attacker;
    {
        let mut harness = TestHarness::new(&mut test_app.app);
        target = harness.spawn_unit_at_grid(ObjectEnum::Peacekeeper, 20, 20, Owner(Some(0)));
        attacker = harness.spawn_unit_at_grid(ObjectEnum::Peacekeeper, 25, 25, Owner(Some(1)));
    }
    test_app.step();

    let start_hp = {
        let world = test_app.app.world();
        world.get::<ObjectInstance>(target).unwrap().hp.unwrap()
    };

    let target_pos = {
        let world = test_app.app.world();
        world.get::<Transform>(target).unwrap().translation
    };

    let sil_w = 2.0_f32;
    let sil_h = 2.0_f32;
    let aoe_radius = 2.0_f32; // Small radius so unit is only partially inside
    let aoe_damage = 100.0_f32;
    let full_armor = 2.0_f32;

    // Place AoE center offset from unit so only partial overlap
    let aoe_center = Vec3::new(target_pos.x + aoe_radius, target_pos.y, target_pos.z);

    {
        let world = test_app.app.world_mut();
        world.entity_mut(target).insert(Silhouette { width: sil_w, height: sil_h });
        world.entity_mut(target).insert(Armor {
            point_armor: 0.0,
            full_armor,
            directional_armor: false,
        });
        world.entity_mut(target).insert(DamageEvent::AreaOfEffect {
            damage: aoe_damage,
            source: attacker,
            center: aoe_center,
            radius: aoe_radius,
            source_owner: Owner(Some(1)),
        });
    }
    test_app.step();

    let end_hp = {
        let world = test_app.app.world();
        world.get::<ObjectInstance>(target).unwrap().hp.unwrap()
    };

    // Compute expected values using the same overlap function
    let overlap = circle_rect_overlap_area(
        Vec2::new(aoe_center.x, aoe_center.z),
        aoe_radius,
        Vec2::new(target_pos.x, target_pos.z),
        sil_w,
        sil_h,
    );
    let aoe_area = PI * aoe_radius * aoe_radius;
    let unit_area = sil_w * sil_h;

    assert!(
        overlap > 0.0 && overlap < unit_area,
        "Expected partial overlap, got overlap={}, unit_area={}", overlap, unit_area
    );

    let damage_share = aoe_damage * (overlap / aoe_area);
    let effective_armor = full_armor * (overlap / unit_area);
    let expected_damage = (damage_share - effective_armor).max(0.0);

    let damage_taken = start_hp - end_hp;
    assert!(
        (damage_taken - expected_damage).abs() < 0.5,
        "Expected ~{:.1} partial AoE damage, got {:.1}", expected_damage, damage_taken
    );
}

/// QA Step 5 [auto]: Domain filtering — Ground AoE does not damage Air units.
/// Verify is_domain_compatible rejects Ground attack against Air unit.
#[test]
fn step_5_domain_filtering_ground_aoe_vs_air() {
    // Ground attack should NOT be compatible with Air units
    assert!(
        !is_domain_compatible(&TargetDomainEnum::Ground, &DomainEnum::Air),
        "Ground attack should not be compatible with Air domain"
    );

    // Ground attack SHOULD be compatible with Ground units
    assert!(
        is_domain_compatible(&TargetDomainEnum::Ground, &DomainEnum::Ground),
        "Ground attack should be compatible with Ground domain"
    );

    // Ground attack SHOULD be compatible with Underground units
    assert!(
        is_domain_compatible(&TargetDomainEnum::Ground, &DomainEnum::Underground),
        "Ground attack should be compatible with Underground domain"
    );
}

/// QA Step 6 [auto]: Directional armor — unit facing attacker gets increased armor (front bonus).
#[test]
fn step_6_directional_armor_facing_attacker() {
    let mut test_app = TestApp::new();
    test_app.step();

    let target;
    let attacker;
    {
        let mut harness = TestHarness::new(&mut test_app.app);
        target = harness.spawn_unit_at_grid(ObjectEnum::Peacekeeper, 20, 20, Owner(Some(0)));
        attacker = harness.spawn_unit_at_grid(ObjectEnum::Peacekeeper, 20, 25, Owner(Some(1)));
    }
    test_app.step();

    let start_hp = {
        let world = test_app.app.world();
        world.get::<ObjectInstance>(target).unwrap().hp.unwrap()
    };

    // Get attacker position
    let attacker_pos = {
        let world = test_app.app.world();
        world.get::<Transform>(attacker).unwrap().translation
    };

    // Rotate target to face the attacker using look_at (accounts for Bevy's -Z forward)
    {
        let world = test_app.app.world_mut();
        world.get_mut::<Transform>(target).unwrap().look_at(attacker_pos, Vec3::Y);
    }

    let base_armor = 10.0_f32;
    let damage = 20.0_f32;

    {
        let world = test_app.app.world_mut();
        world.entity_mut(target).insert(Armor {
            point_armor: base_armor,
            full_armor: 0.0,
            directional_armor: true,
        });
        world.entity_mut(target).insert(DamageEvent::SingleTarget {
            damage,
            source: attacker,
            source_position: attacker_pos,
        });
    }
    test_app.step();

    let end_hp = {
        let world = test_app.app.world();
        world.get::<ObjectInstance>(target).unwrap().hp.unwrap()
    };

    // When facing attacker, armor should be multiplied by FRONT_MULTIPLIER (1.5)
    let front_armor = base_armor * DIRECTIONAL_ARMOR_FRONT_MULTIPLIER;
    let expected_damage = (damage - front_armor).max(0.0);

    let damage_taken = start_hp - end_hp;
    assert!(
        (damage_taken - expected_damage).abs() < 0.5,
        "Front-facing armor should increase. Expected {:.1} damage (armor {:.1}), got {:.1}",
        expected_damage, front_armor, damage_taken
    );
    // Verify the damage is LESS than without directional bonus
    assert!(
        damage_taken < (damage - base_armor),
        "Front-facing should take less damage than base armor. Took {:.1}, base would be {:.1}",
        damage_taken, damage - base_armor
    );
}

/// QA Step 7 [auto]: Directional armor — unit facing away from attacker gets decreased armor (rear penalty).
#[test]
fn step_7_directional_armor_facing_away() {
    let mut test_app = TestApp::new();
    test_app.step();

    let target;
    let attacker;
    {
        let mut harness = TestHarness::new(&mut test_app.app);
        target = harness.spawn_unit_at_grid(ObjectEnum::Peacekeeper, 20, 20, Owner(Some(0)));
        attacker = harness.spawn_unit_at_grid(ObjectEnum::Peacekeeper, 20, 25, Owner(Some(1)));
    }
    test_app.step();

    let start_hp = {
        let world = test_app.app.world();
        world.get::<ObjectInstance>(target).unwrap().hp.unwrap()
    };

    let attacker_pos = {
        let world = test_app.app.world();
        world.get::<Transform>(attacker).unwrap().translation
    };

    // Rotate target to face AWAY from attacker using look_at on a point opposite the attacker
    {
        let world = test_app.app.world_mut();
        let target_pos = world.get::<Transform>(target).unwrap().translation;
        let away_pos = 2.0 * target_pos - attacker_pos;
        world.get_mut::<Transform>(target).unwrap().look_at(away_pos, Vec3::Y);
    }

    let base_armor = 10.0_f32;
    let damage = 20.0_f32;

    {
        let world = test_app.app.world_mut();
        world.entity_mut(target).insert(Armor {
            point_armor: base_armor,
            full_armor: 0.0,
            directional_armor: true,
        });
        world.entity_mut(target).insert(DamageEvent::SingleTarget {
            damage,
            source: attacker,
            source_position: attacker_pos,
        });
    }
    test_app.step();

    let end_hp = {
        let world = test_app.app.world();
        world.get::<ObjectInstance>(target).unwrap().hp.unwrap()
    };

    // When facing away, armor should be multiplied by REAR_MULTIPLIER (0.5)
    let rear_armor = base_armor * DIRECTIONAL_ARMOR_REAR_MULTIPLIER;
    let expected_damage = (damage - rear_armor).max(0.0);

    let damage_taken = start_hp - end_hp;
    assert!(
        (damage_taken - expected_damage).abs() < 0.5,
        "Rear-facing armor should decrease. Expected {:.1} damage (armor {:.1}), got {:.1}",
        expected_damage, rear_armor, damage_taken
    );
    // Verify the damage is MORE than without directional penalty
    assert!(
        damage_taken > (damage - base_armor),
        "Rear-facing should take more damage than base armor. Took {:.1}, base would be {:.1}",
        damage_taken, damage - base_armor
    );
}

/// QA Step 8 [auto]: Directional armor only applies to units with directional_armor = true.
/// Units without it use base armor regardless of angle.
#[test]
fn step_8_no_directional_armor_ignores_angle() {
    let mut test_app = TestApp::new();
    test_app.step();

    let target;
    let attacker;
    {
        let mut harness = TestHarness::new(&mut test_app.app);
        target = harness.spawn_unit_at_grid(ObjectEnum::Peacekeeper, 20, 20, Owner(Some(0)));
        attacker = harness.spawn_unit_at_grid(ObjectEnum::Peacekeeper, 20, 25, Owner(Some(1)));
    }
    test_app.step();

    let start_hp = {
        let world = test_app.app.world();
        world.get::<ObjectInstance>(target).unwrap().hp.unwrap()
    };

    let attacker_pos = {
        let world = test_app.app.world();
        world.get::<Transform>(attacker).unwrap().translation
    };

    // Rotate target to face attacker (which would normally trigger front bonus)
    {
        let world = test_app.app.world_mut();
        world.get_mut::<Transform>(target).unwrap().look_at(attacker_pos, Vec3::Y);
    }

    let base_armor = 10.0_f32;
    let damage = 20.0_f32;

    {
        let world = test_app.app.world_mut();
        // directional_armor = false: angle should NOT matter
        world.entity_mut(target).insert(Armor {
            point_armor: base_armor,
            full_armor: 0.0,
            directional_armor: false,
        });
        world.entity_mut(target).insert(DamageEvent::SingleTarget {
            damage,
            source: attacker,
            source_position: attacker_pos,
        });
    }
    test_app.step();

    let end_hp = {
        let world = test_app.app.world();
        world.get::<ObjectInstance>(target).unwrap().hp.unwrap()
    };

    // Without directional armor, damage = (attack - base_armor).max(0)
    let expected_damage = (damage - base_armor).max(0.0);
    let damage_taken = start_hp - end_hp;
    assert!(
        (damage_taken - expected_damage).abs() < 0.01,
        "Without directional armor, damage should use base armor only. Expected {:.1}, got {:.1}",
        expected_damage, damage_taken
    );
}

/// QA Step 9 [auto]: AoE directional armor uses direction from AoE center to unit,
/// not from attacker to unit.
#[test]
fn step_9_aoe_directional_armor_uses_aoe_center() {
    // Verify using the directional_armor_multiplier function directly.
    // The apply_damage_system passes `center` (AoE center) as source_pos for directional calc.

    // Setup: attacker at (100, 0, 0), AoE center at (0, 0, 10), unit at (0, 0, 0) facing +Z.
    let attacker_pos = Vec3::new(100.0, 0.0, 0.0);
    let aoe_center = Vec3::new(0.0, 0.0, 10.0);
    let unit_pos = Vec3::new(0.0, 0.0, 0.0);
    let unit_forward = Vec3::new(0.0, 0.0, 1.0); // facing +Z (toward AoE center)

    // From AoE center: attack comes from +Z direction, unit faces +Z -> frontal hit
    let mult_from_center = directional_armor_multiplier(aoe_center, unit_pos, unit_forward);
    assert!(
        (mult_from_center - DIRECTIONAL_ARMOR_FRONT_MULTIPLIER).abs() < 0.01,
        "AoE center is in front of unit, should get front multiplier ({:.1}), got {:.1}",
        DIRECTIONAL_ARMOR_FRONT_MULTIPLIER, mult_from_center
    );

    // From attacker position: attack comes from +X direction, unit faces +Z -> side hit
    let mult_from_attacker = directional_armor_multiplier(attacker_pos, unit_pos, unit_forward);
    assert!(
        (mult_from_attacker - 1.0).abs() < 0.01,
        "Attacker is to the side, should get side multiplier (1.0), got {:.1}",
        mult_from_attacker
    );

    // These should differ, proving AoE uses center not attacker pos
    assert!(
        (mult_from_center - mult_from_attacker).abs() > 0.1,
        "Multiplier from AoE center ({:.1}) should differ from attacker pos ({:.1})",
        mult_from_center, mult_from_attacker
    );
}

/// QA Step 10 [auto]: Domain filtering — Ground attack does not damage Air units;
/// Universal attack damages all units in AoE.
#[test]
fn step_10_domain_filtering_ground_vs_universal() {
    // Ground attack domain compatibility
    assert!(
        !is_domain_compatible(&TargetDomainEnum::Ground, &DomainEnum::Air),
        "Ground attack must NOT hit Air units"
    );
    assert!(
        is_domain_compatible(&TargetDomainEnum::Ground, &DomainEnum::Ground),
        "Ground attack must hit Ground units"
    );

    // Universal attack domain compatibility
    assert!(
        is_domain_compatible(&TargetDomainEnum::Universal, &DomainEnum::Air),
        "Universal attack must hit Air units"
    );
    assert!(
        is_domain_compatible(&TargetDomainEnum::Universal, &DomainEnum::Ground),
        "Universal attack must hit Ground units"
    );
    assert!(
        is_domain_compatible(&TargetDomainEnum::Universal, &DomainEnum::Underground),
        "Universal attack must hit Underground units"
    );

    // Air attack domain compatibility
    assert!(
        is_domain_compatible(&TargetDomainEnum::Air, &DomainEnum::Air),
        "Air attack must hit Air units"
    );
    assert!(
        !is_domain_compatible(&TargetDomainEnum::Air, &DomainEnum::Ground),
        "Air attack must NOT hit Ground units"
    );
}
