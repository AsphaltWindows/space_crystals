# enter-right-click-integration

## Metadata
- **From**: task_splitter
- **To**: task_planner

## Content

## Parent Feature

task_splitter-enter-command-tunnel.md

## Task

Add Enter command right-click resolution for BasicCombatUnit and tier validation, plus AwaitingTarget[Enter] entity click handling.

### What exists already
- Agent right-click own Tunnel in `right_click_move_command` (`src/game/units/systems/core.rs` lines ~390-412): issues Enter if not carrying, DropOff if carrying — but does NOT validate tunnel tier
- AwaitingTarget[Enter] ground click handler (core.rs line ~582): resets to Default (correct — Enter needs entity target)
- `can_enter_tunnel()` utility in `src/game/units/utils.rs`
- Entity-click section in `right_click_move_command` (core.rs line ~256) handles Attack, Chopper targets, Agent targets — but NOT BasicCombatUnit tunnel targets

### What needs to be implemented

1. **BasicCombatUnit right-click own Tunnel → Enter** (in the entity-click section of `right_click_move_command`, core.rs):
   - After the Agent-specific block (~line 414) and before the fall-through to ground Move
   - When: `is_right_click && command_type == CommandType::Default`, target is an own Tunnel
   - For each selected Syndicate unit (not Agent — Agent is already handled above): validate `can_enter_tunnel()` with the unit's base type and tunnel tier
   - If valid: clear movement state, insert `UnitCommand::Enter(target_entity)`
   - The query needs access to `TunnelState` (for tier) and `Owner` on the target, and `UnitBaseEnum` + faction info on selected units
   - Note: the selected_units query tuple may need extending to include `UnitBaseEnum` if not already present

2. **Add tier validation to Agent right-click Enter** (core.rs lines ~390-412):
   - Currently the Agent right-click own Tunnel block issues `UnitCommand::Enter` without checking tier
   - Add `can_enter_tunnel()` check — if tier insufficient, fall through (don't issue Enter, treat as regular right-click → Move)
   - Agent is HeavyInfantry, so Tier 1+ always allows them — but the validation should still be called for correctness and future-proofing

3. **AwaitingTarget[Enter] left-click entity resolution** (in the entity-click section of `right_click_move_command`, core.rs):
   - When `command_type == CommandType::Enter` and `is_left_click` and target is an own Tunnel with sufficient tier
   - Issue `UnitCommand::Enter(target_entity)` to selected Syndicate units
   - Reset interface state to Default
   - If target is not a valid tunnel, reset interface state without issuing command

4. **Tests**:
   - Test BasicCombatUnit (Guard) right-click own tunnel issues Enter
   - Test tier validation rejects vehicle Enter on Tier 1 tunnel
   - Test Agent carrying resources right-clicks tunnel → DropOff (not Enter)
   - Test AwaitingTarget[Enter] left-click valid tunnel issues Enter command
