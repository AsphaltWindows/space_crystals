# basic-combat-unit-interface-verify

## Metadata
- **From**: task_planner
- **To**: developer

## Content

## Parent Feature

task_splitter-basic-combat-unit-interface.md

## Task

Verify that the BasicCombatUnitInterfaceState is fully implemented. All components appear to be in place — this is a verification-only task.

**What already exists (verify each):**

1. **Grid layout** (command_panel.rs ~line 142): ObjectInterfaceState::Default match produces Q=Move, W=Reverse(conditional), E=HoldPosition, A=Attack(conditional), S=Patrol, D=AttackGround(conditional), X=Stop.

2. **Right-click resolution** (core.rs right_click_move_command):
   - Enemy → Attack (line ~260, should_attack match)
   - Ground → Move (line ~490, CommandType::Default match)
   - Own Tunnel (Syndicate Guard) → Enter (line ~444, dedicated block)
   - Friendly/Neutral non-Tunnel → falls through entity block to ground Move handler

3. **AwaitingTarget resolutions** (core.rs right_click_move_command):
   - Attack + left-click enemy entity → Attack command (line ~260)
   - Attack + left-click ground → AttackMove (line ~543)
   - Move + left-click ground → Move (line ~490)
   - Move + left-click entity → falls through to ground Move (same effect)
   - Patrol + left-click ground → Patrol (line ~517)
   - AttackGround + left-click ground → AttackLocation (line ~592)
   - Reverse + left-click ground → Reverse (line ~616)

4. **Cancel** (command_panel.rs): AwaitingTarget shows Z=Back (line ~164), Escape handling resets state.

**Verification steps:**
- Confirm all grid slots render correctly for a Peacekeeper (GDO, no Reverse, has Attack, no TargetGround) and Guard (Syndicate, no Reverse, has Attack, no TargetGround).
- Confirm right-click on enemy issues Attack, on ground issues Move, on own Tunnel (Syndicate) issues Enter, on friendly object issues Move.
- Confirm each AwaitingTarget mode resolves as specified.
- Confirm Escape/right-click/Z cancels AwaitingTarget back to Default.
- Run existing tests: cargo test in artifacts/developer/ — all must pass.

## Technical Context

This is a **verification-only task** — no code changes expected. The developer should confirm existing implementations and run tests.

### Files to inspect (all read-only verification):

1. **`artifacts/developer/src/ui/command_panel.rs`**
   - Line 143-153: `get_grid_slot_action()` for `ObjectInterfaceState::Default` — the 3x3 grid layout. Confirmed present with: Move at (0,0), Reverse at (0,1) conditional on `caps.can_reverse`, HoldPosition at (0,2), Attack at (1,0) conditional on `caps.has_attack`, Patrol at (1,1), AttackGround at (1,2) conditional on `caps.can_target_ground`, Stop at (2,1).
   - Line 165-168: `AwaitingTarget(_)` state shows Back at (2,0) — confirmed.
   - Line 886-925: Escape handler in `command_panel_hotkeys` — handles AwaitingTarget cancel. Returns to AgentDefault if active selection is SyndicateAgent, otherwise Default. Confirmed working correctly.

2. **`artifacts/developer/src/game/units/systems/core.rs`** (`right_click_move_command`)
   - Line 260-288: Attack handler (entity click) — `should_attack` matches AwaitingTarget(Attack) or right-click-enemy. Issues `UnitCommand::AttackTarget`. Confirmed.
   - Line 321-342: AwaitingTarget(Enter) entity click — validates own tunnel with tier check, issues `UnitCommand::Enter`. Confirmed.
   - Line 476-508: Guard right-click own tunnel → Enter (skips Agents, checks SyndicateGuard). Confirmed.
   - Line 522-553: Ground click Move/Default handler. Confirmed.
   - Line 556-586: Ground click Patrol handler. Confirmed.
   - Line 588-617: Ground click Attack → AttackMove. Confirmed.
   - Line 649-677: Ground click AttackGround → AttackLocation. Confirmed.
   - Line 679-710: Ground click Reverse handler (with `can_reverse` check). Confirmed.
   - Line 713-717: Ground click Enter → resets interface state (no-op, requires entity target). Confirmed.

3. **Existing test coverage** (core.rs test module):
   - `guard_right_click_own_tunnel_issues_enter_command` (line 2720)
   - `agent_right_click_own_tunnel_not_carrying_issues_enter` (line 2751)
   - `agent_carrying_right_click_tunnel_issues_dropoff_not_enter` (line 2780)
   - `enter_awaiting_target_left_click_valid_tunnel_issues_enter` (line 2818)
   - `enter_awaiting_target_left_click_invalid_target_resets_state` (line 2849)
   - Plus Guard-specific AwaitingTarget Enter test (line 2895)

### Key types involved:
- `ObjectInterfaceState` — enum resource: Default, AwaitingTarget(CommandType), StructureMenu(...), AgentMenu(...)
- `CommandType` — enum: Default, Move, Attack, AttackMove, AttackGround, Patrol, Reverse, Enter, DropOff, Gather, BuildTunnel
- `SelectedUnitCapabilities` — resource with `has_attack`, `can_reverse`, `can_target_ground` flags driving conditional grid slots
- `CommandButtonAction` — enum mapping grid slots to actions (UnitMove, UnitAttack, UnitPatrol, etc.)

### Verification approach:
1. Run `cargo test` from `artifacts/developer/` — all tests must pass.
2. Read through each file/line reference above and confirm the implementation matches the spec.
3. No code changes needed unless a discrepancy is found (unexpected given all evidence confirms implementation is complete).

## Dependencies

None — this is a standalone verification task. All referenced systems are already implemented.
