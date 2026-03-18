# Design Update: Agent Control Panel & Commands

**Date**: 2026-03-06
**Files modified**: `design/syndicate_objects.md`

## Changes

### Agent ObjectInterfaceState (new section)

Added full control panel spec for the Agent unit:

- **DefaultState commands**:
  - A: Build Tunnel — enters AwaitingPlacement with ghost preview, standard placement controls
  - B: Drop Off Resources — targeted command, click own Tunnel. Always visible, greyed out when not carrying resources. Agent auto-routes to correct Tunnel side (B for crystals, C for supplies).

- **Unit Commands**: Move, Stop, Attack, Enter, Gather, Drop Off Resources, Build Tunnel

- **Right-Click Resolution**:
  - Crystal field → Gather
  - Supply source → Gather
  - Own Tunnel (carrying) → Drop off resources
  - Own Tunnel (not carrying) → Enter
  - Enemy → Attack (melee)
  - Ground → Move

### Multi-select behavior

Agent remains ungroupable (no shared control panel), but right-click commands are issued to all selected Agents simultaneously.

### Construction clarification

Explicitly documented that only one Agent may construct a given Tunnel — multiple Agents cannot speed up construction.
