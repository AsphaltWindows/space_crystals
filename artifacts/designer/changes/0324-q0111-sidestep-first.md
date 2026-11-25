# 0324 — Q0111: measure the new sidestep with the 1-square hatch first; then a 2-square hatch; then the backstop

- **Date**: 2026-11-26T17:30:00-05:00
- **Answers**: Q0111
- **Docs**: `factions/syndicate/units/agent.md > Gathering` (Hatch jams: the plan); `systems/units.md > Ground Collision` (0323)

## What changed and why
User: "Yes 1 agent delivering at a time, but let's try with 1 grid hatch first with the new sidestep."

None of Q0111's options yet. Steps, in order:
1. **Now:** keep the 1-square hatch and 0315's rule. Implement 0323's sidestep (90°, one body width). Measure it on Q0111's 30-layout grid and the `dev_skirmish` soak, with 4–6 Agents.
2. **If Agents still lock up:** a **2-square crystal hatch** (both middle squares of Side B), still with **one Agent delivering at a time**. The Supplies hatch is undecided; ask if it matters.
3. **Then, if still needed:** option (a), stuck gathering Agents make way like idle units. The user suggests "loaded has right of way" between two stuck Agents. That is not yet confirmed.

Bring the measurements back, as a new question, if step 1 isn't enough.

## Implementation impact
- Step 1 only: 0323's sidestep.
- No hatch change yet.
- 0315 stays.

## Verification
- The Q0111 grid: count the layouts where an Agent goes 60 s or more without delivering, against 20/30 now.
- The `dev_skirmish` soak, with 4 and 6 Agents: no Tunnel stops for good.
