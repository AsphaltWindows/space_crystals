# Task 017: Fix Attack Command Routing Bug

## Status
**Completed** - 2026-02-01

Implementation Notes:
- Fixed the bug in `src/units.rs` at lines 559-563 where attack commands were not properly routed
- The attack-on-enemy logic (lines 434-481) was already functional for detecting enemy clicks in Attack mode
- Replaced the stub code with proper handling for both `CommandType::Attack` and `CommandType::AttackGround`
- For Attack mode clicking on ground (no enemy): now logs informative message and returns to Default mode
- For AttackGround mode: implemented full functionality with pathfinding to target location and issuing `UnitCommand::AttackLocation(target_pos)`

## Description
Fix the bug preventing units from attacking. The attack system is fully implemented in the combat module, but the command routing in the units module is not connected. When a player uses Attack mode (A key) and clicks on an enemy unit, the code currently just prints "Attack commands not yet implemented" and returns to default mode instead of actually issuing the AttackTarget command to the selected units.

## Why Needed
This is a critical bug that prevents units from being able to attack enemies when commanded by the player. The underlying attack mechanics (attack phases, damage application, projectiles, etc.) are all working correctly - only the UI command routing is broken. Without this fix, players cannot control when their units attack.

## Acceptance Criteria
- Pressing 'A' key enters Attack command mode
- Left-clicking on an enemy unit while in Attack mode issues AttackTarget command to all selected units
- Units move toward and attack the targeted enemy
- Attack mode returns to Default mode after issuing the command
- Console logs confirm attack command was issued with unit count
- The existing auto-target system continues to work for idle units
- Attack commands work correctly for all unit types with AttackCapability

## Relevant Files/Components
- `src/units.rs` - Lines 559-563 contain the bug (attack command handling in right_click_move_command system)
- `src/commands.rs` - UnitCommand::AttackTarget enum variant (already exists)
- `src/combat.rs` - Attack systems (already fully implemented, no changes needed)

## Technical Considerations

**Current Bug Location**:
In `src/units.rs`, the `right_click_move_command` system at lines 559-563:
```rust
CommandType::Attack | CommandType::AttackGround => {
    // Attack commands - not implemented yet (Task #6)
    info!("Attack commands not yet implemented (Task #6)");
    command_mode.mode = CommandType::Default;
}
```

**The Fix**:
This code block should be replaced with the actual attack command logic. The system already has code for detecting enemy unit clicks at lines 434-481, which properly handles the Attack mode case. The issue is that this code only runs BEFORE the raycast to ground, but the fallback case at lines 559-563 overrides it.

**Implementation Approach**:
1. Remove lines 559-563 (the stub code that prints "not implemented")
2. The attack-on-enemy logic at lines 434-481 already works correctly
3. For AttackGround mode (CommandType::AttackGround), implement attack-location command using UnitCommand::AttackLocation(Vec3)
4. Ensure command mode resets to Default after issuing attack commands

**Attack Ground Implementation** (if including it in this task):
- Cast ray to ground plane to get target location
- Issue UnitCommand::AttackLocation(target_pos) to selected units
- The combat system will need to handle AttackLocation commands (may require additional work in combat.rs)
- For now, can implement just the Attack (target unit) command and leave AttackGround as a separate task

**Testing Strategy**:
1. Select friendly units (Player 0)
2. Press 'A' key to enter Attack mode
3. Click on enemy unit (Player 1)
4. Verify units move to attack the target
5. Verify damage is applied when in range
6. Test with different unit types

## Prerequisites
None - This is a bug fix for existing functionality

## Complexity
Simple

## Notes
- The attack system in combat.rs is fully functional (attack phases, damage, projectiles, AOE, etc.)
- The auto-target system works correctly for idle units
- Only the player-commanded attack routing is broken
- This is specifically the bug mentioned in the comment "Task #6" which likely referred to an older task numbering
- AttackGround can be implemented in a separate task if needed for complexity management
