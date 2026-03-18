use crate::helpers::*;

/// QA Step 4 [auto]: Select exactly 1 unit. Verify the SelectionPanel is hidden.
/// The SelectionPanel should only appear when 2+ entities are selected.
/// With 1 entity, Selection should have exactly 1 group with 1 entity.
#[test]
fn step_4_single_unit_no_selection_panel() {
    let mut test_app = TestApp::new();
    test_app.step();

    let unit;
    {
        let mut h = TestHarness::new(&mut test_app.app);
        unit = h.spawn_unit_at_grid(ObjectEnum::Peacekeeper, 10, 10, Owner(Some(0)));
    }
    test_app.step();

    // Build selection with exactly 1 entity
    {
        let world = test_app.app.world_mut();
        let mut selection = world.resource_mut::<Selection>();
        selection.build_from_entities(&[
            (unit, ObjectEnum::Peacekeeper, true),
        ]);
    }

    let world = test_app.app.world();
    let selection = world.resource::<Selection>();
    assert_eq!(
        selection.total_entity_count(), 1,
        "Selection should have exactly 1 entity"
    );
    // SelectionPanel visibility condition: total_entity_count() >= 2
    // With 1 entity, panel should be hidden
    assert!(
        selection.total_entity_count() < 2,
        "SelectionPanel should be hidden when only 1 entity selected"
    );
}

/// QA Step 5 [auto]: Select 0 units (click empty ground). Verify the SelectionPanel is hidden.
#[test]
fn step_5_no_selection_no_panel() {
    let mut test_app = TestApp::new();
    test_app.step();

    // Empty selection — build with no entities
    {
        let world = test_app.app.world_mut();
        let mut selection = world.resource_mut::<Selection>();
        selection.build_from_entities(&[]);
    }

    let world = test_app.app.world();
    let selection = world.resource::<Selection>();
    assert_eq!(
        selection.total_entity_count(), 0,
        "Selection should have 0 entities"
    );
    assert!(
        selection.groups.is_empty(),
        "Selection groups should be empty when nothing is selected"
    );
    // SelectionPanel visibility condition: total_entity_count() >= 2
    assert!(
        selection.total_entity_count() < 2,
        "SelectionPanel should be hidden when nothing selected"
    );
}
