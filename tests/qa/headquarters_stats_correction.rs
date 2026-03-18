use space_crystals::game::types::structures::syndicate_structure_stats::*;
use space_crystals::types::ObjectEnum;
use space_crystals::game::types::objects::ObjectInstance;

/// QA Step 1 [auto]: Verify HQ_MAX_HP is 400.0
#[test]
fn step_1_hq_max_hp_is_400() {
    assert_eq!(HQ_MAX_HP, 400.0, "HQ_MAX_HP should be 400.0");
}

/// QA Step 2 [auto]: Verify HQ_POINT_ARMOR is 1 and HQ_FULL_ARMOR is 4
#[test]
fn step_2_hq_armor_values() {
    assert_eq!(HQ_POINT_ARMOR, 1, "HQ_POINT_ARMOR should be 1");
    assert_eq!(HQ_FULL_ARMOR, 4, "HQ_FULL_ARMOR should be 4");
}

/// QA Step 3 [auto]: Verify HQ_SC_COST is 200 and HQ_BUILD_FRAMES is 400
#[test]
fn step_3_hq_cost_and_build_frames() {
    assert_eq!(HQ_SC_COST, 200, "HQ_SC_COST should be 200");
    assert_eq!(HQ_BUILD_FRAMES, 400, "HQ_BUILD_FRAMES should be 400 (25 seconds at 16 FPS)");
}

/// QA Step 4 [auto]: All existing HQ tests pass with updated expected values
#[test]
fn step_4_hq_destructible_instance_uses_correct_hp() {
    let obj = ObjectInstance::destructible(ObjectEnum::Headquarters, HQ_MAX_HP);
    assert_eq!(obj.hp, Some(400.0), "Destructible HQ should have 400 HP");
    assert_eq!(obj.max_hp, Some(400.0), "Destructible HQ should have 400 max HP");
    assert!(obj.is_alive(), "Freshly created HQ should be alive");
}
