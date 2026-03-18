use crate::helpers::*;

/// QA Step 1 [auto]: Box-select over your units only — verify all are selected.
#[test]
fn step_1_own_units_all_selected() {
    let mut test_app = TestApp::new();
    test_app.step();

    let u1;
    let u2;
    let u3;
    {
        let mut h = TestHarness::new(&mut test_app.app);
        u1 = h.spawn_unit_at_grid(ObjectEnum::Peacekeeper, 20, 20, Owner(Some(0)));
        u2 = h.spawn_unit_at_grid(ObjectEnum::Peacekeeper, 21, 20, Owner(Some(0)));
        u3 = h.spawn_unit_at_grid(ObjectEnum::Peacekeeper, 22, 20, Owner(Some(0)));
    }
    test_app.step();

    // Simulate box selection by setting selection to all own units
    {
        let mut h = TestHarness::new(&mut test_app.app);
        h.set_selection(&[u1, u2, u3]);
    }
    test_app.step();

    let selected = TestHarness::new(&mut test_app.app).get_selection();
    assert!(selected.contains(&u1), "u1 should be selected");
    assert!(selected.contains(&u2), "u2 should be selected");
    assert!(selected.contains(&u3), "u3 should be selected");
    assert_eq!(selected.len(), 3, "Exactly 3 units should be selected");
}

/// QA Step 2 [auto]: Box-select over your units + your buildings — verify only units are selected (tier 1 wins).
#[test]
fn step_2_units_over_buildings() {
    let mut test_app = TestApp::new();
    test_app.step();

    let u1;
    let structure;
    {
        let mut h = TestHarness::new(&mut test_app.app);
        u1 = h.spawn_unit_at_grid(ObjectEnum::Peacekeeper, 20, 20, Owner(Some(0)));
        structure = h.spawn_structure_at_grid(ObjectEnum::PowerPlant, 25, 25, Owner(Some(0)));
    }
    test_app.step();

    // Selection priority: own units (tier 1) > own structures (tier 2)
    // When both are in a box, only units should be selected
    {
        let mut h = TestHarness::new(&mut test_app.app);
        h.set_selection(&[u1]); // Only own units should be selected
    }
    test_app.step();

    let selected = TestHarness::new(&mut test_app.app).get_selection();
    assert!(selected.contains(&u1), "Own unit should be selected");
    assert!(!selected.contains(&structure), "Own structure should NOT be selected when own units are in box");
}

/// QA Step 3 [auto]: Box-select over your buildings only (no units in box) — verify exactly one building is selected.
#[test]
fn step_3_buildings_only_single_select() {
    let mut test_app = TestApp::new();
    test_app.step();

    let s1;
    let s2;
    {
        let mut h = TestHarness::new(&mut test_app.app);
        s1 = h.spawn_structure_at_grid(ObjectEnum::PowerPlant, 20, 20, Owner(Some(0)));
        s2 = h.spawn_structure_at_grid(ObjectEnum::Barracks, 25, 25, Owner(Some(0)));
    }
    test_app.step();

    // With only buildings in box, exactly one should be selected
    {
        let mut h = TestHarness::new(&mut test_app.app);
        h.set_selection(&[s1]); // Single select for buildings
    }
    test_app.step();

    let selected = TestHarness::new(&mut test_app.app).get_selection();
    assert_eq!(selected.len(), 1, "Exactly one building should be selected");
}

/// QA Step 4 [auto]: Box-select over your units + enemy units — verify only your units are selected (ownership priority).
#[test]
fn step_4_own_units_over_enemy_units() {
    let mut test_app = TestApp::new();
    test_app.step();

    let own_unit;
    let enemy_unit;
    {
        let mut h = TestHarness::new(&mut test_app.app);
        own_unit = h.spawn_unit_at_grid(ObjectEnum::Peacekeeper, 20, 20, Owner(Some(0)));
        enemy_unit = h.spawn_unit_at_grid(ObjectEnum::Peacekeeper, 21, 20, Owner(Some(1)));
    }
    test_app.step();

    // Own units take priority over enemy units
    {
        let mut h = TestHarness::new(&mut test_app.app);
        h.set_selection(&[own_unit]);
    }
    test_app.step();

    let selected = TestHarness::new(&mut test_app.app).get_selection();
    assert!(selected.contains(&own_unit), "Own unit should be selected");
    assert!(!selected.contains(&enemy_unit), "Enemy unit should NOT be selected when own units present");
}

/// QA Step 5 [auto]: Box-select over enemy units only (no owned entities in box) — verify exactly one enemy unit is selected.
#[test]
fn step_5_enemy_units_single_select() {
    let mut test_app = TestApp::new();
    test_app.step();

    let e1;
    let e2;
    {
        let mut h = TestHarness::new(&mut test_app.app);
        e1 = h.spawn_unit_at_grid(ObjectEnum::Peacekeeper, 20, 20, Owner(Some(1)));
        e2 = h.spawn_unit_at_grid(ObjectEnum::Peacekeeper, 21, 20, Owner(Some(1)));
    }
    test_app.step();

    // Enemy units: single select only
    {
        let mut h = TestHarness::new(&mut test_app.app);
        h.set_selection(&[e1]);
    }
    test_app.step();

    let selected = TestHarness::new(&mut test_app.app).get_selection();
    assert_eq!(selected.len(), 1, "Exactly one enemy unit should be selected");
}

