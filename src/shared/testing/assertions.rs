use bevy::prelude::*;
use crate::game::combat::types::{AttackPhase, AttackState};
use crate::game::types::objects::ObjectInstance;
use crate::game::world::types::FogOfWarMap;
use crate::types::{ObjectEnum, Selected, Selection, VisibilityStateEnum};
use crate::ui::types::{
    ObjectInterfaceState, CommandButtonAction, CommandButtonEnabled,
    GridSlot,
};

/// Assert that an entity's position is within `tolerance` of `expected`.
/// Panics with a descriptive message showing actual vs expected.
pub fn assert_position_near(world: &World, entity: Entity, expected: Vec3, tolerance: f32) {
    let actual = world
        .get::<Transform>(entity)
        .unwrap_or_else(|| panic!("Entity {entity:?} has no Transform component"))
        .translation;
    let dist = (actual - expected).length();
    assert!(
        dist <= tolerance,
        "Entity {entity:?}: position mismatch — expected {expected:?} (tolerance {tolerance}), got {actual:?} (distance: {dist:.3})"
    );
}

/// Assert that an entity's current HP equals `expected_hp` (within f32 epsilon).
/// Panics if the entity has no ObjectInstance or HP is None.
pub fn assert_health_equals(world: &World, entity: Entity, expected_hp: f32) {
    let obj = world
        .get::<ObjectInstance>(entity)
        .unwrap_or_else(|| panic!("Entity {entity:?} has no ObjectInstance component"));
    let actual_hp = obj
        .hp
        .unwrap_or_else(|| panic!("Entity {entity:?} has no HP (indestructible)"));
    let diff = (actual_hp - expected_hp).abs();
    assert!(
        diff < 0.01,
        "Entity {entity:?}: HP mismatch — expected {expected_hp:.2}, got {actual_hp:.2}"
    );
}

/// Assert that an entity's attack phase equals `expected`.
/// Panics if the entity has no AttackState.
pub fn assert_phase_equals(world: &World, entity: Entity, expected: AttackPhase) {
    let attack_state = world
        .get::<AttackState>(entity)
        .unwrap_or_else(|| panic!("Entity {entity:?} has no AttackState component"));
    assert_eq!(
        attack_state.phase, expected,
        "Entity {entity:?}: attack phase mismatch — expected {expected:?}, got {:?}",
        attack_state.phase
    );
}

/// Assert that an entity's BaseBehaviorState discriminant matches `expected`.
/// Uses `std::mem::discriminant` for variant-only comparison (ignores inner data).
pub fn assert_behavior_is(
    world: &World,
    entity: Entity,
    expected: &crate::game::units::types::state::BaseBehaviorState,
) {
    use crate::game::units::types::state::BaseBehaviorState;
    let actual = world
        .get::<BaseBehaviorState>(entity)
        .unwrap_or_else(|| panic!("Entity {entity:?} has no BaseBehaviorState component"));
    assert_eq!(
        std::mem::discriminant(actual),
        std::mem::discriminant(expected),
        "Entity {entity:?}: behavior mismatch — expected variant {:?}, got {:?}",
        expected,
        actual
    );
}

/// Assert that an entity is dead (HP <= 0 or entity no longer exists).
/// Panics if the entity is alive.
pub fn assert_dead(world: &World, entity: Entity) {
    match world.get_entity(entity) {
        Err(_) => return, // Entity despawned — considered dead
        Ok(entity_ref) => {
            if let Some(obj) = entity_ref.get::<ObjectInstance>() {
                if let Some(hp) = obj.hp {
                    assert!(
                        hp <= 0.0,
                        "Entity {entity:?}: expected dead but HP is {hp:.2}"
                    );
                } else {
                    panic!("Entity {entity:?}: expected dead but entity is indestructible (no HP)");
                }
            } else {
                panic!("Entity {entity:?}: expected dead but has no ObjectInstance");
            }
        }
    }
}

/// Assert that a tile is visible to a player.
/// Panics if the tile is not `VisibilityStateEnum::Visible`.
pub fn assert_visible(world: &World, player_id: u8, x: i32, z: i32) {
    let fog_map = world.resource::<FogOfWarMap>();
    let actual = fog_map.get(player_id, x, z);
    assert_eq!(
        actual,
        VisibilityStateEnum::Visible,
        "Tile ({x}, {z}) for player {player_id}: expected Visible, got {actual:?}"
    );
}

