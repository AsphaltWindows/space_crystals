# agent-interface

## Metadata
- **From**: designer
- **To**: task_splitter

## Content

Implement the Agent's ObjectInterfaceState as defined in `artifacts/designer/design/syndicate_objects.md` under 'Agent ObjectInterfaceState'.

The Agent has a DefaultState with two panel commands and a full right-click resolution system:

**DefaultState commands:**
- **A: Build Tunnel** — StateOnlyTransition to AwaitingPlacement. Ghost preview follows cursor, snapped to grid. Green when valid, red when invalid. R/Shift+R rotate, F/Shift+F flip. Left-click valid location dispatches Agent (CommandIssuingTransition, returns to DefaultState). Escape/right-click cancels.
- **B: Drop Off Resources** — CommandIssuingTransition. Requires clicking an own Tunnel. Agent auto-routes to correct side (Side B for crystals, Side C for supplies). Always visible, greyed out when Agent is not carrying resources.

**Right-Click Resolution:**
- Crystal field → Gather crystals
- Supply source → Gather supplies
- Own Tunnel (carrying resources) → Drop off resources (auto-routes to correct side)
- Own Tunnel (not carrying resources) → Enter
- Enemy unit/building → Attack (melee)
- Ground → Move

**Unit Commands (7 total):** Move, Stop, Attack, Enter, Gather, Drop Off Resources, Build Tunnel.

Multiple Agents can be selected simultaneously. Right-click commands are issued to all selected Agents even though Agent is ungroupable (no shared control panel — panel displays one Agent's interface).

## QA Instructions

1. Select an Agent unit.
2. Verify the command panel shows A (Build Tunnel) and B (Drop Off Resources).
3. Press A — verify a ghost Tunnel preview appears on the cursor, snapped to grid.
4. Move cursor over valid/invalid terrain — verify green/red tinting respectively.
5. Press R to rotate the ghost, Shift+R for counter-clockwise. Press F to flip. Confirm all work.
6. Left-click a valid location — verify the Agent begins moving toward it.
7. Press Escape or right-click while in placement mode — verify it cancels back to DefaultState.
8. Give an Agent resources (gather from a crystal field). Verify B (Drop Off) is now clickable (not greyed).
9. Press B, then click an own Tunnel — verify the Agent walks to the correct side (B for crystals, C for supplies).
10. Right-click a crystal field — verify a Gather command is issued.
11. Right-click an own Tunnel while carrying resources — verify Drop Off is issued.
12. Right-click an own Tunnel while NOT carrying resources — verify Enter is issued.
13. Right-click an enemy — verify Attack (melee) is issued.
14. Right-click ground — verify Move is issued.
15. Select multiple Agents. Right-click ground — verify all selected Agents receive the Move command.
