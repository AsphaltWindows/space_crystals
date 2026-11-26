# 0326 — Agent panel: A opens a Construct menu (Q Tunnel, W Laser Turret); Repair moves to D

- **Date**: 2026-11-26T17:55:00-05:00
- **Answers**: —
- **Docs**: `factions/syndicate/units/agent.md > Agent ObjectInterfaceState`

## What changed and why
User: "Agents should have a separate construct menu where they select between tunnel and laser turret."
- **A: Construct** opens a menu with **Q** Tunnel, **W** Laser Turret (greyed until available) and **Z** Back.
- The panel's old **D: Build Laser Turret** is gone. **Repair moves to D** (was F in 0325), matching the Recruit's panel.
- The Agent panel is now: Q Move, A Construct, S Drop Off, D Repair, X Stop.

## Implementation impact
- Replaces the direct A and D build buttons from Q0068.
- Placement flows are unchanged.

## Verification
- An Agent's A opens the menu.
- Pressing A then Q places a Tunnel; A then W places a Laser Turret.
- D targets a repair.
