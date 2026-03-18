use crate::helpers::*;

/// QA Steps 1-2 [auto]: Spawn a SupplyChopper (air unit) and move it over a structure.
/// Verify the air unit accepts the move command and is not blocked by the structure.
/// Air units exist in a different domain and should pass freely over structures.
#[test]
fn steps_1_2_air_unit_passes_over_structure() {
    let mut test_app = TestApp::new();
    test_app.step();

    let air_unit;
    {
        let mut h = TestHarness::new(&mut test_app.app);
        air_unit = h.spawn_unit_at_grid(ObjectEnum::SupplyChopper, 20, 20, Owner(Some(0)));
        // Place a structure directly between the air unit and its target
        h.spawn_structure_at_grid(ObjectEnum::PowerPlant, 22, 20, Owner(Some(0)));
    }
    test_app.step();

    let air_start = TestHarness::new(&mut test_app.app).get_position(air_unit).unwrap();

    // Order air unit to move to a position beyond the structure
    {
        let mut h = TestHarness::new(&mut test_app.app);
        h.issue_command(
            air_unit,
            UnitCommand::Move(Vec3::new(air_start.x + 8.0, air_start.y, air_start.z)),
        );
    }

    test_app.step_n(5);

    // The air unit should have accepted the move command (not rejected due to structure blocking)
    let h = TestHarness::new(&mut test_app.app);
    assert!(h.is_alive(air_unit), "Air unit should still be alive");

    let cmd = h.get_command(air_unit);
    assert!(
        matches!(cmd, Some(UnitCommand::Move(_))),
        "Air unit should accept Move command over a structure, got {:?}",
        cmd
    );
}

/// QA Steps 3-4 [auto]: Spawn a SupplyChopper and move it through a group of ground units.
/// Verify the air unit accepts the move command and is not blocked by ground units.
#[test]
fn steps_3_4_air_unit_passes_through_ground_units() {
    let mut test_app = TestApp::new();
    test_app.step();

    let air_unit;
    {
        let mut h = TestHarness::new(&mut test_app.app);
        air_unit = h.spawn_unit_at_grid(ObjectEnum::SupplyChopper, 20, 20, Owner(Some(0)));
        // Place a cluster of ground units between the air unit and its target
        h.spawn_unit_at_grid(ObjectEnum::Peacekeeper, 22, 20, Owner(Some(0)));
        h.spawn_unit_at_grid(ObjectEnum::Peacekeeper, 22, 21, Owner(Some(0)));
        h.spawn_unit_at_grid(ObjectEnum::Peacekeeper, 23, 20, Owner(Some(0)));
        h.spawn_unit_at_grid(ObjectEnum::Peacekeeper, 23, 21, Owner(Some(0)));
    }
    test_app.step();

    let air_start = TestHarness::new(&mut test_app.app).get_position(air_unit).unwrap();

    // Order the air unit to move to a position beyond the ground unit cluster
    {
        let mut h = TestHarness::new(&mut test_app.app);
        h.issue_command(
            air_unit,
            UnitCommand::Move(Vec3::new(air_start.x + 8.0, air_start.y, air_start.z)),
        );
    }

    test_app.step_n(5);

    // The air unit should have accepted the move command without being blocked by ground units
    let h = TestHarness::new(&mut test_app.app);
    assert!(h.is_alive(air_unit), "Air unit should still be alive");

    let cmd = h.get_command(air_unit);
    assert!(
        matches!(cmd, Some(UnitCommand::Move(_))),
        "Air unit should accept Move command through ground units, got {:?}",
        cmd
    );
}

/// QA Steps 5-6 [auto]: Spawn two SupplyChopper units close together (within separation radius).
/// Verify they do not perfectly stack — the separation system should push them apart.
/// SeparationRadius for SupplyChopper is 1.25, so units at adjacent grid positions (1 unit apart)
/// are within the threshold and should experience repulsion.
#[test]
fn steps_5_6_two_air_units_separate_when_close() {
    let mut test_app = TestApp::new();
    test_app.step();

    let u1;
    let u2;
    {
        let mut h = TestHarness::new(&mut test_app.app);
        // Spawn at adjacent grid positions — within the 1.25 separation radius
        u1 = h.spawn_unit_at_grid(ObjectEnum::SupplyChopper, 20, 20, Owner(Some(0)));
        u2 = h.spawn_unit_at_grid(ObjectEnum::SupplyChopper, 21, 20, Owner(Some(0)));
    }
    test_app.step();

    let pos1_before = TestHarness::new(&mut test_app.app).get_position(u1).unwrap();
    let pos2_before = TestHarness::new(&mut test_app.app).get_position(u2).unwrap();

    let dist_before = ((pos1_before.x - pos2_before.x).powi(2)
        + (pos1_before.z - pos2_before.z).powi(2))
    .sqrt();

    // Let the separation system run for several frames to push them apart
    test_app.step_n(30);

    let pos1_after = TestHarness::new(&mut test_app.app).get_position(u1).unwrap();
    let pos2_after = TestHarness::new(&mut test_app.app).get_position(u2).unwrap();

    let dist_after = ((pos1_after.x - pos2_after.x).powi(2)
        + (pos1_after.z - pos2_after.z).powi(2))
    .sqrt();

    // The units should have drifted apart — distance after should be greater than before
    assert!(
        dist_after > dist_before,
        "Close air units should drift apart via separation. Before: {}, After: {}",
        dist_before,
        dist_after
    );
}

