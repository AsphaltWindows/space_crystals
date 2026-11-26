# 0397 — Attack is always on A, and an Attack goes to every selected unit

- **Date**: 2026-11-27T13:00:00-05:00
- **Answers**: —
- **Docs**: `systems/controls/interface.md > CommandPanel` (Common means the same command; Attack goes to everyone), `> CommandPanel > Standard Slot Assignments` (A: Attack, on every unit)

## What changed and why
User:
1. "Attack should probably be in the same spot with the same key, so that it's always the same regardless of who's in focus." **A is Attack on every unit panel.** No unit puts another command on A. The worker panels that use A today are being rearranged with the user; letters follow in the next entry:
   - Recruit: A Basic Construct, E Attack (from 0389);
   - Agent: A Build Tunnel;
   - Drone: A Deploy;
   - Field Engineer: A Construct.
2. "It should go to everyone then, I guess. b." An Attack is issued to **every selected unit**. Units that can attack carry it out (Attack, or Attack Move on ground). Units that can't **Move** to the spot (ground) or **follow** the target (object).

Also clarified: "common command" means the same command, wherever its button sits.

## Implementation impact
- Attack becomes a selection-wide command. Non-attackers get a Move or a follow (MovingToObject) in its place.
- Worker panel letters: wait for the next entry.

## Verification
- Select Militants and a Prayer-mobile, press A and click ground: the Militants Attack Move, and the Prayer-mobile moves with them.
