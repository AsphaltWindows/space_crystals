# tunnel_interface

## Metadata
- **From**: qa_router
- **To**: manual_qa

## Content

## Content

## Content

# tunnel-interface

## Metadata
- **From**: designer
- **To**: task_splitter

## Content

Implement the Tunnel's ObjectInterfaceState as defined in `artifacts/designer/design/syndicate_objects.md` under 'Tunnel ObjectInterfaceState'.

The Tunnel has a 4-state interface:

**DefaultState commands:**
- **A: Upgrade Tunnel** — CommandIssuingTransition. Costs Supplies per upgrade cost formula. Unavailable if already Tier 3 or currently performing an operation.
- **B: Expand Tunnel** — StateOnlyTransition to ExpandMenu. Multi-stage: select expansion type, then place within Tunnel Area.
- **C: Eject** — StateOnlyTransition to EjectMenu. Select units from Tunnel Network to eject from this Tunnel.
- **X: Cancel Upgrade** — CommandIssuingTransition. Full refund of Supplies. Only available while upgrade is in progress.

**EjectMenu:**
- Grid of unit type tiles showing all units in the Tunnel Network. Each tile has unit type icon + count.
- Unit types exceeding this Tunnel's tier are visible but greyed out (disabled).
- Click enabled tile: ejects one unit from Side A (CommandIssuingTransition). Units queue with 8-frame minimum spacing.
- **Z**: returns to DefaultState (StateOnlyTransition).

**ExpandMenu:**
- Displays available underground expansion types for this Tunnel's tier.
- Click expansion type: enters AwaitingPlacement for that expansion.
- **Z**: returns to DefaultState (StateOnlyTransition).

**AwaitingPlacement (Expansion):**
- Ghost preview follows cursor within Tunnel Area, snapped to grid. Green=valid, red=invalid.
- R/Shift+R rotate, F/Shift+F flip.
- Left-click valid location: places expansion, begins construction (CommandIssuingTransition, returns to DefaultState).
- **Z**: returns to ExpandMenu (StateOnlyTransition).

## QA Instructions

1. Select a Tier 1 Tunnel.
2. Verify DefaultState shows A (Upgrade), B (Expand), C (Eject), and X is hidden (no upgrade in progress).
3. Press A — verify Supplies are deducted and upgrade begins. Verify X (Cancel Upgrade) now appears.
4. Press X — verify upgrade is cancelled and Supplies are fully refunded.
5. Press B — verify ExpandMenu shows available Tier 1 expansion types.
6. Click an expansion type — verify AwaitingPlacement shows a ghost preview within the Tunnel Area.
7. Move cursor inside/outside Tunnel Area — verify green/red tinting.
8. R/Shift+R/F/Shift+F — verify rotation and flipping work.
9. Left-click a valid location — verify expansion is placed and construction begins.
10. Press Z from AwaitingPlacement — verify return to ExpandMenu. Press Z again — verify return to DefaultState.
11. Press C — verify EjectMenu shows unit types in the Tunnel Network with counts.
12. If Tunnel is Tier 1, verify vehicle/air unit types are greyed out.
13. Click an enabled unit type — verify one unit ejects from Side A.
14. Rapidly eject multiple units — verify 8-frame minimum spacing between ejections.
