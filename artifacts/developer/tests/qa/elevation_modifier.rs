use crate::helpers::*;
use space_crystals::game::world::types::{elevation_modifier, ElevationMap};
use space_crystals::types::DomainEnum;

/// QA Step 1 [auto]: Ground unit on high ground sees farther downhill (SightRange + 1)
/// We verify the elevation_modifier function returns +1 for higher source.
#[test]
fn step_1_high_ground_positive_modifier() {
    let result = elevation_modifier(DomainEnum::Ground, 8, DomainEnum::Ground, 5);
    assert_eq!(result, 1, "Source on higher ground should get +1 modifier");
}

/// QA Step 2 [auto]: Ground unit on low ground sees less uphill (SightRange - 1)
#[test]
fn step_2_low_ground_negative_modifier() {
    let result = elevation_modifier(DomainEnum::Ground, 3, DomainEnum::Ground, 7);
    assert_eq!(result, -1, "Source on lower ground should get -1 modifier");
}

/// QA Step 3 [auto]: Equal elevation: no modifier
#[test]
fn step_3_equal_elevation_no_modifier() {
    let result = elevation_modifier(DomainEnum::Ground, 5, DomainEnum::Ground, 5);
    assert_eq!(result, 0, "Equal elevation should give 0 modifier");
}

/// QA Step 4 [auto]: Large elevation gap still only +1/-1
#[test]
fn step_4_binary_modifier_regardless_of_gap() {
    let result_up = elevation_modifier(DomainEnum::Ground, 16, DomainEnum::Ground, 0);
    assert_eq!(result_up, 1, "Large height advantage should still be +1, not proportional");

    let result_down = elevation_modifier(DomainEnum::Ground, 0, DomainEnum::Ground, 16);
    assert_eq!(result_down, -1, "Large height disadvantage should still be -1, not proportional");
}

/// QA Step 5 [auto]: Air units exempt (both as source and target)
#[test]
fn step_5_air_units_exempt() {
    let result1 = elevation_modifier(DomainEnum::Air, 10, DomainEnum::Ground, 2);
    assert_eq!(result1, 0, "Air source should be exempt (0 modifier)");

    let result2 = elevation_modifier(DomainEnum::Ground, 10, DomainEnum::Air, 2);
    assert_eq!(result2, 0, "Air target should be exempt (0 modifier)");

    let result3 = elevation_modifier(DomainEnum::Air, 10, DomainEnum::Air, 2);
    assert_eq!(result3, 0, "Both air should be exempt (0 modifier)");
}

/// QA Step 6 [auto]: Underground units use surface tile elevation
#[test]
fn step_6_underground_uses_surface_elevation() {
    let result = elevation_modifier(DomainEnum::Underground, 8, DomainEnum::Ground, 5);
    assert_eq!(result, 1, "Underground unit at elevation 8 vs ground at 5 should get +1");

    let result2 = elevation_modifier(DomainEnum::Ground, 5, DomainEnum::Underground, 8);
    assert_eq!(result2, -1, "Ground at 5 vs underground at 8 should get -1");

    let result3 = elevation_modifier(DomainEnum::Underground, 5, DomainEnum::Underground, 5);
    assert_eq!(result3, 0, "Underground vs underground at same elevation should be 0");
}

/// Integration test: Verify ElevationMap resource is initialized in TestApp
#[test]
fn elevation_map_resource_exists() {
    let mut test_app = TestApp::new();
    test_app.step();

    let world = test_app.app.world();
    let _elev_map = world.resource::<ElevationMap>();
}
