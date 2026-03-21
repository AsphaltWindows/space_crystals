# basic-combat-unit-interface-verify

## Metadata
- **From**: task_splitter
- **To**: task_planner

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
- Run existing tests: `cargo test` in artifacts/developer/ — all must pass.
