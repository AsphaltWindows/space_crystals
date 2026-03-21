# common-command-classification-fix

## Metadata
- **From**: task_planner
- **To**: developer

## Content

## Parent Feature

task_splitter-common-vs-group-commands.md

## Task

Fix the CommonCommand vs GroupCommand classification logic in `src/ui/command_panel.rs`.

**Current bug:** `is_common_command()` (line ~2128) uses a hardcoded whitelist of command actions (Move, Patrol, HoldPosition, Stop, AgentBuildTunnel, AgentDropOff) that are always considered common when no structures are selected. All other commands (Attack, AttackGround, Reverse) are hardcoded as never-common. This is wrong.

**Required behavior:** A command is a CommonCommand if and only if EVERY SelectionGroup in the Selection supports that command. If even one group does not support it, it is a GroupCommand (only shown/issued when its supporting group is the ActiveGroup).

**Implementation approach:**
1. Add a helper function (or method on ObjectEnum/SelectionGroup) that determines which `CommandButtonAction` variants a given `ObjectEnum` type supports.
2. Rewrite `is_common_command()` to iterate over all groups in the Selection and check if every group's `object_type` supports the given action.
3. Update `SelectedUnitCapabilities` computation in `update_command_panel_state()` so that when determining which buttons to show in the grid, it uses the **active group's** capabilities (not aggregate of all selected units).
4. The `command_target_entities()` function already correctly uses `is_common_command` to decide whether to issue to all selected or just active group — this should work correctly once `is_common_command` is fixed.

**Tests to add/update:**
- Test: selecting only Peacekeepers -> Attack is common
- Test: selecting Peacekeepers + Supply Chopper -> Attack is NOT common, Stop IS common
- Test: all unit-only commands (Move, Stop, HoldPosition, Patrol) common when all groups are units
- Update existing `attack_is_not_common_command` test (currently asserts always false — should be conditionally common)

## Technical Context

### Files to Change

1. **`src/ui/command_panel.rs`** — Primary file. Contains all functions that need modification:
   - `is_common_command()` (line 2128): Current hardcoded whitelist approach must be replaced with per-ObjectEnum capability checking
   - `compute_selected_unit_capabilities()` (line 1619): Already filters by active group entities — this is CORRECT behavior, no change needed here
   - `get_grid_slot_action()` (line 41): The grid layout function uses `caps: &SelectedUnitCapabilities` to conditionally show Attack (line 147), Reverse (line 145), AttackGround (line 149). Since `compute_selected_unit_capabilities` already filters to active group, this should work correctly after `is_common_command` is fixed.
   - `command_target_entities()` (line 2157): Uses `is_common_command()` to decide all-selected vs active-group dispatch — no changes needed, will automatically benefit from the fix
   - `rebuild_command_panel_ui()` (line 429): Spawns grid buttons with `is_common` flag at line 786: `let is_common = selection.groups.len() <= 1 || is_common_command(&action, &selection);` — no change needed, this drives button tinting (green=common, yellow=group)

2. **`src/ui/types.rs`** — `SelectedUnitCapabilities` struct (line 368) and `CommandButtonAction` enum (line 240). May not need changes unless you want to add a method to `CommandButtonAction`.

3. **`src/shared/types.rs`** — `Selection` (line 138), `SelectionGroup` (line 129), `ObjectEnum` (line 316). Could add a `supports_action()` method to `ObjectEnum` here, or keep the helper in command_panel.rs.

4. **`src/game/types/objects.rs`** — `ObjectEnum` impl block with `is_structure()` (line 359), `is_unit()` (line 364). Alternative location for the `supports_action()` method if you want it close to the ObjectEnum definition.

### Key Implementation Details

**New helper function — `object_type_supports_action(obj: &ObjectEnum, action: &CommandButtonAction) -> bool`:**

Create this in `command_panel.rs` (keeps the dependency on `CommandButtonAction` within the UI module). The mapping should be:

