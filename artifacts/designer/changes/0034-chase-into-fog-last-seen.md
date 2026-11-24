# 0034 — Attack target lost into fog: go to last seen position, then complete

- **Date**: 2026-11-23T19:14:28-05:00
- **Answers**: Q0010 (part 1; patrol origin and failed patrol legs still open)
- **Docs**: `systems/controls/behaviors.md > AttackingObject > Algorithm` (step 6)

## What changed and why
Option A. When an attack target leaves its owner's vision, the attacker moves to the target's **last seen position** and the order completes there. Its real position is never tracked through fog. Afterwards the unit is idle, and normal idle auto-targeting with the leash applies if the enemy reappears.

## Implementation impact
Matches the Director's default. This also covers a Follow (0032) of an enemy unit via the Move command if the target goes into fog: treat it the same way (go to last seen position, complete). That's an extension for consistency; tell me if you see a problem.

## Verification
- Attack a fast enemy that runs out of sight: your unit runs to where it vanished and stops. If the enemy is visible from there and within auto-target rules, the unit engages again on its own.