/// QA Steps 7-8 [auto]: Move one air unit directly through another air unit's position.
/// Verify the units can temporarily overlap during movement but separate when both stop.
#[test]
fn steps_7_8_air_units_overlap_during_movement_then_separate() {
    let mut test_app = TestApp::new();
    test_app.step();

    let u1;
    let u2;
    {
        let mut h = TestHarness::new(&mut test_app.app);
        u1 = h.spawn_unit_at_grid(ObjectEnum::SupplyChopper, 20, 20, Owner(Some(0)));
        u2 = h.spawn_unit_at_grid(ObjectEnum::SupplyChopper, 23, 20, Owner(Some(0)));
    }
    test_app.step();

    let u2_pos = TestHarness::new(&mut test_app.app).get_position(u2).unwrap();

    // Move u1 to u2's exact position — air units should be able to overlap temporarily
    {
        let mut h = TestHarness::new(&mut test_app.app);
        h.issue_command(u1, UnitCommand::Move(u2_pos));
    }

    // Let u1 travel toward u2's location
    test_app.step_n(40);

    // Now let the separation system settle them
    test_app.step_n(30);

    let pos1 = TestHarness::new(&mut test_app.app).get_position(u1).unwrap();
    let pos2 = TestHarness::new(&mut test_app.app).get_position(u2).unwrap();

    let dist = ((pos1.x - pos2.x).powi(2) + (pos1.z - pos2.z).powi(2)).sqrt();

    // After settling, the units should have separated somewhat (not perfectly stacked)
    assert!(
        dist > 0.1,
        "Air units should separate after both stop. Final distance: {}",
        dist
    );
}

/// QA Steps 9-10 [auto]: Spawn 6 air units and group-move them to one spot.
/// Verify they spread out around the target rather than stacking, forming a loose cluster.
#[test]
fn steps_9_10_group_of_air_units_spread_out() {
    let mut test_app = TestApp::new();
    test_app.step();

    let mut units = Vec::new();
    {
        let mut h = TestHarness::new(&mut test_app.app);
        // Spawn 6 air units at various nearby positions
        units.push(h.spawn_unit_at_grid(ObjectEnum::SupplyChopper, 18, 18, Owner(Some(0))));
        units.push(h.spawn_unit_at_grid(ObjectEnum::SupplyChopper, 18, 20, Owner(Some(0))));
        units.push(h.spawn_unit_at_grid(ObjectEnum::SupplyChopper, 18, 22, Owner(Some(0))));
        units.push(h.spawn_unit_at_grid(ObjectEnum::SupplyChopper, 20, 18, Owner(Some(0))));
        units.push(h.spawn_unit_at_grid(ObjectEnum::SupplyChopper, 20, 20, Owner(Some(0))));
        units.push(h.spawn_unit_at_grid(ObjectEnum::SupplyChopper, 20, 22, Owner(Some(0))));
    }
    test_app.step();

    // Get a common rally point
    let rally_pos = TestHarness::new(&mut test_app.app).get_position(units[4]).unwrap();

    // Order all units to move to the same rally point
    {
        let mut h = TestHarness::new(&mut test_app.app);
        for &unit in &units {
            h.issue_command(unit, UnitCommand::Move(rally_pos));
        }
    }

    // Let them converge and then let the separation system spread them out
    test_app.step_n(60);

    // Collect final positions
    let h = TestHarness::new(&mut test_app.app);
    let positions: Vec<Vec3> = units
        .iter()
        .map(|&u| h.get_position(u).expect("Unit should be alive"))
        .collect();

    // Compute pairwise distances to verify they are not all stacked
    let mut min_dist = f32::MAX;
    let mut max_dist: f32 = 0.0;
    for i in 0..positions.len() {
        for j in (i + 1)..positions.len() {
            let d = ((positions[i].x - positions[j].x).powi(2)
                + (positions[i].z - positions[j].z).powi(2))
            .sqrt();
            if d < min_dist {
                min_dist = d;
            }
            if d > max_dist {
                max_dist = d;
            }
        }
    }

    // The units should form a spread cluster, not a single point.
    // Max pairwise distance should be non-trivial, indicating spread.
    assert!(
        max_dist > 0.3,
        "Group of air units should spread out, not stack. Max pairwise distance: {}",
        max_dist
    );

    // No two units should be perfectly overlapping
    assert!(
        min_dist > 0.01,
        "No two air units should perfectly overlap. Min pairwise distance: {}",
        min_dist
    );
}