| ObjectEnum | Supported unit CommandButtonActions |
|---|---|
| `Peacekeeper` | UnitMove, UnitAttack, UnitAttackGround (has TailDisjointed attack), UnitAttackMove, UnitPatrol, UnitHoldPosition, UnitStop, UnitReverse (HeavyInfantry, can_reverse=true per unit_data) |
| `SupplyChopper` | UnitMove, UnitPatrol, UnitHoldPosition, UnitStop (NO attack — spawn at utils.rs:928 explicitly skips AttackCapability) |
| `SyndicateAgent` | UnitMove, UnitPatrol, UnitHoldPosition, UnitStop, UnitAttack (has melee attack), AgentBuildTunnel, AgentDropOff, UnitGather, UnitEnter |
| `SyndicateGuard` | UnitMove, UnitAttack, UnitPatrol, UnitHoldPosition, UnitStop, UnitEnter (ranged, HeavyInfantry) |
| Any structure | No unit commands supported |
| Any resource | No unit commands supported |

Note: `UnitReverse` — only Peacekeeper has `can_reverse=true` (HeavyInfantry unit base). SyndicateGuard is also HeavyInfantry but check `guard_type_data().unit_base.data().can_reverse` to confirm.

**Rewritten `is_common_command()`:**
```rust
fn is_common_command(action: &CommandButtonAction, selection: &Selection) -> bool {
    // Structure/resource-specific commands are never common across unit groups
    if !is_unit_action(action) {
        return false;
    }
    // A unit command is common iff every group supports it
    selection.groups.iter().all(|g| object_type_supports_action(&g.object_type, action))
}
```

Where `is_unit_action()` returns true for all `Unit*` and `Agent*` variants of `CommandButtonAction`.

### Existing Patterns to Follow

- **Test helpers**: Use existing `units_only_selection()` (line 2468), `mixed_unit_structure_selection()` (line 2485), `structures_only_selection()` (line 2502) for test fixtures
- **Entity creation in tests**: `Entity::from_raw_u32(N).unwrap()` (used throughout test module)
- **SelectedUnitCapabilities fixtures**: `attack_only()` helper at line 2518

### Existing Tests to Update

- `attack_is_not_common_command` (line 2788): Currently asserts Attack is NEVER common. After fix, Attack should be common when ALL groups support it (e.g., Peacekeeper + SyndicateGuard). Update to test with a selection where all groups have attack.
- `reverse_is_not_common_command` (line 2794): Similar — Reverse should be common when ALL groups support reverse.
- `agent_commands_are_common_with_units_only` (line 3258): Currently asserts AgentBuildTunnel/AgentDropOff are common with Peacekeeper+SyndicateAgent selection. After fix, these should NOT be common since Peacekeeper doesn't support AgentBuildTunnel. Update this test.

### New Tests to Add

- Selection of only Peacekeepers: Attack IS common (single group => always common via `groups.len() <= 1` check at line 786)
- Selection of Peacekeeper + SupplyChopper: Attack is NOT common (SupplyChopper has no attack)
- Selection of Peacekeeper + SupplyChopper: Move, Stop, HoldPosition, Patrol ARE common (both support these)
- Selection of Peacekeeper + SyndicateGuard: Attack IS common (both have attack)
- Selection of Peacekeeper + SyndicateAgent: AgentBuildTunnel is NOT common (Peacekeeper doesn't support it)
- Selection of SyndicateAgent + SyndicateAgent (2 ungrouped): AgentBuildTunnel IS common

### System Ordering — No Changes Needed

The existing system order is correct:
1. `update_command_panel_state` computes `SelectedUnitCapabilities` from active group
2. `rebuild_command_panel_ui` reads `SelectedUnitCapabilities` + `Selection` to build grid with `is_common` flags
3. `command_target_entities` uses `is_common_command` at command-issue time to determine targets

All these run in the same frame cycle and the data flow is correct.

## Dependencies

None — this is a standalone fix within the UI module. It only reads from `Selection` (shared resource) and `ObjectEnum` (shared type), both of which are stable.
