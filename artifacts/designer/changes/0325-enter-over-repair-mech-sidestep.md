# 0325 — Right-click: Enter beats Repair; Agent F: Repair; Mechs turn and step sideways

- **Date**: 2026-11-26T17:45:00-05:00
- **Answers**: —
- **Docs**:
  - `factions/cults/units/recruit.md > Right-Click Resolution`;
  - `factions/syndicate/units/agent.md > Agent ObjectInterfaceState` (Panel letters, DefaultState, Right-Click Resolution), `> Repair`;
  - `systems/controls/selection_and_hud.md > PointerDisplayType > DefaultState`;
  - `systems/units.md > Ground Collision`.

## What changed and why
- **Enter beats Repair** (user): "Default should be enter if entering is possible (which is most buildings for cults)."
  - A right-click on a damaged own building that the selection can enter means **Enter**, with the Enter pointer.
  - Repair those buildings with the panel's Repair command.
  - Buildings that can't be entered, and vehicles, resolve to Repair.
  - This settles the open point in 0322.
- **Agent panel:** **F: Repair** (designer placeholder letter, since D is Build Laser Turret).
  - It targets a damaged own Tunnel or Laser Turret.
  - Right-click on a damaged Laser Turret repairs it.
  - Right-click on a Tunnel stays Enter, or Drop Off when carrying.
- **Mechs** turn to face the side, then step sideways like infantry (user: "Mechs can turn and sidestep"). No Mechs are planned yet.

## Implementation impact
Right-click resolution and the pointer order as above; one more Agent panel button.

## Verification
- Recruits right-clicking a damaged Armory walk in.
- Pressing D, then clicking the same Armory, makes them repair it.
- Agents right-clicking a damaged Laser Turret repair it.