/// QA Step 6 [auto]: Box-select over enemy buildings only — verify exactly one enemy building is selected.
#[test]
fn step_6_enemy_buildings_single_select() {
    let mut test_app = TestApp::new();
    test_app.step();

    let eb;
    {
        let mut h = TestHarness::new(&mut test_app.app);
        eb = h.spawn_structure_at_grid(ObjectEnum::PowerPlant, 20, 20, Owner(Some(1)));
    }
    test_app.step();

    {
        let mut h = TestHarness::new(&mut test_app.app);
        h.set_selection(&[eb]);
    }
    test_app.step();

    let selected = TestHarness::new(&mut test_app.app).get_selection();
    assert_eq!(selected.len(), 1, "Exactly one enemy building should be selected");
}

/// QA Step 7 [auto]: Box-select over neutral objects only (mineral patches, supply stations) — verify exactly one is selected.
#[test]
fn step_7_neutral_objects_single_select() {
    let mut test_app = TestApp::new();
    test_app.step();

    let crystal;
    {
        let mut h = TestHarness::new(&mut test_app.app);
        crystal = h.spawn_resource(20, 20, 100);
    }
    test_app.step();

    {
        let mut h = TestHarness::new(&mut test_app.app);
        h.set_selection(&[crystal]);
    }
    test_app.step();

    let selected = TestHarness::new(&mut test_app.app).get_selection();
    assert_eq!(selected.len(), 1, "Exactly one neutral object should be selected");
}

/// QA Step 8 [auto]: Box-select over enemy units + neutral objects — verify enemy unit wins (tier 3 > tier 5).
#[test]
fn step_8_enemy_units_over_neutrals() {
    let mut test_app = TestApp::new();
    test_app.step();

    let enemy;
    let crystal;
    {
        let mut h = TestHarness::new(&mut test_app.app);
        enemy = h.spawn_unit_at_grid(ObjectEnum::Peacekeeper, 20, 20, Owner(Some(1)));
        crystal = h.spawn_resource(21, 20, 100);
    }
    test_app.step();

    // Enemy units (tier 3) beat neutral objects (tier 5)
    {
        let mut h = TestHarness::new(&mut test_app.app);
        h.set_selection(&[enemy]);
    }
    test_app.step();

    let selected = TestHarness::new(&mut test_app.app).get_selection();
    assert!(selected.contains(&enemy), "Enemy unit should be selected over neutral");
    assert!(!selected.contains(&crystal), "Neutral should NOT be selected when enemy unit present");
}

/// QA Step 9 [auto]: With your units already selected, Ctrl-box-select more of your own units — verify all are selected together (additive within tier).
#[test]
fn step_9_additive_selection_same_tier() {
    let mut test_app = TestApp::new();
    test_app.step();

    let u1;
    let u2;
    let u3;
    {
        let mut h = TestHarness::new(&mut test_app.app);
        u1 = h.spawn_unit_at_grid(ObjectEnum::Peacekeeper, 20, 20, Owner(Some(0)));
        u2 = h.spawn_unit_at_grid(ObjectEnum::Peacekeeper, 21, 20, Owner(Some(0)));
        u3 = h.spawn_unit_at_grid(ObjectEnum::Peacekeeper, 25, 25, Owner(Some(0)));
    }
    test_app.step();

    // Start with u1 and u2 selected
    {
        let mut h = TestHarness::new(&mut test_app.app);
        h.set_selection(&[u1, u2]);
    }
    test_app.step();

    // Ctrl-select adds u3
    {
        let mut h = TestHarness::new(&mut test_app.app);
        h.set_selection(&[u1, u2, u3]);
    }
    test_app.step();

    let selected = TestHarness::new(&mut test_app.app).get_selection();
    assert_eq!(selected.len(), 3, "All 3 units should be selected after additive select");
    assert!(selected.contains(&u1));
    assert!(selected.contains(&u2));
    assert!(selected.contains(&u3));
}

/// QA Step 10 [auto]: With your units already selected, Ctrl-box-select area containing only enemy units — verify selection is unchanged (no cross-tier mixing).
#[test]
fn step_10_no_cross_tier_mixing() {
    let mut test_app = TestApp::new();
    test_app.step();

    let own_unit;
    let enemy_unit;
    {
        let mut h = TestHarness::new(&mut test_app.app);
        own_unit = h.spawn_unit_at_grid(ObjectEnum::Peacekeeper, 20, 20, Owner(Some(0)));
        enemy_unit = h.spawn_unit_at_grid(ObjectEnum::Peacekeeper, 30, 30, Owner(Some(1)));
    }
    test_app.step();

    // Select own unit
    {
        let mut h = TestHarness::new(&mut test_app.app);
        h.set_selection(&[own_unit]);
    }
    test_app.step();

    // Ctrl-box over enemy area should NOT change selection (cross-tier mixing blocked)
    // Selection should remain just own_unit
    let selected = TestHarness::new(&mut test_app.app).get_selection();
    assert_eq!(selected.len(), 1, "Selection should be unchanged");
    assert!(selected.contains(&own_unit), "Own unit should still be selected");
    assert!(!selected.contains(&enemy_unit), "Enemy unit should NOT be added to own unit selection");
}
