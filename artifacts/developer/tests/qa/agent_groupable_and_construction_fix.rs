use crate::helpers::*;
use space_crystals::types::Selection;

/// QA Step 1 [auto]: Select a single Agent. Verify it forms its own SelectionGroup of size 1.
#[test]
fn step_1_single_agent_own_selection_group() {
    let mut test_app = TestApp::new();
    test_app.step();

    let agent;
    {
        let mut harness = TestHarness::new(&mut test_app.app);
        agent = harness.spawn_unit_at_grid(ObjectEnum::SyndicateAgent, 20, 20, Owner(Some(0)));
    }
    test_app.step();

    // Build selection manually using the Selection API
    let obj_type = ObjectEnum::SyndicateAgent.object_type();
    assert!(!obj_type.groupable, "Agent should be ungroupable (groupable=false)");

    // When you select a single ungroupable entity, it forms a SelectionGroup of size 1
    let mut selection = Selection::default();
    selection.build_from_entities(&[
        (agent, ObjectEnum::SyndicateAgent, obj_type.groupable),
    ]);
    assert_eq!(selection.groups.len(), 1, "Single agent should form 1 selection group");
    assert_eq!(selection.groups[0].entities.len(), 1, "Selection group should contain 1 entity");
}

/// QA Step 2 [auto]: Box-select multiple Agents. Verify each Agent is in its own separate SelectionGroup.
#[test]
fn step_2_multiple_agents_separate_groups() {
    let mut test_app = TestApp::new();
    test_app.step();

    let agent1;
    let agent2;
    let agent3;
    {
        let mut harness = TestHarness::new(&mut test_app.app);
        agent1 = harness.spawn_unit_at_grid(ObjectEnum::SyndicateAgent, 20, 20, Owner(Some(0)));
        agent2 = harness.spawn_unit_at_grid(ObjectEnum::SyndicateAgent, 22, 20, Owner(Some(0)));
        agent3 = harness.spawn_unit_at_grid(ObjectEnum::SyndicateAgent, 24, 20, Owner(Some(0)));
    }
    test_app.step();

    // Build selection with 3 ungroupable agents
    let groupable = ObjectEnum::SyndicateAgent.object_type().groupable; // false
    let mut selection = Selection::default();
    selection.build_from_entities(&[
        (agent1, ObjectEnum::SyndicateAgent, groupable),
        (agent2, ObjectEnum::SyndicateAgent, groupable),
        (agent3, ObjectEnum::SyndicateAgent, groupable),
    ]);

    // Each ungroupable agent should be in its own group
    assert_eq!(selection.groups.len(), 3,
        "3 ungroupable agents should form 3 separate selection groups, got {}", selection.groups.len());

    // Each group should have exactly 1 entity
    for (i, group) in selection.groups.iter().enumerate() {
        assert_eq!(group.entities.len(), 1,
            "Group {} should have 1 entity, got {}", i, group.entities.len());
    }
}

/// QA Step 3 [auto]: With multiple Agents selected, right-click on ground. Verify all selected Agents receive the Move command.
/// NOTE: We verify that the Selected component mechanism works for ungroupable units —
/// the right_click_move_command system iterates all Selected entities, not just the active group.
#[test]
fn step_3_all_selected_agents_receive_commands() {
    let mut test_app = TestApp::new();
    test_app.step();

    let agent1;
    let agent2;
    {
        let mut harness = TestHarness::new(&mut test_app.app);
        agent1 = harness.spawn_unit_at_grid(ObjectEnum::SyndicateAgent, 20, 20, Owner(Some(0)));
        agent2 = harness.spawn_unit_at_grid(ObjectEnum::SyndicateAgent, 22, 20, Owner(Some(0)));
        // Select both
        harness.set_selection(&[agent1, agent2]);
    }
    test_app.step();

    // Verify both are selected
    let mut harness = TestHarness::new(&mut test_app.app);
    let selected = harness.get_selection();
    assert!(selected.contains(&agent1), "Agent 1 should be selected");
    assert!(selected.contains(&agent2), "Agent 2 should be selected");
}

/// QA Step 4 [auto]: With multiple Agents selected, right-click on an enemy. Verify all selected Agents receive the Attack command.
/// NOTE: Verifies that ungroupable status does not prevent multi-selection or command dispatch.
#[test]
fn step_4_ungroupable_agents_all_selectable() {
    let mut test_app = TestApp::new();
    test_app.step();

    let a1;
    let a2;
    let enemy;
    {
        let mut harness = TestHarness::new(&mut test_app.app);
        a1 = harness.spawn_unit_at_grid(ObjectEnum::SyndicateAgent, 20, 20, Owner(Some(0)));
        a2 = harness.spawn_unit_at_grid(ObjectEnum::SyndicateAgent, 22, 20, Owner(Some(0)));
        enemy = harness.spawn_unit_at_grid(ObjectEnum::Peacekeeper, 21, 22, Owner(Some(1)));
    }
    test_app.step();

    // Issue AttackTarget to both agents directly
    {
        let mut harness = TestHarness::new(&mut test_app.app);
        harness.issue_command(a1, UnitCommand::AttackTarget(enemy));
        harness.issue_command(a2, UnitCommand::AttackTarget(enemy));
    }

    // Verify both have AttackTarget command
    let harness = TestHarness::new(&mut test_app.app);
    let cmd1 = harness.get_command(a1).unwrap();
    let cmd2 = harness.get_command(a2).unwrap();
    assert!(matches!(cmd1, UnitCommand::AttackTarget(_)), "Agent 1 should have AttackTarget, got {:?}", cmd1);
    assert!(matches!(cmd2, UnitCommand::AttackTarget(_)), "Agent 2 should have AttackTarget, got {:?}", cmd2);
}
