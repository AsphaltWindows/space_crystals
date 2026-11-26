# 0398 — Worker panels: building moves to W, Attack on A; unarmed units hide A and treat Attack as Move

- **Date**: 2026-11-27T13:15:00-05:00
- **Answers**: —
- **Docs**:
  - `factions/cults/units/recruit.md > ObjectInterfaceState`;
  - `factions/syndicate/units/agent.md` (Panel letters, DefaultState);
  - `factions/gdo/units/field_engineer.md` (Panel);
  - `factions/colonists/units/drone.md` (DefaultState);
  - `systems/controls/interface.md > CommandPanel` (Attack goes to everyone; Standard Slot Assignments).

## What changed and why
This follows 0397: A is Attack on every unit. The user accepted the proposed worker layouts and added two rules.

| Unit | New panel |
|---|---|
| Recruit | Q Move, **W Basic Construct**, **E Advanced Construct**, **A Attack**, S Assist, D Repair, X Stop |
| Agent | Q Move, **W Build Tunnel**, **A Attack**, S Drop Off, D Repair, X Stop |
| Field Engineer | Q Move, **W Construct**, D Repair, X Stop |
| Drone | Q Move, **W Deploy**, S Set Drop-off, X Stop |

The Recruit's E Attack from 0389 moves to A.

- **Unarmed units don't show A** (user). Pressing A while an unarmed group is in focus still issues the Attack to the whole selection, if anyone in it can attack. This is the designer's reading of "always the same regardless of who's in focus".
- **Non-attackers treat Attack and Attack Move as Move** (user): on ground they move to the spot, on a target they move to it. This replaces 0397's "follow".

## Implementation impact
- Panel letters change for the four workers.
- Non-attackers turn an Attack into a Move.

## Verification
- Recruits: W opens Basic Construct, E opens Advanced Construct, A attacks.
- Drones and Militants selected, with the Drones in focus: the panel has no A. Pressing A and clicking ground makes the Militants Attack Move and the Drones move there.
