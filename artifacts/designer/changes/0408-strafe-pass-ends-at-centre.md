# 0408 — A glider's pass ends when the target's centre is out of range

- **Date**: 2026-11-26T15:55:00-05:00
- **Answers**: Q0149
- **Docs**: `systems/controls/command_behaviors.md > AttackingObject > GliderMovement exception` (step 3); `factions/cults/units/sparrow.md > Placeholder stats` (Flight rules > Strafing)

## What changed and why
The user chose option (c). A strafing pass used to end when the target's far edge left weapon range (edge-to-edge range), so the loop grew with the target's size and a lone Sparrow lost Agents, vehicles and buildings beyond its 8 GU sight. Now the pass ends when the **target's centre** is out of weapon range, measured from the glider's edge to the target's centre. The loop is the same size whatever the target, and a lone Sparrow's sight 8 covers it.

## Implementation impact
- Only the "when to stop flying on and loop back" check changes. **Firing** still uses normal edge-to-edge range, and so do all other range checks.
- Applies to every glider's strafing (Sparrow today; also attacks on a ground location, which have no size anyway).
- SightRange 8 (0403) unchanged.

## Verification
- A lone Sparrow with no spotter given a direct Attack on a Seeker, a Guard and a building: it keeps making passes on each until the target dies, never losing it.
- The farthest point of its loop is about the same (~500 su from the target's centre) for a Peacekeeper and a building.
