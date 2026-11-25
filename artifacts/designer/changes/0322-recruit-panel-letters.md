# 0322 — The Recruit's panel letters; Assist and Repair pointers

- **Date**: 2026-11-26T16:55:00-05:00
- **Answers**: Q0095, Q0108 (point 2)
- **Docs**:
  - `factions/cults/units/recruit.md > Recruit ObjectInterfaceState`;
  - `systems/controls/selection_and_hud.md > PointerDisplayType` (Types; DefaultState and AwaitingTarget resolution).

## What changed and why
User confirmed the proposal.
- **Recruit panel, the same layout as the Agent's:**
  - **Q** Move
  - **A** Construct
  - **S** Assist Construction (was the off-grid "B")
  - **D** Repair (new; entry 0321)
  - **X** Stop
- **Two new pointer types (placeholder looks):**
  - **Assist**: over an own building under construction, when the selection can assist.
  - **Repair**: over a damaged own building or vehicle the selection can repair.
  - Both also apply in the matching AwaitingTarget states.
- **Right-click:** a damaged, completed own building or vehicle now resolves to **Repair**.

## Implementation impact
- Panel and pointer additions.
- **Open point:** right-clicking a damaged building that can also be entered (Armory, Tunnel, …) is unresolved. The docs flag it, and the user is being asked. Until then, keep **Enter** for enterable buildings, and use the D or S panel command to repair them.

## Verification
- Select Recruits: the panel shows Q, A, S, D and X.
- Hover over a site under construction: the Assist pointer shows.
- Hover over a damaged Storage: the Repair pointer shows.
