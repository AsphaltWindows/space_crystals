# common-command-classification-fix

## Metadata
- **From**: task_splitter
- **To**: task_planner

## Content

## Parent Feature

task_splitter-common-vs-group-commands.md

## Task

Fix the CommonCommand vs GroupCommand classification logic in `src/ui/command_panel.rs`.

**Current bug:** `is_common_command()` (line ~2128) uses a hardcoded whitelist of command actions (Move, Patrol, HoldPosition, Stop, AgentBuildTunnel, AgentDropOff) that are always considered common when no structures are selected. All other commands (Attack, AttackGround, Reverse) are hardcoded as never-common. This is wrong.

**Required behavior:** A command is a CommonCommand if and only if EVERY SelectionGroup in the Selection supports that command. If even one group does not support it, it is a GroupCommand (only shown/issued when its supporting group is the ActiveGroup).

**Implementation approach:**
1. Add a helper function (or method on ObjectEnum/SelectionGroup) that determines which `CommandButtonAction` variants a given `ObjectEnum` type supports. For example, `ObjectEnum::Peacekeeper` supports Attack (has AttackCapability), but `ObjectEnum::SupplyChopper` does not.
2. Rewrite `is_common_command()` to iterate over all groups in the Selection and check if every group's `object_type` supports the given action. Return true only if all groups support it.
3. Update `SelectedUnitCapabilities` computation in `update_command_panel_state()` (~line 284) so that when determining which buttons to show in the grid, it uses the **active group's** capabilities (not aggregate of all selected units). This ensures that when you Tab to Supply Chopper, Attack button does not appear; when you Tab to Peacekeeper, Attack appears as a GroupCommand.
4. The `command_target_entities()` function (line ~2157) already correctly uses `is_common_command` to decide whether to issue to all selected or just active group — this should work correctly once `is_common_command` is fixed.

**Key files:**
- `src/ui/command_panel.rs`: `is_common_command()`, `update_command_panel_state()`, grid button layout
- `src/ui/types.rs`: `SelectedUnitCapabilities`, `CommandButtonAction`
- `src/shared/types.rs`: `Selection`, `SelectionGroup`, `ObjectEnum`

**Tests to add/update:**
- Test: selecting only Peacekeepers → Attack is common
- Test: selecting Peacekeepers + Supply Chopper → Attack is NOT common, Stop IS common
- Test: all unit-only commands (Move, Stop, HoldPosition, Patrol) common when all groups are units
- Update existing `attack_is_not_common_command` test (currently asserts always false — should be conditionally common)