/// Assert that a faction's resource amount is at least `min_amount`.
/// `resource_field` is "space_crystals" or "supplies".
/// `player_id` is 0 for GDO, 1 for Syndicate.
pub fn assert_resource_at_least(world: &mut World, player_id: u8, resource_field: &str, min_amount: i32) {
    use crate::game::types::{Player, GdoPlayerResources, SyndicatePlayerResources};

    let mut found = false;
    let mut actual = 0i32;

    // Query Player entities to find the matching player
    let mut query_gdo = world.query::<(&Player, &GdoPlayerResources)>();
    for (player, res) in query_gdo.iter(world) {
        if player.player_number == player_id {
            actual = match resource_field {
                "space_crystals" => res.space_crystals,
                "supplies" => res.supplies,
                _ => panic!("Unknown resource field: {resource_field}"),
            };
            found = true;
            break;
        }
    }

    if !found {
        let mut query_syn = world.query::<(&Player, &SyndicatePlayerResources)>();
        for (player, res) in query_syn.iter(world) {
            if player.player_number == player_id {
                actual = match resource_field {
                    "space_crystals" => res.space_crystals,
                    "supplies" => res.supplies,
                    _ => panic!("Unknown resource field: {resource_field}"),
                };
                found = true;
                break;
            }
        }
    }

    assert!(
        found,
        "No player entity found with id {player_id}"
    );
    assert!(
        actual >= min_amount,
        "Player {player_id} {resource_field}: expected at least {min_amount}, got {actual}"
    );
}

/// Assert that the number of selected entities equals `expected`.
pub fn assert_selection_count(world: &mut World, expected: usize) {
    let mut query = world.query_filtered::<Entity, With<Selected>>();
    let actual = query.iter(world).count();
    assert_eq!(
        actual, expected,
        "Selection count mismatch — expected {expected}, got {actual}"
    );
}

/// Assert that the ObjectInterfaceState resource matches `expected`.
/// Panics with both values on mismatch.
pub fn assert_interface_state(world: &World, expected: ObjectInterfaceState) {
    let actual = world.resource::<ObjectInterfaceState>().clone();
    assert_eq!(
        actual, expected,
        "Interface state mismatch — expected {expected:?}, got {actual:?}"
    );
}

/// Assert that a command button with the given action exists at the specified grid slot.
/// Panics if no button with that action is at that slot.
pub fn assert_command_visible(world: &mut World, slot: (u8, u8), action: &CommandButtonAction) {
    let mut query = world.query::<(&CommandButtonAction, &GridSlot)>();
    let found = query.iter(world).any(|(a, gs)| {
        gs.row == slot.0 && gs.col == slot.1 && std::mem::discriminant(a) == std::mem::discriminant(action)
    });
    assert!(
        found,
        "Expected command {action:?} at slot ({}, {}), but none found",
        slot.0, slot.1
    );
}

/// Assert that no command button with the given action exists in the command panel.
/// Panics if a button with that action is found.
pub fn assert_command_not_visible(world: &mut World, action: &CommandButtonAction) {
    let mut query = world.query::<&CommandButtonAction>();
    let found = query.iter(world).any(|a| {
        std::mem::discriminant(a) == std::mem::discriminant(action)
    });
    assert!(
        !found,
        "Expected command {action:?} to NOT be visible, but it was found"
    );
}

/// Assert that the command button at the specified grid slot is enabled.
/// Panics if no button at that slot or if it is disabled.
pub fn assert_command_enabled(world: &mut World, slot: (u8, u8)) {
    let mut query = world.query::<(&GridSlot, &CommandButtonEnabled)>();
    let found = query.iter(world).find(|(gs, _)| gs.row == slot.0 && gs.col == slot.1);
    match found {
        None => panic!("No command button found at slot ({}, {})", slot.0, slot.1),
        Some((_, enabled)) => assert!(
            enabled.0,
            "Command button at slot ({}, {}) is disabled, expected enabled",
            slot.0, slot.1
        ),
    }
}

/// Assert that the active selection group's object type matches `expected`.
/// Panics if no active group or type mismatch.
pub fn assert_active_group_type(world: &World, expected: ObjectEnum) {
    let selection = world.resource::<Selection>();
    let actual = selection.active_type();
    assert_eq!(
        actual, Some(expected),
        "Active group type mismatch — expected Some({expected:?}), got {actual:?}"
    );
}

