# Ticket: Fix CommonCommand vs GroupCommand Classification for Mixed Unit+Structure Selections

## Current State
When a unit and a structure are selected together (with the unit as the ActiveGroup), the commands Move, Stop, HoldPosition, and Patrol appear as CommonCommands (visually distinguished as shared by all selected entities). However, structures do not support any of these commands — only Attack and AttackMove are correctly classified as GroupCommands.

## Desired State
When determining CommonCommands vs GroupCommands, a command should only be classified as Common if **every object in the Selection** can execute it. Since structures cannot execute Move, Stop, HoldPosition, or Patrol, these must appear as GroupCommands (specific to the ActiveGroup) in any mixed unit+structure selection.

## Justification
`features/control_system.md` defines:
- **CommonCommands**: "available to every object in Selection"
- **GroupCommands**: "available to objects of type ActiveGroup"

The current `is_common_command()` logic in `src/ui/command_panel.rs` incorrectly classifies unit-only commands as common when structures are also selected. This violates the spec and misleads the player about which entities will execute which commands.

Originated from forum topic: `common_vs_group_command_classification_wrong.md`

## Technical Context

### Root Cause
`is_common_command()` at `src/ui/command_panel.rs:1993` is a static function that hardcodes certain `CommandButtonAction` variants as "common" based solely on action type, without considering the actual selection composition:

```rust
fn is_common_command(action: &CommandButtonAction) -> bool {
    matches!(action,
        CommandButtonAction::UnitMove |
        CommandButtonAction::UnitPatrol |
        CommandButtonAction::UnitHoldPosition |
        CommandButtonAction::UnitStop |
        CommandButtonAction::AgentBuildTunnel |
        CommandButtonAction::AgentDropOff
    )
}
```

This is consumed in two places:
1. **Visual rendering** at line 809: `let is_common = selection.groups.len() <= 1 || is_common_command(&action);` — determines green (common) vs yellow (group-specific) button tinting
2. **Command dispatch** at `command_target_entities()` (line 2016): decides whether to send the command to all selected units or only the active group's entities

### Fix Approach
Change `is_common_command()` to accept `&Selection` and check whether the selection contains groups of different categories (units vs structures). A command is only "common" if it could apply to ALL groups.

**Key type helpers available:**
- `ObjectEnum::is_unit()` at `src/game/types/objects.rs:364` — returns true for Peacekeeper, SupplyChopper, SyndicateAgent, SyndicateGuard
- `ObjectEnum::is_structure()` at `src/game/types/objects.rs:359` — returns true for structures (DC, PP, BK, EF, EP, ST, Tunnel, HQ)
- `Selection.groups: Vec<SelectionGroup>` — each group has `object_type: ObjectEnum`

**Logic:**
- If all groups are unit types → unit commands (Move, Stop, etc.) ARE common across them
- If any group is a structure type → unit commands are NOT common (structures can't execute them)
- `AgentBuildTunnel` and `AgentDropOff` are agent-specific commands that are also unit commands, so they follow the same rule
- No structure command is ever common (structure commands are type-specific: BkTrain, DcBuild, etc.)
- When `groups.len() <= 1`, the short-circuit at line 809 already handles this correctly (all commands treated as common)

**Suggested signature change:**
```rust
fn is_common_command(action: &CommandButtonAction, selection: &Selection) -> bool
```

### Files to Change
1. **`src/ui/command_panel.rs:1993`** — Update `is_common_command()` signature and logic
2. **`src/ui/command_panel.rs:809`** — Update call site (pass `&selection`)
3. **`src/ui/command_panel.rs:2021`** — Update call site in `command_target_entities()` (needs `&Selection` parameter added)
4. **`src/ui/command_panel.rs:2016`** — Update `command_target_entities()` signature to accept `&Selection`
5. **Tests** starting at line 2564 — Update all `is_common_command` tests to pass a `Selection` parameter. Tests should cover:
   - Single unit group → unit commands are common (existing behavior preserved)
   - Mixed unit+structure groups → unit commands are NOT common
   - Structure-only selection → structure commands remain not common

### Visual Rendering Path
`spawn_grid_button()` at line 2178 uses `is_common` to set button color:
- `is_common && enabled` → green tint `Color::srgb(0.25, 0.35, 0.25)`
- `!is_common && enabled` → yellow tint `Color::srgb(0.3, 0.3, 0.2)`

No changes needed to the rendering itself — just the `is_common` value passed to it.

### Dependencies
None — this is a self-contained bug fix in `command_panel.rs`.

## QA Steps
1. [human] Select a GDO Agent and a GDO Headquarters simultaneously (box-select or shift-click).
2. [human] Confirm the Agent is the ActiveGroup (its portrait should be highlighted in the SelectionPanel).
3. [human] Observe the CommandPanel — verify that **all** unit commands (Move, Stop, HoldPosition, Patrol, Attack, AttackMove) appear as **group-specific** commands (not common).
4. [human] Select only a single Agent (no structure). Verify all unit commands appear normally.
5. [human] Repeat steps 1-3 with a Syndicate Agent and Syndicate Tunnel to confirm cross-faction correctness.

## Expected Experience
- In step 3, every command in the panel should be visually styled as a group-specific command (yellow tint, not green), since no command is shared between the unit and the structure.
- In step 4, commands appear normally as they do for single-unit selection.
- In step 5, same visual distinction as step 3 — all commands are group-specific.