/// Assert that the given entity is selected and would be displayed in the info panel.
/// Checks that the entity is in the active group.
pub fn assert_info_panel_shows(world: &World, entity: Entity) {
    let selection = world.resource::<Selection>();
    let active_group = selection.active_group()
        .unwrap_or_else(|| panic!("No active selection group — cannot verify info panel for {entity:?}"));
    assert!(
        active_group.entities.contains(&entity),
        "Entity {entity:?} is not in the active selection group — info panel would not show it. Active group entities: {:?}",
        active_group.entities
    );
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn assert_position_near_passes_within_tolerance() {
        let mut world = World::new();
        let entity = world
            .spawn(Transform::from_xyz(10.0, 0.0, 5.0))
            .id();
        assert_position_near(&world, entity, Vec3::new(10.0, 0.0, 5.0), 0.01);
    }

    #[test]
    #[should_panic(expected = "position mismatch")]
    fn assert_position_near_panics_when_out_of_tolerance() {
        let mut world = World::new();
        let entity = world
            .spawn(Transform::from_xyz(10.0, 0.0, 5.0))
            .id();
        assert_position_near(&world, entity, Vec3::new(20.0, 0.0, 5.0), 1.0);
    }

    #[test]
    fn assert_health_equals_passes_at_full_hp() {
        let mut world = World::new();
        let entity = world
            .spawn(ObjectInstance::destructible(crate::types::ObjectEnum::Peacekeeper, 100.0))
            .id();
        assert_health_equals(&world, entity, 100.0);
    }

    #[test]
    #[should_panic(expected = "HP mismatch")]
    fn assert_health_equals_panics_on_mismatch() {
        let mut world = World::new();
        let entity = world
            .spawn(ObjectInstance::destructible(crate::types::ObjectEnum::Peacekeeper, 100.0))
            .id();
        assert_health_equals(&world, entity, 50.0);
    }

    #[test]
    fn assert_phase_equals_passes_on_match() {
        let mut world = World::new();
        let entity = world.spawn(AttackState::default()).id();
        assert_phase_equals(&world, entity, AttackPhase::None);
    }

    #[test]
    #[should_panic(expected = "attack phase mismatch")]
    fn assert_phase_equals_panics_on_mismatch() {
        let mut world = World::new();
        let entity = world.spawn(AttackState::default()).id();
        assert_phase_equals(&world, entity, AttackPhase::Aiming);
    }

    #[test]
    fn assert_behavior_is_passes_on_matching_variant() {
        use crate::game::units::types::state::BaseBehaviorState;
        let mut world = World::new();
        let entity = world.spawn(BaseBehaviorState::None).id();
        assert_behavior_is(&world, entity, &BaseBehaviorState::None);
    }

    #[test]
    #[should_panic(expected = "behavior mismatch")]
    fn assert_behavior_is_panics_on_variant_mismatch() {
        use crate::game::units::types::state::BaseBehaviorState;
        let mut world = World::new();
        let entity = world.spawn(BaseBehaviorState::None).id();
        assert_behavior_is(
            &world,
            entity,
            &BaseBehaviorState::TurnRate {
                planned_path: vec![],
                path_index: 0,
            },
        );
    }

    #[test]
    fn assert_dead_passes_for_zero_hp() {
        let mut world = World::new();
        let mut obj = ObjectInstance::destructible(crate::types::ObjectEnum::Peacekeeper, 100.0);
        obj.hp = Some(0.0);
        let entity = world.spawn(obj).id();
        assert_dead(&world, entity);
    }

    #[test]
    fn assert_dead_passes_for_despawned_entity() {
        let mut world = World::new();
        let entity = world.spawn(Transform::default()).id();
        world.despawn(entity);
        assert_dead(&world, entity);
    }

    #[test]
    #[should_panic(expected = "expected dead but HP is")]
    fn assert_dead_panics_for_alive_entity() {
        let mut world = World::new();
        let entity = world
            .spawn(ObjectInstance::destructible(crate::types::ObjectEnum::Peacekeeper, 100.0))
            .id();
        assert_dead(&world, entity);
    }

    #[test]
    fn assert_selection_count_passes_on_match() {
        let mut world = World::new();
        world.spawn(Selected);
        world.spawn(Selected);
        assert_selection_count(&mut world, 2);
    }

    #[test]
    #[should_panic(expected = "Selection count mismatch")]
    fn assert_selection_count_panics_on_mismatch() {
        let mut world = World::new();
        world.spawn(Selected);
        assert_selection_count(&mut world, 5);
    }

    #[test]
    fn assert_visible_passes_for_visible_tile() {
        let mut world = World::new();
        let mut fog_map = FogOfWarMap::new(64, 64);
        fog_map.ensure_player(0);
        fog_map.set(0, 10, 10, VisibilityStateEnum::Visible);
        world.insert_resource(fog_map);
        assert_visible(&world, 0, 10, 10);
    }

    #[test]
    #[should_panic(expected = "expected Visible, got Unexplored")]
    fn assert_visible_panics_for_unexplored_tile() {
        let mut world = World::new();
        let mut fog_map = FogOfWarMap::new(64, 64);
        fog_map.ensure_player(0);
        world.insert_resource(fog_map);
        assert_visible(&world, 0, 10, 10);
    }

    // === UI State assertion tests ===

    #[test]
    fn assert_interface_state_passes_on_match() {
        let mut world = World::new();
        world.insert_resource(ObjectInterfaceState::Default);
        assert_interface_state(&world, ObjectInterfaceState::Default);
    }

    #[test]
    #[should_panic(expected = "Interface state mismatch")]
    fn assert_interface_state_panics_on_mismatch() {
        use crate::ui::types::StructureMenuState;
        let mut world = World::new();
        world.insert_resource(ObjectInterfaceState::Default);
        assert_interface_state(
            &world,
            ObjectInterfaceState::StructureMenu(StructureMenuState::BarracksMenu),
        );
    }

    #[test]
    fn assert_command_visible_passes_when_button_exists() {
        use crate::ui::types::CommandButtonEnabled;
        let mut world = World::new();
        world.spawn((
            CommandButtonAction::UnitMove,
            GridSlot { row: 0, col: 0 },
            CommandButtonEnabled(true),
        ));
        assert_command_visible(&mut world, (0, 0), &CommandButtonAction::UnitMove);
    }

    #[test]
    #[should_panic(expected = "Expected command")]
    fn assert_command_visible_panics_when_missing() {
        let mut world = World::new();
        assert_command_visible(&mut world, (0, 0), &CommandButtonAction::UnitMove);
    }

    #[test]
    fn assert_command_not_visible_passes_when_absent() {
        let mut world = World::new();
        assert_command_not_visible(&mut world, &CommandButtonAction::UnitAttack);
    }

    #[test]
    #[should_panic(expected = "NOT be visible")]
    fn assert_command_not_visible_panics_when_present() {
        let mut world = World::new();
        world.spawn((
            CommandButtonAction::UnitAttack,
            GridSlot { row: 1, col: 0 },
        ));
        assert_command_not_visible(&mut world, &CommandButtonAction::UnitAttack);
    }

    #[test]
    fn assert_command_enabled_passes_for_enabled_button() {
        let mut world = World::new();
        world.spawn((
            CommandButtonAction::UnitMove,
            GridSlot { row: 0, col: 0 },
            CommandButtonEnabled(true),
        ));
        assert_command_enabled(&mut world, (0, 0));
    }

    #[test]
    #[should_panic(expected = "is disabled")]
    fn assert_command_enabled_panics_for_disabled_button() {
        let mut world = World::new();
        world.spawn((
            CommandButtonAction::UnitMove,
            GridSlot { row: 0, col: 0 },
            CommandButtonEnabled(false),
        ));
        assert_command_enabled(&mut world, (0, 0));
    }

    #[test]
    fn assert_active_group_type_passes_on_match() {
        use crate::types::SelectionGroup;
        let mut world = World::new();
        let entity = world.spawn_empty().id();
        world.insert_resource(Selection {
            groups: vec![SelectionGroup {
                object_type: ObjectEnum::Peacekeeper,
                entities: vec![entity],
            }],
            active_group_index: Some(0),
        });
        assert_active_group_type(&world, ObjectEnum::Peacekeeper);
    }

    #[test]
    #[should_panic(expected = "Active group type mismatch")]
    fn assert_active_group_type_panics_on_mismatch() {
        use crate::types::SelectionGroup;
        let mut world = World::new();
        let entity = world.spawn_empty().id();
        world.insert_resource(Selection {
            groups: vec![SelectionGroup {
                object_type: ObjectEnum::Peacekeeper,
                entities: vec![entity],
            }],
            active_group_index: Some(0),
        });
        assert_active_group_type(&world, ObjectEnum::DeploymentCenter);
    }

    #[test]
    fn assert_info_panel_shows_passes_when_entity_in_active_group() {
        use crate::types::SelectionGroup;
        let mut world = World::new();
        let entity = world.spawn_empty().id();
        world.insert_resource(Selection {
            groups: vec![SelectionGroup {
                object_type: ObjectEnum::Peacekeeper,
                entities: vec![entity],
            }],
            active_group_index: Some(0),
        });
        assert_info_panel_shows(&world, entity);
    }

    #[test]
    #[should_panic(expected = "not in the active selection group")]
    fn assert_info_panel_shows_panics_when_entity_not_in_group() {
        use crate::types::SelectionGroup;
        let mut world = World::new();
        let entity_a = world.spawn_empty().id();
        let entity_b = world.spawn_empty().id();
        world.insert_resource(Selection {
            groups: vec![SelectionGroup {
                object_type: ObjectEnum::Peacekeeper,
                entities: vec![entity_a],
            }],
            active_group_index: Some(0),
        });
        assert_info_panel_shows(&world, entity_b);
    }
}
